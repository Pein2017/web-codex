#!/usr/bin/env python3
"""Point-in-time, non-model setup/read probe for the named native Runner route.

This helper never launches a worker. Invoke through the same Runner run_process
context selected for the worker; a Desktop invocation is diagnostic only.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import secrets
import selectors
import signal
import subprocess
import sys
import time


SUPPORTED_EXECUTABLE = "/root/.local/bin/codex"
SUPPORTED_VERSION = "codex-cli 0.159.2"
DEADLINE_SECONDS = 10.0
OUTPUT_BYTES = 4096
PROBE = """import json, os, sys
with open(sys.argv[1], 'rb') as source:
    source.read(1)
print(json.dumps({'nonce': sys.argv[2], 'uid': os.getuid(),
                  'cwd': os.getcwd(), 'readable': True,
                  'mount_namespace': os.readlink('/proc/self/ns/mnt')}))
"""


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True).encode()).hexdigest()


def context():
    """Hash the full inherited environment; never disclose its values."""
    namespaces = {name: os.readlink('/proc/self/ns/' + name)
                  for name in ('mnt', 'user', 'pid', 'net')}
    return {'uid': os.getuid(), 'gid': os.getgid(), 'namespaces': namespaces,
            'environment_sha256': digest(dict(os.environ))}


def run_bounded(argv, cwd, deadline):
    """Drain both pipes with capped memory and one shared absolute deadline."""
    if time.monotonic() >= deadline:
        raise TimeoutError
    child = subprocess.Popen(argv, cwd=cwd, stdin=subprocess.DEVNULL,
                             stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                             start_new_session=True)
    streams = {'stdout': bytearray(), 'stderr': bytearray()}
    counts = {'stdout': 0, 'stderr': 0}
    try:
        with selectors.DefaultSelector() as selector:
            selector.register(child.stdout, selectors.EVENT_READ, 'stdout')
            selector.register(child.stderr, selectors.EVENT_READ, 'stderr')
            while selector.get_map():
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    raise TimeoutError
                for key, _ in selector.select(remaining):
                    chunk = os.read(key.fileobj.fileno(), OUTPUT_BYTES)
                    if not chunk:
                        selector.unregister(key.fileobj)
                        continue
                    name = key.data
                    counts[name] += len(chunk)
                    streams[name].extend(chunk[:max(0, OUTPUT_BYTES - len(streams[name]))])
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise TimeoutError
            code = child.wait(timeout=remaining)
    except (TimeoutError, subprocess.TimeoutExpired):
        # Only this new process group belongs to the probe. No host discovery,
        # fallback invocation or broad process stop is permitted here.
        try:
            os.killpg(child.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        child.wait(timeout=1)
        raise TimeoutError from None
    finally:
        child.stdout.close()
        child.stderr.close()
    return {'exit_code': code, 'stdout': bytes(streams['stdout']).decode('utf-8', 'replace'),
            'stderr': bytes(streams['stderr']).decode('utf-8', 'replace'), 'bytes': counts}


def preflight(executable, cwd, read_path, permission_profile, config_profile=None,
              expected_context=None):
    started = time.monotonic()
    deadline = started + DEADLINE_SECONDS
    receipt = {'status': 'unproven', 'reason': 'unsupported_route',
               'probe_scope': 'native setup and selected file readability only',
               'worker_started': False, 'task_acceptance': 'not_evaluated'}

    def finish(status, reason):
        receipt.update(status=status, reason=reason,
                       elapsed_seconds=round(time.monotonic() - started, 3))
        return receipt

    if (sys.platform != 'linux' or executable != SUPPORTED_EXECUTABLE
            or permission_profile != ':read-only'
            or not Path(cwd).is_absolute() or not Path(read_path).is_absolute()):
        return finish('unproven', 'unsupported_route')
    # Profile selection is native; reject option-like or path-like profile names.
    if config_profile is not None and (not config_profile or config_profile.startswith('-')
                                       or '/' in config_profile or '\\' in config_profile):
        return finish('unproven', 'unsupported_profile')
    try:
        observation = context()
        receipt['context'] = observation
        if expected_context is not None and expected_context != observation:
            return finish('unproven', 'runner_context_changed')
        # Physical cwd must match the caller's Runner cwd; no silent cd/retarget.
        if not os.path.samefile(cwd, os.getcwd()):
            return finish('unproven', 'runner_cwd_mismatch')
        executable_path = Path(executable).resolve(strict=True)
        identity = executable_path.stat()
        selected = {'executable': executable, 'resolved_executable': str(executable_path),
                    'executable_identity': [identity.st_dev, identity.st_ino, identity.st_size,
                                            identity.st_mtime_ns],
                    'cwd': str(Path(cwd).resolve()), 'read_path': read_path,
                    'permission_profile': permission_profile, 'config_profile': config_profile,
                    'include_managed_config': True}
        receipt['selected'] = selected
        receipt['selected_sha256'] = digest(selected)
        try:
            with open(read_path, 'rb') as source:
                source.read(1)
            receipt['host_readable'] = True
        except OSError:
            receipt['host_readable'] = False
        version = run_bounded([executable, '--version'], cwd, deadline)
        if version['exit_code'] != 0 or version['stdout'].strip() != SUPPORTED_VERSION:
            return finish('unproven', 'unsupported_cli_version')
        grammar = run_bounded([executable, 'sandbox', '--help'], cwd, deadline)
        required = ('[COMMAND]...', '--permission-profile', '--include-managed-config',
                    '--profile', '--cd')
        if grammar['exit_code'] != 0 or not all(word in grammar['stdout'] for word in required):
            return finish('unproven', 'unsupported_native_grammar')
        command = [executable, 'sandbox', '--permission-profile', permission_profile,
                   '--include-managed-config', '--cd', cwd]
        if config_profile is not None:
            command.extend(['--profile', config_profile])
        nonce = secrets.token_hex(16)
        command.extend(['--', sys.executable, '-I', '-c', PROBE, read_path, nonce])
        result = run_bounded(command, cwd, deadline)
        receipt['native_exit_code'] = result['exit_code']
        receipt['output_bytes'] = result['bytes']
        receipt['output_capped'] = any(count > OUTPUT_BYTES for count in result['bytes'].values())
        # Only fixed diagnostic categories leave this helper. Arbitrary native
        # output may contain credentials/config values and is never echoed.
        if result['exit_code'] != 0:
            if 'Failed to make / slave' in result['stderr'] and 'Permission denied' in result['stderr']:
                reason = 'native_mount_permission_denied'
            elif 'No such file or directory' in result['stderr']:
                reason = 'native_path_unavailable'
            elif 'Permission denied' in result['stderr']:
                reason = 'native_access_denied'
            else:
                reason = 'native_setup_or_read_failed'
            return finish('blocked', reason)
        try:
            evidence = json.loads(result['stdout'])
        except (ValueError, TypeError):
            return finish('unproven', 'native_read_evidence_missing')
        if not isinstance(evidence, dict):
            return finish('unproven', 'native_read_evidence_missing')
        if (evidence.get('nonce') != nonce or evidence.get('uid') != observation['uid']
                or evidence.get('cwd') != selected['cwd'] or evidence.get('readable') is not True):
            return finish('unproven', 'native_read_evidence_mismatch')
        if (not evidence.get('mount_namespace')
                or evidence['mount_namespace'] == observation['namespaces']['mnt']):
            return finish('unproven', 'native_namespace_unproven')
        receipt['native_mount_namespace'] = evidence['mount_namespace']
        current_identity = executable_path.stat()
        if (context() != observation
                or Path(executable).resolve(strict=True) != executable_path
                or [current_identity.st_dev, current_identity.st_ino, current_identity.st_size,
                    current_identity.st_mtime_ns] != selected['executable_identity']):
            return finish('unproven', 'selected_context_changed')
        return finish('probe_passed', 'selected_native_setup_read_passed')
    except TimeoutError:
        return finish('unproven', 'deadline_exceeded')
    except (OSError, ValueError):
        return finish('unproven', 'selected_context_unavailable')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--executable', required=True)
    parser.add_argument('--cwd', required=True)
    parser.add_argument('--read-path', required=True)
    parser.add_argument('--permission-profile', required=True)
    parser.add_argument('--profile')
    parser.add_argument('--expected-context-json', type=json.loads)
    args = parser.parse_args()
    result = preflight(args.executable, args.cwd, args.read_path, args.permission_profile,
                       args.profile, args.expected_context_json)
    print(json.dumps(result, sort_keys=True))
    return {'probe_passed': 0, 'blocked': 2, 'unproven': 3}[result['status']]


if __name__ == '__main__':
    sys.exit(main())

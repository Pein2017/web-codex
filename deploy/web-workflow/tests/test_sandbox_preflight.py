"""Non-model preflight controls; no production worker or service is started."""

import importlib.util
import json
import os
from pathlib import Path
import sys
import subprocess
import tempfile
import unittest
from unittest.mock import patch


HELPER = Path(__file__).resolve().parents[1] / "sandbox_preflight.py"


class SandboxPreflightTests(unittest.TestCase):
    def setUp(self):
        spec = importlib.util.spec_from_file_location("sandbox_preflight", HELPER)
        self.module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.module)
        self.temp = tempfile.TemporaryDirectory(prefix="webcodex-preflight-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.read_path = self.root / "read target"
        self.read_path.write_text("fixture")
        self.cli = self.root / "codex"
        self.calls = self.root / "calls.jsonl"
        self.worker_counter = self.root / "workers"
        self.cli.write_text("""#!/usr/bin/env python3
import json, os, pathlib, subprocess, sys, time
args = sys.argv[1:]
with open(os.environ['TEST_CALLS'], 'a') as out:
    out.write(json.dumps(args) + '\\n')
if 'exec' in args or 'resume' in args:
    pathlib.Path(os.environ['TEST_WORKERS']).touch()
    sys.exit(9)
mode = os.environ.get('TEST_MODE', 'blocked')
if args == ['--version']:
    print('codex-cli 0.159.2' if mode != 'version' else 'codex-cli 0.100.0')
elif args == ['sandbox', '--help']:
    print('Usage: codex sandbox [OPTIONS] [COMMAND]... --permission-profile --include-managed-config --profile --cd' if mode != 'grammar' else 'Usage: codex sandbox linux')
elif mode == 'timeout':
    time.sleep(30)
elif mode == 'no-marker':
    sys.exit(0)
elif mode == 'malformed-marker':
    print('[]')
elif mode == 'host-only':
    sys.exit(subprocess.run(args[args.index('--') + 1:]).returncode)
elif mode == 'controlled-native':
    evidence = json.loads(subprocess.check_output(args[args.index('--') + 1:]))
    evidence['mount_namespace'] = 'mnt:[controlled-fixture]'
    print(json.dumps(evidence))
elif mode == 'overflow':
    sys.stderr.write('secret-fixture' * 10000)
    sys.exit(1)
else:
    print('bwrap: Failed to make / slave: Permission denied sk-secret-fixture password=secret-fixture', file=sys.stderr)
    sys.exit(1)
""")
        self.cli.chmod(0o755)
        self.env = dict(os.environ, TEST_CALLS=str(self.calls),
                        TEST_WORKERS=str(self.worker_counter))

    def probe(self, mode="blocked", **kwargs):
        with patch.dict(os.environ, self.env | {"TEST_MODE": mode}), \
                patch.object(self.module, "SUPPORTED_EXECUTABLE", str(self.cli)):
            previous = os.getcwd()
            try:
                os.chdir(self.root)
                return self.module.preflight(str(self.cli), str(self.root),
                                             str(self.read_path), ":read-only", **kwargs)
            finally:
                os.chdir(previous)

    def recorded(self):
        return [json.loads(row) for row in self.calls.read_text().splitlines()]

    def test_blocked_before_worker_and_no_fallback(self):
        result = self.probe()
        self.assertEqual(result["status"], "blocked")
        self.assertEqual(result["reason"], "native_mount_permission_denied")
        self.assertFalse(self.worker_counter.exists())
        command = self.recorded()[-1]
        self.assertEqual(command[:6], ["sandbox", "--permission-profile", ":read-only",
                                      "--include-managed-config", "--cd", str(self.root)])
        self.assertNotIn("linux", command)
        self.assertNotIn("dangerously", json.dumps(command))
        self.assertEqual(len(self.recorded()), 3)

    def test_diagnostics_do_not_echo_child_secrets(self):
        result = self.probe()
        self.assertNotIn("secret-fixture", json.dumps(result))
        self.assertLess(len(json.dumps(result)), 8192)

    def test_unknown_version_and_grammar_never_probe(self):
        for mode, count in (("version", 1), ("grammar", 2)):
            self.calls.unlink(missing_ok=True)
            self.assertEqual(self.probe(mode)["status"], "unproven")
            self.assertEqual(len(self.recorded()), count)
        self.assertFalse(self.worker_counter.exists())

    def test_absolute_deadline_kills_only_probe_process_group(self):
        with patch.object(self.module, "DEADLINE_SECONDS", 0.3):
            result = self.probe("timeout")
        self.assertEqual(result["status"], "unproven")
        self.assertEqual(result["reason"], "deadline_exceeded")
        self.assertLess(result["elapsed_seconds"], 1.5)
        self.assertFalse(self.worker_counter.exists())

    def test_exit_zero_without_probe_marker_is_not_success(self):
        for mode in ('no-marker', 'malformed-marker'):
            self.assertEqual(self.probe(mode)["status"], "unproven")

    def test_host_read_or_identical_namespace_is_not_sandbox_proof(self):
        result = self.probe("host-only")
        self.assertTrue(result["host_readable"])
        self.assertEqual(result["status"], "unproven")
        self.assertEqual(result["reason"], "native_namespace_unproven")

    def test_wrong_context_rejects_before_cli(self):
        result = self.probe(expected_context={"uid": -1})
        self.assertEqual(result["status"], "unproven")
        self.assertEqual(result["reason"], "runner_context_changed")
        self.assertFalse(self.calls.exists())

    def test_unknown_policy_or_executable_is_unproven(self):
        with patch.dict(os.environ, self.env), \
                patch.object(self.module, "SUPPORTED_EXECUTABLE", str(self.cli)):
            for executable, policy in ((str(self.cli), "other"), (sys.executable, ":read-only")):
                result = self.module.preflight(executable, str(self.root), str(self.read_path), policy)
                self.assertEqual(result["status"], "unproven")
        self.assertFalse(self.calls.exists())

    def test_controlled_native_positive_remains_narrow_and_no_worker(self):
        result = self.probe("controlled-native", config_profile="selected")
        self.assertEqual(result["status"], "probe_passed")
        self.assertEqual(result["task_acceptance"], "not_evaluated")
        self.assertFalse(result["worker_started"])
        self.assertIn('--profile', self.recorded()[-1])
        self.assertFalse(self.worker_counter.exists())

    def test_oversized_native_output_is_drained_without_disclosure(self):
        result = self.probe("overflow")
        self.assertEqual(result["status"], "blocked")
        self.assertTrue(result["output_capped"])
        self.assertGreater(result["output_bytes"]["stderr"], self.module.OUTPUT_BYTES)
        self.assertNotIn('secret-fixture', json.dumps(result))

    def test_selected_cwd_cannot_silently_retarget_runner(self):
        with patch.object(self.module, 'SUPPORTED_EXECUTABLE', str(self.cli)):
            result = self.module.preflight(str(self.cli), str(self.root),
                                         str(self.read_path), ':read-only')
        self.assertEqual(result['reason'], 'runner_cwd_mismatch')
        self.assertFalse(self.calls.exists())

    def test_absolute_deadline_is_shared_across_native_stages(self):
        original = self.module.run_bounded
        deadlines = []

        def staged(argv, cwd, deadline):
            import time
            deadlines.append(deadline)
            time.sleep(0.08)
            return original(argv, cwd, deadline)

        with patch.object(self.module, 'DEADLINE_SECONDS', 0.2), \
                patch.object(self.module, 'run_bounded', staged):
            result = self.probe()
        self.assertEqual(result['reason'], 'deadline_exceeded')
        self.assertEqual(len(set(deadlines)), 1)
        self.assertFalse(self.worker_counter.exists())

    def test_wrong_nonce_or_uid_cannot_certify_selected_probe(self):
        original = self.module.run_bounded

        def swapped(argv, cwd, deadline):
            result = original(argv, cwd, deadline)
            if '--' in argv:
                evidence = json.loads(result['stdout'])
                evidence['nonce'] = 'different-probe'
                result['stdout'] = json.dumps(evidence)
            return result

        with patch.object(self.module, 'run_bounded', swapped):
            result = self.probe('controlled-native')
        self.assertEqual(result['reason'], 'native_read_evidence_mismatch')


def native_fixture():
    """Explicit real Runner diagnostic: blocked is not a positive qualification."""
    with tempfile.TemporaryDirectory(prefix='webcodex-native-read-') as directory:
        target = Path(directory) / 'selected-input'
        target.write_text('disposable readable fixture')
        result = subprocess.run([sys.executable, str(HELPER), '--executable',
                                 '/root/.local/bin/codex', '--cwd', directory,
                                 '--read-path', str(target), '--permission-profile', ':read-only'],
                                cwd=directory, stdin=subprocess.DEVNULL,
                                text=True, capture_output=True, timeout=12)
        receipt = json.loads(result.stdout)
        print(json.dumps({'helper_exit': result.returncode, 'receipt': receipt}, sort_keys=True))
        assert receipt['host_readable'] is True
        assert receipt['worker_started'] is False
        assert receipt['task_acceptance'] == 'not_evaluated'
        if receipt['status'] == 'blocked':
            assert result.returncode == 2
            assert receipt['reason'] == 'native_mount_permission_denied'
            assert receipt['native_exit_code'] != 0
        else:
            assert receipt['status'] == 'probe_passed'
            assert result.returncode == 0
        return 0


if __name__ == "__main__":
    if sys.argv[1:] == ['--native-fixture']:
        sys.exit(native_fixture())
    unittest.main()

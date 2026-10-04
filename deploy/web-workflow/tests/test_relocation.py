"""Focused operator-script checks; no live tmux session or service is used."""

import json
import os
from pathlib import Path
import shlex
import subprocess
import tempfile
import unittest


SCRIPTS = Path(__file__).resolve().parents[1]
DEFAULT_ROOT = "/data/CoordExp/codex-tools/web-codex/deployment"


class RelocationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="webcodex-relocation-")
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        # Spaces exercise shell quoting at the tmux shell-command boundary.
        self.root = self.base / "deployment with spaces"
        self.fake_bin = self.base / "fake-bin"
        self.fake_bin.mkdir()
        self.calls = self.base / "tmux.jsonl"
        self.executions = self.base / "services.jsonl"
        self.live = self.base / "live-session"
        self.env = dict(os.environ, WEBCODEX_DEPLOY_ROOT=str(self.root),
                        XDG_STATE_HOME="/stale/caller/state",
                        FAKE_TMUX_CALLS=str(self.calls),
                        FAKE_SERVICE_CALLS=str(self.executions),
                        FAKE_TMUX_LIVE=str(self.live))
        self.env["PATH"] = str(self.fake_bin) + os.pathsep + self.env["PATH"]
        self.executable(self.fake_bin / "tmux", """#!/usr/bin/env python3
import json, os, pathlib, subprocess, sys
args = sys.argv[1:]
with open(os.environ['FAKE_TMUX_CALLS'], 'a') as out:
    out.write(json.dumps(args) + '\\n')
action = args[2]
live = pathlib.Path(os.environ['FAKE_TMUX_LIVE'])
if action == 'has-session':
    sys.exit(0 if live.exists() else 1)
if action in ('new-session', 'new-window'):
    cached = dict(os.environ, WEBCODEX_DEPLOY_ROOT='/stale/tmux/deployment',
                  XDG_STATE_HOME='/stale/tmux/state')
    subprocess.run(['bash', '-c', args[-1]], env=cached, check=True)
if action == 'kill-session':
    live.unlink()
if action == 'list-panes':
    print('webcodex:server pid=fixture dead=0')
""")
        self.release = self.root / "releases" / "fixture"
        self.release.mkdir(parents=True)
        (self.root / "current").symlink_to("releases/fixture")
        for name in ("bin/webcodex-server", "bin/webcodex-runner", "bin/webcodex-cli"):
            self.executable(self.release / name, "#!/usr/bin/env bash\nexit 0\n")
        for name in ("runtime/bin/webcodex-cli", "runtime/bin/tunnel-client",
                     "runtime/bin/node", "runtime/git/bin/git"):
            self.executable(self.root / name, "#!/usr/bin/env bash\nexit 0\n")
        for name in ("AGENTS.md", "plugins/web-workflow/plugin.mjs",
                     "plugins/web-workflow/pytest_report.py"):
            self.touch(self.release / name)
        for name in ("server.env", "runner.toml", "tunnel.env"):
            self.touch(self.root / "config" / name)
        for path in (self.release / "service.sh", self.root / "bin" / "service.sh"):
            self.executable(path, """#!/usr/bin/env python3
import json, os, sys
with open(os.environ['FAKE_SERVICE_CALLS'], 'a') as out:
    out.write(json.dumps({'script': sys.argv[0], 'service': sys.argv[1],
                         'root': os.environ.get('WEBCODEX_DEPLOY_ROOT'),
                         'state': os.environ.get('XDG_STATE_HOME')}) + '\\n')
""")

    def executable(self, path, contents):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(contents)
        path.chmod(0o755)

    def touch(self, path):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.touch()

    def run_control(self, *args, env=None):
        return subprocess.run(["bash", str(SCRIPTS / "control.sh"), *args],
                              env=self.env if env is None else env,
                              text=True, capture_output=True, timeout=10)

    def recorded(self, path):
        return [json.loads(line) for line in path.read_text().splitlines()]

    def test_default_root_status(self):
        env = self.env.copy()
        env.pop("WEBCODEX_DEPLOY_ROOT")
        result = self.run_control("status", env=env)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.recorded(self.calls)[0][:2],
                         ["-S", DEFAULT_ROOT + "/state/tmux.sock"])

    def test_three_launchers_override_cached_tmux_environment(self):
        before = (self.release / "service.sh").read_bytes()
        result = self.run_control("start")
        self.assertEqual(result.returncode, 0, result.stderr)
        launches = [args for args in self.recorded(self.calls)
                    if args[2] in ("new-session", "new-window")]
        self.assertEqual(len(launches), 3)
        for args, service in zip(launches, ("server", "runner", "tunnel")):
            self.assertEqual(args[:2], ["-S", str(self.root / "state/tmux.sock")])
            command = shlex.split(args[-1])
            self.assertEqual(command, ["env", "WEBCODEX_DEPLOY_ROOT=" + str(self.root),
                                      "XDG_STATE_HOME=" + str(self.root / "state/xdg/state"),
                                      str(self.root / "current/service.sh"), service])
        self.assertEqual([row["service"] for row in self.recorded(self.executions)],
                         ["server", "runner", "tunnel"])
        for row in self.recorded(self.executions):
            self.assertEqual(row["root"], str(self.root))
            self.assertEqual(row["state"], str(self.root / "state/xdg/state"))
        self.assertEqual((self.release / "service.sh").read_bytes(), before)

    def test_legacy_launcher_fallback(self):
        (self.release / "service.sh").unlink()
        result = self.run_control("start")
        self.assertEqual(result.returncode, 0, result.stderr)
        for row in self.recorded(self.executions):
            self.assertEqual(row["script"], str(self.root / "bin/service.sh"))
            self.assertEqual(row["root"], str(self.root))
            self.assertEqual(row["state"], str(self.root / "state/xdg/state"))

    def test_existing_session_start_status_stop(self):
        self.live.touch()
        result = self.run_control("start")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertFalse(self.executions.exists())
        result = self.run_control("status")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("releases/fixture", result.stdout)
        self.assertIn("pid=fixture", result.stdout)
        result = self.run_control("stop")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertFalse(self.live.exists())
        result = self.run_control("status")
        self.assertIn("services are stopped", result.stdout)

    def test_switch_requires_stopped_complete_named_release(self):
        state = self.root / "state" / "retained"
        self.touch(state)
        state.write_text("persistent fixture")
        self.live.touch()
        result = self.run_control("switch", "fixture")
        self.assertEqual(result.returncode, 1)
        self.assertEqual(os.readlink(self.root / "current"), "releases/fixture")
        self.live.unlink()
        for version in ("../fixture", "missing"):
            result = self.run_control("switch", version)
            self.assertEqual(result.returncode, 1)
        (self.root / "current").unlink()
        result = self.run_control("switch", "fixture")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(os.readlink(self.root / "current"), "releases/fixture")
        self.assertEqual(state.read_text(), "persistent fixture")
        self.assertEqual(list(self.root.glob(".current.*.tmp")), [])

    def test_service_owns_xdg_state_root(self):
        self.executable(self.release / "bin/webcodex-server", """#!/usr/bin/env python3
import json, os, signal
with open(os.environ['FAKE_SERVICE_CALLS'], 'a') as out:
    out.write(json.dumps({key: os.environ.get(key) for key in
              ('XDG_CONFIG_HOME', 'XDG_DATA_HOME', 'XDG_CACHE_HOME', 'XDG_STATE_HOME',
               'WEBCODEX_MCP_INSTRUCTIONS_FILE')}) + '\\n')
os.kill(os.getppid(), signal.SIGTERM)
""")
        result = subprocess.run(["bash", str(SCRIPTS / "service.sh"), "server"],
                                env=self.env, text=True, capture_output=True, timeout=10)
        self.assertEqual(result.returncode, 0, result.stderr)
        row = self.recorded(self.executions)[0]
        for key, suffix in (("XDG_CONFIG_HOME", "config"), ("XDG_DATA_HOME", "data"),
                            ("XDG_CACHE_HOME", "cache"), ("XDG_STATE_HOME", "state")):
            expected = self.root / "state/xdg" / suffix
            self.assertEqual(row[key], str(expected))
            self.assertTrue(expected.is_dir())
        self.assertEqual(row["WEBCODEX_MCP_INSTRUCTIONS_FILE"],
                         str(self.root / "current/AGENTS.md"))

    def test_cli_wrapper_preserves_arguments_and_binds_xdg_roots(self):
        self.executable(self.release / "bin/webcodex-cli", """#!/usr/bin/env python3
import json, os, sys
print(json.dumps({'args': sys.argv[1:], 'xdg': {key: os.environ.get(key) for key in
      ('XDG_CONFIG_HOME', 'XDG_DATA_HOME', 'XDG_CACHE_HOME', 'XDG_STATE_HOME')}}))
""")
        env = dict(self.env, XDG_CONFIG_HOME="/stale/config", XDG_DATA_HOME="/stale/data",
                   XDG_CACHE_HOME="/stale/cache", XDG_STATE_HOME="/stale/state")
        args = ["--help", "argument with spaces", "", "$literal;$(not-a-command)"]
        result = subprocess.run([str(SCRIPTS / "webcodex"), *args], env=env,
                                text=True, capture_output=True, timeout=10)
        self.assertEqual(result.returncode, 0, result.stderr)
        row = json.loads(result.stdout)
        self.assertEqual(row["args"], args)
        for key, suffix in (("XDG_CONFIG_HOME", "config"), ("XDG_DATA_HOME", "data"),
                            ("XDG_CACHE_HOME", "cache"), ("XDG_STATE_HOME", "state")):
            self.assertEqual(row["xdg"][key], str(self.root / "state/xdg" / suffix))


if __name__ == "__main__":
    unittest.main()

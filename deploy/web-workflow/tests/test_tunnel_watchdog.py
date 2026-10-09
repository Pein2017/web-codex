"""Network recovery must restart only the owned Tunnel, once per outage."""

import importlib.util
from pathlib import Path
import subprocess
import sys
import threading
import unittest
import urllib.error
from unittest.mock import patch


SCRIPT = Path(__file__).resolve().parents[1] / "tunnel_watchdog.py"


class WatchdogTests(unittest.TestCase):
    def load(self):
        spec = importlib.util.spec_from_file_location("tunnel_watchdog", SCRIPT)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module

    def run_sequence(self, online):
        module = self.load()
        stopped = threading.Event()
        ticks = iter(range(100))
        probes = iter(online)
        child = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(60)"])
        self.addCleanup(lambda: child.poll() is None and child.kill())

        def probe():
            try:
                return next(probes)
            except StopIteration:
                stopped.set()
                return True

        with patch.object(module, "proxy_online", side_effect=probe), \
                patch.object(module.time, "monotonic", side_effect=lambda: next(ticks)), \
                patch.object(stopped, "wait", return_value=False), \
                patch.object(module.subprocess, "Popen", return_value=child) as spawn, \
                patch.object(module, "log") as log, \
                patch.object(child, "terminate", wraps=child.terminate) as terminate:
            module.supervise(["fixture-tunnel"], stopped)
        self.assertIsNotNone(child.poll(), "Owned child must exit before retry")
        reconnects = [c for c in log.call_args_list
                      if c.args == ("watchdog_network_recovered_reconnect",)]
        return spawn.call_count, terminate.call_count, len(reconnects)

    def test_stable_network_does_not_restart(self):
        self.assertEqual(self.run_sequence([True] * 40), (1, 1, 0))

    def test_outage_allows_grace_then_one_reconnect(self):
        self.assertEqual(self.run_sequence([True, False] + [True] * 35), (1, 1, 1))

    def test_flapping_network_resets_grace(self):
        self.assertEqual(self.run_sequence([True, False] + [True] * 10 +
                                           [False] + [True] * 10), (1, 1, 0))

    def test_offline_start_does_not_launch_tunnel(self):
        module = self.load()
        stopped = threading.Event()
        with patch.object(module, "proxy_online", return_value=False), \
                patch.object(stopped, "wait", side_effect=lambda _: stopped.set()), \
                patch.object(module.subprocess, "Popen") as spawn:
            module.supervise(["fixture-tunnel"], stopped)
        spawn.assert_not_called()

    def test_proxy_requires_upstream_https_not_a_listening_port(self):
        module = self.load()
        with patch.object(module.urllib.request, "build_opener") as build:
            build.return_value.open.side_effect = urllib.error.URLError(
                "Tunnel connection failed: 502")
            self.assertFalse(module.proxy_online())
            build.return_value.open.side_effect = urllib.error.HTTPError(
                "https://api.openai.com/v1/models", 401, "Unauthorized", {}, None)
            self.assertTrue(module.proxy_online())

    def test_child_exit_is_returned_without_respawning(self):
        module = self.load()
        child = subprocess.Popen([sys.executable, "-c", "raise SystemExit(7)"])
        child.wait(timeout=5)
        with patch.object(module, "proxy_online", return_value=True), \
                patch.object(module.subprocess, "Popen", return_value=child) as spawn:
            self.assertEqual(module.supervise(["fixture"], threading.Event()), 7)
        spawn.assert_called_once()


if __name__ == "__main__":
    unittest.main()

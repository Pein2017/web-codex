"""Give the native Tunnel a grace period, then reconnect after proxy recovery."""

import json
import signal
import subprocess
import sys
import threading
import time
import urllib.error
import urllib.request


def proxy_online():
    # A listening frps port can survive a disconnected Wi-Fi client.
    opener = urllib.request.build_opener(urllib.request.ProxyHandler(
        {"https": "http://127.0.0.1:9090"}))
    try:
        with opener.open("https://api.openai.com/v1/models", timeout=4):
            return True
    except urllib.error.HTTPError as error:
        # Origin HTTP errors still prove CONNECT and certificate-verified TLS.
        error.close()
        return True
    except (OSError, urllib.error.URLError):
        return False


def log(event):
    print(json.dumps({"event": event}), flush=True)


def supervise(command, stopped):
    child = None
    outage = False
    recovered_at = None
    try:
        while not stopped.is_set():
            online = proxy_online()
            if stopped.is_set():
                break
            if child is None and online:
                child = subprocess.Popen(command)
                log("watchdog_tunnel_started")
            if child is not None and child.poll() is not None:
                return child.returncode
            if child is not None:
                if not online:
                    if not outage:
                        log("watchdog_proxy_offline")
                    outage = True
                    recovered_at = None
                elif outage:
                    if recovered_at is None:
                        recovered_at = time.monotonic()
                    if time.monotonic() - recovered_at >= 30:
                        # Local /readyz may remain true despite a lost Host link.
                        log("watchdog_network_recovered_reconnect")
                        return 0
            stopped.wait(5)
        return 0
    finally:
        if child is not None:
            if child.poll() is None:
                child.terminate()
            # Never start a replacement until the CLI and its Tunnel exit.
            child.wait()


if __name__ == "__main__":
    stopped = threading.Event()
    for sig in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
        signal.signal(sig, lambda *_: stopped.set())
    sys.exit(supervise(sys.argv[1:], stopped))

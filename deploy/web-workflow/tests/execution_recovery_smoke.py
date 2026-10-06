#!/usr/bin/env python3
"""Disposable native caller-to-consumer checks; never installs or touches live state.

All child commands operate on generated CPU fixtures. Keep the private temporary
directory as evidence; process shutdown targets only this invocation's Popen groups.
The direct loop has no Codex CLI agent or sandbox probe. Physical storage caps
are additionally qualified by the focused real-filesystem store/Runner tests;
this script qualifies their actual Server/Runner consumption and rollback seam.
"""

import argparse
import json
import os
from pathlib import Path
import signal
import shutil
import shlex
import socket
import sqlite3
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request


def require(condition, message):
    if not condition:
        raise AssertionError(message)


class Runtime:
    def __init__(self, server, runner, previous, previous_runner, evidence):
        self.server_bin, self.runner_bin, self.previous_bin = server, runner, previous
        self.previous_runner_bin = previous_runner
        self.root = Path(tempfile.mkdtemp(prefix="native-recovery-", dir=evidence))
        self.records = []
        self.children = {}
        self.handles = []
        self.token = "disposable-execution-recovery-only"
        with socket.socket() as sock:
            sock.bind(("127.0.0.1", 0))
            self.port = sock.getsockname()[1]
        self.url = f"http://127.0.0.1:{self.port}"
        self.client = "execution-recovery-fixture"
        self.project = f"agent:{self.client}:fixture"
        self.fixture = self.root / "fixture"
        self.fixture.mkdir()
        for name in ("data", "home", "projects", "xdg/config", "xdg/data", "xdg/state", "xdg/cache"):
            (self.root / name).mkdir(parents=True)
        (self.root / "empty.env").touch()
        # A deliberately unavailable CLI route counts attempted fallback. The
        # real installed Codex is never discovered or invoked by this harness.
        self.cli_counter = self.root / "codex-attempts"
        self.cli_counter.touch()
        tools = self.root / "direct-tools"
        tools.mkdir()
        denied_cli = tools / "codex"
        denied_cli.write_text("#!/bin/sh\nprintf attempted\\n >> " + shlex.quote(str(self.cli_counter)) + "\nexit 126\n")
        denied_cli.chmod(0o700)
        for tool in ("cargo", "rustc"):
            target = shutil.which(tool)
            require(target is not None, "required system compiler tool unavailable")
            (tools / tool).symlink_to(target)
        (self.fixture / "README.md").write_text("fixture input\n")
        (self.fixture / "Cargo.toml").write_text('[workspace]\n[package]\nname="recovery-fixture"\nversion="0.1.0"\nedition="2021"\n[lib]\npath="lib.rs"\n')
        (self.fixture / "lib.rs").write_text("pub fn value() -> u8 { 1 }\n")
        for argv in (["git", "init", "-b", "main"], ["git", "config", "user.name", "Recovery Fixture"],
                     ["git", "config", "user.email", "fixture@example.invalid"],
                     ["git", "add", "README.md", "Cargo.toml", "lib.rs"], ["git", "commit", "-m", "fixture"]):
            subprocess.run(argv, cwd=self.fixture, check=True, stdout=subprocess.DEVNULL,
                           stderr=subprocess.PIPE, timeout=10)
        (self.root / "projects/fixture.toml").write_text(
            f'id="fixture"\npath={json.dumps(str(self.fixture))}\nname="Recovery Fixture"\nallow_patch=true\nkind="text"\n')
        (self.root / "runner.toml").write_text(
            f'server_url={json.dumps(self.url)}\ntoken={json.dumps(self.token)}\nclient_id={json.dumps(self.client)}\n'
            f'project_registry_dir={json.dumps(str(self.root / "projects"))}\npoll_interval_ms=100\ntransport="polling"\n'
            f'[policy]\nallow_raw_shell=true\nallow_cwd_anywhere=false\nallowed_roots=[{json.dumps(str(self.fixture))}]\n'
            'max_timeout_secs=30\nmax_output_bytes=1048576\n')
        self.env = {"PATH": str(tools) + ":/usr/bin:/bin", "LANG": "C.UTF-8", "HOME": str(self.root / "home"),
                    "WEBCODEX_ENV_FILE": str(self.root / "empty.env"), "RUST_LOG": "warn",
                    "WEBCODEX_ADDR": f"127.0.0.1:{self.port}", "WEBCODEX_DATA": str(self.root / "data"),
                    "WEBCODEX_TOKEN": self.token, "WEBCODEX_TOOL_REQUEST_TRACE": "full",
                    "WEBCODEX_SHARED_KEY_ENABLED": "true",
                    "WEBCODEX_TOOL_REQUEST_TRACE_DIR": str(self.root / "traces")}
        for name in ("CONFIG", "DATA", "STATE", "CACHE"):
            self.env[f"XDG_{name}_HOME"] = str(self.root / f"xdg/{name.lower()}")
        # System toolchain dependency only; WebCodex-owned runtime state stays private.
        self.env["RUSTUP_HOME"] = os.environ.get("RUSTUP_HOME", "/root/.rustup")
        self.env["CARGO_HOME"] = str(self.root / "cargo")

    def start(self, kind, previous=False):
        require(kind not in self.children, "one live invocation per child")
        executable = ((self.previous_runner_bin if previous else self.runner_bin) if kind == "runner"
                      else self.previous_bin if previous else self.server_bin)
        argv = [str(executable)] + (["--config", str(self.root / "runner.toml")] if kind == "runner" else [])
        handle = open(self.root / f"{kind}-{'previous' if previous else 'candidate'}.log", "ab")
        self.handles.append(handle)
        self.children[kind] = subprocess.Popen(argv, cwd=self.fixture, env=self.env,
                                               stdin=subprocess.DEVNULL, stdout=handle,
                                               stderr=handle, start_new_session=True)

    def stop(self, kind):
        child = self.children.pop(kind, None)
        if child is None:
            return
        if child.poll() is None:
            os.killpg(child.pid, signal.SIGTERM)
        try:
            child.wait(timeout=10)
        except subprocess.TimeoutExpired:
            os.killpg(child.pid, signal.SIGKILL)
            child.wait(timeout=3)
        self.records.append({"process": kind, "pid": child.pid, "exit": child.returncode})

    def crash(self, kind):
        child = self.children.pop(kind)
        require(child.poll() is None, "fault target is not the owned live child")
        os.killpg(child.pid, signal.SIGKILL)
        child.wait(timeout=5)
        self.records.append({"process": kind, "pid": child.pid, "exit": child.returncode,
                             "fault": "owned disposable process crash"})

    def request(self, path, body, token=None):
        headers = {"Content-Type": "application/json", "Accept": "application/json, text/event-stream",
                   "Authorization": "Bearer " + (self.token if token is None else token)}
        if path == "/mcp":
            headers["mcp-protocol-version"] = "2026-07-28"
            headers["mcp-method"] = body["method"]
            headers["mcp-name"] = body["params"]["name"]
        request = urllib.request.Request(self.url + path, json.dumps(body).encode(),
                                         headers)
        try:
            with urllib.request.urlopen(request, timeout=40) as response:
                raw = response.read(2 * 1024 * 1024 + 1)
        except urllib.error.HTTPError as error:
            raw = error.read(128 * 1024)
        require(len(raw) <= 2 * 1024 * 1024, "response exceeded bounded reader")
        if raw.startswith(b"event:") or raw.startswith(b"data:"):
            raw = next(line[6:] for line in raw.splitlines() if line.startswith(b"data: "))
        return json.loads(raw)

    def call(self, name, arguments, session=None):
        body = {"tool": name, "params": arguments}
        if session:
            body["recording_session_id"] = session
        value = self.request("/api/tools/call", body)
        self.records.append({"tool": name, "arguments": arguments, "response": value})
        return value

    def app(self, selector, session):
        value = self.request("/mcp", {"jsonrpc": "2.0", "id": len(self.records) + 1,
            "method": "tools/call", "params": {"name": "work_result_state",
                "arguments": {"project": selector, "session_id": session},
                "_meta": {"io.modelcontextprotocol/protocolVersion": "2026-07-28",
                    "io.modelcontextprotocol/clientCapabilities": {"extensions": {
                        "io.modelcontextprotocol/ui": {"mimeTypes": ["text/html;profile=mcp-app"]}}}}}})
        self.records.append({"app_selector": selector, "response": value})
        return value.get("result", {}).get("structuredContent", {})

    def wait_ready(self, runner=True):
        deadline = time.monotonic() + 20
        value = None
        while time.monotonic() < deadline:
            require(all(child.poll() is None for child in self.children.values()), "fixture process exited")
            try:
                arguments = {"compact": True}
                if runner:
                    arguments["client_id"] = self.client
                value = self.request("/api/runtime/status", arguments)
                if value.get("success") and (not runner or value["output"].get("focus", {}).get("connected")):
                    return value
            except (OSError, ValueError, KeyError, StopIteration):
                pass
            time.sleep(0.1)
        self.records.append({"readiness_last_response": value, "runner_required": runner})
        raise AssertionError("disposable runtime did not become ready")

    def page(self, job, stdout=1, stderr=1, count=200):
        value = self.call("observe_jobs", {"items": [{"job_id": job,
            "since_stdout_line": stdout, "since_stderr_line": stderr}], "tail_lines": count})
        require(value.get("success"), "archive caller failed")
        output = value["output"]
        item = output.get("items", [output])[0]
        require(item.get("success", True), "archive item failed")
        return item.get("output", item)

    def save(self, status, error=None):
        (self.root / "receipt.json").write_text(json.dumps(
            {"status": status, "error": error, "source": str(self.server_bin),
             "no_install": True, "records": self.records}, indent=2))
        print(json.dumps({"status": status, "receipt": str(self.root / "receipt.json"),
                          "no_install": True, "error": error}))


def smoke(runtime):
    runtime.start("server")
    runtime.wait_ready(runner=False)
    runtime.start("runner")
    runtime.wait_ready()
    rejected = runtime.call("run_shell", {"project": runtime.project, "command": "pwd",
                                          "shell": "bash", "cwd": "missing-cwd", "timeout_secs": 5})
    require(not rejected.get("success"), "prestart rejection unexpectedly passed")
    require("structured_job_lifecycle_invalid" not in json.dumps(rejected), "producer still stamps premature start")
    require(rejected["output"].get("execution_state") == "not_started", "missing truthful prestart state")
    effect = runtime.call("run_shell", {"project": runtime.project,
        "command": "printf effect\\n >> one-effect; printf done", "shell": "bash", "timeout_secs": 5})
    require(effect.get("success") and effect["output"].get("exit_code") == 0, "side-effect command failed")
    require((runtime.fixture / "one-effect").read_text().count("effect") == 1, "payload repeated")
    boot = runtime.call("work_on_project", {"project": runtime.project,
        "instruction": "Disposable recovery acceptance only", "include_extension_catalog": False})
    require(boot.get("success"), "Session bootstrap failed")
    session = boot["output"]["session_id"]
    for selector in (runtime.project, "fixture", boot["output"]["project_ref"]):
        value = runtime.app(selector, session)
        require(value.get("success"), "authorized equivalent selector rejected")
        require(value["output"]["work_result"]["project"] == runtime.project, "canonical Session target drift")
    checked = runtime.call("cargo_check", {"project": runtime.project, "session_id": session,
        "all_targets": False, "timeout_secs": 30, "sync_wait_secs": 20}, session)
    require(checked.get("success"), "actual CPU validator failed")
    before = runtime.app(runtime.project, session)
    require(before.get("success"), "initial validation consumer failed")
    guarded = runtime.call("write_project_file", {"project": runtime.project, "session_id": session,
        "path": "README.md", "content": "must not be written", "overwrite": True,
        "expected_read_revision": 999}, session)
    require(not guarded.get("success") and guarded["output"].get("error_kind") == "unknown_read_revision",
            "guarded edit did not fail before dispatch")
    after = runtime.app(runtime.project, session)
    require(after.get("success") and after["output"]["work_result"]["validation"].get("current_status") == "unproven",
            "proven no-write falsely staled or certified source")
    require((runtime.fixture / "README.md").read_text() == "fixture input\n", "guarded rejection wrote source")
    read = runtime.call("read_files", {"project": runtime.project, "session_id": session,
        "items": [{"path": "lib.rs", "start_line": 1, "limit": 1}]}, session)
    require(read.get("success") and read["output"].get("failed_count", 0) == 0, "direct source read failed")
    read_item = read["output"]["items"][0]
    revision = read_item.get("output", read_item)["read_revision"]
    edited = runtime.call("write_project_file", {"project": runtime.project, "session_id": session,
        "path": "lib.rs", "content": "pub fn value() -> u8 { 2 }\n", "overwrite": True,
        "expected_read_revision": revision}, session)
    require(edited.get("success") and (runtime.fixture / "lib.rs").read_text() == "pub fn value() -> u8 { 2 }\n",
            "direct revision-guarded write failed")
    stale = runtime.call("write_project_file", {"project": runtime.project, "session_id": session,
        "path": "lib.rs", "content": "must not be written", "overwrite": True,
        "expected_read_revision": revision}, session)
    require(not stale.get("success") and (runtime.fixture / "lib.rs").read_text() == "pub fn value() -> u8 { 2 }\n",
            "stale read guard admitted a second write")
    direct_check = runtime.call("cargo_check", {"project": runtime.project, "session_id": session,
        "all_targets": False, "timeout_secs": 30, "sync_wait_secs": 20}, session)
    require(direct_check.get("success") and direct_check["output"].get("exit_code") == 0,
            "direct edited-source CPU validation failed")
    escaped = runtime.call("run_process", {"project": runtime.project, "executable": sys.executable,
        "args": ["-c", "raise SystemExit('must not run')"], "cwd": "..", "timeout_secs": 5})
    require(not escaped.get("success"), "direct Project cwd boundary was relaxed")
    output = runtime.call("run_process", {"project": runtime.project, "executable": sys.executable,
        "args": ["-c", "import sys,time; time.sleep(1.5); [sys.stdout.write(f'{i:05d}:'+'x'*90+'\\n') for i in range(2200)]"],
        "timeout_secs": 10, "sync_wait_secs": 1})
    require(output.get("success"), "archive output producer failed")
    job = output["output"]["job_id"]
    # Reconcile the same exact Job before paging, never replay its producer.
    continuation = dict(output["output"]["continuation"]["arguments"])
    continuation["wait_secs"] = 10
    terminal = runtime.call("observe_jobs", continuation)
    require(terminal.get("success"), "same Job terminal observation failed")
    terminal_item = terminal["output"]["items"][0]
    terminal_observation = terminal_item.get("output", terminal_item)
    same = runtime.call("observe_jobs", {"items": [{"job_id": job,
        "after_observation_token": terminal_observation["observation_token"]}]})
    same_item = same.get("output", {}).get("items", [{}])[0]
    same_observation = same_item.get("output", same_item)
    require(same.get("success") and same_observation.get("log_delta_status") == "unchanged"
            and not same_observation.get("stdout_tail") and not same_observation.get("stderr_tail"),
            "archive diverted ordinary terminal-token continuation and repeated output")
    position, chunks = 1, []
    for _ in range(20):
        page = runtime.page(job, position)
        require(not page.get("archive_unavailable"), "committed archive unavailable")
        chunks.append(page.get("stdout_tail", ""))
        following = page["cursor"]["stdout"]
        require(following >= position, "archive cursor regressed")
        if following == position or following >= 2201:
            break
        position = following
    text = "".join(chunks)
    require(text.startswith("00000:") and "02199:" in text and len(text) > 64 * 1024,
            "bounded model pages did not recover both ends beyond live tail")
    for index in range(70):
        result = runtime.call("run_shell", {"project": runtime.project,
            "command": f"printf inventory-{index}", "shell": "bash", "timeout_secs": 5})
        require(result.get("success"), "bounded inventory-eviction fixture failed")
    require(runtime.page(job)["stdout_tail"].startswith("00000:"), "live inventory eviction lost exact archive")
    malformed = runtime.call("observe_jobs", {"items": [{"job_id": job,
        "after_observation_token": "not-an-observation-token"}]})
    require(malformed.get("output", {}).get("failed_count") == 1,
            "archive lookup bypassed malformed-token rejection")
    runtime.stop("runner")
    runtime.start("runner")
    runtime.wait_ready()
    require(runtime.page(job)["stdout_tail"].startswith("00000:"), "instance replacement orphaned archive")
    runtime.stop("runner")
    runtime.stop("server")
    # Explicit clock-age simulation on disposable evidence only; no sealed or live state.
    database = runtime.root / "data/webcodex.db"
    locator = runtime.root / "data/job-archive-locator/archives.sqlite3"
    require(locator.is_file(), "independent locator database was not consumed")
    with sqlite3.connect(locator) as connection:
        row = connection.execute("SELECT payload FROM wc_job_archives WHERE job_id=?", (job,)).fetchone()
        require(row is not None, "archive locator absent before simulated age")
        value = json.loads(row[0])
        aged = value["archive"]["committed_at"] - 25 * 3600
        value["archive"]["committed_at"] = aged
        value["receipt"]["terminal_observed_at"] = aged
        value["receipt"]["expires_at"] = aged + 24 * 3600
        connection.execute("UPDATE wc_job_archives SET payload=?,committed_at=?,expires_at=? WHERE job_id=?",
                           (json.dumps(value), aged, aged + 7 * 24 * 3600, job))
    with sqlite3.connect(database) as connection:
        require(not connection.execute("SELECT 1 FROM sqlite_master WHERE name='wc_job_archives'").fetchone(),
                "archive locator still uses shared Session database")
        # Force ordinary startup pruning to exercise the real age boundary.
        connection.execute("UPDATE wc_job_receipts SET expires_at=?", (int(time.time()) - 1,))
    manifests = list((runtime.root / "xdg/state/webcodex/runner-job-archives-v1").glob("*/*/terminal.json"))
    matched = 0
    for manifest in manifests:
        descriptor = json.loads(manifest.read_text())
        if descriptor["job_id"] == job:
            descriptor["committed_at"] = aged
            manifest.write_text(json.dumps(descriptor))
            matched += 1
    require(matched == 1, "simulated age lost exact native manifest")
    runtime.start("server")
    runtime.wait_ready(runner=False)
    offline = runtime.page(job)
    require(set(offline) == {"job_id", "status", "exit_code"} and offline["status"] == "completed"
            and offline["exit_code"] == 0, "offline recovery leaked metadata or lost terminal truth")
    offline_invalid = runtime.call("observe_jobs", {"items": [{"job_id": job,
        "after_observation_token": "not-an-observation-token"}]})
    require(offline_invalid.get("output", {}).get("failed_count") == 1,
            "offline terminal fact bypassed malformed-token rejection")
    denied = runtime.request("/api/tools/call", {"tool": "observe_jobs", "params": {"items": [{"job_id": job}]}},
                             token="different-disposable-credential")
    runtime.records.append({"foreign_credential_observation": denied})
    # A successfully evaluated batch can contain only failed items; the outer
    # success flag is transport/batch success, not per-Job authorization.
    denied_output = denied.get("output", {})
    require((not denied.get("success", False) or
             (denied_output.get("failed_count") == 1 and denied_output.get("succeeded_count") == 0))
            and "00000:" not in json.dumps(denied)
            and not any(item.get("output") for item in denied_output.get("items", [])),
            "foreign credential saw archive")
    runtime.start("runner")
    runtime.wait_ready()
    require(runtime.page(job)["stdout_tail"].startswith("00000:"), "Server restart lost exact archive")
    # Only this fresh Server/expired ordinary receipt boundary removes its live
    # observation owner. Runner's 64-entry inventory alone does not imply that.
    recovered = runtime.call("observe_jobs", {"items": [{"job_id": job,
        "after_observation_token": terminal_observation["observation_token"]}]})
    recovered_item = recovered.get("output", {}).get("items", [{}])[0]
    recovered_observation = recovered_item.get("output", recovered_item)
    require(recovered.get("success") and recovered_observation.get("log_delta_status") == "reset"
            and recovered_observation.get("changed") is True,
            "historical archive falsely claimed live delta continuity")
    restored = runtime.call("session_summary", {"session_id": session, "limit": 5})
    require(restored.get("success"), "current Session did not restore")
    require(runtime.cli_counter.stat().st_size == 0, "direct loop attempted a local Codex fallback")
    runtime.records.append({"direct_loop": "PASS", "codex_cli_attempts": 0,
                            "read_edit_validate_same_job_restart": True})
    # Same canonical path, new root inode: neither metadata nor bytes may escape.
    runtime.stop("runner")
    original = runtime.root / "original-fixture"
    runtime.fixture.rename(original)
    runtime.fixture.mkdir()
    (runtime.fixture / "README.md").write_text("replacement root\n")
    runtime.start("runner")
    runtime.wait_ready()
    replaced = runtime.call("observe_jobs", {"items": [{"job_id": job, "since_stdout_line": 1}]})
    require(not replaced.get("success") or replaced.get("output", {}).get("failed_count", 0) == 1,
            "same-path replacement returned archive metadata")
    require("00000:" not in json.dumps(replaced) and '"committed_at"' not in json.dumps(replaced),
            "root replacement disclosed retained evidence")
    runtime.stop("runner")
    runtime.stop("server")
    archives_before = []
    for database in [locator]:
        with sqlite3.connect(database) as connection:
            if connection.execute("SELECT 1 FROM sqlite_master WHERE name='wc_job_archives'").fetchone():
                archives_before.append((database, connection.execute("SELECT job_id FROM wc_job_archives ORDER BY job_id").fetchall()))
    require(archives_before, "candidate did not commit archive locators")
    runtime.start("server", previous=True)
    runtime.wait_ready(runner=False)
    # Session authorization needs a current registered Project on both versions.
    # Use the retained previous pair, not mixed wire versions or auth bypass.
    runtime.start("runner", previous=True)
    runtime.wait_ready()
    old_session = runtime.call("session_summary", {"session_id": session, "limit": 5})
    require(old_session.get("success"), "retained previous app cannot read candidate Session state")
    runtime.stop("runner")
    runtime.stop("server")
    for database, expected in archives_before:
        with sqlite3.connect(database) as connection:
            require(connection.execute("SELECT job_id FROM wc_job_archives ORDER BY job_id").fetchall() == expected,
                    "previous-app pruning destroyed new archive state")
    # Re-admit the original root, not a different inode, then prove the final
    # candidate can consume the same archive after old-app pruning. No restore
    # of either database is used, and no producer is rerun.
    runtime.fixture.rename(runtime.root / "replacement-fixture")
    original.rename(runtime.fixture)
    runtime.start("server")
    runtime.wait_ready(runner=False)
    runtime.start("runner")
    runtime.wait_ready()
    require(runtime.page(job)["stdout_tail"].startswith("00000:"),
            "candidate cannot recover original archive after previous-app pruning")
    require(runtime.cli_counter.stat().st_size == 0, "rollback/restart attempted a local Codex fallback")
    runtime.records.append({"independent_locator_rollback_readback": "PASS", "codex_cli_attempts": 0})
    # Lose the execution owner after an observable effect but before any terminal
    # commit. A fresh Server may not manufacture a verdict or repeat the payload.
    uncertain = runtime.call("run_process", {"project": runtime.project, "executable": sys.executable,
        "args": ["-c", "import os,pathlib,time; pathlib.Path('uncertain.pid').write_text(str(os.getpid())); "
                 "p=pathlib.Path('uncertain-once'); "
                 "p.write_text(p.read_text()+'effect\\n' if p.exists() else 'effect\\n'); time.sleep(3)"],
        "timeout_secs": 8, "sync_wait_secs": 1})
    require(uncertain.get("success") and uncertain["output"].get("job_id"), "uncertain fixture not admitted")
    uncertain_job = uncertain["output"]["job_id"]
    require(uncertain["output"].get("execution_state") != "completed", "fixture terminated before injected crash")
    require((runtime.fixture / "uncertain-once").read_text() == "effect\n", "uncertain effect missing")
    runtime.crash("runner")
    runtime.stop("server")
    runtime.start("server")
    runtime.wait_ready(runner=False)
    lost = runtime.call("observe_jobs", {"items": [{"job_id": uncertain_job}]})
    require(not lost.get("success") or lost.get("output", {}).get("failed_count") == 1,
            "missing terminal commit manufactured a recovered verdict")
    require((runtime.fixture / "uncertain-once").read_text() == "effect\n"
            and runtime.cli_counter.stat().st_size == 0, "unknown outcome triggered replay or local-agent fallback")
    native_pid = int((runtime.fixture / "uncertain.pid").read_text())
    deadline = time.monotonic() + 8
    while time.monotonic() < deadline:
        try:
            # A zombie has exited and owns no executable file descriptors.
            state = Path(f"/proc/{native_pid}/stat").read_text().rsplit(")", 1)[1].split()[0]
            if state == "Z":
                break
        except FileNotFoundError:
            break
        time.sleep(0.05)
    else:
        raise AssertionError("disposable native child did not exit after owner loss")
    runtime.records.append({"unknown_outcome": "not promoted to terminal proof", "original_job_id": uncertain_job,
                            "payload_effects": 1, "native_child_no_longer_running": True, "codex_cli_attempts": 0})
    return "candidate_native_direct_recovery_passed"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("server", "runner", "previous-server", "previous-runner", "evidence-root"):
        parser.add_argument("--" + name, required=True, type=Path)
    args = parser.parse_args()
    os.umask(0o077)
    require(args.evidence_root.is_dir(), "explicit private evidence parent required")
    runtime = Runtime(args.server.resolve(strict=True), args.runner.resolve(strict=True),
                      args.previous_server.resolve(strict=True), args.previous_runner.resolve(strict=True),
                      args.evidence_root.resolve(strict=True))
    status, error = "FAIL", None
    try:
        status = smoke(runtime)
        return_code = 0
    except Exception as failure:
        error = f"{type(failure).__name__}: {failure}"
        return_code = 1
    finally:
        runtime.stop("runner")
        runtime.stop("server")
        for handle in runtime.handles:
            handle.close()
        runtime.save(status, error)
    return return_code


if __name__ == "__main__":
    sys.exit(main())

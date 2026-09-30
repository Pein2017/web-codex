#!/usr/bin/env python3
"""Disposable native MCP Server + Runner smoke for the current Web workflow."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import secrets
import shutil
import signal
import socket
import subprocess
import sys
import tempfile
import time
from typing import Any
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen


MAX_TIMEOUT_SECS = 300
SERVER_VERSION = "0.4.3"
RUNTIME_PROJECT = "agent:web-workflow-e2e:isolated"
STATELESS_MCP_VERSION = "2026-07-28"
REQUIRED_TOOLS = {
    "runtime_status",
    "list_projects",
    "list_runners",
    "work_on_project",
    "read_files",
    "apply_text_edits",
    "run_process",
    "run_script",
    "run_shell",
    "run_job",
    "observe_jobs",
    "show_changes",
    "finish_coding_task",
    "skill_list",
    "skill_read_file",
    "session_handoff_summary",
    "workspace_hygiene_check",
    "git_diff_hunks",
    "plugin_tool",
}


class Failure(RuntimeError):
    pass


def parse_args() -> argparse.Namespace:
    root = Path(__file__).resolve().parent.parent
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--server-bin", type=Path, default=root / "target/release/webcodex-server"
    )
    parser.add_argument(
        "--runner-bin", type=Path, default=root / "target/release/webcodex-runner"
    )
    parser.add_argument(
        "--artifact-dir",
        type=Path,
        default=Path("/tmp")
        / f"webcodex-e2e-web-workflow-{int(time.time())}-{os.getpid()}",
    )
    parser.add_argument("--timeout-secs", type=int, default=180)
    args = parser.parse_args()
    if not 1 <= args.timeout_secs <= MAX_TIMEOUT_SECS:
        parser.error(f"--timeout-secs must be between 1 and {MAX_TIMEOUT_SECS}")
    return args


def bounded(value: Any, limit: int = 1600) -> str:
    rendered = json.dumps(value, sort_keys=True, separators=(",", ":"))
    return rendered if len(rendered) <= limit else rendered[:limit] + "...<truncated>"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise Failure(message)


def require_native_context(value: Any) -> None:
    retired = {
        "context_revision", "session_context_revision", "ack_session_context_revision",
        "session_continuity", "session_recovery",
    }
    if isinstance(value, dict):
        require(not retired.intersection(value), "retired Session context fields were emitted")
        for nested in value.values():
            require_native_context(nested)
    elif isinstance(value, list):
        for nested in value:
            require_native_context(nested)


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def clean_env() -> dict[str, str]:
    env = {k: v for k, v in os.environ.items() if not k.startswith("WEBCODEX_")}
    env["RUST_LOG"] = "info"
    return env


def terminate_group(process: subprocess.Popen[bytes] | None) -> None:
    if process is None:
        return
    if process.poll() is None:
        try:
            os.killpg(process.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        try:
            process.wait(timeout=3)
        except subprocess.TimeoutExpired:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            process.wait(timeout=3)
    else:
        process.wait()


def main() -> int:
    args = parse_args()
    started = time.monotonic()
    deadline = started + args.timeout_secs
    artifacts = args.artifact_dir.resolve()
    artifacts.mkdir(parents=True, exist_ok=False)
    smoke_path = artifacts / "e2e-web-workflow.log"
    server_path = artifacts / "server.log"
    runner_path = artifacts / "runner.log"
    receipt_path = artifacts / "receipt.json"
    checks: list[str] = []
    binaries: dict[str, Any] = {}
    server: subprocess.Popen[bytes] | None = None
    runner: subprocess.Popen[bytes] | None = None
    temp: tempfile.TemporaryDirectory[str] | None = None
    outcome = "failed"
    error_text: str | None = None

    with (
        smoke_path.open("w", encoding="utf-8") as smoke_log,
        server_path.open("wb") as server_log,
        runner_path.open("wb") as runner_log,
    ):
        def log(message: str) -> None:
            line = f"[e2e-web] {message}"
            print(line, flush=True)
            smoke_log.write(line + "\n")
            smoke_log.flush()

        def ok(message: str) -> None:
            checks.append(message)
            log(f"[ok] {message}")

        def remaining() -> float:
            seconds = deadline - time.monotonic()
            if seconds <= 0:
                raise Failure(f"overall timeout exceeded ({args.timeout_secs}s maximum)")
            return seconds

        def setup(argv: list[str], cwd: Path) -> None:
            try:
                subprocess.run(
                    argv,
                    cwd=cwd,
                    check=True,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    timeout=min(15.0, remaining()),
                )
            except subprocess.CalledProcessError as exc:
                detail = exc.stderr.decode("utf-8", errors="replace")[-1000:]
                raise Failure(f"setup command {argv[0]} failed: {detail}") from exc

        def binary_identity(label: str, configured: Path) -> Path:
            path = configured.resolve()
            require(path.is_file() and os.access(path, os.X_OK), f"bad {label} binary: {path}")
            version = subprocess.run(
                [str(path), "--version"],
                check=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
                timeout=min(10.0, remaining()),
            ).stdout.strip()
            binaries[label] = {
                "path": str(path),
                "version": version,
                "sha256": sha256_file(path),
            }
            ok(f"{label} binary identity: {version}")
            return path

        try:
            server_bin = binary_identity("server", args.server_bin)
            runner_bin = binary_identity("runner", args.runner_bin)
            node = shutil.which("node")
            plugin_entry = Path(__file__).resolve().parent.parent / "plugins/web-workflow/plugin.mjs"
            require(node is not None and plugin_entry.is_file(), "Node/plugin fixture unavailable")
            temp = tempfile.TemporaryDirectory(prefix="webcodex-e2e-web-workflow-")
            root = Path(temp.name)
            data = root / "data"
            registry = root / "project-registry"
            project = root / "project"
            memory = root / "memory"
            for directory in (data, registry, project, memory):
                directory.mkdir()
            memory_body = "DISPOSABLE_MEMORY_SENTINEL: isolated provider content only.\n"
            (memory / "MEMORY.md").write_text(memory_body, encoding="utf-8")

            original = "# Isolated Web Workflow\n\nstate: original"
            edited = "# Isolated Web Workflow\n\nstate: edited-and-validated"
            repository_guidance = (
                "REPOSITORY_GUIDANCE_SENTINEL_MUST_NOT_APPEAR_IN_DEFAULT_BOOTSTRAP\n"
            )
            guidance = (
                "DEDICATED_WEB_GUIDANCE_SENTINEL: use the configured Runner project "
                "and validate changes before completion."
            )
            (project / "README.md").write_text(original, encoding="utf-8")
            (project / "AGENTS.md").write_text(repository_guidance, encoding="utf-8")
            (project / "existing.txt").write_text("existing original\n", encoding="utf-8")
            (project / "new.txt").write_text("new original\n", encoding="utf-8")
            skill_dir = project / ".agents/skills/workflow-smoke"
            skill_dir.mkdir(parents=True)
            skill_body = (
                "---\nname: workflow-smoke\ndescription: Isolated MCP Skill read fixture\n"
                "---\nSKILL_READ_SENTINEL: preserve unrelated work.\n"
            )
            (skill_dir / "SKILL.md").write_text(skill_body, encoding="utf-8")
            # Cross the production Runner's 256 KiB retained-output boundary
            # without exceeding its per-path bounds. No real checkout is used.
            tracked_dir = project / "tracked-fixture"
            tracked_dir.mkdir()
            for index in range(1100):
                (tracked_dir / (f"{index:04d}-" + "x" * 211 + ".txt")).touch()
            (project / "tests").mkdir()
            (project / "tests/test_workflow.py").write_text(
                "from pathlib import Path\n\n"
                "def test_guarded_readme():\n"
                "    assert 'state: edited-and-validated' in Path('README.md').read_text()\n",
                encoding="utf-8",
            )
            (project / "tests/test_async_workflow.py").write_text(
                "from pathlib import Path\n"
                "import time\n\n"
                "def test_async_guarded_readme():\n"
                "    time.sleep(3)\n"
                "    assert 'state: edited-and-validated' in Path('README.md').read_text()\n",
                encoding="utf-8",
            )
            guidance_path = root / "mcp-instructions.md"
            guidance_path.write_text(guidance, encoding="utf-8")
            setup(["git", "init", "-b", "main"], project)
            setup(["git", "config", "user.email", "e2e@example.invalid"], project)
            setup(["git", "config", "user.name", "Web Workflow E2E"], project)
            setup(
                [
                    "git", "add", "README.md", "AGENTS.md", "existing.txt",
                    "new.txt", "tests/test_workflow.py",
                    "tests/test_async_workflow.py",
                    ".agents/skills/workflow-smoke/SKILL.md",
                    "tracked-fixture",
                ],
                project,
            )
            setup(["git", "commit", "-m", "isolated fixture"], project)

            (registry / "isolated.toml").write_text(
                'id = "isolated"\n'
                f"path = {json.dumps(str(project))}\n"
                'name = "Isolated Web Workflow"\n'
                "allow_patch = true\n"
                'kind = "repo"\n',
                encoding="utf-8",
            )
            token = secrets.token_urlsafe(32)
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
                sock.bind(("127.0.0.1", 0))
                port = int(sock.getsockname()[1])
            runner_config = root / "runner.toml"
            runner_config.write_text(
                f'server_url = "http://127.0.0.1:{port}"\n'
                f"token = {json.dumps(token)}\n"
                'client_id = "web-workflow-e2e"\n'
                'display_name = "Web Workflow E2E"\n'
                'owner = "e2e"\n'
                f"project_registry_dir = {json.dumps(str(registry))}\n"
                "poll_interval_ms = 250\n"
                'transport = "websocket"\n\n'
                "[policy]\n"
                "allow_raw_shell = true\n"
                "allow_cwd_anywhere = false\n"
                f"allowed_roots = [{json.dumps(str(project))}]\n"
                "max_timeout_secs = 60\n"
                "max_output_bytes = 262144\n\n"
                "[shell.profiles.web-workflow]\n"
                'program = "sh"\nargs = ["-c"]\n\n'
                "[shell.profiles.web-workflow.env]\n"
                f"WEBCODEX_WEB_WORKFLOW_PROJECTS_JSON = {json.dumps(json.dumps({'isolated': str(project)}))}\n"
                f"WEBCODEX_WEB_WORKFLOW_MEMORY_ROOT = {json.dumps(str(memory))}\n\n"
                "[plugins]\nrequest_timeout_secs = 20\n\n"
                "[[plugins.providers]]\n"
                'id = "web-workflow"\nname = "Disposable Web Workflow"\n'
                f"command = {json.dumps(node)}\n"
                f"args = [{json.dumps(str(plugin_entry))}]\n"
                f"cwd = {json.dumps(str(project))}\n"
                'profile = "web-workflow"\ntimeout_secs = 20\n',
                encoding="utf-8",
            )

            server_env = clean_env()
            server_env.update(
                {
                    "WEBCODEX_ADDR": f"127.0.0.1:{port}",
                    "WEBCODEX_DATA": str(data),
                    "WEBCODEX_TOKEN": token,
                    "WEBCODEX_MCP_INSTRUCTIONS_FILE": str(guidance_path),
                }
            )
            server = subprocess.Popen(
                [str(server_bin)],
                env=server_env,
                stdout=server_log,
                stderr=subprocess.STDOUT,
                start_new_session=True,
            )
            while True:
                remaining()
                require(server.poll() is None, f"Server exited early ({server.returncode})")
                try:
                    with socket.create_connection(("127.0.0.1", port), timeout=0.2):
                        break
                except OSError:
                    time.sleep(0.1)
            ok(f"Server listening on isolated port {port}")
            runner = subprocess.Popen(
                [str(runner_bin), "--config", str(runner_config)],
                env=clean_env(),
                stdout=runner_log,
                stderr=subprocess.STDOUT,
                start_new_session=True,
            )

            rpc_id = 0

            def rpc(
                method: str,
                params: Any,
                *,
                expect_error: bool = False,
                legacy_initialize: bool = False,
            ) -> dict[str, Any]:
                nonlocal rpc_id
                rpc_id += 1
                headers = {
                    "Authorization": f"Bearer {token}",
                    "Content-Type": "application/json",
                }
                if not legacy_initialize:
                    require(isinstance(params, dict), f"{method} params must be an object")
                    params = {
                        **params,
                        "_meta": {
                            "io.modelcontextprotocol/protocolVersion": STATELESS_MCP_VERSION,
                            "io.modelcontextprotocol/clientCapabilities": {},
                        },
                    }
                    headers["MCP-Protocol-Version"] = STATELESS_MCP_VERSION
                    headers["Mcp-Method"] = method
                    if method == "tools/call":
                        require(isinstance(params.get("name"), str), "missing tool name")
                        headers["Mcp-Name"] = params["name"]
                payload = json.dumps(
                    {"jsonrpc": "2.0", "id": rpc_id, "method": method, "params": params},
                    separators=(",", ":"),
                ).encode()
                request = Request(
                    f"http://127.0.0.1:{port}/mcp",
                    data=payload,
                    headers=headers,
                    method="POST",
                )
                try:
                    with urlopen(
                        request,
                        timeout=min(
                            25.0 if method == "tools/call"
                            and params.get("name") == "observe_jobs" else 10.0,
                            remaining(),
                        ),
                    ) as response:
                        body = json.loads(response.read())
                except HTTPError as exc:
                    detail = exc.read().decode("utf-8", errors="replace")
                    try:
                        body = json.loads(detail)
                    except json.JSONDecodeError as parse_error:
                        raise Failure(
                            f"HTTP {exc.code} for MCP {method}: {detail[-1200:]}"
                        ) from parse_error
                except (URLError, OSError, json.JSONDecodeError) as exc:
                    raise Failure(f"MCP {method} transport failed: {exc}") from exc
                require(body.get("id") == rpc_id, bounded(body))
                if expect_error:
                    error = body.get("error")
                    require(isinstance(error, dict), f"missing RPC error: {bounded(body)}")
                    return error
                require("error" not in body, bounded(body))
                require(isinstance(body.get("result"), dict), bounded(body))
                return body["result"]

            recording_session_id: str | None = None
            by_name: dict[str, dict[str, Any]] = {}
            contracts: dict[str, dict[str, Any]] = {}

            def recorded_arguments(arguments: Any) -> Any:
                if recording_session_id is None:
                    return arguments
                require(isinstance(arguments, dict), "recorded call needs object arguments")
                return {**arguments, "recording_session_id": recording_session_id}

            def tool_request(name: str, arguments: Any) -> dict[str, Any]:
                arguments = recorded_arguments(arguments)
                if name in by_name:
                    request = {"name": name, "arguments": arguments}
                else:
                    require(name in contracts, f"undiscovered tool: {name}")
                    route = contracts[name].get("route", {})
                    selected = route.get("primary", {})
                    if selected.get("mode") == "direct":
                        selected = route.get("fallback") or {}
                    require(
                        selected.get("mode") == "gateway"
                        and selected.get("tool") == "call_runtime_tool"
                        and selected.get("target") == name
                        and selected.get("blocked_when_mcp_apps_enabled") is not True
                        and "call_runtime_tool" in by_name,
                        f"no admitted callable route for {name}: {bounded(route)}",
                    )
                    request = {
                        "name": "call_runtime_tool",
                        "arguments": {"tool": name, "arguments": arguments},
                    }
                return request

            def tool_rpc(
                name: str, arguments: Any, *, expect_error: bool = False,
            ) -> dict[str, Any]:
                return rpc("tools/call", tool_request(name, arguments), expect_error=expect_error)

            def call(name: str, arguments: Any) -> Any:
                result = tool_rpc(name, arguments)
                structured = result.get("structuredContent")
                require(
                    result.get("isError") is not True
                    and isinstance(structured, dict)
                    and structured.get("success") is True
                    and "output" in structured,
                    f"{name} failed: {bounded(result)}",
                )
                require_native_context(structured["output"])
                return structured["output"]

            def call_failure(name: str, arguments: dict[str, Any]) -> dict[str, Any]:
                result = tool_rpc(name, arguments)
                structured = result.get("structuredContent")
                require(
                    result.get("isError") is True
                    and isinstance(structured, dict)
                    and structured.get("success") is False
                    and isinstance(structured.get("output"), dict),
                    f"{name} did not return a structured tool failure: {bounded(result)}",
                )
                require_native_context(structured["output"])
                return structured["output"]

            def plugin(arguments: dict[str, Any]) -> dict[str, Any]:
                result = tool_rpc("plugin_tool", arguments)
                require(
                    result.get("isError") is not True
                    and isinstance(result.get("structuredContent"), dict)
                    and isinstance(result.get("content"), list),
                    f"Plugin gateway failed: {bounded(result)}",
                )
                require_native_context(result)
                return result

            initialized = rpc(
                "initialize",
                {"protocolVersion": "2025-06-18", "capabilities": {}},
                legacy_initialize=True,
            )
            require(
                initialized.get("serverInfo", {}).get("name") == "webcodex"
                and initialized.get("serverInfo", {}).get("version") == SERVER_VERSION
                and "runtimeExposure" not in initialized.get("serverInfo", {})
                and initialized.get("instructions") == guidance,
                f"initialize contract mismatch: {bounded(initialized)}",
            )
            ok(f"initialize returns configured guidance and exact native Server {SERVER_VERSION}")

            page = rpc("tools/list", {})
            seen_cursors: set[str] = set()
            while True:
                tools = page.get("tools")
                require(isinstance(tools, list), "tools/list omitted tools")
                by_name.update({tool["name"]: tool for tool in tools if isinstance(tool, dict)})
                cursor = page.get("nextCursor")
                if cursor is None:
                    break
                require(isinstance(cursor, str) and cursor not in seen_cursors,
                        "tools/list returned an invalid/non-progressing cursor")
                seen_cursors.add(cursor)
                page = rpc("tools/list", {"cursor": cursor})
            require("tool_manifest" in by_name, "native tool_manifest unavailable")
            require("job_tail" not in by_name, "ModelHidden job_tail was exposed")
            for name in sorted(REQUIRED_TOOLS):
                contract = call("tool_manifest", {"tool_name": name})
                require(contract.get("name") == name
                        and isinstance(contract.get("input_schema"), dict),
                        f"exact manifest omitted {name}: {bounded(contract)}")
                contracts[name] = contract
                tool_request(name, {})  # Check admission without dispatching an operation.
            work_props = contracts["work_on_project"]["input_schema"].get("properties", {})
            require(
                work_props.get("guidance_profile", {}).get("default") == "direct"
                and work_props.get("include_extension_catalog", {}).get("default") is True
                and "include_project_instructions" not in work_props
                and "include_workflow_guidance" not in work_props,
                "work_on_project schema defaults drifted",
            )
            observe_input = contracts["observe_jobs"]["input_schema"]
            require(
                "items" in observe_input.get("required", [])
                and observe_input.get("properties", {}).get("wake_on", {}).get("enum")
                == ["change", "terminal", "all_terminal"],
                "observe_jobs schema drifted",
            )
            languages = contracts["run_script"]["input_schema"].get(
                "properties", {}
            ).get("language", {}).get("enum")
            require(
                isinstance(languages, list) and "javascript" in languages,
                "run_script did not advertise JavaScript",
            )
            ok(
                f"native tools/list plus exact manifests admit every smoke tool "
                f"({len(by_name)} direct tools)"
            )

            compact_error = tool_rpc(
                "git_diff_hunks", {
                    "project": RUNTIME_PROJECT,
                    "max_lines_per_hunk": 80,
                },
                expect_error=True,
            )
            error_message = compact_error.get("message")
            require(
                compact_error.get("code") == -32602
                and isinstance(error_message, str)
                and "unknown field" in error_message
                and "properties" not in error_message
                and "additionalProperties" not in error_message
                and len(error_message) < 800,
                f"invalid arguments leaked a schema: {bounded(compact_error)}",
            )
            ok("a server-side invalid argument returns a compact MCP error")
            skill_error = tool_rpc("skill_read_file", {
                "project": RUNTIME_PROJECT,
            }, expect_error=True)
            require(
                skill_error.get("code") == -32602
                and len(json.dumps(skill_error)) < 800
                and "skill_id" in skill_error.get("message", "")
                and "additionalProperties" not in json.dumps(skill_error)
                and '"properties"' not in json.dumps(skill_error),
                f"Skill argument error leaked its schema: {bounded(skill_error)}",
            )
            ok("Skill argument rejection is compact and never dumps the complete schema")

            registered = None
            while registered is None:
                remaining()
                require(runner.poll() is None, f"Runner exited early ({runner.returncode})")
                projects = call("list_projects", {}).get("projects", [])
                registered = next(
                    (item for item in projects if item.get("id") == RUNTIME_PROJECT), None
                )
                if registered is None:
                    time.sleep(0.2)
            require(
                registered.get("capabilities", {}).get("git_available") is True,
                f"registered project was not identified as Git: {bounded(registered)}",
            )
            ok("real WebSocket Runner registered the isolated Git project")

            runner_view = call(
                "list_runners", {"client_id": "web-workflow-e2e", "summary_only": False}
            )
            runner_agents = runner_view.get("agents", [])
            require(
                len(runner_agents) == 1
                and runner_agents[0].get("capabilities", {}).get(
                    "structured_script_javascript"
                ) is True,
                f"registered Runner lacks the advertised JavaScript protocol capability: {bounded(runner_view)}",
            )
            ok("the actual Runner advertises JavaScript typed-script semantics")

            hygiene = call("workspace_hygiene_check", {
                "project": RUNTIME_PROJECT, "include_tracked": True,
            })
            require(
                hygiene.get("git_available") is True
                and hygiene.get("clean") is True
                and hygiene.get("truncated", False) is False
                and not any(
                    finding.get("kind") == "dirty_worktree"
                    for finding in hygiene.get("findings", [])
                ),
                f"large clean tracked listing fabricated dirty Git status: {bounded(hygiene)}",
            )
            ok("large clean tracked inventory cannot fabricate a dirty-worktree finding")
            (project / "existing.txt").write_text("already dirty at startup\n", encoding="utf-8")

            bootstrap = call(
                "work_on_project",
                {
                    "project": RUNTIME_PROJECT,
                    "instruction": "Edit README.md and validate the resulting state.",
                },
            )
            recording_session_id = bootstrap.get("session_id")
            require(
                isinstance(recording_session_id, str)
                and recording_session_id.startswith("wc_sess_"),
                f"bootstrap omitted an explicit Workflow Session: {bounded(bootstrap)}",
            )
            expected_guidance = {
                "revision": "sha256:" + hashlib.sha256(guidance.encode()).hexdigest(),
                "size_bytes": len(guidance.encode()),
            }
            sources = bootstrap.get("instructions", {}).get("sources")
            serialized = json.dumps(bootstrap, sort_keys=True)
            require(
                bootstrap.get("mcp_guidance") == expected_guidance
                and "workflow" not in bootstrap
                and isinstance(sources, list)
                and len(sources) == 1
                and isinstance(sources[0].get("fingerprint"), str)
                and "content" not in sources[0]
                and guidance not in serialized
                and repository_guidance.strip() not in serialized,
                f"default bootstrap guidance contract mismatch: {bounded(bootstrap)}",
            )
            ok("bootstrap defaults omit guidance bodies while preserving both identities")
            baseline = bootstrap.get("workspace_baseline", {})
            require(
                baseline.get("status") == "complete"
                and baseline.get("pre_existing_dirty_count") == 1,
                f"startup did not retain the already-dirty path: {bounded(bootstrap)}",
            )
            ok("Session startup observed one pre-existing dirty Git path")

            startup_skills = bootstrap.get("extensions", {}).get("skills", {}).get("entries", [])
            startup_skill = next((s for s in startup_skills if s.get("name") == "workflow-smoke"), None)
            require(startup_skill is not None, "startup omitted fixture Skill")
            require(
                isinstance(startup_skill.get("skill_id"), str)
                and "suggested_call" not in startup_skill
                and "definition_revision" not in startup_skill,
                "startup Skill catalog must remain selection-only",
            )

            catalog = call("skill_list", {"project": RUNTIME_PROJECT, "query": "workflow-smoke"})
            skills = catalog.get("skills", [])
            require(len(skills) == 1, f"Skill fixture was not discovered: {bounded(catalog)}")
            skill = skills[0]
            require(skill.get("skill_id") == startup_skill["skill_id"]
                    and isinstance(skill.get("definition_revision"), str),
                    "skill_list did not resolve the selected Skill definition revision")
            skill_text = call("skill_read_file", {
                "project": RUNTIME_PROJECT,
                "skill_id": skill["skill_id"],
                "expected_definition_revision": skill["definition_revision"],
            })
            require(
                "SKILL_READ_SENTINEL" in skill_text.get("text", ""),
                f"discovered opaque Skill ID cannot be read: {bounded(skill_text)}",
            )
            ok("selection-only startup Skill resolves through skill_list to guarded skill_read_file")

            startup_plugins = bootstrap.get("extensions", {}).get("plugins", {}).get("entries", [])
            require(any(entry.get("plugin") == "web-workflow"
                        and entry.get("tool") == "memory_read" for entry in startup_plugins),
                    "startup omitted the disposable project-scoped Plugin")
            providers = plugin({"action": "list", "runner": "web-workflow-e2e"})
            require(any(entry.get("plugin") == "web-workflow"
                        for entry in providers["structuredContent"].get("plugins", [])),
                    "Runner did not list the configured disposable Plugin")
            plugin_catalog = plugin({
                "action": "list", "runner": "web-workflow-e2e", "plugin": "web-workflow",
            })["structuredContent"]
            plugin_names = {entry.get("name") for entry in plugin_catalog.get("tools", [])}
            require({"memory_read", "memory_search", "pytest_report_summary"}.issubset(plugin_names)
                    and "public_history_read" not in plugin_names
                    and "codegraph_scoped_query" not in plugin_names,
                    "disposable Plugin advertised an unconfigured capability")
            described = plugin({
                "action": "describe", "runner": "web-workflow-e2e",
                "plugin": "web-workflow", "tool": "memory_read",
            })["structuredContent"]
            provider_schema = described.get("tool", {}).get("inputSchema", {})
            require(isinstance(described.get("binding"), str)
                    and described.get("tool", {}).get("name") == "memory_read"
                    and provider_schema.get("required") == ["path"]
                    and provider_schema.get("additionalProperties") is False,
                    f"Plugin describe omitted its exact schema/binding: {bounded(described)}")
            memory_result = plugin({
                "action": "call", "binding": described["binding"],
                "arguments": {"path": "MEMORY.md", "offset": 0, "maxBytes": 64},
            })
            memory_output = memory_result["structuredContent"]
            expected_provider_text = (
                f"Read {len(memory_body.encode())} UTF-8 source bytes from MEMORY.md at byte 0. End of file."
            )
            require(
                memory_output.get("text") == memory_body
                and memory_output.get("source") == {"root": str(memory.resolve()), "path": "MEMORY.md"}
                and memory_output.get("eof") is True
                and memory_output.get("nextOffset") == len(memory_body.encode())
                and "success" not in memory_output
                and memory_result["content"] == [{"type": "text", "text": expected_provider_text}]
                and (memory / "MEMORY.md").read_text(encoding="utf-8") == memory_body,
                f"native Plugin call changed provider content/provenance: {bounded(memory_result)}",
            )
            ok("real Plugin list/describe/bound memory read preserves provider content and fixture provenance")

            handoff = call("session_handoff_summary", {
                "session_id": recording_session_id,
                "project": RUNTIME_PROJECT,
                "include_workspace": True,
                "include_checkpoints": True,
                "include_validation": True,
                "diagnostic": True,
                "limit": 20,
            })
            require(
                handoff.get("session_id") == recording_session_id
                and isinstance(handoff.get("handoff_brief"), dict)
                and handoff.get("diagnostic") is True,
                f"explicit native handoff omitted the exact Session/evidence: {bounded(handoff)}",
            )
            ok("explicit Session handoff follows native recovery without retired context metadata")
            for _ in range(2):
                checkpoint = call("run_process", {
                    "project": RUNTIME_PROJECT,
                    "session_id": recording_session_id,
                    "executable": "python3",
                    "args": ["-c", "print('explicit-recorder-ok')"],
                    "purpose": "diagnostic",
                    "sync_wait_secs": 20,
                    "timeout_secs": 20,
                })
                require(
                    "explicit-recorder-ok" in checkpoint.get("stdout_tail", "")
                    and "workflow_recording_attention" not in checkpoint,
                    f"explicit recorder was lost: {bounded(checkpoint)}",
                )
            ok("explicit recorder stays bound across two effects without a context ACK handshake")

            js = call(
                "run_script",
                {
                    "project": RUNTIME_PROJECT,
                    "language": "javascript",
                    "script": (
                        "import { readFileSync } from 'node:fs';\n"
                        "if (!readFileSync('README.md', 'utf8').includes('state: original')) "
                        "throw new Error('wrong read snapshot');\n"
                        "console.log('js-typed-script-read-only-ok');\n"
                    ),
                    "timeout_secs": 20,
                    "sync_wait_secs": 20,
                    "purpose": "diagnostic",
                },
            )
            require(
                "js-typed-script-read-only-ok" in str(js.get("stdout_tail", ""))
                and "job_id" not in js
                and (project / "README.md").read_text(encoding="utf-8") == original,
                f"advertised JavaScript execution failed or modified source: {bounded(js)}",
            )
            ok("advertised JavaScript executes as a read-only typed script")

            shell_change = call(
                "run_shell",
                {
                    "project": RUNTIME_PROJECT,
                    "command": "printf 'later shell edit\\n' > new.txt",
                    "shell": "sh",
                    "timeout_secs": 20,
                    "purpose": "operation",
                },
            )
            require(
                (project / "new.txt").read_text(encoding="utf-8") == "later shell edit\n",
                f"recorded shell did not change an initially-clean file: {bounded(shell_change)}",
            )
            ok("recorded shell changed a Git-clean path after Session startup")

            try:
                pytest_ready = call(
                    "run_process",
                    {
                        "project": RUNTIME_PROJECT,
                        "executable": "python3",
                        "args": ["-c", "import pytest; print('pytest-runner-ready')"],
                        "timeout_secs": 20,
                        "sync_wait_secs": 20,
                        "purpose": "diagnostic",
                    },
                )
            except Failure as exc:
                raise Failure(
                    "pytest is required in the Runner's python3 environment; "
                    f"install it before running this smoke: {exc}"
                ) from exc
            require(
                "pytest-runner-ready" in str(pytest_ready.get("stdout_tail", "")),
                f"pytest preflight did not execute: {bounded(pytest_ready)}",
            )
            ok("the Runner Python environment has pytest installed")

            read = call(
                "read_files",
                {"project": RUNTIME_PROJECT, "items": [{"path": "README.md"}]},
            )
            item = read.get("items", [{}])[0]
            revision = item.get("output", {}).get("read_revision")
            require(
                item.get("output", {}).get("text") == original and isinstance(revision, int),
                f"read_files contract mismatch: {bounded(read)}",
            )
            pytest_assertion = "isolated README pytest guarded edit validation"
            pytest_request = {
                "project": RUNTIME_PROJECT,
                "executable": "python3",
                "args": [
                    "-B", "-m", "pytest", "-q", "-p", "no:cacheprovider",
                    "tests/test_workflow.py",
                ],
                "timeout_secs": 35,
                "sync_wait_secs": 35,
                "purpose": "test",
                "assertion_name": pytest_assertion,
            }
            pytest_before = call_failure("run_process", pytest_request)
            require(
                pytest_before.get("exit_code") == 1
                and pytest_before.get("command_completed") is True
                and "1 failed" in str(pytest_before.get("stdout_tail", "")),
                f"the real pytest fixture did not fail before the fix: {bounded(pytest_before)}",
            )
            ok("recorded pytest fails against the initial README snapshot")
            edit = call(
                "apply_text_edits",
                {
                    "project": RUNTIME_PROJECT,
                    "changes": [
                        {
                            "kind": "edit",
                            "path": "README.md",
                            "expected_read_revision": revision,
                            "edits": [
                                {
                                    "kind": "replace_exact",
                                    "old_text": original,
                                    "new_text": edited,
                                }
                            ],
                        }
                    ],
                },
            )
            require(
                edit.get("state_changed") is True
                and edit.get("execution_state") == "completed"
                and (project / "README.md").read_text(encoding="utf-8") == edited,
                f"guarded edit failed: {bounded(edit)}",
            )
            ok("apply_text_edits changed README.md under its read_revision guard")

            stale = call_failure(
                "apply_text_edits",
                {
                    "project": RUNTIME_PROJECT,
                    "changes": [{
                        "kind": "edit",
                        "path": "README.md",
                        "expected_read_revision": revision,
                        "edits": [{
                            "kind": "replace_exact",
                            "old_text": edited,
                            "new_text": "state: stale-guard-must-not-write",
                        }],
                    }],
                },
            )
            require(
                stale.get("state_changed") is False
                and stale.get("error_kind") == "stale_file_revision"
                and (project / "README.md").read_text(encoding="utf-8") == edited,
                f"stale read_revision did not reject transactionally: {bounded(stale)}",
            )
            ok("stale read_revision rejects the edit without changing any file")

            pytest_after = call("run_process", pytest_request)
            require(
                pytest_after.get("exit_code") in (None, 0)
                and "1 passed" in str(pytest_after.get("stdout_tail", ""))
                and "job_id" not in pytest_after,
                f"the same recorded pytest assertion did not pass after the fix: {bounded(pytest_after)}",
            )
            ok("the same recorded pytest assertion passes after the guarded fix")

            async_pytest_assertion = "isolated README async pytest validation"
            async_pytest = call(
                "run_process",
                {
                    "project": RUNTIME_PROJECT,
                    "session_id": recording_session_id,
                    "executable": "python3",
                    "args": [
                        "-B", "-m", "pytest", "-q", "-p", "no:cacheprovider",
                        "tests/test_async_workflow.py",
                    ],
                    "timeout_secs": 35,
                    "sync_wait_secs": 1,
                    "purpose": "test",
                    "assertion_name": async_pytest_assertion,
                },
            )
            async_job_id = async_pytest.get("job_id")
            require(
                async_pytest.get("command_started") is True
                and async_pytest.get("command_completed") is False
                and async_pytest.get("execution_source") == "run_process"
                and isinstance(async_job_id, str) and async_job_id,
                f"typed pytest did not hand off the original execution: {bounded(async_pytest)}",
            )
            async_baseline = call(
                "observe_jobs", {"items": [{"job_id": async_job_id}], "tail_lines": 40}
            )
            async_first = async_baseline.get("items", [{}])[0]
            require(
                async_first.get("job_id") == async_job_id
                and isinstance(async_first.get("observation_token"), str),
                f"typed pytest Job lacked a resumable observation: {bounded(async_baseline)}",
            )
            async_observed = call(
                "observe_jobs",
                {
                    "items": [{
                        "job_id": async_job_id,
                        "after_observation_token": async_first["observation_token"],
                    }],
                    "tail_lines": 40,
                    "wait_secs": 20,
                    "wake_on": "terminal",
                },
            )
            async_last = async_observed.get("items", [{}])[0]
            detected = async_last.get("detected_summary", {})
            require(
                async_last.get("job_id") == async_job_id
                and async_last.get("terminal") is True
                and async_last.get("status") == "completed"
                and async_last.get("exit_code") == 0
                and detected.get("tests_detected") is True
                and detected.get("tests_run_count") == 1
                and detected.get("tests_passed") == 1
                and detected.get("tests_failed") == 0
                and detected.get("zero_tests_run") is False,
                f"typed pytest Job did not reach counted terminal evidence: {bounded(async_observed)}",
            )
            ok("typed run_process pytest handed off once and terminal Job counts are real")

            validation = call(
                "run_process",
                {
                    "project": RUNTIME_PROJECT,
                    "executable": "python3",
                    "args": [
                        "-c",
                        "from pathlib import Path; assert 'state: edited-and-validated' "
                        "in Path('README.md').read_text(); print('sync-validation-ok')",
                    ],
                    "timeout_secs": 20,
                    "sync_wait_secs": 20,
                    "purpose": "test",
                    "assertion_name": "isolated README guarded edit validation",
                },
            )
            require(
                "sync-validation-ok" in str(validation.get("stdout_tail", ""))
                and "job_id" not in validation,
                f"run_process validation failed: {bounded(validation)}",
            )
            ok("run_process performed an actual successful validation command")

            job = call(
                "run_job",
                {
                    "project": RUNTIME_PROJECT,
                    "command": "printf 'async-log-start\\n'; sleep 1; "
                    "printf 'async-log-terminal\\n'",
                    "timeout_secs": 20,
                    "purpose": "operation",
                    "shell": "sh",
                },
            )
            job_id = job.get("job_id")
            require(isinstance(job_id, str) and job_id, f"run_job failed: {bounded(job)}")
            baseline = call(
                "observe_jobs", {"items": [{"job_id": job_id}], "tail_lines": 40}
            )
            first = baseline.get("items", [{}])[0]
            require(
                "output" not in first
                and first.get("job_id") == job_id
                and isinstance(first.get("observation_token"), str),
                f"stale observe_jobs projection: {bounded(baseline)}",
            )
            terminal = call(
                "observe_jobs",
                {
                    "items": [
                        {
                            "job_id": job_id,
                            "after_observation_token": first["observation_token"],
                        }
                    ],
                    "tail_lines": 40,
                    "wait_secs": 15,
                    "wake_on": "terminal",
                },
            )
            last = terminal.get("items", [{}])[0]
            output = str(first.get("stdout_tail", "")) + str(last.get("stdout_tail", ""))
            require(
                last.get("terminal") is True
                and last.get("status") == "completed"
                and last.get("exit_code") in (None, 0)
                and "async-log-start" in output
                and "async-log-terminal" in output,
                f"async Job lifecycle failed: {bounded([baseline, terminal])}",
            )
            ok("run_job reached terminal state and observe_jobs returned both log markers")

            changes = call("show_changes", {"project": RUNTIME_PROJECT})
            require(
                changes.get("clean") is False and "README.md" in json.dumps(changes),
                f"show_changes missed the edit: {bounded(changes)}",
            )
            ok("show_changes reports the guarded disposable repository edit")
            finish = call(
                "finish_coding_task",
                {
                    "project": RUNTIME_PROJECT,
                    "session_id": recording_session_id,
                    "include_diff": False,
                    "include_handoff": False,
                    "summary_only": False,
                },
            )
            validation_result = finish.get("validation", {})
            observations = finish.get("workspace_observations", {})
            pre_existing = {
                item.get("path") for item in observations.get("pre_existing_dirty", [])
            }
            newly_dirty = {
                item.get("path") for item in observations.get("newly_dirty", [])
            }
            overlapping = {
                item.get("path") for item in observations.get("overlapping_dirty", [])
            }
            require(
                observations.get("status") == "comparable"
                and observations.get("startup", {}).get("complete") is True
                and observations.get("finish", {}).get("complete") is True
                and "existing.txt" in pre_existing
                and "existing.txt" in overlapping
                and {"README.md", "new.txt"}.issubset(newly_dirty)
                and observations.get("overlap_content_unknown") is True,
                f"finish lost startup/shell path comparison: {bounded(observations)}",
            )
            ok("finish distinguishes startup dirty overlap from shell/newly dirty paths")
            pytest_events = [
                event
                for event in validation_result.get("events", [])
                if event.get("assertion_name") == pytest_assertion
            ]
            require(
                len(pytest_events) == 2,
                f"finish omitted the recorded pytest attempts: {bounded(validation_result)}",
            )
            pytest_by_success = {event.get("success"): event for event in pytest_events}
            failed_event = pytest_by_success.get(False, {})
            passed_event = pytest_by_success.get(True, {})
            count_evidence = [
                {key: event.get(key) for key in (
                    "success", "validation_passed", "tests_detected", "tests_run_count",
                    "tests_passed", "tests_failed", "zero_tests_run", "stdout_truncated",
                )}
                for event in pytest_events
            ]
            sync_pass_evidence = {
                key: pytest_after.get(key) for key in (
                    "execution_source", "purpose", "execution_state", "exit_code",
                    "stdout_truncated", "stderr_truncated", "process_summary",
                    "command_summary", "stdout_tail", "stderr_tail", "detected_summary",
                    "tests_detected", "tests_run_count", "tests_passed", "tests_failed",
                )
            }
            require(
                len(pytest_by_success) == 2
                and failed_event.get("tests_detected") is True
                and failed_event.get("tests_run_count") == 1
                and failed_event.get("tests_passed") == 0
                and failed_event.get("tests_failed") == 1
                and failed_event.get("zero_tests_run") is False
                and passed_event.get("tests_detected") is True
                and passed_event.get("tests_run_count") == 1
                and passed_event.get("tests_passed") == 1
                and passed_event.get("tests_failed") == 0
                and passed_event.get("zero_tests_run") is False,
                f"recorded real pytest counts are missing or fabricated: {bounded(count_evidence)}; synchronous pass: {bounded(sync_pass_evidence)}",
            )
            ok("full Session evidence detects actual pytest fail/pass terminal counts")
            async_events = [
                event
                for event in validation_result.get("events", [])
                if event.get("assertion_name") == async_pytest_assertion
                and event.get("tool_name") == "run_process"
            ]
            validation_event_ids = [
                {key: event.get(key) for key in (
                    "tool_name", "assertion_name", "command_summary", "execution_state",
                    "success", "validation_passed", "tests_detected", "tests_run_count",
                    "tests_passed", "tests_failed", "zero_tests_run",
                )}
                for event in validation_result.get("events", [])
            ]
            require(
                len(async_events) == 1
                and async_events[0].get("success") is True
                and async_events[0].get("validation_passed") is True
                and async_events[0].get("exit_code") == 0
                and async_events[0].get("tests_detected") is True
                and async_events[0].get("tests_run_count") == 1
                and async_events[0].get("tests_passed") == 1
                and async_events[0].get("tests_failed") == 0
                and async_events[0].get("zero_tests_run") is False,
                f"async typed pytest evidence was not persisted at finish: {bounded(validation_event_ids, 4000)}",
            )
            ok("explicit Session finish persists the async typed pytest counts")
            require(
                validation_result.get("latest_status") == "passed"
                and validation_result.get("resolved_failures", {}).get("count") == 1
                and validation_result.get("unresolved_failures", {}).get("count") == 0
                and validation_result.get("evidence_gaps", {}).get("count") == 0
                and validation_result.get("current_evidence", {}).get("status") == "unproven"
                and validation_result.get("current_evidence", {}).get("reason")
                == "validation_source_unproven"
                and validation_result.get("current_evidence", {}).get("successes") == 0
                and validation_result.get("current_evidence", {}).get("latest_status")
                == "inconclusive"
                and all(event.get("source_state", {}).get("freshness") == "unproven"
                        for event in pytest_events + async_events)
                and finish.get("task_outcome", {}).get("blocking") is False
                and "validation_inconclusive" in finish.get("advisories", [])
                and finish.get("tool_failures", {}).get(
                    "actionable_unexpected_count"
                ) == 0,
                f"finish lost pytest history or changed the current-source boundary: {bounded(finish)}",
            )
            ok("finish preserves pytest fail/pass history without claiming current source proof")
            outcome = "passed"
            log(f"PASS: {len(checks)} checks in {time.monotonic() - started:.2f}s")
        except (Failure, OSError, subprocess.SubprocessError) as exc:
            error_text = str(exc)
            log(f"FAIL: {error_text}")
        finally:
            terminate_group(runner)
            terminate_group(server)
            if temp is not None:
                temp.cleanup()

    receipt: dict[str, Any] = {
        "schema_version": 1,
        "result": outcome,
        "server_version": SERVER_VERSION,
        "runtime_project_id": RUNTIME_PROJECT,
        "elapsed_seconds": round(time.monotonic() - started, 3),
        "timeout_seconds": args.timeout_secs,
        "binaries": binaries,
        "checks": checks,
        "logs": {
            "smoke": str(smoke_path),
            "server": str(server_path),
            "runner": str(runner_path),
        },
    }
    if error_text is not None:
        receipt["error"] = error_text[:1600]
    receipt_path.write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n")
    print(f"[e2e-web] receipt: {receipt_path}", flush=True)
    return 0 if outcome == "passed" else 1


if __name__ == "__main__":
    sys.exit(main())

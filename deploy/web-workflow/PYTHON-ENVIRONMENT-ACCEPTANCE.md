# CoordExp ms Python deployment acceptance — 2026-09-17

- Scope: Runner execution environment and MCP initialization guidance only.
  No global shell changes, dependency installation, research-file edits, or tests
  of research behavior; no Rust/runtime protocol changes.
- Change commit: `c3716dfe`. Fork branch: `coordexp/web-workflow`.
- Local deployment: `coordexp-2026.09.17.3`. Native binaries are byte-identical to
  `coordexp-2026.09.17.2`, with actual clean build `98d9301cba22` on Server/Runner.
  Deployment source and binary source are recorded separately in `RELEASE.json`.

## Evidence

- RED, actual pre-update Runner: `run_process(executable="python")` returned
  `spawn_failed`, `command_started=false`, `No such file or directory`.
- GREEN, actual post-update Runner: all four canonical projects resolve `python`
  to `/root/miniconda3/envs/ms/bin/python`; `sys.prefix` is the ms environment and
  pytest imports successfully (8.4.2). `python3` and `run_shell` also use ms.
- `run_process(executable="python", args=["-m", "pytest", "--version"])` succeeds.
  This is interpreter/tool availability, not a research test-suite claim.
- Persistent Git still wins PATH resolution. Server/Tunnel PATH is unchanged.
- MCP `initialize` and `server/discover` return the exact updated guidance,
  including the configured interpreter and literal-argv pytest usage.
- Four projects online; no active Jobs before restart or after acceptance.
- Stopped-state SQLite backup passed `quick_check`; existing `sessions.json`
  remained byte-identical. Old deployment/data were not removed.
- Tunnel local `/readyz` returns 200. Browser-side invocation is not claimed.
- `bash -n` and `git diff --check` passed.

Runnable acceptance and full receipt:
`/data/CoordExp/.local/webcodex-custom/verification/coordexp-2026.09.17.3/accept-python.py`
and `live.json` alongside it. Run with `python <acceptance-script>`.

## Recovery

Backup: `rollback/pre-coordexp-2026.09.17.3/` under the persistent deployment root,
including state, private config, project registry and lifecycle scripts.
After reconciling active Jobs, use `bin/control.sh stop`, `switch
coordexp-2026.09.17.2`, then `start`. The old release has no versioned launcher and
uses the unchanged retained `bin/service.sh`, restoring its former PATH. No
database restoration is needed for this environment-only rollback. A live
rollback cycle was not performed in this update.

The user-selected `/root/miniconda3/envs/ms` is a non-persistent prerequisite:
restore it after container recreation. Runner refuses startup when its Python
executables are missing; it does not silently substitute system Python.

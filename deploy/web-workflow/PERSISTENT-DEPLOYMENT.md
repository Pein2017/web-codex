# CoordExp container recovery

The source checkout is `/data/CoordExp/codex-tools/web-codex/`; its operator root
is `deployment/` and dedicated dependency packages live in `dependencies/`.
Only `/data` is assumed durable. Both runtime directories are root-anchored
Git ignores, including their private configuration and credentials.
This is a named Linux dogfood deployment, not an npm or GitHub release.

This complete Git checkout is the local fork, with its existing history and
`coordexp/web-workflow` branch preserved. `upstream` already points to
`https://github.com/yyjeqhc/webcodex.git`; the existing `origin` remains optional
for publication. Work on source in this checkout; no new GitHub repository or
push is part of relocation. Source and deployment-script edits do not require
rebuilding the native binaries when Rust source is unchanged.

## Layout and prerequisites

Paths below are relative to `deployment/` unless stated otherwise.

- `releases/<id>/`: immutable matching Server/Runner/CLI binaries, guidance, and the **complete**
  workflow Plugin bundle (`plugin.mjs`, `pytest_report.py`, package metadata).
- `current`: symlink selecting one release; state is never stored beneath it.
- `config/`: private Server env, Runner TOML and Tunnel control-plane credentials.
- `state/server/`: existing Server database, Workflow Sessions and private state;
  `state/project-registry/`: registered canonical worktrees.
- `state/xdg/{config,data,cache,state}/`: dedicated XDG roots, including CLI
  state at `state/xdg/state/webcodex/`. The relocation includes the former
  `/data/CoordExp/.local/state/webcodex/` and all other dedicated state.
- `bin/control.sh`, `bin/service.sh`: persistent startup/recovery entrypoints;
  `bin/webcodex`: CLI entrypoint selecting `current/bin/webcodex-cli` and
  explicitly binding all four XDG roots without loading private credentials.
- `logs/`, `verification/`, `rollback/`: private operational records and backups.
- `verification/historical-local/`: retained historical material from the old
  `.local` layout. Old receipts preserve the paths and identities observed at
  their original acceptance; those recorded paths are not current commands.
- `runtime/bin/`: persistent native CLI/Tunnel, a dedicated rg binary and a
  link to the relocated Node package; `runtime/git/`: a self-contained newer
  Git prefix. `../dependencies/` owns the dedicated Node package, CodeGraph
  package at `../dependencies/codegraph/`, and retained historical dependency
  material. The handwritten CodeGraph adapter and check script are tracked
  source at `../deploy/codegraph/`. Unused legacy npm
  and git-tools packages are historical material, not qualified active runtimes.

The standard-PATH `/data/CoordExp/.local/bin/webcodex` alias is only a symlink to
`deployment/bin/webcodex`; its executable and dedicated data live in this tree.
Shared system tools, the Conda environment, `/data/CoordExp/.codex` Skills and
memories, and canonical research/project checkouts remain external prerequisites.

The container image must provide Linux x86-64 with compatible glibc/libstdc++,
Bash, coreutils, tmux, Python >=3.10, system CA certificates and ordinary local
networking, and GitHub CLI (`gh`) for the existing Git credential helper.
Selected Git configuration and GitHub credentials are copied privately into
`config/`; service environment overrides keep them independent of `/root`.
Training environments/GPU drivers are independent prerequisites, not recreated by WebCodex startup.
The CoordExp Runner requires `/root/miniconda3/envs/ms/bin/python` and `python3`;
its children default to that environment without activation. Persistent operator
tools remain ahead of Conda on PATH; Server/Tunnel PATH is unchanged. The `ms`
environment is an explicit non-persistent container prerequisite: restore it after
container recreation before starting Runner. Missing interpreters stop Runner
startup rather than silently choosing system Python. Git must support `check-attr --source=HEAD`;
system Git 2.34 is insufficient. The Plugin JUnit parser needs only Python stdlib.

Restore the user-owned SSH/proxy forward at `127.0.0.1:9090` for external access.
Server/Runner can communicate locally without it; Tunnel reconnects when network
access returns. Neither a surviving tmux process nor local MCP health proves that
the externally hosted ChatGPT conversation can reach the Tunnel.

## Restore after container recreation

```bash
bash /data/CoordExp/codex-tools/web-codex/deployment/bin/control.sh start
bash /data/CoordExp/codex-tools/web-codex/deployment/bin/control.sh status
webcodex --help
```

The dedicated tmux socket lives in `state/tmux.sock`; this does not touch the
default tmux server or research sessions. `start` does not duplicate an existing
session. If an existing session is partially unhealthy, inspect its logs and live
Jobs before stopping/restarting; do not blindly launch a second runtime. Service
loops append private logs and retry unexpected exits. Restore network access,
then check the Tunnel ready event as well as authenticated Server/Runner status.
Credentials remain the existing private files; do not paste them into commands.
`WEBCODEX_DEPLOY_ROOT` can select an explicit alternate operator root. Each tmux
window receives that root and `XDG_STATE_HOME=<root>/state/xdg/state` in its launch
command, so a cached tmux environment cannot send state back to the former tree.
The source `service.sh` also exports and creates the XDG state root alongside
the config, data and cache roots. Existing immutable release launchers are
selected unchanged; the explicit environment binds their deployment/state paths.

## Controlled update and rollback

New deployment bundles include executable `service.sh` at their release root and
their matching CLI at `bin/webcodex-cli`.
`bin/control.sh` uses that versioned launcher when present; older releases use
the retained `bin/service.sh`. Thus switching back also restores the former
execution environment. Relocating the tree changes filesystem paths, not the
immutable release's build identity or historical acceptance. The current native
0.4.3 CLI and ms Runner environment remain selected. A configuration-only release
may reuse unchanged native binaries: record the deployment source commit separately from their actual build
commit, and verify hashes instead of claiming the binaries were rebuilt.

First inspect live Jobs and in-flight work through the authenticated runtime.
Do not stop active research or execution just to switch a release. Save a
consistent stopped-state backup of `state/server`, `config` and the project
registry, preserving permissions. Keep the previous immutable release untouched.

```bash
bash /data/CoordExp/codex-tools/web-codex/deployment/bin/control.sh stop
# Verify exact old Server/Runner/CLI/tunnel-client processes have exited.
bash /data/CoordExp/codex-tools/web-codex/deployment/bin/control.sh switch <existing-release>
bash /data/CoordExp/codex-tools/web-codex/deployment/bin/control.sh start
```

Switching is refused while the dedicated tmux session exists and never changes
persistent state. Use the same sequence for the recorded rollback release. Do not
restore a database backup over newer accepted work; restoring state is a separate
explicit recovery decision. Validate matching Server/Runner build identities,
existing Session records, canonical projects, Skills, Plugin describe/call,
pytest report parsing and Tunnel connectivity. Inspect permissions and remove no
old data merely because the new process started.

The migration receipt names the exact source/build, dependency checksums, backup
and tested rollback target. A rollback package may repair an omitted packaging
file without modifying the original immutable release; its identity and
provenance must be recorded separately.

The [ms Python acceptance](PYTHON-ENVIRONMENT-ACCEPTANCE.md) and OpenSpec
acceptance records are sealed historical evidence and retain their original
path references. The former deployment's `verification/<id>/` and
`rollback/<id>/` material now resides beneath the same relative paths in
`/data/CoordExp/codex-tools/web-codex/deployment/`; other retained old-layout
material is under `verification/historical-local/`. Consult the relocation
receipt for exact retained locators instead of rewriting historical receipts.

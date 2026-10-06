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

## Local tool surface and upstream updates

The local source uses the existing `ToolDefinition` visibility contract, not a
second registry or a runtime profile. The model uses
`rotate_agent_continuation_endpoint` and `project_artifact` with its
`metadata`, `inspect`, `image` and `export` actions. The duplicate
`attach_agent_endpoint`, `read_project_artifact_metadata`,
`read_project_artifact`, and four `artifact_upload_*` primitives are
`ModelHidden`: they remain available to authorized typed/internal consumers,
but are not discoverable or callable through the model's MCP/GPT Actions
gateway. Host attachment import and project-to-project transfer retain their
internal upload protocol, authorization and content fences.

This intentionally narrows the local model interface; it does not change the
Server/Runner wire protocol, durable replay identities or persisted state.
With the same features and protocol extensions as the original deployment,
source discovery has seven fewer tools (151 to 144). This is a source change:
the installed `app/` binaries change only through an explicitly authorized
build and deployment. Refresh the Host connection's metadata after deployment
and verify an actual gateway call in a new conversation.

Upstream synchronization is explicit operator work. When authorized, fetch
`upstream`, inspect the selected revision and divergence, and merge reviewed
upstream changes into `coordexp/web-workflow` without overwriting local work or
rewriting its history. Recheck the local visibility declarations, discovery
catalog and recovery suggestions alongside the affected upstream contracts;
then run focused model-admission and internal artifact-transfer/import checks.
Keep source integration separate from deployment, with matching Server/Runner
builds and rollback under the controlled-update procedure below. An existing
remote-tracking ref is only the last fetched snapshot, not proof that this
checkout matches current upstream. No automatic merge, deployment or push is
performed, and conflict-free future updates are not guaranteed.

## Layout and prerequisites

Paths below are relative to `deployment/` unless stated otherwise.

- `app/`: the single installed application directory, with matching native
  Server/Runner/CLI binaries in `bin/`, guidance in `AGENTS.md`, and the complete
  workflow Plugin bundle (`plugin.mjs`, `pytest_report.py`, package metadata).
  The directory contains the retained application bytes; moving them does not
  change their build identity. Historical release directories remain evidence.
- `config/`: private Server env, Runner TOML and Tunnel control-plane credentials.
  `config/regular-tunnel-runtime/` holds the native CLI's fixed-path Tunnel
  runtime state as a real directory.
- `state/server/`: existing Server database, Workflow Sessions and private state;
  `state/project-registry/`: registered canonical worktrees.
- `state/xdg/{config,data,cache,state}/`: dedicated XDG roots, including CLI
  state at `state/xdg/state/webcodex/`. The relocation includes the former
  `/data/CoordExp/.local/state/webcodex/` and all other dedicated state.
- `bin/control.sh`, `bin/service.sh`: persistent startup/recovery entrypoints;
  `bin/webcodex`: CLI entrypoint executing `app/bin/webcodex-cli` and
  explicitly binding all four XDG roots without loading private credentials.
- `logs/`, `verification/`, `rollback/`: private operational records and backups.
  `rollback/simplify-install-20261004/retired-packages/` is the recoverable
  holding area for removed legacy packages and the unused old CLI; none is
  an active dependency. These bytes have not been permanently purged.
- `verification/historical-local/`: retained historical material from the old
  `.local` layout. Old receipts preserve the paths and identities observed at
  their original acceptance; those recorded paths are not current commands.
- `runtime/bin/`: dedicated Tunnel client and rg binaries; `runtime/git/`: a
  self-contained newer Git prefix. `../dependencies/codegraph/` owns CodeGraph
  and its packaged Node executable at
  `node_modules/@colbymchenry/codegraph-linux-x64/node`. Service PATH starts with
  that package directory, then `runtime/bin` and `runtime/git/bin`; Runner adds
  the selected ms Conda environment after those operator tools. The handwritten
  CodeGraph adapter and check script are tracked source at `../deploy/codegraph/`.

Use `/data/CoordExp/codex-tools/web-codex/deployment/bin/webcodex` directly, or
explicitly add `deployment/bin` to your own PATH. Application entrypoints are
real files in this tree; ordinary system and dependency-package links retain
their standard roles.
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
tools remain ahead of Conda on PATH; Server/Tunnel use the operator PATH without
Conda. The `ms`
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
/data/CoordExp/codex-tools/web-codex/deployment/bin/webcodex --help
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
`control.sh` always invokes `bin/service.sh`. That script exports and creates
all four XDG roots, executes the binaries under `app/bin`, and points Server
guidance at `app/AGENTS.md`. Startup requires the installed application and
complete Plugin bundle, dedicated Node, Git, rg, Tunnel client and private
configuration before it creates service windows.

## Direct workflow operation

This workflow runs ChatGPT → WebCodex Server → Runner. Use the selected Project's
admitted file/Git tools, validation operations and bounded `run_process`; use a Job
for asynchronous work and observe or continue that same Job rather than redispatching
when its result is unknown. Keep Project/policy authorization, source
`read_revision` guards and Session/`recording_session_id` binding in force. A command
exit status reports execution only; task completion requires the relevant consumer
to accept its output or artifact against the declared criteria. This workflow does
not launch a local Codex CLI agent or run a sandbox readiness probe.

## Controlled update and recovery

`control.sh` supports only `start`, `stop` and `status`. Installation changes
are explicit operator work on the real `app/` directory. Preserve the native
0.4.3 build identity and selected ms Runner environment unless the authorized
update changes them. An entrypoint-only update can reuse unchanged application
bytes; record source and binary build identities separately.

First inspect live Jobs and in-flight work through the authenticated runtime.
Do not stop active research or execution merely to update the installation. Save a
consistent stopped-state backup of `state/server`, `config` and the project
registry, preserving permissions. Retain historical application bundles and
backups for explicit recovery.

```bash
bash /data/CoordExp/codex-tools/web-codex/deployment/bin/control.sh stop
# Verify exact Server/Runner/CLI/tunnel-client processes have exited.
# Perform the explicitly authorized installation update or backup recovery.
bash /data/CoordExp/codex-tools/web-codex/deployment/bin/control.sh start
```

Recovery uses a specifically selected retained backup or historical application
bundle after services stop and process exit is verified. State restoration is a
separate explicit decision; startup never restores a database automatically.
Do not overwrite newer accepted work with an older database. Validate matching Server/Runner build identities,
existing Session records, canonical projects, Skills, Plugin describe/call,
pytest report parsing and Tunnel connectivity. Inspect permissions and remove no
old data merely because the new process started.

The [clean-install acceptance](CLEAN-INSTALL-ACCEPTANCE.md) owns the current
single-directory installation and its recovery locators.
The [relocation acceptance](RELOCATION-ACCEPTANCE.md),
[ms Python acceptance](PYTHON-ENVIRONMENT-ACCEPTANCE.md) and OpenSpec
acceptance records are sealed historical evidence and retain their original
path references. The former deployment's `verification/<id>/` and
`rollback/<id>/` material now resides beneath the same relative paths in
`/data/CoordExp/codex-tools/web-codex/deployment/`; other retained old-layout
material is under `verification/historical-local/`. Consult the current
clean-install receipt for operational paths instead of rewriting historical receipts.

# WebCodex local-fork relocation — 2026-10-04

Status: lead-accepted. User authorized relocation of the complete WebCodex
installation and dedicated dependencies/state, service downtime, and the shared
CodeGraph daemon restart required to release the old source index. No new GitHub
repository, push, publication, upstream upgrade, or native rebuild was performed.

## Current owners

- Canonical Git checkout: `/data/CoordExp/codex-tools/web-codex`.
- Existing local branch: `coordexp/web-workflow`; original source HEAD:
  `2da7c40b93b32f24190f7981fc03b36ba3e59464`. Existing history and remotes retained.
- Active installation: `deployment/`; selected immutable bundle:
  `coordexp-2026.09.30.1`, native version `0.4.3`, build commit `004e108a7e5e`,
  `git_dirty=false`. The local relocation commit is separate from this unchanged
  native build identity.
- Dedicated packages: `dependencies/codegraph`, `dependencies/legacy-cli`, and
  `dependencies/git-tools`. The latter two retain unused historical packages;
  active Git is `deployment/runtime/git`, not the old Git-tools environment.
- Maintained CodeGraph adapter: `deploy/codegraph/`, tracked with this checkout.
- All dedicated XDG roots: `deployment/state/xdg/{config,data,cache,state}`;
  migrated Runner/CLI state: `state/xdg/state/webcodex`.
- Historical local payloads: `deployment/verification/historical-local/`.
- The ordinary PATH alias `.local/bin/webcodex` points only to
  `deployment/bin/webcodex`; no installation or dedicated state remains there.

Shared system libraries/tools, Conda `ms`, `.codex` Skills/memories, research
checkouts, and external reference source trees remain external prerequisites.
Research Projects and managed research worktrees were not relocated or retargeted.

## Fresh evidence

Private evidence owner:
`deployment/verification/migrate-codex-tools-20261004/`.

- Twelve directories moved by same-filesystem rename; device/inode identities
  checked against the pre-move baseline. All twelve old directory paths absent.
- Stopped-state Server/registry/Runner state matched byte-for-byte immediately
  after relocation. After startup, canonical Session content still matched:
  74 Sessions, 4,248 events, 3 messages. Credentials unchanged; only the three
  path-bearing private configuration files changed.
- Current immutable release files unchanged, including its launcher. The
  operator launcher explicitly passes deployment and XDG state roots into every
  tmux window; native processes were observed using the new roots.
- Focused entrypoint RED: original scripts failed 4/6 checks. GREEN: all seven
  `test_relocation.py` checks passed, including explicit tmux roots, lifecycle
  invariants, fallback launchers, and CLI argument/XDG propagation.
- Actual Server/Runner: online, exact build/source alignment, compatible protocol,
  five original canonical Project IDs/paths, four ready Plugins, zero active Jobs.
- Actual Runner process: source checkout as cwd, Conda `ms` Python, relocated Git
  2.55.0, and successful `check-attr --source=HEAD`.
- Actual Plugin describe/call: CodeGraph status reports the relocated checkout
  and index, no worktree mismatch. Existing index preserved; pending new source
  files are not claimed fully reindexed. External-source admission checker passed
  its allowed-root and rejected-parent/subdirectory cases.
- Installed Git helper text prefixes relocated. `shasum --version` failed first
  because of its removed interpreter prefix, then passed after the mechanical
  prefix correction. No native executable or immutable release was rebuilt.
- Shared CodeGraph daemon gracefully replaced; native desktop graph query against
  the new source path succeeded. The existing desktop proxy may use its native
  in-process fallback; this does not claim attachment to the replacement daemon.
- Tunnel reports `tunnel_ready=true` and `local_mcp_ready=true`; an actual connected
  ChatGPT app `runtime_status` call returned the aligned online build after moving.
- Shell/Node syntax and Git whitespace checks passed. No whole Rust suite was
  repeated because the native runtime/source implementation did not change.

The Server log also contains a pre-existing `terminal Job attention persistence
degraded count=3` warning, observed since 2026-10-03 before relocation. This task
preserves its state and does not claim to repair that unrelated condition.

## Recovery

Consistent pre-migration archives, with private permissions:

- `pre-migration-state-config.tar`:
  `745a46afeb29e00a328e110e1a7ceb29d6022189f1ee045e2e478119b2380c5c`.
- `pre-migration-runner-state.tar`:
  `0f2928ef1d3a053db2607a304f2e23930fd6703f693a1978a4f030accfca4ba6`.

Both are in the private evidence owner above. Existing immutable releases and
older rollback archives also remain in `deployment/`. Historical receipts retain
their original paths and identities. Restoring old databases over newer state is
a separate authorized recovery action, never an automatic rollback step.

For operations and future source maintenance, see
[PERSISTENT-DEPLOYMENT.md](PERSISTENT-DEPLOYMENT.md).

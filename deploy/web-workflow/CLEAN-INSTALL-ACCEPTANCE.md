# WebCodex single-directory installation — 2026-10-04

Status: lead-accepted. The user authorized removal of backward-compatible
installation paths and application-level links, while allowing ordinary system
and dependency-package links. Scope is the local installation, not removal of
upstream runtime/API compatibility contracts.

## Current installation

- Source/local fork: `/data/CoordExp/codex-tools/web-codex`, existing
  `coordexp/web-workflow` branch and remotes. Pre-change source: `caa80a9d`.
- Application: real `deployment/app/`, moved intact from the formerly selected
  `coordexp-2026.09.30.1` bundle. Build remains clean `0.4.3 / 004e108a7e5e`;
  application files are unchanged and were not rebuilt.
- Sole maintained launcher: `deployment/bin/service.sh`. The immutable bundle's
  packaged launcher is inert provenance, not an operator entrypoint or fallback.
- Lifecycle: `deployment/bin/control.sh {start|stop|status}`. No `switch`,
  version-selecting link, old-release launcher fallback, or old-path fallback.
- CLI: invoke `deployment/bin/webcodex` directly. The former `.local/bin/webcodex`
  alias is removed; no shell PATH/profile configuration was changed.
- Node: actual CodeGraph package directory on service PATH, without the custom
  `runtime/bin/node` link. Normal package-owned npm/Git/shared-library links remain.
- Tunnel: real `deployment/config/regular-tunnel-runtime/`, moved intact from
  `state/tunnel`. This is the native CLI's existing env-file-parent path; no
  credential copy, native configuration extension, or compatibility link added.
- Old `dependencies/legacy-cli`, `dependencies/git-tools`, and unused
  `runtime/bin/webcodex-cli` are absent from active installation paths. They were
  moved into `deployment/rollback/simplify-install-20261004/retired-packages/`.
  The execution environment rejected forced deletion, so retirement is recoverable
  and has not reclaimed their disk space (approximately 335 MB for the two packages).

## Fresh acceptance

Private evidence: `deployment/verification/simplify-install-20261004/`.

- Verified zero active Jobs and actual old native/plugin process exit before
  admitted stopped-state backup. The first immediate exit check observed the
  Runner's shutdown grace; the deadline-bound follow-up confirmed exit. Preliminary
  snapshot evidence was not used in place of the fresh stopped-state capture.
- Before restart: Server, registry, Runner and Tunnel state byte-identical;
  credentials unchanged. Only the three Plugin application paths in private
  Runner TOML changed.
- After restart: canonical Session content unchanged, 74 Sessions / 4,248 events /
  3 messages; all five original Project IDs and paths unchanged; four ready Plugins.
- Installed source scripts: frozen contract RED failed before implementation;
  GREEN all nine focused checks passed. Unknown `switch` exits 2; missing `app`
  cannot start from a retained old `current` fixture. Shell/whitespace checks passed.
- Actual Server/Runner/CLI executables under real `app/bin`; four XDG roots remain
  bound to the dedicated installation. No active application-layer symlinks.
- Actual Runner consumer: source cwd, Conda `ms` Python, relocated Git 2.55.0 with
  working `check-attr --source=HEAD`, actual packaged Node v24.16.0, and correct
  dedicated XDG state. Actual CodeGraph Plugin describe/call succeeded at the
  exact source checkout with no worktree mismatch.
- Tunnel reports `tunnel_ready=true`, `local_mcp_ready=true`; a real connected
  ChatGPT app `runtime_status` call confirms aligned online 0.4.3 Server/Runner,
  compatible protocol, five Projects and zero active Jobs.
- No native rebuild, upstream synchronization, GitHub repository creation, push,
  publication, research-worktree change, or credential/Session reset.

## Recovery

Fresh consistent `pre-clean-state-config.tar` in the private evidence directory:
`3a6a011050b0e8ed687b3e7294f1e46eb42ae14fd7b7b1898f7983b83466a470`.
Private permissions preserved. Existing historical releases, receipts and backups
retain their bytes and original recorded paths; they are not current launch paths.
Recover only from an explicitly selected backup with services stopped. Database
restoration over newer accepted state requires its own authorization.

See [PERSISTENT-DEPLOYMENT.md](PERSISTENT-DEPLOYMENT.md) for current commands.

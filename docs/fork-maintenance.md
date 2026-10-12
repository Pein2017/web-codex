# Maintained WebCodex overlay

## Ownership and names

- Repository: <https://github.com/Pein2017/web-codex>, parent
  <https://github.com/yyjeqhc/webcodex>.
- Canonical local source: `/data/CoordExp/codex-tools/web-codex`.
- Since 2026-10-12 the maintained branch is a **zero-fork overlay**: unmodified
  upstream source plus additive CoordExp directories. The Rust crates, npm
  packages, binaries, authentication audiences and runtime state are upstream's.
- Overlay-owned paths: `deploy/web-workflow/`, `deploy/codegraph/`,
  `plugins/web-workflow/`, `openspec/`, this file, and the two root-anchored
  Git ignores `/deployment/` and `/dependencies/`. Nothing else is modified.
- The former fork (upstream v0.4.3 plus 59 local commits, tip `8b8321fc`) is
  retained unchanged on branch `history/fork-0.4.3-main`. Its
  `workspace_baseline` startup observations, durable Job output archive, pushed
  MCP initialization instructions and artifact-tool consolidation were retired
  with the overlay; see
  `openspec/changes/archive/2026-10-12-zero-fork-upstream-v05/` for the record.

## Remotes

Use `origin` for this repository and `upstream` for the original author, with
upstream pushes disabled:

```bash
git remote add upstream https://github.com/yyjeqhc/webcodex.git
git remote set-url --push upstream no_push://yyjeqhc/webcodex
git config remote.pushDefault origin
```

## Updating upstream

Upstream changes are never merged automatically. The overlay has no source
conflicts by construction, so an update is a fast-forward of the upstream base
followed by a rebuild:

```bash
git fetch --no-tags upstream main
git log --oneline HEAD..upstream/main          # review what changes
git rebase upstream/main                       # overlay commits replay cleanly
```

Before rebuilding, check the items the overlay depends on:

- `docs/RUNNER.md` `[instructions].files`, `[[mcp.providers]]`,
  `[[plugins.providers]]` and `[skills] roots` still exist with the same keys;
- `crates/webcodex-tool-contracts/src/tests/fixtures/v05_tool_renames.json` and
  `src/mcp/discovery.rs` for tool renames that `deploy/web-workflow/AGENTS.md`
  must follow (retired names fail closed; the ChatGPT connector must rescan
  tools after a surface change);
- `crates/webcodex-workflow-session/src/model.rs` `PersistedSessionRecord` for
  persisted Session fields, because unknown fields make a row silently skipped;
- the plugin protocol version expected by `plugins/web-workflow/plugin.mjs`.

Then follow the controlled update in
[container recovery](../deploy/web-workflow/PERSISTENT-DEPLOYMENT.md): stop,
back up, build with the current stable Rust toolchain
(`cargo +<stable> build --locked --profile dogfood -p webcodex -p webcodex-cli
-p webcodex-runner --features
'webcodex/workspace-checkpoints,webcodex-runner/workspace-checkpoints'`),
install, record the exact installed binary identity in `app/RELEASE.json`,
start and smoke. Never rebase or force-push published `main`; publish a rebased
overlay only under an explicit request naming the operation.

## Runtime boundary

The named local deployment is owned by
[container recovery](../deploy/web-workflow/PERSISTENT-DEPLOYMENT.md). Source
updates alone do not restart native processes. Keep databases, credentials,
sessions, binaries and historical evidence out of source changes; rebuild or
deploy only through the controlled-update procedure with rollback preserved.

## Why

The CoordExp fork (upstream v0.4.3 plus 59 local commits) diverged from upstream by 308 commits, and upstream #836 renamed 84 model-facing tools atomically, so no further cherry-pick is possible. Every local capability except two has an equivalent outside the binaries in upstream v0.5, and the deployed Tunnel watchdog restarted the Tunnel about 240 times a day. Run unmodified upstream and keep CoordExp specifics as an additive overlay.

## What Changes

- Replace the fork branch with an overlay branch based on upstream `dd57bb6f` (v0.5.0 development tip): unmodified source plus `deploy/web-workflow/`, `deploy/codegraph/`, `plugins/web-workflow/`, `openspec/`, `docs/fork-maintenance.md` and two root-anchored ignores. Retain the fork history on `history/fork-0.4.3-main`.
- **BREAKING**: retire the fork-only `WEBCODEX_MCP_INSTRUCTIONS_FILE` push (replaced by Runner `[instructions].files`, pulled by the model through `project.instructions`), `workspace_baseline` startup observations, the durable Job output archive, the artifact-tool consolidation, fork pytest counting (replaced by `project_validate`) and the Python Tunnel watchdog (replaced by upstream's native Tunnel client).
- Migrate the 100 persisted Workflow Sessions by removing the retired `workspace_baseline` field so upstream's strict record decoder keeps them.
- Rewrite the Runner-global instruction file for the v0.5 tool names and the real environment (Jobs, GPU work in tmux, `ms` Python, shared memory via `mcp_tool`, no host-file import).
- Rebuild with stable Rust, install as a named release with honest build identity, start, smoke, and leave the ChatGPT connector rescan to the user.

## Capabilities

### Modified Capabilities

- `web-workflow`: upstream-owned runtime and tool surface; CoordExp owns only configuration, plugin, instructions and deployment.

## Impact

Deployment `deployment/` (binaries, config, sessions), repository branch layout, model-visible tool names. No upstream source change. Not pushed to `origin` by this change.

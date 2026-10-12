## Why

Two real Web research traces show avoidable retries for a project alias, zero-progress batch reads, irrelevant CodeGraph results, and repeated ad-hoc Codex history parsing. The user also wants `include_project_instructions` to omit `CLAUDE.md` without disabling project `AGENTS.md`.

## What Changes

- Canonicalize an already authorized project alias before constructing execution/recovery identity, retaining all Runner and Session fences.
- Make bounded file/search results distinguish unfinished work and zero progress; a single oversized file must not suppress independent small files.
- Reuse the existing scoped CodeGraph query for known paths/symbols, expose actionable coverage/freshness limitations, and improve discovery guidance rather than build a new search engine.
- Add bounded, read-only public Codex-message retrieval to the existing web-workflow plugin, with an operator-configured history root, explicit thread selection and pagination.
- Add operator-controlled instruction-source exclusion for `CLAUDE.md`; apply it consistently to startup and on-demand project guidance. Preserve upstream defaults when unset and retain ordinary explicit file reads.
- Clarify canonical project IDs, recorder versus business Session IDs, explicit context ACK, partial-result continuation, and host safety denials in Web guidance.

## Capabilities

### New Capabilities

- `web-research-tool-path`: Reliable bounded research-tool routing, evidence retrieval and project-instruction selection.

### Modified Capabilities

None. Existing changes remain unarchived; no existing main spec is replaced.

## Impact

Targets the existing fork checkout, principally `src/tool_runtime`, affected tool contracts/tests, `crates/webcodex-core/src/project_instructions.rs`, `plugins/web-workflow`, and `deploy/web-workflow` guidance. No new service, database, orchestration layer or dependency is intended.

Planning and subsequent local implementation are separate workflow phases. Intended implementation owner: one Luna/max agent; lead owns acceptance. Push, production deployment/restart, automatic reindexing, checkpoint feature activation, research worktree edits and GPU work are outside this change's current execution authority.

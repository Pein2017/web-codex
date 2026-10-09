## Why

The accepted local history reader is bounded but only starts at the beginning: a 121,478,056-byte native rollout requires at least 232 forward calls to reach its newest feedback. A synthetic ten-page read consumed 47,090 bytes but fetched 4,662,688 bytes; routine plugin `check` also prepares a disposable provider unnecessarily.

## What Changes

- Add an opt-in bounded latest-history mode to the existing public_history_read tool, preserving forward defaults and public-only admission.
- Read incrementally and stop when the requested message/output budget is met, retaining fixed per-call bounds and honest byte accounting.
- Recommend list/describe/binding reuse for routine plugin discovery; reserve check for configuration validation or diagnostics.
- Keep implementation, tests and acceptance local. No deployment, protocol fusion, CodeGraph concurrency, caching service, reindex or research mutation.

## Capabilities

### New Capabilities
- `efficient-public-history`: Bounded newest-first traversal, efficient forward reads and economical discovery guidance. Extends the unarchived local `improve-web-research-tool-path` implementation; no main specs currently exist.

### Modified Capabilities
None.

## Impact

Existing plugins/web-workflow/plugin.mjs and native-protocol tests; deployment guidance and plugin/operator README files. No Rust or dependency changes. Existing dirty changes belong to the preceding accepted local change and must be preserved.

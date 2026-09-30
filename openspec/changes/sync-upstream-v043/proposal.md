## Why

The deployed CoordExp fork is based on the 0.4.1 development line while upstream 0.4.3 adds useful bounded reads, compact tool contracts, Job continuity and MCP reliability. Integrate the release without retaining parallel implementations where native upstream behavior already satisfies our recorded requirements.

## What Changes

- Merge pinned upstream `v0.4.3` (`b96a59a712ca5355ad8609cd861cd0c7acb9f99e`) into an isolated integration branch while preserving the original fork history and dirty source checkout.
- Use upstream implementations, schemas, guidance and tests as the default. Remove superseded local recovery, discovery and evidence patches after checking the existing consumer requirements; retain only demonstrated gaps and operator-specific integrations.
- **BREAKING**: adopt upstream removal of Session context ACK/revision and automatic recovery overlays. Preserve explicit Session/recorder identity, message ACK authority, bounded handoff and restoration of existing local Session rows.
- Retain configured MCP initialization guidance, shared read-only memory/public history/scoped CodeGraph integration and Runner-only `ms` Python where upstream does not provide the same behavior. Remove the obsolete CLAUDE exclusion customization as explicitly requested; automatic instruction loading follows upstream.
- Validate the merged source through relevant contract tests and a disposable real Server/Runner path; perform the subsequently authorized source consolidation, fork push and controlled local activation after source acceptance.

## Capabilities

### New Capabilities

None; upstream owns its new product features and contracts.

### Modified Capabilities

- `web-workflow`: upstream-first feature ownership and a pinned, independently checked integration candidate.
- `web-research-tool-path`: guidance follows native upstream Session recovery without the retired context ACK protocol.
- `web-workflow-evidence`: existing local v2 Session rows remain recoverable during the native continuity transition.

## Impact

WebCodex core/runtime/contracts, the existing optional workflow plugin and operator deployment guidance, and their nearest consumer tests. The integration branch starts from `50281db4c3e04fa474d9724b03dd3dc5b91d7ea8`; the original `deploy/web-workflow/AGENTS.md` dirty extension-source guidance is preserved and carried deliberately into the candidate. The initial authority covered source integration only. Subsequent explicit user instructions authorize keeping one canonical source checkout, pushing the accepted source to the existing GitHub fork and restarting the local service so ChatGPT Web can use it. This includes the required stopped-state backup and normal compatible state restoration, not public release/tag creation, force push, credential changes, research launches or overwriting newer state with a backup.

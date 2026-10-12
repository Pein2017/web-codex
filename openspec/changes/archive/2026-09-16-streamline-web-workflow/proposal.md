## Why

ChatGPT Web needs a reliable, concise entry into an existing checkout, with shared operator guidance and useful Python/code intelligence support. The deployment currently carries an unversioned MCP instructions patch and an older Runner; upstream now supplies startup catalogs and configured skills that should be reused.

## What Changes

- Preserve configurable MCP initialization instructions and provide a Web-specific operator guidance example.
- Enhance the existing work_on_project result only where needed to identify effective guidance and truthful capability state; reuse upstream catalogs and authority checks.
- Make operator-configured shared skills and read-only memory accessible without automatically importing Codex harness instructions or merging project execution state.
- Reproduce and repair workspace hygiene Git false negatives if still present after upstream integration.
- Provide bounded pytest report summaries and scoped CodeGraph access using existing execution/plugin mechanisms.
- Track source, checks, rollback and the named Linux deployment in this fork; manual result retrieval only.

## Capabilities

### New Capabilities

- `web-workflow`: Concise project startup, explicit shared context and bounded Python/code intelligence support.

### Modified Capabilities

None.

## Impact

Server MCP/runtime startup, Runner configuration, optional native plugins and operator documentation. Existing canonical checkout, authorization, SHA editing, Job and Session semantics remain authoritative. No browser wake-up, project-family framework, memory migration, GPU launch, or upstream npm publication.

## Why

Browser acceptance of the previous release exposed remaining invocation friction: Plugin ACK metadata reaches the concrete parser, Skill pagination lacks executable continuation, and specific failures can become generic Session evidence. Fix these existing paths before introducing more tools.

## What Changes

- Align Plugin gateway message ACK handling with the existing authorized Session wrapper contract; never merely discard or auto-ack metadata.
- Emit revision-guarded Skill continuation calls and actionable, bounded invalid-skill diagnostics.
- Preserve structured failure classification through the Session ledger and recovery projection.
- Recommend task-appropriate LSP, narrow CodeGraph relationship tools, and bounded literal search through existing discovery and Web guidance.
- Retain existing explicit context recovery; no universal recovery envelope, relationship router, evidence graph, automatic reindexing, or retry engine.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `web-workflow`: consistent invocation metadata, guarded continuation, useful failure evidence, and bounded relationship discovery.

## Impact

Existing Rust MCP/runtime gateway, Skills, Session evidence, discovery tests, and Web guidance. No new dependency, top-level tool, persistence subsystem, or change to project authority. Implementation and local tests are authorized; commit/push, release, deployment, and production restarts are not part of this change.

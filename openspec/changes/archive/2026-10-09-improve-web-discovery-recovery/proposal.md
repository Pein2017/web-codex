## Why

A 46-call ChatGPT Web review trace demonstrates working Skills and three successful CodeGraph queries, but also a rejected plugin parameter, incomplete Session recovery, and expensive/truncated exploration. The next step should remove avoidable call construction and recovery ambiguity without weakening evidence or authority.

## What Changes

- Provide bounded parser-ready startup follow-ups for selected Skills and Plugins, retaining exact project, Runner and definition revision.
- Make context-recovery suggestions explicitly request the complete handoff needed to establish a baseline.
- Clarify Web guidance for relationship discovery, bounded analysis, continuation and recorder/ACK discipline.
- Preserve timeout failure classification, explicit Session identity and all permission/schema fences.
- Per the user's deployment amendment, relocate the local operator state and required dependencies under `/data/CoordExp/.local/webcodex-custom/`, with immutable releases, a current entrypoint, separate durable state, backup and container-rebuild recovery.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `web-workflow`: Actionable bounded discovery and explicit complete context recovery.

## Impact

Existing startup catalog producers/projection/tests, Session recovery hint/tests and deployment guidance. No new MCP tools, aliases, product dependency, ledger, automatic indexing or research changes. Root owns scoped fork push and immutable local dogfood Server/Runner/Tunnel migration with rollback; no npm or public release. Source checkout stays in place. Credentials and databases stay untracked and private.

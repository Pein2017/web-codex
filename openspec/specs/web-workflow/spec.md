# web-workflow Specification

## Purpose
Make ChatGPT Web project work concise and reproducible while preserving canonical checkout authority and shared-context provenance.

## Requirements

### Requirement: Explicit Web guidance
The server SHALL expose configured bounded Web-specific instructions at MCP initialization and identify the effective guidance in project startup without copying full repository instructions by default.

#### Scenario: Shared guidance across worktrees
- **WHEN** two registered worktrees are opened with repository instructions disabled
- **THEN** their startup identifies the same configured guidance revision while retaining distinct canonical project identities.

### Requirement: Truthful compact startup
The existing work_on_project tool SHALL report available extension context using current authority and distinguish unprobed or unavailable capabilities from verified failures without introducing a second project entry tool.

#### Scenario: Optional provider missing
- **WHEN** an optional plugin is not configured
- **THEN** startup remains usable and does not claim that provider is ready.

### Requirement: Shared context remains explicit
Configured shared skills and memory SHALL be accessible on demand from each authorized project; reading context SHALL NOT create a memory copy, merge project state, or implicitly grant unsupported harness capabilities.

#### Scenario: Worktree reads shared context
- **WHEN** the user requests a shared skill or memory from a registered worktree
- **THEN** the response identifies its source and bounded content without modifying the shared store.

### Requirement: Accurate Git hygiene
Hygiene checks SHALL distinguish a Git worktree from a non-Git directory on the deployed runner.

#### Scenario: Registered Git worktree
- **WHEN** git_status succeeds for a registered Git worktree
- **THEN** hygiene recognizes Git or returns an explicit diagnostic failure instead of silently reporting non-Git.

### Requirement: Bounded Python and code intelligence results
The deployment SHALL provide a bounded pytest report summary and scoped CodeGraph queries through existing execution or plugin mechanisms, preserving full evidence references and explicit truncation or error states.

#### Scenario: Failing pytest report
- **WHEN** a generated report includes passed, failed and skipped tests
- **THEN** its summary preserves counts and bounded failing identities without claiming report parsing executed tests.

#### Scenario: Restricted code search
- **WHEN** the caller selects a code scope
- **THEN** the result respects that scope or rejects unsupported scope explicitly rather than silently ignoring it.

### Requirement: Reproducible self-hosted delivery
The named Linux deployment SHALL use a committed build with recorded identity, retained rollback and real MCP/Runner smoke evidence. Long-running jobs SHALL be retrieved manually; no automatic ChatGPT wake-up is promised.

#### Scenario: Deployment acceptance
- **WHEN** the candidate is installed
- **THEN** server and runner identity, project access, guidance, shared context and a disposable edit/test loop are verified.

### Requirement: Native upstream feature ownership

The integrated fork SHALL include the pinned upstream release and use native upstream tool names, schemas and behavior when they satisfy existing CoordExp requirements. A local implementation SHALL remain only for an operator-specific integration or a demonstrated requirement gap with a consumer-facing check. Retired context ACK/revision behavior MUST NOT be reinstated to preserve old code or fixtures.

#### Scenario: Native behavior replaces a local patch
- **WHEN** upstream supplies equivalent discovery, recovery or validation behavior
- **THEN** the fork uses that native implementation and removes the redundant local implementation, guidance and dedicated compatibility tests

#### Scenario: A requirement gap remains
- **WHEN** native behavior fails an existing boundedness, identity or durability scenario
- **THEN** the smallest local correction preserves that scenario while retaining unrelated upstream behavior and its tests

#### Scenario: Source integration completes
- **WHEN** the candidate is accepted
- **THEN** its history includes the pinned release and original fork base, the remaining local differences have explicit owners and checks, and production activation remains a separate action

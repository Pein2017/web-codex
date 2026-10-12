## Purpose

Make ChatGPT Web project work concise and reproducible while preserving canonical checkout authority and shared-context provenance.

## ADDED Requirements

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

# web-workflow-evidence Specification

## Purpose
Make the existing Web workflow report bounded, honest workspace and validation evidence so a model can use the real shared environment with less protocol guesswork.

## Requirements

### Requirement: Validation truth and recovery

Closeout SHALL preserve actual validation outcome separately from invocation misuse and expected negatives. A matching later successful assertion SHALL supersede its earlier failed assertion; unrelated successes and expected negative results SHALL NOT resolve it. Recognized pytest terminal summaries SHALL contribute bounded counts through existing execution evidence; missing or incomplete summaries SHALL NOT invent counts or success. Execution success and current-source freshness SHALL remain independent: native unproven source evidence MUST NOT be promoted to a current-workspace validation certificate. A completed operation with authoritative pre-first-write proof that no relevant source mutation ever began SHALL NOT alone invalidate a previously retained source observation; unchanged final content or successful rollback SHALL NOT supply this proof. This refinement SHALL preserve uncertainty for active writers, asynchronous execution, incomplete mutation truth, restart or changed identity, and SHALL NOT claim coverage of external filesystem writes.

#### Scenario: Repair and revalidate

- **WHEN** an assertion fails and the same assertion subsequently passes
- **THEN** execution evidence reflects the later passing result while historical failure remains inspectable

#### Scenario: Failure remains unresolved

- **WHEN** another assertion passes or an expected negative result is observed
- **THEN** an unmatched real failure remains actionable

#### Scenario: Pytest summary evidence

- **WHEN** completed test execution supplies a supported pytest terminal summary
- **THEN** its counts are detected without claiming a report-reader executed tests
- **AND** malformed, missing or insufficient output remains unknown rather than a fabricated pass

#### Scenario: Execution pass does not certify current source

- **WHEN** matching real pytest executions resolve the historical failure but native source freshness remains unproven
- **THEN** closeout retains the pass/count evidence and resolved failure while current evidence remains unproven with an explicit source limitation
- **AND** the disposable smoke checks this distinction instead of demanding a fabricated current-source PASS

#### Scenario: Proven no-effect operation does not invent source change

- **WHEN** a guarded edit or admission rejection finishes with authoritative evidence that no source write began, and no other writer invalidates the observation
- **THEN** that operation alone does not mark retained validation source evidence stale
- **AND** unproven evidence remains unproven rather than becoming current

#### Scenario: Commit is not a source-neutral exemption

- **WHEN** a commit operation can run hooks, filters or otherwise lacks proof that covered source was unchanged
- **THEN** its effect remains conservative regardless of tool name or exit zero
- **AND** earlier test success does not certify the post-commit workspace

#### Scenario: Concurrent or uncertain writer cannot restore freshness

- **WHEN** a no-effect operation overlaps a real or unknown writer, an asynchronous handoff, cancellation or an exhausted/lost source epoch
- **THEN** the no-effect result cannot erase the other writer's invalidation or uncertainty

#### Scenario: Rolled-back mutation is not a no-write attempt

- **WHEN** a write or directory creation begins and is subsequently rolled back while validation can overlap it
- **THEN** the operation conservatively invalidates source observation even if its final effect report is unchanged
- **AND** that report cannot be promoted to pre-write no-effect proof

### Requirement: Existing MCP path is demonstrably usable

Web guidance SHALL direct callers to current discovery, explicit Session recording, existing `read_revision` guards and Runner language prerequisites. Disposable real Server/Runner verification SHALL cover advertised JS execution, guarded edits, stale-guard rejection and validation/finish behavior. No hidden Session selection or context acknowledgment SHALL be introduced.

#### Scenario: Read then edit

- **WHEN** the caller passes a returned `read_revision` as `expected_read_revision`
- **THEN** the edit uses the exact source guard and rejects stale source before mutation

#### Scenario: Deploy current contracts

- **WHEN** the integrated fork is installed locally
- **THEN** Server and Runner identify the same clean source, registered Projects remain accessible and the previous deployment remains recoverable

### Requirement: Structured execution lifecycle remains truthful

Structured execution SHALL produce mutually consistent admission, start and terminal evidence for the same Job. A pre-start rejection SHALL NOT assert that execution began; completion SHALL retain the actual exit outcome. Invalid or missing lifecycle evidence SHALL remain an explicit unknown outcome with a same-execution diagnostic or reconciliation path, never implicit retry authorization. Correcting a lifecycle producer SHALL NOT broaden accepted invalid state combinations.

#### Scenario: Fast shell terminal transition

- **WHEN** a shell command completes before its start response or the next observation is delivered
- **THEN** the admitted Job's sequenced evidence remains consistent and the terminal outcome is delivered without a protocol violation
- **AND** the payload command is dispatched exactly once

#### Scenario: Reject before process creation

- **WHEN** executable, profile or policy admission rejects a structured request before process creation
- **THEN** the result identifies a not-started outcome without a start timestamp or fabricated exit success

#### Scenario: Inconsistent lifecycle cannot permit replay

- **WHEN** terminal status and command execution state contradict one another or required evidence is missing
- **THEN** the consumer rejects the inconsistent evidence and retains uncertainty
- **AND** it does not retry, relabel the Job as completed or accept a weaker lifecycle contract

### Requirement: Work Result uses authorized canonical resolution

Work Result reads SHALL accept an explicit Project selector that the ordinary authenticated resolver uniquely resolves to the target Session's canonical Project. Resolution SHALL recheck current authority and Project incarnation. Business Session identity SHALL remain explicit; recorder, Window affinity and history SHALL NOT choose it. Ambiguous, stale, unauthorized or conflicting targets SHALL fail closed before presentation or mutation.

#### Scenario: Short reference selects the same Project

- **WHEN** a canonical id, current principal-scoped Project reference or uniquely authorized short name resolves to the explicit Session's Project
- **THEN** the same Work Result is returned without an additional exact-id repair call

#### Scenario: Explicit target conflict remains a failure

- **WHEN** the supplied Project resolves to a different Project than the supplied Session
- **THEN** the result rejects the mismatch and does not replace either explicit target

#### Scenario: Stale or ambiguous reference remains a failure

- **WHEN** a short name has multiple visible matches or a pinned Project reference is stale, revoked or owned by another principal
- **THEN** the request fails without Session-based target inference or information disclosure

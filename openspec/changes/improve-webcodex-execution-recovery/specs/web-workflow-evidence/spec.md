## ADDED Requirements

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

### Requirement: Bounded durable Job evidence recovery

Within operator-declared retention and disk bounds, terminal receipts and exact retained decoded stdout/stderr SHALL be recoverable through existing authorized Job readers for the original execution after live-tail or terminal-inventory eviction, ordinary receipt expiry or Server/Runner restart. Archived evidence SHALL retain an authenticated exact-Job lookup distinct from live lifecycle restoration; temporary Runner-instance replacement SHALL NOT orphan committed history. Output retention SHALL remain separate from execution survival and task acceptance. Readers SHALL enforce current Project visibility and pinned root incarnation for archive metadata/output, exact Job/executor identity, private storage, bounded response size and explicit completeness. When a fresh Server lacks current Project authority and the Runner is offline, only the explicit minimal-terminal-fact exception below MAY disclose historical facts under normal current request authentication and immutable admission ownership. Archive failure, truncation, expiration or unsupported historical evidence SHALL NOT be represented as empty complete output or authorize re-execution. Quota-bound retention SHALL NOT promise minimum availability or silently sanitize the retained evidence.

The total archive bound SHALL account for actual output, active and temporary files, locator/index storage and its auxiliary files, and concurrent reservations across all namespaces of the named deployment. Retained row counts or serialized-payload limits alone SHALL NOT constitute a physical storage bound. Reaching an evidence-storage bound SHALL stop or reject additional retention without blocking child output draining, changing the actual process outcome or consuming unrelated Session/ordinary-receipt storage as an overflow path.

#### Scenario: Read output older than the live tail

- **WHEN** a Job produces more output than its live snapshot can hold and the retained archive contains earlier output
- **THEN** the caller can read that earlier output in bounded pages for the same Job
- **AND** the result retains stream position, continuation and completeness without repeating the command

#### Scenario: Recover a terminal receipt after restart

- **WHEN** the Server or Runner restarts after a Job terminal receipt has been durably committed
- **THEN** an authorized reader recovers the original Job identity, actual exit outcome and retained evidence
- **AND** a historical receipt does not become permission to launch, stop or reattach a process

#### Scenario: Output retention does not detach a running process

- **WHEN** an ordinary non-detached Job loses its owning Runner before terminal completion
- **THEN** recovery reports the reconciled lost or unknown execution state
- **AND** the presence of archived output is not presented as proof of continued execution or completion

#### Scenario: Archive outlives ordinary inventory retention

- **WHEN** a retained terminal archive remains within its configured bounds but the Job has left the live terminal inventory or ordinary receipt lifetime
- **THEN** an authorized exact-id read recovers the original committed terminal evidence and available archived bytes without registering the entire archive as live inventory
- **AND** an unavailable Runner is reported as output unavailable rather than an empty complete stream

#### Scenario: Evidence unavailable or capped

- **WHEN** a retention limit, quota, write failure or old non-archived Job makes some output unavailable
- **THEN** the reader explicitly reports the retained range and missing evidence
- **AND** actual execution outcome remains distinct from evidence-storage failure

#### Scenario: Physical archive bound includes storage overhead

- **WHEN** repeated archive commits, concurrent reservations or a long-lived reader retain storage page versions or temporary data
- **THEN** the total physical archive usage remains within the declared deployment-wide bound, including index and auxiliary bytes
- **AND** an insertion that cannot stay within that bound fails as evidence retention, not as a fabricated process failure or an unbounded spill into another store

#### Scenario: Locator capacity does not alter other durable state

- **WHEN** the archive locator reaches its physical capacity or an allowed old terminal archive is evicted
- **THEN** current Session identities/events and ordinary receipt behavior remain intact
- **AND** active Jobs, unrelated traces/configuration and sealed qualification/research receipts are not removed to make space

#### Scenario: Fresh Server and offline Runner expose only authenticated terminal facts

- **WHEN** an exact archived Job remains within retention, a fresh Server cannot establish current Project authority, and the owning Runner is offline
- **THEN** normal current request authentication and immutable admission partition/owner checks permit only a dedicated `job_id`, canonical terminal `status` and recorded `exit_code` projection
- **AND** no Project/root/path, Session, command, error, archive descriptor, sizes, timestamps or output are returned and no archive read is dispatched
- **AND** a different or revoked principal is rejected without confirming the Job, the deadline is not renewed, and online evidence of Project revocation or retargeting cannot use this exception

#### Scenario: Archive cannot broaden access

- **WHEN** a caller cannot currently access the Job's exact Project, or a storage reference attempts path traversal, symlink escape or identity substitution
- **THEN** the read fails closed without disclosing archived payloads

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

## MODIFIED Requirements

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

## MODIFIED Requirements

### Requirement: Workspace baseline observations

Fresh project Workflow Sessions SHALL retain a bounded startup Git path/status observation and SHALL preserve that baseline across resume and restart. Finish SHALL distinguish pre-existing dirty paths from later newly dirty, cleared and overlapping paths without claiming exclusive Session authorship or unchanged content for overlapping paths. Existing canonical local v2 Session rows containing this optional baseline SHALL remain restorable when adopting upstream continuity changes; Session identity, events, messages and baseline MUST NOT be silently discarded because retired context metadata is removed.

#### Scenario: Shell changes a previously clean path
- **WHEN** a Session starts with dirty `existing.txt` and later a shell changes clean `new.txt`
- **THEN** finish identifies `existing.txt` as pre-existing and `new.txt` as newly dirty since the observation
- **AND** tool classification counts are not presented as proof that no filesystem write occurred

#### Scenario: Existing dirty path remains dirty
- **WHEN** a baseline path is dirty both before and after work
- **THEN** finish reports overlap without asserting that its contents or author are unchanged

#### Scenario: Baseline cannot prove completeness
- **WHEN** startup or finish inspection fails, is truncated, changes HEAD/target, or the restored Session lacks a baseline
- **THEN** the result explicitly reports the evidence limitation and does not infer absent paths as clean or cleared

#### Scenario: Restore an existing local v2 row
- **WHEN** a frozen pre-upgrade canonical Session row contains a workspace baseline and retired context revision metadata
- **THEN** restoration retains its identity, events, messages and baseline, and subsequent serialization follows upstream behavior by omitting retired context revision metadata

### Requirement: Validation truth and recovery

Closeout SHALL preserve actual validation outcome separately from invocation misuse and expected negatives. A matching later successful assertion SHALL supersede its earlier failed assertion; unrelated successes and expected negative results SHALL NOT resolve it. Recognized pytest terminal summaries SHALL contribute bounded counts through existing execution evidence; missing or incomplete summaries SHALL NOT invent counts or success. Execution success and current-source freshness SHALL remain independent: native unproven source evidence MUST NOT be promoted to a current-workspace validation certificate.

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

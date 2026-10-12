## Purpose

Make the existing Web workflow report bounded, honest workspace and validation evidence so a model can use the real shared environment with less protocol guesswork.

## ADDED Requirements

### Requirement: Workspace baseline observations

Fresh project Workflow Sessions SHALL retain a bounded startup Git path/status observation and SHALL preserve that baseline across resume and restart. Finish SHALL distinguish pre-existing dirty paths from later newly dirty, cleared and overlapping paths without claiming exclusive Session authorship or unchanged content for overlapping paths.

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

### Requirement: Validation truth and recovery

Closeout SHALL preserve actual validation outcome separately from invocation misuse and expected negatives. A matching later successful assertion SHALL supersede its earlier failed assertion; unrelated successes and expected negative results SHALL NOT resolve it. Recognized pytest terminal summaries SHALL contribute bounded counts through existing execution evidence; missing or incomplete summaries SHALL NOT invent counts or success.

#### Scenario: Repair and revalidate

- **WHEN** an assertion fails and the same assertion subsequently passes
- **THEN** current validation reflects the later passing evidence while historical failure remains inspectable

#### Scenario: Failure remains unresolved

- **WHEN** another assertion passes or an expected negative result is observed
- **THEN** an unmatched real failure remains actionable

#### Scenario: Pytest summary evidence

- **WHEN** completed test execution supplies a supported pytest terminal summary
- **THEN** its counts are detected without claiming a report-reader executed tests
- **AND** malformed, missing or insufficient output remains unknown rather than a fabricated pass

### Requirement: Existing MCP path is demonstrably usable

Web guidance SHALL direct callers to current discovery, explicit Session recording, existing `read_revision` guards and Runner language prerequisites. Disposable real Server/Runner verification SHALL cover advertised JS execution, guarded edits, stale-guard rejection and validation/finish behavior. No hidden Session selection or context acknowledgment SHALL be introduced.

#### Scenario: Read then edit

- **WHEN** the caller passes a returned `read_revision` as `expected_read_revision`
- **THEN** the edit uses the exact source guard and rejects stale source before mutation

#### Scenario: Deploy current contracts

- **WHEN** the integrated fork is installed locally
- **THEN** Server and Runner identify the same clean source, registered Projects remain accessible and the previous deployment remains recoverable

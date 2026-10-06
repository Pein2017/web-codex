## ADDED Requirements

### Requirement: Native worker launch preflight is sandbox-bound

The named deployment's supported native Codex worker launch route SHALL run a bounded, non-model preflight through the same Runner execution context and native sandbox policy construction for the explicitly selected executable, checkout, cwd and configuration before substantive worker execution. It SHALL distinguish host visibility from native setup/read probe success, expose known mount/path admission failures, and retain an unproven result for unsupported routes or when same-policy context cannot be established. A successful probe SHALL NOT certify arbitrary later commands. A blocked or unproven preflight SHALL NOT automatically weaken sandbox policy, change checkout, start an alternative worker or claim that the task ran. Generic process completion SHALL remain separate from worker acceptance.

#### Scenario: Known nested sandbox failure

- **WHEN** the selected worker sandbox cannot mount or access the authorized checkout
- **THEN** preflight returns a bounded admission diagnostic before the worker's substantive/model invocation
- **AND** no unsafe fallback or duplicate worker is started

#### Scenario: Host access is insufficient

- **WHEN** the host can read the checkout but the same-policy sandbox probe is unavailable or cannot be shown equivalent
- **THEN** preflight reports sandbox access unproven, not ready

#### Scenario: Successful preflight does not accept the task

- **WHEN** a same-policy probe succeeds and an explicitly authorized worker subsequently exits zero with a blocked or incomplete report
- **THEN** the execution receipt retains exit zero while task acceptance remains blocked or incomplete
- **AND** preflight is not represented as proof of future task completion or unchanged permissions

## MODIFIED Requirements

### Requirement: Reproducible self-hosted delivery

The named Linux deployment SHALL use a committed build with recorded identity, retained rollback and real MCP/Runner smoke evidence. Long-running jobs SHALL be retrieved manually; no automatic ChatGPT wake-up is promised. Reinstallation SHALL update matching Server, Runner, CLI and Web workflow assets under the existing `codex-tools/web-codex` owner, preserve current private configuration, registered Projects, Sessions and accepted evidence, and validate real consumer routing after activation. Local dogfood delivery SHALL NOT imply a version bump, public tag, push, GitHub/npm release or database restoration.

#### Scenario: Deployment acceptance

- **WHEN** the candidate is installed
- **THEN** server and runner identity, project access, guidance, shared context and a disposable edit/test loop are verified
- **AND** the five changed execution/recovery boundaries have their declared consumer evidence rather than only unit-test counts

#### Scenario: Preserve current runtime state

- **WHEN** the qualified local fork replaces the installed application
- **THEN** current credentials, Project roots, Session identities and archived evidence remain intact
- **AND** old deployment-path aliases or soft links are not introduced

#### Scenario: Candidate activation fails

- **WHEN** the candidate cannot pass its post-install smoke
- **THEN** the exact retained previous application bundle can be restored through the controlled lifecycle
- **AND** restoring an older database requires a separate explicit decision and cannot overwrite newer accepted evidence automatically

#### Scenario: Downgrade boundary qualified before activation

- **WHEN** a candidate can write new durable metadata before the previous application would need to recover it
- **THEN** disposable candidate-write/previous-read and pruning checks prove existing Session/receipt behavior and evidence preservation before live activation
- **AND** incompatible downgrade keeps activation on HOLD unless the user explicitly changes the recovery boundary

## ADDED Requirements

### Requirement: Direct Runner operation does not depend on Codex CLI

The named deployment's Web workflow SHALL complete authorized file, Git, validation, process and Job recovery operations through the existing WebCodex Server → Runner path. Its guidance, startup, packaged assets and acceptance workflow SHALL NOT require or invoke Codex CLI, a Codex sandbox probe or a local Codex agent to perform those operations. It SHALL NOT configure or implicitly delegate to a local coding-agent provider as a replacement for direct tools. Removing that dependency SHALL preserve ordinary authentication, Project/root authority, permissions, source guards and execution uncertainty. Process completion SHALL remain distinct from task acceptance.

#### Scenario: Direct work with Codex CLI unavailable

- **WHEN** Codex CLI is unavailable and a caller requests authorized work on a disposable Project
- **THEN** existing tools read and guard-edit a file, run a bounded test and recover the original terminal Job through the Runner
- **AND** no Codex CLI invocation, sandbox readiness check or local-agent fallback is needed

#### Scenario: Direct-operation denial cannot delegate around authority

- **WHEN** Project authority, Runner policy or an edit revision fence rejects a request
- **THEN** the existing direct path preserves that rejection and any uncertainty
- **AND** it does not launch a local agent, change target or weaken permissions to continue

#### Scenario: Process completion does not accept the task

- **WHEN** a directly executed program exits zero but its required consumer artifact or validation is missing, blocked or incomplete
- **THEN** the execution receipt retains the actual exit outcome while task acceptance remains incomplete
- **AND** generic process success is not presented as proof that the user's goal was achieved

## MODIFIED Requirements

### Requirement: Reproducible self-hosted delivery

The named Linux deployment SHALL use a committed build with recorded identity, retained rollback and real MCP/Runner smoke evidence. Long-running jobs SHALL be retrieved manually; no automatic ChatGPT wake-up is promised. Reinstallation SHALL update matching WebCodex Server, Runner, WebCodex CLI and Web workflow assets under the existing `codex-tools/web-codex` owner, preserve current private configuration, registered Projects, Sessions and accepted evidence, and validate real consumer routing after activation. Codex CLI worker or sandbox readiness SHALL NOT be a delivery prerequisite. Local dogfood delivery SHALL NOT imply a version bump, public tag, push, GitHub/npm release or database restoration. The current no-install ruling SHALL remain binding until the user explicitly authorizes later activation.

#### Scenario: Deployment acceptance

- **WHEN** the candidate is installed
- **THEN** server and runner identity, project access, guidance, shared context and a disposable edit/test loop are verified
- **AND** lifecycle truth, bounded archive recovery, authorized selectors, no-write source proof and direct operation without Codex CLI have their declared consumer evidence rather than only unit-test counts

#### Scenario: Planning approval is not activation authority

- **WHEN** the user approves revised direct-operation planning while the no-install ruling remains in force
- **THEN** that approval does not replace the live application or restart services
- **AND** later activation still requires a qualified candidate and a new explicit installation request

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

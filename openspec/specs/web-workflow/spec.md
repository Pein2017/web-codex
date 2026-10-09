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

### Requirement: Actionable extension discovery
Startup extension entries SHALL provide bounded parser-ready follow-ups for their exact authorized selection. Skill reads SHALL include the observed definition revision; Plugin discovery SHALL use the exact Runner and require describe before call. Startup SHALL retain existing projection budgets and exclusion rules.

#### Scenario: Selected extension can be opened
- **WHEN** startup includes a Skill or Plugin tool
- **THEN** its suggested call parses through the existing tool contract and targets that exact project/Skill revision or Runner/provider/tool without inferred recorder or execution authority.

#### Scenario: Catalog is incomplete or unauthorized
- **WHEN** a catalog exceeds its budget or the caller lacks access
- **THEN** bounded omission or unavailability remains explicit and no follow-up advertises an excluded selection.

### Requirement: Complete explicit recovery suggestion
A context-recovery hint SHALL explicitly request the complete handoff view needed for current-state rebaselining. The hint SHALL NOT acknowledge context, rerun the original effect, infer a recording Session or weaken failure classification.

#### Scenario: Missing ACK is recovered by explicit observation
- **WHEN** a caller follows the suggested complete handoff for the exact authorized Session and state remains stable
- **THEN** the normal recovery path can return a baseline revision without another parameter-repair round trip.

#### Scenario: Incomplete or racing recovery remains uncertain
- **WHEN** a caller requests a partial handoff or a concurrent checkpoint invalidates the observation
- **THEN** no complete baseline is certified and original timeout or execution failures remain truthful.

### Requirement: Persistent container deployment
The named local deployment SHALL retain configuration, credentials, database, launchers, logs, verification and rollback records under `/data/CoordExp/.local/webcodex-custom/`. Release artifacts SHALL be immutable and selected through a current entrypoint; durable state SHALL be independent of the selected version. Required nonpersistent runtime dependencies SHALL be relocated or identified as explicit image prerequisites.

#### Scenario: Container recreation and recovery
- **WHEN** the persistent tree survives but `/root` and `/var` do not
- **THEN** the documented recovery entrypoint can start Server, Runner and Tunnel after stated image/network prerequisites are restored, without recreating credentials or discarding stored Sessions.

#### Scenario: Migration or release rollback
- **WHEN** deployment switches versions or restores the preceding accepted build
- **THEN** exact prior artifacts and consistent state backups remain available, secrets retain private permissions, and research/proxy processes are not implicitly stopped.

### Requirement: Consistent Plugin acknowledgement metadata
The Plugin gateway SHALL process supported invocation acknowledgement metadata through the authorized Workflow Session contract without forwarding it as provider business arguments or silently discarding an accepted message ACK.

#### Scenario: Explicit guidance acknowledgement
- **WHEN** an authorized caller supplies a valid message ACK for its explicit recording Session on a Plugin invocation
- **THEN** the invocation accepts the wrapper, records the existing first-ACK observation semantics, and does not resolve the guidance or infer a Session.

#### Scenario: Invalid or unauthorized acknowledgement
- **WHEN** a caller supplies malformed or wrong-Session metadata or lacks Session authority
- **THEN** existing validation and authority boundaries remain enforced and another Session's guidance is not acknowledged.

### Requirement: Executable Skill continuation
An incomplete Skill read SHALL return one parser-ready continuation with the same authorized project, opaque Skill identity, resource path, effective page limit, next line, and observed definition revision guard. A completed read SHALL NOT advertise another page.

#### Scenario: Continue and detect changes
- **WHEN** a caller executes the suggested continuation
- **THEN** it reads the next bounded page without skipping content, or fails closed if the guarded definition changed.

### Requirement: Actionable Skill diagnostics
Invalid Skill discovery entries SHALL provide bounded safe logical identity and source scope where available, without exposing native paths or inventing a usable Skill identity.

#### Scenario: Missing definition
- **WHEN** a configured candidate has no Skill definition
- **THEN** discovery identifies the failing logical candidate and its scope, retains the reason code, and remains bounded.

### Requirement: Faithful structured failure evidence
Session evidence SHALL preserve available bounded structured failure codes through recording and handoff rather than replacing them with generic runtime errors. Unstructured error prose SHALL NOT become a machine reason code.

#### Scenario: Language service unavailable
- **WHEN** a language-service observation reports a structured unavailable error
- **THEN** the Session evidence and handoff retain that classification without claiming the call succeeded or is safe to retry.

### Requirement: Discoverable bounded relationship exploration
Existing exploration discovery and Web guidance SHALL describe symbol relationship exploration alongside file search, selecting available backends by task and freshness, without adding a new routing tool.

#### Scenario: Unavailable semantic backend
- **WHEN** LSP is unavailable or a graph is stale
- **THEN** guidance recommends an appropriate available narrow relationship query or bounded literal search, does not claim equivalent completeness, and does not automatically reindex or retry.

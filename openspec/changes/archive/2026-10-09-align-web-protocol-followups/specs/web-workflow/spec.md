## ADDED Requirements

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

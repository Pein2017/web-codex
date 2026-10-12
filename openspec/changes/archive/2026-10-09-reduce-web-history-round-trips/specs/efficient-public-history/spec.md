## Purpose

Let Web users retrieve recent public feedback without scanning an entire rollout, while retaining bounded work, source identity and safe forward continuation.

## ADDED Requirements

### Requirement: Opt-in latest public history
public_history_read SHALL accept mode `latest` or `forward`, defaulting to forward without a cursor and inferring the mode from a cursor otherwise. Latest pages MUST select the newest eligible complete records within a bounded backward scan, return each page in chronological order, and continue toward older records without duplicating source occurrences. A conflicting explicit mode and cursor MUST be rejected.

#### Scenario: Large native rollout with recent feedback
- **WHEN** the last three public messages are near the end of a native rollout larger than 64 MiB and latest mode requests three messages
- **THEN** one bounded call returns those three messages without reading the file prefix or claiming all history was scanned

#### Scenario: Older pages and concurrent append
- **WHEN** a latest cursor is continued after new records are appended
- **THEN** it continues older records in its original snapshot; a fresh latest request discovers the append, and replacement/truncation is rejected

### Requirement: Honest provenance and partial records
Latest messages MUST identify exact file and byte positions. Absolute line numbers MUST be null rather than fabricated when unknown. Unknown or nonpublic content MUST remain excluded using the existing public-message policy. Unterminated, malformed, oversized, UTF-8 and boundary-split records MUST NOT cause unbounded allocation, duplicate output or a non-advancing older-page loop. Omission and incomplete-coverage evidence MUST remain bounded and explicit.

#### Scenario: Private and split records at the tail
- **WHEN** the tail contains private records, an incomplete append and a record crossing a scan boundary
- **THEN** no private or partial text leaks, complete public records remain recoverable across pages, and bounded cursors or exhaustion explicitly describe remaining work

### Requirement: Demand-driven reads preserve forward behavior
Both modes MUST keep at most 512 KiB scan bytes plus 16 KiB fingerprint reads per call and at most the existing message/output/record budgets. Forward defaults and existing valid forward cursors MUST remain usable, including EOF append continuation. Source reads MUST stop promptly after enough messages fit rather than eagerly filling the maximum window.

#### Scenario: Small-page read amplification
- **WHEN** a synthetic 1000-record file with approximately 400-byte user texts is read for ten forward pages of ten messages each
- **THEN** exactly 100 occurrences are returned in order, total reported bytesRead is at most 1 MiB, and source integrity and cursor checks remain active

### Requirement: Economical discovery guidance
Routine guidance SHALL use list only when needed, describe the selected tool, and reuse valid bindings; it MUST reserve check for configuration validation or diagnosis without weakening provider admission or stale-binding checks.

#### Scenario: Repeated calls to an unchanged plugin
- **WHEN** a valid binding is already available
- **THEN** guidance does not require another check, list or describe before each call

# web-research-tool-path Specification

## Purpose
Enable bounded Web research against existing checkouts with correct project identity, useful scoped evidence, and explicit instruction selection without expanding execution authority.

## Requirements

### Requirement: Canonical execution identity

An unambiguous authorized project alias SHALL address the same execution target as its canonical project ID. Canonicalization MUST NOT bypass Runner, Project or Session authorization.

#### Scenario: Alias with a matching Session
- **WHEN** the same harmless process is requested using an authorized alias and canonical ID with the matching explicit Session
- **THEN** both requests reach the same Runner/project and return equivalent execution outcomes without an identity-format rejection

#### Scenario: Mismatched Session or ambiguous alias
- **WHEN** the Session targets another project or an alias does not resolve uniquely
- **THEN** execution is rejected before process start without guessing a target

### Requirement: Bounded batch progress is truthful

File and search batch observations SHALL retain explicit incompleteness and recoverable omitted-item identities. An oversized read item MUST NOT suppress later independent items that fit. A zero-content result MUST explicitly state that no requested content was returned rather than imply successful inspection.

#### Scenario: Oversized first line and small second file
- **WHEN** the first requested file has an indivisible line above the output budget and a later small file fits
- **THEN** the small file is returned within budget and the omitted first item remains explicitly recoverable without silently cutting its source line

#### Scenario: Partial search and zero progress
- **WHEN** only one of three searches fits, or no read item fits at all
- **THEN** remaining requests are identified with a bounded valid follow-up or explicit non-progress reason; no result claims complete coverage

### Requirement: Scoped CodeGraph evidence

Web guidance and plugin discovery SHALL direct known-directory symbol queries to the existing scoped CodeGraph capability. Scoped results MUST exclude outside-directory matches and retain freshness and bounded-coverage limitations. Missing graph evidence MUST NOT be presented as missing source code.

#### Scenario: Training module lookup
- **WHEN** a query names `probes/training_set_completion` and a source256 symbol
- **THEN** the supported scoped route returns only matches under that directory, or explicit no-match/incomplete evidence with source-search guidance; unrelated worker code is not substituted as an answer

### Requirement: Bounded public history access

The optional workflow plugin SHALL provide read-only public-message retrieval for an explicit Codex thread under an operator-configured history root. It MUST bound discovery, scanned bytes, record size and output, reject traversal/symlink escape, retain source provenance, report skipped or malformed data, and provide progress-preserving pagination. It MUST NOT return hidden reasoning, credentials/configuration records or raw tool payloads, or write/steer sessions.

#### Scenario: Mixed and growing rollout
- **WHEN** a fixture contains public user/assistant messages, duplicate event representations, analysis records, a malformed line and an oversized record
- **THEN** only allowed public messages are returned with provenance, omissions and a usable continuation; append-only growth can be resumed and replacement/truncation is detected rather than silently reusing a stale cursor

#### Scenario: Unconfigured or escaped root
- **WHEN** history access is unconfigured or a request attempts to escape its configured root
- **THEN** the capability is absent or access is denied without reading outside that root

### Requirement: Guidance preserves execution boundaries

Web guidance SHALL distinguish resolved project identity, business Session, recorder and explicit collaboration-message ACK, and explain partial-result continuation. It SHALL follow upstream explicit recovery/handoff behavior without requesting retired Session context ACKs or context revisions. It MUST NOT infer recorder identity, auto-ACK messages, rerun an effect solely to record it, or recommend changing execution tools to evade a host safety denial.

#### Scenario: Successful execution with missing recorder
- **WHEN** execution succeeds but recorder metadata is missing
- **THEN** guidance preserves the execution result and requests explicit metadata on future calls rather than re-executing the command

#### Scenario: Native recovery after missing context
- **WHEN** prior task context is missing or an explicit handoff is requested
- **THEN** guidance uses the exact authorized upstream Session handoff path without a context revision handshake or automatic effect retry

#### Scenario: Collaboration message ACK
- **WHEN** the caller explicitly acknowledges messages for an authorized recorder
- **THEN** the acknowledgment remains request-scoped, is not forwarded as provider business input, and grants neither effect authority nor durable task acceptance

## MODIFIED Requirements

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

## REMOVED Requirements

### Requirement: Exclude CLAUDE from automatic instruction loading
**Reason**: The user explicitly retired the obsolete customization; native upstream instruction loading is sufficient.
**Migration**: Remove the exclusion environment switch, implementation, dedicated tests and current operator guidance. Keep the upstream candidate policy and separately authorized explicit file reads unchanged.

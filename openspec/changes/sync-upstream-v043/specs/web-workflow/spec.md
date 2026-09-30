## ADDED Requirements

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

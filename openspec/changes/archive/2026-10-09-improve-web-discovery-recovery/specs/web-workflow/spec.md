## ADDED Requirements

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

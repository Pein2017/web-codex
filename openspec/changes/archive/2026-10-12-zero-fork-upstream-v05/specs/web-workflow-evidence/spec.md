## REMOVED Requirements

### Requirement: Workspace baseline observations
**Reason**: Fork-only Session field with no upstream equivalent; upstream `review_changes`/`get_git_status` cover review needs. Persisted rows were migrated by removing the field so upstream's strict decoder retains them.

### Requirement: Bounded durable Job evidence recovery
**Reason**: Fork-only durable Job output archive retired; upstream reconnect recovery (#887/#976) is used, and long GPU work runs outside WebCodex Jobs.

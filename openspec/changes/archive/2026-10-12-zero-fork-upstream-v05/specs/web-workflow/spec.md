## ADDED Requirements

### Requirement: Pulled Runner-global Web guidance
The Runner SHALL project the operator-reviewed Web instruction file through upstream `[instructions].files`; the model receives it by requesting `project.instructions` on `work_on_project`. The Server MUST NOT carry a fork-only push of instructions at MCP initialization.

#### Scenario: Guidance is pulled, not pushed
- **WHEN** a fresh model context calls `work_on_project` with `_wc.context=["project.instructions"]`
- **THEN** the projection contains the configured Runner-global file ahead of project-local instructions within the shared sidecar budget.

### Requirement: Zero-fork upstream overlay
The maintained branch SHALL be unmodified upstream source plus additive CoordExp directories (`deploy/`, `plugins/web-workflow/`, `openspec/`, maintenance docs, root-anchored ignores). CoordExp behavior SHALL be expressed only through configuration, plugins, instruction files and deployment scripts. A source patch requires a demonstrated requirement gap recorded in OpenSpec before it is applied.

#### Scenario: Upstream update
- **WHEN** a reviewed upstream revision is adopted
- **THEN** the overlay rebases without source conflicts, the instruction file follows any tool renames, and the ChatGPT connector rescans tools when the surface changed.

#### Scenario: Native behavior replaces a local capability
- **WHEN** upstream supplies an equivalent to a retired fork capability
- **THEN** the overlay configures or documents the native path instead of reinstating the fork implementation.

## REMOVED Requirements

### Requirement: Explicit Web guidance
**Reason**: Replaced by "Pulled Runner-global Web guidance"; the fork-only initialization push is retired with the overlay.

### Requirement: Native upstream feature ownership
**Reason**: Replaced by "Zero-fork upstream overlay"; no local source implementations remain.

### Requirement: Consistent Plugin acknowledgement metadata
**Reason**: Fork-only Session ACK metadata; upstream v0.5 owns plugin result contracts and the web-workflow plugin does not depend on it.

### Requirement: Executable Skill continuation
**Reason**: Fork-only skill continuation contract retired with the overlay; upstream `list_skills`/`load_skill` are used as shipped.

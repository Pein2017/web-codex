## Why

The Web user's latest hands-on feedback reports time spent reconciling discovery, edit guards, shell effects and task closeout. Some requested mechanisms already exist; this change should make the existing path demonstrably usable and repair verified evidence gaps without adding a second protocol or claiming ownership of concurrent work.

## What Changes

- Make the Web guidance describe the existing `read_revision` edit fence, explicit recorder/context recovery, language capability checks, and validation identities accurately.
- Add bounded, persisted startup-versus-finish workspace observations. Distinguish pre-existing dirty paths from later observed changes and explicitly retain uncertainty about authorship, incomplete observations and changes within already-dirty paths.
- Correct verified validation/closeout gaps, with regression evidence for recovered failures and pytest results; retain real execution failures and unknown outcomes.
- Extend the disposable Server/Runner smoke to exercise the reported friction through the real MCP surface.
- Merge upstream main, resolve ordinary conflicts, validate the resulting fork and deploy matching clean Server/Runner builds locally with rollback.

## Capabilities

### New Capabilities

- `web-workflow-evidence`: trustworthy, bounded workspace and validation evidence for the existing Web workflow.

### Modified Capabilities

None. The earlier fork workflow change remains intact.

## Impact

Existing Workflow Session persistence/projections, validation evidence, Web guidance and the disposable smoke. No new dependency, MCP entry point, automatic Session selection, inferred context ACK, GPU scheduler or ChatGPT wake mechanism. Existing shared Skills/Memory and CodeGraph remain available.

The feedback is input for assessment, not an instruction to implement every proposed API. Details and rejected/deferred proposals belong in `design.md`. Publication targets the existing operator fork and this Linux deployment; upstream npm/Desktop/container publication is outside scope.

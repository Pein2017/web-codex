## Why

The 2026-10-06 WebCodex dogfood refactor completed substantial real work, but its retained Server ledger exposes avoidable execution/recovery friction. In the three implementation Sessions, 353 tool requests included two `run_shell` protocol failures with `structured_job_lifecycle_invalid` and an unknown execution outcome, three Work Result selector rejections, and worker logs whose earlier lines were no longer available from bounded Job tails. The original worker reports also identify sandbox admission failures and one repeated native qualification after prior evidence was missed. These observations justify targeted reliability and turn-economy work, not another orchestration framework.

The historical execution ran with full trace disabled, so the two protocol errors have no retained exact request/update payloads. Two ChatGPT turn interruptions have no failing MCP request at their boundaries; their Host-side cause is unknown and is outside this change. Full trace has now been enabled on the existing deployment to support prospective diagnosis, with private storage and native 7-day/2-GiB retention limits.

## What Changes

- Reproduce and repair the demonstrated structured Job lifecycle violation at its native producer/ordering owner. Preserve fail-closed validation, execution uncertainty and single-dispatch behavior; lack of reproduction is an explicit HOLD, not a reason to relax the validator.
- Retain bounded, private Job output and terminal receipts beyond in-memory tails, recover the same execution through existing Job tools, and surface truthful missing/truncated/expired evidence. Reuse native Job persistence and observation owners; do not rerun work to recover its output. The user approved a narrow offline exception: current request authentication against immutable admission ownership may recover only Job id, canonical terminal status and recorded exit outcome when current Project authority is unavailable after Server restart; no paths, logs or archive metadata are disclosed.
- Let Work Result accept the same currently authorized, unambiguous Project selectors as ordinary tools while retaining explicit Session identity and Project/Session consistency. Do not infer a Session or override a conflicting target.
- Reduce avoidable validation invalidation only where the existing source-evidence owner can prove a no-effect mutation. Unknown commands, active/asynchronous writers, external edits and unbound source remain conservative; execution PASS is never a whole-workspace certificate.
- Add a bounded preflight to the operator's native Codex worker launch path, using the selected checkout, executable, cwd and sandbox policy. Detect known path/mount admission failure before substantive work; never automatically select an unsandboxed fallback, and do not redefine generic process exit zero as task acceptance.
- After implementation and consumer qualification, build and reinstall matching Server/Runner/CLI plus Web workflow assets in the named local Linux deployment, retaining rollback and existing credentials, Projects, Sessions and evidence. This is a local dogfood release, not a GitHub/npm publication.

## Capabilities

### New Capabilities

None. Extend the existing Web workflow and evidence contracts rather than create a parallel workflow, tool registry or memory store.

### Modified Capabilities

- `web-workflow-evidence`: structured execution lifecycle correctness, bounded recoverable Job evidence, source-neutral validation recovery and authorized Work Result selectors.
- `web-workflow`: sandbox-aware native worker preflight and reproducible local deployment of the qualified fork.

## Impact

Native owners include Runner Job execution/output persistence, Runner registry lifecycle validation, existing Server Job readers and Session closeout, Project resolution/Work Result authorization, and the validation source observation fence. The optional read-only `plugins/web-workflow` provider must not acquire worker execution authority. Operator-specific preflight belongs in `deploy/web-workflow` tooling/guidance unless an inspected existing native launch owner already provides the needed seam.

Model-facing changes use canonical `ToolDefinition` contracts and current Job/Project readers; do not add backward aliases or soft links. Preserve restorable current Sessions, existing receipt schemas and authenticated access. Any additive Server/Runner contract needs explicit capability admission and safe behavior against the currently installed `0.4.3 / 004e108a7e5e` build. Acceptance uses disposable CPU projects and real Server/Runner calls; no research/GPU rerun, automatic upstream merge, push, public tag/release, ChatGPT wake-up or shared-service restart is included.

The user subsequently retained the native sandbox positive qualification gate
and explicitly requested code completion **without installation** on 2026-10-06.
Preparing and testing the candidate remains in scope; live activation requires
both the original qualification and a later explicit installation request.

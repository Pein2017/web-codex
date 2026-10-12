## Why

The 2026-10-06 WebCodex dogfood refactor completed substantial real work, but its retained Server ledger exposes avoidable execution/recovery friction. In the three implementation Sessions, 353 tool requests included two `run_shell` protocol failures with `structured_job_lifecycle_invalid` and an unknown execution outcome, three Work Result selector rejections, and execution logs whose earlier lines were no longer available from bounded Job tails. These observations justify targeted reliability and turn-economy work within the existing WebCodex Server → Runner loop, not another agent or orchestration framework.

The historical execution ran with full trace disabled, so the two protocol errors have no retained exact request/update payloads. Two ChatGPT turn interruptions have no failing MCP request at their boundaries; their Host-side cause is unknown and is outside this change. Full trace has now been enabled on the existing deployment to support prospective diagnosis, with private storage and native 7-day/2-GiB retention limits.

## What Changes

- Reproduce and repair the demonstrated structured Job lifecycle violation at its native producer/ordering owner. Preserve fail-closed validation, execution uncertainty and single-dispatch behavior; lack of reproduction is an explicit HOLD, not a reason to relax the validator.
- Retain bounded, private Job output and terminal receipts beyond in-memory tails, recover the same execution through existing Job tools, and surface truthful missing/truncated/expired evidence. Reuse native Job persistence and observation owners, with an independently hard-capped SQLite file only for archive locators under the same deployment owner; leave Session and ordinary receipt storage unchanged. Count database, auxiliary and temporary bytes within the total archive limit rather than treating retained row counts as a physical bound. Do not rerun work to recover its output. The user-approved offline exception returns only authenticated Job id, terminal status and recorded exit outcome, never paths, logs or archive metadata.
- Let Work Result accept the same currently authorized, unambiguous Project selectors as ordinary tools while retaining explicit Session identity and Project/Session consistency. Do not infer a Session or override a conflicting target.
- Reduce avoidable validation invalidation only where the existing source-evidence owner can prove a no-effect mutation. Unknown commands, active/asynchronous writers, external edits and unbound source remain conservative; execution PASS is never a whole-workspace certificate.
- Make the named workflow explicitly direct: ChatGPT calls WebCodex, and the Runner performs authorized file, Git, validation and process operations. No Codex CLI agent, sandbox probe or local-agent delegation is required or invoked by this workflow. Retire this change's local Codex preflight helper, dedicated tests, launch guidance and packaging/smoke dependencies; verify the direct edit/test/recovery loop with Codex CLI unavailable. Preserve ordinary permission and Project boundaries and the distinction between process completion and task acceptance.
- After implementation and consumer qualification, build and reinstall matching Server/Runner/CLI plus Web workflow assets in the named local Linux deployment, retaining rollback and existing credentials, Projects, Sessions and evidence. This is a local dogfood release, not a GitHub/npm publication.

## Capabilities

### New Capabilities

None. Extend the existing Web workflow and evidence contracts rather than create a parallel workflow, tool registry or memory store.

### Modified Capabilities

- `web-workflow-evidence`: structured execution lifecycle correctness, bounded recoverable Job evidence, source-neutral validation recovery and authorized Work Result selectors.
- `web-workflow`: direct Server/Runner operation without a Codex CLI dependency and reproducible local deployment of the qualified fork.

## Impact

Native owners include Runner Job execution/output persistence, Runner registry lifecycle validation, existing Server Job readers and Session closeout, Project resolution/Work Result authorization, and the validation source observation fence. The optional read-only `plugins/web-workflow` provider remains read-only. The archive-locator file reuses the existing receipt interface and authentication; it is not a second execution ledger or database service. Removing the local Codex preflight does not authorize changing the user's installed Codex CLI, container security or unrelated upstream agent capabilities.

Model-facing changes use canonical `ToolDefinition` contracts and current Job/Project readers; do not add backward aliases or soft links. Preserve restorable current Sessions, existing receipt schemas and authenticated access. Any additive Server/Runner contract needs explicit capability admission and safe behavior against the currently installed `0.4.3 / 004e108a7e5e` build. Acceptance uses disposable CPU projects and real Server/Runner calls. The user explicitly authorizes pushing the existing fork branch `coordexp/web-workflow` to `origin` and reinstalling the named local deployment after qualification. No research/GPU rerun, automatic upstream merge, public tag/GitHub Release/npm publication, ChatGPT wake-up or shared-service restart is included.

## Current scope and authority

The user's latest explicit correction is: “我从来没需要它能够拉起我的codex cli;一切web-codex -> Runner 闭环即可.” It supersedes the earlier native Codex sandbox positive gate: that route is removed from this change, not relabeled as passing. The archive-locator SQLite architecture decision is delegated to the lead and resolved in `design.md`; its physical bound still requires implementation and qualification.

The latest explicit ruling is “推送现有分支，并本机安装；不做公开 Release”. It supersedes the earlier without-installation restriction for this qualified candidate only. Activation still requires the revised direct-loop gates, pre-activation recovery qualification and an idle named deployment. `verification.md` and the sealed receipts for candidate `f78ce3d279da5d9d324c569d20da393810208f6d` retain their historical results and superseded sandbox HOLD; they are not qualification of the revised candidate.

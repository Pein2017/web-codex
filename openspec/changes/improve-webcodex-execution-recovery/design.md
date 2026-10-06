## Context

See `proposal.md` for motivation and scope. The maintained fork is `/data/CoordExp/codex-tools/web-codex`, branch `coordexp/web-workflow`, planning baseline `d9005f204cfab4cfc4b7304ccd197fe31a4c7c63`. The installed native Server/Runner are clean `0.4.3 / 004e108a7e5e`; source integration and installed identity are deliberately separate. Runtime state, credentials, rollback and verification belong to ignored `deployment/`, not tracked source or a new store.

### Evidence and existing owners

| Finding | Retained observation and owner | Limitation |
| --- | --- | --- |
| Lifecycle violation | `action_events` ids `8ed045de-c17c-486c-a83e-6adbcac504b2` and `be981a06-aca4-4859-b053-2c3b6a6cff4b`, `run_shell`, 50/51 ms; `crates/webcodex-runner-registry/src/job_updates.rs` rejects inconsistent terminal execution state | Historical full trace was off; exact arguments, offending update and root cause are unknown |
| Output loss | Five retained Job receipts start stdout above line 1, including the three original worker Jobs; current snapshot bound is 64 KiB/stream, terminal inventory 64 Jobs/24 h | Missing output alone does not prove it caused the repeated qualification |
| Selector friction | `src/tool_runtime/work_result.rs` authorizes and resolves the Project, then rejects the original spelling when it differs from canonical id | Removing spelling rejection must not remove Project/Session equality or principal-scoped reference checks |
| Source over-invalidation | `src/tool_runtime/validation_source.rs` advances generation at both potential-write start and finish, including proven no-op attempts | `crates/webcodex-core/src/validation_source.rs` explicitly has no current/fresh state and cannot observe arbitrary external writes |
| Worker admission | Original qualification record names three blocked initial native Codex worker Jobs; `coding_agent_providers=[]` on the current deployment | Those workers used generic execution, not an already configured ACP provider; no reason to add another provider framework |

The audit denominator is 353 implementation tool requests, not all 478 Server action rows (which include audit calls and UI polling). The named implementation Sessions are `wc_sess_M6B2xQMMJcoL5yQu`, `wc_sess_2Lr-nSKH62jdLOgk`, and `wc_sess_DGGnpNFu5-RifTti`; the original evidence owner is `/data/CoordExp/openspec/changes/refactor-canonical-code-ownership/verification.md`. This change does not modify that refactor, its accepted evidence or its outstanding qualification HOLDs.

Two Web conversation pauses have no failing MCP request at their boundaries, and an existing Runner Job completed during one pause. There is no Host error reason proving an OpenAI runtime limit. Neither diagnosing nor bypassing those limits is part of this change.

## Goals / Non-Goals

Goals: repair a demonstrated execution contract at its native owner; retain bounded same-execution evidence; remove target-spelling repair turns; eliminate source-fence invalidation by proven no-effect attempts; and detect worker sandbox setup failure before paid substantive work. Use existing tool, Job, Session, Project, validation and deployment owners.

Non-goals: no new TaskGroup/Goal framework, automatic continuation/wake-up, tool-pruning campaign, generic retry engine, authorship inference, whole-workspace snapshot certificate, worker permission escalation, research execution, shared-service maintenance or public release. The optional `plugins/web-workflow` provider remains read-only. Preserve current durable consumers, not unused model aliases or legacy deployment paths.

## Decisions

### 1. Diagnose the lifecycle producer before modifying acceptance

Use native full trace to correlate model request, effective request, Runner request/result and registry transition by request/Job identity. Start with a bounded real disposable Server/Runner slice for fast successful shell execution and pre-start rejection, then use the actual invalid transition to build a deterministic regression in existing Job-manager/registry tests. Investigate native update coalescing/start ordering as hypotheses, not established causes.

Fix the narrow producer or ordering seam responsible for the invalid combination; keep `validate_command_execution_state` fail-closed and preserve all existing not-started, completed, timed-out and outcome-unknown meanings. Test one side-effect marker to disprove duplicate dispatch. If the historical defect cannot be reproduced or linked to a qualified correction, retain a protocol-repair HOLD and do not call all five optimizations complete. Diagnostic-only shipping requires a separate explicit scope ruling.

Alternative: accept more status/state combinations or relabel unknown outcomes. Rejected because it hides protocol damage and can authorize duplicate effects. Merely adding a sleep or retry is not a repair.

### 2. Extend native Job evidence, not full trace or a second execution store

The Runner Job/output owner captures exact decoded stdout/stderr before live-tail trimming into bounded private files keyed by existing Job/request identity inside a stable namespace bound to the normalized Server endpoint plus Runner client id, following `detached_job.rs:274-302`. The original transient Runner-instance id is provenance, not the archive namespace or a restart access requirement. Reuse current durable-file ownership, atomic terminal-write and safe path handling patterns without changing ordinary Jobs into detached Jobs or modifying the detached execution-control schema. Full trace is a separate operator diagnostic, not the durable Job reader backend. Sanitized diagnostics and exact retained output are distinct: no secret-free guarantee or silently rewritten archive is implied.

Proposed operator bounds: 16 MiB per stream, 1 GiB total archive across all namespaces of this named deployment (including active files, temporary files, receipt/index overhead and concurrent reservations), at most seven days after terminal subject to earlier quota eviction, at most 256 retained terminal archive entries, and existing model-response ceilings. Use shared owned-root quota accounting, not a separate 1-GiB allowance per Job or Runner namespace. Keep active Job memory/concurrency and snapshot limits unchanged. Stop retaining additional bytes when a bound is reached, continue draining child pipes, and record the retained range and loss reason. Archive storage failure remains separate from the actual child exit result. A receipt is called durable only after its terminal commit succeeds; partial output never supplies a terminal status.

Current Job readers (`observe_jobs`/`job_tail` as appropriate to their canonical contracts) use the same authority checks and executor identity to return older archived output in bounded pages. Add an explicit exact-Job historical lookup when the live registry lacks that id: the existing Server terminal-receipt owner (`crates/webcodex-runner-registry/src/receipts.rs`, `JobReceiptStore`, root SQLite adapter `src/job_receipts.rs`) retains the small authenticated archive locator and canonical terminal evidence under bounded archival retention. Keep that archival lookup separate from active lifecycle restoration and its 24-hour retention; do not pretend an expired receipt is active or hydrate all 256 archive entries into the 64-entry Runner terminal inventory. The stable named Runner resolves the corresponding owned archived bytes; an offline/unavailable Runner means output unavailable while committed Server terminal evidence stays readable. Every metadata/output lookup rechecks current visibility, auth partition and pinned Project/root identity before returning metadata or requesting bytes. The dedicated three-field offline terminal-fact exception below does not return metadata or request bytes. An unknown id cannot trigger cross-Runner scanning or guessed authority.

Reuse current line-position/continuation machinery; do not expose arbitrary filesystem paths, a new generic cursor vocabulary or a new top-level log tool. Server records remain small; do not put full output blobs into SQLite or Job inventories. Ordinary receipt pruning cannot silently remove an archive locator that remains within its own bound. Test exact-id read after more than 64 terminal Jobs, simulated age beyond 24 hours and Server/Runner restart, plus unavailable Runner and revoked/replaced Project. Retention is an upper bound subject to quota, not a seven-day availability promise.

On restart, recover committed terminal evidence for the original execution and explicitly distinguish it from live process control. An ordinary Job interrupted by Runner death stays lost/unknown; archive presence cannot imply restart survival. Old receipts without an archive remain readable under their existing contract. Current visibility and pinned Project incarnation apply to every archive metadata/output read, not merely at archive creation. Quota cleanup operates only on verified owned terminal archives and cannot delete active Jobs, unrelated trace/config/state or sealed research receipts.

**User-approved offline terminal-fact exception (2026-10-06):** a fresh Server
has no current Runner Project inventory while the Runner is offline. An old
catalog cannot distinguish unchanged and removed/replaced offline Projects.
In that specific unavailable-authority case, reuse normal current request
authentication and the existing immutable receipt admission partition/owner to
return a dedicated minimal projection containing only `job_id`, canonical
terminal `status` and recorded `exit_code` (including null). Do not return a
general `ShellJobInfo` with blank tails: Project/root/path, Session, command,
error text, archive descriptor, sizes, timestamps and output remain withheld.
No archive read is dispatched, no deadline is renewed, and no process control
or replay authority is created. Online evidence of revocation/retargeting still
denies the archive lookup. Once current Project authority is restored, all
archive metadata/bytes require the ordinary visibility and root-incarnation
checks. The user accepts that a minimal historical fact can remain visible to
its authenticated admission principal even if the offline Project changed.
Existing 24-hour/64-receipt behavior is not weakened or extended in place.

Alternative: enlarge in-memory tails or use Server full trace as the archive. Rejected: neither supplies a bounded same-Job durable contract across restart, and trace availability/authority differs from Job access. A separate artifact/task ledger is unnecessary.

### 3. Resolve selectors once, then retain exact semantic equality

Remove Work Result's extra original-spelling comparison after `resolve_project_input_for_auth`; use its resolved canonical id for all later logic. Keep `authorize_session_target`, explicit business `session_id`, summary Project equality, current visibility and principal-scoped stale-reference rejection. Test canonical id, a freshly issued `project_ref`, unique short name, ambiguous name, another principal's ref, revoked/retargeted ref and explicit cross-Project Session conflict through the real caller path.

Alternative: infer Project or Session from recent Window activity. Rejected as a new authority/continuity rule. No automatic correction of conflicting explicit inputs.

### 4. Refine potential-mutation observations without a source certificate

Keep the existing source fence and its `unproven`/`stale` vocabulary. Separate in-flight potential-writer accounting from completed mutation generation: admission makes the observation non-quiescent; source-effect completion or uncertainty advances mutation generation. Completion, generation advancement and active-count decrement are atomic under the existing per-Project lock. Never decrement or restore a prior generation, and never erase uncertainty from another writer. A validation observed during an active writer remains non-quiescent/unknown even if that writer later proves no-effect. Abandoned/unknown guards advance conservatively and retain uncertainty.

No-effect means **no relevant source write ever began**, not net-zero final state, restored content or successful rollback. Admit only an internal proof minted before the first relevant write, initially in a small named set of guarded-edit/admission rejection paths; all other outcomes default conservative. Do not undertake an effect taxonomy across every tool. A concrete retained counterexample is `src/tool_runtime/files/mutations.rs:4887-4893`: an accepted write-failure fixture returns `state_changed=false` after parent creation was rolled back, although a concurrent validator could have observed that mutation. Generic serialized `state_changed=false`, `command_completed=true`, tool classification or exit zero never supplies no-write proof. Unknown shell/format/commit effects, partial failures, asynchronous handoff, cancellation and missing outcome remain conservative. `git_commit_paths` gets no blanket exemption: hooks and filters can change files. A broader content-bound certificate is explicitly deferred; do not add full-tree hashing or a new manifest system in this change.

Tests must reject the nearest wrong shortcuts: trusting every `state_changed=false`, exempting write-then-rollback overlapping validation, ignoring every commit, rolling generation backward after a concurrent real writer, or clearing uncertainty after a no-op. Replay/restored old source observations remain unproven on epoch loss. A proven no-op can avoid a false stale observation, never establish current workspace coverage or suppress a needed real validation.

Alternative: bind every test to a complete workspace manifest. Rejected for this iteration because it creates a much larger source/dependency/cost contract than the measured friction warrants. Leaving all invalidation unchanged is safe but retains avoidable no-op repair turns.

### 5. Use a small operator-native sandbox preflight, not worker orchestration

Add one deployment-owned helper usable through existing `run_process`, plus matching guidance and focused tests. Initially support exactly the current local route: the explicit `/root/.local/bin/codex` installation (actual Runner `--version` reports `codex-cli 0.159.2`), its native `codex sandbox` policy construction, the selected explicit config/profile inputs and the same Runner execution context as the proposed worker. The helper does not guess a profile, search credentials, resolve model settings, supply worker prompts or launch/resume a worker. Official documentation describes the native helper's policy purpose: [Developer commands](https://learn.chatgpt.com/docs/developer-commands?surface=cli). Use inspected local grammar, not an assumed `sandbox linux` subcommand; no arbitrary Codex-version/policy reconstruction framework is included.

A non-model native sandbox probe checks only the selected path/readability and setup with a fixed 10-second deadline and bounded sanitized output. Invoke it through the actual Runner `run_process` caller context, including the same user/environment/namespace nesting and selected working directory; a probe run directly by Desktop or in another mount namespace is insufficient. No production-file write is needed. Reuse native policy construction from the same selected executable/config stack, including managed requirements when applicable, rather than reconstructing readable/write roots or network policy ourselves. Report narrowly that this setup/read probe passed, with its selected-input/equivalence evidence separate; unsupported routes or unprovable equivalence remain unproven and cannot greenlight a substantive launch. Do not trust host `test -r` as sandbox proof. Retain one bounded receipt binding execution context and selected inputs, recheck launch inputs, and keep it as a point-in-time observation, not a permission lease or certification of later commands.

An explicit already-authorized no-sandbox policy may be described as such, never reported as sandbox proven. Changing policy requires user direction; the helper never supplies `--dangerously-bypass-approvals-and-sandbox` or another fallback. Leave generic `run_process` semantics unchanged. Worker handoffs must retain process terminal evidence separately from native blocked/incomplete output and consumer/task acceptance; no string-matching acceptance classifier for arbitrary executables.

Alternative: automatic unsandboxed relaunch or an ACP provider installation. Rejected: the former broadens authority and the latter solves a different problem. A preflight cannot certify later arbitrary commands, credential availability, model completion or write safety.

## Risks / Trade-offs

- Historical protocol error lacks full payloads → bounded reproduction is the first implementation gate; unexplained failure remains HOLD rather than a speculative fix.
- Persisted stdout/stderr may contain secrets → private roots/files, authenticated existing Job reads, sanitized projections and bounded retention; do not publish payloads or private config. Full trace remains operator-only.
- Archive I/O and quotas can interfere with pipe draining → bounded buffering and explicit dropped-evidence receipts; test backpressure, disk failure and output beyond caps without blocking payload completion.
- Source no-effect claims can be too weak → internal operation proof and concurrency counterexamples; preserve `unproven`, unknown effects and current-content limitations.
- Sandbox CLI/config drift → version/grammar inspection and explicit policy binding; unsupported equivalence remains unproven, not an automatic fallback.
- Current installed build predates local surface changes → package and qualify the complete reviewed source, not just cherry-picked binaries; capability-gate any additive wire fields against older Runner contracts.

## Migration Plan

1. **Completed configuration step, not candidate deployment:** enable full trace on the existing build with private configuration backup. Server-only lifecycle change caused native Runner-loop restart/re-registration; Tunnel stayed running. Real `run_process` trace `99ca05ca-2e96-4481-aaf5-8826fdf3123a` retains raw/effective arguments, Runner request/result and final response; the read-back result proves exit 0 and completed execution. Private operation receipt is retained in `deployment/verification/full-trace-20261006/`.
2. After a new apply request, first retire the archive exact-lookup/restart/rollback seam and native preflight route in the smallest disposable real caller-to-consumer slice; then expand implementation and deterministic RED/GREEN. Do not rerun original research qualifications. Keep all unresolved gates visible in the change's verification record.
3. Finalize narrow source/docs/schema checks; review owned paths and make the local reviewed source commit required for a clean dogfood build. Do not bump upstream version or claim a public release.
4. Build matching Server/Runner/CLI using the `dogfood` profile and package the complete installed guidance/Plugin/helper assets in a candidate directory under `deployment/`. Record exact source commit, target and content identities at the packaging boundary, not by repeated whole-tree inventories. **Before live activation**, use disposable state to have the candidate write representative Session/terminal/archive metadata, then open it with the retained previous application and exercise existing Session/receipt reads and pruning. Prove old-app/new-state downgrade safety without database restoration separately from mixed-version wire capability admission. A failure holds activation until the design is compatible or the user explicitly chooses a changed recovery boundary.
5. The user's later decision is to finish code but not install while native sandbox positive qualification is HOLD. Do not stop or replace the current application in this turn; later activation requires a renewed explicit installation request and the original gates. Then recheck live Jobs and in-flight work. If external work is active, defer activation rather than stop it. Follow `deploy/web-workflow/PERSISTENT-DEPLOYMENT.md`: stop only the named deployment, verify exact process exit, save consistent private state/config/registry backup, retain old application bundle, replace the single real `app/` directory and restart. Keep trace configuration enabled; never auto-restore state.
6. Verify matching clean build identities, protocol/capability alignment, current Projects and restored Sessions, Python/Skills/Plugin describe-call, authenticated Tunnel MCP routing, bounded traces, and disposable five-point acceptance. If Host descriptor staleness is actually observed, report the exact declaration mismatch and required connection refresh rather than assuming a new chat fixes it.
7. On activation failure, restore the retained previous application through the same controlled lifecycle using the already-qualified downgrade boundary; do not overwrite newer state automatically. Report rollback outcome and leave failed qualification visible.

## Advisor Review

The requested `ask-advisor` consultation used a distinct read-only `gpt-6-astra` / high child, `/root/execution_recovery_advisor`, on 2026-10-06. Ruling: **ACCEPT_WITH_CORRECTIONS for planning only**; historical protocol repair remains HOLD until reproduction or a causally qualified correction. No edits, tests, runtime probes, additional agents or service operations were delegated.

All four plan corrections were source-checked and incorporated above and into specs/tasks:

1. Pre-first-write no-effect proof only, atomic writer bookkeeping and a write-then-rollback overlap counterexample (`files/mutations.rs:4887`).
2. Exact-id archived evidence lookup after live inventory eviction/24-hour expiry; the existing Server receipt owner retains bounded authenticated locators and the stable Server-endpoint/Runner-client namespace addresses Runner bytes (`job_updates.rs:2014`, `reconciliation.rs:487`, `detached_job.rs:274`). Exact decoded evidence is not silently sanitized; diagnostic output is.
3. Candidate-written-state/previous-application rollback qualification **before** activation, without changing the detached control schema or automatically restoring state (`detached_job.rs:930`).
4. One demonstrated native `codex-cli 0.159.2` route in the actual Runner context; setup/read success is not arbitrary policy or future-command certification.

The advisor's strongest lower-cost alternative is leaving the source fence conservative and reporting unsupported preflight routes. That would sacrifice requested outcomes and needs an explicit scope ruling, not silent all-five acceptance. Reverse this recommendation to HOLD if implementation needs weak no-write inference, broader archive authority, arbitrary policy reconstruction or irreversible state changes. Advice closes the bounded planning review, not implementation acceptance or user release authority.

A bounded follow-up inspection by the same read-only advisor confirmed the
offline Project-authority conflict in native receipt access and empty startup
Runner inventory. It recommended HOLD or an explicitly approved minimal-fact
exception, not a new persisted Project catalog. The user selected the exception
above; that ruling changes only the named metadata disclosure boundary, not
the preserved native-positive/no-install gate.

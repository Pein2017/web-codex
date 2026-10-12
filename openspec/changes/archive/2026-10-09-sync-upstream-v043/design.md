## Context

See `proposal.md` for motivation and authority. Original fork HEAD is `50281db4c3e04fa474d9724b03dd3dc5b91d7ea8`; upstream release is `b96a59a712ca5355ad8609cd861cd0c7acb9f99e`. Integration uses `/data/CoordExp/.local/src/webcodex-upstream-v043`, branch `codex/sync-upstream-v043`. The original checkout and its uncommitted external-source guidance remain untouched through source acceptance; subsequent consolidation preserves and absorbs that exact guidance patch.

The merge preview reports 1,243 changed upstream files, 140 local changed files, 74 overlapping paths and 40 textual conflicts. Source parity assessment is recorded below; source resemblance alone is not acceptance.

## Goals / Non-Goals

Keep native upstream ownership and retain only bounded operator integrations or counterexample-backed local behavior. Preserve old Session data and real execution identity. Produce a source-integrated, locally checked candidate, then complete the user's subsequent explicit source-push and local-activation request. Public release/tag creation and post-0.4.3 feature selection remain excluded. Browser acceptance requires an actual ChatGPT-side call and is distinct from host/Tunnel readiness.

## Decisions

### Merge history; prefer upstream implementations

Use a normal two-parent merge in the isolated worktree. Resolve shared implementation surfaces from the pinned upstream version, then reapply only the gaps below. Retain local plugin/deployment assets and historical OpenSpec evidence. Do not mechanically preserve all auto-merged local core edits; compilation and nearest consumer checks decide whether each retained correction is complete.

| Need | Owner after integration | Evidence / nearest check |
| --- | --- | --- |
| Configured MCP guidance | Small local configuration/protocol/startup identity correction | Initialization instructions plus `work_on_project_identifies_configured_mcp_guidance_without_copying_it` |
| Automatic instruction loading | Unmodified upstream candidate policy; CLAUDE exclusion retired by user request | Native instruction-loading tests and no local exclusion switch |
| Canonical alias execution | Native resolution with corrected Runner request identity | Alias/canonical execution equivalence and mismatched Session rejection |
| Non-starving bounded read/search batches | Native caching/projection plus narrow packing/continuation correction | Oversized first item followed by small fitting item; serialized budget and omitted identities |
| Shared memory/public history/scoped graph/report | Existing optional operator plugin | Existing privacy, path, provenance, pagination and report tests |
| Skills | Native machinery plus proven configured-root/continuation corrections | Rejected project root must not hide configured skills; stale definition rejection |
| Plugin discovery/message ACK | Native discovery plus recorder-bound ACK correction only if counterexample persists | Correct recorder acknowledgment, other recorder unchanged, provider content intact |
| Dirty-path baseline | Existing bounded observation adapted to native Session types | Startup/resume/finish plus frozen local-v2 restore fixture |
| Assertion recovery/pytest counts | Native assertion recovery plus typed pytest count correction | Matching assertion recovery, malformed/truncated summaries and shell spoofing |
| Current-source validation freshness | Unmodified native unproven-source policy | Real execution pass/counts and failure resolution do not certify the mutable current workspace |
| Git hygiene | Native diagnostics plus bounded sentinel correction | Large tracked listing, actual dirty path and tracked-secret detection |
| Context ACK/revision | Native removal and inert historical-field restoration | Native retired-field restore/no-reemit tests; obsolete local overlays removed |
| ms Python/checkpoints | Operator wrapper and native compile feature | Runner Python identity; matching Server/Runner feature exposure |

Upstream `PersistedSessionRecord` denies unknown fields and restoration skips invalid rows. The local optional `workspace_baseline` field therefore needs preservation before any old-state restoration check; retaining inert `context_revision` alone is insufficient. Source acceptance never touches production persistence. Subsequent authorized activation verifies a private ledger copy first and preserves a stopped-state backup before native restoration.

### One owner per package

The lead owns Git, OpenSpec, shared integration, remaining corrections and final acceptance. Independent execution packages may own workspace baseline/persistence, batch read/search, and typed execution counts/identity on disjoint paths after the upstream reset is complete. No worker changes shared interfaces without reconciling ownership. Rust builds use one shared task-owned target and bounded jobs; the lead owns the final real-entry invocation.

### Checks follow the changed contract

Preserve upstream suites and existing focused local counterexamples. Install or retain tests before changing silent-correctness paths and prove sensitivity against the pinned upstream baseline where applicable. Run the optional plugin's existing test suite. Build matching dogfood Server/Runner with retained `workspace-checkpoints`; experimental V8 Code Mode remains opt-in. A disposable real-entry smoke must cover initialization, Skills/plugin access, guarded edits, actual Python validation, Job completion and Session finish. Use existing `scripts/e2e_web_workflow.py` and adapt obsolete wire expectations rather than creating another harness.

## Risks / Trade-offs

- Old local Session rows can be skipped by native strict parsing -> preserve the optional baseline and test a frozen pre-upgrade row through restore and serialization.
- Auto-merges can retain obsolete protocol code -> take upstream shared surfaces first, remove retired ACK references, then run compiler/contract checks.
- Similar native features can still miss boundedness or identity requirements -> retain only the listed falsifying examples, not old infrastructure by default.
- Builds or smokes can touch current deployment -> use bounded CPU-only builds, fresh disposable state/ports and task-owned logs; never point acceptance at production state.
- A smaller diff can conceal weaker validation -> report retained gaps, removed patches and exact checks; no claimed runtime or cost improvement without measurement.

## Migration Plan

Complete and check the isolated source merge, local tests and disposable Server/Runner smoke; commit the reviewed local candidate. For the subsequently authorized activation, build matching clean Server/Runner/CLI binaries and verify a private copy of the existing Session ledger. Reconcile live Jobs and requests, stop only the dedicated WebCodex service, preserve a consistent state/config/project-registry backup, switch the complete versioned bundle and restart. Verify Session restoration, canonical project access, shared Skills/plugin context, Runner Python, Tunnel readiness and current MCP identity. Preserve the old release and credentials.

Consolidate into the original canonical checkout `/data/CoordExp/.local/src/webcodex-mcp-instructions` by fast-forward only after preserving its exact dirty guidance patch and verifying that content is carried into the accepted candidate. Push the accepted commit to existing fork branches only when their current remote tips are ancestors; never force push. Retire the clean integration worktree only after its content and receipts are preserved. Refresh ChatGPT developer-mode metadata and use a new conversation for browser-side verification; no browser success is inferred from a local health check.

## 1. Canonical routing and bounded results

- [x] 1.1 Reproduce the alias recovery-context failure at the execution caller boundary, normalize the resolved identity, and prove alias/canonical equivalence plus wrong-Session/wrong-Runner rejection with focused Rust tests.
- [x] 1.2 Reproduce oversized-first-item zero progress; repair bounded read packing and exact omitted/partial follow-ups. Verify oversized+small, all-oversized, revision-guard and exact serialized-budget cases in existing read_files tests.
- [x] 1.3 Verify the three-query partial-search continuation returns exactly unfinished requests; adjust misleading zero-progress projection if reproduced, retaining producer truncation and search output-schema tests.

## 2. Instruction policy and useful evidence

- [x] 2.1 Implement one operator CLAUDE exclusion setting across instruction loaders, startup and project.instructions sidecars; verify enabled/unset settings, AGENTS preservation, intentional exclusions versus unavailable reads, and unchanged explicit file access in focused tests.
- [x] 2.2 Improve scoped CodeGraph discovery and guidance using the existing plugin. Verify source256-shaped fixtures exclude irrelevant worker paths, preserve freshness/incompleteness, and do not reindex automatically via `npm test --prefix plugins/web-workflow`.
- [x] 2.3 Add the optional bounded public-history reader to the existing plugin. Verify native-protocol discovery/read/pagination, dual record representations, public-only filtering, redaction, oversized/malformed records, append/replacement detection and root/symlink rejection with synthetic fixtures through the same plugin test entrypoint. Initially reopened by lead audit, then repaired directly by the lead; native layout/large sources, fail-closed public admission, occurrence preservation and append-at-EOF regressions now pass. See acceptance.md for bounded identity/deduplication limitations.
- [x] 2.4 Update Web guidance and operator documentation with the actual exclusion/history settings, resolved project usage, scoped retrieval, explicit recorder/ACK and partial-result continuation; review examples against actual schemas without adding automatic retries or claiming host-filter elimination.

## 3. Acceptance and implementation handoff

- [x] 3.1 Run affected formatting/package checks, focused regression tests and `openspec validate improve-web-research-tool-path --strict`; record exact commands, RED/GREEN evidence and remaining limitations in this change's acceptance.md, without copying private traces.
- [x] 3.2 Lead reviews the exact diff and replays the identity and oversized-batch consumer tests plus native-plugin checks; record lead acceptance separately from Luna's candidate report. Verify only authorized fork files changed and no push/deployment/restart/research mutations occurred.

Implementation assignment after planning approval: one `gpt-5.6-luna` agent at `max`, owning the complete package in this checkout, no further delegation. Read this change and applicable repository instructions; do not regenerate planning. Stop and ask the lead on a contract conflict, new authority requirement, or dependency requiring a broader architecture. Return changed paths, exact tests and unresolved limitations. Lead retains acceptance; the worker does not publish or operate live services.

## 1. History package — single implementation owner

- [x] 1.1 Reproduce missing latest mode and eager-read amplification through native-protocol tests in plugin.test.mjs; record RED results before production edits. Worker RED: latest rejected; ten forward pages read 3,798,216 bytes, exceeding 1 MiB. Lead acceptance remains separate.
- [x] 1.2 Implement latest snapshot pagination and incremental forward/backward reads in plugin.mjs; verify newest selection, chronological page order, older continuation, byte provenance, cursor mode conflict, append snapshot behavior and v2 forward compatibility with focused tests.
- [x] 1.3 Verify boundary/privacy/output invariants (UTF-8/chunk/page splits, incomplete tail, malformed/oversized records, sparse empty-page progress, redaction, ambiguity, replacement/truncation, symlink rejection); require all plugin tests pass and ten-page amplification fixture stays within 1 MiB.

Owner: one Terra/high worker; only plugins/web-workflow/plugin.mjs and plugin.test.mjs. No delegation. Existing helpers/parser and accepted dirty baseline are the starting point. Ask lead immediately if frozen cursor/coverage semantics conflict with implementation; do not invent a framework. Report first RED, boundary decisions and final exact commands. Return candidate, not acceptance.

## 2. Guidance package — disjoint documentation owner

- [x] 2.1 Update deploy/web-workflow/AGENTS.md, deploy/web-workflow/README.md and plugins/web-workflow/README.md for list/describe/binding reuse, diagnostic-only check, latest/forward cursor semantics and byte/line provenance; verify examples against this design and final tool schema, without claiming deployment or automatic polling.

Owner: one Luna/medium worker; only the three named Markdown files. May proceed alongside package 1 using the frozen design. Do not edit code, tests or OpenSpec. Report any runtime/schema mismatch to lead; do not silently change the contract.

## 3. Lead acceptance — no second scheduler

- [x] 3.1 Inspect exact worker diffs and replay npm test --prefix plugins/web-workflow, node --check for both JavaScript files, and independent latest/order/append/bytes diagnostics; record deterministic evidence and distinguish local from deployed state.
- [x] 3.2 Run git diff --check and openspec validate reduce-web-history-round-trips --strict; reconcile guidance/schema, preserve preceding dirty changes, update task status and acceptance receipt. No commit/push/build/restart/live config/reindex/GPU/research mutations.

The lead alone owns this change directory and final acceptance. User explicitly authorized autonomous implementation after proposal drafting; no further planning gate. Defer CodeGraph concurrency, describe protocol fusion and broader performance work. Stop when these checks pass; no extra reviewer, full Cargo suite or durable monitor is required for this bounded work.

# Acceptance record

Status: lead-accepted implementation and persistent local deployment; clean build source pushed atomically to Pein2017/webcodex main and coordexp/web-workflow.

## Trace assessment

Source: user-uploaded `tool-call-data (3).json`, 46 calls. This record intentionally excludes raw research code, command output and opaque Session/binding ids.

- Skill: one successful complete definition read; no Skill ID schema rejection.
- CodeGraph: three successful scoped calls, returning 5/5/1 symbols with complete/current index reports. One earlier describe used `client_id` instead of `runner`, then recovered.
- Execution: one shell timeout confirmed by the Session ledger, although its direct exported output is null. The script compared long AST strings pairwise; no timing profile proves its exact hotspot. Do not infer disconnect or safe retry.
- Continuity: 11 unacknowledged hints; one explicitly partial handoff cannot establish a baseline. Recorder and business Session fields are inconsistently supplied, including one malformed read Session string. No authority exploit is demonstrated by the export.
- Completeness: one batch read hits the response budget; eight read calls have at least one partial file and a suggested continuation. Partial files do not by themselves require reading every remaining line. One large process diff has stdout truncation. Later targeted reads show recovery of some relevant excerpts, so missing full-file continuation is not by itself proof of an invalid final review.
- No final review or Session close is in the export. Do not infer a leak, completed audit, or scientific conclusion.

## RED evidence

- `/tmp/webcodex-discovery-recovery-red.log`: exact complete-handoff argument assertion fails against prior producer (1 failed).
- `/tmp/webcodex-discovery-catalog-red.log`: startup selected and configured Skill follow-ups absent (2 failed).
- `/tmp/webcodex-discovery-plugin-red.log`: exact Runner/provider describe follow-up absent (1 failed).
- `/tmp/webcodex-discovery-e2e-red/receipt.json`: old dogfood real stateless MCP fails at startup Skill follow-up project/revision check, after normal Runner/project startup succeeds.

## Ownership

- Terra recovery: session_context producer + focused contract assertion; stable/race/recorder guards retained.
- Terra discovery: startup Skill/Plugin producers and catalog/consumer tests.
- Lead: guidance, real MCP test extension, exact diff review, serialized Cargo verification, fork publication and local deployment.

No nested delegation, new controller, timeout waiver, auto-ACK or automatic indexing.

## Focused GREEN

- Startup/work_on_project: 38 passed; catalog byte/prefix/UTF-8/unavailable contracts: 4 passed.
- Session context suite: 14 passed, including real HTTP returned-suggestion recovery and stable/race/recorder fences.
- Lead corrected two historical exact-shape assertions revealed by the broader focused recovery run (12 pass/2 fail before correction); it now replays returned arguments rather than a separately handcrafted default handoff.
- Rust formatting, Python compilation and strict OpenSpec validation passed. Live deployment and the updated disposable binary E2E remain pending.

## Deployment amendment

The user explicitly added /data-only persistence while verification was underway. Read-only process/dependency audit preserved all services. Existing binaries, native Node/CodeGraph and Tunnel client are already persistent; operator config, credentials, SQLite/Session state and receipts are not. System Git 2.34.1 rejects `check-attr --source=HEAD`; existing ms Git 2.55.0 passes. System Python 3.10 can run the stdlib JUnit helper.

The preceding `.09.16.2` installation omitted `pytest_report.py` despite source retaining it unchanged. The new bundle must include it and verify actual parsing. A separately named rollback bundle may retain the old binaries with that exact unchanged helper; the original release remains untouched.

## Final deployment evidence

- Clean source `b3baff24db1858309c1d42a802d7e595ea5b13e3`, dogfood release `coordexp-2026.09.17.1`; Server/Runner aligned, four projects/four providers online.
- Disposable real MCP: 31 checks passed. Final live MCP verifies guarded Skill/Plugin suggestions, complete recovery, Skill read, scoped CodeGraph (five results including expected symbol), actual JUnit parsing (one each pass/fail/error/skip; no test execution). Own Sessions closed.
- Independent deployment falsification verified invalid/incomplete releases, live-switch refusal, dependency admission, duplicate-start prevention and child shutdown. Lead fixed stopped-status reporting and replayed it.
- Persistent root `/data/CoordExp/.local/webcodex-custom`: current/release split, private config/credentials, independent state, logs/verification/rollback, native dependencies, standalone Git 2.55.0 and recovery entrypoint. Source location unchanged; image prerequisites documented.
- Quiesced backup and exact 42-Session copy verified; SQLite quick_check passed. Original IDs/non-title records survive rollback. One overlong title is bounded by existing restore sanitization; original remains in backup. No old data removed.
- New -> old -> new rollback and clean-environment startup without /root configuration passed. Active Tunnel /readyz=200; clipboard-dependent ready_for_chatgpt is not the connectivity assertion.
- Private receipts: persistent `verification/coordexp-2026.09.17.1/{migration.md,live-final.json,e2e/receipt.json,artifacts.sha256}`. Research processes/proxy/unrelated dirty work preserved. Canonical hygiene findings remain user-owned, not a clean-project claim.
- Publication scope: user's fork and local dogfood, no npm/GitHub Release. Browser wrapper cache/selection and browser invocation remain separate user verification.

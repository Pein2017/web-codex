# Acceptance receipt

Status: 2026-09-16 source follow-up lead-accepted, fork publication and local activation passed; separate tracked-token-path classifier warning remains.

## Source and scope

- Initial checkout: `coordexp/web-workflow` at `47bf37df`; fork `Pein2017/webcodex`, upstream `yyjeqhc/webcodex`.
- Planning: `6e7cfd3e`; guidance: `edaa9fe0`. Strict OpenSpec validation passed before implementation.
- Upstream fetched at assessment: `6a1433d0`, four incoming commits; refreshed to `01dc7d31` (one additional execution-guidance update) and merged as `0c660be3`. Seven ordinary overlapping files were reconciled: compact IDs, under-lock allocation, additive baseline fields, fixture identity, opt-in guidance prose, and the existing bounded SSH shutdown check. No authority or instruction-default change was accepted.
- Pre-upgrade deployed baseline observed: clean matching Server/Runner `b0edfa887a83`, four online Projects, no active Jobs.

## Evidence retained

- Live stateless MCP discovery declares all five script languages. Existing source implements `read_revision`-to-SHA guarding; no second fence/schema registry added.
- Pytest RED: validation evidence expected `tests_detected=true`, got null; generic execution expected true, got false. Both reached the target assertions and exited 101 on the pre-fix implementation. GREEN: parser 3, validation/recovery 5, typed execution 5, output-schema 1 tests passed under Rust 1.95.0. Implementation commit: `3f07e8ab`.
- Baseline GREEN: persistence/resume/legacy 2, real startup/finish and bounded comparisons 6, existing startup projections 6 tests passed. Lead reviewed persistence, raw Git capture, completeness and public projection boundaries. Implementation commit: `502a1f1f`.
- Existing optional Native Plugin suite: 11 passed, 0 failed. Its source/dependencies are unchanged. Raw log remains operator-local under this deployment's verification directory.
- Guidance: 5371 UTF-8 bytes (below 16 KiB); initial Markdown local-link check passed (88 files, 529 links, zero missing targets).
- First integrated root run: 2655 passed, 2 failed, 2 ignored. One fork-only Skill fixture still used the upstream-retired hexadecimal ID; its isolated failure reproduced, then passed with the actual compact ID format and unchanged behavioral assertions. The other failure exceeded existing startup budgets by 41/40 bytes; initial prose shortening then required preserving upstream's literal ordering/role checks. Both original budgets and behavioral assertions were retained in the final GREEN below.
- The four implementation package suites passed before merge: Core 252, Tool Contracts 132, Validation 97, Workflow Session 178. Post-merge optional plugin tests passed 11/11; strict OpenSpec and local Markdown links passed (90 files, 529 links).
- Integrated suites subsequently passed 4537 tests (6 intentionally ignored). The final guidance keeps upstream's tested ordering/role language and saves 50 bytes without raising startup budgets. A production-only build failure exposed a test-gated constant re-export; `5a5b9949` corrects that visibility, and the clean release build passed.
- Real stateless MCP evidence passed JS, revision-guarded edit/stale rejection, baseline comparison, synchronous fail/fix/pass, and async pytest Job counts. It then exposed missing synchronous PASS counts in the outer Session ledger despite correct public counts. A matching kernel caller fixture reproduced this (exit 101), then passed after deferring process/script compaction until after the canonical outer recorder. No inferred exit code, new wire field, hidden recorder, or assertion-budget relaxation was introduced. Rebuilt real-MCP GREEN is recorded below.

## Final candidate acceptance

- Clean binary source: `69f0273dcf80e1232f7a22e0e7f77c5778d1c447`, upstream `01dc7d31` included. Both release binaries report `dirty=false`, version `0.4.1`, and the same source/built-at identity.
- Server suite after the recorder fix: 2658 passed, 2 ignored. Unchanged package/Runner suites contribute 1880 passed, 4 ignored: total 4538 passed, 6 intentionally ignored. Optional Native Plugin tests: 11 passed.
- Real stateless MCP smoke: 25 checks passed in 7.22 seconds (7.369 seconds including cleanup), fixture resources removed. Operator-local receipt: `verification/coordexp-2026.09.15.2/e2e-owned-async/receipt.json`.
- Async Job ownership remains explicit: a supported business `session_id` is required in addition to `recording_session_id` for Session-owned closeout. No runtime authority change was made. The synchronous regression remains outer-recorder-only.
- Server SHA-256: `93336d8409e13cd59e9bbbdade9e493195db6d02f048eb25ff39c0f799e4ce6e`.
- Runner SHA-256: `04fa884fed87bbaece083c1e6546436e2c720837e1ba852c5bce00f3fb2baa74`.
- Final operator guidance: 5527 UTF-8 bytes, SHA-256 `30fc7b623263919e2ecf8e225b0e7dbef405733d9bf9604348ca64b5988c05e2`. The final guidance/smoke clarification does not alter compiled Rust source.

## Publication and local activation

- Reviewed history was fast-forward pushed atomically to `Pein2017/webcodex` branches `main` and `coordexp/web-workflow` at `e345afbb`; this acceptance closeout is a subsequent documentation-only commit. No force push or upstream publication.
- Activated local release `coordexp-2026.09.15.2` in the existing Server/Runner tmux windows. Existing Tunnel process remained untouched; its health endpoint returned HTTP 200.
- Stopped-state Server data, private config and old wrappers were backed up before activation; backup SQLite `quick_check=ok`. Previous `.1` binaries remain. Backup retained operator-locally at `rollback/coordexp-2026.09.15.2`; a rollback was not executed.
- Live stateless MCP acceptance passed: both binaries `69f0273dcf80`, `dirty=false`, source `aligned`; all four existing Projects online; CodeGraph plus three Web workflow providers ready; initialize/discover return the expected 5527-byte guidance. A new main-project Session returned a complete clean startup baseline and was then closed.
- Production repositories and research processes were not modified or restarted by the release workflow. No npm, container, desktop, or GitHub binary release was published.

Path/status comparison is not content auditing or Session authorship; no browser
automatic wake or ChatGPT UI acceptance is claimed.

## 2026-09-16 real Git-task feedback follow-up

Scope: reproduce the reported protocol symptoms, repair proven Server defects,
and strengthen the existing Web guidance and real MCP checks. Do not modify,
merge, or commit either research worktree based on the feedback's Git inventory.
The earlier release evidence above remains historical, not proof of this delta.

### Reproduced and distinguished

- Deployed `69f0273dcf80` stateless MCP advertises 152 tools. Live
  `tools/list` and `tool_manifest(skill_read_file)` agree on the compact opaque
  Skill ID pattern; the direct surface adds only its legitimate wrapper metadata.
  `git_diff_summary` and `call_runtime_tool` are not advertised on this Full
  Operator surface. The reported old hexadecimal-only host wrapper does not
  match current Server discovery. No compatibility alias or second schema
  registry is justified by that stale host declaration.
- Actual disposable Server/Runner Skill discovery followed by direct
  `skill_read_file` succeeds with the returned ID and definition revision.
  Missing required `skill_id` produces a compact server-side JSON-RPC error,
  not a complete JSON Schema. This does not claim control of ChatGPT's own
  pre-dispatch wrapper validation or prove the browser has refreshed its cache.
- Explicit recorder, complete handoff recovery, and two successive process calls
  echoing the retained revision succeed without missing-recorder or unacknowledged
  hints on those calls. Business `session_id` alone is not recorder identity;
  neither identity proves retained model context. Preserve independent authority
  checks and do not implement inferred binding or automatic ACK.
- Real MCP RED: a clean disposable Git repository with 1,100 long ordinary
  tracked filenames returns a fabricated `(worktree) dirty_worktree` finding
  with `diagnostic_output_truncated`. The Runner's retained-tail marker and
  truncated tracked-path fragment were interpreted as unframed porcelain status.
  This is a Server defect, not browser cache.

Operator-local raw evidence is retained under
`/var/lib/webcodex/coordexp-full/verification/coordexp-2026.09.16.1/`:
`e2e-current-contract/receipt.json` (27 checks passed before adding the large
inventory counterexample) and `e2e-skill-required-and-hygiene-red/receipt.json`
(Skill error check passes, then the hygiene counterexample fails).

### Final candidate acceptance

- Fixed the tracked-path scan using an early broad candidate filter in the
  existing `sed` stage; Rust remains the exact secret-path classifier. No new
  executable dependency or hidden file-content reads were introduced.
- Parse the Runner's truncation marker and following partial line correctly;
  neither is Git porcelain. Preserve real dirty findings and tracked secret
  candidates that previously fell outside the retained output tail.
- The existing 500-entry internal cap now marks an incomplete scan explicitly
  (`diagnostic_scan_incomplete`, `truncated=true`, `clean=false`), including a
  late secret behind many weak candidates. The output-schema description
  reflects that existing Boolean's additional completeness cause.
- Caller-facing pre-fix RED: exit 101. Final `hygiene::` tests: 36 passed;
  the included `workspace_hygiene_check` subset: 22 passed. Direct MCP
  Skill/manifest schema parity and compact missing-ID error test: 1 passed.
  Raw logs are retained in the same operator-local verification directory.
- Lead replay: real stateless MCP Server/Runner smoke passed all 29 checks in
  7.40 seconds, including the exact large-inventory counterexample, Skill read,
  recorder/ACK sequence, existing guarded edits, pytest recovery and Job evidence.
  Receipt: `e2e-final/receipt.json`. Disposable process groups and fixture were
  cleaned up by the harness; no production repository was a mutation target.
- Verification build: Rust 1.95.0, `dogfood`, both binaries identify base
  `bc74dd334a88`, `dirty=true`. This is an uncommitted development candidate,
  not a clean release. Exact Rust source/test diff SHA-256:
  `e38cbdc7ff11c6586f6dbc54f6672ed734481aa206fd501ccb8dcd2ed79203e7`.
  Binary paths and SHA-256 are in the smoke receipt.
- Formatting, Python syntax, Git whitespace, strict OpenSpec validation and
  Markdown local-link validation passed. Web guidance is 6,710 UTF-8 bytes and
  now gives the exact recorder and retained-ACK recovery sequence while warning
  against stale wrappers, guessed aliases and unadvertised gateways.

The user subsequently authorized publication to the existing fork and replacement
of the named local Server/Runner. Activation evidence is recorded below.
ChatGPT's cached direct callable schema still needs host-side refresh
and browser acceptance; a Server-side pass cannot certify that refresh.

Adjacent pre-existing behavior, outside this fix: failed/truncated untracked-file
size probes are not fully reflected in hygiene completeness. This patch does not
claim to make the entire hygiene scanner a security audit, and does not alter
that separate path. No research Git history or dirty work was changed.

### Authorized publication and activation

- Fetched `origin`; both target branches still equaled `bc74dd33`, so no merge
  or history rewrite was needed. Committed the eight scoped files as
  `81d82121f2e2597d319f9b7b06fe25dc0c00ad72` and atomically fast-forward pushed
  to `Pein2017/webcodex` branches `main` and `coordexp/web-workflow`.
  This receipt update is a subsequent documentation-only commit.
- Clean `dogfood` Server and Runner build passed; both report `81d82121f2e2`,
  `dirty=false`, `built_at=1789523100`. The expanded disposable real MCP smoke
  passed all 29 checks again in 7.26 seconds (`e2e-clean/receipt.json`).
- Activated `coordexp-2026.09.16.1` at the existing local deployment. Prior
  `.2` binaries remain intact. No npm, tag, GitHub binary release or container
  publication was performed; this is a local reviewed development build.
- Immediately before shutdown, the Job inventory had zero matched Jobs and
  was not truncated. Only the verified old Server/Runner and retry wrappers
  were stopped. Consistent stopped Server data and prior private configuration
  are retained at `rollback/coordexp-2026.09.16.1`; SQLite `quick_check=ok`.
- Existing tmux Server/Runner windows were recreated with the same working
  directory. Tunnel wrapper PID `1930425` was untouched and its health endpoint
  returned HTTP 200. Research processes and repositories were not modified.
- Live receipt `live-acceptance.json`: matching clean Server/Runner, source
  aligned, four registered Projects online, CodeGraph and all three workflow
  plugins ready, updated 6,710-byte guidance served by initialize/discover,
  actual Runner-scoped `git-hygiene` Skill read successful. The acceptance
  Session was closed afterward.
- Live main Project Git state is clean and hygiene returns no `dirty_worktree`
  finding and no truncation. Its overall hygiene verdict is still non-clean:
  the existing path-only classifier flags 16 tracked source/test files below
  `reference/legacy_src/tokens/` and `tests/tokens/` as sensitive paths. This is
  a separate classifier limitation, not a recurrence of the truncated-output
  dirty bug; no file contents were inspected or classifier rules changed.
- Deployment checksums, clean source identity, verification and rollback paths
  are recorded in the installed `RELEASE.json`. Browser-side cached callable
  declaration refresh remains outside this server-side acceptance claim.

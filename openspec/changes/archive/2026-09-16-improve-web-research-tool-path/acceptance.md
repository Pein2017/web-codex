# Acceptance receipt

Status: lead-accepted for the local implementation; all 9 tasks complete.
The initial candidate was held for the four public-history failures in
lead-audit.md. The lead repaired those directly, without another worker or
delegated review. The original worker report is retained below as historical
provenance, not a claim that its initial candidate passed the lead audit.

## Lead correction and acceptance — 2026-09-16

The lead kept the authorized alias resolver, batch revision guards, scoped
CodeGraph route and instruction-policy fixes. Direct production edits after
the audit were confined to the existing history plugin and its documentation;
regressions were added to its existing native-protocol suite.

### Falsification and repair

Before the lead repair, the three new tests selected by
`node --test --test-name-pattern='native nested|fails closed for unclassified|resumes from EOF' plugins/web-workflow/plugin.test.mjs`
all failed: native nested large source inaccessible, synthetic nonpublic marker
exposed, and EOF cursor absent. The separate initial audit also reproduced loss
of two identical independent user turns. The corrected suite includes these
cases and preserves both occurrences with distinct source identities.

- Filename-only discovery handles native dated rollouts and flat exports,
  bounds traversal to 8192 entries, rejects ambiguity and uses existing root
  confinement. The synthetic native fixture is 122 MiB; no total-size rejection
  or production-history copying is needed.
- Admission now allows only the observed public event/message shapes and typed
  public text. Unknown assistant channels/content no longer default to final.
- Removed speculative nested-text extraction, session-ID aliases and text-hash
  deduplication. Only explicit message IDs suppress duplicate representations
  within a bounded 20-ID window. ID-less records remain distinct; potential
  ID-less dual representations are intentionally not guessed away.
- EOF retains a cursor and incomplete trailing records wait for append. Large
  records advance using bounded discard state; malformed terminated records
  produce omission receipts. A 48 KiB serialized-message budget preserves
  continuation even for heavily JSON-escaped text.
- Removed full-prefix hashing. A single open descriptor supplies the scan and
  fixed head/tail fingerprints: at most 512 KiB scan plus 16 KiB identity reads
  per call. Device/inode, observed size/mtime and bounded fingerprints detect
  tested replacement/truncation. Arbitrary same-inode interior rewrites followed
  by growth are outside the documented append-only-source contract.
- Removed the duplicate CodeGraph guidance bullet that confused resolved
  runtime IDs with operator-configured plugin project IDs.

### Real-source bounded smoke

One direct local plugin call selected the exact known thread under the existing
`.codex/sessions` root, without changing live configuration. It resolved the
native dated filename and returned one public user message with a continuation:
`isError=false`, `bytesScanned=106309`, `bytesRead=532480`, six records scanned.
Only path, counts, role and cursor availability were emitted to the audit;
message bodies were not printed or copied into repository records. This proves
the local plugin entrypoint against the real layout, not browser deployment.

### Verification and release boundary

The lead independently replayed alias (1), read-batch (15), search-batch (12),
session instructions (7), workspace instruction policy (1), core instruction
policy (9), and explicit CLAUDE file-read (1) tests. Native plugin protocol tests
now cover 23 passing cases, including escaped-output continuation, ambiguity,
session-metadata mismatch and inode replacement. `node --check` on both plugin
files, `cargo fmt --all -- --check`, `git diff --check`, and
`openspec validate improve-web-research-tool-path --strict` all passed.
No full-library rerun was used to paper over the historical unrelated failure
described below.

No commit, push, release build, restart, live config edit, reindex, GPU work or
research-worktree mutation is included. The optional history root and CLAUDE
exclusion still require a separately authorized deployment. Credential-text
redaction remains heuristic, not an assurance that all possible secrets are
recognized. No new scheduler, generic parser framework or reviewer was added.

## Original worker candidate report (historical)

## Source and scope

- Checkout: `coordexp/web-workflow`, baseline `ab089186`.
- Implemented tasks 1.1–3.1 only. Task 3.2 remains intentionally unchecked.
- No commit, push, deployment, service restart, live configuration change,
  reindex, GPU work, or research-worktree mutation was performed.
- The existing untracked OpenSpec plan was preserved; only its task checkboxes
  and this receipt were added or updated.

## Changed files

Implementation and tests:

- `crates/webcodex-core/src/project_instructions.rs`
- `crates/webcodex-workspace/src/project_context.rs`
- `deploy/web-workflow/AGENTS.md`
- `deploy/web-workflow/README.md`
- `plugins/web-workflow/README.md`
- `plugins/web-workflow/plugin.mjs`
- `plugins/web-workflow/plugin.test.mjs`
- `src/tool_runtime/cargo.rs`
- `src/tool_runtime/files/inspection.rs`
- `src/tool_runtime/process.rs`
- `src/tool_runtime/read_files.rs`
- `src/tool_runtime/script.rs`
- `src/tool_runtime/search_project_texts.rs`
- `src/tool_runtime/shell.rs`
- `src/tool_runtime/startup_brief.rs`
- `src/tool_runtime/tests/files.rs`
- `src/tool_runtime/tests/process.rs`
- `src/tool_runtime/tests/sessions_instructions.rs`

OpenSpec records:

- `openspec/changes/improve-web-research-tool-path/.openspec.yaml`
- `openspec/changes/improve-web-research-tool-path/design.md`
- `openspec/changes/improve-web-research-tool-path/proposal.md`
- `openspec/changes/improve-web-research-tool-path/specs/web-research-tool-path/spec.md`
- `openspec/changes/improve-web-research-tool-path/tasks.md`
- `openspec/changes/improve-web-research-tool-path/acceptance.md`

## RED/GREEN evidence

- Alias recovery RED: `cargo test --lib tool_runtime::tests::process::run_process_alias_binds_canonical_runtime_project_id_in_recovery_context -- --exact` failed before the fix because recovery metadata contained `demo` instead of `agent:process-alias-recovery-context:demo`. The same test is GREEN after resolving the authorized alias before execution metadata construction: 1 passed.
- Oversized read RED: `cargo test --lib tool_runtime::read_files::tests::oversized_first_item_does_not_suppress_later_small_items_or_revision_guards -- --exact` returned no items instead of the later fitting item. The repaired packing and continuation guard are GREEN: 1 passed.
- Read regressions: `cargo test --lib tool_runtime::read_files::tests` — 15 passed.
- Search regressions: `cargo test --lib tool_runtime::search_project_texts::tests` — 12 passed, including non-prefix omitted-query continuation and zero-progress handling.

## Focused verification

- `cargo test --lib tool_runtime::tests::process::run_process_alias_binds_canonical_runtime_project_id_in_recovery_context -- --exact` — passed (1).
- `cargo test --lib tool_runtime::read_files::tests` — passed (15).
- `cargo test --lib tool_runtime::search_project_texts::tests` — passed (12).
- `cargo test -p webcodex-core project_instructions::tests -- --nocapture` — passed (9).
- `cargo test -p webcodex-workspace project_context::tests::project_instruction_sidecar_honors_claude_exclusion_without_reindexing -- --exact` — passed (1).
- `cargo test --lib tool_runtime::tests::sessions_instructions` — passed (7).
- The enabled-policy session case loads eligible `AGENTS.md` while omitting
  `CLAUDE.md`; the same focused module also covers unset defaults and distinct
  unavailable-source reporting.
- `cargo test --lib tool_runtime::tests::files::read_file_routes_safe_and_bulk_skipped_explicit_paths_to_agent -- --exact` — passed (1), including an explicit `CLAUDE.md` read while automatic exclusion was enabled.
- `node --check plugins/web-workflow/plugin.mjs && node --check plugins/web-workflow/plugin.test.mjs && npm test --prefix plugins/web-workflow` — passed (18 native-protocol tests).
- `cargo fmt --all -- --check` — passed.
- `git diff --check` — passed.
- `openspec validate improve-web-research-tool-path --strict` — passed: change is valid.

The native plugin fixture covers configured-only discovery, scoped CodeGraph
source256 retrieval with irrelevant worker exclusion and freshness metadata,
public user/assistant projection, dual-record duplicate suppression,
credential-like redaction, malformed and oversized records (including an
unterminated oversized line), bounded pagination, append continuation,
replacement/truncation rejection, root confinement and symlink rejection.

## Full-library result and limitation

`cargo test --lib` completed 2668 passing tests and exited 101 on the unrelated
existing `project_entry::cloudflared_service::tests::npm_config_query_uses_fixed_argv_and_normalizes_null`
test (`None` observed where its fake npm expected a proxy URL). An isolated
rerun with the exact test filter passed: 1 passed, 2670 filtered out. No
Cloudflared source was changed. This cross-test/environment-sensitive failure
remains a limitation of the full-run result, not evidence against the focused
change checks above.

## Boundary and remaining limitations

- The lead must independently inspect the exact diff and replay the key
  identity, oversized-batch and native-plugin checks; that is task 3.2 and is
  not claimed here.
- Public history is optional and remains unavailable unless an operator sets
  `WEBCODEX_WEB_WORKFLOW_HISTORY_ROOT`; no live history root or browser-host
  cache refresh was exercised. Tests use synthetic records only.
- CodeGraph reuses the existing configured runtime and reports its returned
  freshness/completeness envelope; this implementation does not reindex or
  claim coverage for an unindexed or stale project.
- CLAUDE exclusion defaults off, applies to automatic instruction candidates
  across startup, aggregate/first-match loading and sidecar snapshots, and
  does not alter explicit file reads. Host safety denials and browser-side
  schema/cache behavior remain outside this local candidate.

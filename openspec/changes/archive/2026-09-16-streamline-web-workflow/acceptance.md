# Fork acceptance

Status: lead-accepted and deployed. Published as [coordexp-2026.09.15.1](https://github.com/Pein2017/webcodex/releases/tag/coordexp-2026.09.15.1), with Linux bundle and SHA256SUMS assets confirmed uploaded. Exact build/tag source: `b0edfa887a836491c9150550e3c7da067db669b0`. This final receipt is a documentation-only follow-up; the immutable release tag is not moved.

## Accepted source checks

- Initialization instructions: 3 tests passed; digest/size startup contract and schema tests passed.
- Startup/tool contracts: 132 tool-contract and 86 runtime-contract tests passed.
- Git hygiene: 16 tests passed, including bounded-output identity and incomplete-diagnostic regressions. The old sentinel-before-tracked-list ordering lost repository identity when the retained output tail exceeded its bound.
- Optional plugin: lead replay passed 11 tests. Concrete repair checks cover non-JUnit roots, UTF-16 entity declarations, invalid UTF-8 continuation, and the admitted Native Plugin schema subset; no memory mutation API exists. Native admission rejects numeric bounds/pattern schema keywords, so descriptions state bounds while unchanged runtime validation enforces them.
- Final Server library: 2642 passed, 0 failed, 2 intentionally ignored real-process tests. The two timeout/process-group tests were also run explicitly and passed.
- Final Runner: 845 passed, 0 failed, 4 intentionally ignored tests using Rust 1.95. Formatting and strict OpenSpec validation passed.
- The upstream SSH reconnect fixture failed before repair because sending SIGTERM plus sleeping 50 ms did not establish that its temporary master had stopped. The test now observes bounded control-socket shutdown before asserting dead-master reconnection; its exact regression and full Runner suite passed. Production SSH behavior is unchanged.
- Synchronous command fixture: a 20,000-byte stdout plus stderr case timed out before repair and passed after anonymous file-backed capture. This is a test-only change; production command execution is unchanged.
- Markdown links: 84 files, 529 links, zero missing local targets.

## Real-entry evidence

- Lead replay of `scripts/e2e_web_workflow.py` passed 11 checks against clean release binaries: real WebSocket registration, configured MCP instructions, current schemas, guarded edit, actual structured validation, asynchronous Job terminal/log observation, and Git change readback. The script terminates only its own process groups and removes its disposable workspace.
- The old `e2e_zero_config_ws.sh` did not pass. Its consumer assertions assume obsolete Actions/adaptive schemas and `observe_jobs` nesting. Current model-facing Job items expose status/log fields directly, not under a nested `output`; the new live smoke proves that equivalent lifecycle rather than suppressing its check.
- Initial deployed checks passed: matching clean Server/Runner identity, three online Projects, unchanged authenticated exposure, and the same 3033-byte guidance hash through initialize and server/discover. All three real worktrees report `git_available=true`.
- Live Native Plugin admission passed for all three configured provider instances. A real pytest report preserved 1 pass, 1 failure, 1 skip and `testsExecuted=false`. Shared memory search/read returned bounded source-provenanced results. Scoped CodeGraph returned a symbol inside the requested research directory with freshness and truncation metadata.
- Main/infra currently have no CodeGraph CLI index; the scoped adapter reports `codegraph_uninitialized` rather than fabricating results. Research and the WebCodex source checkout have indexes; existing CodeGraph access remains installed.
- Final deployed acceptance confirmed 36 shared Runner Skills discoverable and readable from each of the three canonical Projects. Their common memory source was read through all three provider instances without copies. Main reports `project_skill_source_rejected` while retaining the valid configured Runner catalog; its existing symlink was preserved. Research dirty work was preserved.
- The main-project source-isolation repair passed caller-facing RED/GREEN, 14 Server Skill tests, 6 real Runner file-handler tests, and schema checks. It excludes only the fixed project-root `skill_path_escape` source with `project_skill_source_rejected`; the configured Runner source stays independently readable. Genuine transport/I/O/format failures remain unavailable, and the existing project symlink is never traversed or modified.
- Final binaries passed the 11-check isolated smoke again (2.01 seconds). External connector verification returned matching clean `b0edfa887a83` Server/Runner builds, three online Projects, and no active Jobs. The private `live-acceptance.json` receipt captures guidance hash, canonical HEADs, shared-Skill diagnostics, memory reads, Git recognition, actual pytest counts and scoped CodeGraph freshness.
- Original binaries and private configuration were retained, and the stopped Server database backup passed SQLite integrity checking. A second pre-final state copy is retained. Operator rollback instructions and exact deployment paths live alongside that private snapshot, not in public release assets. The existing tunnel window was not restarted or replaced.

## User-accepted Web-host check (2026-09-15)

The user supplied a ChatGPT Web read-only acceptance report and explicitly accepted this phase ("OK,完美"). This is user-reported host evidence, separate from the lead's earlier local and connector receipts; it was not regenerated during the documentation closeout.

- All six requested feature groups passed on `b0edfa887a83`: aligned clean Server/Runner builds and three online canonical Projects; common 3033-byte startup guidance without repository/workflow bodies; shared Runner Skill discovery/read; native-plugin discovery and read-only shared-memory search/read; directory-scoped research CodeGraph with reported fresh index metadata; and the expected pytest report counts plus correct Git recognition.
- The intentional pytest failure remained untouched. Existing research dirty work was preserved. The report states that no files were changed, no experiments were started, and all three acceptance Workflow Sessions were closed.
- LSP capability probes returned `probe_timeout`: semantic-navigation readiness remains unverified and was outside this acceptance scope. `missing_skill_definition` and `project_skill_source_rejected` diagnostics did not prevent the authorized shared Runner Skills from being discovered or read.
- Phase disposition: user-accepted and closed at the agreed feature boundary. This closeout adds evidence only; it does not authorize LSP repairs, additional indexing, another release, service restarts, or changes to the immutable published tag.

## OpenSpec verification

| Dimension | Result |
| --- | --- |
| Completeness | 11/11 tasks and 6/6 requirements covered |
| Correctness | Changed trust/bounds paths have discriminating regressions; final real MCP/Runner and deployed checks passed |
| Coherence | Existing project entry, canonical checkout authority, guarded edits, optional native plugins and manual Job retrieval retained |

Requirement evidence maps to `src/config.rs`, `src/tool_runtime/coding_task.rs`, `src/tool_runtime/startup_brief.rs` (guidance/startup); `src/tool_runtime/skills.rs` and `plugins/web-workflow/` (shared context); `src/tool_runtime/hygiene.rs` (Git diagnostics); and `scripts/e2e_web_workflow.py` plus the published build receipt (real delivery). No critical verification issue remains. Portability, unindexed-project and browser-host limitations above are explicit scope boundaries, not claims of tested behavior.

## Build prerequisites discovered

- Git 2.34 lacks the reviewed-commit attribute operation needed by current upstream. Git 2.55.0 passes the attribute probe; after upgrading, both reviewed-commit attribute regressions passed in the full Server suite.
- The host's default Rust 1.93 rejects an upstream test-only AtomicUsize::try_update call. Use the already-installed Rust 1.95 toolchain for Runner verification and release builds; do not patch production behavior to work around the compiler.

## Delivery boundary

This is a self-hosted Linux fork prerelease, not an upstream npm, Desktop, or six-platform release. The release must identify exact clean source, matching Server/Runner build identity, artifact checksums, isolated real-process smoke, and post-deploy MCP checks. Private operational logs/configuration and shared memory content are not release assets.

# Source acceptance

Lead-accepted integration of upstream stable `v0.4.3`, pinned at
`b96a59a712ca5355ad8609cd861cd0c7acb9f99e`, into original fork
`50281db4c3e04fa474d9724b03dd3dc5b91d7ea8`. Latest stable was rechecked;
post-release upstream main is not part of this candidate.

The normal merge retains history and native ownership. Retained corrections are
the bounded requirement gaps listed in `design.md`; no CLAUDE exclusion,
context-revision ACK handshake, automatic recovery overlay or alternative
assertion matcher remains. Existing optional plugin/deployment assets and
historical OpenSpec records remain. All three added/changed lines in the original
dirty external-source guidance are carried verbatim into the candidate.

## Fresh checks

Private raw logs and receipts are under
`/data/CoordExp/.local/scratch/webcodex-sync-v043/`.

| Check | Result | Receipt |
| --- | --- | --- |
| Server library | 2,965 passed, 3 upstream ignored | `sensitivity-restored-green.log` |
| Workflow Session | 199 passed | `sensitivity-restored-green.log` |
| Core / Runner registry / CLI libraries | 294 / 298 / 404 passed | `native-contracts-green-2.log` |
| Runner binary unit tests | 926 passed, 4 upstream ignored | `runner-bin-green-195.log` |
| Optional plugin | 29 passed | `plugin-final.log` |
| Native disposable Server–Runner | 31 passed | `source-accepted-e2e/receipt.json` |
| Syntax, shell syntax, formatting, whitespace | Passed | Python compilation, `bash -n`, `cargo fmt --all -- --check`, `git diff --check` |
| Strict change and canonical specs | Passed | `openspec-native-evidence.log`, `openspec-native-specs.log` |

Rust 1.95 is selected explicitly for the matching build and Runner tests. The
installed default Rust 1.93 rejected upstream's test-only `AtomicUsize::try_update`;
no compatibility patch or global toolchain change was introduced. Local builds
use `dogfood`, locked dependencies, four build jobs and `workspace-checkpoints`.
Experimental V8 remains off. These are relevant local-runtime checks, not the
formal cross-platform desktop/package release matrix.

Six temporary fault mutations each made the intended consumer test fail:
baseline field identity, oversized-first read, oversized-first search,
configured Skill rejection, revision-fenced Skill continuation and tracked-file
hygiene. The source was restored byte-for-byte to the pre-mutation diff before
the green regression. Recorder-bound Plugin ACK and premature execution-output
sparsification also have captured behavioral RED/GREEN evidence. No valid code
was deleted to manufacture a negative check.

The real-entry smoke covers configured initialization guidance, native adaptive
discovery, real Skill/plugin reads, JavaScript execution, guarded edits and stale
rejection, actual stable-assertion pytest fail then pass, typed async pytest
counts, Job terminal logs, dirty-path baseline comparison and finish. Matching
historical failure resolves; current-source evidence deliberately stays
`unproven` with `validation_source_unproven`, as required by native v1. Execution
success is not a mutable-workspace currentness certificate.

## Existing state and delivery boundary

The preliminary isolated ledger check restored 62/62 existing Sessions, 3,470
events, three messages and 32 baselines. Strict raw comparison exposed ten title
normalizations and four changed-path-list normalizations (96 path references
outside the retained prefix). The deployed source `98d9301cba22`, original fork
HEAD and pinned upstream all have identical 240-character title / eight-path
restore bounds. The explicit existing-native-normalization check admits only
those exact old rules and rejects other differences; identity, message and
baseline comparisons remain strict. It reported no additional divergence and
removed 695 retired context fields. Raw, expected-native and rewritten payload
digests are separately recorded in
`session-restore-native-preliminary/receipt.json`; this is not bitwise preservation
of the original raw event payloads. Complete raw records must remain in the
stopped-state backup before production activation.

Original source/dirty guidance and the production release were unchanged through
source acceptance. Clean-build, canonical-checkout consolidation, non-force fork
push, stopped-state backup, local activation and connected-client verification
are subsequent authorized delivery tasks; no public Release/tag is authorized.
Operational evidence belongs at the existing deployment root's `verification/`
and `rollback/`, not in public source assets. No manual ChatGPT browser result is
inferred from a host smoke.

## Delivered local runtime

The accepted implementation is merge commit
`004e108a7e5e9bd72e023b13939f763383cb6ad6`. Both fork branches `main` and
`coordexp/web-workflow` were updated atomically without force. The canonical
checkout is `/data/CoordExp/.local/src/webcodex-mcp-instructions`; the clean,
absorbed integration worktree was removed. The original dirty guidance remains
recoverable in scoped stash `162188d610af33547077c4bd41c97764103d4f8e`
and the private task patch. This delivery-record follow-up changes only these
OpenSpec records; it does not change the deployed implementation or require
rebuilding the immutable bundle.

Clean matching Server, Runner and CLI binaries are deployed as private local
bundle `coordexp-2026.09.30.1`, built from the implementation commit above.
The final clean-binary real-entry smoke passed all 31 checks. The actual private
ledger shadow check restored all 62 old Sessions and found zero divergence
beyond the explicitly documented existing-native normalization. Live Runtime
status reports 62 restored Sessions with no persistence error. All five original
canonical Project id/path pairs are unchanged. Configured shared Skill discovery
and a revision-guarded read match the shared file; existing project-source and
missing-definition diagnostics are retained rather than concealed. Native
CodeGraph and all three Web workflow providers report ready. Runner `python3`
still resolves to the `ms` environment; initialize/discover guidance matches the
bundle's 10,992-byte file exactly. All eight bundle checksums pass.

Only the dedicated local tmux service was stopped/restarted, after zero live Jobs
and pending requests were verified. A consistent stopped-state state/config
backup is retained under
`/data/CoordExp/.local/webcodex-custom/rollback/pre-v043-2026.09.30.1/`;
archive SHA-256 is
`c0c2d1f1001434d11b62c802d2ea69db6be1fd70cd5382f2388eb2c8c99a8452`.
The old release remains intact. Server, Runner and Tunnel configuration files
are byte-identical to the backup. No public Release/tag, package publication,
credential change or research launch occurred.

The existing connected ChatGPT tool actually returned Server/Runner version
`0.4.3`, clean build `004e108a7e5e`, exact build alignment and online Runner
`coordexp-web-runner`. This is remote connected-client evidence, not merely a
localhost smoke. The current conversation still caches old direct tools:
native discovery correctly routes `list_projects` through `call_runtime_tool`,
which is absent from that cached catalog. Refresh the existing connection at
ChatGPT Plugins and start a new conversation; reinstalling is unnecessary.
Manual browser Refresh/new-conversation verification remains a user-side step,
not a claimed completed check. See the
[official refresh flow](https://developers.openai.com/plugins/deploy/connect-chatgpt#refresh-metadata).

The private operational receipt is
`/data/CoordExp/.local/webcodex-custom/verification/coordexp-2026.09.30.1/live-receipt.json`;
matching-binary smoke and state-shadow receipts are beside it. The read-only
`verify_live.py` reproduces the live host checks without exposing credentials.

Use this read-only prompt after Refresh in a new ChatGPT conversation:

```text
Verify the existing WebCodex connection with actual tool calls only. Call
runtime_status with client_id=coordexp-web-runner and compact=true. Require
Server and Runner 0.4.3, git_commit=004e108a7e5e, git_dirty=false, online Runner
and aligned sources. Follow tool_manifest's advertised route for list_projects
and confirm five Projects; list the Runner's plugins and report ready/error.
Do not guess a direct route or reuse retired tools. If call_runtime_tool is
absent, report stale connection metadata and request Refresh/new conversation.
Report each actual result as PASS/FAIL. Do not create Sessions, launch Jobs,
edit files, train, index, reinstall, or change credentials.
```

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

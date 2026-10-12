# Lead audit — initial HOLD, subsequently corrected

The findings below preserve the pre-repair audit. The lead's direct correction
and final acceptance are recorded in acceptance.md; do not interpret the
historical disposition below as the status of the corrected candidate.

Scope: the Luna/max candidate against baseline `ab089186`, reviewed on
2026-09-16. No implementation edits, commits, push, deployment, restart or
research changes were performed by this audit. Only acceptance records changed;
diagnostic fixtures were created under `/tmp/webcodex-lead-audit.RDIHFZ`.

## Fresh checks

- Alias regression: 1 passed.
- Read projection regressions: 15 passed.
- Search projection regressions: 12 passed.
- Native plugin protocol suite: 18 passed.
- Session instruction suite: 7 passed.
- Workspace instruction-policy case: 1 passed.
- `git diff --check`: passed.
- Real history root, exact known thread: `history_not_found`; only path metadata
  and the error code were returned, no private message body was inspected.
- Independent synthetic diagnostic command:
  `node /tmp/webcodex-lead-audit.RDIHFZ/history-audit.mjs`.

The worker's full-library attempt and isolated Cloudflared retry are recorded
separately in acceptance.md. No full-library rerun was necessary for this audit.

## Blocking findings

### P1 — New reader does not address the actual Codex history

`plugins/web-workflow/plugin.mjs:810` constructs `<threadId>.jsonl` with no
native rollout-path resolution. The actual selected thread exists under
`2026/09/16/rollout-2026-09-16T02-45-11-01a0a81a-9e32-7db1-bd07-86fa601f4276.jsonl`.
Setting the history root to the existing `.codex/sessions` and calling the
explicit thread returns `history_not_found`. Additionally, its actual file size
is 121,478,056 bytes, above the new 64 MiB whole-source rejection at line 862.
Fixing just the filename would not make the motivating workflow work.

The tests manufacture short `<threadId>.jsonl` files; they verify an invented
layout, not the requested existing-history integration. Repair should address
native path selection without copying/renaming real records, and bound per-call
work rather than reject an otherwise readable large rollout solely for size.

### P1 — Public-only admission is permissive

At lines 1059–1071 an unclassified assistant message defaults to `final` and
`nestedText` recursively extracts arbitrary content fields without admitting
known public content types. A synthetic assistant `response_item/message` with
no public channel and a `reasoning_text` content block returned the nonpublic
marker as a public message. This violates the explicit fail-closed public-only
contract; it is not evidence that real private content was exposed in service.

Use the supported native record shapes and public content types as an
allowlist; unknown shapes must not inherit public status.

### P1 — Text equality is not message identity

At lines 1083–1084 the fallback identity hashes role, phase and redacted text.
Two independent, ID-less user messages with identical text are reduced to one,
with the second labeled `duplicate_representation`. The synthetic protocol test
expected two and observed one. Distinct long messages with the same bounded
prefix or messages differing only in a redacted token can also collapse.

Suppress only proven alternate representations of one occurrence, not repeated
instructions or statements at different source positions.

### P2 — EOF and partial last records break append continuation

At lines 1159–1194 an incomplete last JSON record is consumed as malformed when
it reaches the current EOF. At lines 1223–1226 EOF removes the cursor entirely.
Synthetic cases confirm `nextCursor=null` both after a complete initial file and
after a partially written trailing record. Appending afterward cannot resume
from that returned cursor; restarting rereads old messages and cannot preserve
the intended append continuation. Retain a safe resume position and do not
discard an unfinished active record as final malformed input.

## Entropy audit candidates (read-only)

### High confidence / medium risk — Speculative multi-format history parser

Production consumers of `nestedText`, `recordThreadId`, and the broad message-ID
fallbacks are confined to this new reader. It tries many conversation/session
aliases and arbitrary nested keys, yet misses the actual on-disk rollout layout.
Replace this speculative compatibility surface with the two observed Codex
record representations and exact occurrence provenance. Remove corresponding
synthetic compatibility-only branches/tests, preserving public-boundary and
native-layout tests. No currently demonstrated external consumer requires the
invented formats. This narrows supported formats, so document the choice before
implementation. Net reduction is expected in parser branches and identity
states; an exact line estimate requires the replacement diff.

### High confidence / medium risk — Rehashing all consumed history each page

`hashFilePrefix` is used both to validate the incoming cursor (line 954) and
create the next cursor (line 1109). Each later page rereads the entire consumed
prefix, even though reported scan limits describe only the new 512 KiB window.
Across a long sequential read, hashing work grows quadratically in page count
for roughly fixed-sized pages. This is source-derived cost analysis, not a
timing benchmark. The 64 MiB cap does not cure the workflow mismatch.

Do not simply delete identity checks. Choose a bounded native-source identity
contract that still detects the required replacement/truncation cases; if
arbitrary in-place rewrites must also be detected exactly, that cost/semantics
decision needs to be explicit. Reuse existing file identity where sufficient.

### Kept — Authorization, revision guards and instruction source policy

The execution identity changes use the existing authorized resolver and retain
Runner fences. Batch follow-ups preserve original request identities and read
revisions in fresh tests. CLAUDE exclusion removes a duplicated candidate list
from the workspace path and leaves explicit file access intact. These protect
real contracts; no deletion is justified merely because several entrypoints
need the same policy. No generic batch framework is recommended.

## Disposition

Do not deploy or mark the whole change accepted. Retain useful routing,
batch and instruction-policy fixes; the history reader needs one focused
correction with native-shaped fixtures and the four counterexamples above.
No second implementation or delegated review was started during this audit.

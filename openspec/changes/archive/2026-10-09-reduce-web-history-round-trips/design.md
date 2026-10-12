## Context

See proposal.md. Work starts from the existing dirty but lead-accepted local candidate at ab089186, not from clean HEAD. The public parser, root checks, fingerprint checks and tests already exist. No indexed-history service is needed.

## Goals / Non-Goals

Goals: bounded recent-feedback retrieval and lower unnecessary I/O with a small additive API.
Non-goals: full-history indexing, arbitrary file reads, raw/private messages, new dependencies, Rust protocol changes, CodeGraph parallelism, caching, automatic polling or release operations.

## Decisions

1. Keep one tool. Add optional mode=forward|latest. Omitted mode defaults forward on fresh calls, otherwise inherits the cursor. Reject explicit conflicts. Preserve existing v2 forward cursors; a distinct tagged latest cursor binds direction, source identity, snapshot upper bound and next older byte boundary. Never accept caller offsets as authority.
2. Latest uses bounded backward chunks from a fixed initial EOF. Select newest eligible messages first, then reverse each returned page to chronological order. nextCursor means older history in latest mode, not future appends. Return mode, snapshotBytes, hasMore and complete (= snapshot exhausted); nextCursor is null at BOF. Do not reuse forward eof/waitingForAppend fields with contradictory meaning. A fresh latest call observes newer appends.
3. Provenance uses exact byte offset and source.path; latest source.line=null. Do not scan the prefix to manufacture a line number. Preserve the existing explicit-ID deduplication policy within its bounded window, and preserve ID-less distinct occurrences.
4. Use small incremental reads (32 KiB starting target) for both directions, cap total scan at 512 KiB and fingerprints at 16 KiB. Carry bounded partial/discard boundary state where necessary; reuse common source validation, redaction and message admission, not a generic parser framework. Oversized records can produce empty but advancing pages. Incomplete trailing records are excluded; fresh latest reads can see them after completion. Do not buffer an unbounded line or silently lose a valid <=128 KiB line at chunk/page boundaries.
5. Alternative rejected: shell tail without stable cursors loses source/boundary semantics; a persistent index/cache adds lifecycle and invalidation beyond this task. Full prefix hashing remains excluded under the established append-only-source contract.
6. Documentation owner follows this frozen API while implementation owner handles both reader code and its tests. The lead owns all OpenSpec files, checkpoints and acceptance. No concurrent writers on parser/test surfaces.

## Risks / Trade-offs

- Backward boundaries and append races -> synthetic byte/UTF-8/partial/oversized tests, snapshot-bound cursors and source replacement tests.
- Sparse public history -> pages may be empty but must advance; no promise that every request returns N messages.
- Absolute line number unavailable -> null plus exact byte offset, explicitly documented.
- Performance assertions -> deterministic bytes and request counts, not flaky wall-time gates.
- Existing uncommitted baseline -> preserve all prior changes; no reset, broad staging or release.

## Migration Plan

Local additive implementation and targeted tests only. Forward clients retain defaults and cursor support. Deployment/browser declaration refresh remains a separate authorized operation; no live service changes in this change.

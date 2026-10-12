## Context

See proposal.md for motivation. Planning baseline: clean `coordexp/web-workflow` at `ab089186`. Evidence is local attachment directory `/data/CoordExp/.codex/attachments/15431d76-744f-4a3b-8932-65f56aaeb713`, containing `tool-call-data.json` (24 calls) and `tool-call-data (1).json` (19 calls). Do not commit full traces or private history.

Observed source anchors:
- Runner `main.rs` checks a recovery context's `runtime_project_id` for its `agent:<client>:` prefix. The second trace succeeds immediately after changing only the project spelling. Fix the producer/resolution path, not this fence.
- `src/tool_runtime/read_files.rs::apply_output_budget` stops on the first non-fitting item; `truncate_read_item` cannot split an indivisible line. Existing follow-ups use a suffix index, which must be updated coherently if items can be skipped.
- `plugins/web-workflow/plugin.mjs` already implements directory-scoped CodeGraph queries, freshness metadata, safe root handling and bounded memory reads. Its tests exercise the native plugin protocol.
- `crates/webcodex-core/src/project_instructions.rs::INSTRUCTION_CANDIDATE_PATHS` includes `CLAUDE.md`; startup and inspection loaders share this list. No exclusion control was found in those paths.
- `deploy/web-workflow/AGENTS.md` already documents Session and CodeGraph boundaries; refine it instead of adding another mandatory bootstrap.

## Goals / Non-Goals

**Goals:** Fix the observed identity and batch failures, prefer existing scoped evidence, provide bounded public-history reads, and consistently exclude CLAUDE guidance when configured.

**Non-Goals:** Replace CodeGraph retrieval, normalize arbitrary authority identifiers, merge Session fields, remove ACK checks, suppress safety denials, automatically reindex research projects, revive checkpoints, create services/databases, or alter active research/GPU tasks. No push/release/restart authorization is implied.

## Decisions

1. Normalize once using the existing authorized project resolver before building execution/recovery metadata. Preserve the Runner fence and cross-project negative tests. Do not add spelling-based fallback on the Runner.
2. Preserve each batch item's original identity while packing small independent results around oversized ones. Build follow-ups from all omitted/partial original requests, preserving revision guards; do not use a suffix cursor if it loses or duplicates completed items. Retain bounded output including continuation overhead. Prefer existing incompleteness fields and an explicit zero-progress diagnostic over a new generic status framework.
3. Reuse `codegraph_scoped_query(project, search, pathPrefix)` and its existing freshness envelope. Guidance recommends exact symbol/path search before relationship expansion for named modules. Test with fixtures containing source256 and irrelevant worker symbols. Real source256 reads can establish current coverage without reindexing; lack of coverage remains an explicit limitation, not an excuse to widen scope. No invented native CLI flags or new ranking engine.
4. Extend the existing optional plugin with a single typed public-history reader and a separate operator-configured root. Reuse root confinement and output limits. Inputs select thread identity and bounded pagination, never arbitrary shell/expressions. Project only public user text and assistant final/commentary (or explicitly public event-message equivalents); exclude analysis, system/developer/config and tool payloads. Support the two observed rollout representations and avoid double-emitting the same message when both exist. Use source identity plus bounded scan progress; oversized/malformed records must advance with an omission receipt, never allocate an unbounded line. Do not scan all historical payloads merely to locate a known thread. Missing configuration hides this tool. This is access to authorized local records, not a safety-filter workaround.
5. Add one narrow operator exclusion switch for CLAUDE to the existing server configuration pattern (final name selected consistently during implementation), default off. Filter fixed candidates before reads, not merely after rendering. Apply to first-match and aggregate loaders and sidecar consumers; snapshots report only effective candidates. Avoid caller-supplied file lists and a general policy language. Keep explicit file tools unchanged. Document how a later authorized deployment enables the switch.
6. Guidance uses `resolved_project` after startup, the existing separate Session fields, truthful omitted-item continuation and explicit ACK recovery. Do not auto-select a recorder, auto-ACK, or add `client_id` as a speculative plugin alias. Defer optional LSP probes and skill-catalog ranking.

## Risks / Trade-offs

- Out-of-order batch packing can break continuation → preserve original indices, revision guards and test exact omitted coverage.
- History files are sensitive and mutable → explicit operator root, fail-closed paths, public-only projection, bounded parsing, credential redaction using established patterns, and cursor identity tests. No real private message bodies in fixtures.
- CodeGraph may not index new dirty files → disclose coverage; verify through literal source search without automatic index mutation.
- Instruction exclusion can diverge across entrypoints → one effective-candidate policy with startup and sidecar integration tests.
- Host safety denials are outside local runtime control → do not claim elimination or implement evasion retries.

## Migration Plan

Implement and validate locally after proposal approval, with one Luna/max owner and lead replay of decision-bearing tests. Keep existing configuration defaults. Later deployment requires explicit authority: build reviewed Server/Runner, preserve rollback, enable CLAUDE exclusion and optional history root, retain compact schemas, measure discovery size, then verify through real MCP and request browser metadata refresh if tool declarations changed. Rollback is previous binaries/configuration; no data migration is intended.

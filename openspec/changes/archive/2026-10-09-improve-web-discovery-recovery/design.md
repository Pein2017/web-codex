## Context

See proposal.md. The uploaded 46-call trace has one successful Skill read, three successful scoped CodeGraph calls, one corrected `client_id`/`runner` schema rejection and one recorded shell timeout. Partial handoff leaves ACK recovery unresolved; later process output and reads also truncate. No final task report or closeout is included, so this trace cannot establish final review quality or a Session leak.

## Goals / Non-Goals

Goals: reduce call construction and recovery mistakes at existing producer seams; keep bounded Web guidance useful.
Non-goals: new gateway/tools, skill relevance engine, automatic ACK/session selection, timeout suppression, index rebuild, research mutations, broader LSP repairs or shell lifecycle rewrite. A malformed `session_id=wc_s` on one read is noted, not promoted into an authority bug without tracing that tool's actual Session semantics.

## Decisions

- Reuse the existing `{tool, arguments}` follow-up shape in startup entries. Derive identity from authorized domain data; do not add `client_id` alias or serialize full schemas/bindings. Skill revision belongs in the guarded follow-up rather than duplicate fields.
- Keep hard byte/entry ceilings and current provider scope. Do not inject a global CodeGraph provider into a project-scoped catalog. Explain relationship-tool discovery in deployment guidance instead.
- Make recovery defaults explicit (`summary_only=false`, required include flags true, normal limit). No recorder inference and no automatic ACK. Existing stable-snapshot/concurrency guards remain authoritative.
- Update concise Web guidance: early bounded relation queries; changed-file-first analysis; avoid all-pairs long AST similarity; follow continuation only for needed evidence; preserve timeout/unknown outcomes and no blind retry. Existing Session rules prohibit silently demoting a timeout based on diagnostic purpose, superseding the earlier tentative recommendation.

## Risks / Trade-offs

- Larger entries reduce catalog capacity -> preserve explicit truncation and test hard ceilings.
- Correct suggestions do not compel model compliance -> validate emitted calls, retain browser acceptance as a distinct boundary.
- Model-host null timeout response cannot be diagnosed from an export alone -> preserve runtime failure evidence; do not claim a transport fix.

## Migration Plan

Two disjoint workers own startup discovery and recovery; root owns guidance, tests orchestration, review and release. Build committed source with dogfood and validate disposable real MCP. Push only scoped commits to the user's fork; no force push, tag or npm publication.

The user's later container persistence requirement extends deployment only: `/data` is persistent, `/root` and `/var` are not. Audit actual process/dependency paths before selecting necessary copies. Under `/data/CoordExp/.local/webcodex-custom/`, keep immutable `releases/<version>` and `current`, version-independent private state/config, service launchers, logs, verification and rollback records. Source stays at its current location. Copy before switching, preserve modes and secret values without printing them, and take a consistent stopped-state backup before state relocation. Keep prior data and releases intact.

Restart only confirmed WebCodex Server/Runner/Tunnel owners after checking active Jobs and persistence. Do not interrupt research, Codex, SSH/proxy services or claim that an SSH-forwarded proxy survives container recreation. Provide a persistent recovery entrypoint that checks external prerequisites. Verify local startup, Runner/plugin connectivity, preserved Session evidence, Tunnel connectivity, and a reversible release switch. Record system-image prerequisites that cannot be vendored rather than claiming arbitrary-container portability.

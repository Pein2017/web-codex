## Context

See `proposal.md` for motivation. Assessment source is the last assistant message (`561dd04e-bcbb-4040-91fe-9ebe1a50648d`) in the user-supplied shared conversation, reviewed on 2026-09-15. Treat its reported symptoms as hypotheses, not authoritative API descriptions.

At `47bf37df`, `run_script` already declares JavaScript/TypeScript, Runner admission checks language capabilities, and execution resolves Node. Live `b0edfa887a83` stateless MCP discovery also advertises all five languages. `read_files` already issues `read_revision`; `apply_text_edits` resolves `expected_read_revision` to the exact Runner SHA. Session logical-invocation correlation and declared result expectations also already exist.

`SessionCounts.write_like` classifies tools; it is not a filesystem detector. Startup and finish independently inspect current `show_changes`, with no durable startup Git baseline. The existing shared JUnit plugin reads reports but does not itself execute or attest tests.

## Goals / Non-Goals

**Goals:** reduce repeated model bookkeeping using existing primitives; provide truthful startup/finish observations; make actual validation evidence and failure recovery testable; deliver an integrated local build.

**Non-Goals:** automatic browser wake, GPU leases, agent orchestration, new universal context/content tokens, hidden recorder selection or ACK, arbitrary JSON-query tools, new named-worktree API, additional Skill libraries, or proving exclusive filesystem authorship. The shared filesystem has independent writers; a shell can write outside Git or restore its own changes before observation.

## Decisions

### Reuse canonical schemas and guards

Keep current tools and `read_revision`; document copying it into `expected_read_revision`. Check current Runner capabilities and execution prerequisites rather than creating a second schema/version registry. Add real MCP smoke for these paths and compact invalid-argument failures. Host-side schema expansion remains unlocalized; do not claim a server error-formatting fix without a captured server failure.

### Observe a bounded workspace baseline, not inferred ownership

Capture one optional immutable baseline on fresh project Session startup using the existing bounded Git change observation. Preserve it across resume/restart. Store only HEAD, bounded path/status metadata and completeness, without source contents, commands or secrets. Compare against finish's fresh observation and expose pre-existing dirty paths, newly dirty paths, cleared paths and overlapping dirty paths. Overlap does not prove unchanged bytes; absence from a truncated snapshot proves nothing. Missing/legacy baseline, failed inspection, changed project/HEAD or incomplete observations must remain explicit.

This deliberately starts with path/status evidence instead of hashing every file or wrapping every command in two expensive Git scans. It detects a clean file becoming dirty after a shell operation, while explicitly leaving within-already-dirty content changes and actor attribution unknown. Existing `write_like` semantics remain unchanged and guidance explains the distinction. Baseline information supplements closeout; it must not suppress conflicts, secret warnings or unresolved validation failures.

### Validation remains evidence-based

Use existing explicit assertion identity and result expectations. Verify that later successful matching assertions resolve earlier failures without making unrelated successes or expected negatives count as validation. Add pytest summary support through the existing validation evidence path where the captured execution metadata permits it; missing/truncated evidence must remain unknown. Do not add another test executor or rely on a model-authored report as proof of execution.

Inspection confirmed that matching-assertion failure recovery already exists.
Started generic execution failures intentionally remain actionable unless their
declared validation/expectation evidence resolves them. The verified parser gap is
that generic test summaries currently use Cargo grammar, so pytest counts are not
recognized. Repair that gap without redefining all previous shell failures as
protocol misuse.

New pytest counts require a direct typed `run_process` execution identity (or its
Job with equivalent retained source identity), not a raw shell/script preview.
Extract supported counts before the Session shortens its output excerpt; preserve
original Runner truncation/unknown status. Raw shell assertion recovery still uses
its existing explicit identity and actual outcome.

Real MCP validation additionally requires the outer Session recorder to consume
canonical process/script results before the public success projection removes
redundant execution fields. Defer that existing projection to the kernel's final
model-facing boundary when the outer recorder owns the call; do not reconstruct
missing exit/completeness facts from a compact success response.

### Feedback disposition

| Feedback | Decision |
| --- | --- |
| Schema revision / content fence | Existing canonical schema and read revision; guide and verify them |
| Large errors | Verify server errors are bounded; host rejection requires its own evidence |
| Shell mutation / dirty baseline | Add honest path/status baseline comparison; no exclusive attribution |
| Finish failure lifecycle / pytest | Verify existing lifecycle, repair demonstrable parser/projection gaps |
| Session auto-binding / ACK | Keep explicit identities and actual model acknowledgment |
| Ledger event collapse | Existing logical-invocation projection; exercise rather than duplicate |
| CodeGraph session-native | Keep current optional plugin binding; reuse while valid |
| Named worktree | Existing Git + register path works; no new lifecycle abstraction |
| GPU/process lease, wake, JSON tool, new Skills | Outside this bounded reliability pass |

## Risks / Trade-offs

- Concurrent writers or reverted transient edits → call results observations, not ownership proofs.
- Truncated/failed Git capture or legacy Sessions → unavailable/partial baseline; never fabricate a clean baseline.
- False pytest positives → parse supported terminal summary grammar conservatively, keep execution identity/outcome authoritative and use negative fixtures.
- Upstream identifier changes → validate schema bindings and read fences after merge; refresh host discovery when required.
- Restart with active work → recheck live Jobs, preserve prior binaries/config and consistent state backup before switching.

## Migration Plan

Implement and commit scoped packages, then merge fetched upstream `main` (`6a1433d0` at assessment). Recheck current upstream before publication. Preserve fork history and resolve routine conflicts; escalate a conflict in meaning or authority. Build matching Server/Runner from a clean commit, run focused tests plus disposable real MCP smoke, push to the existing fork and deploy locally with rollback. Retain the existing tunnel. This is a self-hosted Linux fork delivery, not upstream npm/Desktop/container publication.

## Context

See proposal.md. Baseline is clean commit 719d7ee3. The specialized gateway parses concrete arguments after stripping only expectation metadata, unlike generic invocation handling. Both Runner and project Skill reads expose next_start_line but no continuation call. Session store classification has a runtime_error fallback. Existing SuggestedToolCall, Session ACK implementation, narrow CodeGraph tools, and exploration manifest are the intended reuse points.

## Goals / Non-Goals

Goals: complete existing caller-facing contracts with bounded changes and falsifying tests.
Non-goals: new recovery envelope, router, evidence graph, automatic ACK, inferred recorder, retry engine, production reindexing, deployment or publication.

## Decisions

1. Preserve gateway authorization and effect policy. Reuse existing validated message-ACK extraction/observation rather than accepting and dropping fields. Context ACK applicability remains governed by ToolDefinition; an inapplicable known field is explicitly ignored, never converted into retained context. No broad gateway/kernel rewrite.
2. Skill continuation uses existing SuggestedToolCall and observed definition_revision. Both source scopes and EOF must agree. Diagnostics identify candidates by safe logical name and scope, not absolute path or fabricated callable ID.
3. Keep failure identity in existing error_kind where possible. Trace the consumer-facing LSP failure through record and handoff before choosing the smallest producer/classification correction. No error-prose parsing or new ledger subsystem.
4. Improve the existing exploration projection and Web guide. Narrow callers/callees tools precede explore for narrow questions. LSP is not universally preferred; freshness and language/task determine selection. Graph failure is not evidence of absent code.
5. Ownership: gateway/ACK package; Skill/discovery package; lead-owned ledger fidelity and integration. Workers share no production source write surface. Any needed overlap is resolved with the lead before editing.

## Risks / Trade-offs

- ACK accepted but not applied -> test first_ack_observed_at, request-scoped suppression, and wrong-Session isolation through the caller path.
- Pagination drift or EOF loop -> replay generated calls; mutate definition between pages; verify final page has no follow-up.
- Diagnostic path leakage -> adversarial candidate names and bounded results.
- Error taxonomy inflation -> preserve existing structured codes, fallback only where no trusted structured classification exists.
- Discovery payload growth -> reuse existing descriptions/projections; no new top-level tools or full source payloads.

## Migration Plan

No production state changes in this task. Focused RED/GREEN tests, exact diff review, formatting and strict OpenSpec validation establish a local candidate. A later explicit publication request may build immutable releases and use the existing /data lifecycle/rollback entrypoint. Local acceptance does not claim browser deployment acceptance.

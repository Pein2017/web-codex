## Context

See proposal.md. The source fork preserves the initialization patch and merges upstream 3d27c9bf. Upstream already has bounded startup skill/plugin catalogs and configured Runner skills. Actual local installation paths and credentials remain outside this public repository.

## Goals / Non-Goals

Goals: reuse upstream entrypoints, add only missing behavior, publish reviewable source and deploy to the existing self-hosted Linux MCP service.
Non-goals: a project-family framework, automatic browser wake, another scheduler, memory migration, or upstream npm/desktop/container distribution.

## Decisions

- Keep Web-specific operator instructions in a versioned example loaded via WEBCODEX_MCP_INSTRUCTIONS_FILE. Startup exposes digest/size, not another full instruction copy. Repository AGENTS remains developer guidance, separate from runtime guidance.
- In this fork, omitted include_project_instructions/include_workflow_guidance flags default to false in both parsing and schema metadata; explicit true retains upstream opt-in behavior. This avoids contradictory fresh-context tool guidance reintroducing Codex harness instructions. Internal identity/fingerprint observations retain their existing contract.
- Reuse upstream startup catalogs and Runner-configured skill directories. Treat Codex-specific skills as guidance whose tool requirements must be checked, not automatically executable commands.
- Shared memories are read-only source files through a bounded native plugin where existing exact-project memory tools cannot reach them; no write or synchronization API is added.
- Use an optional native plugin for scoped CodeGraph and pytest report projection when no equivalent upstream mechanism exists. Preserve existing guarded file edits and process/job execution.
- Git hygiene fixes retain the existing semantic contract and require a discriminating regression test.
- Keep server and runner on the same committed source. Use a local optimized build and a fork-specific source version/deployment receipt. Do not invoke upstream npm or six-platform release machinery for this single Linux deployment.

## Risks / Trade-offs

- Upstream integration changes runtime schemas → validate current manifest and tell the user to refresh the connection if the host caches the old schema.
- Shared context may contain stale or harness-specific instructions → preserve provenance, use it on demand and revalidate live facts.
- Runner restart can interrupt ordinary jobs → check active jobs before replacement; preserve existing detached jobs and rollback binaries/state.
- CodeGraph can be stale → report index freshness and use native file/search tools for unindexed evidence.

## Migration Plan

Commit and push scoped changes to the fork; run focused tests and a real isolated Server/Runner smoke, retain current binaries/configuration and state backup, deploy the exact candidate to the existing tmux service, then verify all three canonical projects. Record checks and limitations in the change acceptance receipt. Roll back binaries and configuration if post-deploy checks fail, accounting for any upstream state migration.

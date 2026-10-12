# Zero-fork overlay acceptance (2026-10-12)

Sealed record of the user-run ChatGPT Web verification of the first zero-fork
deployment. Paths and identities are those observed at acceptance; later
changes do not rewrite this file.

## Candidate

- Release `coordexp-2026.10.12-upstream-v05`; binaries
  `0.5.0 (commit 83d51796b179, dirty=false, built_at=1791821031)`, Rust 1.95.0,
  `dogfood` profile, `workspace-checkpoints` features.
- Source: `origin/main` `e6fe1a67` = upstream `dd57bb6f` + overlay commits;
  former fork retained on `origin/history/fork-0.4.3-main` (`8b8321fc`).
- Rollback: `deployment/rollback/pre-0.5-20261012T155141Z/`.
- OpenSpec: `openspec/changes/archive/2026-10-12-zero-fork-upstream-v05/`.

## Operator smoke (local, read-only)

MCP `initialize` on the `2025-06-18` surface, compact `tools/list` of 35 direct
tools with the v0.5 names present and retired names absent; 14 registered
projects; migrated `sessions.json` (100 records) readable through
`read_session_handoff`; `load_skill web-harness` on five projects; shared-memory
reachable through `mcp_tool` (21 tools); Tunnel `ready_for_chatgpt: true`
with zero restarts after the watchdog removal.

## ChatGPT Web verification (user-run, new conversation after Scan Tools)

The user ran the six-step read-only prompt from a fresh ChatGPT conversation
through the OpenAI Tunnel; all six steps succeeded and the workspace stayed
clean:

1. `load_skill(project="agent:coordexp-web-runner:web-codex", name="web-harness")`
   loaded the 35-line guidance and the model restated the Job/GPU rules.
2. `work_on_project` on `agent:coordexp-web-runner:coordexp` returned a fresh
   Session (`wc_sess_bBFXEntq8Q2PowLm`), branch `main`, HEAD `287c89532804`.
3. `read_files AGENTS.md` lines 1–20, untruncated.
4. `run_process nvidia-smi --query-gpu=index,memory.used --format=csv`
   returned all 8 GPUs.
5. `mcp_tool` list → describe → call `search` on `shared-memory`
   (21 tools, 37 total hits, 8 returned under budget).
6. `read_session_handoff` returned the deterministic handoff for the same
   Session (`ready_to_continue`, 0 file changes, 0 active Jobs).

## Boundaries

- ChatGPT negotiates the legacy `2025-06-18` surface: no `_wc` envelope; the
  Runner `[instructions].files` body is not projected there, which is why the
  guidance is served as the `web-harness` skill.
- Not exercised: edits through `edit_project_files`, Jobs longer than a few
  seconds, `project_validate`, and native Tunnel recovery after a long proxy
  outage.
- `persistent_shell` remains off by decision; the 7-day trace showed no
  state-dependent shell workflows.

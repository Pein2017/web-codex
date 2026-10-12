## 1. Prepare

- [x] 1.1 Stop services, verify process exit, back up `state/server`, `config`, `state/project-registry`, `app`, `bin` to `rollback/pre-0.5-<ts>/`.
- [x] 1.2 Create `overlay/main` from upstream `dd57bb6f`; keep `main` as `history/fork-0.4.3-main`; restore overlay paths; drop watchdog, its test and the execution-recovery smoke.

## 2. Adapt the overlay

- [x] 2.1 `service.sh`/`control.sh`: remove instruction env, watchdog and external tunnel client; `runner.toml` gains `[instructions].files`; relocation tests pass.
- [x] 2.2 Rewrite `deploy/web-workflow/AGENTS.md` and `README.md` for v0.5 tool names and the real environment (≤ 8 KB).
- [x] 2.3 Migrate `sessions.json` (100 records, `workspace_baseline` removed, verified by re-parse and record equality).
- [x] 2.4 Update `PERSISTENT-DEPLOYMENT.md` and `docs/fork-maintenance.md` for the overlay model.

## 3. Build, install, verify

- [ ] 3.1 Commit overlay; build `cargo +1.95.0 --locked --profile dogfood` with `workspace-checkpoints`; identity `0.5.0 (commit dd57bb6f, dirty=false)`.
- [ ] 3.2 Install as `releases/coordexp-2026.10.12-upstream-v05`, copy to `app/` and `bin/`, write `RELEASE.json` with installed sha256s.
- [ ] 3.3 Start; smoke: MCP `initialize` + `tools/list` (new names present, `apply_text_edits` absent), Runner registered with 14 projects, Sessions loaded, Tunnel ready and no restart loop.
- [ ] 3.4 Rename local branches (`overlay/main` → `main`); report push options without pushing.

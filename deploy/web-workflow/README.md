# Self-hosted Web workflow

This fork keeps upstream runtime contracts and adds a small deployment overlay. The repository root `AGENTS.md` guides developers; the sibling [AGENTS.md](AGENTS.md) is the Runner-global instruction file served to the web model.

## Configuration

Runner-global guidance uses upstream's Runner-level configured instructions (see `docs/RUNNER.md`):

```toml
[instructions]
files = ["/data/CoordExp/codex-tools/web-codex/deployment/app/AGENTS.md"]
```

The path must be absolute, Runner-local, at most 1 MiB and not a symlink (configure the resolved physical path). Its body reaches the model only when `work_on_project` is called with `_wc.context=["project.instructions", ...]`, and it shares one 20 KiB envelope with the project's own `AGENTS.md` / `.codex/AGENTS.md`; global text is shortened first when over budget, so keep it at 8 KB or less with the most important rules first. Restart the Runner after changing it.

Shared skills come from absolute operator-controlled roots:

```toml
[skills]
roots = ["/data/CoordExp/.codex/skills", "/data/CoordExp/codex-tools/shared-memory-mcp/skills"]
```

Use `list_skills`, `load_skill` and `read_skill_file`; skills written for Codex CLI are not executable in this host.

Shared memory is a Runner-owned stdio MCP provider (`[[mcp.providers]] id = "shared-memory"`) reached through `mcp_tool` list/describe/call.

Memory reads, pytest report parsing, CodeGraph scoped queries and optional public history reads come from the optional [web-workflow plugin](../../plugins/web-workflow/README.md). Use `plugin_tool(action="list")` to enumerate, `plugin_tool(action="describe")` for the selected tool's schema, then reuse that binding for `call`; use `check` only for operator configuration diagnosis. `public_history_read` (thread id, source-bound cursor, `forward`/`latest` modes) is documented in that README and by `plugin_tool describe`. Keep private roots and credentials outside this repository.

## Normal use

Select the exact registered Project in `work_on_project` (one Session = one Project), retain the Session identifier, and pass `_wc.record` on calls that belong to that Session. Edit through `read_files -> edit_project_files`, copying `read_revision` to `expected_read_revision` unchanged. Run pytest through `project_validate(action=test, adapter=python)`. Retain Job continuations (`wait_for_job_readiness`, `observe_jobs`) and never redispatch. Every Job kind, including `run_detached_process`, is clamped by `[policy] max_timeout_secs = 3600`; long trainings run in the user's default tmux server outside WebCodex. The Runner itself runs inside `control.sh`'s private tmux server and `TMUX` is inherited by Jobs, so model-issued tmux commands must pass `-L default`. No ChatGPT auto-wake integration is provided; results are retrieved manually.

`python` on the Runner resolves to the Conda `ms` environment (`service.sh` prepends `/root/miniconda3/envs/ms/bin`). `persistent_shell = false`, so session-shell tools are not exposed. `import_host_files` requires trusted host-file provenance that ChatGPT does not supply.

The merged upstream Git review path requires Git with `check-attr --source` support. Verify `git check-attr --source=HEAD binary -- README.md` in this checkout before deployment and use the operator-managed Git on the Runner PATH. Python report parsing requires Python 3 and the optional plugin requires Node.js 18+.

## Fork maintenance and delivery

- `upstream` is the original WebCodex repository; `origin` is the operator's fork. See [docs/fork-maintenance.md](../../docs/fork-maintenance.md).
- Use scoped development branches and repository-local OpenSpec changes. Merge upstream before accepting an update, preserving original history and only demonstrated local requirement gaps.
- Fork deployment tags use `coordexp-YYYY.MM.DD.N`: self-hosted Linux prereleases, not upstream npm/desktop/container releases. The inherited container-release workflow skips `coordexp-*` tags.
- Before deployment: focused changed-contract tests, plugin tests, strict OpenSpec validation and a disposable real Server/Runner smoke. Build matching Server/Runner binaries from the same clean commit with the `dogfood` profile; record checksums and build identities. Local deployment alone does not create a tag or GitHub Release.
- Retain old binaries, operator config and a consistent Server-state backup; check active jobs before restarting the Server/Runner. See [PERSISTENT-DEPLOYMENT.md](PERSISTENT-DEPLOYMENT.md) and the acceptance notes in this directory.
- Publish only reviewed source and Linux artifacts to the fork. Never publish private configuration, memory content or operational credentials.
- After deployment, verify MCP initialization, effective guidance identity, registered projects and the edit/test path on a disposable project. Refresh the ChatGPT connection when tool metadata changes, then start a new conversation if it retained old schemas.

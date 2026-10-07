# Maintained WebCodex fork

## Ownership and names

- Fork: <https://github.com/Pein2017/web-codex>, parent
  <https://github.com/yyjeqhc/webcodex>.
- Canonical local source: `/data/CoordExp/codex-tools/web-codex`.
- Local and GitHub default maintained branch: `main`, tracking `origin/main`.
- Retain upstream license, authorship and history. Renaming the GitHub repository
  does not rename the WebCodex product, Rust crates, npm packages, binaries,
  authentication audiences or runtime state.
- This independent repository owns its local OpenSpec plans and source contracts.
  Parent CoordExp is a host location, not an implicit source/specification owner.

On 2026-10-07, the user authorized renaming `Pein2017/webcodex` to
`Pein2017/web-codex` and promoting the maintained `coordexp/web-workflow` branch
(tip `b60a259e`) to `main`. GitHub branch rename preserves the former remote main
(tip `2da7c40b`) as `history/main-before-unification-20261007`.
No merge, force-push, package rename or runtime deployment was involved.
Historical release/acceptance records mentioning old names are retained unchanged.
GitHub may redirect old URLs; use the new explicit repository name for future work.

## Remotes and reviewed synchronization

Use `origin` for the fork and `upstream` for the original author. Disable upstream
pushes and set the default push destination in each clone:

```bash
git clone https://github.com/Pein2017/web-codex.git
cd web-codex
git remote add upstream https://github.com/yyjeqhc/webcodex.git
git remote set-url --push upstream no_push://yyjeqhc/webcodex
git config remote.pushDefault origin
git branch --set-upstream-to=origin/main main
```

Upstream changes are not automatically merged. Fetching updates references only:

```bash
git fetch --no-tags upstream main
git log --oneline HEAD..upstream/main
git diff --stat HEAD...upstream/main
```

Before integration, reconcile dirty ownership and commit only authorized changes.
Do not automatically stash, reset or force-sync. Use a clean review branch:

```bash
git fetch --no-tags origin main
git merge --ff-only origin/main
git switch -c sync/upstream-YYYYMMDD
git merge --no-ff upstream/main
# Review conflicts and retained local contracts; run affected tests.
# Push/open a PR only under the applicable authorization:
gh pr create --repo Pein2017/web-codex --base main
```

Never rebase published main or use `gh repo sync --force` to erase custom work.
Preserve credentials, process-tree ownership, transport, durable sessions and
bounded execution semantics. No automatic upstream merge or deployment is
installed by this normalization. Existing CI/release policies are not changed;
source publication is separate from release tags, registry publication and
runtime adoption.

## Runtime boundary

The named local deployment remains owned by
[container recovery](../deploy/web-workflow/PERSISTENT-DEPLOYMENT.md) and the
repository's controlled update/release instructions. Source branch/repository
renames alone do not require installing packages or restarting native processes.
Keep databases, credentials, sessions, binaries and historical evidence out of
mechanical name updates. Rebuild or deploy only when required by a separately
qualified runtime change; preserve rollback and matching Server/Runner builds.

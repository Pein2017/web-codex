#!/usr/bin/env bash
# Operator-local lifecycle; dedicated tmux socket never targets research sessions.
set -euo pipefail
root="${WEBCODEX_DEPLOY_ROOT:-/data/CoordExp/codex-tools/web-codex/deployment}"
socket="$root/state/tmux.sock"
session=webcodex
action="${1:-status}"
case "$action" in
  start)
    for path in "$root/app/bin/webcodex-server" "$root/app/bin/webcodex-runner" \
      "$root/app/bin/webcodex-cli" "$root/runtime/bin/tunnel-client" \
      "$root/runtime/bin/rg" "$root/runtime/git/bin/git" "$root/bin/service.sh" \
      "$root/../dependencies/codegraph/node_modules/@colbymchenry/codegraph-linux-x64/node"; do
      test -x "$path" || { echo "Missing executable: $path" >&2; exit 1; }
    done
    for path in "$root/config/server.env" "$root/config/runner.toml" "$root/config/tunnel.env"; do
      test -r "$path" || { echo "Missing private config: $path" >&2; exit 1; }
    done
    for path in AGENTS.md plugins/web-workflow/plugin.mjs \
      plugins/web-workflow/pytest_report.py plugins/web-workflow/package.json \
      sandbox_preflight.py; do
      test -r "$root/app/$path" || { echo "Missing installed application file: $path" >&2; exit 1; }
    done
    command -v tmux >/dev/null
    command -v python3 >/dev/null
    mkdir -p "$root/state" "$root/logs"
    chmod 700 "$root/state" "$root/logs"
    if tmux -S "$socket" has-session -t "$session" 2>/dev/null; then
      echo "WebCodex session already exists; inspect status, do not duplicate it."
      exit 0
    fi
    # tmux may retain an environment from before relocation. Bind every window
    # explicitly to the single installed application and its persistent state.
    printf -v launcher 'env %q %q %q' "WEBCODEX_DEPLOY_ROOT=$root" \
      "XDG_STATE_HOME=$root/state/xdg/state" "$root/bin/service.sh"
    tmux -S "$socket" new-session -d -s "$session" -n server -c /data/CoordExp "$launcher server"
    tmux -S "$socket" new-window -d -t "$session" -n runner -c /data/CoordExp "$launcher runner"
    tmux -S "$socket" new-window -d -t "$session" -n tunnel -c /data/CoordExp "$launcher tunnel"
    echo "Started WebCodex service loops; verify runtime and Tunnel connectivity separately."
    ;;
  stop)
    # Caller must reconcile live Jobs/active work before explicitly stopping services.
    if tmux -S "$socket" has-session -t "$session" 2>/dev/null; then
      tmux -S "$socket" kill-session -t "$session"
      echo "Requested shutdown of the dedicated WebCodex session; verify process exit before changing the installation."
    fi
    ;;
  status)
    if tmux -S "$socket" has-session -t "$session" 2>/dev/null; then
      tmux -S "$socket" list-panes -a -F '#{session_name}:#{window_name} pid=#{pane_pid} dead=#{pane_dead}'
    else
      echo "WebCodex services are stopped."
    fi
    ;;
  *) echo "Usage: control.sh {start|status|stop}" >&2; exit 2 ;;
esac

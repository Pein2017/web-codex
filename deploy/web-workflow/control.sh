#!/usr/bin/env bash
# Operator-local lifecycle; dedicated tmux socket never targets research sessions.
set -euo pipefail
root="${WEBCODEX_DEPLOY_ROOT:-/data/CoordExp/codex-tools/web-codex/deployment}"
socket="$root/state/tmux.sock"
session=webcodex
action="${1:-status}"
case "$action" in
  start)
    for path in "$root/current/bin/webcodex-server" "$root/current/bin/webcodex-runner" \
      "$root/runtime/bin/webcodex-cli" "$root/runtime/bin/tunnel-client" \
      "$root/runtime/bin/node" "$root/runtime/git/bin/git" "$root/bin/service.sh"; do
      test -x "$path" || { echo "Missing executable: $path" >&2; exit 1; }
    done
    for path in "$root/config/server.env" "$root/config/runner.toml" "$root/config/tunnel.env"; do
      test -r "$path" || { echo "Missing private config: $path" >&2; exit 1; }
    done
    command -v tmux >/dev/null
    command -v python3 >/dev/null
    test -r "$root/current/plugins/web-workflow/pytest_report.py" || {
      echo "Release is missing the pytest report helper" >&2; exit 1;
    }
    mkdir -p "$root/state" "$root/logs"
    chmod 700 "$root/state" "$root/logs"
    if tmux -S "$socket" has-session -t "$session" 2>/dev/null; then
      echo "WebCodex session already exists; inspect status, do not duplicate it."
      exit 0
    fi
    # New releases own their environment; old releases retain the legacy launcher.
    service_script="$root/current/service.sh"
    if [[ ! -x "$service_script" ]]; then service_script="$root/bin/service.sh"; fi
    # tmux may retain an environment from before relocation. Bind every window
    # explicitly without modifying the selected immutable release launcher.
    printf -v launcher 'env %q %q %q' "WEBCODEX_DEPLOY_ROOT=$root" \
      "XDG_STATE_HOME=$root/state/xdg/state" "$service_script"
    tmux -S "$socket" new-session -d -s "$session" -n server -c /data/CoordExp "$launcher server"
    tmux -S "$socket" new-window -d -t "$session" -n runner -c /data/CoordExp "$launcher runner"
    tmux -S "$socket" new-window -d -t "$session" -n tunnel -c /data/CoordExp "$launcher tunnel"
    echo "Started WebCodex service loops; verify runtime and Tunnel connectivity separately."
    ;;
  stop)
    # Caller must reconcile live Jobs/active work before explicitly stopping services.
    if tmux -S "$socket" has-session -t "$session" 2>/dev/null; then
      tmux -S "$socket" kill-session -t "$session"
      echo "Requested shutdown of the dedicated WebCodex session; verify process exit before switching."
    fi
    ;;
  status)
    readlink "$root/current" || echo "No current release selected."
    if tmux -S "$socket" has-session -t "$session" 2>/dev/null; then
      tmux -S "$socket" list-panes -a -F '#{session_name}:#{window_name} pid=#{pane_pid} dead=#{pane_dead}'
    else
      echo "WebCodex services are stopped."
    fi
    ;;
  switch)
    version="${2:?Usage: control.sh switch <existing-release>}"
    [[ "$version" =~ ^[A-Za-z0-9][A-Za-z0-9._-]*$ ]] || { echo "Invalid release name" >&2; exit 1; }
    for path in bin/webcodex-server bin/webcodex-runner AGENTS.md plugins/web-workflow/plugin.mjs plugins/web-workflow/pytest_report.py; do
      test -f "$root/releases/$version/$path" || { echo "Incomplete release: $version" >&2; exit 1; }
    done
    if tmux -S "$socket" has-session -t "$session" 2>/dev/null; then
      echo "Stop and verify WebCodex services before switching releases." >&2
      exit 1
    fi
    next="$root/.current.$$.tmp"
    trap 'rm -f -- "$next"' EXIT
    ln -s "releases/$version" "$next"
    mv -Tf -- "$next" "$root/current"
    echo "Selected $version; persistent state unchanged."
    ;;
  *) echo "Usage: control.sh {start|status|stop|switch <existing-release>}" >&2; exit 2 ;;
esac

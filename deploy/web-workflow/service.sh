#!/usr/bin/env bash
set -euo pipefail
umask 077
root="${WEBCODEX_DEPLOY_ROOT:-/data/CoordExp/codex-tools/web-codex/deployment}"
service="${1:?Expected server, runner or tunnel}"
case "$service" in server|runner|tunnel) ;; *) exit 2 ;; esac
export PATH="$root/runtime/bin:$root/runtime/git/bin:/usr/bin:/bin"
export GIT_EXEC_PATH="$root/runtime/git/libexec/git-core"
export GIT_TEMPLATE_DIR="$root/runtime/git/share/git-core/templates"
export GIT_CONFIG_GLOBAL="$root/config/gitconfig"
export GH_CONFIG_DIR="$root/config/gh"
export XDG_CONFIG_HOME="$root/state/xdg/config"
export XDG_DATA_HOME="$root/state/xdg/data"
export XDG_CACHE_HOME="$root/state/xdg/cache"
export XDG_STATE_HOME="$root/state/xdg/state"
export HTTP_PROXY=http://127.0.0.1:9090 HTTPS_PROXY=http://127.0.0.1:9090 ALL_PROXY=http://127.0.0.1:9090
export http_proxy="$HTTP_PROXY" https_proxy="$HTTPS_PROXY" all_proxy="$ALL_PROXY"
export NO_PROXY=127.0.0.1,localhost no_proxy=127.0.0.1,localhost
mkdir -p "$root/logs" "$XDG_CONFIG_HOME" "$XDG_DATA_HOME" "$XDG_CACHE_HOME" "$XDG_STATE_HOME"
exec >> "$root/logs/$service.log" 2>&1
case "$service" in
  server)
    export WEBCODEX_AUTHORITY_MODE=trusted_agent
    export WEBCODEX_MCP_COMPACT_SCHEMAS=true
    export WEBCODEX_MCP_INSTRUCTIONS_FILE="$root/current/AGENTS.md"
    export WEBCODEX_ENV_FILE="$root/config/server.env"
    command=("$root/current/bin/webcodex-server")
    ;;
  runner)
    # This deployment's execution environment; do not change Server/Tunnel PATH.
    for executable in /root/miniconda3/envs/ms/bin/python /root/miniconda3/envs/ms/bin/python3; do
      test -x "$executable" || { echo "Missing ms interpreter: $executable; restore the Conda environment before starting Runner." >&2; exit 1; }
    done
    # Keep persistent operator tools (notably Git) ahead of Conda tools.
    export PATH="$root/runtime/bin:$root/runtime/git/bin:/root/miniconda3/envs/ms/bin:/usr/bin:/bin"
    command=("$root/current/bin/webcodex-runner" --config "$root/config/runner.toml")
    ;;
  tunnel)
    set -a
    source "$root/config/tunnel.env"
    set +a
    export WEBCODEX_TUNNEL_CLIENT_BIN="$root/runtime/bin/tunnel-client"
    command=("$root/current/bin/webcodex-cli" server tunnel --provider openai
      --env-file "$root/config/server.env" --json --stop-on-stdin-eof)
    ;;
esac
child=
shutdown() {
  trap '' TERM INT HUP
  if [[ -n "$child" ]]; then
    kill -TERM "$child" 2>/dev/null || true
    wait "$child" 2>/dev/null || true
  fi
  exit 0
}
trap shutdown TERM INT HUP
while true; do
  status=0
  "${command[@]}" <&0 &
  child=$!
  wait "$child" || status=$?
  child=
  printf '%s %s exited status=%s; retrying in 15 seconds\n' "$(date --iso-8601=seconds)" "$service" "$status"
  sleep 15 &
  child=$!
  wait "$child" || true
  child=
done

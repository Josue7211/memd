#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ENV_PATH="${MAC_BRIDGE_ENV:-$ROOT/integrations/mac-bridge/.env}"
PLIST_PATH="${MAC_BRIDGE_PLIST:-$HOME/Library/LaunchAgents/com.memd.mac-bridge.plist}"
LABEL="${MAC_BRIDGE_LABEL:-com.memd.mac-bridge}"
BRIDGE_PORT_FROM_ENV="${BRIDGE_PORT:-}"
BRIDGE_PORT="${BRIDGE_PORT:-4100}"
BRIDGE_API_KEY="${MAC_BRIDGE_API_KEY:-}"
CONNECT_TIMEOUT="${MAC_BRIDGE_CONNECT_TIMEOUT_SECS:-0.6}"
MAX_TIME="${MAC_BRIDGE_MAX_TIME_SECS:-1.2}"
RESTART_ON_UNHEALTHY="${MAC_BRIDGE_RESTART_ON_UNHEALTHY:-1}"

say() {
  printf 'memd mac-bridge guard: %s\n' "$*" >&2
}

if [[ -f "$ENV_PATH" ]]; then
  while IFS='=' read -r key value; do
    case "$key" in
      BRIDGE_PORT)
        if [[ -z "$BRIDGE_PORT_FROM_ENV" ]]; then
          BRIDGE_PORT="${value%$'\r'}"
        fi
        ;;
      BRIDGE_API_KEY)
        if [[ -z "$BRIDGE_API_KEY" ]]; then
          value="${value%$'\r'}"
          value="${value#\"}"
          value="${value%\"}"
          value="${value#\'}"
          value="${value%\'}"
          BRIDGE_API_KEY="$value"
        fi
        ;;
    esac
  done < "$ENV_PATH"
fi

health_url="http://127.0.0.1:${BRIDGE_PORT}/health"

probe() {
  [[ -n "$BRIDGE_API_KEY" ]] || return 2
  curl -fsS \
    --connect-timeout "$CONNECT_TIMEOUT" \
    --max-time "$MAX_TIME" \
    -H "X-API-Key: $BRIDGE_API_KEY" \
    "$health_url" >/dev/null
}

if probe; then
  exit 0
fi

say "unhealthy url=$health_url; memd-owned bridge only, no ClawControl launch"

if [[ "$RESTART_ON_UNHEALTHY" != "1" && "$RESTART_ON_UNHEALTHY" != "true" ]]; then
  exit 2
fi

if [[ "$(uname -s)" != "Darwin" || ! -f "$PLIST_PATH" || ! -x "$(command -v launchctl)" ]]; then
  say "restart unavailable"
  exit 2
fi

domain="gui/$(id -u)"
launchctl bootout "$domain" "$PLIST_PATH" >/dev/null 2>&1 || true
launchctl bootstrap "$domain" "$PLIST_PATH" >/dev/null 2>&1 || launchctl load "$PLIST_PATH" >/dev/null 2>&1 || {
  say "restart failed"
  exit 2
}
launchctl kickstart -k "$domain/$LABEL" >/dev/null 2>&1 || true

for _ in {1..20}; do
  if probe; then
    say "ready after restart url=$health_url"
    exit 0
  fi
  sleep 0.25
done

say "still unhealthy after restart"
exit 2

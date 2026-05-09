#!/usr/bin/env bash
set -euo pipefail

BRIDGE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PLIST_NAME="com.memd.mac-bridge.plist"
PLIST_PATH="$HOME/Library/LaunchAgents/$PLIST_NAME"
LABEL="com.memd.mac-bridge"
LOCAL_BIN="${MEMD_LOCAL_BIN:-$HOME/.local/bin}"
BRIDGE_PORT="${BRIDGE_PORT:-4100}"
REMINDCTL_VERSION="${REMINDCTL_VERSION:-v0.2.0}"
REMINDCTL_SHA256="${REMINDCTL_SHA256:-1d931f010cb11ed617dd0ff3ab273a3a593d0ef341adda0d8b04e3a1af7d0f60}"

say() {
  printf 'memd mac-bridge: %s\n' "$*"
}

fail() {
  printf 'memd mac-bridge: error: %s\n' "$*" >&2
  exit 1
}

if [ "$(uname -s)" != "Darwin" ]; then
  say "skipping; macOS required"
  exit 0
fi

command -v node >/dev/null 2>&1 || fail "Node.js missing"
command -v npm >/dev/null 2>&1 || fail "npm missing"
command -v curl >/dev/null 2>&1 || fail "curl missing"
command -v unzip >/dev/null 2>&1 || fail "unzip missing"
command -v openssl >/dev/null 2>&1 || fail "openssl missing"

mkdir -p "$LOCAL_BIN" "$HOME/Library/LaunchAgents"

ENV_PATH="$BRIDGE_DIR/.env"
if [ -f "$ENV_PATH" ]; then
  # shellcheck disable=SC1090
  set -a && . "$ENV_PATH" && set +a
fi

BRIDGE_PORT="${BRIDGE_PORT:-4100}"
BRIDGE_API_KEY="${BRIDGE_API_KEY:-}"
if [ -z "$BRIDGE_API_KEY" ]; then
  BRIDGE_API_KEY="$(openssl rand -hex 32)"
fi

tmp_env="$(mktemp)"
{
  printf 'BRIDGE_PORT=%s\n' "$BRIDGE_PORT"
  printf 'BRIDGE_API_KEY=%s\n' "$BRIDGE_API_KEY"
} >"$tmp_env"
mv "$tmp_env" "$ENV_PATH"
chmod 600 "$ENV_PATH"

if [ ! -d "$BRIDGE_DIR/node_modules" ]; then
  say "installing Node dependencies"
  (cd "$BRIDGE_DIR" && npm install)
fi

install_remindctl() {
  if command -v remindctl >/dev/null 2>&1; then
    say "found remindctl at $(command -v remindctl)"
    return 0
  fi
  if [ -x "$LOCAL_BIN/remindctl" ]; then
    say "found remindctl at $LOCAL_BIN/remindctl"
    return 0
  fi

  local tmp zip bin
  tmp="$(mktemp -d)"
  zip="$tmp/remindctl-macos.zip"
  bin="$tmp/remindctl"
  say "installing remindctl $REMINDCTL_VERSION"
  curl -fsSL \
    "https://github.com/openclaw/remindctl/releases/download/$REMINDCTL_VERSION/remindctl-macos.zip" \
    -o "$zip"
  actual_sha="$(shasum -a 256 "$zip" | awk '{print $1}')"
  if [ "$actual_sha" != "$REMINDCTL_SHA256" ]; then
    fail "remindctl checksum mismatch"
  fi
  unzip -q "$zip" -d "$tmp"
  install -m 0755 "$bin" "$LOCAL_BIN/remindctl"
  xattr -dr com.apple.quarantine "$LOCAL_BIN/remindctl" 2>/dev/null || true
  rm -rf "$tmp"
}

install_remindctl

seed_calendar_cache() {
  local calendar_db cache_dir cache_file
  calendar_db="$HOME/Library/Group Containers/group.com.apple.calendar/Calendar.sqlitedb"
  cache_dir="${TMPDIR:-/tmp}/mac-bridge-private"
  cache_file="$cache_dir/calendar-cache.json"
  if [ ! -r "$calendar_db" ] || ! command -v sqlite3 >/dev/null 2>&1; then
    return 0
  fi
  mkdir -p "$cache_dir"
  chmod 700 "$cache_dir"
  local query
  query="
    SELECT
      ci.ROWID AS id,
      COALESCE(ci.summary, '') AS title,
      strftime('%Y-%m-%dT%H:%M:%SZ', ci.start_date + 978307200, 'unixepoch') AS start,
      strftime('%Y-%m-%dT%H:%M:%SZ', ci.end_date + 978307200, 'unixepoch') AS end,
      CASE WHEN COALESCE(ci.all_day, 0) = 0 THEN 0 ELSE 1 END AS allDay,
      COALESCE(c.title, '') AS calendar
    FROM CalendarItem ci
    LEFT JOIN Calendar c ON c.ROWID = ci.calendar_id
    WHERE ci.start_date BETWEEN (strftime('%s','now','-30 days') - 978307200)
      AND (strftime('%s','now','+30 days') - 978307200)
      AND COALESCE(ci.hidden, 0) = 0
    ORDER BY ci.start_date ASC
    LIMIT 500
  "
  if events="$(sqlite3 -json "$calendar_db" "$query" 2>/dev/null)"; then
    printf '{"events":%s,"cachedAt":"%s"}\n' "$events" "$(date -u +"%Y-%m-%dT%H:%M:%SZ")" >"$cache_file"
    chmod 600 "$cache_file"
    say "seeded Calendar cache"
  fi
}

seed_calendar_cache

NODE_PATH="$(command -v node)"
if command -v realpath >/dev/null 2>&1; then
  NODE_PATH="$(realpath "$NODE_PATH")"
fi
PATH_VALUE="$LOCAL_BIN:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:$PATH"

if launchctl print "gui/$(id -u)/$LABEL" >/dev/null 2>&1; then
  say "stopping existing service"
  launchctl bootout "gui/$(id -u)" "$PLIST_PATH" 2>/dev/null || true
elif launchctl list 2>/dev/null | grep -q "$LABEL"; then
  launchctl unload "$PLIST_PATH" 2>/dev/null || true
fi

cat >"$PLIST_PATH" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>$LABEL</string>
    <key>ProgramArguments</key>
    <array>
        <string>$NODE_PATH</string>
        <string>$BRIDGE_DIR/server.js</string>
    </array>
    <key>WorkingDirectory</key>
    <string>$BRIDGE_DIR</string>
    <key>EnvironmentVariables</key>
    <dict>
        <key>BRIDGE_PORT</key>
        <string>$BRIDGE_PORT</string>
        <key>BRIDGE_API_KEY</key>
        <string>$BRIDGE_API_KEY</string>
        <key>PATH</key>
        <string>$PATH_VALUE</string>
    </dict>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/tmp/memd-mac-bridge.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/memd-mac-bridge.log</string>
</dict>
</plist>
EOF

chmod 600 "$PLIST_PATH"
launchctl bootstrap "gui/$(id -u)" "$PLIST_PATH" 2>/dev/null || launchctl load "$PLIST_PATH"

say "installed"
say "url: http://127.0.0.1:$BRIDGE_PORT"
say "env: $ENV_PATH"
say "logs: /tmp/memd-mac-bridge.log"
say "run '~/.local/bin/remindctl authorize' if Reminders access is not granted yet"

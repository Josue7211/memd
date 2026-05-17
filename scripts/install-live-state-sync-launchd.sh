#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LABEL="${LABEL:-com.memd.live-state-sync-clawcontrol}"
INTERVAL_SECS="${INTERVAL_SECS:-300}"
TARGET="${TARGET:-$HOME/Library/LaunchAgents/$LABEL.plist}"
ACTION="${1:---print}"

domain() {
  echo "gui/$(id -u)"
}

need_launchd() {
  if [[ "$(uname -s)" != "Darwin" ]]; then
    echo "launchd install is only supported on macOS" >&2
    exit 1
  fi
  command -v launchctl >/dev/null 2>&1 || {
    echo "launchctl command not found" >&2
    exit 127
  }
}

usage() {
  cat <<USAGE
usage: $0 [--print|--install|--uninstall]

Environment:
  LABEL=$LABEL
  INTERVAL_SECS=$INTERVAL_SECS
  TARGET=$TARGET

The generated launchd job runs scripts/live-state-sync-clawcontrol.sh every
INTERVAL_SECS seconds. The sync script only imports when memd live-state is
missing, stale, or due, so frequent launchd checks are safe.
USAGE
}

render_plist() {
  local script_path="$ROOT/scripts/live-state-sync-clawcontrol.sh"
  cat <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>$LABEL</string>
  <key>ProgramArguments</key>
  <array>
    <string>$script_path</string>
  </array>
  <key>StartInterval</key>
  <integer>$INTERVAL_SECS</integer>
  <key>RunAtLoad</key>
  <true/>
  <key>StandardOutPath</key>
  <string>$HOME/Library/Logs/$LABEL.out.log</string>
  <key>StandardErrorPath</key>
  <string>$HOME/Library/Logs/$LABEL.err.log</string>
</dict>
</plist>
PLIST
}

case "$ACTION" in
  --print)
    render_plist
    ;;
  --install)
    need_launchd
    mkdir -p "$(dirname "$TARGET")" "$HOME/Library/Logs"
    render_plist >"$TARGET"
    launchctl bootout "$(domain)" "$TARGET" >/dev/null 2>&1 || true
    launchctl bootstrap "$(domain)" "$TARGET"
    launchctl kickstart -k "$(domain)/$LABEL" >/dev/null 2>&1 || true
    echo "installed $TARGET"
    ;;
  --uninstall)
    need_launchd
    launchctl bootout "$(domain)" "$TARGET" >/dev/null 2>&1 || true
    rm -f "$TARGET"
    echo "removed $TARGET"
    ;;
  -h|--help)
    usage
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac

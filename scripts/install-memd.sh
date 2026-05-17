#!/usr/bin/env bash
set -euo pipefail

PREFIX="${MEMD_PREFIX:-$HOME/.local}"
BIN_DIR="$PREFIX/bin"
MEMD_BIN="$BIN_DIR/memd"

say() {
  printf 'memd install: %s\n' "$*"
}

fail() {
  printf 'memd install: error: %s\n' "$*" >&2
  exit 1
}

detect_repo_root() {
  if [ -n "${MEMD_REPO:-}" ] && [ -d "$MEMD_REPO/crates/memd-client" ]; then
    printf '%s\n' "$MEMD_REPO"
    return 0
  fi
  if [ -d "crates/memd-client" ]; then
    pwd
    return 0
  fi
  local script_dir
  script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  if [ -d "$script_dir/../crates/memd-client" ]; then
    cd "$script_dir/.." && pwd
    return 0
  fi
  return 1
}

ensure_path_line() {
  local shell_rc="$1"
  local line='export PATH="$HOME/.local/bin:$PATH"'
  [ -f "$shell_rc" ] || touch "$shell_rc"
  if ! grep -Fq "$line" "$shell_rc"; then
    printf '\n%s\n' "$line" >>"$shell_rc"
    say "added ~/.local/bin to PATH in $shell_rc"
  fi
}

memd_supports_current_cli() {
  local candidate="$1"
  [ -x "$candidate" ] || return 1
  "$candidate" capabilities sync --help >/dev/null 2>&1
  "$candidate" live-state --help >/dev/null 2>&1
}

install_memd_from_source() {
  command -v cargo >/dev/null 2>&1 || fail "Rust cargo missing. Install Rust, then rerun this script."
  REPO_ROOT="$(detect_repo_root)" || fail "run from a memd checkout or set MEMD_REPO=/path/to/memd"
  say "building memd from $REPO_ROOT"
  cargo install --path "$REPO_ROOT/crates/memd-client" --bin memd --root "$PREFIX" --locked --force
}

mkdir -p "$BIN_DIR"

EXISTING_MEMD="$(command -v memd 2>/dev/null || true)"
if [ -n "$EXISTING_MEMD" ] && memd_supports_current_cli "$EXISTING_MEMD"; then
  say "found current memd at $EXISTING_MEMD"
elif [ -n "$EXISTING_MEMD" ]; then
  say "found stale memd at $EXISTING_MEMD; rebuilding"
  install_memd_from_source
elif memd_supports_current_cli "$MEMD_BIN"; then
  say "found current memd at $MEMD_BIN"
elif [ -x "$MEMD_BIN" ]; then
  say "found stale memd at $MEMD_BIN; rebuilding"
  install_memd_from_source
else
  install_memd_from_source
fi

case ":$PATH:" in
  *":$BIN_DIR:"*) ;;
  *)
    export PATH="$BIN_DIR:$PATH"
    case "${SHELL:-}" in
      */zsh) ensure_path_line "$HOME/.zshrc" ;;
      */bash) ensure_path_line "$HOME/.bashrc" ;;
      *) say "add $BIN_DIR to PATH if your shell cannot find memd" ;;
    esac
    ;;
esac

if [ -d ".memd" ]; then
  say "bundle already initialized"
else
  say "initializing bundle"
  memd setup --summary
fi

if [ "${MEMD_INSTALL_REPAIR:-0}" = "1" ]; then
  say "repairing/verifying setup"
  memd doctor --repair --summary
else
  say "verifying setup"
  memd doctor --summary
fi

if [ -s ".memd/state/devices.json" ] || find "docs/verification/release-1-0-0/devices" -maxdepth 1 -type f -name '*device*.json' 2>/dev/null | grep -q .; then
  say "device evidence already present"
else
  say "registering this machine"
  memd device add --summary
fi

if [ "$(uname -s)" = "Darwin" ] && [ "${MEMD_INSTALL_MAC_BRIDGE:-1}" != "0" ]; then
  REPO_ROOT="$(detect_repo_root)" || fail "run from a memd checkout or set MEMD_REPO=/path/to/memd"
  if [ -x "$REPO_ROOT/integrations/mac-bridge/install.sh" ]; then
    say "installing bundled Mac Bridge"
    "$REPO_ROOT/integrations/mac-bridge/install.sh"
  else
    fail "bundled Mac Bridge installer missing"
  fi
fi

say "ready"
say "next: run 'memd dogfood enroll --user-id <your-name> --consent --summary' if this is a real dogfood machine"

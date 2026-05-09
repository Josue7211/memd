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

mkdir -p "$BIN_DIR"

if command -v memd >/dev/null 2>&1; then
  say "found existing memd at $(command -v memd)"
elif [ -x "$MEMD_BIN" ]; then
  say "found existing memd at $MEMD_BIN"
else
  command -v cargo >/dev/null 2>&1 || fail "Rust cargo missing. Install Rust, then rerun this script."
  REPO_ROOT="$(detect_repo_root)" || fail "run from a memd checkout or set MEMD_REPO=/path/to/memd"
  say "building memd from $REPO_ROOT"
  cargo install --path "$REPO_ROOT/crates/memd-client" --bin memd --root "$PREFIX" --locked
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

say "initializing bundle"
memd setup --summary

say "repairing/verifying setup"
memd doctor --repair --summary

say "registering this machine"
memd device add --summary

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

#!/usr/bin/env bash
# G4.5 — V4 proof harness CI entrypoint.
#
# Runs the v4_proof_harness test suite. Exit 0 on green, 1 on memd
# regression. Retries ONCE on infra flake (closed allow-list, mirrors
# `is_infra_flake` in crates/memd-client/src/main_tests/v4_proof_harness/ci.rs).
#
# Usage:
#   bash scripts/ci/v4-proof-harness.sh
#
# Env:
#   MEMD_G4_HARNESS_FAIL_FAST  default 1 — stop at first failed assertion.
#   CARGO_TARGET_DIR           default /tmp/memd-target.

set -eo pipefail

: "${CARGO_TARGET_DIR:=/tmp/memd-target}"
: "${MEMD_G4_HARNESS_FAIL_FAST:=1}"
export CARGO_TARGET_DIR MEMD_G4_HARNESS_FAIL_FAST

TMPDIR_V4=""
cleanup() {
  if [[ -n "$TMPDIR_V4" && -d "$TMPDIR_V4" ]]; then
    rm -rf "$TMPDIR_V4"
  fi
}
trap cleanup EXIT

readonly INFRA_FLAKE_PATTERNS=(
  "No space left on device"
  "Resource temporarily unavailable"
  "Connection refused"
  "Network is unreachable"
  "Too many open files"
  "tmpfile create"
  "stale NFS file handle"
)

is_infra_flake() {
  local stderr_text="$1"
  for pat in "${INFRA_FLAKE_PATTERNS[@]}"; do
    if [[ "$stderr_text" == *"$pat"* ]]; then
      return 0
    fi
  done
  return 1
}

run_once() {
  local stderr_log="$1"
  cargo test \
    --target-dir "$CARGO_TARGET_DIR" \
    -p memd-client \
    v4_proof_harness \
    2>"$stderr_log"
}

main() {
  TMPDIR_V4="$(mktemp -d)"

  local stderr1="$TMPDIR_V4/stderr-1.log"
  if run_once "$stderr1"; then
    echo "[v4-proof] PASS on first attempt"
    return 0
  fi

  local stderr_text
  stderr_text="$(cat "$stderr1")"
  if is_infra_flake "$stderr_text"; then
    echo "[v4-proof] infra flake detected, retrying once" >&2
    local stderr2="$TMPDIR_V4/stderr-2.log"
    if run_once "$stderr2"; then
      echo "[v4-proof] PASS on retry"
      return 0
    fi
    echo "[v4-proof] FAIL on retry — surfacing stderr" >&2
    cat "$stderr2" >&2
    return 1
  fi

  echo "[v4-proof] FAIL — memd regression, no retry" >&2
  cat "$stderr1" >&2
  return 1
}

main "$@"

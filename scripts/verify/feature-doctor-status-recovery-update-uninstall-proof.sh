#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MEMD_BIN="${MEMD_BIN:-$ROOT/target/debug/memd}"
KEEP="${MEMD_DOCTOR_RECOVERY_KEEP:-0}"
TMP="$(mktemp -d)"
cleanup() {
  if [[ "$KEEP" == "1" ]]; then
    echo "doctor recovery proof kept temp dir: $TMP"
  else
    rm -rf "$TMP"
  fi
}
trap cleanup EXIT

log() { printf 'proof: %s\n' "$*"; }
fail() { printf 'proof: FAIL: %s\n' "$*" >&2; exit 1; }
run_capture() {
  local name="$1"
  shift
  log "run $name: $*"
  "$@" >"$TMP/$name.out" 2>"$TMP/$name.err"
}
run_expect_fail() {
  local name="$1"
  shift
  log "run expected-fail $name: $*"
  set +e
  "$@" >"$TMP/$name.out" 2>"$TMP/$name.err"
  local status=$?
  set -e
  if [[ "$status" -eq 0 ]]; then
    fail "$name unexpectedly passed"
  fi
  log "$name failed as expected with exit=$status"
}
assert_file() { [[ -f "$1" ]] || fail "missing file: $1"; }
assert_dir() { [[ -d "$1" ]] || fail "missing directory: $1"; }
assert_contains() {
  local file="$1"
  local needle="$2"
  if ! grep -Fq "$needle" "$file"; then
    printf '%s\n' "--- $file ---" >&2
    sed -n '1,160p' "$file" >&2 || true
    fail "expected '$needle' in $file"
  fi
}
assert_not_contains() {
  local file="$1"
  local needle="$2"
  if grep -Fq "$needle" "$file"; then
    fail "unexpected '$needle' in $file"
  fi
}
checksum_tree() {
  local dir="$1"
  (cd "$dir" && find . -type f -print0 | sort -z | xargs -0 sha256sum)
}

if [[ ! -x "$MEMD_BIN" ]]; then
  log "memd binary missing; building with scripts/memd-cargo-guard.sh"
  (cd "$ROOT" && MEMD_CARGO_TARGET_DIR="$ROOT/target" scripts/memd-cargo-guard.sh build -p memd-client --bin memd >/dev/null)
fi
if [[ ! -x "$MEMD_BIN" && -x "${MEMD_CARGO_TARGET_DIR:-${TMPDIR:-/tmp}/memd-cargo-target}/debug/memd" ]]; then
  MEMD_BIN="${MEMD_CARGO_TARGET_DIR:-${TMPDIR:-/tmp}/memd-cargo-target}/debug/memd"
fi
[[ -x "$MEMD_BIN" ]] || fail "memd binary not executable after build: $MEMD_BIN"

SANDBOX="$TMP/project"
BUNDLE="$SANDBOX/.memd"
mkdir -p "$SANDBOX"
cd "$SANDBOX"

log "using memd=$MEMD_BIN"
log "sandbox=$SANDBOX"
run_capture setup "$MEMD_BIN" setup --output "$BUNDLE" --summary --force --allow-localhost-read-only-fallback
assert_contains "$TMP/setup.out" "setup"
assert_dir "$BUNDLE"
assert_file "$BUNDLE/config.json"

mkdir -p "$BUNDLE/memory" "$BUNDLE/state"
printf 'doctor-status-recovery-update-uninstall sentinel\n' >"$BUNDLE/memory/preserve.txt"
printf '{"kind":"sentinel","value":"preserve"}\n' >"$BUNDLE/state/preserve.json"
checksum_tree "$BUNDLE" >"$TMP/before.sha256"

run_capture doctor "$MEMD_BIN" doctor --output "$BUNDLE" --summary
assert_contains "$TMP/doctor.out" "status bundle="
assert_not_contains "$TMP/doctor.out" "panic"
run_capture status "$MEMD_BIN" status --output "$BUNDLE" --summary
assert_contains "$TMP/status.out" "status bundle="
assert_not_contains "$TMP/status.out" "panic"

run_capture doctor_missing_bundle "$MEMD_BIN" doctor --output "$TMP/missing-bundle" --summary
assert_contains "$TMP/doctor_missing_bundle.out" "ready=false"
assert_contains "$TMP/doctor_missing_bundle.out" "missing="
assert_contains "$TMP/doctor_missing_bundle.out" "setup_next="

rm -f "$BUNDLE/env" "$BUNDLE/env.ps1"
run_capture doctor_repair "$MEMD_BIN" doctor --output "$BUNDLE" --repair --summary
assert_contains "$TMP/doctor_repair.out" "status bundle="
assert_file "$BUNDLE/env"
assert_file "$BUNDLE/env.ps1"

# Reset/recovery proof: this repository has no destructive `memd reset` lifecycle
# command. The local 25-star check therefore proves recovery of generated bundle
# files only, and explicitly verifies memory-bearing files remain intact.
run_capture status_after_repair "$MEMD_BIN" status --output "$BUNDLE" --summary
assert_contains "$TMP/status_after_repair.out" "status bundle="
assert_file "$BUNDLE/memory/preserve.txt"
assert_file "$BUNDLE/state/preserve.json"
assert_contains "$BUNDLE/memory/preserve.txt" "sentinel"
assert_contains "$BUNDLE/state/preserve.json" "preserve"
checksum_tree "$BUNDLE" >"$TMP/after-repair.sha256"
assert_contains "$TMP/after-repair.sha256" "./memory/preserve.txt"
assert_contains "$TMP/after-repair.sha256" "./state/preserve.json"

run_capture update_dry_run "$ROOT/scripts/update-memd.sh" --dry-run
assert_contains "$TMP/update_dry_run.out" "dry-run ok; no files changed"
assert_contains "$TMP/update_dry_run.out" "will preserve"
MEMD_BIN="$TMP/nonexistent-memd-bin" run_capture uninstall_dry_run "$ROOT/scripts/uninstall-memd.sh" --dry-run
assert_contains "$TMP/uninstall_dry_run.out" "dry-run ok; would remove binary only"
assert_contains "$TMP/uninstall_dry_run.out" "memory is preserved by default"
assert_file "$BUNDLE/memory/preserve.txt"
assert_file "$BUNDLE/state/preserve.json"

cat >"$TMP/proof-summary.md" <<SUMMARY
# Doctor/status/recovery/update/uninstall local proof

- memd: $MEMD_BIN
- sandbox: $SANDBOX
- doctor: pass
- status: pass
- doctor missing bundle failure mode: pass (reports ready=false/missing/setup_next)
- doctor repair recovery of generated env files: pass
- reset coverage: pending destructive reset command; proved non-destructive generated-file recovery only
- update dry-run: pass
- uninstall dry-run: pass
- memory preservation: pass for sentinel memory/state files
- external validation: pending; this is local executable proof only
SUMMARY

log "proof summary: $TMP/proof-summary.md"
log "feature-doctor-status-recovery-update-uninstall-proof=pass"

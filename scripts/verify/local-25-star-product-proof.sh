#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

STAMP="$(date -u +%Y-%m-%dT%H%M%SZ)"
OUT_DIR="${LOCAL_25_STAR_OUT_DIR:-/tmp/memd-local-25-star-$STAMP}"
mkdir -p "$OUT_DIR"
REPORT="$OUT_DIR/local-25-star-product-proof.md"
MEMD_BIN="${MEMD_BIN:-$ROOT/target/debug/memd}"
if [[ ! -x "$MEMD_BIN" ]]; then
  cargo build -p memd-client --bin memd >/dev/null
fi
if [[ ! -x "$MEMD_BIN" ]]; then
  echo "local-25-star: memd binary not found after cargo build: $MEMD_BIN" >&2
  exit 1
fi

before="$(git status --short)"

{
  echo "# Local 25-Star Product Proof"
  echo
  echo "- date_utc: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "- host: $(hostname 2>/dev/null || echo unknown)"
  echo "- os: $(uname -a)"
  echo "- commit: $(git rev-parse --short HEAD)"
  echo "- memd_bin: $MEMD_BIN"
  echo
  echo "## Gates"
  echo
  echo '```text'
  echo "cargo fmt --check"
  cargo fmt --check
  echo
  echo "cargo test -p memd-client setup_interactive -- --nocapture"
  cargo test -p memd-client setup_interactive -- --nocapture
  echo
  echo "memd setup --guided --summary"
  "$MEMD_BIN" setup --guided --summary
  echo
  echo "memd setup-demo --summary"
  "$MEMD_BIN" setup-demo --summary
  echo
  echo "scripts/update-memd.sh --dry-run"
  scripts/update-memd.sh --dry-run
  echo
  echo "scripts/uninstall-memd.sh --dry-run"
  scripts/uninstall-memd.sh --dry-run
  echo
  echo "scripts/verify/setup-experience-smoke.sh"
  PATH="$ROOT/target/debug:$PATH" scripts/verify/setup-experience-smoke.sh
  echo
  echo "scripts/doc-lint.sh"
  scripts/doc-lint.sh
  echo '```'
  echo
  echo "## External validation"
  echo
  echo "External checklist: docs/verification/EXTERNAL-25-STAR-VERIFIERS.md"
  echo "status: external-validation-pending"
  echo
  echo "## Result"
  echo
  echo "local-25-star-product-proof=pass"
} | tee "$REPORT"

after="$(git status --short)"
if [[ "$before" != "$after" ]]; then
  echo "local-25-star: git status changed during proof" >&2
  diff <(printf '%s\n' "$before") <(printf '%s\n' "$after") || true
  exit 1
fi

echo "local 25-star proof report: $REPORT"

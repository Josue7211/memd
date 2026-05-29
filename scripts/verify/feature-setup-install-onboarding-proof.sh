#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DOC="$ROOT/docs/verification/feature-setup-install-onboarding-25.md"
REGISTRY="$ROOT/docs/verification/features.registry.json"
TMP="$(mktemp -d)"
KEEP="${MEMD_SETUP_ONBOARDING_KEEP:-0}"
REPORT="$TMP/feature-setup-install-onboarding-proof.md"

cleanup() {
  if [ "$KEEP" != "1" ]; then
    rm -rf "$TMP"
  else
    echo "setup onboarding proof kept temp dir: $TMP"
  fi
}
trap cleanup EXIT

fail() {
  echo "feature-setup-install-onboarding-proof: ERROR: $*" >&2
  exit 1
}

require_file() {
  local file="$1"
  [ -f "$ROOT/$file" ] || fail "missing required file: $file"
}

require_executable() {
  local file="$1"
  [ -x "$ROOT/$file" ] || fail "required script is not executable: $file"
}

require_contains() {
  local file="$1"
  local needle="$2"
  local path="$ROOT/$file"
  case "$file" in
    /*) path="$file" ;;
  esac
  python3 - "$path" "$needle" <<'PY'
import sys
from pathlib import Path
path = Path(sys.argv[1])
needle = sys.argv[2]
text = path.read_text(encoding="utf-8")
if needle not in text:
    print(f"missing {needle!r} in {path}", file=sys.stderr)
    sys.exit(1)
PY
}

require_registry_value() {
  local expr="$1"
  python3 - "$REGISTRY" "$expr" <<'PY'
import json
import sys
from pathlib import Path
registry = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
expr = sys.argv[2]
feature = next((f for f in registry["features"] if f.get("id") == "feature.setup_install_onboarding"), None)
if feature is None:
    print("missing feature.setup_install_onboarding", file=sys.stderr)
    sys.exit(1)
checks = {
    "proof-command": lambda f: "bash scripts/verify/feature-setup-install-onboarding-proof.sh" in f.get("proof_commands", []),
    "proof-doc": lambda f: "docs/verification/feature-setup-install-onboarding-25.md" in f.get("docs", []),
    "external-pending": lambda f: f.get("external_status") in {"none", "planned", "replayable"} and f.get("external_status") != "external_verified",
    "blocks-25": lambda f: f.get("blocks_25_25") is True,
    "forbidden-honesty": lambda f: any("Do not claim" in item and "external" in item.lower() for item in f.get("forbidden_claims", [])),
}
if expr not in checks:
    print(f"unknown registry check: {expr}", file=sys.stderr)
    sys.exit(1)
if not checks[expr](feature):
    print(f"registry check failed: {expr}", file=sys.stderr)
    sys.exit(1)
PY
}

cd "$ROOT"

for f in \
  README.md \
  START-HERE.md \
  docs/setup/README.md \
  docs/setup/install.md \
  docs/setup/update.md \
  docs/setup/uninstall.md \
  docs/setup/first-run.md \
  docs/setup/troubleshooting.md \
  docs/setup/data-and-privacy.md \
  docs/verification/setup-experience-scorecard.md \
  docs/verification/feature-setup-install-onboarding-25.md \
  docs/verification/features.registry.json \
  scripts/install-memd.sh \
  scripts/update-memd.sh \
  scripts/uninstall-memd.sh \
  scripts/verify/setup-experience-smoke.sh; do
  require_file "$f"
done
for f in \
  scripts/install-memd.sh \
  scripts/update-memd.sh \
  scripts/uninstall-memd.sh \
  scripts/verify/setup-experience-smoke.sh; do
  require_executable "$f"
done

require_contains README.md "## Quickstart"
require_contains README.md "scripts/install-memd.sh"
require_contains README.md "memd setup --interactive"
require_contains README.md "memd setup --guided --summary"
require_contains README.md "memd doctor --summary"
require_contains README.md "memd status --output .memd --summary"
require_contains README.md "memd resume --output .memd --intent current_task"
require_contains README.md "memd setup-demo --summary"
require_contains START-HERE.md "README Quickstart"
require_contains START-HERE.md "memd setup --guided --summary"
require_contains docs/setup/README.md "Best path"
require_contains docs/setup/README.md "scripts/install-memd.sh"
require_contains docs/setup/README.md "memd setup --guided --summary"
require_contains docs/setup/README.md "memd setup --interactive"
require_contains docs/setup/first-run.md "memd resume --output .memd --intent current_task"

require_contains docs/setup/install.md "scripts/install-memd.sh"
require_contains docs/setup/install.md "memd doctor --summary"
require_contains docs/setup/install.md "memd status --output .memd --summary"
require_contains docs/setup/update.md "scripts/update-memd.sh --dry-run"
require_contains docs/setup/update.md "scripts/update-memd.sh"
require_contains docs/setup/update.md 'preserves `.memd` by default'
require_contains docs/setup/uninstall.md "scripts/uninstall-memd.sh --dry-run"
require_contains docs/setup/uninstall.md "scripts/uninstall-memd.sh"
require_contains docs/setup/uninstall.md "preserves it by default"

require_contains docs/verification/feature-setup-install-onboarding-25.md "25-star implementation complete, external validation pending"
require_contains docs/verification/feature-setup-install-onboarding-25.md "Not claimed"
require_contains docs/verification/feature-setup-install-onboarding-25.md "External human validation remains pending"
require_contains docs/verification/feature-setup-install-onboarding-25.md "bash scripts/verify/feature-setup-install-onboarding-proof.sh"
require_contains docs/verification/feature-setup-install-onboarding-25.md "bash scripts/verify/setup-experience-smoke.sh"

require_registry_value proof-command
require_registry_value proof-doc
require_registry_value external-pending
require_registry_value blocks-25
require_registry_value forbidden-honesty

bash scripts/update-memd.sh --dry-run >"$TMP/update-dry-run.out"
require_contains "$TMP/update-dry-run.out" "update: dry-run ok; no files changed"
bash scripts/uninstall-memd.sh --dry-run >"$TMP/uninstall-dry-run.out"
require_contains "$TMP/uninstall-dry-run.out" "uninstall: dry-run ok; would remove binary only"

if [ "${MEMD_SETUP_ONBOARDING_SKIP_SMOKE:-0}" != "1" ]; then
  bash scripts/verify/setup-experience-smoke.sh >"$TMP/setup-experience-smoke.out"
  require_contains "$TMP/setup-experience-smoke.out" "setup-experience-smoke=pass"
fi

{
  echo "# Feature setup/install/onboarding proof"
  echo
  echo "- date_utc: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "- root: $ROOT"
  echo "- smoke: $([ "${MEMD_SETUP_ONBOARDING_SKIP_SMOKE:-0}" = "1" ] && echo skipped || echo pass)"
  echo "- lifecycle_dry_runs: pass"
  echo "- docs: pass"
  echo "- registry_honesty: pass"
  echo "- external_validation: pending_not_claimed"
  echo
  echo "feature-setup-install-onboarding-proof=pass"
} | tee "$REPORT"

echo "setup onboarding proof report: $REPORT"

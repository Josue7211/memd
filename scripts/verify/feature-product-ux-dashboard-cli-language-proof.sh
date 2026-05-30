#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DOC="$ROOT/docs/verification/feature-product-ux-dashboard-cli-language-25.md"
REGISTRY="$ROOT/docs/verification/features.registry.json"
TMPDIR="${TMPDIR:-/tmp}/memd-product-ux-proof.$$"
mkdir -p "$TMPDIR"
trap 'rm -rf "$TMPDIR"' EXIT

fail() {
  echo "feature-product-ux-dashboard-cli-language-proof: ERROR: $*" >&2
  exit 1
}

require_file() {
  local path=$1
  [[ -f "$ROOT/$path" ]] || fail "missing file: $path"
}

require_text() {
  local path=$1
  local text=$2
  grep -Fq -- "$text" "$ROOT/$path" || fail "$path missing text: $text"
}

require_doc_text() {
  local text=$1
  grep -Fq -- "$text" "$DOC" || fail "proof doc missing text: $text"
}

require_file "docs/verification/feature-product-ux-dashboard-cli-language-25.md"
require_file "docs/verification/features.registry.json"
require_file "scripts/verify/feature-registry-audit.sh"
require_file "scripts/memd-cargo-guard.sh"
require_file "scripts/doc-lint.sh"
require_file "README.md"
require_file "START-HERE.md"
require_file "crates/memd-client/src/cli/args.rs"
require_file "crates/memd-client/src/cli/args_memory.rs"

for text in \
  "not a browser walkthrough" \
  "No real browser session" \
  "Forbidden claim" \
  'external_status`: none' \
  "setup/getting-started" \
  "strong local proof" \
  "dashboard static build"; do
  require_doc_text "$text"
done

require_text "README.md" "## Quickstart"
require_text "README.md" "memd setup"
require_text "START-HERE.md" "README Quickstart"
require_text "START-HERE.md" "memd setup --guided --summary"
require_text "START-HERE.md" "memd setup --interactive"
require_text "START-HERE.md" "Setup Troubleshooting"
require_text "crates/memd-client/src/cli/args.rs" "Configure memd for a local project, provider, and harness."
require_text "crates/memd-client/src/cli/args.rs" "Run an isolated setup proof without changing the current repository."
require_text "crates/memd-client/src/cli/args.rs" "Check local memd health and print actionable repair guidance."
require_text "crates/memd-client/src/cli/args_memory.rs" "Print the beginner guided setup path and exact proof commands"
require_text "crates/memd-client/src/cli/args_memory.rs" "Open a centered arrow-key provider/harness picker"

bash "$ROOT/scripts/memd-cargo-guard.sh" run -q -p memd-client --bin memd -- --help >"$TMPDIR/memd-help.txt"
bash "$ROOT/scripts/memd-cargo-guard.sh" run -q -p memd-client --bin memd -- setup --help >"$TMPDIR/memd-setup-help.txt"
bash "$ROOT/scripts/memd-cargo-guard.sh" run -q -p memd-client --bin memd -- setup-demo --help >"$TMPDIR/memd-setup-demo-help.txt"
bash "$ROOT/scripts/memd-cargo-guard.sh" run -q -p memd-client --bin memd -- doctor --help >"$TMPDIR/memd-doctor-help.txt"

grep -Fq "Compact CLI for memd" "$TMPDIR/memd-help.txt" || fail "top-level CLI help missing memd description"
for command in setup setup-demo doctor help; do
  grep -Eq "(^|[[:space:]])${command}([[:space:]]|$)" "$TMPDIR/memd-help.txt" || fail "top-level CLI help missing command: $command"
done
grep -Fq "Configure memd for a local project, provider, and harness" "$TMPDIR/memd-help.txt" || fail "top-level CLI help missing setup description"
grep -Fq "Run an isolated setup proof without changing the current repository" "$TMPDIR/memd-help.txt" || fail "top-level CLI help missing setup-demo description"
grep -Fq "Check local memd health and print actionable repair guidance" "$TMPDIR/memd-help.txt" || fail "top-level CLI help missing doctor description"
for flag in --guided --interactive --summary --json; do
  grep -Fq -- "$flag" "$TMPDIR/memd-setup-help.txt" || fail "setup help missing flag: $flag"
done
grep -Fq "beginner guided setup path" "$TMPDIR/memd-setup-help.txt" || fail "setup help missing guided explanation"
grep -Fq "provider/harness picker" "$TMPDIR/memd-setup-help.txt" || fail "setup help missing interactive explanation"
grep -Fq -- "--summary" "$TMPDIR/memd-setup-demo-help.txt" || fail "setup-demo help missing --summary"
grep -Fq -- "--repair" "$TMPDIR/memd-doctor-help.txt" || fail "doctor help missing --repair"

if [[ -d "$ROOT/apps/dashboard" ]]; then
  require_file "apps/dashboard/package.json"
  require_file "apps/dashboard/package-lock.json"
  require_file "apps/dashboard/DESIGN.md"
  require_file "apps/dashboard/app/main.tsx"
  require_file "apps/dashboard/app/router.tsx"
  require_file "apps/dashboard/app/routes/__root.tsx"
  require_file "apps/dashboard/app/routes/index.tsx"
  require_file "apps/dashboard/app/routes/ask.tsx"
  require_file "apps/dashboard/app/routes/memory.tsx"
  require_file "apps/dashboard/app/routes/atlas.tsx"
  require_file "apps/dashboard/app/components/ui/empty-state.tsx"
  require_file "apps/dashboard/app/components/ui/harness-health.tsx"
  require_text "apps/dashboard/package.json" "memd-dashboard"
  require_text "apps/dashboard/DESIGN.md" "memd Dashboard"
  require_text "apps/dashboard/app/router.tsx" "basepath: \"/dashboard\""
  require_text "apps/dashboard/app/routes/__root.tsx" "memd dashboard"
  require_text "apps/dashboard/app/routes/__root.tsx" "Memory"
  require_text "apps/dashboard/app/routes/__root.tsx" "Ask"
  require_text "apps/dashboard/app/routes/index.tsx" "memd control center"
  require_text "apps/dashboard/app/routes/index.tsx" "Runtime status is local evidence only"
  require_text "apps/dashboard/app/routes/index.tsx" "This dashboard does not claim those gates are complete"
  require_text "apps/dashboard/app/routes/index.tsx" "auditor review packet"
  require_text "apps/dashboard/app/routes/index.tsx" "third-party replay packet"
  require_text "apps/dashboard/app/routes/ask.tsx" "Ask memd"
  require_text "apps/dashboard/app/routes/ask.tsx" "Try different words or broaden your search"
  require_text "apps/dashboard/app/routes/memory.tsx" "Memory Browser"
  require_text "apps/dashboard/app/routes/memory.tsx" "Adjust filters or search query"
  require_text "apps/dashboard/app/routes/atlas.tsx" "Atlas"
  require_text "apps/dashboard/app/routes/atlas.tsx" "No atlas regions"
  require_text "apps/dashboard/app/components/ui/harness-health.tsx" "Harness Bootstrap Health"
  if grep -RInE "live bar is real evidence|Runtime cleanup is landed|V19 auditor|V20 third-party replay" "$ROOT/apps/dashboard/app"; then
    fail "dashboard source still contains unsupported completed-evidence copy"
  fi
  if command -v npm >/dev/null 2>&1; then
    (cd "$ROOT/apps/dashboard" && npm ci && npm run build)
  else
    fail "npm is required for dashboard static build proof"
  fi
else
  fail "apps/dashboard is missing; update proof doc honestly before passing this proof"
fi

python3 - "$REGISTRY" <<'PYREG'
import json, sys
from pathlib import Path
registry = json.loads(Path(sys.argv[1]).read_text())
features = [f for f in registry.get("features", []) if f.get("id") == "feature.product_ux_dashboard_cli_language"]
if len(features) != 1:
    raise SystemExit(f"expected exactly one product UX feature entry, got {len(features)}")
f = features[0]
assert f["category"] == "product UX surfaces/dashboard/CLI language"
assert f["current_status"] == "partial"
assert f["proof_status"] == "strong"
assert f["dogfood_status"] == "none"
assert f["external_status"] == "none"
assert f["blocks_25_25"] is True
assert "docs/verification/feature-product-ux-dashboard-cli-language-25.md" in f["docs"]
assert "bash scripts/verify/feature-product-ux-dashboard-cli-language-proof.sh" in f["proof_commands"]
assert "docs/verification/feature-product-ux-dashboard-cli-language-25.md" in f["proof_artifacts"]
allowed = " ".join(f.get("allowed_claims", [])).lower()
for term in ["strong local", "docs", "cli", "dashboard source", "setup", "static build"]:
    if term not in allowed:
        raise SystemExit(f"allowed_claims missing {term!r}")
forbidden = " ".join(f.get("forbidden_claims", [])).lower()
for term in ["do not claim", "polished", "complete dashboard", "browser", "external", "25/25"]:
    if term not in forbidden:
        raise SystemExit(f"forbidden_claims missing {term!r}")
print("feature-product-ux-dashboard-cli-language-proof: registry entry ok")
PYREG

bash "$ROOT/scripts/doc-lint.sh"
bash "$ROOT/scripts/verify/feature-registry-audit.sh"

echo "feature-product-ux-dashboard-cli-language-proof: ok"

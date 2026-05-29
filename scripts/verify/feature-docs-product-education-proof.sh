#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PRODUCT="$ROOT/docs/product/INDEX.md"
PROOF="$ROOT/docs/verification/feature-docs-product-education-25.md"
REGISTRY="$ROOT/docs/verification/features.registry.json"

fail() {
  echo "feature-docs-product-education-proof: ERROR: $*" >&2
  exit 1
}

require_file() {
  [[ -f "$1" ]] || fail "missing file: ${1#$ROOT/}"
}

require_text() {
  local file=$1
  local pattern=$2
  local label=$3
  grep -Eiq "$pattern" "$file" || fail "${file#$ROOT/} missing $label"
}

require_file "$PRODUCT"
require_file "$PROOF"
require_file "$REGISTRY"

require_text "$PRODUCT" "New-User Path" "new-user path heading"
require_text "$PRODUCT" "START-HERE\.md" "START-HERE pointer"
require_text "$PRODUCT" "README\.md" "README pointer"
require_text "$PRODUCT" "docs/WHERE-AM-I\.md" "WHERE-AM-I pointer"
require_text "$PRODUCT" "Plain-Language Product Summary" "plain-language summary"
require_text "$PRODUCT" "Jargon Guardrail" "jargon guardrail"
require_text "$PRODUCT" "Claim-to-Proof Rule" "claim-to-proof rule"
require_text "$PRODUCT" "External validation status.*pending|external validation remains pending" "honest external pending language"
require_text "$PRODUCT" "Dogfood evidence remains pending|dogfood evidence pending" "honest dogfood pending language"

require_text "$PROOF" "Claim-to-Proof Map" "claim-to-proof map"
require_text "$PROOF" "feature\.docs_product_education" "feature id"
require_text "$PROOF" "External validation status: pending" "external pending status"
require_text "$PROOF" "not external validation" "local-only proof caveat"

python3 - "$REGISTRY" <<PY
import json, sys
from pathlib import Path
registry = json.loads(Path(sys.argv[1]).read_text())
features = registry.get("features", [])
rows = [f for f in features if f.get("id") == "feature.docs_product_education"]
if len(rows) != 1:
    raise SystemExit(f"expected exactly one feature.docs_product_education row, found {len(rows)}")
f = rows[0]
required_docs = {
    "docs/product/INDEX.md",
    "docs/verification/feature-docs-product-education-25.md",
}
missing_docs = sorted(required_docs - set(f.get("docs", [])))
if missing_docs:
    raise SystemExit("registry row missing docs: " + ", ".join(missing_docs))
cmd = "bash scripts/verify/feature-docs-product-education-proof.sh"
if cmd not in f.get("proof_commands", []):
    raise SystemExit("registry row missing proof command: " + cmd)
artifact = "docs/verification/feature-docs-product-education-25.md"
if artifact not in f.get("proof_artifacts", []):
    raise SystemExit("registry row missing proof artifact: " + artifact)
if f.get("external_status") == "external_verified" or f.get("proof_status") == "external_verified":
    raise SystemExit("docs/product education must not claim external_verified without external artifact")
forbidden = " ".join(f.get("forbidden_claims", [])).lower()
if "external" not in forbidden or "do not claim" not in forbidden:
    raise SystemExit("forbidden_claims must explicitly block external/unsupported claims")
PY

python3 - "$PRODUCT" <<PY
import re, sys
from pathlib import Path
text = Path(sys.argv[1]).read_text().lower()
errors = []
# Product education should not use unexplained insider labels as the first path.
banned = [
    "pillar 01",
    "substrate",
    "hivemind",
    "25/25 achieved",
    "externally verified",
    "production ready",
]
for term in banned:
    if term in text:
        errors.append(f"banned or over-strong product education wording: {term}")
if len(re.findall(r"pending", text)) < 2:
    errors.append("expected repeated pending language for external/dogfood limits")
if errors:
    raise SystemExit("; ".join(errors))
PY

echo "feature-docs-product-education-proof: ok"

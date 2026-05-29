#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DOC="$ROOT/docs/verification/feature-network-identity-federation-market-25.md"
REGISTRY="$ROOT/docs/verification/features.registry.json"

fail() {
  echo "feature-network-identity-federation-market-proof: ERROR: $*" >&2
  exit 1
}

require_file() {
  local file="$1"
  [ -f "$ROOT/$file" ] || fail "missing required file: $file"
}

cd "$ROOT"

for file in \
  docs/verification/feature-network-identity-federation-market-25.md \
  docs/verification/features.registry.json \
  docs/verification/feature-coverage-report.md \
  docs/verification/FEATURES.md \
  integrations/codex/README.md \
  integrations/hermes/README.md \
  integrations/hooks/README.md \
  docs/verification/v17-proof-runs/2026-05-12-routine-marketplace-suite.md \
  docs/verification/v17-proof-runs/2026-05-12-routine-marketplace-suite.ndjson \
  docs/verification/25-star-CONTRACT.md \
  docs/verification/25-star-phase-ledger.md; do
  require_file "$file"
done

v26_status="absent_pending_not_claimed"
if [ -x "$ROOT/scripts/verify/v26-network-identity-proof.sh" ]; then
  bash "$ROOT/scripts/verify/v26-network-identity-proof.sh"
  v26_status="script_ran"
elif [ -f "$ROOT/scripts/verify/v26-network-identity-proof.sh" ]; then
  bash "$ROOT/scripts/verify/v26-network-identity-proof.sh"
  v26_status="script_ran"
fi

python3 - "$ROOT" "$DOC" "$REGISTRY" "$v26_status" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
doc = Path(sys.argv[2])
registry_path = Path(sys.argv[3])
v26_status = sys.argv[4]
errors = []

def rel(path: Path) -> str:
    try:
        return str(path.relative_to(root))
    except Exception:
        return str(path)

def require_text(path: str, needles):
    p = root / path
    if not p.is_file():
        errors.append(f"missing required file: {path}")
        return ""
    text = p.read_text(errors="replace")
    for needle in needles:
        if needle not in text:
            errors.append(f"{path} missing required text: {needle}")
    return text

doc_text = require_text("docs/verification/feature-network-identity-federation-market-25.md", [
    "partial local proof only",
    "Single user/org across app surfaces",
    "one user/org memory identity across app surfaces",
    "absent_pending_not_claimed",
    "Federation and market boundaries",
    "Forbidden claim: do not claim active network identity service",
    "external_status: none",
])

# User-corrected scope: app surfaces must point to one shared local control plane/bundle,
# not to distinct per-app user identities.
require_text("integrations/codex/README.md", [
    "memd owns the same memory control plane",
    "Codex should use the same `memd` surface as every other agent",
    "`.memd/wake.md`",
    "`.memd/mem.md`",
    "`.memd/events.md`",
    "same shared local files",
])
require_text("integrations/hermes/README.md", [
    "Hermes should use `memd` as the shared memory control plane",
    "same core memory loop",
    "`.memd/wake.md`",
    "`.memd/mem.md`",
    "`.memd/events.md`",
    "same visible truth as the other agent packs",
])
require_text("integrations/hooks/README.md", [
    "memd setup --output .memd --project <project> --namespace <namespace> --agent <agent>",
    "codex.sh",
    "claude-code.sh",
    "openclaw.sh",
    "hermes.sh",
    "same bundle",
])
require_text("README.md", [
    "portability across Codex, Claude Code, OpenClaw, Hermes, OpenCode, and future harnesses",
])

# Boundary evidence: V17 exists but honestly remains local/synthetic and dogfood-pending.
require_text("docs/verification/v17-proof-runs/2026-05-12-routine-marketplace-suite.md", [
    "1000-user synthetic federation preserved isolation",
    "Remaining gate: real 30-day marketplace dogfood with cross-user installs",
])
ndjson_path = root / "docs/verification/v17-proof-runs/2026-05-12-routine-marketplace-suite.ndjson"
try:
    rows = [json.loads(line) for line in ndjson_path.read_text().splitlines() if line.strip()]
except Exception as exc:
    errors.append(f"invalid V17 ndjson: {exc}")
    rows = []
if rows:
    if not any(r.get("check") == "federation_scale" and r.get("synthetic_users") == 1000 and r.get("isolation_violations") == 0 for r in rows):
        errors.append("V17 ndjson missing synthetic federation isolation evidence")
    if not any(r.get("check") == "axis_lift" and r.get("gate") == "code_complete_dogfood_pending" for r in rows):
        errors.append("V17 ndjson missing dogfood-pending gate")

require_text("docs/verification/25-star-CONTRACT.md", [
    "V26 Network Identity",
    "V27 Federation Protocol",
    "V28 Agent Work Market",
])
require_text("docs/verification/25-star-phase-ledger.md", [
    "V26",
    "network identity proof",
    "V27",
    "federation proof",
    "V28",
    "work market proof",
])

v26_artifacts = sorted((root / "docs/verification").glob("v26-proof-runs/*network-identity*"))
if v26_status == "script_ran" and not doc_text:
    errors.append("internal doc read failure after V26 script run")
if not v26_artifacts and v26_status != "absent_pending_not_claimed":
    errors.append("V26 status inconsistent with missing artifacts")

try:
    registry = json.loads(registry_path.read_text())
except Exception as exc:
    errors.append(f"invalid registry json: {exc}")
    registry = {}
features = [f for f in registry.get("features", []) if f.get("id") == "feature.network_identity_federation_market"]
if len(features) != 1:
    errors.append(f"expected exactly one network identity/federation/market registry row, got {len(features)}")
else:
    f = features[0]
    expected = {
        "current_status": "partial",
        "proof_status": "partial",
        "dogfood_status": "none",
        "external_status": "none",
        "blocks_25_25": True,
    }
    for key, value in expected.items():
        if f.get(key) != value:
            errors.append(f"registry {key} expected {value!r}, got {f.get(key)!r}")
    for item in [
        "docs/verification/feature-network-identity-federation-market-25.md",
        "integrations/codex/README.md",
        "integrations/hermes/README.md",
        "integrations/hooks/README.md",
    ]:
        if item not in f.get("docs", []):
            errors.append(f"registry docs missing {item}")
    if "bash scripts/verify/feature-network-identity-federation-market-proof.sh" not in f.get("proof_commands", []):
        errors.append("registry missing feature proof command")
    for item in [
        "docs/verification/feature-network-identity-federation-market-25.md",
        "docs/verification/v17-proof-runs/2026-05-12-routine-marketplace-suite.md",
        "docs/verification/v17-proof-runs/2026-05-12-routine-marketplace-suite.ndjson",
    ]:
        if item not in f.get("proof_artifacts", []):
            errors.append(f"registry proof_artifacts missing {item}")
    joined_allowed = " ".join(f.get("allowed_claims", []))
    joined_forbidden = " ".join(f.get("forbidden_claims", []))
    if "single user/org local memory identity" not in joined_allowed:
        errors.append("registry allowed_claims missing single user/org scope")
    for phrase in ["active network identity service", "cross-org federation", "public marketplace", "external verification"]:
        if phrase not in joined_forbidden:
            errors.append(f"registry forbidden_claims missing {phrase!r}")

coverage = require_text("docs/verification/feature-coverage-report.md", [
    "| `feature.network_identity_federation_market` | `partial` | `partial` | `none` | `none` |",
    "single user/org local memory identity across Codex/Hermes/OpenClaw-style surfaces",
])
features_md = require_text("docs/verification/FEATURES.md", [
    "| `feature.network_identity_federation_market` | network identity/federation/market layer | `partial` | `partial` | `none` | `none` | yes |",
])

if errors:
    for error in errors:
        print(f"feature-network-identity-federation-market-proof: ERROR: {error}", file=sys.stderr)
    sys.exit(1)

v26_artifact_text = ", ".join(rel(p) for p in v26_artifacts) if v26_artifacts else "none"
print("feature-network-identity-federation-market-proof: ok")
print("single_user_org_surfaces=pass surfaces=codex,hermes,claude-code/openclaw-hooks")
print(f"v26_network_identity={v26_status} artifacts={v26_artifact_text}")
print("federation_market_boundary=pass v17_local_synthetic_cited=true v27_v28_planned_not_claimed=true external_status=none")
PY

bash "$ROOT/scripts/verify/feature-registry-audit.sh"

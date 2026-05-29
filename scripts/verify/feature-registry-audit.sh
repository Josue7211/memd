#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REGISTRY="$ROOT/docs/verification/features.registry.json"
SCHEMA="$ROOT/docs/verification/features.schema.json"
REPORT="$ROOT/docs/verification/feature-coverage-report.md"

python3 - "$REGISTRY" "$SCHEMA" "$REPORT" <<'PY'
import json
import re
import sys
from pathlib import Path

registry_path = Path(sys.argv[1])
schema_path = Path(sys.argv[2])
report_path = Path(sys.argv[3])
errors = []

if not registry_path.exists():
    errors.append(f"missing registry: {registry_path}")
if not schema_path.exists():
    errors.append(f"missing schema: {schema_path}")
if not report_path.exists():
    errors.append(f"missing coverage report: {report_path}")
if errors:
    for e in errors:
        print(f"feature-registry-audit: ERROR: {e}", file=sys.stderr)
    sys.exit(1)

try:
    registry = json.loads(registry_path.read_text())
except Exception as exc:
    print(f"feature-registry-audit: ERROR: invalid registry json: {exc}", file=sys.stderr)
    sys.exit(1)
try:
    schema = json.loads(schema_path.read_text())
except Exception as exc:
    print(f"feature-registry-audit: ERROR: invalid schema json: {exc}", file=sys.stderr)
    sys.exit(1)

for field in schema.get("required", []):
    if field not in registry:
        errors.append(f"registry missing top-level required field: {field}")

feature_required = (schema.get("properties", {})
                    .get("features", {})
                    .get("items", {})
                    .get("required", []))
features = registry.get("features")
if not isinstance(features, list) or not features:
    errors.append("registry features must be a non-empty array")
    features = []

current_allowed = {"not_started", "stub", "partial", "implemented", "broken", "unknown"}
proof_allowed = {"none", "planned", "smoke", "partial", "strong", "external_verified", "stale"}
dogfood_allowed = {"none", "planned", "ad_hoc", "windowed", "continuous", "stale"}
external_allowed = {"none", "planned", "replayable", "external_verified", "stale"}
seen = set()

for idx, feature in enumerate(features):
    prefix = f"feature[{idx}]"
    if not isinstance(feature, dict):
        errors.append(f"{prefix} must be an object")
        continue
    fid = feature.get("id", f"<missing:{idx}>")
    prefix = fid
    if fid in seen:
        errors.append(f"duplicate feature id: {fid}")
    seen.add(fid)
    if not isinstance(fid, str) or not fid.startswith("feature."):
        errors.append(f"{prefix}: id must start with feature.")
    for field in feature_required:
        if field not in feature:
            errors.append(f"{prefix}: missing required field {field}")
    for field in ["implementation_surfaces", "docs", "proof_commands", "proof_artifacts", "scorecard_axes", "allowed_claims", "forbidden_claims"]:
        value = feature.get(field)
        if not isinstance(value, list):
            errors.append(f"{prefix}: {field} must be an array")
        elif field in {"allowed_claims", "forbidden_claims", "scorecard_axes"} and not value:
            errors.append(f"{prefix}: {field} must not be empty")
    for field in ["name", "category", "user_contract", "current_status", "proof_status", "dogfood_status", "external_status", "freshness_policy"]:
        value = feature.get(field)
        if not isinstance(value, str) or not value.strip():
            errors.append(f"{prefix}: {field} must be a non-empty string")
    if not isinstance(feature.get("blocks_25_25"), bool):
        errors.append(f"{prefix}: blocks_25_25 must be boolean")
    if feature.get("current_status") not in current_allowed:
        errors.append(f"{prefix}: invalid current_status {feature.get('current_status')!r}")
    if feature.get("proof_status") not in proof_allowed:
        errors.append(f"{prefix}: invalid proof_status {feature.get('proof_status')!r}")
    if feature.get("dogfood_status") not in dogfood_allowed:
        errors.append(f"{prefix}: invalid dogfood_status {feature.get('dogfood_status')!r}")
    if feature.get("external_status") not in external_allowed:
        errors.append(f"{prefix}: invalid external_status {feature.get('external_status')!r}")

    artifacts = feature.get("proof_artifacts") or []
    commands = feature.get("proof_commands") or []
    proof = feature.get("proof_status")
    external = feature.get("external_status")
    current = feature.get("current_status")
    dogfood = feature.get("dogfood_status")
    forbidden = " ".join(feature.get("forbidden_claims") or []).lower()

    if proof in {"strong", "external_verified"} and (not commands or not artifacts):
        errors.append(f"{prefix}: strong/external proof requires proof_commands and proof_artifacts")
    if external == "external_verified" and proof != "external_verified":
        errors.append(f"{prefix}: external_verified external_status requires proof_status external_verified")
    if current == "implemented" and proof in {"none", "planned"}:
        errors.append(f"{prefix}: implemented current_status needs proof beyond none/planned")
    if dogfood in {"windowed", "continuous"} and not artifacts:
        errors.append(f"{prefix}: windowed/continuous dogfood requires proof_artifacts")
    if "do not claim" not in forbidden and "do not" not in forbidden:
        errors.append(f"{prefix}: forbidden_claims should contain explicit do-not-claim language")

required_categories = {
    "setup/install/onboarding",
    "docs/product education",
    "doctor/status/recovery/update/uninstall",
    "memory capture/lookup/recall/corrections/provenance/trust",
    "context compiler/token savings",
    "shared research cache/donor repo extraction",
    "hive/hivemind coordination",
    "competitor/public benchmark replay",
    "dogfood/reliability windows",
    "external replay/auditor proof",
    "product UX surfaces/dashboard/CLI language",
    "network identity/federation/market layer",
    "release/claim honesty gates",
    "cross-harness continuity",
}
present_categories = {f.get("category") for f in features if isinstance(f, dict)}
for category in sorted(required_categories - present_categories):
    errors.append(f"missing required first-class category: {category}")

summary_counts = {
    "Registered first-class feature areas": len(features),
    "Areas blocking any honest 25/25 claim": sum(1 for f in features if f.get("blocks_25_25") is True),
    "Areas with no executable proof commands listed": sum(1 for f in features if not (f.get("proof_commands") or [])),
    "Areas with no proof artifacts listed": sum(1 for f in features if not (f.get("proof_artifacts") or [])),
    "Areas externally verified": sum(1 for f in features if f.get("external_status") == "external_verified"),
    "Areas with sustained/continuous dogfood": sum(1 for f in features if f.get("dogfood_status") == "continuous"),
}
try:
    report_text = report_path.read_text()
except Exception as exc:
    errors.append(f"invalid coverage report text: {exc}")
else:
    for label, expected in summary_counts.items():
        match = re.search(rf"^- {re.escape(label)}: (\d+)\s*$", report_text, re.MULTILINE)
        if not match:
            errors.append(f"coverage report missing summary count: {label}")
            continue
        actual = int(match.group(1))
        if actual != expected:
            errors.append(f"coverage report summary drift for {label}: report has {actual}, registry has {expected}")

if errors:
    for error in errors:
        print(f"feature-registry-audit: ERROR: {error}", file=sys.stderr)
    sys.exit(1)

blocking = sum(1 for f in features if f.get("blocks_25_25") is True)
print(f"feature-registry-audit: ok ({len(features)} features, {blocking} block 25/25)")
PY

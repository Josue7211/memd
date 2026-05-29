#!/usr/bin/env bash
# Current proof gate for feature.competitor_public_benchmark_replay.
#
# This intentionally proves only local/public fixture replay unless a same-day
# competitor replay artifact is present. Live external/third-party replay stays
# planned when no current external artifact exists.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${OUT_DIR:-$ROOT/docs/verification/25-5-memory-os-runs}"
RUN_DATE="${RUN_DATE:-$(date +%F)}"
PUBLIC_REPORT="${PUBLIC_REPORT:-$OUT_DIR/${RUN_DATE}-public-benchmark-fixtures.json}"
COMPETITOR_REPORT="${COMPETITOR_REPORT:-$OUT_DIR/${RUN_DATE}-competitor-head-to-head.json}"
FEATURE_DOC="$ROOT/docs/verification/feature-competitor-public-benchmark-replay-25.md"
REGISTRY="$ROOT/docs/verification/features.registry.json"
MAX_AGE_DAYS="${MAX_AGE_DAYS:-1}"
SKIP_REPLAY="${SKIP_REPLAY:-0}"

mkdir -p "$OUT_DIR"

if [[ "$SKIP_REPLAY" != "1" ]]; then
  if [[ ! -x "$ROOT/target/debug/memd-server" || ! -x "$ROOT/target/debug/memd" ]]; then
    (cd "$ROOT" && MEMD_CARGO_TARGET_DIR="${MEMD_CARGO_TARGET_DIR:-$ROOT/target}" \
      bash scripts/memd-cargo-guard.sh build -q -p memd-server --bin memd-server -p memd-client --bin memd)
  fi
  MEMD_CARGO_TARGET_DIR="${MEMD_CARGO_TARGET_DIR:-$ROOT/target}" \
    bash "$ROOT/scripts/verify/25-5-public-benchmark-fixtures.sh"
fi

python3 - "$ROOT" "$RUN_DATE" "$PUBLIC_REPORT" "$COMPETITOR_REPORT" "$FEATURE_DOC" "$REGISTRY" "$MAX_AGE_DAYS" <<'PY'
import datetime as dt
import json
import re
import sys
from pathlib import Path

root = Path(sys.argv[1])
run_date = sys.argv[2]
public_report = Path(sys.argv[3])
competitor_report = Path(sys.argv[4])
feature_doc = Path(sys.argv[5])
registry_path = Path(sys.argv[6])
max_age_days = int(sys.argv[7])
errors = []
warnings = []

def rel(path: Path) -> str:
    try:
        return str(path.resolve().relative_to(root.resolve()))
    except Exception:
        return str(path)

def load_json(path: Path, label: str):
    if not path.exists():
        errors.append(f"missing {label}: {rel(path)}")
        return None
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        errors.append(f"invalid {label} json: {rel(path)}: {exc}")
        return None

def require_current_dated_artifact(path: Path, label: str):
    name = path.name
    match = re.match(r"(\d{4}-\d{2}-\d{2})-", name)
    if not match:
        errors.append(f"{label} must use YYYY-MM-DD filename prefix: {rel(path)}")
        return
    artifact_date = dt.date.fromisoformat(match.group(1))
    expected_date = dt.date.fromisoformat(run_date)
    age = (expected_date - artifact_date).days
    if age < 0 or age > max_age_days:
        errors.append(
            f"{label} is not fresh: artifact_date={artifact_date} run_date={expected_date} max_age_days={max_age_days}"
        )
    mtime_date = dt.datetime.fromtimestamp(path.stat().st_mtime).date()
    if abs((expected_date - mtime_date).days) > max_age_days:
        errors.append(
            f"{label} mtime is not fresh: mtime_date={mtime_date} run_date={expected_date} max_age_days={max_age_days}"
        )

public = load_json(public_report, "public fixture replay report")
if public is not None:
    require_current_dated_artifact(public_report, "public fixture replay report")
    if public.get("suite") != "25_5_public_benchmark_fixtures":
        errors.append("public report suite mismatch")
    if public.get("status") != "pass":
        errors.append(f"public report is not passing: {public.get('status')!r}")
    expected_datasets = {"longmemeval", "locomo", "membench", "convomem"}
    seen = {(row.get("dataset"), row.get("backend")) for row in public.get("rows", [])}
    for dataset in expected_datasets:
        for backend in ("lexical", "memd"):
            if (dataset, backend) not in seen:
                errors.append(f"missing public fixture row: dataset={dataset} backend={backend}")
    for row in public.get("rows", []):
        fixture = row.get("fixture")
        if not fixture or not fixture.startswith("fixtures/") or not fixture.endswith("-mini.json"):
            errors.append(f"row lacks public mini fixture reference: {row}")
            continue
        if not (root / fixture).exists():
            errors.append(f"referenced public fixture does not exist: {fixture}")
        if row.get("failures") not in (0, [], None):
            errors.append(f"fixture replay row has failures: {row}")

competitor_status = "external_planned"
if competitor_report.exists():
    competitor = load_json(competitor_report, "competitor replay report")
    if competitor is not None:
        require_current_dated_artifact(competitor_report, "competitor replay report")
        if competitor.get("status") == "pass":
            competitor_status = "current_local_same_fixture_replay"
            if not competitor.get("rows"):
                errors.append("passing competitor report has no comparison rows")
            for row in competitor.get("rows", []):
                if row.get("competitor_status") != "replayed":
                    errors.append(f"competitor row is not replayed: {row}")
                if row.get("competitor_limit_scope") != "items":
                    errors.append(f"competitor comparison is not item-scoped: {row}")
                if row.get("competitor_metric") != "accuracy":
                    errors.append(f"competitor metric boundary is not explicit accuracy: {row}")
                if row.get("competitor_source") is None or row.get("competitor_command") is None:
                    errors.append(f"competitor row lacks replay source/command: {row}")
        elif competitor.get("status") in {"blocked", "planned"}:
            competitor_status = "external_planned"
            warnings.append(f"competitor replay is {competitor.get('status')}; treating live external replay as planned")
        else:
            errors.append(f"competitor report has unsupported status: {competitor.get('status')!r}")
else:
    warnings.append(f"no same-day competitor report found at {rel(competitor_report)}; live external replay remains planned")

if not feature_doc.exists():
    errors.append(f"missing feature proof doc: {rel(feature_doc)}")
else:
    text = feature_doc.read_text(encoding="utf-8")
    required_phrases = [
        "External live replay: planned",
        "not an external verification",
        "must not claim competitor superiority",
        "same-fixture",
        "fixtures/longmemeval-mini.json",
        "fixtures/locomo-mini.json",
        "fixtures/membench-mini.json",
        "fixtures/convomem-mini.json",
    ]
    for phrase in required_phrases:
        if phrase not in text:
            errors.append(f"feature proof doc missing required honesty phrase/reference: {phrase}")
    overclaims = [
        r"(?i)\bbest\b",
        r"(?i)\bmarket[- ]leading\b",
        r"(?i)\bSOTA\b",
        r"(?i)\bsuperior\b",
        r"(?i)\bbeats all\b",
        r"(?i)\boutperforms all\b",
    ]
    for pattern in overclaims:
        if re.search(pattern, text):
            errors.append(f"feature proof doc contains marketing overclaim pattern: {pattern}")

registry = load_json(registry_path, "feature registry")
if registry is not None:
    matches = [f for f in registry.get("features", []) if f.get("id") == "feature.competitor_public_benchmark_replay"]
    if len(matches) != 1:
        errors.append(f"expected exactly one registry entry for feature.competitor_public_benchmark_replay, found {len(matches)}")
    else:
        feature = matches[0]
        commands = feature.get("proof_commands") or []
        artifacts = feature.get("proof_artifacts") or []
        forbidden = " ".join(feature.get("forbidden_claims") or []).lower()
        allowed = " ".join(feature.get("allowed_claims") or []).lower()
        if "bash scripts/verify/feature-competitor-public-benchmark-replay-proof.sh" not in commands:
            errors.append("registry entry does not list this proof command")
        if "docs/verification/feature-competitor-public-benchmark-replay-25.md" not in artifacts:
            errors.append("registry entry does not list feature proof doc artifact")
        if feature.get("external_status") != "planned":
            errors.append("registry must keep external_status planned until live external replay exists")
        if "do not claim" not in forbidden or "superiority" not in forbidden:
            errors.append("registry forbidden_claims must ban superiority claims")
        if "local public fixture replay" not in allowed:
            errors.append("registry allowed_claims must limit claim to local public fixture replay")

if errors:
    for error in errors:
        print(f"feature-competitor-public-benchmark-replay-proof: ERROR: {error}", file=sys.stderr)
    for warning in warnings:
        print(f"feature-competitor-public-benchmark-replay-proof: WARNING: {warning}", file=sys.stderr)
    sys.exit(1)
for warning in warnings:
    print(f"feature-competitor-public-benchmark-replay-proof: WARNING: {warning}")
print(
    "feature-competitor-public-benchmark-replay-proof: ok "
    f"public_report={rel(public_report)} competitor_status={competitor_status}"
)
PY

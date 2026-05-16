#!/usr/bin/env bash
# Honest competitor head-to-head gate.
#
# This does not use published marketing numbers as proof. It requires local
# same-fixture competitor replay artifacts, currently the MemPalace replay JSON
# produced by `scripts/bench-mempalace.py`. Missing artifacts produce a
# `blocked` report and non-zero exit, so the 25/5 claim cannot silently pass.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${OUT_DIR:-$ROOT/docs/verification/25-5-memory-os-runs}"
RUN_DATE="${RUN_DATE:-$(date +%F)}"
REPORT="${REPORT:-$OUT_DIR/${RUN_DATE}-competitor-head-to-head.json}"
TRY_REPLAY_LOG="${TRY_REPLAY_LOG:-$OUT_DIR/${RUN_DATE}-competitor-head-to-head-try-replay.log}"
MEMD_REPORT="${MEMD_REPORT:-}"
COMPETITOR_NAME="${COMPETITOR_NAME:-mempalace}"
COMPETITOR_REPLAYS="${COMPETITOR_REPLAYS:-$ROOT/.memd/benchmarks/baselines/mempalace_replays.json}"
TRY_REPLAY="${TRY_REPLAY:-0}"
EPSILON="${EPSILON:-0.000001}"

mkdir -p "$OUT_DIR"

if [[ -z "$MEMD_REPORT" ]]; then
  MEMD_REPORT="$(python3 - "$OUT_DIR" <<'PY'
import pathlib
import re
import sys

out_dir = pathlib.Path(sys.argv[1])
candidates = []
for path in out_dir.glob("*external-public-scale-*.json"):
    match = re.search(r"external-public-scale-(\d+)\.json$", path.name)
    if match:
        candidates.append((int(match.group(1)), path.stat().st_mtime, path))
if candidates:
    print(max(candidates)[2])
PY
)"
fi

if [[ "$TRY_REPLAY" == "1" && "$COMPETITOR_NAME" == "mempalace" && ! -f "$COMPETITOR_REPLAYS" ]]; then
  set +e
  "$ROOT/scripts/bench-mempalace.py" \
    --benchmark longmemeval \
    --benchmark locomo \
    --benchmark membench \
    --benchmark convomem >"$TRY_REPLAY_LOG" 2>&1
  REPLAY_EXIT="$?"
  set -e
  if [[ "$REPLAY_EXIT" != "0" ]]; then
    printf 'competitor replay attempt failed; continuing to verifier report. log=%s\n' "$TRY_REPLAY_LOG" >&2
  fi
fi

python3 - "$REPORT" "$MEMD_REPORT" "$COMPETITOR_NAME" "$COMPETITOR_REPLAYS" "$EPSILON" <<'PY'
import json
import pathlib
import sys

report_path = pathlib.Path(sys.argv[1])
memd_report_path = pathlib.Path(sys.argv[2]) if sys.argv[2] else None
competitor_name = sys.argv[3]
competitor_replays_path = pathlib.Path(sys.argv[4])
epsilon = float(sys.argv[5])
datasets = ["longmemeval", "locomo", "membench", "convomem"]

def write_and_exit(status, exit_code, **extra):
    payload = {
        "suite": "25_5_competitor_head_to_head",
        "status": status,
        "competitor": competitor_name,
        "memd_report": str(memd_report_path) if memd_report_path else None,
        "competitor_replays": str(competitor_replays_path),
        **extra,
    }
    report_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(
        f"25_5_competitor_head_to_head {status} "
        f"competitor={competitor_name} report={report_path}"
    )
    raise SystemExit(exit_code)

if not memd_report_path or not memd_report_path.exists():
    write_and_exit(
        "blocked",
        2,
        reason="missing memd external public scale report",
        required="run scripts/verify/25-5-external-public-scale.sh first",
    )

memd_report = json.loads(memd_report_path.read_text(encoding="utf-8"))
if memd_report.get("status") != "pass":
    write_and_exit(
        "blocked",
        2,
        reason="memd report is not passing",
        memd_status=memd_report.get("status"),
    )
memd_limit = memd_report.get("limit")

if not competitor_replays_path.exists():
    write_and_exit(
        "blocked",
        2,
        reason="missing local same-fixture competitor replay artifacts",
        required=(
            "install/run the competitor replay, e.g. "
            "TRY_REPLAY=1 scripts/verify/25-5-competitor-head-to-head.sh"
        ),
    )

competitor = json.loads(competitor_replays_path.read_text(encoding="utf-8"))
memd_rows = {
    row.get("dataset"): row
    for row in memd_report.get("rows", [])
    if row.get("dataset") in datasets
}

missing = []
rows = []
failures = []
for dataset in datasets:
    memd_row = memd_rows.get(dataset)
    competitor_row = competitor.get(dataset)
    if memd_row is None:
        missing.append({"dataset": dataset, "missing": "memd_row"})
        continue
    if competitor_row is None:
        missing.append({"dataset": dataset, "missing": "competitor_replay"})
        continue
    competitor_status = competitor_row.get("status") or "unknown"
    competitor_score = competitor_row.get("accuracy")
    competitor_limit = competitor_row.get("limit")
    competitor_limit_scope = competitor_row.get("limit_scope")
    if competitor_status != "replayed":
        missing.append(
            {
                "dataset": dataset,
                "missing": "replayed_status",
                "competitor_status": competitor_status,
            }
        )
        continue
    if competitor_score is None:
        missing.append({"dataset": dataset, "missing": "competitor_accuracy"})
        continue
    if memd_limit is not None:
        if competitor_limit_scope != "items":
            missing.append(
                {
                    "dataset": dataset,
                    "missing": "item_comparable_limit_scope",
                    "memd_limit": memd_limit,
                    "competitor_limit": competitor_limit,
                    "competitor_limit_scope": competitor_limit_scope,
                }
            )
            continue
        if competitor_limit != memd_limit:
            missing.append(
                {
                    "dataset": dataset,
                    "missing": "matching_limit",
                    "memd_limit": memd_limit,
                    "competitor_limit": competitor_limit,
                    "competitor_limit_scope": competitor_limit_scope,
                }
            )
            continue
    competitor_score = float(competitor_score)
    if competitor_score > 1.0:
        competitor_score /= 100.0
    memd_score = memd_row.get("accuracy")
    if memd_score is None:
        memd_score = memd_row.get("recall_at_k")
    if memd_score is None:
        missing.append({"dataset": dataset, "missing": "memd_primary_metric"})
        continue
    memd_score = float(memd_score)
    delta = memd_score - competitor_score
    row = {
        "dataset": dataset,
        "memd_score": memd_score,
        "competitor_score": competitor_score,
        "delta": delta,
        "memd_metric": "accuracy" if memd_row.get("accuracy") is not None else "recall_at_k",
        "competitor_metric": "accuracy",
        "competitor_status": competitor_status,
        "competitor_limit": competitor_limit,
        "competitor_limit_scope": competitor_limit_scope,
        "competitor_source": competitor_row.get("source"),
        "competitor_command": competitor_row.get("command"),
        "competitor_artifact_path": competitor_row.get("artifact_path"),
    }
    rows.append(row)
    if delta + epsilon < 0.0:
        failures.append(row)

if missing:
    write_and_exit(
        "blocked",
        2,
        reason="incomplete local same-fixture competitor coverage",
        missing=missing,
        rows=rows,
    )

if failures:
    write_and_exit(
        "fail",
        1,
        reason="memd below competitor on at least one same-fixture replay",
        failed=failures,
        rows=rows,
    )

write_and_exit(
    "pass",
    0,
    reason="memd meets or exceeds local same-fixture competitor replay on every covered dataset",
    rows=rows,
)
PY

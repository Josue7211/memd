#!/usr/bin/env bash
# Aggregates multiple external public-dataset proof slices into one no-RAG
# stratified proof. This script does not rerun expensive benchmark slices by
# default; it verifies existing dated JSON reports and rejects overlaps/failures.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${OUT_DIR:-$ROOT/docs/verification/25-5-memory-os-runs}"
RUN_DATE="${RUN_DATE:-$(date +%F)}"
RUN_LABEL="${RUN_LABEL:-external-public-stratified}"
SUITE_NAME="${SUITE_NAME:-25_5_external_public_stratified}"
STRATIFIED_SLICES="${STRATIFIED_SLICES:-0:50,50:25}"
STRATIFIED_MIN_ITEMS_PER_DATASET="${STRATIFIED_MIN_ITEMS_PER_DATASET:-75}"
STRATIFIED_REQUIRE_ANNOTATED_TOP1="${STRATIFIED_REQUIRE_ANNOTATED_TOP1:-0}"
REPORT="$OUT_DIR/${RUN_DATE}-${RUN_LABEL}.json"

mkdir -p "$OUT_DIR"

python3 - "$OUT_DIR" "$REPORT" "$SUITE_NAME" "$STRATIFIED_SLICES" "$STRATIFIED_MIN_ITEMS_PER_DATASET" "$STRATIFIED_REQUIRE_ANNOTATED_TOP1" <<'PY'
import json
import pathlib
import sys

out_dir = pathlib.Path(sys.argv[1])
report_path = pathlib.Path(sys.argv[2])
suite_name = sys.argv[3]
slices_arg = sys.argv[4]
min_items_per_dataset = int(sys.argv[5])
require_annotated_top1 = sys.argv[6].lower() in {"1", "true", "yes", "on"}
datasets = ["longmemeval", "locomo", "membench", "convomem"]


def parse_slices(raw):
    slices = []
    for chunk in raw.split(","):
        chunk = chunk.strip()
        if not chunk:
            continue
        try:
            offset_s, limit_s = chunk.split(":", 1)
            offset = int(offset_s)
            limit = int(limit_s)
        except Exception as exc:
            raise RuntimeError(
                f"invalid STRATIFIED_SLICES entry `{chunk}`; expected offset:limit"
            ) from exc
        if offset < 0 or limit <= 0:
            raise RuntimeError(f"invalid STRATIFIED_SLICES entry `{chunk}`")
        slices.append({"offset": offset, "limit": limit})
    if not slices:
        raise RuntimeError("STRATIFIED_SLICES selected no slices")
    return slices


def load_reports():
    reports = []
    for path in sorted(out_dir.glob("*.json")):
        if path.name.endswith("-external-public-stratified.json"):
            continue
        try:
            report = json.loads(path.read_text(encoding="utf-8"))
        except Exception:
            continue
        rows = report.get("rows")
        if not isinstance(rows, list):
            continue
        row_datasets = {row.get("dataset") for row in rows if isinstance(row, dict)}
        if not set(datasets).issubset(row_datasets):
            continue
        reports.append({"path": path, "report": report})
    return reports


def row_offset(row, report):
    offset = row.get("offset", report.get("offset"))
    return 0 if offset is None else int(offset)


def row_limit(row, report):
    limit = row.get("limit", report.get("limit"))
    return None if limit is None else int(limit)


def report_matches_slice(candidate, expected):
    report = candidate["report"]
    if report.get("status") != "pass":
        return False
    rows = {row.get("dataset"): row for row in report.get("rows", [])}
    for dataset in datasets:
        row = rows.get(dataset)
        if not isinstance(row, dict):
            return False
        if row_limit(row, report) != expected["limit"]:
            return False
        if row_offset(row, report) != expected["offset"]:
            return False
    return True


def select_latest_report(reports, expected):
    matches = [candidate for candidate in reports if report_matches_slice(candidate, expected)]
    if not matches:
        return None
    matches.sort(key=lambda candidate: candidate["path"].stat().st_mtime, reverse=True)
    return matches[0]


def ratio(hit, total):
    return None if total == 0 else hit / total


selected = []
all_reports = load_reports()
missing = []
for expected in parse_slices(slices_arg):
    candidate = select_latest_report(all_reports, expected)
    if candidate is None:
        missing.append(expected)
    else:
        selected.append({"slice": expected, **candidate})

failures = []
if missing:
    failures.append({"kind": "missing_slice_reports", "slices": missing})

aggregate = {}
seen_qids = {dataset: set() for dataset in datasets}
overlaps = []

for dataset in datasets:
    aggregate[dataset] = {
        "items": 0,
        "hits": 0,
        "failures": 0,
        "annotated_top1_total": 0,
        "annotated_top1_hits": 0,
        "answer_supported_top1_total": 0,
        "answer_supported_top1_hits": 0,
        "source_reports": [],
    }

for entry in selected:
    report = entry["report"]
    source_path = str(entry["path"])
    rows = {row.get("dataset"): row for row in report.get("rows", [])}
    for dataset in datasets:
        row = rows[dataset]
        summary = aggregate[dataset]
        items = row.get("items")
        if not isinstance(items, list):
            items = []
        row_item_count = len(items) or row_limit(row, report) or 0
        summary["items"] += row_item_count
        summary["failures"] += len(row.get("failures") or [])
        summary["source_reports"].append(
            {
                "path": source_path,
                "offset": row_offset(row, report),
                "limit": row_limit(row, report),
                "accuracy": row.get("accuracy"),
                "recall_at_k": row.get("recall_at_k"),
                "answer_supported_top1_hit_rate": row.get("answer_supported_top1_hit_rate"),
                "annotated_top1_hit_rate": row.get("annotated_top1_hit_rate"),
            }
        )
        if row.get("failures"):
            failures.append({"kind": "row_failures", "dataset": dataset, "path": source_path})
        if row.get("accuracy") is not None and row.get("accuracy") < 1.0:
            failures.append({"kind": "accuracy_below_1", "dataset": dataset, "path": source_path})
        if row.get("recall_at_k") is not None and row.get("recall_at_k") < 1.0:
            failures.append({"kind": "recall_below_1", "dataset": dataset, "path": source_path})
        if (
            row.get("answer_supported_top1_hit_rate") is not None
            and row.get("answer_supported_top1_hit_rate") < 1.0
        ):
            failures.append(
                {"kind": "answer_supported_top1_below_1", "dataset": dataset, "path": source_path}
            )
        if require_annotated_top1 and (
            row.get("annotated_top1_hit_rate") is not None
            and row.get("annotated_top1_hit_rate") < 1.0
        ):
            failures.append(
                {"kind": "annotated_top1_below_1", "dataset": dataset, "path": source_path}
            )
        for item in items:
            qid = str(item.get("question_id"))
            if qid in seen_qids[dataset]:
                overlaps.append({"dataset": dataset, "question_id": qid, "path": source_path})
            seen_qids[dataset].add(qid)
            if item.get("hit") is True:
                summary["hits"] += 1
            correctness = item.get("correctness") or {}
            annotated_hit = item.get("annotated_top1_hit")
            if isinstance(annotated_hit, bool):
                summary["annotated_top1_total"] += 1
                if annotated_hit:
                    summary["annotated_top1_hits"] += 1
            answer_hit = item.get("answer_supported_top1_hit")
            if isinstance(answer_hit, bool):
                summary["answer_supported_top1_total"] += 1
                if answer_hit:
                    summary["answer_supported_top1_hits"] += 1

for dataset, summary in aggregate.items():
    summary["hit_rate"] = ratio(summary["hits"], summary["items"])
    summary["annotated_top1_hit_rate"] = ratio(
        summary["annotated_top1_hits"], summary["annotated_top1_total"]
    )
    summary["answer_supported_top1_hit_rate"] = ratio(
        summary["answer_supported_top1_hits"], summary["answer_supported_top1_total"]
    )
    if summary["items"] < min_items_per_dataset:
        failures.append(
            {
                "kind": "insufficient_items",
                "dataset": dataset,
                "items": summary["items"],
                "required": min_items_per_dataset,
            }
        )
    if summary["hit_rate"] is not None and summary["hit_rate"] < 1.0:
        failures.append({"kind": "aggregate_hit_rate_below_1", "dataset": dataset})
    if (
        summary["answer_supported_top1_hit_rate"] is not None
        and summary["answer_supported_top1_hit_rate"] < 1.0
    ):
        failures.append({"kind": "aggregate_answer_supported_top1_below_1", "dataset": dataset})
    if require_annotated_top1 and (
        summary["annotated_top1_hit_rate"] is not None
        and summary["annotated_top1_hit_rate"] < 1.0
    ):
        failures.append({"kind": "aggregate_annotated_top1_below_1", "dataset": dataset})

if overlaps:
    failures.append({"kind": "overlapping_question_ids", "overlaps": overlaps[:50]})

proof = {
    "suite": suite_name,
    "status": "pass" if not failures else "fail",
    "selected_slices": [
        {"offset": entry["slice"]["offset"], "limit": entry["slice"]["limit"], "path": str(entry["path"])}
        for entry in selected
    ],
    "min_items_per_dataset": min_items_per_dataset,
    "require_annotated_top1": require_annotated_top1,
    "datasets": datasets,
    "aggregate": aggregate,
    "failures": failures,
}
report_path.write_text(json.dumps(proof, indent=2), encoding="utf-8")
if failures:
    raise AssertionError(proof)
print(
    "{} pass slices={} min_items={} report={}".format(
        suite_name, len(selected), min_items_per_dataset, report_path
    )
)
PY

printf '%s wrote %s\n' "$SUITE_NAME" "${REPORT#"$ROOT/"}"

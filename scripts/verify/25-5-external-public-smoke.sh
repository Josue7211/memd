#!/usr/bin/env bash
# External public-dataset proof runner: runs memd against auto-downloaded
# upstream benchmark sources, not the repo mini fixtures. Defaults to a small
# smoke limit so it can run often; wrappers can raise PUBLIC_BENCH_LIMIT for
# larger public-corpus proof runs.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${OUT_DIR:-$ROOT/docs/verification/25-5-memory-os-runs}"
RUN_DATE="${RUN_DATE:-$(date +%F)}"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/memd-external-public.XXXXXX")"
SERVER_DB="$WORK_DIR/memd.db"
RUN_LABEL="${RUN_LABEL:-external-public-smoke}"
SUITE_NAME="${SUITE_NAME:-25_5_external_public_smoke}"
PUBLIC_BENCH_LIMIT="${PUBLIC_BENCH_LIMIT:-2}"
PUBLIC_BENCH_TIMEOUT="${PUBLIC_BENCH_TIMEOUT:-900}"
PUBLIC_BENCH_OFFSET="${PUBLIC_BENCH_OFFSET:-0}"
SERVER_LOG="$OUT_DIR/${RUN_DATE}-${RUN_LABEL}-server.log"
REPORT="$OUT_DIR/${RUN_DATE}-${RUN_LABEL}.json"
DATASET_CACHE_DIR="${DATASET_CACHE_DIR:-$OUT_DIR/external-public-cache}"
SERVER_PID=""

mkdir -p "$OUT_DIR"
mkdir -p "$DATASET_CACHE_DIR"

cleanup() {
  if [[ -n "${SERVER_PID:-}" ]]; then
    kill "$SERVER_PID" >/dev/null 2>&1 || true
    wait "$SERVER_PID" >/dev/null 2>&1 || true
  fi
  rm -rf "$WORK_DIR"
}
trap cleanup EXIT

free_port() {
  python3 - <<'PY'
import socket
s = socket.socket()
s.bind(("127.0.0.1", 0))
print(s.getsockname()[1])
s.close()
PY
}

if [[ ! -x "$ROOT/target/debug/memd-server" || ! -x "$ROOT/target/debug/memd" ]]; then
  (
    cd "$ROOT"
    cargo build -q -p memd-server -p memd-client --bin memd
  )
fi

SERVER_PORT="$(free_port)"
SERVER_URL="http://127.0.0.1:$SERVER_PORT"

(
  cd "$ROOT"
  env -u MEMD_RAG_URL \
    MEMD_DB_PATH="$SERVER_DB" \
    MEMD_BIND_ADDR="127.0.0.1:$SERVER_PORT" \
    MEMD_RATE_LIMIT_DISABLED=1 \
    MEMD_STORE_AUTO_LINK_DISABLED=1 \
    "$ROOT/target/debug/memd-server"
) >"$SERVER_LOG" 2>&1 &
SERVER_PID="$!"

python3 - "$ROOT" "$SERVER_URL" "$REPORT" "$SERVER_PID" "$WORK_DIR" "$DATASET_CACHE_DIR" "$PUBLIC_BENCH_LIMIT" "$SUITE_NAME" "$PUBLIC_BENCH_TIMEOUT" "$PUBLIC_BENCH_OFFSET" <<'PY'
import json
import os
import pathlib
import subprocess
import sys
import time
import urllib.request

root = pathlib.Path(sys.argv[1])
server_url = sys.argv[2]
report_path = pathlib.Path(sys.argv[3])
server_pid = int(sys.argv[4])
work_dir = pathlib.Path(sys.argv[5])
dataset_cache_dir = pathlib.Path(sys.argv[6])
public_bench_limit = int(sys.argv[7])
suite_name = sys.argv[8]
public_bench_timeout = int(sys.argv[9])
public_bench_offset = int(sys.argv[10])

def wait_health(url, pid):
    deadline = time.time() + 180
    while True:
        try:
            with urllib.request.urlopen(url + "/healthz", timeout=2) as response:
                if response.status == 200:
                    return json.load(response)
        except Exception:
            try:
                os.kill(pid, 0)
            except OSError as exc:
                raise RuntimeError("memd-server exited before healthz") from exc
            if time.time() > deadline:
                raise
            time.sleep(0.5)

def parse_json_stdout(raw):
    start = raw.find("{")
    end = raw.rfind("}")
    if start < 0 or end < start:
        raise RuntimeError(f"no JSON object in benchmark stdout:\n{raw[-2000:]}")
    return json.loads(raw[start:end + 1])

def dataset_json_path(dataset):
    filenames = {
        "longmemeval": "longmemeval_s_cleaned.json",
        "locomo": "locomo10.json",
        "membench": "membench-firstagent.json",
        "convomem": "convomem-evidence-sample.json",
    }
    return (
        dataset_cache_dir
        / dataset
        / "benchmarks"
        / "datasets"
        / dataset
        / filenames[dataset]
    )

def qids_for_dataset_slice(dataset, offset, limit):
    if offset <= 0:
        return []
    path = dataset_json_path(dataset)
    if not path.exists():
        raise RuntimeError(
            f"PUBLIC_BENCH_OFFSET={offset} requires cached dataset file: {path}"
        )
    raw = json.loads(path.read_text(encoding="utf-8"))
    qids = []
    if dataset == "longmemeval":
        qids = [str(row["question_id"]) for row in raw]
    elif dataset == "locomo":
        for row in raw:
            sample_id = str(row["sample_id"])
            for index, _ in enumerate(row.get("qa") or []):
                qids.append(f"{sample_id}::{index}")
    elif dataset == "membench":
        for topic in sorted(raw.keys()):
            for entry in raw.get(topic) or []:
                tid = entry.get("tid") or 0
                qa = entry.get("QA") or entry.get("qa") or {}
                qid = qa.get("qid") or 0
                qids.append(f"{topic}::{tid}::{qid}")
    elif dataset == "convomem":
        items = raw.get("items") or []
        qids = [str(item.get("question_id")) for item in items]
    selected = qids[offset : offset + limit]
    if len(selected) != limit:
        raise RuntimeError(
            f"dataset {dataset} has {len(qids)} qids; cannot select offset={offset} limit={limit}"
        )
    return selected

def run_external_public(dataset, limit):
    out = dataset_cache_dir / dataset
    out.mkdir(parents=True, exist_ok=True)
    cmd = [
        str(root / "target/debug/memd"),
        "benchmark",
        "public",
        dataset,
        "--mode",
        "raw",
        "--retrieval-backend",
        "memd",
        "--memd-url",
        server_url,
        "--limit",
        str(limit),
        "--top-k",
        "5",
        "--out",
        str(out),
        "--json",
    ]
    env = os.environ.copy()
    env.pop("MEMD_RAG_URL", None)
    env.setdefault("CARGO_INCREMENTAL", "0")
    qid_filter = qids_for_dataset_slice(dataset, public_bench_offset, limit)
    if qid_filter:
        env["MEMD_BENCH_QID_FILTER"] = ",".join(qid_filter)
    completed = subprocess.run(
        cmd,
        cwd=root,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=public_bench_timeout,
        check=False,
    )
    if completed.returncode != 0:
        raise RuntimeError(
            f"external benchmark {dataset} failed with {completed.returncode}\n"
            f"stderr:\n{completed.stderr[-5000:]}\nstdout:\n{completed.stdout[-5000:]}"
        )
    report = parse_json_stdout(completed.stdout)
    metrics = report.get("metrics") or {}
    manifest = report.get("manifest") or {}
    top_items = []
    annotated_top1_total = 0
    annotated_top1_hits = 0
    answer_supported_top1_total = 0
    answer_supported_top1_hits = 0
    top1_gaps = []
    answer_support_gaps = []
    for item in report.get("items") or []:
        ranked = item.get("ranked_items") or []
        correctness = item.get("correctness") or {}
        expected_targets = correctness.get("expected_targets")
        top_id = ranked[0].get("item_id") if ranked else None
        answer_supported = correctness.get("top1_answer_supported")
        if isinstance(answer_supported, bool):
            answer_supported_top1_total += 1
            if answer_supported:
                answer_supported_top1_hits += 1
            else:
                answer_support_gaps.append(
                    {
                        "question_id": item.get("question_id"),
                        "question": item.get("question"),
                        "top_id": top_id,
                    }
                )
        top1_hit = None
        if isinstance(expected_targets, list) and expected_targets:
            annotated_top1_total += 1
            expected_set = {str(target) for target in expected_targets}
            top1_hit = str(top_id) in expected_set
            if top1_hit:
                annotated_top1_hits += 1
            else:
                top1_gaps.append(
                    {
                        "question_id": item.get("question_id"),
                        "question": item.get("question"),
                        "top_id": top_id,
                        "expected_targets": expected_targets,
                    }
                )
        top_items.append(
            {
                "question_id": item.get("question_id"),
                "question": item.get("question"),
                "hit": item.get("hit"),
                "top_id": top_id,
                "expected_targets": expected_targets,
                "annotated_top1_hit": top1_hit,
                "answer_supported_top1_hit": answer_supported,
            }
        )
    annotated_top1_hit_rate = (
        annotated_top1_hits / annotated_top1_total
        if annotated_top1_total
        else None
    )
    answer_supported_top1_hit_rate = (
        answer_supported_top1_hits / answer_supported_top1_total
        if answer_supported_top1_total
        else None
    )
    return {
        "dataset": dataset,
        "backend": "memd",
        "limit": limit,
        "dataset_source_url": manifest.get("dataset_source_url"),
        "dataset_checksum": manifest.get("dataset_checksum"),
        "dataset_items": (manifest.get("runtime_settings") or {}).get("dataset_items"),
        "offset": public_bench_offset,
        "qid_filter_count": len(qid_filter),
        "accuracy": metrics.get("accuracy"),
        "hit_rate": metrics.get("hit_rate"),
        "recall_at_k": metrics.get("recall_at_k"),
        "answer_supported_at_1": metrics.get("answer_supported_at_1"),
        "annotated_top1_hit_rate": annotated_top1_hit_rate,
        "annotated_top1_total": annotated_top1_total,
        "annotated_top1_gaps": top1_gaps,
        "answer_supported_top1_hit_rate": answer_supported_top1_hit_rate,
        "answer_supported_top1_total": answer_supported_top1_total,
        "answer_supported_top1_gaps": answer_support_gaps,
        "mean_latency_ms": metrics.get("mean_latency_ms"),
        "failures": report.get("failures") or [],
        "items": top_items,
    }

wait_health(server_url, server_pid)

datasets = ["longmemeval", "locomo", "membench", "convomem"]
rows = [run_external_public(dataset, public_bench_limit) for dataset in datasets]
failed = [
    row for row in rows
    if row["dataset_source_url"] is None
    or not str(row["dataset_source_url"]).startswith("http")
    or row["failures"]
    or (row["accuracy"] is not None and row["accuracy"] < 1.0)
    or (row["recall_at_k"] is not None and row["recall_at_k"] < 1.0)
    or (
        row["answer_supported_top1_hit_rate"] is not None
        and row["answer_supported_top1_hit_rate"] < 1.0
    )
]
summary = {
    "suite": suite_name,
    "status": "pass" if not failed else "fail",
    "rag_url": os.environ.get("MEMD_RAG_URL"),
    "server_url": server_url,
    "limit": public_bench_limit,
    "offset": public_bench_offset,
    "timeout_seconds": public_bench_timeout,
    "datasets": datasets,
    "rows": rows,
    "failed": failed,
}
report_path.write_text(json.dumps(summary, indent=2), encoding="utf-8")
if failed:
    raise AssertionError(summary)
print("{} passed datasets={} limit={} report={}".format(suite_name, len(rows), public_bench_limit, report_path))
PY

printf '%s wrote %s\n' "$SUITE_NAME" "${REPORT#"$ROOT/"}"

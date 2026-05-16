#!/usr/bin/env bash
# Runs memd against the repo's public benchmark mini fixtures through the real
# benchmark CLI and a real memd-server. This is fixture replay, not a full
# upstream LongMemEval/LoCoMo/MemBench/ConvoMem submission.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${OUT_DIR:-$ROOT/docs/verification/25-5-memory-os-runs}"
RUN_DATE="${RUN_DATE:-$(date +%F)}"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/memd-public-fixtures.XXXXXX")"
SERVER_DB="$WORK_DIR/memd.db"
SERVER_LOG="$OUT_DIR/${RUN_DATE}-public-benchmark-fixtures-server.log"
REPORT="$OUT_DIR/${RUN_DATE}-public-benchmark-fixtures.json"
SERVER_PID=""

mkdir -p "$OUT_DIR"

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
  MEMD_DB_PATH="$SERVER_DB" \
  MEMD_BIND_ADDR="127.0.0.1:$SERVER_PORT" \
  "$ROOT/target/debug/memd-server"
) >"$SERVER_LOG" 2>&1 &
SERVER_PID="$!"

python3 - "$ROOT" "$SERVER_URL" "$REPORT" "$SERVER_PID" "$WORK_DIR" <<'PY'
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

def run_benchmark(dataset, backend):
    fixture = root / "fixtures" / f"{dataset}-mini.json"
    out = work_dir / "bench-output" / backend / dataset
    out.mkdir(parents=True, exist_ok=True)
    cmd = [
        str(root / "target/debug/memd"),
        "benchmark",
        "public",
        dataset,
        "--mode",
        "raw",
        "--retrieval-backend",
        backend,
        "--dataset-root",
        str(fixture),
        "--limit",
        "2",
        "--top-k",
        "5",
        "--out",
        str(out),
        "--json",
    ]
    if backend == "memd":
        cmd.extend(["--memd-url", server_url])
    env = os.environ.copy()
    env.setdefault("CARGO_INCREMENTAL", "0")
    completed = subprocess.run(
        cmd,
        cwd=root,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=240,
        check=False,
    )
    if completed.returncode != 0:
        raise RuntimeError(
            f"benchmark {dataset}/{backend} failed with {completed.returncode}\n"
            f"stderr:\n{completed.stderr[-4000:]}\nstdout:\n{completed.stdout[-4000:]}"
        )
    report = parse_json_stdout(completed.stdout)
    metrics = report.get("metrics") or {}
    return {
        "dataset": dataset,
        "backend": backend,
        "items": report.get("item_count"),
        "accuracy": metrics.get("accuracy"),
        "hit_rate": metrics.get("hit_rate"),
        "recall_at_k": metrics.get("recall_at_k"),
        "session_recall_any_at_1": metrics.get("session_recall_any@1"),
        "duration_ms": (report.get("manifest") or {}).get("duration_ms"),
        "failures": len(report.get("failures") or []),
        "fixture": str(fixture.relative_to(root)),
    }

wait_health(server_url, server_pid)

datasets = ["longmemeval", "locomo", "membench", "convomem"]
rows = []
for dataset in datasets:
    rows.append(run_benchmark(dataset, "lexical"))
    rows.append(run_benchmark(dataset, "memd"))

memd_rows = [row for row in rows if row["backend"] == "memd"]
failed = [
    row for row in memd_rows
    if row["failures"] != 0
    or (row["accuracy"] is not None and row["accuracy"] < 0.75)
    or (row["recall_at_k"] is not None and row["recall_at_k"] < 0.75)
]
summary = {
    "suite": "25_5_public_benchmark_fixtures",
    "status": "pass" if not failed else "fail",
    "server_url": server_url,
    "datasets": datasets,
    "limit": 2,
    "rows": rows,
    "failed": failed,
}
report_path.write_text(json.dumps(summary, indent=2), encoding="utf-8")
if failed:
    raise AssertionError(summary)
print(
    "public benchmark fixtures passed "
    f"datasets={len(datasets)} memd_rows={len(memd_rows)}"
)
PY

printf 'public benchmark fixtures wrote %s\n' "${REPORT#"$ROOT/"}"

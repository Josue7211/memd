#!/usr/bin/env bash
# Runs memd against the repo's public benchmark mini fixtures through the real
# benchmark CLI and a real memd-server. This is fixture replay, not a full
# upstream LongMemEval/LoCoMo/MemBench/ConvoMem submission.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT/scripts/lib/memd-cargo-env.sh"
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
import hashlib
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

EXPECTED_FIXTURES = {
    "longmemeval": {"path": "fixtures/longmemeval-mini.json", "sha256": "9476cbe708707821fb462ceda53a8c9613e3a111a65df2ba010625b15c009c5e", "bytes": 2051},
    "locomo": {"path": "fixtures/locomo-mini.json", "sha256": "bf3fc32257dd5cd66f355d5eadff352d8059645b2ef2b44dd6b9cc994df741e2", "bytes": 2604},
    "membench": {"path": "fixtures/membench-mini.json", "sha256": "342479e970508ada756c6cc793d27aaeac1d8f96b420a46609d7ae8096c59e8e", "bytes": 2238},
    "convomem": {"path": "fixtures/convomem-mini.json", "sha256": "a3bd49bcd82a1f0382aa5d0c3dc8a6b94e0cde6ae3fb074669dc874e060065eb", "bytes": 1917},
}

EXPECTED_BASELINE_ROWS = {
    ("longmemeval", "lexical"),
    ("locomo", "lexical"),
    ("membench", "lexical"),
    ("convomem", "lexical"),
}


def sha256_file(path):
    h = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def validate_fixture(dataset):
    expected = EXPECTED_FIXTURES[dataset]
    fixture = root / expected["path"]
    if not fixture.exists():
        raise RuntimeError(f"missing fixture: {expected['path']}")
    actual_sha = sha256_file(fixture)
    actual_bytes = fixture.stat().st_size
    if actual_sha != expected["sha256"] or actual_bytes != expected["bytes"]:
        raise RuntimeError(
            f"fixture drift for {expected['path']}: "
            f"sha256={actual_sha} bytes={actual_bytes}"
        )
    return fixture, {
        "fixture": expected["path"],
        "sha256": "sha256:" + actual_sha,
        "bytes": actual_bytes,
    }


def run_benchmark(dataset, backend):
    fixture, fixture_meta = validate_fixture(dataset)
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
        "failures": len(report.get("failures") or []),
        "fixture": str(fixture.relative_to(root)),
        "fixture_sha256": fixture_meta["sha256"],
        "fixture_bytes": fixture_meta["bytes"],
    }

wait_health(server_url, server_pid)

datasets = ["longmemeval", "locomo", "membench", "convomem"]
fixture_checksums = {dataset: validate_fixture(dataset)[1] for dataset in datasets}
rows = []
for dataset in datasets:
    rows.append(run_benchmark(dataset, "lexical"))
    rows.append(run_benchmark(dataset, "memd"))

seen_baselines = {(row["dataset"], row["backend"]) for row in rows if row["backend"] == "lexical"}
missing_baselines = sorted(EXPECTED_BASELINE_ROWS - seen_baselines)
if missing_baselines:
    raise AssertionError(f"missing lexical baseline rows: {missing_baselines}")

for row in rows:
    if row["items"] != 2:
        raise AssertionError(f"unexpected item count in row: {row}")
    for metric_name in ("accuracy", "hit_rate", "recall_at_k", "session_recall_any_at_1"):
        value = row.get(metric_name)
        if value is not None and not (0 <= value <= 1):
            raise AssertionError(f"metric out of range {metric_name}={value}: {row}")

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
    "execution_boundary": "local deterministic public mini-fixture replay; dynamic server port and timing values intentionally omitted",
    "external_live_replay": "planned",
    "datasets": datasets,
    "fixture_checksums": fixture_checksums,
    "baseline_backend": "lexical",
    "comparison_backend": "memd",
    "limit": 2,
    "top_k": 5,
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

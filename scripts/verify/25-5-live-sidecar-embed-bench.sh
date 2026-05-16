#!/usr/bin/env bash
# Proves `memd embed bench --rag-url` against a real sparse rag-sidecar process.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${OUT_DIR:-$ROOT/docs/verification/25-5-memory-os-runs}"
RUN_DATE="${RUN_DATE:-$(date +%F)}"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/memd-live-sidecar-bench.XXXXXX")"
STATE_FILE="$WORK_DIR/rag-sidecar.json"
QRELS="$WORK_DIR/live-qrels.json"
SIDECAR_EMBEDDING_BACKEND="${SIDECAR_EMBEDDING_BACKEND:-sparse}"
REPORT_STEM="${REPORT_STEM:-live-sidecar-embed-bench}"
if [[ "$SIDECAR_EMBEDDING_BACKEND" != "sparse" && "${REPORT_STEM:-}" == "live-sidecar-embed-bench" ]]; then
  REPORT_STEM="live-sidecar-${SIDECAR_EMBEDDING_BACKEND}-embed-bench"
fi
REPORT="$OUT_DIR/${RUN_DATE}-${REPORT_STEM}.json"
LOG="$OUT_DIR/${RUN_DATE}-${REPORT_STEM}.log"
PID=""

mkdir -p "$OUT_DIR"

cleanup() {
  if [[ -n "${PID:-}" ]]; then
    kill "$PID" >/dev/null 2>&1 || true
    wait "$PID" >/dev/null 2>&1 || true
  fi
  rm -rf "$WORK_DIR"
}
trap cleanup EXIT

PORT="$(python3 - <<'PY'
import socket
s = socket.socket()
s.bind(("127.0.0.1", 0))
print(s.getsockname()[1])
s.close()
PY
)"
URL="http://127.0.0.1:$PORT"

(
  cd "$ROOT"
  cargo run -q -p memd-sidecar -- \
    --host 127.0.0.1 \
    --port "$PORT" \
    --state-file "$STATE_FILE" \
    --persist true \
    --embedding-backend "$SIDECAR_EMBEDDING_BACKEND"
) >"$LOG" 2>&1 &
PID="$!"

python3 - "$URL" "$QRELS" "$PID" <<'PY'
import json
import os
import sys
import time
import urllib.error
import urllib.request
import uuid

url = sys.argv[1]
qrels = sys.argv[2]
pid = int(sys.argv[3])
records = [
    (
        str(uuid.uuid4()),
        "Live sidecar proof: Helio trace explains lexical fuzzy dense trust recency and rerank evidence.",
        "explain why search result was chosen",
    ),
    (
        str(uuid.uuid4()),
        "Live sidecar proof: Fjord queue captures failed writes while backend offline then replays.",
        "capture memories when server is down then sync later",
    ),
    (
        str(uuid.uuid4()),
        "Live sidecar proof: Cedar packets give Ollama labeled context instead of raw memory dumps.",
        "give Ollama safe compact context instead of raw dumps",
    ),
    (
        str(uuid.uuid4()),
        "Live sidecar proof: Garnet aliases connect owner names paths commands and identifiers.",
        "find owner names paths commands identifiers",
    ),
    (
        str(uuid.uuid4()),
        "Live sidecar proof: Juno corrections outrank stale facts in final server truth ranking.",
        "corrections outrank stale facts final truth ranking",
    ),
    (
        str(uuid.uuid4()),
        "Live sidecar proof: Kilo visibility filters keep private memory out of other agents.",
        "visibility filters private memory other agents",
    ),
    (
        str(uuid.uuid4()),
        "Live sidecar proof: Lyra model bench records recall MRR latency and selected profile.",
        "model bench recall MRR latency selected profile",
    ),
    (
        str(uuid.uuid4()),
        "Live sidecar proof: Mica authority sync lets Claude Codex Hermes and Ollama share canonical facts.",
        "authority sync Claude Codex Hermes Ollama canonical facts",
    ),
]

deadline = time.time() + 180
while True:
    try:
        with urllib.request.urlopen(url + "/healthz", timeout=2) as response:
            if response.status == 200:
                break
    except Exception:
        try:
            os.kill(pid, 0)
        except OSError as exc:
            raise RuntimeError(f"sidecar process exited before healthz: pid={pid}") from exc
        if time.time() > deadline:
            raise
        time.sleep(0.5)

for record_id, content, _query in records:
    payload = {
        "project": "memd",
        "namespace": "live-sidecar",
        "source": {
            "id": record_id,
            "kind": "fact",
            "content": content,
            "mime": "text/plain",
            "bytes": len(content.encode("utf-8")),
            "source_quality": "canonical",
            "source_agent": "codex",
            "source_path": record_id,
            "tags": ["live-sidecar", "25-5-proof"],
        },
    }
    data = json.dumps(payload).encode()
    request = urllib.request.Request(
        url + "/v1/ingest",
        data=data,
        headers={"content-type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(request, timeout=5) as response:
        if response.status != 200:
            raise RuntimeError(f"ingest failed {response.status}")

cases = []
for record_id, content, query in records:
    candidates = [
        {"id": other_id, "text": other_content}
        for other_id, other_content, _ in records
    ]
    cases.append(
        {
            "query": query,
            "relevant_id": record_id,
            "project": "memd",
            "namespace": "live-sidecar",
            "candidates": candidates,
            "scores": {},
        }
    )

with open(qrels, "w", encoding="utf-8") as handle:
    json.dump({"corpus": "memd-live-sidecar-smoke", "qrels": cases}, handle, indent=2)
PY

(
  cd "$ROOT"
  cargo run -q -p memd-client --bin memd -- \
    embed bench \
    --input "$QRELS" \
    --rag-url "$URL" \
    --project memd \
    --namespace live-sidecar \
    --limit 5 \
    --json
) >"$REPORT"

python3 - "$REPORT" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
assert report["live"] is True, report
results = {row["role"]: row for row in report["results"]}
retrieve = results.get("LiveRetrieve")
rerank = results.get("LiveRerank")
assert retrieve, report
assert rerank, report
expected_cases = report["cases"]
assert expected_cases >= 8, report
assert retrieve["cases"] == expected_cases, retrieve
assert rerank["cases"] == expected_cases, rerank
assert retrieve["recall_at_1"] >= 0.80, retrieve
assert retrieve["mrr"] >= 0.85, retrieve
assert rerank["recall_at_1"] >= 0.80, rerank
assert rerank["mrr"] >= 0.85, rerank
print(
    "live sidecar embed bench passed "
    f"backend={report['results'][0]['model_id']} "
    f"cases={expected_cases} "
    f"retrieve_recall@1={retrieve['recall_at_1']:.3f} "
    f"rerank_recall@1={rerank['recall_at_1']:.3f}"
)
PY

printf 'live sidecar embed bench wrote %s\n' "${REPORT#"$ROOT/"}"

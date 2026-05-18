#!/usr/bin/env bash
# Runs a live no-RAG vs FastEmbed sidecar RAG lift suite over a larger
# public-benchmark-style memory corpus. This is a retrieval lift proof, not a
# canonical upstream benchmark submission.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT/scripts/lib/memd-cargo-env.sh"
OUT_DIR="${OUT_DIR:-$ROOT/docs/verification/25-5-memory-os-runs}"
RUN_DATE="${RUN_DATE:-$(date +%F)}"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/memd-live-rag-lift.XXXXXX")"
SIDECAR_STATE="$WORK_DIR/rag-sidecar.json"
NO_RAG_DB="$WORK_DIR/no-rag.db"
RAG_DB="$WORK_DIR/rag.db"
REPORT="$OUT_DIR/${RUN_DATE}-live-rag-lift-corpus.json"
NO_RAG_LOG="$OUT_DIR/${RUN_DATE}-live-rag-lift-no-rag-server.log"
RAG_LOG="$OUT_DIR/${RUN_DATE}-live-rag-lift-rag-server.log"
SIDECAR_LOG="$OUT_DIR/${RUN_DATE}-live-rag-lift-sidecar.log"
EMBED_MODEL="${MEMD_EMBED_MODEL:-all-minilm-l6-v2}"
NO_RAG_PID=""
RAG_PID=""
SIDECAR_PID=""

mkdir -p "$OUT_DIR"

cleanup() {
  for pid in "${NO_RAG_PID:-}" "${RAG_PID:-}" "${SIDECAR_PID:-}"; do
    if [[ -n "$pid" ]]; then
      kill "$pid" >/dev/null 2>&1 || true
      wait "$pid" >/dev/null 2>&1 || true
    fi
  done
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

NO_RAG_PORT="$(free_port)"
RAG_PORT="$(free_port)"
SIDECAR_PORT="$(free_port)"
NO_RAG_URL="http://127.0.0.1:$NO_RAG_PORT"
RAG_URL="http://127.0.0.1:$RAG_PORT"
SIDECAR_URL="http://127.0.0.1:$SIDECAR_PORT"

if [[ ! -x "$ROOT/target/debug/memd-server" || ! -x "$ROOT/target/debug/memd-sidecar" ]]; then
  (
    cd "$ROOT"
    cargo build -q -p memd-server -p memd-sidecar
  )
fi

(
  cd "$ROOT"
  MEMD_EMBED_MODEL="$EMBED_MODEL" \
  MEMD_SIDECAR_RERANK=heuristic \
  "$ROOT/target/debug/memd-sidecar" \
    --host 127.0.0.1 \
    --port "$SIDECAR_PORT" \
    --state-file "$SIDECAR_STATE" \
    --persist true \
    --embedding-backend fastembed
) >"$SIDECAR_LOG" 2>&1 &
SIDECAR_PID="$!"

(
  cd "$ROOT"
  MEMD_DB_PATH="$NO_RAG_DB" \
  MEMD_BIND_ADDR="127.0.0.1:$NO_RAG_PORT" \
  "$ROOT/target/debug/memd-server"
) >"$NO_RAG_LOG" 2>&1 &
NO_RAG_PID="$!"

(
  cd "$ROOT"
  MEMD_DB_PATH="$RAG_DB" \
  MEMD_BIND_ADDR="127.0.0.1:$RAG_PORT" \
  MEMD_RAG_URL="$SIDECAR_URL" \
  MEMD_RETRIEVAL_RAG_DENSE=1 \
  MEMD_RETRIEVAL_RERANK=0 \
  MEMD_RAG_TIMEOUT_MS=8000 \
  "$ROOT/target/debug/memd-server"
) >"$RAG_LOG" 2>&1 &
RAG_PID="$!"

python3 - "$NO_RAG_URL" "$RAG_URL" "$SIDECAR_URL" "$REPORT" "$NO_RAG_PID" "$RAG_PID" "$SIDECAR_PID" <<'PY'
import json
import os
import sys
import time
import urllib.request

no_rag_url, rag_url, sidecar_url, report_path = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4]
no_rag_pid, rag_pid, sidecar_pid = map(int, sys.argv[5:8])

def wait_health(url, pid, name):
    deadline = time.time() + 360
    while True:
        try:
            with urllib.request.urlopen(url + "/healthz", timeout=2) as response:
                if response.status == 200:
                    return json.load(response)
        except Exception:
            try:
                os.kill(pid, 0)
            except OSError as exc:
                raise RuntimeError(f"{name} exited before healthz") from exc
            if time.time() > deadline:
                raise
            time.sleep(0.5)

def post(url, path, payload):
    data = json.dumps(payload).encode()
    request = urllib.request.Request(
        url + path,
        data=data,
        headers={"content-type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(request, timeout=20) as response:
        if response.status != 200:
            raise RuntimeError(f"{path} failed with {response.status}")
        return json.load(response)

def get_json(url, path):
    with urllib.request.urlopen(url + path, timeout=20) as response:
        if response.status != 200:
            raise RuntimeError(f"{path} failed with {response.status}")
        return json.load(response)

wait_health(sidecar_url, sidecar_pid, "sidecar")
wait_health(no_rag_url, no_rag_pid, "no-rag server")
wait_health(rag_url, rag_pid, "rag server")

project = "memd-live-rag-lift"
namespace = "public-style"
workspace = "shared"
corpus = [
    ("restart", "Aster capsule preserves restart breadcrumbs after interrupted agent work.", "resume a crashed assistant session with prior context"),
    ("harness", "Boreal matrix binds Claude Code Codex OpenCode Hermes and Ollama to one memory authority.", "switch assistants while retaining shared memory"),
    ("gateway", "Cedar packet carries labeled evidence into local models while blocking raw instruction dumps.", "safe compact context for a local language model"),
    ("offline", "Fjord queue stores failed memory writes until the backend returns.", "remember facts when the server is unavailable"),
    ("aliases", "Garnet alias fabric connects owner names file paths shell commands and identifiers.", "find misspelled files commands names and ids"),
    ("trace", "Helio trace lists lexical fuzzy dense trust recency and rerank evidence.", "explain why a retrieval result was selected"),
    ("truth", "Juno correction supersedes stale ownership claims in final truth ranking.", "latest correction beats an old mistaken fact"),
    ("privacy", "Kilo visibility filter blocks private memory from other agents and workspaces.", "stop another assistant from seeing private notes"),
    ("sync", "Mica authority shares canonical facts across devices and harness sessions.", "one self hosted backend keeps agents synchronized"),
    ("procedures", "Nacre procedure memory stores reusable runbooks and invocation evidence.", "reuse a repeated operational workflow"),
    ("atlas", "Orion atlas links entities aliases sessions procedures and corrections.", "connect related people projects sessions and decisions"),
    ("firewall", "Pallas firewall labels retrieved text as data and quarantines instruction attacks.", "prevent memory from changing system policy or tools"),
    ("modelbench", "Quartz bench compares embedding profiles by recall MRR latency and cost.", "choose the best vector model from measured qrels"),
    ("sidecar", "Rhea sidecar mirrors compact canonical records as optional dense recall candidates.", "vector service should boost recall without becoming truth"),
    ("rerank", "Sol reranker reorders candidate memories after dense lexical and fuzzy retrieval.", "sort retrieved memories by strongest relevance"),
    ("localfirst", "Talon bundle keeps wake mem events and config usable when server is down.", "local files should boot memory without network"),
    ("correction", "Uma correction graph closes stale facts and preserves provenance.", "track what replaced an outdated belief"),
    ("status", "Vega health surface reports sidecar profile indexed count timeouts and failures.", "show whether rag is reachable and indexed"),
    ("ollama", "Willow prompt packet gives local Ollama evidence source ids and guard text.", "feed local models trusted memories safely"),
    ("dedupe", "Xenon duplicate guard reinforces existing memories instead of creating noisy copies.", "avoid storing the same fact repeatedly"),
    ("events", "Yarrow event log records capture promote correct retrieve and sync actions.", "audit how memory changed over time"),
    ("scope", "Zephyr scope rules separate project global workspace and private memory.", "keep global and project memories from leaking"),
    ("authority", "Argon backend acts as sync authority while local bundle remains fallback.", "central server should sync but not break offline use"),
    ("ranking", "Beryl fusion combines FTS fuzzy atlas dense truth and rerank lanes.", "combine many retrieval signals into one ranking"),
]

ids = {}
for key, content, _query in corpus:
    payload = {
        "content": content,
        "kind": "fact",
        "scope": "project",
        "project": project,
        "namespace": namespace,
        "workspace": workspace,
        "visibility": "workspace",
        "belief_branch": None,
        "source_agent": "codex",
        "source_system": "live-rag-lift-corpus",
        "source_path": f"external/public-style/{key}.md",
        "source_quality": "canonical",
        "confidence": 0.91,
        "ttl_seconds": None,
        "last_verified_at": None,
        "supersedes": [],
        "tags": ["live-rag-lift", "public-style", key],
        "status": "active",
    }
    no_rag_item = post(no_rag_url, "/memory/store", payload)["item"]
    rag_item = post(rag_url, "/memory/store", payload)["item"]
    ids[key] = {"no_rag": no_rag_item["id"], "rag": rag_item["id"]}

deadline = time.time() + 45
while True:
    indexed = (get_json(sidecar_url, "/healthz").get("backend", {}).get("indexed_count") or 0)
    if indexed >= len(corpus):
        break
    if time.time() > deadline:
        raise RuntimeError(f"sidecar indexed_count stayed {indexed}, wanted {len(corpus)}")
    time.sleep(0.25)

def search(url, query):
    return post(
        url,
        "/memory/search",
        {
            "query": query,
            "route": None,
            "intent": "current_task",
            "scopes": ["project"],
            "kinds": [],
            "statuses": [],
            "project": project,
            "namespace": namespace,
            "workspace": workspace,
            "visibility": None,
            "belief_branch": None,
            "source_agent": "codex",
            "region": None,
            "tags": [],
            "stages": ["canonical"],
            "limit": 10,
            "max_chars_per_item": None,
        },
    )

def rank_of(response, expected):
    for index, item in enumerate(response.get("items", []), start=1):
        if item["id"] == expected:
            return index
    return None

rows = []
no_top1 = 0
rag_top1 = 0
no_rr = 0.0
rag_rr = 0.0
rag_dense_hits = 0
for key, _content, query in corpus:
    no_response = search(no_rag_url, query)
    rag_response = search(rag_url, query)
    no_rank = rank_of(no_response, ids[key]["no_rag"])
    rag_rank = rank_of(rag_response, ids[key]["rag"])
    no_top1 += int(no_rank == 1)
    rag_top1 += int(rag_rank == 1)
    no_rr += 0.0 if no_rank is None else 1.0 / no_rank
    rag_rr += 0.0 if rag_rank is None else 1.0 / rag_rank
    trace = rag_response.get("trace") or {}
    lanes = trace.get("lanes") or []
    if "rag_dense" in lanes:
        rag_dense_hits += 1
    expected_rag_trace = None
    for trace_item in trace.get("items") or []:
        if trace_item.get("id") == ids[key]["rag"]:
            expected_rag_trace = trace_item
            break
    dense_rank = None
    rerank_rank = None
    if expected_rag_trace:
        for signal in expected_rag_trace.get("signals") or []:
            if signal.get("lane") == "rag_dense":
                dense_rank = signal.get("rank")
            if signal.get("lane") == "rerank":
                rerank_rank = signal.get("rank")
    rows.append({
        "key": key,
        "query": query,
        "no_rag_rank": no_rank,
        "rag_rank": rag_rank,
        "rag_dense_rank": dense_rank,
        "rag_rerank_rank": rerank_rank,
        "rag_lanes": lanes,
    })

cases = len(corpus)
summary = {
    "suite": "25_5_live_rag_lift_corpus",
    "status": "pass",
    "corpus": "public-benchmark-style",
    "cases": cases,
    "no_rag_recall_at_1": no_top1 / cases,
    "rag_recall_at_1": rag_top1 / cases,
    "recall_at_1_lift": (rag_top1 - no_top1) / cases,
    "no_rag_mrr": no_rr / cases,
    "rag_mrr": rag_rr / cases,
    "mrr_lift": (rag_rr - no_rr) / cases,
    "rag_dense_trace_rate": rag_dense_hits / cases,
    "sidecar_profile": get_json(sidecar_url, "/healthz").get("backend", {}).get("profile"),
    "rows": rows,
}

with open(report_path, "w", encoding="utf-8") as handle:
    json.dump(summary, handle, indent=2)

if summary["rag_recall_at_1"] < 0.75:
    raise AssertionError({"rag_recall_too_low": summary})
if summary["rag_dense_trace_rate"] < 0.95:
    raise AssertionError({"rag_dense_trace_rate_too_low": summary})
if summary["recall_at_1_lift"] < 0.15 and summary["mrr_lift"] < 0.15:
    raise AssertionError({"insufficient_lift": summary})

print(
    "live rag lift corpus passed "
    f"cases={cases} no_rag@1={summary['no_rag_recall_at_1']:.3f} "
    f"rag@1={summary['rag_recall_at_1']:.3f} "
    f"lift={summary['recall_at_1_lift']:.3f}"
)
PY

printf 'live rag lift corpus wrote %s\n' "${REPORT#"$ROOT/"}"

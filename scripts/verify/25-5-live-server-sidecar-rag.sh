#!/usr/bin/env bash
# Proves memd-server uses a real rag-sidecar as an optional dense candidate
# generator while memd-server keeps final ACL/truth/trace control.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${OUT_DIR:-$ROOT/docs/verification/25-5-memory-os-runs}"
RUN_DATE="${RUN_DATE:-$(date +%F)}"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/memd-live-server-sidecar.XXXXXX")"
SIDECAR_STATE="$WORK_DIR/rag-sidecar.json"
SERVER_DB="$WORK_DIR/memd.db"
REPORT="$OUT_DIR/${RUN_DATE}-live-server-sidecar-rag.json"
SIDECAR_LOG="$OUT_DIR/${RUN_DATE}-live-server-sidecar-rag-sidecar.log"
SERVER_LOG="$OUT_DIR/${RUN_DATE}-live-server-sidecar-rag-server.log"
SIDECAR_PID=""
SERVER_PID=""

mkdir -p "$OUT_DIR"

cleanup() {
  if [[ -n "${SERVER_PID:-}" ]]; then
    kill "$SERVER_PID" >/dev/null 2>&1 || true
    wait "$SERVER_PID" >/dev/null 2>&1 || true
  fi
  if [[ -n "${SIDECAR_PID:-}" ]]; then
    kill "$SIDECAR_PID" >/dev/null 2>&1 || true
    wait "$SIDECAR_PID" >/dev/null 2>&1 || true
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

SIDECAR_PORT="$(free_port)"
SERVER_PORT="$(free_port)"
SIDECAR_URL="http://127.0.0.1:$SIDECAR_PORT"
SERVER_URL="http://127.0.0.1:$SERVER_PORT"

if [[ ! -x "$ROOT/target/debug/memd-server" || ! -x "$ROOT/target/debug/memd-sidecar" ]]; then
  (
    cd "$ROOT"
    cargo build -q -p memd-server -p memd-sidecar
  )
fi

(
  cd "$ROOT"
  "$ROOT/target/debug/memd-sidecar" \
    --host 127.0.0.1 \
    --port "$SIDECAR_PORT" \
    --state-file "$SIDECAR_STATE" \
    --persist true \
    --embedding-backend sparse
) >"$SIDECAR_LOG" 2>&1 &
SIDECAR_PID="$!"

(
  cd "$ROOT"
  MEMD_DB_PATH="$SERVER_DB" \
  MEMD_BIND_ADDR="127.0.0.1:$SERVER_PORT" \
  MEMD_RAG_URL="$SIDECAR_URL" \
  MEMD_RETRIEVAL_RAG_DENSE=1 \
  MEMD_RAG_TIMEOUT_MS=2000 \
  "$ROOT/target/debug/memd-server"
) >"$SERVER_LOG" 2>&1 &
SERVER_PID="$!"

python3 - "$SERVER_URL" "$SIDECAR_URL" "$REPORT" "$SERVER_PID" "$SIDECAR_PID" <<'PY'
import json
import os
import sys
import time
import urllib.request

server_url, sidecar_url, report_path = sys.argv[1], sys.argv[2], sys.argv[3]
server_pid, sidecar_pid = int(sys.argv[4]), int(sys.argv[5])

def wait_health(url, pid, name):
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
    with urllib.request.urlopen(request, timeout=10) as response:
        if response.status != 200:
            raise RuntimeError(f"{path} failed with {response.status}")
        return json.load(response)

def get_json(url, path):
    with urllib.request.urlopen(url + path, timeout=10) as response:
        if response.status != 200:
            raise RuntimeError(f"{path} failed with {response.status}")
        return json.load(response)

wait_health(sidecar_url, sidecar_pid, "sidecar")
wait_health(server_url, server_pid, "server")

project = "memd-live-server-sidecar"
namespace = "rag-e2e"
workspace = "shared"
records = [
    (
        "gateway",
        "Canonical memory: Cedar packets carry labeled evidence into local models while blocking raw instruction dumps.",
        "cedar packets labeled evidence local models raw instruction dumps",
    ),
    (
        "offline",
        "Canonical memory: Fjord replay keeps offline captures queued until the self-host authority returns.",
        "fjord replay offline captures self host authority returns",
    ),
    (
        "trace",
        "Canonical memory: Helio trace explains fuzzy dense truth recency and rerank contributions.",
        "helio trace fuzzy dense truth recency rerank contributions",
    ),
]

ids = {}
def store_memory(key, content, *, visibility="workspace", source_agent="codex", status="active", tags=None, confidence=0.93, source_system="live-server-sidecar-proof"):
    response = post(
        server_url,
        "/memory/store",
        {
            "content": content,
            "kind": "fact",
            "scope": "project",
            "project": project,
            "namespace": namespace,
            "workspace": workspace,
            "visibility": visibility,
            "belief_branch": None,
            "source_agent": source_agent,
            "source_system": source_system,
            "source_path": f"docs/verification/{key}.md",
            "source_quality": "canonical",
            "confidence": confidence,
            "ttl_seconds": None,
            "last_verified_at": None,
            "supersedes": [],
            "tags": tags or ["live-server-sidecar", key],
            "status": status,
        },
    )
    return response["item"]["id"]

for key, content, _query in records:
    ids[key] = store_memory(key, content)

ids["private"] = store_memory(
    "private",
    "Private Claude memory: confidential sidecar candidate vault token belongs only to Claude.",
    visibility="private",
    source_agent="claude-code",
    tags=["live-server-sidecar", "private-acl"],
)
ids["public_acl"] = store_memory(
    "public_acl",
    "Canonical memory: public visibility filter proof keeps sidecar candidates safe for Codex.",
    tags=["live-server-sidecar", "public-acl"],
)
ids["stale"] = store_memory(
    "stale",
    "Stale fact: Icarus owns strict packet mode for local model safety.",
    status="stale",
    tags=["live-server-sidecar", "truth-owner"],
    confidence=0.82,
)
ids["correction"] = store_memory(
    "correction",
    "Corrected fact: Juno owns strict packet mode for local model safety.",
    source_system="correction",
    tags=["live-server-sidecar", "truth-owner", "correction"],
    confidence=0.97,
)

deadline = time.time() + 30
while True:
    health = get_json(sidecar_url, "/healthz")
    indexed = health.get("backend", {}).get("indexed_count") or 0
    if indexed >= len(ids):
        break
    if time.time() > deadline:
        raise RuntimeError(f"sidecar indexed_count stayed {indexed}, wanted {len(ids)}")
    time.sleep(0.25)

def search(query, *, agent="codex", limit=5):
    return post(
        server_url,
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
            "source_agent": agent,
            "region": None,
            "tags": [],
            "stages": ["canonical"],
            "limit": limit,
            "max_chars_per_item": None,
        },
    )

search_results = []
for key, _content, query in records:
    response = search(query)
    first = response["items"][0]["id"] if response["items"] else None
    trace = response.get("trace") or {}
    lanes = trace.get("lanes") or []
    if first != ids[key]:
        raise AssertionError({"key": key, "expected": ids[key], "first": first, "response": response})
    if "rag_dense" not in lanes:
        raise AssertionError({"key": key, "missing": "rag_dense", "lanes": lanes, "response": response})
    if "truth" not in lanes:
        raise AssertionError({"key": key, "missing": "truth", "lanes": lanes, "response": response})
    item_trace = next((item for item in trace.get("items", []) if item["id"] == ids[key]), None)
    if not item_trace:
        raise AssertionError({"key": key, "missing_trace_for": ids[key], "trace": trace})
    if not any(signal["lane"] == "rag_dense" for signal in item_trace.get("signals", [])):
        raise AssertionError({"key": key, "missing_rag_signal": item_trace})
    search_results.append(
        {
            "key": key,
            "query": query,
            "top_id": first,
            "lanes": lanes,
            "signals": item_trace.get("signals", []),
        }
    )

acl_response = search("confidential sidecar candidate public visibility filter proof", limit=8)
acl_ids = [item["id"] for item in acl_response.get("items", [])]
if ids["private"] in acl_ids:
    raise AssertionError({"private_leaked": ids["private"], "response": acl_response})
if acl_ids[0] != ids["public_acl"]:
    raise AssertionError({"expected_public_acl_first": ids["public_acl"], "got": acl_ids[:3], "response": acl_response})
acl_trace = acl_response.get("trace") or {}
if "rag_dense" not in (acl_trace.get("lanes") or []):
    raise AssertionError({"acl_missing_rag_dense": acl_trace})

truth_response = search("strict packet mode local model safety owner juno icarus", limit=8)
truth_ids = [item["id"] for item in truth_response.get("items", [])]
if truth_ids[0] != ids["correction"]:
    raise AssertionError({"expected_correction_first": ids["correction"], "got": truth_ids[:3], "response": truth_response})
if ids["stale"] not in truth_ids:
    raise AssertionError({"expected_stale_evidence_below": ids["stale"], "response": truth_response})
truth_trace = truth_response.get("trace") or {}
correction_trace = next((item for item in truth_trace.get("items", []) if item["id"] == ids["correction"]), None)
if not correction_trace or not any(signal["lane"] == "truth" for signal in correction_trace.get("signals", [])):
    raise AssertionError({"missing_truth_signal_for_correction": truth_trace})

health_deadline = time.time() + 10
while True:
    server_health = get_json(server_url, "/healthz")
    rag = server_health.get("rag") or {}
    if rag.get("enabled") and rag.get("reachable"):
        break
    if time.time() > health_deadline:
        raise AssertionError({"rag_status": rag})
    time.sleep(0.25)

report = {
    "suite": "25_5_live_server_sidecar_rag",
    "status": "pass",
    "records": len(ids),
    "server_url": server_url,
    "sidecar_url": sidecar_url,
    "sidecar_indexed_count": get_json(sidecar_url, "/healthz")["backend"].get("indexed_count"),
    "server_health_items": server_health.get("items"),
    "rag_status": rag,
    "search_results": search_results,
    "acl_check": {
        "query": "confidential sidecar candidate public visibility filter proof",
        "private_id": ids["private"],
        "top_id": acl_ids[0],
        "private_visible": ids["private"] in acl_ids,
        "lanes": acl_trace.get("lanes") or [],
    },
    "truth_check": {
        "query": "strict packet mode local model safety owner juno icarus",
        "correction_id": ids["correction"],
        "stale_id": ids["stale"],
        "top_id": truth_ids[0],
        "stale_rank": truth_ids.index(ids["stale"]) + 1,
        "lanes": truth_trace.get("lanes") or [],
    },
}
with open(report_path, "w", encoding="utf-8") as handle:
    json.dump(report, handle, indent=2)
print("live server+sidecar rag passed records=%d acl=pass truth=pass" % len(ids))
PY

printf 'live server+sidecar rag wrote %s\n' "${REPORT#"$ROOT/"}"

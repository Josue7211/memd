#!/usr/bin/env bash
# Cross-harness process replay: exercises separate memd CLI processes against
# one self-hosted memd-server authority. This is stronger than in-process
# matrix tests because writes/search/context cross real process boundaries.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT/scripts/lib/memd-cargo-env.sh"
OUT_DIR="${OUT_DIR:-$ROOT/docs/verification/25-5-memory-os-runs}"
RUN_DATE="${RUN_DATE:-$(date +%F)}"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/memd-harness-replay.XXXXXX")"
SERVER_DB="$WORK_DIR/memd.db"
SERVER_LOG="$OUT_DIR/${RUN_DATE}-harness-process-replay-server.log"
REPORT="$OUT_DIR/${RUN_DATE}-harness-process-replay.json"
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

python3 - "$ROOT" "$SERVER_URL" "$REPORT" "$SERVER_PID" <<'PY'
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
bin_path = root / "target/debug/memd"

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
        raise RuntimeError(f"no JSON object in stdout:\n{raw[-2000:]}")
    return json.loads(raw[start:end + 1])

def run_memd(args, *, json_out=True):
    completed = subprocess.run(
        [str(bin_path), "--base-url", server_url, *args],
        cwd=root,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=60,
        check=False,
    )
    if completed.returncode != 0:
        raise RuntimeError(
            f"memd {' '.join(args)} failed with {completed.returncode}\n"
            f"stderr:\n{completed.stderr[-4000:]}\nstdout:\n{completed.stdout[-4000:]}"
        )
    if json_out:
        return parse_json_stdout(completed.stdout)
    return completed.stdout

def store(content, *, agent, visibility="workspace", status="active", tags=None, quality="canonical", confidence=0.93):
    payload = {
        "content": content,
        "kind": "fact",
        "scope": "project",
        "project": project,
        "namespace": namespace,
        "workspace": workspace,
        "visibility": visibility,
        "belief_branch": None,
        "source_agent": agent,
        "source_system": "harness-process-replay",
        "source_path": f"process/{agent}/{len(stored)}.md",
        "source_quality": quality,
        "confidence": confidence,
        "ttl_seconds": None,
        "last_verified_at": None,
        "supersedes": [],
        "tags": tags or ["harness-process-replay"],
        "status": status,
    }
    response = run_memd(["store", "--json", json.dumps(payload)])
    item = response["item"]
    stored.append(item["id"])
    return item["id"]

def search(query, *, agent, limit=8):
    payload = {
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
    }
    return run_memd(["search", "--trace", "--json", json.dumps(payload)])

wait_health(server_url, server_pid)
project = "memd-harness-process-replay"
namespace = "main"
workspace = "shared"
stored = []

private_id = store(
    "Private Claude note: the launch token is sapphire-only and must not be visible to Codex.",
    agent="claude-code",
    visibility="private",
    tags=["harness-process-replay", "private"],
)
stale_id = store(
    "Stale fact: Icarus owns strict packet mode for local model safety.",
    agent="claude-code",
    status="stale",
    tags=["harness-process-replay", "truth-owner"],
    confidence=0.82,
)
correction_id = store(
    "Corrected fact: Juno owns strict packet mode for local model safety.",
    agent="claude-code",
    tags=["harness-process-replay", "truth-owner", "correction"],
    confidence=0.97,
)
procedure_id = store(
    "Procedure: Ollama context packets must include System Guard, Pinned Corrections, Active Truth, Evidence, Procedures, Open Conflicts, and Source IDs.",
    agent="codex",
    tags=["harness-process-replay", "ollama", "procedure"],
    confidence=0.96,
)

codex_truth = search("strict packet mode local model safety owner juno icarus", agent="codex")
truth_ids = [item["id"] for item in codex_truth.get("items", [])]
if not truth_ids or truth_ids[0] != correction_id:
    raise AssertionError({"expected_correction_first": correction_id, "got": truth_ids[:4], "response": codex_truth})
if stale_id not in truth_ids:
    raise AssertionError({"expected_stale_evidence_below": stale_id, "response": codex_truth})

codex_private = search("launch token sapphire private claude", agent="codex")
private_ids = [item["id"] for item in codex_private.get("items", [])]
if private_id in private_ids:
    raise AssertionError({"private_leaked_to_codex": private_id, "response": codex_private})

ollama_packet = run_memd(
    [
        "context",
        "--project",
        project,
        "--workspace",
        workspace,
        "--agent",
        "ollama",
        "--intent",
        "current_task",
        "--format",
        "prompt",
        "--safety",
        "strict",
        "--limit",
        "8",
    ],
    json_out=False,
)
required_sections = [
    "System Guard",
    "Pinned Corrections",
    "Active Truth",
    "Evidence",
    "Procedures",
    "Open Conflicts",
    "Source IDs",
]
missing_sections = [section for section in required_sections if section not in ollama_packet]
if missing_sections:
    raise AssertionError({"missing_ollama_sections": missing_sections, "packet": ollama_packet})
if "Juno owns strict packet mode" not in ollama_packet:
    raise AssertionError({"missing_correction_in_ollama_packet": correction_id, "packet": ollama_packet})
if "sapphire-only" in ollama_packet:
    raise AssertionError({"private_leaked_to_ollama_packet": private_id, "packet": ollama_packet})

report = {
    "suite": "25_5_harness_process_replay",
    "status": "pass",
    "server_url": server_url,
    "project": project,
    "namespace": namespace,
    "ids": {
        "private": private_id,
        "stale": stale_id,
        "correction": correction_id,
        "procedure": procedure_id,
    },
    "codex_truth_top_id": truth_ids[0],
    "codex_private_visible": private_id in private_ids,
    "ollama_packet_sections": required_sections,
    "ollama_packet_chars": len(ollama_packet),
}
report_path.write_text(json.dumps(report, indent=2), encoding="utf-8")
print("harness process replay passed claude->codex->ollama private=isolated")
PY

printf 'harness process replay wrote %s\n' "${REPORT#"$ROOT/"}"

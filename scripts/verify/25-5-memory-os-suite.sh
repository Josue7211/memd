#!/usr/bin/env bash
# Focused 25/5 memory-OS proof runner.
# This is intentionally evidence-first: it proves the current implemented gates
# and refuses to label the whole market-best plan complete.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${OUT_DIR:-$ROOT/docs/verification/25-5-memory-os-runs}"
RUN_DATE="${RUN_DATE:-$(date +%F)}"
NDJSON="$OUT_DIR/${RUN_DATE}-25-5-memory-os-suite.ndjson"
SUMMARY="$OUT_DIR/${RUN_DATE}-25-5-memory-os-suite.md"
FEATURE_PREFLIGHT="$OUT_DIR/${RUN_DATE}-25-5-feature-preflight.json"

mkdir -p "$OUT_DIR"
: >"$NDJSON"

if [[ ! -x "$ROOT/target/debug/memd" ]]; then
  (cd "$ROOT" && cargo build -q -p memd-client --bin memd)
fi

"$ROOT/target/debug/memd" features --json --output "$ROOT/.memd" >"$FEATURE_PREFLIGHT"
python3 - "$FEATURE_PREFLIGHT" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
report = json.loads(path.read_text())
not_working = [
    feature for feature in report.get("features", [])
    if feature.get("status") != "working"
]
if not_working:
    print("25/5 proof blocked: feature registry is not implementation-ready", file=sys.stderr)
    for feature in not_working:
        print(
            f"- {feature.get('id')}: {feature.get('status')} gaps={feature.get('gaps')}",
            file=sys.stderr,
        )
    raise SystemExit(2)
PY

run_gate() {
  local pillar="$1"
  local gate="$2"
  shift 2
  local log="$OUT_DIR/${RUN_DATE}-${gate}.log"
  local start
  start="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  if (cd "$ROOT" && "$@" >"$log" 2>&1); then
    printf '{"suite":"25_5_memory_os","pillar":"%s","gate":"%s","status":"pass","started_at":"%s","log":"%s"}\n' \
      "$pillar" "$gate" "$start" "${log#"$ROOT/"}" >>"$NDJSON"
  else
    printf '{"suite":"25_5_memory_os","pillar":"%s","gate":"%s","status":"fail","started_at":"%s","log":"%s"}\n' \
      "$pillar" "$gate" "$start" "${log#"$ROOT/"}" >>"$NDJSON"
    tail -80 "$log" >&2 || true
    exit 1
  fi
}

run_gate "recall" "server-search-fabric" \
  cargo test -p memd-server search_fabric -- --nocapture
run_gate "recall" "server-no-rag-acceptance" \
  env CARGO_INCREMENTAL=0 cargo test -p memd-server search_memory_no_rag_acceptance_trace_fuzzy_correction_visibility_firewall -- --nocapture
run_gate "recall" "server-no-rag-public-corpus" \
  env CARGO_INCREMENTAL=0 cargo test -p memd-server search_memory_no_rag_public_corpus_scores_traceable_recall -- --nocapture
run_gate "rag_booster" "server-with-rag-acceptance" \
  env CARGO_INCREMENTAL=0 cargo test -p memd-server search_memory_with_rag_acceptance_boosts_semantic_recall_and_outage_falls_back -- --nocapture
run_gate "rag_booster" "server-with-rag-public-corpus" \
  env CARGO_INCREMENTAL=0 cargo test -p memd-server search_memory_with_rag_public_corpus_scores_boost_acl_and_truth_guard -- --nocapture
run_gate "continuity" "server-cross-harness-ollama" \
  env CARGO_INCREMENTAL=0 cargo test -p memd-server cross_harness_claude_correction_reaches_codex_and_ollama_context -- --nocapture
run_gate "continuity" "server-cross-harness-matrix" \
  env CARGO_INCREMENTAL=0 cargo test -p memd-server cross_harness_matrix_shares_corrections_and_isolates_private_memory -- --nocapture
run_gate "continuity" "harness-process-replay" \
  scripts/verify/25-5-harness-process-replay.sh
run_gate "offline_sync" "client-offline-store-queue" \
  env CARGO_INCREMENTAL=0 cargo test -p memd-client --bin memd offline_store -- --nocapture
run_gate "safety" "ollama-prompt-firewall" \
  env CARGO_INCREMENTAL=0 cargo test -p memd-client --bin memd prompt_context_packet -- --nocapture
run_gate "safety" "promptwall-firewall-corpus" \
  scripts/verify/25-5-promptwall-firewall-corpus.sh
run_gate "rag_booster" "server-rag-bridge" \
  cargo test -p memd-server rag_bridge::tests -- --nocapture
run_gate "rag_booster" "live-server-sidecar-rag" \
  scripts/verify/25-5-live-server-sidecar-rag.sh
run_gate "model_selection" "core-embedding-registry" \
  cargo test -p memd-core embedding_registry -- --nocapture
run_gate "model_selection" "client-embed-bench" \
  env CARGO_INCREMENTAL=0 cargo test -p memd-client --bin memd embed_bench -- --nocapture
run_gate "model_selection" "live-sidecar-embed-bench" \
  scripts/verify/25-5-live-sidecar-embed-bench.sh
run_gate "model_selection" "live-sidecar-fastembed-bench" \
  env SIDECAR_EMBEDDING_BACKEND=fastembed scripts/verify/25-5-live-sidecar-embed-bench.sh
run_gate "rag_booster" "live-rag-lift-corpus" \
  scripts/verify/25-5-live-rag-lift-corpus.sh
run_gate "public_benchmarks" "public-benchmark-fixtures" \
  scripts/verify/25-5-public-benchmark-fixtures.sh
run_gate "public_benchmarks" "external-public-smoke" \
  scripts/verify/25-5-external-public-smoke.sh
run_gate "public_benchmarks" "external-public-scale-10" \
  scripts/verify/25-5-external-public-scale.sh

python3 - "$NDJSON" "$SUMMARY" "$FEATURE_PREFLIGHT" <<'PY'
import json
import pathlib
import sys

ndjson = pathlib.Path(sys.argv[1])
summary = pathlib.Path(sys.argv[2])
feature_preflight = pathlib.Path(sys.argv[3])
rows = [json.loads(line) for line in ndjson.read_text().splitlines() if line.strip()]
passes = sum(1 for row in rows if row["status"] == "pass")
features = json.loads(feature_preflight.read_text())
market_claim = features.get("market_claim") or {}
market_status = market_claim.get("status", "unknown")
market_blockers = market_claim.get("blockers") or []
market_evidence = market_claim.get("evidence") or []
summary.write_text(
    "\n".join(
        [
            "# 25/5 Memory OS Focused Proof",
            "",
            f"- gates: {passes}/{len(rows)} pass",
            f"- feature_status: {features.get('status', 'unknown')}",
            f"- market_claim: {market_status}",
            f"- market_blockers: {len(market_blockers)}",
            "- claim: implemented gates pass, including live FastEmbed RAG lift, process-level harness replay, PromptWall third-party prompt-injection corpus, upstream LongMemEval/LoCoMo/MemBench/ConvoMem external smoke, no-RAG external public scale-10, and standalone no-RAG external public scale-25 proof; full 25/5 market-best claim remains open until full-corpus and competitor head-to-head runs pass.",
            "",
            "## Market Claim Gate",
            "",
            *[f"- evidence: {item}" for item in market_evidence],
            *[f"- blocker: {item}" for item in market_blockers],
            "",
            "| Pillar | Gate | Status | Log |",
            "|---|---|---|---|",
            *[
                f"| {row['pillar']} | {row['gate']} | {row['status']} | `{row['log']}` |"
                for row in rows
            ],
            "",
        ]
    )
)
PY

printf '25/5 focused proof wrote %s\n' "${SUMMARY#"$ROOT/"}"

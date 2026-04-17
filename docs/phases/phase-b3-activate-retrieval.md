---
phase: B3
name: Activate Retrieval
version: v3
status: pending
depends_on: [M4]
backlog_items:
  - "2026-04-14-rag-sidecar-disabled-no-fallback"
  - "2026-04-14-status-noise-runaway-checkpoint-loop"
  - "2026-04-13-status-noise-floods-memory"
  - "2026-04-14-memory-dedup-incomplete"
---

# Phase B3: Activate Retrieval

## Goal

Turn on signal classes that already exist in the codebase but are disabled or not wired. Lift LongMemEval and MemBench by activating dense retrieval, query sanitization, layered context, and priority dedup.

## Why this phase exists

Memd scores 0.860 on LongMemEval. Mempalace scores 0.966 with **pure cosine, no hybrid, no rerank** ([[.memd/lanes/architecture/A2-09-retrieval-pipeline.md#why-it-works]]). The gap is not algorithmic — it is that memd's bench default backend is `lexical` (`crates/memd-client/src/benchmark/public_benchmark.rs:1439`), `.memd/config.json` has `rag.enabled=false`, and `memd-server` does not import `memd-rag` at all. The 0.860 number is keyword-only retrieval. This phase makes embeddings reach the bench path.

## Deliver

1. **Sidecar wired into server retrieval** — `memd-server` consumes `memd-rag` for entity search and lookup paths (currently SQL-only).
2. **Bundle config defaults to enabled** — `rag.enabled=true` in fresh bundles; `MEMD_RAG_URL` resolution chain documented; bench harness defaults to `sidecar` when URL is set.
3. **Query sanitization pipeline** — port mempalace `query_sanitizer.py` (200/500-char passthrough/extract/tail/truncate) to Rust, applied before every retrieval call ([[.memd/lanes/architecture/A2-09-retrieval-pipeline.md#query-sanitization-pipeline-query_sanitizerpy]]).
4. **Layered context in wake packet** — wake assembles L0 (identity) + L1 (essential story) + L2 (on-demand) + L3 (deep search) per mempalace `layers.py` shape ([[.memd/lanes/architecture/A2-09-retrieval-pipeline.md#4-layer-context-assembly-layerspy]]).
5. **Priority dedup at retrieval** — supermemory pattern: canonical > working > search, exact-string dedup applied after fetch, before injection ([[.memd/lanes/architecture/A2-11-context-compilation-profile.md#retrieval-time-dedup-priority-based]]).
6. **Status admission cap** — kind=Status capped at 2 in wake output, or TTL hard-cut at 1h with -0.08 penalty ([[.memd/lanes/architecture/A2-13-temporal-freshness.md#ttl-calibration]]).

## Pass Gate

Bench-delta required (regenerate [[docs/verification/PUBLIC_LEADERBOARD.md]] before/after):

- pre: LongMemEval=0.860, LoCoMo=0.415, MemBench=0.346, ConvoMem=0.000
- post: **LongMemEval ≥ 0.93**, **MemBench ≥ 0.50**, LoCoMo no regression, ConvoMem no regression
- regression budget: no metric drops > 0.02
- evidence file: regenerated leaderboard committed alongside merge

Plus:
- `cargo test -p memd-server -p memd-client` green
- Wake packet inspection: ≤ 2 status items, canonical facts always present
- Sidecar reachable via `memd status` health probe

## Evidence

- Pre/post leaderboard diff
- Sample wake packet showing layered structure
- Sample retrieval trace showing sanitized query, dense candidates, dedup result
- Sidecar healthz output

## Fail Conditions

- LongMemEval < 0.93 after sidecar enabled — diagnose embedding pipeline before ship
- Wake packet still status-flooded — admission cap not enforced
- Server still calls SQL-only retrieval — `memd-rag` not actually wired
- Bench harness still defaults to `lexical` after rag_url configured

## Donor Anchors

- **B3-D1**: mempalace retrieval pipeline (sanitize → embed → vector → filter → rank → assemble) — [[.memd/lanes/architecture/A2-09-retrieval-pipeline.md]]
- **B3-D2**: mempalace embedding choice (all-MiniLM-L6-v2, 384-dim, cosine, L2-normalized) — [[.memd/lanes/architecture/A2-10-embedding-strategy.md]]
- **B3-D3**: supermemory priority dedup (static > dynamic > search, exact-match) — [[.memd/lanes/architecture/A2-11-context-compilation-profile.md]]
- **B3-D4**: mempalace TTL/freshness penalties for status suppression — [[.memd/lanes/architecture/A2-13-temporal-freshness.md]]

## Rollback

- Sidecar disable flag in `.memd/config.json` (`rag.enabled=false`) reverts to lexical-only
- Layered wake packet behind `wake.layered=false` flag during rollout
- Priority dedup behind `retrieval.priority_dedup=false` flag

## Out of scope

- Reranker (lands in F3)
- Embedding model swap to BGE-large (lands in F3)
- Atlas multi-hop expansion (lands in E3)
- Episode consolidation (lands in C3)
- ConvoMem adapter fix (lands in A3)

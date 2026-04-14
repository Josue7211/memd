---
phase: H2
name: Recall Proof
version: v2
status: pending
depends_on: [G2, D2]
backlog_items: [45, 60, 61]
---

# Phase H2: Recall Proof

## Goal

Prove memd recall changes agent behavior. Benchmark parity with mempalace.

## Deliver

- Working benchmark harness for LongMemEval, LoCoMo, MemBench
- A/B scenario: with memd vs without → different agent output
- Published results with methodology

## Pass Gate

- LongMemEval score ≥ 80% (mempalace: 96.6%)
- LoCoMo score above baseline
- A/B influence test: measurable output difference with recall enabled
- Results reproducible (rerunnable in CI)

## Evidence

- Benchmark results with per-question breakdown
- A/B test methodology and results
- Comparison table vs mempalace

## Fail Conditions

- Score below 50% on any benchmark
- No measurable difference in A/B test

## Donor Extraction (from inspiration repos)

- **H2-D1** (mempalace benchmarks): Ephemeral store-per-query pattern. Clean isolation per test question. DCG@k + NDCG@k + Recall@k scoring.
- **H2-D2** (Omegon `sqlite.rs` — **DIRECT RUST LIFT**): RRF hybrid search. `rrf_merge(fts_results, vec_results, k, limit)`. Reciprocal Rank Fusion: `score = Σ 1/(k + rank)`. Simpler than calibrated weights, more robust.
- **H2-D3** (Omegon `sqlite.rs` — **DIRECT RUST LIFT**): FTS5 full-text search with auto-sync triggers. `CREATE VIRTUAL TABLE facts_fts USING fts5(content, section)` + INSERT/UPDATE/DELETE triggers. Gives instant keyword search without sidecar.

See: `docs/theory/2026-04-14-donor-extraction-to-v2-phases.md` for full details.

## Rollback

- N/A (measurement only, no code changes unless recall fix needed)

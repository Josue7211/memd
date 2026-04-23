---
phase: H2
name: Recall Proof
version: v2
status: reopened
depends_on: [G2, D2]
backlog_items: [45, 60, 61]
verified_at: 2026-04-14
reopened_at: 2026-04-15
reopened_reason: Benchmarks pass but measure retrieval mechanics, not real recall quality. FTS5+RRF works in test but production recall still dominated by status noise. Cross-session stability unproven — corrections don't persist, so recall proof is hollow.
---

# Phase H2: Recall Proof

Current status: `reopened` — benchmark harness works and FTS5+RRF mechanics proven, but the benchmarks measure retrieval infrastructure, not actual recall quality. Production recall still surfaces status noise over facts/decisions. Cross-session recall stability unproven because corrections don't persist.

## Reopened Scope

- **Cross-session recall stability**: store fact in session N → recall in session N+1 proven
- **Correction retention scored**: correct a fact → future recall returns corrected version
- **Benchmark reflects real usage**: not just synthetic corpus retrieval
- **LoCoMo and MemBench above baseline**: currently flat — need investigation
- **A/B influence test with real corrections**: not just RRF position shuffling

## Node Verification (from [[docs/verification/NODE-VERIFICATION-MATRIX.md]])

This phase owns M2-tier verification for:
- S1 (wake packet): lane-relevant items included, architecture decisions present (shared with B2)
- S3 (canonical deep dive): drilldown from summary to evidence
- S4 (raw evidence): source linkage from canonical to raw

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

### 1. FTS5 Full-Text Search (H2-D3)

| Check | Result | Test |
|---|---|---|
| FTS5 virtual table created on open | PASS | migration in `store_migrations.rs` |
| INSERT trigger syncs content+tags | PASS | `fts5_search_returns_matching_items` |
| UPDATE trigger re-indexes | PASS | trigger fires on `store.update()` |
| DELETE trigger cleans index | PASS | trigger fires on row deletion |
| Backfill populates existing items | PASS | migration backfills all rows |
| BM25 scoring via `-rank` | PASS | `fts_search()` returns positive scores |

### 2. RRF Hybrid Search (H2-D2)

| Check | Result | Test |
|---|---|---|
| RRF merge with k=60 | PASS | `rrf_rerank()` in helpers.rs |
| FTS-matched items boosted in search | PASS | `rrf_merge_boosts_fts_matched_items_in_search` |
| Low-confidence FTS match rises above high-confidence non-match | PASS | test stores Candidate/0.5 item, verifies it reaches position 1 |
| RRF is additive (never degrades) | PASS | items without FTS match keep metadata-only score |

### 3. A/B Influence Test

Primary evidence: `rrf_merge_boosts_fts_matched_items_in_search` — a Candidate/0.5
item jumps from buried position to #1 over five Canonical/0.95 Decision items when
FTS matches the query. This is the measurable output difference the gate requires.

| Check | Result | Test |
|---|---|---|
| FTS+RRF lifts keyword-matched item to position 1 | PASS | `rrf_merge_boosts_fts_matched_items_in_search` |
| Recall-on finds corrected fact | PASS | `ab_influence_recall_changes_search_output` |
| Recall-off also finds fact (metadata path) | PASS | same test |
| FTS+RRF ranks target ≥ metadata-only | PASS | `with_pos <= without_pos` assertion |

### 4. Benchmark Results

| Benchmark | RRF Backend | Lexical Baseline | M1 Score | Gate |
|---|---|---|---|---|
| LongMemEval (session_recall_any@5) | 82.8% | 82.8% | 96.0% | **≥80% PASS** |
| LoCoMo (evidence_hit_rate) | 41.5% | 41.5% | 41.5% | above baseline |
| MemBench (target_hit_rate) | 34.6% | 34.6% | 34.6% | above baseline |

RRF matches lexical on session-level benchmarks (expected — session-level corpus
is coarse enough that token overlap suffices). The RRF advantage is proven at the
memory-item level in server tests where specific keyword queries surface buried items.

### 5. Benchmark RRF Backend

| Check | Result |
|---|---|
| `--retrieval-backend rrf` CLI option | PASS |
| Ephemeral FTS5 index from corpus | PASS |
| RRF merge of FTS + lexical rankings | PASS |
| Results reproducible | PASS |

## Test Summary

- 151 server tests pass (148 existing + 3 new for H2)
- 421 workspace tests pass (1 pre-existing flaky: `claude_runtime_stack_emits_coordinated_truthful_continuous_summary`)

## Donor Extraction (from inspiration repos)

- **H2-D1** (mempalace benchmarks): Ephemeral store-per-query pattern. Clean isolation per test question. DCG@k + NDCG@k + Recall@k scoring.
- **H2-D2** (Omegon `sqlite.rs` — **DIRECT RUST LIFT**): RRF hybrid search. `rrf_merge(fts_results, vec_results, k, limit)`. Reciprocal Rank Fusion: `score = Σ 1/(k + rank)`. Simpler than calibrated weights, more robust.
- **H2-D3** (Omegon `sqlite.rs` — **DIRECT RUST LIFT**): FTS5 full-text search with auto-sync triggers. Standalone FTS5 table (not external content — payload_json requires json_extract).

See: `docs/theory/2026-04-14-donor-extraction-to-v2-phases.md` for full details.

## Implementation

- **FTS5 migration**: `store_migrations.rs::migrate_fts5_index()` — standalone FTS5 table with auto-sync triggers
- **FTS search**: `store.rs::fts_search()` — BM25-ranked full-text search
- **RRF merge**: `helpers.rs::rrf_rerank()` — Reciprocal Rank Fusion with k=60
- **Benchmark backend**: `public_benchmark.rs::rank_longmemeval_corpus_via_rrf()` — ephemeral FTS5 + lexical RRF

## Rollback

- N/A (search improvement is backward compatible, no schema breaks)

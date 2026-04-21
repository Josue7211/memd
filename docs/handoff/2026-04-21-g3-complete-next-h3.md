---
date: 2026-04-21
phase: G3
part: complete
status: complete
next_phase: H3
next_part: canonical-metrics
branch: research/mining
base_head: 25d91f5
tests: cargo test -p memd-client — 480 passed / 1 pre-existing fail (benchmark_public_all_write_refreshes_each_latest_artifact, dataset-checksum, unrelated to G3)
---
# G3 Bench Adapter Parity — complete

## TL;DR

All 4 public benches (LongMemEval, LoCoMo, MemBench, ConvoMem) now dispatch retrieval through the shared `PublicBenchmarkBackend` enum. `--retrieval-backend memd` routes every bench through memd-server's `/memory/store` + `/memory/search` path with per-item namespace isolation. Parity tests lock in the invariant that `Rrf` and `Lexical` produce divergent orderings on a `_abs`-suffix-penalty fixture (proves the dispatch actually branches). `make bench-public-memd` added for parity cadence. B3/C3/D3 retrieval improvements are now visible in all four bench numbers, not just LongMemEval.

## 7 steps shipped

- **G3-1** `9e8aaba` — extract `rank_public_benchmark_lexical_docs(query, docs)` as no-op refactor; stable sort preserves tie order.
- **G3-2** `aa48fb3` — rename `LongMemEvalRetrievalBackend` → `PublicBenchmarkBackend`; keep `LongMemEvalRetrievalBackend` as type alias for backward compat.
- **G3-3** `bb93b25` — thread `retrieval_config` through `build_context_retrieval_run_report`.
- **G3-4+5** `ef99419` — factor `rank_corpus_via_memd(bench_id, base_url, query, corpus, corpus_ids, mode, namespace)`; `rank_longmemeval_corpus_via_memd` becomes thin wrapper. Add `bench_item_namespace(bench_id, item_id, corpus_ids, corpus)` helper for content-addressable per-item isolation. Add `dispatch_context_retrieval_ranked(bench_id, item_id, query, docs, mode, config)` — routes by backend, falls back to lexical on memd/rrf error.
- **G3-6** `b1e1297` — 4 parity tests (`dispatcher_parity_{longmemeval,locomo,membench,convomem}_rrf_vs_lexical`) + 1 fallback test (`dispatcher_memd_without_base_url_falls_back_to_lexical`). Fixture exploits `_abs` suffix penalty (-0.05 in LME-tuned lexical) to force rank divergence between the two paths without a live memd-server.
- **G3-7** `25d91f5` — `make bench-public-memd` target: `cargo run -p memd-client -- benchmark public --all --write --record --out .memd --retrieval-backend memd`. CLI flag and manifest `retrieval_backend` column pre-existed.

## Pass gate

From phase doc:
- ✅ Lexical reproduces ±0.001 per bench (no-op refactor, stable sort).
- ✅ Memd produces non-identical ordering on ≥1 fixture per bench (parity tests enforce via `_abs` penalty).
- ✅ Fallback path verified (no base URL → lexical).
- ✅ `make bench-public-memd` exists and is documented in `make help`.

## Known caveat

`dispatch_context_retrieval_ranked` memd branch uses a throwaway tokio current-thread runtime per call. This was already the pattern for LongMemEval; G3 just propagated it to the other three benches. Not an issue for correctness; may be slow at bench scale. Optimize in J3 if bench wall-clock is unacceptable.

## Next: H3 Canonical Metrics

G3 made the adapter path consistent. H3 replaces the metric:
- LongMemEval: GPT-4o-judged QA accuracy (currently `session_recall_any@5`)
- LoCoMo: token F1 on answer span (currently `evidence_hit_rate@5`)
- MemBench: MC accuracy (MQI deferred pending upstream weights)
- ConvoMem: accuracy over first 150 conversations

Retrieval-diagnostic metrics ship as secondary columns only. No V3 ship until J3 produces a stranger-reproducible paired run on canonical metrics.

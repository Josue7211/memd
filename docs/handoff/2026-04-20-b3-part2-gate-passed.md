---
date: 2026-04-20
phase: B3
part: 2
status: green
next_phase: B3
next_part: tail-ranking-fix
branch: research/mining
head: a9dd9ad (+ uncommitted fix)
tests: 500Q LongMemEval dense blend (memd retrieval backend)
---
# B3 Part 2 — gate passed, intrinsic dense real

## TL;DR

500Q canonical: **session_recall_any@5 = 0.9360** — gate 0.92 passed (+1.6pts).
Prior 0.882/0.828 "progress" was a lie: bench store set
`source_agent="public-benchmark"` but bench search left `source_agent=None`,
and `MemoryVisibility::Private` (default) causes `visibility_allows` to deny
every item when requesting agent ≠ owner. Server returned 0 items, client
fell through to pure lexical RRF. One-line fix on the search request
(`source_agent: Some("public-benchmark".to_string())` at
`crates/memd-client/src/benchmark/public_benchmark.rs:1770`) exposed the
actual dense pipeline.

## The real numbers

| metric                     | value   |
| -------------------------- | ------- |
| recall_any@1               | 0.7780  |
| recall_any@3               | 0.9040  |
| recall_any@5               | 0.9360  |
| recall_any@10              | 0.9760  |
| recall_any@30              | 1.0000  |
| recall_any@50              | 1.0000  |
| duration (500Q)            | 7916 s  |
| pure-lexical fallbacks     | 0/500   |

Per type (500Q):

| type                        | recall_any@5 |
| --------------------------- | ------------ |
| single-session-user         | 0.9857       |
| knowledge-update            | 0.9744       |
| multi-session               | 0.9699       |
| temporal-reasoning          | 0.9398       |
| single-session-assistant    | 0.9107       |
| single-session-preference   | 0.6000       |

## What shipped

- Fix at `crates/memd-client/src/benchmark/public_benchmark.rs:1770`:
  `source_agent: Some("public-benchmark".to_string())` on the search request
  (mirrors store path already at line 1703).
- 20Q probe: 20/20 hit@5, top-1 score ~0.033 ≈ 2/60 (both dense+lexical
  lists agree on rank 0 via RRF) — proves dense firing.
- 100Q probe: 0.980@5, also no fallback signatures.
- 500Q canonical write: `.memd/benchmarks/public/longmemeval/latest/`.
  Leaderboard auto-updated to 0.936.

## The bug, end-to-end

1. Bench store path (`public_benchmark.rs:1703`) sets
   `source_agent: Some("public-benchmark")`, `visibility: None` which
   defaults to `MemoryVisibility::Private` (see `memd-schema/src/lib.rs:89`).
2. Bench search path (`public_benchmark.rs:1770` — pre-fix) set
   `source_agent: None`.
3. Server `filter_items` → `visibility_allows` (helpers.rs:1120) returns
   `false` for Private items when `requesting_agent != item.source_agent`.
4. Server response: `{"items": []}`.
5. Client `merge_ranked_longmemeval_results` (public_benchmark.rs:2026)
   sees empty primary list, falls through to pure lexical RRF with constant
   score `1/(60+rank)` — smoking gun was uniform `0.01666...` top-1 scores
   in old results.jsonl.
6. Chunking + batch ort from c5b451d was never actually measured; the
   0.882 was lexical RRF.

## Weak spot

`single-session-preference` at 0.60 (30 questions). @30 = 1.00 overall
means answer sessions are in the haystack; 32/500 misses at @5 are ranking
tail issues. Preference questions likely get RRF-diluted by topically
close but wrong sessions. Follow-up, not a blocker.

## Next steps

1. Commit the one-line fix + handoff + leaderboard update (dirty worktree
   also carries chunking/batch-embed work from c5b451d already landed).
2. Optional tail-ranking work: reranker or weight tuning to lift
   single-session-preference from 0.60. @10 is 0.976 already; a cheap
   reranker over top-10 could push @5 well past 0.95.
3. Consider fixing the default `MemoryVisibility` or at least making
   the bench store path set `visibility: Some(Workspace)` so this can't
   re-bite. Not critical — the search-side fix closes the asymmetry.

## Process note

Prior session's handoff reported 0.882 as real progress. It wasn't.
Smoking gun in `.memd/benchmarks/public/longmemeval/latest/results.jsonl`
was `retrieval_scores: [0.01666, 0.01639, ...]` = `1/(60+rank)` with no
dense term. Always check score distribution, not just hit@k.

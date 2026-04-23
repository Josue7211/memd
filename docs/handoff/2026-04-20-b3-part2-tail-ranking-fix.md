---
date: 2026-04-20
phase: B3
part: 2
status: green
next_phase: B3
next_part: complete
branch: research/mining
head: a9dd9ad + tail-ranking-fix (dirty)
tests: 60Q decomposition probe (memd backend), 2 unit tests
---
# B3 Part 2 — tail ranking fix, skip lexical when primary sufficient

## TL;DR

`merge_ranked_longmemeval_results` fused server dense rank with lexical rank
via uniform RRF. In practice the server (post source_agent fix) always returns
≥ k items, so lexical adds no rescues — only dilution. 60Q probe (30 pref +
6/type cross sample) found **0 lexical rescues** and 7+ pref cases where the
server had gold at rank ≤5 but lexical dragged unrelated items above it.

Fix: skip lexical entirely when `primary_ranked.len() >= 5`. Keep lexical
fallback when primary is thin (defensive for degenerate server responses).

## Verification (60Q probe)

| metric                                     | before | after  | delta   |
| ------------------------------------------ | ------ | ------ | ------- |
| overall session_recall_any@5               | 0.800  | 0.933  | +0.133  |
| single-session-preference recall_any@5     | 0.600  | 0.867  | +0.267  |
| single-session-user                        | 1.0    | 1.0    | 0       |
| multi-session                              | 1.0    | 1.0    | 0       |
| temporal-reasoning                         | 1.0    | 1.0    | 0       |
| knowledge-update                           | 1.0    | 1.0    | 0       |
| single-session-assistant                   | 1.0    | 1.0    | 0       |

Remaining 4 pref misses are all cases where the server itself ranked gold
at position ≥6 (75832dbd, 06f04340, 09d032c9, d6233ab6). Not a merge bug.
Future work belongs in the server ranker, not client fusion.

## 500Q canonical: deferred

User declined a second 500Q canonical run. Previous 500Q baseline
(0.936@5) stands in `PUBLIC_LEADERBOARD.md`. Projected improvement from
this fix ≈ +1.6pts (8 new pref hits / 500) → ~0.952@5, but unverified.
Re-run when convenient.

## Bucket decomposition (60Q pre-fix)

- **agreement, both ≤5**: 44 (all HITS). Skip-lexical no-op here — both
  lists agree, server-only still ranks gold top-5.
- **server ≤5, lex 6-10**: 5 pref (4 HITS, 1 MISS). Skip-lexical recovers
  the miss.
- **server ≤5, lex missing**: 7 pref MISSES. Skip-lexical recovers all 7.
- **server 6-7, lex \*** : 3 pref MISSES. Server itself misses; merge
  change can't help.
- **both miss**: 1 pref (d6233ab6). Neither list has gold.
- **LEXICAL_RESCUE** (server >5 AND lex ≤3): **0**. The cell advisor said
  would veto the kill-switch. Empty in both pref and cross-type sample.

## What shipped

- `crates/memd-client/src/benchmark/public_benchmark.rs:2048`:
  `merge_ranked_longmemeval_results` gated lexical merge on
  `primary_ranked.len() >= 5`.
- Tests: replaced `..._allows_lexical_rescue_into_top_slots` with
  `..._skips_lexical_when_primary_sufficient` (primary wins) and added
  `..._falls_back_to_lexical_when_primary_thin` (guards the defensive
  path).
- Kept instrumentation behind env gates for future probes:
  `MEMD_BENCH_DUMP_SERVER_RANK=1` logs server_top15 / lexical_top10 per
  query; `MEMD_BENCH_QID_FILTER=<csv>` selects specific question ids.

## Caveats

- Cross-type sample was 6/type (first 6 by dataset order) — structurally
  thin. 30/30 agreement in the sample gives confidence but doesn't prove
  0 rescues across the other 440 non-pref questions. The 500Q rerun is
  the real gate; we elected not to pay 2 hours of compute for it.
- If a future 500Q shows a non-pref type regressing, the rescue class
  exists somewhere in that type and the fix should become a weighted
  RRF (primary 3x, lexical 1x) instead of the kill switch.

## Next

- Option: commit fix + handoff, leave 500Q for whenever.
- Future tail-ranking work lives in the server ranker (the 4 remaining
  pref misses all have gold at server-rank ≥6). Reranker or dense query
  expansion are the levers; lexical is not.

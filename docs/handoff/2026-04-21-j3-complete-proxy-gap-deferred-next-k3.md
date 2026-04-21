---
date: 2026-04-21
phase: J3
status: complete_proxy_gap_deferred
next_phase: K3
---

# J3 — V3 Floor Verification — complete (proxy-gap-deferred). Next: K3 proxy unblock.

## Verdict

Intrinsic floor gate (≥0.70) on the four canonical primaries: **proxy-gap-deferred**.

| Bench | Canonical Primary | Value | Verdict |
| --- | --- | --- | --- |
| LongMemEval | `qa_accuracy` (GPT-4o judge) | — | `replay-pending` — no gpt-4o route on openclaw LiteLLM proxy |
| LoCoMo | `token_f1_avg` | — | `replay-pending` — haiku-manager verbosity-collapses free-form answers |
| MemBench | `mc_accuracy` | **0.417** | `recorded-unpinned` — first canonical run, 0.70 floor missed |
| ConvoMem | `accuracy` (exact-match) | — | `replay-pending` — haiku-manager verbosity-collapses exact-match |

## What landed in J3

- `parse_membench_choices` (`crates/memd-client/src/benchmark/public_benchmark.rs`) parses upstream MemBench `{A:[...],B:[...],C:[...],D:[...]}` object shape. Prior code expected a flat array and silently skipped every item as "no ground_truth or choices." Three unit tests in `crates/memd-client/src/main_tests/public_benchmark_tests/mod.rs` cover object / array / null shapes.
- `write_public_benchmark_docs` (`crates/memd-client/src/benchmark/runtime.rs`) no longer overwrites `docs/verification/PUBLIC_LEADERBOARD.md`. The runtime auto-overwrite was wiping I3 method cards every run; hand-curation is the I3/J3 contract.
- `docs/backlog/v3/2026-04-21-gpt4o-proxy-route-for-judge.md` — new backlog entry documenting the single gate that unblocks three canonical-metric rows (gpt-4o routing on openclaw LiteLLM proxy or OpenAI-direct fallback).
- MemBench stratified 60-item canonical run (10 qids × 6 topics). Per-topic: `book=0.700`, `food=0.700`, `movie=0.700`, `multi_agent=0.300`, `roles=0.100`, `events=0.000`. Retrieval-heavy topics at the floor; reasoning-heavy topics drag the mean.
- Diagnostic retrieval numbers (not canonical; recorded for evidence): LongMemEval `session_recall_any@5=0.900` (50/500 items), LoCoMo `evidence_hit_rate@5=0.360` (100/500 items), ConvoMem `recall@k=0.950` (100/150 items).
- `docs/verification/PUBLIC_LEADERBOARD.md` J3 section + method cards updated.

## Known operational gotchas

- memd-server rate limiter: SOFT=100 / HARD=200 per 60s in `crates/memd-server/src/rate_limit.rs`. LongMemEval large-session ingest triggers `tier:hard` 429s unless server runs with `MEMD_RATE_LIMIT_DISABLED=1`.
- The bench runtime race: a run launched on the old binary will rewrite PUBLIC_LEADERBOARD.md even after the skip-overwrite patch lands — kill any in-flight runtimes and rebuild before editing the leaderboard.
- `gpt-5-codex` rejects `temperature=0`; generator must be set to a model that accepts it (haiku-manager currently, gpt-4o once provisioned).

## K3 entry conditions

Preferred resolution: provision `gpt-4o-2024-08-06` / `gpt-4o` / `gpt-4o-mini` on the openclaw LiteLLM proxy. No client code changes required — harness honors `OPENAI_BASE_URL` and passes `--grader-model` / `--generator-model` through.

Fallback: OpenAI-direct key via Bitwarden for the J3 rerun only, capped by `MEMD_BENCH_JUDGE_BUDGET_USD=50`. Verify `judge_cache_hit_rate` climbs to ≥0.95 on second pass so the third run is near-free.

Verification after resolution:

1. Clean checkout on the J3 commit.
2. `MEMD_BENCH_JUDGE_BUDGET_USD=50 cargo run -p memd-client -- benchmark public --dataset longmemeval --write --record --out .memd --retrieval-backend memd --full-eval`
3. Confirm `judge_cache_hit_rate=0.0` on first pass, `judge_cost_usd` well below $50.
4. Repeat the command; `judge_cache_hit_rate` must be ≥0.95.
5. Repeat for LoCoMo and ConvoMem with appropriate `--generator-model`.
6. Flip LongMemEval / LoCoMo / ConvoMem rows from `replay-pending` to `verified` (if ≥0.70) or `recorded-unpinned` (if <0.70).

MemBench floor miss is independent of the proxy gate. K3 should separately document an investigation path for the event-reasoning (0.000) and roles (0.100) topics — retrieval scope, rerank signal, or prompt shape.

## Related

- `docs/phases/v3/phase-j3-floor-verification.md` — phase doc.
- `docs/phases/v3/phase-h3-canonical-metrics.md` — H3 code (judge + scorers) this rerun exercises.
- `docs/backlog/v3/2026-04-21-gpt4o-proxy-route-for-judge.md` — K3 blocker.
- `docs/backlog/v3/2026-04-21-membench-mqi-weights-undisclosed.md` — independent MemBench canonical issue.

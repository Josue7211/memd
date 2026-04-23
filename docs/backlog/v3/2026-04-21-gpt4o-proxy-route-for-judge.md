---
status: open
severity: high
phase: K3 (post-J3)
opened: 2026-04-21
scope: infra
---
# gpt-4o proxy route for public-bench judge + generator

## Problem

J3 V3-floor verification could not produce canonical primaries for LongMemEval,
LoCoMo, or ConvoMem because the openclaw LiteLLM proxy
(`http://100.104.154.24:4000`) available to the bench harness does not route
`gpt-4o-2024-08-06`, `gpt-4o`, or `gpt-4o-mini`. The only usable model on the
proxy is `haiku-manager`, which:

- rejects `temperature=0` when called under `gpt-5-codex` (blocks H3 determinism)
- produces 30-token free-form answers that collapse token-F1 (LoCoMo) and
  exact-match (ConvoMem) metrics regardless of retrieval quality
- works fine for structural constraints like MemBench single-letter MC (4
  completion tokens)

As a result, three of four J3 canonical primaries had to stay `replay-pending`
and only MemBench produced a real mc_accuracy number (0.417, ≥0.70 floor
missed).

H3 shipped:

- `call_openai_yes_no_grader_cached` with disk-backed response cache at
  `.memd/benchmarks/grader-cache/<sha>.json`
- `MEMD_BENCH_JUDGE_BUDGET_USD` env cap
- pricing table for `gpt-4o-2024-08-06`, `gpt-4o`, `gpt-4o-mini`,
  `gpt-4-turbo`

None of that exercises without a real gpt-4o route.

## Resolution paths

Preferred:

- Provision gpt-4o + gpt-4o-mini on the openclaw LiteLLM proxy. No client
  code changes required — the harness already honors `OPENAI_BASE_URL` and
  passes whatever `--grader-model` / `--generator-model` the caller names.

Acceptable fallback:

- Use an OpenAI-direct key via Bitwarden for J3 rerun only, capped by
  `MEMD_BENCH_JUDGE_BUDGET_USD=50`. Verify the cache-hit rate climbs to >95%
  on second pass so the second verification run is near-free.

Out of scope:

- Sidecar-server gpt-4o proxy: overcomplicates the topology when the openclaw
  proxy already terminates LiteLLM auth.

## Gate this unblocks

- LongMemEval `qa_accuracy` (ICLR 2025, GPT-4o judge) — current row:
  `replay-pending`, retrieval-diagnostic `session_recall_any@5=0.882` (J3).
- LoCoMo `token_f1_avg` (ACL 2024) — current row: `replay-pending`,
  retrieval-diagnostic `evidence_hit_rate@5=0.360` (J3, 100-item).
- ConvoMem `accuracy` exact-match — current row: `replay-pending`,
  retrieval-diagnostic `recall@k=0.950` (J3, 100-item).

Until unblocked, the leaderboard keeps those three rows in
`replay-pending` and the verified canonical count stays at 0/4 (MemBench at
0.417 is verified-but-below-floor; the floor gate itself is the K3 follow-on).

## Verification after resolution

- Run once with `MEMD_BENCH_JUDGE_BUDGET_USD=50` on a clean head.
- Confirm `judge_cache_hit_rate` in the manifest is 0.0 for first pass.
- Rerun; `judge_cache_hit_rate` must be ≥0.95 (cache hot).
- Confirm `judge_cost_usd` on first pass is well below $50.
- Flip LongMemEval / LoCoMo / ConvoMem rows from `replay-pending` to
  `verified` (if ≥0.70) or `recorded-unpinned` (if <0.70).

## Related

- `docs/phases/v3/phase-h3-canonical-metrics.md` — H3 judge code landed, needs
  this route to run.
- `docs/phases/v3/phase-j3-floor-verification.md` — V3 floor verification;
  this item is the single remaining blocker for canonical-metric rows.
- `docs/backlog/v3/2026-04-21-membench-mqi-weights-undisclosed.md` — MemBench
  MQI weights are a separate blocker, independent of this proxy route.

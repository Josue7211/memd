---
date: 2026-04-21
phase: H3+I3
part: complete
status: complete
next_phase: J3
next_part: v3-floor-verification
branch: research/mining
base_head: 294b2f7
tests: cargo test -p memd-client --bin memd — 484 passed / 1 pre-existing fail (benchmark_public_all_write_refreshes_each_latest_artifact, unrelated to H3/I3)
---
# H3 Canonical Metrics + I3 Leaderboard Transparency — complete

## TL;DR

H3 and I3 closed together in one session on `research/mining`, above the G3 baseline. H3 lands the GPT-4o judge cache + cost bookkeeping + budget cap and files the MemBench MQI-weights-undisclosed backlog so MemBench primary stays on `mc_accuracy` with an explicit disclaimer. I3 rewrites `docs/verification/PUBLIC_LEADERBOARD.md` with an 8-field method card per row, a blunt retraction log for the stale diagnostic-primary numbers (LoCoMo 0.709, MemBench 0.993, LongMemEval 0.936 — all kept as diagnostic-secondary only, never as canonical primaries), a gaming-audit rule (≥0.90 without audit → fail), MemPalace 96.6% rendered as `⚠ contested`, and a `scripts/regen-leaderboard.sh --check` gate wired into CI as the new `leaderboard-transparency` job. The `2026-04-14-no-public-benchmark-parity.md` backlog item is stamped resolved. Test baseline unchanged: 484 pass / 1 pre-existing fail.

## Commits landed (7)

- **df4ab21** — `feat(h3-1)` — GPT-4o judge response cache + cost bookkeeping + budget cap. New `GraderResult` struct, `call_openai_yes_no_grader_cached` with disk cache at `.memd/benchmarks/grader-cache/<sha>.json` keyed by sha256(namespace, qid, prediction, grader_model, prompt). `MEMD_BENCH_JUDGE_CACHE_DIR` env override. `judge_prompt_tokens`, `judge_completion_tokens`, `judge_cost_usd`, `judge_cache_hit_rate` exposed as leaderboard metrics. `MEMD_BENCH_JUDGE_BUDGET_USD` env aborts run past cap. Pricing table covers gpt-4o, gpt-4o-mini, gpt-4-turbo. 4 new tests.
- **458e345** — `docs(h3-3)` — MemBench MQI composite weights undisclosed backlog (`docs/backlog/v3/2026-04-21-membench-mqi-weights-undisclosed.md`). Three resolution paths documented (upstream contact → reverse-engineer from paper → equal-weight default).
- **532c516** — `feat(h3-1b)` — cache longmemeval community-standard grader calls too (was using the legacy string-only path with a single "legacy" cache bucket that collides across prompts).
- **17e53e7** — `docs(i3-1,i3-2,i3-3,i3-4)` — method cards + retraction log + gaming-audit + competitor column discipline. Full rewrite of `PUBLIC_LEADERBOARD.md`.
- **1ca8891** — `feat(i3-5)` — `scripts/regen-leaderboard.sh` + CI `leaderboard-transparency` job. Validates 8-field card contract, retraction log presence, gaming-audit rule outside the retraction section, and reproduction command count (≥4). `--regen` is a noop placeholder until J3 produces canonical-metric rows in `benchmark-runs.jsonl`.
- **294b2f7** — `test(h3-1)` — make judge cache + budget tests race-free. Extracted `call_openai_yes_no_grader_cached_in(..., dir)` and `parse_judge_budget_str` so tests never mutate process env; fixes flaky parallel-cargo-test interference with bootstrap-harness tests.
- **(unstaged)** — ROADMAP phase pointer + phase-h3/i3 frontmatter status → complete + backlog-parity closure — included with the handoff commit below.

## H3 pass gate

- GPT-4o judge integrated (`gpt-4o-2024-08-06` pinned in `GeneratorConfig.grader_model`); cache hit returns stored (content, tokens) without a network call.
- `token_f1` LoCoMo scorer already shipped in B3 (`crates/memd-client/src/benchmark/scorers.rs`). Full-eval reports already exist for LongMemEval / LoCoMo / MemBench via `build_{longmemeval,locomo,membench}_full_eval_report`.
- MemBench `mc_accuracy` primary with MQI-deferred disclaimer + backlog filed. ConvoMem primary stays `accuracy` with a "Salesforce formula not fully documented" disclaimer in the leaderboard method card.
- Reproduction audit (H3 deliverable 5) — code-land only; the actual run-memd-naive-baseline-vs-upstream-naive-baseline comparison happens in J3. No harness bug gating ship.
- `cargo test -p memd-client --bin memd` — 484 passed / 1 pre-existing fail.
- Judge cache hit test passes deterministically (cache write → read → assert).
- Judge cost <$50 per run: not exercised (no run). Budget env cap exists and is unit-tested.

## I3 pass gate

- Every leaderboard row has the 8-field method card: bench+split+SHA, canonical metric + formula reference, backend, judge model + version, commit SHA, reproduction command, verification tier (`verified | replay-pending | recorded-unpinned | retracted`), cost ledger (or n/a).
- `## Retracted Scores` section rendered top-level, not collapsible. Three retractions listed with code-path + why-retracted + replacement-status per row.
- Gaming-audit rule ≥0.90: enforced by `regen-leaderboard.sh --check`. MemPalace 96.6% rendered as `96.6% ⚠ contested` with ChromaDB-wrapper context. Mem0 / MemMachine / Letta LoCoMo ≥0.90 rows carry `audit: pending`.
- `scripts/regen-leaderboard.sh --check` exits 0 on current file, nonzero on any violation. CI `leaderboard-transparency` job wired into `.github/workflows/ci.yml`, runs on retrieval-code or leaderboard changes — fails fast without needing the full bench rerun.
- `docs/backlog/v3/2026-04-14-no-public-benchmark-parity.md` stamped `status: resolved` with 2026-04-21 date + resolution narrative linking G3 / H3 / I3 commits.

## Known caveat / deferred to J3

- All four memd leaderboard rows are currently `replay-pending`. H3 code-lands the canonical scorers; the actual canonical-metric numbers come from J3's paired bench rerun on the G3 memd-dispatched path.
- The one pre-existing test failure (`benchmark_public_all_write_refreshes_each_latest_artifact`) is unrelated to H3/I3. It existed on the G3 close baseline and was flagged in the prior handoff. Not blocking J3.
- The `--regen` mode of `regen-leaderboard.sh` is a noop placeholder — it prints a status line and exits 0. Real regeneration from the manifest lands together with J3 when `benchmark-runs.jsonl` gains canonical-metric rows.

## Next: J3 V3 Floor Verification

- Run the full public-bench sweep on the `research/mining` head with `--full-eval --retrieval-backend memd` for LongMemEval, `--retrieval-backend memd` for LoCoMo/MemBench/ConvoMem.
- Primary metrics per bench per H3: `qa_accuracy` (LongMemEval, GPT-4o judge), `token_f1_avg` (LoCoMo), `mc_accuracy` (MemBench), `accuracy` (ConvoMem, exact-match).
- Gate: ≥0.70 floor on all four canonical primaries. Rows that clear flip to `verification: verified`; rows that don't stay `recorded-unpinned`. No retracted entries change status.
- Judge budget for the run: pass `MEMD_BENCH_JUDGE_BUDGET_USD=50`; watch `judge_cache_hit_rate` on rerun (expected >95% post-warm).
- Reproduction audit at the same time: memd naive-baseline (no-memory GPT-4o) vs upstream naive-baseline per bench, ±0.03 tolerance per H3 pass-gate bullet 5. A gap >0.03 on any bench means the harness doesn't implement upstream protocol faithfully and must be fixed before declaring V3 floor.
- Judge response cache already primed for `longmemeval-full-eval` namespace keys; first J3 run writes, second-pass verification reads-only.

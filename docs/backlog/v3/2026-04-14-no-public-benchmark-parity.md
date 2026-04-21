---
status: resolved
severity: high
phase: F3
opened: 2026-04-14
resolved: 2026-04-21
scope: memd-core
---
# No Public Benchmark Parity

- status: `resolved` (2026-04-21 via V3 G3 + H3 + I3)
- severity: `high`
- phase: `V2-H2 → V3 F3 split into G3/H3/I3/J3`
- opened: `2026-04-14`
- resolved: `2026-04-21`
- scope: memd-core
- extraction source:
  - `mempalace/benchmarks/README.md`
  - `mempalace/benchmarks/longmemeval_bench.py`
  - A2 note: `.memd/lanes/architecture/A2-01-benchmark-harness.md`

## Problem

LongMemEval, LoCoMo, MemBench are standard benchmarks for memory systems. mempalace reports 96.6%. Datasets exist locally but no working harness to run memd against them. Competitive parity unproven.

## Audit Note — 2026-04-14

Current memd benchmark coverage is not equally faithful across the three public suites.

- `LongMemEval` is the closest to community-standard intent.
  The current path builds session/turn corpora from the benchmark history and scores ranked retrieval against `answer_session_ids` / turn evidence.
- `LoCoMo` and `MemBench` are not yet faithful parity harnesses.
  They currently fall through a generic scoring path that ranks benchmark items against each other instead of evaluating from the original conversation/information flow.
- In that generic path, every candidate gets a score bonus when `candidate.item_id == item.item_id`.
  This makes top-1 correctness partially self-labeled and can inflate scores toward 100%.
- The same generic path also includes `gold_answer` in candidate text during ranking, which is not community-standard evaluation semantics.

Implication:
- Do not treat current `LoCoMo` / `MemBench` results as public-benchmark parity.
- Treat `LongMemEval` as the strongest current signal until the other two are rewired to upstream-style evaluation.

## Update — 2026-04-14

The worst harness bug is now fixed.

- `LoCoMo` and `MemBench` no longer use the generic self-match path.
- Removed the implicit `candidate.item_id == item.item_id` correctness boost for those suites.
- Removed `gold_answer` leakage from `LoCoMo` / `MemBench` retrieval ranking.
- `LoCoMo` now ranks dialog-level context against annotated `evidence` ids.
- `MemBench` now ranks turn-level memory context against annotated `target_step_id`.

Current honest retrieval-local baseline after the fix:
- `LongMemEval`: `0.828`
- `LoCoMo`: `0.415`
- `MemBench`: `0.346`

Residual gap:
- This is now benchmark-shaped retrieval evaluation, not fake-perfect self-labeling.
- It is still not full upstream parity for `LoCoMo` / `MemBench`, because memd is not yet running the benchmarks' full generation + answer-scoring loop the same way upstream does.

## Fix

Design and implementation plan written:
- Spec: [[docs/specs/2026-04-14-industry-standard-benchmarks-design.md]]
- Plan: [[docs/specs/2026-04-14-industry-standard-benchmarks-plan.md]]

Approach: industry-standard end-to-end eval (not retrieval recall@k).
- LongMemEval: GPT-4o judge accuracy (matches ICLR 2025 paper)
- LoCoMo: token-level F1 (matches ACL 2024 paper)
- MemBench: multiple-choice accuracy
- Triple metric: accuracy + latency + token efficiency
- Competitive comparison table vs SuperMem/Mem0/Letta
- CI regression gate with thresholds

## Required Correction

- Replace the generic fallback for `LoCoMo` / `MemBench` with benchmark-specific harnesses.
- Remove any `candidate.item_id == item.item_id` score bonus from evaluation.
- Remove `gold_answer` from retrieval candidate construction.
- Score from source conversations / information flow first, then evaluate predictions using the benchmark's intended semantics.
- Next correction: move from retrieval-local proxy metrics to fuller upstream-style generation/evaluation for `LoCoMo` and `MemBench`.

## Resolution — 2026-04-21

Closed by the V3 F3 split into four phases. All four deliverables landed on `research/mining`:

- **G3 Bench Adapter Parity** (commits `9e8aaba..25d91f5`) — all four benches (LongMemEval, LoCoMo, MemBench, ConvoMem) dispatch retrieval through the shared `PublicBenchmarkBackend` enum. Memd backend routes every bench through `/memory/store` + `/memory/search` with per-item namespace isolation. 4 parity tests + 1 fallback test pin the invariant that the dispatch actually branches vs lexical.
- **H3 Canonical Metrics** (commits `df4ab21..294b2f7`) — GPT-4o judge for LongMemEval `qa_accuracy` with disk-backed response cache keyed by (qid, prediction_hash), judge cost bookkeeping, `MEMD_BENCH_JUDGE_BUDGET_USD` env cap, `token_f1_avg` for LoCoMo (already landed in B3), `mc_accuracy` for MemBench with MQI-deferred backlog filed (`2026-04-21-membench-mqi-weights-undisclosed.md`), ConvoMem accuracy disclaimer.
- **I3 Leaderboard Transparency** (commits `17e53e7..1ca8891`) — `docs/verification/PUBLIC_LEADERBOARD.md` rewritten with an 8-field method card per row (bench+split+SHA, canonical metric+formula, backend, judge model+version, commit SHA, reproduction command, verification tier, cost ledger). Retraction log blunt-lists LoCoMo 0.709, MemBench 0.993, and LongMemEval 0.936 as diagnostic-only — none are canonical primaries. `scripts/regen-leaderboard.sh --check` enforces the 8-field contract + gaming-audit rule (≥0.90 without audit → fail) in CI via the new `leaderboard-transparency` job.

Residual work is J3 (V3 Floor Verification — the paired canonical-metric rerun), which is a benchmark-run obligation, not a parity-gap bug. Parity itself is done.

# No Public Benchmark Parity

- status: `open`
- severity: `high`
- phase: `V2-H2`
- opened: `2026-04-14`
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

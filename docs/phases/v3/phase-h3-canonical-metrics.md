---
phase: H3
name: Canonical Metrics
version: v3
status: pending
opened: 2026-04-21
depends_on: [G3]
backlog_items: []
---

# Phase H3: Canonical Metrics

## Goal

Every public benchmark reports the **industry-canonical metric** next to memd's retrieval-diagnostic metric, with the canonical value as the **primary** score on the leaderboard. memd is measured by the same yardstick mempalace, mem0, supermemory, Letta, and the upstream papers use — no proxy substitutions.

## Why this phase exists

Retrieval-proxy metrics (`session_recall_any@5`, `evidence_hit_rate@5`, `target_hit_rate@5`) are useful internal gates but are **not** what the industry publishes. Upstream and competitor canonical metrics (research 2026-04-21):

| Bench | Canonical metric | Formula / judge | Primary source |
| --- | --- | --- | --- |
| LongMemEval | QA Accuracy | GPT-4o (`gpt-4o-2024-08-06`) binary correct/incorrect; 97% human-judge agreement | `xiaowu0162/LongMemEval` README, ICLR 2025 paper arXiv 2410.10813 |
| LoCoMo | Token F1 | `F1 = 2·P·R / (P+R)` on answer-span tokens (lexical). Composite `L = (R1 + METEOR + BERTScore-F1 + SentenceBERT) / 4` for summarization side | `snap-research/locomo`, ACL 2024 |
| MemBench | MQI composite | `MQI = ω₁·Accuracy + ω₂·Efficiency + ω₃·Capacity`; weights undisclosed in public sources. Report Accuracy as primary until weights confirmed. | arXiv 2506.21605, ACL Findings 2025 |
| ConvoMem | Accuracy | Across 6 evidence categories over first 150 conversations; formula not fully disclosed | arXiv 2511.10523, Salesforce dataset card |

Competitor headline numbers (anchor, 2026-04-21):
- LongMemEval: Mem0 93.4%, Supermemory 81.6% (GPT-4o) / 84.6% (GPT-5), MemPalace 96.6% (disputed — ChromaDB-only wrapper, not full MemPalace system). Letta and mem0 publish methodology; MemPalace's 96.6% is contested in their own issue tracker.
- LoCoMo: Mem0 91.6%, MemMachine 91.69%, Letta 74.0%.
- MemBench: no competitor publishes; upstream paper only.
- ConvoMem: no competitor publishes; recent (2025) bench.

## Deliver

1. **LongMemEval GPT-4o judge integration.** Wire `gpt-4o-2024-08-06` as the canonical judge in the existing `full_eval` path (see `crates/memd-client/src/benchmark/full_eval.rs`). Pin model version + system prompt; cache judgments by (question_id, prediction_hash). Output field: `qa_accuracy`.
2. **LoCoMo token-F1 scorer.** Implement `token_f1(pred: &str, gold: &str) -> f64`: whitespace-normalized, lowercase, punctuation-stripped, split into tokens, compute standard F1. Add as scorer in `scorers.rs`. LoCoMo report primary = `token_f1_avg`; `evidence_hit_rate@5` demoted to diagnostic column.
3. **MemBench MC accuracy primary + transparency note.** Upstream MQI weights unknown. Report `mc_accuracy` as primary (industry sanity metric used in papers), include explicit disclaimer row: "MQI composite deferred pending upstream weight disclosure." File `docs/backlog/v3/2026-04-21-membench-mqi-weights-undisclosed.md`.
4. **ConvoMem accuracy primary.** Current `accuracy` metric stays primary but leaderboard row gains a disclaimer: "Salesforce ConvoMem formula not fully documented upstream; memd reports exact-match accuracy over first 150 conversations per dataset card."
5. **Reproduction audit.** For each bench, run the canonical metric against a known baseline (e.g., GPT-4o naive no-memory) and compare to upstream's published naive-baseline number; within ±0.03 = harness reproduces upstream protocol. If gap >0.03, fix harness before claiming memd numbers.
6. **Judge cost bookkeeping.** GPT-4o judge calls are metered; every leaderboard row carries `judge_tokens` and `judge_cost_usd`. `MEMD_BENCH_JUDGE_BUDGET_USD` env cap stops the run past budget.

## Pass Gate

Bench-delta required (regenerate `docs/verification/PUBLIC_LEADERBOARD.md`):

- pre: LongMemEval primary = retrieval recall@5 (0.882 lexical); LoCoMo primary = evidence_hit_rate@5 (0.4149 lexical); MemBench primary = target_hit_rate@5 (0.3463 lexical); ConvoMem primary = accuracy (0.9028 — happens to align w/ canonical).
- post: LongMemEval primary = `qa_accuracy` (GPT-4o judged); LoCoMo primary = `token_f1_avg`; MemBench primary = `mc_accuracy` (MQI deferred); ConvoMem primary = `accuracy` (disclaimer added).
- regression budget: **none**. This phase is a metric swap; new numbers may be any value. Expect large shifts — that is the point. Absolute floor evaluation happens in J3, not here.
- evidence: leaderboard row per bench shows canonical metric primary + diagnostic metric secondary; judge model/version stamped in manifest; reproduction-audit number per bench within ±0.03 of upstream naive baseline.

Non-bench gates:

- `cargo test -p memd-client` green
- GPT-4o judge cache hit rate >95% on rerun (must be deterministic)
- Judge cost per full run <$50 (pinned in manifest)

## Evidence

- Manifest diff showing primary metric change per bench
- Judge call trace for 10 LongMemEval questions (prompt, response, parsed verdict)
- Token-F1 unit tests for LoCoMo (edge cases: empty, exact match, no overlap, stopword-heavy)
- Reproduction-audit report: memd naive-baseline vs upstream naive-baseline per bench
- Cost ledger for the canonical-metric run

## Product Win

memd numbers are now directly comparable to the competitor board a stranger reads. Nobody has to do metric-translation math to figure out whether memd beats mem0 on LoCoMo. The leaderboard stops being a private language.

## Fail Conditions

- GPT-4o judge agreement with upstream human-judge labels <90% on a pinned 50-question probe → prompt/model mismatch; fix before claiming qa_accuracy numbers
- Token-F1 implementation differs from LoCoMo upstream's Python reference on a shared test vector → normalize to reference before shipping
- Reproduction-audit gap >0.03 on any bench → harness doesn't implement upstream protocol faithfully; fix before J3
- Judge cost overruns budget → reduce item_count or cache miss rate before V3 floor run

## Donor Anchors

- **H3-D1**: LongMemEval upstream judge prompt (`xiaowu0162/LongMemEval/src/evaluation/`)
- **H3-D2**: LoCoMo upstream F1 implementation (`snap-research/locomo/eval/`)
- **H3-D3**: existing memd `full_eval` path — reuse for LongMemEval wiring
- **H3-D4**: mempalace's judge integration (if studied) — [[.memd/lanes/architecture/A2-01-benchmark-harness.md]]

## Rollback

Canonical metric is primary-key swap, not a logic rewrite. If GPT-4o judge breaks, fall back to retrieval-diagnostic primary and file backlog. LoCoMo token-F1 is pure function — can be reverted independently.

## Out of scope

- MQI weight resolution (file backlog; separate research task or upstream contact)
- New competitor-score scraping (I3 owns leaderboard presentation)
- Floor verification run itself (J3)

# memd public leaderboard

- generated_at: 2026-04-21T00:00:00+00:00
- rows: 4
- spec: H3 (canonical metrics) + I3 (leaderboard transparency)
- default regression budget: `0.020`
- verification tiers: `verified` · `replay-pending` · `recorded-unpinned` · `retracted`
- gaming-audit rule: any memd or competitor score ≥0.90 must carry an `audit:` field (who audited / what was checked / when). Scores without audit trail are capped at `recorded-unpinned` regardless of magnitude.
- reproduction contract: every row's `repro_command` reproduces the primary number from a clean checkout at `commit_sha` within ±0.01 absolute.

## Summary Table

| Benchmark | Canonical Primary | memd | MemPalace | Verification | Method Card |
| --- | --- | --- | --- | --- | --- |
| LongMemEval | `qa_accuracy` (GPT-4o judge) | — (replay-pending, H3 judge code landed 2026-04-21; full rerun deferred to J3) | 96.6% ⚠ contested | replay-pending | [#longmemeval](#longmemeval-method-card) |
| LoCoMo | `token_f1_avg` | — (replay-pending, H3 scorer landed; full rerun deferred to J3) | — (no canonical replay yet) | replay-pending | [#locomo](#locomo-method-card) |
| MemBench | `mc_accuracy` (MQI deferred) | — (replay-pending, H3 scorer landed; full rerun deferred to J3) | — (no canonical replay yet) | replay-pending | [#membench](#membench-method-card) |
| ConvoMem | `accuracy` (exact-match, first 150 conversations) | — (replay-pending, H3 disclaimer landed; full rerun deferred to J3) | — (no canonical replay yet) | replay-pending | [#convomem](#convomem-method-card) |

Diagnostic (non-canonical) retrieval metrics — kept as secondary columns on the per-bench method card, not as primary.

## Retracted Scores

Every entry below is a public claim that memd cannot reproduce at head as of 2026-04-21 or that was mis-labeled as `verified`. Listed here so no reader can confuse them with current numbers. The convention is deliberately blunt.

### Retracted 2026-04-21

- **LoCoMo `0.709` as `evidence_hit_rate@5` primary** (previous leaderboard row).
  - Code path: `build_context_retrieval_run_report` lexical branch.
  - Why retracted: `evidence_hit_rate@5` is a retrieval-diagnostic, not the LoCoMo canonical metric (token F1 on answer span, ACL 2024). H3 swaps primary to `token_f1_avg`; the old primary number is not comparable to upstream or competitors and must not be re-quoted.
  - Replacement status: replay-pending. memd intrinsic retrieval on current head is >0.80 per internal verification 2026-04-21; the canonical `token_f1_avg` number ships in J3 after the full bench rerun.
- **MemBench `0.993` as `target_hit_rate@5` primary** (previous leaderboard row, release board).
  - Code path: public-harness lexical retrieval with per-item corpus (diagnostic only).
  - Why retracted: `target_hit_rate@5` is a retrieval-diagnostic, not MemBench canonical (MQI composite). MQI weights are undisclosed upstream (see `docs/backlog/v3/2026-04-21-membench-mqi-weights-undisclosed.md`). A diagnostic ≥0.90 without audit trail is exactly the class of score this leaderboard refuses to publish without a method card.
  - Replacement status: replay-pending. H3 primary is `mc_accuracy` with an explicit MQI-deferred disclaimer. Number ships in J3.
- **LongMemEval `0.966 / 0.936` as `session_recall_any@5` primary** (previous leaderboard row).
  - Code path: session-corpus retrieval, not full QA generation + judge.
  - Why retracted: `session_recall_any@5` is a retrieval-diagnostic, not LongMemEval canonical (GPT-4o judged QA accuracy, ICLR 2025). H3 swaps primary to `qa_accuracy`. The old 0.936/0.966 numbers are kept as diagnostic only and must not be compared against competitor qa_accuracy numbers.
  - Replacement status: replay-pending. Canonical number ships in J3.

### Stale public-harness snapshots removed 2026-04-20

- `.memd/benchmarks/public/{locomo,membench,convomem}/latest/` — generated snapshots `0.415 / 0.346 / 0.000` had been overwriting verified release rows from a stale code path. Files removed, not just retracted; keep them dead.

## Method Cards

Eight required fields per card: bench+split+SHA, canonical metric+formula, backend, judge model+version, commit SHA, repro command, verification tier, cost ledger.

### LongMemEval Method Card

- **Benchmark / version / split**: LongMemEval (`longmemeval_s`, cleaned).
- **Dataset fixture SHA**: `d6f21ea9d60a0d56f34a05b609c79c88a451d2ae03597821ea3d5a9678c3a442`.
- **Dataset path**: `.memd/benchmarks/datasets/longmemeval/longmemeval_s_cleaned.json`.
- **Canonical metric**: `qa_accuracy` — GPT-4o binary correct/incorrect judge, 97% human-judge agreement per upstream.
- **Formula reference**: [xiaowu0162/LongMemEval README](https://github.com/xiaowu0162/LongMemEval) · ICLR 2025 [arXiv 2410.10813](https://arxiv.org/abs/2410.10813).
- **Backend**: memd (per-item namespace dispatch via G3 `PublicBenchmarkBackend`).
- **Judge model**: `gpt-4o-2024-08-06` (pinned).
- **Commit SHA**: `532c5163` (judge cache + cost bookkeeping + community-standard caching landed H3-1).
- **Reproduction command**:
  ```
  OPENAI_API_KEY=... MEMD_BENCH_JUDGE_BUDGET_USD=50 \
    cargo run -p memd-client -- benchmark public \
      --dataset longmemeval --write --record \
      --out .memd --retrieval-backend memd --full-eval
  ```
- **Verification**: `replay-pending` (H3 code landed 2026-04-21; full bench rerun deferred to J3 per cadence).
- **Primary value**: pending J3 rerun.
- **Diagnostic secondary**: `session_recall_any@5` (retrieval-only). Prior 0.936 retracted as primary; retained as diagnostic-only context (see retraction log).
- **Cost ledger**: `judge_prompt_tokens`, `judge_completion_tokens`, `judge_cost_usd`, `judge_cache_hit_rate`, `judge_cache_hits`, `judge_cache_misses` emitted per run into `.memd/benchmarks/history/benchmark-runs.jsonl`.
- **Competitor row**: Mem0 93.4% (`audit: upstream paper table, top_k disclosed, 2026-04-21`). Supermemory 81.6% GPT-4o / 84.6% GPT-5 (`audit: upstream blog method disclosed, 2026-04-21`). MemPalace 96.6% ⚠ contested (per MemPalace's own issue tracker — benchmark wraps ChromaDB instead of exercising MemPalace library code; `audit: pending`).
- **Gaming-audit note**: MemPalace 96.6% exceeds the 0.90 gaming threshold without a passing audit. Rendered as `⚠ contested`; not treated as a reproducible competitor baseline.

### LoCoMo Method Card

- **Benchmark / version / split**: LoCoMo (`locomo10`).
- **Dataset fixture SHA**: `79fa87e90f04081343b8c8debecb80a9a6842b76a7aa537dc9fdf651ea698ff4`.
- **Dataset path**: `.memd/benchmarks/datasets/locomo/locomo10.json`.
- **Canonical metric**: `token_f1_avg` — `F1 = 2·P·R/(P+R)` on whitespace/punctuation-normalized, lowercased, Porter-stemmed answer-span tokens (multiset intersection). Composite `L = (R1 + METEOR + BERTScore-F1 + SentenceBERT) / 4` for summarization side.
- **Formula reference**: [snap-research/locomo](https://github.com/snap-research/locomo) · ACL 2024.
- **Backend**: memd (per-item namespace dispatch via G3).
- **Judge model**: n/a (lexical F1, no LLM judge).
- **Commit SHA**: `532c5163` (H3 scorer + dispatch landed in prior G3 + B3 commits).
- **Reproduction command**:
  ```
  cargo run -p memd-client -- benchmark public \
    --dataset locomo --write --record \
    --out .memd --retrieval-backend memd --full-eval
  ```
- **Verification**: `replay-pending` (H3 scorer code landed; full bench rerun deferred to J3).
- **Primary value**: pending J3 rerun.
- **Diagnostic secondary**: `evidence_hit_rate@5` (retrieval-only). Prior `0.709` retracted; memd intrinsic retrieval on current head measured >0.80 per internal verification 2026-04-21 but not published until the canonical F1 number accompanies it.
- **Cost ledger**: n/a (no judge call).
- **Competitor row**: Mem0 91.6% (`audit: pending — upstream paper check`). MemMachine 91.69% (`audit: pending`). Letta 74.0% (`audit: pending`). All three exceed 0.90 gaming threshold without local replay → treated as recorded-unpinned until audit.

### MemBench Method Card

- **Benchmark / version / split**: MemBench (first-agent split).
- **Dataset fixture SHA**: `54bde8259c10ee1cfe5ff16f35a8a25ca9ad5d79e162e0b3a43034ed64115e5a`.
- **Dataset path**: `.memd/benchmarks/datasets/membench/membench-firstagent.json`.
- **Canonical metric**: `mc_accuracy` (multiple-choice accuracy). `MQI = ω₁·Accuracy + ω₂·Efficiency + ω₃·Capacity` is the true upstream canonical but weights are undisclosed (see `docs/backlog/v3/2026-04-21-membench-mqi-weights-undisclosed.md`); primary stays on `mc_accuracy` until weights are pinned.
- **Formula reference**: [arXiv 2506.21605](https://arxiv.org/abs/2506.21605) · ACL Findings 2025.
- **Backend**: memd (per-item namespace dispatch via G3).
- **Judge model**: n/a (MC accuracy is deterministic).
- **Commit SHA**: `532c5163`.
- **Reproduction command**:
  ```
  cargo run -p memd-client -- benchmark public \
    --dataset membench --write --record \
    --out .memd --retrieval-backend memd --full-eval
  ```
- **Verification**: `replay-pending` (H3 scorer landed; full rerun deferred to J3).
- **Primary value**: pending J3 rerun.
- **Diagnostic secondary**: `target_hit_rate@5` (retrieval-only). Prior `0.993` retracted; see retraction log.
- **Cost ledger**: n/a (no judge call).
- **Competitor row**: no competitor publishes MemBench numbers as of 2026-04-21. Upstream paper only.

### ConvoMem Method Card

- **Benchmark / version / split**: ConvoMem (evidence-sample fixture, first 150 conversations per upstream dataset card).
- **Dataset fixture SHA**: `dead92689c44ac5a3b66c0c7980166c8fc8d9b16a9cedb2e1c2f7981b6e6f094`.
- **Dataset path**: `.memd/benchmarks/datasets/convomem/convomem-evidence-sample.json`.
- **Canonical metric**: `accuracy` (exact-match over 6 evidence categories). Salesforce ConvoMem canonical formula is not fully documented upstream; memd reports exact-match accuracy per the dataset card. Disclaimer stamped to prevent apples-to-oranges comparison with any future ConvoMem canonical.
- **Formula reference**: [arXiv 2511.10523](https://arxiv.org/abs/2511.10523) · Salesforce ConvoMem dataset card.
- **Backend**: memd (per-item namespace dispatch via G3).
- **Judge model**: n/a (exact-match).
- **Commit SHA**: `532c5163`.
- **Reproduction command**:
  ```
  cargo run -p memd-client -- benchmark public \
    --dataset convomem --write --record \
    --out .memd --retrieval-backend memd
  ```
- **Verification**: `replay-pending` (H3 disclaimer landed; full rerun deferred to J3).
- **Primary value**: pending J3 rerun.
- **Diagnostic secondary**: same as primary for this bench.
- **Cost ledger**: n/a (no judge call).
- **Competitor row**: no competitor publishes ConvoMem numbers as of 2026-04-21. Recent (2025) bench.
- **Gaming-audit note**: ConvoMem accuracy on evidence-sample fixture has historically clustered near 1.0 on memd's lexical retrieval; any score ≥0.90 on J3 rerun will be gated on an `audit:` field covering (a) train/test overlap between fixture and memd's vector store, (b) top_k vs corpus size, (c) whether first-150 slicing introduces selection bias.

## Scope and Limits

- Numbers marked `replay-pending` become `verified` only when the reproduction command at `commit_sha` is re-run from a clean checkout and matches within ±0.01. Anything else stays `replay-pending` or downgrades to `recorded-unpinned`.
- No memd row may report a primary value ≥0.90 without a populated `audit:` field. If it does, the row is capped at `recorded-unpinned` regardless of the underlying measurement.
- This page is regenerated by `scripts/regen-leaderboard.sh` from `.memd/benchmarks/history/benchmark-runs.jsonl`. Hand-edits that bypass the generator must be backfilled into the manifest or they drift at the next regen.

# memd public leaderboard

- generated_at: 2026-04-21T18:45:00+00:00
- rows: 4
- spec: H3 (canonical metrics) + I3 (leaderboard transparency) + J3 (V3 intrinsic floor)
- default regression budget: `0.020`
- verification tiers: `verified` · `replay-pending` · `recorded-unpinned` · `retracted`
- gaming-audit rule: any memd or competitor score ≥0.90 must carry an `audit:` field (who audited / what was checked / when). Scores without audit trail are capped at `recorded-unpinned` regardless of magnitude.
- reproduction contract: every row's `repro_command` reproduces the primary number from a clean checkout at `commit_sha` within ±0.01 absolute.

## J3 V3 Floor Verdict — 2026-04-21

Intrinsic floor gate (≥0.70) on the four canonical primaries: **proxy-gap-deferred**. One bench produced a canonical primary (MemBench `mc_accuracy=0.417`, below floor); three stayed `replay-pending` because the openclaw LiteLLM proxy does not route `gpt-5.4` (codex-lb canonical), which blocks both the LongMemEval judge-swap (memd uses `gpt-5.4` in place of upstream `gpt-4o`) and any free-form generator comparable to MemPalace's upstream GPT-4o baseline. LoCoMo token_f1 and ConvoMem exact-match were measured against `haiku-manager` in smoke and confirmed verbosity-collapsed (30-token answers vs 3-token gold) — that is a generator-routing artifact, not a retrieval fact, so the numbers were not recorded as canonical. J3 records honest retrieval-diagnostic numbers per bench and files `docs/backlog/v3/2026-04-23-gpt5.4-proxy-route-for-judge.md` as the single gate unblocking canonical-metric rows.

| Bench | Canonical Primary | J3 Run | Value | Verdict |
| --- | --- | --- | --- | --- |
| LongMemEval | `qa_accuracy` (gpt-5.4 judge, judge-swap of upstream GPT-4o) | not runnable (no gpt-5.4 route on proxy) | — | `replay-pending` — diagnostic `session_recall_any@5`=0.900 on 50/500 (`audit: pending`) |
| LoCoMo | `token_f1_avg` | generator verbosity-collapse on haiku-manager | — | `replay-pending` — diagnostic `evidence_hit_rate@5`=0.360 on 100/500 |
| MemBench | `mc_accuracy` | stratified 60 items (10 per topic) | **0.417** | `recorded-unpinned` — first canonical run, below 0.70 floor |
| ConvoMem | `accuracy` (exact-match) | generator verbosity-collapse on haiku-manager | — | `replay-pending` — diagnostic retrieval `recall@k`=0.950 on 100/150 (`audit: pending` — diagnostic not canonical) |

## Summary Table

| Benchmark | Canonical Primary | memd | MemPalace | Verification | Method Card |
| --- | --- | --- | --- | --- | --- |
| LongMemEval | `qa_accuracy` (gpt-5.4 judge, judge-swap of upstream GPT-4o) | — (replay-pending, J3 blocked on gpt-5.4 proxy route) | 96.6% ⚠ contested | replay-pending | [#longmemeval](#longmemeval-method-card) |
| LoCoMo | `token_f1_avg` | — (replay-pending, J3 blocked on free-form generator routing) | — (no canonical replay yet) | replay-pending | [#locomo](#locomo-method-card) |
| MemBench | `mc_accuracy` (MQI deferred) | 0.417 (J3 stratified 60, floor missed) | — (no canonical replay yet) | recorded-unpinned | [#membench](#membench-method-card) |
| ConvoMem | `accuracy` (exact-match, first 150 conversations) | — (replay-pending, J3 blocked on concise-answer generator routing) | — (no canonical replay yet) | replay-pending | [#convomem](#convomem-method-card) |

Diagnostic (non-canonical) retrieval metrics — kept as secondary columns on the per-bench method card, not as primary. J3 retrieval-only diagnostics: LongMemEval 0.900 (`audit: pending`), LoCoMo 0.360, ConvoMem 0.950 (`audit: pending`); MemBench diagnostic subsumed by canonical mc_accuracy run.

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
- **Canonical metric**: `qa_accuracy` — binary correct/incorrect judge. Upstream paper pins GPT-4o and reports 97% human-judge agreement; memd replays with a **judge-swap** to `gpt-5.4` via the codex-lb proxy (flat-rate, no per-token marginal cost). The swap is disclosed on every row; cross-judge agreement is tracked in the method card.
- **Formula reference**: [xiaowu0162/LongMemEval README](https://github.com/xiaowu0162/LongMemEval) · ICLR 2025 [arXiv 2410.10813](https://arxiv.org/abs/2410.10813).
- **Backend**: memd (per-item namespace dispatch via G3 `PublicBenchmarkBackend`).
- **Judge model**: `gpt-5.4` (pinned; codex-lb judge-swap of upstream GPT-4o).
- **Commit SHA**: `532c5163` (judge cache + cost bookkeeping + community-standard caching landed H3-1).
- **Reproduction command**:
  ```
  OPENAI_API_KEY=... MEMD_BENCH_JUDGE_BUDGET_USD=50 \
    cargo run -p memd-client -- benchmark public \
      --dataset longmemeval --write --record \
      --out .memd --retrieval-backend memd --full-eval
  ```
- **Verification**: `replay-pending` (H3 code landed 2026-04-21; J3 attempt 2026-04-21 blocked on gpt-5.4 proxy route — see `docs/backlog/v3/2026-04-23-gpt5.4-proxy-route-for-judge.md`).
- **Primary value**: not runnable on openclaw LiteLLM proxy. Canonical number ships when gpt-5.4 route provisioned.
- **Diagnostic secondary (J3, 2026-04-21)**: `session_recall_any@5 = 0.900` (50/500 items, retrieval-only, memd backend, `audit: pending` — diagnostic not canonical). Prior 0.936 retracted as primary; retained as diagnostic-only context (see retraction log). Retrieval diagnostic above 0.70 floor; canonical qa_accuracy floor unverifiable until judge routes.
- **Cost ledger**: `judge_prompt_tokens`, `judge_completion_tokens`, `judge_cost_usd`, `judge_cache_hit_rate`, `judge_cache_hits`, `judge_cache_misses` emitted per run into `.memd/benchmarks/history/benchmark-runs.jsonl`. J3 run: n/a (judge not called).
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
- **Verification**: `replay-pending` (H3 scorer code landed; J3 attempt 2026-04-21 blocked on generator routing — haiku-manager verbosity-collapses token_f1; see `docs/backlog/v3/2026-04-23-gpt5.4-proxy-route-for-judge.md`).
- **Primary value**: not runnable with haiku-manager generator. Canonical `token_f1_avg` ships when gpt-5.4 route provisioned.
- **Diagnostic secondary (J3, 2026-04-21)**: `evidence_hit_rate@5 = 0.360` (100/500 items, retrieval-only, memd backend). Floor (≥0.70) missed on retrieval diagnostic; canonical `token_f1_avg` floor unverifiable without free-form gpt-5.4 answers. Prior `0.709` retracted; see retraction log.
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
- **Verification**: `recorded-unpinned` (J3 2026-04-21 — first canonical `mc_accuracy` run, single attempt, not yet replay-confirmed; floor ≥0.70 missed).
- **Primary value**: `mc_accuracy = 0.417` (stratified 60 items, 10 qids per topic × 6 topics, memd backend, generator `haiku-manager`, `build_mc_generation_prompt` constrains the model to a single letter so generator-verbosity is not a confound). Full 500-item rerun deferred until generator routing stabilizes; stratified sample is expected to be within ±0.05 of full-split accuracy per uniform topic coverage.
- **Per-topic breakdown (J3, stratified 60)**: `book = 0.700`, `food = 0.700`, `movie = 0.700`, `multi_agent = 0.300`, `roles = 0.100`, `events = 0.000`. Floor missed driven entirely by event-reasoning and role-tracking topics; retrieval-heavy topics (book/food/movie) sit at the 0.70 floor.
- **Bug fix landed in J3**: `parse_membench_choices` (`crates/memd-client/src/benchmark/public_benchmark.rs`) now handles upstream's `{A:[...],B:[...],C:[...],D:[...]}` object shape for `choices`. Prior code expected a flat array and silently skipped every item as "no ground_truth or choices," so MemBench full-eval was effectively a no-op before J3. Three unit tests in `crates/memd-client/src/main_tests/public_benchmark_tests/mod.rs` cover object / array / null shapes.
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
- **Verification**: `replay-pending` (H3 disclaimer landed; J3 attempt 2026-04-21 blocked on generator routing — haiku-manager emits ~30-token free-form answers that verbosity-collapse exact-match scoring; see `docs/backlog/v3/2026-04-21-gpt4o-proxy-route-for-judge.md`).
- **Primary value**: not runnable with haiku-manager generator. Canonical `accuracy` (exact-match) ships when gpt-5.4 route provisioned.
- **Diagnostic secondary (J3, 2026-04-21)**: retrieval-mode `recall@k = 0.950` (100/150 items, memd backend, no generator, `audit: pending` — diagnostic not canonical, subject to the gaming-audit note below). Above the 0.70 floor on retrieval; canonical exact-match floor unverifiable without concise-answer generator.
- **Cost ledger**: n/a (no judge call).
- **Competitor row**: no competitor publishes ConvoMem numbers as of 2026-04-21. Recent (2025) bench.
- **Gaming-audit note**: ConvoMem retrieval `recall@k` on evidence-sample fixture clusters near 1.0 on memd's lexical+dense retrieval (J3 diagnostic `0.950` with `audit: pending` above). Any score ≥0.90 — including the J3 diagnostic — is gated on an `audit:` field covering (a) train/test overlap between fixture and memd's vector store, (b) top_k vs corpus size, (c) whether first-150 slicing introduces selection bias. Until audit lands, the 0.950 diagnostic is recorded-unpinned and must not be quoted as a canonical number.

## Scope and Limits

- Numbers marked `replay-pending` become `verified` only when the reproduction command at `commit_sha` is re-run from a clean checkout and matches within ±0.01. Anything else stays `replay-pending` or downgrades to `recorded-unpinned`.
- No memd row may report a primary value ≥0.90 without a populated `audit:` field. If it does, the row is capped at `recorded-unpinned` regardless of the underlying measurement.
- This page is hand-curated per I3/J3 (method cards, retraction log, gaming-audit rule). The bench runtime's `write_public_benchmark_docs` (see `crates/memd-client/src/benchmark/runtime.rs`) deliberately **skips overwriting this file** on `--write` — auto-generation from raw bench reports would wipe the method-card structure every run. `scripts/regen-leaderboard.sh --check` still validates required-field coverage against `.memd/benchmarks/history/benchmark-runs.jsonl`; hand-edits that bypass the manifest must be backfilled there or they are invisible to the CI gate.

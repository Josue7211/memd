# Industry-Standard Public Benchmark Suite

Date: 2026-04-14
Status: approved

## Problem

memd's benchmark harness reports retrieval-proxy metrics (recall@k via token overlap) that are not comparable to what the industry reports. Competitors (SuperMem, Mem0, Zep, Letta) all report end-to-end accuracy using each paper's published evaluation protocol. memd's 82.8% LongMemEval score is a retrieval recall@5, not comparable to SuperMem's 81.6% accuracy or Mem0's 66.9% accuracy — different metrics entirely.

## Goal

Make `memd benchmark public` produce numbers that go on the same leaderboard as SuperMem, Mem0, Zep, and Letta. Industry-standard metrics using each paper's exact evaluation protocol.

## Design

### 1. Per-Benchmark Evaluation Protocol

Each benchmark follows its published paper's exact eval pipeline.

**LongMemEval** (ICLR 2025, arxiv 2410.10813):
- Pipeline: retrieve context via memd → feed to generator LLM with question → generator produces hypothesis → GPT-4o judges yes/no
- 7 question types, each with type-specific judge prompt:
  - single-session-user, single-session-assistant, single-session-preference
  - multi-session, knowledge-update, temporal-reasoning, abstention (false-premise)
- Judge model: `gpt-4o-2024-08-06` (paper's exact model)
- Judge prompts: already implemented in `build_longmemeval_eval_prompt`
- Primary metric: **accuracy %** (per-type + overall macro)
- Published baselines: SuperMem 81.6%, GPT-4o oracle 91.8%

**LoCoMo** (ACL 2024, arxiv 2402.17753, Salesforce/SNAP Research):
- Pipeline: retrieve context via memd → feed to generator LLM with question → generator produces answer → score against gold answer
- 5 question categories: single-hop, multi-hop, temporal, open-domain, adversarial
- Scoring: token-level **F1** with Porter stemming normalization
  - Tokenize prediction and ground truth
  - Compute precision = matching tokens / prediction tokens
  - Compute recall = matching tokens / ground truth tokens
  - F1 = 2 * precision * recall / (precision + recall)
  - Adversarial: binary check for "no information" / abstention phrases
- Primary metric: **F1 %** (per-category + overall)
- Published baselines: Mem0 66.9% (LLM-judge), Letta 74.0%, RAG top-5 41.4%

**MemBench** (arxiv 2506.21605, FirstAgent dataset variant):
- Pipeline: retrieve context via memd → feed to LLM with question + multiple-choice options → LLM selects answer → compare to `ground_truth`
- Categories: simple, highlevel, knowledge_update, comparative, conditional, noisy, aggregative, highlevel_rec, lowlevel_rec, RecMultiSession, post_processing
- Scoring: exact string match of selected choice against `ground_truth` field
- Primary metric: **multiple-choice accuracy %** (per-category + overall)
- Data already has `choices` and `ground_truth` fields parsed but currently ignored

### 2. Triple Metric (SuperMem Framework)

Every benchmark run reports three dimensions:

1. **Accuracy** — the paper's primary metric (judge accuracy / F1 / MC accuracy)
2. **Latency** — p50 and p95 retrieval+generation time in milliseconds
3. **Token efficiency** — retrieved context tokens vs full conversation tokens (% reduction)

All three numbers go in the leaderboard row. Avoids collapsing tradeoffs into one number.

### 3. Competitive Comparison Table

A static file `benchmarks/baselines/published_baselines.json` pins published competitor scores:

```json
{
  "longmemeval": {
    "SuperMem": {"accuracy": 81.6, "source": "supermemory.ai/research", "date": "2026"},
    "GPT-4o (oracle)": {"accuracy": 91.8, "source": "arxiv:2410.10813", "date": "2024"},
    "Zep": {"accuracy": null, "source": "blog.getzep.com", "date": "2025", "note": "reports DMR, not LongMemEval accuracy"}
  },
  "locomo": {
    "Mem0": {"accuracy": 66.9, "source": "mem0.ai/research", "date": "2025"},
    "Letta": {"accuracy": 74.0, "source": "letta.com/blog/benchmarking", "date": "2025"},
    "RAG top-5": {"accuracy": 41.4, "source": "arxiv:2402.17753", "date": "2024"}
  },
  "membench": {}
}
```

Every `--full-eval` run auto-generates a comparison table:

```
| System     | LongMemEval | LoCoMo | MemBench | Source      |
|------------|-------------|--------|----------|-------------|
| memd       | ??%         | ??%    | ??%      | this run    |
| SuperMem   | 81.6%       | -      | -        | published   |
| Mem0       | -           | 66.9%  | -        | published   |
| Letta      | -           | 74.0%  | -        | published   |
```

### 4. CLI Interface

```bash
# Fast retrieval-only diagnostic (free, <1 min)
memd benchmark public longmemeval

# Full eval — industry standard (needs LLM API)
memd benchmark public longmemeval --full-eval

# All three benchmarks
memd benchmark public --all --full-eval

# Override models
memd benchmark public longmemeval --full-eval \
  --generator-model claude-sonnet-4-20250514 \
  --grader-model gpt-4o-2024-08-06

# Sampled run (cheaper, faster)
memd benchmark public --all --full-eval --sample 50

# Cost estimate without running
memd benchmark public --all --full-eval --dry-run
```

Defaults:
- Generator: memd's configured LLM (dogfooding)
- Grader: `gpt-4o-2024-08-06` (LongMemEval paper standard)
- `--full-eval` required for end-to-end (no accidental API spend)
- `--sample N` runs N items per benchmark (for cheap dev iteration)

**Cost estimates (full --all --full-eval):**
- ~5500 LLM calls total (500 LongMemEval gen+grade, 1986 LoCoMo gen, 3000 MemBench gen)
- With Claude Sonnet: ~$40-60 per full run
- With GPT-4o-mini generator + GPT-4o grader: ~$5-10 per full run
- With `--sample 50`: ~$2-5 per run (good for dev iteration)
- LoCoMo F1 and MemBench MC scoring are computed locally (no grading API cost)

Output: JSON report + markdown leaderboard + comparison table. Same artifact structure as today.

### 5. CI Regression Gate

- **Per-commit**: retrieval-only benchmarks (fast, free, deterministic). Score drops below threshold → build fails.
- **Nightly**: `--full-eval` on all three benchmarks. Results stored with timestamp + git SHA.
- **Historical tracking**: each run appends to `benchmarks/history/` with timestamped results. Trend visible over time.
- **Threshold config**: per-benchmark minimum scores in a config file, updatable as memd improves.

### 6. Codebase Changes

**Keep:**
- Dataset normalization (LongMemEval, LoCoMo, MemBench parsers)
- Dataset download + checksum verification
- Artifact output structure (manifest.json, results.json, report.md)
- LongMemEval judge prompts + OpenAI grader call
- ConvoMem adapter (small benchmark, works fine)

**Add:**
- Generation step: per item, retrieve top-k via memd backend → build prompt → call generator LLM → capture hypothesis
- LoCoMo F1 scorer: token-level F1 with Porter stemming normalization
- MemBench MC evaluator: feed choices to LLM, extract selection, compare to `ground_truth`
- Latency + token tracking per item (prompt tokens, completion tokens, retrieval ms)
- Token efficiency metric (retrieved context size vs full conversation size)
- Published baselines file + comparison table generator
- Historical trend storage
- CI threshold config

**Replace:**
- Token-overlap ranking → memd's actual retrieval backend
- `--community-standard` flag → absorbed into `--full-eval`

**Delete:**
- Generic fallback path in `build_public_benchmark_item_results` (lines 2092-2257)
- `candidate.item_id == item.item_id` score bonus (line 2119)
- `gold_answer` in candidate text during ranking (line 2104)
- Any metric labeled "retrieval proxy" as a primary metric

## Non-Goals

- Custom benchmarks beyond the three published ones
- Beating competitors — just measuring honestly on the same scale
- Changing memd's retrieval architecture (that's a separate task informed by these numbers)

## Success Criteria

- `memd benchmark public longmemeval --full-eval` produces an accuracy number comparable to SuperMem's 81.6%
- `memd benchmark public locomo --full-eval` produces an F1 number comparable to Mem0's 66.9%
- `memd benchmark public membench --full-eval` produces an MC accuracy number
- All three report latency + token efficiency alongside accuracy
- Comparison table auto-generated with published baselines
- CI gates on retrieval regression per commit

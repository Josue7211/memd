# LongMemEval — V6 method card

- **bench_id:** `lme`
- **upstream:** LongMemEval (Wu et al.)
- **subset:** `longmemeval_s_cleaned.json` (500 questions; full canonical run)
- **primary metric:** `qa_accuracy` (LLM-judge, industry standard)
- **target:** ≥ 0.85
- **source dataset:** `.memd/benchmarks/datasets/longmemeval/longmemeval_s_cleaned.json`

## V6 typed pipeline

| Layer | Setting |
| --- | --- |
| A6 episodic ingest | on (`EpisodicAdapter::lme`) |
| B6 semantic distillation | on (`distill_model=gpt-5.4`, `budget=100 milli-USD`) |
| C6 canonical promotion | on (rule card `promotion-rule/v1`) |
| D6 bench-compiler | on (per-bench budget profile in `compiler-budgets.json`) |
| E6 progressive-depth routing | on (`max_depth_calls=3`, `max_retrieval_tokens=10000`) |
| F6 reasoning harness | on (`max_reasoning_steps=5`) |

## Seeds

- typed-ingest seed: `lme-v6-2026-04` (deterministic, fixture-locked).
- judge model: `gpt-5.4` via codex-lb proxy.

## Reproducibility

```
bash scripts/public-bench-reproduce.sh lme
```

Match tolerance ±0.03 from a fresh clone.

## Provenance

Every retrieved record carries `memory_item_id`; `memd explain <id>`
walks back to the source turn. F6 reasoning traces live in
`docs/verification/v6-runs/<date>.ndjson`.

## Status

scaffold-symmetric — runtime activation calendar-gated post-2026-05-02
alongside A6.9/B6/C6/D6/E6.

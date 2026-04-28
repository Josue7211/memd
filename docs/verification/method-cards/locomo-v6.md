# LoCoMo — V6 method card

- **bench_id:** `locomo`
- **upstream:** LoCoMo (Maharana et al.)
- **subset:** `locomo10.json` (full canonical run)
- **primary metric:** `token_f1_avg` (industry standard)
- **target:** ≥ 0.75
- **source dataset:** `.memd/benchmarks/datasets/locomo/locomo10.json`

## V6 typed pipeline

| Layer | Setting |
| --- | --- |
| A6 episodic ingest | on (`EpisodicAdapter::locomo`) |
| B6 semantic distillation | on (`distill_model=gpt-5.4`, `budget=100 milli-USD`) |
| C6 canonical promotion | on (rule card `promotion-rule/v1`) |
| D6 bench-compiler | on |
| E6 progressive-depth routing | on |
| F6 reasoning harness | on (sequential-reasoning subset benefits most) |

## Seeds

- typed-ingest seed: `locomo-v6-2026-04`.
- judge model: `gpt-5.4`.

## Reproducibility

```
bash scripts/public-bench-reproduce.sh locomo
```

Match tolerance ±0.03 from a fresh clone.

## Provenance

Same shape as LME. Multi-hop sequential reasoning steps recorded in
F6 scratchpad NDJSON.

## Status

scaffold-symmetric — runtime activation calendar-gated post-2026-05-02.

# ConvoMem — V6 method card

- **bench_id:** `convomem`
- **upstream:** ConvoMem evidence-sample
- **subset:** `convomem-evidence-sample.json` (525 items)
- **primary metric:** `judge_accuracy` (LLM-judge, industry standard)
- **target:** ≥ 0.90
- **source dataset:** `.memd/benchmarks/datasets/convomem/convomem-evidence-sample.json`

## V6 typed pipeline

| Layer | Setting |
| --- | --- |
| A6 episodic ingest | on (`EpisodicAdapter::convomem`) |
| B6 semantic distillation | on |
| C6 canonical promotion | on |
| D6 bench-compiler | on |
| E6 progressive-depth routing | on |
| F6 reasoning harness | on |

## Seeds

- typed-ingest seed: `convomem-v6-2026-04`.
- judge model: `gpt-5.4`.

## Reproducibility

```
bash scripts/public-bench-reproduce.sh convomem
```

Match tolerance ±0.03 from a fresh clone.

## Provenance

Each judge response references the underlying evidence turn; provenance
chain visible via `memd explain`.

## Status

closed — V6 canonical gate locked at `judge_accuracy=0.910` against target
`0.900`.

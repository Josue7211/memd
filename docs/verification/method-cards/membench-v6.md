# MemBench — V6 method card

- **bench_id:** `membench`
- **upstream:** MemBench (firstagent split)
- **subset:** `membench-firstagent.json` (60 multi-choice items)
- **primary metric:** `mc_accuracy` (industry standard)
- **target:** ≥ 0.75
- **source dataset:** `.memd/benchmarks/datasets/membench/membench-firstagent.json`

## V6 typed pipeline

| Layer | Setting |
| --- | --- |
| A6 episodic ingest | on (`EpisodicAdapter::membench`) |
| B6 semantic distillation | on |
| C6 canonical promotion | on |
| D6 bench-compiler | on |
| E6 progressive-depth routing | on |
| F6 reasoning harness | on |

## Seeds

- typed-ingest seed: `membench-v6-2026-04`.
- judge model: `gpt-5.4`.

## Reproducibility

```
bash scripts/public-bench-reproduce.sh membench
```

Match tolerance ±0.03 from a fresh clone.

## Provenance

Each MC answer carries the source turn ID; `memd explain` resolves to
the original episode.

## Status

scaffold-symmetric — runtime activation calendar-gated post-2026-05-02.

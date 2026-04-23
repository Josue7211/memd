---
phase: G5
name: AdversarialNoise Bench + V5 Completion Gate
version: v5
status: planned
opened: 2026-04-22
depends_on: [A5, B5, C5, D5, E5, F5]
axis: raw_retrieval, trust_provenance
---

# Phase G5: AdversarialNoise Bench + V5 Completion Gate

## Goal

Two jobs. (1) Plant contradictory noise next to canonical facts — memd must surface canonical, not the loudest or most recent. (2) Gate V5: all 7 suites run together, numbers publish to `docs/verification/SUBSTRATE_BENCHMARKS.md`, composite moves 4.0 → ≥5.5.

## Why this phase exists

Adversarial-noise is the honest-robustness test. The gate role is V5's closing contract — numbers in one place, re-runnable, third-party verifiable.

## Deliver

1. **Noise scenario generator.** For each of 50 canonical facts, plant 3 contradicting semantic noise records with slightly-higher recency. Query; canonical must win.
2. **Metrics.** `canonical-wins-rate`, `noise-leak-count`, `tie-break-by-provenance-rate`.
3. **Suite runner.** `memd bench substrate --all` runs A5–G5 in parallel, aggregates into `SUBSTRATE_BENCHMARKS.md`.
4. **Third-party rerun script.** `scripts/substrate-bench-reproduce.sh` — clone, build, run, compare to published numbers ±0.03.
5. **Competitor card.** Template for competitor-run scorecard; `docs/verification/SUBSTRATE_COMPETITOR.md`.
6. **10-STAR regeneration.** G5 writes axis deltas to `docs/verification/MEMD-10-STAR.md`.

## Pass Gate

- pre: composite 4.0 (post-V4); no suite aggregation
- post: canonical-wins ≥ 0.90; noise-leak ≤ 0.05; composite ≥ 5.5
- evidence: `SUBSTRATE_BENCHMARKS.md` regenerated, 10 CI runs, reproducibility script passes
- regression budget: any suite pass-gate miss keeps V5 open

## Product Win

V5 closes with an honest, reproducible, competitor-runnable bench suite. memd's substrate claim is numbers, not prose.

## Evidence

- all 7 suites green in one CI run
- reproducibility script green from a fresh clone
- `SUBSTRATE_BENCHMARKS.md` committed
- 10-STAR composite ≥ 5.5

## Fail Conditions

- Adversarial noise defeats memd: canonical-promotion path weak; file per-axis recovery.
- Reproducibility fails ±0.03: non-determinism in pipeline; root-cause.

## Rollback

N/A — this is the gate.

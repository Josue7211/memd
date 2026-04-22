---
milestone: v5
name: Substrate-Native Benchmark Suite
status: planned
opened: 2026-04-22
depends_on: [v4]
composite_pre: 4.0
composite_target: 5.5
axes_lifted: [cross_harness, trust_provenance, typed_retrieval]
---

# Milestone v5 Audit — Substrate-Native Benchmark Suite

## Goal

Ship memd's own benchmark suite. Open-source, reproducible, runnable by competitors. Public benches (LME, LoCoMo, MemBench, ConvoMem) measure flat RAG-over-transcript; V5 benches measure what memd is actually for — cross-session recall, correction propagation, cross-harness handoff, progressive depth, provenance integrity, typed retrieval, adversarial noise resistance.

## 10-STAR axis targets (pre / post)

| axis | weight | pre | post |
| --- | --- | --- | --- |
| session continuity | 20% | 4 | 5 |
| correction retention | 15% | 4 | 6 |
| procedural reuse | 15% | 3 | 4 |
| cross-harness | 15% | 3 | 6 |
| raw retrieval | 15% | 6 | 7 |
| token efficiency | 10% | 4 | 5 |
| trust + provenance | 10% | 3 | 6 |

composite: 4.0 → 5.5

## Suites

- **A5 CrossSessionRecall** — plant facts in session 1, query in session N, measure recall.
- **B5 CorrectionPropagation** — correct a fact in session 1, assert session N retrieval uses corrected version and provenance shows the correction turn.
- **C5 CrossHarnessContinuity** — write in claude-code, read in codex, round-trip. Truth conserved, visibility honored.
- **D5 ProgressiveDepth** — wake/lookup/resume ladder. Shallow wake gets summary; resume reconstructs task.
- **E5 ProvenanceIntegrity** — every retrieved record carries source. Unsourced record in result set = fail.
- **F5 TypedRetrieval** — query shape routes to right type (episodic vs semantic vs canonical vs candidate). Wrong-type result penalized.
- **G5 AdversarialNoise** — plant wrong facts alongside canonical; memd must surface canonical, not noise.

## Completion gate

All 7 suites runnable via `make bench-substrate`, numbers published in `docs/verification/SUBSTRATE_BENCHMARKS.md`, a second team clones + runs + matches ±0.03 per suite. Competitor (one of mempalace / supermemory / letta / mem0) runs the suite against their product; scorecard published.

## Non-goals

- tuning the suites to make memd look good — suites are honest or they don't ship
- merging substrate benches with public benches — they measure different things

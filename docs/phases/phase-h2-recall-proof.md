---
phase: H2
name: Recall Proof
version: v2
status: pending
depends_on: [G2, D2]
backlog_items: [58, 59, 60]
---

# Phase H2: Recall Proof

## Goal

Prove memd recall changes agent behavior. Benchmark parity with mempalace.

## Deliver

- Working benchmark harness for LongMemEval, LoCoMo, MemBench
- A/B scenario: with memd vs without → different agent output
- Published results with methodology

## Pass Gate

- LongMemEval score ≥ 80% (mempalace: 96.6%)
- LoCoMo score above baseline
- A/B influence test: measurable output difference with recall enabled
- Results reproducible (rerunnable in CI)

## Evidence

- Benchmark results with per-question breakdown
- A/B test methodology and results
- Comparison table vs mempalace

## Fail Conditions

- Score below 50% on any benchmark
- No measurable difference in A/B test

## Rollback

- N/A (measurement only, no code changes unless recall fix needed)

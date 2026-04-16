---
phase: P2
name: Measurement Proof
version: v2
status: pending
depends_on: [J2, O2]
backlog_items: []
---

# Phase P2: Measurement Proof

## Goal

Every quality dimension measured. Token efficiency tracked. Benchmarks automated with regression gates.

## Deliver

- Per-kind token tracking (fact, decision, preference, procedure, status, topology)
- Token efficiency per operation (wake, recall, handoff, working memory)
- Automated benchmark suite (all 4 benchmarks in CI-compatible mode)
- Benchmark regression gate with thresholds
- Measurement results persisted for trend analysis (git SHA indexed)
- Full-eval pipeline run (LLM-graded accuracy)

## Pass Gate

- Token efficiency: per-kind counters reporting for all 6 memory kinds
- Token efficiency: per-operation metrics for wake, recall, handoff, working memory
- LongMemEval >= 80% (regression gate, currently 82.8%)
- All 4 benchmarks run with full-eval pipeline
- Benchmark results stored with git SHA for trend tracking
- Regression gate: any benchmark drop below threshold = block

## Evidence

- Token efficiency report (per-kind, per-operation)
- Benchmark run results (all 4 benchmarks)
- Trend comparison table (current vs. M2 baseline)

## Fail Conditions

- Any benchmark below regression gate threshold
- Token efficiency unmeasured for any memory kind or operation type
- Benchmark results not persisted (no trend tracking)

## Rollback

- Measurement infrastructure is additive, no rollback needed

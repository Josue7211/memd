# No Public Benchmark Parity

- status: `open`
- severity: `high`
- phase: `V2-H2`
- opened: `2026-04-14`
- scope: memd-core

## Problem

LongMemEval, LoCoMo, MemBench are standard benchmarks for memory systems. mempalace reports 96.6%. Datasets exist locally but no working harness to run memd against them. Competitive parity unproven.

## Fix

- Port benchmark harnesses to memd test suite (LongMemEval first)
- Run against locally downloaded datasets
- Report results vs. baseline (mempalace 96.6%)
- Add to phase-H2 acceptance criteria (benchmark validation)

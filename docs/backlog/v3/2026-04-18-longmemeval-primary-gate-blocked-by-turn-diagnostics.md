---
status: closed
severity: high
phase: B3
opened: 2026-04-18
scope: benchmark-harness
---
# LongMemEval Primary Gate Blocked by Turn-Diagnostic Harness Cost

- status: `closed`
- severity: `high`
- phase: `B3`
- opened: `2026-04-18`
- scope: `benchmark-harness`

## Problem

The B3 close-out gate depends on the **500-question intrinsic LongMemEval**
run, but the current harness spends most of its runtime on **turn-level
diagnostic retrieval** even though the primary gate metric is
`session_recall_any@5`.

Measured on 2026-04-18:

- total store operations for the current product-path run: `146,203`
- session-level corpus docs: `23,796`
- turn-level corpus docs: `122,407`

That means the benchmark is paying most of its runtime on secondary
diagnostics, which makes honest B3 verification take far too long and
creates pressure to treat a harness bottleneck like a product-quality
problem.

## Why this matters

- Blocks timely B3 pass/fail evidence even when package tests are green and
  product code is landed.
- Blurs the line between the **primary gate** (session-level recall quality)
  and **secondary diagnostics** (turn-level retrieval analysis).
- Makes repeated verification expensive enough that regressions are slower to
  catch and slower to confirm.

## Fix

- Split LongMemEval retrieval verification into:
  - a **primary gate path** that runs the session-level metric needed for B3
    close-out
  - a **deep diagnostic path** that includes turn-level metrics when explicitly
    requested
- Keep turn-level metrics available, but make them opt-in for full 500-question
  sweeps.
- Document in the phase/verification surfaces which metric is the gate and
  which metrics are diagnostics.
- Preserve enough metadata so deeper diagnostic runs can still be compared
  honestly against primary-gate runs.

## Acceptance

- Full 500-question intrinsic LongMemEval primary-gate run finishes in a
  reasonable developer loop window.
- Default B3 verification path reports the primary session metric without
  silently dropping or relabeling it.
- Turn-level metrics remain available via an explicit diagnostic mode.

## Resolution

Resolved on 2026-04-18 by making turn diagnostics opt-in via
`--turn-diagnostics`. The default 500-question product-path rerun now
finishes in `1468764 ms` (~24.5 min) while still reporting the primary
session metric honestly. Remaining B3 blocker moved from harness cost to
retrieval quality: `session_recall_any@5` is still `0.828`, below the
phase target `≥0.92`.

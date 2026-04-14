# No Token Efficiency Measurement

- status: `open`
- severity: `medium`
- phase: `V2-K2`
- opened: `2026-04-14`
- scope: memd-core

## Problem

Memory can be captured and refined but there's no per-kind token tracking. Total tokens spent on memory operations unmeasured. Delta-only capture not implemented. Cost analysis required to justify memory retention vs. pruning trades.

## Fix

- Add token counter per kind (long-term fact, session artifact, working state)
- Measure token cost of capture, compaction, consolidation
- Implement delta-only capture (append-only changes, not full rewrites)
- Add to phase-K2 acceptance criteria (cost accountability)

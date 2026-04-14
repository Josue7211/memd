# No Overnight Evolution Loop

- status: `open`
- severity: `high`
- phase: `V2-M2`
- opened: `2026-04-14`
- scope: memd-core

## Problem

Dream, autodream, and autoevolve loops enable off-session memory refinement. Loops not implemented. Memory remains static between agent sessions instead of improving through background consolidation and optimization.

## Fix

- Implement dream worker (background memory compaction)
- Implement autodream (periodic deep consolidation)
- Implement autoevolve (improvement loop feedback)
- Add to phase-M2 acceptance criteria (evolution)

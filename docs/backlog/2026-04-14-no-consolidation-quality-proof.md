# No Consolidation Quality Proof

- status: `open`
- severity: `medium`
- phase: `V2-M2`
- opened: `2026-04-14`
- scope: memd-core

## Problem

Consolidation merges fragmented memory items in worker. Output quality unmeasured. No test verifies that consolidated items remain semantically coherent or improve recall.

## Fix

- Add quality scoring post-consolidation
- Test on realistic memory fragmentation
- Measure recall improvement vs. original
- Add to phase-M2 acceptance criteria (consolidation quality)

# No Compaction Quality Proof

- status: `open`
- severity: `medium`
- phase: `V2-K2`
- opened: `2026-04-14`
- scope: memd-core

## Problem

Memory compaction reduces working memory size by deduplicating and coalescing items. Code exists but quality metrics after compaction are unmeasured. No test verifies that compacted state preserves semantic fidelity.

## Fix

- Add quality scoring post-compaction (information retention %)
- Test compaction on realistic memory loads
- Measure token savings vs. semantic loss
- Add to phase-K2 acceptance criteria (space-quality trade)

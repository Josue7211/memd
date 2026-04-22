---
milestone: v10
name: Self-Improvement
status: planned
opened: 2026-04-22
depends_on: [v9]
composite_pre: 9.0
composite_target: 9.5
axes_lifted: [all]
---

# Milestone v10 Audit — Self-Improvement

## Goal

memd improves itself. Overnight consolidation promotes stable truths. Auto-correction from user behavior catches drift without manual correction. Bench regression canary blocks bad merges. 10-STAR scorecard self-scores. Continuous-deployment memory: the memory substrate evolves with the product.

## 10-STAR axis targets (pre / post)

| axis | weight | pre | post |
| --- | --- | --- | --- |
| all 7 axes | 100% | 8-9 | 9+ |

composite: 9.0 → 9.5+

## Phases

- **A10** Consolidation-as-dream — overnight pass merges candidate → canonical, dedups, decays stale.
- **B10** Auto-correction from user behavior — user ignores a suggestion 3x → memd lowers its weight.
- **C10** Memory-driven agentic replay — re-run past sessions with updated canonical, validate consistency.
- **D10** Bench-score regression canary — any phase commit that drops a substrate bench by >0.02 blocks merge.
- **E10** Gap-audit self-scoring — 10-STAR composite computed from live telemetry, not manual grade.
- **F10** Continuous-deployment memory — weekly composite score published, trend line public.

## Completion gate

30-day run:
- composite ≥ 9.0 sustained
- self-improvement loop demonstrated (E10 scorecard moved ≥0.3 in 30 days on at least one axis, without human intervention)
- zero regression across substrate benches
- D10 canary blocked ≥1 bad PR in the 30 days (evidence of working)

## Non-goals

- generalized AGI claims — this milestone is about memory substrate self-improvement, not model self-improvement

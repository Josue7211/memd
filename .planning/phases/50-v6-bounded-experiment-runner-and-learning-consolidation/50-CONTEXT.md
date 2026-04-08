---
phase: "50"
name: "v6 Bounded Experiment Runner and Learning Consolidation"
created: 2026-04-06
---

# Phase 50: v6 Bounded Experiment Runner and Learning Consolidation - Context

## Why This Exists

Phase 49 gave `memd` a deterministic composite gate. Phase 50 should use that
gate to decide which experiments are safe to keep, and then consolidate the
accepted results into durable memory.

## Decisions

- Build on the composite gate instead of inventing another scoring source.
- Keep experiments bounded and reversible.
- Preserve a compact trail of accepted and rejected runs.
- Keep consolidation separate from evaluation so accepted learnings are explicit.

## Discretion Areas

- Whether the first runner works on temp branches, reversible patches, or both.
- How much automated acceptance the runner should attempt before asking for
  manual review.
- What trail format is most useful for later research loops.

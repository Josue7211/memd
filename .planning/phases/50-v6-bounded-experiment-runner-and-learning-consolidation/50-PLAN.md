---
phase: "50"
name: "v6 Bounded Experiment Runner and Learning Consolidation"
created: 2026-04-06
type: plan
status: complete
---

# Phase 50: `v6` Bounded Experiment Runner and Learning Consolidation — Plan

## Goal

Run bounded experiments against the composite gate, keep only winning changes,
and consolidate accepted learnings into durable project memory.

## Plan

1. Add a bounded experiment runner that can evaluate candidate changes.
2. Route acceptance decisions through the composite gate.
3. Record accepted and rejected experiment outcomes in a compact trail.
4. Consolidate accepted results into memory/autodream inputs.
5. Add tests for acceptance, rejection, and trail persistence.

## Verification

- `cargo test -p memd-client`
- experiment trail artifacts are written for accepted and rejected runs

## Result

The project gets a safe keep-or-discard loop for self-improvement candidates.

---
phase: "49"
name: "v6 Composite Scoring and Acceptance Gates"
created: 2026-04-06
---

# Phase 49: v6 Composite Scoring and Acceptance Gates - Context

## Why This Exists

`memd` already has separate eval, scenario, gap, and coordination signals. The
next step is a single composite scorer that combines those signals into an
explicit acceptance gate for self-improvement loops.

## Decisions

- Reuse the existing `eval`, `scenario`, and `coordination` surfaces instead of
  inventing a parallel scoring source.
- Keep weighting explicit and deterministic.
- Keep hard correctness failures separate from softer quality degradations.
- Keep the first slice local to the CLI and persisted artifacts before any
  experiment runner consumes it.

## Discretion Areas

- Exact weighting between correctness, memory quality, coordination quality,
  latency, and bloat.
- Whether the composite command should become the canonical gate for `improve`
  immediately or remain a standalone scorer first.
- Which penalties should fail the gate versus merely lower the score.

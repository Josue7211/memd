---
phase: C5
name: CrossHarnessContinuity Bench
version: v5
status: planned
opened: 2026-04-22
depends_on: [V4]
axis: cross_harness
---

# Phase C5: CrossHarnessContinuity Bench

## Goal

Measure truth conservation across harness boundaries: write a fact in claude-code, read it in codex. Preferences, corrections, canonical facts all round-trip. Visibility scopes honored — project-scope stays project, local-scope stays local.

## Why this phase exists

memd's competitive moat is being one memory backend for any harness. Today the hook bridges exist but there is zero automated test that cross-harness retrieval is lossless.

## Deliver

1. **Scenario generator.** 3 scenarios × 3 harness pairs (claude↔codex, claude↔gemini-cli if present, codex↔gemini-cli).
2. **Harness adapters.** Thin shims that drive each harness through a scripted session via its CLI / API.
3. **Metrics.** `truth-conservation-rate`, `visibility-leak-rate` (should be 0), `round-trip-latency`.
4. **Runner.** `memd bench substrate --suite cross-harness`.
5. **Scorer.** Exact match on fact content + scope string.

## Pass Gate

- pre: no cross-harness bench
- post: truth-conservation ≥ 0.95; visibility-leak = 0 (hard floor); latency p95 ≤ 2s end-to-end
- evidence: results NDJSON for each harness pair + report
- regression budget: visibility-leak regression blocks merge on any value > 0

## Product Win

"any-harness memory" becomes provable.

## Evidence

- harness adapter code
- per-pair result NDJSON
- shared report

## Fail Conditions

- Visibility leak: freeze merges, root-cause. Privacy bug is a release blocker.
- Truth drift: ingest path inconsistency between harnesses.

## Rollback

Bench-only; no runtime behavior change.

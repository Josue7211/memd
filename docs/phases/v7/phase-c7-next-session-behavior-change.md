---
phase: C7
name: Next-Session Behavior Change Test
version: v7
status: planned
opened: 2026-04-22
depends_on: [B7, V5 B5]
axis: correction_retention, session_continuity
plan_spec: docs/phases/v7/phase-c7-plan.md
---

# Phase C7: Next-Session Behavior Change Test

## Goal

Prove a correction in session 1 changes behavior in session 2+. Plant correction in S1 → compact → new session S2 → query → memd returns Y (not X), provenance shows correction turn.

## Why this phase exists

A7 verifies ingest. B7 ships the promotion rule. C7 is the behavior proof — the actual product claim. Without C7, "correction retention" is a number with no anchor.

## Deliver

1. **Dogfood scenario harness.** 3-session scripted scenario with 5 planted corrections. Runs nightly.
2. **Behavior-change metric.** For each correction: `behavior_changed_at_session = N | never`. Aggregate: mean, p95, correction-respected-rate@session=2 / 3 / 5.
3. **Integration with V5 B5.** Feed scenario outputs into B5 CorrectionPropagation suite; B5 gains a `next-session-behavior` sub-metric.
4. **Telemetry.** `.memd/logs/correction-behavior.ndjson` per-correction persistence.

## Pass Gate

- pre: no behavior-change measurement
- post: correction-respected-rate@session=2 ≥ 0.90; @session=5 ≥ 0.85
- evidence: nightly NDJSON + rollup report

## Product Win

memd's core correction claim is numbered and backed by traces.

## Evidence

- nightly runs ≥ 7 days
- rollup report
- B5 delta

## Fail Conditions

- respected-rate@2 < 0.90: compiler priority wrong or promotion threshold too strict; root-cause.
- Any correction honored at session 2 but lost at session 5: decay bug; root-cause.

## Non-Goals

- contradiction detection (D7)
- user-visible surface (F7)

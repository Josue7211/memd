---
phase: E8
name: Diff + Rollback UI
version: v8
status: planned
opened: 2026-04-22
depends_on: [A8, B8, D8, V7 G7]
axis: correction_retention, trust_provenance
plan_spec: docs/phases/v8/phase-e8-plan.md
---

# Phase E8: Diff + Rollback UI

## Goal

See exactly what changed (record diff) and roll back with one click (V7 G7 engine). Diff is line-level for content, field-level for metadata.

## Why this phase exists

V7 G7 shipped rollback CLI. E8 turns it into a visible undo affordance: "oh, that correction was wrong — undo". Without this UI, users stay afraid of corrections.

## Deliver

1. **Diff renderer.** Monaco-based; shows before/after for content + metadata.
2. **Rollback button.** One-click; surfaces confirmation with consequence preview.
3. **Contradiction resolution UI.** D7 `ContradictionReceipt` rendered as side-by-side candidates + accept button.
4. **Bulk undo disabled.** Per V7 non-goal; surface explains.
5. **Audit log entry.** Every UI-initiated rollback writes an audit entry with actor = "ui".

## Pass Gate

- pre: rollback is CLI + confidence-testing
- post: diff + rollback UI covers all V7 G7 paths; 10 E2E flows green
- evidence: playwright suite, audit log sample

## Product Win

Users feel safe correcting memd because they can see the change + undo it visually.

## Evidence

- E2E tests
- audit log sample
- screenshots

## Fail Conditions

- Rollback button active on ineligible records (chain conflicts): disable + explain.
- Diff miscounts lines on mixed line endings: normalize before render.

## Non-Goals

- Rollback multiple corrections at once (out of V8).
- Time-travel restore (V10 C10 overlap).

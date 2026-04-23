---
phase: B8
name: Correction UX
version: v8
status: planned
opened: 2026-04-22
depends_on: [A8, V7]
axis: correction_retention, trust_provenance
plan_spec: docs/phases/v8/phase-b8-plan.md
---

# Phase B8: Correction UX

## Goal

Inline correction capture + live before/after preview. User reads a fact in the atlas, clicks "correct this", types new value, sees the prospective retrieval change before committing.

## Why this phase exists

V7 shipped correction CLI + surfaces. CLI is for power users. B8 turns correction into a first-class UX affordance — the competitive win over mem0/letta.

## Deliver

1. **Correction modal.** Inline editor anchored to atlas node; emits `memd correction propose` via HTTP.
2. **Before/after preview.** Runs retrieval with prospective canonical; shows diff of top-5 results.
3. **Judge live path.** Calls codex-lb judge for confidence estimate; surfaces before commit.
4. **Contradiction inline.** If D7 detector fires pre-commit, modal shows receipt + resolve flow.
5. **Undo button (redundant to V7 G7).** Rollback accessible from atlas node menu.

## Pass Gate

- pre: CLI-only correction
- post: correction modal works; preview accurate; 10 E2E correction flows green
- evidence: playwright suite, screenshot set

## Product Win

"I can correct memd in one click" is the claim that gets memd reviewed favorably.

## Evidence

- E2E tests (10 flows)
- screenshots
- judge-latency numbers

## Fail Conditions

- Preview diverges from actual post-commit retrieval: stale retrieval state — fix before merge.
- Judge call > 3s p95: unusable; surface a "processing" state and investigate budget.

## Non-Goals

- Bulk correction (out of scope).
- Collaborative correction (V9).

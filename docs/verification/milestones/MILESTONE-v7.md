---
milestone: v7
name: Correction + Behavior-Change E2E
status: planned
opened: 2026-04-22
depends_on: [v6]
composite_pre: 7.0
composite_target: 7.8
axes_lifted: [correction_retention, trust_provenance]
---

# Milestone v7 Audit — Correction + Behavior-Change E2E

## Goal

Correction lane works end-to-end. User says "no, X is Y" in session 1 — session 3 uses Y, retrieval surfaces Y not X, provenance shows the correction turn, user can inspect, user can roll back if correction was wrong. Tier 2 gaps 10–17 closed.

## 10-STAR axis targets (pre / post)

| axis | weight | pre | post |
| --- | --- | --- | --- |
| correction retention | 15% | 7 | 9 |
| trust + provenance | 10% | 7 | 8 |
| session continuity | 20% | 6 | 7 |
| other axes | — | stable | stable |

composite: 7.0 → 7.8

## Phases

- **A7** Correction lane ingestion — verify capture path end-to-end.
- **B7** Correction → canonical promotion rule — when does a correction replace canonical?
- **C7** Next-session behavior change test — planted correction honored N sessions later.
- **D7** Contradiction detection — correction conflicts surfaced, not silently merged.
- **E7** Provenance trail — corrected record carries source-turn pointer.
- **F7** "I learned X from Y" surface — user-visible correction log.
- **G7** Rollback — user can undo a correction, provenance preserved.

## Completion gate

V5 B5 CorrectionPropagation suite passes 100%. Rollback suite (new in V7) passes. 3-session claude-code dogfood: 5 corrections planted, all honored, 1 rollback exercised, provenance clean.

## Non-goals

- automatic correction (that's V10)
- cross-user correction (that's V9)

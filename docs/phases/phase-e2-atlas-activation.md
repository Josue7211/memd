---
phase: E2
name: Atlas Activation
version: v2
status: pending
depends_on: [B2, C2]
backlog_items: [44, 51, 52]
---

# Phase E2: Atlas Activation

## Goal

Atlas wakes from dormancy. Memory is navigable: wake → region → node → evidence.

## Deliver

- Atlas wired into resume/wake path (top regions in wake packet)
- Entity links auto-populated from co-occurrence
- Client methods for explore/region/trail
- Wiki link parsing in stored content
- Progressive zoom working: wake → region → entity → raw evidence

## Pass Gate

- `memd explore` returns non-empty regions for a project with 10+ items
- Entity links table has rows (not permanently 0)
- Wake packet includes atlas region hint
- Wiki link `[[entity]]` in stored content creates entity link
- Navigate from wake to raw evidence in ≤ 4 hops

## Evidence

- Atlas region count before/after wiring
- Entity links table row count
- Navigation trace from wake → evidence (logged)
- Wiki link resolution test

## Fail Conditions

- Atlas still unreachable from runtime
- Entity links don't populate
- Navigation dead-ends before evidence

## Rollback

- Revert atlas integration if it bloats wake packet beyond budget

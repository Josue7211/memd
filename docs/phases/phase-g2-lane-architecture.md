---
phase: G2
name: Lane Architecture
version: v2
status: pending
depends_on: [A2, F2]
backlog_items: [38, 40]
---

# Phase G2: Lane Architecture

## Goal

6 starter lanes functional. Auto-activation from task context. Retrieval boosting.

## Deliver

- `lane` field on `memory_items` table (schema migration)
- 6 lane directories with multi-file source material
- Lane auto-detection from working context signals
- Lane-tagged items boosted in working memory retrieval
- Lane tagging at ingest time

## Pass Gate

- Schema has `lane` column on memory_items
- All 6 lanes have source material in `.memd/lanes/<name>/`
- Working on frontend code → design lane activates without explicit request
- Design-lane items rank higher in working memory than untagged items
- New memory stored during design work gets `lane:design` tag automatically

## Evidence

- Schema migration test
- Lane auto-detection test (context → detected lanes)
- Retrieval ranking test: lane-boosted vs unboosted
- Tag-at-ingest test

## Fail Conditions

- Lane migration breaks existing queries
- Auto-detection fires on wrong lane
- Boosting drowns non-lane items

## Rollback

- Revert migration if data loss
- Revert boosting if recall quality drops

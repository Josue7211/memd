---
phase: G2
name: Lane Architecture
version: v2
status: complete
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

## Donor Extraction (from inspiration repos)

- **G2-D1** (mempalace `miner.py`): 4-priority room routing — path component → filename → content keywords (2KB window) → fallback "general". 94-entry folder map → 13 canonical rooms. Zero LLM dependency.
- **G2-D2** (Omegon `minds` table): Layered memory inheritance. Minds have `parent` field. Facts have `layer` field: "project" | "persona" | "working". Query scoped to layer. Maps to memd scope precedence.
- **G2-D3** (Omegon `types.rs`): Section-based fact organization — Architecture, Decisions, Constraints, KnownIssues, PatternsConventions, Specs, RecentWork. Used for context rendering grouping.

See: `docs/theory/2026-04-14-donor-extraction-to-v2-phases.md` for full details.

## Rollback

- Revert migration if data loss
- Revert boosting if recall quality drops

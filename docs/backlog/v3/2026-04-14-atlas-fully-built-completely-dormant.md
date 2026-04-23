---
status: resolved
severity: high
phase: D3
opened: 2026-04-14
resolved: 2026-04-21
scope: unspecified
---
# Atlas Fully Built but Completely Dormant — RESOLVED 2026-04-21

status: resolved
severity: high
phase: Phase I
opened: 2026-04-14
resolved: 2026-04-21
extraction source:
- `mempalace/entity_detector.py`
- `mempalace/knowledge_graph.py`
- `supermemory/packages/memory-graph/*`
- A2 notes: `.memd/lanes/architecture/A2-02-atlas-entity-graph.md`, `.memd/lanes/architecture/A2-08-plugin-packaging-and-graph-ui.md`

## Problem

Atlas has 974 lines of code, 7 routes, region clustering, trail tracking,
entity search with multi-factor scoring, 18 passing tests. Never called
from resume/wake path. Entity links table is permanently empty. No client
methods for atlas queries. CLI `memd explore` exists but is never invoked
in any harness integration.

## Evidence

- atlas.rs: 974 lines, 7 routes
- routes.rs: all 7 mapped
- Tests: 18 pass
- Entity links table: created in schema, never written to
- lib.rs: no atlas client methods
- No harness calls atlas

## Fix

1. Wire atlas into resume path — show relevant regions in wake packet ✓ (E2 — `resume/mod.rs:345` calls `atlas_regions`)
2. Populate entity links table (auto-link from co-occurrence already built) ✓ (live bundle: 285 rows at 2026-04-21)
3. Add client methods for explore/region/trail ✓ (`client::atlas_regions/explore/expand/generate`)
4. Surface atlas navigation in dashboard (Phase I) — deferred to M4 Phase I (not a V3 blocker)

## Resolution 2026-04-21

Empirical audit on `research/mining`:

- `sqlite3 .memd/memd.db "SELECT count(*) FROM memory_entity_links"` → 285 "related" edges. Auto-linker (`auto_link_entity` at `main.rs:281`) fires on every `store_item`. Not dormant in practice.
- `atlas_regions` / `atlas_trails` tables were empty because `generate_regions_for_project` was only invoked lazily from `atlas_explore` (no anchor) and `filter_to_region`, never from the wake-path `GET /atlas/regions`. Fix: lazy-on-empty in `get_atlas_regions` (routes.rs:2398) — same pattern as `filter_to_region` (routes.rs:35-39). Scoped by project+namespace from the caller so buckets don't collide.
- Trails left as explicit-save-only (user curation via `POST /atlas/trail/save`); auto-persistence would change semantics.

Atlas is now reachable from wake with zero extra plumbing. Closing blocker — no standalone phase needed.

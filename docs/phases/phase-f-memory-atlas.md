# Phase F Memory Atlas

<!-- PHASE_STATE
phase: f
status: verified
truth_date: 2026-04-12
version: v1
next_step: none
-->

- status: `verified`
- version: `v1`
- truth date: `2026-04-12`
- next step: none

## Purpose

Let users and agents move from compact wake packets to linked memory regions
and raw evidence without transcript search.

## Done

- theory is defined
- roadmap says this is the next active phase after Phase E audit closure
- schema types: `AtlasRegion`, `AtlasNode`, `AtlasLink`, `AtlasTrail`, `AtlasLinkKind`
- schema request/response: `AtlasRegionsRequest/Response`, `AtlasExploreRequest/Response`
- server: `atlas_regions` table + `atlas_region_members` junction table
- server: region generation from existing memory (kind-based + lane-based bucketing)
- server: explore endpoint with region/node anchor, neighborhood expansion via entity links
- server: cross-dimensional pivot filters (min_trust, pivot_kind, pivot_time, project, namespace)
- server: 3 API routes: `GET /atlas/regions`, `POST /atlas/explore`, `POST /atlas/generate`
- client: `MemdClient` methods for all 3 endpoints
- client: `memd atlas regions/explore/generate/compile` CLI commands
- client: human-readable markdown rendering for regions and explore results
- tests: 7 atlas tests (generate, explore region, explore node, trust pivot, trails, time pivot, lanes)
- trail auto-generation: salience trail (by confidence) and zoom trail (by depth)
- time-based pivot: `pivot_time` filters items by created_at
- Obsidian atlas surface: `memd atlas compile` writes region notes + index to vault
- lane convention documented in setup.md

## Open

None.

## Links

- [[ROADMAP]]
- [[MILESTONE-v1]]
- [[2026-04-11-memd-ralph-roadmap]]
- [[2026-04-11-memd-atlas-theory-lock-v1]]

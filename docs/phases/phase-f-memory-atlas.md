# Phase F Memory Atlas

<!-- PHASE_STATE
phase: f
status: in_progress
truth_date: 2026-04-12
version: v1
next_step: verify progressive zoom and trail generation
-->

- status: `in_progress`
- version: `v1`
- truth date: `2026-04-12`
- next step: verify progressive zoom and trail generation

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
- server: cross-dimensional pivot filters (min_trust, pivot_kind, project, namespace)
- server: 3 API routes: `GET /atlas/regions`, `POST /atlas/explore`, `POST /atlas/generate`
- client: `MemdClient` methods for all 3 endpoints
- client: `memd atlas regions`, `memd atlas explore`, `memd atlas generate` CLI commands
- client: human-readable markdown rendering for regions and explore results
- tests: 4 atlas tests (generate, explore region, explore node, trust pivot filter)

## Open

- trail generation not yet auto-populated (trails struct exists but empty in responses)
- lane-based region generation relies on `lane:` tag prefix convention
- no Obsidian atlas surface yet
- no time-based pivot implemented yet

## Links

- [[ROADMAP]]
- [[MILESTONE-v1]]
- [[2026-04-11-memd-ralph-roadmap]]
- [[2026-04-11-memd-atlas-theory-lock-v1]]

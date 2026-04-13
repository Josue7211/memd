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
- tests: 9 atlas tests (generate, explore region, explore node, trust pivot, trails, time pivot, lanes, expand, evidence count)
- trail auto-generation: salience trail (by confidence) and zoom trail (by depth)
- time-based pivot: `pivot_time` filters items by created_at
- Obsidian atlas surface: `memd atlas compile` writes region notes + index to vault
- lane convention documented in setup.md
- entity_id populated on atlas nodes via store entity resolution
- evidence_count on each node shows raw evidence depth (event spine linkage)
- include_evidence flag drills from nodes to raw MemoryEventRecords
- atlas expand endpoint (`POST /atlas/expand`) for retrieval stage 4 integration
- region rename endpoint (`POST /atlas/rename`) for user curation (D2)
- tag-overlap neighborhood fallback when no entity links exist
- full pivot dimensions: trust, kind, time, scope, provenance, harness, salience (all 7)
- from_working flag auto-seeds explore from current working memory (Status/LiveTruth/Pattern)
- correction-aware neighborhood: supersedes-linked items appear with corrective links
- persisted atlas links table for D3 durable high-value links
- 13 tests, 85 total server tests

## Pass Gate (10-Star Atlas Standard)

- [x] user starts from current task, moves outward naturally (from_working=true)
- [x] agent pulls nearby context without rereads (expand + tag fallback + persisted links)
- [x] moving deeper feels like zooming (wake→region→node→deep dive→evidence)
- [x] truth and source linkage at every depth (entity_id + evidence events + supersedes)

## Locked Decisions Verified

- [x] D1: canonical node = promoted memory object (entity_id linked)
- [x] D2: regions are hybrid (auto-generated, deterministic, user-nameable via rename)
- [x] D3: persist (regions, links, rename), derive (neighborhoods, tag overlap, trails, supersedes)

## Open

None.

## Links

- [[ROADMAP]]
- [[MILESTONE-v1]]
- [[2026-04-11-memd-ralph-roadmap]]
- [[2026-04-11-memd-atlas-theory-lock-v1]]

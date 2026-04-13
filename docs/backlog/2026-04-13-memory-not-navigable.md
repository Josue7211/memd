# Memory Is Not Navigable — Core Product Promise Broken

- status: `deferred-phase-h`
- deferred: `2026-04-13`
- reason: Multi-phase product work — auto-linking from content, atlas from entity clusters, wiki link resolution. Depends on atlas-dormant and lane-architecture-gaps. Core Phase H deliverable.
- found: `2026-04-13`
- scope: memd-server, memd-client, product
- severity: critical

## Summary

memd's product promise is "obsidian hybrid" — navigable, linked, explorable memory.
But memory items are flat text blobs with no structured links. Entity links table is
empty. Atlas is dormant. Wiki link syntax exists in docs but not in the memory layer.
The navigation infrastructure exists (atlas, entities, links, trails) but nothing
connects them. Memory is a flat store, not a graph.

## Symptom

- Memory items have no links to other items
- Entity links table: 0 rows
- Atlas never called from runtime
- `memd explore` not wired to wake/resume
- `[[wikilink]]` syntax in docs but memd doesn't parse or resolve links in content
- No graph navigation from one memory to related memories
- Wake packet shows flat list, not linked neighborhood

## Root Cause

- Memory content is stored as flat `String` — no structured link extraction
- `store_item()` at `crates/memd-server/src/main.rs` creates entities but never links them
- Entity links require explicit `POST /memory/entity/link` — nothing auto-populates
- Atlas explore exists but nothing in the pipeline calls it
- Wiki link syntax `[[path|name]]` not parsed from memory content
- No co-occurrence or citation-based auto-linking
- The product was built bottom-up (storage → retrieval → types) not top-down (navigation → linking → storage)

## Fix Shape

Phase 1 — Auto-link from content:
- Parse `[[wikilink]]` references in memory item content during `store_item()`
- Resolve links to existing items by path, name, or entity key
- Auto-create entity links from resolved references
- Entity co-occurrence: items mentioning the same entities get linked

Phase 2 — Atlas from links:
- Atlas regions auto-generated from entity link clusters (not just tag overlap)
- Wake packet includes linked neighborhood (1-hop from focus item)
- `memd explore` wired into resume as optional depth expansion

Phase 3 — Full obsidian hybrid:
- Wiki link resolution in retrieval (follow links during context building)
- Backlinks surfaced (what links TO this memory)
- Graph view via `memd atlas compile` → obsidian vault
- Trail auto-creation from navigation patterns

## Evidence

- `crates/memd-server/src/store.rs` — `insert_or_get_duplicate()` stores flat content
- `crates/memd-server/src/store_entities.rs` — entities auto-created but no links
- `crates/memd-server/src/atlas.rs` — atlas explore uses tag overlap fallback when no entity links
- `crates/memd-server/src/atlas.rs:722` — `get_entity_links_for_item()` called but returns empty
- Entity links table: `memory_entity_links` has schema, indexes, CRUD methods — 0 rows

## Dependencies

- blocked-by: [[docs/backlog/2026-04-13-atlas-dormant.md|atlas-dormant]] (atlas must be active first)
- blocked-by: [[docs/backlog/2026-04-13-wake-packet-kind-coverage.md|wake-packet-kind-coverage]] (linked items must surface)
- blocked-by: [[docs/backlog/2026-04-13-status-noise-floods-memory.md|status-noise-floods-memory]] (graph useless if nodes are all status)

## Related

- [[docs/audits/2026-04-13-full-codebase-audit.md]] — entity system: "infrastructure built, zero visibility"
- [[docs/theory/locks/2026-04-11-memd-atlas-theory-lock-v1.md]] — "region navigation, progressive zoom, neighborhood expansion"
- [[docs/theory/teardowns/2026-04-11-mempalace-theory-teardown.md]] — "palace graph is navigation, not truth" — memd should be both
- [[docs/theory/models/2026-04-11-memd-10-star-memory-model-v2.md]] — retrieval order: "wake → session → typed → atlas expansion → canonical → raw"
- [[docs/theory/locks/2026-04-11-memd-evaluation-theory-lock-v1.md]] — Journey 5: atlas navigation
- [[docs/verification/MEMD-10-STAR.md]] — pillar 8: compiled knowledge and evidence workspaces
- [[docs/core/obsidian.md]] — obsidian integration docs

---
status: landed
severity: high
phase: B3
opened: 2026-04-23
landed: 2026-04-23
scope: crates/memd-server/src/store.rs, crates/memd-server/src/store_migrations.rs, crates/memd-server/src/main.rs
---
# `list_entities()` Full-Table Scan on Store Hot Path

status: landed
severity: high
phase: B3
opened: 2026-04-23
landed: 2026-04-23

## Problem

`AppState::store_item` calls three helpers per `/memory/store` —
`auto_link_entity`, `create_wiki_links`, `create_named_entity_links`
(`crates/memd-server/src/main.rs:438-577`). Each helper runs
`SqliteStore::list_entities()`
(`crates/memd-server/src/store.rs:1890-1907`), which is
`SELECT payload_json FROM memory_entities ORDER BY updated_at DESC` plus a
per-row JSON deserialize. No index on project (lives inside the JSON
payload) or on aliases (JSON array). Cost grows O(N) per store and 3×N per
helper; bulk ingests (e.g. LongMemEval ~26.5k stores) stall around N=100.

A kill-switch env var `MEMD_STORE_AUTO_LINK_DISABLED=1` (self-documented at
`crates/memd-server/src/main.rs:116-129`) skips all three helpers. Bench
runs opt in via `scripts/bench-server.sh`; product still runs the scans and
pays the quadratic cost.

## Fix (landed)

Schema M5 → M6 via `migrate_memory_entities_indexed_lookups`:
1. Virtual generated column `project_id` on `memory_entities` from
   `json_extract(payload_json, '$.context.project')` (NOTE: path is
   `$.context.project`, not `$.project` — project lives inside the
   `MemoryContextFrame`, see `crates/memd-schema/src/lib.rs:1544-1562`).
2. Index `idx_memory_entities_project_updated` on `(project_id, updated_at DESC)`.
3. Companion table `memory_entity_aliases(entity_id, alias COLLATE NOCASE, project)`
   with indexes on `(project, alias)` and `(alias)`. Backfilled from existing
   `payload_json` rows and kept in sync by `upsert_entity`.

Callers switched (`crates/memd-server/src/main.rs:438-577`):
- `auto_link_entity` → `list_entities_by_project(project, 4)`.
- `create_wiki_links` → `find_entities_by_alias_contains(project, wiki_ref)`.
  Entity-type substring match dropped as noisy.
- `create_named_entity_links` → `find_entities_by_alias_exact(project, mention)`.

`list_entities()` retained for backfill / admin only. Kill-switch
`MEMD_STORE_AUTO_LINK_DISABLED` kept as zero-cost escape hatch.

## Evidence

- Pre-fix self-documenting comment at `crates/memd-server/src/main.rs:116-129`.
- Bench wrapper `scripts/bench-server.sh` (2026-04-23) unblocked LME runs
  before the real fix landed.

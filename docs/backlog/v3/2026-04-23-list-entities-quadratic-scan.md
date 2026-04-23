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

### Scale validation (post-M6, 2026-04-23)

LongMemEval `s_cleaned` `--limit 30 --retrieval-backend memd` with
`MEMD_STORE_AUTO_LINK_DISABLED=0` (real indexed path, fresh bench DB,
auto-link helpers ON). 1482 stores / 500-item haystack sessions, per-100-
store means:

```
[    1-  100] mean= 206.2ms
[  101-  200] mean= 221.9ms
[  201-  300] mean= 238.4ms
[  301-  400] mean= 244.4ms
[  401-  500] mean= 257.5ms
[  501-  600] mean= 251.6ms
[  601-  700] mean= 246.5ms
[  701-  800] mean= 259.4ms
[  801-  900] mean= 200.6ms
[  901- 1000] mean=  92.4ms
[ 1001- 1100] mean=  86.0ms
[ 1101- 1200] mean=  87.6ms
[ 1201- 1300] mean=  90.9ms
[ 1301- 1400] mean=  87.2ms
[ 1401- 1482] mean=  91.0ms
```

Overall: 1482 stores, mean 178.5ms, max 882ms. Drift in windows 1-8 is
session-corpus-size variance, not cost growth; windows 9-15 drop to ~87ms as
smaller-corpus sessions dominate. No quadratic climb over 20× the pre-fix
stall point (N=100 pre-fix → 283ms at N=75; post-fix → 91ms at N=1500).

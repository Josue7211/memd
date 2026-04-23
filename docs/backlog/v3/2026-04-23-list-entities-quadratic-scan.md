---
status: open
severity: high
phase: B3
opened: 2026-04-23
scope: crates/memd-server/src/store.rs, crates/memd-server/src/main.rs
---
# `list_entities()` Full-Table Scan on Store Hot Path

status: open
severity: high
phase: B3
opened: 2026-04-23

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

## Fix

1. Add SQLite columns / indexes:
   - Generated column `project_id` from `json_extract(payload_json, '$.project')` with an index.
   - Alias lookup via a companion table `memory_entity_aliases(entity_id, alias, project)` populated on insert/update, with an index on `(project, alias)`.
2. Replace the three `list_entities()` callers with targeted lookups:
   - `auto_link_entity` → project-scoped top-3 by `updated_at`.
   - `create_wiki_links` → alias-indexed lookup per wiki token.
   - `create_named_entity_links` → alias-indexed lookup per detected entity.
3. Delete the kill-switch once the new path is green on bench and product
   paths; keep `list_entities()` for backfill / admin only.

## Evidence

- Self-documenting comment at `crates/memd-server/src/main.rs:116-129`.
- Bench wrapper landed at `scripts/bench-server.sh` (2026-04-23) as the
  temporary unblock.

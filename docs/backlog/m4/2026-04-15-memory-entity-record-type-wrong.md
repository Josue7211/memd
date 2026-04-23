---
status: open
severity: high
phase: I2
opened: 2026-04-15
scope: unspecified
---
# MemoryEntityRecord Frontend Type Does Not Match Server

status: open
severity: high
phase: Phase I2
opened: 2026-04-15

## Problem

The frontend `MemoryEntityRecord` type has 7 fields. The server struct has 16 fields.
The fields that DO exist have different names. This causes every component that renders
entity data to show wrong or missing information.

Frontend type (apps/dashboard/app/lib/types.ts:100-108):
```ts
{ id, name, kind?, project?, namespace?, created_at, updated_at }
```

Server type (crates/memd-schema/src/lib.rs:1346-1363):
```rust
{ id, entity_type, aliases, current_state, state_version, confidence,
  salience_score, rehearsal_count, created_at, updated_at, last_accessed_at,
  last_seen_at, valid_from, valid_to, tags, context }
```

Key mismatches:
- Frontend has `name` — server has no `name` field, uses `aliases[]`
- Frontend has `kind` — server has `entity_type`
- Server has `current_state`, `confidence`, `salience_score`, `tags` — frontend missing all of these

## Evidence

- `apps/dashboard/app/lib/types.ts:100-108`: wrong MemoryEntityRecord
- `crates/memd-schema/src/lib.rs:1346-1363`: real MemoryEntityRecord
- `apps/dashboard/app/routes/graph.tsx:169`: uses `entity.name` (doesn't exist on server response)
- `apps/dashboard/app/routes/graph.tsx:171`: uses `entity.kind` (should be entity_type)
- `apps/dashboard/app/components/graph/entity-detail.tsx`: renders entity.name/kind

## Fix

1. Replace frontend MemoryEntityRecord with full server shape
2. Update graph.tsx buildEntityGraph to use `aliases[0] ?? id.slice(0,8)` for display name
3. Update entity-detail.tsx to use entity_type, aliases, current_state
4. Update force-graph-wrapper.tsx GraphNode type if it references entity fields

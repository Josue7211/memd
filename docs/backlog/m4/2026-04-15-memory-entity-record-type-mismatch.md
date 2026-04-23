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

The frontend `MemoryEntityRecord` type has 7 fields. The server struct has 17 fields
with completely different names. The frontend uses `name` and `kind` — neither exists
on the server. The server uses `entity_type` and `aliases[]`.

Frontend (apps/dashboard/app/lib/types.ts:100-108):
```ts
{ id, name, kind?, project?, namespace?, created_at, updated_at }
```

Server (crates/memd-schema/src/lib.rs:1346-1363):
```rust
{ id, entity_type, aliases, current_state, state_version, confidence,
  salience_score, rehearsal_count, created_at, updated_at, last_accessed_at,
  last_seen_at, valid_from, valid_to, tags, context }
```

Every component that displays entity data renders blank or wrong values.

## Evidence

- `apps/dashboard/app/routes/graph.tsx:170`: uses `entity?.name` — field doesn't exist
- `apps/dashboard/app/routes/graph.tsx:171`: uses `entity?.kind` — field doesn't exist
- `apps/dashboard/app/components/graph/entity-detail.tsx`: displays entity.name, entity.kind
- curl /memory/entity/search returns `entity_type`, `aliases`, no `name` or `kind`

## Fix

1. Update `MemoryEntityRecord` in types.ts to match server schema exactly
2. Update all consumers to use `entity_type` instead of `kind`
3. Use `aliases[0]` or derive display name from aliases for `name`
4. Add missing fields: confidence, salience_score, tags, current_state, etc.

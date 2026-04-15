# Graph Page Crash — EntitySearchResponse Type Mismatch

status: open
severity: high
phase: Phase I2
opened: 2026-04-15

## Problem

Graph page (/dashboard/graph) crashes with "Cannot read properties of undefined
(reading '0')" when user types any search query. The frontend TypeScript type
`EntitySearchResponse` says `{ entities: MemoryEntityRecord[] }` but the actual
API at /memory/entity/search returns a completely different shape:

```json
{
  "route": "all",
  "intent": "general",
  "query": "caveman",
  "best_match": { "entity": {...}, "score": 1.0, "reasons": [...] },
  "candidates": [{ "entity": {...}, "score": 1.0, "reasons": [...] }],
  "ambiguous": true
}
```

The graph page accesses `search.data?.entities[0]?.id` which is undefined because
there is no `entities` field.

## Evidence

- `apps/dashboard/app/lib/types.ts:400-402`: EntitySearchResponse = `{ entities: MemoryEntityRecord[] }`
- `crates/memd-schema/src/lib.rs:1439-1446`: real EntitySearchResponse = `{ route, intent, query, best_match, candidates, ambiguous }`
- `apps/dashboard/app/routes/graph.tsx:35-38`: accesses `search.data?.entities[0]?.id`
- Browser screenshot: red error "Cannot read properties of undefined (reading '0')"

## Fix

1. Update `EntitySearchResponse` in types.ts to match server schema (candidates[], best_match, etc.)
2. Add `EntitySearchHit` type: `{ entity: MemoryEntityRecord, score: number, reasons: string[] }`
3. Update graph.tsx to extract entities from `candidates[].entity`
4. Update any other consumers of EntitySearchResponse

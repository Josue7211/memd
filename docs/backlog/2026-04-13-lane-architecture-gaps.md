# Lane Architecture Gaps

- status: `deferred-phase-h`
- deferred: `2026-04-13`
- reason: Lane architecture requires schema migration (lane field on memory_items), auto-detection from working context, and retrieval boosting. Product-level work, not hardening.
- found: `2026-04-13`
- scope: memd-client, memd-server, memd-schema

## Summary

Lane theory lock defines lanes as atlas tags on memory items with automatic
activation. Implementation is grep-over-markdown-files. Theory and code fully
divergent. 5 of 6 starter lanes completely missing.

## Symptom

- `memd inspiration --query "..."` greps raw files instead of querying the DB
- `INSPIRATION_FILES` constant lists 4 of 6 files — misses `INSPIRATION-CLAUDE-CODE.md`
  and `INSPIRATION-DOCTRINE.md` (never searched)
- `lane_id` exists on hive sessions but NOT on `memory_items` — can't tag a memory
- No lane auto-activation, no lane retrieval boosting, no lane tagging on ingest
- 5 starter lanes (Design, Architecture, Research, Workflow, Preference) have zero code

## Root Cause

- `inspiration_search.rs:105-110` hardcodes 4 file paths and does substring match
- Theory lock says "source files are ingested on `memd wake`" — never implemented
- Schema has `lane_id` on `HiveSessionRecord` but not on `MemoryItem`
- Atlas `lane` column exists but `generate_regions_for_project()` never sets it

## Fix Shape

1. Add `lane` field to `memory_items` table (schema migration)
2. Implement lane auto-detection from working context signals
3. Tag memory items with lane at ingest time
4. Boost lane-tagged items in working memory retrieval
5. Ingest inspiration source files into DB as lane-tagged items
6. Fix `INSPIRATION_FILES` constant to include all 6 files
7. Seed remaining 5 lanes (even if empty initially)

## Evidence

- [[docs/theory/locks/2026-04-13-memd-lane-theory-lock-v1]] — canonical theory
- `inspiration_search.rs:105-110` — hardcoded file list
- `store.rs:2064-2065` — `lane_id` on hive sessions only

# Atlas System Dormant — Never Called from Runtime

- status: `deferred-phase-h`
- deferred: `2026-04-13`
- reason: Atlas integration is product architecture, not a hardening fix. Requires wiring into wake/working pipeline, entity link auto-population, and cross-session navigation — all Phase H scope.
- found: `2026-04-13`
- scope: memd-server, memd-client
- severity: high

## Summary

Phase F Memory Atlas is fully implemented (7 routes, regions, trails, explore, expand,
rename, generate). Never called from the dogfood loop. Not in wake packets, not in
context retrieval, not in working memory. Entities are auto-created on every item but
never surfaced to users/agents. Entity links table is permanently empty.

## Symptom

- `memd atlas regions` works but returns auto-generated regions nobody uses
- Wake packet has no atlas section
- Entity search works but nothing in the pipeline calls it
- Entity links designed as atlas backbone but nothing populates them

## Root Cause

- Atlas was built as Phase F deliverable, verified via unit tests
- No integration point in resume/wake/working pipeline
- Entity links require explicit creation via `POST /memory/entity/link` — nothing calls this
- Atlas explore requires explicit requests — nothing in auto-pipeline calls it
- Phase F pass gate verified code existence, not runtime integration

## Fix Shape

- Wire atlas regions into working memory response (summary of active regions)
- Add top-region hint to wake packet (1 line: "atlas: {region_name} ({node_count} items)")
- Auto-populate entity links from entity co-occurrence in events
- Or: defer full atlas integration to Phase H where hive makes cross-session navigation real

## Evidence

- `crates/memd-server/src/atlas.rs` — 974 lines, fully implemented
- `crates/memd-server/src/main.rs:415-421` — 7 atlas routes registered
- `crates/memd-client/src/runtime/resume/wakeup.rs` — no atlas section
- `crates/memd-server/src/working/mod.rs` — no atlas in working memory
- Entity links table: 0 rows in production

## Dependencies

- blocked-by: [[docs/backlog/2026-04-13-status-noise-floods-memory.md|status-noise-floods-memory]] (atlas can't surface if working memory is all status)
- blocked-by: [[docs/backlog/2026-04-13-wake-packet-kind-coverage.md|wake-packet-kind-coverage]] (atlas hints must surface in wake)
- can defer to Phase H if hive makes navigation real

## Related

- [[docs/audits/2026-04-13-full-codebase-audit.md]] — full audit findings
- [[docs/theory/locks/2026-04-11-memd-atlas-theory-lock-v1.md]] — atlas theory: navigation, not truth
- [[docs/phases/phase-f-memory-atlas.md]] — Phase F pass gate claims "user starts from current task, moves outward"
- [[docs/theory/teardowns/2026-04-11-mempalace-theory-teardown.md]] — "palace graph is navigation, not truth"

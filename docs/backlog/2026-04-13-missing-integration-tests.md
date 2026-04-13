# Missing Integration Tests Across Multiple Subsystems

- status: `closed`
- closed: `2026-04-13`
- resolution: Route coverage now at 100%. 12 new integration tests cover coordination, skill-policy, tasks, claims, maintenance, workspace, profile, verify, and runtime maintain routes. Total: 114 tests.
- found: `2026-04-13`
- scope: memd-server
- severity: medium

## Summary

Consolidation, decay, workspace memory, and source memory have zero integration
tests. Runtime maintain flow untested. 15 of 72 API routes (21%) untested.
Complex scoring formulas (trust score, entity search, decay algorithm) have no
edge-case coverage.

## Symptom

- No test verifies consolidation produces correct canonical items
- No test verifies decay formula under boundary conditions
- No test verifies workspace aggregation correctness
- No test verifies runtime maintain orchestrates correctly
- Scoring bugs could silently degrade retrieval quality

## Root Cause

- Phase velocity prioritized feature shipping over test coverage
- Server tests cover memory store/retrieve/promote/expire and hive coordination
- Maintenance and analytics subsystems were wired last and tests deferred
- 98 server tests exist but concentrated in core memory + hive paths

## Fix Shape

- Add integration tests for:
  - `POST /memory/maintenance/consolidate` — verify canonical items created from events
  - `POST /memory/maintenance/decay` — verify salience reduction, rehearsal resistance
  - `GET /memory/workspaces` — verify aggregation, trust scoring, filtering
  - `GET /memory/source` — verify source trust formula, aggregation
  - `POST /runtime/maintain` — verify mode switching, apply flag, receipt creation
  - Remaining 15 untested routes (mostly coordination/tasks)
- Target: 100% route coverage, every scoring formula boundary-tested

## Evidence

- `crates/memd-server/src/tests/mod.rs` — 98 tests, concentrated in core memory + hive
- `docs/backlog/2026-04-13-untested-api-routes.md` — 15/72 routes untested
- Untested scoring: `source_trust_score()`, `working_item_priority()`, `decay_entities()`

## Dependencies

- independent: can be written in parallel with all other fixes
- blocked-by: nothing

## Related

- [[docs/audits/2026-04-13-full-codebase-audit.md]] — full audit findings
- [[docs/backlog/2026-04-13-untested-api-routes.md|untested-api-routes]] — overlapping scope (15/72 routes)
- [[docs/verification/MEMD-10-STAR.md]] — pillar 1: core memory correctness requires regression tests

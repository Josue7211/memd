# Phase G Procedural Learning

<!-- PHASE_STATE
phase: g
status: verified
truth_date: 2026-04-12
version: v1
next_step: none
-->

- status: `verified`
- version: `v1`
- truth date: `2026-04-12`
- next step: none

## Purpose

Memory learns how to operate, not just what is true.

Repeated successful workflows should be captured, promoted, and reused so
future sessions stop re-deriving the same procedures.

## Theory Lock Requirements

From `memd-theory-lock-v1.md`:

- Procedural memory stores workflows, learned routines, recovery patterns,
  and user/repo operating preferences
- Procedures can be promoted from candidate-procedural to canonical
- Procedural retrieval is a native retrieval kind (not semantic search)

From `memd-canonical-theory-synthesis.md`:

- Repeated successful workflows can be promoted and reused
- Future sessions stop re-deriving the same procedure
- Procedures exist only as docs with no runtime effect = FAIL

## Deliver

1. **Procedural store model** — `procedures` DB table with trigger,
   steps, success criteria, source provenance, use_count, confidence
2. **Procedure extraction** — auto-detect from episodic event spine
3. **Promotion pipeline** — candidate → promoted with confidence boost,
   use_count tracking, manual promote endpoint
4. **Procedural retrieval** — keyword match against context with
   use_count/confidence scoring, promoted-only by default
5. **Procedure application** — wake packet compiler includes matched procedures
6. **Recovery patterns** — `ProcedureKind::Recovery` as first-class kind
7. **Procedure retirement** — retire endpoint, retired excluded from match
8. **Cross-session tracking** — session_count, last_session on each procedure

## Pass Gate

- [x] repeated successful workflows can be promoted and reused
- [x] future sessions stop re-deriving the same procedure (auto-detect + promote + wake integration)
- [x] procedural retrieval returns relevant procedures for current task
- [x] procedures have runtime effect (working memory auto-matches, wake packet surfaces)

## Pass Gate Evidence

- record → promote → match: test `match_procedures_returns_promoted_only`
- auto-detect from event spine: test `detect_procedures_from_events`
- duplicate detection prevention: test verifies re-detect creates 0 new
- cross-session tracking: test `use_procedure_tracks_sessions` (session_count increments on new sessions)
- retirement: test `retire_procedure_sets_retired_status` (promoted → retired → invisible in match)
- wake integration: working memory builds context string, matches procedures, surfaces in wake packet
- 9 procedural tests, 98 total server tests, 529 workspace tests

## Fail Conditions

- procedures exist only as docs with no runtime effect
- procedural promotion that causes bad automation or brittle habits

## Rollback

- revert procedural promotion that causes bad automation or brittle habits

## Schema

- `Procedure` with id, name, description, kind, status, trigger, steps, success_criteria,
  source_ids, project, namespace, use_count, confidence, session_count, last_session, tags
- `ProcedureKind`: Workflow, Policy, Recovery
- `ProcedureStatus`: Candidate, Promoted, Retired
- 9 request/response type pairs (list, record, match, promote, use, retire, detect)

## API Routes (7)

- `GET /procedures` — list with filters (kind, status, project, namespace)
- `POST /procedures/record` — explicit capture
- `POST /procedures/match` — context-aware retrieval (promoted only)
- `POST /procedures/promote` — candidate → promoted
- `POST /procedures/use` — record use with session tracking
- `POST /procedures/retire` — mark retired
- `POST /procedures/detect` — auto-detect from episodic event patterns

## CLI

`memd procedure list|record|match|promote|use|retire|detect`

## Design Answers

1. Procedure record = name + trigger + steps + success_criteria + source_ids + tags
2. Detection = scan event spine for entities with repeated events, extract summaries as steps
3. Promotion evidence = manual promote (use_count/session_count inform human decision)
4. Wake integration = working memory context string matched against promoted procedures
5. Recovery patterns = ProcedureKind::Recovery, same store and lifecycle

## Open

None.

## Links

- [[ROADMAP]]
- [[MILESTONE-v1]]
- [[2026-04-11-memd-ralph-roadmap]]
- [[2026-04-11-memd-theory-lock-v1]]
- [[2026-04-11-memd-canonical-theory-synthesis]]

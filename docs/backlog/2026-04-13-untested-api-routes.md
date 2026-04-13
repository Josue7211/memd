# 15 Untested API Routes

- status: `closed`
- closed: `2026-04-13`
- resolution: 12 integration tests added covering all 15 previously untested routes
- found: `2026-04-13`
- scope: memd-server

## Summary

15 of 72 API routes (21%) have no test coverage. Most are in coordination,
skill-policy, and tasks — exactly the areas Phase H will expand.

## Symptom

- No tests catch regressions on these routes
- Phase H will add more coordination routes on top of untested ones

## Root Cause

- Routes were added incrementally without matching test coverage
- Skill-policy and task routes are newer Phase G/H additions

## Untested Routes

- `POST /memory/verify` — delegates to tested repair_item
- `GET/POST /memory/profile` — agent profile CRUD
- `GET /memory/workspaces` — workspace memory
- `GET /memory/policy` — stateless policy snapshot
- `GET /coordination/inbox` — coordination inbox
- `GET /coordination/receipts` — receipt listing
- `POST/GET /coordination/skill-policy/apply` — skill policy
- `GET /coordination/skill-policy/activations` — skill activations
- `POST /coordination/claims/transfer` — claim transfer
- `POST /coordination/claims/recover` — claim recovery
- `POST /coordination/tasks/upsert` — task upsert
- `POST /coordination/tasks/assign` — task assign
- `GET /coordination/tasks` — task listing
- `POST /runtime/maintain` — runtime maintenance

## Fix Shape

- Write integration tests for each route before Phase H adds more
- Priority: coordination/tasks routes (Phase H builds on these)

## Evidence

- 72 routes in `main.rs:312-420`
- 34 integration tests in `tests/mod.rs`
- 12 procedural unit tests
- 36+ store unit tests

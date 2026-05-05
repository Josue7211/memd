---
phase: A9
name: Per-User Harness State Isolation
version: v9
kind: implementation-plan
status: ready-to-execute
opened: 2026-05-05
depends_on: [V8, docs/contracts/federated-memory-visibility.md]
phase_doc: docs/phases/v9/V9-INTEGRATION.md
granularity: "one step = <=1 agent session; TDD; commit per task"
axis: session_continuity
---

# Phase A9 - Implementation Plan

## 0. Executive Summary

Add the substrate identity layer V9 depends on: per-user harness state, immutable
agent authorship, and shared multi-user fixtures. A9 does not close V9 by
itself; it creates the state boundary B9..G9 must enforce.

## 1. Surface Area

### Files To Create

| Path | Responsibility |
| --- | --- |
| `docs/contracts/multi-user-harness-state.md` | User/session/agent identity contract for wake, lookup, and handoff. |
| `crates/memd-client/fixtures/shared/multi-user/ua-ub-ua-3session.jsonl` | A -> B -> A focus fixture. |
| `crates/memd-client/fixtures/shared/multi-user/flip-ua-ub-ua.jsonl` | Shared fixture seed used by C9 and G9. |
| `crates/memd-server/src/store_tests/multi_user.rs` | Store-level isolation tests. |

### Files To Modify

| Path | Change |
| --- | --- |
| `crates/memd-schema/src/lib.rs` | Add explicit user identity fields only where needed by runtime requests and persisted memory records. |
| `crates/memd-server/src/store.rs` | Persist/backfill user identity and session ordering metadata. |
| `crates/memd-server/src/store_migrations.rs` | Idempotent migration for new V9 identity columns and indexes. |
| `crates/memd-server/src/helpers.rs` | Filter wake/context/search by user identity before ranking. |
| `crates/memd-server/src/working/mod.rs` | Keep working memory scoped to the requesting user. |
| `crates/memd-server/src/store_tests/mod.rs` | Wire A9 tests. |

## 2. Schema Changes

Required locks from `V9-INTEGRATION.md`:

| Table | Change | Rule |
| --- | --- | --- |
| `memory_items` | `user_id TEXT` | Bound to source user when present; used before agent fallback. |
| `memory_items` | `harness_preset TEXT` | Records caller harness; immutable post-insert. |
| `memory_items` | `user_id_session_seq INTEGER` | Monotonic order per `(user_id, source_agent)` when available. |

Indexes:

```sql
CREATE INDEX IF NOT EXISTS idx_memory_user_session
  ON memory_items(user_id, source_agent, user_id_session_seq);
CREATE INDEX IF NOT EXISTS idx_memory_harness_preset
  ON memory_items(harness_preset);
```

Backfill:

- Existing rows get `user_id = source_agent` only when no better user field
  exists. This preserves current private visibility semantics.
- Existing rows get `harness_preset = source_system` when source system is a
  known harness; otherwise `legacy`.
- Existing rows get `user_id_session_seq = 0`.

## 3. API Shape

Runtime request structs accept optional `user_id` and `harness_preset` without
breaking old callers. When absent, old agent-based behavior remains.

Private visibility rule after A9:

1. If item has `user_id`, requester must have the same `user_id`.
2. Else fallback to existing `source_agent` ownership check.
3. Workspace/Public behavior is unchanged until B9 adds workspace leak guards.

## 4. Test Matrix

1. `a9_migration_adds_identity_columns_idempotently`
2. `a9_store_backfills_legacy_user_identity`
3. `a9_private_item_visible_to_same_user_different_session`
4. `a9_private_item_hidden_from_other_user_same_agent_name`
5. `a9_working_memory_filters_user_b_from_user_a_focus`
6. `a9_search_filters_user_private_items_before_ranking`
7. `a9_harness_preset_persists_on_insert`
8. `a9_identity_columns_survive_store_update`

## 5. Fixtures

- `ua-ub-ua-3session.jsonl` has 15 turns: user A session, user B session, user A
  return session.
- `flip-ua-ub-ua.jsonl` carries the shared correction and focus records C9 uses
  for the multi-harness flip proof.

## 6. Telemetry

A9 writes no final proof packet. It must leave enough structured test output for
G9 to cite in the adversarial suite.

## 7. Feature Flags

None. Identity fields are backward-compatible and always on.

## 8. Task List

### Task A9.1 - identity contract

- [ ] Write `docs/contracts/multi-user-harness-state.md`.
- [ ] Commit: `docs(v9): define multi-user harness state contract (A9)`.

### Task A9.2 - schema migration

- [ ] Tests 1 + 2 failing first.
- [ ] Add columns, indexes, and backfill.
- [ ] Commit: `feat(server/a9): add user-scoped identity columns (A9)`.

### Task A9.3 - request identity plumbing

- [ ] Tests 3 + 4 failing first.
- [ ] Thread optional `user_id` / `harness_preset` through schema and store.
- [ ] Commit: `feat(schema/a9): carry user and harness identity (A9)`.

### Task A9.4 - retrieval isolation

- [ ] Tests 5 + 6 failing first.
- [ ] Filter private user state before context/search/working ranking.
- [ ] Commit: `feat(retrieval/a9): isolate private user state (A9)`.

### Task A9.5 - shared fixtures

- [ ] Add `ua-ub-ua-3session.jsonl` and `flip-ua-ub-ua.jsonl`.
- [ ] Commit: `test(fixtures/a9): add multi-user round-trip fixtures (A9)`.

### Task A9.6 - phase verification

- [ ] Run `cargo fmt --check`.
- [ ] Run `cargo test -p memd-server a9 -- --nocapture`.
- [ ] Run `cargo test -p memd-client multi_user -- --nocapture` if client fixtures are wired.
- [ ] Run `git diff --check`.
- [ ] Commit: `test(a9): verify per-user harness isolation (A9)`.

## 9. Bench Impact

A9 unlocks SC +1, but credit is only awarded after C9/G9 round-trip proof.

## 10. Dependency Graph

- Requires: V8 closed internal.
- Blocks: B9, C9, D9, E9, F9, G9.

## Exit Criteria

1. Identity migration is idempotent.
2. Private memory cannot bleed across users.
3. Working memory focus is user-scoped.
4. Shared fixtures exist for later phases.
5. Tests pass and commits are atomic.

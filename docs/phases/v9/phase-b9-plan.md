---
phase: B9
name: Visibility/ACL Honored by Retrieval
version: v9
kind: implementation-plan
status: ready-to-execute
opened: 2026-05-05
depends_on: [A9, docs/contracts/federated-memory-visibility.md]
phase_doc: docs/phases/v9/V9-INTEGRATION.md
granularity: "one step = <=1 agent session; TDD; commit per task"
axis: cross_harness
---

# Phase B9 - Implementation Plan

## 0. Executive Summary

Make retrieval enforce federated visibility before ranking, FTS, dense recall, or
working-memory admission can see a forbidden item. B9 owns the read/write
negative controls C9 and G9 depend on.

## 1. Surface Area

### Files To Create

| Path | Responsibility |
| --- | --- |
| `crates/memd-server/src/visibility.rs` | Central visibility/ACL decision helpers. |
| `crates/memd-server/src/store_tests/visibility_acl.rs` | B9 server tests. |
| `docs/verification/v9-runs/b9-visibility-audit.md` | Phase proof summary. |

### Files To Modify

| Path | Change |
| --- | --- |
| `crates/memd-server/src/helpers.rs` | Use central ACL filter in context/search. |
| `crates/memd-server/src/routes.rs` | Apply ACL to FTS/dense neighbor candidates. |
| `crates/memd-server/src/working/mod.rs` | Apply same ACL in working memory. |
| `crates/memd-server/src/store.rs` | Add write-path validation for caller identity. |
| `crates/memd-schema/src/lib.rs` | Add caller identity fields to retrieval/store requests if A9 has not already done so. |

## 2. Schema Changes

No new tables beyond A9. B9 may add typed request fields only if A9 left them
out.

## 3. API Shape

Central predicate:

```rust
fn visibility_decision(caller: &CallerIdentity, item: &MemoryItem) -> VisibilityDecision
```

Rules:

- Private: same `user_id` required; fallback to same `source_agent` for legacy
  rows.
- Workspace: caller workspace must equal item workspace.
- Public: visible only inside allowed project/namespace scope.
- Write as other user/agent: deny unless an explicit trusted operator path
  exists. No trusted path is introduced in B9.

## 4. Test Matrix

1. `b9_private_memory_hidden_from_other_user_search`
2. `b9_private_memory_hidden_from_other_user_context`
3. `b9_private_memory_hidden_from_other_user_working`
4. `b9_workspace_memory_does_not_cross_workspace_search`
5. `b9_workspace_memory_does_not_cross_workspace_context`
6. `b9_project_scope_respects_project_boundary`
7. `b9_fts_hit_dropped_when_acl_forbids_item`
8. `b9_dense_hit_dropped_when_acl_forbids_item`
9. `b9_write_as_other_agent_rejected`
10. `b9_visibility_audit_logs_denials`

## 5. Fixtures

Reuses A9 `ua-ub-ua-3session.jsonl`. Adds phase-local denial cases under
`crates/memd-client/fixtures/v9/b9/`.

## 6. Telemetry

Append denial receipts to `.memd/logs/visibility-audit.ndjson`:

```json
{"phase":"B9","decision":"deny","reason":"workspace_mismatch","item_id":"..."}
```

The audit log is proof support only; tests assert behavior.

## 7. Feature Flags

None. Contract enforcement is always on.

## 8. Task List

### Task B9.1 - central ACL predicate

- [ ] Tests 1, 4, 6 failing first.
- [ ] Commit: `feat(visibility/b9): central retrieval ACL predicate (B9)`.

### Task B9.2 - search/context enforcement

- [ ] Tests 2, 5, 7 failing first.
- [ ] Commit: `feat(retrieval/b9): enforce ACL before ranking (B9)`.

### Task B9.3 - working memory enforcement

- [ ] Test 3 failing first.
- [ ] Commit: `feat(working/b9): enforce ACL in working memory (B9)`.

### Task B9.4 - dense/neighbor enforcement

- [ ] Test 8 failing first.
- [ ] Commit: `feat(retrieval/b9): filter expanded candidates by ACL (B9)`.

### Task B9.5 - write-path denial

- [ ] Test 9 failing first.
- [ ] Commit: `feat(store/b9): reject cross-agent writes (B9)`.

### Task B9.6 - audit log

- [ ] Test 10 failing first.
- [ ] Commit: `feat(observability/b9): emit visibility denial audit (B9)`.

### Task B9.7 - phase verification

- [ ] Run `cargo fmt --check`.
- [ ] Run `cargo test -p memd-server b9 -- --nocapture`.
- [ ] Run `git diff --check`.
- [ ] Commit: `test(b9): verify visibility ACL enforcement (B9)`.

## 9. Bench Impact

B9 contributes to CH +2 but receives no final score until G9 proves all 8
adversarial scenarios.

## 10. Dependency Graph

- Requires: A9 identity fields and fixtures.
- Blocks: C9 read-path flip, D9 adversarial identity tests, G9 gate.

## Exit Criteria

1. Forbidden items are filtered before scoring.
2. FTS/dense expansion cannot reintroduce forbidden items.
3. Cross-agent write spoofing is denied.
4. Visibility denial audit exists.
5. Tests pass and commits are atomic.

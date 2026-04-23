---
phase: E8
name: Diff + Rollback UI
version: v8
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [A8, B8, D8, V7 G7]
phase_doc: docs/phases/v8/phase-e8-diff-rollback-ui.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: correction_retention, trust_provenance
---

# Phase E8 — Implementation Plan

## 0. Executive summary

Monaco-based diff renderer; one-click rollback calling V7 G7; D7 contradiction accept side-by-side UI; audit log entries with actor="ui".

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `apps/memd-atlas/src/diff/DiffRoute.tsx` | /diff/:pair route. |
| `apps/memd-atlas/src/diff/MonacoDiff.tsx` | Diff renderer. |
| `apps/memd-atlas/src/diff/RollbackButton.tsx` | Confirm + consequence preview. |
| `apps/memd-atlas/src/diff/ContradictionAccept.tsx` | Side-by-side receipt UI. |
| `crates/memd-server/src/routes/diff.rs` | Diff + rollback HTTP. |
| `apps/memd-atlas/tests/e2e/diff-rollback.spec.ts` | E2E. |

### Files to modify

| Path | Change |
| --- | --- |
| `apps/memd-atlas/src/panel/NodePanel.tsx` (A8) | "View diff" + "Rollback" entries. |
| Audit log writer | Add `actor` field; UI sets "ui". |
| Phase doc. |

## 2. Schema changes

```sql
ALTER TABLE correction_audit_log ADD COLUMN actor TEXT NOT NULL DEFAULT 'cli';
```

## 3. API shape

```
GET /diff/:before_id/:after_id         -> { content_diff, metadata_diff }
POST /diff/rollback/:correction_id     { reason }
POST /diff/resolve/:receipt_id         { accept_id }
```

## 4. Test matrix

1. `monaco_renders_content_diff`
2. `metadata_diff_field_level`
3. `rollback_button_only_active_on_eligible`
4. `rollback_confirm_surfaces_consequence`
5. `rollback_writes_audit_entry_with_ui_actor`
6. `contradiction_accept_renders_receipt`
7. `contradiction_accept_calls_v7_resolve`
8. `diff_normalizes_line_endings`
9. `e2e_rollback_full_flow`
10. `e2e_contradiction_resolve_flow`

## 5. Fixtures

- `apps/memd-atlas/tests/fixtures/diff-scenarios.json` — 10 before/after pairs including mixed line endings.

## 6. Telemetry

Audit log samples → `docs/verification/v8-runs/ui/diff/`.

## 7. Feature flags

None.

## 8. Task list

### Task E8.1 — schema + audit actor

- [ ] Migration.
- [ ] Test 5 failing.
- [ ] Commit: `feat(schema/e8): audit actor (E8)`.

### Task E8.2 — diff renderer

- [ ] Tests 1 + 2 + 8 failing.
- [ ] Commit: `feat(apps/e8): monaco diff (E8)`.

### Task E8.3 — rollback button

- [ ] Tests 3 + 4 failing.
- [ ] Commit: `feat(apps/e8): rollback button (E8)`.

### Task E8.4 — contradiction accept

- [ ] Tests 6 + 7 failing.
- [ ] Commit: `feat(apps/e8): contradiction accept (E8)`.

### Task E8.5 — E2E

- [ ] Tests 9 + 10 failing.
- [ ] Commit: `test(apps/e8): rollback + resolve E2E (E8)`.

### Task E8.6 — CI

- [ ] CI playwright.
- [ ] Commit: `ci(apps/e8): diff-rollback smoke (E8)`.

## 9. Bench impact

None direct.

## 10. Dependency graph

- Requires: A8, B8, D8, V7 G7, V7 D7.
- Blocks: F8.

## Exit criteria

1. Tests 1–10 green.
2. Audit log shows UI actor.
3. Rollback UI covers V7 G7 paths.
4. Atomic commits.

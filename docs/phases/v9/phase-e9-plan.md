---
phase: E9
name: Correction Provenance Across Users
version: v9
kind: implementation-plan
status: ready-to-execute
opened: 2026-05-05
depends_on: [C9, D9]
phase_doc: docs/phases/v9/V9-INTEGRATION.md
granularity: "one step = <=1 agent session; TDD; commit per task"
axis: cross_harness
---

# Phase E9 - Implementation Plan

## 0. Executive Summary

Make corrections shared where scope allows, attributed to the source user/agent,
and blocked where scope forbids. E9 converts the C9 flip into a team correction
story without weakening privacy boundaries.

## 1. Surface Area

### Files To Create

| Path | Responsibility |
| --- | --- |
| `crates/memd-client/fixtures/shared/multi-user/cross-workspace-leak-negative.jsonl` | Workspace W1 -> W2 leak negative. |
| `crates/memd-client/fixtures/shared/multi-user/per-scope-retention-negative.jsonl` | Workspace correction must not leak global. |
| `crates/memd-client/src/main_tests/substrate_e9_tests/mod.rs` | E9 correction propagation tests. |
| `docs/verification/v9-runs/e9-correction-provenance.md` | Phase proof summary. |

### Files To Modify

| Path | Change |
| --- | --- |
| `crates/memd-server/src/store.rs` | Preserve correction source identity on supersede/repair. |
| `crates/memd-server/src/store_divergence.rs` | Attach user/agent attribution to divergence receipts. |
| `crates/memd-server/src/helpers.rs` | Rank visible shared corrections without crossing workspace boundaries. |
| `crates/memd-client/src/main_tests/mod.rs` | Wire E9 tests. |

## 2. Schema Changes

No new schema expected if `MemoryItem.correction_meta`, `source_agent`,
`source_system`, `workspace`, `visibility`, and A9 identity fields are enough.
If not enough, add only attribution fields required by tests.

## 3. API Shape

Correction visibility:

- Workspace correction in W1 is visible to user B only when B queries W1.
- Workspace correction in W1 is not visible to W2 or Global.
- Attribution includes original correcting `user_id`, `source_agent`, and
  `harness_preset` when present.

## 4. Test Matrix

1. `e9_user_b_sees_user_a_workspace_correction_with_attribution`
2. `e9_user_a_sees_user_b_workspace_correction_with_attribution`
3. `e9_cross_workspace_correction_does_not_leak`
4. `e9_workspace_correction_does_not_leak_global`
5. `e9_supersede_history_keeps_old_and_new_rows`
6. `e9_divergence_receipt_names_both_users`
7. `e9_shared_correction_ranked_above_stale_claim`
8. `e9_proof_summary_links_fixture_and_tests`

## 5. Fixtures

E9 owns:

- `cross-workspace-leak-negative.jsonl`
- `per-scope-retention-negative.jsonl`

E9 reuses:

- `cross-user-corrections.jsonl`
- `flip-ua-ub-ua.jsonl`

## 6. Telemetry

Proof rows include:

```json
{"phase":"E9","correction_visible":true,"attribution_user":"user-a","workspace":"W1"}
```

## 7. Feature Flags

None.

## 8. Task List

### Task E9.1 - correction attribution tests

- [ ] Tests 1 + 2 failing first.
- [ ] Commit: `test(e9): assert cross-user correction attribution (E9)`.

### Task E9.2 - boundary fixtures

- [ ] Tests 3 + 4 failing first.
- [ ] Commit: `test(fixtures/e9): add correction boundary negatives (E9)`.

### Task E9.3 - propagation implementation

- [ ] Tests 1..4 green.
- [ ] Commit: `feat(correction/e9): propagate shared corrections with attribution (E9)`.

### Task E9.4 - history and divergence receipts

- [ ] Tests 5 + 6 failing first.
- [ ] Commit: `feat(provenance/e9): preserve correction history receipts (E9)`.

### Task E9.5 - ranking guard

- [ ] Test 7 failing first.
- [ ] Commit: `feat(retrieval/e9): rank visible corrections over stale claims (E9)`.

### Task E9.6 - phase proof

- [ ] Test 8 failing first.
- [ ] Commit: `test(e9): record correction provenance proof (E9)`.

### Task E9.7 - phase verification

- [ ] Run `cargo fmt --check`.
- [ ] Run `cargo test -p memd-client e9 -- --nocapture`.
- [ ] Run `cargo test -p memd-server e9 -- --nocapture`.
- [ ] Run `git diff --check`.
- [ ] Commit: `test(e9): verify correction provenance across users (E9)`.

## 9. Bench Impact

E9 supports CH +2 and prepares V10 correction retention, but V9 only scores CH.

## 10. Dependency Graph

- Requires: C9 flip proof, D9 identity fixtures.
- Blocks: F9 rehearsal and G9 adversarial suite.

## Exit Criteria

1. Shared corrections propagate with source attribution.
2. Cross-workspace and workspace-to-global leaks fail closed.
3. Supersede history remains queryable.
4. Divergence receipts name both users when claims conflict.
5. Tests pass and commits are atomic.

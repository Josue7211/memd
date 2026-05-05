---
phase: C9
name: Multi-User Multi-Harness Flip
version: v9
kind: implementation-plan
status: ready-to-execute
opened: 2026-05-05
depends_on: [A9, B9]
phase_doc: docs/phases/v9/V9-INTEGRATION.md
granularity: "one step = <=1 agent session; TDD; commit per task"
axis: session_continuity, cross_harness
---

# Phase C9 - Implementation Plan

## 0. Executive Summary

Run the core V9 dogfood shape in code: user A in harness 1, user B in harness 2,
then user A again. Prove truth survives the flip, user focus does not bleed, and
shared corrections keep attribution.

## 1. Surface Area

### Files To Create

| Path | Responsibility |
| --- | --- |
| `crates/memd-client/src/main_tests/substrate_c9_tests/mod.rs` | In-process C9 flip tests. |
| `crates/memd-client/fixtures/shared/multi-user/cross-user-corrections.jsonl` | Shared correction fixture. |
| `docs/verification/v9-runs/c9-flip-proof.ndjson` | Recorded phase proof. |

### Files To Modify

| Path | Change |
| --- | --- |
| `crates/memd-client/src/main_tests/mod.rs` | Wire C9 test module. |
| `crates/memd-client/src/main_tests/test_support.rs` | Add multi-user fixture runner helpers if needed. |
| `crates/memd-server/src/store_divergence.rs` | Surface divergence receipt if the flip produces competing canonical claims. |

## 2. Schema Changes

No new schema expected. If divergence receipts need structure, prefer existing
memory status/provenance fields before adding tables.

## 3. API Shape

Harness scenario:

```text
S1: user_a / claude-code / agent_alpha writes claim + correction + focus
S2: user_b / codex / agent_beta wakes, cannot see A private focus, sees shared correction attribution
S3: user_a / claude-code / agent_alpha wakes, sees own focus restored and B shared correction/provenance
```

## 4. Test Matrix

1. `c9_flip_user_b_wake_excludes_user_a_private_focus`
2. `c9_flip_user_b_sees_user_a_shared_correction_with_attribution`
3. `c9_flip_user_a_round_trip_restores_own_focus`
4. `c9_flip_user_a_sees_user_b_shared_correction_with_attribution`
5. `c9_flip_canonical_consistent_after_three_sessions`
6. `c9_flip_divergence_receipt_when_claims_conflict`
7. `c9_fixture_replays_without_order_dependence`
8. `c9_proof_ndjson_contains_three_session_cuts`

## 5. Fixtures

- `flip-ua-ub-ua.jsonl` from A9 remains the anchor.
- `cross-user-corrections.jsonl` adds explicit correction attribution checks.

## 6. Telemetry

Write one NDJSON row per session cut:

```json
{"phase":"C9","cut":"S2","user_a_focus_visible_to_b":false}
```

## 7. Feature Flags

None.

## 8. Task List

### Task C9.1 - fixture runner

- [ ] Test 7 failing first.
- [ ] Commit: `test(c9): add multi-user flip fixture runner (C9)`.

### Task C9.2 - wake isolation proof

- [ ] Tests 1 + 3 failing first.
- [ ] Commit: `test(c9): prove wake focus isolation across users (C9)`.

### Task C9.3 - cross-user correction proof

- [ ] Tests 2 + 4 failing first.
- [ ] Commit: `test(c9): prove cross-user correction attribution (C9)`.

### Task C9.4 - canonical consistency

- [ ] Tests 5 + 6 failing first.
- [ ] Commit: `feat(divergence/c9): surface multi-user flip receipts (C9)`.

### Task C9.5 - proof artifact

- [ ] Test 8 failing first.
- [ ] Commit: `test(c9): record multi-user flip proof (C9)`.

### Task C9.6 - phase verification

- [ ] Run `cargo fmt --check`.
- [ ] Run `cargo test -p memd-client c9 -- --nocapture`.
- [ ] Run `git diff --check`.
- [ ] Commit: `test(c9): verify multi-user multi-harness flip (C9)`.

## 9. Bench Impact

C9 is the first direct SC + CH proof, but final score waits for G9.

## 10. Dependency Graph

- Requires: A9 user-scoped state, B9 visibility enforcement.
- Blocks: D9 identity adversarial suite, E9 correction propagation, G9 gate.

## Exit Criteria

1. A -> B -> A replay passes.
2. User B never sees user A private focus.
3. Shared corrections carry source user/agent attribution.
4. Canonical truth is consistent or explicit divergence is emitted.
5. Proof NDJSON is committed.

---
phase: D9
name: Identity Collision + Adversarial Suite
version: v9
kind: implementation-plan
status: ready-to-execute
opened: 2026-05-05
depends_on: [C9]
phase_doc: docs/phases/v9/V9-INTEGRATION.md
granularity: "one step = <=1 agent session; TDD; commit per task"
axis: cross_harness
---

# Phase D9 - Implementation Plan

## 0. Executive Summary

Build the adversarial cases that make V9 trustworthy: same content from
different users, spoofed agent identity, scope escalation, and retention
boundary attempts. D9 provides most G9 fixtures and negative controls.

## 1. Surface Area

### Files To Create

| Path | Responsibility |
| --- | --- |
| `crates/memd-client/fixtures/shared/multi-user/identity-collision-10turn.jsonl` | Same claim hash from two users. |
| `crates/memd-client/fixtures/shared/multi-user/scope-escalation-negative.jsonl` | Local -> Project/Global escalation attempt. |
| `crates/memd-client/fixtures/shared/multi-user/agent-spoofing-negative.jsonl` | Agent B writes as agent A. |
| `docs/contracts/federated-visibility-matrix.json` | Truth table for V9 scope/visibility/caller decisions. |
| `crates/memd-client/src/main_tests/substrate_d9_tests/mod.rs` | D9 adversarial tests. |

### Files To Modify

| Path | Change |
| --- | --- |
| `crates/memd-server/src/store_dedup.rs` | Preserve co-attribution on content-hash dedup. |
| `crates/memd-server/src/store.rs` | Forbid mutable identity changes and automatic scope promotion. |
| `crates/memd-schema/src/lib.rs` | Add co-author/provenance shape only if existing payload cannot express it. |
| `crates/memd-client/src/main_tests/mod.rs` | Wire D9 tests. |

## 2. Schema Changes

Prefer payload-compatible provenance first. If a table is required:

```sql
CREATE TABLE IF NOT EXISTS memory_item_authors (
  memory_id TEXT NOT NULL,
  user_id TEXT,
  agent_id TEXT NOT NULL,
  harness_preset TEXT,
  first_seen_at TEXT NOT NULL,
  PRIMARY KEY (memory_id, user_id, agent_id, harness_preset)
);
```

Do not replace the original author row. D9 requires co-attribution, not silent
dedup discard.

## 3. API Shape

- Store rejects caller identity mismatch.
- Promote requires an explicit trusted promotion API, never lookup side effects.
- Dedup response includes enough data to prove both authors survived.

## 4. Test Matrix

1. `d9_identity_collision_preserves_both_authors`
2. `d9_identity_collision_does_not_duplicate_canonical_claim`
3. `d9_agent_spoofing_write_rejected`
4. `d9_agent_id_immutable_after_insert`
5. `d9_local_to_project_auto_promotion_rejected`
6. `d9_lookup_promote_global_requires_trusted_flag`
7. `d9_visibility_matrix_covers_scope_visibility_agent_axes`
8. `d9_negative_controls_fail_when_acl_disabled`

## 5. Fixtures

D9 owns these shared fixtures:

- `identity-collision-10turn.jsonl`
- `scope-escalation-negative.jsonl`
- `agent-spoofing-negative.jsonl`
- `federated-visibility-matrix.json`

## 6. Telemetry

Adversarial result rows include `scenario`, `expected`, `actual`, and
`negative_control_fired`.

## 7. Feature Flags

None. Identity checks are contract enforcement.

## 8. Task List

### Task D9.1 - visibility matrix

- [ ] Test 7 failing first.
- [ ] Commit: `docs(v9): add federated visibility matrix (D9)`.

### Task D9.2 - identity collision fixtures

- [ ] Tests 1 + 2 failing first.
- [ ] Commit: `test(fixtures/d9): add identity collision cases (D9)`.

### Task D9.3 - co-attribution dedup

- [ ] Tests 1 + 2 green.
- [ ] Commit: `feat(dedup/d9): preserve co-authors on collision (D9)`.

### Task D9.4 - spoofing and immutability

- [ ] Tests 3 + 4 failing first.
- [ ] Commit: `feat(identity/d9): reject spoofing and identity mutation (D9)`.

### Task D9.5 - scope escalation guard

- [ ] Tests 5 + 6 failing first.
- [ ] Commit: `feat(scope/d9): block automatic promotion (D9)`.

### Task D9.6 - negative controls

- [ ] Test 8 failing first.
- [ ] Commit: `test(d9): prove adversarial negative controls fire (D9)`.

### Task D9.7 - phase verification

- [ ] Run `cargo fmt --check`.
- [ ] Run `cargo test -p memd-client d9 -- --nocapture`.
- [ ] Run `cargo test -p memd-server d9 -- --nocapture`.
- [ ] Run `git diff --check`.
- [ ] Commit: `test(d9): verify identity adversarial suite (D9)`.

## 9. Bench Impact

D9 is the backbone for CH +2. Final score waits for G9.

## 10. Dependency Graph

- Requires: C9 flip proof.
- Blocks: E9 provenance checks and G9 full suite.

## Exit Criteria

1. Content-hash collision keeps both authors.
2. Spoofing and identity mutation fail closed.
3. Scope escalation cannot happen as a read side effect.
4. Negative controls prove the tests would fail on a leak.
5. Tests pass and commits are atomic.

---
phase: G9
name: Multi-User Adversarial Gate Harness
version: v9
kind: implementation-plan
status: ready-to-execute
opened: 2026-05-05
depends_on: [F9]
phase_doc: docs/phases/v9/V9-INTEGRATION.md
granularity: "one step = <=1 agent session; TDD; commit per task"
axis: session_continuity, cross_harness
---

# Phase G9 - Implementation Plan

## 0. Executive Summary

Close V9 only if the full adversarial suite passes: zero visibility leaks, zero
identity collisions, zero scope escalation, negative controls firing, and strict
scorecard regeneration to composite 5.60.

## 1. Surface Area

### Files To Create

| Path | Responsibility |
| --- | --- |
| `scripts/verify/v9-adversarial-suite.sh` | Final gate runner. |
| `crates/memd-client/src/main_tests/substrate_g9_tests/mod.rs` | G9 gate tests. |
| `docs/verification/v9-proof-runs/YYYY-MM-DD-adversarial-suite.ndjson` | Signed final proof artifact. |
| `docs/verification/v9-proof-runs/YYYY-MM-DD-adversarial-suite.md` | Human summary. |
| `docs/handoff/YYYY-MM-DD-v9-closed-v10-next.md` | Final V9 handoff. |

### Files To Modify

| Path | Change |
| --- | --- |
| `docs/verification/MEMD-10-STAR.md` | Regenerate V9 scorecard: composite 5.60. |
| `docs/verification/milestones/MILESTONE-v9.md` | Fill evidence paths and mark closed. |
| `ROADMAP.md` | Mark V9 closed and V10 ready. |
| `docs/handoff/LATEST.md` / `INDEX.md` | Updated by `scripts/handoff-latest.sh`. |

## 2. Schema Changes

None. Any schema change at G9 is a failure to finish earlier phases.

## 3. API Shape

Final command:

```bash
scripts/verify/v9-adversarial-suite.sh
```

Required result:

```json
{"phase":"G9","scenario_count":8,"pass_count":8,"fail_count":0,"negative_controls_fired":8,"composite":5.60}
```

## 4. Test Matrix

1. `g9_suite_runs_all_contract_scenarios`
2. `g9_cross_user_read_leak_passes`
3. `g9_cross_user_write_escape_passes`
4. `g9_correction_provenance_passes`
5. `g9_content_hash_coattribution_passes`
6. `g9_agent_spoofing_passes`
7. `g9_scope_escalation_passes`
8. `g9_retention_boundary_passes`
9. `g9_multi_user_flip_passes`
10. `g9_negative_controls_all_fire`
11. `g9_scorecard_caps_sc_and_ch`
12. `g9_non_owned_axes_unchanged`
13. `g9_proof_artifact_required_for_close`
14. `g9_close_updates_roadmap_milestone_and_handoff`

## 5. Fixtures

Consumes all shared V9 fixtures:

- `ua-ub-ua-3session.jsonl`
- `cross-user-corrections.jsonl`
- `identity-collision-10turn.jsonl`
- `scope-escalation-negative.jsonl`
- `agent-spoofing-negative.jsonl`
- `cross-workspace-leak-negative.jsonl`
- `per-scope-retention-negative.jsonl`
- `flip-ua-ub-ua.jsonl`
- `federated-visibility-matrix.json`

## 6. Telemetry

Final artifact directory:

```text
docs/verification/v9-proof-runs/
```

Each scenario row must include:

- scenario id
- assertion result
- negative control result
- caller identity
- item identity
- visibility decision
- proof timestamp

## 7. Feature Flags

None. Final gate must run against production defaults.

## 8. Task List

### Task G9.1 - final suite runner

- [ ] Tests 1 + 13 failing first.
- [ ] Commit: `test(g9): add final adversarial suite runner (G9)`.

### Task G9.2 - scenario assertions

- [ ] Tests 2..9 failing first.
- [ ] Commit: `test(g9): assert all V9 adversarial scenarios (G9)`.

### Task G9.3 - negative controls

- [ ] Test 10 failing first.
- [ ] Commit: `test(g9): require adversarial negative controls (G9)`.

### Task G9.4 - strict scorecard regenerator

- [ ] Tests 11 + 12 failing first.
- [ ] Commit: `test(g9): enforce V9 scorecard caps (G9)`.

### Task G9.5 - proof artifact

- [ ] Run final suite and commit NDJSON + markdown.
- [ ] Commit: `test(g9): record V9 adversarial proof (G9)`.

### Task G9.6 - roadmap and milestone close

- [ ] Test 14 failing first.
- [ ] Update `MEMD-10-STAR.md`, `MILESTONE-v9.md`, and `ROADMAP.md`.
- [ ] Commit: `docs(v9): close multi-user milestone (G9)`.

### Task G9.7 - final handoff

- [ ] Create V9 close handoff.
- [ ] Run `scripts/handoff-latest.sh`.
- [ ] Commit: `docs(handoff): V9 closed V10 next`.

## 9. Bench Impact

G9 awards V9 target only:

- SC = 6
- CH = 6
- CR = 5 unchanged
- PR = 4 unchanged
- RR = 7 unchanged
- TE = 5 unchanged
- TP = 6 unchanged
- Composite = 5.60

## 10. Dependency Graph

- Requires: F9 dry-run green.
- Blocks: V10 entry and release-candidate window.

## Exit Criteria

1. All 8 adversarial scenarios pass.
2. All negative controls fire.
3. Proof NDJSON and summary are committed.
4. Scorecard strict-mode prevents over-credit.
5. ROADMAP and milestone mark V9 closed, V10 next.
6. Handoff is committed.

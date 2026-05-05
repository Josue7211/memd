---
phase: F9
name: Pre-Ship Harness Dry-Run
version: v9
kind: implementation-plan
status: ready-to-execute
opened: 2026-05-05
depends_on: [A9, B9, C9, D9, E9]
phase_doc: docs/phases/v9/V9-INTEGRATION.md
granularity: "one step = <=1 agent session; TDD; commit per task"
axis: session_continuity, cross_harness
---

# Phase F9 - Implementation Plan

## 0. Executive Summary

Run the G9 suite in rehearsal mode before milestone close. F9 proves the harness
shape, runtime budget, negative-control behavior, and artifact format without
flipping roadmap scores.

## 1. Surface Area

### Files To Create

| Path | Responsibility |
| --- | --- |
| `scripts/verify/v9-adversarial-dry-run.sh` | Local dry-run wrapper. |
| `crates/memd-client/src/main_tests/substrate_f9_tests/mod.rs` | Dry-run harness tests. |
| `docs/verification/v9-runs/f9-dry-run.ndjson` | Rehearsal output. |
| `docs/verification/v9-runs/f9-dry-run.md` | Human-readable summary. |

### Files To Modify

| Path | Change |
| --- | --- |
| `crates/memd-client/src/main_tests/mod.rs` | Wire F9 tests. |
| `scripts/verify/README.md` | Document V9 dry-run command if README exists. |
| `docs/verification/milestones/MILESTONE-v9.md` | Add dry-run evidence path without closing V9. |

## 2. Schema Changes

None.

## 3. API Shape

Dry-run command:

```bash
scripts/verify/v9-adversarial-dry-run.sh
```

Output:

```json
{"phase":"F9","suite":"v9-dry-run","pass_count":8,"fail_count":0,"negative_controls_fired":8}
```

## 4. Test Matrix

1. `f9_dry_run_invokes_all_8_scenarios`
2. `f9_dry_run_records_negative_control_results`
3. `f9_dry_run_fails_when_any_scenario_fails`
4. `f9_dry_run_fails_when_negative_control_missing`
5. `f9_dry_run_ndjson_schema_is_stable`
6. `f9_dry_run_runtime_within_ci_budget`
7. `f9_dry_run_does_not_update_scorecard`
8. `f9_script_exits_nonzero_on_failure`

## 5. Fixtures

Consumes all shared fixtures from A9, C9, D9, and E9. F9 owns no new memory
fixtures.

## 6. Telemetry

F9 writes rehearsal artifacts only:

- `docs/verification/v9-runs/f9-dry-run.ndjson`
- `docs/verification/v9-runs/f9-dry-run.md`

## 7. Feature Flags

None.

## 8. Task List

### Task F9.1 - harness inventory

- [ ] Test 1 failing first.
- [ ] Commit: `test(f9): enumerate V9 adversarial scenarios (F9)`.

### Task F9.2 - dry-run runner

- [ ] Tests 3 + 8 failing first.
- [ ] Commit: `test(f9): add adversarial dry-run runner (F9)`.

### Task F9.3 - negative controls

- [ ] Tests 2 + 4 failing first.
- [ ] Commit: `test(f9): require negative controls in dry-run (F9)`.

### Task F9.4 - artifact schema

- [ ] Test 5 failing first.
- [ ] Commit: `test(f9): lock dry-run artifact schema (F9)`.

### Task F9.5 - runtime and scorecard guards

- [ ] Tests 6 + 7 failing first.
- [ ] Commit: `test(f9): enforce dry-run budget and no score update (F9)`.

### Task F9.6 - phase proof

- [ ] Run dry-run command and commit artifacts.
- [ ] Commit: `test(f9): record V9 dry-run proof (F9)`.

### Task F9.7 - phase verification

- [ ] Run `cargo fmt --check`.
- [ ] Run `cargo test -p memd-client f9 -- --nocapture`.
- [ ] Run `scripts/verify/v9-adversarial-dry-run.sh`.
- [ ] Run `git diff --check`.
- [ ] Commit: `test(f9): verify pre-ship dry-run (F9)`.

## 9. Bench Impact

No score changes. F9 is rehearsal only.

## 10. Dependency Graph

- Requires: A9..E9 completed.
- Blocks: G9 milestone gate.

## Exit Criteria

1. All 8 G9 scenarios run in dry-run mode.
2. All negative controls fire.
3. Artifact schema is stable.
4. Scorecard stays unchanged.
5. Tests pass and commits are atomic.

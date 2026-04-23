---
phase: C7
name: Next-Session Behavior Change Test
version: v7
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [B7, V5 B5]
phase_doc: docs/phases/v7/phase-c7-next-session-behavior-change.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: correction_retention, session_continuity
---

# Phase C7 — Implementation Plan

## 0. Executive summary

3-session scripted scenario × 5 corrections × nightly CI. Measures `behavior_changed_at_session` per correction. Hits respected-rate@2 ≥ 0.90, @5 ≥ 0.85. Feeds V5 B5 sub-metric.

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `crates/memd-client/src/benchmark/substrate/correction_behavior.rs` | C7 runner (inside substrate suite module). |
| `.memd/benchmarks/substrate/correction-behavior.yaml` | Spec. |
| `.memd/benchmarks/substrate/fixtures/c7/3-session-5-correction.jsonl` | Scenario script. |
| `crates/memd-client/src/main_tests/correction_behavior_tests/mod.rs` | Tests. |

### Files to modify

| Path | Change |
| --- | --- |
| `benchmark/substrate/mod.rs` | Register new suite alongside V5 suites. |
| `benchmark/substrate/correction_propagation.rs` (V5 B5) | Accept sub-metric from C7 runner. |
| Phase doc. |

## 2. Schema changes

None.

Metric schema:

```yaml
per_correction:
  id: string
  planted_session: int
  behavior_changed_at_session: int | null
aggregate:
  respected_rate_at_session_2: float
  respected_rate_at_session_3: float
  respected_rate_at_session_5: float
```

## 3. API shape

```
memd bench substrate --suite correction-behavior
memd bench substrate --suite correction-behavior --nightly-ci
```

## 4. Test matrix

1. `scenario_harness_runs_3_sessions`
2. `scenario_plants_5_corrections_in_session_1`
3. `scenario_measures_behavior_at_sessions_2_3_5`
4. `metric_respected_rate_correct_on_synthetic`
5. `runner_feeds_sub_metric_to_b5`
6. `cli_correction_behavior_happy`
7. `cli_correction_behavior_reproducibility`
8. `respected_rate_at_2_meets_0_90_floor`
9. `respected_rate_at_5_meets_0_85_floor`

## 5. Fixtures

`3-session-5-correction.jsonl`: scripted session-1 messages with 5 plants, session-2 + session-3 + session-5 query messages, expected-answer ground truth.

## 6. Telemetry

`.memd/logs/correction-behavior.ndjson` per-correction persistence.

## 7. Feature flags

None.

## 8. Task list

### Task C7.1 — scenario harness

- [ ] Tests 1–3 failing.
- [ ] Commit: `feat(bench/c7): scenario harness (C7)`.

### Task C7.2 — metric

- [ ] Test 4 failing.
- [ ] Commit: `feat(bench/c7): respected-rate metric (C7)`.

### Task C7.3 — B5 integration

- [ ] Test 5 failing.
- [ ] Commit: `feat(bench/c7): B5 sub-metric (C7)`.

### Task C7.4 — CLI

- [ ] Tests 6 + 7 failing.
- [ ] Commit: `feat(bench/c7): CLI (C7)`.

### Task C7.5 — floor lock

- [ ] Tests 8 + 9 failing; run nightly; lock.
- [ ] Commit: `bench(c7): floor locked ≥0.90/≥0.85 (C7)`.

### Task C7.6 — CI wire

- [ ] CI nightly; failure blocks.
- [ ] Commit: `ci(c7): nightly correction behavior (C7)`.

## 9. Bench impact

B5 suite gains `next-session-behavior` sub-metric.

## 10. Dependency graph

- Requires: B7, V5 B5 in main.
- Blocks: G7.

## Exit criteria

1. Tests 1–9 green.
2. respected-rate@2 ≥ 0.90; @5 ≥ 0.85.
3. Nightly CI running ≥ 7 days.
4. B5 sub-metric wired.
5. Atomic commits.

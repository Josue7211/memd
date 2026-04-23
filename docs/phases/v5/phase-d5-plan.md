---
phase: D5
name: ProgressiveDepth Bench
version: v5
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [V4, A5]
phase_doc: docs/phases/v5/phase-d5-progressive-depth.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: — (integrates token_efficiency, raw_retrieval; no credit per 0.1.0-AXIS-OWNERSHIP overlap 3 — V4 E4 / V6 E6 own)
---

# Phase D5 — Implementation Plan

## 0. Executive summary

Measures V4 E4's depth contract adherence numerically. 30 queries × 3 depths = 90 invocations per run. Metrics per depth: token cost, completeness (exact-match of required facts in output), irrelevant-record ratio, latency.

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `crates/memd-client/src/benchmark/substrate/progressive_depth.rs` | D5 runner. |
| `.memd/benchmarks/substrate/progressive-depth.yaml` | Spec. |
| `.memd/benchmarks/substrate/fixtures/d5/` | 90 queries + expected completeness tuples. |
| `crates/memd-client/src/main_tests/substrate_d5_tests/mod.rs` | Tests. |

### Files to modify

- `substrate/mod.rs` register.
- phase doc `plan_spec:` line.

## 2. Schema changes

None. YAML only.

```yaml
suite: progressive-depth
depth_classes: [overview, targeted, resume]
queries_per_class: 30
pass_gate:
  wake_p95_tokens: 2000
  wake_completeness: 0.80
  lookup_completeness: 0.85
  lookup_tokens_p95: 500
  resume_completeness: 0.95
  resume_tokens_p95: 6000
  contract_adherence_rate: 0.95
```

## 3. API shape

```
memd bench substrate --suite progressive-depth [--depth-only wake]
```

## 4. Test matrix

1. `fixture_loader_groups_queries_by_depth_class`
2. `runner_invokes_each_depth_via_memd_lookup_depth_flag`
3. `scorer_completeness_exact_match_on_required_facts`
4. `scorer_irrelevant_record_ratio`
5. `runner_measures_token_cost_per_call` — reuses D4 wake-budget NDJSON.
6. `cli_d5_happy`
7. `cli_d5_fails_when_wake_exceeds_budget`
8. `cli_d5_reproducibility`
9. `d5_baseline_lock`

## 5. Fixtures

90 queries authored by hand or sampled from dogfood. Expected-completeness tuples: `{query_id: [required_fact_id, …]}`.

## 6. Telemetry

NDJSON per scenario + shared report.

## 7. Feature flags

`MEMD_SUBSTRATE_D5_DEPTH_ONLY` optional gate.

## 8. Task list

### Task D5.1 — runner scaffold

- [ ] Stub + register; Commit: `scaffold(bench/d5): runner stub`.

### Task D5.2 — fixtures

- [ ] Author queries + expected tuples; Commit: `test-fixtures(bench/d5): 90 queries`.

### Task D5.3 — scorer

- [ ] Tests 3 + 4.
- [ ] Commit: `feat(bench/d5): scorer (D5)`.

### Task D5.4 — runner invocation

- [ ] Tests 1 + 2 + 5.
- [ ] Commit: `feat(bench/d5): runner (D5)`.

### Task D5.5 — CLI + pass-gate

- [ ] Tests 6 + 7 + 8.
- [ ] Commit: `feat(bench/d5): CLI + pass-gate (D5)`.

### Task D5.6 — baseline + CI

- [ ] Test 9 + CI wire.
- [ ] Commit: `bench+ci(d5): baseline + CI (D5)`.

### Task D5.7 — 10-STAR

- [ ] token_efficiency bump.
- [ ] Commit: `docs(10-star): D5 token_efficiency`.

## 9. Bench impact

D5 is the E4 contract adherence measurement.

## 10. Dependency graph

- Requires: A5 runtime, D4 compiler + E4 depth flag.
- Blocks: G5.
- Parallelizable with B5/C5/E5/F5.

## Exit criteria

1. Tests 1–9 green.
2. All depth pass-gates hit.
3. Contract-adherence ≥ 0.95.
4. 10-STAR updated.
5. Atomic commits.

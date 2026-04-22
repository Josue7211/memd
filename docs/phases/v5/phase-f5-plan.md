---
phase: F5
name: TypedRetrieval Bench
version: v5
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [V4, A5]
phase_doc: docs/phases/v5/phase-f5-typed-retrieval.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: raw_retrieval, typed_retrieval
---

# Phase F5 — Implementation Plan

## 0. Executive summary

Measures whether query shape routes to the right MemoryKind. 50 queries × 11 kinds = 550 invocations. Reports confusion matrix + `correct-type-rate@1`.

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `crates/memd-client/src/benchmark/substrate/typed_retrieval.rs` | F5 runner. |
| `.memd/benchmarks/substrate/typed-retrieval.yaml` | Spec. |
| `.memd/benchmarks/substrate/fixtures/f5/` | 550-query corpus + expected kinds. |
| `docs/contracts/type-taxonomy.md` | Taxonomy card. |
| `crates/memd-client/src/main_tests/substrate_f5_tests/mod.rs` | Tests. |

### Files to modify

| Path | Change |
| --- | --- |
| `crates/memd-client/src/runtime/lookup.rs` | Add `--explain-route` flag exposing per-result kind + router rationale. |
| `substrate/mod.rs` | Register F5. |
| Phase doc. |

## 2. Schema changes

None. `--explain-route` emits additional JSON on existing lookup path.

```yaml
suite: typed-retrieval
queries_per_kind: 50
kinds: [Fact, Decision, Preference, Runbook, Procedural, SelfModel, Topology, Status, LiveTruth, Pattern, Constraint, Correction]
pass_gate:
  correct_type_rate_at_1: 0.85
  wrong_type_ratio: 0.05
  per_kind_min_rate: 0.75
```

## 3. API shape

```
memd bench substrate --suite typed-retrieval
memd lookup --query "…" --explain-route --json
```

`--explain-route` output shape:

```json
{"query":"…","routed_kinds":["Decision","Fact"],"router_rationale":"shape-match:decision-verbs","results":[{"id":"…","kind":"Decision","score":0.91}]}
```

## 4. Test matrix

1. `lookup_explain_route_emits_kinds_and_rationale`
2. `scorer_correct_type_at_1_on_top_result`
3. `scorer_confusion_matrix_emission`
4. `runner_550_queries_complete`
5. `cli_f5_happy`
6. `cli_f5_fails_on_under_0_85`
7. `cli_f5_reproducibility`
8. `f5_baseline_lock`
9. `taxonomy_card_round_trip` — taxonomy card parseable + references real kinds.

## 5. Fixtures

550 queries × expected kind. Authored; reference taxonomy card. Regenerable from taxonomy + query template generator.

## 6. Telemetry

NDJSON with per-query expected/actual, plus aggregate confusion matrix CSV.

## 7. Feature flags

`MEMD_LOOKUP_EXPLAIN_ROUTE` default `1` — off in production if overhead matters.

## 8. Task list

### Task F5.1 — `--explain-route` flag

- [ ] Test 1 failing.
- [ ] Extend `LookupArgs` + router to emit route info.
- [ ] Green.
- [ ] Commit: `feat(memd-client/lookup): --explain-route (F5)`.

### Task F5.2 — taxonomy card

- [ ] Test 9 failing.
- [ ] Write `docs/contracts/type-taxonomy.md`.
- [ ] Green.
- [ ] Commit: `docs(contracts): type-taxonomy (F5)`.

### Task F5.3 — scorer + confusion matrix

- [ ] Tests 2 + 3 failing.
- [ ] Commit: `feat(bench/f5): scorer + confusion matrix (F5)`.

### Task F5.4 — runner + fixtures

- [ ] Test 4 failing.
- [ ] Commit: `feat(bench/f5): runner + 550 fixtures (F5)`.

### Task F5.5 — CLI + pass-gate

- [ ] Tests 5 + 6 + 7 failing.
- [ ] Commit: `feat(bench/f5): CLI + pass-gate (F5)`.

### Task F5.6 — baseline + CI + 10-STAR

- [ ] Test 8 + CI + axis bump.
- [ ] Commit: `bench+ci+docs(f5): baseline + CI + 10-STAR (F5)`.

## 9. Bench impact

F5 makes typed retrieval a number. Feeds V6 (public-bench typing uses same router).

## 10. Dependency graph

- Requires: A5 runtime; MemoryKind::Correction (C4).
- Blocks: G5, V6 all phases (taxonomy card consumed).
- Parallelizable with B5/C5/D5/E5.

## Exit criteria

1. Tests 1–9 green.
2. correct-type-rate@1 ≥ 0.85; no kind under 0.75.
3. Taxonomy card linked.
4. 10-STAR updated.
5. Atomic commits.

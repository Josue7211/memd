---
phase: B5
name: CorrectionPropagation Bench
version: v5
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [V4, A5]
phase_doc: docs/phases/v5/phase-b5-correction-propagation.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: — (integrates correction_retention; no credit per 0.1.0-AXIS-OWNERSHIP overlap 1 — V4 C4 owns 1→4, V7 A7/C7 owns 4→5)
---

# Phase B5 — Implementation Plan

## 0. Executive summary

Reuses A5's session driver + scorer scaffolding. Adds a correction-propagation suite: plant N facts, correct each in session 2, query in sessions {3, 5, 8}. Validates both value propagation (lookup returns corrected) and provenance linkage (retrieved record cites correction turn).

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `crates/memd-client/src/benchmark/substrate/correction_propagation.rs` | B5 runner. |
| `.memd/benchmarks/substrate/correction-propagation.yaml` | Bench spec. |
| `crates/memd-client/src/main_tests/substrate_b5_tests/mod.rs` | Integration tests. |
| `docs/verification/substrate-baselines/b5-YYYY-MM-DD.json` | Locked floor. |
| `.memd/benchmarks/substrate/fixtures/b5/` | Fixture facts + correction scripts. |

### Files to modify

| Path | Change |
| --- | --- |
| `crates/memd-client/src/benchmark/substrate/mod.rs` | Register B5 in dispatcher. |
| `crates/memd-client/src/benchmark/substrate/scorers.rs` | Add `ProvenanceChainScorer`. |
| `docs/phases/v5/phase-b5-correction-propagation.md` | `plan_spec:` line. |
| `docs/verification/SUBSTRATE_BENCHMARKS.md` | B5 section placeholder. |

## 2. Schema changes

None. YAML + NDJSON.

### Bench spec

```yaml
suite: correction-propagation
version: 1
seed: 43
fact_count: 20
correct_in_session: 2
query_sessions: [3, 5, 8]
scorer:
  primary: exact_match_plus_provenance_chain
pass_gate:
  propagation_rate_s3: 0.85
  propagation_rate_s8: 0.80
  provenance_correctness: 0.95
```

### NDJSON

```json
{"suite":"correction-propagation","run_id":"…","seed":43,"fact_id":"f-12","corrected_at_session":2,"query_session":5,"returned_value":"ulid","expected_corrected":"ulid","propagated":true,"provenance_cites_correction_turn":true}
```

## 3. API shape

```
memd bench substrate --suite correction-propagation [--seed 43] […]
```

Same flag set as A5; dispatcher routes.

## 4. Test matrix

1. `scorer_provenance_chain_passes_when_correction_turn_cited`
2. `scorer_provenance_chain_fails_when_chain_broken`
3. `runner_applies_correction_in_session_2_via_c4_path`
4. `runner_queries_each_target_session`
5. `cli_b5_happy_path`
6. `cli_b5_fails_when_propagation_under_floor`
7. `cli_b5_reproducibility_same_seed_identical_output`
8. `b5_baseline_lock`
9. `b5_rollback_reassert_preserves_chain` — after user re-asserts original, both nodes exist in chain.

### Rebuild + smoke

```
cargo test --target-dir /tmp/memd-target -p memd-client substrate_b5
```

## 5. Fixtures

`.memd/benchmarks/substrate/fixtures/b5/`:

| File | Contents |
| --- | --- |
| `facts-seed43-n20.jsonl` | 20 facts. |
| `corrections-s2.jsonl` | 20 corrections (value + phrasing). |
| `queries-per-fact.jsonl` | 3 queries per fact. |
| `rollback-reassert-s5.jsonl` | 5 facts that revert in session 5. |

## 6. Telemetry

Results NDJSON + markdown section + grader cache at `.memd/benchmarks/grader-cache/b5/`.

## 7. Feature flags

Inherits A5 flags. Additional: `MEMD_SUBSTRATE_B5_ROLLBACK` default `1` (runs rollback-reassert subtest).

## 8. Task list

### Task B5.1 — runner + dispatcher hook

- [ ] New `correction_propagation.rs` stub + dispatcher register.
- [ ] Commit: `scaffold(bench/substrate): B5 runner stub`.

### Task B5.2 — provenance-chain scorer

- [ ] Tests 1 + 2 failing.
- [ ] Implement `ProvenanceChainScorer` in `scorers.rs`.
- [ ] Green.
- [ ] Commit: `feat(bench/substrate/scorers): provenance-chain (B5)`.

### Task B5.3 — runner fully

- [ ] Tests 3 + 4 failing.
- [ ] Implement using A5 session driver + C4 `memd correction capture` path.
- [ ] Green.
- [ ] Commit: `feat(bench/substrate): B5 runner (B5)`.

### Task B5.4 — CLI + pass-gate

- [ ] Tests 5 + 6 + 7 failing.
- [ ] Wire dispatcher → runner.
- [ ] Green.
- [ ] Commit: `feat(bench/substrate): B5 CLI + pass-gate (B5)`.

### Task B5.5 — baseline lock

- [ ] Test 8.
- [ ] Commit: `bench(baselines): B5 floor`.

### Task B5.6 — rollback subtest

- [ ] Test 9 failing.
- [ ] Implement + green.
- [ ] Commit: `test(bench/b5): rollback-reassert subtest (B5)`.

### Task B5.7 — CI + 10-STAR

- [ ] CI nightly wire + axis bump.
- [ ] Commit: `ci+docs(bench): B5 nightly + correction_retention +1 (B5)`.

## 9. Bench impact

B5 is the correction-retention substrate bench. Shares grader cache with C4/F4 judge pool.

## 10. Dependency graph

- Requires: A5 (session driver, fixture helpers, scorer base).
- Blocks: G5 aggregation, V7 smoke tests (V7 reuses B5 as its happy path).
- Parallelizable with C5–F5 after A5 Task A5.4.

## Exit criteria

1. Tests 1–9 green 10/10.
2. Pass-gate numbers hit and locked.
3. Rollback subtest green.
4. CI nightly green.
5. 10-STAR correction_retention +1.
6. Atomic commits on `research/mining`.

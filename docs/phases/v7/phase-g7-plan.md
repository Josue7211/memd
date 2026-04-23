---
phase: G7
name: Rollback + V7 Completion Gate
version: v7
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [A7, B7, C7, D7, E7, F7]
phase_doc: docs/phases/v7/phase-g7-rollback.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: correction_retention, trust_provenance, V7 completion gate
---

# Phase G7 — Implementation Plan

## 0. Executive summary

Two jobs: (1) rollback CLI + chain-preserving restore; (2) V7 gate — 3-session dogfood + V7 aggregator + 10-STAR composite ≥ 7.8.

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `crates/memd-core/src/correction/rollback.rs` | Rollback engine. |
| `crates/memd-client/src/commands/correction_rollback.rs` | CLI. |
| `crates/memd-client/src/benchmark/v7_aggregator.rs` | V7 audit regenerator. |
| `docs/verification/V7_CORRECTION_AUDIT.md` | Regenerated audit. |
| `crates/memd-core/src/main_tests/rollback_tests/mod.rs` | Tests. |

### Files to modify

| Path | Change |
| --- | --- |
| `memd-schema/src/lib.rs` | `Stage::RetractedByRollback`. |
| Schema migration. |
| `docs/verification/MEMD-10-STAR.md` | Regenerated. |
| `docs/verification/milestones/MILESTONE-v7.md` | Filled. |
| `ROADMAP.md` | V7 → closed, V8 → in progress. |
| Phase doc. |

## 2. Schema changes

```sql
ALTER TYPE memory_stage ADD VALUE 'retracted_by_rollback';
```

## 3. API shape

```
memd correction rollback <correction-id> [--reason "…"]
memd correction rollback --dry-run <correction-id>
memd bench v7 --aggregate [--regenerate-audit] [--regenerate-10star]
```

## 4. Test matrix

### Rollback

1. `rollback_restores_prior_canonical`
2. `rollback_demotes_corrector_to_retracted_by_rollback`
3. `rollback_preserves_chain_with_reverse_link`
4. `rollback_chain_of_three_corrections`
5. `rollback_with_intervening_correction_errors`
6. `rollback_dry_run_no_writes`
7. `cli_rollback_happy`
8. `cli_rollback_invalid_id`

### V7 aggregator + gate

9. `aggregator_rolls_up_a7_miss_rate`
10. `aggregator_rolls_up_b7_promotion_count`
11. `aggregator_rolls_up_c7_respected_rate`
12. `aggregator_rolls_up_d7_contradictions`
13. `aggregator_rolls_up_e7_chain_completeness`
14. `aggregator_rolls_up_f7_surface_snapshot`
15. `aggregator_rolls_up_g7_rollback_count`
16. `star_regen_refuses_composite_below_7_8`
17. `three_session_dogfood_clean_run`
18. `b5_suite_at_or_above_1_00`

## 5. Fixtures

- `tests/fixtures/correction/g7/rollback-10.jsonl` — 10 rollback scenarios with expected end-states.
- `.memd/benchmarks/substrate/fixtures/g7/3-session-5-correction-1-rollback.jsonl` — dogfood scenario.

## 6. Telemetry

`.memd/logs/rollback.ndjson`; aggregated at `docs/verification/v7-runs/v7-<date>.ndjson`.

## 7. Feature flags

| Var | Default | Effect |
| --- | --- | --- |
| `MEMD_V7_ALLOW_BELOW_TARGET` | `0` | 10-STAR writer refuses <7.8 unless set. |

## 8. Task list

### Task G7.1 — rollback engine

- [ ] Tests 1–5 failing.
- [ ] Commit: `feat(correction/g7): rollback engine (G7)`.

### Task G7.2 — CLI + dry-run

- [ ] Tests 6 + 7 + 8 failing.
- [ ] Commit: `feat(cli/g7): rollback (G7)`.

### Task G7.3 — V7 aggregator

- [ ] Tests 9–15 failing.
- [ ] Commit: `feat(bench/g7): V7 aggregator (G7)`.

### Task G7.4 — 10-STAR writer

- [ ] Test 16 failing.
- [ ] Commit: `feat(bench/g7): V7 10-STAR writer (G7)`.

### Task G7.5 — dogfood

- [ ] Test 17 failing; run 10× CI; lock.
- [ ] Commit: `bench(g7): 3-session dogfood clean (G7)`.

### Task G7.6 — B5 floor

- [ ] Test 18 failing; B5 must hit 1.00.
- [ ] Commit: `bench(g7): B5 at 1.00 (G7)`.

### Task G7.7 — milestone close

- [ ] 10/10 CI over 7 days; composite ≥ 7.8.
- [ ] Fill MILESTONE-v7; flip ROADMAP; open V8.
- [ ] Commit: `docs(milestone): V7 closed, composite ≥7.8 (G7)`.

## 9. Bench impact

V7 close. B5 at 1.00; composite ≥ 7.8.

## 10. Dependency graph

- Requires: A7–F7 closed.
- Blocks: V8.

## Exit criteria (V7 milestone)

1. Rollback tests 1–8 green.
2. Aggregator tests 9–15 green.
3. Composite gate tests 16 + 18 green.
4. 3-session dogfood 10/10.
5. Composite ≥ 7.8 written.
6. MILESTONE-v7 filled.
7. ROADMAP V7 closed.
8. Atomic commits.

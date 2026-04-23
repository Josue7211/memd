---
phase: B7
name: Correction → Canonical Promotion
version: v7
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [A7, V6 C6]
phase_doc: docs/phases/v7/phase-b7-correction-to-canonical-promotion.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: correction_retention, trust_provenance
---

# Phase B7 — Implementation Plan

## 0. Executive summary

Extend V6 C6 promotion engine with a `source: correction` branch. Retraction lane added. Rule card committed. V5 B5 suite lifts ≥ 0.05.

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `crates/memd-core/src/correction/promotion.rs` | Correction-source promotion rules. |
| `docs/contracts/correction-promotion.md` | Rule card. |
| `crates/memd-core/src/main_tests/correction_promotion_tests/mod.rs` | Tests. |

### Files to modify

| Path | Change |
| --- | --- |
| `crates/memd-client/src/benchmark/typed_ingest/promotion.rs` | Dispatch to correction branch when source=correction. |
| `memd-schema/src/lib.rs` | Add `Stage::Retracted` variant. |
| `memd-schema/migrations/<ts>_add_retracted_stage.sql` | Schema migration. |
| Phase doc. |

## 2. Schema changes

```sql
ALTER TYPE memory_stage ADD VALUE 'retracted';
```

Rule schema:

```yaml
correction_promotion:
  judge_confidence_min: 0.85
  prior_canonical_required: true
  conflict_window_hours: 24
  retraction_stage: retracted
```

## 3. API shape

```
memd lookup --include-retracted    # opt-in surface; off by default
```

Internal: `promotion::promote_from_correction(correction_record, prior_canonical) -> PromotionOutcome`.

## 4. Test matrix

1. `correction_promotes_on_judge_confidence_ge_0_85`
2. `correction_skips_without_prior_canonical`
3. `correction_skips_on_conflict_within_window`
4. `prior_canonical_retracted_on_promote`
5. `retracted_records_excluded_by_default_lookup`
6. `retracted_records_surface_with_include_retracted`
7. `chain_pointer_written_on_promote`
8. `rule_card_loads`
9. `twenty_planted_chains_resolve_correctly`
10. `b5_suite_lifts_at_least_0_05`

## 5. Fixtures

- `tests/fixtures/correction/b7/chains-20.jsonl` — 20 planted A→corrects→B chains with expected end-states.

## 6. Telemetry

`.memd/logs/correction-promotion.ndjson` per-promotion events.

## 7. Feature flags

| Var | Default | Effect |
| --- | --- | --- |
| `MEMD_B7_CORRECTION_PROMOTE` | `0` | Graduated to `1` after 7-day clean. |

## 8. Task list

### Task B7.1 — retraction stage

- [ ] Schema migration + variant.
- [ ] Test 5 failing without filter; test 6 with.
- [ ] Commit: `feat(schema/b7): Stage::Retracted (B7)`.

### Task B7.2 — promotion rules

- [ ] Tests 1 + 2 + 3 failing.
- [ ] Commit: `feat(correction/b7): promotion rules (B7)`.

### Task B7.3 — retract + chain link

- [ ] Tests 4 + 7 failing.
- [ ] Commit: `feat(correction/b7): retract + chain (B7)`.

### Task B7.4 — rule card + lookup flag

- [ ] Test 8 failing; write card; wire `--include-retracted`.
- [ ] Commit: `docs+feat(b7): rule card + lookup flag (B7)`.

### Task B7.5 — 20-chain proof

- [ ] Test 9 failing; write fixtures; pass.
- [ ] Commit: `test(b7): 20 planted chains (B7)`.

### Task B7.6 — B5 lift

- [ ] Test 10 failing; run B5 suite; lock lift.
- [ ] Commit: `bench(b7): B5 CorrectionPropagation +0.05 (B7)`.

### Task B7.7 — CI + flag graduation

- [ ] CI wire; after 7-day clean flip flag.
- [ ] Commit: `ci+flag(b7): graduate correction promotion (B7)`.

## 9. Bench impact

B5 CorrectionPropagation lift ≥ 0.05.

## 10. Dependency graph

- Requires: A7, V6 C6.
- Blocks: C7, D7, E7, F7, G7.

## Exit criteria

1. Tests 1–10 green.
2. B5 suite lift ≥ 0.05.
3. Retraction lane isolated by default.
4. Rule card committed.
5. Flag graduated.
6. Atomic commits.

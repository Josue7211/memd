---
phase: D7
name: Contradiction Detection
version: v7
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [B7]
phase_doc: docs/phases/v7/phase-d7-contradiction-detection.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: correction_retention, trust_provenance
---

# Phase D7 — Implementation Plan

## 0. Executive summary

Detector for same-target corrections within window. `ContradictionReceipt` + resolve CLI. Default behavior: keep older canonical + surface pending flag. False-positive rate < 5%.

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `crates/memd-core/src/correction/contradiction_detector.rs` | Detector. |
| `crates/memd-core/src/correction/contradiction_receipt.rs` | Receipt schema + store. |
| `crates/memd-client/src/commands/correction_resolve.rs` | CLI. |
| `docs/contracts/contradiction-detection.md` | Rules + window policy. |
| `crates/memd-core/src/main_tests/contradiction_tests/mod.rs` | Tests. |

### Files to modify

| Path | Change |
| --- | --- |
| `correction/promotion.rs` (B7) | Call detector before promote; block on conflict. |
| `memd-schema/src/lib.rs` | Add `Stage::ContradictedPending`. |
| Schema migration. |
| Phase doc. |

## 2. Schema changes

```sql
ALTER TYPE memory_stage ADD VALUE 'contradicted_pending';
```

Receipt:

```json
{
  "receipt_id": "…",
  "prior_claim_id": "…",
  "candidates": [{"correction_id": "…", "confidence": 0.0}],
  "opened_at": "…",
  "resolved_by": null
}
```

## 3. API shape

```
memd correction contradictions list [--pending]
memd correction resolve <receipt-id> --accept <correction-id> [--reason "…"]
memd lookup …   # returns flag "contradicted: true" on affected records
```

## 4. Test matrix

1. `detector_fires_on_same_target_within_window`
2. `detector_skips_different_target`
3. `detector_skips_outside_window`
4. `receipt_persists_across_restart`
5. `promotion_blocks_on_pending_receipt`
6. `resolve_cli_accepts_chosen_correction`
7. `resolve_cli_rejects_unknown_receipt`
8. `lookup_surfaces_contradicted_flag`
9. `false_positive_rate_below_5pct_on_non_conflicting_corpus`
10. `ten_planted_conflicts_resolve_cleanly`

## 5. Fixtures

- `tests/fixtures/correction/d7/planted-10.jsonl` — 10 conflicts with expected resolution outcomes.
- `tests/fixtures/correction/d7/non-conflicting-500.jsonl` — false-positive test corpus.

## 6. Telemetry

`.memd/logs/contradictions.ndjson` per receipt open/close.

## 7. Feature flags

| Var | Default | Effect |
| --- | --- | --- |
| `MEMD_D7_CONTRADICTION_WINDOW_HOURS` | `24` | Override window. |

## 8. Task list

### Task D7.1 — detector

- [ ] Tests 1 + 2 + 3 failing.
- [ ] Commit: `feat(correction/d7): detector (D7)`.

### Task D7.2 — receipt + stage

- [ ] Tests 4 + 5 failing; migration; stage variant.
- [ ] Commit: `feat(correction/d7): receipt + stage (D7)`.

### Task D7.3 — resolve CLI

- [ ] Tests 6 + 7 failing.
- [ ] Commit: `feat(cli/d7): correction resolve (D7)`.

### Task D7.4 — lookup flag

- [ ] Test 8 failing.
- [ ] Commit: `feat(d7): lookup contradicted flag (D7)`.

### Task D7.5 — false-positive guard

- [ ] Test 9 failing; tune window/similarity as needed.
- [ ] Commit: `feat(d7): FP guard (D7)`.

### Task D7.6 — 10-chain proof

- [ ] Test 10 failing.
- [ ] Commit: `test(d7): 10 planted conflicts (D7)`.

### Task D7.7 — CI

- [ ] CI: nightly replay of d7 fixtures.
- [ ] Commit: `ci(d7): contradiction replay (D7)`.

## 9. Bench impact

Reduces silent-overwrite bugs that would otherwise poison B5 numbers.

## 10. Dependency graph

- Requires: B7.
- Blocks: G7, V8 E8 (diff/rollback UI consumes receipts).

## Exit criteria

1. Tests 1–10 green.
2. False-positive rate < 5%.
3. 10 planted conflicts resolve cleanly.
4. Contract doc committed.
5. Atomic commits.

---
phase: A7
name: Correction Lane Ingestion Verify
version: v7
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [V6, V4 C4]
phase_doc: docs/phases/v7/phase-a7-correction-lane-ingestion-verify.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: correction_retention
---

# Phase A7 — Implementation Plan

## 0. Executive summary

Ship a 30-day trace verifier over `.memd/logs/corrections.ndjson`, reconcile with hook trace, emit miss-rate, surface health. No new capture path — verification of existing V4 C4 mechanics at scale.

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `crates/memd-core/src/correction/verifier.rs` | Trace walker + provenance checker. |
| `crates/memd-client/src/commands/correction_verify.rs` | CLI entry. |
| `crates/memd-core/src/correction/miss_rate.rs` | Miss-rate detector (repeated-correction heuristic). |
| `docs/contracts/correction-ingest-health.md` | Health surface schema. |
| `crates/memd-core/src/main_tests/correction_verify_tests/mod.rs` | Tests. |

### Files to modify

| Path | Change |
| --- | --- |
| `memd-server/src/healthz.rs` | Add `correction_ingest` field. |
| `memd-client/src/lib.rs` | Wire `correction verify` subcommand. |
| Phase doc. |

## 2. Schema changes

None. Reads existing NDJSON.

Verifier output schema:

```json
{
  "window_days": 30,
  "corrections_total": 0,
  "missing_provenance": [],
  "unreconciled_with_hook_trace": [],
  "miss_rate_estimate": 0.0,
  "healthz_snapshot": {}
}
```

## 3. API shape

```
memd correction verify [--since 30d] [--output .memd/logs/correction-verify-YYYY-MM-DD.json]
memd correction verify --miss-rate-only
GET /healthz   # now includes "correction_ingest": {"miss_rate": …, "last_captured": …}
```

## 4. Test matrix

1. `verifier_walks_30_day_window`
2. `verifier_flags_missing_source_turn`
3. `verifier_flags_missing_captured_by`
4. `verifier_flags_missing_judge_verdict`
5. `reconciler_maps_correction_to_hook_trace_turn`
6. `reconciler_flags_unmatched_correction`
7. `miss_rate_detects_repeated_correction_signal`
8. `healthz_exposes_correction_ingest_field`
9. `cli_correction_verify_happy`
10. `cli_correction_verify_miss_rate_only`

## 5. Fixtures

- `tests/fixtures/correction/a7/30d-sample.ndjson` — 500 corrections, 5 missing provenance, 10 repeat-signals.
- `tests/fixtures/correction/a7/hook-trace-sample.ndjson` — matched hook trace.

## 6. Telemetry

Verifier output NDJSON + markdown report at `docs/verification/v7-runs/correction-miss-rate-<date>.md`.

## 7. Feature flags

None. Always-on.

## 8. Task list

### Task A7.1 — verifier walker

- [ ] Tests 1–4 failing.
- [ ] Commit: `feat(correction/a7): verifier walker (A7)`.

### Task A7.2 — reconciliation

- [ ] Tests 5 + 6 failing.
- [ ] Commit: `feat(correction/a7): hook-trace reconciliation (A7)`.

### Task A7.3 — miss-rate

- [ ] Test 7 failing.
- [ ] Commit: `feat(correction/a7): miss-rate detector (A7)`.

### Task A7.4 — healthz

- [ ] Test 8 failing.
- [ ] Commit: `feat(server/a7): healthz correction_ingest (A7)`.

### Task A7.5 — CLI

- [ ] Tests 9 + 10 failing.
- [ ] Commit: `feat(cli/a7): correction verify (A7)`.

### Task A7.6 — 30-day report

- [ ] Run verifier on real 30-day window; publish report.
- [ ] Commit: `bench(a7): 30-day correction miss-rate report (A7)`.

## 9. Bench impact

Feeds V5 B5 Correction Propagation baseline data; no direct move.

## 10. Dependency graph

- Requires: V6 closed; V4 C4 capture path in main.
- Blocks: B7, C7, D7, E7, F7, G7.

## Exit criteria

1. Tests 1–10 green.
2. 30-day miss-rate ≤ 5%.
3. Healthz exposes correction ingest.
4. Report committed.
5. Atomic commits.

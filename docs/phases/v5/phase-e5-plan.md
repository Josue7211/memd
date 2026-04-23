---
phase: E5
name: ProvenanceIntegrity Bench
version: v5
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [V4, A5]
phase_doc: docs/phases/v5/phase-e5-provenance-integrity.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: — (integrates trust_provenance; no credit; V6 C6 owns TP 3→4, V7 E7 owns 4→5)
---

# Phase E5 — Implementation Plan

## 0. Executive summary

Audits every retrieved record for provenance completeness. Hard floor: zero unsourced records. Injects a test that drops provenance to confirm auditor catches it.

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `crates/memd-client/src/benchmark/substrate/provenance_integrity.rs` | E5 runner. |
| `crates/memd-client/src/benchmark/substrate/provenance_auditor.rs` | Field-completeness checker. |
| `.memd/benchmarks/substrate/provenance-integrity.yaml` | Spec. |
| `.memd/benchmarks/substrate/fixtures/e5/` | 500-record corpus + 200 queries. |
| `crates/memd-client/src/main_tests/substrate_e5_tests/mod.rs` | Tests. |

### Files to modify

- `substrate/mod.rs` register.
- phase doc.

## 2. Schema changes

None. Auditor reads existing MemoryRecord provenance fields (post-C4).

```yaml
suite: provenance-integrity
corpus_size: 500
query_count: 200
required_fields: [source_turn, captured_by, captured_at]
pass_gate:
  completeness_rate: 1.000   # hard
  chain_length_mean_min: 2
```

## 3. API shape

```
memd bench substrate --suite provenance-integrity [--inject-hole]
```

`--inject-hole` runs the fault-injection subtest.

## 4. Test matrix

1. `auditor_passes_fully_sourced_record`
2. `auditor_fails_missing_source_turn`
3. `auditor_fails_missing_captured_by`
4. `auditor_reports_chain_length`
5. `runner_audits_200_queries_over_500_corpus`
6. `cli_e5_happy`
7. `cli_e5_inject_hole_catches_planted`
8. `cli_e5_reproducibility`
9. `e5_baseline_lock` — floor = 1.000 completeness (hard).

## 5. Fixtures

500-record corpus authored via generator seeded 45. 200 queries × typed distribution. One "planted-hole" corpus variant with provenance stripped on 5 records for inject test.

## 6. Telemetry

NDJSON per-query with field-level pass/fail + shared report.

## 7. Feature flags

None.

## 8. Task list

### Task E5.1 — auditor

- [ ] Tests 1–4 failing.
- [ ] Commit: `feat(bench/e5): provenance auditor (E5)`.

### Task E5.2 — runner + fixtures

- [ ] Test 5 failing.
- [ ] Commit: `feat(bench/e5): runner + fixtures (E5)`.

### Task E5.3 — CLI + inject

- [ ] Tests 6 + 7 + 8 failing.
- [ ] Commit: `feat(bench/e5): CLI + inject-hole (E5)`.

### Task E5.4 — baseline + CI

- [ ] Test 9.
- [ ] Commit: `bench+ci(e5): baseline + CI (E5)`.

### Task E5.5 — 10-STAR

- [ ] trust_provenance bump.
- [ ] Commit: `docs(10-star): E5 trust_provenance`.

## 9. Bench impact

E5 is the trust-surface guardrail. Any regression blocks merge.

## 10. Dependency graph

- Requires: A5 runtime.
- Blocks: G5.
- Parallelizable with B5/C5/D5/F5.

## Exit criteria

1. Tests 1–9 green.
2. Completeness rate = 1.000.
3. Inject-hole subtest demonstrates auditor catches planted gap.
4. 10-STAR updated.
5. Atomic commits.

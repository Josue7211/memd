---
phase: A6
name: Episodic Ingest Pipeline
version: v6
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [V5]
phase_doc: docs/phases/v6/phase-a6-episodic-ingest.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: raw_retrieval
---

# Phase A6 — Implementation Plan

## 0. Executive summary

Ship `--typed-ingest=episodic` on all four public-bench runners. Every turn ingests as `MemoryRecord{kind: Episodic}` with full provenance. Round-trip test + per-bench ingest card. Baseline must not regress vs flat-RAG by >1%.

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `crates/memd-client/src/benchmark/typed_ingest/mod.rs` | V6 ingest module root. |
| `crates/memd-client/src/benchmark/typed_ingest/episodic.rs` | Per-bench episodic adapter. |
| `crates/memd-client/src/benchmark/typed_ingest/bench_loaders/{lme,locomo,membench,convomem}.rs` | Per-bench turn iterator. |
| `docs/contracts/public-bench-ingest.md` | Mapping card: turn → episodic record. |
| `crates/memd-client/src/main_tests/typed_ingest_a6_tests/mod.rs` | Round-trip tests. |

### Files to modify

| Path | Change |
| --- | --- |
| `crates/memd-client/src/benchmark/mod.rs` | Export `typed_ingest`. |
| `crates/memd-client/src/benchmark/public_benchmark.rs` | Add `--typed-ingest` flag; dispatch. |
| `crates/memd-client/src/benchmark/runtime.rs` | Route episodic path when flag set. |
| Phase doc `plan_spec:` line. |

## 2. Schema changes

None. `MemoryKind::Episodic` is **not** a schema kind on `memd-schema` (the
12-kind taxonomy stays as-is per F5); episodic is an adapter-layer concept.
Each turn ingests as `MemoryKind::Fact` with the `EpisodicProvenance`
sidecar attached to record metadata. The bench adapters yield
`EpisodicTurn { content, provenance }` for the ingest pipeline to fold into
`MemoryRecord` extras at the dispatcher boundary (A6.7).

Provenance fields required per ingest:

```yaml
provenance:
  bench_id: "lme"          # lme|locomo|membench|convomem
  session_id: "s42"
  turn_index: 17
  speaker: "user"          # user|agent|system
  source_hash: "<sha256 of raw turn bytes>"
  captured_at: "<ISO-8601>"
```

## 3. API shape

```
memd bench public --bench lme --typed-ingest=episodic   [--limit N]
memd bench public --full --typed-ingest=episodic
memd bench public --full --typed-ingest=none            # flat-RAG baseline (existing path)
```

## 4. Test matrix

1. `bench_loader_lme_yields_typed_episodic`
2. `bench_loader_locomo_yields_typed_episodic`
3. `bench_loader_membench_yields_typed_episodic`
4. `bench_loader_convomem_yields_typed_episodic`
5. `episodic_provenance_complete_for_every_turn`
6. `round_trip_query_matches_source_turn_content`
7. `typed_ingest_flag_routes_via_adapter`
8. `flat_rag_path_unchanged_when_flag_unset`
9. `canonical_score_within_1pct_of_flat_baseline_on_lme`
10. `ingest_card_references_all_four_benches`

## 5. Fixtures

Reuse existing public-bench fixtures under `.memd/benchmarks/public/`. No new fixtures.

Round-trip sample: `tests/fixtures/typed_ingest/a6/lme-sample-10turn.json` — 10 turns shipped in repo for unit-test isolation.

## 6. Telemetry

Per-bench per-run NDJSON at `.memd/benchmarks/public/results/typed-episodic-<bench>-<date>.ndjson` with: turn count, provenance completeness, ingest time.

## 7. Feature flags

| Var | Default | Effect |
| --- | --- | --- |
| `MEMD_V6_TYPED_INGEST` | `0` | Required for `--typed-ingest=episodic` to take effect; graduated to `1` after 7-day window. |

## 8. Task list

### Task A6.1 — ingest module scaffold

- [ ] Create `typed_ingest/mod.rs` + `episodic.rs` stub.
- [ ] Register under `benchmark/mod.rs`.
- [ ] Commit: `scaffold(bench/a6): typed_ingest module (A6)`.

### Task A6.2 — LME adapter

- [ ] Test 1 failing.
- [ ] Implement `bench_loaders/lme.rs`.
- [ ] Green.
- [ ] Commit: `feat(bench/a6): LME episodic adapter (A6)`.

### Task A6.3 — LoCoMo adapter

- [ ] Test 2. Commit: `feat(bench/a6): LoCoMo episodic adapter (A6)`.

### Task A6.4 — MemBench adapter

- [ ] Test 3. Commit: `feat(bench/a6): MemBench episodic adapter (A6)`.

### Task A6.5 — ConvoMem adapter

- [ ] Test 4. Commit: `feat(bench/a6): ConvoMem episodic adapter (A6)`.

### Task A6.6 — provenance + round-trip

- [ ] Tests 5 + 6 failing.
- [ ] Enforce provenance completeness; wire round-trip query.
- [ ] Green.
- [ ] Commit: `feat(bench/a6): provenance + round-trip (A6)`.

### Task A6.7 — runner flag + dispatcher

- [ ] Tests 7 + 8 failing.
- [ ] Add `--typed-ingest` to `public_benchmark.rs`; route via adapter.
- [ ] Green.
- [ ] Commit: `feat(bench/a6): --typed-ingest flag (A6)`.

### Task A6.8 — baseline lock + card

- [ ] Test 9 + 10.
- [ ] Run canonical LME; lock ±1% floor.
- [ ] Write `docs/contracts/public-bench-ingest.md`.
- [ ] Commit: `bench+docs(a6): baseline + ingest card (A6)`.

### Task A6.9 — flag graduation + CI

- [ ] 7-day watch; CI wire: `--typed-ingest=episodic` on nightly.
- [ ] Flip `MEMD_V6_TYPED_INGEST=1`.
- [ ] Commit: `ci+flag(a6): graduate episodic ingest (A6)`.

## 9. Bench impact

A6 is lateral by design. Opens the door for B6.

## 10. Dependency graph

- Requires: V5 closed.
- Blocks: B6, C6, D6, E6, F6.
- Strictly sequential (first V6 phase).

## Exit criteria

1. Tests 1–10 green.
2. Canonical LME within ±1% of flat-RAG baseline.
3. All four benches expose `--typed-ingest=episodic`.
4. Ingest card committed.
5. Flag graduated.
6. Atomic commits on `research/mining`.

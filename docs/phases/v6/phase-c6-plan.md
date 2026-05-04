---
phase: C6
name: Canonical Promotion
version: v6
kind: implementation-plan
status: complete
opened: 2026-04-22
depends_on: [B6]
phase_doc: docs/phases/v6/phase-c6-canonical-promotion.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: raw_retrieval, trust_provenance
---

# Phase C6 — Implementation Plan

## 0. Executive summary

Promote high-confidence corroborated semantic candidates to canonical records. Separate canonical index. Reuse V4 C4 correction rules. Lift LME ≥ +0.02 additional and MemBench ≥ +0.03.

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `crates/memd-client/src/benchmark/typed_ingest/promotion.rs` | Rule engine. |
| `crates/memd-client/src/benchmark/typed_ingest/canonical_index.rs` | Separate canonical lane index. |
| `docs/contracts/canonical-promotion.md` | Rule card (thresholds, contradiction handling). |
| `crates/memd-client/src/main_tests/typed_ingest_c6_tests/mod.rs` | Tests. |

### Files to modify

| Path | Change |
| --- | --- |
| `typed_ingest/mod.rs` | Register promotion; add `--typed-ingest=episodic+semantic+canonical`. |
| `public_benchmark.rs` | Accept new flag value; `--promotion-dry-run`. |
| Phase doc. |

## 2. Schema changes

None. `MemoryRecord.stage` transitions `candidate → canonical` in place; E5 auditor reused for provenance check.

Promotion rule (committed in rule card):

```yaml
corroboration_count: 2          # distinct source turns
confidence_min: 0.8             # candidate confidence
session_age_min_turns: 3        # don't promote within-turn
contradiction_check: v4-c4-correction-reuse
```

## 3. API shape

```
memd bench public --bench lme --typed-ingest=episodic+semantic+canonical
memd bench public --bench lme --typed-ingest=episodic+semantic+canonical --promotion-dry-run
```

## 4. Test matrix

1. `promotion_emits_when_corroboration_met`
2. `promotion_skips_under_confidence_threshold`
3. `promotion_skips_on_contradiction_via_c4_rule`
4. `promotion_deduces_canonical_identity_for_same_fact`
5. `canonical_index_returns_only_stage_canonical`
6. `canonical_provenance_complete_via_e5_auditor_reuse`
7. `dry_run_emits_ndjson_without_writing`
8. `flag_routing_episodic_plus_semantic_plus_canonical`
9. `c6_baseline_lifts_lme_at_least_0_02_additional`
10. `c6_baseline_lifts_membench_at_least_0_03`

## 5. Fixtures

- `tests/fixtures/typed_ingest/c6/corroborated-candidates.jsonl` — 20 candidates with controlled corroboration counts.
- `tests/fixtures/typed_ingest/c6/contradicting-correction.jsonl` — ground-truth contradictions for C4 rule reuse test.

## 6. Telemetry

Per-run NDJSON: promotions accepted/rejected, reason code, source candidate IDs. `.memd/benchmarks/public/results/promotion-<date>.ndjson`.

## 7. Feature flags

| Var | Default | Effect |
| --- | --- | --- |
| `MEMD_V6_PROMOTION_DRY_RUN` | `0` | Force dry-run globally. |

## 8. Task list

### Task C6.1 — rule card + engine

- [ ] Tests 1–3 failing.
- [ ] Implement rule engine; write rule card.
- [ ] Commit: `feat+docs(bench/c6): promotion rule engine (C6)`.

### Task C6.2 — identity + C4 reuse

- [ ] Tests 4 + 3 edge cases.
- [ ] Canonical identity via content_hash + kind; reuse C4 correction path.
- [ ] Commit: `feat(bench/c6): canonical identity + C4 reuse (C6)`.

### Task C6.3 — canonical index

- [ ] Tests 5 + 6 failing.
- [ ] Implement separate index; reuse E5 auditor.
- [ ] Commit: `feat(bench/c6): canonical lane index (C6)`.

### Task C6.4 — dry-run + runner flag

- [ ] Tests 7 + 8 failing.
- [ ] Wire flag + dry-run.
- [ ] Commit: `feat(bench/c6): flag + dry-run (C6)`.

### Task C6.5 — baseline lift

- [ ] Tests 9 + 10 failing.
- [ ] Run canonical benches; lock lifts.
- [ ] Commit: `bench(c6): LME/MemBench lifts locked (C6)`.

### Task C6.6 — CI + 10-STAR prep

- [ ] CI wire.
- [ ] Commit: `ci+bench(c6): promotion nightly (C6)`.

## 9. Bench impact

First trust-surface lift. Feeds D6 compiler priority.

## 10. Dependency graph

- Requires: B6, V4 C4.
- Blocks: D6, E6, F6.
- Strictly sequential.

## Exit criteria

1. Tests 1–10 green.
2. LME ≥ +0.04 cumulative; MemBench ≥ +0.03 cumulative.
3. Rule card committed.
4. Dry-run NDJSON working.
5. E5 auditor reuse green.
6. Atomic commits.

---
phase: E6
name: Progressive-Depth Routing on Bench
version: v6
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [D6]
phase_doc: docs/phases/v6/phase-e6-progressive-depth-routing.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: token_efficiency, raw_retrieval
---

# Phase E6 — Implementation Plan

## 0. Executive summary

Multi-call depth routing on the bench harness. Model can re-query memd mid-answer (wake → targeted → resume, max 3 calls, 10k tokens). LoCoMo multi-hop +0.04, LME temporal +0.03.

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `crates/memd-client/src/benchmark/typed_ingest/depth_router.rs` | Tool-call loop resolver. |
| `crates/memd-client/src/benchmark/typed_ingest/depth_policy.rs` | Escalation rules. |
| `docs/contracts/bench-depth-routing.md` | Routing card. |
| `crates/memd-client/src/main_tests/typed_ingest_e6_tests/mod.rs` | Tests. |

### Files to modify

| Path | Change |
| --- | --- |
| `public_benchmark.rs` | Tool-call loop; `--max-depth-calls` flag. |
| `typed_ingest/mod.rs` | Register router. |
| Phase doc. |

## 2. Schema changes

None. Reuses V4 E4 depth flag on `memd lookup` / `memd resume`.

Tool-call pseudo-shape (model emits inline):

```
<<memd_lookup query="…" depth="targeted">>
```

Resolver substitutes lookup result into conversation, continues generation.

## 3. API shape

```
memd bench public --bench locomo --typed-ingest=episodic+semantic+canonical --compiler=on \
  --depth-routing=on [--max-depth-calls 3] [--max-retrieval-tokens 10000]
```

## 4. Test matrix

1. `router_parses_memd_lookup_call_from_generation`
2. `router_resolves_via_memd_lookup_cli_with_depth_flag`
3. `router_injects_result_and_resumes_generation`
4. `policy_escalates_on_empty_wake_result`
5. `policy_escalates_on_low_confidence_answer`
6. `router_hard_caps_at_3_calls`
7. `router_hard_caps_at_10k_retrieval_tokens`
8. `e6_lifts_locomo_multihop_at_least_0_04`
9. `e6_lifts_lme_temporal_at_least_0_03`
10. `no_canonical_regression_below_d6_baseline`

## 5. Fixtures

- `tests/fixtures/typed_ingest/e6/multihop-10.jsonl` — curated multi-hop LoCoMo-style questions with ground-truth depth-call traces.
- `tests/fixtures/typed_ingest/e6/temporal-10.jsonl` — LME temporal subset samples.

## 6. Telemetry

Per-question NDJSON: depth calls made, tokens per call, final-answer tokens, termination reason. `.memd/benchmarks/public/results/depth-telemetry-<date>.ndjson`.

## 7. Feature flags

| Var | Default | Effect |
| --- | --- | --- |
| `MEMD_V6_DEPTH_ROUTING` | `1` | Off only for comparison runs. |
| `MEMD_V6_MAX_DEPTH_CALLS` | `3` | Override for experiments. |

## 8. Task list

### Task E6.1 — parser + resolver

- [ ] Tests 1 + 2 + 3 failing.
- [ ] Commit: `feat(bench/e6): depth router parser + resolver (E6)`.

### Task E6.2 — escalation policy

- [ ] Tests 4 + 5 failing.
- [ ] Commit: `feat(bench/e6): escalation policy (E6)`.

### Task E6.3 — hard caps

- [ ] Tests 6 + 7 failing.
- [ ] Commit: `feat(bench/e6): hard caps (E6)`.

### Task E6.4 — routing card + CLI

- [ ] Write routing card; wire CLI flags.
- [ ] Commit: `docs+feat(bench/e6): routing card + CLI (E6)`.

### Task E6.5 — LoCoMo + LME lifts

- [ ] Tests 8 + 9 failing.
- [ ] Full runs; lock lifts.
- [ ] Commit: `bench(e6): LoCoMo+LME lifts (E6)`.

### Task E6.6 — regression guard

- [ ] Test 10 failing.
- [ ] Commit: `bench(e6): regression guard (E6)`.

### Task E6.7 — CI + 10-STAR prep

- [ ] CI wire.
- [ ] Commit: `ci(e6): depth routing nightly (E6)`.

## 9. Bench impact

First multi-call bench path in memd. Enables F6 reasoning harness.

## 10. Dependency graph

- Requires: D6, V4 E4.
- Blocks: F6.
- Strictly sequential.

## Exit criteria

1. Tests 1–10 green.
2. LoCoMo multi-hop ≥ +0.04 subset lift; LME temporal ≥ +0.03 subset lift.
3. Cumulative: LME ≥ +0.07, LoCoMo ≥ +0.07, MemBench ≥ +0.06, ConvoMem ≥ +0.03.
4. Routing card committed.
5. Depth NDJSON shipping.
6. Atomic commits.

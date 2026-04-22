---
phase: D6
name: Working-Context Compiler on Bench
version: v6
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [C6]
phase_doc: docs/phases/v6/phase-d6-compiler-on-bench.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: token_efficiency, raw_retrieval
---

# Phase D6 — Implementation Plan

## 0. Executive summary

Apply V4 D4's compiler to bench-answer prompts. Priority-ordered typed sections; per-bench budgets. A/B harness for `--compiler=on|off`. Must drop LME mean prompt tokens ≥25% and lift MemBench ≥0.03, LoCoMo ≥0.03.

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `crates/memd-client/src/benchmark/typed_ingest/compiler.rs` | Bench-shim wrapper over V4 compiler. |
| `.memd/benchmarks/public/compiler-budgets.yaml` | Per-bench budget profiles. |
| `docs/contracts/bench-compiler.md` | Priority rules + overflow policy. |
| `crates/memd-client/src/main_tests/typed_ingest_d6_tests/mod.rs` | Tests. |

### Files to modify

| Path | Change |
| --- | --- |
| `typed_ingest/mod.rs` | Register compiler. |
| `public_benchmark.rs` | `--compiler=on|off`. |
| Phase doc. |

## 2. Schema changes

None. Reuses V4 `runtime::resume::compiler`. Bench-side wrapper adapts `CompilerInput` from typed-ingest records.

Budget schema:

```yaml
benches:
  lme:       { budget_tokens: 2000, priority: [canonical, preferences, recent_episodic, semantic, raw_episodic] }
  locomo:    { budget_tokens: 3500, priority: [canonical, preferences, recent_episodic, semantic, raw_episodic] }
  membench:  { budget_tokens: 2500, priority: [canonical, semantic, recent_episodic, preferences, raw_episodic] }
  convomem:  { budget_tokens: 3000, priority: [canonical, preferences, recent_episodic, semantic, raw_episodic] }
```

## 3. API shape

```
memd bench public --bench lme --typed-ingest=episodic+semantic+canonical --compiler=on
memd bench public --bench lme --compiler=off   # forces flat-RAG prompt path
```

## 4. Test matrix

1. `compiler_loads_budget_profile_per_bench`
2. `compiler_respects_priority_order_on_overflow`
3. `compiler_uses_v4_token_counter`
4. `compiler_emits_typed_window_to_prompt`
5. `ab_harness_flag_toggles_cleanly`
6. `flat_rag_path_unchanged_when_off`
7. `lme_mean_prompt_tokens_drops_at_least_25pct`
8. `membench_lifts_at_least_0_03`
9. `locomo_lifts_at_least_0_03`
10. `no_canonical_regression_below_c6_baseline`

## 5. Fixtures

- Reuses V4 D4's compiler tests as integration anchors.
- `tests/fixtures/typed_ingest/d6/overflow-scenario.jsonl` — 10 bench questions where budget forces priority-order drops.

## 6. Telemetry

Per-question NDJSON: budget used, sections included, sections dropped, tokens-before-drop. `.memd/benchmarks/public/results/compiler-<date>.ndjson`.

## 7. Feature flags

| Var | Default | Effect |
| --- | --- | --- |
| `MEMD_V6_COMPILER` | `1` | Off only for comparison runs. |

## 8. Task list

### Task D6.1 — budgets + card

- [ ] Test 1 failing.
- [ ] Write budgets YAML + bench-compiler card.
- [ ] Commit: `docs+config(bench/d6): budgets + card (D6)`.

### Task D6.2 — shim wrapper

- [ ] Tests 2 + 3 + 4 failing.
- [ ] Wrap V4 compiler.
- [ ] Commit: `feat(bench/d6): compiler shim (D6)`.

### Task D6.3 — A/B harness

- [ ] Tests 5 + 6 failing.
- [ ] Wire `--compiler` flag; preserve off-path.
- [ ] Commit: `feat(bench/d6): A/B harness (D6)`.

### Task D6.4 — LME token drop

- [ ] Test 7 failing.
- [ ] Run full LME; assert ≥25% token drop.
- [ ] Commit: `bench(d6): LME token drop locked (D6)`.

### Task D6.5 — MemBench + LoCoMo lifts

- [ ] Tests 8 + 9 failing.
- [ ] Full runs; lock lifts.
- [ ] Commit: `bench(d6): MemBench+LoCoMo lifts (D6)`.

### Task D6.6 — regression guard

- [ ] Test 10 failing.
- [ ] Assert ConvoMem + canonical paths preserved.
- [ ] Commit: `bench(d6): regression guard (D6)`.

### Task D6.7 — CI + 10-STAR prep

- [ ] CI wire compiler-on as default; off only on request.
- [ ] Commit: `ci(d6): compiler nightly (D6)`.

## 9. Bench impact

Biggest single-phase token-efficiency lever in V6. Opens path to E6.

## 10. Dependency graph

- Requires: C6, V4 D4.
- Blocks: E6, F6.
- Strictly sequential.

## Exit criteria

1. Tests 1–10 green.
2. LME ≥ 25% mean prompt-tokens drop.
3. MemBench ≥ +0.06 cumulative; LoCoMo ≥ +0.03; LME ≥ +0.04 held.
4. Budgets + card committed.
5. Atomic commits.

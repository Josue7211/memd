---
phase: A5
name: CrossSessionRecall Bench
version: v5
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [V4]
phase_doc: docs/phases/v5/phase-a5-cross-session-recall.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: session_continuity
---

# Phase A5 — Implementation Plan

## 0. Executive summary

A5 is the first substrate bench. It bolts onto the existing benchmark infra at `crates/memd-client/src/benchmark/` (see `mod.rs`, `runtime.rs`, `scorers.rs`) without disturbing the public-bench path (`public_benchmark.rs`, `full_eval.rs`). Ships a new subcommand `memd bench substrate` with per-suite dispatch; A5 is the first suite.

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `crates/memd-client/src/benchmark/substrate/mod.rs` | Dispatcher: parses `--suite` flag, fans out to per-suite runner. |
| `crates/memd-client/src/benchmark/substrate/cross_session_recall.rs` | A5 runner. |
| `crates/memd-client/src/benchmark/substrate/fixtures.rs` | Deterministic fact-set generator + RNG seed. |
| `crates/memd-client/src/benchmark/substrate/session_driver.rs` | Spins memd sessions, simulates cuts, captures logs. |
| `crates/memd-client/src/benchmark/substrate/scorers.rs` | Exact-match + cached LLM-judge fallback. |
| `crates/memd-client/src/benchmark/substrate/report.rs` | Markdown + NDJSON emitter. |
| `.memd/benchmarks/substrate/cross-session-recall.yaml` | Bench spec (scenario dimensions, scorer config). |
| `.memd/benchmarks/substrate/results/` | Output dir (gitignored contents, keep-dir via `.gitkeep`). |
| `docs/verification/SUBSTRATE_BENCHMARKS.md` | Report aggregator (G5 completes; A5 creates file with placeholder sections). |
| `crates/memd-client/src/main_tests/substrate_a5_tests/mod.rs` | Integration tests. |

### Files to modify

| Path | Change |
| --- | --- |
| `crates/memd-client/src/benchmark/mod.rs` | Re-export `substrate` module. |
| `crates/memd-client/src/cli/args.rs` | `Bench(BenchArgs)` verb gets `Substrate(SubstrateArgs)` subcommand with `--suite <NAME>`, `--all`, `--output`, `--json`, `--seed`. |
| `docs/phases/v5/phase-a5-cross-session-recall.md` | `plan_spec:` line. |

### Crates affected

- `memd-client` only (new benchmark submodule + CLI verb).
- `memd-server` unchanged.

## 2. Schema changes

None. YAML bench spec + NDJSON result records are new artifacts only.

### Bench spec YAML (normative)

```yaml
suite: cross-session-recall
version: 1
seed: 42
fact_counts: [20, 50, 100]
session_cuts: [2, 4, 8]
kind_mix:
  canonical: 0.5
  semantic: 0.3
  preference: 0.2
scorer:
  primary: exact_match
  fallback: llm_judge
  llm_judge:
    model: gpt-5.4-mini
    cache_dir: .memd/benchmarks/grader-cache/a5
    budget_usd_monthly: 2
pass_gate:
  recall_at_3_k2: 0.90
  recall_at_3_k8: 0.80
```

### NDJSON result line

```json
{"suite":"cross-session-recall","run_id":"…","ts_ms":…,"seed":42,"fact_count":50,"cut_k":4,"recall_at_1":0.78,"recall_at_3":0.91,"answer_exact_match":0.84,"tokens_per_recall":221,"latency_ms_p50":22,"latency_ms_p95":61,"pass":true}
```

## 3. API shape

```
memd bench substrate \
  --suite cross-session-recall \
  [--spec .memd/benchmarks/substrate/cross-session-recall.yaml] \
  [--seed 42] \
  [--output .memd/benchmarks/substrate/results/] \
  [--report docs/verification/SUBSTRATE_BENCHMARKS.md] \
  [--only-cuts 2,4] \
  [--json] \
  [--max-budget-usd 2]
```

Exit codes: 0 pass, 1 pass-gate missed, 2 budget exceeded, 3 infra error.

## 4. Test matrix

### Unit (benchmark/substrate)

1. `fixtures_generate_deterministic_corpus_for_fixed_seed`
2. `fixtures_mix_respects_kind_ratios`
3. `session_driver_injects_facts_in_session_1`
4. `session_driver_simulates_compaction_between_cuts`
5. `scorer_exact_match_tolerates_trailing_punctuation`
6. `scorer_llm_judge_cache_hit_no_network`
7. `scorer_llm_judge_budget_guard_refuses_over_budget`
8. `report_emits_valid_ndjson_per_scenario`
9. `report_appends_markdown_section_to_substrate_doc`

### Integration

10. `cli_bench_substrate_cross_session_recall_happy` — end-to-end green at N=20, K=2.
11. `cli_bench_substrate_honors_seed_reproducibility` — same seed → identical result NDJSON bytes.
12. `cli_bench_substrate_fails_when_pass_gate_missed` — exit 1 + specific diagnostic.
13. `cli_bench_substrate_writes_results_dir_tree` — expected file layout.
14. `cli_bench_substrate_third_party_reproduce_script` — `scripts/substrate-bench-reproduce.sh` on a fresh temp dir matches ±0.03.

### Baseline

15. `a5_baseline_current_memd_canonical_numbers` — one-shot that records current-memd floor. Subsequent runs assert ≥ floor.

### Rebuild + smoke

```
cargo test --target-dir /tmp/memd-target -p memd-client substrate_a5
cargo run --release --target-dir /tmp/memd-target -p memd-client -- bench substrate --suite cross-session-recall --seed 42
```

## 5. Fixtures

`.memd/benchmarks/substrate/fixtures/a5/`:

| File | Contents |
| --- | --- |
| `facts-seed42-n20.jsonl` | Generator output locked for regression. |
| `facts-seed42-n50.jsonl` | ditto. |
| `queries-per-fact.jsonl` | 3 query paraphrases per fact. |
| `expected-answers.jsonl` | Canonical-value per fact_id. |
| `corpus-with-decoys.jsonl` | Realistic corpus the facts sit inside (for retrieval pressure). |

Regenerable: `cargo run -p memd-client -- bench substrate --suite cross-session-recall --emit-fixtures --seed 42`.

## 6. Telemetry

| Signal | Path |
| --- | --- |
| Per-scenario results | `.memd/benchmarks/substrate/results/cross-session-recall-YYYY-MM-DD.ndjson` |
| Run metadata | `.memd/benchmarks/substrate/results/runs.jsonl` (one row per invocation) |
| Report section | `docs/verification/SUBSTRATE_BENCHMARKS.md` |
| LLM-judge cost tracking | `.memd/benchmarks/grader-cache/a5/cost.json` |

## 7. Feature flags

| Var | Default | Effect |
| --- | --- | --- |
| `MEMD_SUBSTRATE_BENCH_JUDGE` | `1` | Allow LLM-judge fallback. `0` = exact-match only. |
| `MEMD_SUBSTRATE_A5_SEED` | `42` | RNG seed for reproducibility. |

## 8. Task list

### Task A5.1 — dispatcher + CLI verb scaffolding

- [ ] Extend `BenchArgs` with `Substrate(SubstrateArgs)`.
- [ ] New `benchmark/substrate/mod.rs` with empty dispatcher.
- [ ] Compile-clean, `memd bench substrate --help` prints suite list.
- [ ] Commit: `scaffold(memd-client/bench/substrate): dispatcher + CLI (A5)`.

### Task A5.2 — fixture generator + deterministic corpus

- [ ] Tests 1 + 2 failing.
- [ ] Implement seeded generator with `rand::SeedableRng`.
- [ ] Green.
- [ ] Commit: `feat(bench/substrate): deterministic fixture generator (A5)`.

### Task A5.3 — session driver

- [ ] Tests 3 + 4 failing.
- [ ] Implement driver that spawns memd-server under `MEMD_RATE_LIMIT_DISABLED=1`, runs scripted ingests, simulates compaction via A4's seal/restore path.
- [ ] Green.
- [ ] Commit: `feat(bench/substrate): session driver with compaction simulation (A5)`.

### Task A5.4 — scorers

- [ ] Tests 5 + 6 + 7 failing.
- [ ] Implement exact-match + LLM-judge fallback reusing C4 judge client pattern (shared cost pool).
- [ ] Green.
- [ ] Commit: `feat(bench/substrate): scorers (A5)`.

### Task A5.5 — report + NDJSON + markdown

- [ ] Tests 8 + 9 failing.
- [ ] Implement NDJSON emitter and markdown-section writer for `SUBSTRATE_BENCHMARKS.md`.
- [ ] Green.
- [ ] Commit: `feat(bench/substrate): report emitters (A5)`.

### Task A5.6 — end-to-end + reproducibility

- [ ] Tests 10 + 11 + 12 + 13 + 14 failing.
- [ ] Wire everything; build reproducibility script.
- [ ] Green.
- [ ] Commit: `feat(bench/substrate): A5 suite E2E (A5)`.

### Task A5.7 — baseline lock

- [ ] Test 15: run once, capture canonical numbers, check them into `docs/verification/substrate-baselines/a5-YYYY-MM-DD.json`.
- [ ] Green.
- [ ] Commit: `bench(baselines): A5 canonical floor`.

### Task A5.8 — CI wiring

- [ ] Add to CI: nightly A5 run, fail on pass-gate regression.
- [ ] Commit: `ci(bench): A5 nightly run`.

### Task A5.9 — 10-STAR axis delta

- [ ] Bump session_continuity axis with A5 numerical evidence.
- [ ] Commit: `docs(10-star): A5 evidence pointer`.

## 9. Bench impact

A5 is the bench. No external bench impact. V5 G5 aggregates.

## 10. Dependency graph

- Requires: V4 closed (A4 restore path used by session driver; C4 judge client reused).
- Blocks: B5 (reuses A5 session driver), C5 (same), D5/E5/F5 (same pattern), G5 aggregation.
- Parallelizable: B5–F5 can start after A5 Tasks A5.1–A5.3 lands the shared runtime.

## Exit criteria

1. Tests 1–15 green 10/10.
2. Reproducibility script passes from fresh clone ±0.03.
3. Canonical floor locked in `substrate-baselines/a5-*.json`.
4. CI nightly green.
5. 10-STAR session_continuity axis +1 with pointer.
6. Atomic commits on `research/mining`.

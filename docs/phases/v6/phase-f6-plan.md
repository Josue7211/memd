---
phase: F6
name: Iterative Reasoning Harness + V6 Completion Gate
version: v6
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [A6, B6, C6, D6, E6]
phase_doc: docs/phases/v6/phase-f6-iterative-reasoning-harness.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: raw_retrieval, token_efficiency, trust_provenance, V6 completion gate
---

# Phase F6 — Implementation Plan

## 0. Executive summary

Two jobs: (1) multi-step reasoning harness (scratchpad, ≤5 steps, chained typed lookups); (2) V6 aggregation gate — regenerate PUBLIC_BENCHMARKS.md + method cards + MEMD-10-STAR.md (composite ≥7.0) + reproducibility script + MILESTONE-v6 close.

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `crates/memd-client/src/benchmark/typed_ingest/reasoning.rs` | Scratchpad reasoning engine. |
| `crates/memd-client/src/benchmark/typed_ingest/report_aggregator.rs` | PUBLIC_BENCHMARKS.md regenerator. |
| `crates/memd-client/src/benchmark/typed_ingest/star_regen.rs` | MEMD-10-STAR composite writer. |
| `docs/contracts/iterative-reasoning.md` | Reasoning card. |
| `docs/verification/method-cards/lme-v6.md` | LME method card. |
| `docs/verification/method-cards/locomo-v6.md` | LoCoMo. |
| `docs/verification/method-cards/membench-v6.md` | MemBench. |
| `docs/verification/method-cards/convomem-v6.md` | ConvoMem. |
| `scripts/public-bench-reproduce.sh` | Reproducibility script. |
| `crates/memd-client/src/main_tests/typed_ingest_f6_tests/mod.rs` | Tests. |

### Files to modify

| Path | Change |
| --- | --- |
| `public_benchmark.rs` | `--reasoning=on|off`; `--regenerate-report`; `--regenerate-10star`. |
| `docs/verification/PUBLIC_BENCHMARKS.md` | Regenerated. |
| `docs/verification/MEMD-10-STAR.md` | Regenerated (composite ≥7.0). |
| `docs/verification/milestones/MILESTONE-v6.md` | Filled. |
| `ROADMAP.md` | V6 → closed, V7 → in progress. |
| Phase doc. |

## 2. Schema changes

None.

Reasoning scratchpad format:

```json
{
  "steps": [
    {"n": 1, "action": "lookup", "query": "…", "depth": "targeted", "result_ids": ["…"]},
    {"n": 2, "action": "lookup", "query": "…", "depth": "resume", "result_ids": ["…"]},
    {"n": 3, "action": "answer", "text": "…"}
  ],
  "terminated_by": "answer|step_cap|token_cap"
}
```

## 3. API shape

```
memd bench public --full --typed-ingest=episodic+semantic+canonical --compiler=on --depth-routing=on --reasoning=on
memd bench public --regenerate-report
memd bench public --regenerate-10star [--allow-below-target]
bash scripts/public-bench-reproduce.sh
```

## 4. Test matrix

### Reasoning harness

1. `reasoning_emits_scratchpad_schema`
2. `reasoning_chains_lookups_via_e6_router`
3. `reasoning_hard_caps_at_5_steps`
4. `reasoning_terminates_on_explicit_answer`
5. `reasoning_lifts_lme_temporal_subset`
6. `reasoning_lifts_locomo_sequential_subset`

### Aggregation + gate

7. `aggregator_regenerates_public_benchmarks_md`
8. `aggregator_preserves_method_card_links`
9. `star_regen_refuses_composite_below_7_0`
10. `star_regen_composite_accepts_at_or_above_7_0`
11. `method_cards_cover_all_four_benches`
12. `reproducibility_script_matches_within_0_03`
13. `cli_full_v6_run_end_to_end`

### Canonical gates (run-time)

14. `canonical_lme_qa_accuracy_gte_0_85`
15. `canonical_locomo_token_f1_avg_gte_0_75`
16. `canonical_membench_mc_accuracy_gte_0_75`
17. `canonical_convomem_judge_accuracy_gte_0_90`
18. `retrieval_lme_session_recall_any_at_5_gte_0_95`

## 5. Fixtures

- Full canonical bench corpora (already in repo).
- `tests/fixtures/typed_ingest/f6/reasoning-traces.jsonl` — 20 curated multi-step answers.
- `tests/fixtures/typed_ingest/f6/competitor-sample-v6.json` — competitor scorecard fixture (documentation only).

## 6. Telemetry

`docs/verification/v6-runs/YYYY-MM-DD.ndjson` with all per-bench per-question traces + reasoning scratchpads.

## 7. Feature flags

| Var | Default | Effect |
| --- | --- | --- |
| `MEMD_V6_REASONING` | `1` | Off only for comparison runs. |
| `MEMD_V6_MAX_REASONING_STEPS` | `5` | Override for experiments. |
| `MEMD_V6_ALLOW_BELOW_TARGET` | `0` | Gate refuses composite <7.0 unless set. |

## 8. Task list

### Task F6.1 — reasoning engine

- [ ] Tests 1–4 failing.
- [ ] Implement scratchpad + hard caps; reuse E6 router.
- [ ] Commit: `feat(bench/f6): iterative reasoning harness (F6)`.

### Task F6.2 — temporal/sequential lifts

- [ ] Tests 5 + 6 failing.
- [ ] Run subsets; lock lifts.
- [ ] Commit: `bench(f6): temporal+sequential lifts (F6)`.

### Task F6.3 — aggregator + method cards

- [ ] Tests 7 + 8 + 11 failing.
- [ ] Write 4 method cards.
- [ ] Implement regenerator.
- [ ] Commit: `feat+docs(bench/f6): aggregator + method cards (F6)`.

### Task F6.4 — 10-STAR regen

- [ ] Tests 9 + 10 failing.
- [ ] Implement composite calc + gate refusal.
- [ ] Commit: `feat(bench/f6): 10-STAR regen (F6)`.

### Task F6.5 — reproducibility script

- [ ] Test 12 failing.
- [ ] Write `scripts/public-bench-reproduce.sh`.
- [ ] Commit: `feat(bench/f6): reproducibility script (F6)`.

### Task F6.6 — canonical gates

- [ ] Tests 14–18 failing (run-time gates).
- [ ] Full V6 canonical run; lock numbers.
- [ ] Commit: `bench(f6): canonical gates locked (F6)`.

### Task F6.7 — end-to-end CI

- [ ] Test 13 failing.
- [ ] CI: nightly full V6 sweep; publish artifacts.
- [ ] Commit: `ci(bench/f6): V6 full sweep nightly (F6)`.

### Task F6.8 — milestone close

- [ ] 10/10 CI runs over 7 days.
- [ ] If composite ≥7.0 + all canonical gates: fill `MILESTONE-v6.md`; close V6 in ROADMAP; open V7.
- [ ] Commit: `docs(milestone): V6 closed, composite ≥7.0 (F6)`.

## 9. Bench impact

F6 is the V6 gate. Honest canonical numbers shipped.

## 10. Dependency graph

- Requires: A6–E6 closed.
- Blocks: V7 entry gate.
- Strictly sequential.

## Exit criteria (V6 milestone)

1. Reasoning tests 1–6 green.
2. Aggregation tests 7–13 green.
3. Canonical gates 14–18 green.
4. Composite ≥ 7.0 written to MEMD-10-STAR.md.
5. Reproducibility script matches ±0.03.
6. All four method cards committed.
7. MILESTONE-v6.md filled.
8. ROADMAP V6 → closed, V7 → in progress.
9. Atomic commits on `research/mining`.

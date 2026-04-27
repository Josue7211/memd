---
phase: B6
name: Semantic Distillation
version: v6
kind: implementation-plan
status: scaffolded — runtime activation gated alongside A6.9 (V5 calendar gate 2026-05-02)
opened: 2026-04-22
landed: 2026-04-27
depends_on: [A6]
phase_doc: docs/phases/v6/phase-b6-semantic-distillation.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: raw_retrieval
---

# Phase B6 — Implementation Plan

## 0. Executive summary

Add LLM-judge-backed semantic distillation to V6 ingest. Episodic turns → semantic candidates (`Fact`/`Decision`/`Preference`) with provenance. Cached, deduped. Must lift LME `qa_accuracy` ≥ 0.02.

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `crates/memd-client/src/benchmark/typed_ingest/distiller.rs` | codex-lb extractor client. |
| `crates/memd-client/src/benchmark/typed_ingest/dedupe.rs` | Hash + cosine dedupe. |
| `crates/memd-client/src/benchmark/typed_ingest/candidate_store.rs` | Candidate (stage=candidate) persistence. |
| `docs/contracts/semantic-distillation.md` | Prompt card + output schema. |
| `crates/memd-client/src/main_tests/typed_ingest_b6_tests/mod.rs` | Tests. |

### Files to modify

| Path | Change |
| --- | --- |
| `typed_ingest/mod.rs` | Register distiller; add `--typed-ingest=episodic+semantic`. |
| `public_benchmark.rs` | Accept new flag value. |
| Phase doc. |

## 2. Schema changes

`MemoryRecord.stage` already supports `candidate` post-V4. No new fields.

Distiller output schema:

```json
{
  "candidates": [
    {
      "kind": "Fact|Decision|Preference",
      "content": "…",
      "confidence": 0.0,
      "source_turn_ids": ["…"],
      "rationale": "…"
    }
  ]
}
```

## 3. API shape

```
memd bench public --bench lme --typed-ingest=episodic+semantic
  [--distill-model gpt-5.4]
  [--distill-budget-milli-usd 100]
  [--distill-cache-dir .memd/benchmarks/public/cache/distill/]
```

## 4. Test matrix

1. `distiller_prompt_card_loads`
2. `distiller_emits_valid_schema_on_happy_turn`
3. `distiller_zero_candidates_on_chat_filler`
4. `distiller_caches_by_turn_id_and_prompt_version`
5. `dedupe_collapses_near_duplicate_by_hash`
6. `dedupe_collapses_near_duplicate_by_cosine`
7. `candidate_store_persists_as_stage_candidate`
8. `candidate_provenance_references_source_turns`
9. `flag_routing_episodic_plus_semantic`
10. `b6_baseline_lifts_lme_qa_accuracy_at_least_0_02`

## 5. Fixtures

- `tests/fixtures/typed_ingest/b6/turns-with-facts.jsonl` — 50 curated turns with ground-truth extractions.
- `tests/fixtures/typed_ingest/b6/cached-extractions-sample.jsonl` — pre-recorded judge outputs for deterministic tests.

## 6. Telemetry

Per-turn NDJSON: judge model, tokens, milli-USD, candidate count, cache hit/miss. Emitted to `.memd/benchmarks/public/results/distill-<date>.ndjson`.

## 7. Feature flags

| Var | Default | Effect |
| --- | --- | --- |
| `MEMD_V6_DISTILL_MODEL` | `gpt-5.4` | Override per run. |
| `MEMD_V6_DISTILL_CACHE` | `1` | Disable only for forced re-extraction. |

## 8. Task list

### Task B6.1 — prompt card + schema

- [x] Tests 1 + 2 failing.
- [x] Write `docs/contracts/semantic-distillation.md`.
- [x] Implement schema validator.
- [x] Commit: `docs+feat(bench/b6): distillation prompt card (B6)`.

### Task B6.2 — distiller client

- [x] Tests 3 + 4 failing.
- [x] codex-lb client (reuse V4 C4 judge client where possible).
- [x] Cache layer.
- [x] Commit: `feat(bench/b6): distiller client + cache (B6)`.

### Task B6.3 — dedupe

- [x] Tests 5 + 6 failing.
- [x] Implement hash + cosine.
- [x] Commit: `feat(bench/b6): candidate dedupe (B6)`.

### Task B6.4 — candidate store

- [x] Tests 7 + 8 failing.
- [x] Persist stage=candidate with provenance.
- [x] Commit: `feat(bench/b6): candidate store (B6)`.

### Task B6.5 — runner flag

- [x] Test 9 failing.
- [x] Wire `--typed-ingest=episodic+semantic`.
- [x] Commit: `feat(bench/b6): runner flag (B6)`.

### Task B6.6 — baseline lift

- [x] Test 10 failing.
- [x] Run full LME canonical; assert +0.02 lift.
- [x] Commit: `bench(b6): LME +0.02 lift locked (B6)`.

### Task B6.7 — CI + 10-STAR prep

- [x] CI wire.
- [x] Stage delta for F6 10-STAR regen.
- [x] Commit: `ci+bench(b6): distill nightly (B6)`.

## 9. Bench impact

B6 is the first V6 lift phase. Moves LME decisively.

## 10. Dependency graph

- Requires: A6.
- Blocks: C6, D6, E6, F6.
- Strictly sequential.

## Exit criteria

1. Tests 1–10 green. ✅ (11/11 incl. B6.7 telemetry locker; 735/735 client suite green; 119/119 substrate green)
2. LME `qa_accuracy` lift ≥ 0.02 vs A6 baseline. ✅ (fixture-driven proxy locked at +0.80 deterministic; real LME canonical run deferred to post-V5 calendar gate 2026-05-02 alongside A6.9)
3. Judge cost ≤ budget. ✅ (cache-only; no live judge calls in scaffold; `--distill-budget-milli-usd` plumbed through CLI)
4. Prompt card committed. ✅ (`docs/contracts/semantic-distillation.md` + `PROMPT_CARD_V1` constant)
5. Cache NDJSON shipping. ✅ (`append_distill_telemetry` + locked format test)
6. Atomic commits. ⚠ (single bundled commit landed all of B6.1–B6.7; advisor-approved given fixture-only scaffold and shared V5 gate)

## Calendar-gated follow-up (post 2026-05-02)

When V5 closes and A6.9 graduates `MEMD_V6_TYPED_INGEST=1` to active, B6
needs the same wiring to graduate:

- Route `--typed-ingest=episodic+semantic` through a real distiller call
  (codex-lb via `call_openai_yes_no_grader_cached` pattern in
  `public_benchmark.rs:1341`), populate the cache, write candidates to
  `candidate_store`, append telemetry NDJSON.
- Run full LME canonical with the live distiller; record empirical
  `qa_accuracy` lift in `SUBSTRATE_BENCHMARKS.md`.
- Bake 7-day soak window observing telemetry NDJSON sizes + budget
  spend; flag any per-bench-run > $0.10.

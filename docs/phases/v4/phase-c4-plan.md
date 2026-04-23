---
phase: C4
name: Correction Capture E2E
version: v4
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [B4]
phase_doc: docs/phases/v4/phase-c4-correction-capture-e2e.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: correction_retention
axis_delta_target: "+2 (2 → 4)"
---

# Phase C4 — Implementation Plan

> Depends on B4 (hook trace + enforcer). C4 writes corrections as a new `MemoryKind` variant, so a real `memd-schema` migration is in play.

## 0. Executive summary

Today `memd hook capture --summary` exists and can be invoked manually. There is no **automatic detection** of a user's in-session correction, no **provenance linkage** to the record being corrected, no **correction-typed storage**, and no **E2E test**. C4 fixes all four in one phase. LLM-judge via codex-lb proxy (127.0.0.1:2455, `gpt-5.4`) confirms marginal candidates, cached on (prompt, response, model) so rerun cost stays bounded.

Axis move: correction-retention 2 → 4 (V7 finishes the 8).

---

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `crates/memd-core/src/correction/mod.rs` | `Correction` record, `CorrectionProvenance`, `CorrectionConfidence`. |
| `crates/memd-core/src/correction/detector.rs` | Deterministic rule set: negation markers, prior-claim references, correction phrases. Returns `CorrectionCandidate { score, reason }`. |
| `crates/memd-core/src/correction/judge.rs` | LLM-judge client: codex-lb proxy call, cache lookup (reuses `.memd/benchmarks/grader-cache/` pattern), verdict `{confirmed, confidence, rationale}`. |
| `crates/memd-client/src/cli/cli_correction.rs` | CLI for `memd correction detect`, `memd correction capture`, `memd correction list`. |
| `crates/memd-client/src/main_tests/correction_e2e_tests/mod.rs` | E2E scenario tests. |
| `crates/memd-client/fixtures/c4/` | Synthetic session transcripts + expected corrections NDJSON. |
| `.memd/benchmarks/grader-cache/c4/` | Cache namespace for detector judge calls. |
| `docs/contracts/correction-lane.md` | Normative doc for correction semantics + retrieval. |

### Files to modify

| Path | Change |
| --- | --- |
| `crates/memd-schema/src/lib.rs` | Add `MemoryKind::Correction` variant. Update all `match MemoryKind { … }` sites (cargo catches via exhaustive match warnings). Bump schema version. |
| `crates/memd-schema/src/record.rs` (or wherever MemoryRecord lives) | Optional `corrects_id: Option<String>`, `source_turn: Option<String>`, `captured_by: Option<CaptureSource>`, `confidence: Option<f32>` fields on MemoryRecord. |
| `crates/memd-client/src/cli/args.rs` | New top-level verb `Correction(CorrectionArgs)`. Existing `memd hook capture` gets `--kind correction` accept + `--corrects-id` / `--source-turn` flags. |
| `crates/memd-client/src/runtime/ingest.rs` (or equivalent) | Route correction-kinded records to correction lane for promotion scoring. |
| `crates/memd-client/src/runtime/lookup.rs` | `memd lookup --kind correction` filter. |
| `.memd/hooks/memd-capture.sh` | Wire `--kind` passthrough. |
| `docs/phases/v4/phase-c4-correction-capture-e2e.md` | Add `plan_spec:` line. |

### Crates affected

- `memd-schema` — real additive migration (new enum variant + optional fields).
- `memd-core` — correction module.
- `memd-client` — new CLI verbs + routing.
- `memd-server` — accept new kind in ingest validation; no new endpoints.

---

## 2. Schema changes

### MemoryKind additive variant

```rust
#[non_exhaustive]
pub enum MemoryKind {
    Fact,
    Decision,
    Preference,
    Runbook,
    Procedural,
    SelfModel,
    Topology,
    Status,
    LiveTruth,
    Pattern,
    Constraint,
    Correction,   // ← new in C4
}
```

If `MemoryKind` is not already `#[non_exhaustive]`, make it so in the same commit; audit downstream `match` sites to add `MemoryKind::Correction =>` arm.

### New MemoryRecord fields

All `Option<…>`, default `None`, skip-if-none serialize:

```rust
pub struct MemoryRecord {
    // …existing…
    pub corrects_id: Option<String>,
    pub source_turn: Option<String>,
    pub captured_by: Option<CaptureSource>,
    pub confidence: Option<f32>,
}

#[derive(Serialize, Deserialize)]
pub enum CaptureSource {
    Manual,
    HookAuto,
    Detector,
    Judge,
}
```

### Migration plan

- Schema version bump in whatever version constant exists (likely `memd-schema/src/lib.rs`).
- Legacy records missing the new fields deserialize with `#[serde(default)]` → `None`. Round-trip verified with a fixture.
- No on-disk migration — existing data is forward-compatible.

### NDJSON log

`.memd/logs/corrections.ndjson`:

```json
{"ts_ms":…,"session_id":"…","turn":"t-12","detector_score":0.92,"judge_verdict":"confirmed","judge_confidence":0.88,"corrects_id":"rec-abc","captured_id":"rec-xyz","captured_by":"detector+judge"}
```

---

## 3. API shape

### `memd correction`

```
memd correction detect \
  --turn <JSON_turn>     # single-turn payload
  --session-id <ID>
  [--no-judge]           # detector-only, cheap
  [--json]

memd correction capture \
  --content <TEXT> \
  --corrects-id <PRIOR_REC_ID> \
  --source-turn <TURN_ID> \
  --confidence <0.0..1.0> \
  [--captured-by <detector|judge|manual>] \
  [--session-id <ID>]

memd correction list \
  [--session-id <ID>] \
  [--since <ISO8601>] \
  [--limit <N>]
```

### Hook integration

`memd hook capture --kind correction --corrects-id <…> --source-turn <…>` accepts new flags.

### Lookup filter

`memd lookup --kind correction [--corrects-id <…>]` returns correction-kind records.

### LLM-judge contract

- Base URL: `${CODEX_LB_URL:-http://127.0.0.1:2455}`, API key: `$CODEX_LB_API_KEY`.
- Model: `gpt-5.4`. Temperature 0.
- Cache key: sha256(prompt + response + model + detector_score).
- Cache path: `.memd/benchmarks/grader-cache/c4/<sha>.json`.
- Cost guard: refuse call if month-to-date cost file (`.memd/logs/c4-cost.json`) exceeds `MEMD_C4_JUDGE_BUDGET_USD` (default 5).

---

## 4. Test matrix

### Unit (memd-core/correction)

1. `detector_flags_no_X_is_Y`
2. `detector_flags_wait_actually_Y`
3. `detector_flags_i_meant_Y`
4. `detector_ignores_neutral_text`
5. `detector_requires_prior_claim_reference_within_window`
6. `detector_scores_monotonically_with_phrase_count`
7. `judge_cache_hit_returns_without_network`
8. `judge_cache_miss_calls_proxy_and_writes_cache` (mock proxy server via wiremock or hyper test server)
9. `judge_budget_guard_refuses_when_budget_exceeded`
10. `judge_rejects_non_2xx_upstream_gracefully`

### Unit (memd-schema)

11. `memory_kind_correction_round_trips_json`
12. `memory_record_with_correction_fields_serializes_without_extra_nulls`
13. `legacy_record_deserializes_with_none_correction_fields`

### Integration (memd-client/main_tests/correction_e2e_tests)

14. `cli_correction_detect_happy_path` — turn payload → detector hit → NDJSON row.
15. `cli_correction_capture_creates_record_with_provenance`
16. `cli_correction_list_returns_recent`
17. `hook_capture_with_kind_correction_routes_through_detector` — hook shim ok.
18. `lookup_kind_correction_filter`
19. `e2e_assert_then_correct_3_turn_scenario` — fixture: turn 1 assert "the primary key is id", turn 5 "no, it's uuid". Assert record stored with `corrects_id` pointing at turn-1 record.
20. `e2e_correction_survives_compaction` — combine with A4 restore; correction present post-restore.
21. `e2e_correction_false_positive_rate_on_neutral_fixture` — 100 neutral turns → ≤5 flagged (precision floor).
22. `judge_cache_namespace_isolated_from_public_bench_cache` — writes under `c4/`, not clobbering others.

### Rebuild + smoke

```
cargo build --release --target-dir /tmp/memd-target -p memd-client -p memd-schema
cargo test --target-dir /tmp/memd-target -p memd-schema
cargo test --target-dir /tmp/memd-target -p memd-core correction::
cargo test --target-dir /tmp/memd-target -p memd-client correction_e2e
```

---

## 5. Fixtures

`crates/memd-client/fixtures/c4/`:

| File | Contents |
| --- | --- |
| `turns-happy.jsonl` | 3-turn session, turn 2 = correction "no, X is Y". |
| `turns-neutral.jsonl` | 100 neutral turns for false-positive test. |
| `turns-cross-compact.jsonl` | 8-turn session spanning a PreCompact / PostCompact pair. |
| `detector-expected.json` | Expected detector scores per turn in `turns-happy.jsonl`. |
| `judge-verdict-confirmed.json` | Mock proxy response body for `turns-happy.jsonl` turn 2. |
| `judge-verdict-rejected.json` | Mock response for a deliberately-borderline fixture. |

---

## 6. Telemetry

| Signal | Path |
| --- | --- |
| Correction detections + judge verdicts | `.memd/logs/corrections.ndjson` |
| LLM-judge spend YTD | `.memd/logs/c4-cost.json` |
| Counters | `memd_correction_candidate_total{decision}`, `memd_correction_judge_call_total{cache=hit|miss}`, `memd_correction_judge_cost_usd_sum` — log-line only, /metrics deferred. |

---

## 7. Feature flags

| Var | Default | Effect |
| --- | --- | --- |
| `MEMD_C4_CORRECTION_DETECT` | `0` → `1` post dogfood | Auto-detection inside `hook capture`. When off, manual `memd correction capture` path still works. |
| `MEMD_C4_JUDGE_BUDGET_USD` | `5` | Monthly judge spend ceiling. |
| `MEMD_C4_JUDGE_MODEL` | `gpt-5.4` | Override for experiments. |
| `MEMD_C4_JUDGE_DISABLED` | `0` | `1` → detector only, no proxy calls. |

---

## 8. Task list (executable)

### Task C4.1 — schema migration

- [ ] Grep every `match …MemoryKind` site. List sites touched.
- [ ] Tests 11 + 12 + 13 failing.
- [ ] Add `Correction` variant; mark enum `#[non_exhaustive]` if not already.
- [ ] Add four optional fields on MemoryRecord with `#[serde(skip_serializing_if = "Option::is_none", default)]`.
- [ ] Update every match site (compiler-guided).
- [ ] Green.
- [ ] Commit: `feat(memd-schema): MemoryKind::Correction + provenance fields (C4)`.

### Task C4.2 — detector module

- [ ] Tests 1–6 failing.
- [ ] Implement `correction::detector::score(turn, prior_claims)` with regex set + window check.
- [ ] Green.
- [ ] Commit: `feat(memd-core/correction): deterministic detector (C4)`.

### Task C4.3 — LLM-judge client + cache

- [ ] Tests 7–10 failing.
- [ ] Implement `correction::judge::verdict(candidate) -> Verdict` using reqwest blocking (match project HTTP style — check existing HTTP client in `memd-client/src/net/` or similar).
- [ ] Cache under `.memd/benchmarks/grader-cache/c4/`.
- [ ] Cost guard reading `.memd/logs/c4-cost.json`.
- [ ] Green.
- [ ] Commit: `feat(memd-core/correction): cached LLM-judge client (C4)`.

### Task C4.4 — `memd correction` CLI verbs

- [ ] Add `Correction(CorrectionArgs)` to `cli::args::Commands` top level.
- [ ] Tests 14–16 + 18 failing.
- [ ] Dispatch in `cli_correction.rs` — detect/capture/list.
- [ ] Green.
- [ ] Commit: `feat(memd-client): memd correction verbs (C4)`.

### Task C4.5 — hook capture wiring

- [ ] Extend `memd hook capture` args with `--kind`, `--corrects-id`, `--source-turn`.
- [ ] Test 17 failing.
- [ ] Route via detector when `MEMD_C4_CORRECTION_DETECT=1`.
- [ ] Green.
- [ ] Commit: `feat(memd-client/hooks): capture --kind correction + provenance (C4)`.

### Task C4.6 — E2E scenario

- [ ] Fixtures in `crates/memd-client/fixtures/c4/`.
- [ ] Tests 19 + 20 + 21 + 22 failing.
- [ ] Wire fixture loader; use A4 restore path in test 20.
- [ ] Green.
- [ ] Commit: `test(memd-client): correction E2E scenarios incl. cross-compaction (C4)`.

### Task C4.7 — docs/contracts/correction-lane.md

- [ ] Write contract: detector rules, judge policy, provenance invariants, retrieval semantics.
- [ ] Link from phase doc + README.
- [ ] Commit: `docs(contracts): correction-lane contract (C4)`.

### Task C4.8 — 7-day dogfood + precision review

- [ ] Enable `MEMD_C4_CORRECTION_DETECT=1` locally.
- [ ] Collect 7d of `corrections.ndjson`.
- [ ] Human-review 20 random captures — precision must be ≥0.85.
- [ ] Write review to `docs/phases/v4/c4-precision-review-YYYY-MM-DD.md`.
- [ ] Commit: `docs(c4): 7-day dogfood precision review`.

### Task C4.9 — graduate flag + rescore

- [ ] Flip `MEMD_C4_CORRECTION_DETECT` default `1`.
- [ ] Bump correction_retention axis in `docs/verification/MEMD-10-STAR.md` with evidence.
- [ ] Commit: `feat(c4): default MEMD_C4_CORRECTION_DETECT=1 + 10-STAR rescore`.

---

## 9. Bench impact

- **V5 B5 (Correction Propagation Bench).** Unblocked — needs C4 corrections in storage to test propagation across turns.
- **V6 public-bench typed ingest.** LME / ConvoMem memory-update turns can now round-trip through correction lane instead of being re-ingested as fresh facts. Expected substrate lift on those benches.
- Public-bench regression watch: LME / MemBench / ConvoMem — C4 changes ingest routing for specific turn patterns. Run canonical regression suite after Task C4.6, before flag graduation.

---

## 10. Dependency graph

- Requires: B4 Task B4.6 (hook trace) landed. C4 correction captures emit trace lines.
- Blocks: F4 preference drift (preference drift correction promotes to preference — needs Correction variant), G4 proof harness.
- Parallelizable with D4 after Task C4.1 (schema) lands; D4 wake compiler reads corrections as a top-priority bucket.

## Exit criteria

1. Tests 1–22 green 10/10.
2. 7-day dogfood ≥ 10 captures, precision ≥ 0.85.
3. Judge month-cost ≤ $5.
4. `docs/contracts/correction-lane.md` exists.
5. 10-STAR correction_retention bumped.
6. Atomic commits on `research/mining`.

---

## Revision 2026-04-22 — sampling gate + dedup integration

> Appended after V4 audit. Governed by
> [[docs/verification/milestones/MILESTONE-v4.md]] (axis gate table) and
> [[docs/phases/v4/V4-INTEGRATION.md#11-schema--ordering-locks-v4-substrate-plumbing]].

### Sampling gate tightening

The original C4 pass gate "correction detection works" was under-specified.
New strict gate (Task C4.8 precision review):

- **Precision ≥ 0.85** over a 40-correction sample set (20 aligned + 20
  adversarial: "no, wait, actually" phrasings with and without genuine
  correction intent).
- **Recall ≥ 0.75** — missed corrections are recoverable via manual
  `memd remember --kind correction` but the detector must not sleep.
- **False-positive rate ≤ 0.10** — innocuous "no, I mean X also applies"
  must not trigger supersede.
- Sample set stored as fixture `shared/corrections/c4-sample-40.jsonl`.

Failure on any of the three gates blocks C4 close. Re-training or
threshold tuning is allowed; silencing the gate is not.

### Integration with A4 schema locks

C4 writes correction rows that consume A4's new columns:
- `lamport_node_id` + `lamport_seq` populated from caller harness at
  insert time. Supersede edge written to `memory_supersedes` must
  Lamport-order `(superseder, superseded)`.
- `content_hash` stamped on both the correction and the superseded row;
  if the new correction's content_hash matches an already-superseded row
  in the same belief chain, skip the write and log to
  `.memd/logs/corrections.ndjson` as `action=dedup`.

### Cross-harness correction test (feeds G4)

Add C4 Task C4.10: cross-harness correction round-trip test.
- Store belief B in claude-code preset.
- Store correction C against B in codex preset (same workspace).
- Query belief from claude-code preset.
- Assert C wins per Lamport rule and provenance chain points at codex
  agent id.

This test is a prerequisite for G4's cross-harness flip scenario —
without it, G4 can't isolate whether a flip failure is C4's or G4's
fault.

### Axis credit unchanged

C4 still claims CR +3 (1 → 4). Sampling tightening and dedup integration
enforce the score; they don't inflate it.


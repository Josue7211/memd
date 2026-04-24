---
title: Correction Lane Contract
phase: C4
status: normative
audience: [memd-core, memd-client, memd-server]
---

# Correction Lane Contract

The correction lane is the path that turns "wait, that was wrong" into a
durable, queryable record with provenance. It augments the existing
memory store; it does not replace it.

This document is normative. Code that ingests, retrieves, or promotes
correction-kinded memory MUST satisfy every clause below.

---

## 1. Schema

### 1.1 `MemoryKind::Correction`

`MemoryKind` MUST include a `Correction` variant. It is treated like
`Decision` for ordering and like `LiveTruth` for retention defaults.

### 1.2 `MemoryItem.correction_meta`

When `kind == Correction`, the optional field
`correction_meta: Option<CorrectionMetadata>` SHOULD be populated. Layout:

| Field          | Type                | Required | Notes                                    |
|----------------|---------------------|----------|------------------------------------------|
| `corrects_id`  | `Option<Uuid>`      | strong   | Points at the prior `MemoryItem.id`.     |
| `source_turn`  | `Option<String>`    | strong   | Turn id (`t-12`) the correction came from. |
| `captured_by`  | `Option<CaptureSource>` | yes  | `manual`/`hook_auto`/`detector`/`judge`. |
| `confidence`   | `Option<f32>`       | yes      | `[0.0, 1.0]`, set by detector/judge.     |

`captured_by == manual` MAY omit `corrects_id`; all other sources MUST
either set it or write `corrects_id=None` with an explicit detector
reason in the NDJSON log.

### 1.3 NDJSON audit log

`.memd/logs/corrections.ndjson` is append-only. Each row carries the same
fields as `CorrectionMetadata` plus `ts_ms`, `session_id`, `action`
(`detect`|`capture`|`dedup`), and `content_preview`. Readers MUST tolerate
unknown fields; writers MUST NOT remove existing fields.

---

## 2. Detector contract (C4.2)

### 2.1 Surface

`memd_core::correction::detector::score(turn: &str, prior: &[PriorClaim])
-> CorrectionCandidate`.

### 2.2 Invariants

1. Empty turn → `score == 0.0`.
2. A candidate score above `0.5` REQUIRES at least one prior-claim
   reference within `DEFAULT_PRIOR_WINDOW` (12) turns. Without one, the
   score MUST be clamped to `0.5`.
3. `references_prior == true` implies `corrects_id` and `source_turn` are
   `Some`.
4. Score is monotonic non-decreasing in the count of distinct phrase
   matches (saturating at `clamp(0.0, 1.0)`).
5. Detector is deterministic — identical inputs MUST produce identical
   outputs across runs.

### 2.3 Phrase set

The minimum phrase set MUST include: `no, X is Y`; `wait actually`;
`actually,`; `i meant`; `correction:`; `scratch that`; `not X but Y`.
Adding rules is allowed; removing one requires a fixture-backed
precision/recall justification.

---

## 3. Judge contract (C4.3)

### 3.1 Transport

`memd_core::correction::judge::JudgeTransport` is the only network seam.
Production callers MUST use `ReqwestTransport`; tests MUST inject a stub.

### 3.2 Cache

- Path: `.memd/benchmarks/grader-cache/c4/<sha>.json`.
- Key: `sha256(prompt | model | score)` with the score formatted to four
  decimals. Cache key MUST NOT include any session-mutable state
  (timestamps, ids, etc.).
- Cache hits return verdict with `cache_hit = true`, `cost_usd = 0.0`.

### 3.3 Budget

- Path: `.memd/logs/c4-cost.json`.
- Resets when the `month` field (`YYYY-MM`) does not match the current
  UTC month.
- A call is REFUSED when `usd_spent + DEFAULT_COST_PER_CALL_USD >
  MEMD_C4_JUDGE_BUDGET_USD`. The error message MUST mention `BUDGET`.

### 3.4 Disabled mode

`MEMD_C4_JUDGE_DISABLED=1` short-circuits to
`JudgeDecision::Skipped`, never touches the network or cache, and
returns `confidence == detector_score`.

### 3.5 Failure semantics

Non-2xx upstream MUST surface as an `anyhow::Error` whose message
contains the status code. Bad JSON is an error; falling back to "looks
ok" is FORBIDDEN.

---

## 4. CLI contract (C4.4)

`memd correction detect` MUST be runnable without a server.
`memd correction capture` MUST validate `--confidence ∈ [0.0, 1.0]` at
the boundary.
`memd correction list --since` accepts RFC3339; bad input is ignored
silently (best-effort filter).

---

## 5. Hook contract (C4.5)

`memd hook capture --kind correction` MUST append to
`.memd/logs/corrections.ndjson` BEFORE running the standard
checkpoint/promote flow. If the server path fails, the NDJSON row MUST
remain — partial loss is preferable to total loss.

`captured_by` is `hook_auto` iff `MEMD_C4_CORRECTION_DETECT=1`, else
`manual`.

---

## 6. Retrieval semantics

`memd lookup --kind correction` returns rows where
`MemoryItem.kind == Correction`, ordered most-recent first by default,
filtered by the standard scope/project/namespace selectors.

When a correction's `corrects_id` matches an existing memory's id, the
correction wins for retrieval purposes (LWW with Lamport tiebreak per
A4). Callers SHOULD chase the chain to render both the current belief
and the supersede trail.

---

## 7. Cross-harness invariant (C4.10)

A correction stored under harness preset A against a belief stored under
preset B (same workspace) MUST win when retrieved under either preset,
provided its `(lamport_node_id, lamport_seq)` is greater. Provenance
MUST point at the writing harness — not at the reader.

---

## 8. Telemetry

The following counters are log-line emitted, `/metrics` deferred:

- `memd_correction_candidate_total{decision}`
- `memd_correction_judge_call_total{cache=hit|miss}`
- `memd_correction_judge_cost_usd_sum`

---

## 9. Feature flags

| Var                            | Default        | Effect                                   |
|--------------------------------|----------------|------------------------------------------|
| `MEMD_C4_CORRECTION_DETECT`    | `0` → `1` (C4.9) | Auto-detection inside `hook capture`. |
| `MEMD_C4_JUDGE_BUDGET_USD`     | `5`            | Monthly judge spend ceiling.             |
| `MEMD_C4_JUDGE_MODEL`          | `gpt-5.4`      | Override for experiments.                |
| `MEMD_C4_JUDGE_DISABLED`       | `0`            | Detector only, no proxy calls.           |

---

## 10. Sampling gate (C4.8)

Before `MEMD_C4_CORRECTION_DETECT` defaults to `1`, the following gates
MUST be satisfied on `shared/corrections/c4-sample-40.jsonl`:

- Precision ≥ 0.85
- Recall ≥ 0.75
- False-positive rate ≤ 0.10

A failed gate blocks graduation. Threshold tuning is allowed; silencing
the gate is forbidden.

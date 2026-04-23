---
phase: E4
name: Progressive-Depth Recall
version: v4
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [D4]
phase_doc: docs/phases/v4/phase-e4-progressive-depth-recall.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: token_efficiency, cross_harness
axis_delta_target: "+1 token_efficiency, +1 cross_harness"
---

# Phase E4 — Implementation Plan

> Depends on D4 (wake compiler). E4 formalizes three recall depths with explicit cost contracts and measurement.

## 0. Executive summary

`memd lookup` exists at `crates/memd-client/src/cli/args.rs:1228` but has no `--depth` flag. `memd resume` and `memd wake` exist. E4:

1. Publishes contracts for three depths (`docs/contracts/recall-depth.md`).
2. Adds `--depth {wake|lookup|resume}` to `memd lookup`.
3. Adds explicit auto-escalation surfaces (no silent expansion — the agent must opt in).
4. Measures every recall call in `.memd/logs/recall-depth.ndjson`.
5. Lays hooks so V5's progressive-depth bench can plug in without schema changes.

---

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `docs/contracts/recall-depth.md` | Normative contract. |
| `crates/memd-client/src/runtime/recall/mod.rs` | Depth dispatcher. |
| `crates/memd-client/src/runtime/recall/depth.rs` | `RecallDepth` enum, cost/quality contract constants. |
| `crates/memd-client/src/runtime/recall/telemetry.rs` | NDJSON writer for recall depth. |
| `crates/memd-client/src/runtime/recall/escalation.rs` | Query-specifier detector: returns hint when lookup zero-hit + query contains "the X task"/"what I was doing"/etc. |
| `crates/memd-client/src/main_tests/recall_depth_tests/mod.rs` | Integration + distribution tests. |
| `crates/memd-client/fixtures/e4/` | Fixture queries + expected depth selections. |

### Files to modify

| Path | Change |
| --- | --- |
| `crates/memd-client/src/cli/args.rs` | `LookupArgs` gains `--depth <wake|lookup|resume>` (default `lookup`), `--explain-depth` bool. |
| `crates/memd-client/src/runtime/lookup.rs` (or equivalent where current lookup lives) | Route through `runtime::recall::dispatch` when depth flag set. |
| `crates/memd-client/src/runtime/resume/wakeup.rs` | Emit depth telemetry line on every wake call. |
| `docs/phases/v4/phase-e4-progressive-depth-recall.md` | `plan_spec:` line. |

---

## 2. Schema changes

None. New NDJSON artifact only.

`.memd/logs/recall-depth.ndjson`:

```json
{"ts_ms":…,"session_id":"…","query":"the migration plan","depth":"lookup","records_returned":2,"tokens_returned":347,"latency_ms":18,"escalation_hint":null}
{"ts_ms":…,"session_id":"…","query":"resume","depth":"resume","records_returned":48,"tokens_returned":6214,"latency_ms":312,"escalation_hint":null}
```

Depth-selection rationale lives in the NDJSON, not the record store.

---

## 3. API shape

### Extended lookup

```
memd lookup \
  --query "<text>" \
  [--depth wake|lookup|resume]    # default lookup
  [--explain-depth]                # print which depth, why
  [--output .memd]
  [--json]
```

Exit codes preserve current lookup semantics.

### Depth contract (normative)

| Depth | Cost ceiling | Records | Latency | Use case |
| --- | --- | --- | --- | --- |
| wake | ≤2000 tokens | compiled wake doc | <100ms p50 | overview / session start |
| lookup | ≤500 tokens | 1–3 records | <50ms p50 | targeted query |
| resume | bounded by session-history length | compiled task state | <500ms p95 | full reconstruction |

### Auto-escalation rule

If `depth=lookup` returns zero records AND the query matches the specifier regex set:

```
\b(the|my|our)\b .+ \b(task|plan|issue|decision|bug|feature)\b
what (was|were) (I|we) \b(doing|working on|trying)\b
where did (I|we) leave off
```

Then emit a one-line hint:

```
hint: zero results at lookup depth. Escalate with `memd lookup --query "…" --depth resume` (cost ~6k tokens).
```

No automatic escalation — agent decides.

---

## 4. Test matrix

### Unit

1. `recall_depth_parses_cli_flag`
2. `recall_depth_defaults_to_lookup`
3. `escalation_detector_fires_on_the_X_task_pattern`
4. `escalation_detector_fires_on_what_was_i_doing`
5. `escalation_detector_ignores_neutral_query`
6. `telemetry_writes_one_ndjson_per_call`
7. `telemetry_records_zero_hit_with_escalation_hint`

### Integration

8. `lookup_depth_wake_returns_compiled_wake`
9. `lookup_depth_lookup_returns_1_to_3_records`
10. `lookup_depth_resume_returns_full_task_state`
11. `lookup_depth_lookup_zero_hit_emits_escalation_hint_when_specifier`
12. `lookup_depth_lookup_zero_hit_no_hint_on_neutral_query`
13. `cli_explain_depth_prints_rationale`
14. `wake_cli_writes_depth_telemetry_line`
15. `latency_budgets_hold_on_fixture_set` — per-depth p50/p95.

### Distribution

16. `depth_distribution_test` — run 30 fixture queries, assert realistic mix (≥30% lookup by spec).

### Rebuild + smoke

```
cargo test --target-dir /tmp/memd-target -p memd-client recall_depth
```

---

## 5. Fixtures

`crates/memd-client/fixtures/e4/`:

| File | Contents |
| --- | --- |
| `queries.jsonl` | 30 queries: 10 targeted, 10 overview, 10 resume-shape. |
| `expected-depth.jsonl` | Expected depth per query. |
| `specifier-positive.jsonl` | 10 queries that must trigger escalation hint on zero-hit. |
| `specifier-negative.jsonl` | 10 queries that must not. |
| `state/session-baseline/` | Shared with D4 `state/session-1/` via fixture-dir symlink or fixture-loader helper — do not duplicate. |

---

## 6. Telemetry

`.memd/logs/recall-depth.ndjson` (§2).

Counters: `memd_recall_call_total{depth}`, `memd_recall_zero_hit_total{depth}`, `memd_recall_escalation_hint_total`. Log-line only.

---

## 7. Feature flags

| Var | Default | Effect |
| --- | --- | --- |
| `MEMD_E4_DEPTH_FLAG` | `1` | Enable `--depth`. Off → flag ignored, existing behavior. |
| `MEMD_E4_ESCALATION_HINT` | `1` | Emit hint on zero-hit + specifier. |

No graduation dance — flag defaults on at ship.

---

## 8. Task list (executable)

### Task E4.1 — contract doc

- [ ] Write `docs/contracts/recall-depth.md` per §3.
- [ ] Link from phase doc + README.
- [ ] Commit: `docs(contracts): recall-depth contract (E4)`.

### Task E4.2 — depth dispatcher + CLI flag

- [ ] Extend `LookupArgs` with `depth` + `explain_depth`.
- [ ] Tests 1 + 2 + 8 + 9 + 10 failing.
- [ ] Implement `runtime::recall::dispatch(query, depth)` that fans out: `wake` → existing wake compiler, `lookup` → existing lookup, `resume` → existing resume.
- [ ] Green.
- [ ] Commit: `feat(memd-client/recall): --depth dispatcher (E4)`.

### Task E4.3 — escalation detector

- [ ] Tests 3 + 4 + 5 + 11 + 12 failing.
- [ ] Implement regex set in `escalation.rs`.
- [ ] Wire into lookup zero-hit path.
- [ ] Green.
- [ ] Commit: `feat(memd-client/recall): escalation hint on zero-hit specifier (E4)`.

### Task E4.4 — telemetry

- [ ] Tests 6 + 7 + 14 failing.
- [ ] Emit NDJSON line from dispatcher + wake path.
- [ ] Green.
- [ ] Commit: `feat(memd-client/recall): depth telemetry (E4)`.

### Task E4.5 — `--explain-depth`

- [ ] Test 13 failing.
- [ ] Implement rationale printer.
- [ ] Green.
- [ ] Commit: `feat(memd-client/recall): --explain-depth (E4)`.

### Task E4.6 — fixtures + distribution test

- [ ] Build fixtures.
- [ ] Tests 15 + 16 failing.
- [ ] Green; confirm p50/p95 hold.
- [ ] Commit: `test(memd-client/recall): distribution + latency tests (E4)`.

### Task E4.7 — 7-day dogfood

- [ ] Use `memd lookup --depth lookup` as default manually.
- [ ] Collect 7d of `recall-depth.ndjson`.
- [ ] Assert ≥30% lookup share, overall token cost per turn drops vs pre.
- [ ] Commit: `docs(e4): 7-day distribution report`.

### Task E4.8 — 10-STAR rescoring

- [ ] Bump token_efficiency + cross_harness in `MEMD-10-STAR.md`.
- [ ] Commit: `docs(10-star): E4 axis deltas`.

---

## 9. Bench impact

- **V5 E5 (Progressive-Depth Bench).** Unblocked. E4 lands the telemetry; E5 derives pass/fail from distribution + quality-per-depth.
- Public-bench regression: should improve; watch for misrouted queries.

---

## 10. Dependency graph

- Requires: D4 (wake compiler) — needed for `wake` depth output.
- Blocks: F4 (preference drift reads from lookup), G4 proof harness.
- Parallelizable with F4 after Task E4.2.

## Exit criteria

1. Tests 1–16 green 10/10.
2. 7-day distribution ≥30% lookup share.
3. p50/p95 latency budgets hold.
4. 10-STAR axes 6 + 4 rescored.
5. `docs/contracts/recall-depth.md` linked.
6. Atomic commits on `research/mining`.

---

## Revision 2026-04-22 — FTS5 + RRF + query sanitization

> Appended after V4 audit. Donor pattern lift from Omegon (FTS5+RRF) and
> Smriti (query sanitization). Governed by
> [[docs/phases/v4/V4-INTEGRATION.md#11-schema--ordering-locks-v4-substrate-plumbing]].

### E4.7 — FTS5 + RRF hybrid retrieval (new task)

E4 progressive-depth recall gains a hybrid retrieval path underneath
the depth contract:

- **FTS5 virtual table** `memory_items_fts` mirrors `memory_items.content`
  with tokenizer `porter unicode61`. Indexed columns: content,
  normalized_content, kind, tags.
- **RRF fusion** of FTS5 rank + existing semantic score with default
  `k=60`. Reciprocal rank formula:
  `rrf(d) = sum_over_retrievers(1 / (k + rank_i(d)))`.
- Configurable per-query weights via `memd lookup --fusion fts:0.5,sem:0.5`;
  default 0.6/0.4 favoring FTS5 for exact-term recall.

FTS5 migration is an additive table, not a column add on `memory_items`,
so A4's migration is not disturbed. E4 owns the FTS5 migration directly.

### Query sanitization (donor: Smriti)

All query text entering the retrieval path passes through
`memd_core::query::sanitize()`:

- Strip FTS5 operators (`*`, `^`, `"`, `NEAR`) unless the caller passes
  `--raw-fts` (CLI) or `raw_fts: true` (API).
- Normalize whitespace and case for hashing.
- Reject queries > 1024 chars with `MEMD_ERR_QUERY_TOO_LONG`.
- Log sanitized-vs-raw diff to `.memd/logs/query-sanitize.ndjson` when
  sanitization changed the query materially (non-whitespace edit).

Rationale: avoid the Smriti-documented class of injection where a user
query with a rogue `*` flips FTS5 into prefix-match-all mode.

### Content-hash dedup in result sets

After ranking, E4 deduplicates the result list by `content_hash`. When
two rows share hash, keep the one with the highest trust tier; ties
broken by newest `lamport_seq`. Losers drop to explain surface with
`elided_as=dup` marker.

### E4 axis credit

E4 still contributes to SC +3 and TE +2 claims (shared with A4/D4).
FTS5+RRF is the retrieval substrate uplift; without it, E4's "depth
contract" is a policy wrapping an undifferentiated retrieval path.
Raw_retrieval axis does **not** lift in V4 (that's V5's bench work);
E4's contribution is to make the lift *possible* in V5.


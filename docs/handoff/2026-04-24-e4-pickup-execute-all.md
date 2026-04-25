---
opened: 2026-04-24T22:00-04:00
phase: E4
status: ready-to-execute
prev_handoff: 2026-04-24-d4-dogfood-clock-started.md
next_step: execute E4.1 → E4.6 in order; E4.7 dogfood runs in parallel with D4.8 clock; E4.8 rescore last
day7_dogfood_earliest: 2026-05-01
---

# E4 pickup — execute all of Progressive-Depth Recall

You are picking up immediately after D4 dogfood-clock-start (commit
`6e07c48`). Your job is to land **E4 Progressive-Depth Recall** end-to-end
on `research/mining`. D4 dogfood runs passively in the background — do
not touch its env vars or the compiler default.

## 30-second orientation

- **Branch**: `research/mining`
- **Tip**: `6e07c48 docs(d4): start dogfood clock`
- **Verify green pre-work**: `CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client --bin memd` → 552 passed (D4 baseline)
- **Phase spec**: `docs/phases/v4/phase-e4-progressive-depth-recall.md`
- **Phase plan**: `docs/phases/v4/phase-e4-plan.md` (8 tasks, has FTS5+RRF appendix)
- **Goal**: ship `memd lookup --depth {wake|lookup|resume}` + escalation hint + telemetry + 30-query distribution test. Then dogfood ≥7 days, then rescore 10-STAR.

## What's already done (don't redo)

The plan's "Revision 2026-04-22" appendix says E4 must add FTS5+RRF
hybrid retrieval and query sanitization. **These already exist
server-side** — likely landed in V3/A4. Verify before touching:

- `crates/memd-server/src/store_migrations.rs:348-380` — `memory_items_fts` virtual table (porter unicode61).
- `crates/memd-server/src/helpers.rs:252-290` — `rrf_rerank()` with `RRF_K`.
- `crates/memd-server/src/query_sanitize.rs` — sanitize fn (plan said it'd live in `memd-core`; it doesn't, do NOT move it).

So **E4.7 FTS5 sub-task in the appendix is mostly done**. What remains is wiring the depth flag through to make sure the existing hybrid path is exercised, plus the depth contract + telemetry + escalation hint that are pure E4 work.

## What's NOT done (your work)

| Plan task | Files | TDD signal |
|-----------|-------|------------|
| **E4.1** contract doc | `docs/contracts/recall-depth.md` (new) — copy §3 of the plan verbatim | n/a, doc only |
| **E4.2** depth dispatcher + CLI flag | `crates/memd-client/src/cli/args.rs:1297` (extend `LookupArgs`); new `crates/memd-client/src/runtime/recall/{mod,depth}.rs`; route from existing lookup runtime | tests 1, 2, 8, 9, 10 |
| **E4.3** escalation detector | new `crates/memd-client/src/runtime/recall/escalation.rs` w/ regex set from plan §3 | tests 3, 4, 5, 11, 12 |
| **E4.4** telemetry NDJSON | new `crates/memd-client/src/runtime/recall/telemetry.rs` writing `<bundle>/logs/recall-depth.ndjson`; emit from dispatcher AND from `runtime/resume/wakeup.rs` (every wake gets a depth line too) | tests 6, 7, 14 |
| **E4.5** `--explain-depth` flag | rationale printer in dispatcher | test 13 |
| **E4.6** fixtures + distribution test | `crates/memd-client/fixtures/e4/{queries,expected-depth,specifier-positive,specifier-negative}.jsonl`; new `crates/memd-client/src/main_tests/recall_depth_tests/mod.rs` | tests 15, 16 |
| **E4.7** 7-day dogfood | use `memd lookup --depth lookup` as default, collect `recall-depth.ndjson` ≥7d | data-only |
| **E4.8** 10-STAR rescore | bump `token_efficiency` + `cross_harness` in `docs/verification/MEMD-10-STAR.md` | doc only |

## Required test totals at exit

- recall_depth tests: 16/16 (all green per plan §4)
- memd-client full suite: ≥568 (552 + 16 new), still 0 failures
- workspace-wide: green
- p50/p95 latency budgets hold (lookup <50ms p50, wake <100ms p50, resume <500ms p95)

## Task ordering (executable, atomic commits)

```
E4.1 → docs(contracts): recall-depth contract (E4)
E4.2 → feat(memd-client/recall): --depth dispatcher (E4)
E4.3 → feat(memd-client/recall): escalation hint on zero-hit specifier (E4)
E4.4 → feat(memd-client/recall): depth telemetry (E4)
E4.5 → feat(memd-client/recall): --explain-depth (E4)
E4.6 → test(memd-client/recall): distribution + latency tests (E4)
E4.7 → docs(e4): 7-day distribution report  (after ≥7d data)
E4.8 → docs(10-star): E4 axis deltas         (after E4.7 passes)
```

E4.1–E4.6 are code-side and should land in one agent session. E4.7 is
a measurement gate (parallels D4.8 dogfood; same 7-day clock). E4.8 is
the rescore once data lands.

## Test command (single canonical invocation)

```sh
CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client --bin memd recall_depth
```

Full sweep before commit:

```sh
CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client --bin memd
```

## Wiring guidance — depth dispatcher

1. Add to `LookupArgs` (after existing flags around line 1338):
   ```rust
   #[arg(long, value_enum, default_value_t = RecallDepth::Lookup)]
   pub(crate) depth: RecallDepth,

   #[arg(long)]
   pub(crate) explain_depth: bool,
   ```
2. Define `RecallDepth { Wake, Lookup, Resume }` in `runtime/recall/depth.rs` with `clap::ValueEnum` derive.
3. Dispatcher in `runtime/recall/mod.rs`:
   - `Wake` → call existing wake compiler path used by `memd wake` (currently `crates/memd-client/src/bundle/turn_runtime.rs:104`).
   - `Lookup` → existing lookup runtime (no behavior change at depth=lookup default).
   - `Resume` → existing `memd resume` codepath.
4. Always emit one telemetry line regardless of depth.

## Wiring guidance — escalation

- Compile regex set once via `once_cell::sync::Lazy` or `std::sync::LazyLock` (project already uses both — check existing pattern in `compiler/priority.rs`).
- `EscalationDetector::matches(query: &str) -> bool` — pure function, no I/O.
- In dispatcher `Lookup` branch: if `result.records.is_empty() && detector.matches(&query)`, set `escalation_hint = Some(...)` on telemetry record AND print the hint line to stderr (per plan §3).

## Wiring guidance — telemetry

- One file: `<bundle>/logs/recall-depth.ndjson`.
- Reuse the ledger pattern from `crates/memd-client/src/runtime/resume/compiler/ledger.rs` — same atomic-append idiom, same `serde_json::to_string` + `\n` line. Don't reinvent.
- Schema (frozen by plan §2):
  ```json
  {"ts_ms":…,"session_id":"…","query":"…","depth":"lookup","records_returned":…,"tokens_returned":…,"latency_ms":…,"escalation_hint":null}
  ```
- For `wake` depth, use the same compiled-token count the D4 ledger emits.

## Feature flags

Plan §7 defines two:

| Var | Default | Effect |
|-----|---------|--------|
| `MEMD_E4_DEPTH_FLAG` | `1` | Enable `--depth`. Unset/0 → flag rejected with usage error. |
| `MEMD_E4_ESCALATION_HINT` | `1` | Emit hint on zero-hit + specifier. |

Default-on at ship; gate exists for emergency rollback only. Mirror
the `compiler_enabled()` pattern in `runtime/resume/compiler/mod.rs:183`.

## Fixtures

`crates/memd-client/fixtures/e4/`:

- `queries.jsonl` — 30 lines, mix per plan §5 (10 targeted, 10 overview, 10 resume-shape)
- `expected-depth.jsonl` — same length, one expected depth per query
- `specifier-positive.jsonl` — 10 queries that MUST match the escalation regex
- `specifier-negative.jsonl` — 10 queries that MUST NOT
- For session state: **do not duplicate D4 fixtures**. Use a fixture-loader helper that reads from `crates/memd-client/fixtures/d4/scenarios/` directly. Symlinks were the plan's hint but `git` cross-platform safety → prefer a loader.

## Gotchas (read these before coding)

1. **`memd lookup` already has `recall_depth: Option<usize>` at args.rs:479** — that's an unrelated numeric flag from another struct (likely WakeArgs). Confusing name. Don't conflate with the new `RecallDepth` enum on `LookupArgs`.
2. **Don't refactor sanitize/RRF location.** Plan said they'd live in `memd-core::query`; they actually live in `memd-server`. Plan is stale on this point. Substrate works; leave it.
3. **Don't break D4 dogfood.** The compiler env-var `MEMD_D4_COMPILER=1` is set in `~/.zshrc` and is collecting day-1 data right now. Don't unset it. Don't change `compiler_enabled()` default. Don't touch `runtime/resume/compiler/`.
4. **Wake-path telemetry.** Plan task E4.4 + test 14 require *every* wake call to emit a `recall-depth.ndjson` line, not just lookup calls. Wire a single line at the end of `wake` in `bundle/turn_runtime.rs` next to the existing budget/cost ledger emit.
5. **Latency budgets are p50/p95 of 30 fixture queries**, not single-call asserts. Implement the percentile calc in the test (sort + index) — there's no helper yet.
6. **NDJSON path = bundle root + `logs/`**, not `<bundle>/.memd/logs/`. Match the D4 pattern at `compiler/ledger.rs:21-22` (`logs/wake-budget.ndjson`).
7. **Auto-escalation = HINT ONLY** (plan §3). The dispatcher must NOT silently re-run at `resume` depth. Print, return zero-hit + hint, exit normal.

## Pass gate (whole phase)

From spec + plan §10:

1. Tests 1–16 green 10/10.
2. 7-day `recall-depth.ndjson` shows ≥30% of calls at `lookup` depth.
3. p50/p95 latency budgets hold (see §3 table).
4. `token_efficiency` + `cross_harness` axes bumped in `MEMD-10-STAR.md` with evidence link to the 7-day report.
5. `docs/contracts/recall-depth.md` linked from phase doc + README.
6. Atomic commits on `research/mining` matching the 8-task message list above.

## Dependent phases unblocked by E4

- **F4** Preference Drift — reads from lookup; needs depth-aware retrieval.
- **G4** Continuity Proof — uses recall telemetry to assert "right depth was chosen".
- **V5** Progressive-Depth Bench — derives pass/fail from `recall-depth.ndjson` distribution.
- **V6** Public-bench lift — needs depth contracts to claim token-efficiency win.

## Parallel: D4 dogfood passive monitoring

D4 dogfood clock is running in your shell. If you want to spot-check
during E4 work:

```sh
wc -l .memd/logs/wake-budget.ndjson
tail -1 .memd/logs/wake-budget.ndjson | jq .
```

Day-7 earliest = **2026-05-01**. Don't aggregate before then.

## When you finish E4.6 (code-complete)

Write a follow-up handoff:
`docs/handoff/<date>-e4-code-complete-dogfood-deferred.md` mirroring
the D4 code-complete handoff structure. Repoint `LATEST.md` via
`bash scripts/handoff-latest.sh`. Set ROADMAP `current_phase=E4`,
`phase_status=code-complete-dogfood-deferred`. Then F4 is unblocked
in parallel with E4's 7-day clock.

## What to do if you get stuck

- Tests for D4 compiler are at `crates/memd-client/src/runtime/resume/compiler/tests.rs` — 18 working examples of the same telemetry/ledger pattern you need for E4.
- Hybrid retrieval already integrated in `memd-server/src/helpers.rs:252` — read it before deciding whether to add anything new at the depth-dispatch layer.
- If a plan section conflicts with the codebase, **codebase wins**. Plan was written 2026-04-22; substrate has shifted. Note the deviation in your handoff like D4.7 did with the fixture-deviation note.

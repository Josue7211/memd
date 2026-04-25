---
opened: 2026-04-24
phase: E4
status: code-complete-dogfood-deferred
prev_handoff: 2026-04-24-e4-pickup-execute-all.md
next_step: E4.7 7-day dogfood gate + E4.8 10-STAR `progressive_recall` rescore — both blocked on 7 calendar days of live `recall-depth.ndjson` data
---

# E4 code complete — dogfood gate is the only thing left

## Pickup quickstart (30s read)

- **Branch**: `research/mining` (15 commits ahead of `e7bfdbb`)
- **Tip**: `a780ac7 test(memd-client/recall): distribution + latency tests (E4)`
- **Verify green**: `CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client --bin memd` → 569 passed
- **Next executable phase**: **F4 Preference Drift Repair** — `docs/phases/v4/phase-f4-preference-drift.md`. No code-side blocker on E4.7.
- **Blocked on measurement**: E4.7 (7-day distribution + latency dogfood) and E4.8 (10-STAR `progressive_recall` rescore 1 → 4) — gates that need real `recall-depth.ndjson` traffic.
- **Don't do**: don't flip `MEMD_E4_DEPTH_FLAG` or `MEMD_E4_ESCALATION_HINT` defaults — they are already on by default; the env vars exist only as emergency rollback. Don't remove the wake-arm double-record guard in `dispatch_lookup_with_depth` — it preserves the contract's "one line per recall call" invariant.

## What landed this session

| Task | Commit | Status |
|------|--------|--------|
| E4.1 contract doc (`docs/contracts/recall-depth.md`) | `58c1c30` | landed |
| E4.2 `--depth` dispatcher + `RecallDepth` enum + CLI flag | `75dae9b` | landed |
| E4.3 escalation specifier regex set + `escalation::detect`/`hint_line` | `f5385c2` | landed |
| E4.4 `recall-depth.ndjson` telemetry (lookup + wake CLI) | `de424f3` | landed |
| E4.5 `--explain-depth` rationale printer | `aaec32f` | landed |
| E4.6 fixtures + distribution + latency tests + double-record fix | `a780ac7` | landed |
| E4.7 7-day dogfood gate | n/a | **deferred** |
| E4.8 `progressive_recall` 10-STAR rescore | n/a | **blocked on E4.7** |

## Test totals

- recall_depth tests: **17/17** green (target was 16; +1 bonus `specifier_fixtures_match_regex_set` to guard fixture/regex drift)
- memd-client `--bin memd` full suite: **569/569** (was 552 pre-E4)
- workspace-wide: green

## What ships

### CLI surface

```
memd lookup \
  --query "<text>" \
  [--depth wake|lookup|resume]    # default: lookup
  [--explain-depth]                # one-line rationale on stderr
  [--output .memd]
  [--json]
```

`memd wake` and `memd resume` stand unchanged. `memd lookup --depth wake`
and `memd lookup --depth resume` route through the same dispatcher and
emit one telemetry line per call.

### Auto-escalation (hint only)

Zero-hit `lookup` + query matching the specifier regex set →
stderr line:

```
hint: zero results at lookup depth. Escalate with `memd lookup --query "…" --depth resume` (cost ~6k tokens).
```

Specifier set (case-insensitive):

```
\b(the|my|our)\b .+ \b(task|plan|issue|decision|bug|feature)\b
what (was|were) (I|we) \b(doing|working on|trying)\b
where did (I|we) leave off
```

The dispatcher does **not** silently re-run at deeper depth.

### Telemetry — `<bundle_root>/logs/recall-depth.ndjson`

One NDJSON line per recall call (any depth, including standalone
`memd wake`). Schema frozen for V4:

```json
{
  "ts_ms": 1714000000000,
  "session_id": "session-…",
  "query": "the migration plan",
  "depth": "lookup",
  "records_returned": 2,
  "tokens_returned": 347,
  "latency_ms": 18,
  "escalation_hint": null
}
```

`escalation_hint` carries the literal hint string only when the line
corresponds to a zero-hit lookup that triggered the specifier hint.

### Feature flags (default-on, emergency rollback only)

| Var                       | Default | Effect                                              |
|---------------------------|---------|-----------------------------------------------------|
| `MEMD_E4_DEPTH_FLAG`      | `1`     | Enable `--depth`. Off → flag rejected with usage error. |
| `MEMD_E4_ESCALATION_HINT` | `1`     | Emit hint on zero-hit + specifier match.            |

## Why E4.7 / E4.8 are deferred

E4.7 is the 7-day distribution + latency report. The contract pass
gate requires:

1. `recall-depth.ndjson` records ≥30% of recall calls at `lookup` depth.
2. p50 latencies meet table (`wake` <100ms, `lookup` <50ms, `resume`
   p95 <500ms).
3. Per-turn token cost (sum of `tokens_returned` per session) drops vs
   pre-E4 baseline.

These need real wake/lookup/resume traffic over a working week.
Synthetic fixtures already prove the dispatcher routes correctly,
emits the schema, and respects the 30% pass gate on 30 fixture queries
— but real-world distribution depends on agent behavior, which only
live use measures.

E4.8 is the 10-STAR `progressive_recall` axis rescore (1 → 4) that
consumes the E4.7 distribution as evidence.

## Pickup from here

1. **Use memd normally** for ≥7 calendar days. Default flags are on.
2. Aggregate `<bundle>/logs/recall-depth.ndjson`:
   - count by `depth` → assert lookup share ≥30%.
   - compute p50/p95 of `latency_ms` per `depth` → assert against
     contract budgets.
   - sum `tokens_returned` per session → compare vs pre-E4 baseline.
3. Write the report at `docs/phases/v4/phase-e4-progressive-depth-recall.md`
   §pass-gate-evidence.
4. If gates clear → rescore `progressive_recall` 1 → 4 in
   `docs/verification/MEMD-10-STAR.md`.

## Notable design decisions

- **Default depth is `lookup`**: matches the most common agent call
  pattern (targeted query). Wake/resume are explicit opt-ins.
- **`--depth wake` does NOT honor the D4 compiler env-gate**: when the
  user explicitly types `--depth wake`, they want the compiled wake
  brief. We bypass `compiler_enabled()` so the depth flag works
  regardless of D4 dogfood state.
- **Lookup limit clamp** (E4.2): `clamp_lookup_limit` enforces the
  contract's 1–3 record range *before* `build_lookup_request`, which
  previously defaulted to 6.
- **Wake-arm double-record fix** (E4.6): `run_bundle_wake_command`
  emits its own telemetry line for parity with the standalone
  `memd wake` CLI. The dispatcher's outer record now skips Wake to
  avoid double-counting; lookup and resume still record once at the
  dispatcher.
- **Escalation hint is HINT only**: explicit "agent picks depth,
  nothing escalates silently" per contract §auto-escalation. Plumbed
  via `LookupArmOutcome { escalation_hint: Option<String> }` so tests
  assert without stderr capture.
- **Schema frozen**: `DepthLine` is V4-frozen. New fields must be
  additive.

## Dependent phases

- **F4** Preference Drift can decide when to issue
  `memd lookup --depth resume` based on drift telemetry.
- **G4** Continuity Proof reuses the recall-depth NDJSON as part of
  its cross-harness invariant suite.
- **V6** public-bench lift expects the depth flag — gated by E4.7/E4.8.

## Files touched this session

- `docs/contracts/recall-depth.md` — frozen contract
- `crates/memd-client/Cargo.toml` — `regex = "1"` dependency
- `crates/memd-client/src/runtime/recall/{mod,depth,escalation,telemetry}.rs` — dispatcher + escalation + telemetry
- `crates/memd-client/src/runtime/mod.rs` — module wiring
- `crates/memd-client/src/cli/{args,mod,cli_memory_runtime}.rs` — `--depth`, `--explain-depth`, dispatch route
- `crates/memd-client/src/bundle/turn_runtime.rs` — wake CLI telemetry emit
- `crates/memd-client/src/main_tests/recall_depth_tests/mod.rs` — 17 tests
- `crates/memd-client/fixtures/e4/{queries,expected-depth,specifier-positive,specifier-negative}.jsonl` — 30 + 10 + 10 fixtures
- 6 LookupArgs construction sites updated for new fields (`depth`, `explain_depth`)
- `ROADMAP.md` — `phase_status=code-complete-dogfood-deferred`

## Next executable phase

**F4: Preference Drift Repair** — `docs/phases/v4/phase-f4-preference-drift.md`. No code-side blocker on E4.7.

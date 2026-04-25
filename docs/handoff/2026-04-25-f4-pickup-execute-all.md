---
opened: 2026-04-25
phase: F4
status: ready-to-execute
prev_handoff: 2026-04-24-e4-code-complete-dogfood-deferred.md
next_step: execute F4.1 → F4.6 in order; F4.7 dogfood runs in parallel with D4.8 + E4.7 clocks; F4.8 rescore last
day7_dogfood_earliest_d4: 2026-05-01
day7_dogfood_earliest_e4: 2026-05-01
day7_dogfood_earliest_f4: depends on F4.7 enable date (default ≥7 days after MEMD_F4_PREF_DRIFT=1)
---

# F4 pickup — execute all of Preference Replay + Drift Detection

You are picking up immediately after E4 code-complete (commit `7be4a86`).
Your job: land **F4 Preference Drift Repair** end-to-end on
`research/mining`. D4 + E4 dogfood clocks run passively in the
background — do not touch their env vars or defaults.

## 30-second orientation

- **Branch**: `research/mining`
- **Tip**: `7be4a86 docs(e4): mark code-complete, defer E4.7 dogfood + E4.8 rescore`
- **Verify green pre-work**: `CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client --bin memd` → 569 passed
- **Phase spec**: `docs/phases/v4/phase-f4-preference-drift.md`
- **Phase plan**: `docs/phases/v4/phase-f4-plan.md` (8 tasks)
- **Goal**: ship `memd preference {list,drift,confirm,promote}` + drift detector reusing C4's judge client + D4 compiler integration (preference bucket non-demotable, drift line surfaces on next wake) + `.memd/logs/preference-drift.ndjson` telemetry.

## Prereqs already in place (don't rebuild)

- **C4 judge client** at `crates/memd-core/src/correction/judge.rs` —
  `JudgeTransport`, `JudgeVerdict`, `JudgeBudgetState`, on-disk cache,
  $/month budget guard. F4 calls `judge::call(...)` directly.
- **`MemoryKind::Correction`** in `memd-schema` — F4.4 promote path
  writes correction-kinded records via the existing C4 lane; no schema
  changes needed.
- **D4 compiler** at `crates/memd-client/src/runtime/resume/compiler/`
  — F4.3 marks the preference bucket non-demotable in `priority.rs`
  and prepends a drift line in `render.rs` when outstanding state is
  present.
- **B4 universal hook trace** — F4.7 procedural-detection seed hooks
  into `observe_tool_sequence()` from the existing PostToolUse path.

## What's NOT done (your work)

| Plan task | Files | TDD signal |
|-----------|-------|------------|
| **F4.1** preference module + drift detector | new `crates/memd-core/src/preference/{mod,drift}.rs`; reuse `correction::judge` client | tests 1, 2, 3, 5, 6 |
| **F4.2** outstanding-drift state | new `.memd/state/preference-drift-outstanding.json` reader/writer in `preference::drift` | test 4 |
| **F4.3** D4 compiler integration | edit `crates/memd-client/src/runtime/resume/compiler/{priority,render}.rs` | tests 10, 11 |
| **F4.4** `memd preference` CLI | new `crates/memd-client/src/cli/cli_preference.rs`; extend `cli/args.rs` Commands enum; subcommands list/drift/confirm/promote | tests 7, 8, 9 |
| **F4.5** per-turn invocation hook | locate per-turn runtime tick; if missing, wire into PostToolUse hook; honor `MEMD_F4_DRIFT_N_TURNS` (default 10) | test 12 |
| **F4.6** E2E + dogfood simulation | new `crates/memd-client/src/main_tests/preference_drift_tests/mod.rs`; fixtures at `crates/memd-client/fixtures/f4/` | tests 13, 14 |
| **F4.7** 7-day dogfood + graduate | flip `MEMD_F4_PREF_DRIFT=1` locally, collect 7d log, tune false positives, flip default when cost ≤ $2/week | data-only |
| **F4.8** 10-STAR rescore | bump `correction_retention` axis in `docs/verification/MEMD-10-STAR.md` | doc only |

## Required test totals at exit

- preference_drift tests: 14/14 (per plan §4)
- memd-core preference unit tests: 6+ (tests 1–6)
- memd-client full suite: ≥577 (569 + 8 client-side preference tests)
- workspace-wide: green
- judge cost ≤ $2/week steady-state (F4.7 gate)

## Task ordering (executable, atomic commits)

```
F4.1  feat(memd-core/preference): drift detector
F4.2  feat(memd-core/preference): outstanding drift state
F4.3  feat(memd-client/compiler): preference non-demotion + drift surface
F4.4  feat(memd-client): memd preference verbs
F4.5  feat(memd-client/runtime): per-turn drift tick
F4.6  test(memd-client): preference drift E2E + restate-rate benchmark
F4.7  feat(f4): default MEMD_F4_PREF_DRIFT=1                  ← deferred
F4.8  docs(10-star): F4 correction_retention delta            ← deferred
```

F4.1 → F4.6 land code-side this session. F4.7 is the dogfood gate
(needs ≥7 days post-enable). F4.8 consumes F4.7 evidence for the
10-STAR rescore.

## Key gotchas

- **Drift detector reuses C4 judge** — call via the existing `judge`
  module; do NOT spin up a second proxy client. Cost guard is a
  shared pool: `judge::check_budget("c4+f4")`.
- **Preference bucket non-demotion** is a D4 contract change. The
  D4 compiler render currently allows demotion under tight budget;
  F4.3 must add a hard "preferences never demote" rule and update
  the compiler tests so the gate still holds.
- **Drift surface line in wake** must be ≤80 chars (one-line
  contract). Current wake compiler renders preferences in their own
  section; prepend on top of the section, not above the whole brief.
- **`MEMD_F4_PREF_DRIFT` defaults to 0** at land. Only flip to 1 in
  F4.7 after dogfood. This protects users from judge-cost spikes
  during initial rollout.
- **F4.7 procedural-detection seed** (revision 2026-04-22 in plan)
  is INSTRUMENTATION ONLY — emits routine candidates to NDJSON for
  V5 to consume, no behavior change. Claims `procedural_reuse` 1→2
  but **not** higher; G4 scorecard regenerator must reject any
  attempt to push above 2 until V5 lands.

## Dogfood clocks running in parallel

| Phase | Day-7 earliest | Status |
|-------|----------------|--------|
| D4.8 (compiler dogfood) | 2026-05-01 | `MEMD_D4_COMPILER=1` in shell since 2026-04-24 |
| E4.7 (recall depth distribution) | 2026-05-01 | default-on; collecting `recall-depth.ndjson` |
| F4.7 (preference drift) | TBD | gated on F4.6 land + `MEMD_F4_PREF_DRIFT=1` opt-in |

A scheduled agent fires 2026-05-01 to harvest D4 + E4 dogfood data,
write the histogram + distribution reports, and rescore the 10-STAR
axes. F4.7 trails the others.

## Test commands

```sh
# Per-task TDD loop
CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-core preference::
CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client preference_drift

# Pre-commit gate
CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client --bin memd
```

## Files this session will touch

- `crates/memd-core/src/preference/{mod,drift}.rs` (new)
- `crates/memd-core/src/lib.rs` (`pub mod preference;`)
- `crates/memd-client/src/cli/cli_preference.rs` (new)
- `crates/memd-client/src/cli/{args,mod}.rs` (extend Commands)
- `crates/memd-client/src/runtime/resume/compiler/{priority,render}.rs`
- `crates/memd-client/src/main_tests/preference_drift_tests/mod.rs` (new)
- `crates/memd-client/fixtures/f4/*.jsonl` (new)
- `.memd/benchmarks/grader-cache/f4/` (cache namespace, created lazily)
- `ROADMAP.md` — `current_phase=F4`, `phase_status=...`

## Voice

caveman-ultra. Terse, fix don't explain, no trailing summaries.

## Next executable phase after F4

**G4 Continuity Proof** — `docs/phases/v4/phase-g4-continuity-proof.md`.
F4 must land first because G4's cross-harness suite reuses the
preference drift signal.

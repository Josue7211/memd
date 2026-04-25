---
opened: 2026-04-24
phase: F4
status: code-complete
prev_handoff: 2026-04-24-f4-pickup-execute-all.md
next_step: F4.7 dogfood — flip `MEMD_F4_PREF_DRIFT=1` locally, collect 7d, then F4.8 rescore
day7_dogfood_earliest_d4: 2026-05-01
day7_dogfood_earliest_e4: 2026-05-01
day7_dogfood_earliest_f4: 2026-05-01 (if `MEMD_F4_PREF_DRIFT=1` flipped today)
---

# F4 code-complete — F4.1 → F4.6 landed; next is F4.7 dogfood

F4 Preference Drift Repair is code-complete on `research/mining`.
F4.7 (default-on flip) and F4.8 (10-STAR rescore) remain — both
gated on dogfood evidence, not code.

## What landed (this session)

| Task | Commit | Notes |
|------|--------|-------|
| F4.1 preference module + drift detector | landed | reuses C4 `JudgeTransport` + budget pool |
| F4.2 outstanding-drift state | landed | `<.memd>/state/preference-drift-outstanding.json` |
| F4.3 D4 compiler integration | landed | preference bucket non-demotable; `⚠ drift` line under Preferences |
| F4.4 `memd preference` CLI | landed | list / drift / confirm / promote |
| F4.5 per-turn drift tick | landed | `MEMD_F4_DRIFT_N_TURNS` (default 10), `MEMD_F4_PREF_DRIFT=0` gate |
| F4.6 E2E + restate-rate benchmark | `be1c68c` | tests 13 + 14, fixtures at `crates/memd-client/fixtures/f4/` |

Wake compiler wire-up (`turn_runtime.rs:135`) calls
`drift_notes_from_outstanding(&args.output)` between snapshot adapter
and `compile_wake`. `args.output` is `WakeArgs.output: PathBuf`
(default `default_bundle_root_path()`) — same bundle root used for
state/logs/metrics across all wake call sites. Reader and writers
both flow through `outstanding_state_path(memd_dir)` so paths agree
by construction.

## Verify-green commands

```sh
CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-core preference::
CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client preference_drift
CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client --bin memd
```

Last run: memd-client 581 passed, memd-core 111 passed.

## Not done (next pickup)

### F4.7 — dogfood + graduate

1. Enable locally: `export MEMD_F4_PREF_DRIFT=1` in shell env.
2. Run for ≥7 days. Earliest harvest: 2026-05-01 (matches D4.8 +
   E4.7 clocks).
3. Inspect `<.memd>/logs/preference-drift.ndjson` for false-positive
   rate; tune detector threshold if FP > 10%.
4. Confirm shared C4+F4 budget guard stays under $2/week steady-state.
5. Flip default to 1 in `tick.rs::drift_tick_enabled` only after
   the cost gate passes.

### F4.8 — 10-STAR rescore

Bump `correction_retention` axis in
`docs/verification/MEMD-10-STAR.md` once F4.7 evidence shows the
restate-rate drop holds in real (non-fixture) usage.

## Known small risk (not blocking)

`crates/memd-core/src/preference/tick.rs` test
`n_turns_from_env_defaults_to_ten` mutates `MEMD_F4_DRIFT_N_TURNS`
without an env-mutation lock. Other tests in the workspace already
guard env mutation via `OnceLock<Mutex<()>>`; if F4.5 tests start
flaking under parallel `cargo test`, add the same lock to the tick
test mod.

## Voice

caveman-ultra. Terse, fix don't explain, no trailing summaries.

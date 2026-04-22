---
phase: B4
name: Hook Contract Enforcement
version: v4
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [A4]
phase_doc: docs/phases/v4/phase-b4-hook-contract.md
granularity: "one step = ≤1 agent session; TDD inside; commit per task"
axis: session_continuity
axis_delta_target: "+1 on continuity + +1 on trust/provenance (paired)"
---

# Phase B4 — Implementation Plan

> Depends on A4. Assumes `docs/contracts/hook-handoff.md` has landed. Rebuild via `/tmp/memd-target`.

## 0. Executive summary

B4 gives hooks a **runtime**. Today the hook scripts under `.memd/hooks/*.sh` fire via harness integration (claude-code `~/.claude/settings.json`, codex `~/.codex/hooks.json`), call `memd hook …`, and exit — silent on failure. B4 wraps the `memd hook` CLI in an enforcer that:

1. Consults the fire-order contract (`docs/contracts/hook-order.md`).
2. Records every fire to `.memd/logs/hook-trace.ndjson`.
3. Applies a per-hook timeout budget.
4. Promotes specific failure classes from silent to visible, distinguishing write-path (halt) from observability (log-and-continue).

No new hook scripts. B4 upgrades the runtime `memd` commands those scripts already call — plus one new guard shim that wraps arbitrary future hooks.

---

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `docs/contracts/hook-order.md` | Canonical fire-order table + per-hook contract (budget, failure-class, required side-effects). Extends the A4 handoff doc — same directory. |
| `crates/memd-core/src/hook_runtime/mod.rs` | Pure-Rust enforcer primitives: `HookEvent`, `HookBudget`, `HookTrace`, `FireOrderValidator`. |
| `crates/memd-core/src/hook_runtime/trace.rs` | NDJSON trace writer — append-only, crash-safe (O_APPEND + fsync-on-close optional). |
| `crates/memd-core/src/hook_runtime/budget.rs` | Timeout wrapper using `tokio::time::timeout` where async, `crossbeam::channel::recv_timeout` where sync. |
| `crates/memd-client/src/cli/cli_hook_enforce.rs` | CLI dispatcher for `memd hooks enforce` + extension of `memd hooks doctor`. |
| `crates/memd-client/src/main_tests/hook_contract_tests/mod.rs` | 12 integration tests: happy path + 11 fault injections. |
| `crates/memd-client/fixtures/b4/` | 12 fixture hook-trace NDJSON files, one per test. |

### Files to modify

| Path | Change |
| --- | --- |
| `crates/memd-core/src/lib.rs` | `pub mod hook_runtime;`. |
| `crates/memd-client/src/cli/args.rs` | New `HookMode::Enforce(HookEnforceArgs)`. Extend `HookDoctorArgs` to accept `--check contract`. |
| `crates/memd-client/src/cli/cli_hook_runtime.rs` | Route `HookMode::Enforce` to enforcer. Every existing hook dispatch (context, capture, spill, file-interaction, seal-ledger, restore-from-A4) writes to trace. |
| `.memd/hooks/memd-*.sh` + `.ps1` | Prefix the `memd hook …` call with `memd hooks enforce --event <NAME> --budget-ms <N> --` when `MEMD_HOOK_ENFORCE=1`. No-op otherwise. |
| `.memd/hooks/MANIFEST.json` | Bump manifest version to 0.3; add `contract_version: "0.3"` field. |
| `docs/phases/v4/phase-b4-hook-contract.md` | Add `plan_spec:` line. |

### Crates affected

- `memd-core` — new `hook_runtime` module.
- `memd-client` — new CLI verb + hook-trace write on every existing dispatch.
- `memd-server` — none.

---

## 2. Schema changes

### New on-disk: `.memd/logs/hook-trace.ndjson`

```json
{"ts_ms":1713811200000,"event":"PreCompact","harness":"claude-code","session_id":"sess-abc","budget_ms":5000,"elapsed_ms":182,"exit_code":0,"failure_class":"none","trace_id":"01HWX…"}
{"ts_ms":1713811200500,"event":"PostCompact","harness":"claude-code","session_id":"sess-abc","budget_ms":2000,"elapsed_ms":8021,"exit_code":124,"failure_class":"timeout","trace_id":"01HWX…"}
```

### Contract-version field in MANIFEST.json

```json
{ "contract_version": "0.3", "hooks": [ … ] }
```

Older manifests (0.2) continue to load. Missing `contract_version` → treated as 0.2 for compatibility; enforcer still runs.

### Backward-compat posture

No breaking change. When `MEMD_HOOK_ENFORCE=0`, hook scripts bypass enforcer and call `memd hook …` directly — same code path as V3.

---

## 3. API shape

### `memd hooks enforce`

```
memd hooks enforce \
  --event <PreCompact|PostCompact|PreEdit|PreRead|UserPromptSubmit|SessionStart|Stop|PostToolUse> \
  --harness <claude-code|codex|other> \
  --session-id <ID> \
  --budget-ms <N>                 # default per event in contract
  --failure-class <halt|log>      # override contract default
  --trace <path>                  # default .memd/logs/hook-trace.ndjson
  -- <inner command…>             # the actual memd hook subcommand
```

Exit codes:
- `0` — inner command ok, within budget, correct fire order.
- `1` — fire-order violation (halt class) or inner non-zero (halt class).
- `2` — budget exceeded.
- `3` — contract parse failure (manifest or hook-order.md invalid).
- `0` with non-zero inner — for `log` class, returns 0 but writes `failure_class: "inner-nonzero"` to trace.

### Extended `memd hooks doctor --check contract`

Reads `docs/contracts/hook-order.md` + `.memd/hooks/MANIFEST.json` + `.memd/logs/hook-trace.ndjson`, reports:

- events observed vs contract list
- budget overruns in last N hours
- silent swallows (inner non-zero with failure_class=log)
- missing hooks (in contract but never fired)

Exit 0 clean, 1 with findings.

### New contract doc structure

`docs/contracts/hook-order.md` defines per-event:
- fire position (relative ordering)
- failure class (halt / log)
- budget_ms
- required output fields
- observability expectations

---

## 4. Test matrix

### Unit (memd-core/hook_runtime)

1. `fire_order_validator_accepts_canonical_sequence`
2. `fire_order_validator_flags_swap` — PostCompact before PreCompact → Err.
3. `fire_order_validator_permits_gaps` — missing optional events ok.
4. `budget_wraps_command_and_respects_timeout` — sleep 500ms, budget 200ms → timeout.
5. `budget_passes_through_on_success`
6. `trace_append_is_line_delimited_and_parseable`
7. `trace_survives_concurrent_writers` — 4 threads, 25 records each → 100 parseable lines, no overlap.
8. `trace_writes_trace_id_ulid`
9. `failure_class_halt_returns_exit_1_on_inner_failure`
10. `failure_class_log_returns_exit_0_on_inner_failure_records_class`

### Integration (memd-client/main_tests/hook_contract_tests)

11. `enforce_happy_path_precompact` — wraps `memd hook seal-ledger`, trace line ok.
12. `enforce_fires_restore_after_precompact` — A4 integration.
13. `enforce_blocks_out_of_order_postcompact_first` — returns exit 1.
14. `enforce_times_out_on_stuck_inner` — injected 10s sleep, budget 500ms → exit 2 trace `timeout`.
15. `enforce_records_bad_json_inner` — inner emits invalid JSON → `failure_class=bad-output`.
16. `enforce_concurrent_same_event_races_are_serialized` — two concurrent PreEdit on same session → second queues or errors per contract.
17. `enforce_disabled_flag_bypasses` — `MEMD_HOOK_ENFORCE=0` → inner runs, no trace line.
18. `doctor_contract_check_happy` — real trace file → exit 0.
19. `doctor_contract_check_surfaces_timeout` — trace with one timeout line → exit 1, diagnostic cites event name.
20. `doctor_contract_check_surfaces_silent_swallow` — log-class inner-nonzero → exit 1.
21. `doctor_contract_check_missing_hook_in_manifest` — manifest missing PostCompact entry → exit 1, cites manifest path.
22. `latency_regression_under_50ms_median` — 1000 wraps of `true`, p50 ≤ 50ms, p99 ≤ 200ms.

### Rebuild + smoke

```
cargo test --target-dir /tmp/memd-target -p memd-core hook_runtime::
cargo test --target-dir /tmp/memd-target -p memd-client hook_contract
```

---

## 5. Fixtures

`crates/memd-client/fixtures/b4/`:

| File | Contents |
| --- | --- |
| `trace-happy.ndjson` | 8-line canonical trace covering SessionStart → UserPromptSubmit → PreRead → PreEdit → PostToolUse → PreCompact → PostCompact → Stop. |
| `trace-timeout.ndjson` | Happy + one line `exit_code:124, failure_class:timeout`. |
| `trace-silent-swallow.ndjson` | Log-class inner-nonzero. |
| `trace-order-swap.ndjson` | PostCompact before PreCompact. |
| `manifest-0.3-complete.json` | Valid. |
| `manifest-0.3-missing-postcompact.json` | Missing entry. |
| `contract-hook-order-sample.md` | Mini contract used by doctor tests. |

Static, checked in. Regen via hand-edit.

---

## 6. Telemetry

| Signal | Path | New? |
| --- | --- | --- |
| Hook trace | `.memd/logs/hook-trace.ndjson` | New in B4 (A4 referenced but did not write). |
| Budget-exceeded counter | `memd_hook_budget_exceeded_total{event}` | Log-line counter; /metrics wiring deferred to V7. |
| Silent-swallow counter | `memd_hook_silent_swallow_total{event}` | Log-line only. |
| Contract-violation counter | `memd_hook_contract_violation_total{kind}` | Log-line only. |

Retention: V7 owns rotation. B4 caps file at 100 MiB — above that, the enforcer writes one `truncation-required` breach line per session and stops appending until manual rotation.

---

## 7. Feature flags

| Var | Default | Effect |
| --- | --- | --- |
| `MEMD_HOOK_ENFORCE` | `0` week 1, `1` after dogfood | Wraps hooks in enforcer. When off, hook scripts call inner `memd hook` directly. |
| `MEMD_HOOK_TRACE_PATH` | `.memd/logs/hook-trace.ndjson` | Override trace destination. Used by tests. |
| `MEMD_HOOK_BUDGET_OVERRIDE` | unset | Comma-sep `event=ms` overrides for debugging. |

Flip `MEMD_HOOK_ENFORCE=1` default after 7-day dogfood shows latency p99 ≤ 200ms.

---

## 8. Task list (executable)

### Task B4.1 — contract doc

- [ ] Read A4's `docs/contracts/hook-handoff.md` for style.
- [ ] Write `docs/contracts/hook-order.md`: table (event, harness, budget_ms, failure_class, required fire-position predecessors), rationale paragraph per event, worked examples.
- [ ] Link from B4 phase doc + README.
- [ ] Commit: `docs(contracts): hook-order canonical contract (B4)`.

### Task B4.2 — `hook_runtime::trace`

- [ ] Failing test 6 + 7 + 8.
- [ ] Implement `HookTrace` with `append(event: &HookRecord) -> io::Result<()>` using `OpenOptions::new().create(true).append(true)`.
- [ ] Green.
- [ ] Commit: `feat(memd-core/hook_runtime): ndjson trace writer (B4)`.

### Task B4.3 — `hook_runtime::budget`

- [ ] Tests 4 + 5.
- [ ] Implement `run_with_budget(cmd, budget) -> BudgetOutcome`. Spawns child via `std::process::Command`, polls or `wait_timeout`-crate if already a dep (check `cargo tree`), else hand-rolled timer thread.
- [ ] Green.
- [ ] Commit: `feat(memd-core/hook_runtime): budget-bounded command wrapper (B4)`.

### Task B4.4 — `FireOrderValidator`

- [ ] Tests 1–3 + 9 + 10.
- [ ] Implement validator that parses contract table into a state machine, `observe(event) -> Result<(), ViolationKind>`.
- [ ] Green.
- [ ] Commit: `feat(memd-core/hook_runtime): fire-order validator + failure classes (B4)`.

### Task B4.5 — `memd hooks enforce` CLI

- [ ] Extend `HookMode` with `Enforce(HookEnforceArgs)` in `args.rs`.
- [ ] New `cli/cli_hook_enforce.rs` dispatcher.
- [ ] Tests 11 + 14 + 15 + 17 + 22.
- [ ] Implement dispatch: parse contract → validate order → wrap inner → trace line.
- [ ] Green.
- [ ] Commit: `feat(memd-client/hooks): enforce verb (B4)`.

### Task B4.6 — every existing hook dispatch writes trace

- [ ] In `cli_hook_runtime.rs`, thread a `HookTrace` writer through every `HookMode` arm (context, capture, spill, file-interaction, seal-ledger, restore).
- [ ] Test 12 (A4 restore trace line).
- [ ] Green.
- [ ] Commit: `feat(memd-client/hooks): universal trace emission (B4)`.

### Task B4.7 — concurrency + serialization

- [ ] Test 16.
- [ ] Implement per-event, per-session fcntl lock on a sidecar file in `.memd/state/session-<id>/hook.lock`. Second concurrent fire queues up to 1s then errors.
- [ ] Green.
- [ ] Commit: `feat(memd-core/hook_runtime): per-event per-session serialization (B4)`.

### Task B4.8 — doctor `--check contract`

- [ ] Tests 18–21.
- [ ] Extend `cli_hook_doctor.rs` with contract-check branch reading trace + manifest + contract doc.
- [ ] Green.
- [ ] Commit: `feat(memd-client/hooks): doctor --check contract (B4)`.

### Task B4.9 — hook scripts call enforce when flag on

- [ ] In each `.memd/hooks/memd-*.sh`: prefix the inner `memd hook` call with the enforce wrapper when `${MEMD_HOOK_ENFORCE:-0}` = 1.
- [ ] Mirror in `.ps1`.
- [ ] Update MANIFEST.json `contract_version: "0.3"`, resha.
- [ ] Run `scripts/sync-integration-hooks.sh`.
- [ ] Commit: `feat(hooks): scripts route through enforce wrapper behind flag (B4)`.

### Task B4.10 — 24h dogfood + graduate default

- [ ] Set `MEMD_HOOK_ENFORCE=1` in local profile.
- [ ] Run usual workload 24h.
- [ ] Collect trace stats: median/p99 latency, silent-swallow count.
- [ ] If p99 ≤ 200ms and silent swallows = 0, flip default to `1` in hook scripts.
- [ ] Commit: `feat(hooks): default MEMD_HOOK_ENFORCE=1 after dogfood (B4)`.

### Task B4.11 — 10-STAR rescoring

- [ ] Bump session_continuity + trust/provenance axes in `docs/verification/MEMD-10-STAR.md`.
- [ ] Commit: `docs(10-star): axes 1+7 rescored after B4 enforcer green`.

---

## 9. Bench impact

- **V5 G5 (Trust/Provenance Bench).** B4 unblocks — every memd action is now traceable. G5 scenario: injected hook failure, assert user-visible signal within 1 turn.
- Public benches: unaffected.

---

## 10. Dependency graph

- Requires: A4 tasks 1–6 (hook-handoff.md) landed.
- Blocks: C4 correction capture (writes to trace), D4 compiler (reads trace for "last session continuity"), G4 (reads trace for assertion).
- Parallelizable after Task B4.1: C4/D4 can start against placeholder trace schema.

## Exit criteria

1. Tests 1–22 green 10/10.
2. 24h dogfood trace: zero silent swallows, p99 ≤ 200ms.
3. `memd hooks doctor --check contract` red on planted fault, green on clean state.
4. `docs/contracts/hook-order.md` exists, linked.
5. MANIFEST.json contract_version 0.3.
6. 10-STAR axes 1 + 7 rescored.
7. Atomic commits on `research/mining`.

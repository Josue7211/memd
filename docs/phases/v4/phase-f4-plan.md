---
phase: F4
name: Preference Replay + Drift Detection
version: v4
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [C4]
phase_doc: docs/phases/v4/phase-f4-preference-drift.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: correction_retention
axis_delta_target: "+1 (4 → 5 after C4 floor)"
---

# Phase F4 — Implementation Plan

> Depends on C4 (correction lane + MemoryKind::Correction). Piggybacks on C4's cached LLM-judge infrastructure.

## 0. Executive summary

Preferences already store. They load at wake (D4 compiler top-priority bucket). F4 adds:

1. Guaranteed replay — preferences section is non-demotable in D4 compiler.
2. Drift detector — every N agent turns, cached LLM-judge checks recent agent behavior against stored preferences.
3. Drift surface — on next wake, surface divergence as a one-line note.
4. Promotion path — user confirms drift → correction lane bumps preference confidence + re-pins.

---

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `crates/memd-core/src/preference/mod.rs` | Preference record helpers. |
| `crates/memd-core/src/preference/drift.rs` | `DriftDetector` — prompt assembly, judge call, verdict classification. Reuses C4's `correction::judge` client. |
| `crates/memd-client/src/cli/cli_preference.rs` | `memd preference` verbs: list, drift, confirm. |
| `crates/memd-client/src/main_tests/preference_drift_tests/mod.rs` | Scenarios. |
| `crates/memd-client/fixtures/f4/` | Preference sets + agent behavior transcripts. |
| `.memd/benchmarks/grader-cache/f4/` | Cache namespace. |

### Files to modify

| Path | Change |
| --- | --- |
| `crates/memd-client/src/runtime/resume/compiler/priority.rs` (D4) | Mark preference bucket non-demotable. |
| `crates/memd-client/src/runtime/resume/compiler/render.rs` (D4) | If drift outstanding, prepend one-line note to preferences section. |
| `crates/memd-client/src/runtime/turn.rs` (or wherever per-turn tick lives) | Call drift detector every `MEMD_F4_DRIFT_N_TURNS` turns. |
| `crates/memd-client/src/cli/args.rs` | Top-level `Preference(PreferenceArgs)` + subcommands. |
| `docs/phases/v4/phase-f4-preference-drift.md` | `plan_spec:` line. |

---

## 2. Schema changes

None in memd-schema. F4 produces correction-kinded records via C4's path, so no new variants.

### New log: `.memd/logs/preference-drift.ndjson`

```json
{"ts_ms":…,"session_id":"…","preference_id":"pref-voice-terse","checked_turns":10,"violation_count":3,"judge_verdict":"drift","judge_confidence":0.81,"surfaced":true}
```

### New state file: `.memd/state/preference-drift-outstanding.json`

Carries the most-recent unacknowledged drift signal per preference; consumed by D4 render. Cleared on user-confirm via `memd preference confirm`.

---

## 3. API shape

### `memd preference`

```
memd preference list [--session-id <ID>]
memd preference drift --preference-id <ID> [--turns <N>]   # force a check
memd preference confirm --preference-id <ID>               # ack drift, optionally promote
memd preference promote --preference-id <ID>               # bump confidence via correction lane
```

### Drift detector invocation

Called from the per-turn pipeline. Budget guard: `MEMD_F4_JUDGE_BUDGET_USD` shares pool with C4's budget. Shared function `judge::check_budget("c4+f4")`.

---

## 4. Test matrix

### Unit

1. `drift_detector_detects_verbose_against_terse_preference`
2. `drift_detector_passes_on_aligned_behavior`
3. `drift_detector_caches_verdict_by_preference_id_behavior_hash`
4. `drift_outstanding_state_persists_and_clears_on_confirm`
5. `drift_prompt_includes_recent_turn_window`
6. `judge_budget_shared_with_c4`

### Integration

7. `cli_preference_drift_force_check`
8. `cli_preference_confirm_clears_outstanding`
9. `cli_preference_promote_writes_correction_record_via_c4_path`
10. `d4_compiler_surfaces_drift_line_when_outstanding`
11. `d4_compiler_preferences_bucket_non_demotable_under_budget`
12. `per_turn_pipeline_invokes_drift_every_n_turns`

### E2E

13. `e2e_7day_dogfood_simulation` — scripted: 10 turns aligned, 10 turns violating, assert drift surfaces on next wake.
14. `e2e_user_restate_rate_drops` — baseline fixture captures user restating same preference; replay with F4 on, assert restate count drops ≥50% on repeatable scenario.

### Rebuild + smoke

```
cargo test --target-dir /tmp/memd-target -p memd-core preference::
cargo test --target-dir /tmp/memd-target -p memd-client preference_drift
```

---

## 5. Fixtures

`crates/memd-client/fixtures/f4/`:

| File | Contents |
| --- | --- |
| `preferences.jsonl` | 5 preferences (voice=terse, tabs-not-spaces, no-emojis, commit-style-caveman, no-ai-footer). |
| `aligned-behavior.jsonl` | 10 turns matching prefs. |
| `drift-behavior.jsonl` | 10 turns violating voice=terse. |
| `judge-verdict-drift.json` | Mock proxy response. |
| `judge-verdict-ok.json` | Mock proxy response. |
| `user-restate-baseline.jsonl` | 7-day fixture where user restates voice=terse 4 times. |
| `user-restate-withF4.jsonl` | Same 7-day fixture with F4 simulated — restate count 1. |

---

## 6. Telemetry

| Signal | Path |
| --- | --- |
| Drift checks | `.memd/logs/preference-drift.ndjson` |
| Outstanding drift state | `.memd/state/preference-drift-outstanding.json` |
| Counters | `memd_preference_drift_check_total{verdict}`, `memd_preference_drift_surface_total`, `memd_preference_promote_total` — log-line. |

---

## 7. Feature flags

| Var | Default | Effect |
| --- | --- | --- |
| `MEMD_F4_PREF_DRIFT` | `0` → `1` post dogfood | Enable drift detector + surface. |
| `MEMD_F4_DRIFT_N_TURNS` | `10` | Turn interval. |
| `MEMD_F4_JUDGE_BUDGET_USD` | shares with C4 pool | Spend ceiling. |
| `MEMD_F4_DRIFT_FALSE_POSITIVE_TIGHTEN` | `0` | Raises threshold, cuts false positives. |

---

## 8. Task list (executable)

### Task F4.1 — preference module + drift detector

- [ ] Tests 1 + 2 + 3 + 5 + 6 failing.
- [ ] Implement `preference::drift::detect(prefs, recent_turns) -> Verdict` via C4 judge client.
- [ ] Shared cost guard.
- [ ] Green.
- [ ] Commit: `feat(memd-core/preference): drift detector (F4)`.

### Task F4.2 — outstanding-state persistence

- [ ] Test 4 failing.
- [ ] Implement read/write for `preference-drift-outstanding.json`.
- [ ] Green.
- [ ] Commit: `feat(memd-core/preference): outstanding drift state (F4)`.

### Task F4.3 — D4 compiler integration

- [ ] Tests 10 + 11 failing.
- [ ] Mark preference bucket non-demotable in `compiler::priority`.
- [ ] Prepend drift line in `compiler::render` when outstanding.
- [ ] Green.
- [ ] Commit: `feat(memd-client/compiler): preference non-demotion + drift surface (F4)`.

### Task F4.4 — `memd preference` CLI

- [ ] Add top-level verb.
- [ ] Tests 7 + 8 + 9 failing.
- [ ] Promote uses C4's `correction capture` path.
- [ ] Green.
- [ ] Commit: `feat(memd-client): memd preference verbs (F4)`.

### Task F4.5 — per-turn invocation hook

- [ ] Locate per-turn runtime tick site. If absent, wire into the hook that fires on PostToolUse.
- [ ] Test 12 failing.
- [ ] Implement rate-limited invocation honoring `MEMD_F4_DRIFT_N_TURNS`.
- [ ] Green.
- [ ] Commit: `feat(memd-client/runtime): per-turn drift tick (F4)`.

### Task F4.6 — E2E + dogfood simulation

- [ ] Build fixtures.
- [ ] Tests 13 + 14 failing.
- [ ] Green.
- [ ] Commit: `test(memd-client): preference drift E2E + restate-rate benchmark (F4)`.

### Task F4.7 — 7-day dogfood + graduate

- [ ] Enable `MEMD_F4_PREF_DRIFT=1` locally.
- [ ] Collect 7d log.
- [ ] Tune false positives if judge precision drops below 0.80.
- [ ] Flip default to `1` when cost ≤ $2/week.
- [ ] Commit: `feat(f4): default MEMD_F4_PREF_DRIFT=1`.

### Task F4.8 — 10-STAR rescoring

- [ ] Bump correction_retention axis with F4 evidence.
- [ ] Commit: `docs(10-star): F4 correction_retention delta`.

---

## 9. Bench impact

- **V5 F5 (Preference Retention Bench).** F4 produces the signal; V5 F5 sets the pass criterion.
- Public-bench regression watch: LME Memory Updates — preference promotion lane may interact with LME's update turns. Run canonical regression post-Task F4.6.

---

## 10. Dependency graph

- Requires: C4 judge client + Correction kind. D4 compiler for surface.
- Blocks: G4 proof harness.
- Parallelizable with E4 after Tasks F4.1 + F4.2.

## Exit criteria

1. Tests 1–14 green 10/10.
2. 7-day dogfood: user-restate rate drops ≥50%.
3. Judge cost ≤ $2/week.
4. `MEMD_F4_PREF_DRIFT=1` default.
5. 10-STAR correction_retention bumped.
6. Atomic commits on `research/mining`.

---

## Revision 2026-04-22 — F4.7 procedural-detection seed

> Appended after V4 audit. New intra-F4 task seeds the dormant
> `RetrievalIntent::Procedural` path. Governed by
> [[docs/verification/milestones/MILESTONE-v4.md]] procedural_reuse gate.
> F4.7 claims **zero axis credit** — instrumentation only; no behavior
> proof. The axis lift to 3+ is V5 scope.

### F4.7 — routine-detection seed (new task)

Wire the dead-code `RetrievalIntent::Procedural` path enough to emit
metrics that V5 can later prove against.

**Scope:**
1. Add `memd_core::procedural::detect::observe_tool_sequence()` called
   by B4's universal hook trace on every PostToolUse event.
2. Detector observes (tool_name, target_prefix) tuples and counts
   repetition within a sliding 20-turn window. Window advances per turn.
3. When a tuple repeats ≥ 3 times with ≥ 2 distinct session_ids, emit
   a `routine_candidate` event to `.memd/logs/routine-candidates.ndjson`:
   ```json
   {"ts":"...","tuple":["read","src/ledger"],"count":4,"sessions":["s1","s2"],"candidate_id":"..."}
   ```
4. Increment Prometheus-style counter `routine_candidates_observed`
   (stored in `.memd/metrics/counters.json`, not a real Prometheus server).
5. Add `memd procedural candidates` read-only CLI that prints the
   log and counter.

**Explicit non-scope:**
- No promotion from candidate to procedural memory.
- No retrieval-path consumption of `routine_candidates` (that path stays
  dead in V4).
- No behavior change — this is pure instrumentation.

### F4.7 pass gate

- Counter `routine_candidates_observed` ≥ 1 after the 25-turn G4
  scenario completes.
- NDJSON log contains at least one entry with `count ≥ 3` and
  `sessions ≥ 2`.
- `memd procedural candidates` returns the log without panicking.

G4 asserts these at turns T16.5 and T25.5 (see V4-INTEGRATION §4).

### F4.7 explicitly does not claim axis credit

MILESTONE-v4's `procedural_reuse` axis goes 1 → 2 because
instrumentation exists. **Any attempt to regenerate the scorecard with
procedural_reuse > 2 while V5 routine-detection-live has not landed is
invalid** and the G4 scorecard regenerator must reject it.

### F4 axis credit

F4 still claims CR +1 shared with C4 (preference drift is a correction-
family surface) and the new F4.7 contributes procedural_reuse +1 with
the strict no-credit-above-2 constraint.


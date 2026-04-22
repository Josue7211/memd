---
phase: G4
name: Session-Continuity Proof Harness
version: v4
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [A4, B4, C4, D4, E4, F4]
phase_doc: docs/phases/v4/phase-g4-continuity-proof.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: V4 completion gate (all seven axes rescored)
axis_delta_target: "composite 2.15 → ≥4.0"
---

# Phase G4 — Implementation Plan

> Gate phase. All V4 deliverables must be integrated; nothing lands after G4 in V4 scope.

## 0. Executive summary

G4 proves V4 as a product, not a list of features. Three-session dogfood scenario, scripted turn-by-turn, asserts A4+B4+C4+D4+E4+F4 work together. Runs nightly in CI. Regenerates `docs/verification/MEMD-10-STAR.md`. If composite misses 4.0, V4 stays open.

---

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `crates/memd-client/src/main_tests/v4_proof_harness/mod.rs` | 3-session scenario driver. |
| `crates/memd-client/src/main_tests/v4_proof_harness/assertions.rs` | Composable asserters reading across all V4 outputs. |
| `crates/memd-client/src/main_tests/v4_proof_harness/scorecard.rs` | 10-STAR composite regenerator. |
| `crates/memd-client/fixtures/g4/` | 3-session scripted transcripts + expected-state-per-cut. |
| `scripts/ci/v4-proof-harness.sh` | CI entrypoint. |
| `.github/workflows/v4-proof.yml` (or equivalent CI file — check `.github/workflows/` first; if project uses a different CI, mirror) | Nightly schedule + push-gate on `main`. |
| `docs/verification/milestones/MILESTONE-v4.md` (stub exists) | Filled in with G4 pass evidence. |

### Files to modify

| Path | Change |
| --- | --- |
| `docs/verification/MEMD-10-STAR.md` | G4 regenerates axis scores + composite. |
| `docs/phases/v4/phase-g4-continuity-proof.md` | `plan_spec:` line. |

---

## 2. Schema changes

None. G4 reads artifacts.

### One new artifact

`docs/verification/v4-proof-runs/YYYY-MM-DD.ndjson`:

```json
{"run_id":"…","ts_ms":…,"passed":true,"axis_scores":{"session_continuity":4,"correction_retention":4,"procedural_reuse":1,"cross_harness":4,"raw_retrieval":4,"token_efficiency":4,"trust_provenance":6},"composite":4.05,"failed_assertions":[]}
```

---

## 3. API shape

### Scenario driver CLI (test-only)

```
cargo run -p memd-client --bin v4-proof-harness -- \
  --fixture-dir crates/memd-client/fixtures/g4 \
  --output /tmp/g4-run-<id> \
  --report docs/verification/v4-proof-runs/YYYY-MM-DD.ndjson
```

Builds by default as a test binary; CI invokes this.

### Scorecard regenerator

```rust
pub fn regenerate_10star(
    proof_run: &ProofRun,
    current: &Scorecard,
) -> Scorecard;
```

Writes markdown back into `docs/verification/MEMD-10-STAR.md` axis table with evidence pointer to the run NDJSON.

---

## 4. Test matrix — the 3-session scenario

### Session 1 (10 turns)

- T1 user: "We use Postgres for the ledger."
- T2 user: "The primary ID is uuid."  → fact A.
- T3 user: "No wait, actually the primary ID is ulid."  → correction of fact A.
- T4 user: "Prefer terse replies." → preference P1.
- T5 user: "No emojis ever." → preference P2.
- T6 user: "Focus: finish the Q1 migration."
- T7 user: "The migration deadline is 2026-05-01."
- T8–T9 agent tool calls: Read 3 files.
- T10 session end → PreCompact.

Cut 1 assertions:
- fact A stored with `corrects_id` pointing at T2 record (C4)
- P1 + P2 stored with kind=Preference (F4 replay requires this)
- Read ledger contains 3 files (A4)
- Hook trace has: SessionStart → 10 × PostToolUse → PreCompact → (sealed) (B4)

### Session 2 (10 turns)

- Wake: D4 compiler compiles brief. Assertions:
  - Preferences section present with P1 + P2, non-demoted (D4 + F4)
  - Focus section present with "Q1 migration"
  - Wake tokens ≤ 2000 (D4)
  - PostCompact restore ran before first tool call (A4)
- T11 agent tool: Read file-4.
- T12 user query: `memd lookup --query "primary ID" --depth lookup`.
  - Must return `ulid` (the correction), not `uuid` (E4 + C4)
- T13–T17 agent touches 3 of the 5 facts from session 1.
- T18 user: "No, migration deadline is 2026-05-15." → correction C2.
- T19 agent drifts to verbose reply (deliberate seeded behavior).
- T20 session end → PreCompact.

Cut 2 assertions:
- correction lookup returns corrected value (C4)
- C2 stored with provenance (C4)
- drift detected for P1 (F4): preference-drift.ndjson has 1 line
- Read ledger has file-4 + session 1 files (A4)
- No silent hook failures (B4)

### Session 3 (5 turns)

- Wake: D4 compiler. Assertions:
  - Drift surface line prepended to Preferences section (F4)
  - Both corrections (C1, C2) in the wake canonical section
  - Wake tokens ≤ 2000
- T21 user: "What's the migration deadline?" Agent must answer "2026-05-15" not "2026-05-01".
- T22 user: "And the primary ID?" Agent answers "ulid".
- T23 user: "Confirm, drop the drift note — I noticed." `memd preference confirm P1`.
- T24 agent behaves tersely. Post-turn drift check should clear or remain clean.
- T25 session end.

Cut 3 assertions:
- Agent answers use corrected values (E4 lookup routing)
- Drift outstanding state cleared after confirm (F4)
- No `continuity-breach.log` lines across all sessions (A4)
- 10-STAR composite regenerated, ≥4.0

### Unit tests around harness

1. `harness_parses_fixture_script`
2. `harness_runs_3_sessions_in_sequence_with_simulated_compaction`
3. `assertions_fail_when_a4_restore_skipped`
4. `assertions_fail_when_b4_trace_has_silent_swallow`
5. `assertions_fail_when_c4_correction_missing_provenance`
6. `assertions_fail_when_d4_wake_exceeds_budget`
7. `assertions_fail_when_e4_lookup_returns_stale_value`
8. `assertions_fail_when_f4_drift_undetected_on_seeded_violation`
9. `scorecard_regenerator_writes_markdown_axis_table`
10. `scorecard_regenerator_preserves_prior_axes_not_in_scope`

### CI-shaped runs

11. `ci_harness_passes_10_of_10_on_clean_tree`
12. `ci_harness_retries_only_on_infra_flake` — injected `/tmp` full error → retry once; injected memd failure → no retry.

### Rebuild + smoke

```
cargo test --target-dir /tmp/memd-target -p memd-client v4_proof_harness
bash scripts/ci/v4-proof-harness.sh
```

---

## 5. Fixtures

`crates/memd-client/fixtures/g4/`:

| File | Contents |
| --- | --- |
| `session-1.jsonl` | 10-turn script. |
| `session-2.jsonl` | 10-turn script. |
| `session-3.jsonl` | 5-turn script. |
| `expected-cut-1.json` | Expected memd state post-session-1. |
| `expected-cut-2.json` | Post-session-2. |
| `expected-cut-3.json` | Post-session-3 + scorecard. |
| `seed-state.json` | Empty-memd starting point. |
| `inject-faults/` | Fault fixtures used by tests 3–8 to verify harness catches regressions. |

---

## 6. Telemetry

| Signal | Path |
| --- | --- |
| Proof run records | `docs/verification/v4-proof-runs/*.ndjson` |
| CI log artifact | attached to CI run |
| Harness counters | `memd_v4_proof_assertion_fail_total{cut,assertion}` log-line. |

---

## 7. Feature flags

| Var | Default | Effect |
| --- | --- | --- |
| `MEMD_G4_HARNESS_FAIL_FAST` | `1` | Stop at first failed assertion in CI. |
| `MEMD_G4_FAULT_INJECT` | `0` | Enable fault-injection fixtures. |

Harness itself does not have a kill-switch — it is a gate, always on.

---

## 8. Task list (executable)

### Task G4.1 — scenario fixtures

- [ ] Author `session-{1,2,3}.jsonl` per §4.
- [ ] Author `expected-cut-{1,2,3}.json`.
- [ ] Author `seed-state.json`.
- [ ] Commit: `test-fixtures(g4): 3-session V4 proof scenario (G4)`.

### Task G4.2 — harness driver

- [ ] Test 1 + 2 failing.
- [ ] Implement driver that loads fixture, spawns `memd-server` under `MEMD_RATE_LIMIT_DISABLED=1`, runs turns, simulates compaction between sessions.
- [ ] Green.
- [ ] Commit: `feat(memd-client/v4-proof): scenario driver (G4)`.

### Task G4.3 — cross-V4 assertions module

- [ ] Tests 3–8 failing.
- [ ] Implement asserters reading `.memd/logs/*.ndjson`, state files, ledger, preference-drift state.
- [ ] Green.
- [ ] Commit: `feat(memd-client/v4-proof): cross-phase assertions (G4)`.

### Task G4.4 — scorecard regenerator

- [ ] Tests 9 + 10 failing.
- [ ] Parse existing `MEMD-10-STAR.md` axis table; update in-place with pointers to proof-run NDJSON.
- [ ] Green.
- [ ] Commit: `feat(memd-client/v4-proof): 10-STAR regenerator (G4)`.

### Task G4.5 — CI entrypoint + workflow

- [ ] Write `scripts/ci/v4-proof-harness.sh`.
- [ ] Write CI workflow — inspect `.github/workflows/` first to mirror existing style.
- [ ] Test 11 + 12 failing.
- [ ] Green on local run.
- [ ] Commit: `ci(v4-proof): nightly + push-gate (G4)`.

### Task G4.6 — 7-day stability

- [ ] Run harness manually 10× back-to-back. All pass.
- [ ] Watch CI nightly for 7 days. Must be 10/10 clean.
- [ ] If any run fails, root-cause (not retry) and fix source phase.
- [ ] Commit: `docs(verification): V4 proof 7-day stability log`.

### Task G4.7 — milestone close

- [ ] Fill `docs/verification/milestones/MILESTONE-v4.md` with pass evidence: 10 CI runs, regenerated scorecard, proof-run NDJSON pointers.
- [ ] Assert composite ≥ 4.0. If lower, V4 stays open → file per-axis recovery phase per phase-doc `fail_conditions`.
- [ ] Update `ROADMAP.md` V4 → complete, V5 → in progress.
- [ ] Commit: `docs(milestone): V4 closed, 10-STAR composite → 4.0+ (G4)`.

---

## 9. Bench impact

G4 is a bench. It is **the** V4 bench. Public benches: no direct impact — G4 measures substrate axes, not public bench numbers.

---

## 10. Dependency graph

- Requires: **all** V4 phases complete. A4 → B4 → (C4, D4 parallel after B4) → (E4 after D4, F4 after C4) → G4.
- Blocks: V5.
- Strictly sequential at the phase level (cannot start G4 until A4–F4 merged to `research/mining`).

## Exit criteria — the V4 gate

1. Harness passes 10/10 CI runs over 7 days.
2. 10-STAR composite ≥ 4.0 after regeneration.
3. `docs/verification/milestones/MILESTONE-v4.md` filled.
4. `docs/verification/v4-proof-runs/` has ≥10 NDJSON records.
5. `ROADMAP.md` V4 → closed.
6. No `continuity-breach.log` entries across any of the 10 runs.
7. Atomic commits on `research/mining`.

If any fails: V4 stays open. File recovery phase. Do not advance to V5.

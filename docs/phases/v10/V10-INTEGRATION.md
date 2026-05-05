---
version: v10
kind: integration-plan
status: closed
opened: 2026-04-22
revised: 2026-05-05
scope: A10..G10
depends_on: [../../verification/0.1.0-CONTRACT.md, ../../verification/0.1.0-AXIS-OWNERSHIP.md, ../../verification/milestones/MILESTONE-v10.md]
---

# V10 Integration — Self-Improvement Production Floor

> V10 closes the production-floor gate, not the `0.1.0` release gate. Release
> remains V13 per `docs/verification/0.1.0-CONTRACT.md`.

## 1. Execution-order discipline

Phase-level dependency (strict):

```
A10 ──► B10 ──┐
        │     │
        ├──► C10 ──┐
        │          │
        └──► D10 ──┤
                   │
                   ▼
                   E10 ──► F10 ──► G10
```

Rules:

- A10 tasks 1–4 land first (missed-correction detector, schema additions). B10 cannot start until A10 Task A10.4 (handoff contract) commits.
- C10 and D10 parallelize after A10. E10 requires D10 feedback-loop metrics.
- F10 specifies the reproducible proof-run directory structure; G10 harness populates it.
- G10 requires everything. No phase may skip a prior dependency to hit its own pass gate.

## 2. Self-Improvement Harness Architecture (Replaces V4's Dogfood Scenario)

V10 harness is NOT a single multi-turn scenario like V4's 3-session flip. Instead, it's a **continuous feedback loop harness** that runs daily-to-weekly during V10 development:

### 2.1 Loop structure

1. **Capture phase (continuous, per session):** memd collects:
   - Session transcript + agent reasoning turns
   - Corrections issued by user (T5, T18, etc.)
   - Agent queries and retrieval results
   - Preference drift detections (F10)

2. **Consolidation phase (nightly, A10):** Overnight batch:
   - Detects missed corrections (A10: where agent gave advice, user contradicted)
   - Promotes stable candidate→canonical truths (A10)
   - Deduplicates, decays stale entries (A10)

3. **Routine detection phase (per session, C10):** Pattern matching:
   - Files touched in T13, T14, T15 of session (e.g., pattern: touch `migrations/*.sql` → recall `migrations/` in next query)
   - Store as candidate routine, log `routine_candidates_observed`
   - Measure accuracy when invoked

4. **Retrieval feedback phase (continuous, D10):** Track:
   - Which memory items helped answer quality (binary: useful/noisy)
   - Aggregate over 30-day window
   - Adjust index weights, re-rank depth routes

5. **Scoring phase (weekly, E10 → G10):** Recompute MEMD-10-STAR.md:
   - Run G10 scorecard regenerator in strict-mode (zero-generosity)
   - Compare to prior week
   - If any axis regresses: file recovery phase; do not close V10

### 2.2 Per-axis harness touches

| Axis | Phases | Harness fixture | Measurement |
|---|---|---|---|
| SC | A10 | missed-correction re-ingest scenario (T20–T23) | detector fires ≥1 per test session |
| CR | B10 | cross-session auto-apply (T28–T30) | T5 correction auto-surfaces in session 2 |
| PR | C10 | routine detection (T13–T15 file pattern) | `routine_candidates_observed` ≥ 3 per session |
| CH | G10 integration | multi-harness cross-test (A, B, C agents) | beliefs sync across harnesses; regressions caught |
| RR | D10 | feedback loop: query quality (30-day aggregate) | weight update δ ≤ 0.05 per cycle |
| TE | G10 integration | integrated; no V10 lift; TE contingency armed | TE < 4 → activate recovery phases |
| TP | G10 integration | integrated; no V10 lift | provenance chain reviewed, no new assertion |

## 3. Production-Floor Gate Ceremony (G10 Phase)

**G10 is not a normal phase.** It is the V10 production-floor verification
gate. It does not tag `0.1.0`; V13 owns that release.

### 3.1 Four-condition check

Run in this order. All must pass simultaneously:

**Condition 1: Composite ≥ 6.0 on regeneration**
- Task G10.1: Run scorecard regenerator on main (post all A10–F10 merges).
- Task G10.2: Assert composite = 6.40 ± 0.01 (two-decimal precision).
- Regenerator fails loud if any axis > target (anti-over-claim).

**Condition 2: Every axis ≥ 3**
- Task G10.3: Verify per-axis scores:
  - SC ≥ 3: 7 ✓
  - CR ≥ 3: 6 ✓
  - PR ≥ 3: 6 ✓
  - CH ≥ 3: 6 ✓
  - RR ≥ 3: 8 ✓
  - TE ≥ 3: 5 ✓
  - TP ≥ 3: 6 ✓

**Condition 3: Zero blocker-severity backlog**
- Task G10.4: Scan issue tracker for blocker+ issues tagged with any 10-STAR axis label. None allowed. If found, file recovery phase; do not close V10.

**Condition 4: Reproducible proof run**
- Task G10.5: Populate `docs/verification/v10-proof-runs/` with:
  - `YYYY-MM-DD-HARNESS-RUN.ndjson` (full G10 harness trace)
  - `YYYY-MM-DD-AXIS-EVIDENCE/` per-axis subdirectories with fixtures and assertions
  - `YYYY-MM-DD-NEGATIVE-CONTROLS.ndjson` (fault-injection test results)
  - `YYYY-MM-DD-HUMAN-REVIEW.md` (dated sign-off)

### 3.2 TE Contingency Activation

TE is the first axis to fall if underdelivery occurs (margin +2 vs +3 others).

**Automatic activation (blocks V10 close):**
- If G10 regeneration shows TE < 4: immediately file recovery phases.
- If D10 feedback loop shows TE degradation >3% from V9 baseline: same.

**Recovery phases (do NOT enter V11, re-harness V10):**
1. `v10-recovery-te-profile` — identify token sinks
2. `v10-recovery-te-retarget` — reallocate budgets
3. Re-run G10 harness, regenerate scorecard
4. If TE ≥ 4 after: proceed to V10 close
5. If TE < 4: do not close V10 until fixed

Currently (2026-04-22) no evidence of TE risk; contingency is precaution.

## 4. Scorecard Regenerator Strict-Mode

G10 Task G10.1 runs the scorecard regenerator with zero-generosity rules. This is different from prior milestone regenerators.

### 4.1 Regenerator input

- A10 missed-correction detector metric (count)
- B10 cross-session auto-apply fixture pass/fail
- C10 routine-detection metrics (`routine_candidates_observed` aggregate)
- D10 retrieval-feedback quality scores (30-day aggregate)
- E10 axis integration reviews (pass/fail per axis)
- F10 proof-run directory completeness check

### 4.2 Regenerator rules (strict-mode, no margin of error)

1. **No over-claim:** If harness evidence supports SC at 6.9, regen outputs 6, not 7. Conservative always.
2. **All-or-nothing per axis:** No axis credit without all its fixture assertions passing.
3. **Composite math verified to 2 decimals:** 6.40 exactly; anything else fails regen loud.
4. **Demotion allowed:** If a harness shows SC regressed to 5.8, write 5, not 6. File recovery phase.
5. **Negative control pass/fail:** If fault-injection tests fail (e.g., skip A10 detector → should break B10), regen aborts and surfaces the failure.

### 4.3 Regenerator output

Replaces the scorecard table in `docs/verification/MEMD-10-STAR.md` in place:

```markdown
## 10-Star Composite Scorecard (V10 Close)

| Axis | Weight | Score | Status |
|------|--------|-------|--------|
| Session continuity | 20% | 7/10 | A10 missed-correction detector + re-ingest |
| Correction retention | 15% | 6/10 | B10 cross-session auto-apply |
| Procedural reuse | 15% | 6/10 | C10 routine detect→store→invoke→measure→prune loop closed |
| Cross-harness continuity | 15% | 6/10 | G10 multi-harness cross-test integration |
| Raw retrieval strength | 15% | 8/10 | D10 retrieval-feedback loop tuning 30-day proven |
| Token efficiency | 10% | 5/10 | TE floor closed at V8; V10 integrated, no lift |
| Trust + provenance | 10% | 6/10 | integrated; evidence chain drilldown verified |

**Composite: 6.40 (V10 production-floor target) — regenerated YYYY-MM-DD by G10 harness run at strict-mode zero-generosity**

Evidence: docs/verification/v10-proof-runs/YYYY-MM-DD-HARNESS-RUN.ndjson

### Production-Floor Checklist ✓

1. Composite ≥ 6.0: 6.40 ✓
2. Every axis ≥ 3: SC 7, CR 6, PR 6, CH 6, RR 8, TE 5, TP 6 — all ≥3 ✓
3. Zero blocker backlog: checked YYYY-MM-DD, zero found ✓
4. Proof run: complete at docs/verification/v10-proof-runs/
```

Append a delta-history entry so prior scorecards are reconstructible. Sign:
`V10-PRODUCTION-FLOOR-VERIFIED`.

## 5. V10 Proof-Run Directory Structure

Harness populates `docs/verification/v10-proof-runs/` with these files:

```
docs/verification/v10-proof-runs/
├── YYYY-MM-DD-HARNESS-RUN.ndjson          # G10 full trace (per-turn instrumentation)
├── YYYY-MM-DD-AXIS-EVIDENCE/
│   ├── SC/
│   │   ├── missed-correction-fixture.jsonl  # A10 detector test input
│   │   ├── missed-correction-output.jsonl   # detector output (re-ingest marked)
│   │   └── SC-ASSERTION.md                  # human review of assertion
│   ├── CR/
│   │   ├── cross-session-fixture.jsonl      # B10 auto-apply test
│   │   ├── cross-session-output.jsonl
│   │   └── CR-ASSERTION.md
│   ├── PR/
│   │   ├── routine-pattern-fixture.jsonl    # C10 file-touch pattern
│   │   ├── routine-metrics.ndjson           # `routine_candidates_observed` log
│   │   └── PR-ASSERTION.md
│   ├── RR/
│   │   ├── retrieval-feedback-30day.ndjson  # D10 quality feedback (30-day aggregate)
│   │   ├── weight-deltas.json               # index weight changes applied
│   │   └── RR-ASSERTION.md
│   ├── CH/
│   │   ├── multi-harness-cross-test.ndjson  # A, B, C agent cross-harness flip
│   │   └── CH-ASSERTION.md
│   ├── TE/
│   │   ├── integration-review.md            # TE contingency status + decision
│   │   └── TE-ASSERTION.md
│   └── TP/
│       ├── provenance-drilldown-review.md   # evidence chain verification
│       └── TP-ASSERTION.md
├── YYYY-MM-DD-NEGATIVE-CONTROLS.ndjson    # fault-injection test results
│   # Variants: skip A10 detector, drop B10 auto-apply, etc.
│   # Each must fail as designed (negative control proves harness is honest)
└── YYYY-MM-DD-HUMAN-REVIEW.md             # dated sign-off on all four conditions
```

## 6. Feature-Flag Graduation Calendar

All feature flags for A10–F10 ship flag-off until G10 harness passes. Post-pass, flag-flip ordering (each flip = its own commit, each after a 7-day clean window):

1. `MEMD_A10_MISSED_CORRECTION_DETECTOR` = 1 (Task A10.4)
2. `MEMD_B10_AUTO_APPLY_CORRECTION` = 1 (Task B10.5)
3. `MEMD_C10_ROUTINE_DETECT_STORE_INVOKE` = 1 (Task C10.6)
4. `MEMD_D10_RETRIEVAL_FEEDBACK_LOOP` = 1 (Task D10.5)
5. `MEMD_F10_PROOF_RUN_PUBLISH` = 1 (Task F10.2)

**Calendar spillover:** 5 graduations × 7-day clean window = 35 days post-G10. V10 code-complete and G10 harness pass close the milestone; flag-graduation runs parallel to post-V10 maintenance.

If a flag flip surfaces TE regression during the 7-day window, activate TE-recovery immediately (per contingency rule in §3.2).

## 7. Cross-Phase API Surface Summary

| Introduced in | Symbol / Path | Consumed by |
| --- | --- | --- |
| A10 | `memd_core::detector::missed_correction::*` | B10 (surface to user context), G10 (assertion) |
| A10 | `docs/contracts/missed-correction-reingestion.md` | B10 (re-ingest rules), E10 (integration review) |
| B10 | `memd_core::correction::auto_apply::*` | C10 (routine routine-apply), G10 |
| B10 | `.memd/logs/auto-applied-corrections.ndjson` | G10 (B10 assertion evidence) |
| C10 | `memd_core::routine::detect_store_invoke_measure_prune::*` | D10 (metrics), G10 |
| C10 | `.memd/logs/routine-candidates.ndjson` | G10 (C10 assertion) |
| D10 | `memd_core::index::feedback_loop::*` | E10 (integration), G10 |
| D10 | `.memd/logs/retrieval-feedback-30day.ndjson` | G10 (RR assertion) |
| E10 | G10 regenerator scorecard input (all axes) | G10 (regeneration) |
| F10 | `docs/verification/v10-proof-runs/` directory | G10 production-floor certification |
| G10 | `docs/verification/MEMD-10-STAR.md` regenerated | V10 close ceremony |

## 8. V10 Close Ceremony

**Timing:** After G10 harness passes and proof-run directory is complete.

**Ceremony steps:**
1. Run G10 scorecard regenerator (Task G10.1).
2. Verify composite = 6.40 ± 0.01 (Task G10.2).
3. Verify every axis ≥ 3 (Task G10.3).
4. Scan backlog for blocker-severity 10-STAR issues (Task G10.4). Zero allowed.
5. Review proof-run directory completeness (Task G10.5).
6. If all pass, sign `docs/verification/v10-proof-runs/YYYY-MM-DD-HUMAN-REVIEW.md`.
7. Do not tag `0.1.0`; V13 owns that tag.
8. Publish handoff notes: link to proof-run directory + scorecard + delta from V9.

If any condition fails: do not close V10. File recovery phase instead.

## 9. Non-Goals

- Lifting session_continuity beyond 7 (V10 owns +1; V7 + V8 already landed +1 each)
- Lifting correction_retention beyond 6 (V10 owns +1 final; no further lift in 0.1.0)
- Procedural reuse detection better than 85% precision (tuning scope, not data science lift)
- Improving token_efficiency in V10 (TE closes at 5 in V8; V10 integrates, no lift)
- Lifting cross_harness or trust_provenance (integrated, not owned, no credit)
- Any axis to 9+ (V10 target is 6.40; SOTA/release roadmap is V11+)

## 10. Backlog and Recovery Phases

Pre-V10 recovery phases (must land before G10):
- v8-recovery-te-*: any TE shortfall from V8 (if not already closed)
- v9-recovery-sc-*: any SC shortfall from V9 (if not already closed)

During V10 (only post-G10 if harness surfaces regression):
- v10-recovery-te-profile: token sink analysis
- v10-recovery-te-retarget: budget reallocation
- v10-recovery-<axis>-<date>: any other axis regression

## 11. Changelog

- 2026-04-22 initial spec.
- 2026-04-22 revision:
  - Composite 5.60 → 6.40 (binding from 0.1.0-CONTRACT.md).
  - Phases A10–G10 names and scope clarified (self-improvement harness architecture, not V4-style dogfood scenario).
  - Original text framed V10 as the 0.1.0 release gate; superseded by 2026-05-05 correction above.
  - G10 scorecard regenerator strict-mode rules added (zero-generosity, no over-claim, all-or-nothing per axis).
  - Release proof-run directory structure specified (NDJSON traces + per-axis subdirs + negative controls + human review).
  - TE contingency activation rules added (automatic if TE < 4; recovery phases specified).
  - Feature-flag graduation calendar post-G10 (5 × 7-day clean windows = 35-day spillover).
  - Ceremony steps clarified (not a normal phase; gate verification only).
- 2026-05-05 correction: aligned stale release-gate language to the active
  `0.1.0-CONTRACT.md`: V10 is production floor; V13 owns `0.1.0`.

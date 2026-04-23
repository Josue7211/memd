---
milestone: v13
name: Evidence + Release
status: planned
opened: 2026-04-22
revised: 2026-04-22
depends_on: [../0.1.0-CONTRACT.md, ../0.1.0-AXIS-OWNERSHIP.md, ../../theory/MEMD-SOTA-THEORY.md, milestones/MILESTONE-v12.md]
composite_pre: 7.75
composite_target: 8.50
axes_lifted: [session_continuity, correction_retention, procedural_reuse, raw_retrieval, trust_provenance]
axes_integrated_with: [cross_harness, token_efficiency]
non_goals: [cross_harness, token_efficiency]
---

# Milestone v13 — Evidence + Release

## Goal

V13 is the release gate for memd 0.1.0. It closes the SOTA push (V11–V13) by lifting five axes to 9/10 (near-perfect) and publishing reproducible proof that memd beats published SOTA on four public benchmarks simultaneously. The release harness (G13) runs the full axis battery and regenerates the 10-STAR scorecard in zero-generosity mode. If any axis regresses at G13 close, 0.1.0 does not tag.

**TE is at the SOTA floor with zero margin (7/10, no room for regression).** Any TE degradation during V13 close blocks 0.1.0 tag directly. CH holds at 8 and is not lifted in V13. The 5-condition release gate (composite ≥8.0, every axis ≥7, zero blocker backlog, reproducible proof run, SOTA head-to-head) is the milestone's completion condition.

## 10-STAR axis targets (pre / post)

Scores imported from 0.1.0-CONTRACT.md and 0.1.0-AXIS-OWNERSHIP.md. V13 owns SC +1, CR +1, PR +1, RR +1, TP +1; integrates with CH, TE; non-goals CH, TE (hold from prior milestones).

| axis | weight | pre | post | basis |
| --- | --- | --- | --- | --- |
| session_continuity   | 20% | 8 | 9 | A13 cross-device sync (CRDT merge) + B13 dormant-project recovery (30-day wake) |
| correction_retention | 15% | 7 | 8 | C13 multi-hop correction chains (A corrects X → downstream Y, Z affected) |
| procedural_reuse     | 15% | 8 | 9 | D13 routine composition (auto-suggest A+B=C when pattern seen) + E13 cross-workspace sharing |
| cross_harness        | 15% | 8 | 8 | non-goal (V12 closed at 8, V13 integrates; no lift claimed) |
| raw_retrieval        | 15% | 8 | 9 | F13 domain-tuned retrieval (code/docs/conversational) + public-bench sweep |
| token_efficiency     | 10% | 7 | 7 | non-goal (V11 closed at 7, floor with zero margin, V13 integrates; no lift claimed) |
| trust_provenance     | 10% | 8 | 9 | G13 export + third-party replay harness (compliance-grade audit trails) |

**Composite: 7.75 → 8.50** (weighted arithmetic: 0.20×9 + 0.15×8 + 0.15×9 + 0.15×8 + 0.15×9 + 0.10×7 + 0.10×9 = 1.80 + 1.20 + 1.35 + 1.20 + 1.35 + 0.70 + 0.90 = **8.50** ✓).

## Release gate checklist

**0.1.0 ships only when all five conditions hold simultaneously:**

1. **Composite ≥ 8.0 on the 10-STAR scorecard** (V13 target 8.50).
   - V13 hardness: zero-generosity regrade rules applied by G13 harness regenerator.
   - Verification: `docs/verification/MEMD-10-STAR.md` composite row re-written by scorecard regenerator at G13 close.

2. **Every axis ≥ 7/10** (SOTA floor, per 0.1.0-CONTRACT.md).
   - SC 9 ✓, CR 8 ✓, PR 9 ✓, CH 8 ✓, RR 9 ✓, TE 7 ✓ (zero margin), TP 9 ✓
   - **TE zero-margin flag**: TE closes at floor 7 / margin 0. Any TE regression during V13 close blocks 0.1.0 tag directly (see Contingency Plan below).

3. **Zero blocker-severity backlog** tagged with any 10-STAR axis label.
   - Verification: `memd lookup --backlog --severity blocker --filter "axis:*"` returns empty.

4. **Reproducible proof run** in `docs/verification/release-0-1-0/` with per-axis evidence trail.
   - Per-axis NDJSON: `docs/verification/release-0-1-0/YYYY-MM-DD-axis-<name>.ndjson`
   - Dated human review: `docs/verification/release-0-1-0/YYYY-MM-DD-axis-<name>-review.md`
   - Release harness run log: `docs/verification/release-0-1-0/YYYY-MM-DD-g13-harness.ndjson`
   - Verification: G13 harness task G13.8 populates directory and documents proof per-axis.

5. **Head-to-head SOTA proof** — on ≥1 public benchmark per axis (where applicable), memd beats published SOTA by ≥5pp margin with dated transcript, or is explicitly parity-with-margin for axes where no public bench exists.
   - RR: LoCoMo (token F1), LongMemEval (judged accuracy), MemBench (MC accuracy), ConvoMem (accuracy) — all four, ≥5pp margin per bench.
   - SC: LongMemEval multi-session (dormant recovery 30-day test) + LoCoMo 300-turn (compaction survival). Parity-with-margin acceptable if <2pp diff from SOTA.
   - CR: LoCoMo multi-turn subset + internal CorrectionPropagation bench. Parity acceptable (<1pp diff).
   - PR: No published bench (memd publishes its own Procedural-Reuse bench in V5 substrate suite). Composition + sharing + curation measured internally. Parity-with-margin.
   - CH: No single public bench for cross-harness (memd's G9 multi-user adversarial suite + G12 universal-protocol parity bench are substrate tests). Parity-with-margin (same memory view in ≥2 harnesses observed).
   - TP: No SOTA number published. Third-party replay harness + audit-trail compliance-readiness (format but not certified). Parity-with-margin.
   - Verification: Margin table lives in `docs/verification/release-0-1-0/YYYY-MM-DD-margin-targets.md`.

## Phases

Outline only (no implementation specs created in V13 milestone-land phase). See `ROADMAP.md` → "V13: Evidence + Release".

- **A13**: Cross-device sync (session_continuity +1). CRDT-style merge on memory state conflicts. Dormant-project recovery: open project after 30 days, wake produces full focus recall without cold-start.
- **B13**: Compaction-aware long-session perf. 1000+ turn sessions with multiple compaction cycles. Session state survives heavy compaction without quality loss.
- **C13**: Multi-hop correction chains (correction_retention +1). Correction A about value X downstream-affects values Y, Z that derived from X. Provenance graph shows all affected items.
- **D13**: Routine composition (procedural_reuse +1). memd suggests A+B=C when it observes the pattern. Composition happens auto when >2 same-day uses of A followed by B.
- **E13**: Cross-workspace routine sharing (procedural_reuse +1 continued). Named routines shareable across workspaces. Trust + provenance on routine origin (who authored, when, usage count).
- **F13**: Domain-tuned retrieval (raw_retrieval +1). Code-context-aware retrieval (file paths, function names), docs-aware (API reference sections), conversational (user-specific patterns). Public-bench sweep (LoCoMo, LongMemEval, MemBench, ConvoMem) all ≥5pp margin.
- **G13**: Export + third-party replay (trust_provenance +1). Full 0.1.0 release harness. Provenance snapshot export format. Independent third-party can ingest export and replay the same answer from memory state. Audit-trail compliance format (not certified, but audit-ready). Scorecard regenerator in zero-generosity strict mode. Release proof-run directory populated.

## Completion gate

G13 harness (full 0.1.0 release harness with all axes represented):

- **SC**: Dormant-project recovery (wake-30d workspace after 30-day gap, focus re-hydrated correctly).
- **CR**: Multi-hop correction chain visible in next-session behavior (correction to X affects Y + Z downstream).
- **PR**: Routine composition suggested and used; cross-workspace share works with provenance visible.
- **RR**: All four public benches (LoCoMo, LongMemEval, MemBench, ConvoMem) pass with ≥5pp margin over published SOTA.
- **TE**: (Integration only) Token counts stable; V11 dynamic compiler remains functional. No regression.
- **CH**: (Integration only) Multi-harness session live (claude-code AND codex running simultaneously, same memory state visible). No regression from V12.
- **TP**: Export and third-party replay harness passes. Audit-trail format documented. Compliance-readiness confirmed by human review (not certified).
- **All axes**: Negative controls fire as designed (suppress feature → harness assertion fails).

Evidence: recorded trace + G13 harness NDJSON + regenerated 10-STAR composite in `docs/verification/MEMD-10-STAR.md` via G13 scorecard regenerator in strict mode.

## TE zero-margin contingency plan

**Trigger**: G13 harness run shows TE <7 at regeneration time.

**Action**: Do not tag 0.1.0. Instead, file a TE-recovery phase (v13.5) that targets the specific axis regression.

1. Roll back V13 axis credits to V12 state (SC 8, CR 7, PR 8, RR 8, TP 8; composite → 7.75).
2. Create `docs/phases/v13.5/` with single phase: C13.5 (TE recovery). Scope: root-cause TE regression, restore dynamic compiler correctness or revert breaking change.
3. C13.5 exit gate: TE ≥7 confirmed by G13.5 harness re-run. No other axes touched.
4. Land V13.5 as a separate milestone (v13.5-recovery) and tag 0.1.0 only after V13.5 closes with TE ≥7.

This ensures TE floor is never violated at release.

## Per-axis harness assertions (required for axis credit)

| axis | concrete assertion | fixture |
| --- | --- | --- |
| session_continuity   | project reopened after 30-day dormancy; wake produces prior session's focus items without cold-start | G13 dormant-project scenario |
| correction_retention | correction to value X applies downstream to derived items Y, Z in next session | G13 multi-hop correction scenario |
| procedural_reuse     | routine composition A+B=C auto-suggested when >2 same-day uses observed; cross-workspace share carries provenance | G13 routine-curation scenario |
| raw_retrieval        | LoCoMo F1, LongMemEval accuracy, MemBench MC, ConvoMem accuracy all ≥5pp above published SOTA | public-bench proof run |
| trust_provenance     | export snapshot; third-party replay produces same answer from same memory state | G13 export + replay harness |

Missing any assertion → axis does not lift, milestone does not close.

## Per-axis integration points (axes not owned by V13)

V13 does not claim credit on CH, TE. Integration notes:

- **CH 8 (V12 closed)**: V13's multi-harness release harness (A13 + E13 + G13) must show CH parity (same memory observable in ≥2 harnesses simultaneously). No regression allowed.
- **TE 7 (V11 closed)**: V13's G13 harness must show TE ≥7 (zero margin). Dynamic compiler must remain functional. No regression allowed. If TE <7 at G13 close, invoke contingency plan.

## Non-goals

- TE lift (capped at 7/10 by V11 theory — 9/10 requires production telemetry post-release).
- CH lift (V12 closed CH at 8, not lifting in V13).
- New public benches (V13 uses existing: LoCoMo, LongMemEval, MemBench, ConvoMem). Hardness of published SOTA is reference only.

## Public-bench margin targets

| Axis | Benchmark | SOTA baseline | V13 target | Margin |
| --- | --- | --- | --- | --- |
| RR | LoCoMo (token F1) | 0.72 | 0.77 | +5pp |
| RR | LongMemEval (judged acc) | 0.68 | 0.73 | +5pp |
| RR | MemBench (MC acc) | 0.75 | 0.80 | +5pp |
| RR | ConvoMem (accuracy) | 0.70 | 0.75 | +5pp |
| SC | LongMemEval multi-session | 0.65 | ≥0.63 (parity-with-margin) | parity |
| CR | LoCoMo multi-turn | 0.58 | ≥0.57 (parity-with-margin) | parity |

Widest goal (RR): 5pp margin on all four benches. Tightest (CR, SC): parity-with-margin acceptable (both V12+ SOTA already).

## Composite math verification

Weights: SC 20%, CR 15%, PR 15%, CH 15%, RR 15%, TE 10%, TP 10%.

Target: (0.20 × 9) + (0.15 × 8) + (0.15 × 9) + (0.15 × 8) + (0.15 × 9) + (0.10 × 7) + (0.10 × 9)
= 1.80 + 1.20 + 1.35 + 1.20 + 1.35 + 0.70 + 0.90
= **8.50** ✓

## Feature-flag graduation calendar

All features lifted in V13 ship feature-flagged. Graduation order (each 7-day clean window):

1. `MEMD_A13_CROSS_DEVICE_SYNC` = 1
2. `MEMD_D13_ROUTINE_COMPOSITION` = 1
3. `MEMD_F13_DOMAIN_TUNED_RETRIEVAL` = 1
4. `MEMD_G13_EXPORT_THIRD_PARTY_REPLAY` = 1

**Calendar spillover**: 4 graduations × 7 days = 28 days of post-G13 observation. V13 code-complete and G13 harness pass are the milestone-close bar; flag-graduation runs post-0.1.0-tag into operational monitoring.

## Theory alignment

- **SC 8→9 (SOTA+margin)**: Cross-device sync with CRDT-merge + dormant-project recovery are the V13 SC targets from MEMD-SOTA-THEORY.md § Session continuity at 9/10 ("Cross-project continuity... cross-device sync with CRDT-style merge on conflicts... Compaction-aware recall").
- **CR 7→8 (SOTA+margin)**: Multi-hop correction chains are the V13 CR target from MEMD-SOTA-THEORY.md § Correction retention at 8/10 ("Multi-hop correction chains: correction A about B downstream-affects C").
- **PR 8→9 (SOTA+margin)**: Routine composition + cross-workspace sharing are V13 PR targets from MEMD-SOTA-THEORY.md § Procedural reuse at 9/10 ("Routine composition... Routines... shared across workspaces...").
- **RR 8→9 (SOTA+margin)**: Domain-tuned retrieval + 5pp margin on all four public benches are V13 RR targets from MEMD-SOTA-THEORY.md § Raw retrieval at 9/10 ("Beats published SOTA by ≥5pp on all four public benches simultaneously").
- **TP 8→9 (SOTA+margin)**: Third-party-verifiable export + compliance-grade audit trails are V13 TP targets from MEMD-SOTA-THEORY.md § Trust + provenance at 9/10 ("Export... independent replay... Compliance-grade audit trails").

## Changelog

- 2026-04-22 opened. V13 is the final milestone and release gate for memd 0.1.0. Composite 7.75 → 8.50. Five axes lifted (SC, CR, PR, RR, TP) to 9/10 (near-perfect). Two axes integrated (CH, TE) with zero-margin flag on TE. Phase letters a13-g13 outlined, no specs created (per binding instructions). Release gate checklist with 5 conditions embedded. TE contingency plan for regression. Public-bench margin targets named (5pp on RR, parity on SC/CR). Composite math verified at 8.50 exactly. Feature-flag graduation: 4 graduations over 28 days. CH and TE non-goals explicit (hold from V12/V11 respectively).


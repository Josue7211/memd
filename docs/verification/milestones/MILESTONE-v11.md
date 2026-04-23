---
milestone: v11
name: Compiler SOTA
status: planned
opened: 2026-04-22
revised: 2026-04-22
depends_on: [../0.1.0-CONTRACT.md, ../0.1.0-AXIS-OWNERSHIP.md, ../../theory/MEMD-SOTA-THEORY.md, milestones/MILESTONE-v10.md]
composite_pre: 6.40
composite_target: 6.95
axes_lifted: [session_continuity, correction_retention, token_efficiency]
axes_integrated_with: []
non_goals: [procedural_reuse, cross_harness, raw_retrieval, trust_provenance]
---

# Milestone v11 — Compiler SOTA

## Goal

V11 pushes three axes to SOTA baseline (7/10). The centerpiece is dynamic per-turn compiler: instead of V4's static 4-layer cap, the compiler decides per-turn what kinds of memory to include, at what depth, based on turn intent. This achieves token efficiency (TE) at 7/10 (SOTA baseline) with wake median ≤1500 tokens. Cross-project project awareness lifts session_continuity (SC) and correction_retention (CR) by removing inter-project pollution and enabling compaction-aware recall. Contradiction-detection latency drops to ≤1 s wake-to-surface, powering the CR lift.

This is the first of three SOTA-push milestones (V11–V13). Composite rises from V10's 6.40 (production floor) to 6.95 at V11 close. Release gate (8.50) lands at V13.

## 10-STAR axis targets (pre / post)

Scores imported from 0.1.0-CONTRACT.md. V11 owns SC +1, CR +1, TE +2; integrates with none; non-goals PR, CH, RR, TP.

| axis | weight | pre | post | basis |
| --- | --- | --- | --- | --- |
| session_continuity   | 20% | 7 | 8 | A11 project-aware wake + B11 compaction-aware recall |
| correction_retention | 15% | 6 | 7 | C11 silent-correction detection ≤1s latency |
| procedural_reuse     | 15% | 6 | 6 | non-goal (V12 owned) |
| cross_harness        | 15% | 6 | 6 | non-goal (V12 owned) |
| raw_retrieval        | 15% | 8 | 8 | non-goal (V13 owned) |
| token_efficiency     | 10% | 5 | 7 | D11 dynamic compiler (per-turn depth), E11 cost UI, G11 proof harness |
| trust_provenance     | 10% | 6 | 6 | non-goal (V12 owned) |

**Composite: 6.40 → 6.95** (weighted arithmetic: 0.20×8 + 0.15×7 + 0.15×6 + 0.15×6 + 0.15×8 + 0.10×7 + 0.10×6 = 1.60 + 1.05 + 0.90 + 0.90 + 1.20 + 0.70 + 0.60 = 6.95).

## Phases

See `ROADMAP.md` → "V11: Compiler SOTA". Phase docs (outline only, no implementation specs created in V11 milestone-land phase):

- **A11**: Project-aware wake (session_continuity +1). Cross-project memory hydration without pollution. Focus pinning per-project at wake time. Measured via 3-project scenario.
- **B11**: Compaction-aware recall. Post-compaction wake produces compressed-but-optimal context, not truncated. Session state survives heavy compaction runs.
- **C11**: Silent-correction detection. Latency target ≤1 s wake-to-surface. Detection rules: user rephrases question ≥2× or ignores suggested value repeatedly.
- **D11**: Dynamic per-turn compiler. Replaces V4's static 4-layer cap. Decision tree: turn intent → memory kinds and depth. Shannon-ish baseline (no redundancy per turn).
- **E11**: Cost UI (operator-facing). Exposes $/M ledger, tunable cost target, per-turn cost breakdown. Surfaces via `memd configure` (V8 surface).
- **F11**: Wake median benchmark. Typical workload ≤1500 tokens measured (V4 baseline: 2000).
- **G11**: Proof harness (multi-axis assertions). 3-project session scenario + silent-correction triggers + dynamic-compiler correctness.

## Completion gate

G11 harness (multi-project scenario with silent-correction triggers):

- Project-aware wake: user switches A → B → A; wake-A re-hydrates A's focus without B's items visible (A11)
- Compaction survival: heavy compaction post-project-switch; wake-A still recovers correct context (B11)
- Silent-correction detection: user rephrases "what was X?" twice; memd surfaces prior answer as potentially wrong with <1s latency (C11)
- Dynamic compiler correctness: per-turn compiler produces stable token counts within budget, per-turn depth decisions observable (D11)
- Cost UI: operator sets `cost_target_per_turn_cents=0.5`; compiler respects it (E11)
- Wake median: across 50-turn workload, median ≤1500 tokens (F11)
- Negative controls: suppress A11 project isolation → wake polluted; inject stale compaction state → assert recovery fails; mute C11 detector → assert no flag

Evidence: recorded trace + G11 harness NDJSON + regenerated 10-STAR composite in `docs/verification/MEMD-10-STAR.md` via G11 scorecard regenerator.

## Per-axis harness assertions (required for axis credit)

| axis | concrete assertion | fixture |
| --- | --- | --- |
| session_continuity   | project-A wake excludes items from project-B session in same workspace | shared/projects/3-project-scenario.jsonl |
| correction_retention | memd surfaces "prior answer T5 may be wrong" flag when user re-asks question T15+T25 | G11 silent-correction trigger |
| token_efficiency     | dynamic compiler decides per-turn depth; wake median 50-turn workload ≤1500 tokens | G11 workload + D11 metrics |

Missing any assertion → axis does not lift, milestone does not close.

## Per-axis integration points (axes not owned by V11)

V11 does not claim credit on PR, CH, RR, TP. Integration notes for future milestones:

- **V10 → V11**: V10 claims SC+1 (6→7), CR+1 (5→6). V11 claims SC+1 (7→8), CR+1 (6→7). No double-claim. V10's self-improvement mechanisms are preserved; V11's project awareness is orthogonal.
- **V12/V13 integrations**: PR (V12 owned), CH (V12 owned), TP (V12 owned) do not touch V11 compiler outputs. RR (V13 owned) integration point: V13's domain-tuned retrieval may reweight compiler's recall-depth decisions post-hoc; V11 compiler owns the initial depth contract.

## Non-goals

- Procedural reuse detection (V12 owned, continues from V10 seeding)
- Cross-harness parity on dynamic-compiler decisions (V12 owned as CH +2)
- Raw retrieval public-bench chasing (V13 owned)
- Trust/provenance graph or audit UI (V12/V13 owned)

## Composite math verification

Weights: SC 20%, CR 15%, PR 15%, CH 15%, RR 15%, TE 10%, TP 10%.

Target: (0.20 × 8) + (0.15 × 7) + (0.15 × 6) + (0.15 × 6) + (0.15 × 8) + (0.10 × 7) + (0.10 × 6)
= 1.60 + 1.05 + 0.90 + 0.90 + 1.20 + 0.70 + 0.60
= **6.95** ✓

## Feature-flag graduation calendar

Dynamic compiler and project awareness ship feature-flagged. Graduation order (each 7-day clean window):

1. `MEMD_A11_PROJECT_AWARE_WAKE` = 1
2. `MEMD_D11_DYNAMIC_COMPILER` = 1
3. `MEMD_C11_SILENT_CORRECTION_DETECT` = 1

**Calendar spillover:** 3 graduations × 7 days = 21 days of post-G11 observation. V12 planning must account for flag-ops work; V12 phase A12 is not blocked on graduation completion — only on the handoff commit from V11's last phase.

## Theory alignment

- **SC 7→8 (SOTA baseline)**: Cross-project continuity is a core SOTA claim in MEMD-SOTA-THEORY.md § Session continuity. Wake-A without B pollution + compaction-aware recall are the enabling pieces.
- **CR 6→7 (SOTA baseline)**: Silent-correction detection (user repeatedly rephrases or ignores) is defined in MEMD-SOTA-THEORY.md § Correction retention at the 7/10 line ("contradiction detection triggers on direct conflicts with <5 s latency"). V11 tightens to ≤1 s.
- **TE 5→7 (SOTA baseline)**: Dynamic per-turn compiler is the centerpiece of MEMD-SOTA-THEORY.md § Token efficiency at 7/10 ("Dynamic per-turn compiler decides per-turn what kinds of memory to include at what depth"). Wake median ≤1500 tokens is the measurement bar.

## Changelog

- 2026-04-22 opened. V11 is the first SOTA-push milestone after V10 production-floor close. Composite 6.40 → 6.95. SC +1 (project awareness), CR +1 (silent-correction detection ≤1s), TE +2 (dynamic compiler). Phase letters a11-g11 outlined, no specs created (per binding instructions). Non-goals explicit (PR, CH, RR, TP). Feature-flag calendar: 3 graduations over 21 days, spillover into V12 planning window. Composite math verified at 6.95 exactly.

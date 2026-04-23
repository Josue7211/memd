---
milestone: v15
name: Self-Tuning Compiler
status: planned
opened: 2026-04-22
depends_on: [v14, ../../verification/1.0.0-CONTRACT.md, ../../verification/1.0.0-AXIS-OWNERSHIP.md]
composite_pre: 8.60
composite_target: 8.70
axes_lifted: [token_efficiency]
axes_integrated_with: [procedural_reuse, cross_harness]
---

# Milestone v15 Audit — Self-Tuning Compiler

## Goal

Compiler learns per-user per-harness what token budget yields best
downstream task quality, adjusts automatically. Builds on V11 dynamic
per-turn compiler + V14 telemetry. TE +1 (8→9) = self-tuning operational
across ≥3 harness-user pairs with measurable quality lift and zero
budget regression.

## 10-STAR axis targets (pre / post)

| axis | weight | pre | post | basis |
| --- | --- | --- | --- | --- |
| session_continuity   | 20% | 9 | 9 | — |
| correction_retention | 15% | 8 | 8 | — |
| procedural_reuse     | 15% | 9 | 9 | INT (compiler learns routines; no credit) |
| cross_harness        | 15% | 8 | 8 | INT (compiler learns per-harness patterns; no credit) |
| raw_retrieval        | 15% | 9 | 9 | — |
| token_efficiency     | 10% | 8 | 9 | **OWNS +1** — self-tuning compiler operational |
| trust_provenance     | 10% | 9 | 9 | — |

**Composite: 8.60 → 8.70**.

## Phases (planned)

- **A15** Self-tuning loop — compiler reads V14 telemetry, adjusts per-turn budget targets
- **B15** Quality-preserving guard — no budget change accepted unless downstream task quality holds or lifts
- **C15** Per-harness tuning profiles (claude-code vs codex vs gemini vs custom)
- **D15** Rollback + manual override (`memd configure compiler.mode=static|dynamic|self_tuning`)
- **E15** A/B bench: self-tuning vs V11 dynamic vs V8 static — token savings + quality delta
- **F15** V15 gate harness (≥3 harness-user pairs, ≥60-day tuning window, TE regen)

## Completion gate

1. ≥60-day self-tuning dogfood across ≥3 harness-user pairs.
2. Measurable token savings ≥20% vs V11 dynamic baseline at equal or better task quality.
3. Zero quality regression events escalated (guard prevents bad tuning).
4. Manual override works (`memd configure compiler.mode=static`).
5. 10-STAR composite regenerated ≥8.70 with TE=9.

## Non-goals

- Cross-user tuning (V17 owns routine marketplace; V15 stays per-user)
- Info-theoretic optimality (V20 owns TE 9→10)

## Changelog

- 2026-04-22 opened.

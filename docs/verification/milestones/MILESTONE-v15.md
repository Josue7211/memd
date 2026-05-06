---
milestone: v15
name: Self-Tuning Compiler
status: code_complete_dogfood_pending
opened: 2026-04-22
code_complete: 2026-05-06
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

## Phases

- **A15** Self-tuning loop — complete in code; compiler reads V14 telemetry and derives budget targets.
- **B15** Quality-preserving guard — complete in code; no budget change accepted unless task quality holds or lifts.
- **C15** Per-harness tuning profiles — complete in code for per-user/per-harness profile groups.
- **D15** Rollback + manual override — complete in code via `memd configure compiler.mode=static|dynamic|self_tuning`.
- **E15** A/B bench — complete in code for static vs dynamic vs self-tuning budget comparison.
- **F15** V15 gate harness — synthetic proof passed; real ≥60-day tuning window remains pending.

## Completion gate

1. ≥60-day self-tuning dogfood across ≥3 harness-user pairs — **pending real time**.
2. Measurable token savings ≥20% vs V11 dynamic baseline at equal or better task quality — **passed in synthetic proof**; minimum savings 27.73%.
3. Zero quality regression events escalated (guard prevents bad tuning) — **passed in synthetic proof**; minimum quality delta +0.02.
4. Manual override works (`memd configure compiler.mode=static`) — **passed**.
5. 10-STAR composite regenerated ≥8.70 with TE=9 — **provisional proof marker passed**; final close waits on 60-day dogfood.

## Evidence

- Core tests: `cargo test -p memd-core self_tuning -- --nocapture`
- Client tests: `cargo test -p memd-client self_tuning_v15 -- --nocapture`
- Suite: `RUN_DATE=2026-05-06 scripts/verify/v15-self-tuning-suite.sh`
- Summary: `docs/verification/v15-proof-runs/2026-05-06-self-tuning-suite.md`
- Artifact: `docs/verification/v15-proof-runs/2026-05-06-self-tuning-suite.ndjson`

## Non-goals

- Cross-user tuning (V17 owns routine marketplace; V15 stays per-user)
- Info-theoretic optimality (V20 owns TE 9→10)

## Changelog

- 2026-04-22 opened.
- 2026-05-06 V15 code complete. Self-tuning core, telemetry-to-profile runtime, `memd compiler` CLI, config override keys, A/B proof harness, and V15 proof artifacts landed. Real 60-day dogfood gate remains open.

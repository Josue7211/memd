---
milestone: v10
name: Self-Improvement
status: closed
opened: 2026-04-22
revised: 2026-05-05
depends_on: [v9]
composite_pre: 5.60
composite_target: 6.40
axes_lifted: [session_continuity, correction_retention, procedural_reuse, raw_retrieval]
axes_integrated_with: [cross_harness, token_efficiency, trust_provenance]
---

# Milestone v10 Audit — Self-Improvement

## Goal

memd closes the V10 production-floor gate, not the `0.1.0` release gate.
Self-improvement harness measures and refines the four owned axes: session
continuity detects + re-ingests missed corrections without user re-prompt
(SC +1); correction retention applies corrections across sessions automatically
(CR +1); procedural reuse detects, stores, invokes, measures, and prunes routines
end-to-end (PR +2, largest lift); raw retrieval tunes weights, depth, and index
based on observed quality (RR +1). Composite 5.60 → 6.40 at zero-generosity
regrade. Every other axis integrated and reviewed but not lifted. `0.1.0` tags
at V13 per `docs/verification/0.1.0-CONTRACT.md`.

## 10-STAR axis targets (pre / post)

Baseline from MILESTONE-v9.md (per 0.1.0-CONTRACT.md):

| axis | weight | pre | post | basis |
| --- | --- | --- | --- | --- |
| session_continuity   | 20% | 6 | 7 | A10 missed-correction detector + re-ingest harness (self-healing) |
| correction_retention | 15% | 5 | 6 | B10 cross-session auto-apply + observability (no user re-prompt needed) |
| procedural_reuse     | 15% | 4 | 6 | C10 routine detection instrumented in V4, live-fired in V5, tuned in V10: detect → store → invoke → measure → prune loop closed |
| cross_harness        | 15% | 6 | 6 | integrated; G10 harness cross-tests all axes (no V10 lift claimed) |
| raw_retrieval        | 15% | 7 | 8 | D10 weight/depth/index tuning via query-answer quality feedback |
| token_efficiency     | 10% | 5 | 5 | TE closes at 5/10, floor 3/10, margin +2 (tightest). No lift in V10; contingency documented (see below). |
| trust_provenance     | 10% | 6 | 6 | integrated; drilldown + evidence paths reviewed (no V10 lift claimed) |

**Composite: 5.60 → 6.40** (weighted: 0.20×7 + 0.15×6 + 0.15×6 + 0.15×6 + 0.15×8 + 0.10×5 + 0.10×6 = 1.40 + 0.90 + 0.90 + 0.90 + 1.20 + 0.50 + 0.60 = 6.40 exactly).

## Phases

See `docs/phases/v10/V10-INTEGRATION.md` for cross-phase coordination. Phase docs at `docs/phases/v10/phase-{a10..g10}-*.md` (not in scope for this revision).

- **A10** Missed-correction detector — scan session transcript for advice memd gave that user contradicted; mark for auto-reingest on next write to that claim.
- **B10** Auto-apply corrections — cross-session: when session 2 ingests T5 correction to claim from session 1 T2, surface to session 1's context if re-opened.
- **C10** Routine detect-store-invoke-measure-prune — close the procedural loop: detect pattern in transcript, store as candidate routine, allow agent to invoke, measure accuracy, prune if noise >threshold.
- **D10** Retrieval quality feedback loop — index weight tuning: track which memory items helped vs hurt agent answer quality; adjust weights based on 30-day feedback.
- **E10** G10 harness fixtures — V10 production-floor verification harness.
- **F10** Proof-run spec — reproducible evidence directory at `docs/verification/v10-proof-runs/`.
- **G10** Scorecard regenerator strict-mode — recompute MEMD-10-STAR.md at production-floor grade (zero-generosity, no margin of error).

## V10 Production-Floor Gate Checklist

V10 closes only when ALL four hold simultaneously:

1. **Composite ≥ 6.0 on G10 regeneration:** 6.40 ✓ (verified to 2 decimals)
2. **Every axis ≥ 3/10 on zero-generosity regrade:**
   - SC 7 ✓ | CR 6 ✓ | PR 6 ✓ | CH 6 ✓ | RR 8 ✓ | TE 5 ✓ | TP 6 ✓
   - All ≥ 3 ✓
3. **Zero blocker-severity backlog tagged with 10-STAR axis labels:** — G10 Task G10.4
4. **Reproducible proof run in `docs/verification/v10-proof-runs/`:** — G10 Task G10.5
   - Harness NDJSON per axis with dated human review
   - Scorecard regenerator output
   - Negative controls (fault-injection) pass

If any axis regresses on G10 regeneration, V10 does not close.

## Token Efficiency (TE) Contingency Plan

TE is the tightest-margin axis (5/10 final, floor 3/10 = +2 margin). Per 0.1.0-AXIS-OWNERSHIP.md, V8 owns TE +1 (4→5). If V8 underdelivered or V10 integration exposes TE weakness:

**Contingency phases (post-G10, blocks V10 close if invoked):**
- **TE-recovery-v10a** — profiling pass: identify token sinks in wake, compiler, and recall paths; retarget budget allocation.
- **TE-recovery-v10b** — if contingency needed, measure impact on TE and composite before re-tag.
- **Decision rule:** If G10 regeneration shows TE < 4, run TE-recovery phases and re-run G10 harness before 0.1.0 tags.

Currently no evidence of TE regression; contingency documented as precaution.

## Per-axis harness assertions (required for axis credit)

| axis | concrete assertion | fixture |
| --- | --- | --- |
| session_continuity   | T22 missed-correction re-ingested + surface in next session without user prompt | G10 scenario T20-T23 |
| correction_retention | T5 correction issued in session 1, auto-applied to session 2 without re-ingestion | G10 scenario T28-T30 |
| procedural_reuse     | routine detected, invoked in T31, measured accuracy ≥0.8 over 30-day window | C10 routine-metric.ndjson |
| raw_retrieval        | query-answer quality feedback loop: weight δ ≤ 0.05 per 30-day cycle, index updates logged | D10 retrieval-feedback.ndjson |

Missing any assertion → axis does not lift and milestone does not close.

## Non-goals

- Session continuity lift beyond 7 (V10 owns SC +1; V7/V8 already landed +1 each)
- Correction retention lift beyond 6 (V10 owns CR +1; no further lift claimed)
- Procedural reuse detection better than 85% precision (tuning phase, not data science)
- Token efficiency improvement in V10 (TE floor closed at 5 in V8; V10 integrates, no lift)
- Cross-harness or trust_provenance axis lifts (integrated only; no credit)

## Completion gate

G10 multi-harness proof run + axis assertions passing + scorecard regeneration to 6.40 + zero blocker backlog:

- Composite ≥ 6.40 on G10 regeneration (not 9.0; that was aspirational; 6.40 is contract binding)
- Every axis ≥ 3: verified
- Reproducible proof-run directory populated
- Negative controls fire as designed
- Production-floor checklist items 1–4 complete

## Changelog

- 2026-04-22 opened.
- 2026-04-22 revised: composite_pre 9.0 → 5.60, composite_target 9.5 → 6.40 (binding from 0.1.0-CONTRACT.md); axes_lifted list made explicit (SC, CR, PR, RR only); axes_integrated_with added (CH, TE, TP); original 0.1.0 release-gate checklist added; TE contingency documented; per-axis assertions table added; non-goals clarified.
- 2026-05-05 closed: V10 production-floor gate passed at composite 6.40; stale release-gate language corrected. V13 remains the `0.1.0` release owner.

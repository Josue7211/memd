---
milestone: v18
name: Correction Graph + Silent Detection
status: planned
opened: 2026-04-22
depends_on: [v17, ../../verification/1.0.0-CONTRACT.md, ../../verification/1.0.0-AXIS-OWNERSHIP.md]
composite_pre: 9.35
composite_target: 9.50
axes_lifted: [correction_retention]
axes_integrated_with: [trust_provenance]
---

# Milestone v18 Audit — Correction Graph + Silent Detection

## Goal

Correction graph across all memory with full multi-hop chains
(correction A about B downstream-affects C). Silent correction
detection reaches ≥0.90 precision / ≥0.85 recall (up from V11's
≥0.70 / ≥0.60). Third-party replay from export reproduces correction
applications deterministically. Ships CR 8→9.

## 10-STAR axis targets (pre / post)

| axis | weight | pre | post | basis |
| --- | --- | --- | --- | --- |
| session_continuity   | 20% | 10 | 10 | — |
| correction_retention | 15% | 8  | 9  | **OWNS +1** — multi-hop graph + high-precision silent detection |
| procedural_reuse     | 15% | 10 | 10 | — |
| cross_harness        | 15% | 10 | 10 | — |
| raw_retrieval        | 15% | 9  | 9  | — |
| token_efficiency     | 10% | 9  | 9  | — |
| trust_provenance     | 10% | 9  | 9  | INT (graph audit; V19 owns crypto lift) |

**Composite: 9.35 → 9.50**.

## Phases (planned)

- **A18** Correction graph data structure (edges: cites/supersedes/affects)
- **B18** Multi-hop propagation engine (correction X affects Y → query Y returns X-aware value)
- **C18** Silent correction detector v2 (LLM-judged + heuristic ensemble; ≥0.90 precision)
- **D18** Downstream-effect surfacing (user sees "this answer was affected by corrections A, B, C")
- **E18** Correction-graph export format (deterministic replay input)
- **F18** Third-party replay harness (external tool takes export + query → same answer)
- **G18** V18 gate harness (≥3-month dogfood; ≥50 multi-hop chains traced; silent detection metrics)

## Completion gate

1. ≥3-month dogfood with ≥50 multi-hop correction chains traced end-to-end.
2. Silent correction detector ≥0.90 precision / ≥0.85 recall on labeled corpus.
3. Downstream-effect surface shows correction chain for ≥95% of affected queries.
4. Third-party replay harness reproduces corrections deterministically on ≥10 exports.
5. 10-STAR composite regenerated ≥9.50 with CR=9.

## Non-goals

- Cryptographic signatures on correction chain (V19 owns TP 9→10 + CR 9→10)
- Auto-resolution of conflicting corrections (human-in-the-loop stays)

## Changelog

- 2026-04-22 opened.

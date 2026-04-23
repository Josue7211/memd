---
milestone: v6
name: Typed Ingest for Public Benches
status: planned
opened: 2026-04-22
revised: 2026-04-22
depends_on: [v5]
composite_pre: 4.20
composite_target: 4.45
axes_lifted: [raw_retrieval, trust_provenance]
axes_integrated_with: [token_efficiency]
---

# Milestone v6 Audit — Typed Ingest for Public Benches

## Goal

Stop pretending public benches are flat-RAG. Apply memd's typing — episodic/semantic/canonical/candidate — to public-bench input. Distill turns into semantic facts, promote repeated high-confidence facts to canonical, compile a working-context window instead of dumping top-k chunks, route re-queries through progressive-depth. Public-bench ingest gains RR +1 lift; provenance trails survive query→answer loop (TP +1 lift). D4 compiler applied to bench inputs integrates token_efficiency (D4 remains owner); no TE-axis credit to V6.

## 10-STAR axis targets (pre / post)

Baseline from V5 post (0.1.0-CONTRACT.md):

| axis | weight | pre | post | basis |
| --- | --- | --- | --- | --- |
| session_continuity | 20% | 4 | 4 | no V6 work — maintained from V4 |
| correction_retention | 15% | 4 | 4 | no V6 work — maintained from V4 |
| procedural_reuse | 15% | 4 | 4 | no V6 work — maintained from V5 |
| cross_harness | 15% | 4 | 4 | no V6 work — maintained from V5 |
| raw_retrieval | 15% | 6 | 7 | A6–F6 typed-ingest pipeline on LME/LoCoMo/MemBench/ConvoMem: parity within ±2% of substrate baseline |
| token_efficiency | 10% | 4 | 4 | D6 integrates D4 compiler on bench inputs; no TE lift claimed (D4 owns TE 2→4, V6 integrates only) |
| trust_provenance | 10% | 3 | 4 | each public-bench answer carries queryable back-pointer to source turn(s) via explain API |

**Composite: 4.20 → 4.45** (weighted arithmetic).

## Phases

- **A6** Episodic Ingest Pipeline — bench turns ingested as episodic, not raw chunks.
- **B6** Semantic Distillation — episodic → semantic facts via LLM extractor.
- **C6** Canonical Promotion — repeated high-confidence facts promoted to canonical lane.
- **D6** Working-Context Compiler on Bench Input — prompt assembled from typed retrieval, not top-k dump; **integrates D4 compiler, no TE-axis credit**.
- **E6** Progressive-Depth Routing — model can re-query memd mid-answer; bench harness supports this.
- **F6** Iterative Reasoning Harness + Scorecard Regeneration — multi-step reasoning over typed memory; G6 harness runs canonical sweep and regenerates 10-STAR + PUBLIC_BENCHMARKS.md.

## Completion gate

### Raw Retrieval (+1: 6→7) — public-bench parity assertion

Canonical intrinsic (sidecar OFF) measured against V5 substrate baseline (same test harness config):

| Bench | V5 Substrate | V6 Target | Parity Tolerance | RR-lift attribution |
| --- | --- | --- | --- | --- |
| LME `qa_accuracy` | ≥0.83 | ≥0.85 | ±2% | A6–F6 typed pipeline |
| LoCoMo `token_f1_avg` | ≥0.73 | ≥0.75 | ±2% | A6–F6 typed pipeline |
| MemBench `mc_accuracy` | ≥0.73 | ≥0.75 | ±2% | A6–F6 typed pipeline |
| ConvoMem LLM-judge `accuracy` | ≥0.88 | ≥0.90 | ±2% | A6–F6 typed pipeline |

No regression on retrieval diagnostics (`session_recall_any@5` stays ≥0.95 on LME).

All four numbers carry method cards per I3 rules with provenance audit trail.

### Trust Provenance (+1: 3→4) — queryable back-pointer assertion

Each public-bench answer generated from a retrieved context passage must:
- Link back to original source turn(s) via `memory_item_id`
- Be queryable via `memd explain <turn_id>` to reconstruct reasoning chain
- Pass drilldown test: user asks "where did you get that fact?"; agent surfaces the turn(s) and their correction status (if any)

Concrete fixture: G6 scenario exercises explain API for ≥3 multi-hop reasoning chains; each turn produces back-pointer, each pointer resolves without error.

## Non-goals

- exceeding published SOTA by benchmaxxing — honest canonical run only
- token_efficiency lift (D6 integrates D4 compiler; no TE-axis credit claimed by V6)
- session_continuity, correction_retention, procedural_reuse, cross_harness lifts (owned by V4/V5, maintained at baseline)
- touching public-bench scoring logic — we run upstream scorers, adapt ingest only

## Per-axis harness assertions (required for axis credit)

| axis | concrete assertion | fixture |
| --- | --- | --- |
| raw_retrieval | LME `qa_accuracy` ≥0.85, LoCoMo `token_f1_avg` ≥0.75, MemBench `mc_accuracy` ≥0.75, ConvoMem `accuracy` ≥0.90 (all ±2% vs V5 substrate baseline) | G6 canonical bench suite |
| trust_provenance | every bench answer queryable via `memd explain <turn_id>`; ≥3 multi-hop chains resolve without error | G6 explain drilldown scenario |

Missing any assertion → axis does not lift, milestone does not close.

D4 owns token_efficiency (2→4); V6 integrates D4 compiler on bench inputs with zero TE-axis credit (enforced by strict-mode scorecard regenerator).

## Changelog

- 2026-04-22 opened.
- 2026-04-22 revised: composite_pre 5.5 → 4.20 (V5-post baseline), composite_target 7.0 → 4.45 (contract target); axes_lifted narrowed to [raw_retrieval, trust_provenance] per 0.1.0-AXIS-OWNERSHIP; axes_integrated_with [token_efficiency] added; axis targets clarified (no SC/CR/PR/CH lifts); non-goals expanded; per-axis harness assertions table added to enforce "no axis credit without G6 harness proof" rule; public-bench parity table added to show RR-lift scope; provenance queryability assertion added for TP lift.

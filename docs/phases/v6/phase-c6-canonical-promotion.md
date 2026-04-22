---
phase: C6
name: Canonical Promotion
version: v6
status: planned
opened: 2026-04-22
depends_on: [B6]
axis: raw_retrieval, trust_provenance
plan_spec: docs/phases/v6/phase-c6-plan.md
---

# Phase C6: Canonical Promotion

## Goal

Promote semantic candidates (B6 output) to canonical when they are repeated, high-confidence, and corroborated. Canonical records are the trust surface: one copy per durable truth, deduped across the whole bench corpus, pinned high in retrieval.

## Why this phase exists

B6 produces many candidates per session — some transient, some genuinely durable. A retriever that treats them flat loses the signal. C6's promotion rule ("cited twice by distinct source turns, confidence ≥ 0.8, no contradicting correction") crystallizes the durable ones and gives the retriever a short, clean canonical lane.

## Deliver

1. **Promotion rule engine.** `crates/memd-client/src/benchmark/typed_ingest/promotion.rs` — walks candidate store, emits promotions when rule matches.
2. **Rule card.** `docs/contracts/canonical-promotion.md` — exact thresholds: corroboration count ≥ 2, confidence ≥ 0.8, no contradiction within window, min session-age.
3. **Contradiction check.** Before promotion, B5 correction propagation scorer reused to ensure no conflicting canonical exists — if conflict, the newer wins per V4 C4 rules.
4. **Canonical lane index.** Promoted records indexed separately; retriever (D6) can query canonical-only or canonical+semantic+episodic.
5. **Dry-run mode.** `--promotion-dry-run` emits what-would-promote NDJSON for audit without writing.
6. **Baseline lift test.** Canonical-on bench run vs B6 baseline. Must lift LME `qa_accuracy` by ≥ 0.02 additional and MemBench `mc_accuracy` by ≥ 0.03 (MemBench prizes canonical recall).

## Pass Gate

- pre: flat semantic candidates only
- post: promotion engine runs, canonical lane indexed, rule card committed; cumulative V6 lift at C6: LME ≥ +0.04, MemBench ≥ +0.03 vs A6 baseline
- evidence: promotion NDJSON, dry-run samples, delta report
- regression budget: LoCoMo ≤ 1% drop tolerated if ConvoMem lifts ≥ 0.03

## Product Win

"canonical memory" becomes a numbered contributor, not a whiteboard noun.

## Evidence

- rule card
- promotion NDJSON
- delta report

## Fail Conditions

- Lift <+0.04 LME: promotion thresholds too tight/loose; adjust via ablation, not scorer.
- Any canonical record lacking provenance: hard fail (E5 auditor reuse must pass).

## Non-Goals

- Retrieval path changes beyond lane index (D6 scope).
- Cross-bench promotion (within-bench only for V6).

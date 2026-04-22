---
phase: E5
name: ProvenanceIntegrity Bench
version: v5
status: planned
opened: 2026-04-22
depends_on: [V4]
axis: trust_provenance
---

# Phase E5: ProvenanceIntegrity Bench

## Goal

Every retrieved record must carry a full provenance chain: source turn, capture source (manual/detector/judge), capture timestamp, and (if corrected) corrects_id. Unsourced record in result set = fail, no exceptions.

## Why this phase exists

Provenance is memd's trust claim. Today nothing audits it. A silent regression that drops provenance on a retrieval path would land undetected.

## Deliver

1. **Scenario generator.** 200 synthetic queries over a 500-record corpus with full provenance.
2. **Auditor.** For each result set, assert every record has all required provenance fields.
3. **Metrics.** `provenance-completeness-rate`, `provenance-chain-length-mean`, `unsourced-record-count` (hard floor 0).
4. **Runner.** `memd bench substrate --suite provenance-integrity`.
5. **Inject test.** Drop provenance on one record path — scorer must catch it.

## Pass Gate

- pre: no audit
- post: completeness-rate = 1.000 (hard); mean chain length ≥ 2 (capture + retrieval + anything else)
- evidence: NDJSON results + inject test demonstrating scorer catches a planted hole
- regression budget: any unsourced record blocks merge

## Product Win

"memd can explain why it returned this" becomes enforceable.

## Evidence

- 200-query audit output
- inject test passing (scorer catches planted hole)
- report

## Fail Conditions

- Completeness < 1.0: retrieval path strips provenance; fix path, not test.

## Rollback

Bench-only.

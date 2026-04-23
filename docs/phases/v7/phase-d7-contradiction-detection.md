---
phase: D7
name: Contradiction Detection
version: v7
status: planned
opened: 2026-04-22
depends_on: [B7]
axis: correction_retention, trust_provenance
plan_spec: docs/phases/v7/phase-d7-plan.md
---

# Phase D7: Contradiction Detection

## Goal

Surface contradictions when a new correction conflicts with an existing one. Two corrections within a window can't both be right — memd flags the conflict and asks the user to disambiguate instead of silently merging.

## Why this phase exists

B7 promotion assumes clean "new corrects old". Real users correct themselves, get interrupted, revise. D7 detects same-target corrections within a decision window (default 24h) and raises a receipt instead of overwriting.

## Deliver

1. **Detector.** `memd-core::correction::contradiction_detector` — compares new correction vs existing corrections on same `prior_claim_id` within window; emits `ContradictionReceipt`.
2. **Receipt schema + store.** `stage: contradicted_pending`; neither wins until user resolves.
3. **Resolution CLI.** `memd correction resolve <receipt-id> --accept <correction-id> [--reason "..."]`.
4. **Default behavior.** Retrieval returns the older accepted canonical until resolved; surfaces "contradicted, pending resolution" flag.

## Pass Gate

- pre: last-write-wins silently
- post: detector fires on planted conflicts; resolve CLI works; 10 planted conflicts resolved cleanly in tests
- evidence: contradiction NDJSON, resolution CLI tests

## Product Win

memd never silently overwrites a corrected truth. Users get a say.

## Evidence

- detector tests
- CLI tests
- 10 planted-conflict scenarios

## Fail Conditions

- Any silent merge: hard fail.
- False-positive rate > 5% on non-conflicting corrections: window too wide or similarity threshold wrong.

## Non-Goals

- UI for resolution (V8 E8)
- cross-user contradictions (V9)

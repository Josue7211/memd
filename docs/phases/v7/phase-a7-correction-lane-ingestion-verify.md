---
phase: A7
name: Correction Lane Ingestion Verify
version: v7
status: planned
opened: 2026-04-22
depends_on: [V6]
axis: correction_retention
plan_spec: docs/phases/v7/phase-a7-plan.md
---

# Phase A7: Correction Lane Ingestion Verify

## Goal

Prove the correction lane captured in V4 C4 is actually working in live sessions at V7 baseline. Walk the full path: user types correction → hook captures → judge confirms → record lands with `kind=Correction` + provenance → downstream retrieval sees it. No simulated fixtures — real harness traces.

## Why this phase exists

C4 landed the mechanics and proved them in tests. V7 is the end-to-end milestone — before promotion (B7), behavior-change (C7), contradiction (D7), rollback (G7) make sense, we verify the ingest is truth-preserving in the real harness loop.

## Deliver

1. **Live-trace verifier.** Walks 30 days of `.memd/logs/corrections.ndjson` and confirms each correction has: kind, source_turn, captured_by, judge_verdict, prior_claim_id.
2. **Harness trace reconciliation.** For each correction, find its originating turn in hook trace; assert one-to-one mapping.
3. **Miss-rate report.** Corrections that user uttered but memd did not capture (signal: user repeats same correction later). Report at `docs/verification/v7-runs/correction-miss-rate-YYYY-MM-DD.md`.
4. **Ingest health dashboard entry.** Add correction-ingest health to the existing healthz surface.

## Pass Gate

- pre: correction capture exists, not verified at scale
- post: 30-day trace verified; miss-rate ≤ 5%; healthz exposes correction-ingest state; all captured corrections have full provenance
- evidence: miss-rate report, verifier NDJSON, healthz snapshot

## Product Win

Claim: "memd captures your corrections" becomes a number backed by a trace.

## Evidence

- miss-rate report
- verifier NDJSON
- healthz correction-ingest field

## Fail Conditions

- Miss-rate >5%: diagnose hook or judge — do not silently drop.
- Any correction lacking provenance: hard fail; V4 C4 regression.

## Non-Goals

- promotion rules (B7)
- behavior change (C7)

---
contract: missed-correction-reingestion
status: active
opened: 2026-05-05
milestone: V10
phase: A10
---

# Missed-Correction Reingestion

V10 A10 detects when an agent makes a claim and a later user turn corrects it.
The detector emits a `MissedCorrection` with:

- `claim_id`: agent claim superseded by the user correction.
- `correction_turn_id`: user turn that supplies fresher truth.
- `confidence`: deterministic overlap score.
- `reingest=true`: eligible for the next correction write path.

Reingest candidates must be stored with `correction`, `missed-correction`, and
`v10-a10` tags. They supersede the stale claim and must keep the user correction
turn as provenance.

This is a production-floor V10 contract. It does not tag `0.1.0`; that release
gate remains V13 per `docs/verification/0.1.0-CONTRACT.md`.

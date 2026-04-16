---
phase: O2
name: Lifecycle Calibration
version: v2
status: pending
depends_on: [M2]
backlog_items: []
---

# Phase O2: Lifecycle Calibration

## Goal

Decay and consolidation parameters data-driven and justified. Not hardcoded guesses.

## Deliver

- Decay constants configurable via policy (not hardcoded 21d/0.12)
- Decay metric collection from production usage
- Decay sensitivity analysis framework
- Consolidation quality scoring (semantic coherence, information preservation)
- Post-consolidation recall comparison (A/B)

## Pass Gate

- Decay constants configurable via MemoryPolicyDecay, wired into decay_entities()
- Decay sensitivity: metrics show impact of threshold changes on retention and recall
- Consolidation quality: semantic coherence test passes (consolidated item preserves original meaning)
- Consolidation recall: post-consolidation retrieval quality >= pre-consolidation
- Calibrated defaults documented with data justification

## Evidence

- Decay metric reports (age distribution, confidence curves, items retained/expired)
- Consolidation quality test results (coherence scores, information preservation %)
- Sensitivity analysis comparison table
- Parameter justification document

## Fail Conditions

- Consolidation produces incoherent summaries (key facts lost)
- Decay parameters chosen without supporting data
- Recall drops after consolidation
- Constants remain hardcoded after phase completion

## Rollback

- Revert to hardcoded constants if calibrated values cause regression

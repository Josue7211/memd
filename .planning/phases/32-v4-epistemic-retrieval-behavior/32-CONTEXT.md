# Phase 32 Context: `v4` Epistemic Retrieval Behavior

## Why This Phase Exists

`memd` should not only remember more; it should also be wrong less often.
Retrieval and ranking need to account for epistemic state so verified evidence
beats merely inferred or synthetic continuity.

## Inputs

- existing retrieval and context ranking paths
- source quality, confidence, status, and verification timestamps on memory
  items
- requirement that verified evidence outrank narrative continuity

## Constraints

- keep ranking deterministic and explainable
- avoid hiding contested or stale memory completely
- make epistemic state influence both search and context retrieval

## Target Outcome

Retrieval explicitly prefers recently verified and canonical evidence while
penalizing unverified and synthetic memory.

# Phase 37 Context: `v5` Coordination Policy and Ownership Guards

## Why This Phase Exists

`memd` now supports peer coordination, shared tasks, coordination inboxes, and
stale-session recovery. The next gap is policy: sessions still need clearer
rules for when work should be exclusive, shared, review-only, or help-only.

## Inputs

- brokered claims and recovery
- shared task orchestration
- coordination inbox and pressure views

## Constraints

- preserve explicit ownership instead of implicit merging
- keep the first policy slice deterministic and visible
- avoid blocking normal coworking with heavy workflow bureaucracy

## Target Outcome

The next phase should add coordination policy and ownership guards so sessions
can distinguish exclusive-write work from shared review/help work automatically.

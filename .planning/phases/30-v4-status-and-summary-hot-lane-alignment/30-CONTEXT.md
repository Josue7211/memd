# Phase 30 Context: `v4` Status and Summary Hot-Lane Alignment

## Why This Phase Exists

Quick inspection surfaces such as `status` and `resume --summary` need to carry
the same task-state signal as the real hot lane. If they only show counts,
operators still have to drop into deeper views for basic awareness.

## Inputs

- `status` output
- `resume --summary`
- existing current-task snapshot and change-summary work

## Constraints

- keep lightweight inspection lightweight
- align with the actual hot lane
- avoid duplicating deep prompt output

## Target Outcome

Quick inspection surfaces become useful on their own by carrying focus,
pressure, and delta signal.

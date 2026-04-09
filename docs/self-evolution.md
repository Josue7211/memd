# Self-Evolution Pipeline

`memd` treats self-evolution as a controlled pipeline, not an ad hoc branch workflow.

For day-to-day operator inspection, use:

- `memd status --summary`
- `memd loops --loop self-evolution`
- `memd autoresearch --loop-slug self-evolution`

`memd status --summary` now surfaces the self-evolution control-plane fields,
so you can check loop and gate state from the readiness surface before drilling
into the loop or autoresearch views.

## Flow

1. a candidate is detected from loop pressure, a gap, or an explicit policy change
2. a proposal artifact is written with the candidate's scope, baseline, evaluation plan, and rollback plan
3. a branch manifest is attached to the evolution branch so the allowed write surface stays explicit
4. an authority ledger records which change classes may move through the low-risk lane
5. the candidate enters the merge queue if it satisfies the current gates
6. low-risk candidates may use the low-risk auto-merge lane when the manifest and authority tier allow it
7. merged changes enter the durability queue for a later re-check
8. the durability queue promotes the change to durable truth or demotes it

## Artifacts

- proposal artifact
- branch manifest
- authority ledger
- merge queue entry
- durability queue entry

## States

Self-evolution states are distinct and should not be collapsed:

- `accepted_proposal`
- `merged`
- `durable_truth`
- `demoted`

Meanings:

- `accepted_proposal` means the candidate won in isolation and is eligible to move forward
- `merged` means the change landed through the approved lane
- `durable_truth` means a later re-check still validates the win
- `demoted` means durability failed after merge

## Lane Policy

- low-risk changes may auto-merge only when the branch manifest matches the allowed surface and the authority ledger permits the class
- broader storage, coordination, and API changes stay review-gated
- accepted proposals are not durable truth until the durability queue re-check passes
- the system may auto-promote runtime policy, but code changes still need explicit branch and lane control

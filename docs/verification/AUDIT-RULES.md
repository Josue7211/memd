# Audit Rules

## Core Rule

Runtime behavior beats planning status.

## Feature Verdicts

- `verified`: all required checks pass
- `partial`: core contract exists but one or more required checks fail or are missing
- `broken`: user contract fails in a material way
- `unverified`: not yet audited
- `auditing`: currently under active audit

## Milestone Verdicts

- `verified`: all claimed features are verified
- `regressed`: one or more previously verified features are now partial or broken
- `unverified`: not yet audited
- `auditing`: currently under active audit

## Exhaustive Verification Standard

A feature is exhaustive only if it has:

- implementation trace
- direct test proof
- workflow proof
- adversarial proof
- rerun command
- cross-harness proof when relevant

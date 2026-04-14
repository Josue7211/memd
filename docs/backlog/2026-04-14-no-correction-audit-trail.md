# No Correction Audit Trail

- status: `open`
- severity: `high`
- phase: `V2-D2`
- opened: `2026-04-14`
- scope: memd-core

## Problem

Supersede mechanics allow overriding memory items but don't log who changed what or why. Trust chain is opaque. Impossible to audit corrections or revert malicious changes.

## Fix

- Add immutable correction log (who, what, when, why)
- Implement signed corrections for trust
- Add reversal capability
- Add to phase-D2 acceptance criteria (trust audit)

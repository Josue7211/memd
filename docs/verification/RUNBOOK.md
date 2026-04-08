# Verification Runbook

## Backward Audit Flow

1. Pick milestone.
2. Mark milestone `auditing`.
3. Enumerate claimed features from the feature registry.
4. Run each feature's rerun commands.
5. Record findings.
6. Mark each feature `verified`, `partial`, or `broken`.
7. Mark milestone `verified` only if all claimed features are verified.

## Post-Change Regression Flow

1. Identify touched features.
2. Run feature rerun commands.
3. Re-run milestone audits affected by those features.
4. Mark regressions immediately in milestone files.

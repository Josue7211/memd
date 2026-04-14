---
phase: D2
name: Correction Flow
version: v2
status: pending
depends_on: [B2, C2]
backlog_items: [43, 57, 58, 78, 80]
---

# Phase D2: Correction Flow

## Goal

User says "wrong" → stale belief replaced → future recall reflects correction. Proven E2E.

## Deliver

- `memd correct --id <uuid> --content "corrected"` CLI command
- Correction audit trail (who, when, why, what replaced)
- Selective memory reset (single item, not nuclear)
- Contradiction detection trigger (two items claim opposite → contested)

## Pass Gate

- E2E test: store fact A → correct to fact B → future recall returns B, not A
- Correction audit trail queryable: `memd explain <id>` shows correction history
- Contradiction scenario: store "X is true" + "X is false" → both marked contested
- Selective reset: corrupt one item → reset it → other items untouched

## Evidence

- Correction E2E test (automated, in CI)
- Contradiction detection test
- Selective reset test
- Audit trail query output

## Fail Conditions

- Corrected fact still returned in recall
- Correction loses provenance
- Selective reset affects other items

## Rollback

- Revert correction mechanics if they break existing recall

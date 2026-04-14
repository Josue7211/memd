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

## Donor Extraction (from inspiration repos)

- **D2-D1** (Omegon `sqlite.rs` — **DIRECT RUST LIFT**): Supersede with Lamport versioning. Archive original, insert replacement with incremented `version: u64`. On concurrent corrections, higher version wins. Deterministic conflict resolution without timestamps.
- **D2-D2** (mempalace `knowledge_graph.py`): Temporal fact invalidation via `valid_from`/`valid_to` on triples. Old fact gets `valid_to=now()`, new gets `valid_from=now()`. Enables "what did we believe at time T?" queries.
- **D2-D3** (mempalace WAL): Write-ahead log for correction audit. Every correction logged before execution: `{timestamp, operation, old_content, new_content, reason}`.
- **D2-D4** (Smriti `CommitModel`): Immutable checkpoints with additive-only annotations. Never UPDATE content in place. Corrections create new items, mark old Superseded.

See: `docs/theory/2026-04-14-donor-extraction-to-v2-phases.md` for full details.

## Rollback

- Revert correction mechanics if they break existing recall

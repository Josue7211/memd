---
phase: D2
name: Correction Flow
version: v2
status: reopened
depends_on: [B2, C2]
backlog_items: [43, 57, 58, 78, 80]
reopened_at: 2026-04-15
reopened_reason: Correction CLI exists but corrections don't change future recall in production. Trust hierarchy not enforced. Contradiction detection not wired. Preferences lost every session because correction→recall pipeline never runs end-to-end.
---

# Phase D2: Correction Flow

Current status: `reopened` — correction mechanics exist in code but the pipeline doesn't execute in production. User corrections don't persist across sessions. Trust hierarchy (human > canonical > candidate) not enforced.

## Reopened Scope

- **Corrections change future recall**: corrected fact must replace stale belief in next wake
- **Trust hierarchy enforced**: human correction outranks all, canonical outranks candidate
- **Contradiction detection wired**: two items claiming opposite → both marked contested
- **Correction audit trail queryable**: `memd explain <id>` shows full correction history
- **Cross-session persistence**: correction made in session N visible in session N+1

## Node Verification (from [[docs/verification/NODE-VERIFICATION-MATRIX.md]])

This phase owns M2-tier verification for:
- P4 (correction, provenance, authority): corrections change future recall, trust hierarchy enforced

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

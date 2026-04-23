---
phase: E7
name: Provenance Trail on Corrected Records
version: v7
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [B7]
phase_doc: docs/phases/v7/phase-e7-provenance-trail.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: trust_provenance
---

# Phase E7 — Implementation Plan

## 0. Executive summary

`correction_chain: Vec<ChainLink>` on corrected canonicals. `memd fact provenance --chain` emits lineage. Audit tool fails on broken link. Extends V5 E5 completeness rule.

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `crates/memd-core/src/correction/chain.rs` | ChainLink + walker. |
| `crates/memd-client/src/commands/fact_provenance.rs` | CLI. |
| `crates/memd-core/src/correction/chain_audit.rs` | Audit-all tool. |
| `crates/memd-core/src/main_tests/chain_tests/mod.rs` | Tests. |

### Files to modify

| Path | Change |
| --- | --- |
| `correction/promotion.rs` (B7) | Append chain link on promote. |
| `memd-schema/src/lib.rs` | `MemoryRecord.correction_chain` field. |
| Schema migration. |
| V5 E5 auditor (`provenance_auditor.rs`) | Extend to chain integrity. |
| Phase doc. |

## 2. Schema changes

```sql
ALTER TABLE memory_records ADD COLUMN correction_chain JSONB NOT NULL DEFAULT '[]'::jsonb;
```

ChainLink:

```json
{
  "prior_canonical_id": "…",
  "correction_record_id": "…",
  "turn_id": "…",
  "judge_confidence": 0.0,
  "promoted_at": "…"
}
```

## 3. API shape

```
memd fact provenance <id>                # short form
memd fact provenance <id> --chain        # full lineage
memd fact provenance --audit-all         # walks every canonical
```

## 4. Test matrix

1. `chain_link_appended_on_promote`
2. `chain_preserved_across_multiple_corrections`
3. `walker_yields_links_oldest_first`
4. `audit_all_passes_on_clean_corpus`
5. `audit_all_fails_on_broken_link`
6. `cli_fact_provenance_short_happy`
7. `cli_fact_provenance_chain_full`
8. `cli_audit_all_happy`
9. `e5_auditor_reuse_chain_completeness`
10. `large_chain_streams_without_truncation`

## 5. Fixtures

- `tests/fixtures/correction/e7/chain-3-deep.jsonl` — A→B→C→D canonical history.
- `tests/fixtures/correction/e7/broken-link-corpus.jsonl` — deliberately orphaned link for fail test.

## 6. Telemetry

Audit output at `docs/verification/v7-runs/chain-audit-<date>.json`.

## 7. Feature flags

None.

## 8. Task list

### Task E7.1 — schema + chain append

- [ ] Migration + field; tests 1 + 2 failing.
- [ ] Commit: `feat(schema/e7): correction_chain field (E7)`.

### Task E7.2 — walker

- [ ] Test 3 failing.
- [ ] Commit: `feat(correction/e7): chain walker (E7)`.

### Task E7.3 — audit tool + E5 reuse

- [ ] Tests 4 + 5 + 9 failing.
- [ ] Extend E5 auditor.
- [ ] Commit: `feat(correction/e7): audit + E5 reuse (E7)`.

### Task E7.4 — CLI

- [ ] Tests 6 + 7 + 8 failing.
- [ ] Commit: `feat(cli/e7): fact provenance (E7)`.

### Task E7.5 — streaming large chains

- [ ] Test 10 failing; implement stream output.
- [ ] Commit: `feat(e7): chain streaming (E7)`.

### Task E7.6 — CI

- [ ] Nightly audit-all in CI; failure blocks.
- [ ] Commit: `ci(e7): audit-all nightly (E7)`.

## 9. Bench impact

E5 ProvenanceIntegrity suite extends to chain — any broken link hard-fails gate.

## 10. Dependency graph

- Requires: B7, V5 E5.
- Blocks: F7, G7, V8 D8 (provenance browser consumes chain).

## Exit criteria

1. Tests 1–10 green.
2. Audit-all passes on main.
3. E5 auditor green with chain extension.
4. CLI streams arbitrarily long chains.
5. Atomic commits.

---
contract: project-isolation
status: active
opened: 2026-05-05
milestone: v11
owners: [A11, B11, C11, D11, G11]
---

# Project Isolation Contract

V11 wake and compiler reads are scoped by the pair
`(project_id, workspace_id)`.

## Invariants

- Same `project_id` + same `workspace_id` -> visible.
- Same `workspace_id` + different `project_id` -> hidden.
- Same `project_id` + different `workspace_id` -> hidden.
- Project filters apply before compaction recovery, silent-correction
  detection, dynamic compiler selection, and cost-ledger attribution.
- G11 negative controls must fail if project isolation is suppressed.

## Schema

- `memory_items.project_id`
- `memory_items.workspace_id`
- `compiler_context`
- `cost_ledger`
- `correction_flags`

The SQLite migration exposes `project_id` and `workspace_id` as generated
columns over existing stored payloads so old bundles are readable while V11
read paths can use stable column names.

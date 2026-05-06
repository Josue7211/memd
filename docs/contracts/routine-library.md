---
contract: routine-library
version: 0.1
owner_phase: A12
status: draft
opened: 2026-05-05
depends_on: [docs/phases/v12/V12-INTEGRATION.md, docs/phases/v12/phase-a12-routine-library-ui.md]
---

# Routine Library Contract

This contract defines the curated routine-library surface for V12. It is the
bridge from existing procedure detection/invocation into user-curated routines
that can later be composed, inherited per project, exported across workspaces,
and proven in G12.

## 1. Scope

A12 owns:

- Browse routines.
- Edit routine name, summary, steps, and status.
- Merge duplicate routines.
- Deprecate routines with a reason.
- Keep existing `memd procedure` behavior compatible.

A12 does not own:

- Routine composition (`A+B=C`), owned by B12.
- Project inheritance, owned by C12.
- Cross-workspace export/import, owned by D12.
- Signed audit entries, owned by H12.

## 2. Routine Record

Every routine exposed by `memd routines` must map to one logical record:

| Field | Type | Required | Rule |
| --- | --- | --- | --- |
| `id` | UUID string | yes | Stable id for the current routine revision. |
| `name` | string | yes | Unique within one library scope after trim/case-fold. |
| `summary` | string | yes | One-line purpose for browse and match output. |
| `steps` | array of strings | yes | Ordered executable or advisory steps. Empty is invalid for `active`. |
| `status` | enum | yes | `candidate`, `active`, `deprecated`, or `merged`. |
| `source_ids` | array of UUID strings | no | Input ids for a merged routine. |
| `replaces_id` | UUID string | no | Prior revision replaced by an edit. |
| `deprecation_reason` | string | no | Required when status is `deprecated`. |
| `updated_by` | string | no | Harness/agent/user label. H12 signs this later. |
| `project_id` | string | no | V11 project boundary when known. |
| `workspace_id` | string | no | V11 workspace boundary when known. |
| `created_at` | RFC3339 string | yes | Creation time. |
| `updated_at` | RFC3339 string | yes | Last mutation time. |

Unknown fields must be preserved by import/export code after D12 lands.

## 3. Status Rules

`candidate`:

- Created by detection or manual seed.
- Visible in `browse`.
- Not invocable by default unless explicitly selected.

`active`:

- User-curated and invocable.
- Must have non-empty `steps`.
- Default status for edited or merged output.

`deprecated`:

- Hidden by default from browse/match.
- Visible with `--status deprecated`, `--status all`, or
  `--include-deprecated`.
- Must carry `deprecation_reason`.

`merged`:

- Hidden by default from browse/match.
- Points forward through merge output provenance.
- Keeps old id addressable for explanation and audit.

## 4. Commands

Browse:

```bash
memd routines browse [--workspace <id>] [--status candidate|active|deprecated|merged|all] [--json]
```

Edit:

```bash
memd routines edit <id> --name <name> [--summary <text>] [--steps-file <path>] [--status candidate|active] [--json]
```

Merge:

```bash
memd routines merge <id> <id>... --name <name> [--summary <text>] [--json]
```

Deprecate:

```bash
memd routines deprecate <id> --reason <text> [--json]
```

Command output:

- Human output starts with `# Routines` for browse.
- JSON output has top-level `routines` for browse and `routine` for mutation.
- Mutation output includes `status`, `id`, `name`, and affected input ids.

## 5. Mutation Semantics

Edit:

1. Validate name.
2. Validate steps if status is `active`.
3. Create a new revision when persisted storage supports revisions.
4. Set `replaces_id` to prior id.
5. Preserve old record for provenance.

Merge:

1. Require at least two input ids.
2. Reject duplicate input ids.
3. Reject deprecated inputs unless `--allow-deprecated` is later added by a
   future phase.
4. Create a new `active` routine with `source_ids`.
5. Mark inputs `merged`.

Deprecate:

1. Require non-empty `--reason`.
2. Mark status `deprecated`.
3. Preserve record for audit and explanation.

## 6. Feature Flag

Mutating commands are gated until A12 graduation:

```text
MEMD_A12_ROUTINE_LIB_UI=1
```

When unset, `edit`, `merge`, and `deprecate` must exit non-zero and print:

```text
routine library UI is behind MEMD_A12_ROUTINE_LIB_UI=1
```

`browse` may remain available without the flag.

## 7. Compatibility

- `memd procedure ...` remains supported.
- Existing procedure records may back routine records.
- Existing procedure statuses must map losslessly where possible:
  - candidate -> candidate
  - promoted/active -> active
  - retired -> deprecated
- No V12 routine command may change visibility rules from V11 project/workspace
  isolation.

## 8. G12 Evidence Hooks

A12 must leave these observable facts for G12:

- `memd routines browse` lists `lint-format` from the shared seed fixture.
- `memd routines edit` can change summary and steps.
- `memd routines merge` records source ids.
- `memd routines deprecate` hides the routine by default and shows it with
  explicit deprecated/all status.

G12 records this evidence in:

```text
docs/verification/v12-proof-runs/*routine-library*.ndjson
```

## 9. Changelog

- 2026-05-05 opened for A12. Defines record fields, lifecycle, CLI shape,
  feature flag, compatibility boundary, and G12 evidence hooks.

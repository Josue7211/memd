---
phase: A12
name: Routine Library UI
version: v12
kind: implementation-plan
status: closed
closed: 2026-05-05
opened: 2026-05-05
depends_on: [V11, docs/phases/v12/V12-INTEGRATION.md, docs/verification/milestones/MILESTONE-v12.md]
phase_doc: docs/phases/v12/V12-INTEGRATION.md
granularity: "one step = <=1 agent session; TDD; commit per task"
axis: procedural_reuse
feature_flag: MEMD_A12_ROUTINE_LIB_UI
---

# Phase A12 - Routine Library UI

## 0. Executive Summary

Add the curated routine-library surface that V12 needs before composition,
inheritance, export/import, and G12 proof. A12 turns existing procedure
substrate into user-facing `memd routines` commands: browse, edit, merge, and
deprecate. It claims no final procedural_reuse lift alone; G12 awards credit
only after A12-D12 work runs in the dual-harness scenario.

## 1. Surface Area

### Files To Create

| Path | Responsibility |
| --- | --- |
| `docs/contracts/routine-library.md` | Routine object contract, lifecycle, merge/deprecation semantics, and audit hooks consumed by B12-D12/G12. |
| `crates/memd-core/src/routine/library.rs` | Pure routine-library model: editable metadata, command steps, merge candidates, deprecation marker. |
| `crates/memd-client/fixtures/shared/routines/seed-library.jsonl` | Shared fixture seeded by A12 and reused by B12/C12/D12/G12. |
| `crates/memd-client/src/main_tests/v12_routine_library_tests/mod.rs` | CLI/runtime tests for A12 behavior. |

### Files To Modify

| Path | Change |
| --- | --- |
| `crates/memd-core/src/routine/mod.rs` | Re-export `library`. |
| `crates/memd-client/src/cli/args.rs` | Add `Routines` command with `browse`, `edit`, `merge`, and `deprecate` subcommands. |
| `crates/memd-client/src/cli/mod.rs` | Route `Commands::Routines` to runtime. |
| `crates/memd-client/src/cli/cli_inspection_runtime.rs` | Reuse existing procedure client APIs where possible; add routine-specific render/output. |
| `crates/memd-client/src/main_tests/mod.rs` | Wire `v12_routine_library_tests`. |
| `crates/memd-server/src/procedural.rs` | Add any missing fields needed for editable title/steps/status without breaking existing `procedure` API. |

## 2. Contract

A routine is a curated procedural memory item with these stable fields:

| Field | Rule |
| --- | --- |
| `id` | Stable UUID from existing procedure record when backed by server state. |
| `name` | Human-readable unique name inside one library scope. |
| `summary` | One-line purpose used in browse/match output. |
| `steps` | Ordered command/action text. A12 supports inline replace; B12 owns composition semantics. |
| `status` | `candidate`, `active`, `deprecated`, or `merged`. |
| `source_ids` | Input routine ids for merge provenance. Empty for direct record/edit. |
| `replaces_id` | Prior routine id when edit creates a new revision. |
| `updated_by` | Harness/agent/user string for H12 audit integration. |
| `workspace_id` | Existing V11 workspace boundary. D12 later exports by workspace. |

Lifecycle:

1. `candidate` routines come from V10 detection or `procedure detect`.
2. `edit` promotes a routine to `active` unless `--status` says otherwise.
3. `merge` creates one `active` routine and marks inputs `merged`.
4. `deprecate` marks a routine `deprecated`; it remains visible with
   `--include-deprecated` and hidden by default.

## 3. CLI Shape

The user-facing command is plural:

```bash
memd routines browse [--workspace <id>] [--status candidate|active|deprecated|merged|all] [--json]
memd routines edit <id> --name <name> [--summary <text>] [--steps-file <path>] [--json]
memd routines merge <id> <id>... --name <name> [--summary <text>] [--json]
memd routines deprecate <id> --reason <text> [--json]
```

Compatibility:

- Existing `memd procedure ...` stays available.
- `memd routines browse` may call the same server endpoint as `memd procedure list`
  during A12, but output must use routine-language labels.
- Later B12 may add `memd routines compose`; do not overload A12 `merge` with
  composition semantics.

## 4. Test Matrix

1. `a12_core_routine_library_serializes_seed_fixture`
2. `a12_browse_lists_active_routines_and_hides_deprecated`
3. `a12_browse_all_includes_deprecated`
4. `a12_edit_updates_name_summary_and_steps`
5. `a12_edit_creates_revision_with_replaces_id`
6. `a12_merge_marks_inputs_merged_and_creates_active_output`
7. `a12_deprecate_requires_reason`
8. `a12_deprecated_routine_not_returned_by_default_match`
9. `a12_cli_json_shape_stable_for_g12_fixture`
10. `a12_feature_flag_off_prints_clear_error_for_mutating_commands`
11. `a12_existing_procedure_commands_still_pass`
12. `a12_seed_library_fixture_replays_end_to_end`

## 5. Fixtures

`crates/memd-client/fixtures/shared/routines/seed-library.jsonl` contains:

- `lint` active routine with two steps.
- `format` active routine with one step.
- `lint-format-candidate` candidate routine from repeated file-touch pattern.
- `old-lint-format` deprecated routine used by hide/show tests.

The fixture must be stable JSONL, one routine per line, with deterministic ids
so B12/C12/D12/G12 can reference the same records.

## 6. Feature Flag

| Var | Default | Effect |
| --- | --- | --- |
| `MEMD_A12_ROUTINE_LIB_UI` | `0` | Mutating commands (`edit`, `merge`, `deprecate`) require flag until A12.N graduation. `browse` may remain read-only without the flag. |

When the flag is off, mutating commands exit non-zero with:

```text
routine library UI is behind MEMD_A12_ROUTINE_LIB_UI=1
```

## 7. Task List

### Task A12.1 - routine contract

- [x] Write `docs/contracts/routine-library.md`.
- [ ] Commit: `docs(v12): define routine library contract (A12)`.

### Task A12.2 - core model

- [x] Add failing tests for serialization, status filtering, edit revision, and merge provenance.
- [x] Implement `memd_core::routine::library`.
- [ ] Commit: `feat(core/a12): model curated routines (A12)`.

### Task A12.3 - seed fixture

- [x] Add `seed-library.jsonl`.
- [x] Add deterministic fixture replay test.
- [ ] Commit: `test(fixtures/a12): seed shared routine library (A12)`.

### Task A12.4 - CLI read path

- [x] Add `memd routines browse`.
- [x] Reuse existing procedure list endpoint if enough fields exist.
- [ ] Commit: `feat(cli/a12): browse routine library (A12)`.

### Task A12.5 - edit path

- [x] Add `memd routines edit`.
- [x] Gate mutating command with `MEMD_A12_ROUTINE_LIB_UI`.
- [ ] Commit: `feat(cli/a12): edit routines (A12)`.

### Task A12.6 - merge path

- [x] Add `memd routines merge`.
- [x] Preserve `source_ids` and mark inputs `merged`.
- [ ] Commit: `feat(cli/a12): merge duplicate routines (A12)`.

### Task A12.7 - deprecate path

- [x] Add `memd routines deprecate`.
- [x] Require non-empty `--reason`.
- [ ] Commit: `feat(cli/a12): deprecate routines (A12)`.

### Task A12.8 - compatibility + proof prep

- [x] Run `cargo test -p memd-core routine -- --nocapture`.
- [x] Run `cargo test -p memd-client v12_routine_library -- --nocapture`.
- [x] Run existing `procedure` CLI tests.
- [x] Run `cargo fmt --check`.
- [x] Run `git diff --check`.
- [ ] Commit: `test(a12): verify routine library UI (A12)`.

## 8. Bench Impact

A12 is procedural_reuse infrastructure only. It moves from detected/invoked
routines toward curated routines, but PR stays `6/10` until G12 proves A12-D12
end-to-end in `routine-library.ndjson`.

## 9. Dependency Graph

- Requires: V11 closed, V12 integration doc read.
- Blocks: B12 routine composition, C12 per-project inheritance, D12
  export/import, G12 routine-library proof.

## Exit Criteria

1. `memd routines browse/edit/merge/deprecate` exist and match this CLI shape.
2. Mutating commands are feature-flagged.
3. Existing `memd procedure` commands remain compatible.
4. Shared seed fixture exists and replays.
5. Tests pass and commits are atomic.

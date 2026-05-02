---
phase: living-skills-phase-2
name: Records-as-Truth Resync + Salience-Ranked Active Skills
version: living-skills
kind: implementation-plan
status: ready-to-execute
opened: 2026-05-02
depends_on: [living-skills-phase-1]
contract: docs/contracts/skill-record.md
phase1_handoff: docs/handoff/2026-05-02-living-skills-phase1-closed-next-phase2.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
---

# Living Skills — Phase 2 Implementation Plan

> Depends on Phase 1 (closed 2026-05-02 on `research/mining`). Phase 1
> shipped the foundation: `MemoryKind::Skill`, atomic mirror writer,
> name-validated catalog, Active Skills wake section. Phase 2 makes the
> record the source of truth: mirror regenerable from records, retire
> deletes record by default, B6 distiller treats skills as gated
> semantic candidates, ranked Active Skills.

## 0. Executive summary

Phase 1 produced records and mirrored them. Phase 2 reverses the polarity:
records become canonical, the mirror becomes a derived view. Five named
deliverables, all anticipated by the Phase 1 contract `§10`:

1. `memd skill sync` — records → mirror, idempotent overwrite, byte-stable
   on no-op runs.
2. `record_id` written into frontmatter on `skill add` (Phase 1 wrote it
   into the catalog only; the disk file's frontmatter omits it).
3. `skill retire` default flips: deletes the record by default; today's
   "soft note" behavior moves to `--keep-record`.
4. B6 distiller skill-record gating: explicit policy for whether
   skill-kinded records participate in semantic synthesis (default off,
   opt-in via `--include-skills` per phase contract §11).
5. Salience-ranked Active Skills section in wake (replaces Phase 1's
   alphabetical sort, keyed on record salience scores already produced
   by the resume compiler).

---

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `crates/memd-client/src/cli/cli_skill_sync.rs` | `memd skill sync` subcommand: read records by `MemoryKind::Skill`, regenerate mirror files, write atomic. |
| `crates/memd-client/src/main_tests/skill_sync_tests/mod.rs` | Sync scenarios: empty-mirror reconstruct, drift-overwrite, no-op idempotence, name-collision detection. |
| `crates/memd-core/src/skill_mirror/sync.rs` | Pure regenerator: `(records: Vec<SkillRecord>) → Vec<MirrorWrite>`. No I/O. |
| `crates/memd-client/fixtures/skill-sync/` | Multi-record fixtures, drift-overwrite cases, name-collision pairs. |

### Files to modify

| Path | Change |
| --- | --- |
| `crates/memd-client/src/cli/cli_skill.rs` | `skill add` writes `record_id:` into frontmatter (per contract §8). Default `skill retire` flips to delete-record; add `--keep-record` flag. |
| `crates/memd-client/src/cli/args.rs` | `Skill(SkillSyncArgs)` subcommand, `SkillRetireArgs::keep_record` flag. |
| `crates/memd-client/src/runtime/resume/compiler/active_skills.rs` (or wherever Phase 1 landed it) | Replace alphabetical sort with salience-rank from `SkillCatalogEntry::salience_score` (Phase 1 already plumbs `record_id`; salience comes from the existing resume compiler scorer). |
| `crates/memd-server/src/distiller/b6.rs` | Skip `MemoryKind::Skill` candidates by default; honor `--include-skills` flag from `memd distill` invocation. |
| `docs/contracts/skill-record.md` | Bump to `v2`. Update §6 (retire default), §8 (record_id in frontmatter is now mandatory), §9 (sort = salience, not alphabetical), §10 (mark Phase 2 items as landed), §11 (B6 gating policy explicit). |

---

## 2. Schema changes

None in `memd-schema`. Phase 1 already added `MemoryKind::Skill` and
the `record_id: Option<Uuid>` field on `SkillCatalogEntry`. Phase 2
just promotes the field from optional metadata to mandatory contract.

---

## 3. API shape

### `memd skill sync`

```
memd skill sync [--output <bundle>] [--dry-run] [--prune]
```

- Default: regenerate mirror entries that exist as records, leave
  unrelated mirror files untouched.
- `--dry-run`: print would-write set without I/O.
- `--prune`: delete mirror entries whose record is missing or retired
  (Phase 2 retire deletes record, so prune handles operator-side rm
  of orphaned mirror).

### `memd skill retire` (behavior change)

```
memd skill retire --name <slug> [--output <bundle>] [--keep-record]
```

- Default: deletes mirror file AND record (Phase 1 default was
  mirror-only with a soft "record retirement pending (Phase 2)" note).
- `--keep-record`: Phase 1 behavior, mirror gone but record stays
  (`status=Retired` on the memory).

---

## 4. Test matrix

### Unit (sync regenerator, pure)

1. `sync_empty_records_yields_empty_mirror_writes`
2. `sync_single_record_produces_one_mirror_write`
3. `sync_drifted_mirror_overwrites_atomically`
4. `sync_idempotent_byte_stable_on_second_run`
5. `sync_detects_name_collision_returns_typed_err`

### Unit (`record_id` write-through)

6. `skill_add_writes_record_id_into_frontmatter`
7. `skill_add_frontmatter_record_id_round_trips_via_parse_skill_metadata`

### Unit (retire flip)

8. `skill_retire_default_deletes_record_and_mirror`
9. `skill_retire_keep_record_preserves_record_status_retired`
10. `skill_retire_keep_record_still_removes_mirror_file`

### Unit (Active Skills sort)

11. `active_skills_section_orders_by_salience_descending`
12. `active_skills_section_falls_back_to_alphabetical_on_tie`

### Unit (B6 gating)

13. `b6_distiller_skips_skill_kind_by_default`
14. `b6_distiller_includes_skills_when_flag_set`
15. `b6_distiller_skill_inclusion_logged_in_audit_trail`

### Integration

16. `phase2_e2e_add_sync_retire_round_trip` — CLI scenarios from Phase 1
    handoff `## Verify-green commands` block, extended for sync + the
    flipped retire default.
17. `phase2_e2e_b6_distill_skill_mode_optin` — distiller round-trip
    with `--include-skills` flag.

---

## 5. Build sequence

| # | Task | Tests | Files |
| --- | --- | --- | --- |
| P2.1 | Pure sync regenerator | 1–5 | `skill_mirror/sync.rs` |
| P2.2 | `memd skill sync` CLI wiring | 16 | `cli_skill_sync.rs`, `args.rs`, `main_tests/skill_sync_tests/` |
| P2.3 | `record_id` frontmatter write-through | 6–7 | `cli_skill.rs` |
| P2.4 | Retire default flip + `--keep-record` | 8–10 | `cli_skill.rs`, `args.rs` |
| P2.5 | Salience-ranked Active Skills | 11–12 | `runtime/resume/compiler/active_skills.rs` |
| P2.6 | B6 distiller skill-record gating | 13–15, 17 | `memd-server/src/distiller/b6.rs` |
| P2.7 | Contract bump to `v2` + migration note | n/a (docs) | `docs/contracts/skill-record.md` |

Sequencing rationale: P2.1 + P2.3 unblock everything (sync needs the
record-id round trip; retire flip uses the same parse path). P2.5 + P2.6
are independent — can land in parallel after P2.4. P2.7 last so the
contract reflects landed state.

---

## 6. Pass gates

- Phase 1 verify-green commands continue passing (regression floor).
- New CLI commands documented in skill contract `§3` (API shape).
- B6 distiller default behavior unchanged for non-skill kinds (no
  silent semantic synthesis of skill text).
- `memd skill sync --dry-run` on a clean Phase-1 bundle reports zero
  writes (idempotence proof).

---

## 7. Out of scope (Phase 3+)

- Cross-project / global skill mirror surface (contract §10 anticipates).
- Skill versioning + migration tooling (rename, body-edit history).
- Skill-record decay policy (today: never decays; future: `inactivity_horizon`).
- Skill-record promotion to "official" with project-level review.

---

## 8. Truth state at plan-write

- Phase 1 commit range: `babea58` (schema kind) → `4cd35fa` (roadmap flip).
- Phase 1 closed 2026-05-02, all four final commits pushed to
  `origin/research/mining` (per Phase 1 close handoff).
- Cache back-compat already shipped in Phase 1: `cache_deserializes_pre_phase1`
  test in `crates/memd-client/src/main/cache.rs` proves
  `kind=skill` records survive deserialization on a pre-Phase-1 binary
  (returns `Skill` as `Other` until binary upgrade).
- `SkillCatalogEntry::record_id: Option<Uuid>` field exists; Phase 2
  promotes it to mandatory in v2 contract.
- `parse_skill_metadata` already extracts `record_id:` from frontmatter
  if present — Phase 1 just doesn't write it on `skill add`.

---

## 9. Phase 2 → Phase 3 lookahead (for plan-doc continuity)

Phase 3 introduces the global skill surface. Phase 2 is the prerequisite:
records-as-truth means the mirror is regeneratable, which is the precondition
for "regenerate into a different scope (global vs project)".

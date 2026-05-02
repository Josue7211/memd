# Skill Record Contract (Living Skills Phase 2)

This document is the Living Skills Phase 2 contract: project-local
"skill" memories carried as canonical memd records, with the on-disk
`.memd/skills/<name>/SKILL.md` mirror **regenerable from records**.
Pairs with `docs/contracts/type-taxonomy.md` (F5 kind boundary) —
`Skill` is the 13th kind on that boundary.

Phase 2 inverts the Phase 1 truth direction: records are the source of
truth, the mirror is a derived view. `memd skill sync` is the
regenerator; `memd skill retire` deletes the record by default.

## 1. Pipeline position

```
memd skill add ──► prepare_and_mirror_skill ──► remember (kind=skill)
                          │                              │
                          ▼                              ▼
              .memd/skills/<name>/SKILL.md        canonical record
                          ▲                              │
                          │                              │
                          └──── memd skill sync ◄────────┘
                          │       (records → mirror)
                          ▼
                       SkillCatalog
                          │
                          ▼
              render_bundle_wakeup_markdown
                  ("## Active Skills", salience-sorted)
```

The mirror is reproducible: given the canonical record set, `memd skill
sync` reconstructs `.memd/skills/` byte-stable. Drift is a transient
state, never a divergence — the next sync resolves it.

## 2. Schema

`SkillBody` lives in `memd-schema/src/skill.rs`:

```rust
pub struct SkillFrontmatter {
    pub name: String,                 // [a-z0-9_-]+ — used as dir name
    pub description: String,          // single line, surfaced by catalog
    pub record_id: Option<Uuid>,      // canonical link back to memd record
    pub salience: Option<f32>,        // P2.5 — wake ranking signal
}

pub struct SkillBody {
    pub frontmatter: SkillFrontmatter,
    pub body: String,                 // markdown, follows YAML head verbatim
}
```

`#[derive]` is `(Debug, Clone, Serialize, Deserialize, PartialEq)`.
`Eq` is intentionally absent — `Option<f32>` precludes it. PartialEq is
sufficient for `assert_eq!`, serde, and round-trip tests.

Render contract (`SkillBody::render_skill_md`):

```
---
name: <frontmatter.name>
description: <frontmatter.description>
record_id: <uuid>          # omitted when None
salience: <f32>            # omitted when None
---

<body>
```

Trailing newline is appended. Frontmatter MUST be parseable by the
existing `SkillCatalog` line-by-line YAML reader, which only consumes
`name:` and `description:`. Unknown keys are ignored.

`SkillBody::parse_skill_md` is the canonical inverse. **Records-as-truth
depends on `parse(render(x)) == x`** for the closed set of Phase 2
fields (name, description, record_id, salience, body). The wake render
path uses `parse_skill_md` directly — no forked YAML reader.

## 3. Name validation

`memd_core::skill_mirror::validate_skill_name` is authoritative:

- Pattern: `^[a-z0-9][a-z0-9_-]*$` (lowercase, no leading separator).
- Rejects: empty, uppercase, leading `-`/`_`, path separators, `..`,
  `.`, embedded slashes, embedded null bytes, names starting with `.`.

Any FS write path MUST validate before joining. `write_mirror`
validates internally; direct callers of `SkillBody::mirror_relpath`
MUST replicate the call (the relpath builder does not validate).

`memd skill sync` (the regenerator) validates every record's name
before producing a mirror write — invalid names surface as typed
`SyncError::InvalidName`, not silent skips.

## 4. Mirror path

`bundle_root.join("skills").join(<name>).join("SKILL.md")`

Bundle root is the resolved `.memd` directory (not its parent). One
directory per skill. The SKILL.md file is the only authoritative
on-disk artifact; the mirror dir is the deletion unit during prune.

## 5. Atomic write semantics

`write_mirror` and `apply_sync` both:

1. Validate name (sync does this in the regenerator step).
2. Create `<root>/skills/<name>/` (idempotent).
3. Write rendered SKILL.md to a sibling `.SKILL.md.tmp`.
4. `fs::rename` tmp → `SKILL.md` (POSIX atomic on same filesystem).

Failure between steps 3 and 4 leaves the tmp file orphaned but never
corrupts the canonical SKILL.md. The next sync overwrites both.

## 6. Remove semantics

Phase 2 default: `memd skill retire --name <n>` removes both the
record and the mirror dir. `--keep-record` retains the record (used
when the mirror layout changes but the underlying knowledge persists).

`remove_mirror(bundle_root, name)`:

- Validates name first (defensive — even though we are deleting).
- Removes `<root>/skills/<name>/` (whole dir, not just SKILL.md).
- **Idempotent**: missing dir is `Ok(())`, not an error.

`apply_sync(..., prune: true)` removes mirror dirs whose name is not
in the live record set. Dry-run lists what would be pruned without
touching disk.

## 7. CLI contract

```
memd skill add    --name <n> --description <d> [--body <s> | --body-file <p> | --stdin]
                  [--output <bundle>] [--scope local|project|global] [--tag <t>...]
memd skill list   [--output <bundle>] [--json]
memd skill show   --name <n> [--output <bundle>]
memd skill retire --name <n> [--output <bundle>] [--keep-record]
memd skill sync   [--output <bundle>] [--dry-run] [--prune]
```

`add` writes the mirror, then calls `remember_with_bundle_defaults`
with `kind="skill"`, `scope="project"` (default), and an auto-tag of
`skill:<name>`. The full `render_skill_md()` output is the record
content (records-as-truth, P2.3) — sync can reconstruct the mirror
from any single record without consulting the disk. On `remember`
failure, the mirror is rolled back via `remove_mirror`; rollback
failure is surfaced (not swallowed).

`sync` reads all `kind=skill` records, dedupes + sorts by name, and
produces the mirror write set via `memd_core::skill_mirror::sync::regenerate`.
With `--prune`, orphan mirror dirs (no matching record) are removed.
`--dry-run` reports what would change without touching disk.

`retire` (Phase 2 default) deletes both record and mirror.
`--keep-record` opts out of the record deletion only.

Output for `add` is single-line JSON:

```json
{"skill":"<name>","mirror":"<absolute path>","record_id":"<uuid>"}
```

`list` and `show` read disk directly (offline catalog discovery).
`sync` reads records from the server — it requires a running daemon.

## 8. SkillCatalog integration

`SkillCatalogEntry` carries `record_id: Option<Uuid>` so the catalog
links discovered skills back to memd records. `parse_skill_metadata`
extracts `record_id:` from the YAML head when present, tolerates
absence, and tolerates surrounding quotes.

Catalog discovery is read-only from the mirror — it does not query
records. This preserves offline operation. Records-as-truth means the
mirror was *produced* from records; catalog consumption of the mirror
is unchanged from Phase 1.

## 9. Wake render integration

`render_bundle_wakeup_markdown` includes an `## Active Skills` section
when `<bundle>/skills/` contains at least one valid SKILL.md (parses
successfully via `SkillBody::parse_skill_md`).

Sort order:

1. **Salience descending** — higher salience surfaces first.
2. **Name ascending** on tie (including the all-`None`-salience case,
   which preserves Phase 1 alphabetical behavior).

The first entry's body is inlined (truncated to 500 chars + ellipsis);
remaining entries are listed with a `memd lookup --kind skill --name
<name>` hint. Section is omitted when no skills exist (no empty
header).

`salience` is part of the schema and round-trips through
`render_skill_md` / `parse_skill_md`. No current code path populates
it: `skill add` writes `None`, `skill sync` parses whatever the record
content contains. With all-`None` skills, sort collapses to
alphabetical (Phase 1 behavior preserved). A future phase will stamp
salience from record state (recall counts, recency, manual pin) — the
contract here is the field and the sort, not the source formula.

## 10. Records-as-truth contract

Phase 2 inverts the Phase 1 truth direction. The canonical record is
the full `render_skill_md()` output of the SkillBody. Sync regenerates
the mirror from records; the mirror never feeds back into records.

Implications:

- **Drift is recoverable**: editing SKILL.md by hand survives only
  until the next sync. To change a skill, change the record.
- **Mirror is byte-stable**: the regenerator (`skill_mirror::sync`)
  uses BTreeMap-ordered output and a deterministic render. Two syncs
  on the same record set produce identical bytes.
- **Single parser**: wake render and sync both use
  `SkillBody::parse_skill_md`. There is no forked frontmatter reader.

## 11. B6 distiller boundary

The B6 distiller (`crates/memd-client/src/benchmark/typed_ingest/distiller.rs`)
does not interact with skill records, by structural guarantee:

- **No emit path**: `CandidateKind` is `Fact | Decision | Preference`.
  The enum cannot represent `Skill`, so the distiller cannot produce
  Skill candidates. `validate_distill_json` rejects any other kind
  string.
- **No read path**: B6 ingests session transcripts, not memory
  records. Grepping `crates/memd-client/src/benchmark/typed_ingest/`
  for `search_memory|MemoryItem|RetrievalIntent|client.search|client.lookup`
  returns no matches.

Phase 2 originally planned a `--include-skills` gating flag for B6.
That collapsed to zero implementation work because the consumption
path doesn't exist. The structural guarantee replaces the flag: a
future B6 change that adds memory consumption MUST keep skill records
out of the distiller's input set, or extend `CandidateKind` and this
contract together.

## 12. Telemetry

Phase 2 emits no skill-specific telemetry beyond standard memory
pipeline events. `memd skill sync` reports its `SyncReport` (writes,
prunes) as command output but does not metric.

## 13. Migration: Phase 1 → Phase 2

Existing Phase 1 mirrors are forward-compatible:

- Phase 1 SKILL.md files lack `salience:` and may lack `record_id:`.
  `parse_skill_md` tolerates both (Option fields default to None).
- Running `memd skill sync` on a Phase 1 bundle is safe: records
  containing rendered Phase 1 mirrors parse cleanly; sync regenerates
  to the Phase 2 shape (still missing salience until a future phase
  stamps it).
- `retire` semantics changed default (Phase 1: `--keep-record` was
  implicit; Phase 2: record deletion is the default). Scripts that
  relied on the old behavior must add `--keep-record` explicitly.

## 14. Versioning

This contract is `v2`. Phase 2 changes from v1:

- **Schema**: `SkillFrontmatter` adds `record_id: Option<Uuid>` and
  `salience: Option<f32>`. `Eq` derive dropped (f32 has no Eq).
- **Records-as-truth**: full `render_skill_md` is the record content;
  mirror is regenerable.
- **CLI**: `memd skill sync [--dry-run] [--prune]` added. `retire`
  default changed to delete-record; `--keep-record` is the opt-out.
- **Wake**: Active Skills section sorts by salience desc, name asc on
  tie. `parse_skill_md` is the single canonical parser.
- **B6 boundary**: documented as structural (CandidateKind enum + no
  read path), not gated by flag.

Future breaking changes (frontmatter rename, mirror path move, name
pattern tightening) require a v3 bump and a migration path for
existing on-disk skills.

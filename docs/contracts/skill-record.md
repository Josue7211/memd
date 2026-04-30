# Skill Record Contract (Living Skills Phase 1)

This document is the Living Skills Phase 1 contract: project-local
"skill" memories carried as canonical memd records that mirror to
`.memd/skills/<name>/SKILL.md` for harness discovery. Pairs with
`docs/contracts/type-taxonomy.md` (F5 kind boundary) — `Skill` is the
13th kind on that boundary.

## 1. Pipeline position

```
memd skill add ──► prepare_and_mirror_skill ──► remember (kind=skill)
                          │                              │
                          ▼                              ▼
              .memd/skills/<name>/SKILL.md        canonical record
                          │                              │
                          └────► SkillCatalog ◄──────────┘
                                       │
                                       ▼
                          render_bundle_wakeup_markdown
                              ("## Active Skills")
```

Phase 1 is the wedge: a single command produces both a record and a
mirror, both surfaces (catalog + wake) read the mirror as truth, and
`memd skill retire` is idempotent against missing files. Phase 2 adds
records-as-truth resync (mirror is regenerated from records).

## 2. Schema

`SkillBody` lives in `memd-schema/src/skill.rs`:

```rust
pub struct SkillFrontmatter {
    pub name: String,        // [a-z0-9_-]+ — used as dir name
    pub description: String, // single line, surfaced by catalog
}

pub struct SkillBody {
    pub frontmatter: SkillFrontmatter,
    pub body: String, // markdown, follows YAML head verbatim
}
```

Render contract (`SkillBody::render_skill_md`):

```
---
name: <frontmatter.name>
description: <frontmatter.description>
---

<body>
```

Trailing newline is appended. Frontmatter MUST be parseable by the
existing `SkillCatalog` line-by-line YAML reader, which only consumes
`name:` and `description:`. Additional keys (e.g. `record_id:`) are
permitted; unknown keys are ignored. Frontmatter delimiter is `---`
(three hyphens) on its own line.

## 3. Name validation

`memd_core::skill_mirror::validate_skill_name` is authoritative:

- Pattern: `^[a-z0-9][a-z0-9_-]*$` (lowercase, no leading separator).
- Rejects: empty, uppercase, leading `-`/`_`, path separators, `..`,
  `.`, embedded slashes, embedded null bytes, names starting with `.`.

Any FS write path MUST validate before joining. `write_mirror`
validates internally; direct callers of `SkillBody::mirror_relpath`
MUST replicate the call (the relpath builder does not validate).

## 4. Mirror path

`bundle_root.join("skills").join(<name>).join("SKILL.md")`

Bundle root is the resolved `.memd` directory (not its parent). One
directory per skill. The SKILL.md file is the only authoritative
on-disk artifact in Phase 1.

## 5. Atomic write semantics

`write_mirror`:

1. Validate name.
2. Create `<root>/skills/<name>/` (idempotent).
3. Write rendered SKILL.md to a sibling `SKILL.md.<uuid>.tmp`.
4. `fs::rename` tmp → `SKILL.md` (POSIX atomic on same filesystem).
5. Return absolute path to SKILL.md.

Failure between steps 3 and 4 leaves the tmp file orphaned but never
corrupts the canonical SKILL.md. Callers MAY garbage-collect orphan
`*.tmp` siblings; Phase 1 does not.

## 6. Remove semantics

`remove_mirror(bundle_root, name)`:

- Validates name first (defensive — even though we are deleting).
- Removes `<root>/skills/<name>/SKILL.md` if present.
- Removes `<root>/skills/<name>/` if empty.
- **Idempotent**: missing file/dir is `Ok(())`, not an error.

The `memd skill retire --keep-record` flag controls only the record
side (Phase 2). The mirror is always removed.

## 7. CLI contract

```
memd skill add    --name <n> --description <d> [--body <s> | --body-file <p> | --stdin]
                  [--output <bundle>] [--scope local|project|global] [--tag <t>...]
memd skill list   [--output <bundle>] [--json]
memd skill show   --name <n> [--output <bundle>]
memd skill retire --name <n> [--output <bundle>] [--keep-record]
```

`add` writes the mirror, then calls `remember_with_bundle_defaults`
with `kind="skill"`, `scope="project"` (default), and an auto-tag of
`skill:<name>`. On `remember` failure, the mirror is rolled back via
`remove_mirror`; rollback failure is surfaced (not swallowed).

Output for `add` is single-line JSON:

```json
{"skill":"<name>","mirror":"<absolute path>","record_id":"<uuid>"}
```

`list` and `show` read disk directly (do not query the server). This
is intentional — Phase 1 catalog discovery must work offline.

## 8. SkillCatalog integration

`SkillCatalogEntry` carries an optional `record_id: Option<Uuid>` so
the catalog can link a discovered skill back to its memd record once
records-as-truth (Phase 2) lands. The cache entry mirrors the field
with `#[serde(default, skip_serializing_if = "Option::is_none")]`,
preserving back-compat with caches written before Phase 1.

`parse_skill_metadata` extracts `record_id:` from the YAML head when
present, tolerates absence, and tolerates surrounding quotes.

## 9. Wake render integration

`render_bundle_wakeup_markdown` includes an `## Active Skills` section
when `<bundle>/skills/` contains at least one `SKILL.md`. The first
entry's body is inlined (truncated to 500 chars + ellipsis); remaining
entries are listed with a `memd lookup --kind skill --name <name>`
hint. Section is omitted entirely when no skills exist (no empty
header).

Sort order is alphabetical by directory name. Phase 2 will replace
this with a ranked surface driven by record salience.

## 10. Forward compatibility

Phase 1 → Phase 2 changes that this contract anticipates:

- **Records as truth**: `memd skill sync` will regenerate the mirror
  from records. The `record_id` frontmatter field already lets the
  catalog round-trip without a re-scan.
- **Retire deletes records**: today `--keep-record` is the default
  behavior (record retirement is logged TODO in `add` output).
- **Cross-project sharing**: `--scope global` is accepted today but
  the mirror is always written under the local bundle root. Phase 3
  adds a global mirror surface.

## 11. Telemetry

Phase 1 emits no skill-specific telemetry. The `remember` call
participates in the standard memory pipeline (B6 distiller will treat
skill records as semantic candidates by default; explicit gating is
Phase 2).

## 12. Versioning

This contract is `v1`. Breaking changes (frontmatter rename, mirror
path move, name pattern tightening) require a new version and a
migration path for existing on-disk skills.

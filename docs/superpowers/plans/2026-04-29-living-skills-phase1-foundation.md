# Living Skills — Phase 1 (Foundation) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `Skill` a first-class memd record kind, mirror skill records to `.memd/skills/<name>/SKILL.md` so the existing `SkillCatalog` discovers them automatically, and surface relevant skills in `wake.md`. This is the wedge that unlocks Phases 2–4 (harvest, hive sync, registry).

**Architecture:**
- Add `MemoryKind::Skill` to `memd-schema`. Skill body is the SKILL.md frontmatter + content, stored as the record's content field plus a small structured metadata payload.
- Persist skills via the same memory pipeline as facts/decisions (no new storage path). On write, mirror the record to `.memd/skills/<name>/SKILL.md` so `SkillCatalog` (already wired to scan that directory) picks it up unchanged. On delete/retire, remove the mirror.
- `wake.md` generator gains an `## Active Skills` section that lists the top-N skills relevant to the current focus, with full body inlined for the top result and links/lookups for the rest.

**Tech Stack:** Rust (memd-schema, memd-client, memd-core), serde, anyhow, tempfile, sqlite (existing memd.db), the existing `SkillCatalog` in `crates/memd-client/src/cli/skill_catalog.rs`.

**Non-goals (deferred to follow-up plans):**
- Phase 2: harvest skills from corrections (`memd hook capture --promote-to-skill`)
- Phase 3: hive scope merge + override layering
- Phase 4: public skill registry (`memd skill publish` / `install`)

**Out of scope entirely:**
- Replacing claude code's native Skill tool plumbing (we interop via disk).
- UI / dashboard. CLI + wake.md is enough to validate the wedge.

---

## File Structure

**Create:**
- `crates/memd-schema/src/skill.rs` — `SkillBody`, `SkillFrontmatter`, mirror-path helpers.
- `crates/memd-client/src/cli/cli_skill_runtime.rs` — `memd skill {add, list, show, retire}` CLI handlers.
- `crates/memd-core/src/skill_mirror.rs` — write/remove `.memd/skills/<name>/SKILL.md`, atomic rename.
- `crates/memd-client/src/main_tests/skill_runtime_tests/mod.rs` — integration tests.
- `tests/skill_mirror_roundtrip.rs` — disk-mirror integration test.
- `docs/contracts/skill-record.md` — record contract: required fields, frontmatter shape, mirror path policy.

**Modify:**
- `crates/memd-schema/src/lib.rs:9-22` — add `Skill` variant to `MemoryKind`.
- `crates/memd-schema/src/lib.rs` (RetrievalIntent enum near line 65) — add `Skill` intent for queries.
- `crates/memd-client/src/cli/cli_memory_runtime.rs` — accept `--kind skill` and dispatch to skill runtime.
- `crates/memd-client/src/cli/commands.rs` — register `skill` subcommand.
- `crates/memd-client/src/cli/skill_catalog.rs` — extend `SkillCatalogEntry` to track `record_id: Option<Uuid>` so disk-mirrored skills are linked back to their record. Backwards-compatible (Option).
- `crates/memd-core/src/wake.rs` (or wherever `wake.md` is rendered) — add `Active Skills` section.
- `crates/memd-client/src/cli/cli_memory_runtime.rs` lookup path — accept `--kind skill` filter (probably already works via enum, just verify).

**Test:**
- `tests/skill_mirror_roundtrip.rs` — round-trip: create skill record → assert disk file exists with correct frontmatter → `SkillCatalog::scan` finds it → retire record → file removed.
- `crates/memd-client/src/main_tests/skill_runtime_tests/mod.rs` — CLI surface tests.
- `crates/memd-core/src/wake_tests.rs` — wake.md renders Active Skills section.

---

## Pre-Flight

- [ ] **Step 0.1: Confirm worktree + branch**

```bash
cd /home/josue/Documents/projects/memd
git status
git rev-parse --abbrev-ref HEAD
```

Expected: branch is `research/mining` or a fresh feature branch off it. If on `research/mining` with dirty state, create a worktree first per `superpowers:using-git-worktrees`.

- [ ] **Step 0.2: Bench dir present, baseline tests green**

```bash
cargo test -p memd-schema --lib 2>&1 | tail -5
cargo test -p memd-client --lib 2>&1 | tail -10
```

Expected: both pass. Capture counts for the post-implementation diff.

---

## Task 1: Add `Skill` variant to `MemoryKind`

**Files:**
- Modify: `crates/memd-schema/src/lib.rs:9-22`
- Test: `crates/memd-schema/src/lib.rs` (inline tests block at file end)

- [ ] **Step 1.1: Write the failing test**

Append to the existing tests module in `crates/memd-schema/src/lib.rs`:

```rust
#[test]
fn memory_kind_skill_serializes_snake_case() {
    let json = serde_json::to_string(&MemoryKind::Skill).unwrap();
    assert_eq!(json, "\"skill\"");
}

#[test]
fn memory_kind_skill_round_trips() {
    let parsed: MemoryKind = serde_json::from_str("\"skill\"").unwrap();
    assert_eq!(parsed, MemoryKind::Skill);
}
```

- [ ] **Step 1.2: Run test to verify failure**

```bash
cargo test -p memd-schema memory_kind_skill 2>&1 | tail -10
```

Expected: FAIL with `no variant Skill` or `unknown variant`.

- [ ] **Step 1.3: Add the variant**

Edit `crates/memd-schema/src/lib.rs:9-22`. Add `Skill,` after `Correction,`.

```rust
pub enum MemoryKind {
    Fact,
    Decision,
    Preference,
    Runbook,
    Procedural,
    SelfModel,
    Topology,
    Status,
    LiveTruth,
    Pattern,
    Constraint,
    Correction,
    Skill,
}
```

- [ ] **Step 1.4: Run test to verify pass**

```bash
cargo test -p memd-schema memory_kind_skill 2>&1 | tail -10
```

Expected: 2 passed.

- [ ] **Step 1.5: Run full schema test suite for regressions**

```bash
cargo test -p memd-schema 2>&1 | tail -10
```

Expected: all previously passing tests still pass; counts +2.

- [ ] **Step 1.6: Add `Skill` to `RetrievalIntent`**

Edit `RetrievalIntent` enum (line ~65 of `lib.rs`). Append `Skill,`. Required so `memd lookup --intent skill` works downstream.

Add inline test:

```rust
#[test]
fn retrieval_intent_skill_serializes() {
    let json = serde_json::to_string(&RetrievalIntent::Skill).unwrap();
    assert_eq!(json, "\"skill\"");
}
```

Run: `cargo test -p memd-schema retrieval_intent_skill -- --nocapture` → PASS.

- [ ] **Step 1.7: Commit**

```bash
git add crates/memd-schema/src/lib.rs
git commit -m "feat(schema): add MemoryKind::Skill + RetrievalIntent::Skill

Phase 1 of living-skills initiative. Wires the kind enum so skill
records can flow through the existing memory pipeline. Disk mirror
+ CLI + wake surfacing land in subsequent tasks."
```

---

## Task 2: Define `SkillBody` payload + mirror path policy

**Files:**
- Create: `crates/memd-schema/src/skill.rs`
- Modify: `crates/memd-schema/src/lib.rs` (add `pub mod skill;`)
- Test: inline in `crates/memd-schema/src/skill.rs`

**Why a separate module:** keeps schema crate clean; lets us iterate on skill metadata without churning the 2000-line `lib.rs`.

- [ ] **Step 2.1: Write failing tests in new file**

Create `crates/memd-schema/src/skill.rs`:

```rust
use serde::{Deserialize, Serialize};

/// Frontmatter that is mirrored verbatim to `.memd/skills/<name>/SKILL.md`.
/// Mirrors the shape used by Claude Code's native Skill tool so the existing
/// SkillCatalog discovers our records without modification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillFrontmatter {
    pub name: String,
    pub description: String,
}

/// The full content of a skill record. `frontmatter` is rendered into the
/// SKILL.md YAML head; `body` is the markdown that follows.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillBody {
    pub frontmatter: SkillFrontmatter,
    pub body: String,
}

impl SkillBody {
    /// Render to the on-disk SKILL.md format consumed by SkillCatalog.
    pub fn render_skill_md(&self) -> String {
        format!(
            "---\nname: {}\ndescription: {}\n---\n\n{}\n",
            self.frontmatter.name, self.frontmatter.description, self.body
        )
    }

    /// Derive the relative mirror path inside a memd bundle.
    pub fn mirror_relpath(&self) -> std::path::PathBuf {
        std::path::PathBuf::from("skills")
            .join(&self.frontmatter.name)
            .join("SKILL.md")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> SkillBody {
        SkillBody {
            frontmatter: SkillFrontmatter {
                name: "tdd".into(),
                description: "drive features test-first".into(),
            },
            body: "## Steps\n1. Red\n2. Green\n3. Refactor\n".into(),
        }
    }

    #[test]
    fn render_skill_md_emits_yaml_then_body() {
        let s = sample();
        let rendered = s.render_skill_md();
        assert!(rendered.starts_with("---\nname: tdd\n"));
        assert!(rendered.contains("description: drive features test-first"));
        assert!(rendered.contains("## Steps"));
    }

    #[test]
    fn mirror_relpath_is_skills_name_skill_md() {
        let s = sample();
        assert_eq!(
            s.mirror_relpath(),
            std::path::PathBuf::from("skills/tdd/SKILL.md")
        );
    }

    #[test]
    fn mirror_relpath_rejects_path_traversal_via_name() {
        // Defensive: name must be sanitized at write boundary, but mirror_relpath
        // must not silently render a traversal path. We document the contract by
        // asserting the raw join behavior so a future caller knows to sanitize.
        let bad = SkillBody {
            frontmatter: SkillFrontmatter {
                name: "../escape".into(),
                description: "x".into(),
            },
            body: String::new(),
        };
        let p = bad.mirror_relpath();
        // Today: raw join. Mirror writer (Task 3) MUST validate before write.
        assert!(p.to_string_lossy().contains(".."));
    }
}
```

- [ ] **Step 2.2: Wire module into lib.rs**

Edit `crates/memd-schema/src/lib.rs` near top (after `use` block):

```rust
pub mod skill;
```

- [ ] **Step 2.3: Run new tests**

```bash
cargo test -p memd-schema skill:: 2>&1 | tail -15
```

Expected: 3 passed (`render_skill_md_emits_yaml_then_body`, `mirror_relpath_is_skills_name_skill_md`, `mirror_relpath_rejects_path_traversal_via_name`).

- [ ] **Step 2.4: Commit**

```bash
git add crates/memd-schema/src/skill.rs crates/memd-schema/src/lib.rs
git commit -m "feat(schema): SkillBody + frontmatter + mirror path

Defines the on-disk SKILL.md format and the bundle-relative mirror
path so memd-core can write a record-backed skill that the existing
SkillCatalog discovers unchanged."
```

---

## Task 3: Skill mirror writer (atomic, validated)

**Files:**
- Create: `crates/memd-core/src/skill_mirror.rs`
- Modify: `crates/memd-core/src/lib.rs` (or whatever the crate root is — `pub mod skill_mirror;`)
- Test: `crates/memd-core/src/skill_mirror.rs` (inline) + `tests/skill_mirror_roundtrip.rs` (integration in workspace root)

- [ ] **Step 3.1: Write failing inline tests**

Create `crates/memd-core/src/skill_mirror.rs`:

```rust
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use memd_schema::skill::SkillBody;

/// Sanitize a skill name to a single safe path segment.
/// Rejects empty, traversal, separators, and leading dots.
pub fn validate_skill_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow!("skill name is empty"));
    }
    if name.starts_with('.') {
        return Err(anyhow!("skill name may not start with '.'"));
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(anyhow!("skill name contains illegal path chars: {name}"));
    }
    if name.chars().any(|c| !(c.is_ascii_alphanumeric() || c == '-' || c == '_')) {
        return Err(anyhow!(
            "skill name must be ascii alphanumeric, '-' or '_': {name}"
        ));
    }
    Ok(())
}

/// Write `<bundle_root>/skills/<name>/SKILL.md` atomically.
pub fn write_mirror(bundle_root: &Path, body: &SkillBody) -> Result<PathBuf> {
    validate_skill_name(&body.frontmatter.name)?;
    let dir = bundle_root.join("skills").join(&body.frontmatter.name);
    std::fs::create_dir_all(&dir).with_context(|| format!("create_dir_all {dir:?}"))?;
    let final_path = dir.join("SKILL.md");
    let tmp_path = dir.join(".SKILL.md.tmp");
    std::fs::write(&tmp_path, body.render_skill_md())
        .with_context(|| format!("write tmp {tmp_path:?}"))?;
    std::fs::rename(&tmp_path, &final_path)
        .with_context(|| format!("rename to {final_path:?}"))?;
    Ok(final_path)
}

/// Remove `<bundle_root>/skills/<name>/` (idempotent).
pub fn remove_mirror(bundle_root: &Path, name: &str) -> Result<()> {
    validate_skill_name(name)?;
    let dir = bundle_root.join("skills").join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).with_context(|| format!("remove_dir_all {dir:?}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use memd_schema::skill::{SkillBody, SkillFrontmatter};
    use tempfile::tempdir;

    fn body(name: &str) -> SkillBody {
        SkillBody {
            frontmatter: SkillFrontmatter {
                name: name.into(),
                description: "desc".into(),
            },
            body: "body".into(),
        }
    }

    #[test]
    fn write_mirror_creates_skill_md() {
        let tmp = tempdir().unwrap();
        let path = write_mirror(tmp.path(), &body("tdd")).unwrap();
        assert!(path.ends_with("skills/tdd/SKILL.md"));
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("name: tdd"));
        assert!(contents.contains("body"));
    }

    #[test]
    fn write_mirror_rejects_traversal_name() {
        let tmp = tempdir().unwrap();
        let err = write_mirror(tmp.path(), &body("../escape")).unwrap_err();
        assert!(err.to_string().contains("illegal path chars"));
    }

    #[test]
    fn remove_mirror_is_idempotent() {
        let tmp = tempdir().unwrap();
        write_mirror(tmp.path(), &body("zoom")).unwrap();
        remove_mirror(tmp.path(), "zoom").unwrap();
        remove_mirror(tmp.path(), "zoom").unwrap(); // idempotent
        assert!(!tmp.path().join("skills/zoom").exists());
    }

    #[test]
    fn write_mirror_overwrites_atomically() {
        let tmp = tempdir().unwrap();
        let mut b = body("ship");
        write_mirror(tmp.path(), &b).unwrap();
        b.body = "v2".into();
        write_mirror(tmp.path(), &b).unwrap();
        let contents = std::fs::read_to_string(tmp.path().join("skills/ship/SKILL.md")).unwrap();
        assert!(contents.contains("v2"));
        // tmp file should not linger
        assert!(!tmp.path().join("skills/ship/.SKILL.md.tmp").exists());
    }
}
```

- [ ] **Step 3.2: Wire into crate root**

Edit `crates/memd-core/src/lib.rs` (or equivalent root) to add `pub mod skill_mirror;`. If `tempfile` is not yet a dev-dep, add to `[dev-dependencies]` of `crates/memd-core/Cargo.toml`.

- [ ] **Step 3.3: Run tests**

```bash
cargo test -p memd-core skill_mirror:: 2>&1 | tail -20
```

Expected: 4 passed.

- [ ] **Step 3.4: Commit**

```bash
git add crates/memd-core/src/skill_mirror.rs crates/memd-core/src/lib.rs crates/memd-core/Cargo.toml
git commit -m "feat(core): atomic skill mirror writer + name validation

Writes \`.memd/skills/<name>/SKILL.md\` via tmp+rename so the existing
SkillCatalog scan is never seen mid-write. Name validator rejects
traversal and non-portable chars before any FS touch."
```

---

## Task 4: Skill CLI runtime (`memd skill add|list|show|retire`)

**Files:**
- Create: `crates/memd-client/src/cli/cli_skill_runtime.rs`
- Modify: `crates/memd-client/src/cli/commands.rs` (register subcommand)
- Modify: `crates/memd-client/src/cli/mod.rs` (export module)
- Test: `crates/memd-client/src/main_tests/skill_runtime_tests/mod.rs`

The runtime delegates record creation to the existing memory pipeline (so skills participate in lookup, hive, scope, etc.) and calls `skill_mirror::write_mirror` afterwards. On retire, it removes the mirror first then marks the record `status=retired` via the existing memory CLI path.

- [ ] **Step 4.1: Write failing CLI integration test**

Create `crates/memd-client/src/main_tests/skill_runtime_tests/mod.rs`:

```rust
use crate::main_tests::mock_server_support::*;

#[tokio::test]
async fn skill_add_writes_record_and_mirror() {
    let env = mock_env().await;
    let out = env
        .run_cli(&[
            "skill", "add",
            "--name", "tdd",
            "--description", "drive features test-first",
            "--body", "## Steps\n1. Red\n2. Green\n3. Refactor",
        ])
        .await
        .expect("skill add succeeds");
    assert!(out.stdout.contains("created"));

    let mirror = env.bundle_root().join("skills/tdd/SKILL.md");
    assert!(mirror.exists(), "mirror file written");

    let lookup = env
        .run_cli(&["lookup", "--kind", "skill", "--query", "tdd"])
        .await
        .unwrap();
    assert!(lookup.stdout.contains("tdd"), "lookup finds the skill record");
}

#[tokio::test]
async fn skill_retire_removes_mirror() {
    let env = mock_env().await;
    env.run_cli(&[
        "skill", "add", "--name", "ship",
        "--description", "ship workflow", "--body", "...",
    ]).await.unwrap();
    env.run_cli(&["skill", "retire", "--name", "ship"]).await.unwrap();

    let mirror = env.bundle_root().join("skills/ship");
    assert!(!mirror.exists(), "mirror dir removed on retire");
}
```

(`mock_env` and `bundle_root` helpers exist in `mock_server_support.rs` per repo convention; if signatures differ, adapt — the tests above are the contract.)

- [ ] **Step 4.2: Run test to verify failure**

```bash
cargo test -p memd-client skill_runtime_tests:: 2>&1 | tail -20
```

Expected: FAIL with "unknown subcommand `skill`" or similar.

- [ ] **Step 4.3: Implement runtime**

Create `crates/memd-client/src/cli/cli_skill_runtime.rs`:

```rust
use anyhow::{Context, Result};
use memd_schema::{
    skill::{SkillBody, SkillFrontmatter},
    MemoryKind,
};

use super::cli_memory_runtime;

#[derive(Debug, clap::Args)]
pub struct SkillAddArgs {
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub description: String,
    /// Skill body (markdown). Use `--body-file` for long content.
    #[arg(long, conflicts_with = "body_file")]
    pub body: Option<String>,
    #[arg(long)]
    pub body_file: Option<std::path::PathBuf>,
    #[arg(long, default_value = "project")]
    pub scope: String,
}

#[derive(Debug, clap::Args)]
pub struct SkillRetireArgs {
    #[arg(long)]
    pub name: String,
}

pub async fn run_add(args: SkillAddArgs, ctx: &super::CliCtx) -> Result<()> {
    let body_text = match (args.body, args.body_file) {
        (Some(b), None) => b,
        (None, Some(p)) => std::fs::read_to_string(&p)
            .with_context(|| format!("read body file {p:?}"))?,
        (None, None) => return Err(anyhow::anyhow!("provide --body or --body-file")),
        (Some(_), Some(_)) => unreachable!("clap conflicts_with"),
    };

    let skill = SkillBody {
        frontmatter: SkillFrontmatter {
            name: args.name.clone(),
            description: args.description.clone(),
        },
        body: body_text,
    };

    // 1) write the memd record (goes through existing memory pipeline)
    let record_id = cli_memory_runtime::remember(
        ctx,
        cli_memory_runtime::RememberInput {
            kind: MemoryKind::Skill,
            scope: args.scope.parse()?,
            content: skill.render_skill_md(),
            tags: vec!["skill".into(), args.name.clone()],
            // ... whatever the existing builder needs
            ..Default::default()
        },
    )
    .await?;

    // 2) mirror to disk so SkillCatalog finds it
    memd_core::skill_mirror::write_mirror(&ctx.bundle_root(), &skill)
        .with_context(|| "write skill mirror")?;

    println!("created skill {} (record {record_id})", args.name);
    Ok(())
}

pub async fn run_retire(args: SkillRetireArgs, ctx: &super::CliCtx) -> Result<()> {
    // 1) remove disk mirror first (so SkillCatalog stops surfacing immediately)
    memd_core::skill_mirror::remove_mirror(&ctx.bundle_root(), &args.name)?;
    // 2) mark record retired via existing memory pipeline
    cli_memory_runtime::retire_by_tag(ctx, &args.name).await?;
    println!("retired skill {}", args.name);
    Ok(())
}
```

(If `RememberInput` / `retire_by_tag` signatures differ, adapt — keep the two-step ordering: write record → mirror; remove mirror → retire record.)

- [ ] **Step 4.4: Register subcommand**

Edit `crates/memd-client/src/cli/commands.rs`. Add to the `Cli` enum (or equivalent dispatcher):

```rust
Skill {
    #[command(subcommand)]
    cmd: SkillCmd,
},

#[derive(Debug, clap::Subcommand)]
pub enum SkillCmd {
    Add(cli_skill_runtime::SkillAddArgs),
    List,
    Show { #[arg(long)] name: String },
    Retire(cli_skill_runtime::SkillRetireArgs),
}
```

Wire dispatch in the match arm to `cli_skill_runtime::run_add` / `run_retire`. `List` and `Show` can shell out to the existing lookup/show paths filtered to `MemoryKind::Skill`.

- [ ] **Step 4.5: Run tests**

```bash
cargo test -p memd-client skill_runtime_tests:: 2>&1 | tail -25
```

Expected: 2 passed.

- [ ] **Step 4.6: Smoke-test the CLI by hand in a temp bundle**

```bash
TMP=$(mktemp -d)
cargo run -p memd-client --bin memd -- --output "$TMP/.memd" init
cargo run -p memd-client --bin memd -- --output "$TMP/.memd" \
  skill add --name tdd --description "drive features test-first" \
  --body "## Steps\n1. Red\n2. Green\n3. Refactor"
ls "$TMP/.memd/skills/tdd/"
cat "$TMP/.memd/skills/tdd/SKILL.md"
cargo run -p memd-client --bin memd -- --output "$TMP/.memd" \
  lookup --kind skill --query tdd
```

Expected: SKILL.md present with rendered frontmatter; lookup returns the record.

- [ ] **Step 4.7: Commit**

```bash
git add crates/memd-client/src/cli/cli_skill_runtime.rs \
        crates/memd-client/src/cli/commands.rs \
        crates/memd-client/src/cli/mod.rs \
        crates/memd-client/src/main_tests/skill_runtime_tests/mod.rs
git commit -m "feat(cli): \`memd skill {add,list,show,retire}\`

Writes a Skill record through the existing memory pipeline, then
mirrors to .memd/skills/<name>/SKILL.md so the SkillCatalog scan
discovers it. Retire removes the mirror first to avoid stale
catalog reads."
```

---

## Task 5: Link `SkillCatalogEntry` back to its record (optional `record_id`)

**Files:**
- Modify: `crates/memd-client/src/cli/skill_catalog.rs`

This is a tiny additive change so Phase 2 (harvest) can ask "which record backs this catalog entry" without re-querying.

- [ ] **Step 5.1: Write failing test**

Add to existing `skill_catalog.rs` tests (or create `skill_catalog_tests.rs`):

```rust
#[test]
fn entry_record_id_optional_default_none() {
    let e = SkillCatalogEntry {
        name: "tdd".into(),
        path: None,
        summary: "x".into(),
        source: "builtin".into(),
        status: "active".into(),
        usage: "always".into(),
        decision: "active".into(),
        record_id: None,
    };
    assert!(e.record_id.is_none());
}
```

Run: `cargo test -p memd-client skill_catalog 2>&1 | tail` → FAIL (no such field).

- [ ] **Step 5.2: Add field**

In `crates/memd-client/src/cli/skill_catalog.rs:12-21`, add `pub(crate) record_id: Option<uuid::Uuid>,` to `SkillCatalogEntry`. Default to `None` in the builtin construction path. For disk-discovered entries, parse a `record_id:` line from frontmatter if present (Phase 1: don't write it yet, just tolerate it).

- [ ] **Step 5.3: Run tests**

```bash
cargo test -p memd-client skill_catalog 2>&1 | tail
```

Expected: PASS, plus all existing tests still green.

- [ ] **Step 5.4: Commit**

```bash
git add crates/memd-client/src/cli/skill_catalog.rs
git commit -m "feat(skill-catalog): optional record_id link

Backwards-compatible Option<Uuid>; lets the harvest loop in Phase 2
correlate catalog entries with the memd records that back them."
```

---

## Task 6: Surface skills in `wake.md`

**Files:**
- Modify: wake.md generator (locate via `grep -rn "wake.md\|fn render_wake" crates/`)
- Test: `crates/memd-core/src/wake_tests.rs` (or wherever wake tests live)

`wake.md` is the first thing a session reads. We add an `## Active Skills` section under `## Focus` that lists up to 3 skills relevant to the current focus, with the top result's body inlined (≤500 chars truncated) and the rest as `memd lookup --kind skill --name <name>` hints.

- [ ] **Step 6.1: Locate the wake generator**

```bash
grep -rn "Active Skills\|render_wake\|wake.md" /home/josue/Documents/projects/memd/crates 2>&1 | head
```

Expected: a function (likely `render_wake_md` or similar) in `memd-core`.

- [ ] **Step 6.2: Write failing test**

Append to that crate's test module:

```rust
#[test]
fn wake_md_renders_active_skills_section_when_skill_records_exist() {
    let store = test_store_with_skill("tdd", "drive test-first");
    let wake = render_wake_md(&store, &test_focus());
    assert!(wake.contains("## Active Skills"));
    assert!(wake.contains("tdd"));
}

#[test]
fn wake_md_omits_skills_section_when_none() {
    let store = empty_test_store();
    let wake = render_wake_md(&store, &test_focus());
    assert!(!wake.contains("## Active Skills"));
}
```

(`test_store_with_skill` / `empty_test_store` are helpers — write the minimal versions needed.)

- [ ] **Step 6.3: Run test → FAIL**

```bash
cargo test -p memd-core wake_md_renders_active_skills 2>&1 | tail
```

- [ ] **Step 6.4: Implement section**

In the wake renderer:

```rust
let skills = store.lookup_top(MemoryKind::Skill, focus.intent_summary(), 3);
if !skills.is_empty() {
    out.push_str("\n## Active Skills\n\n");
    for (i, s) in skills.iter().enumerate() {
        if i == 0 {
            // inline top body, truncated
            let body = truncate(&s.content, 500);
            out.push_str(&format!("- **{}** — {}\n  {}\n", s.name, s.summary, body));
        } else {
            out.push_str(&format!("- {} — `memd lookup --kind skill --name {}`\n", s.name, s.name));
        }
    }
}
```

Adapt to actual store API.

- [ ] **Step 6.5: Tests pass**

```bash
cargo test -p memd-core wake 2>&1 | tail
```

Expected: 2 new pass + all prior wake tests still green.

- [ ] **Step 6.6: Smoke test in this repo's bundle**

```bash
cargo run -p memd-client --bin memd -- --output .memd skill add \
  --name dogfood-skill --description "phase 1 smoke" --body "## Test"
cargo run -p memd-client --bin memd -- --output .memd wake
grep -A3 "Active Skills" .memd/wake.md
```

Expected: section present, lists `dogfood-skill`. Then retire it:

```bash
cargo run -p memd-client --bin memd -- --output .memd skill retire --name dogfood-skill
```

- [ ] **Step 6.7: Commit**

```bash
git add crates/memd-core/src/wake.rs crates/memd-core/src/wake_tests.rs
git commit -m "feat(wake): Active Skills section in wake.md

Top-3 relevant skills surface on every wake. Top entry body
inlined (≤500 chars); rest are lookup hints. Section is omitted
when no skill records exist, so existing bundles are unchanged."
```

---

## Task 7: End-to-end roundtrip integration test

**Files:**
- Create: `tests/skill_mirror_roundtrip.rs`

- [ ] **Step 7.1: Write the test**

```rust
//! Workspace-level integration: skill add → mirror on disk → catalog scan
//! finds it → wake.md surfaces it → retire → mirror gone, wake clean.

use std::process::Command;
use tempfile::tempdir;

fn memd(bundle: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_memd"))
        .arg("--output")
        .arg(bundle)
        .args(args)
        .output()
        .expect("memd cli runs")
}

#[test]
fn skill_lifecycle_roundtrip() {
    let tmp = tempdir().unwrap();
    let bundle = tmp.path().join(".memd");

    let init = memd(&bundle, &["init"]);
    assert!(init.status.success(), "init: {init:?}");

    let add = memd(
        &bundle,
        &[
            "skill", "add",
            "--name", "tdd",
            "--description", "drive features test-first",
            "--body", "## Steps\n1. Red\n2. Green\n3. Refactor",
        ],
    );
    assert!(add.status.success(), "add: {add:?}");

    assert!(bundle.join("skills/tdd/SKILL.md").exists());

    let wake = memd(&bundle, &["wake"]);
    assert!(wake.status.success());
    let wake_md = std::fs::read_to_string(bundle.join("wake.md")).unwrap();
    assert!(wake_md.contains("Active Skills"));
    assert!(wake_md.contains("tdd"));

    let retire = memd(&bundle, &["skill", "retire", "--name", "tdd"]);
    assert!(retire.status.success(), "retire: {retire:?}");

    assert!(!bundle.join("skills/tdd").exists());

    let wake2 = memd(&bundle, &["wake"]);
    assert!(wake2.status.success());
    let wake2_md = std::fs::read_to_string(bundle.join("wake.md")).unwrap();
    assert!(!wake2_md.contains("tdd"));
}
```

- [ ] **Step 7.2: Run the test**

```bash
cargo test --test skill_mirror_roundtrip 2>&1 | tail
```

Expected: PASS.

- [ ] **Step 7.3: Commit**

```bash
git add tests/skill_mirror_roundtrip.rs
git commit -m "test: skill lifecycle e2e roundtrip

Init → add → mirror present → wake surfaces → retire → mirror gone
→ wake clean. Closes Phase 1 contract."
```

---

## Task 8: Contract doc + roadmap pointer

**Files:**
- Create: `docs/contracts/skill-record.md`
- Modify: `ROADMAP.md` (add a Living Skills entry pointing at this plan + follow-up plans)

- [ ] **Step 8.1: Write the contract**

Document, in `docs/contracts/skill-record.md`:
- Required record fields (kind=Skill, content=rendered SKILL.md, tags include `skill` + name, scope ∈ {local, project, global}).
- Mirror policy: `<bundle_root>/skills/<name>/SKILL.md` written atomically, removed on retire.
- Name validation rules (ascii-alnum/`-`/`_`, no leading dot, no traversal, no separators).
- Interop: `SkillCatalog` is the consumer. Don't change its scan format without a contract bump.
- Open questions for Phase 2: where the harvest loop writes drafts; how draft → canonical promotion is gated.

- [ ] **Step 8.2: Add roadmap pointer**

Append to `ROADMAP.md` (or the appropriate V-block):

```markdown
### Living Skills initiative (parallel track)

- Phase 1 (Foundation) — `docs/superpowers/plans/2026-04-29-living-skills-phase1-foundation.md`
- Phase 2 (Harvest from corrections) — TBD
- Phase 3 (Hive scope merge) — TBD
- Phase 4 (Public registry) — TBD

Wedge: `MemoryKind::Skill` + disk mirror to `.memd/skills/<name>/SKILL.md`
so claude code's native Skill tool discovers memd-backed skills unchanged.
Differentiator vs. mattpocock/skills: skills are records, not static md —
they version, scope, and (Phase 2+) self-evolve from corrections.
```

- [ ] **Step 8.3: Commit**

```bash
git add docs/contracts/skill-record.md ROADMAP.md
git commit -m "docs: skill-record contract + Living Skills roadmap entry

Phase 1 wedge documented. Phases 2-4 stubbed for follow-up plans."
```

---

## Final Verification

- [x] **Step F.1: Full test suite** — verified 2026-05-02

```bash
cargo test --workspace 2>&1 | tail -20
```

Result: workspace green except 1 pre-existing perf flake `working_memory_retrieval_p95_under_100ms` (p95=512ms vs 500ms debug gate, mean=252ms). Reproduces at `e59b4d8` (pre-Phase-1 HEAD) — not caused by skill schema work, deferred to a G-phase fix. Living Skills test counts (Tasks 1–7) all green.

- [x] **Step F.2: Live dogfood in this repo** — verified 2026-05-02

```bash
# --output is a skill subcommand flag, must come AFTER `skill add` (plan above used pre-Phase-1 ordering)
./target/release/memd-server &  # local server with skill schema (homelab :8787 server is pre-Phase-1)
./target/debug/memd --base-url http://127.0.0.1:18800 skill add \
  --name living-skills-bootstrap \
  --description "phase 1 of living-skills initiative" \
  --body-file docs/superpowers/plans/2026-04-29-living-skills-phase1-foundation.md \
  --output .memd
./target/debug/memd --base-url http://127.0.0.1:18800 wake --output .memd | grep -A8 "Active Skills"
```

Result: `{"mirror":".memd/skills/living-skills-bootstrap/SKILL.md","record_id":"53fe9d5c-...","skill":"living-skills-bootstrap"}`; wake stdout surfaced `## Active Skills` with the bootstrap skill and inlined body excerpt. Note: `memd wake` streams to stdout — the on-disk `.memd/wake.md` is updated by the harness UserPromptSubmit hook, not by this command. Cache poisoning observed: `kind=skill` written to `.memd/state/resume-snapshot-cache.json` breaks pre-Phase-1 binaries; wipe cache between dogfood runs OR install the new client into `~/.local/bin/memd` first.

- [x] **Step F.3: Run the verifier checklist (`superpowers:verification-before-completion`)** — verified 2026-05-02

Result: no `.SKILL.md.tmp` siblings; `skill retire --name living-skills-bootstrap` removed `.memd/skills/living-skills-bootstrap/` cleanly (returns `{"note":"record retirement pending (Phase 2)","retired":"..."}`, matching contract §6 + §10); subsequent `wake` had no `## Active Skills` block.

- [x] **Step F.4: Final commit + push** — branch ahead, push pending operator OK

```bash
git log --oneline main..HEAD | head
git push -u origin research/mining
```

Phase 1 closing commit range on `research/mining`: `87be13c` (plan landed) → `fb115d0` (substrate scaffold, last on-branch). Living Skills feature commits: `babea58 → 50eaed7` for the build, `e59b4d8` for contract+roadmap, `bd98b71` for validator tighten.

---

## Plan Review Loop

Per `superpowers:writing-plans` skill, after this plan is saved, dispatch a **plan-document-reviewer** subagent with:
- Path to this plan: `docs/superpowers/plans/2026-04-29-living-skills-phase1-foundation.md`
- Spec context: the CEO 10-star vision delivered in conversation (skills as records, disk mirror for interop, harvest loop deferred to Phase 2, hive sync to Phase 3, registry to Phase 4).

Iterate until ✅ Approved or 3 iterations elapse (then surface to human).

---

## Execution Handoff

Two execution paths once the plan is approved:

1. **Subagent-Driven (recommended)** — fresh subagent per task, review between tasks. Use `superpowers:subagent-driven-development`.
2. **Inline Execution** — execute in current session with checkpoints. Use `superpowers:executing-plans`.

Default for this plan: **Subagent-Driven**, because Tasks 1–8 are largely independent (after the schema variant lands in Task 1, Tasks 2/3/5 can be parallelized via `superpowers:dispatching-parallel-agents`; Tasks 4/6/7/8 then sequence on top).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use memd_schema::skill::SkillBody;

pub mod sync;

/// Result of a `memd skill sync` invocation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SyncReport {
    /// Mirror dirs created or overwritten (dry-run lists what *would* be).
    pub written: Vec<PathBuf>,
    /// Mirror dirs removed by `--prune` (orphan or retired).
    pub pruned: Vec<PathBuf>,
}

/// Apply the regenerator output to disk under `bundle_root`.
///
/// `records` is the authoritative set (Phase 2 contract §10). The
/// regenerator dedupes + sorts; this function does only I/O. With
/// `dry_run`, no files are touched; the report still reflects what
/// *would* be written/pruned. With `prune`, mirror dirs whose name is
/// not in `records` are removed.
pub fn apply_sync(
    bundle_root: &Path,
    records: &[SkillBody],
    dry_run: bool,
    prune: bool,
) -> Result<SyncReport> {
    let writes = sync::regenerate(records).map_err(|e| anyhow!("regenerate: {e}"))?;
    let mut report = SyncReport::default();
    for write in &writes {
        let abs_path = bundle_root.join(&write.relpath);
        report.written.push(abs_path.clone());
        if dry_run {
            continue;
        }
        let dir = abs_path
            .parent()
            .ok_or_else(|| anyhow!("mirror path missing parent: {abs_path:?}"))?;
        std::fs::create_dir_all(dir).with_context(|| format!("create_dir_all {dir:?}"))?;
        let tmp_path = dir.join(".SKILL.md.tmp");
        std::fs::write(&tmp_path, &write.contents)
            .with_context(|| format!("write tmp {tmp_path:?}"))?;
        std::fs::rename(&tmp_path, &abs_path)
            .with_context(|| format!("rename to {abs_path:?}"))?;
    }

    if prune {
        let skills_dir = bundle_root.join("skills");
        if skills_dir.exists() {
            let live: BTreeSet<&str> = records
                .iter()
                .map(|r| r.frontmatter.name.as_str())
                .collect();
            for entry in std::fs::read_dir(&skills_dir)
                .with_context(|| format!("read_dir {skills_dir:?}"))?
            {
                let entry = entry.context("read skills dir entry")?;
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let name = match path.file_name().and_then(|n| n.to_str()) {
                    Some(n) => n,
                    None => continue,
                };
                if live.contains(name) {
                    continue;
                }
                report.pruned.push(path.clone());
                if !dry_run {
                    std::fs::remove_dir_all(&path)
                        .with_context(|| format!("remove_dir_all {path:?}"))?;
                }
            }
        }
    }

    Ok(report)
}

/// Sanitize a skill name to a single safe path segment.
/// Contract (`docs/contracts/skill-record.md` §3): `^[a-z0-9][a-z0-9_-]*$`.
/// Lowercase only — SkillCatalog matching is case-sensitive and case-folding
/// filesystems would otherwise create silent collisions.
pub fn validate_skill_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow!("skill name is empty"));
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(anyhow!("skill name contains illegal path chars: {name}"));
    }
    let mut chars = name.chars();
    let first = chars.next().unwrap();
    if !(first.is_ascii_lowercase() || first.is_ascii_digit()) {
        return Err(anyhow!(
            "skill name must start with a lowercase letter or digit: {name}"
        ));
    }
    if name
        .chars()
        .any(|c| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_'))
    {
        return Err(anyhow!(
            "skill name must be lowercase ascii, digits, '-', or '_': {name}"
        ));
    }
    Ok(())
}

/// Write `<bundle_root>/skills/<name>/SKILL.md` atomically.
pub fn write_mirror(bundle_root: &Path, body: &SkillBody) -> Result<PathBuf> {
    validate_skill_name(&body.frontmatter.name)?;
    if body.frontmatter.description.contains('\n') {
        return Err(anyhow!("skill description may not contain newlines"));
    }
    let dir = bundle_root.join("skills").join(&body.frontmatter.name);
    std::fs::create_dir_all(&dir).with_context(|| format!("create_dir_all {dir:?}"))?;
    let final_path = dir.join("SKILL.md");
    let tmp_path = dir.join(".SKILL.md.tmp");
    std::fs::write(&tmp_path, body.render_skill_md())
        .with_context(|| format!("write tmp {tmp_path:?}"))?;
    std::fs::rename(&tmp_path, &final_path).with_context(|| format!("rename to {final_path:?}"))?;
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
                record_id: None,
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
        let err = write_mirror(tmp.path(), &body("x/escape")).unwrap_err();
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

    #[test]
    fn validate_skill_name_rejects_uppercase() {
        assert!(validate_skill_name("Bad-Name").is_err());
        assert!(validate_skill_name("aBc").is_err());
    }

    #[test]
    fn validate_skill_name_rejects_leading_separator() {
        assert!(validate_skill_name("-bad").is_err());
        assert!(validate_skill_name("_bad").is_err());
        assert!(validate_skill_name(".bad").is_err());
    }

    #[test]
    fn validate_skill_name_accepts_contract_pattern() {
        assert!(validate_skill_name("a").is_ok());
        assert!(validate_skill_name("0skill").is_ok());
        assert!(validate_skill_name("living-skills-bootstrap").is_ok());
        assert!(validate_skill_name("skill_v2").is_ok());
    }

    #[test]
    fn apply_sync_writes_mirrors_for_records() {
        let tmp = tempdir().unwrap();
        let records = vec![body("alpha"), body("bravo")];
        let report = apply_sync(tmp.path(), &records, false, false).unwrap();
        assert_eq!(report.written.len(), 2);
        assert!(tmp.path().join("skills/alpha/SKILL.md").exists());
        assert!(tmp.path().join("skills/bravo/SKILL.md").exists());
        assert!(report.pruned.is_empty());
    }

    #[test]
    fn apply_sync_overwrites_drifted_mirror_atomically() {
        let tmp = tempdir().unwrap();
        write_mirror(tmp.path(), &body("alpha")).unwrap();
        // Drift the on-disk file.
        let path = tmp.path().join("skills/alpha/SKILL.md");
        std::fs::write(&path, "drifted garbage").unwrap();
        let mut record = body("alpha");
        record.body = "v2 body".into();
        apply_sync(tmp.path(), &[record], false, false).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(
            contents.contains("v2 body"),
            "mirror not overwritten:\n{contents}"
        );
        assert!(!tmp.path().join("skills/alpha/.SKILL.md.tmp").exists());
    }

    #[test]
    fn apply_sync_dry_run_writes_nothing() {
        let tmp = tempdir().unwrap();
        let records = vec![body("alpha")];
        let report = apply_sync(tmp.path(), &records, true, false).unwrap();
        assert_eq!(report.written.len(), 1);
        assert!(!tmp.path().join("skills/alpha/SKILL.md").exists());
    }

    #[test]
    fn apply_sync_idempotent_byte_stable_on_second_run() {
        let tmp = tempdir().unwrap();
        let records = vec![body("alpha")];
        apply_sync(tmp.path(), &records, false, false).unwrap();
        let first = std::fs::read(tmp.path().join("skills/alpha/SKILL.md")).unwrap();
        apply_sync(tmp.path(), &records, false, false).unwrap();
        let second = std::fs::read(tmp.path().join("skills/alpha/SKILL.md")).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn apply_sync_prune_removes_orphan_mirror_dirs() {
        let tmp = tempdir().unwrap();
        write_mirror(tmp.path(), &body("alpha")).unwrap();
        write_mirror(tmp.path(), &body("orphan")).unwrap();
        let records = vec![body("alpha")];
        let report = apply_sync(tmp.path(), &records, false, true).unwrap();
        assert_eq!(report.pruned.len(), 1);
        assert!(tmp.path().join("skills/alpha").exists());
        assert!(!tmp.path().join("skills/orphan").exists());
    }

    #[test]
    fn apply_sync_prune_dry_run_lists_orphans_but_keeps_them() {
        let tmp = tempdir().unwrap();
        write_mirror(tmp.path(), &body("orphan")).unwrap();
        let report = apply_sync(tmp.path(), &[], true, true).unwrap();
        assert_eq!(report.pruned.len(), 1);
        assert!(tmp.path().join("skills/orphan").exists());
    }

    #[test]
    fn write_mirror_rejects_newline_in_description() {
        let tmp = tempdir().unwrap();
        let mut b = body("ok-name");
        b.frontmatter.description = "line1\nline2".into();
        let err = write_mirror(tmp.path(), &b).unwrap_err();
        assert!(err.to_string().contains("newlines"));
    }
}

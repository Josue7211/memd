//! Pure sync regenerator (Phase 2 P2.1).
//!
//! Records are the source of truth (Phase 2 contract §10). This module
//! turns a flat list of `SkillBody` records into the set of mirror writes
//! that would reconstruct `.memd/skills/<name>/SKILL.md` from scratch.
//!
//! No I/O. No filesystem. Atomicity, drift overwrite, and idempotence all
//! belong to the CLI wiring in `memd-client/src/cli/cli_skill_sync.rs`
//! (P2.2). This module just answers the question: given these records,
//! what should the mirror look like?

use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;

use memd_schema::skill::SkillBody;

use super::validate_skill_name;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirrorWrite {
    pub relpath: PathBuf,
    pub contents: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncError {
    NameCollision(String),
    InvalidName { name: String, reason: String },
}

impl fmt::Display for SyncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyncError::NameCollision(name) => {
                write!(f, "duplicate skill name across records: {name}")
            }
            SyncError::InvalidName { name, reason } => {
                write!(f, "invalid skill name '{name}': {reason}")
            }
        }
    }
}

impl std::error::Error for SyncError {}

/// Compute the deterministic mirror-write set for a slice of records.
///
/// Sort order is stable (BTreeMap by name) so test assertions and
/// idempotence proofs do not flake on iteration order. The first
/// duplicate name encountered wins the error.
pub fn regenerate(records: &[SkillBody]) -> Result<Vec<MirrorWrite>, SyncError> {
    let mut by_name: BTreeMap<String, &SkillBody> = BTreeMap::new();
    for body in records {
        let name = &body.frontmatter.name;
        validate_skill_name(name).map_err(|e| SyncError::InvalidName {
            name: name.clone(),
            reason: e.to_string(),
        })?;
        if by_name.insert(name.clone(), body).is_some() {
            return Err(SyncError::NameCollision(name.clone()));
        }
    }
    Ok(by_name
        .into_values()
        .map(|body| MirrorWrite {
            relpath: body.mirror_relpath(),
            contents: body.render_skill_md(),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use memd_schema::skill::SkillFrontmatter;

    fn rec(name: &str, desc: &str, body: &str) -> SkillBody {
        SkillBody {
            frontmatter: SkillFrontmatter {
                name: name.into(),
                description: desc.into(),
                record_id: None,
                salience: None,
            },
            body: body.into(),
        }
    }

    #[test]
    fn sync_empty_records_yields_empty_mirror_writes() {
        let writes = regenerate(&[]).unwrap();
        assert!(writes.is_empty());
    }

    #[test]
    fn sync_single_record_produces_one_mirror_write() {
        let writes = regenerate(&[rec("tdd", "drive features test-first", "## Steps")]).unwrap();
        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0].relpath, PathBuf::from("skills/tdd/SKILL.md"));
        assert!(writes[0].contents.starts_with("---\nname: tdd\n"));
        assert!(writes[0].contents.contains("## Steps"));
    }

    #[test]
    fn sync_idempotent_byte_stable_on_second_run() {
        let records = vec![
            rec("alpha", "first", "body-a"),
            rec("bravo", "second", "body-b"),
        ];
        let first = regenerate(&records).unwrap();
        let second = regenerate(&records).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn sync_orders_writes_deterministically_by_name() {
        // Input order shuffled; output sorted by name.
        let records = vec![
            rec("zulu", "z", "z"),
            rec("alpha", "a", "a"),
            rec("mike", "m", "m"),
        ];
        let writes = regenerate(&records).unwrap();
        let names: Vec<_> = writes
            .iter()
            .map(|w| w.relpath.to_string_lossy().to_string())
            .collect();
        assert_eq!(
            names,
            vec![
                "skills/alpha/SKILL.md",
                "skills/mike/SKILL.md",
                "skills/zulu/SKILL.md",
            ]
        );
    }

    #[test]
    fn sync_detects_name_collision_returns_typed_err() {
        let records = vec![
            rec("dup", "first", "body-1"),
            rec("dup", "second", "body-2"),
        ];
        let err = regenerate(&records).unwrap_err();
        assert_eq!(err, SyncError::NameCollision("dup".into()));
    }

    #[test]
    fn sync_rejects_invalid_name_with_typed_err() {
        let records = vec![rec("Bad-Name", "x", "y")];
        let err = regenerate(&records).unwrap_err();
        match err {
            SyncError::InvalidName { name, .. } => assert_eq!(name, "Bad-Name"),
            other => panic!("expected InvalidName, got {other:?}"),
        }
    }
}

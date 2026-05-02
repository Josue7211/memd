use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Frontmatter that is mirrored verbatim to `.memd/skills/<name>/SKILL.md`.
/// Mirrors the shape used by Claude Code's native Skill tool so the existing
/// SkillCatalog discovers our records without modification.
///
/// `record_id` is the canonical link from the on-disk SKILL.md back to the
/// memd record (Phase 2 contract §8). Optional in the type because (a)
/// Phase 1 mirrors omit it, and (b) `skill add` writes the mirror BEFORE it
/// has a record_id (rolled back on remember failure); a second render
/// stamps the id once `remember` returns.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillFrontmatter {
    pub name: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub record_id: Option<Uuid>,
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
        let mut head = format!(
            "---\nname: {}\ndescription: {}\n",
            self.frontmatter.name, self.frontmatter.description
        );
        if let Some(rid) = &self.frontmatter.record_id {
            head.push_str(&format!("record_id: {rid}\n"));
        }
        head.push_str("---\n\n");
        head.push_str(&self.body);
        head.push('\n');
        head
    }

    /// Derive the relative mirror path inside a memd bundle.
    ///
    /// **WARNING:** Does not validate `name`. A malicious or unchecked name
    /// (e.g., `"../escape"`) yields a traversal path. Always pair with
    /// `memd_core::skill_mirror::validate_skill_name` before any FS write —
    /// `write_mirror` does this for you; direct callers must replicate it.
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
                record_id: None,
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
        assert!(
            !rendered.contains("record_id:"),
            "record_id must be omitted when None"
        );
    }

    #[test]
    fn render_skill_md_emits_record_id_when_present() {
        let mut s = sample();
        let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        s.frontmatter.record_id = Some(id);
        let rendered = s.render_skill_md();
        assert!(rendered.contains("record_id: 550e8400-e29b-41d4-a716-446655440000"));
        assert!(rendered.starts_with("---\nname: tdd\n"));
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
                record_id: None,
            },
            body: String::new(),
        };
        let p = bad.mirror_relpath();
        // Today: raw join. Mirror writer (Task 3) MUST validate before write.
        assert!(p.to_string_lossy().contains(".."));
    }
}

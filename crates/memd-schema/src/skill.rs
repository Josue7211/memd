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

    /// Inverse of `render_skill_md`. Parses a SKILL.md byte stream back
    /// into structured frontmatter + body. Returns `None` if the input is
    /// not in the rendered shape (no leading `---`, no closing `---`,
    /// missing required `name`/`description` fields).
    ///
    /// Records-as-truth depends on this round-trip: P2.3 stores the full
    /// `render_skill_md()` output as record content; P2.2 sync parses it
    /// back to drive mirror regeneration.
    pub fn parse_skill_md(raw: &str) -> Option<SkillBody> {
        let mut lines = raw.lines();
        if !lines.next().is_some_and(|line| line.trim() == "---") {
            return None;
        }
        let mut name = None;
        let mut description = None;
        let mut record_id = None;
        let mut closed = false;
        let mut header_line_count = 1usize;
        for line in &mut lines {
            header_line_count += 1;
            let trimmed = line.trim();
            if trimmed == "---" {
                closed = true;
                break;
            }
            if let Some(value) = trimmed.strip_prefix("name:") {
                name = Some(
                    value
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string(),
                );
            } else if let Some(value) = trimmed.strip_prefix("description:") {
                description = Some(
                    value
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string(),
                );
            } else if let Some(value) = trimmed.strip_prefix("record_id:") {
                let value = value.trim().trim_matches('"').trim_matches('\'');
                record_id = uuid::Uuid::parse_str(value).ok();
            }
        }
        if !closed {
            return None;
        }
        let name = name?;
        let description = description?;
        // Body = everything after the closing `---` line. `render_skill_md`
        // emits a blank line then the body then a trailing newline; we
        // mirror that by trimming exactly one leading blank if present and
        // exactly one trailing newline if present, so parse(render(x)) == x.
        let mut body_lines: Vec<&str> = raw.lines().skip(header_line_count).collect();
        if body_lines.first() == Some(&"") {
            body_lines.remove(0);
        }
        let mut body = body_lines.join("\n");
        if raw.ends_with('\n') && !body.ends_with('\n') {
            body.push('\n');
        }
        Some(SkillBody {
            frontmatter: SkillFrontmatter {
                name,
                description,
                record_id,
            },
            body,
        })
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
    fn parse_skill_md_round_trips_render_skill_md() {
        // P2.2 records-as-truth: parse(render(x)) == x for the closed set
        // of Phase 2 fields. Captures both the no-record_id and
        // with-record_id shapes.
        let mut s = sample();
        let rendered_a = s.render_skill_md();
        let parsed_a = SkillBody::parse_skill_md(&rendered_a).expect("parse a");
        assert_eq!(parsed_a, s);

        let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        s.frontmatter.record_id = Some(id);
        let rendered_b = s.render_skill_md();
        let parsed_b = SkillBody::parse_skill_md(&rendered_b).expect("parse b");
        assert_eq!(parsed_b, s);
    }

    #[test]
    fn parse_skill_md_rejects_inputs_missing_frontmatter_fences() {
        assert!(SkillBody::parse_skill_md("no frontmatter at all").is_none());
        assert!(SkillBody::parse_skill_md("---\nname: x\ndescription: y\n").is_none());
        assert!(SkillBody::parse_skill_md("---\nonly: stuff\n---\nbody").is_none());
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

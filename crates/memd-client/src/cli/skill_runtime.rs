use super::*;
use anyhow::Context;
use memd_schema::skill::{SkillBody, SkillFrontmatter};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

pub(crate) async fn run_skill_command(
    client: &MemdClient,
    base_url: &str,
    args: SkillArgs,
) -> anyhow::Result<()> {
    match args.command {
        SkillSubcommand::Add(add_args) => run_skill_add(client, base_url, add_args).await,
        SkillSubcommand::List(list_args) => run_skill_list(list_args),
        SkillSubcommand::Show(show_args) => run_skill_show(show_args),
        SkillSubcommand::Retire(retire_args) => run_skill_retire(retire_args),
    }
}

pub(crate) fn prepare_and_mirror_skill(
    output: &Path,
    name: &str,
    description: &str,
    body_text: String,
) -> anyhow::Result<(SkillBody, PathBuf)> {
    memd_core::skill_mirror::validate_skill_name(name)?;

    let skill_body = SkillBody {
        frontmatter: SkillFrontmatter {
            name: name.to_string(),
            description: description.to_string(),
        },
        body: body_text,
    };

    let mirror_path =
        memd_core::skill_mirror::write_mirror(output, &skill_body).context("write skill mirror")?;

    Ok((skill_body, mirror_path))
}

async fn run_skill_add(
    client: &MemdClient,
    base_url: &str,
    args: SkillAddArgs,
) -> anyhow::Result<()> {
    let body_text = if let Some(body) = &args.body {
        body.clone()
    } else if let Some(path) = &args.body_file {
        fs::read_to_string(path).with_context(|| format!("read body file {}", path.display()))?
    } else if args.stdin {
        let mut buffer = String::new();
        std::io::stdin()
            .read_to_string(&mut buffer)
            .context("read skill body from stdin")?;
        buffer
    } else {
        anyhow::bail!("provide --body, --body-file, or --stdin");
    };

    let (skill_body, mirror_path) =
        prepare_and_mirror_skill(&args.output, &args.name, &args.description, body_text)?;

    let mut remember_args = RememberArgs {
        output: args.output.clone(),
        project: None,
        namespace: None,
        workspace: None,
        visibility: None,
        kind: Some("skill".to_string()),
        scope: args.scope.clone().or_else(|| Some("project".to_string())),
        source_agent: None,
        source_system: None,
        source_path: None,
        source_quality: None,
        confidence: None,
        ttl_seconds: None,
        tag: args.tag,
        supersede: vec![],
        content: Some(skill_body.body.clone()),
        input: None,
        stdin: false,
    };

    remember_args.tag.push(format!("skill:{}", &args.name));

    let response = remember_with_bundle_defaults(&remember_args, base_url)
        .await
        .map_err(|e| match memd_core::skill_mirror::remove_mirror(&args.output, &args.name) {
            Ok(()) => e,
            Err(rollback_err) => anyhow::anyhow!(
                "remember failed and rollback also failed (mirror left at {}/skills/{}/SKILL.md): original={e}; rollback={rollback_err}",
                args.output.display(),
                args.name
            ),
        })?;

    let record_id = response.item.id.clone();
    let payload = serde_json::json!({
        "skill": args.name,
        "mirror": mirror_path.display().to_string(),
        "record_id": record_id,
    });
    println!("{}", serde_json::to_string(&payload)?);

    Ok(())
}

fn run_skill_list(args: SkillListArgs) -> anyhow::Result<()> {
    let skills_dir = args.output.join("skills");
    let mut skills = vec![];

    if skills_dir.exists() {
        for entry in fs::read_dir(&skills_dir).context("read skills directory")? {
            let entry = entry.context("read dir entry")?;
            let path = entry.path();
            if path.is_dir() {
                let skill_md = path.join("SKILL.md");
                if skill_md.exists() {
                    let content = fs::read_to_string(&skill_md)
                        .with_context(|| format!("read {}", skill_md.display()))?;

                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        let description = extract_description_from_skill_md(&content);
                        skills.push((name.to_string(), description));
                    }
                }
            }
        }
    }

    if args.json {
        let json_items: Vec<_> = skills
            .iter()
            .map(|(name, desc)| serde_json::json!({ "name": name, "description": desc }))
            .collect();
        print_json(&json_items)?;
    } else {
        for (name, description) in skills {
            println!("{}  {}", name, description);
        }
    }

    Ok(())
}

fn run_skill_show(args: SkillShowArgs) -> anyhow::Result<()> {
    memd_core::skill_mirror::validate_skill_name(&args.name)?;

    let skill_md = args.output.join("skills").join(&args.name).join("SKILL.md");
    let content =
        fs::read_to_string(&skill_md).with_context(|| format!("read skill {}", args.name))?;

    println!("{}", content);
    Ok(())
}

fn run_skill_retire(args: SkillRetireArgs) -> anyhow::Result<()> {
    memd_core::skill_mirror::remove_mirror(&args.output, &args.name)
        .context("remove skill mirror")?;

    if !args.keep_record {
        let payload = serde_json::json!({
            "retired": args.name,
            "note": "record retirement pending (Phase 2)",
        });
        println!("{}", serde_json::to_string(&payload)?);
    } else {
        let payload = serde_json::json!({
            "retired": args.name,
            "mirror_deleted": true,
        });
        println!("{}", serde_json::to_string(&payload)?);
    }

    Ok(())
}

fn extract_description_from_skill_md(content: &str) -> String {
    for line in content.lines() {
        if line.starts_with("description: ") {
            return line.strip_prefix("description: ").unwrap_or("").to_string();
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use memd_schema::skill::{SkillBody, SkillFrontmatter};
    use tempfile::tempdir;

    fn sample_skill_body(name: &str) -> SkillBody {
        SkillBody {
            frontmatter: SkillFrontmatter {
                name: name.into(),
                description: "test skill".into(),
            },
            body: "## Test Body\nSome content".into(),
        }
    }

    #[test]
    fn list_finds_skills() {
        let tmp = tempdir().unwrap();
        memd_core::skill_mirror::write_mirror(tmp.path(), &sample_skill_body("skill1")).unwrap();
        memd_core::skill_mirror::write_mirror(tmp.path(), &sample_skill_body("skill2")).unwrap();

        let args = SkillListArgs {
            output: tmp.path().to_path_buf(),
            json: false,
        };

        assert!(run_skill_list(args).is_ok());
    }

    #[test]
    fn show_reads_skill_md() {
        let tmp = tempdir().unwrap();
        memd_core::skill_mirror::write_mirror(tmp.path(), &sample_skill_body("demo")).unwrap();

        let args = SkillShowArgs {
            name: "demo".to_string(),
            output: tmp.path().to_path_buf(),
        };

        assert!(run_skill_show(args).is_ok());
    }

    #[test]
    fn show_rejects_invalid_name() {
        let tmp = tempdir().unwrap();
        let args = SkillShowArgs {
            name: "../escape".to_string(),
            output: tmp.path().to_path_buf(),
        };

        assert!(run_skill_show(args).is_err());
    }

    #[test]
    fn retire_removes_mirror() {
        let tmp = tempdir().unwrap();
        memd_core::skill_mirror::write_mirror(tmp.path(), &sample_skill_body("demo")).unwrap();

        let args = SkillRetireArgs {
            name: "demo".to_string(),
            output: tmp.path().to_path_buf(),
            keep_record: true,
        };

        assert!(run_skill_retire(args).is_ok());
        assert!(!tmp.path().join("skills/demo").exists());
    }

    #[test]
    fn retire_is_idempotent() {
        let tmp = tempdir().unwrap();
        memd_core::skill_mirror::write_mirror(tmp.path(), &sample_skill_body("demo")).unwrap();

        let args = SkillRetireArgs {
            name: "demo".to_string(),
            output: tmp.path().to_path_buf(),
            keep_record: true,
        };

        assert!(run_skill_retire(args.clone()).is_ok());
        assert!(run_skill_retire(args).is_ok());
    }

    #[test]
    fn prepare_and_mirror_skill_writes_skill_md() {
        let tmp = tempdir().unwrap();
        let (body, mirror_path) = prepare_and_mirror_skill(
            tmp.path(),
            "demo",
            "demo skill",
            "## Body\nhello\n".to_string(),
        )
        .unwrap();

        let written = std::fs::read_to_string(&mirror_path).unwrap();
        assert!(written.starts_with("---\nname: demo\n"));
        assert!(written.contains("description: demo skill"));
        assert!(written.contains("## Body\nhello"));
        assert_eq!(body.frontmatter.name, "demo");
        assert_eq!(mirror_path, tmp.path().join("skills/demo/SKILL.md"));
    }

    #[test]
    fn prepare_and_mirror_skill_rejects_invalid_name_without_writing() {
        let tmp = tempdir().unwrap();
        let result = prepare_and_mirror_skill(tmp.path(), "../escape", "x", String::new());
        assert!(result.is_err());
        let skills = tmp.path().join("skills");
        if skills.exists() {
            let count = std::fs::read_dir(&skills).unwrap().count();
            assert_eq!(count, 0, "no skill dir should be created on invalid name");
        }
    }
}

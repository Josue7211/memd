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
        SkillSubcommand::Retire(retire_args) => run_skill_retire(client, retire_args).await,
        SkillSubcommand::Sync(sync_args) => run_skill_sync(client, sync_args).await,
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
            record_id: None,
            salience: None,
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

    let (mut skill_body, mirror_path) =
        prepare_and_mirror_skill(&args.output, &args.name, &args.description, body_text)?;

    // Records-as-truth (Phase 2 contract §10): the record content holds the
    // full rendered SKILL.md so `memd skill sync` can regenerate the mirror
    // without consulting external state.
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
        content: Some(skill_body.render_skill_md()),
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

    let record_id = response.item.id;

    // Re-stamp mirror with record_id now that the record exists. If this
    // fails we keep the partial state (record + mirror without record_id);
    // parse_skill_metadata tolerates absent record_id and `memd skill sync`
    // will repair the frontmatter on next run.
    skill_body.frontmatter.record_id = Some(record_id);
    let _ = memd_core::skill_mirror::write_mirror(&args.output, &skill_body);

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

async fn run_skill_retire(client: &MemdClient, args: SkillRetireArgs) -> anyhow::Result<()> {
    memd_core::skill_mirror::validate_skill_name(&args.name)?;

    // Phase 2 contract §3: default `retire` deletes the record
    // (operationally — via Expire(status=Expired); no DeleteMemory exists).
    // `--keep-record` leaves it Active so operators can preserve history.
    // record_id comes from the SKILL.md frontmatter we stamp in `skill add`.
    let skill_md = args.output.join("skills").join(&args.name).join("SKILL.md");
    let record_id = if skill_md.exists() {
        let raw = fs::read_to_string(&skill_md)
            .with_context(|| format!("read skill {}", skill_md.display()))?;
        parse_record_id_from_skill_md(&raw)
    } else {
        None
    };

    let mut expired_record = false;
    if !args.keep_record {
        if let Some(id) = record_id {
            client
                .expire(&memd_schema::ExpireMemoryRequest {
                    id,
                    status: Some(memd_schema::MemoryStatus::Expired),
                })
                .await
                .with_context(|| format!("expire skill record {id}"))?;
            expired_record = true;
        }
    }

    memd_core::skill_mirror::remove_mirror(&args.output, &args.name)
        .context("remove skill mirror")?;

    let payload = serde_json::json!({
        "retired": args.name,
        "mirror_deleted": true,
        "record_expired": expired_record,
        "kept_record": args.keep_record,
    });
    println!("{}", serde_json::to_string(&payload)?);

    Ok(())
}

async fn run_skill_sync(client: &MemdClient, args: SkillSyncArgs) -> anyhow::Result<()> {
    // Records-as-truth: pull every active Skill record across visible
    // scopes and reconstruct the mirror from them. Deliberately no
    // text query — kind+status filter is the whole filter set.
    let req = memd_schema::SearchMemoryRequest {
        query: None,
        route: Some(memd_schema::RetrievalRoute::All),
        intent: Some(memd_schema::RetrievalIntent::General),
        scopes: vec![
            memd_schema::MemoryScope::Project,
            memd_schema::MemoryScope::Synced,
            memd_schema::MemoryScope::Global,
        ],
        kinds: vec![memd_schema::MemoryKind::Skill],
        statuses: vec![memd_schema::MemoryStatus::Active],
        project: None,
        namespace: None,
        workspace: None,
        visibility: None,
        belief_branch: None,
        source_agent: None,
        region: None,
        tags: vec![],
        stages: vec![
            memd_schema::MemoryStage::Canonical,
            memd_schema::MemoryStage::Candidate,
        ],
        limit: None,
        max_chars_per_item: None,
    };
    let response = client.search(&req).await.context("search skill records")?;

    let mut records: Vec<memd_schema::skill::SkillBody> = Vec::new();
    let mut skipped: Vec<uuid::Uuid> = Vec::new();
    for item in &response.items {
        match memd_schema::skill::SkillBody::parse_skill_md(&item.content) {
            Some(body) => records.push(body),
            None => skipped.push(item.id),
        }
    }

    let report =
        memd_core::skill_mirror::apply_sync(&args.output, &records, args.dry_run, args.prune)?;

    let payload = serde_json::json!({
        "dry_run": args.dry_run,
        "prune": args.prune,
        "records_seen": response.items.len(),
        "records_parsed": records.len(),
        "records_skipped": skipped.len(),
        "skipped_ids": skipped.iter().map(|id| id.to_string()).collect::<Vec<_>>(),
        "written": report.written.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
        "pruned": report.pruned.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
    });
    println!("{}", serde_json::to_string(&payload)?);

    Ok(())
}

fn parse_record_id_from_skill_md(raw: &str) -> Option<uuid::Uuid> {
    let mut lines = raw.lines();
    if !lines.next().is_some_and(|line| line.trim() == "---") {
        return None;
    }
    for line in lines {
        let trimmed = line.trim();
        if trimmed == "---" {
            break;
        }
        if let Some(value) = trimmed.strip_prefix("record_id:") {
            let value = value.trim().trim_matches('"').trim_matches('\'');
            if let Ok(parsed) = uuid::Uuid::parse_str(value) {
                return Some(parsed);
            }
        }
    }
    None
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
                record_id: None,
                salience: None,
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

    fn dummy_client() -> MemdClient {
        // Used by tests that exercise --keep-record (no HTTP call) and the
        // record_id-less path (no HTTP call). Any non-connecting URL works.
        MemdClient::new("http://127.0.0.1:1").expect("dummy client")
    }

    #[tokio::test]
    async fn retire_removes_mirror() {
        let tmp = tempdir().unwrap();
        memd_core::skill_mirror::write_mirror(tmp.path(), &sample_skill_body("demo")).unwrap();

        let args = SkillRetireArgs {
            name: "demo".to_string(),
            output: tmp.path().to_path_buf(),
            keep_record: true,
        };

        assert!(run_skill_retire(&dummy_client(), args).await.is_ok());
        assert!(!tmp.path().join("skills/demo").exists());
    }

    #[tokio::test]
    async fn retire_is_idempotent() {
        let tmp = tempdir().unwrap();
        memd_core::skill_mirror::write_mirror(tmp.path(), &sample_skill_body("demo")).unwrap();

        let args = SkillRetireArgs {
            name: "demo".to_string(),
            output: tmp.path().to_path_buf(),
            keep_record: true,
        };

        let client = dummy_client();
        assert!(run_skill_retire(&client, args.clone()).await.is_ok());
        assert!(run_skill_retire(&client, args).await.is_ok());
    }

    #[tokio::test]
    async fn skill_retire_default_deletes_record_and_mirror() {
        // P2.4 test 8: default retire (no --keep-record) calls Expire on the
        // record and removes the mirror. Asserts the wiring: state.expired
        // captures one ExpireMemoryRequest with our record_id and Expired status.
        use crate::main_tests::{MockRuntimeState, spawn_mock_runtime_server};
        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state.clone(), false).await;
        let client = MemdClient::new(&base_url).expect("client");

        let tmp = tempdir().unwrap();
        let id = uuid::Uuid::parse_str("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee").unwrap();
        let mut body = sample_skill_body("demo");
        body.frontmatter.record_id = Some(id);
        memd_core::skill_mirror::write_mirror(tmp.path(), &body).unwrap();

        let args = SkillRetireArgs {
            name: "demo".to_string(),
            output: tmp.path().to_path_buf(),
            keep_record: false,
        };
        run_skill_retire(&client, args).await.expect("retire ok");

        assert!(!tmp.path().join("skills/demo").exists(), "mirror removed");
        let expired = state.expired.lock().unwrap();
        assert_eq!(expired.len(), 1, "one expire call");
        assert_eq!(expired[0].id, id);
        assert_eq!(expired[0].status, Some(memd_schema::MemoryStatus::Expired));
    }

    #[tokio::test]
    async fn skill_retire_keep_record_preserves_record_status_retired() {
        // P2.4 test 9: --keep-record removes mirror but leaves record alone
        // (no Expire call). "retired" in the test name is the operation, not
        // a MemoryStatus variant — that variant doesn't exist.
        use crate::main_tests::{MockRuntimeState, spawn_mock_runtime_server};
        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state.clone(), false).await;
        let client = MemdClient::new(&base_url).expect("client");

        let tmp = tempdir().unwrap();
        let id = uuid::Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
        let mut body = sample_skill_body("demo");
        body.frontmatter.record_id = Some(id);
        memd_core::skill_mirror::write_mirror(tmp.path(), &body).unwrap();

        let args = SkillRetireArgs {
            name: "demo".to_string(),
            output: tmp.path().to_path_buf(),
            keep_record: true,
        };
        run_skill_retire(&client, args).await.expect("retire ok");

        assert!(!tmp.path().join("skills/demo").exists(), "mirror removed");
        let expired = state.expired.lock().unwrap();
        assert!(
            expired.is_empty(),
            "--keep-record must not call expire, got {expired:?}"
        );
    }

    fn skill_record_item(name: &str, body: &str) -> memd_schema::MemoryItem {
        let s = SkillBody {
            frontmatter: SkillFrontmatter {
                name: name.into(),
                description: format!("desc for {name}"),
                record_id: Some(uuid::Uuid::new_v4()),
                salience: None,
            },
            body: body.into(),
        };
        memd_schema::MemoryItem {
            id: s.frontmatter.record_id.unwrap(),
            content: s.render_skill_md(),
            redundancy_key: Some(format!("Skill|Project|memd|main||{name}")),
            belief_branch: None,
            preferred: false,
            kind: memd_schema::MemoryKind::Skill,
            scope: memd_schema::MemoryScope::Project,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            visibility: memd_schema::MemoryVisibility::Private,
            source_agent: None,
            source_system: None,
            source_path: None,
            source_quality: None,
            confidence: 0.9,
            ttl_seconds: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_verified_at: None,
            supersedes: vec![],
            tags: vec![format!("skill:{name}")],
            status: memd_schema::MemoryStatus::Active,
            stage: memd_schema::MemoryStage::Canonical,
            lane: None,
            version: 1,
            correction_meta: None,
        }
    }

    #[tokio::test]
    async fn skill_sync_writes_mirrors_for_records_returned_by_search() {
        // P2.2 wiring: search returns records, run_skill_sync parses them,
        // apply_sync writes mirrors. Asserts both mirror files exist with
        // record_id stamped (round-tripped through render → parse → render).
        use crate::main_tests::{MockRuntimeState, spawn_mock_runtime_server};
        let state = MockRuntimeState::default();
        state.injected_skill_records.lock().unwrap().extend(vec![
            skill_record_item("alpha", "## Alpha\n"),
            skill_record_item("bravo", "## Bravo\n"),
        ]);
        let base_url = spawn_mock_runtime_server(state.clone(), false).await;
        let client = MemdClient::new(&base_url).expect("client");

        let tmp = tempdir().unwrap();
        let args = SkillSyncArgs {
            output: tmp.path().to_path_buf(),
            dry_run: false,
            prune: false,
        };
        run_skill_sync(&client, args).await.expect("sync ok");

        for name in ["alpha", "bravo"] {
            let p = tmp.path().join(format!("skills/{name}/SKILL.md"));
            assert!(p.exists(), "missing {p:?}");
            let raw = std::fs::read_to_string(&p).unwrap();
            assert!(raw.contains(&format!("name: {name}")));
            assert!(raw.contains("record_id:"));
        }
    }

    #[tokio::test]
    async fn skill_sync_dry_run_writes_nothing() {
        use crate::main_tests::{MockRuntimeState, spawn_mock_runtime_server};
        let state = MockRuntimeState::default();
        state
            .injected_skill_records
            .lock()
            .unwrap()
            .push(skill_record_item("solo", "## Body\n"));
        let base_url = spawn_mock_runtime_server(state.clone(), false).await;
        let client = MemdClient::new(&base_url).expect("client");

        let tmp = tempdir().unwrap();
        let args = SkillSyncArgs {
            output: tmp.path().to_path_buf(),
            dry_run: true,
            prune: false,
        };
        run_skill_sync(&client, args).await.expect("sync ok");
        assert!(!tmp.path().join("skills/solo/SKILL.md").exists());
    }

    #[tokio::test]
    async fn skill_sync_prune_removes_orphan_mirror() {
        // E2E shape of test 16: mirror has an orphan dir whose record was
        // retired. Sync --prune removes it; live records stay.
        use crate::main_tests::{MockRuntimeState, spawn_mock_runtime_server};
        let state = MockRuntimeState::default();
        state
            .injected_skill_records
            .lock()
            .unwrap()
            .push(skill_record_item("kept", "## Keep\n"));
        let base_url = spawn_mock_runtime_server(state.clone(), false).await;
        let client = MemdClient::new(&base_url).expect("client");

        let tmp = tempdir().unwrap();
        // Pre-existing orphan mirror with no matching record.
        memd_core::skill_mirror::write_mirror(tmp.path(), &sample_skill_body("orphan")).unwrap();

        let args = SkillSyncArgs {
            output: tmp.path().to_path_buf(),
            dry_run: false,
            prune: true,
        };
        run_skill_sync(&client, args).await.expect("sync ok");

        assert!(tmp.path().join("skills/kept/SKILL.md").exists(), "kept");
        assert!(!tmp.path().join("skills/orphan").exists(), "orphan pruned");
    }

    #[tokio::test]
    async fn skill_retire_keep_record_still_removes_mirror_file() {
        // P2.4 test 10: --keep-record always removes the mirror file even
        // when the record stays. Mirror is the visible surface; retiring
        // means the harness should stop seeing it.
        let tmp = tempdir().unwrap();
        memd_core::skill_mirror::write_mirror(tmp.path(), &sample_skill_body("demo")).unwrap();
        let mirror = tmp.path().join("skills/demo/SKILL.md");
        assert!(mirror.exists(), "precondition");

        let args = SkillRetireArgs {
            name: "demo".to_string(),
            output: tmp.path().to_path_buf(),
            keep_record: true,
        };
        run_skill_retire(&dummy_client(), args)
            .await
            .expect("retire ok");

        assert!(!mirror.exists(), "SKILL.md removed");
        assert!(
            !tmp.path().join("skills/demo").exists(),
            "skill dir removed"
        );
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
    fn skill_body_with_record_id_writes_record_id_into_frontmatter() {
        // P2.3 test 6: simulates the post-remember re-stamp step.
        let tmp = tempdir().unwrap();
        let id = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let mut body = sample_skill_body("demo");
        body.frontmatter.record_id = Some(id);
        memd_core::skill_mirror::write_mirror(tmp.path(), &body).unwrap();

        let written = std::fs::read_to_string(tmp.path().join("skills/demo/SKILL.md")).unwrap();
        assert!(
            written.contains("record_id: 550e8400-e29b-41d4-a716-446655440000"),
            "frontmatter must include record_id, got:\n{written}"
        );
    }

    #[test]
    fn skill_body_record_id_round_trips_via_load_skill_catalog() {
        // P2.3 test 7: written record_id is recoverable through the public
        // skill catalog load path (parse_skill_metadata under the hood).
        use crate::cli::skill_catalog::build_skill_catalog;
        let tmp = tempdir().unwrap();
        let id = uuid::Uuid::parse_str("11111111-2222-3333-4444-555555555555").unwrap();
        let mut body = sample_skill_body("round-trip");
        body.frontmatter.record_id = Some(id);
        memd_core::skill_mirror::write_mirror(tmp.path(), &body).unwrap();

        let catalog = build_skill_catalog(&tmp.path().join("skills")).unwrap();
        let entry = catalog
            .custom
            .iter()
            .find(|e| e.name == "round-trip")
            .expect("entry must exist");
        assert_eq!(entry.record_id, Some(id));
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

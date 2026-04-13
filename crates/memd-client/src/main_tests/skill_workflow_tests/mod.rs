use super::*;

#[test]
fn write_skill_policy_artifacts_writes_batch_and_activate_queue() {
    let dir = std::env::temp_dir().join(format!(
        "memd-skill-policy-artifacts-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("create temp bundle");

    let report = SkillLifecycleReport {
        generated_at: Utc::now(),
        proposed: 2,
        sandbox_passed: 1,
        sandbox_review: 1,
        sandbox_blocked: 0,
        activation_candidates: 1,
        activated: 1,
        review_queue: vec![SkillLifecycleRecord {
            harness: "claude".to_string(),
            name: "review-skill".to_string(),
            kind: "skill".to_string(),
            portability_class: "harness-native".to_string(),
            proposal: "proposed".to_string(),
            sandbox: "review".to_string(),
            sandbox_risk: 0.38,
            sandbox_reason: "harness_native".to_string(),
            activation: "review".to_string(),
            activation_reason: "policy gate requires review before activation".to_string(),
            source_path: "/tmp/review-skill".to_string(),
            target_path: None,
            notes: vec!["note".to_string()],
        }],
        activate_queue: vec![SkillLifecycleRecord {
            harness: "codex".to_string(),
            name: "activate-skill".to_string(),
            kind: "skill".to_string(),
            portability_class: "universal".to_string(),
            proposal: "proposed".to_string(),
            sandbox: "pass".to_string(),
            sandbox_risk: 0.05,
            sandbox_reason: "portable".to_string(),
            activation: "activate".to_string(),
            activation_reason: "low-risk sandbox passed and policy allowed auto-activation"
                .to_string(),
            source_path: "/tmp/activate-skill".to_string(),
            target_path: Some("/tmp/target".to_string()),
            notes: vec![],
        }],
        records: vec![],
    };
    let response = MemoryPolicyResponse {
        retrieval_order: Vec::new(),
        route_defaults: vec![memd_schema::MemoryPolicyRouteDefault {
            intent: RetrievalIntent::CurrentTask,
            route: RetrievalRoute::Auto,
        }],
        working_memory: memd_schema::MemoryPolicyWorkingMemory {
            budget_chars: 1,
            max_chars_per_item: 1,
            default_limit: 1,
            rehydration_limit: 1,
        },
        retrieval_feedback: memd_schema::MemoryPolicyFeedback {
            enabled: false,
            tracked_surfaces: Vec::new(),
            max_items_per_request: 1,
        },
        source_trust_floor: 0.0,
        runtime: Default::default(),
        promotion: memd_schema::MemoryPolicyPromotion {
            min_salience: 0.0,
            min_events: 0,
            lookback_days: 0,
            default_ttl_days: 0,
        },
        decay: memd_schema::MemoryPolicyDecay {
            max_items: 0,
            inactive_days: 0,
            max_decay: 0.0,
            record_events: false,
        },
        consolidation: memd_schema::MemoryPolicyConsolidation {
            max_groups: 0,
            min_events: 0,
            lookback_days: 0,
            min_salience: 0.0,
            record_events: false,
        },
    };

    let receipt = write_skill_policy_artifacts(&dir, &response, &report, true)
        .expect("write skill policy artifacts");
    assert!(receipt.is_some());

    let batch_json = dir.join("state").join("skill-policy-batch.json");
    let batch_md = dir.join("state").join("skill-policy-batch.md");
    let review_json = dir.join("state").join("skill-policy-review-queue.json");
    let review_md = dir.join("state").join("skill-policy-review-queue.md");
    let activate_json = dir.join("state").join("skill-policy-activate-queue.json");
    let activate_md = dir.join("state").join("skill-policy-activate-queue.md");
    let apply_json = dir.join("state").join("skill-policy-apply-receipt.json");
    let apply_md = dir.join("state").join("skill-policy-apply-receipt.md");
    assert!(batch_json.exists());
    assert!(batch_md.exists());
    assert!(review_json.exists());
    assert!(review_md.exists());
    assert!(activate_json.exists());
    assert!(activate_md.exists());
    assert!(apply_json.exists());
    assert!(apply_md.exists());

    let parsed: SkillPolicyBatchArtifact =
        serde_json::from_str(&fs::read_to_string(&batch_json).expect("read batch json"))
            .expect("parse batch json");
    assert_eq!(parsed.report.proposed, 2);
    assert_eq!(parsed.report.activate_queue.len(), 1);
    assert_eq!(parsed.report.review_queue.len(), 1);

    let activate: SkillPolicyQueueArtifact =
        serde_json::from_str(&fs::read_to_string(&activate_json).expect("read activate json"))
            .expect("parse activate json");
    assert_eq!(activate.queue, "activate");
    assert_eq!(activate.records.len(), 1);

    let apply: SkillPolicyApplyArtifact =
        serde_json::from_str(&fs::read_to_string(&apply_json).expect("read apply json"))
            .expect("parse apply json");
    assert_eq!(apply.applied_count, 1);
    assert_eq!(apply.skipped_count, 0);
    assert_eq!(apply.applied.len(), 1);

    fs::remove_dir_all(dir).expect("cleanup temp bundle");
}

#[tokio::test]
async fn skill_policy_apply_posts_to_server_sink() {
    let base_url = spawn_mock_hive_server().await;
    let client = MemdClient::new(&base_url).expect("build client");
    let response = client
        .record_skill_policy_apply(&SkillPolicyApplyRequest {
            bundle_root: ".memd".to_string(),
            runtime_defaulted: false,
            source_queue_path: ".memd/state/skill-policy-activate-queue.json".to_string(),
            applied_count: 2,
            skipped_count: 1,
            applied: vec![SkillPolicyActivationRecord {
                harness: "codex".to_string(),
                name: "skill-a".to_string(),
                kind: "skill".to_string(),
                portability_class: "universal".to_string(),
                proposal: "proposed".to_string(),
                sandbox: "pass".to_string(),
                sandbox_risk: 0.05,
                sandbox_reason: "portable".to_string(),
                activation: "activate".to_string(),
                activation_reason: "low-risk sandbox passed and policy allowed auto-activation"
                    .to_string(),
                source_path: "skill-a".to_string(),
                target_path: Some("target-a".to_string()),
                notes: vec!["note".to_string()],
            }],
            skipped: vec![],
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
        })
        .await
        .expect("post apply receipt");
    assert_eq!(response.receipt.applied_count, 2);

    let receipts = client
        .skill_policy_apply_receipts(&SkillPolicyApplyReceiptsRequest {
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            limit: Some(8),
        })
        .await
        .expect("fetch apply receipts");
    assert_eq!(receipts.receipts.len(), 1);
    assert_eq!(receipts.receipts[0].applied_count, 2);

    let activations = client
        .skill_policy_activations(&SkillPolicyActivationEntriesRequest {
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            limit: Some(8),
        })
        .await
        .expect("fetch skill policy activations");
    assert_eq!(activations.activations.len(), 1);
    assert_eq!(activations.activations[0].record.name, "skill-a");
}

#[test]
fn skill_catalog_discovers_project_skill_files() {
    let root = std::env::temp_dir().join(format!("memd-skills-{}", uuid::Uuid::new_v4()));
    let skill_root = root.join("skills").join("project-chat");
    fs::create_dir_all(&skill_root).expect("create skill root");
    fs::write(
        skill_root.join("SKILL.md"),
        r#"---
name: project-chat
description: route project chat through a compact visible lane
---

# Project Chat

Use this skill when the user asks for chat with project memory.
"#,
    )
    .expect("write skill file");

    let catalog = build_skill_catalog(&root.join("skills")).expect("build skill catalog");
    assert!(!catalog.builtins.is_empty());
    assert_eq!(catalog.custom.len(), 1);
    assert_eq!(catalog.custom[0].name, "project-chat");
    assert_eq!(
        catalog.custom[0].summary,
        "route project chat through a compact visible lane"
    );
    assert_eq!(
        catalog.custom[0].usage,
        "edit the file, then propose via skill-policy"
    );
    assert_eq!(
        catalog.custom[0].decision,
        "custom skills stay project-local until promoted by policy"
    );

    fs::remove_dir_all(root).expect("cleanup skill temp dir");
}

#[test]
fn skill_catalog_matches_builtin_and_custom_entries() {
    let root = std::env::temp_dir().join(format!("memd-skills-match-{}", uuid::Uuid::new_v4()));
    let skill_root = root.join("skills").join("project-chat");
    fs::create_dir_all(&skill_root).expect("create skill root");
    fs::write(
        skill_root.join("SKILL.md"),
        r#"---
name: project-chat
description: route project chat through a compact visible lane
---
"#,
    )
    .expect("write skill file");

    let catalog = build_skill_catalog(&root.join("skills")).expect("build skill catalog");
    let builtin_matches = find_skill_catalog_matches(&catalog, "memd init");
    assert!(
        builtin_matches
            .iter()
            .any(|entry| entry.name == "memd-init")
    );

    let custom_matches = find_skill_catalog_matches(&catalog, "compact visible lane");
    assert_eq!(custom_matches.len(), 1);
    assert_eq!(custom_matches[0].name, "project-chat");

    let summary = render_skill_catalog_match_summary(&catalog, "project-chat", &custom_matches);
    assert!(summary.contains("skills query=\"project-chat\""));
    assert!(summary.contains("next=edit the file, then propose via skill-policy"));
    assert!(summary.contains("usage=edit the file, then propose via skill-policy"));
    assert!(summary.contains("decision=custom skills stay project-local until promoted by policy"));

    fs::remove_dir_all(root).expect("cleanup skill match temp dir");
}

#[test]
fn skill_catalog_reuses_cached_entries_for_unchanged_files() {
    let root = std::env::temp_dir().join(format!("memd-skills-cache-{}", uuid::Uuid::new_v4()));
    let skill_root = root.join("skills").join("project-chat");
    fs::create_dir_all(&skill_root).expect("create skill root");
    fs::write(
        skill_root.join("SKILL.md"),
        r#"---
name: project-chat
description: route project chat through a compact visible lane
---
"#,
    )
    .expect("write skill file");

    let first = build_skill_catalog(&root.join("skills")).expect("build skill catalog first");
    assert_eq!(first.cache_hits, 0);
    assert_eq!(first.cache_scanned, 1);

    let second = build_skill_catalog(&root.join("skills")).expect("build skill catalog second");
    assert_eq!(second.cache_hits, 1);
    assert_eq!(second.cache_scanned, 1);
    assert_eq!(second.custom.len(), 1);
    assert_eq!(second.custom[0].name, "project-chat");

    fs::remove_dir_all(root).expect("cleanup skill cache temp dir");
}

#[test]
fn inspiration_search_reuses_cache_for_unchanged_files() {
    let root = std::env::temp_dir().join(format!("memd-inspiration-{}", uuid::Uuid::new_v4()));
    let lane_dir = root.join(".memd").join("lanes").join("inspiration");
    fs::create_dir_all(&lane_dir).expect("create inspiration lane dir");
    fs::write(
        lane_dir.join("INSPIRATION-LANE.md"),
        "# Inspiration Lane\n\nLightRAG keeps the backend swappable.\n",
    )
    .expect("write inspiration lane");

    let first = search_inspiration_lane(&root, "LightRAG", 10).expect("first inspiration search");
    assert_eq!(first.hits.len(), 1);
    assert_eq!(first.cache_hits, 0);
    assert_eq!(first.cache_scanned, 1);

    let second = search_inspiration_lane(&root, "LightRAG", 10).expect("second inspiration search");
    assert_eq!(second.hits.len(), 1);
    assert_eq!(second.cache_hits, 1);
    assert_eq!(second.cache_scanned, 0);

    let summary = render_inspiration_search_summary(&root, "LightRAG", &second);
    assert!(summary.contains("cache_hits=1"));
    assert!(summary.contains("scanned=0"));

    fs::remove_dir_all(root).expect("cleanup inspiration temp dir");
}

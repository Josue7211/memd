use super::*;

#[tokio::test]
async fn run_session_command_rebinds_local_bundle_to_live_session() {
    let _home_lock = lock_home_mutation();
    let temp_root =
        std::env::temp_dir().join(format!("memd-session-rebind-{}", uuid::Uuid::new_v4()));
    let home = temp_root.join("home");
    let repo_root = temp_root.join("repo");
    let global_root = home.join(".memd");
    let local_bundle = repo_root.join(".memd");
    fs::create_dir_all(global_root.join("state")).expect("create global state");
    fs::create_dir_all(local_bundle.join("state")).expect("create local state");
    fs::write(
        global_root.join("config.json"),
        format!(
            r#"{{
  "project": "global",
  "namespace": "global",
  "agent": "codex",
  "session": "codex-fresh",
  "tab_id": "tab-alpha",
  "base_url": "{SHARED_MEMD_BASE_URL}"
}}
"#
        ),
    )
    .expect("write global config");
    fs::write(
        local_bundle.join("config.json"),
        format!(
            r#"{{
  "project": "memd",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-stale",
  "tab_id": "tab-stale",
  "base_url": "{SHARED_MEMD_BASE_URL}",
  "workspace": "shared",
  "visibility": "workspace"
}}
"#
        ),
    )
    .expect("write local config");
    fs::write(
        local_bundle.join("env"),
        "MEMD_SESSION=codex-stale\nMEMD_AGENT=codex@codex-stale\n",
    )
    .expect("write env");
    fs::write(
        local_bundle.join("env.ps1"),
        "$env:MEMD_SESSION = \"codex-stale\"\n$env:MEMD_AGENT = \"codex@codex-stale\"\n",
    )
    .expect("write env ps1");

    let original_home = std::env::var_os("HOME");
    let original_dir = std::env::current_dir().expect("read cwd");
    unsafe {
        std::env::set_var("HOME", &home);
    }
    std::env::set_current_dir(&repo_root).expect("set repo cwd");

    let response = run_session_command(
        &SessionArgs {
            output: local_bundle.clone(),
            rebind: true,
            reconcile: false,
            retire_session: None,
            summary: false,
        },
        SHARED_MEMD_BASE_URL,
    )
    .await
    .expect("run session command");
    assert_eq!(response.action, "rebind");
    assert_eq!(response.bundle_session.as_deref(), Some("codex-fresh"));
    assert_eq!(response.live_session.as_deref(), Some("codex-fresh"));

    let config = fs::read_to_string(local_bundle.join("config.json")).expect("read config");
    assert!(config.contains("\"session\": \"codex-fresh\""));
    assert!(config.contains("\"tab_id\": \"tab-alpha\""));

    std::env::set_current_dir(&original_dir).expect("restore cwd");
    if let Some(value) = original_home {
        unsafe {
            std::env::set_var("HOME", value);
        }
    } else {
        unsafe {
            std::env::remove_var("HOME");
        }
    }
    fs::remove_dir_all(temp_root).expect("cleanup session rebind temp");
}

#[tokio::test]
async fn claude_runtime_stack_emits_coordinated_truthful_continuous_summary() {
    let _home_lock = lock_home_mutation();
    let temp_root =
        std::env::temp_dir().join(format!("memd-runtime-stack-{}", uuid::Uuid::new_v4()));
    let home = temp_root.join("home");
    let repo_root = temp_root.join("repo");
    let sibling_root = temp_root.join("sibling");
    let global_root = home.join(".memd");
    let local_bundle = repo_root.join(".memd");
    let sibling_bundle = sibling_root.join(".memd");
    fs::create_dir_all(global_root.join("state")).expect("create global state");
    fs::create_dir_all(local_bundle.join("state")).expect("create local state");
    fs::create_dir_all(sibling_bundle.join("state")).expect("create sibling state");
    fs::write(
        global_root.join("config.json"),
        format!(
            r#"{{
  "project": "global",
  "namespace": "global",
  "agent": "codex",
  "session": "codex-fresh",
  "tab_id": "tab-alpha",
  "base_url": "{SHARED_MEMD_BASE_URL}"
}}
"#
        ),
    )
    .expect("write global config");
    fs::write(
        local_bundle.join("config.json"),
        format!(
            r#"{{
  "project": "memd",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-stale",
  "base_url": "{SHARED_MEMD_BASE_URL}",
  "route": "auto",
  "intent": "current_task",
  "workspace": "shared",
  "visibility": "workspace"
}}
"#
        ),
    )
    .expect("write local config");
    fs::write(
        sibling_bundle.join("config.json"),
        format!(
            r#"{{
  "project": "memd-helper",
  "namespace": "main",
  "agent": "claude-code",
  "session": "claude-live",
  "tab_id": "tab-beta",
  "hive_system": "codex",
  "hive_role": "agent",
  "hive_groups": ["project:memd"],
  "authority": "participant",
  "base_url": "{SHARED_MEMD_BASE_URL}",
  "workspace": "shared",
  "visibility": "workspace"
}}
"#
        ),
    )
    .expect("write sibling config");
    fs::write(
        bundle_heartbeat_state_path(&sibling_bundle),
        serde_json::to_string_pretty(&BundleHeartbeatState {
            session: Some("claude-live".to_string()),
            agent: Some("claude-code".to_string()),
            effective_agent: Some("claude-code@claude-live".to_string()),
            tab_id: Some("tab-beta".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("agent".to_string()),
            worker_name: Some("claude-code".to_string()),
            display_name: None,
            role: Some("agent".to_string()),
            capabilities: vec!["memory".to_string(), "coordination".to_string()],
            hive_groups: vec!["project:memd".to_string()],
            lane_id: Some(sibling_root.display().to_string()),
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: Some(default_heartbeat_model()),
            project: Some("memd-helper".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            repo_root: Some(repo_root.display().to_string()),
            worktree_root: Some(sibling_root.display().to_string()),
            branch: Some("feature/claude-live".to_string()),
            base_branch: Some("main".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some(SHARED_MEMD_BASE_URL.to_string()),
            base_url_healthy: Some(true),
            host: Some("workstation".to_string()),
            pid: Some(4242),
            topic_claim: None,
            scope_claims: Vec::new(),
            task_id: None,
            focus: Some("Handle coordination backlog".to_string()),
            pressure: Some("Keep the hive lane clean".to_string()),
            next_recovery: None,
            next_action: None,
            needs_help: false,
            needs_review: false,
            handoff_state: None,
            confidence: None,
            risk: None,
            status: "live".to_string(),
            last_seen: Utc::now(),
            authority_mode: Some("shared".to_string()),
            authority_degraded: false,
        })
        .expect("serialize sibling heartbeat")
            + "\n",
    )
    .expect("write sibling heartbeat");

    let original_home = std::env::var_os("HOME");
    let original_dir = std::env::current_dir().expect("read cwd");
    unsafe {
        std::env::set_var("HOME", &home);
    }
    std::env::set_current_dir(&repo_root).expect("set repo cwd");

    let wire = run_hive_command(&HiveArgs {
        command: None,
        agent: None,
        project: None,
        namespace: None,
        global: false,
        project_root: Some(repo_root.clone()),
        seed_existing: false,
        session: None,
        tab_id: None,
        hive_system: Some("codex".to_string()),
        hive_role: Some("agent".to_string()),
        capability: Vec::new(),
        hive_group: vec!["project:memd".to_string()],
        hive_group_goal: None,
        authority: Some("participant".to_string()),
        output: local_bundle.clone(),
        base_url: SHARED_MEMD_BASE_URL.to_string(),
        rag_url: None,
        route: "auto".to_string(),
        intent: "current_task".to_string(),
        workspace: Some("shared".to_string()),
        visibility: Some("workspace".to_string()),
        publish_heartbeat: false,
        force: false,
        summary: false,
    })
    .await
    .expect("run hive command");
    assert_eq!(wire.session.as_deref(), Some("codex-fresh"));

    let mut awareness = read_project_awareness(&AwarenessArgs {
        output: local_bundle.clone(),
        root: Some(temp_root.clone()),
        include_current: true,
        summary: false,
    })
    .await
    .expect("read awareness");
    awareness.entries.push(ProjectAwarenessEntry {
        project_dir: "remote".to_string(),
        bundle_root: format!("remote:{SHARED_MEMD_BASE_URL}:session-stale"),
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        repo_root: None,
        worktree_root: None,
        branch: None,
        base_branch: None,
        agent: Some("claude-code".to_string()),
        session: Some("session-stale".to_string()),
        tab_id: None,
        effective_agent: Some("claude-code@session-stale".to_string()),
        hive_system: None,
        hive_role: None,
        capabilities: vec!["memory".to_string()],
        hive_groups: Vec::new(),
        hive_group_goal: None,
        authority: None,
        base_url: Some(SHARED_MEMD_BASE_URL.to_string()),
        presence: "stale".to_string(),
        host: None,
        pid: None,
        active_claims: 0,
        workspace: Some("shared".to_string()),
        visibility: Some("workspace".to_string()),
        topic_claim: None,
        scope_claims: Vec::new(),
        task_id: None,
        focus: None,
        pressure: None,
        next_recovery: None,
        last_updated: Some(Utc::now() - chrono::TimeDelta::minutes(9)),
    });
    awareness.entries.push(ProjectAwarenessEntry {
        project_dir: "remote".to_string(),
        bundle_root: format!("remote:{SHARED_MEMD_BASE_URL}:session-dead"),
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        repo_root: None,
        worktree_root: None,
        branch: None,
        base_branch: None,
        agent: Some("codex".to_string()),
        session: Some("session-dead".to_string()),
        tab_id: None,
        effective_agent: Some("codex@session-dead".to_string()),
        hive_system: None,
        hive_role: None,
        capabilities: vec!["memory".to_string()],
        hive_groups: Vec::new(),
        hive_group_goal: None,
        authority: None,
        base_url: Some(SHARED_MEMD_BASE_URL.to_string()),
        presence: "dead".to_string(),
        host: None,
        pid: None,
        active_claims: 0,
        workspace: Some("shared".to_string()),
        visibility: Some("workspace".to_string()),
        topic_claim: None,
        scope_claims: Vec::new(),
        task_id: None,
        focus: None,
        pressure: None,
        next_recovery: None,
        last_updated: Some(Utc::now() - chrono::TimeDelta::minutes(30)),
    });
    awareness.entries.push(ProjectAwarenessEntry {
        project_dir: "remote".to_string(),
        bundle_root: format!("remote:{SHARED_MEMD_BASE_URL}:session-superseded"),
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        repo_root: None,
        worktree_root: None,
        branch: None,
        base_branch: None,
        agent: Some("codex".to_string()),
        session: Some("session-superseded".to_string()),
        tab_id: None,
        effective_agent: Some("codex@session-superseded".to_string()),
        hive_system: None,
        hive_role: None,
        capabilities: vec!["memory".to_string()],
        hive_groups: Vec::new(),
        hive_group_goal: None,
        authority: None,
        base_url: Some(SHARED_MEMD_BASE_URL.to_string()),
        presence: "stale".to_string(),
        host: None,
        pid: None,
        active_claims: 0,
        workspace: Some("shared".to_string()),
        visibility: Some("workspace".to_string()),
        topic_claim: None,
        scope_claims: Vec::new(),
        task_id: None,
        focus: None,
        pressure: None,
        next_recovery: None,
        last_updated: Some(Utc::now() - chrono::TimeDelta::minutes(12)),
    });
    let summary = render_project_awareness_summary(&awareness);
    assert!(summary.contains("current_session:"));
    assert!(summary.contains("active_hive_sessions:"));
    assert!(summary.contains("! stale_remote_sessions="));
    assert!(summary.contains("stale_sessions:"));
    assert!(summary.contains("hidden_remote_dead="));
    assert!(summary.contains("hidden_superseded_stale=1"));
    assert!(summary.contains("session=codex-fresh"));
    assert!(summary.contains("truth=current"));

    std::env::set_current_dir(&original_dir).expect("restore cwd");
    if let Some(value) = original_home {
        unsafe {
            std::env::set_var("HOME", value);
        }
    } else {
        unsafe {
            std::env::remove_var("HOME");
        }
    }
    fs::remove_dir_all(temp_root).expect("cleanup runtime stack temp");
}

#[tokio::test]
async fn run_public_locomo_command_writes_artifacts() {
    let dir = std::env::temp_dir().join(format!(
        "memd-public-benchmark-locomo-{}",
        uuid::Uuid::new_v4()
    ));
    let output = dir.join(".memd");
    fs::create_dir_all(&output).expect("create output dir");

    let fixture = public_benchmark_fixture_path("locomo");
    let report = run_public_benchmark_command(&PublicBenchmarkArgs {
        dataset: "locomo".to_string(),
        mode: Some("raw".to_string()),
        retrieval_backend: None,
        rag_url: None,
        top_k: Some(5),
        limit: Some(2),
        dataset_root: Some(fixture),
        reranker: None,
        write: false,
        json: false,
        out: output.clone(),
    })
    .await
    .expect("run locomo public benchmark");

    assert_eq!(report.manifest.benchmark_id, "locomo");
    assert_eq!(report.manifest.dataset_name, "LoCoMo");
    assert_eq!(report.item_count, 2);

    let receipt =
        write_public_benchmark_run_artifacts(&output, &report).expect("write locomo artifacts");
    assert!(receipt.manifest_path.exists());
    assert!(receipt.results_path.exists());
    assert!(receipt.results_jsonl_path.exists());
    assert!(receipt.report_path.exists());

    fs::remove_dir_all(dir).expect("cleanup public benchmark locomo dir");
}

#[tokio::test]
async fn run_public_convomem_command_writes_artifacts() {
    let dir = std::env::temp_dir().join(format!(
        "memd-public-benchmark-convomem-{}",
        uuid::Uuid::new_v4()
    ));
    let output = dir.join(".memd");
    fs::create_dir_all(&output).expect("create output dir");

    let fixture = public_benchmark_fixture_path("convomem");
    let report = run_public_benchmark_command(&PublicBenchmarkArgs {
        dataset: "convomem".to_string(),
        mode: Some("raw".to_string()),
        retrieval_backend: None,
        rag_url: None,
        top_k: Some(5),
        limit: Some(2),
        dataset_root: Some(fixture),
        reranker: None,
        write: false,
        json: false,
        out: output.clone(),
    })
    .await
    .expect("run convomem public benchmark");

    assert_eq!(report.manifest.benchmark_id, "convomem");
    assert_eq!(report.manifest.dataset_name, "ConvoMem");
    assert_eq!(report.item_count, 2);

    let receipt =
        write_public_benchmark_run_artifacts(&output, &report).expect("write convomem artifacts");
    assert!(receipt.manifest_path.exists());
    assert!(receipt.results_path.exists());
    assert!(receipt.results_jsonl_path.exists());
    assert!(receipt.report_path.exists());

    fs::remove_dir_all(dir).expect("cleanup public benchmark convomem dir");
}

#[tokio::test]
async fn run_public_membench_command_writes_artifacts() {
    let dir = std::env::temp_dir().join(format!(
        "memd-public-benchmark-membench-{}",
        uuid::Uuid::new_v4()
    ));
    let output = dir.join(".memd");
    fs::create_dir_all(&output).expect("create output dir");

    let fixture = public_benchmark_fixture_path("membench");
    let report = run_public_benchmark_command(&PublicBenchmarkArgs {
        dataset: "membench".to_string(),
        mode: Some("raw".to_string()),
        retrieval_backend: None,
        rag_url: None,
        top_k: Some(5),
        limit: Some(2),
        dataset_root: Some(fixture),
        reranker: None,
        write: false,
        json: false,
        out: output.clone(),
    })
    .await
    .expect("run membench public benchmark");

    assert_eq!(report.manifest.benchmark_id, "membench");
    assert_eq!(report.manifest.dataset_name, "MemBench");
    assert_eq!(report.item_count, 2);

    let receipt =
        write_public_benchmark_run_artifacts(&output, &report).expect("write membench artifacts");
    assert!(receipt.manifest_path.exists());
    assert!(receipt.results_path.exists());
    assert!(receipt.results_jsonl_path.exists());
    assert!(receipt.report_path.exists());

    fs::remove_dir_all(dir).expect("cleanup public benchmark membench dir");
}

#[test]
fn cli_parses_verify_feature_command() {
    let cli = Cli::try_parse_from([
        "memd",
        "verify",
        "feature",
        "feature.bundle.resume",
        "--output",
        ".memd",
        "--summary",
    ])
    .expect("verify feature command should parse");

    match cli.command {
        Commands::Verify(args) => match args.command {
            VerifyCommand::Feature(feature_args) => {
                assert_eq!(feature_args.feature_id, "feature.bundle.resume");
                assert_eq!(feature_args.output, PathBuf::from(".memd"));
                assert!(feature_args.summary);
            }
            other => panic!("expected verify feature command, got {other:?}"),
        },
        other => panic!("expected verify command, got {other:?}"),
    }
}

#[test]
fn cli_parses_verify_sweep_lane() {
    let cli = Cli::try_parse_from([
        "memd", "verify", "sweep", "--lane", "nightly", "--output", ".memd",
    ])
    .expect("verify sweep command should parse");

    match cli.command {
        Commands::Verify(args) => match args.command {
            VerifyCommand::Sweep(sweep_args) => {
                assert_eq!(sweep_args.output, PathBuf::from(".memd"));
                assert_eq!(sweep_args.lane, "nightly");
            }
            other => panic!("expected verify sweep command, got {other:?}"),
        },
        other => panic!("expected verify command, got {other:?}"),
    }
}

#[test]
fn run_verify_list_command_reports_registry_verifiers_and_fixtures() {
    let dir = std::env::temp_dir().join(format!("memd-verify-list-{}", uuid::Uuid::new_v4()));
    let repo_root = dir.join("repo");
    let output = repo_root.join(".memd");
    fs::create_dir_all(repo_root.join(".git")).expect("create git dir");
    fs::create_dir_all(&output).expect("create output dir");
    write_test_benchmark_registry(&repo_root);

    let report = run_verify_list_command(&VerifyListArgs {
        output: output.clone(),
        lane: Some("nightly".to_string()),
        summary: false,
    })
    .expect("run verify list");

    assert!(report.registry_loaded);
    assert!(report.registry_verifiers > 0);
    assert!(report.registry_fixtures > 0);
    assert_eq!(report.lane.as_deref(), Some("nightly"));
    let summary = render_verify_summary(&report);
    assert!(summary.contains("verifiers="));
    assert!(summary.contains("fixtures="));

    fs::remove_dir_all(dir).expect("cleanup verify list dir");
}

#[test]
fn materialize_continuity_fixture_creates_temp_bundle() {
    let fixture = test_continuity_fixture_record();
    let env = materialize_fixture(&fixture, None).expect("materialize fixture");
    assert!(env.bundle_root.join("config.json").exists());
    assert_eq!(env._fixture_id, "fixture.continuity_bundle");
}

#[test]
fn materialize_fixture_writes_seed_files_into_bundle() {
    let mut fixture = test_continuity_fixture_record();
    fixture.seed_files = vec!["state/checkpoint.txt".to_string()];

    let env = materialize_fixture(&fixture, None).expect("materialize fixture");

    let seeded = env.bundle_root.join("state/checkpoint.txt");
    assert!(seeded.exists());
    let contents = fs::read_to_string(seeded).expect("read seeded file");
    assert!(contents.contains("resume next step"));
}

#[tokio::test]
async fn materialize_hive_fixture_creates_named_session_bundles() {
    let base_url = spawn_mock_runtime_server(MockRuntimeState::default(), false).await;
    let fixture = FixtureRecord {
        id: "fixture.hive-two-session-bundle.test".to_string(),
        kind: "bundle_fixture".to_string(),
        description: "hive".to_string(),
        seed_files: Vec::new(),
        seed_config: json!({
            "project": "demo",
            "namespace": "main",
            "agent": "codex",
            "session": "sender",
            "workspace": "shared",
            "base_url": base_url
        }),
        seed_memories: Vec::new(),
        seed_events: Vec::new(),
        seed_sessions: vec!["sender".to_string(), "target".to_string()],
        seed_claims: Vec::new(),
        seed_vault: None,
        backend_mode: "normal".to_string(),
        isolation: "fresh_temp_dir".to_string(),
        cleanup_policy: "destroy".to_string(),
    };

    let env = materialize_fixture(&fixture, None).expect("materialize hive fixture");

    let sender_bundle = env
        .fixture_vars
        .get("sender_bundle")
        .map(PathBuf::from)
        .expect("sender bundle path");
    let target_bundle = env
        .fixture_vars
        .get("target_bundle")
        .map(PathBuf::from)
        .expect("target bundle path");
    assert!(sender_bundle.join("config.json").exists());
    assert!(target_bundle.join("config.json").exists());
    assert_eq!(env.bundle_root, sender_bundle);
    let sender_config = read_bundle_runtime_config(&env.bundle_root)
        .expect("read sender runtime config")
        .expect("sender runtime config present");
    let target_config = read_bundle_runtime_config(&target_bundle)
        .expect("read target runtime config")
        .expect("target runtime config present");
    assert_eq!(sender_config.agent.as_deref(), Some("Sender"));
    assert_eq!(target_config.agent.as_deref(), Some("Target"));
    assert_eq!(
        env.fixture_vars
            .get("target_session")
            .is_some_and(|value| value.starts_with("target-")),
        true
    );
}

#[tokio::test]
async fn run_resume_feature_verifier_writes_evidence_artifacts() {
    let fixture = test_continuity_fixture_record();
    let verifier = VerifierRecord {
        id: "verifier.feature.bundle.resume".to_string(),
        name: "Resume feature".to_string(),
        verifier_type: "feature_contract".to_string(),
        pillar: "memory-continuity".to_string(),
        family: "bundle-runtime".to_string(),
        subject_ids: vec!["feature.bundle.resume".to_string()],
        fixture_id: fixture.id.clone(),
        baseline_modes: vec!["with_memd".to_string()],
        steps: Vec::new(),
        assertions: Vec::new(),
        metrics: vec!["prompt_tokens".to_string()],
        evidence_requirements: vec!["live_primary".to_string()],
        gate_target: "acceptable".to_string(),
        status: "declared".to_string(),
        lanes: vec!["fast".to_string()],
        helper_hooks: Vec::new(),
    };

    let run = run_verifier_record(&verifier, &fixture, None)
        .await
        .expect("run verifier");
    assert_eq!(run.verifier_id, "verifier.feature.bundle.resume");
    assert!(!run.evidence_ids.is_empty());
    let materialized = materialize_fixture(&fixture, None).expect("materialize fixture again");
    write_verifier_run_artifacts(
        &materialized.bundle_root,
        &run,
        &json!({"verifier_id": verifier.id, "confidence_tier": "live_primary"}),
    )
    .expect("write verifier artifacts");
    assert!(
        verification_reports_dir(&materialized.bundle_root)
            .join("latest.json")
            .exists()
    );
    assert!(verification_evidence_dir(&materialized.bundle_root).exists());
}

#[tokio::test]
async fn run_verifier_record_executes_wake_step_and_writes_wakeup() {
    let fixture = test_continuity_fixture_record();
    let verifier = VerifierRecord {
        id: "verifier.feature.bundle.wake.steps".to_string(),
        name: "Wake feature with steps".to_string(),
        verifier_type: "feature_contract".to_string(),
        pillar: "memory-continuity".to_string(),
        family: "bundle-runtime".to_string(),
        subject_ids: vec!["feature.bundle.wake".to_string()],
        fixture_id: fixture.id.clone(),
        baseline_modes: vec!["with_memd".to_string()],
        steps: vec![VerifierStepRecord {
            kind: "cli".to_string(),
            run: Some("memd wake --output {{bundle}}".to_string()),
            name: None,
            left: None,
            right: None,
        }],
        assertions: vec![VerifierAssertionRecord {
            kind: "file_contains".to_string(),
            path: Some("MEMD_WAKEUP.md".to_string()),
            equals_fixture: None,
            contains_fixture: None,
            exists: Some(true),
            metric: None,
            op: None,
            left: None,
            right: None,
            name: None,
        }],
        metrics: vec!["prompt_tokens".to_string()],
        evidence_requirements: vec!["live_primary".to_string()],
        gate_target: "acceptable".to_string(),
        status: "declared".to_string(),
        lanes: vec!["fast".to_string()],
        helper_hooks: Vec::new(),
    };

    let run = run_verifier_record(&verifier, &fixture, None)
        .await
        .expect("run wake verifier");

    assert_eq!(run.status, "passing");
    assert!(
        run.metrics_observed
            .get("prompt_tokens")
            .and_then(JsonValue::as_u64)
            .is_some_and(|value| value > 0)
    );
}

#[tokio::test]
async fn run_verifier_record_executes_resume_steps_and_records_prompt_tokens() {
    let fixture = test_continuity_fixture_record();
    let verifier = VerifierRecord {
        id: "verifier.feature.bundle.resume.steps".to_string(),
        name: "Resume feature with steps".to_string(),
        verifier_type: "feature_contract".to_string(),
        pillar: "memory-continuity".to_string(),
        family: "bundle-runtime".to_string(),
        subject_ids: vec!["feature.bundle.resume".to_string()],
        fixture_id: fixture.id.clone(),
        baseline_modes: vec!["with_memd".to_string()],
        steps: vec![
            VerifierStepRecord {
                kind: "cli".to_string(),
                run: Some("memd checkpoint --output {{bundle}}".to_string()),
                name: None,
                left: None,
                right: None,
            },
            VerifierStepRecord {
                kind: "cli".to_string(),
                run: Some("memd resume --output {{bundle}}".to_string()),
                name: None,
                left: None,
                right: None,
            },
        ],
        assertions: vec![VerifierAssertionRecord {
            kind: "json_path".to_string(),
            path: Some("resume.project".to_string()),
            equals_fixture: None,
            contains_fixture: None,
            exists: Some(true),
            metric: None,
            op: None,
            left: None,
            right: None,
            name: None,
        }],
        metrics: vec!["prompt_tokens".to_string()],
        evidence_requirements: vec!["live_primary".to_string()],
        gate_target: "acceptable".to_string(),
        status: "declared".to_string(),
        lanes: vec!["fast".to_string()],
        helper_hooks: Vec::new(),
    };

    let run = run_verifier_record(&verifier, &fixture, None)
        .await
        .expect("run verifier");

    assert_eq!(run.status, "passing");
    assert!(
        run.metrics_observed
            .get("prompt_tokens")
            .and_then(JsonValue::as_u64)
            .is_some_and(|value| value > 0)
    );
}

#[tokio::test]
async fn run_verifier_record_executes_compare_steps_and_records_delta_metrics() {
    let fixture = test_continuity_fixture_record();
    let verifier = VerifierRecord {
        id: "verifier.compare.resume.steps".to_string(),
        name: "Resume compare with steps".to_string(),
        verifier_type: "comparative".to_string(),
        pillar: "efficiency".to_string(),
        family: "memory-continuity".to_string(),
        subject_ids: vec!["journey.resume-handoff-attach".to_string()],
        fixture_id: fixture.id.clone(),
        baseline_modes: vec!["no_mempath".to_string(), "with_memd".to_string()],
        steps: vec![
            VerifierStepRecord {
                kind: "helper".to_string(),
                run: None,
                name: Some("run_resume_without_memd".to_string()),
                left: None,
                right: None,
            },
            VerifierStepRecord {
                kind: "helper".to_string(),
                run: None,
                name: Some("run_resume_with_memd".to_string()),
                left: None,
                right: None,
            },
            VerifierStepRecord {
                kind: "compare".to_string(),
                run: None,
                name: None,
                left: Some("no_mempath".to_string()),
                right: Some("with_memd".to_string()),
            },
        ],
        assertions: vec![VerifierAssertionRecord {
            kind: "metric_compare".to_string(),
            path: None,
            equals_fixture: None,
            contains_fixture: None,
            exists: None,
            metric: Some("prompt_tokens".to_string()),
            op: Some("<".to_string()),
            left: Some("with_memd".to_string()),
            right: Some("no_mempath".to_string()),
            name: None,
        }],
        metrics: vec![
            "prompt_tokens".to_string(),
            "reconstruction_steps".to_string(),
            "token_delta".to_string(),
        ],
        evidence_requirements: vec!["live_primary".to_string()],
        gate_target: "strong".to_string(),
        status: "declared".to_string(),
        lanes: vec!["comparative".to_string()],
        helper_hooks: Vec::new(),
    };

    let run = run_verifier_record(&verifier, &fixture, None)
        .await
        .expect("run comparative verifier");

    assert_eq!(run.status, "passing");
    assert_eq!(
        run.metrics_observed
            .get("token_delta")
            .and_then(JsonValue::as_i64),
        Some(500)
    );
    assert_eq!(
        run.metrics_observed
            .get("with_memd_better")
            .and_then(JsonValue::as_bool),
        Some(true)
    );
}

#[tokio::test]
async fn run_verifier_record_supports_file_contains_assertions() {
    let mut fixture = test_continuity_fixture_record();
    fixture.seed_files = vec!["state/checkpoint.txt".to_string()];
    let verifier = VerifierRecord {
        id: "verifier.feature.file-assert".to_string(),
        name: "File assertion verifier".to_string(),
        verifier_type: "feature_contract".to_string(),
        pillar: "memory-continuity".to_string(),
        family: "bundle-runtime".to_string(),
        subject_ids: vec!["feature.bundle.resume".to_string()],
        fixture_id: fixture.id.clone(),
        baseline_modes: vec!["with_memd".to_string()],
        steps: Vec::new(),
        assertions: vec![VerifierAssertionRecord {
            kind: "file_contains".to_string(),
            path: Some("state/checkpoint.txt".to_string()),
            equals_fixture: None,
            contains_fixture: Some("task.next_action".to_string()),
            exists: None,
            metric: None,
            op: None,
            left: None,
            right: None,
            name: None,
        }],
        metrics: Vec::new(),
        evidence_requirements: vec!["live_primary".to_string()],
        gate_target: "acceptable".to_string(),
        status: "declared".to_string(),
        lanes: vec!["fast".to_string()],
        helper_hooks: Vec::new(),
    };

    let run = run_verifier_record(&verifier, &fixture, None)
        .await
        .expect("run verifier with file assertion");

    assert_eq!(run.status, "passing");
}

#[tokio::test]
async fn run_verifier_record_executes_messages_send_ack_flow() {
    let base_url = spawn_mock_runtime_server(MockRuntimeState::default(), false).await;
    let fixture = FixtureRecord {
        id: "fixture.hive-two-session-bundle.test".to_string(),
        kind: "bundle_fixture".to_string(),
        description: "hive".to_string(),
        seed_files: Vec::new(),
        seed_config: json!({
            "project": "demo",
            "namespace": "main",
            "agent": "codex",
            "session": "sender",
            "workspace": "shared",
            "base_url": base_url
        }),
        seed_memories: Vec::new(),
        seed_events: Vec::new(),
        seed_sessions: vec!["sender".to_string(), "target".to_string()],
        seed_claims: Vec::new(),
        seed_vault: None,
        backend_mode: "normal".to_string(),
        isolation: "fresh_temp_dir".to_string(),
        cleanup_policy: "destroy".to_string(),
    };
    let verifier = VerifierRecord {
            id: "verifier.feature.hive.messages-send-ack".to_string(),
            name: "Messages send and ack".to_string(),
            verifier_type: "feature_contract".to_string(),
            pillar: "coordination".to_string(),
            family: "coordination-hive".to_string(),
            subject_ids: vec!["feature.hive.messages".to_string()],
            fixture_id: fixture.id.clone(),
            baseline_modes: vec!["with_memd".to_string()],
            steps: vec![
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd messages --output {{sender_bundle}} --send --target-session {{target_session}} --kind handoff --content \"Pick up the parser refactor\"".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd messages --output {{target_bundle}} --inbox".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "helper".to_string(),
                    run: None,
                    name: Some("capture_message_id".to_string()),
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd messages --output {{target_bundle}} --ack {{message_id}}".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
            ],
            assertions: vec![VerifierAssertionRecord {
                kind: "helper".to_string(),
                path: None,
                equals_fixture: None,
                contains_fixture: None,
                exists: None,
                metric: None,
                op: None,
                left: None,
                right: None,
                name: Some("assert_message_acknowledged".to_string()),
            }],
            metrics: vec!["delivery_count".to_string()],
            evidence_requirements: vec!["live_primary".to_string()],
            gate_target: "acceptable".to_string(),
            status: "declared".to_string(),
            lanes: vec!["nightly".to_string()],
            helper_hooks: Vec::new(),
        };

    let run = run_verifier_record(&verifier, &fixture, Some(&base_url))
        .await
        .expect("run hive message verifier");

    assert_eq!(run.status, "passing");
    assert_eq!(
        run.metrics_observed
            .get("delivery_count")
            .and_then(JsonValue::as_u64),
        Some(1)
    );
}

#[tokio::test]
async fn run_verifier_record_executes_claim_transfer_flow() {
    let base_url = spawn_mock_runtime_server(MockRuntimeState::default(), false).await;
    let fixture = FixtureRecord {
        id: "fixture.hive-claims-bundle.test".to_string(),
        kind: "bundle_fixture".to_string(),
        description: "hive claims".to_string(),
        seed_files: Vec::new(),
        seed_config: json!({
            "project": "demo",
            "namespace": "main",
            "agent": "codex",
            "session": "sender",
            "workspace": "shared",
            "base_url": base_url
        }),
        seed_memories: Vec::new(),
        seed_events: Vec::new(),
        seed_sessions: vec!["sender".to_string(), "target".to_string()],
        seed_claims: Vec::new(),
        seed_vault: None,
        backend_mode: "normal".to_string(),
        isolation: "fresh_temp_dir".to_string(),
        cleanup_policy: "destroy".to_string(),
    };
    let verifier = VerifierRecord {
            id: "verifier.feature.hive.claims-transfer".to_string(),
            name: "Claims transfer".to_string(),
            verifier_type: "feature_contract".to_string(),
            pillar: "coordination".to_string(),
            family: "coordination-hive".to_string(),
            subject_ids: vec!["feature.hive.claims".to_string()],
            fixture_id: fixture.id.clone(),
            baseline_modes: vec!["with_memd".to_string()],
            steps: vec![
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd claims --output {{sender_bundle}} --acquire --scope task:parser-refactor".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd claims --output {{sender_bundle}} --transfer-to-session {{target_session}} --scope task:parser-refactor".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
            ],
            assertions: vec![VerifierAssertionRecord {
                kind: "json_path".to_string(),
                path: Some("claims_transfer.claims.0.session".to_string()),
                equals_fixture: Some("target_session".to_string()),
                contains_fixture: None,
                exists: None,
                metric: None,
                op: None,
                left: None,
                right: None,
                name: None,
            }],
            metrics: vec!["claim_count".to_string()],
            evidence_requirements: vec!["live_primary".to_string()],
            gate_target: "acceptable".to_string(),
            status: "declared".to_string(),
            lanes: vec!["nightly".to_string()],
            helper_hooks: Vec::new(),
        };

    let run = run_verifier_record(&verifier, &fixture, Some(&base_url))
        .await
        .expect("run hive claims verifier");

    assert_eq!(run.status, "passing");
    assert_eq!(
        run.metrics_observed
            .get("claim_count")
            .and_then(JsonValue::as_u64),
        Some(1)
    );
}

#[tokio::test]
async fn run_verifier_record_executes_task_assignment_flow() {
    let base_url = spawn_mock_runtime_server(MockRuntimeState::default(), false).await;
    let fixture = FixtureRecord {
        id: "fixture.hive-tasks-bundle.test".to_string(),
        kind: "bundle_fixture".to_string(),
        description: "hive tasks".to_string(),
        seed_files: Vec::new(),
        seed_config: json!({
            "project": "demo",
            "namespace": "main",
            "agent": "codex",
            "session": "sender",
            "workspace": "shared",
            "base_url": base_url
        }),
        seed_memories: Vec::new(),
        seed_events: Vec::new(),
        seed_sessions: vec!["sender".to_string(), "target".to_string()],
        seed_claims: Vec::new(),
        seed_vault: None,
        backend_mode: "normal".to_string(),
        isolation: "fresh_temp_dir".to_string(),
        cleanup_policy: "destroy".to_string(),
    };
    let verifier = VerifierRecord {
            id: "verifier.feature.hive.tasks-assign".to_string(),
            name: "Tasks assign".to_string(),
            verifier_type: "feature_contract".to_string(),
            pillar: "coordination".to_string(),
            family: "coordination-hive".to_string(),
            subject_ids: vec!["feature.hive.tasks".to_string()],
            fixture_id: fixture.id.clone(),
            baseline_modes: vec!["with_memd".to_string()],
            steps: vec![
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd tasks --output {{sender_bundle}} --upsert --task-id parser-refactor --title \"Parser refactor\" --status in_progress --mode exclusive_write --scope task:parser-refactor".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd tasks --output {{sender_bundle}} --assign-to-session {{target_session}} --task-id parser-refactor".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
            ],
            assertions: vec![VerifierAssertionRecord {
                kind: "json_path".to_string(),
                path: Some("tasks_assign.tasks.0.session".to_string()),
                equals_fixture: Some("target_session".to_string()),
                contains_fixture: None,
                exists: None,
                metric: None,
                op: None,
                left: None,
                right: None,
                name: None,
            }],
            metrics: vec!["task_count".to_string()],
            evidence_requirements: vec!["live_primary".to_string()],
            gate_target: "acceptable".to_string(),
            status: "declared".to_string(),
            lanes: vec!["nightly".to_string()],
            helper_hooks: Vec::new(),
        };

    let run = run_verifier_record(&verifier, &fixture, Some(&base_url))
        .await
        .expect("run hive task verifier");

    assert_eq!(run.status, "passing");
    assert_eq!(
        run.metrics_observed
            .get("task_count")
            .and_then(JsonValue::as_u64),
        Some(1)
    );
}

#[tokio::test]
async fn run_verifier_record_executes_hive_transfer_assign_journey() {
    let base_url = spawn_mock_runtime_server(MockRuntimeState::default(), false).await;
    let fixture = FixtureRecord {
        id: "fixture.hive-journey-bundle.test".to_string(),
        kind: "bundle_fixture".to_string(),
        description: "hive journey".to_string(),
        seed_files: Vec::new(),
        seed_config: json!({
            "project": "demo",
            "namespace": "main",
            "agent": "codex",
            "session": "sender",
            "workspace": "shared",
            "base_url": base_url
        }),
        seed_memories: Vec::new(),
        seed_events: Vec::new(),
        seed_sessions: vec!["sender".to_string(), "target".to_string()],
        seed_claims: Vec::new(),
        seed_vault: None,
        backend_mode: "normal".to_string(),
        isolation: "fresh_temp_dir".to_string(),
        cleanup_policy: "destroy".to_string(),
    };
    let verifier = VerifierRecord {
            id: "verifier.journey.hive-transfer-assign".to_string(),
            name: "Hive transfer assign journey".to_string(),
            verifier_type: "journey".to_string(),
            pillar: "memory-continuity".to_string(),
            family: "coordination-hive".to_string(),
            subject_ids: vec!["journey.hive.transfer-assign".to_string()],
            fixture_id: fixture.id.clone(),
            baseline_modes: vec!["with_memd".to_string()],
            steps: vec![
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd messages --output {{sender_bundle}} --send --target-session {{target_session}} --kind handoff --content \"Pick up {{task.id}}\"".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd messages --output {{target_bundle}} --inbox".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "helper".to_string(),
                    run: None,
                    name: Some("capture_message_id".to_string()),
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd messages --output {{target_bundle}} --ack {{message_id}}".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd claims --output {{sender_bundle}} --acquire --scope task:{{task.id}}".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd claims --output {{sender_bundle}} --transfer-to-session {{target_session}} --scope task:{{task.id}}".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd tasks --output {{sender_bundle}} --upsert --task-id {{task.id}} --title \"Parser refactor\" --status in_progress --mode exclusive_write --scope task:{{task.id}}".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd tasks --output {{sender_bundle}} --assign-to-session {{target_session}} --task-id {{task.id}}".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
            ],
            assertions: vec![
                VerifierAssertionRecord {
                    kind: "helper".to_string(),
                    path: None,
                    equals_fixture: None,
                    contains_fixture: None,
                    exists: None,
                    metric: None,
                    op: None,
                    left: None,
                    right: None,
                    name: Some("assert_message_acknowledged".to_string()),
                },
                VerifierAssertionRecord {
                    kind: "json_path".to_string(),
                    path: Some("claims_transfer.claims.0.session".to_string()),
                    equals_fixture: Some("target_session".to_string()),
                    contains_fixture: None,
                    exists: None,
                    metric: None,
                    op: None,
                    left: None,
                    right: None,
                    name: None,
                },
                VerifierAssertionRecord {
                    kind: "json_path".to_string(),
                    path: Some("tasks_assign.tasks.0.session".to_string()),
                    equals_fixture: Some("target_session".to_string()),
                    contains_fixture: None,
                    exists: None,
                    metric: None,
                    op: None,
                    left: None,
                    right: None,
                    name: None,
                },
            ],
            metrics: vec![
                "delivery_count".to_string(),
                "claim_count".to_string(),
                "task_count".to_string(),
            ],
            evidence_requirements: vec!["live_primary".to_string()],
            gate_target: "strong".to_string(),
            status: "declared".to_string(),
            lanes: vec!["nightly".to_string()],
            helper_hooks: vec![
                "capture_message_id".to_string(),
                "assert_message_acknowledged".to_string(),
            ],
        };

    let run = run_verifier_record(&verifier, &fixture, Some(&base_url))
        .await
        .expect("run hive journey verifier");

    assert_eq!(run.status, "passing");
    assert_eq!(run.gate_result, "strong");
    assert_eq!(
        run.metrics_observed
            .get("delivery_count")
            .and_then(JsonValue::as_u64),
        Some(1)
    );
    assert_eq!(
        run.metrics_observed
            .get("claim_count")
            .and_then(JsonValue::as_u64),
        Some(1)
    );
    assert_eq!(
        run.metrics_observed
            .get("task_count")
            .and_then(JsonValue::as_u64),
        Some(1)
    );
}

#[tokio::test]
async fn run_verifier_record_contains_hive_claim_collision() {
    let base_url = spawn_mock_runtime_server(MockRuntimeState::default(), false).await;
    let fixture = FixtureRecord {
        id: "fixture.hive-adversarial-bundle.test".to_string(),
        kind: "bundle_fixture".to_string(),
        description: "hive adversarial".to_string(),
        seed_files: Vec::new(),
        seed_config: json!({
            "project": "demo",
            "namespace": "main",
            "agent": "codex",
            "session": "sender",
            "workspace": "shared",
            "base_url": base_url
        }),
        seed_memories: Vec::new(),
        seed_events: Vec::new(),
        seed_sessions: vec!["sender".to_string(), "target".to_string()],
        seed_claims: Vec::new(),
        seed_vault: None,
        backend_mode: "normal".to_string(),
        isolation: "fresh_temp_dir".to_string(),
        cleanup_policy: "destroy".to_string(),
    };
    let verifier = VerifierRecord {
        id: "verifier.adversarial.hive-claim-collision".to_string(),
        name: "Hive claim collision containment".to_string(),
        verifier_type: "adversarial".to_string(),
        pillar: "memory-continuity".to_string(),
        family: "coordination-hive".to_string(),
        subject_ids: vec![
            "feature.hive.claims".to_string(),
            "journey.hive.transfer-assign".to_string(),
        ],
        fixture_id: fixture.id.clone(),
        baseline_modes: vec!["with_memd".to_string()],
        steps: vec![
            VerifierStepRecord {
                kind: "cli".to_string(),
                run: Some(
                    "memd claims --output {{sender_bundle}} --acquire --scope task:{{task.id}}"
                        .to_string(),
                ),
                name: None,
                left: None,
                right: None,
            },
            VerifierStepRecord {
                kind: "cli_expect_error".to_string(),
                run: Some(
                    "memd claims --output {{target_bundle}} --acquire --scope task:{{task.id}}"
                        .to_string(),
                ),
                name: None,
                left: None,
                right: None,
            },
        ],
        assertions: vec![
            VerifierAssertionRecord {
                kind: "json_path".to_string(),
                path: Some("expected_error.message".to_string()),
                equals_fixture: None,
                contains_fixture: None,
                exists: Some(true),
                metric: None,
                op: None,
                left: None,
                right: None,
                name: None,
            },
            VerifierAssertionRecord {
                kind: "json_path".to_string(),
                path: Some("claims_acquire.claims.0.session".to_string()),
                equals_fixture: Some("sender_session".to_string()),
                contains_fixture: None,
                exists: None,
                metric: None,
                op: None,
                left: None,
                right: None,
                name: None,
            },
        ],
        metrics: vec!["claim_count".to_string()],
        evidence_requirements: vec!["live_primary".to_string()],
        gate_target: "acceptable".to_string(),
        status: "declared".to_string(),
        lanes: vec!["nightly".to_string()],
        helper_hooks: Vec::new(),
    };

    let run = run_verifier_record(&verifier, &fixture, Some(&base_url))
        .await
        .expect("run hive adversarial verifier");

    assert_eq!(run.status, "passing");
    assert_eq!(run.gate_result, "acceptable");
    assert_eq!(
        run.metrics_observed
            .get("claim_count")
            .and_then(JsonValue::as_u64),
        Some(1)
    );
}

#[tokio::test]
async fn run_verifier_record_contains_hive_task_lane_collision() {
    let base_url = spawn_mock_runtime_server(MockRuntimeState::default(), false).await;
    let fixture = FixtureRecord {
        id: "fixture.hive-task-lane-adversarial-bundle.test".to_string(),
        kind: "bundle_fixture".to_string(),
        description: "hive task lane adversarial".to_string(),
        seed_files: Vec::new(),
        seed_config: json!({
            "project": "demo",
            "namespace": "main",
            "agent": "codex",
            "session": "sender",
            "workspace": "shared",
            "base_url": base_url
        }),
        seed_memories: Vec::new(),
        seed_events: Vec::new(),
        seed_sessions: vec!["sender".to_string(), "target".to_string()],
        seed_claims: Vec::new(),
        seed_vault: None,
        backend_mode: "normal".to_string(),
        isolation: "fresh_temp_dir".to_string(),
        cleanup_policy: "destroy".to_string(),
    };
    let verifier = VerifierRecord {
            id: "verifier.adversarial.hive-task-lane-collision".to_string(),
            name: "Hive task lane collision containment".to_string(),
            verifier_type: "adversarial".to_string(),
            pillar: "memory-continuity".to_string(),
            family: "coordination-hive".to_string(),
            subject_ids: vec![
                "feature.hive.tasks".to_string(),
                "journey.hive.transfer-assign".to_string(),
            ],
            fixture_id: fixture.id.clone(),
            baseline_modes: vec!["with_memd".to_string()],
            steps: vec![
                VerifierStepRecord {
                    kind: "helper".to_string(),
                    run: None,
                    name: Some("setup_target_lane_collision".to_string()),
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd tasks --output {{sender_bundle}} --upsert --task-id {{task.id}} --title \"Parser refactor\" --status in_progress --mode exclusive_write --scope task:{{task.id}}".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "cli_expect_error".to_string(),
                    run: Some("memd tasks --output {{sender_bundle}} --assign-to-session {{target_session}} --task-id {{task.id}}".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
            ],
            assertions: vec![VerifierAssertionRecord {
                kind: "json_path".to_string(),
                path: Some("expected_error.message".to_string()),
                equals_fixture: None,
                contains_fixture: None,
                exists: Some(true),
                metric: None,
                op: None,
                left: None,
                right: None,
                name: None,
            }],
            metrics: vec!["task_count".to_string(), "expected_error_count".to_string()],
            evidence_requirements: vec!["live_primary".to_string()],
            gate_target: "acceptable".to_string(),
            status: "declared".to_string(),
            lanes: vec!["nightly".to_string()],
            helper_hooks: vec!["setup_target_lane_collision".to_string()],
        };

    let run = run_verifier_record(&verifier, &fixture, Some(&base_url))
        .await
        .expect("run hive task lane adversarial verifier");

    assert_eq!(run.status, "passing");
    assert_eq!(run.gate_result, "acceptable");
    assert_eq!(
        run.metrics_observed
            .get("expected_error_count")
            .and_then(JsonValue::as_u64),
        Some(1)
    );
}

#[tokio::test]
async fn run_verifier_record_contains_hive_message_lane_collision() {
    let base_url = spawn_mock_runtime_server(MockRuntimeState::default(), false).await;
    let fixture = FixtureRecord {
        id: "fixture.hive-message-lane-adversarial-bundle.test".to_string(),
        kind: "bundle_fixture".to_string(),
        description: "hive message lane adversarial".to_string(),
        seed_files: Vec::new(),
        seed_config: json!({
            "project": "demo",
            "namespace": "main",
            "agent": "codex",
            "session": "sender",
            "workspace": "shared",
            "base_url": base_url
        }),
        seed_memories: Vec::new(),
        seed_events: Vec::new(),
        seed_sessions: vec!["sender".to_string(), "target".to_string()],
        seed_claims: Vec::new(),
        seed_vault: None,
        backend_mode: "normal".to_string(),
        isolation: "fresh_temp_dir".to_string(),
        cleanup_policy: "destroy".to_string(),
    };
    let verifier = VerifierRecord {
            id: "verifier.adversarial.hive-message-lane-collision".to_string(),
            name: "Hive message lane collision containment".to_string(),
            verifier_type: "adversarial".to_string(),
            pillar: "memory-continuity".to_string(),
            family: "coordination-hive".to_string(),
            subject_ids: vec![
                "feature.hive.messages".to_string(),
                "journey.hive.transfer-assign".to_string(),
            ],
            fixture_id: fixture.id.clone(),
            baseline_modes: vec!["with_memd".to_string()],
            steps: vec![
                VerifierStepRecord {
                    kind: "helper".to_string(),
                    run: None,
                    name: Some("setup_target_lane_collision".to_string()),
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "cli_expect_error".to_string(),
                    run: Some("memd messages --output {{sender_bundle}} --send --target-session {{target_session}} --kind handoff --content \"Pick up {{task.id}}\"".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
            ],
            assertions: vec![VerifierAssertionRecord {
                kind: "json_path".to_string(),
                path: Some("expected_error.message".to_string()),
                equals_fixture: None,
                contains_fixture: None,
                exists: Some(true),
                metric: None,
                op: None,
                left: None,
                right: None,
                name: None,
            }],
            metrics: vec!["expected_error_count".to_string()],
            evidence_requirements: vec!["live_primary".to_string()],
            gate_target: "acceptable".to_string(),
            status: "declared".to_string(),
            lanes: vec!["nightly".to_string()],
            helper_hooks: vec!["setup_target_lane_collision".to_string()],
        };

    let run = run_verifier_record(&verifier, &fixture, Some(&base_url))
        .await
        .expect("run hive message lane adversarial verifier");

    assert_eq!(run.status, "passing");
    assert_eq!(run.gate_result, "acceptable");
    assert_eq!(
        run.metrics_observed
            .get("expected_error_count")
            .and_then(JsonValue::as_u64),
        Some(1)
    );
}

#[tokio::test]
async fn run_verify_feature_command_executes_seeded_handoff_verifier() {
    let dir = std::env::temp_dir().join(format!(
        "memd-verify-handoff-command-{}",
        uuid::Uuid::new_v4()
    ));
    let repo_root = dir.join("repo");
    let output = repo_root.join(".memd");
    fs::create_dir_all(repo_root.join(".git")).expect("create git dir");
    fs::create_dir_all(&output).expect("create output dir");
    write_test_benchmark_registry(&repo_root);

    let report = run_verify_feature_command(&VerifyFeatureArgs {
        feature_id: "feature.bundle.handoff".to_string(),
        output: output.clone(),
        summary: false,
    })
    .await
    .expect("run verify handoff feature command");

    assert_eq!(report.subject.as_deref(), Some("feature.bundle.handoff"));
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding == "verifier_run_status=passing")
    );

    fs::remove_dir_all(dir).expect("cleanup verify handoff dir");
}

#[tokio::test]
async fn run_verify_feature_command_executes_seeded_verifier() {
    let dir = std::env::temp_dir().join(format!(
        "memd-verify-feature-command-{}",
        uuid::Uuid::new_v4()
    ));
    let repo_root = dir.join("repo");
    let output = repo_root.join(".memd");
    fs::create_dir_all(repo_root.join(".git")).expect("create git dir");
    fs::create_dir_all(&output).expect("create output dir");
    write_test_benchmark_registry(&repo_root);

    let report = run_verify_feature_command(&VerifyFeatureArgs {
        feature_id: "feature.bundle.resume".to_string(),
        output: output.clone(),
        summary: false,
    })
    .await
    .expect("run verify feature command");

    assert_eq!(report.subject.as_deref(), Some("feature.bundle.resume"));
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding == "verifier_run_status=passing")
    );
    assert!(
        verification_reports_dir(&output)
            .join("latest.json")
            .exists()
    );

    fs::remove_dir_all(dir).expect("cleanup verify feature dir");
}

#[tokio::test]
async fn run_verify_compare_command_executes_seeded_verifier() {
    let dir = std::env::temp_dir().join(format!(
        "memd-verify-compare-command-{}",
        uuid::Uuid::new_v4()
    ));
    let repo_root = dir.join("repo");
    let output = repo_root.join(".memd");
    fs::create_dir_all(repo_root.join(".git")).expect("create git dir");
    fs::create_dir_all(&output).expect("create output dir");
    write_test_benchmark_registry(&repo_root);

    let report = run_verify_compare_command(&VerifyCompareArgs {
        verifier_id: "verifier.compare.resume-no-memd-vs-with-memd".to_string(),
        output: output.clone(),
        summary: false,
    })
    .await
    .expect("run verify compare command");

    assert_eq!(
        report.subject.as_deref(),
        Some("verifier.compare.resume-no-memd-vs-with-memd")
    );
    assert_eq!(report.baseline.as_deref(), Some("no_mempath,with_memd"));
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding == "gate_result=strong")
    );
    assert!(
        verification_reports_dir(&output)
            .join("latest.json")
            .exists()
    );

    fs::remove_dir_all(dir).expect("cleanup verify compare dir");
}

#[tokio::test]
async fn run_verify_journey_command_executes_seeded_hive_journey() {
    let dir = std::env::temp_dir().join(format!(
        "memd-verify-journey-command-{}",
        uuid::Uuid::new_v4()
    ));
    let repo_root = dir.join("repo");
    let output = repo_root.join(".memd");
    fs::create_dir_all(repo_root.join(".git")).expect("create git dir");
    fs::create_dir_all(&output).expect("create output dir");
    write_test_benchmark_registry(&repo_root);

    let state = MockRuntimeState::default();
    let base_url = spawn_mock_runtime_server(state, false).await;
    write_test_bundle_config(&output, &base_url);

    let report = run_verify_journey_command(&VerifyJourneyArgs {
        journey_id: "journey.hive.transfer-assign".to_string(),
        output: output.clone(),
        summary: false,
    })
    .await
    .expect("run verify journey command");

    assert_eq!(
        report.subject.as_deref(),
        Some("journey.hive.transfer-assign")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding == "verifier_run_status=passing")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding == "gate_result=strong")
    );

    fs::remove_dir_all(dir).expect("cleanup verify journey dir");
}

#[tokio::test]
async fn run_verify_adversarial_command_executes_seeded_hive_collision_verifier() {
    let dir = std::env::temp_dir().join(format!(
        "memd-verify-adversarial-command-{}",
        uuid::Uuid::new_v4()
    ));
    let repo_root = dir.join("repo");
    let output = repo_root.join(".memd");
    fs::create_dir_all(repo_root.join(".git")).expect("create git dir");
    fs::create_dir_all(&output).expect("create output dir");
    write_test_benchmark_registry(&repo_root);

    let state = MockRuntimeState::default();
    let base_url = spawn_mock_runtime_server(state, false).await;
    write_test_bundle_config(&output, &base_url);

    let report = run_verify_adversarial_command(&VerifyAdversarialArgs {
        verifier_id: "verifier.adversarial.hive-claim-collision".to_string(),
        output: output.clone(),
        summary: false,
    })
    .await
    .expect("run verify adversarial command");

    assert_eq!(
        report.subject.as_deref(),
        Some("verifier.adversarial.hive-claim-collision")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding == "verifier_run_status=passing")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding == "gate_result=acceptable")
    );

    fs::remove_dir_all(dir).expect("cleanup verify adversarial dir");
}

#[tokio::test]
async fn run_verify_adversarial_command_executes_seeded_hive_task_lane_collision_verifier() {
    let dir = std::env::temp_dir().join(format!(
        "memd-verify-adversarial-task-lane-command-{}",
        uuid::Uuid::new_v4()
    ));
    let repo_root = dir.join("repo");
    let output = repo_root.join(".memd");
    fs::create_dir_all(repo_root.join(".git")).expect("create git dir");
    fs::create_dir_all(&output).expect("create output dir");
    write_test_benchmark_registry(&repo_root);

    let state = MockRuntimeState::default();
    let base_url = spawn_mock_runtime_server(state, false).await;
    write_test_bundle_config(&output, &base_url);

    let report = run_verify_adversarial_command(&VerifyAdversarialArgs {
        verifier_id: "verifier.adversarial.hive-task-lane-collision".to_string(),
        output: output.clone(),
        summary: false,
    })
    .await
    .expect("run verify adversarial task lane command");

    assert_eq!(
        report.subject.as_deref(),
        Some("verifier.adversarial.hive-task-lane-collision")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding == "verifier_run_status=passing")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding == "gate_result=acceptable")
    );

    fs::remove_dir_all(dir).expect("cleanup verify adversarial task lane dir");
}

#[tokio::test]
async fn run_verify_adversarial_command_executes_seeded_hive_message_lane_collision_verifier() {
    let dir = std::env::temp_dir().join(format!(
        "memd-verify-adversarial-message-lane-command-{}",
        uuid::Uuid::new_v4()
    ));
    let repo_root = dir.join("repo");
    let output = repo_root.join(".memd");
    fs::create_dir_all(repo_root.join(".git")).expect("create git dir");
    fs::create_dir_all(&output).expect("create output dir");
    write_test_benchmark_registry(&repo_root);

    let state = MockRuntimeState::default();
    let base_url = spawn_mock_runtime_server(state, false).await;
    write_test_bundle_config(&output, &base_url);

    let report = run_verify_adversarial_command(&VerifyAdversarialArgs {
        verifier_id: "verifier.adversarial.hive-message-lane-collision".to_string(),
        output: output.clone(),
        summary: false,
    })
    .await
    .expect("run verify adversarial message lane command");

    assert_eq!(
        report.subject.as_deref(),
        Some("verifier.adversarial.hive-message-lane-collision")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding == "verifier_run_status=passing")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding == "gate_result=acceptable")
    );

    fs::remove_dir_all(dir).expect("cleanup verify adversarial message lane dir");
}

#[test]
fn derived_only_evidence_caps_gate_at_fragile() {
    let gate = resolve_verifier_gate("acceptable", &["derived".to_string()], true, true, true);
    assert_eq!(gate, "fragile");
}

#[test]
fn comparative_loss_caps_gate_at_acceptable() {
    let gate = resolve_verifier_gate("strong", &["live_primary".to_string()], true, true, false);
    assert_eq!(gate, "acceptable");
}

#[tokio::test]
async fn nightly_sweep_fails_on_tier_zero_failure() {
    let report = run_verify_sweep_for_test("nightly", vec![test_failing_tier_zero_verifier()])
        .await
        .expect("run nightly sweep");
    assert!(!report.ok);
}

#[tokio::test]
async fn nightly_sweep_reports_noncritical_failures_without_failing() {
    let report = run_verify_sweep_for_test("nightly", vec![test_failing_tier_two_verifier()])
        .await
        .expect("run nightly sweep");
    assert!(report.ok);
    assert_eq!(report.failures.len(), 1);
}

#[tokio::test]
async fn run_feature_benchmark_command_scores_feature_inventory_and_writes_artifacts() {
    let dir = std::env::temp_dir().join(format!("memd-feature-benchmark-{}", uuid::Uuid::new_v4()));
    let repo_root = dir.join("repo");
    let output = repo_root.join(".memd");
    fs::create_dir_all(repo_root.join(".git")).expect("create git dir");
    fs::create_dir_all(&output).expect("create benchmark temp bundle");

    let state = MockRuntimeState::default();
    let base_url = spawn_mock_runtime_server(state, false).await;
    write_test_bundle_config(&output, &base_url);
    write_bundle_command_catalog_files(&output).expect("write command catalog");
    write_test_benchmark_registry(&repo_root);

    let snapshot = test_autoresearch_snapshot(
        false,
        vec!["keep the hive compact".to_string()],
        vec!["crates/memd-client/src/main.rs".to_string()],
    );
    write_bundle_memory_files(&output, &snapshot, None, false)
        .await
        .expect("write bundle memory files");
    refresh_live_bundle_event_pages(&output, &snapshot, None).expect("refresh event pages");

    let eval = high_scoring_eval(&output);
    write_bundle_eval_artifacts(&output, &eval).expect("write eval artifacts");
    let scenario = high_scoring_scenario(&output);
    write_scenario_artifacts(&output, &scenario).expect("write scenario artifacts");
    write_maintain_artifacts(
        &output,
        &MaintainReport {
            mode: "scan".to_string(),
            receipt_id: Some("maint-1".to_string()),
            compacted_items: 2,
            refreshed_items: 1,
            repaired_items: 0,
            findings: vec!["memory drift low".to_string()],
            generated_at: Utc::now(),
        },
    )
    .expect("write maintain artifacts");

    let experiment = test_experiment_report(&output, true, false, 92, 100, Utc::now());
    let experiments_dir = experiment_reports_dir(&output);
    fs::create_dir_all(&experiments_dir).expect("create experiments dir");
    fs::write(
        experiments_dir.join("latest.json"),
        serde_json::to_string_pretty(&experiment).expect("serialize experiment") + "\n",
    )
    .expect("write experiment latest");

    let evolution_dir = evolution_reports_dir(&output);
    fs::create_dir_all(&evolution_dir).expect("create evolution dir");
    let proposal = EvolutionProposalReport {
        bundle_root: output.display().to_string(),
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        agent: Some("codex".to_string()),
        session: Some("codex-a".to_string()),
        workspace: Some("shared".to_string()),
        visibility: Some("workspace".to_string()),
        proposal_id: "prop-1".to_string(),
        scenario: Some("self_evolution".to_string()),
        topic: "feature benchmark".to_string(),
        branch: "auto/evolution/feature-benchmark".to_string(),
        state: "accepted_proposal".to_string(),
        scope_class: "low_risk_evaluation_code".to_string(),
        scope_gate: "auto_merge".to_string(),
        authority_tier: "bundle".to_string(),
        allowed_write_surface: vec!["crates/memd-client/src/main.rs".to_string()],
        merge_eligible: true,
        durable_truth: false,
        accepted: true,
        restored: false,
        composite_score: 92,
        composite_max: 100,
        evidence: vec!["benchmark gate passed".to_string()],
        scope_reasons: vec!["bounded change".to_string()],
        generated_at: Utc::now(),
        durability_due_at: None,
    };
    fs::write(
        evolution_dir.join("latest-proposal.json"),
        serde_json::to_string_pretty(&proposal).expect("serialize proposal") + "\n",
    )
    .expect("write proposal latest");
    let branch = EvolutionBranchManifest {
        proposal_id: "prop-1".to_string(),
        branch: "auto/evolution/feature-benchmark".to_string(),
        branch_prefix: "auto/evolution".to_string(),
        project_root: Some(output.display().to_string()),
        head_sha: Some("abc123".to_string()),
        base_branch: Some("main".to_string()),
        status: "ready".to_string(),
        merge_eligible: true,
        durable_truth: false,
        scope_class: "low_risk_evaluation_code".to_string(),
        scope_gate: "auto_merge".to_string(),
        generated_at: Utc::now(),
        notes: vec!["benchmark branch".to_string()],
    };
    fs::write(
        evolution_dir.join("latest-branch.json"),
        serde_json::to_string_pretty(&branch).expect("serialize branch") + "\n",
    )
    .expect("write branch latest");

    let report = run_feature_benchmark_command(
        &BenchmarkArgs {
            output: output.clone(),
            write: false,
            summary: false,
            subcommand: None,
        },
        &base_url,
    )
    .await
    .expect("run feature benchmark");

    assert_eq!(report.areas.len(), 10);
    assert!(report.score > 0);
    assert!(
        report
            .evidence
            .iter()
            .any(|item| item.contains("benchmark_registry root="))
    );
    assert!(
        report
            .areas
            .iter()
            .any(|area| area.slug == "coordination_hive" && area.score > 0)
    );
    assert!(report.areas.iter().any(|area| {
        area.slug == "core_memory"
            && area
                .evidence
                .iter()
                .any(|item| item.contains("memory_quality="))
    }));

    write_feature_benchmark_artifacts(&output, &report).expect("write benchmark artifacts");
    let benchmark_dir = feature_benchmark_reports_dir(&output);
    assert!(benchmark_dir.join("latest.json").exists());
    assert!(benchmark_dir.join("latest.md").exists());
    let (loaded_root, registry) = load_benchmark_registry_for_output(&output)
        .expect("load benchmark registry")
        .expect("registry present");
    write_benchmark_registry_docs(&loaded_root, &registry, &report)
        .expect("write benchmark registry docs");
    assert!(benchmark_registry_markdown_path(&loaded_root, "BENCHMARKS.md").exists());
    assert!(benchmark_registry_markdown_path(&loaded_root, "LOOPS.md").exists());
    assert!(benchmark_registry_markdown_path(&loaded_root, "COVERAGE.md").exists());
    assert!(benchmark_registry_markdown_path(&loaded_root, "SCORES.md").exists());
    assert!(benchmark_registry_markdown_path(&loaded_root, "MORNING.md").exists());
    assert!(
        benchmark_telemetry_dir(&output)
            .join("latest.json")
            .exists()
    );
    assert!(benchmark_telemetry_dir(&output).join("latest.md").exists());
    fs::remove_dir_all(dir).expect("cleanup benchmark dir");
}

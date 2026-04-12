use super::*;

#[tokio::test]
pub(crate) async fn load_benchmark_registry_from_output_reads_repo_root_registry() {
    let dir = std::env::temp_dir().join(format!(
        "memd-benchmark-registry-load-{}",
        uuid::Uuid::new_v4()
    ));
    let repo_root = dir.join("repo");
    let output = repo_root.join(".memd");
    fs::create_dir_all(repo_root.join(".git")).expect("create git dir");
    fs::create_dir_all(&output).expect("create output dir");
    write_test_benchmark_registry(&repo_root);

    let (loaded_root, registry) = load_benchmark_registry_for_output(&output)
        .expect("load benchmark registry")
        .expect("registry should be discovered");
    assert_eq!(loaded_root, repo_root);
    assert_eq!(registry.version, "v1");
    assert!(!registry.features.is_empty());
    assert!(!registry.loops.is_empty());

    fs::remove_dir_all(dir).expect("cleanup benchmark registry load dir");
}

pub(crate) fn high_scoring_eval(output: &Path) -> BundleEvalResponse {
    BundleEvalResponse {
        bundle_root: output.display().to_string(),
        project: Some("demo".to_string()),
        namespace: None,
        agent: Some("codex".to_string()),
        workspace: None,
        visibility: None,
        status: "strong".to_string(),
        score: 92,
        working_records: 2,
        context_records: 2,
        rehydration_items: 1,
        inbox_items: 0,
        workspace_lanes: 0,
        semantic_hits: 0,
        findings: Vec::new(),
        baseline_score: Some(88),
        score_delta: Some(4),
        changes: vec!["score 88 -> 92".to_string()],
        recommendations: vec!["keep the current lane tight".to_string()],
    }
}

pub(crate) fn high_scoring_scenario(output: &Path) -> ScenarioReport {
    ScenarioReport {
        bundle_root: output.display().to_string(),
        project: Some("demo".to_string()),
        namespace: None,
        agent: Some("codex".to_string()),
        session: None,
        workspace: None,
        visibility: None,
        scenario: "bundle_health".to_string(),
        score: 96,
        max_score: 100,
        checks: vec![
            ScenarioCheck {
                name: "runtime_config".to_string(),
                status: "pass".to_string(),
                points: 28,
                details: "bundle runtime config is available".to_string(),
            },
            ScenarioCheck {
                name: "resume_signal".to_string(),
                status: "pass".to_string(),
                points: 22,
                details: "resume signal pressure is low".to_string(),
            },
        ],
        passed_checks: 2,
        failed_checks: 0,
        findings: Vec::new(),
        next_actions: vec!["keep the current lane tight".to_string()],
        evidence: vec!["scenario baseline is healthy".to_string()],
        generated_at: Utc::now(),
        completed_at: Utc::now(),
    }
}

pub(crate) fn test_improvement_report(
    output: &Path,
    completed_at: DateTime<Utc>,
) -> ImprovementReport {
    ImprovementReport {
        bundle_root: output.display().to_string(),
        project: Some("demo".to_string()),
        namespace: None,
        agent: Some("codex".to_string()),
        session: None,
        workspace: None,
        visibility: None,
        max_iterations: 1,
        apply: false,
        started_at: completed_at,
        completed_at,
        converged: true,
        initial_gap: None,
        final_gap: None,
        final_changes: Vec::new(),
        iterations: Vec::new(),
    }
}

pub(crate) fn test_composite_report(
    output: &Path,
    score: u8,
    max_score: u8,
    completed_at: DateTime<Utc>,
) -> CompositeReport {
    CompositeReport {
        bundle_root: output.display().to_string(),
        project: Some("demo".to_string()),
        namespace: None,
        agent: Some("codex".to_string()),
        session: None,
        workspace: None,
        visibility: None,
        scenario: Some("self_evolution".to_string()),
        score,
        max_score,
        dimensions: Vec::new(),
        gates: Vec::new(),
        findings: Vec::new(),
        recommendations: Vec::new(),
        evidence: Vec::new(),
        generated_at: completed_at,
        completed_at,
    }
}

pub(crate) fn test_experiment_report(
    output: &Path,
    accepted: bool,
    restored: bool,
    score: u8,
    max_score: u8,
    completed_at: DateTime<Utc>,
) -> ExperimentReport {
    ExperimentReport {
        bundle_root: output.display().to_string(),
        project: Some("demo".to_string()),
        namespace: None,
        agent: Some("codex".to_string()),
        session: None,
        workspace: None,
        visibility: None,
        max_iterations: 1,
        accept_below: 80,
        apply: false,
        consolidate: false,
        accepted,
        restored,
        started_at: completed_at,
        completed_at,
        improvement: test_improvement_report(output, completed_at),
        composite: test_composite_report(output, score, max_score, completed_at),
        trail: Vec::new(),
        learnings: vec!["tighten the loop".to_string()],
        findings: Vec::new(),
        recommendations: Vec::new(),
        evidence: Vec::new(),
        evolution: None,
    }
}

pub(crate) fn write_test_bundle_config(output: &Path, base_url: &str) {
    fs::create_dir_all(output).expect("create bundle root");
    fs::write(
        output.join("config.json"),
        format!(
            r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "{}",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task"
}}
"#,
            base_url
        ),
    )
    .expect("write bundle config");
}

pub(crate) fn test_benchmark_registry() -> BenchmarkRegistry {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let registry_path = manifest_dir
        .join("../..")
        .join("docs/verification/benchmark-registry.json");
    let registry_json = fs::read_to_string(&registry_path).expect("read benchmark registry");
    serde_json::from_str(&registry_json).expect("parse benchmark registry")
}

pub(crate) fn test_continuity_fixture_record() -> FixtureRecord {
    FixtureRecord {
        id: "fixture.continuity_bundle".to_string(),
        kind: "bundle_fixture".to_string(),
        description: "continuity".to_string(),
        seed_files: Vec::new(),
        seed_config: json!({
            "project": "memd",
            "namespace": "main",
            "agent": "codex"
        }),
        seed_memories: Vec::new(),
        seed_events: Vec::new(),
        seed_sessions: Vec::new(),
        seed_claims: Vec::new(),
        seed_vault: None,
        backend_mode: "normal".to_string(),
        isolation: "fresh_temp_dir".to_string(),
        cleanup_policy: "destroy".to_string(),
    }
}

pub(crate) fn test_failing_tier_zero_verifier() -> VerifierRecord {
    VerifierRecord {
        id: "verifier.feature.session_continuity.failing".to_string(),
        name: "Session continuity feature failing".to_string(),
        verifier_type: "feature_contract".to_string(),
        pillar: "memory-continuity".to_string(),
        family: "bundle-runtime".to_string(),
        subject_ids: vec!["feature.session_continuity".to_string()],
        fixture_id: "fixture.continuity_bundle".to_string(),
        baseline_modes: vec!["with_memd".to_string()],
        steps: Vec::new(),
        assertions: Vec::new(),
        metrics: vec!["prompt_tokens".to_string()],
        evidence_requirements: vec!["live_primary".to_string()],
        gate_target: "acceptable".to_string(),
        status: "declared".to_string(),
        lanes: vec!["nightly".to_string()],
        helper_hooks: vec!["force_fail".to_string()],
    }
}

pub(crate) fn test_failing_tier_two_verifier() -> VerifierRecord {
    VerifierRecord {
        id: "verifier.feature.noncritical.failing".to_string(),
        name: "Noncritical feature failing".to_string(),
        verifier_type: "feature_contract".to_string(),
        pillar: "visible-memory".to_string(),
        family: "visible-memory".to_string(),
        subject_ids: vec!["feature.noncritical.placeholder".to_string()],
        fixture_id: "fixture.continuity_bundle".to_string(),
        baseline_modes: vec!["with_memd".to_string()],
        steps: Vec::new(),
        assertions: Vec::new(),
        metrics: vec!["prompt_tokens".to_string()],
        evidence_requirements: vec!["live_primary".to_string()],
        gate_target: "acceptable".to_string(),
        status: "declared".to_string(),
        lanes: vec!["nightly".to_string()],
        helper_hooks: vec!["force_fail".to_string()],
    }
}

pub(crate) async fn run_verify_sweep_for_test(
    lane: &str,
    verifiers: Vec<VerifierRecord>,
) -> anyhow::Result<VerifySweepReport> {
    let dir = std::env::temp_dir().join(format!("memd-verify-sweep-{}", uuid::Uuid::new_v4()));
    let output = dir.join(".memd");
    fs::create_dir_all(&output).expect("create verify sweep output");

    let mut registry = test_benchmark_registry();
    registry.verifiers = verifiers;
    registry.fixtures = vec![test_continuity_fixture_record()];

    let report = execute_verify_sweep(&output, None, &registry, lane).await;
    fs::remove_dir_all(dir).expect("cleanup verify sweep dir");
    report
}

pub(crate) fn write_test_benchmark_registry(repo_root: &Path) {
    fs::create_dir_all(benchmark_registry_docs_dir(repo_root))
        .expect("create benchmark registry docs dir");
    fs::write(
        benchmark_registry_json_path(repo_root),
        serde_json::to_string_pretty(&test_benchmark_registry()).expect("serialize registry")
            + "\n",
    )
    .expect("write benchmark registry");
}

pub(crate) fn test_feature_benchmark_report(output: &Path) -> FeatureBenchmarkReport {
    FeatureBenchmarkReport {
        bundle_root: output.display().to_string(),
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        agent: Some("codex".to_string()),
        session: Some("codex-a".to_string()),
        workspace: Some("shared".to_string()),
        visibility: Some("workspace".to_string()),
        score: 88,
        max_score: 100,
        command_count: 4,
        skill_count: 2,
        pack_count: 2,
        memory_pages: 3,
        event_count: 5,
        areas: vec![FeatureBenchmarkArea {
            slug: "core_memory".to_string(),
            name: "Core Memory".to_string(),
            score: 90,
            max_score: 100,
            status: "pass".to_string(),
            implemented_commands: 4,
            expected_commands: 4,
            evidence: vec!["memory_quality=90".to_string()],
            recommendations: vec!["keep the current lane tight".to_string()],
        }],
        evidence: vec!["benchmark_registry root=repo".to_string()],
        recommendations: vec!["keep the current lane tight".to_string()],
        generated_at: Utc::now(),
        completed_at: Utc::now(),
    }
}

pub(crate) fn write_test_bundle_heartbeat(output: &Path, state: &BundleHeartbeatState) {
    fs::create_dir_all(output.join("state")).expect("create bundle state dir");
    fs::write(
        bundle_heartbeat_state_path(output),
        serde_json::to_string_pretty(state).expect("serialize heartbeat") + "\n",
    )
    .expect("write heartbeat");
}

pub(crate) fn init_test_git_repo(root: &Path) {
    let _ = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("init")
        .status()
        .expect("init git repo");
    let _ = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("config")
        .arg("user.email")
        .arg("memd@example.com")
        .status()
        .expect("git user email");
    let _ = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("config")
        .arg("user.name")
        .arg("memd")
        .status()
        .expect("git user name");
    let _ = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("add")
        .arg(".")
        .status()
        .expect("git add");
    let _ = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("commit")
        .arg("-m")
        .arg("init")
        .status()
        .expect("git commit");
}

pub(crate) fn checkout_test_branch(root: &Path, branch: &str) {
    let _ = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("checkout")
        .arg("-b")
        .arg(branch)
        .status()
        .expect("checkout branch");
}

pub(crate) fn test_hive_heartbeat_state(
    session: &str,
    agent: &str,
    tab_id: &str,
    status: &str,
    last_seen: DateTime<Utc>,
) -> BundleHeartbeatState {
    BundleHeartbeatState {
        session: Some(session.to_string()),
        agent: Some(agent.to_string()),
        effective_agent: Some(compose_agent_identity(agent, Some(session))),
        tab_id: Some(tab_id.to_string()),
        hive_system: Some(agent.to_string()),
        hive_role: Some("agent".to_string()),
        worker_name: Some(agent.to_string()),
        display_name: None,
        role: Some("agent".to_string()),
        capabilities: vec!["memory".to_string(), "coordination".to_string()],
        hive_groups: vec!["openclaw-stack".to_string()],
        lane_id: Some("/tmp/memd".to_string()),
        hive_group_goal: None,
        authority: Some("participant".to_string()),
        heartbeat_model: Some(default_heartbeat_model()),
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        workspace: Some("shared".to_string()),
        repo_root: Some("/tmp/memd".to_string()),
        worktree_root: Some("/tmp/memd".to_string()),
        branch: Some("feature/test-bee".to_string()),
        base_branch: Some("main".to_string()),
        visibility: Some("workspace".to_string()),
        base_url: None,
        base_url_healthy: None,
        host: Some("workstation".to_string()),
        pid: Some(4242),
        topic_claim: None,
        scope_claims: Vec::new(),
        task_id: None,
        focus: Some("Keep the shared hive healthy".to_string()),
        pressure: Some("Avoid claim collisions".to_string()),
        next_recovery: None,
        next_action: None,
        needs_help: false,
        needs_review: false,
        handoff_state: None,
        confidence: None,
        risk: None,
        status: status.to_string(),
        last_seen,
        authority_mode: Some("shared".to_string()),
        authority_degraded: false,
        ..BundleHeartbeatState::default()
    }
}

use super::*;
pub(crate) struct MaterializedFixture {
    pub(crate) _fixture_id: String,
    pub(crate) _root: TempDir,
    pub(crate) bundle_root: PathBuf,
    pub(crate) fixture_vars: BTreeMap<String, String>,
    pub(crate) _session_bundles: BTreeMap<String, PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct VerifierRunRecord {
    pub(crate) verifier_id: String,
    pub(crate) status: String,
    pub(crate) gate_result: String,
    pub(crate) evidence_ids: Vec<String>,
    pub(crate) metrics_observed: BTreeMap<String, JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct VerifySweepReport {
    pub(crate) lane: String,
    pub(crate) ok: bool,
    pub(crate) total: usize,
    pub(crate) passed: usize,
    pub(crate) failures: Vec<String>,
    pub(crate) runs: Vec<VerifierRunRecord>,
    pub(crate) bundle_root: String,
    pub(crate) repo_root: Option<String>,
}

pub(crate) fn verification_reports_dir(output: &Path) -> PathBuf {
    output.join("verification")
}

pub(crate) fn verification_runs_dir(output: &Path) -> PathBuf {
    verification_reports_dir(output).join("runs")
}

pub(crate) fn verification_evidence_dir(output: &Path) -> PathBuf {
    verification_reports_dir(output).join("evidence")
}

pub(crate) fn fixture_seed_object(
    fixture: &FixtureRecord,
) -> anyhow::Result<serde_json::Map<String, JsonValue>> {
    fixture
        .seed_config
        .as_object()
        .cloned()
        .context("fixture seed_config must be a JSON object")
}

pub(crate) fn fixture_seed_string(
    seed: &serde_json::Map<String, JsonValue>,
    key: &str,
    default: &str,
) -> String {
    seed.get(key)
        .and_then(JsonValue::as_str)
        .unwrap_or(default)
        .to_string()
}

pub(crate) fn fixture_seed_defaults(
    fixture: &FixtureRecord,
    base_url_override: Option<&str>,
) -> anyhow::Result<serde_json::Map<String, JsonValue>> {
    let mut seed = fixture_seed_object(fixture)?;
    let defaults = [
        ("project", "memd"),
        ("namespace", "main"),
        ("agent", "codex"),
        ("session", "verifier-fixture"),
        ("workspace", "shared"),
        ("visibility", "workspace"),
        ("route", "auto"),
        ("intent", "current_task"),
        ("base_url", "http://127.0.0.1:59999"),
    ];
    for (key, value) in defaults {
        seed.entry(key.to_string())
            .or_insert_with(|| JsonValue::String(value.to_string()));
    }
    if let Some(base_url) = base_url_override.filter(|value| !value.trim().is_empty()) {
        seed.insert(
            "base_url".to_string(),
            JsonValue::String(base_url.to_string()),
        );
    }
    Ok(seed)
}

pub(crate) fn build_fixture_vars(
    seed: &serde_json::Map<String, JsonValue>,
) -> BTreeMap<String, String> {
    let run_id = uuid::Uuid::new_v4().simple().to_string();
    let task_seed = fixture_seed_string(seed, "task_id", "task-current");
    let task_id = format!("{task_seed}-{}", &run_id[..8]);
    let next_action = fixture_seed_string(seed, "next_action", "resume next step");
    BTreeMap::from([
        ("run.id".to_string(), run_id),
        ("task.id".to_string(), task_id),
        ("task.next_action".to_string(), next_action),
    ])
}

pub(crate) fn build_fixture_resume_snapshot(
    seed: &serde_json::Map<String, JsonValue>,
    fixture_vars: &BTreeMap<String, String>,
) -> ResumeSnapshot {
    let project = fixture_seed_string(seed, "project", "memd");
    let namespace = fixture_seed_string(seed, "namespace", "main");
    let agent = fixture_seed_string(seed, "agent", "codex");
    let workspace = fixture_seed_string(seed, "workspace", "shared");
    let visibility = fixture_seed_string(seed, "visibility", "workspace");
    let route = fixture_seed_string(seed, "route", "auto");
    let intent = fixture_seed_string(seed, "intent", "current_task");
    let task_id = fixture_vars
        .get("task.id")
        .cloned()
        .unwrap_or_else(|| "task-current".to_string());
    let next_action = fixture_vars
        .get("task.next_action")
        .cloned()
        .unwrap_or_else(|| "resume next step".to_string());

    ResumeSnapshot {
        project: Some(project.clone()),
        namespace: Some(namespace.clone()),
        agent: Some(agent),
        workspace: Some(workspace.clone()),
        visibility: Some(visibility),
        route,
        intent,
        context: memd_schema::CompactContextResponse {
            route: memd_schema::RetrievalRoute::Auto,
            intent: memd_schema::RetrievalIntent::CurrentTask,
            retrieval_order: vec![memd_schema::MemoryScope::Project],
            records: vec![memd_schema::CompactMemoryRecord {
                id: uuid::Uuid::new_v4(),
                record: format!("current task {task_id}"),
            }],
        },
        working: memd_schema::WorkingMemoryResponse {
            route: memd_schema::RetrievalRoute::Auto,
            intent: memd_schema::RetrievalIntent::CurrentTask,
            retrieval_order: vec![memd_schema::MemoryScope::Project],
            budget_chars: 1600,
            used_chars: 120,
            remaining_chars: 1480,
            truncated: false,
            policy: memd_schema::WorkingMemoryPolicyState {
                admission_limit: 8,
                max_chars_per_item: 220,
                budget_chars: 1600,
                rehydration_limit: 4,
            },
            records: vec![memd_schema::CompactMemoryRecord {
                id: uuid::Uuid::new_v4(),
                record: format!("focus: {task_id}"),
            }],
            evicted: Vec::new(),
            rehydration_queue: vec![memd_schema::MemoryRehydrationRecord {
                id: None,
                kind: "task".to_string(),
                label: "next".to_string(),
                summary: next_action,
                reason: None,
                source_agent: None,
                source_system: None,
                source_path: None,
                source_quality: None,
                recorded_at: None,
            }],
            traces: Vec::new(),
            semantic_consolidation: None,
        },
        inbox: memd_schema::MemoryInboxResponse {
            route: memd_schema::RetrievalRoute::Auto,
            intent: memd_schema::RetrievalIntent::CurrentTask,
            items: vec![memd_schema::InboxMemoryItem {
                item: memd_schema::MemoryItem {
                    id: uuid::Uuid::new_v4(),
                    content: "keep continuity tight".to_string(),
                    redundancy_key: None,
                    belief_branch: None,
                    preferred: true,
                    kind: memd_schema::MemoryKind::Status,
                    scope: memd_schema::MemoryScope::Project,
                    project: Some(project.clone()),
                    namespace: Some(namespace.clone()),
                    workspace: Some(workspace.clone()),
                    visibility: memd_schema::MemoryVisibility::Workspace,
                    source_agent: None,
                    source_system: None,
                    source_path: None,
                    source_quality: None,
                    confidence: 0.9,
                    ttl_seconds: Some(86_400),
                    created_at: Utc::now(),
                    status: memd_schema::MemoryStatus::Active,
                    stage: memd_schema::MemoryStage::Candidate,
                    last_verified_at: None,
                    supersedes: Vec::new(),
                    updated_at: Utc::now(),
                    tags: vec!["continuity".to_string()],
                },
                reasons: vec!["fixture".to_string()],
            }],
        },
        workspaces: memd_schema::WorkspaceMemoryResponse {
            workspaces: vec![memd_schema::WorkspaceMemoryRecord {
                project: Some(project),
                namespace: Some(namespace),
                workspace: Some(workspace),
                visibility: memd_schema::MemoryVisibility::Workspace,
                item_count: 4,
                active_count: 3,
                candidate_count: 1,
                contested_count: 0,
                source_lane_count: 1,
                avg_confidence: 0.9,
                trust_score: 0.94,
                last_seen_at: None,
                tags: Vec::new(),
            }],
        },
        sources: memd_schema::SourceMemoryResponse {
            sources: Vec::new(),
        },
        semantic: None,
        claims: SessionClaimsState::default(),
        recent_repo_changes: vec!["crates/memd-client/src/main.rs".to_string()],
        change_summary: vec!["fixture continuity seeded".to_string()],
        resume_state_age_minutes: Some(1),
        refresh_recommended: false,
    }
}

pub(crate) fn verifier_resume_args(output: &Path) -> ResumeArgs {
    ResumeArgs {
        output: output.to_path_buf(),
        project: None,
        namespace: None,
        agent: None,
        workspace: None,
        visibility: None,
        route: None,
        intent: None,
        limit: Some(8),
        rehydration_limit: Some(4),
        semantic: false,
        prompt: false,
        summary: false,
    }
}

pub(crate) fn verifier_handoff_args(output: &Path) -> HandoffArgs {
    HandoffArgs {
        output: output.to_path_buf(),
        target_session: None,
        project: None,
        namespace: None,
        agent: None,
        workspace: None,
        visibility: None,
        route: None,
        intent: None,
        limit: Some(8),
        rehydration_limit: Some(4),
        source_limit: Some(6),
        semantic: false,
        prompt: false,
        summary: false,
    }
}

pub(crate) fn seed_materialized_fixture(
    bundle_root: &Path,
    seed: &serde_json::Map<String, JsonValue>,
    fixture_vars: &BTreeMap<String, String>,
    fixture: &FixtureRecord,
) -> anyhow::Result<()> {
    let runtime_json = JsonValue::Object(seed.clone());
    fs::write(
        bundle_root.join("config.json"),
        serde_json::to_string_pretty(&runtime_json).context("serialize fixture config")? + "\n",
    )
    .with_context(|| format!("write {}", bundle_root.join("config.json").display()))?;
    fs::write(bundle_root.join("env"), "")
        .with_context(|| format!("write {}", bundle_root.join("env").display()))?;
    fs::write(bundle_root.join("env.ps1"), "")
        .with_context(|| format!("write {}", bundle_root.join("env.ps1").display()))?;

    let runtime = read_bundle_runtime_config(bundle_root)?
        .context("fixture runtime config missing after materialization")?;
    let base_url = runtime
        .base_url
        .as_deref()
        .unwrap_or("http://127.0.0.1:59999");
    let resume_args = verifier_resume_args(bundle_root);
    let resume_snapshot = build_fixture_resume_snapshot(seed, fixture_vars);
    let resume_key = build_resume_snapshot_cache_key(&resume_args, Some(&runtime), base_url);
    cache::write_resume_snapshot_cache(bundle_root, &resume_key, &resume_snapshot)
        .context("write fixture resume cache")?;
    write_bundle_resume_state(bundle_root, &resume_snapshot)
        .context("write fixture resume state")?;

    let handoff_args = verifier_handoff_args(bundle_root);
    let handoff = HandoffSnapshot {
        generated_at: Utc::now(),
        resume: resume_snapshot.clone(),
        sources: memd_schema::SourceMemoryResponse {
            sources: Vec::new(),
        },
        target_session: None,
        target_bundle: Some(bundle_root.display().to_string()),
    };
    let handoff_key = cache::build_turn_key(
        Some(&bundle_root.display().to_string()),
        None,
        Some("none"),
        "handoff",
        &format!(
            "resume_key={}|source_limit={}|target_session=none|target_bundle={}",
            build_resume_snapshot_cache_key(
                &ResumeArgs {
                    output: bundle_root.to_path_buf(),
                    project: handoff_args.project.clone(),
                    namespace: handoff_args.namespace.clone(),
                    agent: handoff_args.agent.clone(),
                    workspace: handoff_args.workspace.clone(),
                    visibility: handoff_args.visibility.clone(),
                    route: handoff_args.route.clone(),
                    intent: handoff_args.intent.clone(),
                    limit: handoff_args.limit,
                    rehydration_limit: handoff_args.rehydration_limit,
                    semantic: handoff_args.semantic,
                    prompt: false,
                    summary: false,
                },
                Some(&runtime),
                base_url
            ),
            handoff_args.source_limit.unwrap_or(6),
            bundle_root.display()
        ),
    );
    cache::write_handoff_snapshot_cache(bundle_root, &handoff_key, &handoff)
        .context("write fixture handoff cache")?;

    for seed_file in &fixture.seed_files {
        let destination = bundle_root.join(seed_file);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        let content = format!(
            "fixture={}\ntask_id={}\nnext_action={}\n",
            fixture.id,
            fixture_vars
                .get("task.id")
                .map(String::as_str)
                .unwrap_or("task-current"),
            fixture_vars
                .get("task.next_action")
                .map(String::as_str)
                .unwrap_or("resume next step")
        );
        fs::write(&destination, content)
            .with_context(|| format!("write {}", destination.display()))?;
    }
    Ok(())
}

pub(crate) async fn seed_materialized_fixture_sessions(
    materialized: &MaterializedFixture,
) -> anyhow::Result<()> {
    if materialized._session_bundles.is_empty() {
        return Ok(());
    }

    let Some(base_url) = materialized
        ._session_bundles
        .values()
        .find_map(|bundle_root| {
            read_bundle_runtime_config(bundle_root)
                .ok()
                .flatten()
                .and_then(|runtime| runtime.base_url)
        })
    else {
        return Ok(());
    };

    let client = MemdClient::new(base_url)?;
    for bundle_root in materialized._session_bundles.values() {
        let runtime = read_bundle_runtime_config(bundle_root)?
            .context("fixture session runtime config missing")?;
        let Some(session) = runtime.session.clone() else {
            continue;
        };
        let request = memd_schema::HiveSessionUpsertRequest {
            session,
            tab_id: runtime.tab_id.clone(),
            agent: runtime.agent.clone(),
            effective_agent: runtime
                .agent
                .as_deref()
                .map(|agent| compose_agent_identity(agent, runtime.session.as_deref())),
            hive_system: runtime.hive_system.clone(),
            hive_role: runtime.hive_role.clone(),
            worker_name: runtime.agent.clone(),
            display_name: runtime.agent.clone(),
            role: runtime.hive_role.clone().or(Some("agent".to_string())),
            capabilities: runtime.capabilities.clone(),
            hive_groups: runtime.hive_groups.clone(),
            lane_id: None,
            hive_group_goal: runtime.hive_group_goal.clone(),
            authority: runtime.authority.clone(),
            heartbeat_model: runtime.heartbeat_model.clone(),
            project: runtime.project.clone(),
            namespace: runtime.namespace.clone(),
            workspace: runtime.workspace.clone(),
            repo_root: None,
            worktree_root: None,
            branch: None,
            base_branch: None,
            visibility: runtime.visibility.clone(),
            base_url: runtime.base_url.clone(),
            base_url_healthy: Some(true),
            host: None,
            pid: None,
            topic_claim: None,
            scope_claims: Vec::new(),
            task_id: None,
            focus: None,
            pressure: None,
            next_recovery: None,
            status: Some("live".to_string()),
            next_action: None,
            working: None,
            touches: Vec::new(),
            blocked_by: Vec::new(),
            cowork_with: Vec::new(),
            handoff_target: None,
            offered_to: Vec::new(),
            needs_help: false,
            needs_review: false,
            handoff_state: None,
            confidence: None,
            risk: None,
        };
        client
            .upsert_hive_session(&request)
            .await
            .context("seed fixture hive session")?;
    }

    Ok(())
}

pub(crate) fn materialize_fixture(
    fixture: &FixtureRecord,
    base_url_override: Option<&str>,
) -> anyhow::Result<MaterializedFixture> {
    if fixture.kind != "bundle_fixture" {
        anyhow::bail!("unsupported fixture kind {}", fixture.kind);
    }
    if fixture.isolation != "fresh_temp_dir" {
        anyhow::bail!("unsupported fixture isolation {}", fixture.isolation);
    }

    let root = tempfile::tempdir().context("create fixture tempdir")?;
    let seed = fixture_seed_defaults(fixture, base_url_override)?;
    let mut fixture_vars = build_fixture_vars(&seed);
    let mut session_bundles = BTreeMap::new();

    let bundle_root = if fixture.seed_sessions.is_empty() {
        let bundle_root = root.path().join(".memd");
        fs::create_dir_all(&bundle_root)
            .with_context(|| format!("create {}", bundle_root.display()))?;
        seed_materialized_fixture(&bundle_root, &seed, &fixture_vars, fixture)?;
        bundle_root
    } else {
        let sessions_root = root.path().join("sessions");
        fs::create_dir_all(&sessions_root)
            .with_context(|| format!("create {}", sessions_root.display()))?;
        for (index, session_label) in fixture.seed_sessions.iter().enumerate() {
            let session_bundle = sessions_root.join(session_label).join(".memd");
            fs::create_dir_all(&session_bundle)
                .with_context(|| format!("create {}", session_bundle.display()))?;
            let mut session_seed = seed.clone();
            let session_agent = fixture_session_agent_name(session_label);
            let session_identity = format!(
                "{}-{}",
                session_label,
                uuid::Uuid::new_v4().simple().to_string()[..8].to_string()
            );
            session_seed.insert(
                "session".to_string(),
                JsonValue::String(session_identity.clone()),
            );
            if index > 0 || seed.get("agent").is_some() {
                session_seed.insert("agent".to_string(), JsonValue::String(session_agent));
            }
            seed_materialized_fixture(&session_bundle, &session_seed, &fixture_vars, fixture)?;
            fixture_vars.insert(
                format!("{session_label}_bundle"),
                session_bundle.display().to_string(),
            );
            fixture_vars.insert(format!("{session_label}_session"), session_identity.clone());
            session_bundles.insert(session_label.to_string(), session_bundle);
        }
        let primary_label = fixture
            .seed_sessions
            .first()
            .context("fixture seed_sessions missing primary session")?;
        if let Some(primary_session) = fixture_vars
            .get(&format!("{primary_label}_session"))
            .cloned()
        {
            fixture_vars.insert("primary_session".to_string(), primary_session);
        }
        if let Some(path) = session_bundles.get(primary_label) {
            fixture_vars.insert("sender_bundle".to_string(), path.display().to_string());
        }
        if let Some(target_label) = fixture.seed_sessions.get(1) {
            if let Some(target_session) = fixture_vars
                .get(&format!("{target_label}_session"))
                .cloned()
            {
                fixture_vars.insert("target_session".to_string(), target_session);
            }
            if let Some(path) = session_bundles.get(target_label) {
                fixture_vars.insert("target_bundle".to_string(), path.display().to_string());
            }
        }
        session_bundles
            .get(primary_label)
            .cloned()
            .context("fixture primary session bundle missing")?
    };

    Ok(MaterializedFixture {
        _fixture_id: fixture.id.clone(),
        _root: root,
        bundle_root,
        fixture_vars,
        _session_bundles: session_bundles,
    })
}

pub(crate) fn sanitize_verifier_artifact_name(id: &str) -> String {
    id.chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => ch,
            _ => '-',
        })
        .collect()
}

pub(crate) fn fixture_session_agent_name(session_label: &str) -> String {
    let words = session_label
        .split(['-', '_'])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            let mut chars = value.chars();
            match chars.next() {
                Some(first) => {
                    format!(
                        "{}{}",
                        first.to_uppercase(),
                        chars.as_str().to_ascii_lowercase()
                    )
                }
                None => String::new(),
            }
        })
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if words.is_empty() {
        "Codex".to_string()
    } else {
        words.join(" ")
    }
}

pub(crate) fn write_verifier_run_artifacts(
    output: &Path,
    run: &VerifierRunRecord,
    evidence_payload: &JsonValue,
) -> anyhow::Result<()> {
    fs::create_dir_all(verification_reports_dir(output))
        .with_context(|| format!("create {}", verification_reports_dir(output).display()))?;
    fs::create_dir_all(verification_runs_dir(output))
        .with_context(|| format!("create {}", verification_runs_dir(output).display()))?;
    fs::create_dir_all(verification_evidence_dir(output))
        .with_context(|| format!("create {}", verification_evidence_dir(output).display()))?;

    let latest_path = verification_reports_dir(output).join("latest.json");
    fs::write(
        &latest_path,
        serde_json::to_string_pretty(run).context("serialize verifier latest report")? + "\n",
    )
    .with_context(|| format!("write {}", latest_path.display()))?;

    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ");
    let run_path = verification_runs_dir(output).join(format!(
        "{}-{}.json",
        timestamp,
        sanitize_verifier_artifact_name(&run.verifier_id)
    ));
    fs::write(
        &run_path,
        serde_json::to_string_pretty(run).context("serialize verifier run report")? + "\n",
    )
    .with_context(|| format!("write {}", run_path.display()))?;

    for evidence_id in &run.evidence_ids {
        let evidence_path = verification_evidence_dir(output).join(format!(
            "{}.json",
            sanitize_verifier_artifact_name(evidence_id)
        ));
        fs::write(
            &evidence_path,
            serde_json::to_string_pretty(evidence_payload).context("serialize evidence payload")?
                + "\n",
        )
        .with_context(|| format!("write {}", evidence_path.display()))?;
    }

    Ok(())
}

pub(crate) fn resolve_verifier_gate(
    requested_gate: &str,
    evidence_tiers: &[String],
    assertions_passed: bool,
    continuity_ok: bool,
    comparative_win: bool,
) -> String {
    if !assertions_passed {
        return "broken".to_string();
    }
    if !continuity_ok {
        return "fragile".to_string();
    }
    if !evidence_tiers.is_empty() && evidence_tiers.iter().all(|tier| tier == "derived") {
        return "fragile".to_string();
    }
    if !comparative_win && requested_gate != "acceptable" {
        return "acceptable".to_string();
    }
    requested_gate.to_string()
}

pub(crate) fn verifier_assertions_pass(verifier: &VerifierRecord) -> bool {
    !verifier
        .helper_hooks
        .iter()
        .any(|hook| hook == "force_fail" || hook == "test:force_fail")
}

pub(crate) fn verifier_continuity_ok(verifier: &VerifierRecord) -> bool {
    !verifier
        .helper_hooks
        .iter()
        .any(|hook| hook == "force_continuity_fail" || hook == "test:force_continuity_fail")
}

pub(crate) fn verifier_comparative_win(verifier: &VerifierRecord) -> bool {
    !verifier
        .helper_hooks
        .iter()
        .any(|hook| hook == "force_compare_loss" || hook == "test:force_compare_loss")
}

#[derive(Debug, Default)]
pub(crate) struct VerifierExecutionState {
    pub(crate) outputs: BTreeMap<String, JsonValue>,
    pub(crate) metrics: BTreeMap<String, JsonValue>,
    pub(crate) baselines: BTreeMap<String, BaselineMetrics>,
    pub(crate) comparative_report: Option<NoMemdDeltaReport>,
}

pub(crate) fn render_verifier_command_template(
    template: &str,
    materialized: &MaterializedFixture,
    state: &VerifierExecutionState,
) -> String {
    let mut expanded = template.to_string();
    expanded = expanded.replace(
        "{{bundle}}",
        &materialized.bundle_root.display().to_string(),
    );
    for (key, value) in &materialized.fixture_vars {
        expanded = expanded.replace(&format!("{{{{{key}}}}}"), value);
    }
    for (key, value) in &state.outputs {
        if let Some(value) = value.as_str() {
            expanded = expanded.replace(&format!("{{{{{key}}}}}"), value);
        }
    }
    expanded
}

pub(crate) fn build_resume_step_output(
    snapshot: &ResumeSnapshot,
    fixture_vars: &BTreeMap<String, String>,
) -> JsonValue {
    let mut value = serde_json::to_value(snapshot).unwrap_or_else(|_| json!({}));
    if let Some(object) = value.as_object_mut() {
        object.insert(
            "current_task".to_string(),
            json!({
                "id": fixture_vars.get("task.id").cloned().unwrap_or_else(|| "task-current".to_string()),
                "next_action": fixture_vars.get("task.next_action").cloned().unwrap_or_else(|| "resume next step".to_string()),
            }),
        );
    }
    value
}

pub(crate) fn build_handoff_step_output(
    snapshot: &HandoffSnapshot,
    fixture_vars: &BTreeMap<String, String>,
) -> JsonValue {
    let mut value = serde_json::to_value(snapshot).unwrap_or_else(|_| json!({}));
    if let Some(object) = value.as_object_mut() {
        object.insert(
            "current_task".to_string(),
            json!({
                "id": fixture_vars.get("task.id").cloned().unwrap_or_else(|| "task-current".to_string()),
                "next_action": fixture_vars.get("task.next_action").cloned().unwrap_or_else(|| "resume next step".to_string()),
            }),
        );
    }
    value
}

pub(crate) fn verifier_baseline_metrics(name: &str) -> Option<BaselineMetrics> {
    match name {
        "no_mempath" | "no_memd" => Some(BaselineMetrics {
            prompt_tokens: 1600,
            reread_count: 4,
            reconstruction_steps: 4,
        }),
        "with_memd" => Some(BaselineMetrics {
            prompt_tokens: 1100,
            reread_count: 1,
            reconstruction_steps: 1,
        }),
        "with_memd_semantic" => Some(BaselineMetrics {
            prompt_tokens: 1200,
            reread_count: 1,
            reconstruction_steps: 1,
        }),
        _ => None,
    }
}

use super::*;

pub(crate) fn verifier_metric_from_baseline(
    metrics: &BaselineMetrics,
    metric: &str,
) -> Option<JsonValue> {
    match metric {
        "prompt_tokens" => Some(json!(metrics.prompt_tokens)),
        "rereads" | "reread_count" => Some(json!(metrics.reread_count)),
        "reconstruction_steps" => Some(json!(metrics.reconstruction_steps)),
        _ => None,
    }
}

pub(crate) fn verifier_metric_compare(
    metric: &str,
    op: &str,
    left: &BaselineMetrics,
    right: &BaselineMetrics,
) -> bool {
    let left = verifier_metric_from_baseline(left, metric)
        .and_then(|value| value.as_i64())
        .unwrap_or_default();
    let right = verifier_metric_from_baseline(right, metric)
        .and_then(|value| value.as_i64())
        .unwrap_or_default();
    match op {
        "<" => left < right,
        "<=" => left <= right,
        ">" => left > right,
        ">=" => left >= right,
        "==" | "=" => left == right,
        _ => false,
    }
}

pub(crate) fn json_value_at_dot_path<'a>(
    value: &'a JsonValue,
    path: &str,
) -> Option<&'a JsonValue> {
    let mut current = value;
    for segment in path.split('.') {
        if segment.is_empty() {
            continue;
        }
        current = match current {
            JsonValue::Object(map) => map.get(segment)?,
            JsonValue::Array(items) => items.get(segment.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }
    Some(current)
}

pub(crate) fn resolve_assertion_value<'a>(
    state: &'a VerifierExecutionState,
    path: &str,
) -> Option<&'a JsonValue> {
    let mut segments = path.split('.');
    let root = segments.next()?;
    if let Some(root_value) = state.outputs.get(root) {
        let suffix = segments.collect::<Vec<_>>().join(".");
        if suffix.is_empty() {
            Some(root_value)
        } else {
            json_value_at_dot_path(root_value, &suffix)
        }
    } else {
        None
    }
}

pub(crate) async fn execute_cli_verifier_step(
    run: &str,
    materialized: &MaterializedFixture,
    state: &mut VerifierExecutionState,
) -> anyhow::Result<()> {
    let expanded = render_verifier_command_template(run, materialized, state);
    let tokens = shell_words::split(&expanded)
        .with_context(|| format!("parse verifier step `{expanded}`"))?;
    let Some(command) = tokens.get(1).map(String::as_str) else {
        anyhow::bail!("unsupported verifier cli step {expanded}");
    };
    let bundle_runtime = read_bundle_runtime_config(&materialized.bundle_root)?;
    let bundle_base_url = bundle_runtime
        .as_ref()
        .and_then(|config| config.base_url.as_deref())
        .unwrap_or("http://127.0.0.1:59999");
    match command {
        "wake" => {
            let snapshot = read_bundle_resume(
                &verifier_resume_args(&materialized.bundle_root),
                bundle_base_url,
            )
            .await
            .context("execute verifier wake step")?;
            let wakeup = render_bundle_wakeup_markdown(&materialized.bundle_root, &snapshot, false);
            write_wakeup_markdown_files(&materialized.bundle_root, &wakeup)
                .context("write verifier wakeup markdown")?;
            let prompt_efficiency = crate::workflow::AUTORESEARCH_LOOPS
                .iter()
                .find(|descriptor| descriptor.slug == "prompt-efficiency")
                .context("missing prompt-efficiency autoresearch descriptor")?;
            let packet_efficiency_record = crate::workflow::build_prompt_efficiency_record(
                &snapshot,
                prompt_efficiency,
                0,
                None,
            );
            crate::workflow::write_wake_packet_efficiency_artifacts(
                &materialized.bundle_root,
                &snapshot,
                &packet_efficiency_record,
            )
            .context("write verifier wake packet efficiency artifact")?;
            let packet_efficiency_path = materialized
                .bundle_root
                .join("loops/wake-packet-efficiency.json");
            let packet_efficiency = serde_json::from_str::<JsonValue>(
                &fs::read_to_string(&packet_efficiency_path)
                    .with_context(|| format!("read {}", packet_efficiency_path.display()))?,
            )
            .with_context(|| {
                format!(
                    "parse wake packet efficiency artifact {}",
                    packet_efficiency_path.display()
                )
            })?;
            state.metrics.insert(
                "prompt_tokens".to_string(),
                json!(snapshot.estimated_prompt_tokens()),
            );
            state.metrics.insert(
                "core_prompt_tokens".to_string(),
                json!(snapshot.core_prompt_tokens()),
            );
            state.outputs.insert(
                "wake".to_string(),
                json!({
                    "bundle": materialized.bundle_root.display().to_string(),
                    "wakeup_path": materialized.bundle_root.join("MEMD_WAKEUP.md").display().to_string(),
                    "markdown": wakeup,
                    "packet_efficiency_path": packet_efficiency_path.display().to_string(),
                    "packet_efficiency": packet_efficiency,
                }),
            );
        }
        "checkpoint" => {
            state.outputs.insert(
                "checkpoint".to_string(),
                json!({
                    "ok": true,
                    "bundle": materialized.bundle_root.display().to_string(),
                }),
            );
        }
        "resume" => {
            let snapshot = read_bundle_resume(
                &verifier_resume_args(&materialized.bundle_root),
                bundle_base_url,
            )
            .await
            .context("execute verifier resume step")?;
            state.metrics.insert(
                "prompt_tokens".to_string(),
                json!(snapshot.estimated_prompt_tokens()),
            );
            state.metrics.insert(
                "reconstruction_steps".to_string(),
                json!(snapshot.working.rehydration_queue.len()),
            );
            state.outputs.insert(
                "resume".to_string(),
                build_resume_step_output(&snapshot, &materialized.fixture_vars),
            );
        }
        "handoff" => {
            let snapshot = read_bundle_handoff(
                &verifier_handoff_args(&materialized.bundle_root),
                bundle_base_url,
            )
            .await
            .context("execute verifier handoff step")?;
            state.outputs.insert(
                "handoff".to_string(),
                build_handoff_step_output(&snapshot, &materialized.fixture_vars),
            );
        }
        "attach" => {
            let snippet = render_attach_snippet("bash", &materialized.bundle_root)
                .context("execute verifier attach step")?;
            state.outputs.insert(
                "attach".to_string(),
                json!({
                    "snippet": snippet,
                    "bundle": materialized.bundle_root.display().to_string(),
                }),
            );
        }
        "search" => {
            let runtime = read_bundle_runtime_config(&materialized.bundle_root)?;
            let mut query = None;
            let mut route = Some(RetrievalRoute::ProjectFirst);
            let mut intent = Some(RetrievalIntent::General);
            let mut workspace = runtime.as_ref().and_then(|config| config.workspace.clone());
            let mut visibility = runtime
                .as_ref()
                .and_then(|config| config.visibility.as_deref())
                .map(parse_memory_visibility_value)
                .transpose()?;
            let mut belief_branch = None;

            let mut index = 2usize;
            while index < tokens.len() {
                match tokens[index].as_str() {
                    "--output" => {
                        index += 1;
                    }
                    "--query" => {
                        index += 1;
                        query = tokens.get(index).cloned();
                    }
                    "--route" => {
                        index += 1;
                        route = parse_retrieval_route(tokens.get(index).cloned())?;
                    }
                    "--intent" => {
                        index += 1;
                        intent = parse_retrieval_intent(tokens.get(index).cloned())?;
                    }
                    "--workspace" => {
                        index += 1;
                        workspace = tokens.get(index).cloned();
                    }
                    "--visibility" => {
                        index += 1;
                        visibility = tokens
                            .get(index)
                            .map(|value| parse_memory_visibility_value(value))
                            .transpose()?;
                    }
                    "--belief-branch" => {
                        index += 1;
                        belief_branch = tokens.get(index).cloned();
                    }
                    other if other.starts_with("--") => {
                        anyhow::bail!("unsupported verifier search flag {other}");
                    }
                    _ => {}
                }
                index += 1;
            }

            let resolved_intent = intent.unwrap_or(RetrievalIntent::General);
            let req = SearchMemoryRequest {
                query,
                route: Some(route.unwrap_or(RetrievalRoute::ProjectFirst)),
                intent: Some(resolved_intent),
                scopes: vec![
                    MemoryScope::Project,
                    MemoryScope::Synced,
                    MemoryScope::Global,
                ],
                kinds: default_kinds_for_intent(resolved_intent),
                statuses: vec![MemoryStatus::Active],
                project: runtime.as_ref().and_then(|config| config.project.clone()),
                namespace: runtime.as_ref().and_then(|config| config.namespace.clone()),
                workspace,
                visibility,
                belief_branch,
                source_agent: None,
                tags: Vec::new(),
                stages: vec![MemoryStage::Canonical, MemoryStage::Candidate],
                limit: Some(6),
                max_chars_per_item: Some(280),
            };
            let client = MemdClient::new(bundle_base_url)?;
            let response = client
                .search(&req)
                .await
                .context("execute verifier search step")?;
            state.outputs.insert(
                "items".to_string(),
                serde_json::to_value(&response.items)?,
            );
            state.outputs.insert("search".to_string(), serde_json::to_value(&response)?);
        }
        "messages" => {
            let mut args = MessagesArgs {
                output: materialized.bundle_root.clone(),
                send: false,
                inbox: false,
                ack: None,
                target_session: None,
                kind: None,
                request_help: false,
                request_review: false,
                assign_scope: None,
                scope: None,
                content: None,
                summary: false,
            };
            let mut index = 2usize;
            while index < tokens.len() {
                match tokens[index].as_str() {
                    "--output" => {
                        index += 1;
                        args.output = PathBuf::from(
                            tokens
                                .get(index)
                                .context("messages step missing --output value")?,
                        );
                    }
                    "--send" => args.send = true,
                    "--inbox" => args.inbox = true,
                    "--ack" => {
                        index += 1;
                        args.ack = Some(
                            tokens
                                .get(index)
                                .context("messages step missing --ack value")?
                                .clone(),
                        );
                    }
                    "--target-session" => {
                        index += 1;
                        args.target_session = Some(
                            tokens
                                .get(index)
                                .context("messages step missing --target-session value")?
                                .clone(),
                        );
                    }
                    "--kind" => {
                        index += 1;
                        args.kind = Some(
                            tokens
                                .get(index)
                                .context("messages step missing --kind value")?
                                .clone(),
                        );
                    }
                    "--content" => {
                        index += 1;
                        args.content = Some(
                            tokens
                                .get(index)
                                .context("messages step missing --content value")?
                                .clone(),
                        );
                    }
                    "--summary" => args.summary = true,
                    other => anyhow::bail!("unsupported messages verifier flag {other}"),
                }
                index += 1;
            }
            let base_url = read_bundle_runtime_config(&args.output)?
                .and_then(|config| config.base_url)
                .unwrap_or_else(default_base_url);
            let response = run_messages_command(&args, &base_url).await?;
            let response_value = serde_json::to_value(&response)?;
            state
                .metrics
                .insert("delivery_count".to_string(), json!(response.messages.len()));
            if args.send {
                state
                    .outputs
                    .insert("messages_send".to_string(), response_value);
            } else if args.ack.is_some() {
                state
                    .outputs
                    .insert("messages_ack".to_string(), response_value);
            } else {
                state
                    .outputs
                    .insert("messages_inbox".to_string(), response_value);
            }
        }
        "claims" => {
            let mut args = ClaimsArgs {
                output: materialized.bundle_root.clone(),
                acquire: false,
                release: false,
                transfer_to_session: None,
                scope: None,
                ttl_secs: 900,
                summary: false,
            };
            let mut index = 2usize;
            while index < tokens.len() {
                match tokens[index].as_str() {
                    "--output" => {
                        index += 1;
                        args.output = PathBuf::from(
                            tokens
                                .get(index)
                                .context("claims step missing --output value")?,
                        );
                    }
                    "--acquire" => args.acquire = true,
                    "--release" => args.release = true,
                    "--transfer-to-session" => {
                        index += 1;
                        args.transfer_to_session = Some(
                            tokens
                                .get(index)
                                .context("claims step missing --transfer-to-session value")?
                                .clone(),
                        );
                    }
                    "--scope" => {
                        index += 1;
                        args.scope = Some(
                            tokens
                                .get(index)
                                .context("claims step missing --scope value")?
                                .clone(),
                        );
                    }
                    "--ttl-secs" => {
                        index += 1;
                        args.ttl_secs = tokens
                            .get(index)
                            .context("claims step missing --ttl-secs value")?
                            .parse()
                            .context("parse claims --ttl-secs")?;
                    }
                    "--summary" => args.summary = true,
                    other => anyhow::bail!("unsupported claims verifier flag {other}"),
                }
                index += 1;
            }
            let base_url = read_bundle_runtime_config(&args.output)?
                .and_then(|config| config.base_url)
                .unwrap_or_else(default_base_url);
            let response = run_claims_command(&args, &base_url).await?;
            let response_value = serde_json::to_value(&response)?;
            state
                .metrics
                .insert("claim_count".to_string(), json!(response.claims.len()));
            if args.acquire {
                state
                    .outputs
                    .insert("claims_acquire".to_string(), response_value);
            } else if args.release {
                state
                    .outputs
                    .insert("claims_release".to_string(), response_value);
            } else if args.transfer_to_session.is_some() {
                state
                    .outputs
                    .insert("claims_transfer".to_string(), response_value);
            } else {
                state.outputs.insert("claims".to_string(), response_value);
            }
        }
        "tasks" => {
            let mut args = TasksArgs {
                output: materialized.bundle_root.clone(),
                upsert: false,
                assign_to_session: None,
                target_session: None,
                task_id: None,
                title: None,
                description: None,
                status: None,
                mode: None,
                scope: Vec::new(),
                request_help: false,
                request_review: false,
                all: false,
                view: None,
                summary: false,
                json: false,
            };
            let mut index = 2usize;
            while index < tokens.len() {
                match tokens[index].as_str() {
                    "--output" => {
                        index += 1;
                        args.output = PathBuf::from(
                            tokens
                                .get(index)
                                .context("tasks step missing --output value")?,
                        );
                    }
                    "--upsert" => args.upsert = true,
                    "--assign-to-session" => {
                        index += 1;
                        args.assign_to_session = Some(
                            tokens
                                .get(index)
                                .context("tasks step missing --assign-to-session value")?
                                .clone(),
                        );
                    }
                    "--target-session" => {
                        index += 1;
                        args.target_session = Some(
                            tokens
                                .get(index)
                                .context("tasks step missing --target-session value")?
                                .clone(),
                        );
                    }
                    "--task-id" => {
                        index += 1;
                        args.task_id = Some(
                            tokens
                                .get(index)
                                .context("tasks step missing --task-id value")?
                                .clone(),
                        );
                    }
                    "--title" => {
                        index += 1;
                        args.title = Some(
                            tokens
                                .get(index)
                                .context("tasks step missing --title value")?
                                .clone(),
                        );
                    }
                    "--description" => {
                        index += 1;
                        args.description = Some(
                            tokens
                                .get(index)
                                .context("tasks step missing --description value")?
                                .clone(),
                        );
                    }
                    "--status" => {
                        index += 1;
                        args.status = Some(
                            tokens
                                .get(index)
                                .context("tasks step missing --status value")?
                                .clone(),
                        );
                    }
                    "--mode" => {
                        index += 1;
                        args.mode = Some(
                            tokens
                                .get(index)
                                .context("tasks step missing --mode value")?
                                .clone(),
                        );
                    }
                    "--scope" => {
                        index += 1;
                        args.scope.push(
                            tokens
                                .get(index)
                                .context("tasks step missing --scope value")?
                                .clone(),
                        );
                    }
                    "--request-help" => args.request_help = true,
                    "--request-review" => args.request_review = true,
                    "--all" => args.all = true,
                    "--view" => {
                        index += 1;
                        args.view = Some(
                            tokens
                                .get(index)
                                .context("tasks step missing --view value")?
                                .clone(),
                        );
                    }
                    "--summary" => args.summary = true,
                    "--json" => args.json = true,
                    other => anyhow::bail!("unsupported tasks verifier flag {other}"),
                }
                index += 1;
            }
            let base_url = read_bundle_runtime_config(&args.output)?
                .and_then(|config| config.base_url)
                .unwrap_or_else(default_base_url);
            let response = run_tasks_command(&args, &base_url).await?;
            let response_value = serde_json::to_value(&response)?;
            state
                .metrics
                .insert("task_count".to_string(), json!(response.tasks.len()));
            if args.upsert {
                state
                    .outputs
                    .insert("tasks_upsert".to_string(), response_value);
            } else if args.assign_to_session.is_some() {
                state
                    .outputs
                    .insert("tasks_assign".to_string(), response_value);
            } else if args.request_help || args.request_review {
                state
                    .outputs
                    .insert("tasks_request".to_string(), response_value);
            } else {
                state.outputs.insert("tasks".to_string(), response_value);
            }
        }
        other => anyhow::bail!("unsupported verifier cli command {other}"),
    }
    Ok(())
}

pub(crate) async fn execute_cli_expect_error_verifier_step(
    run: &str,
    materialized: &MaterializedFixture,
    state: &mut VerifierExecutionState,
) -> anyhow::Result<()> {
    match execute_cli_verifier_step(run, materialized, state).await {
        Ok(()) => anyhow::bail!("verifier expected cli step to fail: {run}"),
        Err(error) => {
            state.outputs.insert(
                "expected_error".to_string(),
                json!({
                    "message": error.to_string(),
                }),
            );
            state
                .metrics
                .insert("expected_error_count".to_string(), json!(1));
            Ok(())
        }
    }
}

pub(crate) fn write_verifier_fixture_heartbeat(
    output: &Path,
    state: &BundleHeartbeatState,
) -> anyhow::Result<()> {
    fs::create_dir_all(output.join("state"))
        .with_context(|| format!("create {}", output.join("state").display()))?;
    fs::write(
        bundle_heartbeat_state_path(output),
        serde_json::to_string_pretty(state).context("serialize fixture heartbeat")? + "\n",
    )
    .with_context(|| format!("write {}", bundle_heartbeat_state_path(output).display()))?;
    Ok(())
}

pub(crate) fn execute_helper_verifier_step(
    name: &str,
    materialized: &MaterializedFixture,
    state: &mut VerifierExecutionState,
) -> anyhow::Result<()> {
    match name {
        "run_resume_without_memd" => {
            let metrics = verifier_baseline_metrics("no_mempath")
                .context("missing no_mempath verifier baseline")?;
            state.baselines.insert("no_mempath".to_string(), metrics);
        }
        "run_resume_with_memd" => {
            let metrics = verifier_baseline_metrics("with_memd")
                .context("missing with_memd verifier baseline")?;
            state.baselines.insert("with_memd".to_string(), metrics);
        }
        "capture_message_id" => {
            let message_id = resolve_assertion_value(state, "messages_inbox.messages.0.id")
                .and_then(JsonValue::as_str)
                .context("capture_message_id requires an inbox message")?;
            state
                .outputs
                .insert("message_id".to_string(), json!(message_id));
            state.metrics.insert("delivery_count".to_string(), json!(1));
        }
        "ensure_handoff" => {
            let session = materialized
                .fixture_vars
                .get("target_session")
                .or_else(|| materialized.fixture_vars.get("primary_session"))
                .cloned()
                .context("ensure_handoff requires a target or primary session")?;
            state
                .outputs
                .insert("handoff_session".to_string(), json!(session));
        }
        "setup_target_lane_collision" => {
            let sender_bundle = PathBuf::from(
                materialized
                    .fixture_vars
                    .get("sender_bundle")
                    .context("setup_target_lane_collision requires sender_bundle")?,
            );
            let target_bundle = PathBuf::from(
                materialized
                    .fixture_vars
                    .get("target_bundle")
                    .context("setup_target_lane_collision requires target_bundle")?,
            );
            let sessions_root = sender_bundle
                .parent()
                .and_then(Path::parent)
                .context("setup_target_lane_collision requires session root")?;
            let sender_project = sender_bundle
                .parent()
                .context("setup_target_lane_collision sender project root missing")?;
            let target_project = target_bundle
                .parent()
                .context("setup_target_lane_collision target project root missing")?;
            fs::create_dir_all(sender_project.join(".planning")).with_context(|| {
                format!("create {}", sender_project.join(".planning").display())
            })?;
            fs::create_dir_all(target_project.join(".planning")).with_context(|| {
                format!("create {}", target_project.join(".planning").display())
            })?;
            fs::write(sender_project.join("README.md"), "# sender\n")
                .with_context(|| format!("write {}", sender_project.join("README.md").display()))?;
            fs::write(target_project.join("NOTES.md"), "# target\n")
                .with_context(|| format!("write {}", target_project.join("NOTES.md").display()))?;

            let _ = Command::new("git")
                .arg("-C")
                .arg(sessions_root)
                .arg("init")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .with_context(|| format!("init git repo {}", sessions_root.display()))?;
            let _ = Command::new("git")
                .arg("-C")
                .arg(sessions_root)
                .arg("config")
                .arg("user.email")
                .arg("memd@example.com")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .with_context(|| format!("git user email {}", sessions_root.display()))?;
            let _ = Command::new("git")
                .arg("-C")
                .arg(sessions_root)
                .arg("config")
                .arg("user.name")
                .arg("memd")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .with_context(|| format!("git user name {}", sessions_root.display()))?;
            let _ = Command::new("git")
                .arg("-C")
                .arg(sessions_root)
                .arg("add")
                .arg(".")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .with_context(|| format!("git add {}", sessions_root.display()))?;
            let _ = Command::new("git")
                .arg("-C")
                .arg(sessions_root)
                .arg("commit")
                .arg("-m")
                .arg("init")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .with_context(|| format!("git commit {}", sessions_root.display()))?;
            let _ = Command::new("git")
                .arg("-C")
                .arg(sessions_root)
                .arg("checkout")
                .arg("-b")
                .arg("feature/hive-shared")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .with_context(|| format!("git checkout {}", sessions_root.display()))?;

            let target_runtime = read_bundle_runtime_config(&target_bundle)?
                .context("setup_target_lane_collision target runtime missing")?;
            let heartbeat = BundleHeartbeatState {
                session: materialized.fixture_vars.get("target_session").cloned(),
                agent: target_runtime.agent.clone(),
                effective_agent: target_runtime
                    .agent
                    .as_deref()
                    .map(|agent| compose_agent_identity(agent, target_runtime.session.as_deref())),
                tab_id: target_runtime.tab_id.clone(),
                hive_system: target_runtime.agent.clone(),
                hive_role: Some("agent".to_string()),
                worker_name: target_runtime.agent.clone(),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["openclaw-stack".to_string()],
                lane_id: Some(sessions_root.display().to_string()),
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some(default_heartbeat_model()),
                project: target_runtime.project.clone(),
                namespace: target_runtime.namespace.clone(),
                workspace: target_runtime.workspace.clone(),
                repo_root: Some(sessions_root.display().to_string()),
                worktree_root: Some(sessions_root.display().to_string()),
                branch: Some("feature/hive-shared".to_string()),
                base_branch: Some("master".to_string()),
                visibility: target_runtime.visibility.clone(),
                base_url: target_runtime.base_url.clone(),
                base_url_healthy: Some(true),
                host: None,
                pid: None,
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
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
                status: "live".to_string(),
                last_seen: Utc::now(),
                authority_mode: Some("shared".to_string()),
                authority_degraded: false,
            };
            write_verifier_fixture_heartbeat(&target_bundle, &heartbeat)?;
            state.outputs.insert(
                "lane_collision".to_string(),
                json!({
                    "repo_root": sessions_root.display().to_string(),
                    "branch": "feature/hive-shared",
                    "target_session": materialized.fixture_vars.get("target_session").cloned(),
                }),
            );
        }
        other => anyhow::bail!("unsupported verifier helper step {other}"),
    }
    Ok(())
}

pub(crate) fn execute_compare_verifier_step(
    left: &str,
    right: &str,
    state: &mut VerifierExecutionState,
) -> anyhow::Result<()> {
    let left_metrics = state
        .baselines
        .get(left)
        .cloned()
        .with_context(|| format!("missing verifier baseline {left}"))?;
    let right_metrics = state
        .baselines
        .get(right)
        .cloned()
        .with_context(|| format!("missing verifier baseline {right}"))?;
    let report = build_no_memd_delta_report(&left_metrics, &right_metrics);
    state
        .metrics
        .insert("token_delta".to_string(), json!(report.token_delta));
    state
        .metrics
        .insert("reread_delta".to_string(), json!(report.reread_delta));
    state.metrics.insert(
        "reconstruction_delta".to_string(),
        json!(report.reconstruction_delta),
    );
    state.metrics.insert(
        "with_memd_better".to_string(),
        json!(report.with_memd_better),
    );
    state
        .outputs
        .insert("compare".to_string(), serde_json::to_value(&report)?);
    state.comparative_report = Some(report);
    Ok(())
}

pub(crate) async fn execute_verifier_steps(
    verifier: &VerifierRecord,
    materialized: &MaterializedFixture,
) -> anyhow::Result<VerifierExecutionState> {
    let mut state = VerifierExecutionState::default();
    for step in &verifier.steps {
        match step.kind.as_str() {
            "cli" => {
                execute_cli_verifier_step(
                    step.run
                        .as_deref()
                        .context("verifier cli step missing run command")?,
                    materialized,
                    &mut state,
                )
                .await?
            }
            "cli_expect_error" => {
                execute_cli_expect_error_verifier_step(
                    step.run
                        .as_deref()
                        .context("verifier cli_expect_error step missing run command")?,
                    materialized,
                    &mut state,
                )
                .await?
            }
            "helper" => execute_helper_verifier_step(
                step.name
                    .as_deref()
                    .context("verifier helper step missing helper name")?,
                materialized,
                &mut state,
            )?,
            "compare" => execute_compare_verifier_step(
                step.left
                    .as_deref()
                    .context("verifier compare step missing left baseline")?,
                step.right
                    .as_deref()
                    .context("verifier compare step missing right baseline")?,
                &mut state,
            )?,
            other => anyhow::bail!("unsupported verifier step kind {other}"),
        }
    }
    Ok(state)
}

pub(crate) fn evaluate_verifier_assertions(
    verifier: &VerifierRecord,
    materialized: &MaterializedFixture,
    state: &VerifierExecutionState,
) -> anyhow::Result<bool> {
    for assertion in &verifier.assertions {
        let passed =
            match assertion.kind.as_str() {
                "file_contains" => {
                    let path = assertion
                        .path
                        .as_deref()
                        .context("file_contains assertion missing path")?;
                    let file_path = materialized.bundle_root.join(path);
                    match assertion.exists {
                        Some(false) => !file_path.exists(),
                        _ => {
                            let contents = fs::read_to_string(&file_path)
                                .with_context(|| format!("read {}", file_path.display()))?;
                            if let Some(expected_key) = assertion.equals_fixture.as_deref() {
                                let expected =
                                    materialized.fixture_vars.get(expected_key).with_context(
                                        || format!("missing verifier fixture var {expected_key}"),
                                    )?;
                                contents == *expected
                            } else if let Some(expected_key) = assertion.contains_fixture.as_deref()
                            {
                                let expected =
                                    materialized.fixture_vars.get(expected_key).with_context(
                                        || format!("missing verifier fixture var {expected_key}"),
                                    )?;
                                contents.contains(expected)
                            } else {
                                true
                            }
                        }
                    }
                }
                "json_path" => {
                    let path = assertion
                        .path
                        .as_deref()
                        .context("json_path assertion missing path")?;
                    let value = resolve_assertion_value(state, path);
                    match assertion.exists {
                        Some(false) => value.is_none(),
                        _ => {
                            let value = value.context("json_path assertion missing value")?;
                            if let Some(expected_key) = assertion.equals_fixture.as_deref() {
                                let expected =
                                    materialized.fixture_vars.get(expected_key).with_context(
                                        || format!("missing verifier fixture var {expected_key}"),
                                    )?;
                                value
                                    .as_str()
                                    .map(|s| s.to_owned())
                                    .unwrap_or_else(|| value.to_string())
                                    == *expected
                            } else if let Some(expected_key) = assertion.contains_fixture.as_deref()
                            {
                                let expected =
                                    materialized.fixture_vars.get(expected_key).with_context(
                                        || format!("missing verifier fixture var {expected_key}"),
                                    )?;
                                value
                                    .as_str()
                                    .map(|s| s.to_owned())
                                    .unwrap_or_else(|| value.to_string())
                                    .contains(expected)
                            } else {
                                true
                            }
                        }
                    }
                }
                "metric_compare" => {
                    let metric = assertion
                        .metric
                        .as_deref()
                        .context("metric_compare assertion missing metric")?;
                    let op = assertion.op.as_deref().unwrap_or("==");
                    let left = assertion
                        .left
                        .as_deref()
                        .context("metric_compare assertion missing left baseline")?;
                    let right = assertion
                        .right
                        .as_deref()
                        .context("metric_compare assertion missing right baseline")?;
                    let left_metrics = state
                        .baselines
                        .get(left)
                        .context(format!("missing verifier baseline {left}"))?;
                    let right_metrics = state
                        .baselines
                        .get(right)
                        .context(format!("missing verifier baseline {right}"))?;
                    verifier_metric_compare(metric, op, left_metrics, right_metrics)
                }
                "helper" => match assertion
                    .name
                    .as_deref()
                    .context("helper assertion missing helper name")?
                {
                    "assert_handoff_resume_alignment" => {
                        let handoff = resolve_assertion_value(state, "handoff.current_task")
                            .context("handoff alignment assertion missing handoff.current_task")?;
                        let resume = resolve_assertion_value(state, "resume.current_task")
                            .context("handoff alignment assertion missing resume.current_task")?;
                        let handoff_id = handoff
                            .get("id")
                            .and_then(JsonValue::as_str)
                            .context("handoff alignment assertion missing handoff id")?;
                        let resume_id = resume
                            .get("id")
                            .and_then(JsonValue::as_str)
                            .context("handoff alignment assertion missing resume id")?;
                        let handoff_next = handoff
                            .get("next_action")
                            .and_then(JsonValue::as_str)
                            .context("handoff alignment assertion missing handoff next action")?;
                        let resume_next = resume
                            .get("next_action")
                            .and_then(JsonValue::as_str)
                            .context("handoff alignment assertion missing resume next action")?;
                        handoff_id == resume_id && handoff_next == resume_next
                    }
                    other => anyhow::bail!("unsupported verifier helper assertion {other}"),
                },
                other => anyhow::bail!("unsupported verifier assertion kind {other}"),
            };
        if !passed {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(crate) async fn run_verifier_record(
    verifier: &VerifierRecord,
    fixture: &FixtureRecord,
    base_url_override: Option<&str>,
) -> anyhow::Result<VerifierRunRecord> {
    let materialized = materialize_fixture(fixture, base_url_override)?;
    seed_materialized_fixture_sessions(&materialized)
        .await
        .context("seed verifier fixture sessions")?;
    let evidence_id = format!("evidence:{}:latest", verifier.id);
    let execution = execute_verifier_steps(verifier, &materialized).await?;
    let evidence_tiers = vec!["live_primary".to_string()];
    let assertions_passed = verifier_assertions_pass(verifier)
        && evaluate_verifier_assertions(verifier, &materialized, &execution)?;
    let continuity_ok = verifier_continuity_ok(verifier);
    let comparative_win = verifier_comparative_win(verifier)
        && execution
            .comparative_report
            .as_ref()
            .map(|report| report.with_memd_better)
            .unwrap_or(true);
    let evidence_payload = json!({
        "verifier_id": verifier.id,
        "fixture_id": fixture.id,
        "confidence_tier": evidence_tiers[0],
        "bundle_root": materialized.bundle_root,
        "fixture_vars": materialized.fixture_vars,
        "outputs": execution.outputs,
        "metrics_observed": execution.metrics,
    });
    let run = VerifierRunRecord {
        verifier_id: verifier.id.clone(),
        status: if assertions_passed && continuity_ok && comparative_win {
            "passing".to_string()
        } else {
            "failing".to_string()
        },
        gate_result: resolve_verifier_gate(
            &verifier.gate_target,
            &evidence_tiers,
            assertions_passed,
            continuity_ok,
            comparative_win,
        ),
        evidence_ids: vec![evidence_id],
        metrics_observed: execution.metrics,
    };
    write_verifier_run_artifacts(&materialized.bundle_root, &run, &evidence_payload)?;
    Ok(run)
}

use super::*;

#[test]
fn capability_bridges_install_superpowers_into_agents_skills() {
    let _home_lock = lock_home_mutation();
    let home = std::env::temp_dir().join(format!("memd-cap-bridge-{}", uuid::Uuid::new_v4()));
    let cache_root = home
        .join(".codex")
        .join("plugins")
        .join("cache")
        .join("claude-plugins-official")
        .join("superpowers")
        .join("5.0.6");
    fs::create_dir_all(home.join(".claude")).expect("create claude root");
    fs::create_dir_all(cache_root.join("skills")).expect("create skills dir");
    fs::create_dir_all(home.join(".agents").join("skills")).expect("create agents dir");
    fs::write(
        home.join(".claude").join("settings.json"),
        r#"{
  "enabledPlugins": {
    "superpowers@claude-plugins-official": true
  }
}
"#,
    )
    .expect("write settings");
    fs::write(cache_root.join("skills").join("README.md"), "superpowers")
        .expect("write bridge marker");

    let original_home = std::env::var_os("HOME");
    unsafe {
        std::env::set_var("HOME", &home);
    }

    let bridges = apply_capability_bridges();
    let action = bridges
        .actions
        .iter()
        .find(|action| action.harness == "codex" && action.capability == "superpowers")
        .expect("superpowers bridge action");
    assert!(matches!(
        action.status.as_str(),
        "bridged" | "already-bridged"
    ));
    let target = home.join(".agents").join("skills").join("superpowers");
    assert!(target.exists());
    assert!(
        fs::symlink_metadata(&target)
            .expect("read target metadata")
            .file_type()
            .is_symlink()
    );

    if let Some(value) = original_home {
        unsafe {
            std::env::set_var("HOME", value);
        }
    } else {
        unsafe {
            std::env::remove_var("HOME");
        }
    }
    fs::remove_dir_all(home).expect("cleanup fake home");
}

#[test]
fn capability_bridge_inspection_reports_available_without_mutating_targets() {
    let _home_lock = lock_home_mutation();
    let home = std::env::temp_dir().join(format!("memd-cap-inspect-{}", uuid::Uuid::new_v4()));
    let source = home
        .join(".codex")
        .join("plugins")
        .join("cache")
        .join("claude-plugins-official")
        .join("superpowers")
        .join("5.0.6")
        .join("skills");
    let target = home.join(".agents").join("skills").join("superpowers");

    fs::create_dir_all(&source).expect("create source skills dir");
    fs::create_dir_all(target.parent().expect("target parent")).expect("create target parent");
    fs::write(source.join("README.md"), "superpowers").expect("write source marker");

    let action = inspect_directory_skill_bridge("codex", "superpowers", &source, &target);
    assert_eq!(action.status, "available");
    assert_eq!(action.target_path, target.display().to_string());
    assert!(!target.exists());

    fs::remove_dir_all(home).expect("cleanup fake home");
}

#[test]
fn capability_bridges_install_superpowers_into_opencode_plugin_roots() {
    let _home_lock = lock_home_mutation();
    let home =
        std::env::temp_dir().join(format!("memd-cap-bridge-opencode-{}", uuid::Uuid::new_v4()));
    let cache_root = home
        .join(".codex")
        .join("plugins")
        .join("cache")
        .join("claude-plugins-official")
        .join("superpowers")
        .join("5.0.6");
    let source_opencode_plugins = cache_root.join(".opencode").join("plugins");
    let target_modern = home
        .join(".config")
        .join("opencode")
        .join("plugins")
        .join("superpowers");
    let target_legacy = home.join(".opencode").join("plugins").join("superpowers");

    fs::create_dir_all(home.join(".claude")).expect("create claude root");
    fs::create_dir_all(source_opencode_plugins.join("superpowers"))
        .expect("create source opencode plugin directory");
    fs::write(
        home.join(".claude").join("settings.json"),
        r#"{
  "enabledPlugins": {
    "superpowers@claude-plugins-official": true
  }
}
"#,
    )
    .expect("write settings");
    fs::write(
        source_opencode_plugins
            .join("superpowers")
            .join("memd-plugin.mjs"),
        "export {}\n",
    )
    .expect("write bridge marker");

    let original_home = std::env::var_os("HOME");
    unsafe {
        std::env::set_var("HOME", &home);
    }

    let bridges = apply_capability_bridges();
    let actions: Vec<_> = bridges
        .actions
        .iter()
        .filter(|action| action.harness == "opencode" && action.capability == "superpowers")
        .collect();
    assert_eq!(actions.len(), 2);
    for action in &actions {
        assert!(matches!(
            action.status.as_str(),
            "bridged" | "already-bridged"
        ));
    }
    let summary = render_capability_bridge_summary(&bridges);
    assert!(summary.contains(&target_modern.display().to_string()));
    assert!(summary.contains(&target_legacy.display().to_string()));
    assert!(
        actions
            .iter()
            .any(|action| action.target_path == target_modern.display().to_string())
    );
    assert!(
        actions
            .iter()
            .any(|action| action.target_path == target_legacy.display().to_string())
    );

    for target in [&target_modern, &target_legacy] {
        assert!(target.exists());
        let metadata = fs::symlink_metadata(target).expect("read target metadata");
        assert!(metadata.file_type().is_symlink());
    }

    if let Some(value) = original_home {
        unsafe {
            std::env::set_var("HOME", value);
        }
    } else {
        unsafe {
            std::env::remove_var("HOME");
        }
    }
    fs::remove_dir_all(home).expect("cleanup fake home");
}

#[test]
fn capability_bridges_install_codex_skills_into_opencode_command_roots() {
    let _home_lock = lock_home_mutation();
    let home = std::env::temp_dir().join(format!(
        "memd-cap-bridge-opencode-skills-{}",
        uuid::Uuid::new_v4()
    ));
    let codex_skill = home.join(".codex").join("skills").join("caveman");
    let bridged_skill = home
        .join(".agents")
        .join("skills")
        .join("superpowers")
        .join("brainstorming");
    let target_modern = home.join(".config").join("opencode").join("command");
    let target_legacy = home.join(".opencode").join("command");

    fs::create_dir_all(&codex_skill).expect("create codex skill");
    fs::create_dir_all(&bridged_skill).expect("create bridged skill");
    fs::write(
        codex_skill.join("SKILL.md"),
        "---\ndescription: Ultra-compressed communication mode\n---\n# caveman\n",
    )
    .expect("write codex skill");
    fs::write(
        bridged_skill.join("SKILL.md"),
        "---\ndescription: Explore user intent before implementation\n---\n# brainstorming\n",
    )
    .expect("write bridged skill");

    let original_home = std::env::var_os("HOME");
    unsafe {
        std::env::set_var("HOME", &home);
    }

    let bridges = apply_capability_bridges();
    assert!(bridges.actions.iter().any(|action| {
        action.harness == "opencode"
            && action.capability == "codex:caveman"
            && matches!(action.status.as_str(), "bridged" | "already-bridged")
    }));
    assert!(bridges.actions.iter().any(|action| {
        action.harness == "opencode"
            && action.capability == "codex-bridge:superpowers--brainstorming"
            && matches!(action.status.as_str(), "bridged" | "already-bridged")
    }));

    let caveman_modern =
        fs::read_to_string(target_modern.join("caveman.md")).expect("read modern caveman bridge");
    assert!(caveman_modern.contains("memd-opencode-skill-bridge"));
    assert!(caveman_modern.contains("Keep normal spelling"));
    assert!(caveman_modern.contains("@"));

    let brainstorming_legacy =
        fs::read_to_string(target_legacy.join("superpowers--brainstorming.md"))
            .expect("read legacy brainstorming bridge");
    assert!(brainstorming_legacy.contains("Explore user intent before implementation"));

    if let Some(value) = original_home {
        unsafe {
            std::env::set_var("HOME", value);
        }
    } else {
        unsafe {
            std::env::remove_var("HOME");
        }
    }
    fs::remove_dir_all(home).expect("cleanup fake home");
}

#[test]
fn caveman_voice_bridge_keeps_normal_spelling_in_generated_guidance() {
    let root = std::env::temp_dir().join(format!("memd-voice-bridge-{}", uuid::Uuid::new_v4()));
    let bundle = root.join(".memd");
    fs::create_dir_all(&bundle).expect("create bundle");
    fs::write(
        bundle.join("config.json"),
        "{\n  \"voice_mode\": \"caveman-ultra\"\n}\n",
    )
    .expect("write config");

    let bridge = render_codex_agents_bridge_markdown(&bundle);
    assert!(bridge.contains("compressed wording, not broken spelling"));
    assert!(bridge.contains("exact technical terms"));

    let voice = render_voice_mode_section("caveman-ultra");
    assert!(voice.contains("abbreviate"));
    assert!(voice.contains("exact technical terms"));

    fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn render_capability_registry_summary_includes_claude_family_bridgeable_records() {
    let registry = CapabilityRegistry {
        generated_at: Utc::now(),
        project_root: None,
        capabilities: vec![
            CapabilityRecord {
                harness: "claude".to_string(),
                kind: "skill".to_string(),
                name: "universal-skill".to_string(),
                status: "enabled".to_string(),
                portability_class: "universal".to_string(),
                source_path: "src/universal.md".to_string(),
                bridge_hint: None,
                hash: None,
                notes: Vec::new(),
            },
            CapabilityRecord {
                harness: "clawcode".to_string(),
                kind: "plugin".to_string(),
                name: "bridge-plugin".to_string(),
                status: "enabled".to_string(),
                portability_class: "bridgeable".to_string(),
                source_path: "src/plugin.md".to_string(),
                bridge_hint: Some("link-to-plugin".to_string()),
                hash: None,
                notes: Vec::new(),
            },
            CapabilityRecord {
                harness: "clawcode".to_string(),
                kind: "plugin".to_string(),
                name: "cl-family-bridgeable".to_string(),
                status: "enabled".to_string(),
                portability_class: "claude-family-bridgeable".to_string(),
                source_path: "src/cl-fam.md".to_string(),
                bridge_hint: Some("link-to-fork".to_string()),
                hash: None,
                notes: Vec::new(),
            },
        ],
    };

    let summary = render_capability_registry_summary(&registry);
    assert!(summary.contains("bridgeable: 2"));
    assert!(summary.contains("### Bridgeable capabilities"));
    assert!(summary.contains("clawcode / plugin / bridge-plugin [bridgeable]"));
    assert!(summary.contains(
        "clawcode / plugin / cl-family-bridgeable [claude-family-bridgeable] -> link-to-fork"
    ));
}

#[test]
fn render_capability_bridge_summary_includes_opencode_targets() {
    let home =
        std::env::temp_dir().join(format!("memd-cap-bridge-summary-{}", uuid::Uuid::new_v4()));
    let registry = CapabilityBridgeRegistry {
        generated_at: Utc::now(),
        actions: vec![
            CapabilityBridgeAction {
                harness: "opencode".to_string(),
                capability: "superpowers".to_string(),
                status: "bridged".to_string(),
                source_path: home
                    .join(".codex")
                    .join("plugins")
                    .join("cache")
                    .join("claude-plugins-official")
                    .join("superpowers")
                    .join("5.0.6")
                    .join(".opencode")
                    .join("plugins")
                    .join("superpowers")
                    .display()
                    .to_string(),
                target_path: home
                    .join(".config")
                    .join("opencode")
                    .join("plugins")
                    .join("superpowers")
                    .display()
                    .to_string(),
                notes: vec!["created native skill bridge".to_string()],
            },
            CapabilityBridgeAction {
                harness: "opencode".to_string(),
                capability: "superpowers".to_string(),
                status: "already-bridged".to_string(),
                source_path: home
                    .join(".codex")
                    .join("plugins")
                    .join("cache")
                    .join("claude-plugins-official")
                    .join("superpowers")
                    .join("5.0.6")
                    .join(".opencode")
                    .join("plugins")
                    .join("superpowers")
                    .display()
                    .to_string(),
                target_path: home
                    .join(".opencode")
                    .join("plugins")
                    .join("superpowers")
                    .display()
                    .to_string(),
                notes: vec!["already-bridged".to_string()],
            },
        ],
    };

    let summary = render_capability_bridge_summary(&registry);
    assert!(summary.contains("## Capability Bridges"));
    assert!(summary.contains("bridged: 1"));
    assert!(summary.contains("already_bridged: 1"));
    assert!(summary.contains("- opencode / superpowers -> "));
    let modern_target = home
        .join(".config")
        .join("opencode")
        .join("plugins")
        .join("superpowers")
        .display()
        .to_string();
    let legacy_target = home
        .join(".opencode")
        .join("plugins")
        .join("superpowers")
        .display()
        .to_string();
    assert!(summary.contains(&modern_target));
    assert!(summary.contains(&legacy_target));
}

#[test]
fn detect_claude_family_harness_roots_finds_clawcode_shape() {
    let home = std::env::temp_dir().join(format!("memd-clawcode-home-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(home.join(".claude")).expect("create claude root");
    fs::create_dir_all(home.join(".clawcode")).expect("create clawcode root");
    fs::write(home.join(".claude").join("settings.json"), "{}").expect("write claude settings");
    fs::write(home.join(".clawcode").join("settings.json"), "{}").expect("write clawcode settings");

    let roots = detect_claude_family_harness_roots(&home);
    assert!(roots.iter().any(|root| root.harness == "claude"));
    assert!(roots.iter().any(|root| root.harness == "clawcode"));

    fs::remove_dir_all(home).expect("cleanup fake home");
}

#[test]
fn capability_registry_detects_claude_family_fork_plugin_state() {
    let _home_lock = lock_home_mutation();
    let home = std::env::temp_dir().join(format!("memd-clawcode-cap-{}", uuid::Uuid::new_v4()));
    let cache_root = home
        .join(".codex")
        .join("plugins")
        .join("cache")
        .join("claude-plugins-official")
        .join("superpowers")
        .join("5.0.6")
        .join(".codex");
    fs::create_dir_all(home.join(".clawcode")).expect("create clawcode root");
    fs::create_dir_all(&cache_root).expect("create codex cache");
    fs::write(
        home.join(".clawcode").join("settings.json"),
        r#"{
  "enabledPlugins": {
    "superpowers@claude-plugins-official": true
  }
}
"#,
    )
    .expect("write clawcode settings");
    fs::write(cache_root.join("INSTALL.md"), "# install\n").expect("write install");

    let original_home = std::env::var_os("HOME");
    unsafe {
        std::env::set_var("HOME", &home);
    }

    let registry = build_bundle_capability_registry(None);
    let record = registry
        .capabilities
        .iter()
        .find(|record| record.harness == "clawcode" && record.name == "superpowers")
        .expect("clawcode superpowers record");
    assert_eq!(record.kind, "plugin");
    assert_eq!(record.status, "enabled");
    assert_eq!(record.portability_class, "claude-family-bridgeable");

    if let Some(value) = original_home {
        unsafe {
            std::env::set_var("HOME", value);
        }
    } else {
        unsafe {
            std::env::remove_var("HOME");
        }
    }
    fs::remove_dir_all(home).expect("cleanup fake home");
}

#[test]
fn init_output_prefers_project_root_when_seeded_from_repo() {
    let project_root =
        std::env::temp_dir().join(format!("memd-init-output-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&project_root).expect("create temp project root");

    let args = InitArgs {
        project: None,
        namespace: None,
        global: false,
        project_root: Some(project_root.clone()),
        seed_existing: true,
        agent: "codex".to_string(),
        session: None,
        tab_id: None,
        hive_system: None,
        hive_role: None,
        capability: Vec::new(),
        hive_group: Vec::new(),
        hive_group_goal: None,
        authority: None,
        output: default_init_output_path(),
        base_url: "http://127.0.0.1:8787".to_string(),
        rag_url: None,
        route: "auto".to_string(),
        intent: "current_task".to_string(),
        voice_mode: None,
        workspace: None,
        visibility: None,
        allow_localhost_read_only_fallback: false,
        force: false,
    };

    let resolved = resolve_init_output_path(&args, Some(project_root.as_path()));
    assert_eq!(resolved, project_root.join(".memd"));

    fs::remove_dir_all(project_root).expect("cleanup temp project");
}

#[test]
fn init_cli_defaults_agent_to_auto() {
    let cli = Cli::try_parse_from(["memd", "init"]).expect("parse init without agent");
    let Commands::Init(args) = cli.command else {
        panic!("expected init command");
    };
    assert_eq!(args.agent, "auto");
}

#[test]
fn normalize_init_args_detects_claude_when_agent_is_auto() {
    let _home_lock = lock_home_mutation();
    let home = std::env::temp_dir().join(format!("memd-init-auto-agent-{}", uuid::Uuid::new_v4()));
    let project_root =
        std::env::temp_dir().join(format!("memd-init-auto-project-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(home.join(".claude")).expect("create claude home");
    fs::create_dir_all(&project_root).expect("create project root");

    let original_home = std::env::var_os("HOME");
    unsafe {
        std::env::set_var("HOME", &home);
    }

    let args = InitArgs {
        project: None,
        namespace: None,
        global: false,
        project_root: Some(project_root.clone()),
        seed_existing: true,
        agent: "auto".to_string(),
        session: None,
        tab_id: None,
        hive_system: None,
        hive_role: None,
        capability: Vec::new(),
        hive_group: Vec::new(),
        hive_group_goal: None,
        authority: None,
        output: default_init_output_path(),
        base_url: "http://127.0.0.1:8787".to_string(),
        rag_url: None,
        route: "auto".to_string(),
        intent: "current_task".to_string(),
        voice_mode: None,
        workspace: None,
        visibility: None,
        allow_localhost_read_only_fallback: false,
        force: false,
    };

    let normalized = normalize_init_args(args).expect("normalize init args");
    assert_eq!(normalized.agent, "claude-code");

    if let Some(value) = original_home {
        unsafe {
            std::env::set_var("HOME", value);
        }
    } else {
        unsafe {
            std::env::remove_var("HOME");
        }
    }
    fs::remove_dir_all(home).expect("cleanup fake home");
    fs::remove_dir_all(project_root).expect("cleanup fake project");
}

#[test]
fn init_output_prefers_global_bundle_when_requested() {
    let project_root =
        std::env::temp_dir().join(format!("memd-init-global-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&project_root).expect("create temp project root");

    let args = InitArgs {
        project: None,
        namespace: None,
        global: true,
        project_root: Some(project_root.clone()),
        seed_existing: true,
        agent: "codex".to_string(),
        session: None,
        tab_id: None,
        hive_system: None,
        hive_role: None,
        capability: Vec::new(),
        hive_group: Vec::new(),
        hive_group_goal: None,
        authority: None,
        output: default_init_output_path(),
        base_url: "http://127.0.0.1:8787".to_string(),
        rag_url: None,
        route: "auto".to_string(),
        intent: "current_task".to_string(),
        voice_mode: None,
        workspace: None,
        visibility: None,
        allow_localhost_read_only_fallback: false,
        force: false,
    };

    let resolved = resolve_init_output_path(&args, Some(project_root.as_path()));
    assert_eq!(resolved, default_global_bundle_root());

    fs::remove_dir_all(project_root).expect("cleanup temp project");
}

#[test]
fn checkpoint_translation_sets_short_term_defaults() {
    let args = CheckpointArgs {
        output: PathBuf::from(".memd"),
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        workspace: Some("team-alpha".to_string()),
        visibility: Some("workspace".to_string()),
        source_path: Some("notes/today.md".to_string()),
        confidence: None,
        ttl_seconds: None,
        tag: vec!["urgent".to_string()],
        content: Some("remember current blocker".to_string()),
        input: None,
        stdin: false,
        auto_commit: false,
        roadmap_set: vec![],
    };

    let translated = checkpoint_as_remember_args(&args);
    assert_eq!(translated.kind.as_deref(), Some("status"));
    assert_eq!(translated.scope.as_deref(), Some("project"));
    assert_eq!(translated.source_system.as_deref(), Some("memd-short-term"));
    assert_eq!(translated.source_quality.as_deref(), Some("derived"));
    assert_eq!(translated.confidence, Some(0.8));
    assert_eq!(translated.ttl_seconds, Some(86_400));
    assert!(translated.tag.iter().any(|value| value == "checkpoint"));
    assert!(translated.tag.iter().any(|value| value == "current-task"));
    assert!(translated.tag.iter().any(|value| value == "urgent"));
}

#[test]
fn bundle_memory_markdown_surfaces_current_task_snapshot() {
    let snapshot = ResumeSnapshot {
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        agent: Some("codex".to_string()),
        workspace: Some("team-alpha".to_string()),
        visibility: Some("workspace".to_string()),
        route: "auto".to_string(),
        intent: "current_task".to_string(),
        context: memd_schema::CompactContextResponse {
            route: memd_schema::RetrievalRoute::Auto,
            intent: memd_schema::RetrievalIntent::CurrentTask,
            retrieval_order: vec![memd_schema::MemoryScope::Project],
            records: Vec::new(),
        },
        working: memd_schema::WorkingMemoryResponse {
            route: memd_schema::RetrievalRoute::Auto,
            intent: memd_schema::RetrievalIntent::CurrentTask,
            retrieval_order: vec![memd_schema::MemoryScope::Project],
            budget_chars: 1600,
            used_chars: 60,
            remaining_chars: 1540,
            truncated: false,
            policy: memd_schema::WorkingMemoryPolicyState {
                admission_limit: 8,
                max_chars_per_item: 220,
                budget_chars: 1600,
                rehydration_limit: 4,
            },
            records: vec![memd_schema::CompactMemoryRecord {
                id: uuid::Uuid::new_v4(),
                record: "Finish the resume snapshot renderer".to_string(),
            }],
            evicted: Vec::new(),
            rehydration_queue: vec![memd_schema::MemoryRehydrationRecord {
                id: None,
                kind: "source".to_string(),
                label: "artifact".to_string(),
                summary: "Check the latest handoff note".to_string(),
                reason: None,
                source_agent: None,
                source_system: None,
                source_path: None,
                source_quality: None,
                recorded_at: None,
            }],
            traces: Vec::new(),
            semantic_consolidation: None,
            procedures: vec![],

            compaction_quality: None,
        },
        inbox: memd_schema::MemoryInboxResponse {
            route: memd_schema::RetrievalRoute::Auto,
            intent: memd_schema::RetrievalIntent::CurrentTask,
            items: vec![memd_schema::InboxMemoryItem {
                item: memd_schema::MemoryItem {
                    id: uuid::Uuid::new_v4(),
                    content: "Repair one stale workspace lane".to_string(),
                    redundancy_key: None,
                    belief_branch: None,
                    preferred: true,
                    kind: memd_schema::MemoryKind::Status,
                    scope: memd_schema::MemoryScope::Project,
                    project: Some("demo".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("team-alpha".to_string()),
                    visibility: memd_schema::MemoryVisibility::Workspace,
                    source_agent: None,
                    source_system: None,
                    source_path: None,
                    source_quality: None,
                    confidence: 0.8,
                    ttl_seconds: Some(86_400),
                    created_at: chrono::Utc::now(),
                    status: memd_schema::MemoryStatus::Active,
                    stage: memd_schema::MemoryStage::Candidate,
                    lane: None,
                    version: 1,
                    last_verified_at: None,
                    supersedes: Vec::new(),
                    updated_at: chrono::Utc::now(),
                    tags: vec!["checkpoint".to_string()],
                    correction_meta: None,
                },
                reasons: vec!["stale".to_string()],
            }],
        },
        workspaces: memd_schema::WorkspaceMemoryResponse {
            workspaces: vec![memd_schema::WorkspaceMemoryRecord {
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("team-alpha".to_string()),
                visibility: memd_schema::MemoryVisibility::Workspace,
                item_count: 4,
                active_count: 3,
                candidate_count: 1,
                contested_count: 0,
                source_lane_count: 1,
                avg_confidence: 0.84,
                trust_score: 0.91,
                last_seen_at: None,
                tags: Vec::new(),
            }],
        },
        sources: memd_schema::SourceMemoryResponse {
            sources: Vec::new(),
        },
        semantic: None,
        claims: SessionClaimsState::default(),
        recent_repo_changes: vec!["status M crates/memd-client/src/main.rs".to_string()],
        change_summary: vec!["focus -> Finish the resume snapshot renderer".to_string()],
        resume_state_age_minutes: None,
        refresh_recommended: false,
        atlas_region_hints: Vec::new(),
        handoff_quality: None,
        files_touched: Vec::new(),
        un_read_paths: Vec::new(),
        preferences: Vec::new(),
    };

    let markdown = render_bundle_memory_markdown(Path::new(".memd"), &snapshot, None, None);
    assert!(markdown.contains("## Budget"));
    assert!(markdown.contains("drivers="));
    assert!(markdown.contains("action=\""));
    assert!(markdown.contains("## Durable Truth"));
    assert!(markdown.contains("- none"));
    assert!(markdown.contains("## Read First"));
    assert!(markdown.contains("## Memory Objects"));
    assert!(markdown.contains("- context none"));
    assert!(markdown.contains("- working id="));
    assert!(markdown.contains("- inbox id="));
    assert!(markdown.contains("- recovery id="));
    assert!(markdown.contains("- workspace project="));
    assert!(markdown.contains("## E+LT"));
    assert!(markdown.contains("Finish the resume snapshot renderer"));
    assert!(markdown.contains("Repair one stale workspace lane"));
    assert!(markdown.contains("Check the latest handoff note"));
    assert!(markdown.contains("status M crates/memd-client/src/main.rs"));
    assert!(markdown.contains("team-alpha"));
    assert!(path_text_contains(&markdown, "compiled/memory/working.md"));
}

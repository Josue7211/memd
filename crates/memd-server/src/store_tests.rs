    use super::*;
    use crate::canonical_key;
    use crate::keys::redundancy_key;
    use memd_schema::{
        HiveRosterRequest, MaintainReportRequest, MemoryKind, MemoryScope, MemoryStage,
        MemoryStatus, MemoryVisibility, SourceQuality,
    };

    fn open_temp_store(prefix: &str) -> (std::path::PathBuf, SqliteStore) {
        let dir = std::env::temp_dir().join(format!("{prefix}-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");
        (dir, store)
    }

    fn sample_memory_item() -> MemoryItem {
        let now = chrono::Utc::now();
        MemoryItem {
            id: Uuid::new_v4(),
            content: "hive resume state".to_string(),
            redundancy_key: None,
            belief_branch: None,
            preferred: false,
            kind: MemoryKind::Status,
            scope: MemoryScope::Project,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("core".to_string()),
            visibility: MemoryVisibility::Workspace,
            source_agent: Some("codex@test".to_string()),
            source_system: Some("memd-test".to_string()),
            source_path: None,
            confidence: 0.9,
            ttl_seconds: None,
            created_at: now,
            updated_at: now,
            last_verified_at: None,
            supersedes: Vec::new(),
            tags: vec!["resume_state".to_string()],
            status: MemoryStatus::Active,
            source_quality: Some(SourceQuality::Canonical),
            stage: MemoryStage::Canonical,
        }
    }

    #[test]
    fn fuzzy_entity_search_scores_alias_hits_highest() {
        let entity = MemoryEntityRecord {
            id: Uuid::new_v4(),
            entity_type: "repo".to_string(),
            aliases: vec!["memd".to_string(), "memory manager".to_string()],
            current_state: Some("main branch with smart memory".to_string()),
            state_version: 1,
            confidence: 0.9,
            salience_score: 0.8,
            rehearsal_count: 3,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_accessed_at: Some(chrono::Utc::now()),
            last_seen_at: Some(chrono::Utc::now()),
            valid_from: Some(chrono::Utc::now()),
            valid_to: None,
            tags: vec!["project".to_string()],
            context: Some(MemoryContextFrame {
                at: Some(chrono::Utc::now()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("core".to_string()),
                repo: Some("memd".to_string()),
                host: Some("laptop".to_string()),
                branch: Some("main".to_string()),
                agent: Some("codex".to_string()),
                location: Some("/tmp/memd".to_string()),
            }),
        };
        let request = EntitySearchRequest {
            query: "memd repo".to_string(),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            at: Some(chrono::Utc::now()),
            host: Some("laptop".to_string()),
            branch: Some("main".to_string()),
            location: Some("/tmp/memd".to_string()),
            route: None,
            intent: None,
            limit: Some(5),
        };

        let (score, reasons) = score_entity_search(
            &request,
            &normalize_search_text("memd repo"),
            &tokenize_search_text("memd repo"),
            &entity,
        );

        assert!(score > 0.5);
        assert!(reasons.iter().any(|reason| reason.contains("token:memd")));
    }

    #[test]
    fn insert_or_get_duplicate_returns_existing_item_without_deadlock() {
        let (dir, store) = open_temp_store("memd-duplicate-path");
        let item = sample_memory_item();
        let canonical_key = canonical_key(&item);
        let redundancy_key = redundancy_key(&item);

        assert!(
            store
                .insert_or_get_duplicate(&item, &canonical_key, &redundancy_key)
                .expect("insert first item")
                .is_none()
        );

        let duplicate = store
            .insert_or_get_duplicate(&item, &canonical_key, &redundancy_key)
            .expect("resolve duplicate");

        assert!(duplicate.is_some());
        assert_eq!(duplicate.as_ref().map(|found| found.id), Some(item.id));
        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn rehearse_entity_by_id_updates_entity_without_deadlock() {
        let (dir, store) = open_temp_store("memd-rehearse-entity");
        let item = sample_memory_item();
        let canonical_key = canonical_key(&item);
        let entity = store
            .resolve_entity_for_item(&item, &canonical_key)
            .expect("resolve entity");

        let rehearsed = store
            .rehearse_entity_by_id(entity.record.id, 0.15)
            .expect("rehearse entity")
            .expect("entity should exist");

        assert_eq!(rehearsed.id, entity.record.id);
        assert_eq!(
            rehearsed.rehearsal_count,
            entity.record.rehearsal_count.saturating_add(1)
        );
        assert!(rehearsed.salience_score >= entity.record.salience_score);
        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn concurrent_write_and_cross_workspace_reads_complete() {
        let (dir, store) = open_temp_store("memd-cross-workspace-concurrency");

        let mut seed = sample_memory_item();
        seed.project = Some("demo".to_string());
        seed.namespace = Some("main".to_string());
        seed.workspace = Some("shared".to_string());
        seed.visibility = MemoryVisibility::Workspace;
        seed.content = "seed item".to_string();
        seed.source_agent = Some("codex@test-a@session-a".to_string());
        seed.source_system = Some("memd".to_string());
        seed.tags = vec!["seed".to_string()];
        let seed_canonical_key = canonical_key(&seed);
        let seed_redundancy_key = redundancy_key(&seed);
        store
            .insert_or_get_duplicate(&seed, &seed_canonical_key, &seed_redundancy_key)
            .expect("insert seed item");
        let entity = store
            .resolve_entity_for_item(&seed, &seed_canonical_key)
            .expect("resolve seed entity");
        store
            .record_event(
                &entity.record,
                seed.id,
                RecordEventArgs {
                    event_type: "stored".to_string(),
                    summary: "seed stored".to_string(),
                    occurred_at: seed.updated_at,
                    project: seed.project.clone(),
                    namespace: seed.namespace.clone(),
                    workspace: seed.workspace.clone(),
                    source_agent: seed.source_agent.clone(),
                    source_system: seed.source_system.clone(),
                    source_path: seed.source_path.clone(),
                    related_entity_ids: Vec::new(),
                    tags: seed.tags.clone(),
                    context: None,
                    confidence: seed.confidence,
                    salience_score: entity.record.salience_score,
                },
            )
            .expect("record seed event");

        let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
        let (done_tx, done_rx) = std::sync::mpsc::channel::<&'static str>();

        let writer_store = store.clone();
        let writer_barrier = barrier.clone();
        let writer_tx = done_tx.clone();
        let writer = std::thread::spawn(move || {
            writer_barrier.wait();
            let mut item = sample_memory_item();
            item.project = Some("demo".to_string());
            item.namespace = Some("main".to_string());
            item.workspace = Some("shared".to_string());
            item.visibility = MemoryVisibility::Workspace;
            item.content = "concurrent item".to_string();
            item.source_agent = Some("codex@test-a@session-a".to_string());
            item.source_system = Some("memd".to_string());
            item.tags = vec!["repro".to_string()];
            let canonical_key = canonical_key(&item);
            let redundancy_key = redundancy_key(&item);
            writer_store
                .insert_or_get_duplicate(&item, &canonical_key, &redundancy_key)
                .expect("insert concurrent item");
            let entity = writer_store
                .resolve_entity_for_item(&item, &canonical_key)
                .expect("resolve concurrent entity");
            writer_store
                .record_event(
                    &entity.record,
                    item.id,
                    RecordEventArgs {
                        event_type: "stored".to_string(),
                        summary: "concurrent item stored".to_string(),
                        occurred_at: item.updated_at,
                        project: item.project.clone(),
                        namespace: item.namespace.clone(),
                        workspace: item.workspace.clone(),
                        source_agent: item.source_agent.clone(),
                        source_system: item.source_system.clone(),
                        source_path: item.source_path.clone(),
                        related_entity_ids: Vec::new(),
                        tags: item.tags.clone(),
                        context: None,
                        confidence: item.confidence,
                        salience_score: entity.record.salience_score,
                    },
                )
                .expect("record concurrent event");
            writer_tx.send("writer").expect("send writer completion");
        });

        let reader_store = store.clone();
        let reader_barrier = barrier.clone();
        let reader_tx = done_tx.clone();
        let reader = std::thread::spawn(move || {
            reader_barrier.wait();
            let workspaces = reader_store
                .workspace_memory(&WorkspaceMemoryRequest {
                    project: Some("demo".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("other".to_string()),
                    visibility: Some(MemoryVisibility::Workspace),
                    source_agent: None,
                    source_system: None,
                    limit: Some(6),
                })
                .expect("cross-workspace lanes");
            let sources = reader_store
                .source_memory(&SourceMemoryRequest {
                    project: Some("demo".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("other".to_string()),
                    visibility: Some(MemoryVisibility::Workspace),
                    source_agent: None,
                    source_system: None,
                    limit: Some(6),
                })
                .expect("cross-workspace sources");
            assert!(workspaces.workspaces.is_empty());
            assert!(sources.sources.is_empty());
            reader_tx.send("reader").expect("send reader completion");
        });

        barrier.wait();
        let first = done_rx
            .recv_timeout(std::time::Duration::from_secs(2))
            .expect("first concurrent operation should finish");
        let second = done_rx
            .recv_timeout(std::time::Duration::from_secs(2))
            .expect("second concurrent operation should finish");
        assert_ne!(first, second);

        writer.join().expect("join writer");
        reader.join().expect("join reader");
        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn hive_sessions_keep_same_named_sessions_separate_across_agents() {
        let dir = std::env::temp_dir().join(format!("memd-hive-sessions-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "shared-session".to_string(),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@shared-session".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("codex".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string(), "coordination".to_string()],
                hive_groups: vec!["openclaw-stack".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some("llama-desktop/qwen".to_string()),
                tab_id: None,
                project: Some("repo-a".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("initiative-alpha".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: Some("laptop-a".to_string()),
                pid: Some(101),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: Some("work a".to_string()),
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
                status: Some("live".to_string()),
            })
            .expect("insert codex session");
        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "shared-session".to_string(),
                agent: Some("claude-code".to_string()),
                effective_agent: Some("claude-code@shared-session".to_string()),
                hive_system: Some("claude-code".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("claude-code".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string(), "coordination".to_string()],
                hive_groups: vec!["openclaw-stack".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some("claude-sonnet-4".to_string()),
                tab_id: None,
                project: Some("repo-b".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("initiative-alpha".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:9797".to_string()),
                base_url_healthy: Some(true),
                host: Some("laptop-b".to_string()),
                pid: Some(202),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: Some("work b".to_string()),
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
                status: Some("live".to_string()),
            })
            .expect("insert claude session");

        let sessions = store
            .hive_sessions(&HiveSessionsRequest {
                session: Some("shared-session".to_string()),
                project: None,
                namespace: None,
                repo_root: None,
                worktree_root: None,
                branch: None,
                workspace: Some("initiative-alpha".to_string()),
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(16),
            })
            .expect("query sessions");
        assert_eq!(sessions.sessions.len(), 2);
        assert_eq!(
            sessions
                .sessions
                .iter()
                .filter(|session| session.agent.as_deref() == Some("codex"))
                .count(),
            1
        );
        assert_eq!(
            sessions
                .sessions
                .iter()
                .filter(|session| session.agent.as_deref() == Some("claude-code"))
                .count(),
            1
        );

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn hive_sessions_keep_same_named_sessions_separate_across_branches() {
        let dir = std::env::temp_dir().join(format!("memd-hive-branches-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "shared-session".to_string(),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@shared-session".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("codex".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["project:demo".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some("llama-desktop/qwen".to_string()),
                tab_id: Some("tab-a".to_string()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                repo_root: Some("/tmp/repo".to_string()),
                worktree_root: Some("/tmp/repo-a".to_string()),
                branch: Some("feature/a".to_string()),
                base_branch: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: Some("workstation".to_string()),
                pid: Some(111),
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
                status: Some("live".to_string()),
            })
            .expect("insert branch a session");
        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "shared-session".to_string(),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@shared-session".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("codex".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["project:demo".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some("llama-desktop/qwen".to_string()),
                tab_id: Some("tab-b".to_string()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                repo_root: Some("/tmp/repo".to_string()),
                worktree_root: Some("/tmp/repo-b".to_string()),
                branch: Some("feature/b".to_string()),
                base_branch: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: Some("workstation".to_string()),
                pid: Some(222),
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
                status: Some("live".to_string()),
            })
            .expect("insert branch b session");

        let sessions = store
            .hive_sessions(&HiveSessionsRequest {
                session: Some("shared-session".to_string()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                repo_root: Some("/tmp/repo".to_string()),
                worktree_root: None,
                branch: None,
                workspace: Some("shared".to_string()),
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(16),
            })
            .expect("query sessions");

        assert_eq!(sessions.sessions.len(), 2);
        assert!(
            sessions
                .sessions
                .iter()
                .any(|session| session.branch.as_deref() == Some("feature/a"))
        );
        assert!(
            sessions
                .sessions
                .iter()
                .any(|session| session.branch.as_deref() == Some("feature/b"))
        );

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn hive_sessions_preserve_service_hive_metadata() {
        let dir = std::env::temp_dir().join(format!("memd-hive-service-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "shell-a".to_string(),
                agent: Some("agent-shell".to_string()),
                effective_agent: Some("agent-shell@shell-a".to_string()),
                hive_system: Some("agent-shell".to_string()),
                hive_role: Some("runtime-shell".to_string()),
                worker_name: Some("agent-shell".to_string()),
                display_name: None,
                role: Some("runtime-shell".to_string()),
                capabilities: vec![
                    "shell".to_string(),
                    "exec".to_string(),
                    "workspace".to_string(),
                ],
                hive_groups: vec!["runtime-core".to_string(), "dependency-owners".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("worker".to_string()),
                heartbeat_model: Some("llama-desktop/qwen".to_string()),
                tab_id: None,
                project: Some("openclaw".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("stack-alpha".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: Some("vm-a".to_string()),
                pid: Some(333),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: Some("repair runtime dependency".to_string()),
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
                status: Some("live".to_string()),
            })
            .expect("insert service hive");

        let sessions = store
            .hive_sessions(&HiveSessionsRequest {
                session: Some("shell-a".to_string()),
                project: Some("openclaw".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                workspace: Some("stack-alpha".to_string()),
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(8),
            })
            .expect("query service hive");

        assert_eq!(sessions.sessions.len(), 1);
        let hive = &sessions.sessions[0];
        assert_eq!(hive.hive_system.as_deref(), Some("agent-shell"));
        assert_eq!(hive.hive_role.as_deref(), Some("runtime-shell"));
        assert_eq!(hive.authority.as_deref(), Some("worker"));
        assert!(hive.capabilities.iter().any(|value| value == "shell"));
        assert!(hive.capabilities.iter().any(|value| value == "exec"));

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn retire_hive_session_removes_scope_sibling_rows_for_same_session() {
        let dir = std::env::temp_dir().join(format!("memd-hive-retire-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "shared-session".to_string(),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@shared-session".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("codex".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["project:demo".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some("llama-desktop/qwen".to_string()),
                tab_id: Some("tab-a".to_string()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: Some("workstation".to_string()),
                pid: Some(111),
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
                status: Some("live".to_string()),
            })
            .expect("insert codex session");
        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "shared-session".to_string(),
                agent: Some("claude-code".to_string()),
                effective_agent: Some("claude-code@shared-session".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("claude-code".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["project:demo".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some("llama-desktop/qwen".to_string()),
                tab_id: Some("tab-b".to_string()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: Some("workstation".to_string()),
                pid: Some(222),
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
                status: Some("live".to_string()),
            })
            .expect("insert claude session");
        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "shared-session".to_string(),
                agent: Some("claude-code".to_string()),
                effective_agent: Some("claude-code@shared-session".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("claude-code".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["project:demo".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some("llama-desktop/qwen".to_string()),
                tab_id: Some("tab-c".to_string()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("other".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: Some("workstation".to_string()),
                pid: Some(333),
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
                status: Some("live".to_string()),
            })
            .expect("insert other workspace session");

        let retired = store
            .retire_hive_session(&HiveSessionRetireRequest {
                session: "shared-session".to_string(),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                workspace: Some("shared".to_string()),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@shared-session".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                host: Some("workstation".to_string()),
                reason: Some("superseded".to_string()),
            })
            .expect("retire codex session");
        assert_eq!(retired.retired, 2);
        assert!(
            retired
                .sessions
                .iter()
                .any(|record| record.agent.as_deref() == Some("codex"))
        );
        assert!(
            retired
                .sessions
                .iter()
                .any(|record| record.agent.as_deref() == Some("claude-code"))
        );

        let sessions = store
            .hive_sessions(&HiveSessionsRequest {
                session: Some("shared-session".to_string()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                workspace: Some("shared".to_string()),
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(8),
            })
            .expect("query remaining sessions");
        assert_eq!(sessions.sessions.len(), 0);

        let other_workspace_sessions = store
            .hive_sessions(&HiveSessionsRequest {
                session: Some("shared-session".to_string()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                workspace: Some("other".to_string()),
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(8),
            })
            .expect("query other workspace sessions");
        assert_eq!(other_workspace_sessions.sessions.len(), 1);
        assert_eq!(
            other_workspace_sessions.sessions[0].workspace.as_deref(),
            Some("other")
        );

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn hive_coordination_auto_retires_stale_session_without_owned_work() {
        let dir =
            std::env::temp_dir().join(format!("memd-hive-auto-retire-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "session-old".to_string(),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@session-old".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("Lorentz".to_string()),
                display_name: None,
                role: Some("worker".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["project:memd".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: None,
                tab_id: None,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: Some("workstation".to_string()),
                pid: Some(111),
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
                status: Some("active".to_string()),
            })
            .expect("insert stale session");

        let mut session = store
            .hive_sessions(&HiveSessionsRequest {
                session: Some("session-old".to_string()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                workspace: Some("shared".to_string()),
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(8),
            })
            .expect("load session")
            .sessions
            .into_iter()
            .next()
            .expect("session exists");
        session.last_seen = chrono::Utc::now() - chrono::TimeDelta::minutes(45);
        let conn = store.connect().expect("connect sqlite");
        conn.execute(
            "UPDATE hive_sessions SET last_seen = ?1, payload_json = ?2 WHERE session = ?3",
            params![
                session.last_seen.to_rfc3339(),
                serde_json::to_string(&session).expect("serialize stale session"),
                session.session.as_str(),
            ],
        )
        .expect("mark session stale");

        let sessions = store
            .hive_sessions(&HiveSessionsRequest {
                session: None,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                workspace: Some("shared".to_string()),
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(8),
            })
            .expect("list sessions");
        assert!(
            sessions
                .sessions
                .iter()
                .any(|session| session.session == "session-old")
        );

        let retired = store
            .auto_retire_stale_hive_sessions(
                Some("memd"),
                Some("main"),
                Some("shared"),
                chrono::Utc::now(),
            )
            .expect("auto retire");
        assert_eq!(retired.retired, vec!["session-old".to_string()]);

        let remaining = store
            .hive_sessions(&HiveSessionsRequest {
                session: None,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                workspace: Some("shared".to_string()),
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(8),
            })
            .expect("list sessions after retire");
        assert!(
            remaining
                .sessions
                .iter()
                .all(|session| session.session != "session-old")
        );

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn hive_sessions_filter_by_hive_identity() {
        let dir = std::env::temp_dir().join(format!("memd-hive-filter-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "shared-session".to_string(),
                agent: Some("agent-a".to_string()),
                effective_agent: Some("agent-a@shared-session".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("runtime-shell".to_string()),
                worker_name: Some("agent-a".to_string()),
                display_name: None,
                role: Some("runtime-shell".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["runtime-core".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("worker".to_string()),
                heartbeat_model: Some("llama-desktop/qwen".to_string()),
                tab_id: None,
                project: Some("repo-a".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: Some("vm-a".to_string()),
                pid: Some(111),
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
                status: Some("live".to_string()),
            })
            .expect("insert codex runtime shell session");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "shared-session".to_string(),
                agent: Some("agent-b".to_string()),
                effective_agent: Some("agent-b@shared-session".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("orchestrator".to_string()),
                worker_name: Some("agent-b".to_string()),
                display_name: None,
                role: Some("orchestrator".to_string()),
                capabilities: vec!["coordination".to_string()],
                hive_groups: vec!["openclaw-stack".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("coordinator".to_string()),
                heartbeat_model: Some("llama-desktop/qwen".to_string()),
                tab_id: None,
                project: Some("repo-a".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:9797".to_string()),
                base_url_healthy: Some(true),
                host: Some("vm-b".to_string()),
                pid: Some(222),
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
                status: Some("live".to_string()),
            })
            .expect("insert codex orchestrator session");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "shared-session".to_string(),
                agent: Some("agent-c".to_string()),
                effective_agent: Some("agent-c@shared-session".to_string()),
                hive_system: Some("claude-code".to_string()),
                hive_role: Some("runtime-shell".to_string()),
                worker_name: Some("agent-c".to_string()),
                display_name: None,
                role: Some("runtime-shell".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["runtime-core".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("worker".to_string()),
                heartbeat_model: Some("claude-opus".to_string()),
                tab_id: None,
                project: Some("repo-a".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:9898".to_string()),
                base_url_healthy: Some(true),
                host: Some("vm-a".to_string()),
                pid: Some(333),
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
                status: Some("live".to_string()),
            })
            .expect("insert claude runtime shell session");

        let codex_sessions = store
            .hive_sessions(&HiveSessionsRequest {
                session: Some("shared-session".to_string()),
                project: Some("repo-a".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                workspace: Some("shared".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(16),
            })
            .expect("query sessions by hive system");
        assert_eq!(codex_sessions.sessions.len(), 2);

        let runtime_session = store
            .hive_sessions(&HiveSessionsRequest {
                session: Some("shared-session".to_string()),
                project: Some("repo-a".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                workspace: Some("shared".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("runtime-shell".to_string()),
                host: Some("vm-a".to_string()),
                hive_group: None,
                active_only: Some(false),
                limit: Some(16),
            })
            .expect("query sessions by hive role and host");
        assert_eq!(runtime_session.sessions.len(), 1);
        assert_eq!(
            runtime_session.sessions[0].hive_system.as_deref(),
            Some("codex")
        );
        assert_eq!(
            runtime_session.sessions[0].hive_role.as_deref(),
            Some("runtime-shell")
        );
        assert_eq!(runtime_session.sessions[0].host.as_deref(), Some("vm-a"));

        let host_a_sessions = store
            .hive_sessions(&HiveSessionsRequest {
                session: Some("shared-session".to_string()),
                project: Some("repo-a".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                workspace: Some("shared".to_string()),
                hive_system: None,
                hive_role: None,
                host: Some("vm-a".to_string()),
                hive_group: None,
                active_only: Some(false),
                limit: Some(16),
            })
            .expect("query sessions by host");
        assert_eq!(host_a_sessions.sessions.len(), 2);
        assert!(
            host_a_sessions
                .sessions
                .iter()
                .all(|session| session.host.as_deref() == Some("vm-a"))
        );

        let runtime_group_sessions = store
            .hive_sessions(&HiveSessionsRequest {
                session: Some("shared-session".to_string()),
                project: Some("repo-a".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                workspace: Some("shared".to_string()),
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: Some("runtime-core".to_string()),
                active_only: Some(false),
                limit: Some(16),
            })
            .expect("query sessions by hive group");
        assert_eq!(runtime_group_sessions.sessions.len(), 2);
        assert!(runtime_group_sessions.sessions.iter().all(|session| {
            session
                .hive_groups
                .iter()
                .any(|value| value == "runtime-core")
        }));

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn hive_sessions_collapse_duplicate_rows_per_session_and_preserve_richer_identity() {
        let dir = std::env::temp_dir().join(format!("memd-hive-dedupe-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "codex-fresh".to_string(),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@codex-fresh".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("codex".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["coordination".to_string(), "memory".to_string()],
                hive_groups: vec!["project:memd".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: None,
                tab_id: Some("tab-alpha".to_string()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: None,
                pid: Some(123),
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
                status: Some("live".to_string()),
            })
            .expect("insert richer session row");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "codex-fresh".to_string(),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@codex-fresh".to_string()),
                hive_system: None,
                hive_role: None,
                worker_name: Some("codex".to_string()),
                display_name: None,
                role: None,
                capabilities: Vec::new(),
                hive_groups: vec!["project:memd".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: None,
                heartbeat_model: None,
                tab_id: Some("tab-alpha".to_string()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: None,
                host: None,
                pid: Some(123),
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
                status: Some("live".to_string()),
            })
            .expect("insert newer sparse session row");

        let response = store
            .hive_sessions(&HiveSessionsRequest {
                session: None,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                workspace: Some("shared".to_string()),
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(16),
            })
            .expect("query deduped sessions");

        assert_eq!(response.sessions.len(), 1);
        let session = &response.sessions[0];
        assert_eq!(session.session, "codex-fresh");
        assert_eq!(session.hive_system.as_deref(), Some("codex"));
        assert_eq!(session.hive_role.as_deref(), Some("agent"));
        assert_eq!(session.role.as_deref(), Some("agent"));
        assert_eq!(session.authority.as_deref(), Some("participant"));
        assert_eq!(
            session.capabilities,
            vec!["coordination".to_string(), "memory".to_string()]
        );

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn hive_sessions_collapse_duplicate_rows_prefers_stronger_newer_worker_identity() {
        let dir = std::env::temp_dir().join(format!("memd-hive-identity-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "session-live-openclaw".to_string(),
                agent: Some("openclaw".to_string()),
                effective_agent: Some("openclaw@session-live-openclaw".to_string()),
                hive_system: Some("openclaw".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("openclaw".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["coordination".to_string(), "memory".to_string()],
                hive_groups: vec!["project:memd".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: None,
                tab_id: Some("tab-alpha".to_string()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://100.104.154.24:8787".to_string()),
                base_url_healthy: Some(true),
                host: None,
                pid: Some(123),
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
                status: Some("live".to_string()),
            })
            .expect("insert older generic identity row");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "session-live-openclaw".to_string(),
                agent: Some("openclaw".to_string()),
                effective_agent: Some("openclaw@session-live-openclaw".to_string()),
                hive_system: Some("openclaw".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("Openclaw".to_string()),
                display_name: Some("Openclaw".to_string()),
                role: Some("agent".to_string()),
                capabilities: vec!["coordination".to_string(), "memory".to_string()],
                hive_groups: vec!["project:memd".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: None,
                tab_id: Some("tab-alpha".to_string()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://100.104.154.24:8787".to_string()),
                base_url_healthy: Some(true),
                host: None,
                pid: Some(456),
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
                status: Some("live".to_string()),
            })
            .expect("insert newer human identity row");

        let response = store
            .hive_sessions(&HiveSessionsRequest {
                session: Some("session-live-openclaw".to_string()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                workspace: Some("shared".to_string()),
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(16),
            })
            .expect("query merged session");

        assert_eq!(response.sessions.len(), 1);
        let session = &response.sessions[0];
        assert_eq!(session.worker_name.as_deref(), Some("Openclaw"));
        assert_eq!(session.display_name.as_deref(), Some("Openclaw"));

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn hive_sessions_collapse_does_not_backfill_generic_display_for_named_worker() {
        let dir = std::env::temp_dir().join(format!("memd-hive-display-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "session-live-openclaw".to_string(),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@session-live-openclaw".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("codex".to_string()),
                display_name: Some("Codex live-openclaw".to_string()),
                role: Some("agent".to_string()),
                capabilities: vec!["coordination".to_string(), "memory".to_string()],
                hive_groups: vec!["project:memd".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: None,
                tab_id: Some("tab-alpha".to_string()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://100.104.154.24:8787".to_string()),
                base_url_healthy: Some(true),
                host: None,
                pid: Some(123),
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
                status: Some("live".to_string()),
            })
            .expect("insert older generic row");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "session-live-openclaw".to_string(),
                agent: Some("openclaw".to_string()),
                effective_agent: Some("openclaw@session-live-openclaw".to_string()),
                hive_system: Some("openclaw".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("Openclaw".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["coordination".to_string(), "memory".to_string()],
                hive_groups: vec!["project:memd".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: None,
                tab_id: Some("tab-alpha".to_string()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://100.104.154.24:8787".to_string()),
                base_url_healthy: Some(true),
                host: None,
                pid: Some(456),
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
                status: Some("live".to_string()),
            })
            .expect("insert newer named row");

        let response = store
            .hive_sessions(&HiveSessionsRequest {
                session: Some("session-live-openclaw".to_string()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                workspace: Some("shared".to_string()),
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(16),
            })
            .expect("query merged session");

        assert_eq!(response.sessions.len(), 1);
        let session = &response.sessions[0];
        assert_eq!(session.worker_name.as_deref(), Some("Openclaw"));
        assert_eq!(session.display_name.as_deref(), None);

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn hive_board_ignores_handoff_scope_receipts_in_overlap_risks() {
        let dir =
            std::env::temp_dir().join(format!("memd-hive-overlap-noise-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "bee-1".to_string(),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@bee-1".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("worker".to_string()),
                worker_name: Some("Avicenna".to_string()),
                display_name: None,
                role: Some("worker".to_string()),
                capabilities: vec!["coordination".to_string()],
                hive_groups: vec!["project:memd".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: None,
                tab_id: None,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: None,
                pid: Some(101),
                topic_claim: Some("Parser lane".to_string()),
                scope_claims: vec!["crates/memd-client/src/main.rs".to_string()],
                task_id: Some("parser-refactor".to_string()),
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
                status: Some("live".to_string()),
            })
            .expect("insert session");

        store
            .record_hive_coordination_receipt(&HiveCoordinationReceiptRequest {
                kind: "queen_handoff".to_string(),
                actor_session: "queen-1".to_string(),
                actor_agent: Some("queen".to_string()),
                target_session: Some("bee-1".to_string()),
                task_id: Some("parser-refactor".to_string()),
                scope: Some("crates/memd-client/src/main.rs".to_string()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                summary: "Handoff to Avicenna (bee-1) task=parser-refactor scopes=crates/memd-client/src/main.rs".to_string(),
            })
            .expect("record handoff receipt");
        store
            .record_hive_coordination_receipt(&HiveCoordinationReceiptRequest {
                kind: "possible_work_overlap".to_string(),
                actor_session: "queen-1".to_string(),
                actor_agent: Some("queen".to_string()),
                target_session: Some("bee-1".to_string()),
                task_id: Some("parser-refactor".to_string()),
                scope: Some("crates/memd-client/src/main.rs".to_string()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                summary: "possible_work_overlap touches=crates/memd-client/src/main.rs sessions=bee-1,bee-2".to_string(),
            })
            .expect("record overlap receipt");

        let board = store
            .hive_board(&HiveBoardRequest {
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
            })
            .expect("read hive board");

        assert_eq!(board.overlap_risks.len(), 1);
        assert!(board.overlap_risks[0].contains("possible_work_overlap"));

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn hive_board_hides_low_signal_sender_sessions_without_active_tasks() {
        let dir =
            std::env::temp_dir().join(format!("memd-hive-sender-noise-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "sender-noise".to_string(),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@sender-noise".to_string()),
                hive_system: None,
                hive_role: None,
                worker_name: Some("codex".to_string()),
                display_name: None,
                role: None,
                capabilities: Vec::new(),
                hive_groups: vec!["project:memd".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: None,
                heartbeat_model: None,
                tab_id: None,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: None,
                pid: Some(301),
                topic_claim: Some("focus: task-current-noise".to_string()),
                scope_claims: Vec::new(),
                task_id: None,
                focus: Some("focus: task-current-noise".to_string()),
                pressure: Some("keep continuity tight".to_string()),
                next_recovery: Some("next: resume next step".to_string()),
                next_action: Some("focus: task-current-noise".to_string()),
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
                status: Some("live".to_string()),
            })
            .expect("insert low signal sender");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "worker-1".to_string(),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@worker-1".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("worker".to_string()),
                worker_name: Some("Avicenna".to_string()),
                display_name: None,
                role: Some("worker".to_string()),
                capabilities: vec!["coordination".to_string()],
                hive_groups: vec!["project:memd".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: None,
                tab_id: None,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: None,
                pid: Some(302),
                topic_claim: Some("Parser lane".to_string()),
                scope_claims: vec!["crates/memd-client/src/main.rs".to_string()],
                task_id: Some("parser-refactor".to_string()),
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
                status: Some("live".to_string()),
            })
            .expect("insert worker");

        let board = store
            .hive_board(&HiveBoardRequest {
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
            })
            .expect("read hive board");
        let roster = store
            .hive_roster(&HiveRosterRequest {
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
            })
            .expect("read hive roster");

        assert_eq!(board.active_bees.len(), 1);
        assert_eq!(board.active_bees[0].session, "worker-1");
        assert_eq!(roster.bees.len(), 1);
        assert_eq!(roster.bees[0].session, "worker-1");

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn hive_sessions_mark_proof_bees_stale_on_shorter_window() {
        let dir =
            std::env::temp_dir().join(format!("memd-hive-proof-presence-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "session-live-proof".to_string(),
                agent: Some("openclaw".to_string()),
                effective_agent: Some("openclaw@session-live-proof".to_string()),
                hive_system: Some("openclaw".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("Openclaw".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["coordination".to_string()],
                hive_groups: vec!["project:memd".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: None,
                tab_id: None,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: None,
                pid: Some(611),
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
                status: Some("live".to_string()),
            })
            .expect("insert proof bee");

        let mut session = store
            .hive_sessions(&HiveSessionsRequest {
                session: Some("session-live-proof".to_string()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(1),
            })
            .expect("load proof bee")
            .sessions
            .into_iter()
            .next()
            .expect("proof bee exists");
        session.last_seen = chrono::Utc::now() - chrono::TimeDelta::minutes(6);
        let conn = store.connect().expect("connect sqlite");
        conn.execute(
            "UPDATE hive_sessions SET last_seen = ?1, payload_json = ?2 WHERE session = ?3",
            params![
                session.last_seen.to_rfc3339(),
                serde_json::to_string(&session).expect("serialize proof bee"),
                session.session.as_str(),
            ],
        )
        .expect("age proof bee");

        let active = store
            .hive_sessions(&HiveSessionsRequest {
                session: None,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(true),
                limit: Some(8),
            })
            .expect("list active proof bees");
        assert!(
            active
                .sessions
                .iter()
                .all(|session| session.session != "session-live-proof")
        );

        let all = store
            .hive_sessions(&HiveSessionsRequest {
                session: Some("session-live-proof".to_string()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(1),
            })
            .expect("load proof bee after aging");
        assert_eq!(all.sessions[0].status, "stale");

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn hive_board_hides_sender_sessions_with_only_lane_path_and_no_task_signal() {
        let dir = std::env::temp_dir().join(format!(
            "memd-hive-sender-lane-only-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "sender-lane-only".to_string(),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@sender-lane-only".to_string()),
                hive_system: None,
                hive_role: None,
                worker_name: Some("codex".to_string()),
                display_name: None,
                role: None,
                capabilities: Vec::new(),
                hive_groups: vec!["project:memd".to_string()],
                lane_id: Some("/tmp/sessions".to_string()),
                hive_group_goal: None,
                authority: None,
                heartbeat_model: None,
                tab_id: None,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: Some("/tmp/sessions".to_string()),
                worktree_root: Some("/tmp/sessions".to_string()),
                branch: Some("feature/hive-shared".to_string()),
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: None,
                pid: Some(303),
                topic_claim: Some("focus: task-current-noise".to_string()),
                scope_claims: Vec::new(),
                task_id: None,
                focus: Some("focus: task-current-noise".to_string()),
                pressure: Some("keep continuity tight".to_string()),
                next_recovery: Some("next: resume next step".to_string()),
                next_action: Some("focus: task-current-noise".to_string()),
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
                status: Some("live".to_string()),
            })
            .expect("insert lane-only sender");

        let board = store
            .hive_board(&HiveBoardRequest {
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
            })
            .expect("read hive board");
        let roster = store
            .hive_roster(&HiveRosterRequest {
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
            })
            .expect("read hive roster");

        assert!(board.active_bees.is_empty());
        assert!(roster.bees.is_empty());

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn hive_board_hides_historical_lane_fault_noise_for_inactive_sessions() {
        let dir = std::env::temp_dir().join(format!(
            "memd-hive-lane-fault-noise-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "worker-1".to_string(),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@worker-1".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("worker".to_string()),
                worker_name: Some("Avicenna".to_string()),
                display_name: None,
                role: Some("worker".to_string()),
                capabilities: vec!["coordination".to_string()],
                hive_groups: vec!["project:memd".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: None,
                tab_id: None,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: None,
                pid: Some(401),
                topic_claim: Some("Parser lane".to_string()),
                scope_claims: vec!["crates/memd-client/src/main.rs".to_string()],
                task_id: Some("parser-refactor".to_string()),
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
                status: Some("live".to_string()),
            })
            .expect("insert worker");

        store
            .record_hive_coordination_receipt(&HiveCoordinationReceiptRequest {
                kind: "lane_fault".to_string(),
                actor_session: "queen-1".to_string(),
                actor_agent: Some("queen".to_string()),
                target_session: Some("old-worker".to_string()),
                task_id: Some("legacy-task".to_string()),
                scope: Some("crates/memd-client/src/main.rs".to_string()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                summary: "Old lane fault for old-worker".to_string(),
            })
            .expect("record stale lane fault");

        {
            let mut receipt = store
                .hive_coordination_receipts(&HiveCoordinationReceiptsRequest {
                    session: None,
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("shared".to_string()),
                    limit: Some(8),
                })
                .expect("load receipts")
                .receipts
                .into_iter()
                .next()
                .expect("stale lane fault receipt");
            receipt.created_at = chrono::Utc::now() - chrono::TimeDelta::minutes(30);
            let payload_json = serde_json::to_string(&receipt).expect("serialize aged receipt");
            let conn = store.connect().expect("connect sqlite store");
            conn.execute(
                "UPDATE hive_coordination_receipts SET created_at = ?1, payload_json = ?2 WHERE id = ?3",
                rusqlite::params![
                    receipt.created_at.to_rfc3339(),
                    payload_json,
                    receipt.id
                ],
            )
            .expect("age receipt row");
        }

        let board = store
            .hive_board(&HiveBoardRequest {
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
            })
            .expect("read hive board");

        assert!(board.blocked_bees.is_empty());
        assert!(board.lane_faults.is_empty());
        assert!(board.recommended_actions.is_empty());

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn hive_board_hides_lane_faults_when_only_actor_session_is_active() {
        let dir = std::env::temp_dir().join(format!(
            "memd-hive-lane-fault-target-filter-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "worker-1".to_string(),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@worker-1".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("worker".to_string()),
                worker_name: Some("Avicenna".to_string()),
                display_name: None,
                role: Some("worker".to_string()),
                capabilities: vec!["coordination".to_string()],
                hive_groups: vec!["project:memd".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: None,
                tab_id: None,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: None,
                pid: Some(501),
                topic_claim: Some("Parser lane".to_string()),
                scope_claims: vec!["crates/memd-client/src/main.rs".to_string()],
                task_id: Some("parser-refactor".to_string()),
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
                status: Some("live".to_string()),
            })
            .expect("insert active worker");

        store
            .record_hive_coordination_receipt(&HiveCoordinationReceiptRequest {
                kind: "queen_deny".to_string(),
                actor_session: "worker-1".to_string(),
                actor_agent: Some("codex".to_string()),
                target_session: Some("inactive-target".to_string()),
                task_id: Some("parser-refactor".to_string()),
                scope: Some("crates/memd-client/src/main.rs".to_string()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                summary: "Queen denied inactive target".to_string(),
            })
            .expect("record deny receipt");

        let board = store
            .hive_board(&HiveBoardRequest {
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
            })
            .expect("read hive board");

        assert!(board.blocked_bees.is_empty());
        assert!(board.lane_faults.is_empty());
        assert!(board.recommended_actions.is_empty());

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn open_migrates_legacy_hive_sessions_before_identity_indexes() {
        let dir = std::env::temp_dir().join(format!("legacy-hive-sessions-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("state.sqlite");
        let conn = Connection::open(&path).expect("open sqlite database");

        conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            CREATE TABLE hive_sessions (
              session_key TEXT PRIMARY KEY,
              session TEXT NOT NULL,
              project TEXT,
              namespace TEXT,
              workspace TEXT,
              status TEXT NOT NULL,
              last_seen TEXT NOT NULL,
              payload_json TEXT NOT NULL
            );
            "#,
        )
        .expect("create legacy hive_sessions");

        drop(conn);

        let store = SqliteStore::open(&path).expect("open migrated sqlite store");
        let conn = store.connect().expect("connect migrated sqlite store");
        let columns = {
            let mut stmt = conn
                .prepare("PRAGMA table_info(hive_sessions)")
                .expect("prepare table info");
            stmt.query_map([], |row| row.get::<_, String>(1))
                .expect("query table info")
                .collect::<Result<Vec<_>, _>>()
                .expect("collect hive session columns")
        };
        assert!(columns.iter().any(|value| value == "hive_system"));
        assert!(columns.iter().any(|value| value == "hive_role"));
        assert!(columns.iter().any(|value| value == "host"));

        let indexes = {
            let mut stmt = conn
                .prepare("PRAGMA index_list(hive_sessions)")
                .expect("prepare index list");
            stmt.query_map([], |row| row.get::<_, String>(1))
                .expect("query index list")
                .collect::<Result<Vec<_>, _>>()
                .expect("collect hive session indexes")
        };
        assert!(
            indexes
                .iter()
                .any(|value| value == "idx_hive_sessions_hive_system")
        );
        assert!(
            indexes
                .iter()
                .any(|value| value == "idx_hive_sessions_hive_role")
        );
        assert!(
            indexes
                .iter()
                .any(|value| value == "idx_hive_sessions_host")
        );

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn maintain_runtime_persists_report_receipt() {
        let dir = std::env::temp_dir().join(format!("runtime-maintain-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

        let report = store
            .maintain_runtime(&MaintainReportRequest {
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                session: Some("session-a".to_string()),
                mode: "scan".to_string(),
                apply: false,
            })
            .expect("run maintain runtime");

        assert_eq!(report.mode, "scan");
        assert!(report.receipt_id.is_some());
        assert!(
            report
                .findings
                .iter()
                .any(|line| line.contains("memory maintain"))
        );

        let conn = store.connect().expect("connect sqlite store");
        let persisted: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM runtime_maintenance_reports",
                [],
                |row| row.get(0),
            )
            .expect("count persisted maintenance reports");
        assert_eq!(persisted, 1);

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

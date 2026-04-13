use super::*;
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

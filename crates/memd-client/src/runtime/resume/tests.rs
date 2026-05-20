use super::*;
use std::sync::{Mutex, OnceLock};

static CWD_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn lock_cwd_mutation() -> std::sync::MutexGuard<'static, ()> {
    CWD_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("cwd mutation lock poisoned")
}

#[test]
fn lossless_full_working_packet_does_not_force_refresh() {
    use memd_schema::{CompactMemoryRecord, CompactionQualityReport, WorkingMemoryEvictionRecord};
    use std::collections::BTreeMap;

    let mut snapshot = ResumeSnapshot::empty();
    snapshot.context.records = (0..8)
        .map(|index| CompactMemoryRecord {
            id: uuid::Uuid::new_v4(),
            record: format!("stable context item {index}"),
        })
        .collect();
    snapshot.working.budget_chars = 1600;
    snapshot.working.used_chars = 1540;
    snapshot.working.remaining_chars = 60;
    snapshot.working.policy.admission_limit = 8;
    snapshot.working.records = (0..7)
        .map(|index| CompactMemoryRecord {
            id: uuid::Uuid::new_v4(),
            record: format!("stable working item {index}"),
        })
        .collect();
    snapshot.working.compaction_quality = Some(CompactionQualityReport {
        admitted: 7,
        evicted: 0,
        per_kind_admitted: BTreeMap::new(),
        per_kind_evicted: BTreeMap::new(),
        chars_per_kind_admitted: BTreeMap::new(),
        budget_chars: 1600,
        used_chars: 1540,
    });

    assert!(!resume_refresh_recommended(
        None,
        &snapshot.working,
        &snapshot.inbox
    ));
    assert_eq!(snapshot.context_pressure(), "low");

    snapshot
        .working
        .compaction_quality
        .as_mut()
        .expect("quality")
        .evicted = 1;
    assert!(resume_refresh_recommended(
        None,
        &snapshot.working,
        &snapshot.inbox
    ));
    assert_eq!(snapshot.context_pressure(), "medium");

    snapshot.working.compaction_quality = Some(CompactionQualityReport {
        admitted: 7,
        evicted: 2,
        per_kind_admitted: BTreeMap::new(),
        per_kind_evicted: BTreeMap::new(),
        chars_per_kind_admitted: BTreeMap::new(),
        budget_chars: 1600,
        used_chars: 1540,
    });
    snapshot.working.evicted = [
            "id=wake-refresh | kind=status | tags=checkpoint,current-task,auto-short-term,bundle-refresh,refresh | c=status: wake project=memd focus=current task",
            "id=next | kind=status | tags=checkpoint,current-task,continuity | c=CURRENT NEXT ACTION after commit 117f7f9",
        ]
        .into_iter()
        .map(|record| WorkingMemoryEvictionRecord {
            id: uuid::Uuid::new_v4(),
            record: record.to_string(),
            reason: "evicted_by_status_cap;kind=Status;status=active".to_string(),
        })
        .collect();

    assert!(!resume_refresh_recommended(
        None,
        &snapshot.working,
        &snapshot.inbox
    ));
    assert_eq!(snapshot.context_pressure(), "low");
}

#[test]
fn cached_handoff_refreshes_live_repo_dirty_state() {
    let root = std::env::temp_dir().join(format!("memd-handoff-live-{}", uuid::Uuid::new_v4()));
    let bundle = root.join(".memd");
    fs::create_dir_all(bundle.join("state")).expect("create bundle state");
    let git = std::process::Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("init")
        .output()
        .expect("run git init");
    assert!(git.status.success());
    fs::write(
        bundle.join("config.json"),
        r#"{
  "voice_mode": "caveman-ultra"
}
"#,
    )
    .expect("write config");
    fs::write(root.join("notes.txt"), "live dirty state\n").expect("write dirty file");

    let resume = ResumeSnapshot {
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        agent: Some("codex".to_string()),
        workspace: None,
        visibility: Some("all".to_string()),
        route: "auto".to_string(),
        intent: "current_task".to_string(),
        context: memd_schema::CompactContextResponse {
            route: RetrievalRoute::Auto,
            intent: RetrievalIntent::CurrentTask,
            retrieval_order: Vec::new(),
            records: Vec::new(),
        },
        working: memd_schema::WorkingMemoryResponse {
            route: RetrievalRoute::Auto,
            intent: RetrievalIntent::CurrentTask,
            retrieval_order: Vec::new(),
            budget_chars: 0,
            used_chars: 0,
            remaining_chars: 0,
            truncated: false,
            policy: memd_schema::WorkingMemoryPolicyState {
                admission_limit: 0,
                max_chars_per_item: 0,
                budget_chars: 0,
                rehydration_limit: 0,
            },
            records: Vec::new(),
            evicted: Vec::new(),
            rehydration_queue: Vec::new(),
            traces: Vec::new(),
            semantic_consolidation: None,
            procedures: Vec::new(),
            compaction_quality: None,
        },
        inbox: memd_schema::MemoryInboxResponse {
            route: RetrievalRoute::Auto,
            intent: RetrievalIntent::CurrentTask,
            items: Vec::new(),
        },
        workspaces: memd_schema::WorkspaceMemoryResponse {
            workspaces: Vec::new(),
        },
        sources: memd_schema::SourceMemoryResponse {
            sources: Vec::new(),
        },
        semantic: None,
        claims: SessionClaimsState::default(),
        recent_repo_changes: vec!["repo_dirty_total=999 tracked=999 untracked=0".to_string()],
        change_summary: Vec::new(),
        resume_state_age_minutes: None,
        refresh_recommended: false,
        atlas_region_hints: Vec::new(),
        handoff_quality: None,
        files_touched: Vec::new(),
        un_read_paths: Vec::new(),
        preferences: Vec::new(),
    };
    let mut handoff = HandoffSnapshot {
        generated_at: Utc::now(),
        resume,
        sources: memd_schema::SourceMemoryResponse {
            sources: Vec::new(),
        },
        voice_mode: "normal".to_string(),
        target_session: None,
        target_bundle: Some(bundle.display().to_string()),
    };

    refresh_handoff_local_recovery_state(&bundle, &mut handoff);

    assert_eq!(handoff.voice_mode, "caveman-ultra");
    assert!(handoff.resume.recent_repo_changes[0].starts_with("repo_dirty_total="));
    assert_ne!(
        handoff.resume.recent_repo_changes[0],
        "repo_dirty_total=999 tracked=999 untracked=0"
    );
    assert!(
        handoff
            .resume
            .recent_repo_changes
            .iter()
            .any(|change| change.contains("notes.txt"))
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn latest_raw_spine_next_action_beats_stale_cached_next_action() {
    let root = std::env::temp_dir().join(format!("memd-raw-next-{}", uuid::Uuid::new_v4()));
    let bundle = root.join(".memd");
    let state = bundle.join("state");
    std::fs::create_dir_all(&state).expect("create state");
    std::fs::write(
            state.join("raw-spine.jsonl"),
            r#"{"id":"raw-new","tags":["next-agent","capability-sync"],"content_preview":"CURRENT NEXT ACTION: finish host-local CLI installer after 642b5a3","recorded_at":"2026-05-15T20:10:49Z"}
{"id":"raw-old","tags":["next-agent"],"content_preview":"CURRENT NEXT ACTION: old payload proof","recorded_at":"2026-05-15T19:10:49Z"}"#,
        )
        .expect("write raw spine");

    let raw_next = latest_raw_spine_next_action(&bundle).expect("raw next action");
    let best = best_next_action_record(vec![
            "id=old | kind=decision | tags=next-agent | upd=1778874974 | c=CURRENT NEXT ACTION: old fresh-machine payload proof".to_string(),
            raw_next,
        ])
        .expect("best next action");

    assert!(best.contains("finish host-local CLI installer"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn latest_raw_spine_next_action_uses_newest_matching_record() {
    let root = std::env::temp_dir().join(format!("memd-raw-next-newest-{}", uuid::Uuid::new_v4()));
    let bundle = root.join(".memd");
    let state = bundle.join("state");
    std::fs::create_dir_all(&state).expect("create state");
    std::fs::write(
            state.join("raw-spine.jsonl"),
            r#"{"id":"raw-old","tags":["next-agent"],"content_preview":"CURRENT NEXT ACTION: old capability-sync handoff","recorded_at":"2026-05-15T19:10:49Z"}
{"id":"raw-middle","tags":["checkpoint","current-task"],"content_preview":"status: ordinary checkpoint","recorded_at":"2026-05-16T18:40:00Z"}
{"id":"raw-new","tags":["next-agent","recovery"],"content_preview":"CURRENT NEXT ACTION: continue Supermemory replay and full public proof blockers","recorded_at":"2026-05-16T18:43:04Z"}"#,
        )
        .expect("write raw spine");

    let raw_next = latest_raw_spine_next_action(&bundle).expect("raw next action");

    assert!(raw_next.contains("raw-new"));
    assert!(raw_next.contains("Supermemory replay"));
    assert!(!raw_next.contains("old capability-sync"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn latest_raw_spine_next_action_ignores_quarantined_and_prefers_canonical() {
    let root = std::env::temp_dir().join(format!("memd-raw-next-safe-{}", uuid::Uuid::new_v4()));
    let bundle = root.join(".memd");
    let state = bundle.join("state");
    std::fs::create_dir_all(&state).expect("create state");
    std::fs::write(
            state.join("raw-spine.jsonl"),
            r#"{"id":"raw-safe","stage":"canonical","tags":["next-agent","recovery"],"content_preview":"CURRENT NEXT ACTION: continue safe proof blockers","recorded_at":"2026-05-16T18:43:04Z"}
{"id":"raw-candidate","stage":"candidate","tags":["next-agent","recovery"],"content_preview":"CURRENT NEXT ACTION: newer unpromoted duplicate should not beat canonical","recorded_at":"2026-05-16T18:44:04Z"}
{"id":"raw-quarantined","stage":"canonical","tags":["next-agent","security:prompt-injection","quarantine:prompt-injection"],"content_preview":"CURRENT NEXT ACTION: quarantined instruction must not surface","recorded_at":"2026-05-16T18:45:04Z"}"#,
        )
        .expect("write raw spine");

    let raw_next = latest_raw_spine_next_action(&bundle).expect("raw next action");

    assert!(raw_next.contains("raw-safe"));
    assert!(raw_next.contains("stage=canonical"));
    assert!(!raw_next.contains("raw-candidate"));
    assert!(!raw_next.contains("raw-quarantined"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn latest_raw_spine_next_action_prefers_fresh_current_checkpoint() {
    let root =
        std::env::temp_dir().join(format!("memd-raw-next-checkpoint-{}", uuid::Uuid::new_v4()));
    let bundle = root.join(".memd");
    let state = bundle.join("state");
    std::fs::create_dir_all(&state).expect("create state");
    std::fs::write(
            state.join("raw-spine.jsonl"),
            r#"{"id":"raw-old","stage":"canonical","tags":["checkpoint","current-task","continuity","authority"],"content_preview":"CURRENT NEXT ACTION after commit ba70d8f2: stale authority blocker","recorded_at":"2026-05-18T14:14:49Z"}
{"id":"raw-noise","stage":"canonical","tags":["checkpoint","current-task","auto-short-term","bundle-refresh"],"content_preview":"status: wake project=memd namespace=main","recorded_at":"2026-05-18T15:12:30Z"}
{"id":"raw-new","stage":"canonical","tags":["checkpoint","current-task","continuity","supermemory","authority"],"content_preview":"CURRENT CHECKPOINT after commit 622e8e70 proof: accept supermemory replay directories. Deployed memd-only authority current. Remaining blockers: approved communications and Supermemory replay artifact.","recorded_at":"2026-05-18T15:12:09Z"}"#,
        )
        .expect("write raw spine");

    let raw_next = latest_raw_spine_next_action(&bundle).expect("raw next action");

    assert!(raw_next.contains("raw-new"), "{raw_next}");
    assert!(raw_next.contains("622e8e70"), "{raw_next}");
    assert!(!raw_next.contains("raw-old"), "{raw_next}");
    assert!(!raw_next.contains("raw-noise"), "{raw_next}");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn latest_raw_spine_next_action_accepts_current_checkpoint_without_continuity_tag() {
    let root = std::env::temp_dir().join(format!(
        "memd-raw-next-plain-checkpoint-{}",
        uuid::Uuid::new_v4()
    ));
    let bundle = root.join(".memd");
    let state = bundle.join("state");
    std::fs::create_dir_all(&state).expect("create state");
    std::fs::write(
            state.join("raw-spine.jsonl"),
            r#"{"id":"raw-wake","stage":"canonical","tags":["checkpoint","current-task","auto-short-term","bundle-refresh","wake"],"content_preview":"status: wake project=memd namespace=main focus=\"stale cached decision\"","recorded_at":"2026-05-20T14:04:01Z"}
{"id":"raw-current","stage":"canonical","tags":["checkpoint","current-task"],"content_preview":"CURRENT CHECKPOINT after commit fefb5866: continuity now surfaces exact unblock actions. Git clean, live map current, hive no blockers. Remaining real gates: approved messages/email metadata or explicit approved-zero; Supermemory replay artifact or process-local approved credential after Bitwarden unlock.","recorded_at":"2026-05-20T14:03:07Z"}
{"id":"raw-stale","stage":"canonical","tags":["checkpoint","current-task","continuity"],"content_preview":"CURRENT CHECKPOINT after commit 75bf0736: stale live-map contract slice.","recorded_at":"2026-05-20T13:57:21Z"}"#,
        )
        .expect("write raw spine");

    let raw_next = latest_raw_spine_next_action(&bundle).expect("raw next action");

    assert!(raw_next.contains("raw-current"), "{raw_next}");
    assert!(raw_next.contains("fefb5866"), "{raw_next}");
    assert!(!raw_next.contains("raw-wake"), "{raw_next}");
    assert!(!raw_next.contains("raw-stale"), "{raw_next}");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn continuity_blocker_prefers_explicit_next_action_blockers_over_refresh_pressure() {
    let mut snapshot = ResumeSnapshot::empty();
    snapshot.refresh_recommended = true;
    snapshot.preferences = vec![
            "id=next | kind=decision | tags=next-agent,recovery | upd=1778958400 | c=CURRENT NEXT ACTION: continue P5 proof. Remaining blockers are Supermemory same-fixture replay and full external public proof opt-in.".to_string(),
        ];

    let continuity = snapshot.continuity_capsule();

    assert!(
        continuity
            .next_action
            .as_deref()
            .unwrap_or_default()
            .contains("continue P5 proof")
    );
    assert_eq!(
        continuity.blocker.as_deref(),
        Some("Supermemory same-fixture replay and full external public proof opt-in")
    );
}

#[test]
fn continuity_next_ignores_auto_bundle_refresh_status_records() {
    let mut snapshot = ResumeSnapshot::empty();
    snapshot.preferences = vec![
            "id=auto | stage=canonical | kind=decision | status=active | tags=checkpoint,current-task,auto-short-term,bundle-refresh,wake | upd=1779285841 | c=status: wake project=memd namespace=main focus=\"id=old | kind=decision | c=stale\" source_path=wake".to_string(),
            "id=current | stage=canonical | kind=decision | status=active | tags=checkpoint,current-task | upd=1779285787 | c=CURRENT CHECKPOINT after commit fefb5866: continuity now surfaces exact unblock actions. Remaining real gates: approved messages/email metadata or explicit approved-zero; Supermemory replay artifact or process-local approved credential after Bitwarden unlock.".to_string(),
        ];

    let continuity = snapshot.continuity_capsule();

    let next = continuity.next_action.as_deref().unwrap_or_default();
    assert!(next.contains("fefb5866"), "{next}");
    assert!(!next.contains("status: wake project"), "{next}");
}

#[test]
fn continuity_blocker_uses_raw_next_action_before_compaction() {
    let mut snapshot = ResumeSnapshot::empty();
    snapshot.refresh_recommended = true;
    let filler = "route-safe proof gate ".repeat(200);
    snapshot.preferences = vec![format!(
        "id=next | kind=decision | tags=next-agent,recovery | upd=1778958400 | c=CURRENT NEXT ACTION: continue P5 proof. {filler} Remaining blockers are Supermemory replay and full public proof."
    )];

    let continuity = snapshot.continuity_capsule();

    assert!(
        !continuity
            .next_action
            .as_deref()
            .unwrap_or_default()
            .contains("Remaining blockers"),
        "test must exercise compacted next-action text"
    );
    assert_eq!(
        continuity.blocker.as_deref(),
        Some("Supermemory replay and full public proof")
    );
}

#[test]
fn continuity_blocker_ignores_medium_refresh_pressure_without_explicit_blocker() {
    let mut snapshot = ResumeSnapshot::empty();
    snapshot.refresh_recommended = true;
    snapshot.preferences = vec![
            "id=next | kind=decision | tags=next-agent,recovery | upd=1778958400 | c=CURRENT NEXT ACTION: continue live-state authority work.".to_string(),
        ];

    assert_eq!(snapshot.context_pressure(), "low");

    let continuity = snapshot.continuity_capsule();

    assert!(
        continuity
            .next_action
            .as_deref()
            .unwrap_or_default()
            .contains("continue live-state authority")
    );
    assert_eq!(continuity.blocker, None);
}

#[test]
fn resume_defaults_bind_repo_identity_without_runtime_config() {
    let _cwd_lock = lock_cwd_mutation();
    let temp_root =
        std::env::temp_dir().join(format!("memd-resume-defaults-{}", uuid::Uuid::new_v4()));
    let repo_root = temp_root.join("repo-b");
    let bundle_root = repo_root.join(".memd");
    let original_cwd = std::env::current_dir().expect("read cwd");

    fs::create_dir_all(repo_root.join(".git")).expect("create repo git dir");
    std::env::set_current_dir(&repo_root).expect("set repo cwd");

    let args = ResumeArgs {
        output: bundle_root.clone(),
        project: None,
        namespace: None,
        agent: None,
        workspace: None,
        visibility: None,
        route: None,
        intent: None,
        limit: None,
        rehydration_limit: None,
        semantic: false,
        prompt: false,
        summary: false,
    };
    let (project, namespace) = infer_resume_bundle_identity_defaults(&bundle_root);
    assert_eq!(project.as_deref(), Some("repo-b"));
    assert_eq!(namespace.as_deref(), Some("main"));

    std::env::set_current_dir(&original_cwd).expect("restore cwd");
    fs::remove_dir_all(temp_root).expect("cleanup temp root");
    let _ = args;
}

#[test]
fn truth_summary_prefers_compact_working_state() {
    let snapshot = ResumeSnapshot {
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        agent: Some("codex".to_string()),
        workspace: Some("team-alpha".to_string()),
        visibility: Some("workspace".to_string()),
        route: "auto".to_string(),
        intent: "current_task".to_string(),
        context: memd_schema::CompactContextResponse {
            route: RetrievalRoute::ProjectFirst,
            intent: RetrievalIntent::CurrentTask,
            retrieval_order: vec![MemoryScope::Project, MemoryScope::Synced],
            records: vec![memd_schema::CompactMemoryRecord {
                id: uuid::Uuid::new_v4(),
                record: "compact context: keep startup surfaces tight".to_string(),
            }],
        },
        working: memd_schema::WorkingMemoryResponse {
            route: RetrievalRoute::ProjectFirst,
            intent: RetrievalIntent::CurrentTask,
            retrieval_order: vec![MemoryScope::Project],
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
                record: "working record: compact truth should steer the prompt".to_string(),
            }],
            evicted: Vec::new(),
            rehydration_queue: Vec::new(),
            traces: Vec::new(),
            semantic_consolidation: None,
            procedures: vec![],

            compaction_quality: None,
        },
        inbox: memd_schema::MemoryInboxResponse {
            route: RetrievalRoute::ProjectFirst,
            intent: RetrievalIntent::CurrentTask,
            items: Vec::new(),
        },
        workspaces: memd_schema::WorkspaceMemoryResponse {
            workspaces: Vec::new(),
        },
        sources: memd_schema::SourceMemoryResponse {
            sources: Vec::new(),
        },
        semantic: None,
        claims: SessionClaimsState::default(),
        recent_repo_changes: vec!["repo change: compact truth should steer the prompt".to_string()],
        change_summary: vec!["change summary: compact truth should steer the prompt".to_string()],
        resume_state_age_minutes: None,
        refresh_recommended: false,
        atlas_region_hints: Vec::new(),

        handoff_quality: None,

        files_touched: Vec::new(),
        un_read_paths: Vec::new(),
        preferences: Vec::new(),
    };

    let summary = build_truth_summary(&snapshot);
    assert_eq!(summary.compact_records, 2);
    assert!(
        summary
            .records
            .iter()
            .any(|record| record.lane == "live_truth" && record.preview.contains("compact truth"))
    );
    assert!(
        summary
            .records
            .iter()
            .any(|record| record.lane == "working_set" && record.preview.contains("compact truth"))
    );
}

#[test]
fn trim_resume_context_records_drops_working_duplicates_and_status_noise() {
    let duplicate_id = uuid::Uuid::new_v4();
    let durable_id = uuid::Uuid::new_v4();
    let status_id = uuid::Uuid::new_v4();
    let live_truth_id = uuid::Uuid::new_v4();
    let working = memd_schema::WorkingMemoryResponse {
            route: RetrievalRoute::ProjectFirst,
            intent: RetrievalIntent::CurrentTask,
            retrieval_order: vec![MemoryScope::Project],
            budget_chars: 1600,
            used_chars: 220,
            remaining_chars: 1380,
            truncated: false,
            policy: memd_schema::WorkingMemoryPolicyState {
                admission_limit: 8,
                max_chars_per_item: 220,
                budget_chars: 1600,
                rehydration_limit: 4,
            },
            records: vec![memd_schema::CompactMemoryRecord {
                id: duplicate_id,
                record: "id=dup | stage=canonical | scope=project | kind=decision | status=active | c=keep working anchor".to_string(),
            }],
            evicted: Vec::new(),
            rehydration_queue: Vec::new(),
            traces: Vec::new(),
            semantic_consolidation: None,
            procedures: Vec::new(),
            compaction_quality: None,
        };
    let mut context = memd_schema::CompactContextResponse {
            route: RetrievalRoute::ProjectFirst,
            intent: RetrievalIntent::CurrentTask,
            retrieval_order: vec![MemoryScope::Project],
            records: vec![
                memd_schema::CompactMemoryRecord {
                    id: duplicate_id,
                    record: "id=dup | stage=canonical | scope=project | kind=decision | status=active | c=keep working anchor".to_string(),
                },
                memd_schema::CompactMemoryRecord {
                    id: durable_id,
                    record: "id=durable | stage=canonical | scope=project | kind=fact | status=active | c=keep durable truth".to_string(),
                },
                memd_schema::CompactMemoryRecord {
                    id: status_id,
                    record: "id=status | stage=canonical | scope=project | kind=status | status=active | c=status: wake working=7".to_string(),
                },
                memd_schema::CompactMemoryRecord {
                    id: live_truth_id,
                    record: "id=live | stage=canonical | scope=local | kind=live_truth | status=active | c=repo_state: clean".to_string(),
                },
                memd_schema::CompactMemoryRecord {
                    id: uuid::Uuid::new_v4(),
                    record: "id=live-2 | stage=canonical | scope=local | kind=live_truth | status=active | c=file_edited: docs/x".to_string(),
                },
            ],
        };

    trim_resume_context_records(&mut context, &working);

    assert_eq!(context.records.len(), 2);
    assert!(context.records.iter().any(|record| record.id == durable_id));
    assert!(
        context
            .records
            .iter()
            .any(|record| record.id == live_truth_id)
    );
    assert!(
        context
            .records
            .iter()
            .all(|record| record.id != duplicate_id)
    );
    assert!(context.records.iter().all(|record| record.id != status_id));
}

#[test]
fn truth_summary_uses_top_source_provenance_for_non_live_truth_lanes() {
    let snapshot = ResumeSnapshot {
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        agent: Some("codex".to_string()),
        workspace: Some("team-alpha".to_string()),
        visibility: Some("workspace".to_string()),
        route: "auto".to_string(),
        intent: "current_task".to_string(),
        context: memd_schema::CompactContextResponse {
            route: RetrievalRoute::ProjectFirst,
            intent: RetrievalIntent::CurrentTask,
            retrieval_order: vec![MemoryScope::Project, MemoryScope::Synced],
            records: vec![memd_schema::CompactMemoryRecord {
                id: uuid::Uuid::new_v4(),
                record: "context truth: source provenance should survive".to_string(),
            }],
        },
        working: memd_schema::WorkingMemoryResponse {
            route: RetrievalRoute::ProjectFirst,
            intent: RetrievalIntent::CurrentTask,
            retrieval_order: vec![MemoryScope::Project],
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
                record: "working truth: source provenance should survive".to_string(),
            }],
            evicted: Vec::new(),
            rehydration_queue: Vec::new(),
            traces: Vec::new(),
            semantic_consolidation: None,
            procedures: vec![],

            compaction_quality: None,
        },
        inbox: memd_schema::MemoryInboxResponse {
            route: RetrievalRoute::ProjectFirst,
            intent: RetrievalIntent::CurrentTask,
            items: Vec::new(),
        },
        workspaces: memd_schema::WorkspaceMemoryResponse {
            workspaces: Vec::new(),
        },
        sources: memd_schema::SourceMemoryResponse {
            sources: vec![memd_schema::SourceMemoryRecord {
                source_agent: Some("codex@test".to_string()),
                source_system: Some("hook-capture".to_string()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("team-alpha".to_string()),
                visibility: memd_schema::MemoryVisibility::Workspace,
                item_count: 2,
                active_count: 2,
                candidate_count: 0,
                derived_count: 0,
                synthetic_count: 0,
                contested_count: 0,
                avg_confidence: 0.93,
                trust_score: 0.97,
                last_seen_at: Some(Utc::now()),
                tags: vec!["raw-spine".to_string(), "correction".to_string()],
            }],
        },
        semantic: None,
        claims: SessionClaimsState::default(),
        recent_repo_changes: Vec::new(),
        change_summary: Vec::new(),
        resume_state_age_minutes: None,
        refresh_recommended: false,
        atlas_region_hints: Vec::new(),

        handoff_quality: None,

        files_touched: Vec::new(),
        un_read_paths: Vec::new(),
        preferences: Vec::new(),
    };

    let summary = build_truth_summary(&snapshot);
    let working = summary
        .records
        .iter()
        .find(|record| record.lane == "working_set")
        .expect("working record");
    assert_eq!(working.provenance, "codex@test / hook-capture");
}

#[test]
fn continuity_answers_surface_core_resume_questions() {
    let snapshot = ResumeSnapshot {
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        agent: Some("codex".to_string()),
        workspace: Some("team-alpha".to_string()),
        visibility: Some("workspace".to_string()),
        route: "auto".to_string(),
        intent: "current_task".to_string(),
        context: memd_schema::CompactContextResponse {
            route: RetrievalRoute::ProjectFirst,
            intent: RetrievalIntent::CurrentTask,
            retrieval_order: vec![MemoryScope::Project],
            records: vec![memd_schema::CompactMemoryRecord {
                id: uuid::Uuid::new_v4(),
                record: "context truth".to_string(),
            }],
        },
        working: memd_schema::WorkingMemoryResponse {
            route: RetrievalRoute::ProjectFirst,
            intent: RetrievalIntent::CurrentTask,
            retrieval_order: vec![MemoryScope::Project],
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
                record: "working truth".to_string(),
            }],
            evicted: Vec::new(),
            rehydration_queue: vec![memd_schema::MemoryRehydrationRecord {
                id: Some(uuid::Uuid::new_v4()),
                kind: "handoff".to_string(),
                label: "handoff".to_string(),
                summary: "resume next step".to_string(),
                recorded_at: None,
                source_agent: Some("codex".to_string()),
                source_system: Some("memd".to_string()),
                source_path: Some("handoff.md".to_string()),
                reason: Some("resume".to_string()),
                source_quality: None,
            }],
            traces: Vec::new(),
            semantic_consolidation: None,
            procedures: vec![],

            compaction_quality: None,
        },
        inbox: memd_schema::MemoryInboxResponse {
            route: RetrievalRoute::ProjectFirst,
            intent: RetrievalIntent::CurrentTask,
            items: Vec::new(),
        },
        workspaces: memd_schema::WorkspaceMemoryResponse {
            workspaces: Vec::new(),
        },
        sources: memd_schema::SourceMemoryResponse {
            sources: Vec::new(),
        },
        semantic: None,
        claims: SessionClaimsState::default(),
        recent_repo_changes: vec!["status M src/lib.rs".to_string()],
        change_summary: vec!["changed focus".to_string()],
        resume_state_age_minutes: None,
        refresh_recommended: false,
        atlas_region_hints: Vec::new(),

        handoff_quality: None,

        files_touched: Vec::new(),
        un_read_paths: Vec::new(),
        preferences: Vec::new(),
    };

    assert_eq!(
        snapshot.continuity_doing().as_deref(),
        Some("working truth")
    );
    assert_eq!(
        snapshot.continuity_left_off().as_deref(),
        Some("resume next step")
    );
    assert_eq!(
        snapshot.continuity_changed().as_deref(),
        Some("changed focus")
    );
    assert_eq!(
        snapshot.continuity_next().as_deref(),
        Some("resume next step")
    );
    let mut prioritized = snapshot.clone();
    prioritized.working.rehydration_queue.insert(
        0,
        memd_schema::MemoryRehydrationRecord {
            id: Some(uuid::Uuid::new_v4()),
            kind: "working_memory_record".to_string(),
            label: "fact".to_string(),
            summary: "id=fact | kind=fact | c=background".to_string(),
            recorded_at: None,
            source_agent: Some("codex".to_string()),
            source_system: Some("memd".to_string()),
            source_path: None,
            reason: Some("evicted_by_budget".to_string()),
            source_quality: None,
        },
    );
    prioritized
            .working
            .rehydration_queue
            .push(memd_schema::MemoryRehydrationRecord {
                id: Some(uuid::Uuid::new_v4()),
                kind: "working_memory_record".to_string(),
                label: "decision".to_string(),
                summary: "id=decision | kind=decision | tags=next-agent,p0-handoff | c=deploy patched server next".to_string(),
                recorded_at: None,
                source_agent: Some("codex".to_string()),
                source_system: Some("memd".to_string()),
                source_path: None,
                reason: Some("evicted_by_budget".to_string()),
                source_quality: None,
            });
    assert!(
        prioritized
            .continuity_next()
            .as_deref()
            .is_some_and(|next| next.contains("kind=decision"))
    );
    let mut preference_next = snapshot.clone();
    preference_next.preferences = vec![
            "id=old | kind=decision | tags=repo-hygiene | upd=1 | c=background".to_string(),
            "id=new | kind=decision | tags=next-agent,capability-sync | upd=2 | c=implement capability materializer".to_string(),
        ];
    preference_next.working.rehydration_queue = vec![memd_schema::MemoryRehydrationRecord {
        id: Some(uuid::Uuid::new_v4()),
        kind: "working_memory_record".to_string(),
        label: "fact".to_string(),
        summary: "id=stale | kind=fact | c=old docs plan".to_string(),
        recorded_at: None,
        source_agent: Some("codex".to_string()),
        source_system: Some("memd".to_string()),
        source_path: None,
        reason: Some("evicted_by_budget".to_string()),
        source_quality: None,
    }];
    assert!(
        preference_next
            .continuity_next()
            .as_deref()
            .is_some_and(|next| next.contains("capability materializer"))
    );

    let mut partial = preference_next.clone();
    partial.handoff_quality = Some(HandoffQualityScore {
        fill_rate: 0.5,
        budget_utilization: 0.9,
        dominant_kind: Some("fact".to_string()),
        eviction_pressure: 0.5,
        fact_coverage: 1.0,
        decision_coverage: 0.0,
        working_depth: 0.8,
        composite: 0.5,
    });
    assert_eq!(
        partial.continuity_next().as_deref(),
        Some(
            "id=new | kind=decision | tags=next-agent,capability-sync | upd=2 | c=implement capability materializer"
        )
    );

    let mut partial_without_next = ResumeSnapshot::empty();
    partial_without_next.handoff_quality = partial.handoff_quality;
    assert_eq!(
        partial_without_next.continuity_next().as_deref(),
        Some("fix partial handoff quality before claiming native recovery ready")
    );
}

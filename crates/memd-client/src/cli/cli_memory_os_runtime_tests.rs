use super::*;

use super::*;

#[test]
fn server_authority_status_probe_warms_transient_benchmark_gate() {
    let local_commit = local_git_commit_short().expect("local git commit");
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind status server");
    let address = listener.local_addr().expect("status server address");
    let handle = std::thread::spawn(move || {
        let mut status_requests = 0usize;
        for stream in listener.incoming().take(3) {
            let mut stream = stream.expect("accept status request");
            let mut reader =
                std::io::BufReader::new(stream.try_clone().expect("clone status request stream"));
            let mut request_line = String::new();
            std::io::BufRead::read_line(&mut reader, &mut request_line)
                .expect("read status request line");
            let path = request_line.split_whitespace().nth(1).unwrap_or("/");
            let body = if path == "/healthz" {
                "ok".to_string()
            } else {
                status_requests += 1;
                let gate = if status_requests == 1 { "fail" } else { "pass" };
                let latency = if status_requests == 1 { 2048.0 } else { 64.0 };
                serde_json::json!({
                    "git_commit": local_commit,
                    "git_dirty": "clean",
                    "benchmark_gate": gate,
                    "latency_p95_ms": latency,
                    "schema_version": 6,
                    "atlas": { "dormant": false }
                })
                .to_string()
            };
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            std::io::Write::write_all(&mut stream, response.as_bytes())
                .expect("write status response");
        }
    });

    let value = fetch_server_status_json_with_warmup(&format!("http://{address}"))
        .expect("fetch warmed status");

    handle.join().expect("status server thread");
    assert_eq!(
        value
            .get("benchmark_gate")
            .and_then(serde_json::Value::as_str),
        Some("pass")
    );
    assert_eq!(
        value
            .get("latency_p95_ms")
            .and_then(serde_json::Value::as_f64),
        Some(64.0)
    );
}

#[test]
fn token_savings_ledger_records_context_packet_savings() {
    let output = std::env::temp_dir().join(format!(
        "memd-token-savings-ledger-{}",
        uuid::Uuid::new_v4()
    ));
    let req = ContextRequest {
        project: Some("memd".to_string()),
        agent: Some("ollama".to_string()),
        workspace: None,
        visibility: None,
        route: None,
        intent: Some(memd_schema::RetrievalIntent::CurrentTask),
        limit: None,
        max_chars_per_item: None,
    };

    let entry = record_context_token_savings(&output, &req, Some("tiny"), 3, 4000, 1000)
        .expect("record token savings")
        .expect("entry present");
    let report = build_token_savings_report(&output, None);

    assert_eq!(entry.tokens_saved, 750);
    assert_eq!(report.ledger_events, 1);
    assert_eq!(report.measured_input_tokens, 1000);
    assert_eq!(report.measured_output_tokens, 250);
    assert_eq!(report.measured_tokens_saved, 750);
    assert!(token_savings_ledger_path(&output).is_file());

    fs::remove_dir_all(output).expect("cleanup token savings ledger temp");
}

#[test]
fn token_savings_ledger_records_source_read_attribution() {
    let output = std::env::temp_dir().join(format!(
        "memd-token-savings-source-read-{}",
        uuid::Uuid::new_v4()
    ));
    let state = output.join("state");
    fs::create_dir_all(&state).expect("create token savings state");
    fs::write(
        state.join("source-registry.json"),
        serde_json::json!({
            "project": "memd",
            "project_root": "/tmp/memd",
            "imported_at": Utc::now(),
            "sources": [{
                "path": "ROADMAP.md",
                "kind": "doc",
                "hash": "sha256-roadmap",
                "bytes": 56000,
                "lines": 620,
                "present": true,
                "imported_at": Utc::now(),
                "modified_at": Utc::now()
            }]
        })
        .to_string(),
    )
    .expect("write source registry");

    let entry = record_source_read_token_savings(
        &output,
        "ROADMAP.md",
        "source:ROADMAP.md#sha256-roadmap".len(),
        "context used durable source id instead of rereading file",
    )
    .expect("record source read savings")
    .expect("source read savings entry");
    let report = build_token_savings_report(&output, None);

    assert_eq!(entry.operation, "source_read_avoided");
    assert_eq!(entry.source_records, 1);
    assert!(entry.baseline_input_tokens > entry.output_tokens);
    assert!(entry.reason.contains("sha256-roadmap"));
    assert_eq!(report.ledger_events, 1);
    assert_eq!(report.measured_tokens_saved, entry.tokens_saved);
    assert_eq!(report.source_reuse_events, 1);
    assert_eq!(report.source_reuse_tokens, entry.tokens_saved);
    assert_eq!(report.source_records, 1);
    assert!(report.estimated_source_tokens >= entry.baseline_input_tokens);

    fs::remove_dir_all(output).expect("cleanup source read token savings temp");
}

#[test]
fn token_savings_ledger_records_wasted_token_telemetry() {
    let output = std::env::temp_dir().join(format!(
        "memd-token-waste-telemetry-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(output.join("state")).expect("create token waste state");

    record_wasted_token_event(
        &output,
        "raw_source_reread",
        4000,
        "unchanged raw source was reread instead of referenced by Source ID",
    )
    .expect("record raw source reread waste")
    .expect("raw source reread waste entry");
    record_wasted_token_event(
        &output,
        "giant_diff",
        8000,
        "giant diff entered context without a compact source handle",
    )
    .expect("record giant diff waste")
    .expect("giant diff waste entry");
    record_wasted_token_event(
        &output,
        "repo_cache_exposure",
        12000,
        "repo-visible cache path entered review/context surface",
    )
    .expect("record cache exposure waste")
    .expect("cache exposure waste entry");

    let report = build_token_savings_report(&output, None);
    let summary = render_tokens_summary(&report);

    assert_eq!(report.wasted_events, 3);
    assert_eq!(report.wasted_raw_reread_tokens, 1000);
    assert_eq!(report.wasted_giant_diff_tokens, 2000);
    assert_eq!(report.wasted_cache_exposure_tokens, 3000);
    assert_eq!(report.wasted_tokens, 6000);
    assert_eq!(report.measured_input_tokens, 0);
    assert_eq!(report.measured_tokens_saved, 0);
    assert_eq!(report.source_reuse_events, 0);
    assert_eq!(report.source_reuse_tokens, 0);
    assert!(summary.contains("wasted=6000"));
    assert!(summary.contains("wasted_events=3"));

    fs::remove_dir_all(output).expect("cleanup token waste telemetry temp");
}

#[test]
fn server_token_savings_report_overrides_measured_totals() {
    let output =
        std::env::temp_dir().join(format!("memd-token-savings-merge-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(output.join("state")).expect("create token temp");
    record_wasted_token_event(
        &output,
        "giant_diff",
        8000,
        "giant diff entered context before server counters existed",
    )
    .expect("record local waste before merge")
    .expect("local waste entry");
    let local = build_token_savings_report(&output, None);
    let merged = merge_server_token_savings_report(
        local,
        memd_schema::TokenSavingsListResponse {
            total: 2,
            measured_input_tokens: 1000,
            measured_output_tokens: 300,
            measured_tokens_saved: 700,
            source_reuse_events: 1,
            source_reuse_tokens: 250,
            wasted_events: 1,
            wasted_tokens: 2000,
            wasted_raw_reread_tokens: 0,
            wasted_giant_diff_tokens: 2000,
            wasted_cache_exposure_tokens: 0,
            records: Vec::new(),
        },
    );

    assert_eq!(merged.source, "server");
    assert_eq!(merged.ledger_events, 2);
    assert_eq!(merged.server_events, 2);
    assert_eq!(merged.measured_tokens_saved, 700);
    assert_eq!(merged.server_measured_tokens_saved, 700);
    assert_eq!(merged.source_reuse_events, 1);
    assert_eq!(merged.source_reuse_tokens, 250);
    assert_eq!(merged.wasted_events, 1);
    assert_eq!(merged.wasted_tokens, 2000);
    assert_eq!(merged.wasted_giant_diff_tokens, 2000);

    fs::remove_dir_all(output).expect("cleanup token merge temp");
}

#[test]
fn empty_server_token_savings_preserves_local_measured_ledger() {
    let output = std::env::temp_dir().join(format!(
        "memd-token-savings-server-empty-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(output.join("state")).expect("create token temp");
    let req = ContextRequest {
        project: Some("memd".to_string()),
        agent: Some("codex".to_string()),
        workspace: None,
        visibility: None,
        route: None,
        intent: Some(memd_schema::RetrievalIntent::CurrentTask),
        limit: None,
        max_chars_per_item: None,
    };

    record_context_token_savings(&output, &req, Some("tiny"), 3, 4000, 1000)
        .expect("record local token savings")
        .expect("local token savings entry");
    let local = build_token_savings_report(&output, None);
    let merged = merge_server_token_savings_report(
        local,
        memd_schema::TokenSavingsListResponse {
            total: 0,
            measured_input_tokens: 0,
            measured_output_tokens: 0,
            measured_tokens_saved: 0,
            source_reuse_events: 0,
            source_reuse_tokens: 0,
            wasted_events: 0,
            wasted_tokens: 0,
            wasted_raw_reread_tokens: 0,
            wasted_giant_diff_tokens: 0,
            wasted_cache_exposure_tokens: 0,
            records: Vec::new(),
        },
    );

    assert_eq!(merged.source, "local");
    assert_eq!(merged.server_events, 0);
    assert_eq!(merged.ledger_events, 1);
    assert_eq!(merged.measured_tokens_saved, 750);
    assert!(
        merged
            .notes
            .iter()
            .any(|note| note.contains("server token ledger was empty"))
    );

    fs::remove_dir_all(output).expect("cleanup empty server token temp");
}

#[test]
fn health_summary_surfaces_sync_queue_and_token_source() {
    let now = Utc::now();
    let report = MemoryOsHealthReport {
            generated_at: now,
            bundle_root: ".memd".to_string(),
            status: "partial".to_string(),
            features: MemoryOsFeatureReport {
                generated_at: now,
                bundle_root: ".memd".to_string(),
                status: "partial".to_string(),
                hygiene_status: "noisy".to_string(),
                token_risk: "medium".to_string(),
                market_claim: MarketClaimGate {
                    status: "blocked".to_string(),
                    evidence: Vec::new(),
                    blockers: vec![
                        "Supermemory same-fixture replay not pass: status=blocked report=supermemory.json missing_requirements=approved_supermemory_access_route_or_process_credential,supermemory_same_fixture_replay_artifact supermemory_request_path=.memd/state/supermemory-replay-request.json reason=missing approved Supermemory credential and replay artifacts".to_string(),
                        "full external public proof not pass: status=blocked report=full.json missing_explicit_env=ALLOW_FULL_PUBLIC_PROOF=1,PUBLIC_BENCH_LIMIT,PUBLIC_BENCH_TIMEOUT,RUN_LABEL reason=full external public proof is intentionally opt-in".to_string(),
                    ],
                },
                features: vec![
                    feature(
                        "server_authority",
                        "partial",
                        Vec::new(),
                        vec![
                            "server git_commit=d819af89 does not match local HEAD f0e2a715; shared authority deploy is stale".to_string(),
                            "server benchmark_gate=fail latency_p95_ms=2048; authority is not proven ready".to_string(),
                        ],
                    ),
                    feature(
                        "live_app_state_authority",
                        "partial",
                        Vec::new(),
                        vec![
                            "live app state source memd is auth_required missing=visible_page,calendar".to_string(),
                            "live app state auto-sync is missing; install with scripts/install-live-state-sync-launchd.sh --install".to_string(),
                            "live app state blocker detail: memd:status=auth_required missing=visible_page,calendar producer_route=\"scripts/live-state-sync-memd.sh\" external_source_note=\"memd-owned producers only; does not launch ClawControl\"".to_string(),
                        ],
                    ),
                ],
            },
            access: AccessReport {
                generated_at: now,
                bundle_root: ".memd".to_string(),
                status: "partial".to_string(),
                routes: Vec::new(),
                notes: Vec::new(),
            },
            sync_queue: OfflineQueueStatus {
                store: OfflineStoreQueueStatus {
                    path: PathBuf::from(".memd/state/offline-store-queue.jsonl"),
                    total: 1,
                    pending: 1,
                    synced: 0,
                    failed: 0,
                },
                sync: OfflineSyncQueueStatus {
                    path: PathBuf::from(".memd/state/offline-sync-queue.jsonl"),
                    total: 1,
                    pending: 0,
                    synced: 0,
                    failed: 1,
                    by_kind: std::collections::BTreeMap::from([(
                        "token_savings".to_string(),
                        OfflineSyncKindStatus {
                            total: 1,
                            pending: 0,
                            synced: 0,
                            failed: 1,
                        },
                    )]),
                },
            },
            token_savings: TokenSavingsReport {
                generated_at: now,
                bundle_root: ".memd".to_string(),
                source: "server".to_string(),
                since: None,
                ledger_path: ".memd/state/token-savings-ledger.ndjson".to_string(),
                ledger_events: 2,
                server_events: 2,
                server_measured_input_tokens: 1000,
                server_measured_output_tokens: 300,
                server_measured_tokens_saved: 700,
                measured_input_tokens: 1000,
                measured_output_tokens: 300,
                measured_tokens_saved: 700,
                source_reuse_events: 1,
                source_reuse_tokens: 250,
                wasted_events: 0,
                wasted_tokens: 0,
                wasted_raw_reread_tokens: 0,
                wasted_giant_diff_tokens: 0,
                wasted_cache_exposure_tokens: 0,
                source_records: 0,
                estimated_source_tokens: 0,
                wake_tokens: None,
                estimated_tokens_saved: 0,
                notes: Vec::new(),
            },
        };
    let summary = render_health_summary(&report);

    assert!(summary.contains("sync_pending=1"));
    assert!(summary.contains("sync_failed=1"));
    assert!(summary.contains("sync_kinds=token_savings:pending:0 failed:1"));
    assert!(summary.contains("hygiene=noisy"));
    assert!(summary.contains("token_risk=medium"));
    assert!(summary.contains("market_claim=blocked"));
    assert!(summary.contains("market_blockers=2"));
    assert!(summary.contains(
            "market_blocker_detail=supermemory:missing_requirements=approved_supermemory_access_route_or_process_credential,supermemory_same_fixture_replay_artifact request=.memd/state/supermemory-replay-request.json;full_public:missing_explicit_env=ALLOW_FULL_PUBLIC_PROOF=1,PUBLIC_BENCH_LIMIT,PUBLIC_BENCH_TIMEOUT,RUN_LABEL"
        ));
    assert!(summary.contains("server_authority_detail="));
    assert!(summary.contains("server git_commit=d819af89 does not match local HEAD f0e2a715"));
    assert!(summary.contains("server benchmark_gate=fail latency_p95_ms=2048"));
    assert!(summary.contains("live_state_blocker_detail="));
    assert!(summary.contains("memd:status=auth_required"));
    assert!(summary.contains("live app state auto-sync is missing"));
    assert!(summary.contains("memd-owned producers only; does not launch ClawControl"));
    assert!(summary.contains("token_source=server"));
    assert!(summary.contains("server_events=2"));
}

#[test]
fn sync_queue_evidence_reports_pending_and_failed_counts() {
    let evidence = sync_queue_evidence(Some(&OfflineQueueStatus {
        store: OfflineStoreQueueStatus {
            path: PathBuf::from(".memd/state/offline-store-queue.jsonl"),
            total: 2,
            pending: 1,
            synced: 0,
            failed: 1,
        },
        sync: OfflineSyncQueueStatus {
            path: PathBuf::from(".memd/state/offline-sync-queue.jsonl"),
            total: 3,
            pending: 2,
            synced: 1,
            failed: 0,
            by_kind: std::collections::BTreeMap::from([
                (
                    "capabilities".to_string(),
                    OfflineSyncKindStatus {
                        total: 1,
                        pending: 1,
                        synced: 0,
                        failed: 0,
                    },
                ),
                (
                    "access_routes".to_string(),
                    OfflineSyncKindStatus {
                        total: 2,
                        pending: 1,
                        synced: 1,
                        failed: 0,
                    },
                ),
            ]),
        },
    }));

    assert!(evidence[0].contains("store_pending:1"));
    assert!(evidence[0].contains("store_failed:1"));
    assert!(evidence[0].contains("sync_pending:2"));
    assert!(evidence[0].contains("sync_failed:0"));
    assert!(evidence[1].contains("capabilities:pending:1"));
    assert!(evidence[1].contains("access_routes:pending:1"));
}

#[test]
fn feature_registry_surfaces_server_authority_replay_proof_honestly() {
    let output =
        std::env::temp_dir().join(format!("memd-feature-server-core-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&output).expect("create feature temp");

    let report = build_feature_report(&output);
    let server = report
        .features
        .iter()
        .find(|feature| feature.id == "server_authority")
        .expect("server authority feature");

    assert_eq!(server.status, "working");
    assert!(
        server
            .evidence
            .iter()
            .any(|item| item.contains("offline sync replay reconciles capabilities"))
    );
    assert!(
        server
            .evidence
            .iter()
            .any(|item| item.contains("PC-A to PC-B reconciliation"))
    );
    assert!(server.gaps.is_empty());

    fs::remove_dir_all(output).expect("cleanup feature temp");
}

#[test]
fn server_authority_status_marks_unknown_or_failing_live_server_partial() {
    let probe = evaluate_server_authority_status(&serde_json::json!({
        "git_commit": "unknown",
        "git_dirty": "unknown",
        "benchmark_gate": "fail",
        "latency_p95_ms": 2048,
        "schema_version": 6,
        "atlas": {
            "dormant": true
        }
    }));

    assert!(
        probe
            .evidence
            .iter()
            .any(|item| item == "server_benchmark_gate=fail")
    );
    assert!(
        probe
            .gaps
            .iter()
            .any(|item| item.contains("git_commit is unknown"))
    );
    assert!(
        probe
            .gaps
            .iter()
            .any(|item| item.contains("benchmark_gate=fail latency_p95_ms=2048"))
    );
    assert!(
        probe
            .gaps
            .iter()
            .any(|item| item.contains("atlas is dormant"))
    );
}

#[test]
fn server_authority_status_marks_dirty_or_stale_live_server_partial() {
    let local_commit = local_git_commit_short().unwrap_or_else(|| "local".to_string());
    let stale_commit = if local_commit == "81f5c61" {
        "0000000"
    } else {
        "81f5c61"
    };
    let probe = evaluate_server_authority_status(&serde_json::json!({
        "git_commit": stale_commit,
        "git_dirty": "dirty",
        "benchmark_gate": "acceptable",
        "schema_version": 6,
        "atlas": {
            "dormant": false
        }
    }));

    assert!(
        probe
            .evidence
            .iter()
            .any(|item| item.starts_with("local_git_commit="))
    );
    assert!(
        probe
            .gaps
            .iter()
            .any(|item| item.contains("shared authority deploy is stale"))
    );
    assert!(
        probe
            .gaps
            .iter()
            .any(|item| item.contains("server_git_dirty=dirty"))
    );
}

#[test]
fn feature_registry_surfaces_mandatory_retrieval_lanes_honestly() {
    let output = std::env::temp_dir().join(format!(
        "memd-feature-retrieval-core-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&output).expect("create feature temp");

    let report = build_feature_report(&output);
    let retrieval = report
        .features
        .iter()
        .find(|feature| feature.id == "mandatory_retrieval_core")
        .expect("retrieval feature");

    assert_eq!(retrieval.status, "working");
    assert!(
        retrieval
            .evidence
            .iter()
            .any(|item| item.contains("split/camel/acronym paths"))
    );
    assert!(
        retrieval
            .evidence
            .iter()
            .any(|item| item.contains("atlas/entity recall"))
    );
    assert!(
        retrieval
            .evidence
            .iter()
            .any(|item| item.contains("source-linked provenance"))
    );
    assert!(
        retrieval
            .evidence
            .iter()
            .any(|item| item.contains("split/camel/acronym path and command token recall"))
    );
    assert!(
        retrieval
            .evidence
            .iter()
            .any(|item| item.contains("public no-RAG corpus route proof passes"))
    );
    assert!(
        retrieval
            .gaps
            .iter()
            .all(|item| !item.contains("temporal/provenance ranking"))
    );
    assert!(
        retrieval
            .evidence
            .iter()
            .any(|item| item.contains("temporal/provenance route proof passes"))
    );
    assert!(retrieval.gaps.is_empty());

    fs::remove_dir_all(output).expect("cleanup feature temp");
}

#[test]
fn feature_registry_surfaces_semantic_lane_no_rag_proof_honestly() {
    let output = std::env::temp_dir().join(format!(
        "memd-feature-semantic-core-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&output).expect("create feature temp");

    let report = build_feature_report(&output);
    let semantic = report
        .features
        .iter()
        .find(|feature| feature.id == "semantic_lane")
        .expect("semantic lane feature");

    assert_eq!(semantic.status, "working");
    assert!(
        semantic
            .evidence
            .iter()
            .any(|item| item.contains("MEMD_RAG_URL unset"))
    );
    assert!(
        semantic
            .evidence
            .iter()
            .any(|item| item.contains("embedding profile registry"))
    );
    assert!(semantic.gaps.is_empty());

    fs::remove_dir_all(output).expect("cleanup feature temp");
}

#[test]
fn feature_registry_surfaces_token_savings_source_read_proof_honestly() {
    let output =
        std::env::temp_dir().join(format!("memd-feature-token-core-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&output).expect("create feature temp");

    let report = build_feature_report(&output);
    let tokens = report
        .features
        .iter()
        .find(|feature| feature.id == "token_savings_engine")
        .expect("token savings feature");

    assert_eq!(tokens.status, "working");
    assert!(
        tokens
            .evidence
            .iter()
            .any(|item| item.contains("source-read attribution records saved tokens"))
    );
    assert!(
        tokens
            .evidence
            .iter()
            .any(|item| item.contains("source-ID reuse checks count"))
    );
    assert!(
        tokens
            .evidence
            .iter()
            .any(|item| item.contains("wasted-token telemetry"))
    );
    assert!(
        tokens
            .evidence
            .iter()
            .any(|item| item.contains("Token Budget prompt section"))
    );
    assert!(
        tokens
            .evidence
            .iter()
            .any(|item| item.contains("token savings payloads"))
    );
    assert!(tokens.gaps.is_empty());

    fs::remove_dir_all(output).expect("cleanup feature temp");
}

#[test]
fn feature_registry_surfaces_proof_gate_preflight_honestly() {
    let output =
        std::env::temp_dir().join(format!("memd-feature-proof-core-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&output).expect("create feature temp");

    let report = build_feature_report(&output);
    let proof = report
        .features
        .iter()
        .find(|feature| feature.id == "proof_gates")
        .expect("proof gates feature");

    assert_eq!(proof.status, "working");
    assert!(
        proof
            .evidence
            .iter()
            .any(|item| item.contains("implementation-readiness preflight"))
    );
    assert!(
        proof
            .evidence
            .iter()
            .any(|item| item.contains("run it after readiness"))
    );
    assert!(proof.gaps.is_empty());

    fs::remove_dir_all(output).expect("cleanup feature temp");
}

#[test]
fn feature_registry_marks_native_handoff_recovery_partial_until_ready() {
    let output =
        std::env::temp_dir().join(format!("memd-feature-handoff-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&output).expect("create feature temp");
    fs::write(
            output.join("wake.md"),
            "# wake\n\n- recovery voice=caveman-ultra | quality=partial:0.66 | dirty=12 | next=fix partial handoff quality | blocker=refresh recommended\n",
        )
        .expect("write wake");
    fs::write(output.join("mem.md"), "# mem\n").expect("write mem");

    let report = build_feature_report(&output);
    let handoff = report
        .features
        .iter()
        .find(|feature| feature.id == "native_handoff_recovery")
        .expect("native handoff feature");

    assert_eq!(handoff.status, "partial");
    assert!(
        handoff
            .evidence
            .iter()
            .any(|item| item == "handoff_quality=partial")
    );
    assert!(
        handoff
            .gaps
            .iter()
            .any(|item| item.contains("not proven ready"))
    );
    assert_eq!(report.status, "partial");

    fs::remove_dir_all(output).expect("cleanup feature temp");
}

#[test]
fn feature_registry_accepts_proof_blocker_recovery_capsule() {
    let output = std::env::temp_dir().join(format!(
        "memd-feature-handoff-proof-blockers-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&output).expect("create feature temp");
    fs::write(
            output.join("wake.md"),
            "# wake\n\n- recovery voice=caveman-ultra | quality=ready:0.99 | dirty=0 | next=abc: CURRENT NEXT ACTION: continue live-state authority | proof_blockers=full_public:missing_explicit_env=RUN_LABEL | live_state_blockers=memd:status=auth_required missing=messages,email\n",
        )
        .expect("write wake");
    fs::write(output.join("mem.md"), "# mem\n").expect("write mem");

    let report = build_feature_report(&output);
    let handoff = report
        .features
        .iter()
        .find(|feature| feature.id == "native_handoff_recovery")
        .expect("native handoff feature");

    assert_eq!(handoff.status, "working");
    assert!(
        handoff
            .evidence
            .iter()
            .any(|item| item == "native_continuity=true")
    );
    assert!(handoff.gaps.is_empty());

    fs::remove_dir_all(output).expect("cleanup feature temp");
}

#[test]
fn repo_hygiene_feature_marks_broad_dirty_tree_partial() {
    let repo = std::env::temp_dir().join(format!(
        "memd-feature-repo-hygiene-{}",
        uuid::Uuid::new_v4()
    ));
    let bundle = repo.join(".memd");
    fs::create_dir_all(&bundle).expect("create repo hygiene temp");
    assert!(
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&repo)
            .output()
            .expect("git init")
            .status
            .success()
    );
    for (key, value) in [
        ("user.email", "memd@example.invalid"),
        ("user.name", "memd"),
    ] {
        assert!(
            std::process::Command::new("git")
                .args(["config", key, value])
                .current_dir(&repo)
                .output()
                .expect("git config")
                .status
                .success()
        );
    }
    for index in 0..6 {
        fs::write(repo.join(format!("file-{index}.txt")), "before\n").expect("write tracked");
    }
    assert!(
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&repo)
            .output()
            .expect("git add")
            .status
            .success()
    );
    assert!(
        std::process::Command::new("git")
            .args(["commit", "-m", "seed"])
            .current_dir(&repo)
            .output()
            .expect("git commit")
            .status
            .success()
    );
    for index in 0..6 {
        fs::write(repo.join(format!("file-{index}.txt")), "after\n").expect("dirty tracked");
    }

    let feature = repo_hygiene_feature(&bundle);

    assert_eq!(feature.status, "partial");
    assert_eq!(feature.hygiene_status, "noisy");
    assert_eq!(feature.token_risk, "medium");
    assert!(
        feature
            .evidence
            .iter()
            .any(|item| item == "dirty_tracked_files=6")
    );
    assert!(
        feature
            .gaps
            .iter()
            .any(|gap| gap.contains("broad dirty tree"))
    );

    fs::remove_dir_all(repo).expect("cleanup repo hygiene temp");
}

#[test]
fn repo_hygiene_audits_promptwall_cache_path() {
    let repo = std::env::temp_dir().join(format!(
        "memd-feature-promptwall-cache-{}",
        uuid::Uuid::new_v4()
    ));
    let cache_path = repo
        .join("docs")
        .join("verification")
        .join("25-5-memory-os-runs")
        .join("promptwall-cache");

    assert!(
        raw_benchmark_cache_paths(&repo)
            .iter()
            .any(|path| path == &cache_path)
    );
}

#[test]
fn capability_inventory_audit_marks_missing_real_inventory_partial() {
    let Some(home) = home_dir() else {
        return;
    };
    let has_codex_inventory = home.join(".codex").join("skills").is_dir()
        || home.join(".codex").join("plugins").join("cache").is_dir();
    if !has_codex_inventory {
        return;
    }

    let registry = CapabilityRegistry {
        generated_at: Utc::now(),
        project_root: None,
        capabilities: Vec::new(),
    };
    let output =
        std::env::temp_dir().join(format!("memd-capability-feature-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&output).expect("create capability feature temp");
    let feature = capability_sync_feature(&output, &registry, None);

    assert_eq!(feature.status, "broken");
    assert!(
        feature
            .gaps
            .iter()
            .any(|gap| gap.contains("capability record"))
    );
    fs::remove_dir_all(output).expect("cleanup capability feature temp");
}

#[test]
fn capability_sync_stays_partial_until_fresh_machine_materializer_exists() {
    let output = std::env::temp_dir().join(format!(
        "memd-capability-materializer-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&output).expect("create capability feature temp");
    let mut capabilities = Vec::new();
    for harness in [
        "agent-zero",
        "claude-code",
        "codex",
        "hermes",
        "openclaw",
        "opencode",
    ] {
        capabilities.push(CapabilityRecord {
            harness: harness.to_string(),
            kind: "harness-pack".to_string(),
            name: harness.to_string(),
            status: "wired".to_string(),
            portability_class: "universal".to_string(),
            source_path: format!(".memd/agents/{harness}.sh"),
            bridge_hint: None,
            hash: None,
            notes: if harness == "hermes" {
                vec![
                    r##"memd:payload-file-json:{"path":"SKILL.md","content":"# Hermes\n"}"##
                        .to_string(),
                ]
            } else {
                Vec::new()
            },
        });
    }
    capabilities.push(CapabilityRecord {
        harness: "codex".to_string(),
        kind: "plugin".to_string(),
        name: "browser-use".to_string(),
        status: "available-server".to_string(),
        portability_class: "harness-native".to_string(),
        source_path: "/remote/.codex/plugins/cache/browser-use/.codex-plugin/plugin.json"
            .to_string(),
        bridge_hint: Some("server inventory only".to_string()),
        hash: None,
        notes: vec!["synced_from_server".to_string()],
    });
    let registry = CapabilityRegistry {
        generated_at: Utc::now(),
        project_root: None,
        capabilities,
    };

    let feature = capability_sync_feature(&output, &registry, None);

    assert_eq!(feature.status, "partial");
    assert!(feature.evidence.iter().any(|item| {
        item == "server-synced text payloads can be materialized for small harness assets"
    }));
    assert!(feature.evidence.iter().any(|item| {
        item == "server-synced payload sets can restore bounded skill/plugin text files"
    }));
    assert!(
        feature
            .evidence
            .iter()
            .any(|item| item == "fresh_machine_materializer=missing-payloads")
    );
    assert!(
        feature
            .evidence
            .iter()
            .any(|item| item == "payload_file_set_records=1")
    );
    assert!(
        feature
            .evidence
            .iter()
            .any(|item| item == "materialization_installable=6")
    );
    assert!(
        feature
            .evidence
            .iter()
            .any(|item| item == "materialization_missing=1")
    );
    assert!(
        feature
            .gaps
            .iter()
            .any(|gap| gap.contains("fresh-machine materialization missing for codex:plugin"))
    );
    assert!(
        feature
            .gaps
            .iter()
            .any(|gap| gap.contains("lack fresh-machine materialization payloads"))
    );

    fs::remove_dir_all(output).expect("cleanup capability feature temp");
}

#[test]
fn capability_sync_counts_host_cli_install_plans_as_honest_blockers() {
    let output = std::env::temp_dir().join(format!(
        "memd-capability-host-cli-plan-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&output).expect("create capability feature temp");
    let registry = CapabilityRegistry {
        generated_at: Utc::now(),
        project_root: None,
        capabilities: vec![CapabilityRecord {
            harness: "local".to_string(),
            kind: "cli".to_string(),
            name: "gh".to_string(),
            status: "available-server".to_string(),
            portability_class: "host-local".to_string(),
            source_path: "/source/bin/gh".to_string(),
            bridge_hint: Some("server inventory only".to_string()),
            hash: None,
            notes: vec![
                "PATH inventory; executable availability is host-local".to_string(),
                "memd:host-cli-install-plan:#!/bin/sh\necho install gh\n".to_string(),
            ],
        }],
    };

    let feature = capability_sync_feature(&output, &registry, None);

    assert_eq!(feature.status, "partial");
    assert!(
        feature
            .evidence
            .iter()
            .any(|item| item == "fresh_machine_materializer=partial-host-local")
    );
    assert!(
        feature
            .evidence
            .iter()
            .any(|item| item == "host_cli_install_plans=1")
    );
    assert!(
        feature
            .evidence
            .iter()
            .any(|item| item == "host_cli_auth_proofs=0")
    );
    assert!(
        feature
            .evidence
            .iter()
            .any(|item| item == "expected_host_cli_records=1/6")
    );
    assert!(
        feature
            .evidence
            .iter()
            .any(|item| item == "materialization_missing=0")
    );
    assert!(
        feature
            .gaps
            .iter()
            .any(|gap| gap == "1 host CLI records lack server-synced auth proof notes")
    );
    assert!(feature.gaps.iter().any(|gap| {
            gap == "missing expected host CLI capability records: codex,opencode,claude,wrangler,supabase"
        }));
    assert!(
        !feature
            .gaps
            .iter()
            .any(|gap| gap.contains("lack server-synced install plans"))
    );

    fs::remove_dir_all(output).expect("cleanup capability feature temp");
}

#[test]
fn capability_sync_counts_host_cli_auth_proof_notes_as_prompt_guidance() {
    let output = std::env::temp_dir().join(format!(
        "memd-capability-host-cli-auth-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&output).expect("create capability feature temp");
    let registry = CapabilityRegistry {
        generated_at: Utc::now(),
        project_root: None,
        capabilities: vec![CapabilityRecord {
            harness: "local".to_string(),
            kind: "cli".to_string(),
            name: "gh".to_string(),
            status: "available-server".to_string(),
            portability_class: "host-local".to_string(),
            source_path: "/source/bin/gh".to_string(),
            bridge_hint: Some("server inventory only".to_string()),
            hash: None,
            notes: vec![
                "PATH inventory; executable availability is host-local".to_string(),
                "memd:host-cli-install-plan:#!/bin/sh\necho install gh\n".to_string(),
                "memd:host-auth-status:unauthenticated".to_string(),
                "memd:host-auth-check:gh auth status".to_string(),
                "memd:host-auth-proof:local-probe".to_string(),
                "memd:host-auth-output-stored:false".to_string(),
            ],
        }],
    };

    let feature = capability_sync_feature(&output, &registry, None);

    assert_eq!(feature.status, "partial");
    assert!(
        feature
            .evidence
            .iter()
            .any(|item| item == "fresh_machine_materializer=partial-host-local")
    );
    assert!(
        feature
            .evidence
            .iter()
            .any(|item| item == "host_cli_auth_proofs=1")
    );
    assert!(
        feature
            .evidence
            .iter()
            .any(|item| item == "host_cli_auth_unauthenticated=1")
    );
    assert!(feature.evidence.iter().any(|item| {
        item == "host_cli_auth_gaps_surface_as_prompt_guidance=unknown:0 unauthenticated:1"
    }));
    assert!(
        !feature
            .gaps
            .iter()
            .any(|gap| gap.contains("lack server-synced auth proof notes"))
    );
    assert!(
        !feature
            .gaps
            .iter()
            .any(|gap| gap.contains("host CLI auth checks are unauthenticated"))
    );

    fs::remove_dir_all(output).expect("cleanup capability feature temp");
}

#[test]
fn capability_sync_counts_probe_skipped_host_cli_auth_notes_as_guidance() {
    let output = std::env::temp_dir().join(format!(
        "memd-capability-host-cli-auth-skipped-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&output).expect("create capability feature temp");
    let registry = CapabilityRegistry {
        generated_at: Utc::now(),
        project_root: None,
        capabilities: vec![CapabilityRecord {
            harness: "local".to_string(),
            kind: "cli".to_string(),
            name: "gh".to_string(),
            status: "available-server".to_string(),
            portability_class: "host-local".to_string(),
            source_path: "/source/bin/gh".to_string(),
            bridge_hint: Some("server inventory only".to_string()),
            hash: None,
            notes: vec![
                "PATH inventory; executable availability is host-local".to_string(),
                "memd:host-cli-install-plan:#!/bin/sh\necho install gh\n".to_string(),
                "memd:host-auth-status:unknown".to_string(),
                "memd:host-auth-check:gh auth status".to_string(),
                "memd:host-auth-proof:probe-skipped".to_string(),
                "memd:host-auth-output-stored:false".to_string(),
            ],
        }],
    };

    let feature = capability_sync_feature(&output, &registry, None);

    assert!(
        feature
            .evidence
            .iter()
            .any(|item| item == "host_cli_auth_proofs=1")
    );
    assert!(
        feature
            .evidence
            .iter()
            .any(|item| item == "host_cli_auth_unknown=1")
    );
    assert!(
        feature.evidence.iter().any(|item| {
            item == "host_cli_auth_gaps_surface_as_prompt_guidance=unknown:1 unauthenticated:0"
        }),
        "{:?}",
        feature.evidence
    );
    assert!(
        !feature
            .gaps
            .iter()
            .any(|gap| gap.contains("lack server-synced auth proof notes")),
        "{:?}",
        feature.gaps
    );

    fs::remove_dir_all(output).expect("cleanup capability feature temp");
}

#[test]
fn feature_registry_audits_live_host_cli_auth_notes() {
    let output = std::env::temp_dir().join(format!(
        "memd-feature-host-cli-auth-live-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&output).expect("create capability feature temp");

    let report = build_feature_report(&output);
    let feature = report
        .features
        .iter()
        .find(|feature| feature.id == "capability_sync")
        .expect("capability sync feature");

    assert!(
        feature
            .evidence
            .iter()
            .any(|item| item == "fresh_machine_materializer=ready-with-host-cli-auth-guidance")
    );
    assert!(
        feature
            .evidence
            .iter()
            .any(|item| item == "host_cli_auth_proofs=6")
    );
    assert!(
        !feature
            .gaps
            .iter()
            .any(|gap| gap.contains("lack server-synced auth proof notes"))
    );

    fs::remove_dir_all(output).expect("cleanup capability feature temp");
}

#[test]
fn feature_registry_separates_implementation_ready_from_market_claim() {
    let output =
        std::env::temp_dir().join(format!("memd-feature-claim-core-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&output).expect("create feature temp");

    let report = build_feature_report(&output);
    let summary = render_feature_summary(&report);
    let repo_hygiene = report
        .features
        .iter()
        .find(|feature| feature.id == "repo_hygiene")
        .expect("repo hygiene feature");

    assert_ne!(report.status, report.market_claim.status);
    assert_eq!(report.market_claim.status, "blocked");
    assert_eq!(repo_hygiene.implementation_status, repo_hygiene.status);
    assert_eq!(repo_hygiene.market_status, "blocked");
    assert_eq!(repo_hygiene.hygiene_status, "clean");
    assert_eq!(repo_hygiene.token_risk, "low");
    assert_eq!(report.hygiene_status, "clean");
    assert!(
        report
            .market_claim
            .blockers
            .iter()
            .any(|item| item.contains("Supermemory"))
    );
    assert!(
        report
            .market_claim
            .blockers
            .iter()
            .any(|item| item.contains("full external public proof"))
    );
    assert!(summary.contains("market_claim=blocked"));
    assert!(summary.contains("hygiene=clean"));
    assert!(summary.contains("token_risk="));
    assert!(summary.contains("blockers="));

    fs::remove_dir_all(output).expect("cleanup feature temp");
}

#[test]
fn report_blocker_detail_surfaces_missing_requirements_without_secret_values() {
    let dir = std::env::temp_dir().join(format!(
        "memd-report-blocker-detail-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("create report detail temp");
    let report = dir.join("blocked.json");
    fs::write(
        &report,
        serde_json::json!({
            "status": "blocked",
            "missing_requirements": [
                "approved_supermemory_access_route_or_process_credential",
                "supermemory_same_fixture_replay_artifact"
            ],
            "missing_explicit_env": ["ALLOW_FULL_PUBLIC_PROOF=1"],
            "reason": "blocked proof gate",
            "supermemory_request_path": ".memd/state/supermemory-replay-request.json",
            "credential_env_present": false
        })
        .to_string(),
    )
    .expect("write report detail fixture");

    let detail = report_blocker_detail(&report).expect("report blocker detail");
    assert!(detail.contains(
            "missing_requirements=approved_supermemory_access_route_or_process_credential,supermemory_same_fixture_replay_artifact"
        ));
    assert!(detail.contains("missing_explicit_env=ALLOW_FULL_PUBLIC_PROOF=1"));
    assert!(
        detail.contains("supermemory_request_path=.memd/state/supermemory-replay-request.json")
    );
    assert!(detail.contains("reason=blocked proof gate"));
    assert!(!detail.contains("SUPERMEMORY_API_KEY"));

    fs::remove_dir_all(dir).expect("cleanup report detail temp");
}

#[test]
fn feature_registry_surfaces_prompt_firewall_trace_evidence_honestly() {
    let output = std::env::temp_dir().join(format!(
        "memd-feature-firewall-core-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&output).expect("create feature temp");

    let report = build_feature_report(&output);
    let firewall = report
        .features
        .iter()
        .find(|feature| feature.id == "prompt_firewall")
        .expect("prompt firewall feature");

    assert_eq!(firewall.status, "working");
    assert!(
        firewall
            .evidence
            .iter()
            .any(|item| item.contains("Firewall Trace labels"))
    );
    assert!(
        firewall
            .evidence
            .iter()
            .any(|item| item.contains("poisoned-memory context packet route proof passes"))
    );
    assert!(firewall.gaps.is_empty());

    fs::remove_dir_all(output).expect("cleanup feature temp");
}

#[test]
fn feature_registry_surfaces_capability_and_access_route_proof_honestly() {
    let output =
        std::env::temp_dir().join(format!("memd-feature-sync-core-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&output).expect("create feature temp");

    let report = build_feature_report(&output);
    let capability = report
        .features
        .iter()
        .find(|feature| feature.id == "capability_sync")
        .expect("capability feature");
    assert!(
        capability
            .evidence
            .iter()
            .any(|item| item.contains("server capability sync/list routes round-trip"))
    );
    assert!(
        capability
            .evidence
            .iter()
            .any(|item| item.contains("PC-A syncing skills/plugins"))
    );
    assert_eq!(capability.status, "partial");
    assert!(
        capability
            .evidence
            .iter()
            .any(|item| item.contains("host_cli_auth_gaps_surface_as_prompt_guidance"))
    );
    assert!(
        capability
            .gaps
            .iter()
            .any(|gap| gap.contains("missing codex harness-pack capability record"))
    );

    let access = report
        .features
        .iter()
        .find(|feature| feature.id == "access_secret_routes")
        .expect("access feature");
    assert!(
        access
            .evidence
            .iter()
            .any(|item| item.contains("server access route sync/list routes round-trip"))
    );
    assert!(
        access
            .evidence
            .iter()
            .any(|item| item.contains("rejects access routes"))
    );
    assert!(
        access
            .evidence
            .iter()
            .any(|item| item.contains("agent-secrets is integrated"))
    );
    assert!(
        access
            .evidence
            .iter()
            .any(|item| item.contains("provider and purpose filters"))
    );
    assert_eq!(access.status, "working");
    assert!(access.gaps.is_empty());

    fs::remove_dir_all(output).expect("cleanup feature temp");
}

#[test]
fn feature_registry_surfaces_live_state_source_health() {
    let output = std::env::temp_dir().join(format!(
        "memd-feature-live-state-health-{}",
        uuid::Uuid::new_v4()
    ));
    let source_status_path = live_app_source_status_path(&output);
    fs::create_dir_all(source_status_path.parent().expect("state dir")).expect("create state dir");
    fs::write(
            &source_status_path,
            r#"{
  "version": 1,
  "updated_at": "2026-05-17T06:00:00Z",
  "sources": [
    {
      "source_app": "memd",
      "status": "auth_required",
      "checked_at": "2026-05-17T06:00:00Z",
      "api_base": "http://127.0.0.1:3000",
      "api_bases": ["http://127.0.0.1:3010", "http://127.0.0.1:3000"],
      "auth_configured": false,
      "visible_page": "missing",
      "produced": [],
      "missing": ["visible_page", "calendar", "todos", "reminders", "messages", "email"],
      "record_count": 0,
      "endpoints": [
        {"module": "calendar", "path": "/api/calendar", "ok": false, "status": 401, "error": "HTTP 401"}
      ],
      "last_error": "missing live-state surfaces: visible_page, calendar, todos, reminders, messages, email; provide CLAWCONTROL_API_KEY or MC_API_KEY for X-API-Key auth"
    }
  ]
}"#,
        )
        .expect("write source status");

    let report = build_feature_report(&output);
    let live_state = report
        .features
        .iter()
        .find(|feature| feature.id == "live_app_state_authority")
        .expect("live state feature");

    assert_eq!(live_state.status, "partial");
    assert!(
        live_state
            .evidence
            .iter()
            .any(|item| item.contains("live_app_source_status"))
    );
    assert!(
        live_state
            .evidence
            .iter()
            .any(|item| item.contains("source_unavailable=1"))
    );
    assert!(
        live_state
            .evidence
            .iter()
            .any(|item| item.contains("source_stale=1"))
    );
    assert!(
        live_state
            .evidence
            .iter()
            .any(|item| item.contains("live_state_source_status=memd"))
    );
    assert!(
        live_state
            .evidence
            .iter()
            .any(|item| item.contains("auth_configured=false"))
    );
    assert!(
        live_state
            .evidence
            .iter()
            .any(|item| item.contains("api_bases=http://127.0.0.1:3010,http://127.0.0.1:3000"))
    );
    assert!(
        live_state
            .evidence
            .iter()
            .any(|item| item.contains("live_state_auto_sync status="))
    );
    assert!(
        live_state
            .gaps
            .iter()
            .any(|gap| gap.contains("1 live app source status checks are stale"))
    );
    assert!(
        live_state
            .gaps
            .iter()
            .any(|gap| gap.contains("source memd is auth_required"))
    );
    assert!(
        live_state
            .gaps
            .iter()
            .any(|gap| { gap.contains("memd-owned producers only; does not launch ClawControl") })
    );

    fs::remove_dir_all(output).expect("cleanup feature temp");
}

#[test]
fn feature_registry_live_state_source_gaps_use_state_map_unmet_modules() {
    let output = std::env::temp_dir().join(format!(
        "memd-feature-live-state-state-map-{}",
        uuid::Uuid::new_v4()
    ));
    let state_path = live_app_state_path(&output);
    let source_status_path = live_app_source_status_path(&output);
    fs::create_dir_all(state_path.parent().expect("state dir")).expect("create state dir");

    let now = Utc::now();
    let expires = now + chrono::Duration::hours(1);
    let stale_source_checked_at = now - chrono::Duration::hours(1);
    let records = vec![
        serde_json::json!({
            "id": "memd:visible_page:current",
            "source_app": "memd",
            "module": "visible_page",
            "scope": "current",
            "visibility": "private",
            "privacy": "metadata",
            "approved": true,
            "agentsecrets_approved": false,
            "labels": ["live-app-state", "visible_page", "metadata"],
            "summary": "visible page fallback fresh",
            "payload": {"producer": "mac-bridge"},
            "payload_hash": "visible-page-hash",
            "captured_at": now,
            "updated_at": now,
            "expires_at": expires
        }),
        serde_json::json!({
            "id": "memd:calendar:primary",
            "source_app": "memd",
            "module": "calendar",
            "scope": "primary",
            "visibility": "private",
            "privacy": "metadata",
            "approved": true,
            "agentsecrets_approved": false,
            "labels": ["live-app-state", "calendar", "metadata"],
            "summary": "calendar fallback fresh",
            "payload": {"producer": "mac-bridge", "events": []},
            "payload_hash": "calendar-hash",
            "captured_at": now,
            "updated_at": now,
            "expires_at": expires
        }),
        serde_json::json!({
            "id": "clawcontrol:reminders:default",
            "source_app": "memd",
            "module": "reminders",
            "scope": "default",
            "visibility": "private",
            "privacy": "metadata",
            "approved": true,
            "agentsecrets_approved": false,
            "labels": ["live-app-state", "reminders", "metadata"],
            "summary": "reminders fallback fresh",
            "payload": {"producer": "mac-bridge", "reminders": []},
            "payload_hash": "reminders-hash",
            "captured_at": now,
            "updated_at": now,
            "expires_at": expires
        }),
        serde_json::json!({
            "id": "clawcontrol:todos:default",
            "source_app": "memd",
            "module": "todos",
            "scope": "default",
            "visibility": "private",
            "privacy": "metadata",
            "approved": true,
            "agentsecrets_approved": false,
            "labels": ["live-app-state", "todos", "metadata"],
            "summary": "todos fallback fresh",
            "payload": {"producer": "mac-bridge", "todos": []},
            "payload_hash": "todos-hash",
            "captured_at": now,
            "updated_at": now,
            "expires_at": expires
        }),
    ];
    fs::write(
        &state_path,
        serde_json::json!({
            "version": 1,
            "updated_at": now,
            "records": records
        })
        .to_string(),
    )
    .expect("write live state");
    fs::write(
        &source_status_path,
        serde_json::json!({
            "version": 1,
            "updated_at": now,
            "sources": [
                {
                    "source_app": "memd",
                    "status": "auth_required",
                    "checked_at": stale_source_checked_at,
                    "api_base": "http://127.0.0.1:3010",
                    "api_bases": ["http://127.0.0.1:3010"],
                    "auth_configured": false,
                    "visible_page": "missing",
                    "produced": [],
                    "missing": ["messages", "email"],
                    "record_count": 0,
                    "endpoints": [],
                    "last_error": "missing live-state surfaces"
                },
                {
                    "source_app": "approved_communications",
                    "status": "missing_approval",
                    "checked_at": now,
                    "api_base": "approved-communications",
                    "api_bases": ["approved-communications"],
                    "auth_configured": false,
                    "visible_page": "not_applicable",
                    "produced": [],
                    "missing": ["messages", "email"],
                    "record_count": 0,
                    "endpoints": [],
                    "last_error": "no approved communications file configured"
                }
            ]
        })
        .to_string(),
    )
    .expect("write source status");

    let report = build_feature_report(&output);
    let live_state = report
        .features
        .iter()
        .find(|feature| feature.id == "live_app_state_authority")
        .expect("live state feature");
    let gaps = live_state.gaps.join(";");

    assert!(!gaps.contains("source memd is auth_required"), "{gaps}");
    assert!(
        !gaps.contains("live app source status checks are stale"),
        "{gaps}"
    );
    assert!(
        gaps.contains("source approved_communications is missing_approval missing=messages,email")
    );
    assert!(
        !gaps.contains("auth_required missing=visible_page,calendar"),
        "{gaps}"
    );
    let evidence = live_state.evidence.join(";");
    assert!(
        evidence.contains("state_map_fresh=visible_page,calendar,reminders,todos"),
        "{evidence}"
    );
    assert!(
        evidence.contains("state_map_unmet=messages,email"),
        "{evidence}"
    );
    assert!(evidence.contains("source_stale=0"), "{evidence}");
    assert!(!gaps.contains("memd:status=auth_required"), "{gaps}");
    assert!(
        gaps.contains("approved_communications:status=missing_approval missing=messages,email")
    );

    fs::remove_dir_all(output).expect("cleanup feature temp");
}

#[test]
fn live_state_auto_sync_status_detects_launchd_plist_shape() {
    let root = std::env::temp_dir().join(format!(
        "memd-live-state-auto-sync-{}",
        uuid::Uuid::new_v4()
    ));
    let home = root.join("home");
    let project = root.join("project");
    let launch_agents = home.join("Library").join("LaunchAgents");
    let plist = launch_agents.join("com.memd.live-state-sync.plist");
    let script = project.join("scripts").join("live-state-sync-memd.sh");
    fs::create_dir_all(script.parent().expect("script dir")).expect("create script dir");
    fs::create_dir_all(&launch_agents).expect("create launch agents");

    let missing = live_state_auto_sync_status_for(
        &home,
        &project,
        "macos",
        "com.memd.live-state-sync-missing",
        300,
    );
    assert_eq!(missing.status, "missing");
    assert_eq!(
        missing.install_command,
        "scripts/install-live-state-sync-launchd.sh --install"
    );

    fs::write(&plist, "<plist><string>/tmp/wrong.sh</string></plist>").expect("write wrong plist");
    let misconfigured =
        live_state_auto_sync_status_for(&home, &project, "macos", "com.memd.live-state-sync", 300);
    assert_eq!(misconfigured.status, "misconfigured");

    fs::write(
        &plist,
        format!("<plist><string>{}</string></plist>", script.display()),
    )
    .expect("write installed plist");
    let installed =
        live_state_auto_sync_status_for(&home, &project, "macos", "com.memd.live-state-sync", 300);
    assert_eq!(installed.status, "installed");

    let unsupported =
        live_state_auto_sync_status_for(&home, &project, "linux", "com.memd.live-state-sync", 300);
    assert_eq!(unsupported.status, "unsupported");

    fs::remove_dir_all(root).expect("cleanup live state auto-sync temp");
}

#[test]
fn access_route_filters_provider_and_surfaces_guidance_without_secret_values() {
    let output = std::env::temp_dir().join(format!(
        "memd-access-route-guidance-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&output).expect("create access temp");
    let args = AccessArgs {
        command: AccessSubcommand::Route(AccessRouteArgs {
            output: output.clone(),
            resource: None,
            purpose: Some("supermemory-api-key".to_string()),
            provider: Some("bitwarden".to_string()),
            agent: Some("codex".to_string()),
            json: false,
        }),
    };

    let report = run_access_command(&args).expect("access route report");
    assert_eq!(report.routes.len(), 1);
    assert_eq!(report.status, "working");
    let route = &report.routes[0];
    assert_eq!(route.provider, "bitwarden");
    assert_eq!(route.scope, "supermemory-api-key");
    assert!(!route.secret_values_stored);
    assert!(
        route.guidance.contains("ask user")
            || route
                .guidance
                .contains("never print or store secret values")
            || route.guidance.contains("not found")
    );

    let summary = render_access_summary(&report);
    assert!(summary.contains("bitwarden:"));
    assert!(summary.contains("["));
    assert!(summary.contains("bundle="));

    fs::remove_dir_all(output).expect("cleanup access temp");
}

#[test]
fn access_route_surfaces_clawcontrol_process_env_without_secret_values() {
    let output = std::env::temp_dir().join(format!(
        "memd-access-route-clawcontrol-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&output).expect("create access temp");
    let args = AccessArgs {
        command: AccessSubcommand::Route(AccessRouteArgs {
            output: output.clone(),
            resource: None,
            purpose: Some("clawcontrol-api-key".to_string()),
            provider: Some("process-env".to_string()),
            agent: Some("codex".to_string()),
            json: false,
        }),
    };

    let report = run_access_command(&args).expect("access route report");
    assert_eq!(report.routes.len(), 1);
    assert!(matches!(report.status.as_str(), "working" | "partial"));
    let route = &report.routes[0];
    assert_eq!(route.id, "process-env:clawcontrol-api-key");
    assert_eq!(route.provider, "process-env");
    assert_eq!(route.scope, "clawcontrol-api-key");
    assert!(matches!(
        route.status.as_str(),
        "available" | "needs_approval"
    ));
    assert!(!route.secret_values_stored);
    assert!(route.guidance.contains("process-local route"));
    assert!(route.guidance.contains("never store") || route.guidance.contains("metadata only"));

    let summary = render_access_summary(&report);
    assert!(summary.contains("process-env:"));
    assert!(summary.contains("bundle="));

    fs::remove_dir_all(output).expect("cleanup access temp");
}

#[test]
fn access_route_surfaces_approved_communications_files_without_values() {
    let output = std::env::temp_dir().join(format!(
        "memd-access-route-approved-communications-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&output).expect("create access temp");
    let args = AccessArgs {
        command: AccessSubcommand::Route(AccessRouteArgs {
            output: output.clone(),
            resource: None,
            purpose: Some("approved-communications-file".to_string()),
            provider: Some("process-env".to_string()),
            agent: Some("codex".to_string()),
            json: false,
        }),
    };

    let report = run_access_command(&args).expect("access route report");
    assert_eq!(report.routes.len(), 1);
    let route = &report.routes[0];
    assert_eq!(route.id, "process-env:approved-communications-file");
    assert_eq!(route.provider, "process-env");
    assert_eq!(route.scope, "approved-communications-file");
    assert!(!route.secret_values_stored);
    assert!(route.guidance.contains("APPROVED_COMMUNICATIONS_FILE"));
    assert!(route.guidance.contains("APPROVED_MESSAGES_FILE"));
    assert!(route.guidance.contains("APPROVED_EMAIL_FILE"));
    assert!(
        route
            .guidance
            .contains("APPROVED_COMMUNICATIONS_EMPTY_APPROVED")
    );
    assert!(route.guidance.contains("approved communications JSON"));
    assert!(route.guidance.contains("explicit empty arrays"));
    assert!(route.guidance.contains("approved=true"));
    assert!(route.guidance.contains("redactedSnippet"));
    assert!(route.guidance.contains("agentsecretsApproved=true"));
    assert!(
        route
            .guidance
            .contains("raw chat/mail body text and raw media are rejected")
    );
    assert!(route.guidance.contains("metadata only"));

    let summary = render_access_summary(&report);
    assert!(summary.contains("process-env:"));
    assert!(summary.contains("approved communications JSON"));
    assert!(summary.contains("bundle="));

    fs::remove_dir_all(output).expect("cleanup access temp");
}

#[test]
fn access_report_marks_refs_only_guided_routes_working() {
    let output =
        std::env::temp_dir().join(format!("memd-access-route-status-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&output).expect("create access temp");

    let report = build_access_report(&output, None, Some("codex"));

    assert_eq!(report.status, "working");
    assert!(
        report
            .routes
            .iter()
            .all(|route| !route.secret_values_stored)
    );
    assert!(report.routes.iter().any(|route| matches!(
        route.status.as_str(),
        "unlocked" | "available" | "installed"
    )));
    assert!(
        report
            .notes
            .iter()
            .any(|note| note.contains("refs-only routes"))
    );

    fs::remove_dir_all(output).expect("cleanup access temp");
}

#[test]
fn access_sync_includes_required_process_env_authority_routes() {
    let output =
        std::env::temp_dir().join(format!("memd-access-sync-routes-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&output).expect("create access temp");
    let args = AccessArgs {
        command: AccessSubcommand::Sync(AccessSyncArgs {
            output: output.clone(),
            json: false,
        }),
    };

    let report = run_access_command(&args).expect("access sync report");
    let route_ids = report
        .routes
        .iter()
        .map(|route| route.id.as_str())
        .collect::<Vec<_>>();
    assert!(route_ids.contains(&"process-env:clawcontrol-api-key"));
    assert!(route_ids.contains(&"process-env:approved-communications-file"));
    assert!(route_ids.contains(&"process-env:supermemory-api-key"));
    assert!(
        report
            .routes
            .iter()
            .all(|route| !route.secret_values_stored)
    );

    fs::remove_dir_all(output).expect("cleanup access temp");
}

#[test]
fn feature_registry_surfaces_tiny_context_packet_route_proof_honestly() {
    let output = std::env::temp_dir().join(format!(
        "memd-feature-context-core-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&output).expect("create feature temp");

    let report = build_feature_report(&output);
    let context = report
        .features
        .iter()
        .find(|feature| feature.id == "model_tier_context_compiler")
        .expect("context compiler feature");
    assert!(
        context
            .evidence
            .iter()
            .any(|item| item.contains("tiny/Ollama context packet route"))
    );
    assert!(
        context
            .evidence
            .iter()
            .any(|item| item.contains("lead with compact content"))
    );
    assert!(
        context
            .evidence
            .iter()
            .any(|item| item.contains("context packet matrix proof"))
    );
    assert!(
        context
            .evidence
            .iter()
            .any(|item| item.contains("tiny/small/medium model-tier budgets"))
    );
    assert!(
        context
            .evidence
            .iter()
            .any(|item| item.contains("Token Budget section"))
    );
    assert_eq!(context.status, "working");
    assert!(context.gaps.is_empty());

    fs::remove_dir_all(output).expect("cleanup feature temp");
}

#[test]
fn feature_registry_surfaces_knowledge_gap_guard_honestly() {
    let output = std::env::temp_dir().join(format!(
        "memd-feature-knowledge-gap-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&output).expect("create feature temp");

    let report = build_feature_report(&output);
    let guard = report
        .features
        .iter()
        .find(|feature| feature.id == "knowledge_gap_guard")
        .expect("knowledge gap guard feature");
    assert_eq!(guard.status, "working");
    assert!(
        guard
            .evidence
            .iter()
            .any(|item| item.contains("ask or look up durable memory"))
    );
    assert!(
        guard
            .evidence
            .iter()
            .any(|item| item.contains("Knowledge Gaps section"))
    );
    assert!(
        guard
            .evidence
            .iter()
            .any(|item| item.contains("save new user-taught facts"))
    );
    assert!(
        guard
            .evidence
            .iter()
            .any(|item| item.contains("memd teach"))
    );
    assert!(guard.gaps.is_empty());

    fs::remove_dir_all(output).expect("cleanup feature temp");
}

#[test]
fn feature_registry_surfaces_harness_context_guardrails_honestly() {
    let output = std::env::temp_dir().join(format!(
        "memd-feature-harness-guardrail-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&output).expect("create feature temp");

    let report = build_feature_report(&output);
    let guardrails = report
        .features
        .iter()
        .find(|feature| feature.id == "harness_context_guardrails")
        .expect("harness guardrails feature");

    assert_eq!(guardrails.status, "working");
    assert!(
        guardrails
            .evidence
            .iter()
            .any(|item| item.contains("include-capabilities"))
    );
    assert!(
        guardrails
            .evidence
            .iter()
            .any(|item| item.contains("unknown important facts"))
    );
    assert!(
        guardrails
            .evidence
            .iter()
            .any(|item| item.contains("save new user-taught facts"))
    );
    assert!(
        guardrails
            .evidence
            .iter()
            .any(|item| item.contains("Active Capabilities and Access Routes"))
    );
    assert!(guardrails.gaps.is_empty());

    fs::remove_dir_all(output).expect("cleanup feature temp");
}

#[test]
fn feature_registry_surfaces_hive_handoff_context_proof_honestly() {
    let output =
        std::env::temp_dir().join(format!("memd-feature-hive-core-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&output).expect("create feature temp");

    let report = build_feature_report(&output);
    let hive = report
        .features
        .iter()
        .find(|feature| feature.id == "shared_context_mesh")
        .expect("shared context mesh feature");
    assert_eq!(hive.status, "working");
    assert!(
        hive.evidence
            .iter()
            .any(|item| item.contains("queen handoff route"))
    );
    assert!(hive.gaps.is_empty());

    fs::remove_dir_all(output).expect("cleanup feature temp");
}

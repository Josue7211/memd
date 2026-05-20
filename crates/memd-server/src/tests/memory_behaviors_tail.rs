use super::*;

#[test]
fn o2_3_decay_sensitivity_analysis() {
    struct Scenario {
        name: &'static str,
        inactive_days: i64,
        max_decay: f32,
        decay_divisor: f32,
        // expectations on the old entities (40 days idle)
        expect_old_decayed: bool,
        // expectations on the recent entities (5 days idle)
        expect_recent_decayed: bool,
    }

    let scenarios = [
        Scenario {
            name: "defaults",
            inactive_days: 21,
            max_decay: 0.12,
            decay_divisor: 14.0,
            expect_old_decayed: true,
            expect_recent_decayed: false,
        },
        Scenario {
            name: "aggressive",
            inactive_days: 14,
            max_decay: 0.20,
            decay_divisor: 7.0,
            expect_old_decayed: true,
            expect_recent_decayed: false,
        },
        Scenario {
            name: "conservative",
            inactive_days: 30,
            max_decay: 0.06,
            decay_divisor: 21.0,
            expect_old_decayed: true,
            expect_recent_decayed: false,
        },
        Scenario {
            name: "fast_decay",
            inactive_days: 7,
            max_decay: 0.25,
            decay_divisor: 5.0,
            expect_old_decayed: true,
            expect_recent_decayed: false,
        },
        Scenario {
            name: "slow_decay",
            inactive_days: 45,
            max_decay: 0.04,
            decay_divisor: 30.0,
            expect_old_decayed: false, // 40 days < 45 threshold
            expect_recent_decayed: false,
        },
    ];

    let mut results_table: Vec<(String, usize, usize, f32)> = Vec::new();

    for scenario in &scenarios {
        let (dir, state) = temp_state(&format!("o2-3-decay-{}", scenario.name));

        // Seed 10 entities via store_item (auto-creates memory_entities rows).
        for i in 0..10 {
            let _ = state
                .store_item(
                    StoreMemoryRequest {
                        content: format!("decay sensitivity entity {i}"),
                        kind: MemoryKind::Fact,
                        scope: MemoryScope::Project,
                        project: Some("memd".to_string()),
                        namespace: Some("main".to_string()),
                        workspace: None,
                        visibility: Some(MemoryVisibility::Public),
                        belief_branch: None,
                        source_agent: Some("test".to_string()),
                        source_system: Some("cli".to_string()),
                        source_path: None,
                        source_quality: Some(SourceQuality::Canonical),
                        confidence: Some(0.7),
                        ttl_seconds: None,
                        last_verified_at: None,
                        supersedes: Vec::new(),
                        tags: Vec::new(),
                        status: Some(MemoryStatus::Active),
                        lane: None,
                    },
                    MemoryStage::Canonical,
                )
                .expect("seed entity");
        }

        // Age the first 5 entities to 40 days idle; last 5 stay at 5 days idle.
        let now = chrono::Utc::now();
        let old_ts = (now - chrono::Duration::days(40)).to_rfc3339();
        let recent_ts = (now - chrono::Duration::days(5)).to_rfc3339();

        {
            let conn = state.store.connect().expect("connect for age patch");
            let all_keys: Vec<String> = {
                let mut stmt = conn
                    .prepare("SELECT entity_key FROM memory_entities ORDER BY rowid ASC")
                    .expect("prepare entity keys query");
                stmt.query_map([], |row| row.get(0))
                    .expect("query entity keys")
                    .map(|r| r.expect("read entity key"))
                    .collect()
            };

            // Also backdate the salience in payload_json so pre-decay salience is predictable.
            for (idx, key) in all_keys.iter().enumerate() {
                let ts = if idx < 5 { &old_ts } else { &recent_ts };
                let salience = if idx < 5 { 0.6f32 } else { 0.8f32 };
                // Read the current payload, patch timestamps + salience, write back.
                let payload_json: String = conn
                    .query_row(
                        "SELECT payload_json FROM memory_entities WHERE entity_key = ?1",
                        rusqlite::params![key],
                        |row| row.get(0),
                    )
                    .expect("read entity payload");
                let mut record: memd_schema::MemoryEntityRecord =
                    serde_json::from_str(&payload_json).expect("deserialize entity");
                record.salience_score = salience;
                record.last_accessed_at = Some(
                    chrono::DateTime::parse_from_rfc3339(ts)
                        .expect("parse ts")
                        .with_timezone(&chrono::Utc),
                );
                record.updated_at = chrono::DateTime::parse_from_rfc3339(ts)
                    .expect("parse ts")
                    .with_timezone(&chrono::Utc);
                let patched = serde_json::to_string(&record).expect("re-serialize entity");
                conn.execute(
                    "UPDATE memory_entities SET updated_at = ?1, payload_json = ?2 WHERE entity_key = ?3",
                    rusqlite::params![ts, patched, key],
                )
                .expect("patch entity age");
            }
        }

        // Run decay_diagnostics (read-only — does not mutate; use decay_entities for real run).
        let req = MemoryDecayRequest {
            max_items: Some(20),
            inactive_days: Some(scenario.inactive_days),
            max_decay: Some(scenario.max_decay),
            decay_divisor: Some(scenario.decay_divisor),
            record_events: Some(false),
        };
        let metrics = state
            .store
            .decay_diagnostics(&req)
            .expect("decay diagnostics");

        // Validate age distribution: all 10 entities were inspected.
        assert_eq!(
            metrics.inspected, 10,
            "[{}] expected 10 entities inspected",
            scenario.name
        );

        // old entities (40 days idle) should fall in over_30d bucket.
        assert_eq!(
            metrics.age_distribution.over_30d, 5,
            "[{}] expected 5 entities in over_30d bucket",
            scenario.name
        );

        // recent entities (5 days idle) should fall in under_7d bucket.
        assert_eq!(
            metrics.age_distribution.under_7d, 5,
            "[{}] expected 5 entities in under_7d bucket",
            scenario.name
        );

        // Check decay expectations.
        if scenario.expect_old_decayed {
            assert!(
                metrics.decayed > 0,
                "[{}] expected old entities to be decayed but decayed={}",
                scenario.name,
                metrics.decayed
            );
        } else {
            assert_eq!(
                metrics.decayed, 0,
                "[{}] expected NO decay (threshold not met) but decayed={}",
                scenario.name, metrics.decayed
            );
        }

        if !scenario.expect_recent_decayed {
            // Recent entities should never be decayed (5 days < all inactive_days thresholds).
            // We can't distinguish which decayed, but if old threshold is met and recent is not:
            // decayed count should be <= 5 (only old entities, not recent).
            // This holds for all scenarios where recent entities are below threshold.
            assert!(
                metrics.decayed <= 5,
                "[{}] recent entities should not be decayed, decayed={}",
                scenario.name,
                metrics.decayed
            );
        }

        results_table.push((
            scenario.name.to_string(),
            metrics.decayed,
            metrics.inspected,
            metrics.total_decay_applied,
        ));

        std::fs::remove_dir_all(dir)
            .unwrap_or_else(|_| eprintln!("warn: cleanup failed for {}", scenario.name));
    }

    // Print comparison table for documentation.
    println!("\nO2.3 Decay Sensitivity Comparison Table:");
    println!(
        "{:<14} {:>8} {:>10} {:>16}",
        "scenario", "decayed", "inspected", "total_decay"
    );
    for (name, decayed, inspected, total) in &results_table {
        println!(
            "{:<14} {:>8} {:>10} {:>16.4}",
            name, decayed, inspected, total
        );
    }

    // Ranking check: aggressive > defaults > conservative for total_decay (when old entities present).
    let aggressive_decay = results_table
        .iter()
        .find(|(n, ..)| n == "aggressive")
        .map(|(_, _, _, d)| *d)
        .unwrap_or(0.0);
    let defaults_decay = results_table
        .iter()
        .find(|(n, ..)| n == "defaults")
        .map(|(_, _, _, d)| *d)
        .unwrap_or(0.0);
    let conservative_decay = results_table
        .iter()
        .find(|(n, ..)| n == "conservative")
        .map(|(_, _, _, d)| *d)
        .unwrap_or(0.0);
    let slow_decay = results_table
        .iter()
        .find(|(n, ..)| n == "slow_decay")
        .map(|(_, _, _, d)| *d)
        .unwrap_or(0.0);

    assert!(
        aggressive_decay >= defaults_decay,
        "aggressive params must decay at least as much as defaults: {aggressive_decay:.4} vs {defaults_decay:.4}"
    );
    assert!(
        defaults_decay >= conservative_decay,
        "defaults must decay at least as much as conservative: {defaults_decay:.4} vs {conservative_decay:.4}"
    );
    assert_eq!(
        slow_decay, 0.0,
        "slow_decay scenario: 40-day-old entities should not decay (threshold=45d)"
    );
}

// O2.5: Post-consolidation A/B recall comparison
//
// Proves that consolidation does NOT degrade retrieval:
//   (a) Store 10 items on the same topic (all map to one entity via shared source_path)
//   (b) Run 5 context queries and record pre-consolidation hit counts
//   (c) Generate retrieval events so consolidation threshold is met
//   (d) Run consolidation
//   (e) Run the same 5 queries again and assert post >= pre for each
#[test]
fn o2_5_post_consolidation_recall_ab_test() {
    let (dir, state) = temp_state("o2-5-recall-ab");
    let plan = RetrievalPlan::resolve(
        Some(RetrievalRoute::ProjectFirst),
        Some(RetrievalIntent::General),
    );

    // (a) Store 10 items on the same topic.  All share source_path so they map to one entity.
    let rust_facts = [
        "rust ownership model prevents use-after-free at compile time",
        "rust borrow checker enforces single mutable reference per scope",
        "rust lifetimes ensure references never outlive their referents",
        "rust move semantics transfer ownership without copying heap data",
        "rust drop trait runs destructors deterministically when scope ends",
        "rust rc and arc provide shared ownership via reference counting",
        "rust box type allocates heap memory with sole ownership",
        "rust slice references provide safe views into contiguous memory",
        "rust unsafe blocks allow raw pointer operations with explicit opt-in",
        "rust pin type prevents moving self-referential structs in memory",
    ];

    let mut all_items: Vec<MemoryItem> = Vec::new();
    for content in &rust_facts {
        let (item, _) = state
            .store_item(
                StoreMemoryRequest {
                    content: content.to_string(),
                    kind: MemoryKind::Fact,
                    scope: MemoryScope::Project,
                    project: Some("probe".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: None,
                    visibility: Some(MemoryVisibility::Public),
                    belief_branch: None,
                    source_agent: Some("codex".to_string()),
                    source_system: Some("cli".to_string()),
                    source_path: Some("topic/rust-memory".to_string()),
                    source_quality: Some(SourceQuality::Canonical),
                    confidence: Some(0.9),
                    ttl_seconds: None,
                    last_verified_at: Some(Utc::now()),
                    supersedes: Vec::new(),
                    tags: vec!["rust".to_string(), "memory".to_string()],
                    status: Some(MemoryStatus::Active),
                    lane: None,
                },
                MemoryStage::Canonical,
            )
            .expect("store rust fact");
        all_items.push(item);
    }
    assert_eq!(all_items.len(), 10, "expected 10 seeded items");

    // Helper closure: build context and count items that contain "rust" (all seeded items do).
    let query = |limit: usize| -> usize {
        build_context(
            &state,
            &ContextRequest {
                project: Some("probe".to_string()),
                agent: Some("o2-5-agent".to_string()),
                workspace: None,
                visibility: None,
                route: Some(RetrievalRoute::ProjectFirst),
                intent: Some(RetrievalIntent::General),
                limit: Some(limit),
                max_chars_per_item: Some(300),
            },
        )
        .expect("build context")
        .items
        .into_iter()
        .filter(|i| i.content.contains("rust"))
        .count()
    };

    // (b) Pre-consolidation baseline: run 5 queries with increasing limits.
    let pre = [query(5), query(8), query(10), query(12), query(20)];
    let pre_total: usize = pre.iter().sum();
    // Sanity: at least the smallest query returns something.
    assert!(
        pre[0] >= 1,
        "pre-consolidation baseline empty at limit=5; got {}",
        pre[0]
    );

    // (c) Record retrieval events twice so the entity hits min_events=2.
    state
        .record_retrieval_feedback(&all_items, all_items.len(), "retrieved_working", &plan)
        .expect("retrieval feedback pass 1");
    state
        .record_retrieval_feedback(&all_items, all_items.len(), "retrieved_working", &plan)
        .expect("retrieval feedback pass 2");

    // (d) Run consolidation — min_events=2 means one entity (all 10 items share source_path)
    //     should be consolidated into a single synthesised item.
    let response = state
        .consolidate_semantic_memory(&MemoryConsolidationRequest {
            project: Some("probe".to_string()),
            namespace: Some("main".to_string()),
            max_groups: Some(4),
            min_events: Some(2),
            lookback_days: Some(7),
            min_salience: Some(0.0),
            record_events: Some(false),
        })
        .expect("consolidate semantic memory");
    assert!(
        response.consolidated >= 1,
        "expected at least 1 consolidated item; got {}",
        response.consolidated
    );

    // (e) Post-consolidation: same 5 queries. Post >= pre for each, and in aggregate.
    let post = [query(5), query(8), query(10), query(12), query(20)];
    let post_total: usize = post.iter().sum();

    for (i, (pre_hits, post_hits)) in pre.iter().zip(post.iter()).enumerate() {
        assert!(
            post_hits >= pre_hits,
            "query[{i}]: recall degraded post-consolidation — pre={pre_hits} post={post_hits}"
        );
    }
    assert!(
        post_total >= pre_total,
        "aggregate recall degraded post-consolidation — pre={pre_total} post={post_total}"
    );

    // Consolidated (Derived) item must be discoverable via retrieval.
    let final_ctx = build_context(
        &state,
        &ContextRequest {
            project: Some("probe".to_string()),
            agent: Some("o2-5-agent".to_string()),
            workspace: None,
            visibility: None,
            route: Some(RetrievalRoute::ProjectFirst),
            intent: Some(RetrievalIntent::General),
            limit: Some(50),
            max_chars_per_item: Some(500),
        },
    )
    .expect("final context");
    assert!(
        final_ctx
            .items
            .iter()
            .any(|i| i.source_quality == Some(SourceQuality::Derived)),
        "consolidated (Derived) item must appear in retrieval after consolidation"
    );

    println!(
        "\nO2.5 A/B Recall: pre=[{},{},{},{},{}] post=[{},{},{},{},{}] total {pre_total}->{post_total}",
        pre[0], pre[1], pre[2], pre[3], pre[4], post[0], post[1], post[2], post[3], post[4],
    );

    std::fs::remove_dir_all(dir).expect("cleanup o2-5-recall-ab");
}

#[test]
fn p2_compaction_quality_report_includes_per_kind_chars() {
    // P2: Verify CompactionQualityReport tracks per-kind character counts
    let (dir, state) = temp_state("p2-per-kind-chars");

    // Store items of different kinds
    let kinds_and_content = vec![
        (MemoryKind::Fact, "The earth revolves around the sun"),
        (
            MemoryKind::Decision,
            "We chose Rust for memory safety and performance",
        ),
        (
            MemoryKind::Preference,
            "User prefers dark mode in the dashboard",
        ),
        (MemoryKind::Status, "M3 phase P2 is in progress"),
    ];

    for (kind, content) in &kinds_and_content {
        let mut item = sample_memory_item(None);
        item.kind = *kind;
        item.content = content.to_string();
        item.project = Some("p2-test".to_string());
        let ck = super::keys::canonical_key(&item);
        let rk = super::keys::redundancy_key(&item);
        state
            .store
            .insert_or_get_duplicate(&item, &ck, &rk)
            .expect("store p2 test item");
    }

    // Build working memory
    let req = memd_schema::WorkingMemoryRequest {
        project: Some("p2-test".to_string()),
        agent: None,
        workspace: None,
        visibility: None,
        route: Some(memd_schema::RetrievalRoute::ProjectFirst),
        intent: Some(memd_schema::RetrievalIntent::CurrentTask),
        limit: None,
        max_chars_per_item: None,
        max_total_chars: None,
        rehydration_limit: None,
        auto_consolidate: None,
        query: None,
    };

    let response = crate::working::working_memory(&state, req).expect("build working memory");

    // Verify compaction quality report exists and has per-kind char breakdown
    let cq = response
        .compaction_quality
        .expect("compaction quality report must exist");

    assert!(cq.admitted > 0, "at least one item should be admitted");
    assert!(
        !cq.per_kind_admitted.is_empty(),
        "per_kind_admitted should have entries"
    );
    assert!(
        !cq.chars_per_kind_admitted.is_empty(),
        "chars_per_kind_admitted should have entries (P2 per-kind char tracking)"
    );

    // Verify chars are non-zero for admitted kinds
    for (kind, chars) in &cq.chars_per_kind_admitted {
        assert!(
            *chars > 0,
            "kind '{kind}' should have non-zero character count"
        );
    }

    // Verify budget utilization
    assert!(cq.budget_chars > 0, "budget_chars should be positive");
    assert!(
        cq.used_chars <= cq.budget_chars,
        "used_chars ({}) should not exceed budget_chars ({})",
        cq.used_chars,
        cq.budget_chars
    );

    println!(
        "\nP2 Token Efficiency: budget={}, used={}, utilization={:.1}%",
        cq.budget_chars,
        cq.used_chars,
        (cq.used_chars as f64 / cq.budget_chars as f64) * 100.0
    );
    println!("Per-kind chars: {:?}", cq.chars_per_kind_admitted);

    std::fs::remove_dir_all(dir).expect("cleanup p2 test");
}

#[test]
fn working_memory_retrieval_p95_under_100ms() {
    // K2.6 CI gate: seed a realistic corpus, issue N working-memory requests
    // through the same path as the /memory/working handler, and assert the
    // histogram p95 stays under the SLA. Threshold is intentionally generous
    // to absorb cold-cache noise on CI; regressions above this point mean
    // the retrieval path has drifted.
    let (dir, state) = temp_state("memd-latency-sla");
    for n in 0..64 {
        state
            .store_item(
                StoreMemoryRequest {
                    content: format!("warm fact {n}"),
                    kind: MemoryKind::Fact,
                    scope: MemoryScope::Project,
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: None,
                    visibility: None,
                    belief_branch: None,
                    source_agent: Some("codex".to_string()),
                    source_system: None,
                    source_path: None,
                    source_quality: None,
                    confidence: None,
                    ttl_seconds: None,
                    last_verified_at: None,
                    supersedes: vec![],
                    tags: vec![],
                    status: None,
                    lane: None,
                },
                MemoryStage::Canonical,
            )
            .expect("seed fact");
    }

    // Warm caches: the first few working-memory calls hit cold SQLite
    // page cache + fresh Tantivy reader; on debug builds those samples
    // routinely land 5-10x slower than steady state and pollute p95
    // when only 20 samples are measured.
    for _ in 0..5 {
        let _ = crate::working::working_memory(
            &state,
            WorkingMemoryRequest {
                project: Some("memd".to_string()),
                agent: Some("codex".to_string()),
                workspace: None,
                visibility: None,
                route: None,
                intent: Some(RetrievalIntent::CurrentTask),
                limit: Some(8),
                max_chars_per_item: Some(220),
                max_total_chars: Some(1600),
                rehydration_limit: Some(4),
                auto_consolidate: Some(false),
                query: None,
            },
        )
        .expect("warm working memory");
    }

    for _ in 0..20 {
        let started = std::time::Instant::now();
        let _ = crate::working::working_memory(
            &state,
            WorkingMemoryRequest {
                project: Some("memd".to_string()),
                agent: Some("codex".to_string()),
                workspace: None,
                visibility: None,
                route: None,
                intent: Some(RetrievalIntent::CurrentTask),
                limit: Some(8),
                max_chars_per_item: Some(220),
                max_total_chars: Some(1600),
                rehydration_limit: Some(4),
                auto_consolidate: Some(false),
                query: None,
            },
        )
        .expect("working memory");
        state
            .latency
            .record_ms(started.elapsed().as_millis() as u64);
    }

    let snap = state.latency.snapshot();
    assert!(
        snap.total >= 20,
        "expected 20 recorded samples, got {}",
        snap.total
    );

    // Debug builds run SQLite-bound paths roughly 5-10x slower than release,
    // so the hard 100ms gate only fires in release/CI. Debug gets a looser
    // smoke check that still catches pathological regressions.
    #[cfg(not(debug_assertions))]
    assert!(
        snap.p95_ms < 100.0,
        "working-memory retrieval p95 exceeded 100ms SLA: p95={} mean={} max={}",
        snap.p95_ms,
        snap.mean_ms,
        snap.max_ms,
    );
    // Debug-build threshold is 1000ms — empirical: on local CachyOS
    // machines, debug-build SQLite + Tantivy lookups land in the
    // 250-500ms band per call even after a 5-call warm-up, so the
    // earlier 500ms gate flaked routinely (observed p95=512, mean=272).
    // The histogram reports bucket upper bounds, so a true sub-1000ms p95 can
    // surface as 1024ms. Keep the debug smoke bound at that bucket edge; the
    // real SLA is the release gate above.
    #[cfg(debug_assertions)]
    assert!(
        snap.p95_ms <= 1024.0,
        "debug-build working-memory p95 regression: p95={} mean={}",
        snap.p95_ms,
        snap.mean_ms,
    );

    std::fs::remove_dir_all(dir).expect("cleanup latency sla test");
}

#[test]
fn spine_verify_reports_no_violations_on_clean_store() {
    let (dir, state) = temp_state("memd-spine-verify");

    for n in 0..3 {
        state
            .store_item(
                StoreMemoryRequest {
                    content: format!("fact {n}"),
                    kind: MemoryKind::Fact,
                    scope: MemoryScope::Project,
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: None,
                    visibility: None,
                    belief_branch: None,
                    source_agent: Some("test".to_string()),
                    source_system: None,
                    source_path: None,
                    source_quality: None,
                    confidence: None,
                    ttl_seconds: None,
                    last_verified_at: None,
                    supersedes: vec![],
                    tags: vec![],
                    status: None,
                    lane: None,
                },
                MemoryStage::Canonical,
            )
            .expect("store fact");
    }

    let report = state.store.verify_spine().expect("verify spine");
    assert!(report.scanned > 0, "at least the store events should scan");
    assert_eq!(report.monotonic_violations, 0);
    assert!(report.first_violation.is_none());
    assert_eq!(report.rolling_sha256.len(), 64);
    assert!(report.rolling_sha256.chars().all(|c| c.is_ascii_hexdigit()));

    let again = state.store.verify_spine().expect("verify spine again");
    assert_eq!(
        again.rolling_sha256, report.rolling_sha256,
        "rolling hash should be deterministic across calls"
    );

    std::fs::remove_dir_all(dir).expect("cleanup spine verify test");
}

#[test]
fn l2_1_lamport_version_increments_on_mutation_and_rejects_stale_imports() {
    let (dir, state) = temp_state("l2-1-lamport-version");
    let item = sample_memory_item(Some("core"));
    let id = item.id;
    let ck = keys::canonical_key(&item);
    let rk = keys::redundancy_key(&item);

    // Insert: persisted version starts at 1.
    state
        .store
        .insert_or_get_duplicate(&item, &ck, &rk)
        .expect("insert new item");
    assert_eq!(
        state.store.get_version(id).expect("read version"),
        Some(1),
        "fresh insert persists at version 1"
    );

    // Local mutation: version auto-increments to 2.
    let mut mutated = item.clone();
    mutated.content = "workspace-ranked memory (edited)".to_string();
    mutated.updated_at = Utc::now();
    let ck2 = keys::canonical_key(&mutated);
    let rk2 = keys::redundancy_key(&mutated);
    state
        .store
        .update(&mutated, &ck2, &rk2)
        .expect("update item");
    assert_eq!(
        state
            .store
            .get_version(id)
            .expect("read version after update"),
        Some(2),
        "update bumps Lamport version by 1"
    );
    let stored = state.store.get(id).expect("get").expect("row present");
    assert_eq!(stored.version, 2, "payload_json and column stay in sync");

    // Import with equal version: rejected.
    let mut stale = stored.clone();
    stale.version = 2;
    let outcome = state
        .store
        .import_with_version(&stale, &ck2, &rk2)
        .expect("import call");
    assert_eq!(
        outcome,
        crate::store::ImportOutcome::RejectedStale {
            stored_version: 2,
            incoming_version: 2,
        },
        "equal-version import must be treated as stale"
    );

    // Import with strictly-greater version: applied, version becomes 5.
    let mut fresh = stored.clone();
    fresh.version = 5;
    fresh.content = "workspace-ranked memory (remote)".to_string();
    fresh.updated_at = Utc::now();
    let ck3 = keys::canonical_key(&fresh);
    let rk3 = keys::redundancy_key(&fresh);
    let outcome = state
        .store
        .import_with_version(&fresh, &ck3, &rk3)
        .expect("import fresh");
    assert_eq!(outcome, crate::store::ImportOutcome::Applied);
    assert_eq!(
        state
            .store
            .get_version(id)
            .expect("read version post-import"),
        Some(5),
        "accepted import preserves incoming version exactly"
    );

    std::fs::remove_dir_all(dir).expect("cleanup L2.1 test");
}

// L2.6: end-to-end rate limit middleware. Wires a one-route router exactly
// like the real app — same `from_fn_with_state` layer — and hammers it. Reads
// stay unthrottled, writes cross both thresholds, header carries agent key.
#[tokio::test]
async fn rate_limit_middleware_throttles_writes_per_agent_and_passes_reads() {
    let dir = std::env::temp_dir().join(format!("memd-rl-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let db_path = dir.join("memd.db");
    let state = AppState {
        store: SqliteStore::open(&db_path).expect("open temp store"),
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::with(
            2,
            4,
            std::time::Duration::from_secs(60),
        )),
        rag: None,
        embedder: None,
    };

    let app = Router::new()
        .route(
            "/ping",
            axum::routing::get(|| async { "pong" }).post(|| async { "wrote" }),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::rate_limit::rate_limit_middleware,
        ))
        .with_state(state);

    async fn status_and_headers(
        app: &Router,
        method: &str,
        agent: Option<&str>,
    ) -> (axum::http::StatusCode, axum::http::HeaderMap) {
        let mut req = Request::builder().method(method).uri("/ping");
        if let Some(a) = agent {
            req = req.header("x-memd-agent", a);
        }
        let resp = app
            .clone()
            .oneshot(req.body(Body::empty()).unwrap())
            .await
            .expect("oneshot");
        (resp.status(), resp.headers().clone())
    }

    // GET is never throttled.
    for _ in 0..10 {
        let (status, _) = status_and_headers(&app, "GET", Some("agent-1")).await;
        assert_eq!(status, axum::http::StatusCode::OK);
    }

    // Writes 1..=2 succeed for agent-1 (soft=2).
    let (s1, h1) = status_and_headers(&app, "POST", Some("agent-1")).await;
    assert_eq!(s1, axum::http::StatusCode::OK);
    assert!(h1.contains_key("x-memd-ratelimit-remaining"));
    let (s2, _) = status_and_headers(&app, "POST", Some("agent-1")).await;
    assert_eq!(s2, axum::http::StatusCode::OK);

    // Writes 3..=4 are soft-throttled → 429 + Retry-After.
    let (s3, h3) = status_and_headers(&app, "POST", Some("agent-1")).await;
    assert_eq!(s3, axum::http::StatusCode::TOO_MANY_REQUESTS);
    assert!(h3.contains_key("retry-after"));
    let (s4, _) = status_and_headers(&app, "POST", Some("agent-1")).await;
    assert_eq!(s4, axum::http::StatusCode::TOO_MANY_REQUESTS);

    // Write 5 is hard-rejected (still 429 but tier="hard").
    let (s5, _) = status_and_headers(&app, "POST", Some("agent-1")).await;
    assert_eq!(s5, axum::http::StatusCode::TOO_MANY_REQUESTS);

    // A different agent's bucket is independent.
    let (s_b1, _) = status_and_headers(&app, "POST", Some("agent-2")).await;
    assert_eq!(s_b1, axum::http::StatusCode::OK);

    std::fs::remove_dir_all(dir).expect("cleanup rl test");
}

// L2.7: 10 threads × 100 writes each. busy_timeout=5000 + WAL journal must
// absorb contention entirely. If any thread surfaces SQLITE_BUSY, L2-D3
// (the WAL+busy_timeout guarantees from M2) regressed — fail hard.
#[test]
fn concurrency_10_threads_100_writes_no_sqlite_busy_surfaces() {
    let dir = std::env::temp_dir().join(format!("memd-concurrency-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let db_path = dir.join("state.sqlite");
    let store = SqliteStore::open(&db_path).expect("open store");

    const THREADS: usize = 10;
    const PER_THREAD: usize = 100;

    let start = std::sync::Arc::new(std::sync::Barrier::new(THREADS));
    let mut handles = Vec::with_capacity(THREADS);
    for tid in 0..THREADS {
        let store = store.clone();
        let start = start.clone();
        handles.push(std::thread::spawn(move || {
            start.wait();
            let mut busy_hits = 0usize;
            let mut other_errors: Vec<String> = Vec::new();
            for i in 0..PER_THREAD {
                let mut item = sample_memory_item(None);
                item.id = uuid::Uuid::new_v4();
                item.content = format!("concurrency t{tid}-i{i}");
                item.tags = vec!["concurrency".to_string(), format!("t{tid}")];
                item.source_agent = Some(format!("agent-t{tid}"));
                let ck = keys::canonical_key(&item);
                let rk = keys::redundancy_key(&item);
                match store.insert_or_get_duplicate(&item, &ck, &rk) {
                    Ok(_) => {}
                    Err(err) => {
                        let msg = format!("{err:#}").to_lowercase();
                        if msg.contains("busy") || msg.contains("database is locked") {
                            busy_hits += 1;
                        } else {
                            other_errors.push(format!("t{tid}-i{i}: {err:#}"));
                        }
                    }
                }
            }
            (busy_hits, other_errors)
        }));
    }

    let mut total_busy = 0usize;
    let mut all_errors: Vec<String> = Vec::new();
    for h in handles {
        let (busy, errs) = h.join().expect("thread joined");
        total_busy += busy;
        all_errors.extend(errs);
    }

    assert_eq!(
        total_busy, 0,
        "SQLITE_BUSY must not surface under WAL + 5000ms busy_timeout (L2-D3 regressed)"
    );
    assert!(
        all_errors.is_empty(),
        "unexpected non-busy errors: {all_errors:#?}"
    );

    // Row count sanity: each thread produced 100 distinct writes.
    let conn = rusqlite::Connection::open(&db_path).expect("reopen to count");
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM memory_items WHERE source_agent LIKE 'agent-t%'",
            [],
            |row| row.get(0),
        )
        .expect("count rows");
    assert_eq!(count, (THREADS * PER_THREAD) as i64);

    std::fs::remove_dir_all(dir).expect("cleanup concurrency test");
}

// L2.8: cross-harness E2E — codex-style harness A hands off to
// claude-code-style harness B, B makes corrections, A wakes up and picks
// them up. Runs against the shared store (single sqlite file) the way two
// harnesses coexist in production: distinct source_agent, shared storage.
#[test]
fn cross_harness_e2e_a_to_b_with_corrections_picked_up_by_a() {
    use memd_schema::{
        CompactMemoryRecord, HiveHandoffPacket, Procedure, ProcedureKind, ProcedureStatus,
        WorkingContextSnapshot,
    };

    let dir = std::env::temp_dir().join(format!("memd-x-harness-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let store = SqliteStore::open(dir.join("state.sqlite")).expect("open store");

    // Shared namespace/workspace — both harnesses operate on the same lane.
    let project = Some("memd".to_string());
    let namespace = Some("main".to_string());
    let workspace = Some("shared".to_string());

    fn seed_as(
        store: &SqliteStore,
        agent: &str,
        kind: MemoryKind,
        content: &str,
        project: &Option<String>,
        namespace: &Option<String>,
        workspace: &Option<String>,
    ) -> MemoryItem {
        let now = Utc::now();
        let item = MemoryItem {
            id: uuid::Uuid::new_v4(),
            content: content.to_string(),
            redundancy_key: None,
            belief_branch: None,
            preferred: false,
            kind,
            scope: MemoryScope::Project,
            project: project.clone(),
            namespace: namespace.clone(),
            workspace: workspace.clone(),
            visibility: MemoryVisibility::Workspace,
            source_agent: Some(agent.to_string()),
            source_system: Some("cross-harness-test".to_string()),
            source_path: None,
            source_quality: Some(SourceQuality::Canonical),
            confidence: 0.9,
            ttl_seconds: None,
            created_at: now,
            updated_at: now,
            last_verified_at: Some(now),
            supersedes: Vec::new(),
            tags: vec!["cross-harness".to_string()],
            status: MemoryStatus::Active,
            stage: MemoryStage::Canonical,
            lane: None,
            version: 1,
            correction_meta: None,
        };
        let ck = keys::canonical_key(&item);
        let rk = keys::redundancy_key(&item);
        store
            .insert_or_get_duplicate(&item, &ck, &rk)
            .expect("seed item");
        item
    }

    // (a) Harness A: 3 facts + 2 decisions + 1 procedure candidate.
    let a_fact_1 = seed_as(
        &store,
        "codex@A",
        MemoryKind::Fact,
        "hive uses Lamport clocks",
        &project,
        &namespace,
        &workspace,
    );
    let a_fact_2 = seed_as(
        &store,
        "codex@A",
        MemoryKind::Fact,
        "writes retry under WAL",
        &project,
        &namespace,
        &workspace,
    );
    let a_fact_3 = seed_as(
        &store,
        "codex@A",
        MemoryKind::Fact,
        "procedural memory is 8-slot bounded",
        &project,
        &namespace,
        &workspace,
    );
    let a_dec_1 = seed_as(
        &store,
        "codex@A",
        MemoryKind::Decision,
        "adopt FTS5 for search",
        &project,
        &namespace,
        &workspace,
    );
    let a_dec_2 = seed_as(
        &store,
        "codex@A",
        MemoryKind::Decision,
        "use SQLite online backup for snapshots",
        &project,
        &namespace,
        &workspace,
    );
    let a_proc_seed = seed_as(
        &store,
        "codex@A",
        MemoryKind::Procedural,
        "when tests fail intermittently, check busy_timeout",
        &project,
        &namespace,
        &workspace,
    );

    // (b) Build a handoff packet from A. Snapshot carries working records +
    //     unresolved procedure candidate.
    let compact = |item: &MemoryItem| CompactMemoryRecord {
        id: item.id,
        record: item.content.clone(),
    };
    let snapshot = WorkingContextSnapshot {
        working_records: vec![
            compact(&a_fact_1),
            compact(&a_fact_2),
            compact(&a_fact_3),
            compact(&a_dec_1),
            compact(&a_dec_2),
        ],
        doing: Some("ship L2".to_string()),
        left_off: Some("just finished L2.7".to_string()),
        next_action: Some("start L2.8".to_string()),
        blocker: None,
        unresolved_procedures: vec![Procedure {
            id: uuid::Uuid::new_v4(),
            name: "retry on flakes".to_string(),
            description: "observed pattern from A".to_string(),
            kind: ProcedureKind::Recovery,
            status: ProcedureStatus::Candidate,
            trigger: "tests flake".to_string(),
            steps: vec!["check busy_timeout".to_string()],
            success_criteria: None,
            source_ids: vec![a_proc_seed.id],
            project: project.clone(),
            namespace: namespace.clone(),
            use_count: 0,
            confidence: 0.6,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: vec!["cross-harness".to_string()],
            session_count: 0,
            last_session: Some("A".to_string()),
            supersedes: None,
        }],
        version: 1,
        captured_at: Some(Utc::now()),
    }
    .truncate_to_cap();

    let packet = HiveHandoffPacket {
        from_session: "A".to_string(),
        from_worker: Some("codex".to_string()),
        to_session: "B".to_string(),
        to_worker: Some("claude-code".to_string()),
        task_id: Some("ship-l2".to_string()),
        topic_claim: Some("L2 hive hardening".to_string()),
        scope_claims: Vec::new(),
        next_action: Some("start L2.8".to_string()),
        blocker: None,
        note: Some("running out of context".to_string()),
        created_at: Utc::now(),
        working_context: Some(snapshot.clone()),
    };

    // (c) Harness B resumes. In production this fans out to store calls;
    //     here we validate the contract: all seeded items referenced by the
    //     snapshot are visible via the shared store.
    for rec in &packet.working_context.as_ref().unwrap().working_records {
        let row = store
            .get(rec.id)
            .expect("store.get works")
            .expect("handed-off item visible to B");
        assert_eq!(row.content, rec.record);
    }
    assert_eq!(
        packet
            .working_context
            .as_ref()
            .unwrap()
            .working_records
            .len(),
        5,
        "3 facts + 2 decisions"
    );
    assert_eq!(
        packet
            .working_context
            .as_ref()
            .unwrap()
            .unresolved_procedures
            .len(),
        1,
        "1 procedure candidate"
    );

    // (e) Harness B: correction to fact_1 + new decision.
    let mut corrected_fact = a_fact_1.clone();
    corrected_fact.id = uuid::Uuid::new_v4();
    corrected_fact.content = "hive uses Lamport clocks (versioned u64)".to_string();
    corrected_fact.source_agent = Some("claude-code@B".to_string());
    corrected_fact.supersedes = vec![a_fact_1.id];
    corrected_fact.updated_at = Utc::now();
    let ck = keys::canonical_key(&corrected_fact);
    let rk = keys::redundancy_key(&corrected_fact);
    store
        .insert_or_get_duplicate(&corrected_fact, &ck, &rk)
        .expect("B writes correction");

    let b_new_dec = seed_as(
        &store,
        "claude-code@B",
        MemoryKind::Decision,
        "prefer online backup over cold copy",
        &project,
        &namespace,
        &workspace,
    );

    // (f) Harness A wakes up — reads from same store, sees B's additions.
    let all_items = {
        let conn = rusqlite::Connection::open(dir.join("state.sqlite")).expect("reopen to query");
        let mut stmt = conn
            .prepare("SELECT payload_json FROM memory_items")
            .unwrap();

        stmt.query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .map(|r| {
                let s = r.unwrap();
                serde_json::from_str::<MemoryItem>(&s).expect("decode payload")
            })
            .collect::<Vec<_>>()
    };

    let correction_visible = all_items
        .iter()
        .any(|i| i.id == corrected_fact.id && i.supersedes == vec![a_fact_1.id]);
    assert!(correction_visible, "A must see B's correction chain");

    let new_dec_visible = all_items.iter().any(|i| i.id == b_new_dec.id);
    assert!(new_dec_visible, "A must see B's newly added decision");

    // And the originals remain reachable so the supersedes chain works.
    for original in [
        &a_fact_1,
        &a_fact_2,
        &a_fact_3,
        &a_dec_1,
        &a_dec_2,
        &a_proc_seed,
    ] {
        let found = all_items.iter().any(|i| i.id == original.id);
        assert!(found, "original item {} still in shared store", original.id);
    }

    std::fs::remove_dir_all(dir).expect("cleanup x-harness");
}

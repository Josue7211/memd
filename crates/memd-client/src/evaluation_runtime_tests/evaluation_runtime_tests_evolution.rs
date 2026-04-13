    mod evaluation_runtime_tests_evolution;

    #[tokio::test]
    async fn run_experiment_command_restores_bundle_when_rejected() {
        let dir =
            std::env::temp_dir().join(format!("memd-experiment-reject-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create experiment temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "demo",
  "agent": "codex",
  "route": "auto",
  "intent": "general"
}
"#,
        )
        .expect("write config");
        fs::write(dir.join("mem.md"), "# sentinel memory\n").expect("write memory seed");
        fs::create_dir_all(dir.join("agents")).expect("create agents dir");
        fs::write(
            dir.join("mem.md"),
            "# agent sentinel\n",
        )
        .expect("write agent memory seed");

        let base_url = spawn_mock_memory_server().await;
        let report = run_experiment_command(
            &ExperimentArgs {
                output: dir.clone(),
                max_iterations: 1,
                limit: Some(8),
                recent_commits: Some(1),
                accept_below: 99,
                apply: true,
                consolidate: false,
                write: false,
                summary: false,
            },
            &base_url,
        )
        .await
        .expect("run experiment");

        assert!(!report.accepted);
        assert!(report.restored);
        assert!(
            fs::read_to_string(dir.join("mem.md"))
                .expect("read restored memory")
                .contains("sentinel memory")
        );
        write_experiment_artifacts(&dir, &report).expect("write experiment artifacts");
        assert!(dir.join("experiments").join("latest.json").exists());

        fs::remove_dir_all(dir).expect("cleanup experiment temp bundle");
    }

    #[tokio::test]
    async fn run_experiment_command_consolidates_accepted_learnings() {
        let dir =
            std::env::temp_dir().join(format!("memd-experiment-accept-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create experiment temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "demo",
  "agent": "codex",
  "route": "auto",
  "intent": "general"
}
"#,
        )
        .expect("write config");
        fs::write(dir.join("mem.md"), "# baseline memory\n").expect("write memory seed");
        fs::create_dir_all(dir.join("agents")).expect("create agents dir");
        fs::write(
            dir.join("mem.md"),
            "# agent baseline\n",
        )
        .expect("write agent memory seed");

        let eval = high_scoring_eval(&dir);
        write_bundle_eval_artifacts(&dir, &eval).expect("write eval artifacts");
        let scenario = high_scoring_scenario(&dir);
        write_scenario_artifacts(&dir, &scenario).expect("write scenario artifacts");

        let base_url = spawn_mock_memory_server().await;
        let report = run_experiment_command(
            &ExperimentArgs {
                output: dir.clone(),
                max_iterations: 0,
                limit: Some(8),
                recent_commits: Some(1),
                accept_below: 80,
                apply: false,
                consolidate: true,
                write: false,
                summary: false,
            },
            &base_url,
        )
        .await
        .expect("run experiment");

        assert!(report.accepted);
        assert!(!report.restored);
        assert!(!report.learnings.is_empty());
        assert!(
            fs::read_to_string(dir.join("mem.md"))
                .expect("read consolidated memory")
                .contains("Accepted Experiment")
        );
        write_experiment_artifacts(&dir, &report).expect("write experiment artifacts");
        let latest_json = dir.join("experiments").join("latest.json");
        assert!(latest_json.exists());
        let parsed: ExperimentReport =
            serde_json::from_str(&fs::read_to_string(&latest_json).expect("read experiment json"))
                .expect("parse experiment report");
        assert!(parsed.accepted);

        fs::remove_dir_all(dir).expect("cleanup experiment temp bundle");
    }

    #[tokio::test]
    async fn run_experiment_command_forces_one_iteration_when_apply_true_and_max_iterations_zero() {
        let dir = std::env::temp_dir().join(format!(
            "memd-experiment-force-iteration-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).expect("create experiment temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "demo",
  "agent": "codex",
  "route": "auto",
  "intent": "general"
}
"#,
        )
        .expect("write config");
        fs::write(dir.join("mem.md"), "# baseline memory\n").expect("write memory seed");
        fs::create_dir_all(dir.join("agents")).expect("create agents dir");
        fs::write(
            dir.join("mem.md"),
            "# agent baseline\n",
        )
        .expect("write agent memory seed");

        let base_url = spawn_mock_memory_server().await;
        let report = run_experiment_command(
            &ExperimentArgs {
                output: dir.clone(),
                max_iterations: 0,
                limit: Some(8),
                recent_commits: Some(1),
                accept_below: 80,
                apply: true,
                consolidate: false,
                write: false,
                summary: false,
            },
            &base_url,
        )
        .await
        .expect("run experiment");

        assert_eq!(report.improvement.max_iterations, 1);
        assert_eq!(report.improvement.iterations.len(), 1);
        assert!(
            report
                .trail
                .iter()
                .any(|line| line.contains("max_iterations=1"))
        );

        fs::remove_dir_all(dir).expect("cleanup experiment temp bundle");
    }

    #[test]
    fn write_experiment_artifacts_writes_evolution_proposal_and_ledger() {
        let dir =
            std::env::temp_dir().join(format!("memd-evolution-proposal-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");

        let mut report = test_experiment_report(&dir, true, false, 92, 100, Utc::now());
        report.improvement.final_changes = vec![
            "tighten scoring heuristic for self-evolution loop".to_string(),
            "adjust loop threshold floor".to_string(),
        ];
        write_experiment_artifacts(&dir, &report).expect("write experiment artifacts");

        let proposal_path = dir.join("evolution").join("latest-proposal.json");
        let ledger_path = dir.join("evolution").join("durability-ledger.json");
        assert!(proposal_path.exists());
        assert!(ledger_path.exists());

        let proposal: EvolutionProposalReport =
            serde_json::from_str(&fs::read_to_string(&proposal_path).expect("read proposal"))
                .expect("parse proposal");
        assert_eq!(proposal.state, "accepted_proposal");
        assert_eq!(proposal.scope_gate, "auto_merge");
        assert!(proposal.merge_eligible);

        let ledger: EvolutionDurabilityLedger =
            serde_json::from_str(&fs::read_to_string(&ledger_path).expect("read ledger"))
                .expect("parse ledger");
        assert_eq!(ledger.entries.len(), 1);
        assert_eq!(ledger.entries[0].state, "accepted_proposal");

        fs::remove_dir_all(dir).expect("cleanup experiment temp bundle");
    }

    #[test]
    fn write_experiment_artifacts_keeps_coordination_scope_on_proposal_branch() {
        let dir = std::env::temp_dir().join(format!(
            "memd-evolution-proposal-coordination-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).expect("create temp bundle");

        let mut report = test_experiment_report(&dir, true, false, 91, 100, Utc::now());
        report.improvement.final_changes =
            vec!["change hive coordination protocol for task claims".to_string()];
        write_experiment_artifacts(&dir, &report).expect("write experiment artifacts");

        let proposal: EvolutionProposalReport = serde_json::from_str(
            &fs::read_to_string(dir.join("evolution").join("latest-proposal.json"))
                .expect("read proposal"),
        )
        .expect("parse proposal");
        assert_eq!(proposal.scope_class, "coordination_semantics");
        assert_eq!(proposal.scope_gate, "proposal_only");
        assert!(!proposal.merge_eligible);
        assert_eq!(proposal.state, "accepted_proposal");

        fs::remove_dir_all(dir).expect("cleanup experiment temp bundle");
    }

    #[test]
    fn write_experiment_artifacts_creates_isolated_evolution_branch_and_authority_ledger() {
        let root =
            std::env::temp_dir().join(format!("memd-evolution-branch-{}", uuid::Uuid::new_v4()));
        let output = root.join(".memd");
        fs::create_dir_all(&output).expect("create temp bundle");
        write_test_bundle_config(&output, "http://127.0.0.1:59999");

        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("init")
            .status()
            .expect("init git repo");
        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("config")
            .arg("user.email")
            .arg("memd@example.com")
            .status()
            .expect("git user email");
        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("config")
            .arg("user.name")
            .arg("memd")
            .status()
            .expect("git user name");
        fs::write(root.join("README.md"), "# demo\n").expect("write readme");
        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("add")
            .arg(".")
            .status()
            .expect("git add");
        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("commit")
            .arg("-m")
            .arg("init")
            .status()
            .expect("git commit");

        let mut report = test_experiment_report(&output, true, false, 100, 100, Utc::now());
        report.improvement.final_changes = vec![
            "tighten evaluation score heuristic".to_string(),
            "adjust loop threshold floor".to_string(),
        ];
        write_experiment_artifacts(&output, &report).expect("write experiment artifacts");

        let branch_manifest: EvolutionBranchManifest = serde_json::from_str(
            &fs::read_to_string(output.join("evolution").join("latest-branch.json"))
                .expect("read branch manifest"),
        )
        .expect("parse branch manifest");
        assert!(matches!(
            branch_manifest.status.as_str(),
            "created" | "existing"
        ));
        assert!(branch_manifest.branch.starts_with("auto/evolution/"));

        let branch_exists = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("show-ref")
            .arg("--verify")
            .arg(format!("refs/heads/{}", branch_manifest.branch))
            .status()
            .expect("show ref")
            .success();
        assert!(branch_exists);

        let authority: EvolutionAuthorityLedger = serde_json::from_str(
            &fs::read_to_string(output.join("evolution").join("authority-ledger.json"))
                .expect("read authority ledger"),
        )
        .expect("parse authority ledger");
        assert_eq!(authority.entries.len(), 1);
        assert_eq!(authority.entries[0].authority_tier, "phase1_auto_merge");

        let merge_queue: EvolutionMergeQueue = serde_json::from_str(
            &fs::read_to_string(output.join("evolution").join("merge-queue.json"))
                .expect("read merge queue"),
        )
        .expect("parse merge queue");
        assert_eq!(merge_queue.entries.len(), 1);
        assert_eq!(merge_queue.entries[0].status, "no_diff");

        let durability_queue: EvolutionDurabilityQueue = serde_json::from_str(
            &fs::read_to_string(output.join("evolution").join("durability-queue.json"))
                .expect("read durability queue"),
        )
        .expect("parse durability queue");
        assert_eq!(durability_queue.entries.len(), 1);
        assert_eq!(durability_queue.entries[0].status, "no_diff");

        fs::remove_dir_all(root).expect("cleanup temp root");
    }

    #[test]
    fn git_worktree_conflicts_with_branch_detects_overlap() {
        let root =
            std::env::temp_dir().join(format!("memd-evolution-dirty-{}", uuid::Uuid::new_v4()));
        let output = root.join(".memd");
        fs::create_dir_all(&output).expect("create temp bundle");
        write_test_bundle_config(&output, "http://127.0.0.1:59999");

        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("init")
            .status()
            .expect("init git repo");
        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("config")
            .arg("user.email")
            .arg("memd@example.com")
            .status()
            .expect("git user email");
        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("config")
            .arg("user.name")
            .arg("memd")
            .status()
            .expect("git user name");
        fs::write(root.join("README.md"), "# demo\n").expect("write readme");
        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("add")
            .arg(".")
            .status()
            .expect("git add");
        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("commit")
            .arg("-m")
            .arg("init")
            .status()
            .expect("git commit");

        let mut report = test_experiment_report(&output, true, false, 100, 100, Utc::now());
        report.improvement.final_changes = vec!["adjust loop threshold floor".to_string()];
        write_experiment_artifacts(&output, &report).expect("write experiment artifacts");

        let proposal: EvolutionProposalReport = serde_json::from_str(
            &fs::read_to_string(output.join("evolution").join("latest-proposal.json"))
                .expect("read proposal"),
        )
        .expect("parse proposal");
        let branch_manifest: EvolutionBranchManifest = serde_json::from_str(
            &fs::read_to_string(output.join("evolution").join("latest-branch.json"))
                .expect("read branch manifest"),
        )
        .expect("parse branch manifest");

        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("checkout")
            .arg(&proposal.branch)
            .status()
            .expect("checkout proposal branch");
        fs::write(root.join("README.md"), "# demo\n\noverlap change\n")
            .expect("write branch change");
        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("add")
            .arg("README.md")
            .status()
            .expect("git add readme");
        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("commit")
            .arg("-m")
            .arg("proposal change")
            .status()
            .expect("commit proposal change");
        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("checkout")
            .arg(branch_manifest.base_branch.as_deref().unwrap_or("master"))
            .status()
            .expect("checkout base branch");
        fs::write(root.join("README.md"), "# demo\n\ndirty overlap\n").expect("write dirty readme");

        assert!(git_worktree_conflicts_with_branch(
            &root,
            branch_manifest.base_branch.as_deref().unwrap_or("master"),
            &proposal.branch
        ));

        fs::remove_dir_all(root).expect("cleanup temp root");
    }

    #[test]
    fn process_evolution_queues_merges_ready_auto_merge_branch() {
        let root = std::env::temp_dir().join(format!(
            "memd-evolution-merge-ready-{}",
            uuid::Uuid::new_v4()
        ));
        let output = root.join(".memd");
        fs::create_dir_all(&output).expect("create temp bundle");
        write_test_bundle_config(&output, "http://127.0.0.1:59999");

        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("init")
            .status()
            .expect("init git repo");
        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("config")
            .arg("user.email")
            .arg("memd@example.com")
            .status()
            .expect("git user email");
        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("config")
            .arg("user.name")
            .arg("memd")
            .status()
            .expect("git user name");
        fs::write(root.join("README.md"), "# demo\n").expect("write readme");
        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("add")
            .arg(".")
            .status()
            .expect("git add");
        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("commit")
            .arg("-m")
            .arg("init")
            .status()
            .expect("git commit");

        let mut report = test_experiment_report(&output, true, false, 100, 100, Utc::now());
        report.improvement.final_changes = vec!["adjust loop threshold floor".to_string()];
        write_experiment_artifacts(&output, &report).expect("write experiment artifacts");

        let proposal: EvolutionProposalReport = serde_json::from_str(
            &fs::read_to_string(output.join("evolution").join("latest-proposal.json"))
                .expect("read proposal"),
        )
        .expect("parse proposal");
        let branch_manifest: EvolutionBranchManifest = serde_json::from_str(
            &fs::read_to_string(output.join("evolution").join("latest-branch.json"))
                .expect("read branch manifest"),
        )
        .expect("parse branch manifest");
        assert_eq!(proposal.authority_tier, "phase1_auto_merge");
        assert!(proposal.merge_eligible);

        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("checkout")
            .arg(&proposal.branch)
            .status()
            .expect("checkout proposal branch");
        fs::write(root.join("README.md"), "# demo\n\nmerged change\n")
            .expect("write branch change");
        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("add")
            .arg("README.md")
            .status()
            .expect("git add readme");
        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("commit")
            .arg("-m")
            .arg("proposal change")
            .status()
            .expect("commit proposal change");
        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("checkout")
            .arg(branch_manifest.base_branch.as_deref().unwrap_or("master"))
            .status()
            .expect("checkout base branch");
        fs::write(root.join("NOTES.txt"), "unrelated dirty file\n")
            .expect("write unrelated dirty file");

        process_evolution_queues(&output).expect("process evolution queues");

        let merge_queue: EvolutionMergeQueue = serde_json::from_str(
            &fs::read_to_string(output.join("evolution").join("merge-queue.json"))
                .expect("read merge queue"),
        )
        .expect("parse merge queue");
        assert_eq!(merge_queue.entries[0].status, "merged");

        let durability_queue: EvolutionDurabilityQueue = serde_json::from_str(
            &fs::read_to_string(output.join("evolution").join("durability-queue.json"))
                .expect("read durability queue"),
        )
        .expect("parse durability queue");
        assert_eq!(durability_queue.entries[0].status, "scheduled");

        let latest_proposal: EvolutionProposalReport = serde_json::from_str(
            &fs::read_to_string(output.join("evolution").join("latest-proposal.json"))
                .expect("read latest proposal"),
        )
        .expect("parse latest proposal");
        assert_eq!(latest_proposal.state, "merged");

        let latest_branch: EvolutionBranchManifest = serde_json::from_str(
            &fs::read_to_string(output.join("evolution").join("latest-branch.json"))
                .expect("read latest branch"),
        )
        .expect("parse latest branch");
        assert_eq!(latest_branch.status, "merged");

        let base_head = git_stdout(
            &root,
            &[
                "rev-parse",
                branch_manifest.base_branch.as_deref().unwrap_or("master"),
            ],
        )
        .expect("read base head");
        let proposal_head =
            git_stdout(&root, &["rev-parse", &proposal.branch]).expect("read proposal head");
        assert_eq!(base_head, proposal_head);

        fs::remove_dir_all(root).expect("cleanup temp root");
    }

    #[test]
    fn process_evolution_queues_promotes_merged_branch_to_durable_truth() {
        let root =
            std::env::temp_dir().join(format!("memd-evolution-durable-{}", uuid::Uuid::new_v4()));
        let output = root.join(".memd");
        fs::create_dir_all(&output).expect("create temp bundle");
        write_test_bundle_config(&output, "http://127.0.0.1:59999");

        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("init")
            .status()
            .expect("init git repo");
        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("config")
            .arg("user.email")
            .arg("memd@example.com")
            .status()
            .expect("git user email");
        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("config")
            .arg("user.name")
            .arg("memd")
            .status()
            .expect("git user name");
        fs::write(root.join("README.md"), "# demo\n").expect("write readme");
        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("add")
            .arg(".")
            .status()
            .expect("git add");
        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("commit")
            .arg("-m")
            .arg("init")
            .status()
            .expect("git commit");

        let mut report = test_experiment_report(&output, true, false, 100, 100, Utc::now());
        report.improvement.final_changes = vec!["adjust loop threshold floor".to_string()];
        write_experiment_artifacts(&output, &report).expect("write experiment artifacts");

        let proposal: EvolutionProposalReport = serde_json::from_str(
            &fs::read_to_string(output.join("evolution").join("latest-proposal.json"))
                .expect("read proposal"),
        )
        .expect("parse proposal");
        let branch_manifest: EvolutionBranchManifest = serde_json::from_str(
            &fs::read_to_string(output.join("evolution").join("latest-branch.json"))
                .expect("read branch manifest"),
        )
        .expect("parse branch manifest");

        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("checkout")
            .arg(&proposal.branch)
            .status()
            .expect("checkout proposal branch");
        fs::write(root.join("README.md"), "# demo\n\ndurable change\n")
            .expect("write branch change");
        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("add")
            .arg("README.md")
            .status()
            .expect("git add readme");
        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("commit")
            .arg("-m")
            .arg("proposal change")
            .status()
            .expect("commit proposal change");
        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("checkout")
            .arg(branch_manifest.base_branch.as_deref().unwrap_or("master"))
            .status()
            .expect("checkout base branch");

        process_evolution_queues(&output).expect("merge proposal");

        let mut durability_queue: EvolutionDurabilityQueue = serde_json::from_str(
            &fs::read_to_string(output.join("evolution").join("durability-queue.json"))
                .expect("read durability queue"),
        )
        .expect("parse durability queue");
        durability_queue.entries[0].due_at = Some(Utc::now() - chrono::TimeDelta::minutes(1));
        write_evolution_durability_queue(&output, &durability_queue)
            .expect("rewrite durability queue");

        process_evolution_queues(&output).expect("process durability");

        let final_queue: EvolutionDurabilityQueue = serde_json::from_str(
            &fs::read_to_string(output.join("evolution").join("durability-queue.json"))
                .expect("read final durability queue"),
        )
        .expect("parse final durability queue");
        assert_eq!(final_queue.entries[0].status, "verified");

        let latest_proposal: EvolutionProposalReport = serde_json::from_str(
            &fs::read_to_string(output.join("evolution").join("latest-proposal.json"))
                .expect("read latest proposal"),
        )
        .expect("parse latest proposal");
        assert_eq!(latest_proposal.state, "durable_truth");
        assert!(latest_proposal.durable_truth);

        let latest_branch: EvolutionBranchManifest = serde_json::from_str(
            &fs::read_to_string(output.join("evolution").join("latest-branch.json"))
                .expect("read latest branch"),
        )
        .expect("parse latest branch");
        assert_eq!(latest_branch.status, "durable_truth");
        assert!(latest_branch.durable_truth);

        fs::remove_dir_all(root).expect("cleanup temp root");
    }

    #[test]
    fn compute_evolution_authority_tier_expands_and_contracts() {
        let output =
            std::env::temp_dir().join(format!("memd-evolution-authority-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(output.join("evolution")).expect("create evolution dir");

        fs::write(
            output.join("evolution").join("authority-ledger.json"),
            serde_json::to_string_pretty(&EvolutionAuthorityLedger {
                entries: vec![
                    EvolutionAuthorityEntry {
                        scope_class: "runtime_policy".to_string(),
                        authority_tier: "phase1_auto_merge".to_string(),
                        accepted: true,
                        merged: true,
                        durable_truth: true,
                        proposal_id: "p3".to_string(),
                        branch: "b3".to_string(),
                        recorded_at: Utc::now(),
                    },
                    EvolutionAuthorityEntry {
                        scope_class: "runtime_policy".to_string(),
                        authority_tier: "phase1_auto_merge".to_string(),
                        accepted: true,
                        merged: true,
                        durable_truth: true,
                        proposal_id: "p2".to_string(),
                        branch: "b2".to_string(),
                        recorded_at: Utc::now(),
                    },
                    EvolutionAuthorityEntry {
                        scope_class: "runtime_policy".to_string(),
                        authority_tier: "phase1_auto_merge".to_string(),
                        accepted: true,
                        merged: true,
                        durable_truth: true,
                        proposal_id: "p1".to_string(),
                        branch: "b1".to_string(),
                        recorded_at: Utc::now(),
                    },
                ],
            })
            .expect("serialize authority"),
        )
        .expect("write authority");
        assert_eq!(
            compute_evolution_authority_tier(&output, "runtime_policy", "auto_merge"),
            "durable_auto_merge"
        );

        fs::write(
            output.join("evolution").join("authority-ledger.json"),
            serde_json::to_string_pretty(&EvolutionAuthorityLedger {
                entries: vec![
                    EvolutionAuthorityEntry {
                        scope_class: "runtime_policy".to_string(),
                        authority_tier: "phase1_auto_merge".to_string(),
                        accepted: false,
                        merged: false,
                        durable_truth: false,
                        proposal_id: "p2".to_string(),
                        branch: "b2".to_string(),
                        recorded_at: Utc::now(),
                    },
                    EvolutionAuthorityEntry {
                        scope_class: "runtime_policy".to_string(),
                        authority_tier: "phase1_auto_merge".to_string(),
                        accepted: true,
                        merged: false,
                        durable_truth: false,
                        proposal_id: "p1".to_string(),
                        branch: "b1".to_string(),
                        recorded_at: Utc::now(),
                    },
                ],
            })
            .expect("serialize authority"),
        )
        .expect("write authority");
        assert_eq!(
            compute_evolution_authority_tier(&output, "runtime_policy", "auto_merge"),
            "proposal_only"
        );

        fs::remove_dir_all(output).expect("cleanup authority output");
    }

    #[test]
    fn classify_evolution_scope_biases_safe_self_evolution_changes_into_auto_merge_lanes() {
        let dir = std::env::temp_dir().join(format!(
            "memd-evolution-classifier-safe-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).expect("create temp bundle");

        let mut report = test_experiment_report(&dir, true, false, 96, 100, Utc::now());
        report.composite.scenario = Some("self_evolution".to_string());
        report.improvement.final_changes =
            vec!["retune pass/fail gate for self-evolution proposals".to_string()];
        let scope = classify_evolution_scope(&report);
        assert_eq!(scope.scope_class, "runtime_policy");
        assert_eq!(scope.scope_gate, "auto_merge");

        report.improvement.final_changes =
            vec!["refine composite dimension calculation for self-evolution".to_string()];
        let scope = classify_evolution_scope(&report);
        assert_eq!(scope.scope_class, "low_risk_evaluation_code");
        assert_eq!(scope.scope_gate, "auto_merge");

        report.improvement.final_changes =
            vec!["adjust acceptance signal aggregation in proposal scoring".to_string()];
        let scope = classify_evolution_scope(&report);
        assert_eq!(scope.scope_class, "low_risk_evaluation_code");
        assert_eq!(scope.scope_gate, "auto_merge");

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn classify_evolution_scope_keeps_high_risk_changes_on_proposal_branch() {
        let dir = std::env::temp_dir().join(format!(
            "memd-evolution-classifier-risky-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).expect("create temp bundle");

        let mut report = test_experiment_report(&dir, true, false, 96, 100, Utc::now());
        report.composite.scenario = Some("self_evolution".to_string());
        report.improvement.final_changes =
            vec!["change hive coordination protocol for task claims".to_string()];
        let scope = classify_evolution_scope(&report);
        assert_eq!(scope.scope_class, "coordination_semantics");
        assert_eq!(scope.scope_gate, "proposal_only");

        report.improvement.final_changes =
            vec!["update storage schema for evolution durability ledger".to_string()];
        let scope = classify_evolution_scope(&report);
        assert_eq!(scope.scope_class, "persistence_semantics");
        assert_eq!(scope.scope_gate, "proposal_only");

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn write_skill_policy_artifacts_writes_batch_and_activate_queue() {
        let dir = std::env::temp_dir().join(format!(
            "memd-skill-policy-artifacts-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).expect("create temp bundle");

        let report = SkillLifecycleReport {
            generated_at: Utc::now(),
            proposed: 2,
            sandbox_passed: 1,
            sandbox_review: 1,
            sandbox_blocked: 0,
            activation_candidates: 1,
            activated: 1,
            review_queue: vec![SkillLifecycleRecord {
                harness: "claude".to_string(),
                name: "review-skill".to_string(),
                kind: "skill".to_string(),
                portability_class: "harness-native".to_string(),
                proposal: "proposed".to_string(),
                sandbox: "review".to_string(),
                sandbox_risk: 0.38,
                sandbox_reason: "harness_native".to_string(),
                activation: "review".to_string(),
                activation_reason: "policy gate requires review before activation".to_string(),
                source_path: "/tmp/review-skill".to_string(),
                target_path: None,
                notes: vec!["note".to_string()],
            }],
            activate_queue: vec![SkillLifecycleRecord {
                harness: "codex".to_string(),
                name: "activate-skill".to_string(),
                kind: "skill".to_string(),
                portability_class: "universal".to_string(),
                proposal: "proposed".to_string(),
                sandbox: "pass".to_string(),
                sandbox_risk: 0.05,
                sandbox_reason: "portable".to_string(),
                activation: "activate".to_string(),
                activation_reason: "low-risk sandbox passed and policy allowed auto-activation"
                    .to_string(),
                source_path: "/tmp/activate-skill".to_string(),
                target_path: Some("/tmp/target".to_string()),
                notes: vec![],
            }],
            records: vec![],
        };
        let response = MemoryPolicyResponse {
            retrieval_order: Vec::new(),
            route_defaults: vec![memd_schema::MemoryPolicyRouteDefault {
                intent: RetrievalIntent::CurrentTask,
                route: RetrievalRoute::Auto,
            }],
            working_memory: memd_schema::MemoryPolicyWorkingMemory {
                budget_chars: 1,
                max_chars_per_item: 1,
                default_limit: 1,
                rehydration_limit: 1,
            },
            retrieval_feedback: memd_schema::MemoryPolicyFeedback {
                enabled: false,
                tracked_surfaces: Vec::new(),
                max_items_per_request: 1,
            },
            source_trust_floor: 0.0,
            runtime: Default::default(),
            promotion: memd_schema::MemoryPolicyPromotion {
                min_salience: 0.0,
                min_events: 0,
                lookback_days: 0,
                default_ttl_days: 0,
            },
            decay: memd_schema::MemoryPolicyDecay {
                max_items: 0,
                inactive_days: 0,
                max_decay: 0.0,
                record_events: false,
            },
            consolidation: memd_schema::MemoryPolicyConsolidation {
                max_groups: 0,
                min_events: 0,
                lookback_days: 0,
                min_salience: 0.0,
                record_events: false,
            },
        };

        let receipt = write_skill_policy_artifacts(&dir, &response, &report, true)
            .expect("write skill policy artifacts");
        assert!(receipt.is_some());

        let batch_json = dir.join("state").join("skill-policy-batch.json");
        let batch_md = dir.join("state").join("skill-policy-batch.md");
        let review_json = dir.join("state").join("skill-policy-review-queue.json");
        let review_md = dir.join("state").join("skill-policy-review-queue.md");
        let activate_json = dir.join("state").join("skill-policy-activate-queue.json");
        let activate_md = dir.join("state").join("skill-policy-activate-queue.md");
        let apply_json = dir.join("state").join("skill-policy-apply-receipt.json");
        let apply_md = dir.join("state").join("skill-policy-apply-receipt.md");
        assert!(batch_json.exists());
        assert!(batch_md.exists());
        assert!(review_json.exists());
        assert!(review_md.exists());
        assert!(activate_json.exists());
        assert!(activate_md.exists());
        assert!(apply_json.exists());
        assert!(apply_md.exists());

        let parsed: SkillPolicyBatchArtifact =
            serde_json::from_str(&fs::read_to_string(&batch_json).expect("read batch json"))
                .expect("parse batch json");
        assert_eq!(parsed.report.proposed, 2);
        assert_eq!(parsed.report.activate_queue.len(), 1);
        assert_eq!(parsed.report.review_queue.len(), 1);

        let activate: SkillPolicyQueueArtifact =
            serde_json::from_str(&fs::read_to_string(&activate_json).expect("read activate json"))
                .expect("parse activate json");
        assert_eq!(activate.queue, "activate");
        assert_eq!(activate.records.len(), 1);

        let apply: SkillPolicyApplyArtifact =
            serde_json::from_str(&fs::read_to_string(&apply_json).expect("read apply json"))
                .expect("parse apply json");
        assert_eq!(apply.applied_count, 1);
        assert_eq!(apply.skipped_count, 0);
        assert_eq!(apply.applied.len(), 1);

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[tokio::test]
    async fn skill_policy_apply_posts_to_server_sink() {
        let base_url = spawn_mock_hive_server().await;
        let client = MemdClient::new(&base_url).expect("build client");
        let response = client
            .record_skill_policy_apply(&SkillPolicyApplyRequest {
                bundle_root: ".memd".to_string(),
                runtime_defaulted: false,
                source_queue_path: ".memd/state/skill-policy-activate-queue.json".to_string(),
                applied_count: 2,
                skipped_count: 1,
                applied: vec![SkillPolicyActivationRecord {
                    harness: "codex".to_string(),
                    name: "skill-a".to_string(),
                    kind: "skill".to_string(),
                    portability_class: "universal".to_string(),
                    proposal: "proposed".to_string(),
                    sandbox: "pass".to_string(),
                    sandbox_risk: 0.05,
                    sandbox_reason: "portable".to_string(),
                    activation: "activate".to_string(),
                    activation_reason: "low-risk sandbox passed and policy allowed auto-activation"
                        .to_string(),
                    source_path: "skill-a".to_string(),
                    target_path: Some("target-a".to_string()),
                    notes: vec!["note".to_string()],
                }],
                skipped: vec![],
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
            })
            .await
            .expect("post apply receipt");
        assert_eq!(response.receipt.applied_count, 2);

        let receipts = client
            .skill_policy_apply_receipts(&SkillPolicyApplyReceiptsRequest {
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                limit: Some(8),
            })
            .await
            .expect("fetch apply receipts");
        assert_eq!(receipts.receipts.len(), 1);
        assert_eq!(receipts.receipts[0].applied_count, 2);

        let activations = client
            .skill_policy_activations(&SkillPolicyActivationEntriesRequest {
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                limit: Some(8),
            })
            .await
            .expect("fetch skill policy activations");
        assert_eq!(activations.activations.len(), 1);
        assert_eq!(activations.activations[0].record.name, "skill-a");
    }

    #[test]
    fn skill_catalog_discovers_project_skill_files() {
        let root = std::env::temp_dir().join(format!("memd-skills-{}", uuid::Uuid::new_v4()));
        let skill_root = root.join("skills").join("project-chat");
        fs::create_dir_all(&skill_root).expect("create skill root");
        fs::write(
            skill_root.join("SKILL.md"),
            r#"---
name: project-chat
description: route project chat through a compact visible lane
---

# Project Chat

Use this skill when the user asks for chat with project memory.
"#,
        )
        .expect("write skill file");

        let catalog = build_skill_catalog(&root.join("skills")).expect("build skill catalog");
        assert!(!catalog.builtins.is_empty());
        assert_eq!(catalog.custom.len(), 1);
        assert_eq!(catalog.custom[0].name, "project-chat");
        assert_eq!(
            catalog.custom[0].summary,
            "route project chat through a compact visible lane"
        );
        assert_eq!(
            catalog.custom[0].usage,
            "edit the file, then propose via skill-policy"
        );
        assert_eq!(
            catalog.custom[0].decision,
            "custom skills stay project-local until promoted by policy"
        );

        fs::remove_dir_all(root).expect("cleanup skill temp dir");
    }

    #[test]
    fn skill_catalog_matches_builtin_and_custom_entries() {
        let root = std::env::temp_dir().join(format!("memd-skills-match-{}", uuid::Uuid::new_v4()));
        let skill_root = root.join("skills").join("project-chat");
        fs::create_dir_all(&skill_root).expect("create skill root");
        fs::write(
            skill_root.join("SKILL.md"),
            r#"---
name: project-chat
description: route project chat through a compact visible lane
---
"#,
        )
        .expect("write skill file");

        let catalog = build_skill_catalog(&root.join("skills")).expect("build skill catalog");
        let builtin_matches = find_skill_catalog_matches(&catalog, "memd init");
        assert!(
            builtin_matches
                .iter()
                .any(|entry| entry.name == "memd-init")
        );

        let custom_matches = find_skill_catalog_matches(&catalog, "compact visible lane");
        assert_eq!(custom_matches.len(), 1);
        assert_eq!(custom_matches[0].name, "project-chat");

        let summary = render_skill_catalog_match_summary(&catalog, "project-chat", &custom_matches);
        assert!(summary.contains("skills query=\"project-chat\""));
        assert!(summary.contains("next=edit the file, then propose via skill-policy"));
        assert!(summary.contains("usage=edit the file, then propose via skill-policy"));
        assert!(
            summary.contains("decision=custom skills stay project-local until promoted by policy")
        );

        fs::remove_dir_all(root).expect("cleanup skill match temp dir");
    }

    #[test]
    fn skill_catalog_reuses_cached_entries_for_unchanged_files() {
        let root = std::env::temp_dir().join(format!("memd-skills-cache-{}", uuid::Uuid::new_v4()));
        let skill_root = root.join("skills").join("project-chat");
        fs::create_dir_all(&skill_root).expect("create skill root");
        fs::write(
            skill_root.join("SKILL.md"),
            r#"---
name: project-chat
description: route project chat through a compact visible lane
---
"#,
        )
        .expect("write skill file");

        let first = build_skill_catalog(&root.join("skills")).expect("build skill catalog first");
        assert_eq!(first.cache_hits, 0);
        assert_eq!(first.cache_scanned, 1);

        let second = build_skill_catalog(&root.join("skills")).expect("build skill catalog second");
        assert_eq!(second.cache_hits, 1);
        assert_eq!(second.cache_scanned, 1);
        assert_eq!(second.custom.len(), 1);
        assert_eq!(second.custom[0].name, "project-chat");

        fs::remove_dir_all(root).expect("cleanup skill cache temp dir");
    }

    #[test]
    fn inspiration_search_reuses_cache_for_unchanged_files() {
        let root = std::env::temp_dir().join(format!("memd-inspiration-{}", uuid::Uuid::new_v4()));
        let lane_dir = root.join(".planning").join("codebase");
        fs::create_dir_all(&lane_dir).expect("create inspiration lane dir");
        fs::write(
            lane_dir.join("INSPIRATION-LANE.md"),
            "# Inspiration Lane\n\nLightRAG keeps the backend swappable.\n",
        )
        .expect("write inspiration lane");

        let first =
            search_inspiration_lane(&root, "LightRAG", 10).expect("first inspiration search");
        assert_eq!(first.hits.len(), 1);
        assert_eq!(first.cache_hits, 0);
        assert_eq!(first.cache_scanned, 1);

        let second =
            search_inspiration_lane(&root, "LightRAG", 10).expect("second inspiration search");
        assert_eq!(second.hits.len(), 1);
        assert_eq!(second.cache_hits, 1);
        assert_eq!(second.cache_scanned, 0);

        let summary = render_inspiration_search_summary(&root, "LightRAG", &second);
        assert!(summary.contains("cache_hits=1"));
        assert!(summary.contains("scanned=0"));

        fs::remove_dir_all(root).expect("cleanup inspiration temp dir");
    }

    #[test]
    fn ui_commands_parse_artifact_and_map_modes() {
        let artifact_id = uuid::Uuid::new_v4().to_string();
        let cli = Cli::parse_from([
            "memd",
            "--base-url",
            "http://localhost:3000",
            "ui",
            "artifact",
            "--id",
            &artifact_id,
            "--json",
            "--follow",
        ]);

        assert_eq!(cli.base_url, "http://localhost:3000");
        match cli.command {
            Commands::Ui(args) => match args.mode {
                UiMode::Artifact(args) => {
                    assert_eq!(args.id, artifact_id);
                    assert!(args.json);
                    assert!(args.follow);
                }
                _ => panic!("expected artifact mode"),
            },
            _ => panic!("expected ui command"),
        }

        let cli = Cli::parse_from(["memd", "ui", "map", "--json"]);
        match cli.command {
            Commands::Ui(args) => match args.mode {
                UiMode::Map(args) => {
                    assert!(args.json);
                    assert!(!args.follow);
                }
                _ => panic!("expected map mode"),
            },
            _ => panic!("expected ui command"),
        }
    }

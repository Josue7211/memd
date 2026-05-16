use super::*;
use serde::Serialize;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct P0FeatureReport {
    pub(crate) generated_at: DateTime<Utc>,
    pub(crate) bundle_root: String,
    pub(crate) status: String,
    pub(crate) features: Vec<P0Feature>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct P0Feature {
    pub(crate) id: String,
    pub(crate) status: String,
    pub(crate) evidence: Vec<String>,
    pub(crate) gaps: Vec<String>,
}

pub(crate) fn run_p0_features_command(args: &FeaturesArgs) -> anyhow::Result<P0FeatureReport> {
    Ok(build_p0_feature_report(&args.output))
}

pub(crate) fn render_p0_feature_summary(report: &P0FeatureReport) -> String {
    let mut counts = BTreeMap::<String, usize>::new();
    for feature in &report.features {
        *counts.entry(feature.status.clone()).or_default() += 1;
    }
    format!(
        "features status={} working={} partial={} broken={} bundle={}",
        report.status,
        counts.get("working").copied().unwrap_or(0),
        counts.get("partial").copied().unwrap_or(0),
        counts.get("broken").copied().unwrap_or(0),
        report.bundle_root
    )
}

fn build_p0_feature_report(output: &Path) -> P0FeatureReport {
    let features = vec![
        native_handoff_feature(output),
        voice_mode_feature(output),
        repo_hygiene_feature(output),
        capability_sync_feature(output),
        token_efficiency_feature(output),
    ];
    let status = if features.iter().any(|feature| feature.status == "broken") {
        "broken"
    } else if features.iter().all(|feature| feature.status == "working") {
        "working"
    } else {
        "partial"
    };
    P0FeatureReport {
        generated_at: Utc::now(),
        bundle_root: output.display().to_string(),
        status: status.to_string(),
        features,
    }
}

fn native_handoff_feature(output: &Path) -> P0Feature {
    let wake_path = output.join("wake.md");
    let mem_path = output.join("mem.md");
    let wake = fs::read_to_string(&wake_path).unwrap_or_default();
    let mem = fs::read_to_string(&mem_path).unwrap_or_default();
    let recovery_line = wake.contains("- recovery voice=");
    let quality_ready = wake.contains("quality=ready:") || mem.contains("quality=ready:");
    let continuity = wake.contains("next=") && wake.contains("blocker=") && wake.contains("dirty=");
    let next_action_content = wake_recovery_next_has_action_content(&wake);
    let status = if recovery_line && quality_ready && continuity && next_action_content {
        "working"
    } else if recovery_line || quality_ready || continuity || next_action_content {
        "partial"
    } else {
        "broken"
    };
    let mut gaps = Vec::new();
    if !recovery_line {
        gaps.push("wake does not expose native recovery line".to_string());
    }
    if !quality_ready {
        gaps.push("handoff quality is not ready in wake/mem surfaces".to_string());
    }
    if !continuity {
        gaps.push("wake does not expose next/blocker/dirty recovery facts".to_string());
    }
    if !next_action_content {
        gaps.push("wake next recovery fact does not expose action content".to_string());
    }
    feature(
        "native_handoff_recovery",
        status,
        vec![
            path_evidence("wake", &wake_path),
            path_evidence("mem", &mem_path),
            format!("wake_recovery_line={recovery_line}"),
            format!("handoff_quality_ready={quality_ready}"),
            format!("native_continuity={continuity}"),
            format!("next_action_content={next_action_content}"),
        ],
        gaps,
    )
}

fn wake_recovery_next_has_action_content(wake: &str) -> bool {
    wake.lines()
        .find(|line| line.starts_with("- recovery "))
        .and_then(|line| line.split("next=").nth(1))
        .is_some_and(|next| {
            let next = next.split(" | blocker=").next().unwrap_or(next).trim();
            next.contains("CURRENT NEXT ACTION")
                || next.contains(": implement ")
                || next.contains(": fix ")
                || next.contains(": triage ")
        })
}

fn voice_mode_feature(output: &Path) -> P0Feature {
    let config_path = output.join("config.json");
    let voice = read_bundle_voice_mode(output).unwrap_or_else(default_voice_mode);
    let valid = normalize_voice_mode_value(&voice).is_ok();
    let config_present = config_path.is_file();
    feature(
        "voice_mode",
        if valid && config_present {
            "working"
        } else {
            "partial"
        },
        vec![
            path_evidence("config", &config_path),
            format!("voice_mode={voice}"),
            format!("valid_voice_mode={valid}"),
            "source_of_truth=.memd/config.json".to_string(),
        ],
        if config_present && valid {
            Vec::new()
        } else {
            vec!["voice mode source of truth is missing or invalid".to_string()]
        },
    )
}

fn repo_hygiene_feature(output: &Path) -> P0Feature {
    let repo_root = infer_bundle_project_root(output)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));
    let (tracked_dirty, untracked) =
        git_dirty_counts(&repo_root).unwrap_or((usize::MAX, usize::MAX));
    let raw_cache_paths = raw_benchmark_cache_paths(&repo_root);
    let mut gaps = Vec::new();
    if tracked_dirty > 5 {
        gaps.push(format!(
            "broad dirty tree: {tracked_dirty} tracked files exceed auto-commit limit 5"
        ));
    }
    if untracked > 50 {
        gaps.push(format!(
            "large untracked surface: {untracked} paths need triage"
        ));
    }
    for path in &raw_cache_paths {
        if path.exists() {
            gaps.push(format!(
                "raw benchmark cache is present in repo-visible path: {}",
                path.display()
            ));
        }
    }
    let mut evidence = vec![
        format!("repo_root={}", repo_root.display()),
        format!("dirty_tracked_files={tracked_dirty}"),
        format!("untracked_paths={untracked}"),
        "auto_commit_max_tracked_files=5".to_string(),
        "implementation work must not run broad benchmarks".to_string(),
    ];
    for path in raw_cache_paths {
        evidence.push(format!(
            "raw_benchmark_cache_path={}:{}",
            if path.exists() { "present" } else { "absent" },
            path.display()
        ));
    }
    feature(
        "repo_hygiene",
        if gaps.is_empty() {
            "working"
        } else {
            "partial"
        },
        evidence,
        gaps,
    )
}

fn capability_sync_feature(output: &Path) -> P0Feature {
    let registry_path = output.join("state").join("capability-registry.json");
    let counts = read_capability_registry_counts(&registry_path).unwrap_or_default();
    let mut gaps = Vec::new();
    if counts.total == 0 {
        gaps.push("capability inventory is missing or empty".to_string());
    }
    if counts.materialization_missing > 0 {
        gaps.push(format!(
            "{} capabilities lack fresh-machine materialization payloads",
            counts.materialization_missing
        ));
    }
    gaps.extend(capability_surface_gaps(&counts));
    if counts.host_cli_assets > counts.host_cli_install_plans {
        gaps.push(format!(
            "{} host CLI records lack server-synced install plans",
            counts.host_cli_assets - counts.host_cli_install_plans
        ));
    }
    if !counts.missing_expected_host_cli_names.is_empty() {
        gaps.push(format!(
            "missing expected host CLI capability records: {}",
            counts.missing_expected_host_cli_names.join(",")
        ));
    }
    gaps.push("fresh-machine materializer is unproven".to_string());
    gaps.push("host-local CLI availability cannot be restored by memd sync alone".to_string());
    feature(
        "capability_sync",
        if counts.total == 0 {
            "broken"
        } else {
            "partial"
        },
        vec![
            path_evidence("capability_registry", &registry_path),
            format!("discovered_capabilities={}", counts.total),
            format!("non_universal_capabilities={}", counts.non_universal),
            format!("materialization_missing={}", counts.materialization_missing),
            format!(
                "materialization_installable={}",
                counts.materialization_installable
            ),
            format!("payload_text_records={}", counts.payload_text_records),
            format!(
                "payload_file_set_records={}",
                counts.payload_file_set_records
            ),
            format!("codex_plugin_assets={}", counts.codex_plugin_assets),
            format!("codex_skill_assets={}", counts.codex_skill_assets),
            format!("claude_code_assets={}", counts.claude_code_assets),
            format!("hermes_assets={}", counts.hermes_assets),
            format!("opencode_assets={}", counts.opencode_assets),
            format!("project_policy_assets={}", counts.project_policy_assets),
            format!("host_cli_assets={}", counts.host_cli_assets),
            format!("host_cli_install_plans={}", counts.host_cli_install_plans),
            format!(
                "expected_host_cli_records={}/{}",
                counts.expected_host_cli_records,
                EXPECTED_HOST_CLIS.len()
            ),
            "server-backed inventory is not the same as installing plugins/skills/CLIs".to_string(),
        ],
        gaps,
    )
}

fn token_efficiency_feature(output: &Path) -> P0Feature {
    let source_registry = output.join("state").join("source-registry.json");
    let raw_spine = output.join("state").join("raw-spine.jsonl");
    let ledger = output.join("state").join("token-savings-ledger.ndjson");
    let counts = read_token_savings_counts(&ledger).unwrap_or_default();
    let state_present = source_registry.is_file() || raw_spine.is_file() || ledger.is_file();
    let measured = counts.events > 0 && counts.tokens_saved > 0;
    let mut gaps = Vec::new();
    if !state_present {
        gaps.push("no token-efficiency state files found".to_string());
    }
    if counts.events == 0 {
        gaps.push("no token-savings ledger events found".to_string());
    } else if counts.tokens_saved == 0 {
        gaps.push("token-savings ledger has no positive measured savings".to_string());
    }
    feature(
        "token_efficiency",
        if measured { "working" } else { "partial" },
        vec![
            path_evidence("source_registry", &source_registry),
            path_evidence("raw_spine", &raw_spine),
            path_evidence("token_savings_ledger", &ledger),
            format!("token_savings_events={}", counts.events),
            format!("measured_tokens_saved={}", counts.tokens_saved),
            "raw benchmark caches must stay outside repo-visible paths".to_string(),
        ],
        gaps,
    )
}

fn feature(id: &str, status: &str, evidence: Vec<String>, gaps: Vec<String>) -> P0Feature {
    P0Feature {
        id: id.to_string(),
        status: status.to_string(),
        evidence,
        gaps,
    }
}

fn path_evidence(label: &str, path: &Path) -> String {
    format!(
        "{label}:{}:{}",
        if path.exists() { "present" } else { "absent" },
        path.display()
    )
}

fn git_dirty_counts(repo_root: &Path) -> anyhow::Result<(usize, usize)> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo_root)
        .output()
        .with_context(|| format!("git status in {}", repo_root.display()))?;
    if !output.status.success() {
        anyhow::bail!("git status failed in {}", repo_root.display());
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut tracked = 0;
    let mut untracked = 0;
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        if line.starts_with("??") {
            untracked += 1;
        } else {
            tracked += 1;
        }
    }
    Ok((tracked, untracked))
}

fn raw_benchmark_cache_paths(repo_root: &Path) -> Vec<PathBuf> {
    vec![
        repo_root.join("external-public-cache"),
        repo_root
            .join("docs")
            .join("verification")
            .join("25-5-memory-os-runs")
            .join("external-public-cache"),
        repo_root
            .join("docs")
            .join("verification")
            .join("25-5-memory-os-runs")
            .join("promptwall-cache"),
    ]
}

#[derive(Debug, Default)]
struct CapabilityAuditCounts {
    total: usize,
    non_universal: usize,
    materialization_installable: usize,
    materialization_missing: usize,
    payload_text_records: usize,
    payload_file_set_records: usize,
    codex_plugin_assets: usize,
    codex_skill_assets: usize,
    claude_code_assets: usize,
    hermes_assets: usize,
    opencode_assets: usize,
    project_policy_assets: usize,
    host_cli_assets: usize,
    host_cli_install_plans: usize,
    expected_host_cli_records: usize,
    missing_expected_host_cli_names: Vec<String>,
}

#[derive(Debug, Default)]
struct TokenSavingsCounts {
    events: usize,
    tokens_saved: u64,
}

fn read_token_savings_counts(path: &Path) -> anyhow::Result<TokenSavingsCounts> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut counts = TokenSavingsCounts::default();
    for line in raw.lines().filter(|line| !line.trim().is_empty()) {
        let value = serde_json::from_str::<serde_json::Value>(line)
            .with_context(|| format!("parse token savings ledger {}", path.display()))?;
        counts.events += 1;
        counts.tokens_saved += value
            .get("tokens_saved")
            .and_then(|value| value.as_u64())
            .unwrap_or(0);
    }
    Ok(counts)
}

fn read_capability_registry_counts(path: &Path) -> anyhow::Result<CapabilityAuditCounts> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let value: serde_json::Value =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    let Some(records) = value.get("capabilities").and_then(|value| value.as_array()) else {
        return Ok(CapabilityAuditCounts::default());
    };
    let mut counts = CapabilityAuditCounts {
        total: records.len(),
        ..CapabilityAuditCounts::default()
    };
    let mut host_cli_names = std::collections::BTreeSet::<String>::new();
    for record in records {
        let portability = record
            .get("portability_class")
            .and_then(|value| value.as_str())
            .unwrap_or_default();
        let harness = record
            .get("harness")
            .and_then(|value| value.as_str())
            .unwrap_or_default();
        let kind = record
            .get("kind")
            .and_then(|value| value.as_str())
            .unwrap_or_default();
        let source_path = record
            .get("source_path")
            .and_then(|value| value.as_str())
            .unwrap_or_default();
        let has_payload = record
            .get("notes")
            .and_then(|value| value.as_array())
            .is_some_and(|notes| {
                notes.iter().any(|note| {
                    note.as_str()
                        .is_some_and(|text| text.starts_with("memd:payload-text:"))
                })
            });
        let has_payload_file_set = record
            .get("notes")
            .and_then(|value| value.as_array())
            .is_some_and(|notes| {
                notes.iter().any(|note| {
                    note.as_str()
                        .is_some_and(|text| text.starts_with("memd:payload-file-json:"))
                })
            });
        let has_host_cli_install_plan = record
            .get("notes")
            .and_then(|value| value.as_array())
            .is_some_and(|notes| {
                notes.iter().any(|note| {
                    note.as_str()
                        .is_some_and(|text| text.starts_with("memd:host-cli-install-plan:"))
                })
            });
        if portability != "universal" {
            counts.non_universal += 1;
        }
        if has_payload {
            counts.payload_text_records += 1;
        }
        if has_payload_file_set {
            counts.payload_file_set_records += 1;
        }
        match (harness, kind) {
            ("codex", value) if value.contains("plugin") => counts.codex_plugin_assets += 1,
            ("codex", "skill") => counts.codex_skill_assets += 1,
            (value, _) if value.contains("claude") => counts.claude_code_assets += 1,
            ("hermes", _) => counts.hermes_assets += 1,
            ("opencode", _) => counts.opencode_assets += 1,
            ("project", "policy") => counts.project_policy_assets += 1,
            ("local", "cli") | (_, "cli") => {
                counts.host_cli_assets += 1;
                host_cli_names.insert(
                    record
                        .get("name")
                        .and_then(|value| value.as_str())
                        .unwrap_or_default()
                        .to_string(),
                );
                if has_host_cli_install_plan {
                    counts.host_cli_install_plans += 1;
                }
            }
            _ => {}
        }
        if has_payload
            || has_payload_file_set
            || capability_has_fresh_machine_payload(harness, kind, portability, source_path)
        {
            counts.materialization_installable += 1;
        } else {
            counts.materialization_missing += 1;
        }
    }
    counts.missing_expected_host_cli_names = EXPECTED_HOST_CLIS
        .iter()
        .copied()
        .filter(|name| !host_cli_names.contains(*name))
        .map(str::to_string)
        .collect();
    counts.expected_host_cli_records =
        EXPECTED_HOST_CLIS.len() - counts.missing_expected_host_cli_names.len();
    Ok(counts)
}

const EXPECTED_HOST_CLIS: &[&str] = &["codex", "gh", "opencode", "claude", "wrangler", "supabase"];

fn capability_has_fresh_machine_payload(
    harness: &str,
    kind: &str,
    portability: &str,
    source_path: &str,
) -> bool {
    let bundle_relative = source_path.starts_with(".memd/")
        || source_path.starts_with("agents/")
        || (harness == "project" && portability == "universal" && !source_path.starts_with('/'));
    bundle_relative
        && portability != "host-local"
        && kind != "cli"
        && !(harness == "codex" && kind.contains("plugin"))
        && !harness.contains("claude")
}

fn capability_surface_gaps(counts: &CapabilityAuditCounts) -> Vec<String> {
    let mut gaps = Vec::new();
    if counts.codex_plugin_assets == 0 {
        gaps.push("missing Codex plugin/skill capability records".to_string());
    }
    if counts.codex_skill_assets == 0 {
        gaps.push("missing Codex skill capability records".to_string());
    }
    if counts.claude_code_assets == 0 {
        gaps.push("missing Claude Code capability records".to_string());
    }
    if counts.hermes_assets == 0 {
        gaps.push("missing Hermes capability records".to_string());
    }
    if counts.opencode_assets == 0 {
        gaps.push("missing OpenCode capability records".to_string());
    }
    if counts.project_policy_assets == 0 {
        gaps.push("missing project policy capability records".to_string());
    }
    if counts.host_cli_assets == 0 {
        gaps.push("missing host CLI capability records".to_string());
    }
    gaps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_hygiene_marks_broad_dirty_tree_partial() {
        let repo =
            std::env::temp_dir().join(format!("memd-p0-feature-repo-{}", uuid::Uuid::new_v4()));
        let bundle = repo.join(".memd");
        fs::create_dir_all(&bundle).expect("create bundle");
        assert!(
            Command::new("git")
                .arg("init")
                .current_dir(&repo)
                .status()
                .unwrap()
                .success()
        );
        assert!(
            Command::new("git")
                .args(["config", "user.email", "memd@example.invalid"])
                .current_dir(&repo)
                .status()
                .unwrap()
                .success()
        );
        assert!(
            Command::new("git")
                .args(["config", "user.name", "memd"])
                .current_dir(&repo)
                .status()
                .unwrap()
                .success()
        );
        for index in 0..6 {
            fs::write(repo.join(format!("file-{index}.txt")), "before\n").unwrap();
        }
        assert!(
            Command::new("git")
                .args(["add", "."])
                .current_dir(&repo)
                .status()
                .unwrap()
                .success()
        );
        assert!(
            Command::new("git")
                .args(["commit", "-m", "seed"])
                .current_dir(&repo)
                .status()
                .unwrap()
                .success()
        );
        for index in 0..6 {
            fs::write(repo.join(format!("file-{index}.txt")), "after\n").unwrap();
        }

        let report = build_p0_feature_report(&bundle);
        let repo_feature = report
            .features
            .iter()
            .find(|feature| feature.id == "repo_hygiene")
            .expect("repo hygiene");
        assert_eq!(repo_feature.status, "partial");
        assert!(
            repo_feature
                .gaps
                .iter()
                .any(|gap| gap.contains("broad dirty tree"))
        );

        fs::remove_dir_all(repo).ok();
    }

    #[test]
    fn repo_hygiene_audits_promptwall_cache_path() {
        let repo = std::env::temp_dir().join(format!(
            "memd-p0-feature-promptwall-cache-{}",
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
    fn native_handoff_requires_next_action_content() {
        let bundle =
            std::env::temp_dir().join(format!("memd-p0-feature-handoff-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&bundle).expect("create bundle");
        fs::write(
            bundle.join("wake.md"),
            "- recovery voice=caveman-ultra | quality=ready:0.96 | dirty=1 | next=id=abc | blocker=none\n",
        )
        .expect("write metadata-only wake");
        fs::write(bundle.join("mem.md"), "quality=ready:0.96").expect("write mem");

        let feature = native_handoff_feature(&bundle);
        assert_eq!(feature.status, "partial");
        assert!(
            feature
                .gaps
                .iter()
                .any(|gap| gap.contains("does not expose action content"))
        );

        fs::write(
            bundle.join("wake.md"),
            "- recovery voice=caveman-ultra | quality=ready:0.96 | dirty=1 | next=abc: CURRENT NEXT ACTION: implement materializer | blocker=none\n",
        )
        .expect("write content wake");
        let feature = native_handoff_feature(&bundle);
        assert_eq!(feature.status, "working");
        assert!(
            feature
                .evidence
                .iter()
                .any(|line| line == "next_action_content=true")
        );

        fs::remove_dir_all(bundle).ok();
    }

    #[test]
    fn capability_sync_stays_partial_without_materializer() {
        let bundle = std::env::temp_dir().join(format!(
            "memd-p0-feature-capability-{}",
            uuid::Uuid::new_v4()
        ));
        let state = bundle.join("state");
        fs::create_dir_all(&state).expect("create state");
        fs::write(
            state.join("capability-registry.json"),
            r#"{"capabilities":[
                {
                    "harness":"codex",
                    "kind":"plugin-skill",
                    "portability_class":"harness-native",
                    "source_path":"/tmp/existing-but-not-fresh-machine-payload"
                },
                {
                    "harness":"hermes",
                    "kind":"harness-pack",
                    "portability_class":"universal",
                    "source_path":".memd/agents/hermes.sh",
                    "notes":["memd:payload-file-json:{\"path\":\"SKILL.md\",\"content\":\"# Hermes\\n\"}"]
                }
            ]}"#,
        )
        .expect("write registry");

        let report = build_p0_feature_report(&bundle);
        let feature = report
            .features
            .iter()
            .find(|feature| feature.id == "capability_sync")
            .expect("capability sync");
        assert_eq!(feature.status, "partial");
        assert!(feature.gaps.iter().any(|gap| gap.contains("materializer")));
        assert!(
            feature
                .gaps
                .iter()
                .any(|gap| gap.contains("lack fresh-machine materialization payloads"))
        );
        assert!(
            feature
                .evidence
                .iter()
                .any(|line| line == "materialization_missing=1")
        );
        assert!(
            feature
                .evidence
                .iter()
                .any(|line| line == "materialization_installable=1")
        );
        assert!(
            feature
                .evidence
                .iter()
                .any(|line| line == "payload_file_set_records=1")
        );

        fs::remove_dir_all(bundle).ok();
    }

    #[test]
    fn capability_sync_requires_all_cross_machine_surfaces() {
        let bundle =
            std::env::temp_dir().join(format!("memd-p0-feature-surfaces-{}", uuid::Uuid::new_v4()));
        let state = bundle.join("state");
        fs::create_dir_all(&state).expect("create state");
        fs::write(
            state.join("capability-registry.json"),
            r#"{"capabilities":[
                {
                    "harness":"codex",
                    "kind":"plugin",
                    "portability_class":"harness-native",
                    "source_path":"/tmp/missing-plugin"
                }
            ]}"#,
        )
        .expect("write sparse registry");

        let feature = capability_sync_feature(&bundle);
        for expected in [
            "missing Codex skill capability records",
            "missing Claude Code capability records",
            "missing Hermes capability records",
            "missing OpenCode capability records",
            "missing project policy capability records",
            "missing host CLI capability records",
        ] {
            assert!(
                feature.gaps.iter().any(|gap| gap == expected),
                "missing expected gap: {expected}"
            );
        }
        assert!(
            feature
                .evidence
                .iter()
                .any(|line| line == "codex_plugin_assets=1")
        );
        assert!(
            feature
                .evidence
                .iter()
                .any(|line| line == "claude_code_assets=0")
        );

        fs::remove_dir_all(bundle).ok();
    }

    #[test]
    fn capability_sync_reports_host_cli_install_plans_without_calling_them_payloads() {
        let bundle = std::env::temp_dir().join(format!(
            "memd-p0-feature-host-cli-plan-{}",
            uuid::Uuid::new_v4()
        ));
        let state = bundle.join("state");
        fs::create_dir_all(&state).expect("create state");
        fs::write(
            state.join("capability-registry.json"),
            r#"{"capabilities":[
                {
                    "harness":"local",
                    "kind":"cli",
                    "name":"gh",
                    "portability_class":"host-local",
                    "source_path":"/usr/local/bin/gh",
                    "notes":["memd:host-cli-install-plan:#!/bin/sh\necho install gh\n"]
                }
            ]}"#,
        )
        .expect("write registry");

        let feature = capability_sync_feature(&bundle);

        assert_eq!(feature.status, "partial");
        assert!(
            feature
                .evidence
                .iter()
                .any(|line| line == "host_cli_assets=1")
        );
        assert!(
            feature
                .evidence
                .iter()
                .any(|line| line == "host_cli_install_plans=1")
        );
        assert!(
            feature
                .evidence
                .iter()
                .any(|line| line == "expected_host_cli_records=1/6")
        );
        assert!(
            feature
                .evidence
                .iter()
                .any(|line| line == "materialization_missing=1")
        );
        assert!(
            feature
                .gaps
                .iter()
                .any(|gap| gap
                    == "host-local CLI availability cannot be restored by memd sync alone")
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

        fs::remove_dir_all(bundle).ok();
    }

    #[test]
    fn token_efficiency_requires_measured_savings() {
        let bundle =
            std::env::temp_dir().join(format!("memd-p0-feature-token-{}", uuid::Uuid::new_v4()));
        let state = bundle.join("state");
        fs::create_dir_all(&state).expect("create state");
        fs::write(
            state.join("token-savings-ledger.ndjson"),
            r#"{"operation":"context_packet","tokens_saved":0}"#,
        )
        .expect("write ledger");

        let report = build_p0_feature_report(&bundle);
        let feature = report
            .features
            .iter()
            .find(|feature| feature.id == "token_efficiency")
            .expect("token efficiency");
        assert_eq!(feature.status, "partial");
        assert!(
            feature
                .gaps
                .iter()
                .any(|gap| gap.contains("no positive measured savings"))
        );

        fs::write(
            state.join("token-savings-ledger.ndjson"),
            r#"{"operation":"context_packet","tokens_saved":12}"#,
        )
        .expect("write positive ledger");

        let report = build_p0_feature_report(&bundle);
        let feature = report
            .features
            .iter()
            .find(|feature| feature.id == "token_efficiency")
            .expect("token efficiency");
        assert_eq!(feature.status, "working");
        assert!(
            feature
                .evidence
                .iter()
                .any(|line| line == "measured_tokens_saved=12")
        );

        fs::remove_dir_all(bundle).ok();
    }
}

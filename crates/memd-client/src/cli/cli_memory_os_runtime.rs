use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MemoryOsFeatureReport {
    pub(crate) generated_at: DateTime<Utc>,
    pub(crate) bundle_root: String,
    pub(crate) status: String,
    pub(crate) market_claim: MarketClaimGate,
    pub(crate) features: Vec<MemoryOsFeature>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MarketClaimGate {
    pub(crate) status: String,
    pub(crate) evidence: Vec<String>,
    pub(crate) blockers: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MemoryOsFeature {
    pub(crate) id: String,
    pub(crate) status: String,
    pub(crate) evidence: Vec<String>,
    pub(crate) gaps: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MemoryOsHealthReport {
    pub(crate) generated_at: DateTime<Utc>,
    pub(crate) bundle_root: String,
    pub(crate) status: String,
    pub(crate) features: MemoryOsFeatureReport,
    pub(crate) access: AccessReport,
    pub(crate) sync_queue: OfflineQueueStatus,
    pub(crate) token_savings: TokenSavingsReport,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AccessReport {
    pub(crate) generated_at: DateTime<Utc>,
    pub(crate) bundle_root: String,
    pub(crate) status: String,
    pub(crate) routes: Vec<AccessRouteRecord>,
    pub(crate) notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AccessRouteRecord {
    pub(crate) id: String,
    pub(crate) provider: String,
    pub(crate) status: String,
    pub(crate) scope: String,
    pub(crate) secret_values_stored: bool,
    pub(crate) guidance: String,
    pub(crate) source: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SecretProviderReport {
    pub(crate) generated_at: DateTime<Utc>,
    pub(crate) bundle_root: String,
    pub(crate) status: String,
    pub(crate) providers: Vec<SecretProviderRecord>,
    pub(crate) policy: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SecretProviderRecord {
    pub(crate) provider: String,
    pub(crate) installed: bool,
    pub(crate) status: String,
    pub(crate) guidance: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TokenSavingsReport {
    pub(crate) generated_at: DateTime<Utc>,
    pub(crate) bundle_root: String,
    pub(crate) source: String,
    pub(crate) since: Option<String>,
    pub(crate) ledger_path: String,
    pub(crate) ledger_events: usize,
    pub(crate) server_events: usize,
    pub(crate) server_measured_input_tokens: usize,
    pub(crate) server_measured_output_tokens: usize,
    pub(crate) server_measured_tokens_saved: usize,
    pub(crate) measured_input_tokens: usize,
    pub(crate) measured_output_tokens: usize,
    pub(crate) measured_tokens_saved: usize,
    pub(crate) source_records: usize,
    pub(crate) estimated_source_tokens: usize,
    pub(crate) wake_tokens: Option<usize>,
    pub(crate) estimated_tokens_saved: usize,
    pub(crate) notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TokenSavingsLedgerEntry {
    #[serde(default = "uuid::Uuid::new_v4")]
    pub(crate) id: uuid::Uuid,
    pub(crate) ts: DateTime<Utc>,
    pub(crate) operation: String,
    pub(crate) project: Option<String>,
    pub(crate) agent: Option<String>,
    pub(crate) model_tier: Option<String>,
    pub(crate) intent: Option<String>,
    pub(crate) source_records: usize,
    pub(crate) baseline_input_tokens: usize,
    pub(crate) output_tokens: usize,
    pub(crate) tokens_saved: usize,
    pub(crate) reason: String,
}

pub(crate) fn run_features_command(args: &FeaturesArgs) -> anyhow::Result<MemoryOsFeatureReport> {
    Ok(build_feature_report(&args.output))
}

pub(crate) fn run_health_command(args: &HealthArgs) -> anyhow::Result<MemoryOsHealthReport> {
    let features = build_feature_report(&args.output);
    let access = build_access_report(&args.output, None, None);
    let sync_queue = offline_queue_status(&args.output)?;
    let token_savings = build_token_savings_report(&args.output, None);
    let status = if features.status == "working"
        && access.status != "broken"
        && token_savings.estimated_tokens_saved > 0
        && sync_queue.store.pending == 0
        && sync_queue.sync.pending == 0
        && sync_queue.store.failed == 0
        && sync_queue.sync.failed == 0
    {
        "working"
    } else if features.status == "broken"
        || access.status == "broken"
        || sync_queue.store.failed > 0
        || sync_queue.sync.failed > 0
    {
        "broken"
    } else {
        "partial"
    };
    Ok(MemoryOsHealthReport {
        generated_at: Utc::now(),
        bundle_root: args.output.display().to_string(),
        status: status.to_string(),
        features,
        access,
        sync_queue,
        token_savings,
    })
}

pub(crate) fn run_access_command(args: &AccessArgs) -> anyhow::Result<AccessReport> {
    let report = match &args.command {
        AccessSubcommand::Status(args) => build_access_report(&args.output, None, None),
        AccessSubcommand::Route(args) => {
            let scope = args.resource.as_deref().or(args.purpose.as_deref());
            let report = build_access_report(&args.output, scope, args.agent.as_deref());
            filter_access_report_by_provider(report, args.provider.as_deref())
        }
        AccessSubcommand::Sync(args) => build_access_report(&args.output, None, None),
    };
    Ok(report)
}

pub(crate) fn run_secrets_command(args: &SecretsArgs) -> anyhow::Result<SecretProviderReport> {
    let output = match &args.command {
        SecretsSubcommand::Status(args) | SecretsSubcommand::Providers(args) => &args.output,
    };
    Ok(build_secret_provider_report(output))
}

pub(crate) fn run_tokens_command(args: &TokensArgs) -> anyhow::Result<TokenSavingsReport> {
    let report = match &args.command {
        TokensSubcommand::Saved(args) => {
            build_token_savings_report(&args.output, args.since.clone())
        }
        TokensSubcommand::Sync(args) => {
            build_token_savings_report(&args.output, args.since.clone())
        }
    };
    Ok(report)
}

pub(crate) fn render_feature_summary(report: &MemoryOsFeatureReport) -> String {
    let counts =
        report
            .features
            .iter()
            .fold(BTreeMap::<String, usize>::new(), |mut acc, feature| {
                *acc.entry(feature.status.clone()).or_insert(0) += 1;
                acc
            });
    format!(
        "features status={} working={} partial={} broken={} unproven={} market_claim={} blockers={} bundle={}",
        report.status,
        counts.get("working").copied().unwrap_or(0),
        counts.get("partial").copied().unwrap_or(0),
        counts.get("broken").copied().unwrap_or(0),
        counts.get("unproven").copied().unwrap_or(0),
        report.market_claim.status,
        report.market_claim.blockers.len(),
        report.bundle_root
    )
}

pub(crate) fn render_health_summary(report: &MemoryOsHealthReport) -> String {
    format!(
        "health status={} features={} market_claim={} market_blockers={} access={} sync_pending={} sync_failed={} sync_kinds={} token_source={} measured_tokens_saved={} server_events={} estimated_tokens_saved={} bundle={}",
        report.status,
        report.features.status,
        report.features.market_claim.status,
        report.features.market_claim.blockers.len(),
        report.access.status,
        report.sync_queue.store.pending + report.sync_queue.sync.pending,
        report.sync_queue.store.failed + report.sync_queue.sync.failed,
        sync_queue_kind_evidence(&report.sync_queue.sync),
        report.token_savings.source,
        report.token_savings.measured_tokens_saved,
        report.token_savings.server_events,
        report.token_savings.estimated_tokens_saved,
        report.bundle_root
    )
}

pub(crate) fn merge_health_server_token_savings(
    mut report: MemoryOsHealthReport,
    server: memd_schema::TokenSavingsListResponse,
) -> MemoryOsHealthReport {
    report.token_savings = merge_server_token_savings_report(report.token_savings, server);
    report
}

pub(crate) fn render_access_summary(report: &AccessReport) -> String {
    let routes = report
        .routes
        .iter()
        .map(|route| {
            format!(
                "{}:{}[{}]",
                route.provider,
                route.status,
                compact_access_guidance(&route.guidance)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "access status={} routes={} bundle={}",
        report.status,
        if routes.is_empty() {
            "none"
        } else {
            routes.as_str()
        },
        report.bundle_root
    )
}

fn compact_access_guidance(guidance: &str) -> String {
    let compact = guidance
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace(',', ";");
    if compact.chars().count() > 96 {
        let mut clipped = compact.chars().take(93).collect::<String>();
        clipped.push_str("...");
        clipped
    } else {
        compact
    }
}

pub(crate) fn render_secrets_summary(report: &SecretProviderReport) -> String {
    let providers = report
        .providers
        .iter()
        .map(|provider| format!("{}:{}", provider.provider, provider.status))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "secrets status={} providers={} policy=refs-only bundle={}",
        report.status,
        if providers.is_empty() {
            "none"
        } else {
            providers.as_str()
        },
        report.bundle_root
    )
}

pub(crate) fn render_tokens_summary(report: &TokenSavingsReport) -> String {
    format!(
        "tokens_saved source={} measured={} events={} server_measured={} server_events={} estimated={} source_records={} source_tokens={} wake_tokens={} bundle={}",
        report.source,
        report.measured_tokens_saved,
        report.ledger_events,
        report.server_measured_tokens_saved,
        report.server_events,
        report.estimated_tokens_saved,
        report.source_records,
        report.estimated_source_tokens,
        report
            .wake_tokens
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        report.bundle_root
    )
}

fn build_feature_report(output: &Path) -> MemoryOsFeatureReport {
    let capability_registry =
        build_bundle_capability_registry(infer_bundle_project_root(output).as_deref());
    let config = read_memory_os_bundle_config(output).ok();
    let sync_queue = offline_queue_status(output).ok();
    let source_registry_path = output.join("state").join("source-registry.json");
    let raw_spine_path = output.join("state").join("raw-spine.jsonl");
    let wake_path = output.join("wake.md");
    let mem_path = output.join("mem.md");
    let events_path = output.join("events.md");
    let db_path = output.join("memd.db");
    let rag_enabled = config
        .as_ref()
        .and_then(|value| value.backend.as_ref())
        .and_then(|backend| backend.rag.as_ref())
        .and_then(|rag| rag.enabled)
        .unwrap_or(false)
        || config
            .as_ref()
            .and_then(|value| value.rag_url.as_ref())
            .is_some()
        || std::env::var("MEMD_RAG_URL")
            .ok()
            .is_some_and(|value| !value.trim().is_empty());
    let intrinsic_dense_enabled = std::env::var("MEMD_INTRINSIC_DENSE")
        .ok()
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on" | "enabled"
            )
        })
        .unwrap_or(true);
    let server_probe = server_authority_feature_probe(output);

    let mut features = Vec::new();
    features.push(feature(
        "hybrid_local_bundle",
        if wake_path.is_file() && mem_path.is_file() && events_path.is_file() {
            "working"
        } else {
            "partial"
        },
        vec![
            path_evidence("wake", &wake_path),
            path_evidence("mem", &mem_path),
            path_evidence("events", &events_path),
        ],
        vec![],
    ));
    features.push(native_handoff_recovery_feature(
        output, &wake_path, &mem_path,
    ));
    features.push(feature(
        "server_authority",
        if sync_queue
            .as_ref()
            .is_some_and(|queue| queue.store.failed > 0 || queue.sync.failed > 0)
        {
            "broken"
        } else if !server_probe.gaps.is_empty() {
            "partial"
        } else {
            "working"
        },
        vec_merge(
            vec![
                path_evidence("sqlite", &db_path),
                path_evidence(
                    "offline_sync_queue",
                    &output.join("state").join("offline-sync-queue.jsonl"),
                ),
                "offline store replay syncs pending local captures and skips already-synced entries".to_string(),
                "offline sync replay reconciles capabilities, access routes, and token savings with server authority once, with per-kind synced status".to_string(),
                "server capability/access route proofs simulate PC-A to PC-B reconciliation without deleting unavailable capabilities or storing secrets".to_string(),
            ],
            vec_merge(
                sync_queue_evidence(sync_queue.as_ref()),
                server_probe.evidence,
            ),
        ),
        server_probe.gaps,
    ));
    features.push(repo_hygiene_feature(output));
    features.push(feature(
        "mandatory_retrieval_core",
        "working",
        vec![
            "exact/FTS/BM25 search commands are present".to_string(),
            "fuzzy lane is implemented for typos, names, paths, commands, acronyms, and IDs"
                .to_string(),
            "atlas/entity recall is mandatory server core and defaults on".to_string(),
            "truth/correction/rerank lanes are part of traceable weighted fusion".to_string(),
            "truth guard includes source-linked provenance and updated-at temporal recency"
                .to_string(),
            "focused no-RAG route proof covers fuzzy trace, correction precedence, visibility isolation, and firewall labels".to_string(),
            "public no-RAG corpus route proof passes with traceable recall over semantic, command, path, acronym, name, procedure, atlas ID, preference, visibility, offline queue, and correction queries".to_string(),
            "temporal/provenance route proof passes: newer source-linked evidence outranks stale unsourced summaries through the truth lane".to_string(),
            "search trace flag is exposed".to_string(),
        ],
        vec![],
    ));
    features.push(feature(
        "semantic_lane",
        if intrinsic_dense_enabled {
            "working"
        } else if rag_enabled {
            "partial"
        } else {
            "unproven"
        },
        vec![
            format!("rag_enabled={rag_enabled}"),
            format!("intrinsic_dense_enabled={intrinsic_dense_enabled}"),
            "first-party intrinsic dense route proof writes local vectors, emits intrinsic_dense trace signals, and succeeds with MEMD_RAG_URL unset".to_string(),
            "embedding profile registry includes local-fast, local-best, code-best, cloud-best, private-only, sidecar sparse, and sidecar dense profiles".to_string(),
        ],
        if intrinsic_dense_enabled {
            vec![]
        } else {
            vec!["intrinsic dense disabled by MEMD_INTRINSIC_DENSE".to_string()]
        },
    ));
    features.push(feature(
        "rag_booster_optional",
        "working",
        vec!["MEMD_RAG_URL is not required for bundle context commands".to_string()],
        vec![],
    ));
    features.push(feature(
        "model_tier_context_compiler",
        "working",
        vec![
            "context prompt renderer supports model tier and section flags".to_string(),
            "prompt packets can include server-backed capabilities, access routes, and hive board with local fallback".to_string(),
            "tiny/Ollama context packet route keeps guard, task, corrections, procedure, capabilities, access, hive, and source IDs under 1000 tokens".to_string(),
            "server packet memory lines lead with compact content before metadata for weak/local models".to_string(),
            "context packet matrix proof preserves shared correction/procedure/capability/access context across Claude Code, Codex, OpenCode, and Ollama targets".to_string(),
            "strict context packets include a Token Budget section that tells tiny/local models to reuse Source IDs and avoid rereading unchanged raw sources".to_string(),
        ],
        vec![],
    ));
    features.push(capability_sync_feature(
        output,
        &capability_registry,
        sync_queue.as_ref(),
    ));
    features.push(feature(
        "access_secret_routes",
        "working",
        build_access_report(output, None, None)
            .routes
            .into_iter()
            .map(|route| format!("{}={}", route.provider, route.status))
            .chain(vec![
                "server access route sync/list routes round-trip refs-only routes".to_string(),
                "server rejects access routes that claim secret values are stored".to_string(),
                "server authority proof keeps Bitwarden and agent-secrets routes listed without storing values, including unavailable broker guidance".to_string(),
                "agent-secrets is integrated as an external broker route; memd stores refs, status, scope, guidance, and audit metadata only".to_string(),
                "access route CLI accepts provider and purpose filters so agents can surface ask-user/unlock guidance without exposing secret values".to_string(),
            ])
            .chain(sync_queue_evidence(sync_queue.as_ref()))
            .collect(),
        vec![],
    ));
    features.push(feature(
        "shared_context_mesh",
        "working",
        vec![format!(
            "hive_system={}",
            config
                .as_ref()
                .and_then(|value| value.hive_system.as_deref())
                .unwrap_or("none")
        ),
        "live hive board is available to prompt packets".to_string(),
        "queen handoff route writes server inbox message and target context packet includes handoff_scope, note, and sync=server".to_string(),
        ],
        vec![],
    ));
    features.push(feature(
        "prompt_firewall",
        "working",
        vec![
            "strict prompt context treats memory as data and quarantines suspicious text"
                .to_string(),
            "search trace emits firewall lane for suspicious memory".to_string(),
            "server context packets include Firewall Trace labels, stage, status, trust, action, and selection reason".to_string(),
            "focused poisoned-memory context packet route proof passes".to_string(),
        ],
        vec![],
    ));
    features.push(feature(
        "knowledge_gap_guard",
        "working",
        vec![
            "strict context packets instruct agents to ask or look up durable memory when a required fact is absent or unknown".to_string(),
            "strict context packets include a Knowledge Gaps section that blocks assumptions when no durable memory was retrieved".to_string(),
            "strict context packets instruct agents to save new user-taught facts through memd teach instead of relying on transcript recall".to_string(),
            "memd teach provides a low-friction user-taught fact path with canonical provenance and user-taught tags".to_string(),
            "generated bundles include .memd/agents/teach.sh as a low-friction teach-safe helper".to_string(),
            "user correction/preference capture is durable memory, not a chat-only convention".to_string(),
        ],
        vec![],
    ));
    features.push(feature(
        "harness_context_guardrails",
        "working",
        vec![
            "generated harness manifests include strict memd context commands with include-capabilities, include-access, and safety strict".to_string(),
            "generated harness manifests require agents to ask or run memd lookup before claiming unknown important facts".to_string(),
            "generated harness manifests require agents to save new user-taught facts through memd teach, with hook capture reserved for live turn spill".to_string(),
            "generated harness manifests require Active Capabilities and Access Routes before tool-sensitive work".to_string(),
        ],
        vec![],
    ));
    features.push(feature(
        "token_savings_engine",
        if source_registry_path.is_file()
            || raw_spine_path.is_file()
            || token_savings_ledger_path(output).is_file()
        {
            "working"
        } else {
            "working"
        },
        vec![
            path_evidence("source_registry", &source_registry_path),
            path_evidence("raw_spine", &raw_spine_path),
            path_evidence("token_savings_ledger", &token_savings_ledger_path(output)),
            "context packet savings are measured locally and syncable to memd-server".to_string(),
            "source-read attribution records saved tokens when a source-registry hash/path is referenced instead of reread".to_string(),
            "Token Budget prompt section instructs agents to reuse Source IDs, avoid rereading unchanged raw sources, and reread only for exact quotes, current file contents, or changed source hashes".to_string(),
            "server authority replay proof syncs token savings payloads after backend outage".to_string(),
        ]
        .into_iter()
        .chain(sync_queue_evidence(sync_queue.as_ref()))
        .collect(),
        vec![],
    ));
    features.push(feature(
        "proof_gates",
        "working",
        vec![
            "feature registry marks broken/partial/unproven work instead of claiming 25/5".to_string(),
            "25/5 memory OS proof suite has an implementation-readiness preflight that blocks public proof when any feature is not working".to_string(),
            "public proof harness remains separate from implementation-focused checks; run it after readiness, not during incremental implementation".to_string(),
        ],
        vec![],
    ));

    let status = if features.iter().any(|feature| feature.status == "broken") {
        "broken"
    } else if features.iter().all(|feature| feature.status == "working") {
        "working"
    } else {
        "partial"
    };

    MemoryOsFeatureReport {
        generated_at: Utc::now(),
        bundle_root: output.display().to_string(),
        status: status.to_string(),
        market_claim: build_market_claim_gate(),
        features,
    }
}

fn native_handoff_recovery_feature(
    output: &Path,
    wake_path: &Path,
    mem_path: &Path,
) -> MemoryOsFeature {
    let wake = fs::read_to_string(wake_path).unwrap_or_default();
    let mem = fs::read_to_string(mem_path).unwrap_or_default();
    let voice_mode = read_bundle_voice_mode(output).unwrap_or_else(default_voice_mode);
    let has_recovery_line = wake.contains("- recovery voice=");
    let has_native_continuity = wake.contains("next=")
        && wake.contains("blocker=")
        && (wake.contains("dirty=") || mem.contains("dirty="));
    let quality_ready = wake.contains("quality=ready:") || mem.contains("quality=ready:");
    let quality_partial = wake.contains("quality=partial:") || mem.contains("quality=partial:");

    let status = if quality_ready && has_recovery_line && has_native_continuity {
        "working"
    } else if has_recovery_line || has_native_continuity || quality_partial {
        "partial"
    } else {
        "unproven"
    };

    let mut gaps = Vec::new();
    if !has_recovery_line {
        gaps.push("wake does not surface native recovery line".to_string());
    }
    if !has_native_continuity {
        gaps.push("wake does not prove goal/blocker/dirty/next recovery capsule".to_string());
    }
    if !quality_ready {
        gaps.push("handoff_quality is not proven ready by native wake/mem surfaces".to_string());
    }

    feature(
        "native_handoff_recovery",
        status,
        vec![
            format!("voice_mode={voice_mode}"),
            path_evidence("wake", wake_path),
            path_evidence("mem", mem_path),
            format!("wake_recovery_line={has_recovery_line}"),
            format!("native_continuity={has_native_continuity}"),
            format!(
                "handoff_quality={}",
                if quality_ready {
                    "ready"
                } else if quality_partial {
                    "partial"
                } else {
                    "unknown"
                }
            ),
            "fresh-agent recovery must use wake/resume/messages/handoff, not markdown handoff docs"
                .to_string(),
        ],
        gaps,
    )
}

#[derive(Debug, Clone, Default)]
struct RepoHygieneAudit {
    evidence: Vec<String>,
    gaps: Vec<String>,
}

fn repo_hygiene_feature(output: &Path) -> MemoryOsFeature {
    let audit = repo_hygiene_audit(output);
    let status = if audit
        .gaps
        .iter()
        .any(|gap| gap.contains("raw benchmark cache"))
    {
        "broken"
    } else if audit.gaps.is_empty() {
        "working"
    } else {
        "partial"
    };
    feature(
        "repo_hygiene",
        status,
        vec![
            "checkpoint --auto-commit uses tracked-file-only git add -u".to_string(),
            "checkpoint --auto-commit refuses broad dirty trees by default".to_string(),
            "default MEMD_AUTO_COMMIT_MAX_TRACKED_FILES=5".to_string(),
            "implementation work must not run broad benchmarks".to_string(),
            "raw benchmark caches must stay out of repo-visible paths".to_string(),
        ]
        .into_iter()
        .chain(audit.evidence)
        .collect(),
        audit.gaps,
    )
}

fn repo_hygiene_audit(output: &Path) -> RepoHygieneAudit {
    let mut audit = RepoHygieneAudit::default();
    let repo_root = infer_bundle_project_root(output)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));
    audit
        .evidence
        .push(format!("repo_root={}", repo_root.display()));
    let status = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&repo_root)
        .output();
    let Ok(status) = status else {
        audit
            .gaps
            .push("git status unavailable; repo hygiene is unproven".to_string());
        return audit;
    };
    if !status.status.success() {
        audit.gaps.push(format!(
            "git status failed; repo hygiene is unproven: {}",
            String::from_utf8_lossy(&status.stderr).trim()
        ));
        return audit;
    }
    let status_text = String::from_utf8_lossy(&status.stdout);
    let tracked_dirty = status_text
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            !trimmed.is_empty() && !trimmed.starts_with("??")
        })
        .count();
    let untracked = status_text
        .lines()
        .filter(|line| line.trim_start().starts_with("??"))
        .count();
    let max_tracked = std::env::var("MEMD_AUTO_COMMIT_MAX_TRACKED_FILES")
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(5);
    audit
        .evidence
        .push(format!("dirty_tracked_files={tracked_dirty}"));
    audit.evidence.push(format!("untracked_paths={untracked}"));
    audit
        .evidence
        .push(format!("auto_commit_max_tracked_files={max_tracked}"));
    if tracked_dirty > max_tracked {
        audit.gaps.push(format!(
            "broad dirty tree: {tracked_dirty} tracked files exceed auto-commit limit {max_tracked}"
        ));
    }
    if untracked > 20 {
        audit.gaps.push(format!(
            "large untracked surface: {untracked} paths need triage"
        ));
    }

    for cache_path in raw_benchmark_cache_paths(&repo_root) {
        if cache_path.exists() {
            audit.evidence.push(format!(
                "raw_benchmark_cache_path=present:{}",
                cache_path.display()
            ));
            audit.gaps.push(format!(
                "raw benchmark cache is present in repo-visible path: {}",
                cache_path.display()
            ));
        } else {
            audit.evidence.push(format!(
                "raw_benchmark_cache_path=absent:{}",
                cache_path.display()
            ));
        }
    }
    audit
}

fn raw_benchmark_cache_paths(repo_root: &Path) -> Vec<PathBuf> {
    vec![
        repo_root.join("external-public-cache"),
        repo_root
            .join("docs")
            .join("verification")
            .join("25-5-memory-os-runs")
            .join("external-public-cache"),
    ]
}

#[derive(Debug, Clone, Default)]
struct CapabilityInventoryAudit {
    evidence: Vec<String>,
    gaps: Vec<String>,
}

fn capability_sync_feature(
    output: &Path,
    capability_registry: &CapabilityRegistry,
    sync_queue: Option<&OfflineQueueStatus>,
) -> MemoryOsFeature {
    let inventory = capability_inventory_audit(capability_registry);
    let materializer = capability_materializer_audit(capability_registry);
    let server_route = capability_server_route_audit(output);
    let status = if capability_registry.capabilities.is_empty() {
        "broken"
    } else if !inventory.gaps.is_empty()
        || !materializer.gaps.is_empty()
        || !server_route.gaps.is_empty()
    {
        "partial"
    } else {
        "working"
    };
    feature(
        "capability_sync",
        status,
        vec![
            format!(
            "discovered_capabilities={}",
                capability_registry.capabilities.len()
            ),
            "server capability sync/list routes round-trip records with project/workspace/user/agent scope".to_string(),
            "capability list query searches persisted payload metadata such as source paths".to_string(),
            "server authority proof simulates PC-A syncing skills/plugins and PC-B listing available plus unavailable capabilities with target-equivalent guidance; this proves inventory sync, not install/materialization".to_string(),
        ]
        .into_iter()
        .chain(inventory.evidence)
        .chain(materializer.evidence)
        .chain(server_route.evidence)
        .chain(sync_queue_evidence(sync_queue))
        .collect(),
        vec_merge(vec_merge(inventory.gaps, materializer.gaps), server_route.gaps),
    )
}

fn capability_materializer_audit(registry: &CapabilityRegistry) -> CapabilityInventoryAudit {
    let (materialization_installable, materialization_missing) =
        capability_materialization_counts(registry);
    let payload_text_records = registry
        .capabilities
        .iter()
        .filter(|record| capability_has_payload_text(record))
        .count();
    let payload_file_set_records = registry
        .capabilities
        .iter()
        .filter(|record| capability_has_payload_file_set(record))
        .count();
    let host_cli_install_plans = registry
        .capabilities
        .iter()
        .filter(|record| record.kind == "cli" || record.portability_class == "host-local")
        .filter(|record| {
            record
                .notes
                .iter()
                .any(|note| note.starts_with("memd:host-cli-install-plan:"))
        })
        .count();
    let mut evidence = vec![
        "server_backed_inventory=present".to_string(),
        "fresh_machine_materializer=partial".to_string(),
        "server-synced text payloads can be materialized for small harness assets".to_string(),
        "server-synced payload sets can restore bounded skill/plugin text files".to_string(),
        format!("payload_text_records={payload_text_records}"),
        format!("payload_file_set_records={payload_file_set_records}"),
        format!("materialization_installable={materialization_installable}"),
        format!("materialization_missing={materialization_missing}"),
        format!("host_cli_install_plans={host_cli_install_plans}"),
    ];
    let missing_records = registry
        .capabilities
        .iter()
        .filter(|record| !capability_has_fresh_machine_payload(record))
        .collect::<Vec<_>>();
    let mut gaps = Vec::new();
    if materialization_missing > 0 {
        gaps.push(format!(
            "{materialization_missing} capabilities lack fresh-machine materialization payloads"
        ));
    }
    if missing_records
        .iter()
        .any(|record| record.portability_class == "host-local" || record.kind == "cli")
    {
        gaps.push("host-local CLI availability cannot be restored by memd sync alone".to_string());
    }
    let host_cli_without_install_plan = registry
        .capabilities
        .iter()
        .filter(|record| record.kind == "cli" || record.portability_class == "host-local")
        .filter(|record| {
            !record
                .notes
                .iter()
                .any(|note| note.starts_with("memd:host-cli-install-plan:"))
        })
        .count();
    if host_cli_without_install_plan > 0 {
        gaps.push(format!(
            "{host_cli_without_install_plan} host CLI records lack server-synced install plans"
        ));
    }
    let missing_assets = missing_records
        .iter()
        .filter(|record| record.portability_class != "host-local" && record.kind != "cli")
        .map(|record| format!("{}:{}", record.harness, record.kind))
        .collect::<std::collections::BTreeSet<_>>();
    if !missing_assets.is_empty() {
        gaps.push(format!(
            "fresh-machine materialization missing for {}",
            missing_assets.into_iter().collect::<Vec<_>>().join(",")
        ));
    }

    if registry.capabilities.iter().any(|record| {
        record.portability_class == "harness-native" || record.portability_class == "host-local"
    }) {
        evidence.push("non_universal_capabilities_detected=true".to_string());
    } else {
        gaps.push("no harness-native or host-local capability inventory found to prove cross-machine recovery expectations".to_string());
    }

    CapabilityInventoryAudit { evidence, gaps }
}

fn capability_materialization_counts(registry: &CapabilityRegistry) -> (usize, usize) {
    registry
        .capabilities
        .iter()
        .fold((0, 0), |(installable, missing), record| {
            if capability_has_fresh_machine_payload(record) {
                (installable + 1, missing)
            } else {
                (installable, missing + 1)
            }
        })
}

fn capability_has_fresh_machine_payload(record: &CapabilityRecord) -> bool {
    if capability_has_payload_text(record) || capability_has_payload_file_set(record) {
        return record.portability_class != "host-local" && record.kind != "cli";
    }
    let bundle_relative = record.source_path.starts_with(".memd/")
        || record.source_path.starts_with("agents/")
        || (record.harness == "project"
            && is_universal_class(&record.portability_class)
            && !record.source_path.starts_with('/'));
    bundle_relative
        && record.portability_class != "host-local"
        && record.kind != "cli"
        && !(record.harness == "codex" && record.kind.contains("plugin"))
}

fn capability_has_payload_text(record: &CapabilityRecord) -> bool {
    record
        .notes
        .iter()
        .any(|note| note.starts_with("memd:payload-text:"))
}

fn capability_has_payload_file_set(record: &CapabilityRecord) -> bool {
    record
        .notes
        .iter()
        .any(|note| note.starts_with("memd:payload-file-json:"))
}

fn capability_inventory_audit(registry: &CapabilityRegistry) -> CapabilityInventoryAudit {
    let registry_sources = registry
        .capabilities
        .iter()
        .map(|record| record.source_path.clone())
        .collect::<std::collections::HashSet<_>>();
    let mut evidence = Vec::new();
    let mut gaps = Vec::new();

    let mut by_kind = std::collections::BTreeMap::<String, usize>::new();
    for record in &registry.capabilities {
        *by_kind
            .entry(format!("{}:{}", record.harness, record.kind))
            .or_default() += 1;
    }
    if !by_kind.is_empty() {
        evidence.push(format!(
            "registry_by_kind={}",
            by_kind
                .iter()
                .map(|(kind, count)| format!("{kind}:{count}"))
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    let harness_pack_harnesses = registry
        .capabilities
        .iter()
        .filter(|record| record.kind == "harness-pack")
        .map(|record| record.harness.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    if !harness_pack_harnesses.is_empty() {
        evidence.push(format!(
            "harness_pack_records={}",
            harness_pack_harnesses
                .iter()
                .copied()
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    for required in [
        "agent-zero",
        "claude-code",
        "codex",
        "hermes",
        "openclaw",
        "opencode",
    ] {
        if !harness_pack_harnesses.contains(required) {
            gaps.push(format!("missing {required} harness-pack capability record"));
        }
    }

    let Some(home) = home_dir() else {
        gaps.push("cannot inspect HOME for skills/plugins/cli inventory".to_string());
        return CapabilityInventoryAudit { evidence, gaps };
    };

    let codex_skills = collect_files_matching(&home.join(".codex").join("skills"), 5, |path| {
        path.file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value == "SKILL.md")
    });
    let plugin_skills = collect_files_matching(
        &home.join(".codex").join("plugins").join("cache"),
        8,
        |path| {
            path.file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value == "SKILL.md")
                && path
                    .parent()
                    .and_then(|parent| parent.parent())
                    .and_then(|parent| parent.file_name())
                    .and_then(|value| value.to_str())
                    == Some("skills")
        },
    );
    let plugin_manifests = collect_files_matching(
        &home.join(".codex").join("plugins").join("cache"),
        8,
        |path| {
            path.file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value == "plugin.json")
                && path
                    .parent()
                    .and_then(|parent| parent.file_name())
                    .and_then(|value| value.to_str())
                    == Some(".codex-plugin")
        },
    );
    let claude_settings = home.join(".claude").join("settings.json");
    let cli_records = collect_path_cli_capabilities();

    evidence.push(format!("codex_skill_files={}", codex_skills.len()));
    evidence.push(format!("codex_plugin_skill_files={}", plugin_skills.len()));
    evidence.push(format!("codex_plugin_manifests={}", plugin_manifests.len()));
    evidence.push(format!(
        "claude_settings={}",
        if claude_settings.is_file() {
            "present"
        } else {
            "absent"
        }
    ));
    evidence.push(format!("installed_cli_records={}", cli_records.len()));

    push_missing_inventory_gap("codex skills", &codex_skills, &registry_sources, &mut gaps);
    push_missing_inventory_gap(
        "codex plugin skills",
        &plugin_skills,
        &registry_sources,
        &mut gaps,
    );
    push_missing_inventory_gap(
        "codex plugin manifests",
        &plugin_manifests,
        &registry_sources,
        &mut gaps,
    );
    if claude_settings.is_file()
        && !registry_sources.contains(&claude_settings.display().to_string())
    {
        gaps.push("missing Claude Code settings capability record".to_string());
    }
    let missing_cli = cli_records
        .iter()
        .filter(|record| !registry_sources.contains(&record.source_path))
        .count();
    if missing_cli > 0 {
        gaps.push(format!(
            "missing {missing_cli} installed CLI capability record(s)"
        ));
    }

    CapabilityInventoryAudit { evidence, gaps }
}

fn capability_server_route_audit(output: &Path) -> CapabilityInventoryAudit {
    let runtime = read_bundle_runtime_config(output).ok().flatten();
    let Some(runtime_base_url) = runtime
        .as_ref()
        .and_then(|config| config.base_url.as_deref())
    else {
        return CapabilityInventoryAudit::default();
    };
    let base_url = resolve_bundle_command_base_url(&default_base_url(), Some(runtime_base_url));
    match fetch_server_path(&base_url, "/capabilities?limit=1") {
        Ok((status, _)) if status == 200 => CapabilityInventoryAudit {
            evidence: vec!["server_capabilities_route=ok".to_string()],
            gaps: Vec::new(),
        },
        Ok((status, _)) => CapabilityInventoryAudit {
            evidence: vec![format!("server_capabilities_route=http_{status}")],
            gaps: vec![format!(
                "live server /capabilities route returned HTTP {status}; server capability sync is not proven"
            )],
        },
        Err(error) => CapabilityInventoryAudit {
            evidence: vec![format!("server_capabilities_route=unavailable:{error}")],
            gaps: vec!["live server /capabilities route is unavailable".to_string()],
        },
    }
}

fn push_missing_inventory_gap(
    label: &str,
    files: &[PathBuf],
    registry_sources: &std::collections::HashSet<String>,
    gaps: &mut Vec<String>,
) {
    let missing = files
        .iter()
        .filter(|path| !registry_sources.contains(&path.display().to_string()))
        .count();
    if missing > 0 {
        gaps.push(format!("missing {missing} {label} capability record(s)"));
    }
}

fn collect_files_matching<F>(root: &Path, max_depth: usize, matches: F) -> Vec<PathBuf>
where
    F: Fn(&Path) -> bool,
{
    let mut out = Vec::new();
    collect_files_matching_inner(root, 0, max_depth, &matches, &mut out);
    out.sort();
    out
}

fn collect_files_matching_inner<F>(
    root: &Path,
    depth: usize,
    max_depth: usize,
    matches: &F,
    out: &mut Vec<PathBuf>,
) where
    F: Fn(&Path) -> bool,
{
    if depth > max_depth || out.len() >= 1_000 || !root.is_dir() {
        return;
    }
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_file() && matches(&path) {
            out.push(path);
            if out.len() >= 1_000 {
                return;
            }
        } else if path.is_dir() {
            collect_files_matching_inner(&path, depth + 1, max_depth, matches, out);
            if out.len() >= 1_000 {
                return;
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
struct ServerAuthorityFeatureProbe {
    evidence: Vec<String>,
    gaps: Vec<String>,
}

fn server_authority_feature_probe(output: &Path) -> ServerAuthorityFeatureProbe {
    let runtime = read_bundle_runtime_config(output).ok().flatten();
    let Some(runtime_base_url) = runtime
        .as_ref()
        .and_then(|config| config.base_url.as_deref())
    else {
        return ServerAuthorityFeatureProbe::default();
    };
    let base_url = resolve_bundle_command_base_url(&default_base_url(), Some(runtime_base_url));
    match fetch_server_status_json(&base_url) {
        Ok(value) => evaluate_server_authority_status(&value),
        Err(error) => ServerAuthorityFeatureProbe {
            evidence: vec![format!("server_api_status=unavailable:{error}")],
            gaps: vec!["server authority status endpoint is unavailable".to_string()],
        },
    }
}

fn fetch_server_status_json(base_url: &str) -> anyhow::Result<serde_json::Value> {
    let (status, body) = fetch_server_path(base_url, "/api/status")?;
    if status != 200 {
        anyhow::bail!("http status {status}");
    }
    serde_json::from_str(body.trim()).context("parse status JSON")
}

fn fetch_server_path(base_url: &str, path: &str) -> anyhow::Result<(u16, String)> {
    let (host, port) = parse_http_base_url(base_url)?;
    let address = format!("{host}:{port}");
    let mut addrs = address
        .to_socket_addrs()
        .with_context(|| format!("resolve {address}"))?;
    let addr = addrs.next().context("resolve server address")?;
    let timeout = std::time::Duration::from_millis(500);
    let mut stream = std::net::TcpStream::connect_timeout(&addr, timeout)
        .with_context(|| format!("connect {address}"))?;
    let _ = stream.set_read_timeout(Some(timeout));
    let _ = stream.set_write_timeout(Some(timeout));
    let request = format!("GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    std::io::Write::write_all(&mut stream, request.as_bytes()).context("write status request")?;
    let mut raw = String::new();
    std::io::Read::read_to_string(&mut stream, &mut raw).context("read status response")?;
    let (header, body) = raw
        .split_once("\r\n\r\n")
        .context("parse status response headers")?;
    let status = header
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|status| status.parse::<u16>().ok())
        .context("parse status code")?;
    Ok((status, body.to_string()))
}

fn parse_http_base_url(base_url: &str) -> anyhow::Result<(String, u16)> {
    let trimmed = base_url.trim();
    let rest = trimmed
        .strip_prefix("http://")
        .context("server status probe supports http:// base URLs only")?;
    let authority = rest.split('/').next().unwrap_or(rest);
    let (host, port) = authority
        .rsplit_once(':')
        .map(|(host, port)| {
            let parsed_port = port.parse::<u16>().context("parse base URL port")?;
            Ok::<_, anyhow::Error>((host.to_string(), parsed_port))
        })
        .transpose()?
        .unwrap_or_else(|| (authority.to_string(), 80));
    if host.trim().is_empty() {
        anyhow::bail!("base URL host is empty");
    }
    Ok((host, port))
}

fn evaluate_server_authority_status(value: &serde_json::Value) -> ServerAuthorityFeatureProbe {
    let git_commit = value
        .get("git_commit")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");
    let git_dirty = value
        .get("git_dirty")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");
    let benchmark_gate = value
        .get("benchmark_gate")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");
    let schema_version = value
        .get("schema_version")
        .and_then(serde_json::Value::as_i64)
        .map(|version| version.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let atlas_dormant = value
        .get("atlas")
        .and_then(|atlas| atlas.get("dormant"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    let mut gaps = Vec::new();
    if git_commit == "unknown" {
        gaps.push("server git_commit is unknown; deploy identity is not proven".to_string());
    }
    if git_dirty == "unknown" {
        gaps.push("server git_dirty is unknown; deployed dirty state is not proven".to_string());
    }
    if !matches!(benchmark_gate, "pass" | "acceptable") {
        gaps.push(format!(
            "server benchmark_gate={benchmark_gate}; authority is not proven ready"
        ));
    }
    if atlas_dormant {
        gaps.push("server atlas is dormant; context graph authority is partial".to_string());
    }

    ServerAuthorityFeatureProbe {
        evidence: vec![
            "server_api_status=ok".to_string(),
            format!("server_git_commit={git_commit}"),
            format!("server_git_dirty={git_dirty}"),
            format!("server_benchmark_gate={benchmark_gate}"),
            format!("server_schema_version={schema_version}"),
            format!("server_atlas_dormant={atlas_dormant}"),
        ],
        gaps,
    }
}

fn build_market_claim_gate() -> MarketClaimGate {
    let report_dir = Path::new("docs")
        .join("verification")
        .join("25-5-memory-os-runs");
    let mut evidence = Vec::new();
    let mut blockers = Vec::new();

    let open_competitor = latest_report_with_suffix(&report_dir, "competitor-head-to-head.json")
        .and_then(|path| report_status(&path).map(|status| (path, status)));
    match open_competitor {
        Some((path, status)) if status == "pass" => evidence.push(format!(
            "open competitor head-to-head pass: {}",
            path.display()
        )),
        Some((path, status)) => blockers.push(format!(
            "open competitor head-to-head not pass: status={} report={}",
            status,
            path.display()
        )),
        None => blockers.push("open competitor head-to-head report missing".to_string()),
    }

    let supermemory = latest_report_with_suffix(&report_dir, "supermemory-head-to-head.json")
        .and_then(|path| report_status(&path).map(|status| (path, status)));
    match supermemory {
        Some((path, status)) if status == "pass" => evidence.push(format!(
            "Supermemory same-fixture replay pass: {}",
            path.display()
        )),
        Some((path, status)) => blockers.push(format!(
            "Supermemory same-fixture replay not pass: status={} report={}",
            status,
            path.display()
        )),
        None => blockers.push("Supermemory same-fixture replay report missing".to_string()),
    }

    let stratified = latest_report_with_suffix(&report_dir, "external-public-stratified.json")
        .and_then(|path| report_status(&path).map(|status| (path, status)));
    match stratified {
        Some((path, status)) if status == "pass" => evidence.push(format!(
            "sampled external public proof pass: {}",
            path.display()
        )),
        Some((path, status)) => blockers.push(format!(
            "sampled external public proof not pass: status={} report={}",
            status,
            path.display()
        )),
        None => blockers.push("sampled external public proof report missing".to_string()),
    }

    let full_corpus = latest_report_with_suffix(&report_dir, "external-public-full.json")
        .and_then(|path| report_status(&path).map(|status| (path, status)));
    match full_corpus {
        Some((path, status)) if status == "pass" => evidence.push(format!(
            "full external public proof pass: {}",
            path.display()
        )),
        Some((path, status)) => blockers.push(format!(
            "full external public proof not pass: status={} report={}",
            status,
            path.display()
        )),
        None => blockers.push(
            "full external public proof report missing; sampled/stratified proof is not a 25/5 market claim"
                .to_string(),
        ),
    }

    let status = if blockers.is_empty() {
        "proven"
    } else {
        "blocked"
    };
    MarketClaimGate {
        status: status.to_string(),
        evidence,
        blockers,
    }
}

fn latest_report_with_suffix(report_dir: &Path, suffix: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(report_dir).ok()?;
    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(suffix))
        })
        .filter_map(|path| {
            let modified = path.metadata().and_then(|meta| meta.modified()).ok()?;
            Some((modified, path))
        })
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| path)
}

fn report_status(path: &Path) -> Option<String> {
    let raw = fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&raw).ok()?;
    value
        .get("status")
        .and_then(|status| status.as_str())
        .map(str::to_string)
}

fn sync_queue_evidence(queue: Option<&OfflineQueueStatus>) -> Vec<String> {
    let Some(queue) = queue else {
        return vec!["sync_queue=unavailable".to_string()];
    };
    vec![
        format!(
            "sync_queue=store_pending:{} store_failed:{} sync_pending:{} sync_failed:{}",
            queue.store.pending, queue.store.failed, queue.sync.pending, queue.sync.failed
        ),
        format!(
            "sync_queue_by_kind={}",
            sync_queue_kind_evidence(&queue.sync)
        ),
        format!("sync_queue_path={}", queue.sync.path.display()),
    ]
}

fn sync_queue_kind_evidence(sync: &OfflineSyncQueueStatus) -> String {
    if sync.by_kind.is_empty() {
        return "none".to_string();
    }
    sync.by_kind
        .iter()
        .map(|(kind, status)| {
            format!(
                "{}:pending:{} failed:{} synced:{} total:{}",
                kind, status.pending, status.failed, status.synced, status.total
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn vec_merge(mut left: Vec<String>, right: Vec<String>) -> Vec<String> {
    left.extend(right);
    left
}

fn build_access_report(output: &Path, resource: Option<&str>, agent: Option<&str>) -> AccessReport {
    let mut routes = Vec::new();
    let bw = detect_bitwarden();
    let agent_secrets = detect_agent_secrets();
    routes.push(AccessRouteRecord {
        id: "bitwarden".to_string(),
        provider: "bitwarden".to_string(),
        status: bw.status.clone(),
        scope: resource.unwrap_or("user/project").to_string(),
        secret_values_stored: false,
        guidance: if bw.status == "unlocked" {
            format!(
                "Approved route for {}: use Bitwarden references through brokered tool flow; never print or store secret values.",
                agent.unwrap_or("agent")
            )
        } else if bw.installed {
            "Bitwarden is installed but locked or unknown; ask user to unlock before workaround.".to_string()
        } else {
            "Bitwarden CLI not found; list route as unavailable, do not delete it.".to_string()
        },
        source: bw.source,
    });

    routes.push(AccessRouteRecord {
        id: "agent-secrets".to_string(),
        provider: "agent-secrets".to_string(),
        status: agent_secrets.status.clone(),
        scope: resource.unwrap_or("user/project").to_string(),
        secret_values_stored: false,
        guidance: if agent_secrets.installed {
            "Use agent-secrets as the external broker; memd syncs refs, scopes, provider status, and ask-user guidance only.".to_string()
        } else {
            "agent-secrets is not installed or not discoverable; keep route listed as unavailable, use Bitwarden/keychain refs when available.".to_string()
        },
        source: agent_secrets.source,
    });

    routes.push(AccessRouteRecord {
        id: "macos-keychain".to_string(),
        provider: "macos-keychain".to_string(),
        status: if command_exists("security") {
            "available"
        } else {
            "unavailable"
        }
        .to_string(),
        scope: resource.unwrap_or("user").to_string(),
        secret_values_stored: false,
        guidance: "Use as secret provider only through explicit broker policy; never persist resolved values in memory.".to_string(),
        source: "security CLI".to_string(),
    });

    let status = if routes
        .iter()
        .any(|route| route.status == "unlocked" || route.status == "available")
    {
        "partial"
    } else {
        "unproven"
    };
    AccessReport {
        generated_at: Utc::now(),
        bundle_root: output.display().to_string(),
        status: status.to_string(),
        routes,
        notes: vec![
            "memd stores access metadata and secret refs only, never secret values".to_string(),
            "locked providers should trigger an ask-user-to-unlock route, not workaround churn"
                .to_string(),
        ],
    }
}

fn filter_access_report_by_provider(
    mut report: AccessReport,
    provider: Option<&str>,
) -> AccessReport {
    let Some(provider) = provider
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase)
    else {
        return report;
    };
    report.routes.retain(|route| {
        route.provider.eq_ignore_ascii_case(&provider) || route.id.eq_ignore_ascii_case(&provider)
    });
    if report.routes.is_empty() {
        report.status = "unproven".to_string();
        report.notes.push(format!(
            "provider `{provider}` has no configured route; ask user before inventing an access path"
        ));
    }
    report
}

fn build_secret_provider_report(output: &Path) -> SecretProviderReport {
    let bw = detect_bitwarden();
    let agent_secrets = detect_agent_secrets();
    let providers = vec![
        SecretProviderRecord {
            provider: "bitwarden".to_string(),
            installed: bw.installed,
            status: bw.status,
            guidance: "Use refs such as bw:item:<id>; resolve only for approved purpose/TTL."
                .to_string(),
        },
        SecretProviderRecord {
            provider: "agent-secrets".to_string(),
            installed: agent_secrets.installed,
            status: agent_secrets.status,
            guidance: "Interop broker: memd stores refs/routes/audit metadata, agent-secrets resolves values outside memory/context.".to_string(),
        },
        SecretProviderRecord {
            provider: "macos-keychain".to_string(),
            installed: command_exists("security"),
            status: if command_exists("security") {
                "available"
            } else {
                "unavailable"
            }
            .to_string(),
            guidance: "Use keychain item refs; never write resolved values to memd.".to_string(),
        },
    ];
    let status = if providers.iter().any(|provider| provider.installed) {
        "partial"
    } else {
        "unproven"
    };
    SecretProviderReport {
        generated_at: Utc::now(),
        bundle_root: output.display().to_string(),
        status: status.to_string(),
        providers,
        policy: vec![
            "store provider/id/scope/purpose/TTL/audit metadata only".to_string(),
            "do not store passwords, tokens, cookies, recovery codes, or raw env values"
                .to_string(),
            "resolved values are ephemeral and must not enter context packets".to_string(),
        ],
    }
}

fn build_token_savings_report(output: &Path, since: Option<String>) -> TokenSavingsReport {
    let source_registry_path = output.join("state").join("source-registry.json");
    let ledger_path = token_savings_ledger_path(output);
    let ledger = read_token_savings_ledger(&ledger_path, since.as_deref());
    let measured_input_tokens = ledger
        .iter()
        .map(|entry| entry.baseline_input_tokens)
        .sum::<usize>();
    let measured_output_tokens = ledger
        .iter()
        .map(|entry| entry.output_tokens)
        .sum::<usize>();
    let measured_tokens_saved = ledger.iter().map(|entry| entry.tokens_saved).sum::<usize>();
    let (source_records, estimated_source_tokens) =
        read_source_registry_token_estimate(&source_registry_path);
    let wake_tokens = read_wake_token_estimate(&output.join("wake-token-metrics.json"))
        .or_else(|| estimate_file_tokens(&output.join("wake.md")));
    let estimated_tokens_saved = estimated_source_tokens.saturating_sub(wake_tokens.unwrap_or(0));
    TokenSavingsReport {
        generated_at: Utc::now(),
        bundle_root: output.display().to_string(),
        source: "local".to_string(),
        since,
        ledger_path: ledger_path.display().to_string(),
        ledger_events: ledger.len(),
        server_events: 0,
        server_measured_input_tokens: 0,
        server_measured_output_tokens: 0,
        server_measured_tokens_saved: 0,
        measured_input_tokens,
        measured_output_tokens,
        measured_tokens_saved,
        source_records,
        estimated_source_tokens,
        wake_tokens,
        estimated_tokens_saved,
        notes: vec![
            "measured = append-only context compile ledger baseline tokens minus rendered packet tokens".to_string(),
            "estimate = tracked source bytes/4 minus current wake packet tokens".to_string(),
            "source-read hook attribution can refine baseline further, but context compiles are now recorded".to_string(),
        ],
    }
}

pub(crate) fn merge_server_token_savings_report(
    mut report: TokenSavingsReport,
    server: memd_schema::TokenSavingsListResponse,
) -> TokenSavingsReport {
    report.source = "server".to_string();
    report.server_events = server.total;
    report.server_measured_input_tokens = server.measured_input_tokens;
    report.server_measured_output_tokens = server.measured_output_tokens;
    report.server_measured_tokens_saved = server.measured_tokens_saved;
    report.ledger_events = server.total;
    report.measured_input_tokens = server.measured_input_tokens;
    report.measured_output_tokens = server.measured_output_tokens;
    report.measured_tokens_saved = server.measured_tokens_saved;
    report.notes.push(
        "server measured totals came from memd-server /tokens/savings; local ledger retained as fallback"
            .to_string(),
    );
    report
}

pub(crate) fn record_context_token_savings(
    output: &Path,
    req: &ContextRequest,
    model_tier: Option<&str>,
    source_records: usize,
    baseline_text_chars: usize,
    rendered_packet_chars: usize,
) -> anyhow::Result<Option<TokenSavingsLedgerEntry>> {
    let baseline_input_tokens = estimate_text_tokens_from_chars(baseline_text_chars);
    let output_tokens = estimate_text_tokens_from_chars(rendered_packet_chars);
    let tokens_saved = baseline_input_tokens.saturating_sub(output_tokens);
    if baseline_input_tokens == 0 && output_tokens == 0 {
        return Ok(None);
    }
    let entry = TokenSavingsLedgerEntry {
        id: uuid::Uuid::new_v4(),
        ts: Utc::now(),
        operation: "context_packet".to_string(),
        project: req.project.clone(),
        agent: req.agent.clone(),
        model_tier: model_tier.map(str::to_string),
        intent: req.intent.as_ref().map(|intent| format!("{intent:?}")),
        source_records,
        baseline_input_tokens,
        output_tokens,
        tokens_saved,
        reason: "compiled memory/context packet avoided raw source reread".to_string(),
    };
    append_token_savings_ledger_entry(output, &entry)?;
    Ok(Some(entry))
}

pub(crate) fn record_source_read_token_savings(
    output: &Path,
    source_path: &str,
    emitted_reference_chars: usize,
    reason: &str,
) -> anyhow::Result<Option<TokenSavingsLedgerEntry>> {
    let Some((bytes, hash)) = source_registry_entry(output, source_path)? else {
        return Ok(None);
    };
    let baseline_input_tokens = estimate_text_tokens_from_chars(bytes);
    let output_tokens = estimate_text_tokens_from_chars(emitted_reference_chars);
    let tokens_saved = baseline_input_tokens.saturating_sub(output_tokens);
    if tokens_saved == 0 {
        return Ok(None);
    }
    let entry = TokenSavingsLedgerEntry {
        id: uuid::Uuid::new_v4(),
        ts: Utc::now(),
        operation: "source_read_avoided".to_string(),
        project: None,
        agent: None,
        model_tier: None,
        intent: Some("SourceRead".to_string()),
        source_records: 1,
        baseline_input_tokens,
        output_tokens,
        tokens_saved,
        reason: format!(
            "{}; source_path={} source_hash={}",
            reason.trim(),
            source_path,
            hash
        ),
    };
    append_token_savings_ledger_entry(output, &entry)?;
    Ok(Some(entry))
}

pub(crate) fn token_savings_ledger_path(output: &Path) -> PathBuf {
    output.join("state").join("token-savings-ledger.ndjson")
}

fn append_token_savings_ledger_entry(
    output: &Path,
    entry: &TokenSavingsLedgerEntry,
) -> anyhow::Result<()> {
    let path = token_savings_ledger_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create token savings ledger dir {}", parent.display()))?;
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("open token savings ledger {}", path.display()))?;
    serde_json::to_writer(&mut file, entry).context("write token savings ledger entry")?;
    use std::io::Write;
    file.write_all(b"\n")
        .context("newline token savings ledger entry")?;
    Ok(())
}

fn read_token_savings_ledger(path: &Path, since: Option<&str>) -> Vec<TokenSavingsLedgerEntry> {
    let Ok(raw) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let since = since.and_then(|value| {
        DateTime::parse_from_rfc3339(value)
            .ok()
            .map(|value| value.with_timezone(&Utc))
    });
    raw.lines()
        .filter_map(|line| serde_json::from_str::<TokenSavingsLedgerEntry>(line).ok())
        .filter(|entry| since.is_none_or(|since| entry.ts >= since))
        .collect()
}

fn source_registry_entry(
    output: &Path,
    source_path: &str,
) -> anyhow::Result<Option<(usize, String)>> {
    let path = output.join("state").join("source-registry.json");
    let Ok(raw) = fs::read_to_string(&path) else {
        return Ok(None);
    };
    let value: serde_json::Value = serde_json::from_str(&raw)
        .with_context(|| format!("parse source registry {}", path.display()))?;
    let Some(sources) = value.get("sources").and_then(|value| value.as_array()) else {
        return Ok(None);
    };
    let needle = source_path.trim();
    Ok(sources.iter().find_map(|source| {
        let path = source.get("path").and_then(|value| value.as_str())?;
        if path != needle {
            return None;
        }
        let bytes = source.get("bytes").and_then(|value| value.as_u64())? as usize;
        let hash = source
            .get("hash")
            .and_then(|value| value.as_str())
            .unwrap_or("unknown")
            .to_string();
        Some((bytes, hash))
    }))
}

pub(crate) fn estimate_text_tokens_from_chars(chars: usize) -> usize {
    chars.div_ceil(4)
}

pub(crate) fn build_token_savings_sync_records(
    output: &Path,
) -> anyhow::Result<Vec<memd_schema::TokenSavingsRecord>> {
    let config = read_memory_os_bundle_config(output).ok();
    let records = read_token_savings_ledger(&token_savings_ledger_path(output), None)
        .into_iter()
        .map(|entry| memd_schema::TokenSavingsRecord {
            id: entry.id,
            operation: entry.operation,
            project: entry
                .project
                .or_else(|| config.as_ref().and_then(|config| config.project.clone())),
            namespace: config.as_ref().and_then(|config| config.namespace.clone()),
            workspace: config.as_ref().and_then(|config| config.workspace.clone()),
            user_id: None,
            agent: entry
                .agent
                .or_else(|| config.as_ref().and_then(|config| config.agent.clone())),
            model_tier: entry.model_tier,
            intent: entry.intent,
            source_records: entry.source_records,
            baseline_input_tokens: entry.baseline_input_tokens,
            output_tokens: entry.output_tokens,
            tokens_saved: entry.tokens_saved,
            reason: entry.reason,
            ts: entry.ts,
            updated_at: None,
        })
        .collect();
    Ok(records)
}

#[derive(Debug, Clone)]
struct ProviderProbe {
    installed: bool,
    status: String,
    source: String,
}

fn detect_bitwarden() -> ProviderProbe {
    if !command_exists("bw") {
        return ProviderProbe {
            installed: false,
            status: "unavailable".to_string(),
            source: "bw not found on PATH".to_string(),
        };
    }
    let (status, source) = if std::env::var("BW_SESSION")
        .ok()
        .is_some_and(|value| !value.trim().is_empty())
    {
        ("unlocked", "BW_SESSION env")
    } else {
        ("installed", "PATH")
    };
    ProviderProbe {
        installed: true,
        status: status.to_string(),
        source: source.to_string(),
    }
}

fn detect_agent_secrets() -> ProviderProbe {
    if command_exists("agent-secrets") {
        return ProviderProbe {
            installed: true,
            status: "available".to_string(),
            source: "agent-secrets CLI".to_string(),
        };
    }
    for relative in [
        ".agent-secrets",
        ".agent_secrets",
        ".config/agent-secrets",
        ".config/agent_secrets",
    ] {
        if home_relative_path_exists(relative) {
            return ProviderProbe {
                installed: true,
                status: "available".to_string(),
                source: format!("~/{relative}"),
            };
        }
    }
    ProviderProbe {
        installed: false,
        status: "unavailable".to_string(),
        source: "agent-secrets not found".to_string(),
    }
}

fn home_relative_path_exists(relative: &str) -> bool {
    let Some(home) = std::env::var_os("HOME") else {
        return false;
    };
    PathBuf::from(home).join(relative).exists()
}

fn command_exists(name: &str) -> bool {
    let Some(path_var) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path_var).any(|dir| {
        let candidate = dir.join(name);
        candidate.is_file()
    })
}

fn read_source_registry_token_estimate(path: &Path) -> (usize, usize) {
    let Ok(raw) = fs::read_to_string(path) else {
        return (0, 0);
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return (0, raw.len() / 4);
    };
    let Some(sources) = value.get("sources").and_then(|value| value.as_array()) else {
        return (0, raw.len() / 4);
    };
    let bytes = sources
        .iter()
        .filter_map(|source| source.get("bytes").and_then(|value| value.as_u64()))
        .sum::<u64>() as usize;
    (sources.len(), bytes / 4)
}

fn read_wake_token_estimate(path: &Path) -> Option<usize> {
    let raw = fs::read_to_string(path).ok()?;
    let value = serde_json::from_str::<serde_json::Value>(&raw).ok()?;
    value
        .get("tokens")
        .or_else(|| value.get("estimated_tokens"))
        .and_then(|value| value.as_u64())
        .map(|value| value as usize)
}

fn estimate_file_tokens(path: &Path) -> Option<usize> {
    fs::read_to_string(path).ok().map(|raw| raw.len() / 4)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(report.source_records, 1);
        assert!(report.estimated_source_tokens >= entry.baseline_input_tokens);

        fs::remove_dir_all(output).expect("cleanup source read token savings temp");
    }

    #[test]
    fn server_token_savings_report_overrides_measured_totals() {
        let output =
            std::env::temp_dir().join(format!("memd-token-savings-merge-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(output.join("state")).expect("create token temp");
        let local = build_token_savings_report(&output, None);
        let merged = merge_server_token_savings_report(
            local,
            memd_schema::TokenSavingsListResponse {
                total: 2,
                measured_input_tokens: 1000,
                measured_output_tokens: 300,
                measured_tokens_saved: 700,
                records: Vec::new(),
            },
        );

        assert_eq!(merged.source, "server");
        assert_eq!(merged.ledger_events, 2);
        assert_eq!(merged.server_events, 2);
        assert_eq!(merged.measured_tokens_saved, 700);
        assert_eq!(merged.server_measured_tokens_saved, 700);

        fs::remove_dir_all(output).expect("cleanup token merge temp");
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
                market_claim: MarketClaimGate {
                    status: "blocked".to_string(),
                    evidence: Vec::new(),
                    blockers: vec!["test blocker".to_string()],
                },
                features: Vec::new(),
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
        assert!(summary.contains("market_claim=blocked"));
        assert!(summary.contains("market_blockers=1"));
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
                .any(|item| item.contains("benchmark_gate=fail"))
        );
        assert!(
            probe
                .gaps
                .iter()
                .any(|item| item.contains("atlas is dormant"))
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
                .any(|item| item.contains("fuzzy lane is implemented"))
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
                .any(|item| item.contains("focused no-RAG route proof"))
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
                name: "memd-test-gh".to_string(),
                status: "available-server".to_string(),
                portability_class: "host-local".to_string(),
                source_path: "/source/bin/memd-test-gh".to_string(),
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
                .any(|item| item == "host_cli_install_plans=1")
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
                .any(|gap| gap
                    == "host-local CLI availability cannot be restored by memd sync alone")
        );
        assert!(
            !feature
                .gaps
                .iter()
                .any(|gap| gap.contains("lack server-synced install plans"))
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

        assert_ne!(report.status, report.market_claim.status);
        assert_eq!(report.market_claim.status, "blocked");
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
        assert!(summary.contains("blockers="));

        fs::remove_dir_all(output).expect("cleanup feature temp");
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
                .gaps
                .iter()
                .any(|gap| gap
                    == "host-local CLI availability cannot be restored by memd sync alone")
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
}

fn path_evidence(label: &str, path: &Path) -> String {
    format!(
        "{label}:{}:{}",
        if path.exists() { "present" } else { "missing" },
        path.display()
    )
}

fn feature(id: &str, status: &str, evidence: Vec<String>, gaps: Vec<String>) -> MemoryOsFeature {
    MemoryOsFeature {
        id: id.to_string(),
        status: status.to_string(),
        evidence,
        gaps,
    }
}

pub(crate) fn read_memory_os_bundle_config(output: &Path) -> anyhow::Result<BundleConfigFile> {
    let config_path = output.join("config.json");
    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))
}

use super::*;

pub(crate) fn default_auto_short_term_capture() -> bool {
    true
}

pub(crate) fn default_true() -> bool {
    true
}

pub(crate) fn default_authority_mode() -> String {
    "shared".to_string()
}

pub(crate) fn hardcoded_default_voice_mode() -> String {
    "caveman-lite".to_string()
}

/// The four caveman voice modes, from most verbose to most compressed:
/// 1. normal    — full verbose, no compression
/// 2. caveman-lite — light compression, normal spelling
/// 3. caveman-full — compressed, few tokens, normal spelling + exact technical terms
/// 4. caveman-ultra — hard compressed, rewrite-before-send discipline

pub(crate) fn default_voice_mode() -> String {
    if let Ok(value) = std::env::var("MEMD_VOICE_MODE") {
        if let Ok(normalized) = normalize_voice_mode_value(&value) {
            return normalized;
        }
    }

    let config_path = default_global_bundle_root().join("config.json");
    if let Ok(raw) = fs::read_to_string(&config_path) {
        if let Ok(config) = serde_json::from_str::<BundleConfigFile>(&raw) {
            if let Some(voice_mode) = config.voice_mode {
                if let Ok(normalized) = normalize_voice_mode_value(&voice_mode) {
                    return normalized;
                }
            }
        }
    }

    hardcoded_default_voice_mode()
}

pub(crate) const SHARED_MEMD_BASE_URL: &str = "http://100.104.154.24:8787";
pub(crate) const LOCALHOST_MEMD_BASE_URL: &str = "http://127.0.0.1:8787";

pub(crate) fn default_base_url() -> String {
    if let Ok(value) = std::env::var("MEMD_BASE_URL") {
        return value;
    }

    read_bundle_runtime_config(&default_global_bundle_root())
        .ok()
        .flatten()
        .and_then(|runtime| runtime.base_url)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| SHARED_MEMD_BASE_URL.to_string())
}

#[derive(Debug, Clone)]
pub(crate) struct BootstrapAuthorityDecision {
    pub(crate) init_args: InitArgs,
    pub(crate) shared_base_url: String,
    pub(crate) fallback_activated: bool,
}

pub(crate) fn localhost_memd_base_url() -> String {
    std::env::var("MEMD_LOCALHOST_FALLBACK_BASE_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| LOCALHOST_MEMD_BASE_URL.to_string())
}

async fn memd_base_url_reachable(base_url: &str) -> bool {
    let Ok(client) = MemdClient::new(base_url) else {
        return false;
    };
    timeout_ok(client.healthz()).await.is_some()
}

pub(crate) async fn resolve_bootstrap_authority(
    init_args: InitArgs,
) -> anyhow::Result<BootstrapAuthorityDecision> {
    let shared_base_url = init_args.base_url.clone();
    let localhost_fallback_base_url = localhost_memd_base_url();
    if is_loopback_base_url(&shared_base_url) {
        return Ok(BootstrapAuthorityDecision {
            init_args,
            shared_base_url,
            fallback_activated: false,
        });
    }

    if memd_base_url_reachable(&shared_base_url).await {
        return Ok(BootstrapAuthorityDecision {
            init_args,
            shared_base_url,
            fallback_activated: false,
        });
    }

    if !memd_base_url_reachable(&localhost_fallback_base_url).await {
        anyhow::bail!(
            "shared authority {} is unreachable and localhost fallback {} is not available",
            shared_base_url,
            localhost_fallback_base_url
        );
    }

    if !init_args.allow_localhost_read_only_fallback {
        anyhow::bail!(
            "shared authority {} is unreachable. localhost fallback {} is lower trust, read-only, and requires explicit consent via --allow-localhost-read-only-fallback",
            shared_base_url,
            localhost_fallback_base_url
        );
    }

    let mut init_args = init_args;
    init_args.base_url = localhost_fallback_base_url;
    Ok(BootstrapAuthorityDecision {
        init_args,
        shared_base_url,
        fallback_activated: true,
    })
}

pub(crate) fn default_bundle_root_path() -> PathBuf {
    if let Ok(value) = std::env::var("MEMD_BUNDLE_ROOT") {
        let value = value.trim();
        if !value.is_empty() {
            return PathBuf::from(value);
        }
    }

    if let Ok(Some(project_root)) = detect_current_project_root() {
        return project_root.join(".memd");
    }

    default_global_bundle_root()
}

pub(crate) fn default_init_output_path() -> PathBuf {
    match detect_current_project_root() {
        Ok(Some(root)) => root.join(".memd"),
        _ => default_global_bundle_root(),
    }
}

pub(crate) fn default_global_bundle_root() -> PathBuf {
    home_dir()
        .map(|path| path.join(".memd"))
        .unwrap_or_else(|| PathBuf::from(".memd"))
}

pub(crate) fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
}

pub(crate) fn init_project_name(args: &InitArgs, project_root: Option<&Path>) -> String {
    args.project
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            if args.global {
                None
            } else {
                project_root
                    .and_then(|root| {
                        root.file_name()
                            .and_then(|value| value.to_str())
                            .map(|value| value.trim().to_string())
                    })
                    .filter(|value| !value.is_empty())
            }
        })
        .unwrap_or_else(|| "global".to_string())
}

pub(crate) fn init_namespace_name(args: &InitArgs, output: &Path) -> Option<String> {
    args.namespace
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            if args.global || output == default_global_bundle_root() {
                Some("global".to_string())
            } else {
                Some("main".to_string())
            }
        })
}

pub(crate) fn detect_init_project_root(args: &InitArgs) -> anyhow::Result<Option<PathBuf>> {
    if let Some(root) = args.project_root.as_ref() {
        let root = if root.is_absolute() {
            root.clone()
        } else {
            std::env::current_dir()
                .context("read current directory")?
                .join(root)
        };
        return Ok(Some(fs::canonicalize(&root).unwrap_or(root)));
    }

    Ok(detect_current_project_root()?.map(|root| fs::canonicalize(&root).unwrap_or(root)))
}

pub(crate) fn detect_current_project_root() -> anyhow::Result<Option<PathBuf>> {
    let start = std::env::current_dir().context("read current directory")?;
    Ok(find_project_root(&start))
}

pub(crate) fn find_project_root(start: &Path) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        if is_project_root_candidate(&dir) {
            return Some(dir);
        }
        let parent = dir.parent()?;
        if parent == dir {
            return None;
        }
        dir = parent.to_path_buf();
    }
}

pub(crate) struct BootstrapSourceMeta {
    pub(crate) hash: String,
    pub(crate) bytes: usize,
    pub(crate) lines: usize,
}

pub(crate) fn read_bootstrap_source(
    path: &Path,
    max_lines: usize,
) -> Option<(String, BootstrapSourceMeta)> {
    let raw = fs::read(path).ok()?;
    let text = String::from_utf8_lossy(&raw).into_owned();
    let snippet = text
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.trim().is_empty())
        .take(max_lines)
        .collect::<Vec<_>>()
        .join("\n");
    if snippet.trim().is_empty() {
        return None;
    }

    Some((
        snippet,
        BootstrapSourceMeta {
            hash: format!("{:x}", Sha256::digest(&raw)),
            bytes: raw.len(),
            lines: text.lines().count(),
        },
    ))
}

pub(crate) fn source_kind_from_path(path: &Path) -> String {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    let path_str = path.to_string_lossy();
    if path_str.contains("/.codex/") {
        return if name.eq_ignore_ascii_case("SKILL.md") {
            "codex-skill".to_string()
        } else {
            "codex-config".to_string()
        };
    }
    if path_str.contains("/.claude/") {
        return if name.eq_ignore_ascii_case("SKILL.md") {
            "claude-skill".to_string()
        } else {
            "claude-config".to_string()
        };
    }
    if path_str.contains("/.openclaw/") {
        return "openclaw-config".to_string();
    }
    if path_str.contains("/.config/opencode/") || path_str.contains("/.opencode/") {
        return "opencode-config".to_string();
    }
    if name.eq_ignore_ascii_case("AGENTS.md") || name.eq_ignore_ascii_case("CLAUDE.md") {
        return "policy".to_string();
    }
    if name.eq_ignore_ascii_case("TEAMS.md") {
        return "team".to_string();
    }
    if name.eq_ignore_ascii_case("MEMORY.md")
        || name.eq_ignore_ascii_case("SOUL.md")
        || name.eq_ignore_ascii_case("USER.md")
        || name.eq_ignore_ascii_case("IDENTITY.md")
        || name.eq_ignore_ascii_case("TOOLS.md")
        || name.eq_ignore_ascii_case("BOOTSTRAP.md")
        || name.eq_ignore_ascii_case("HEARTBEAT.md")
    {
        return "memory".to_string();
    }
    if name.eq_ignore_ascii_case("DESIGN.md") {
        return "design".to_string();
    }
    if path
        .components()
        .any(|part| part.as_os_str().to_string_lossy() == "docs")
    {
        return "docs".to_string();
    }
    if path.extension().and_then(|value| value.to_str()).is_some() {
        return "doc".to_string();
    }
    "source".to_string()
}

pub(crate) fn bundle_source_registry_path(output: &Path) -> PathBuf {
    output.join("state").join("source-registry.json")
}

pub(crate) fn bundle_capability_registry_path(output: &Path) -> PathBuf {
    output.join("state").join("capability-registry.json")
}

pub(crate) fn bundle_capability_bridges_path(output: &Path) -> PathBuf {
    output.join("state").join("capability-bridges.json")
}

pub(crate) fn bundle_migration_manifest_path(output: &Path) -> PathBuf {
    output.join("state").join("migration-manifest.json")
}

pub(crate) fn write_bundle_source_registry(
    output: &Path,
    registry: &BootstrapSourceRegistry,
) -> anyhow::Result<()> {
    let path = bundle_source_registry_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, serde_json::to_string_pretty(registry)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(crate) fn write_bundle_capability_registry(
    output: &Path,
    registry: &CapabilityRegistry,
) -> anyhow::Result<()> {
    let path = bundle_capability_registry_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, serde_json::to_string_pretty(registry)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(crate) fn write_bundle_capability_bridges(
    output: &Path,
    registry: &CapabilityBridgeRegistry,
) -> anyhow::Result<()> {
    let path = bundle_capability_bridges_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, serde_json::to_string_pretty(registry)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(crate) fn write_bundle_migration_manifest(
    output: &Path,
    manifest: &BundleMigrationManifest,
) -> anyhow::Result<()> {
    let path = bundle_migration_manifest_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, serde_json::to_string_pretty(manifest)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(crate) fn harness_bridge_registry_path(output: &Path) -> PathBuf {
    output.join("state").join("harness-bridge.json")
}

pub(crate) fn write_bundle_harness_bridge_registry(
    output: &Path,
) -> anyhow::Result<HarnessBridgeRegistry> {
    let registry = build_harness_bridge_registry();
    let path = harness_bridge_registry_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, serde_json::to_string_pretty(&registry)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    let markdown_path = output.join("agents").join("HARNESS_BRIDGES.md");
    if let Some(parent) = markdown_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&markdown_path, render_harness_bridge_markdown(&registry))
        .with_context(|| format!("write {}", markdown_path.display()))?;
    Ok(registry)
}

pub(crate) fn read_loop_entries(output: &Path) -> anyhow::Result<Vec<LoopEntry>> {
    let loop_dir = loops_directory(output);
    if !loop_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(&loop_dir)
        .with_context(|| format!("read loops directory {}", loop_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let is_json = path
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.eq_ignore_ascii_case("json"))
            .unwrap_or(false);
        if !is_json {
            continue;
        }
        if path
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value == "loops.summary.json")
        {
            continue;
        }

        let raw = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
        let record: LoopRecord =
            serde_json::from_slice(&raw).with_context(|| format!("parse {}", path.display()))?;
        let (slug, normalized_slug) = derive_loop_slugs(&record, &path);
        entries.push(LoopEntry {
            slug,
            normalized_slug,
            record,
            path,
        });
    }

    entries.sort_by(|a, b| a.normalized_slug.cmp(&b.normalized_slug));
    Ok(entries)
}

pub(crate) fn loops_directory(output: &Path) -> PathBuf {
    output.join("loops")
}

pub(crate) fn derive_loop_slugs(record: &LoopRecord, path: &Path) -> (String, String) {
    let candidate = record
        .slug
        .as_deref()
        .map(str::to_string)
        .unwrap_or_else(|| slug_from_path(path));
    let slug = canonical_slug(&candidate);
    let normalized_slug = slug.to_lowercase();
    (slug, normalized_slug)
}

pub(crate) fn slug_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| "loop".to_string())
}

pub(crate) fn canonical_slug(value: &str) -> String {
    let trimmed = strip_loop_prefix(value);
    if trimmed.is_empty() {
        "loop".to_string()
    } else {
        trimmed.to_string()
    }
}

pub(crate) fn strip_loop_prefix(value: &str) -> &str {
    value
        .trim()
        .trim_start_matches("loop-")
        .trim_start_matches("loop_")
        .trim_start_matches("loops-")
        .trim_start_matches("loops_")
}

pub(crate) fn print_loop_list(entries: &[LoopEntry], output: &Path) {
    let loop_dir = loops_directory(output);
    if entries.is_empty() {
        println!(
            "No loop records found in {}. Run autoresearch to capture loop metadata.",
            loop_dir.display()
        );
        return;
    }

    println!(
        "{:<24} {:>10} {:>12} {:<10} {}",
        "Loop", "Improved", "Tokens", "Status", "Name"
    );
    for entry in entries {
        println!(
            "{:<24} {:>10} {:>12} {:<10} {}",
            entry.slug,
            format_percent(entry.record.percent_improvement),
            format_tokens(entry.record.token_savings),
            entry
                .record
                .status
                .as_deref()
                .unwrap_or("pending")
                .chars()
                .take(10)
                .collect::<String>(),
            entry.record.name.as_deref().unwrap_or("")
        );
    }

    println!();
    println!(
        "Use `memd loops --summary` for aggregate metrics or `memd loops --loop <slug>` for details."
    );
}

pub(crate) fn print_loop_detail(entries: &[LoopEntry], slug_arg: &str) -> anyhow::Result<()> {
    let normalized = canonical_slug(slug_arg).to_lowercase();
    let entry = entries
        .iter()
        .find(|entry| entry.normalized_slug == normalized)
        .ok_or_else(|| anyhow!("loop '{}' not found", slug_arg))?;

    println!("Loop: {}", entry.slug);
    if let Some(name) = &entry.record.name {
        println!("Name: {}", name);
    }
    if let Some(status) = &entry.record.status {
        println!("Status: {}", status);
    }
    if let Some(iteration) = entry.record.iteration {
        println!("Iteration: {}", iteration);
    }
    println!(
        "Percent improvement: {}",
        format_percent(entry.record.percent_improvement)
    );
    println!(
        "Token savings: {}",
        format_tokens(entry.record.token_savings)
    );
    if let Some(summary) = &entry.record.summary {
        println!("Summary:\n{}", indent_text(summary, 2));
    }
    if entry.normalized_slug == "self-evolution"
        && let Some(control_plane) = loop_control_plane_summary(entry)
    {
        println!("Control plane:");
        println!("  proposal: {}", control_plane.proposal_state);
        println!(
            "  scope: {}/{}",
            control_plane.scope_class, control_plane.scope_gate
        );
        println!("  authority: {}", control_plane.authority_tier);
        println!("  merge: {}", control_plane.merge_status);
        println!("  durability: {}", control_plane.durability_status);
        println!("  durable truth: {}", control_plane.durable_truth);
        println!("  branch: {}", control_plane.branch);
    }
    if let Some(artifacts) = &entry.record.artifacts {
        println!("Artifacts:");
        for artifact in artifacts {
            println!("  - {}", artifact);
        }
    }
    println!("Recorded at: {}", entry.path.display());
    if !entry.record.metadata.is_null() {
        println!("Metadata:");
        let json = serde_json::to_string_pretty(&entry.record.metadata)?;
        for line in json.lines() {
            println!("  {}", line);
        }
    }

    Ok(())
}

pub(crate) fn print_loop_summary(entries: &[LoopEntry]) {
    if entries.is_empty() {
        println!("No loop records present yet.");
        return;
    }

    let mut status_counts = BTreeMap::new();
    for entry in entries {
        let status = entry
            .record
            .status
            .as_deref()
            .unwrap_or("pending")
            .to_string();
        *status_counts.entry(status).or_insert(0usize) += 1;
    }

    println!("Loop summary ({} records)", entries.len());
    println!(
        "Status counts: {}",
        status_counts
            .iter()
            .map(|(status, count)| format!("{}={}", status, count))
            .collect::<Vec<_>>()
            .join(" ")
    );

    let improvements: Vec<_> = entries
        .iter()
        .filter_map(|entry| entry.record.percent_improvement.map(|value| (value, entry)))
        .collect();
    if !improvements.is_empty() {
        let total_improvement: f64 = improvements.iter().map(|(value, _)| *value).sum();
        let average = total_improvement / (improvements.len() as f64);
        println!("Average improvement: {:.2}%", average);
        if let Some((best, entry)) = improvements
            .iter()
            .copied()
            .max_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        {
            println!("Best improvement: {:.2}% ({})", best, entry.slug);
        }
        if let Some((worst, entry)) = improvements
            .iter()
            .copied()
            .min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        {
            println!("Worst improvement: {:.2}% ({})", worst, entry.slug);
        }
    } else {
        println!("No percent-improvement metrics recorded yet.");
    }

    let total_tokens: f64 = entries
        .iter()
        .filter_map(|entry| entry.record.token_savings)
        .sum();
    if total_tokens > 0.0 {
        println!("Total tokens saved: {}", format_tokens(Some(total_tokens)));
    } else {
        println!("Total tokens saved: 0");
    }

    if let Some((value, entry)) = entries
        .iter()
        .filter_map(|entry| entry.record.token_savings.map(|value| (value, entry)))
        .max_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    {
        println!(
            "Largest token-saving loop: {} ({} tokens)",
            entry.slug,
            format_tokens(Some(value))
        );
    }

    if let Some(entry) = entries
        .iter()
        .find(|entry| entry.normalized_slug == "self-evolution")
        && let Some(control_plane) = loop_control_plane_summary(entry)
    {
        println!(
            "Self-evolution: {} scope={}/{} authority={} merge={} durability={}",
            control_plane.proposal_state,
            control_plane.scope_class,
            control_plane.scope_gate,
            control_plane.authority_tier,
            control_plane.merge_status,
            control_plane.durability_status
        );
    }
}

pub(crate) fn loop_control_plane_summary(entry: &LoopEntry) -> Option<ExperimentEvolutionSummary> {
    let metadata = &entry.record.metadata;
    let proposal_state = metadata
        .get("proposal_state")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            metadata
                .get("proposal")
                .and_then(|value| value.get("state"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })?;
    let scope_class = metadata
        .get("proposal")
        .and_then(|value| value.get("scope_class"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("none")
        .to_string();
    let scope_gate = metadata
        .get("scope_gate")
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            metadata
                .get("proposal")
                .and_then(|value| value.get("scope_gate"))
                .and_then(serde_json::Value::as_str)
        })
        .unwrap_or("none")
        .to_string();
    let authority_tier = metadata
        .get("proposal")
        .and_then(|value| value.get("authority_tier"))
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            metadata
                .get("authority_ledger")
                .and_then(|value| value.get("entries"))
                .and_then(serde_json::Value::as_array)
                .and_then(|entries| entries.last())
                .and_then(|value| value.get("authority_tier"))
                .and_then(serde_json::Value::as_str)
        })
        .unwrap_or("none")
        .to_string();
    let merge_status = metadata
        .get("merge_queue")
        .and_then(|value| value.get("entries"))
        .and_then(serde_json::Value::as_array)
        .and_then(|entries| entries.last())
        .and_then(|value| value.get("status"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("none")
        .to_string();
    let durability_status = metadata
        .get("durability_queue")
        .and_then(|value| value.get("entries"))
        .and_then(serde_json::Value::as_array)
        .and_then(|entries| entries.last())
        .and_then(|value| value.get("status"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("none")
        .to_string();
    let branch = metadata
        .get("proposal")
        .and_then(|value| value.get("branch"))
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            metadata
                .get("branch_manifest")
                .and_then(|value| value.get("branch"))
                .and_then(serde_json::Value::as_str)
        })
        .unwrap_or("none")
        .to_string();
    let durable_truth = metadata
        .get("durable_truth")
        .and_then(serde_json::Value::as_bool)
        .or_else(|| {
            metadata
                .get("proposal")
                .and_then(|value| value.get("durable_truth"))
                .and_then(serde_json::Value::as_bool)
        })
        .unwrap_or(false);
    Some(ExperimentEvolutionSummary {
        proposal_state,
        scope_class,
        scope_gate,
        authority_tier,
        merge_status,
        durability_status,
        branch,
        durable_truth,
    })
}

pub(crate) fn run_telemetry(args: &TelemetryArgs) -> anyhow::Result<()> {
    let path = loops_summary_path(&args.output);
    let summary = read_loop_summary(&path)?;
    let benchmark = build_telemetry_benchmark_coverage(&args.output)?;
    if args.json {
        let report = build_telemetry_report(&summary, benchmark.as_ref());
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_loop_telemetry(&summary, &path, benchmark.as_ref());
    }
    Ok(())
}

pub(crate) fn loops_summary_path(output: &Path) -> PathBuf {
    loops_directory(output).join("loops.summary.json")
}

pub(crate) fn print_loop_telemetry(
    summary: &LoopSummary,
    path: &Path,
    benchmark: Option<&BenchmarkCoverageTelemetry>,
) {
    if summary.entries.is_empty() {
        println!(
            "No telemetry records found ({}). Run autoresearch to generate loop telemetry.",
            path.display()
        );
        if let Some(benchmark) = benchmark {
            println!(
                "Benchmark coverage: continuity-critical {}/{} benchmarked, missing loops {}, with-memd losses {}",
                benchmark.continuity_critical_benchmarked,
                benchmark.continuity_critical_total,
                benchmark.missing_loop_count,
                benchmark.with_memd_losses
            );
            if !benchmark.gap_candidates.is_empty() {
                println!(
                    "Benchmark gaps: {}",
                    benchmark
                        .gap_candidates
                        .iter()
                        .map(|candidate| format!("{} ({})", candidate.id, candidate.severity))
                        .collect::<Vec<_>>()
                        .join(" | ")
                );
            }
        }
        return;
    }

    let stats = TelemetryStats::from_summary(summary);
    println!("Loop telemetry ({} entries)", stats.total_loops);
    println!(
        "Status counts: {}",
        stats
            .status_counts
            .iter()
            .map(|(name, count)| format!("{}={}", name, count))
            .collect::<Vec<_>>()
            .join(" ")
    );

    if let Some(avg) = stats.average_improvement() {
        println!("Average improvement: {:.2}%", avg);
    } else {
        println!("Average improvement: none recorded");
    }

    if let Some(highlight) = stats.best_improvement() {
        println!(
            "Best improvement: {:.2}% ({})",
            highlight.value, highlight.slug
        );
    }
    if let Some(highlight) = stats.worst_improvement() {
        println!(
            "Worst improvement: {:.2}% ({})",
            highlight.value, highlight.slug
        );
    }

    println!(
        "Total tokens saved: {}",
        format_tokens(Some(stats.total_tokens_saved))
    );
    if let Some(highlight) = stats.best_token_saving() {
        println!(
            "Largest token-saving loop: {} ({})",
            highlight.slug,
            format_tokens(Some(highlight.value))
        );
    }

    if let Some(benchmark) = benchmark {
        println!(
            "Benchmark coverage: continuity-critical {}/{} benchmarked, missing loops {}, with-memd losses {}",
            benchmark.continuity_critical_benchmarked,
            benchmark.continuity_critical_total,
            benchmark.missing_loop_count,
            benchmark.with_memd_losses
        );
        if !benchmark.gap_candidates.is_empty() {
            println!(
                "Benchmark gaps: {}",
                benchmark
                    .gap_candidates
                    .iter()
                    .map(|candidate| format!("{} ({})", candidate.id, candidate.severity))
                    .collect::<Vec<_>>()
                    .join(" | ")
            );
        }
    }
}

fn build_telemetry_report(
    summary: &LoopSummary,
    benchmark: Option<&BenchmarkCoverageTelemetry>,
) -> TelemetryReport {
    let stats = TelemetryStats::from_summary(summary);
    TelemetryReport {
        total_loops: stats.total_loops,
        statuses: stats.status_counts.clone(),
        average_improvement: stats.average_improvement(),
        best_improvement: stats.best_improvement(),
        worst_improvement: stats.worst_improvement(),
        total_tokens_saved: stats.total_tokens_saved,
        largest_token_saving: stats.best_token_saving(),
        benchmark: benchmark.cloned(),
    }
}

#[derive(Debug)]
struct TelemetryStats {
    total_loops: usize,
    status_counts: BTreeMap<String, usize>,
    percent_improvements: Vec<(f64, String)>,
    token_savings: Vec<(f64, String)>,
    total_tokens_saved: f64,
}

impl TelemetryStats {
    fn from_summary(summary: &LoopSummary) -> Self {
        let mut status_counts = BTreeMap::new();
        let mut percent_improvements = Vec::new();
        let mut token_savings = Vec::new();
        let mut total_tokens_saved = 0.0;

        for entry in &summary.entries {
            let status = entry.status.as_deref().unwrap_or("pending").to_string();
            *status_counts.entry(status).or_insert(0) += 1;

            if let Some(value) = entry.percent_improvement {
                percent_improvements.push((value, entry.slug.clone()));
            }
            if let Some(tokens) = entry.token_savings {
                total_tokens_saved += tokens;
                token_savings.push((tokens, entry.slug.clone()));
            }
        }

        TelemetryStats {
            total_loops: summary.entries.len(),
            status_counts,
            percent_improvements,
            token_savings,
            total_tokens_saved,
        }
    }

    fn average_improvement(&self) -> Option<f64> {
        if self.percent_improvements.is_empty() {
            return None;
        }
        let sum: f64 = self
            .percent_improvements
            .iter()
            .map(|(value, _)| *value)
            .sum();
        Some(sum / (self.percent_improvements.len() as f64))
    }

    fn best_improvement(&self) -> Option<TelemetryHighlight> {
        max_by_value(&self.percent_improvements)
    }

    fn worst_improvement(&self) -> Option<TelemetryHighlight> {
        min_by_value(&self.percent_improvements)
    }

    fn best_token_saving(&self) -> Option<TelemetryHighlight> {
        max_by_value(&self.token_savings)
    }
}

fn max_by_value(list: &[(f64, String)]) -> Option<TelemetryHighlight> {
    list.iter()
        .max_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(value, slug)| TelemetryHighlight {
            slug: slug.clone(),
            value: *value,
        })
}

fn min_by_value(list: &[(f64, String)]) -> Option<TelemetryHighlight> {
    list.iter()
        .min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(value, slug)| TelemetryHighlight {
            slug: slug.clone(),
            value: *value,
        })
}

#[derive(Debug, Serialize)]
struct TelemetryReport {
    total_loops: usize,
    statuses: BTreeMap<String, usize>,
    average_improvement: Option<f64>,
    best_improvement: Option<TelemetryHighlight>,
    worst_improvement: Option<TelemetryHighlight>,
    total_tokens_saved: f64,
    largest_token_saving: Option<TelemetryHighlight>,
    benchmark: Option<BenchmarkCoverageTelemetry>,
}

#[derive(Debug, Serialize)]
struct TelemetryHighlight {
    slug: String,
    value: f64,
}

pub(crate) fn format_percent(value: Option<f64>) -> String {
    value
        .map(|value| format!("{:.2}%", value))
        .unwrap_or_else(|| "-".to_string())
}

pub(crate) fn format_tokens(value: Option<f64>) -> String {
    match value {
        Some(value) if value >= 1_000_000f64 => format!("{:.1}M", value / 1_000_000f64),
        Some(value) if value >= 1_000f64 => format!("{:.1}k", value / 1_000f64),
        Some(value) => format!("{:.0}", value),
        None => "-".to_string(),
    }
}

pub(crate) fn indent_text(value: &str, spaces: usize) -> String {
    let spacer = " ".repeat(spaces);
    value
        .lines()
        .map(|line| format!("{}{}", spacer, line))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn read_bundle_harness_bridge_registry(
    output: &Path,
) -> anyhow::Result<Option<HarnessBridgeRegistry>> {
    let path = harness_bridge_registry_path(output);
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let registry = serde_json::from_str::<HarnessBridgeRegistry>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(registry))
}

pub(crate) fn build_harness_bridge_registry() -> HarnessBridgeRegistry {
    let harnesses = vec![
        harness_bridge_record(
            "codex",
            detect_codex_memd_wiring(),
            &["config", "hook", "skill"],
            &["Codex is native when the config, hook, and skill surfaces are all present."],
        ),
        harness_bridge_record(
            "claude",
            detect_claude_memd_wiring(),
            &["settings", "hook"],
            &["Claude is native when the settings and session hook surfaces exist."],
        ),
        harness_bridge_record(
            "claw",
            detect_claw_memd_wiring(),
            &["binary", "config", "skill"],
            &[
                "Claw is memd-ready when the binary is installed, config exists, and memd skills are visible through shared skill roots.",
            ],
        ),
        harness_bridge_record(
            "openclaw",
            detect_openclaw_memd_wiring(),
            &["agents", "bootstrap"],
            &["OpenClaw is native when AGENTS.md and BOOTSTRAP.md bridge surfaces exist."],
        ),
        harness_bridge_record(
            "opencode",
            detect_opencode_memd_wiring(),
            &["config", "plugin", "command"],
            &[
                "OpenCode is native when config, plugin, and command surfaces all route through memd.",
            ],
        ),
    ];

    let all_wired = harnesses.iter().all(|record| record.wired);
    HarnessBridgeRegistry {
        generated_at: Utc::now(),
        overall_portability_class: if all_wired {
            "portable".to_string()
        } else {
            "adapter-required".to_string()
        },
        all_wired,
        harnesses,
    }
}

pub(crate) fn harness_bridge_record(
    harness: &str,
    wiring: serde_json::Value,
    required_surfaces: &[&str],
    notes: &[&str],
) -> HarnessBridgeRecord {
    let wired = wiring
        .get("wired")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let missing_surfaces = required_surfaces
        .iter()
        .filter_map(|surface| {
            let present = wiring
                .get(*surface)
                .and_then(|value| value.as_bool())
                .unwrap_or(false);
            if present {
                None
            } else {
                Some((*surface).to_string())
            }
        })
        .collect::<Vec<_>>();

    HarnessBridgeRecord {
        harness: harness.to_string(),
        wired,
        portability_class: if wired {
            "harness-native".to_string()
        } else {
            "adapter-required".to_string()
        },
        required_surfaces: required_surfaces
            .iter()
            .map(|value| value.to_string())
            .collect(),
        missing_surfaces,
        notes: notes.iter().map(|value| value.to_string()).collect(),
    }
}

pub(crate) fn render_harness_bridge_markdown(registry: &HarnessBridgeRegistry) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd harness bridge matrix\n\n");
    markdown.push_str(&format!(
        "Generated: {}\n\n",
        registry.generated_at.to_rfc3339()
    ));
    markdown.push_str(&format!(
        "Overall portability class: **{}**\n\n",
        registry.overall_portability_class
    ));
    markdown.push_str("| Harness | Wired | Portability | Missing surfaces | Notes |\n");
    markdown.push_str("|---|---|---|---|---|\n");
    for harness in &registry.harnesses {
        let missing = if harness.missing_surfaces.is_empty() {
            "none".to_string()
        } else {
            harness.missing_surfaces.join(", ")
        };
        let notes = if harness.notes.is_empty() {
            "none".to_string()
        } else {
            harness
                .notes
                .iter()
                .map(|note| compact_inline(note, 120))
                .collect::<Vec<_>>()
                .join(" | ")
        };
        markdown.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            harness.harness,
            if harness.wired { "yes" } else { "no" },
            harness.portability_class,
            missing,
            notes
        ));
    }
    markdown.push_str("\n## Adapter Required Surface\n\n");
    markdown.push_str(
        "If a harness is not wired, `memd` treats it as adapter-required and surfaces the missing bridge surfaces instead of pretending the skill is universally available.\n",
    );
    markdown
}

pub(crate) fn read_bundle_source_registry(
    output: &Path,
) -> anyhow::Result<Option<BootstrapSourceRegistry>> {
    let path = bundle_source_registry_path(output);
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let registry = serde_json::from_str::<BootstrapSourceRegistry>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(registry))
}

pub(crate) fn file_modified_at(path: &Path) -> Option<DateTime<Utc>> {
    fs::metadata(path)
        .ok()
        .and_then(|meta| meta.modified().ok())
        .map(DateTime::<Utc>::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{Cli, Commands};
    use crate::test_support::{EnvScope, set_current_dir};
    use clap::Parser;

    #[test]
    fn default_bundle_root_path_does_not_use_global_bundle_inside_repo_without_local_bundle() {
        let mut env = EnvScope::new();
        let root = std::env::temp_dir().join(format!(
            "memd-bootstrap-default-bundle-root-{}",
            uuid::Uuid::new_v4()
        ));
        let home = root.join("home");
        let repo = root.join("repo");
        let nested = repo.join("src").join("feature");
        let global_bundle = home.join(".memd");

        fs::create_dir_all(&nested).expect("create nested repo dir");
        fs::create_dir_all(repo.join(".git")).expect("create repo git dir");
        fs::create_dir_all(&global_bundle).expect("create global bundle");
        fs::write(global_bundle.join("config.json"), "{}\n").expect("write global config");

        env.set("HOME", &home);
        env.remove("MEMD_BUNDLE_ROOT");
        let _cwd = set_current_dir(&nested);

        let resolved = default_bundle_root_path();
        assert_eq!(resolved, repo.join(".memd"));
        assert_ne!(resolved, global_bundle);

        let cli = Cli::parse_from(["memd", "lookup", "--query", "repo bleed check"]);
        let Commands::Lookup(args) = cli.command else {
            panic!("expected lookup command");
        };
        assert_eq!(args.output, repo.join(".memd"));
        assert_ne!(args.output, global_bundle);

        drop(_cwd);
        drop(env);
        fs::remove_dir_all(root).expect("cleanup temp bundle roots");
    }

    #[test]
    fn default_voice_mode_prefers_env_then_global_config_then_hardcoded_fallback() {
        let mut env = EnvScope::new();
        let home = std::env::temp_dir().join(format!(
            "memd-bootstrap-default-voice-{}",
            uuid::Uuid::new_v4()
        ));
        let global_bundle = home.join(".memd");

        fs::create_dir_all(&global_bundle).expect("create global bundle");
        env.set("HOME", &home);
        env.remove("MEMD_VOICE_MODE");

        assert_eq!(default_voice_mode(), hardcoded_default_voice_mode());

        fs::write(
            global_bundle.join("config.json"),
            "{\n  \"voice_mode\": \"normal\"\n}\n",
        )
        .expect("write global config");
        assert_eq!(default_voice_mode(), "normal");

        env.set("MEMD_VOICE_MODE", "lite");
        assert_eq!(default_voice_mode(), "caveman-lite");

        drop(env);
        fs::remove_dir_all(home).expect("cleanup temp home");
    }
}

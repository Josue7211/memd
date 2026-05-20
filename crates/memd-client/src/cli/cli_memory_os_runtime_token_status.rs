fn build_token_savings_report(output: &Path, since: Option<String>) -> TokenSavingsReport {
    let source_registry_path = output.join("state").join("source-registry.json");
    let ledger_path = token_savings_ledger_path(output);
    let ledger = read_token_savings_ledger(&ledger_path, since.as_deref());
    let savings_ledger = ledger
        .iter()
        .filter(|entry| entry.waste_kind.is_none())
        .collect::<Vec<_>>();
    let measured_input_tokens = ledger
        .iter()
        .filter(|entry| entry.waste_kind.is_none())
        .map(|entry| entry.baseline_input_tokens)
        .sum::<usize>();
    let measured_output_tokens = savings_ledger
        .iter()
        .map(|entry| entry.output_tokens)
        .sum::<usize>();
    let measured_tokens_saved = savings_ledger
        .iter()
        .map(|entry| entry.tokens_saved)
        .sum::<usize>();
    let source_reuse_events = ledger
        .iter()
        .filter(|entry| entry.operation == "source_read_avoided")
        .count();
    let source_reuse_tokens = ledger
        .iter()
        .filter(|entry| entry.operation == "source_read_avoided")
        .map(|entry| entry.tokens_saved)
        .sum::<usize>();
    let wasted_events = ledger
        .iter()
        .filter(|entry| entry.wasted_tokens > 0)
        .count();
    let wasted_tokens = ledger
        .iter()
        .map(|entry| entry.wasted_tokens)
        .sum::<usize>();
    let wasted_raw_reread_tokens = wasted_tokens_for_kind(&ledger, "raw_source_reread");
    let wasted_giant_diff_tokens = wasted_tokens_for_kind(&ledger, "giant_diff");
    let wasted_cache_exposure_tokens = wasted_tokens_for_kind(&ledger, "repo_cache_exposure");
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
        source_reuse_events,
        source_reuse_tokens,
        wasted_events,
        wasted_tokens,
        wasted_raw_reread_tokens,
        wasted_giant_diff_tokens,
        wasted_cache_exposure_tokens,
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
    report.server_events = server.total;
    report.server_measured_input_tokens = server.measured_input_tokens;
    report.server_measured_output_tokens = server.measured_output_tokens;
    report.server_measured_tokens_saved = server.measured_tokens_saved;
    if server.total == 0 && report.ledger_events > 0 {
        report.source = "local".to_string();
        report.notes.push(
            "server token ledger was empty; preserved local measured ledger instead of hiding dogfood evidence"
                .to_string(),
        );
        return report;
    }
    report.source = "server".to_string();
    report.ledger_events = server.total;
    report.measured_input_tokens = server.measured_input_tokens;
    report.measured_output_tokens = server.measured_output_tokens;
    report.measured_tokens_saved = server.measured_tokens_saved;
    report.source_reuse_events = server.source_reuse_events;
    report.source_reuse_tokens = server.source_reuse_tokens;
    if server.wasted_events > 0 || server.wasted_tokens > 0 {
        report.wasted_events = server.wasted_events;
        report.wasted_tokens = server.wasted_tokens;
        report.wasted_raw_reread_tokens = server.wasted_raw_reread_tokens;
        report.wasted_giant_diff_tokens = server.wasted_giant_diff_tokens;
        report.wasted_cache_exposure_tokens = server.wasted_cache_exposure_tokens;
    }
    report.notes.push(
        "server measured totals came from memd-server /tokens/savings; local ledger retained as fallback"
            .to_string(),
    );
    report.notes.push(
        "local wasted-token telemetry is retained until server sync supports waste counters"
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
        wasted_tokens: 0,
        waste_kind: None,
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
        wasted_tokens: 0,
        waste_kind: None,
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

pub(crate) fn record_wasted_token_event(
    output: &Path,
    waste_kind: &str,
    observed_chars: usize,
    reason: &str,
) -> anyhow::Result<Option<TokenSavingsLedgerEntry>> {
    let wasted_tokens = estimate_text_tokens_from_chars(observed_chars);
    if wasted_tokens == 0 {
        return Ok(None);
    }
    let entry = TokenSavingsLedgerEntry {
        id: uuid::Uuid::new_v4(),
        ts: Utc::now(),
        operation: "token_waste_observed".to_string(),
        project: None,
        agent: None,
        model_tier: None,
        intent: Some("TokenWaste".to_string()),
        source_records: 0,
        baseline_input_tokens: wasted_tokens,
        output_tokens: 0,
        tokens_saved: 0,
        wasted_tokens,
        waste_kind: Some(waste_kind.trim().to_string()),
        reason: reason.trim().to_string(),
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

fn wasted_tokens_for_kind(ledger: &[TokenSavingsLedgerEntry], kind: &str) -> usize {
    ledger
        .iter()
        .filter(|entry| entry.waste_kind.as_deref() == Some(kind))
        .map(|entry| entry.wasted_tokens)
        .sum()
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
            wasted_tokens: entry.wasted_tokens,
            waste_kind: entry.waste_kind,
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
#[path = "cli_memory_os_runtime_tests.rs"]
mod cli_memory_os_runtime_tests;

fn path_evidence(label: &str, path: &Path) -> String {
    format!(
        "{label}:{}:{}",
        if path.exists() { "present" } else { "missing" },
        path.display()
    )
}

fn feature(id: &str, status: &str, evidence: Vec<String>, gaps: Vec<String>) -> MemoryOsFeature {
    let axes = feature_axes(id, status, &gaps);
    MemoryOsFeature {
        id: id.to_string(),
        status: status.to_string(),
        implementation_status: axes.implementation_status,
        dogfood_status: axes.dogfood_status,
        proof_status: axes.proof_status,
        market_status: axes.market_status,
        hygiene_status: axes.hygiene_status,
        token_risk: axes.token_risk,
        evidence,
        gaps,
    }
}

#[derive(Debug, Clone)]
struct FeatureAxes {
    implementation_status: String,
    dogfood_status: String,
    proof_status: String,
    market_status: String,
    hygiene_status: String,
    token_risk: String,
}

fn feature_axes(id: &str, status: &str, gaps: &[String]) -> FeatureAxes {
    let hygiene_status = if id == "repo_hygiene" {
        if status == "working" {
            "clean"
        } else if gaps
            .iter()
            .any(|gap| gap.contains("raw benchmark cache") || gap.contains("cache path"))
        {
            "broken"
        } else {
            "noisy"
        }
    } else if gaps.iter().any(|gap| {
        gap.contains("raw benchmark cache")
            || gap.contains("repo-visible")
            || gap.contains("cache path")
    }) {
        "broken"
    } else {
        "clean"
    };
    let token_risk = if hygiene_status == "broken" || status == "broken" {
        "high"
    } else if hygiene_status == "noisy" || status == "partial" || status == "unproven" {
        "medium"
    } else {
        "low"
    };
    let proof_status = match id {
        "proof_gates" => "focused",
        "capability_sync" | "access_secret_routes" | "server_authority" => {
            if status == "working" {
                "sampled"
            } else {
                "focused"
            }
        }
        _ if status == "unproven" => "blocked",
        _ => "focused",
    };
    let dogfood_status = if id == "repo_hygiene" {
        if hygiene_status == "clean" {
            "working"
        } else {
            status
        }
    } else {
        status
    };

    FeatureAxes {
        implementation_status: status.to_string(),
        dogfood_status: dogfood_status.to_string(),
        proof_status: proof_status.to_string(),
        market_status: "blocked".to_string(),
        hygiene_status: hygiene_status.to_string(),
        token_risk: token_risk.to_string(),
    }
}

fn aggregate_hygiene_status(features: &[MemoryOsFeature]) -> String {
    if features
        .iter()
        .any(|feature| feature.hygiene_status == "broken")
    {
        "broken".to_string()
    } else if features
        .iter()
        .any(|feature| feature.hygiene_status == "noisy")
    {
        "noisy".to_string()
    } else {
        "clean".to_string()
    }
}

fn aggregate_token_risk(features: &[MemoryOsFeature]) -> String {
    if features.iter().any(|feature| feature.token_risk == "high") {
        "high".to_string()
    } else if features
        .iter()
        .any(|feature| feature.token_risk == "medium")
    {
        "medium".to_string()
    } else {
        "low".to_string()
    }
}

pub(crate) fn read_memory_os_bundle_config(output: &Path) -> anyhow::Result<BundleConfigFile> {
    let config_path = output.join("config.json");
    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))
}

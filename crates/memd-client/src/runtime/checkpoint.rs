use super::*;
use uuid::Uuid;

pub(crate) fn append_raw_spine_record(
    output: &Path,
    event_type: &str,
    stage: &str,
    project: Option<&str>,
    namespace: Option<&str>,
    workspace: Option<&str>,
    source_system: Option<&str>,
    source_path: Option<&str>,
    confidence: Option<f32>,
    tags: &[String],
    content: &str,
) -> anyhow::Result<()> {
    let tag_refs = tags.iter().map(String::as_str).collect::<Vec<_>>();
    let record = derive_raw_spine_record(
        event_type,
        stage,
        source_system,
        source_path,
        project,
        namespace,
        workspace,
        confidence,
        &tag_refs,
        content,
    );
    write_raw_spine_records(output, &[record])
}

pub(crate) fn infer_bundle_identity_defaults(output: &Path) -> (Option<String>, Option<String>) {
    let Some(project_root) = infer_bundle_project_root(output) else {
        return (None, None);
    };

    let project = project_root
        .file_name()
        .and_then(|value| value.to_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let namespace = project.as_ref().map(|_| "main".to_string());
    (project, namespace)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct OfflineStoreQueueEntry {
    pub(crate) id: Uuid,
    pub(crate) dedup_key: String,
    pub(crate) queued_at: chrono::DateTime<chrono::Utc>,
    pub(crate) attempts: u32,
    pub(crate) status: String,
    pub(crate) last_error: Option<String>,
    pub(crate) request: memd_schema::StoreMemoryRequest,
    pub(crate) synced_item_id: Option<Uuid>,
    pub(crate) synced_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct OfflineStoreQueueStatus {
    pub(crate) path: PathBuf,
    pub(crate) total: usize,
    pub(crate) pending: usize,
    pub(crate) synced: usize,
    pub(crate) failed: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct OfflineStoreReplayReport {
    pub(crate) path: PathBuf,
    pub(crate) attempted: usize,
    pub(crate) synced: usize,
    pub(crate) failed: usize,
    pub(crate) pending: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "request")]
pub(crate) enum OfflineSyncPayload {
    Capabilities(memd_schema::CapabilitySyncRequest),
    AccessRoutes(memd_schema::AccessRouteSyncRequest),
    TokenSavings(memd_schema::TokenSavingsSyncRequest),
    Candidates(Vec<memd_schema::CandidateMemoryRequest>),
}

impl OfflineSyncPayload {
    pub(crate) fn kind_label(&self) -> &'static str {
        match self {
            OfflineSyncPayload::Capabilities(_) => "capabilities",
            OfflineSyncPayload::AccessRoutes(_) => "access_routes",
            OfflineSyncPayload::TokenSavings(_) => "token_savings",
            OfflineSyncPayload::Candidates(_) => "candidates",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct OfflineSyncQueueEntry {
    pub(crate) id: Uuid,
    pub(crate) dedup_key: String,
    pub(crate) queued_at: chrono::DateTime<chrono::Utc>,
    pub(crate) attempts: u32,
    pub(crate) status: String,
    pub(crate) last_error: Option<String>,
    pub(crate) payload: OfflineSyncPayload,
    pub(crate) synced_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct OfflineSyncQueueStatus {
    pub(crate) path: PathBuf,
    pub(crate) total: usize,
    pub(crate) pending: usize,
    pub(crate) synced: usize,
    pub(crate) failed: usize,
    pub(crate) by_kind: std::collections::BTreeMap<String, OfflineSyncKindStatus>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub(crate) struct OfflineSyncKindStatus {
    pub(crate) total: usize,
    pub(crate) pending: usize,
    pub(crate) synced: usize,
    pub(crate) failed: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct OfflineQueueStatus {
    pub(crate) store: OfflineStoreQueueStatus,
    pub(crate) sync: OfflineSyncQueueStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct OfflineQueueReplayReport {
    pub(crate) store: OfflineStoreReplayReport,
    pub(crate) sync: OfflineSyncReplayReport,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct OfflineSyncReplayReport {
    pub(crate) path: PathBuf,
    pub(crate) attempted: usize,
    pub(crate) synced: usize,
    pub(crate) failed: usize,
    pub(crate) pending: usize,
}

pub(crate) fn offline_store_queue_path(output: &Path) -> PathBuf {
    output.join("state").join("offline-store-queue.jsonl")
}

pub(crate) fn offline_sync_queue_path(output: &Path) -> PathBuf {
    output.join("state").join("offline-sync-queue.jsonl")
}

pub(crate) fn offline_queue_status(output: &Path) -> anyhow::Result<OfflineQueueStatus> {
    Ok(OfflineQueueStatus {
        store: offline_store_queue_status(output)?,
        sync: offline_sync_queue_status(output)?,
    })
}

pub(crate) fn offline_store_queue_status(output: &Path) -> anyhow::Result<OfflineStoreQueueStatus> {
    let path = offline_store_queue_path(output);
    let entries = read_offline_store_queue(output)?;
    let pending = entries
        .iter()
        .filter(|entry| entry.status == "pending")
        .count();
    let synced = entries
        .iter()
        .filter(|entry| entry.status == "synced")
        .count();
    let failed = entries
        .iter()
        .filter(|entry| entry.status == "failed")
        .count();
    Ok(OfflineStoreQueueStatus {
        path,
        total: entries.len(),
        pending,
        synced,
        failed,
    })
}

pub(crate) fn read_offline_store_queue(
    output: &Path,
) -> anyhow::Result<Vec<OfflineStoreQueueEntry>> {
    let path = offline_store_queue_path(output);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("read offline store queue {}", path.display()))?;
    raw.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| {
            serde_json::from_str::<OfflineStoreQueueEntry>(line).with_context(|| {
                format!(
                    "parse offline store queue {} line {}",
                    path.display(),
                    index + 1
                )
            })
        })
        .collect()
}

pub(crate) fn offline_sync_queue_status(output: &Path) -> anyhow::Result<OfflineSyncQueueStatus> {
    let path = offline_sync_queue_path(output);
    let entries = read_offline_sync_queue(output)?;
    let pending = entries
        .iter()
        .filter(|entry| entry.status == "pending")
        .count();
    let synced = entries
        .iter()
        .filter(|entry| entry.status == "synced")
        .count();
    let failed = entries
        .iter()
        .filter(|entry| entry.status == "failed")
        .count();
    let mut by_kind = std::collections::BTreeMap::<String, OfflineSyncKindStatus>::new();
    for entry in &entries {
        let status = by_kind
            .entry(entry.payload.kind_label().to_string())
            .or_default();
        status.total += 1;
        match entry.status.as_str() {
            "pending" => status.pending += 1,
            "synced" => status.synced += 1,
            "failed" => status.failed += 1,
            _ => {}
        }
    }
    Ok(OfflineSyncQueueStatus {
        path,
        total: entries.len(),
        pending,
        synced,
        failed,
        by_kind,
    })
}

pub(crate) fn read_offline_sync_queue(output: &Path) -> anyhow::Result<Vec<OfflineSyncQueueEntry>> {
    let path = offline_sync_queue_path(output);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("read offline sync queue {}", path.display()))?;
    raw.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| {
            serde_json::from_str::<OfflineSyncQueueEntry>(line).with_context(|| {
                format!(
                    "parse offline sync queue {} line {}",
                    path.display(),
                    index + 1
                )
            })
        })
        .collect()
}

fn write_offline_sync_queue(
    output: &Path,
    entries: &[OfflineSyncQueueEntry],
) -> anyhow::Result<()> {
    let path = offline_sync_queue_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create offline sync queue dir {}", parent.display()))?;
    }
    let body = entries
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");
    fs::write(&path, format!("{body}\n"))
        .with_context(|| format!("write offline sync queue {}", path.display()))
}

fn write_offline_store_queue(
    output: &Path,
    entries: &[OfflineStoreQueueEntry],
) -> anyhow::Result<()> {
    let path = offline_store_queue_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create offline queue dir {}", parent.display()))?;
    }
    let body = entries
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");
    fs::write(&path, format!("{body}\n"))
        .with_context(|| format!("write offline store queue {}", path.display()))
}

fn offline_store_dedup_key(req: &memd_schema::StoreMemoryRequest) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    let tags = {
        let mut tags = req.tags.clone();
        tags.sort();
        tags.dedup();
        tags
    };
    let normalized = serde_json::json!({
        "content": req.content.split_whitespace().collect::<Vec<_>>().join(" ").to_ascii_lowercase(),
        "kind": req.kind,
        "scope": req.scope,
        "project": req.project,
        "namespace": req.namespace,
        "workspace": req.workspace,
        "visibility": req.visibility,
        "source_agent": req.source_agent,
        "source_system": req.source_system,
        "source_path": req.source_path,
        "tags": tags,
    });
    hasher.update(normalized.to_string().as_bytes());
    format!("{:x}", hasher.finalize())
}

pub(crate) fn queue_offline_store_request(
    output: &Path,
    req: &memd_schema::StoreMemoryRequest,
    error: &str,
) -> anyhow::Result<OfflineStoreQueueEntry> {
    let mut entries = read_offline_store_queue(output)?;
    let dedup_key = offline_store_dedup_key(req);
    if let Some(existing) = entries
        .iter()
        .find(|entry| entry.status != "synced" && entry.dedup_key == dedup_key)
    {
        return Ok(existing.clone());
    }
    let entry = OfflineStoreQueueEntry {
        id: Uuid::new_v4(),
        dedup_key,
        queued_at: chrono::Utc::now(),
        attempts: 0,
        status: "pending".to_string(),
        last_error: Some(error.to_string()),
        request: req.clone(),
        synced_item_id: None,
        synced_at: None,
    };
    entries.push(entry.clone());
    write_offline_store_queue(output, &entries)?;
    Ok(entry)
}

fn offline_sync_dedup_key(payload: &OfflineSyncPayload) -> anyhow::Result<String> {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(serde_json::to_vec(payload)?);
    Ok(format!("{:x}", hasher.finalize()))
}

pub(crate) fn queue_offline_sync_payload(
    output: &Path,
    payload: OfflineSyncPayload,
    error: &str,
) -> anyhow::Result<OfflineSyncQueueEntry> {
    let mut entries = read_offline_sync_queue(output)?;
    let dedup_key = offline_sync_dedup_key(&payload)?;
    if let Some(existing) = entries
        .iter()
        .find(|entry| entry.status != "synced" && entry.dedup_key == dedup_key)
    {
        return Ok(existing.clone());
    }
    let entry = OfflineSyncQueueEntry {
        id: Uuid::new_v4(),
        dedup_key,
        queued_at: chrono::Utc::now(),
        attempts: 0,
        status: "pending".to_string(),
        last_error: Some(error.to_string()),
        payload,
        synced_at: None,
    };
    entries.push(entry.clone());
    write_offline_sync_queue(output, &entries)?;
    Ok(entry)
}

pub(crate) async fn replay_offline_store_queue(
    output: &Path,
    client: &MemdClient,
) -> anyhow::Result<OfflineStoreReplayReport> {
    let mut entries = read_offline_store_queue(output)?;
    let mut attempted = 0usize;
    let mut synced = 0usize;
    let mut failed = 0usize;
    for entry in entries.iter_mut().filter(|entry| entry.status != "synced") {
        attempted += 1;
        entry.attempts = entry.attempts.saturating_add(1);
        match client.store(&entry.request).await {
            Ok(response) => {
                entry.status = "synced".to_string();
                entry.synced_item_id = Some(response.item.id);
                entry.synced_at = Some(chrono::Utc::now());
                entry.last_error = None;
                synced += 1;
            }
            Err(error) => {
                entry.status = "failed".to_string();
                entry.last_error = Some(format!("{error:#}"));
                failed += 1;
            }
        }
    }
    write_offline_store_queue(output, &entries)?;
    let pending = entries
        .iter()
        .filter(|entry| entry.status != "synced")
        .count();
    Ok(OfflineStoreReplayReport {
        path: offline_store_queue_path(output),
        attempted,
        synced,
        failed,
        pending,
    })
}

pub(crate) async fn replay_offline_sync_queue(
    output: &Path,
    client: &MemdClient,
) -> anyhow::Result<OfflineSyncReplayReport> {
    let mut entries = read_offline_sync_queue(output)?;
    let mut attempted = 0usize;
    let mut synced = 0usize;
    let mut failed = 0usize;
    for entry in entries.iter_mut().filter(|entry| entry.status != "synced") {
        attempted += 1;
        entry.attempts = entry.attempts.saturating_add(1);
        let result = match &entry.payload {
            OfflineSyncPayload::Capabilities(req) => {
                client.capabilities_sync(req).await.map(|_| ())
            }
            OfflineSyncPayload::AccessRoutes(req) => {
                client.access_routes_sync(req).await.map(|_| ())
            }
            OfflineSyncPayload::TokenSavings(req) => {
                client.token_savings_sync(req).await.map(|_| ())
            }
            OfflineSyncPayload::Candidates(reqs) => client.candidate_batch(reqs).await.map(|_| ()),
        };
        match result {
            Ok(()) => {
                entry.status = "synced".to_string();
                entry.synced_at = Some(chrono::Utc::now());
                entry.last_error = None;
                synced += 1;
            }
            Err(error) => {
                entry.status = "failed".to_string();
                entry.last_error = Some(format!("{error:#}"));
                failed += 1;
            }
        }
    }
    write_offline_sync_queue(output, &entries)?;
    let pending = entries
        .iter()
        .filter(|entry| entry.status != "synced")
        .count();
    Ok(OfflineSyncReplayReport {
        path: offline_sync_queue_path(output),
        attempted,
        synced,
        failed,
        pending,
    })
}

pub(crate) async fn replay_offline_queue(
    output: &Path,
    client: &MemdClient,
) -> anyhow::Result<OfflineQueueReplayReport> {
    Ok(OfflineQueueReplayReport {
        store: replay_offline_store_queue(output, client).await?,
        sync: replay_offline_sync_queue(output, client).await?,
    })
}

pub(crate) fn offline_queued_response(
    req: &memd_schema::StoreMemoryRequest,
    queue_id: Uuid,
) -> memd_schema::StoreMemoryResponse {
    let now = chrono::Utc::now();
    let mut tags = req.tags.clone();
    if !tags.iter().any(|tag| tag == "offline:queued") {
        tags.push("offline:queued".to_string());
    }
    memd_schema::StoreMemoryResponse {
        item: memd_schema::MemoryItem {
            id: queue_id,
            content: req.content.clone(),
            redundancy_key: None,
            belief_branch: req.belief_branch.clone(),
            preferred: false,
            kind: req.kind,
            scope: req.scope,
            project: req.project.clone(),
            namespace: req.namespace.clone(),
            workspace: req.workspace.clone(),
            visibility: req.visibility.unwrap_or_default(),
            source_agent: req.source_agent.clone(),
            source_system: req.source_system.clone(),
            source_path: req.source_path.clone(),
            source_quality: req.source_quality,
            confidence: req.confidence.unwrap_or(0.5).min(0.5),
            ttl_seconds: req.ttl_seconds,
            created_at: now,
            updated_at: now,
            last_verified_at: req.last_verified_at,
            supersedes: req.supersedes.clone(),
            tags,
            status: req.status.unwrap_or(MemoryStatus::Active),
            stage: memd_schema::MemoryStage::Candidate,
            lane: req.lane.clone(),
            version: 1,
            correction_meta: None,
        },
    }
}

pub(crate) fn is_offline_queued_response(response: &memd_schema::StoreMemoryResponse) -> bool {
    response.item.tags.iter().any(|tag| tag == "offline:queued")
}

pub(crate) fn should_queue_offline_store_error(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        cause
            .downcast_ref::<reqwest::Error>()
            .is_some_and(|err| err.is_connect() || err.is_timeout() || err.is_request())
    })
}

pub(crate) async fn remember_with_bundle_defaults(
    args: &RememberArgs,
    base_url: &str,
) -> anyhow::Result<memd_schema::StoreMemoryResponse> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    let session = runtime.as_ref().and_then(|config| config.session.clone());
    let (inferred_project, inferred_namespace) = infer_bundle_identity_defaults(&args.output);
    let project = args
        .project
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.project.clone()))
        .or(inferred_project);
    let namespace = args
        .namespace
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.namespace.clone()))
        .or(inferred_namespace);
    let workspace = args
        .workspace
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.workspace.clone()));
    let visibility_raw = args.visibility.clone().or_else(|| {
        runtime
            .as_ref()
            .and_then(|config| config.visibility.clone())
    });
    let base_url = resolve_bundle_command_base_url(
        base_url,
        runtime
            .as_ref()
            .and_then(|config| config.base_url.as_deref()),
    );
    let source_agent = args
        .source_agent
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.agent.clone()))
        .as_deref()
        .map(|value| compose_agent_identity(value, session.as_deref()));

    let content = if let Some(content) = &args.content {
        content.clone()
    } else if let Some(path) = &args.input {
        fs::read_to_string(path)
            .with_context(|| format!("read remember input file {}", path.display()))?
    } else if args.stdin {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("read remember payload from stdin")?;
        buffer
    } else {
        anyhow::bail!("provide --content, --input, or --stdin");
    };

    let kind = args
        .kind
        .as_deref()
        .map(parse_memory_kind_value)
        .transpose()?
        .unwrap_or(MemoryKind::Fact);
    let scope = args
        .scope
        .as_deref()
        .map(parse_memory_scope_value)
        .transpose()?
        .unwrap_or_else(|| {
            if project.is_some() {
                MemoryScope::Project
            } else {
                MemoryScope::Synced
            }
        });
    let source_quality = args
        .source_quality
        .as_deref()
        .map(parse_source_quality_value)
        .transpose()?;
    let visibility = visibility_raw
        .as_deref()
        .map(parse_memory_visibility_value)
        .transpose()?;
    let supersedes = parse_uuid_list(&args.supersede)?;

    let client = MemdClient::new(&base_url)?;
    let store_req = memd_schema::StoreMemoryRequest {
        content,
        kind,
        scope,
        project,
        namespace,
        workspace,
        visibility,
        belief_branch: None,
        source_agent,
        source_system: args.source_system.clone().or(Some("memd".to_string())),
        source_path: args.source_path.clone(),
        source_quality,
        confidence: args.confidence,
        ttl_seconds: args.ttl_seconds,
        last_verified_at: None,
        supersedes,
        tags: args.tag.clone(),
        status: Some(MemoryStatus::Active),
        lane: None,
    };
    let response = match client.store(&store_req).await {
        Ok(response) => response,
        Err(error) if should_queue_offline_store_error(&error) => {
            let entry =
                queue_offline_store_request(&args.output, &store_req, &format!("{error:#}"))?;
            offline_queued_response(&store_req, entry.id)
        }
        Err(error) => return Err(error),
    };

    append_raw_spine_record(
        &args.output,
        if is_offline_queued_response(&response) {
            "remember_offline_queued"
        } else {
            "remember"
        },
        if is_offline_queued_response(&response) {
            "candidate"
        } else {
            "canonical"
        },
        response.item.project.as_deref(),
        response.item.namespace.as_deref(),
        response.item.workspace.as_deref(),
        response.item.source_system.as_deref(),
        response.item.source_path.as_deref(),
        Some(response.item.confidence),
        &response.item.tags,
        &response.item.content,
    )?;

    Ok(response)
}

pub(crate) async fn checkpoint_with_bundle_defaults(
    args: &CheckpointArgs,
    base_url: &str,
) -> anyhow::Result<memd_schema::StoreMemoryResponse> {
    if args.auto_commit {
        let suffix = args
            .content
            .as_deref()
            .map(|value| value.chars().take(72).collect::<String>())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "checkpoint".to_string());
        let commit_msg = format!("memd auto-commit: {suffix}");
        if let Some(repo_root) = infer_bundle_project_root(&args.output) {
            git_auto_commit_if_dirty_in(&commit_msg, Some(&repo_root))?;
        } else {
            git_auto_commit_if_dirty(&commit_msg)?;
        }
    }
    let translated = checkpoint_as_remember_args(args);
    let response = remember_with_bundle_defaults(&translated, base_url).await?;
    append_raw_spine_record(
        &args.output,
        "checkpoint",
        "candidate",
        translated.project.as_deref(),
        translated.namespace.as_deref(),
        translated.workspace.as_deref(),
        translated.source_system.as_deref(),
        translated.source_path.as_deref(),
        translated.confidence,
        &translated.tag,
        translated.content.as_deref().unwrap_or_default(),
    )?;
    Ok(response)
}

pub(crate) fn remember_args_from_hook_capture(
    args: &HookCaptureArgs,
    content: String,
) -> RememberArgs {
    let tags = if args.promote_tag.is_empty() {
        vec![
            "promoted".to_string(),
            "durable-memory".to_string(),
            "from-hook-capture".to_string(),
        ]
    } else {
        args.promote_tag.clone()
    };

    RememberArgs {
        output: args.output.clone(),
        project: args.project.clone(),
        namespace: args.namespace.clone(),
        workspace: args.workspace.clone(),
        visibility: args.visibility.clone(),
        kind: args.promote_kind.clone(),
        scope: args.promote_scope.clone(),
        source_agent: None,
        source_system: Some("memd".to_string()),
        source_path: args
            .source_path
            .clone()
            .or(Some("hook-capture-promotion".to_string())),
        source_quality: Some("canonical".to_string()),
        confidence: args.promote_confidence.or(args.confidence),
        ttl_seconds: None,
        tag: tags,
        supersede: args.promote_supersede.clone(),
        content: Some(content),
        input: None,
        stdin: false,
    }
}

pub(crate) fn infer_promote_kind_from_capture(content: &str) -> Option<&'static str> {
    let trimmed = content.trim_start();
    let normalized = trimmed.to_ascii_lowercase();
    for kind in [
        "decision",
        "preference",
        "constraint",
        "fact",
        "runbook",
        "procedural",
        "status",
    ] {
        let prefix = format!("{kind}:");
        if normalized.starts_with(&prefix) {
            return Some(kind);
        }
    }
    None
}

pub(crate) fn effective_hook_capture_promote_kind(
    args: &HookCaptureArgs,
    content: &str,
) -> Option<String> {
    args.promote_kind
        .clone()
        .or_else(|| infer_promote_kind_from_capture(content).map(str::to_string))
}

pub(crate) fn infer_supersede_query_from_capture(content: &str) -> Option<String> {
    let trimmed = content.trim();
    let normalized = trimmed.to_ascii_lowercase();
    let prefixes = [
        "corrected fact:",
        "corrected decision:",
        "corrected preference:",
        "corrected constraint:",
        "correction:",
    ];
    for prefix in prefixes {
        if normalized.starts_with(prefix) {
            let query = trimmed[prefix.len()..].trim();
            if !query.is_empty() {
                return Some(query.to_string());
            }
        }
    }
    None
}

pub(crate) fn effective_hook_capture_supersede_query(
    args: &HookCaptureArgs,
    content: &str,
) -> Option<String> {
    args.promote_supersede_query.clone().or_else(|| {
        if args.promote_supersede.is_empty() {
            infer_supersede_query_from_capture(content)
        } else {
            None
        }
    })
}

pub(crate) fn condensed_supersede_query(query: &str) -> Option<String> {
    let tokens = supersede_query_keywords(query)
        .into_iter()
        .take(5)
        .collect::<Vec<_>>();
    if tokens.len() >= 2 {
        Some(tokens.join(" "))
    } else {
        None
    }
}

pub(crate) fn supersede_query_keywords(query: &str) -> Vec<String> {
    let stopwords = [
        "does", "do", "did", "not", "prove", "should", "would", "could", "must", "keep", "that",
        "this", "from", "with", "into", "about", "memory", "agent", "usable",
    ];
    query
        .split(|c: char| !c.is_ascii_alphanumeric())
        .map(|token| token.trim().to_ascii_lowercase())
        .filter(|token| token.len() >= 4)
        .filter(|token| !stopwords.iter().any(|stop| stop == token))
        .collect::<Vec<_>>()
}

pub(crate) fn supersede_query_candidates(query: &str) -> Vec<String> {
    let mut candidates = vec![query.trim().to_string()];
    if let Some(condensed) = condensed_supersede_query(query)
        && !candidates.iter().any(|existing| existing == &condensed)
    {
        candidates.push(condensed);
    }
    let keywords = supersede_query_keywords(query);
    for window in keywords.windows(2).take(3) {
        let candidate = window.join(" ");
        if !candidates.iter().any(|existing| existing == &candidate) {
            candidates.push(candidate);
        }
    }
    for window in keywords.windows(3).take(2) {
        let candidate = window.join(" ");
        if !candidates.iter().any(|existing| existing == &candidate) {
            candidates.push(candidate);
        }
    }
    candidates
}

pub(crate) fn infer_promote_tags_from_capture(
    promote_kind: &str,
    content: &str,
    has_supersedes: bool,
) -> Vec<String> {
    let normalized = content.to_ascii_lowercase();
    let mut tags = vec![
        "promoted".to_string(),
        "durable-memory".to_string(),
        "from-hook-capture".to_string(),
        "auto-promoted".to_string(),
        promote_kind.to_string(),
    ];

    if has_supersedes
        || normalized.starts_with("corrected ")
        || normalized.starts_with("correction:")
    {
        tags.push("correction".to_string());
    }
    if normalized.contains("design")
        || normalized.contains(" ux")
        || normalized.contains("ux/")
        || normalized.contains(" ui")
        || normalized.contains("ui/")
    {
        tags.push("design-memory".to_string());
    }
    if normalized.contains("product direction")
        || normalized.contains("startup surface")
        || normalized.contains("memory loop")
        || normalized.contains("live memory")
    {
        tags.push("product-direction".to_string());
    }

    tags.sort();
    tags.dedup();
    tags
}

pub(crate) fn remember_args_from_effective_hook_capture(
    args: &HookCaptureArgs,
    content: String,
    promote_kind: String,
    supersedes: Vec<uuid::Uuid>,
) -> RememberArgs {
    let mut remember = remember_args_from_hook_capture(args, content);
    remember.kind = Some(promote_kind);
    remember.supersede = supersedes.into_iter().map(|id| id.to_string()).collect();
    if args.promote_tag.is_empty() {
        remember.tag = infer_promote_tags_from_capture(
            remember.kind.as_deref().unwrap_or("fact"),
            remember.content.as_deref().unwrap_or(""),
            !remember.supersede.is_empty(),
        );
    }
    remember
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SupersedeSearchDiagnostics {
    pub(crate) inferred_query: Option<String>,
    pub(crate) tried_queries: Vec<String>,
    pub(crate) matched_ids: Vec<String>,
    pub(crate) candidate_hits: Vec<SupersedeCandidateHit>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SupersedeCandidateHit {
    pub(crate) query: String,
    pub(crate) ids: Vec<String>,
    pub(crate) statuses: Vec<String>,
    pub(crate) kinds: Vec<String>,
    pub(crate) previews: Vec<String>,
}

pub(crate) async fn find_hook_capture_supersede_targets(
    base_url: &str,
    args: &HookCaptureArgs,
    content: &str,
) -> anyhow::Result<(Vec<uuid::Uuid>, SupersedeSearchDiagnostics)> {
    let mut superseded_ids = parse_uuid_list(&args.promote_supersede)?;
    let inferred_query = effective_hook_capture_supersede_query(args, content);
    let mut tried_queries = Vec::new();
    let mut candidate_hits = Vec::new();
    if let Some(query) = inferred_query.as_deref() {
        let result = search_supersede_candidates(base_url, args, query).await?;
        tried_queries = result.tried_queries;
        candidate_hits = result.candidate_hits;
        superseded_ids.extend(
            result
                .items
                .into_iter()
                .filter(|item| matches!(item.status, MemoryStatus::Stale | MemoryStatus::Contested))
                .map(|item| item.id),
        );
    }
    superseded_ids.sort();
    superseded_ids.dedup();
    Ok((
        superseded_ids,
        SupersedeSearchDiagnostics {
            inferred_query,
            tried_queries,
            matched_ids: Vec::new(),
            candidate_hits,
        },
    ))
}

pub(crate) async fn mark_hook_capture_supersede_targets(
    base_url: &str,
    args: &HookCaptureArgs,
    superseded_ids: &[uuid::Uuid],
    promoted_id: uuid::Uuid,
) -> anyhow::Result<Vec<memd_schema::RepairMemoryResponse>> {
    if superseded_ids.is_empty() {
        return Ok(Vec::new());
    }
    let visibility = args
        .visibility
        .as_deref()
        .map(parse_memory_visibility_value)
        .transpose()?;
    let client = MemdClient::new(base_url)?;
    let mut responses = Vec::with_capacity(superseded_ids.len());
    for id in superseded_ids {
        let response = client
            .repair(&RepairMemoryRequest {
                id: *id,
                mode: MemoryRepairMode::Supersede,
                confidence: Some(0.25),
                status: Some(MemoryStatus::Superseded),
                workspace: args.workspace.clone(),
                visibility,
                source_agent: None,
                source_system: Some("memd-correction".to_string()),
                source_path: args
                    .source_path
                    .clone()
                    .or(Some("hook-capture-promotion".to_string())),
                source_quality: Some(memd_schema::SourceQuality::Canonical),
                content: None,
                tags: Some(vec![
                    "superseded".to_string(),
                    "from-hook-capture".to_string(),
                    promoted_id.to_string(),
                ]),
                supersedes: vec![],
            })
            .await?;
        responses.push(response);
    }
    Ok(responses)
}

pub(crate) struct SupersedeSearchResult {
    items: Vec<memd_schema::MemoryItem>,
    pub(crate) tried_queries: Vec<String>,
    pub(crate) candidate_hits: Vec<SupersedeCandidateHit>,
}

pub(crate) async fn search_supersede_candidates(
    base_url: &str,
    args: &HookCaptureArgs,
    query: &str,
) -> anyhow::Result<SupersedeSearchResult> {
    let visibility = args
        .visibility
        .as_deref()
        .map(parse_memory_visibility_value)
        .transpose()?;
    let client = MemdClient::new(base_url)?;
    let kinds = if let Some(kind) = effective_hook_capture_promote_kind(args, query) {
        vec![parse_memory_kind_value(&kind)?]
    } else {
        default_supersede_search_kinds()
    };
    let mut tried_queries = Vec::new();
    let mut candidate_hits = Vec::new();
    for candidate_query in supersede_query_candidates(query) {
        tried_queries.push(candidate_query.clone());
        let response = client
            .search(&SearchMemoryRequest {
                query: Some(candidate_query.clone()),
                route: Some(RetrievalRoute::ProjectFirst),
                intent: Some(RetrievalIntent::General),
                scopes: vec![MemoryScope::Project, MemoryScope::Synced],
                kinds: kinds.clone(),
                statuses: vec![
                    MemoryStatus::Active,
                    MemoryStatus::Stale,
                    MemoryStatus::Contested,
                ],
                project: args.project.clone(),
                namespace: args.namespace.clone(),
                workspace: args.workspace.clone(),
                visibility,
                belief_branch: None,
                source_agent: None,
                region: None,
                tags: Vec::new(),
                stages: vec![MemoryStage::Canonical],
                limit: Some(3),
                max_chars_per_item: Some(220),
            })
            .await?;
        let mut items = response.items;
        items.sort_by_key(|item| match item.status {
            MemoryStatus::Stale => 0,
            MemoryStatus::Contested => 1,
            MemoryStatus::Active => 2,
            MemoryStatus::Superseded => 3,
            MemoryStatus::Expired => 4,
        });
        candidate_hits.push(summarize_supersede_candidate_hit(&candidate_query, &items));
        if !items.is_empty() {
            return Ok(SupersedeSearchResult {
                items,
                tried_queries,
                candidate_hits,
            });
        }
    }
    let recent_fallback =
        search_recent_supersede_candidates(&client, args, visibility, &kinds, query).await?;
    candidate_hits.push(summarize_supersede_candidate_hit(
        "recent-scan",
        &recent_fallback,
    ));
    Ok(SupersedeSearchResult {
        items: recent_fallback,
        tried_queries,
        candidate_hits,
    })
}

pub(crate) async fn search_recent_supersede_candidates(
    client: &MemdClient,
    args: &HookCaptureArgs,
    visibility: Option<memd_schema::MemoryVisibility>,
    kinds: &[MemoryKind],
    query: &str,
) -> anyhow::Result<Vec<memd_schema::MemoryItem>> {
    let response = client
        .search(&SearchMemoryRequest {
            query: None,
            route: Some(RetrievalRoute::ProjectFirst),
            intent: Some(RetrievalIntent::General),
            scopes: vec![MemoryScope::Project, MemoryScope::Synced],
            kinds: kinds.to_vec(),
            statuses: vec![
                MemoryStatus::Active,
                MemoryStatus::Stale,
                MemoryStatus::Contested,
            ],
            project: args.project.clone(),
            namespace: args.namespace.clone(),
            workspace: args.workspace.clone(),
            visibility,
            belief_branch: None,
            source_agent: None,
            region: None,
            tags: Vec::new(),
            stages: vec![MemoryStage::Canonical],
            limit: Some(24),
            max_chars_per_item: Some(220),
        })
        .await?;
    let ranked = rank_recent_supersede_candidates(query, response.items);
    if !ranked.is_empty() || kinds.len() != 1 {
        return Ok(ranked);
    }

    let broad_response = client
        .search(&SearchMemoryRequest {
            query: None,
            route: Some(RetrievalRoute::ProjectFirst),
            intent: Some(RetrievalIntent::General),
            scopes: vec![MemoryScope::Project, MemoryScope::Synced],
            kinds: default_supersede_search_kinds(),
            statuses: vec![
                MemoryStatus::Active,
                MemoryStatus::Stale,
                MemoryStatus::Contested,
            ],
            project: args.project.clone(),
            namespace: args.namespace.clone(),
            workspace: args.workspace.clone(),
            visibility,
            belief_branch: None,
            source_agent: None,
            region: None,
            tags: Vec::new(),
            stages: vec![MemoryStage::Canonical],
            limit: Some(32),
            max_chars_per_item: Some(220),
        })
        .await?;
    Ok(rank_recent_supersede_candidates(
        query,
        broad_response.items,
    ))
}

pub(crate) fn rank_recent_supersede_candidates(
    query: &str,
    items: Vec<memd_schema::MemoryItem>,
) -> Vec<memd_schema::MemoryItem> {
    let query_terms = lexical_terms(query);
    let mut ranked = items
        .into_iter()
        .filter_map(|item| {
            let score = lexical_overlap_score(&query_terms, &item.content);
            if score == 0 {
                None
            } else {
                Some((score, supersede_status_rank(item.status), item))
            }
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    ranked
        .into_iter()
        .map(|(_, _, item)| item)
        .take(3)
        .collect()
}

pub(crate) fn lexical_overlap_score(
    query_terms: &std::collections::HashSet<String>,
    content: &str,
) -> usize {
    let content_terms = lexical_terms(content);
    query_terms.intersection(&content_terms).count()
}

pub(crate) fn lexical_terms(value: &str) -> std::collections::HashSet<String> {
    supersede_query_keywords(value).into_iter().collect()
}

pub(crate) fn supersede_status_rank(status: MemoryStatus) -> usize {
    match status {
        MemoryStatus::Stale => 0,
        MemoryStatus::Contested => 1,
        MemoryStatus::Active => 2,
        MemoryStatus::Superseded => 3,
        MemoryStatus::Expired => 4,
    }
}

pub(crate) fn default_supersede_search_kinds() -> Vec<MemoryKind> {
    vec![
        MemoryKind::Fact,
        MemoryKind::Decision,
        MemoryKind::Preference,
        MemoryKind::Constraint,
        MemoryKind::Status,
    ]
}

pub(crate) fn summarize_supersede_candidate_hit(
    query: &str,
    items: &[memd_schema::MemoryItem],
) -> SupersedeCandidateHit {
    SupersedeCandidateHit {
        query: query.to_string(),
        ids: items.iter().map(|item| item.id.to_string()).collect(),
        statuses: items
            .iter()
            .map(|item| format!("{:?}", item.status).to_ascii_lowercase())
            .collect(),
        kinds: items
            .iter()
            .map(|item| format!("{:?}", item.kind).to_ascii_lowercase())
            .collect(),
        previews: items
            .iter()
            .map(|item| summarize_supersede_content_preview(&item.content))
            .collect(),
    }
}

pub(crate) fn summarize_supersede_content_preview(content: &str) -> String {
    const MAX_LEN: usize = 48;
    let compact = content.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() <= MAX_LEN {
        compact
    } else {
        let trimmed = compact.chars().take(MAX_LEN).collect::<String>();
        format!("{trimmed}...")
    }
}

pub(crate) fn format_supersede_candidate_hit(hit: &SupersedeCandidateHit) -> String {
    let entries = hit
        .ids
        .iter()
        .zip(hit.statuses.iter())
        .zip(hit.kinds.iter())
        .zip(hit.previews.iter())
        .map(|(((id, status), kind), preview)| {
            format!("{}:{}:{}:{}", short_uuid_label(id), status, kind, preview)
        })
        .collect::<Vec<_>>();
    if entries.is_empty() {
        format!("{}=>none", hit.query)
    } else {
        format!("{}=>{}", hit.query, entries.join(","))
    }
}

pub(crate) fn summarize_hook_capture_supersede_diagnostics(
    diagnostics: &SupersedeSearchDiagnostics,
) -> (String, String, String) {
    let query = diagnostics
        .inferred_query
        .as_deref()
        .unwrap_or("none")
        .to_string();
    let tried = if diagnostics.tried_queries.is_empty() {
        "none".to_string()
    } else {
        diagnostics.tried_queries.join("|")
    };
    let hits = if diagnostics.candidate_hits.is_empty() {
        "none".to_string()
    } else {
        diagnostics
            .candidate_hits
            .iter()
            .map(format_supersede_candidate_hit)
            .collect::<Vec<_>>()
            .join("|")
    };
    (query, tried, hits)
}

pub(crate) fn short_uuid_label(value: &str) -> String {
    value.chars().take(8).collect()
}

pub(crate) async fn auto_checkpoint_bundle_event(
    output: &Path,
    base_url: &str,
    source_path: &str,
    content: String,
    tags: Vec<String>,
    confidence: f32,
) -> anyhow::Result<()> {
    if read_bundle_runtime_config(output)?.is_none() {
        return Ok(());
    }
    if !bundle_auto_short_term_capture_enabled(output)? {
        return Ok(());
    }
    if content.trim().is_empty() {
        return Ok(());
    }

    checkpoint_with_bundle_defaults(
        &CheckpointArgs {
            output: output.to_path_buf(),
            project: None,
            namespace: None,
            workspace: None,
            visibility: None,
            source_path: Some(source_path.to_string()),
            confidence: Some(confidence),
            ttl_seconds: Some(3_600),
            tag: tags,
            content: Some(content),
            input: None,
            stdin: false,
            auto_commit: false,
            roadmap_set: vec![],
        },
        base_url,
    )
    .await?;

    let snapshot = read_bundle_resume(
        &ResumeArgs {
            output: output.to_path_buf(),
            project: None,
            namespace: None,
            agent: None,
            workspace: None,
            visibility: None,
            route: None,
            intent: Some("current_task".to_string()),
            limit: Some(8),
            rehydration_limit: Some(4),
            semantic: false,
            prompt: false,
            summary: false,
        },
        base_url,
    )
    .await?;
    write_bundle_memory_files(output, &snapshot, None, false).await?;
    Ok(())
}

pub(crate) async fn auto_checkpoint_live_snapshot(
    output: &Path,
    base_url: &str,
    snapshot: &ResumeSnapshot,
    source_path: &str,
) -> anyhow::Result<()> {
    if read_bundle_runtime_config(output)?.is_none() {
        return Ok(());
    }
    if !bundle_auto_short_term_capture_enabled(output)? {
        return Ok(());
    }

    let content = format!(
        "status: {} source_path={source_path}",
        render_bundle_wakeup_summary(snapshot)
    );
    checkpoint_with_bundle_defaults(
        &CheckpointArgs {
            output: output.to_path_buf(),
            project: snapshot.project.clone(),
            namespace: snapshot.namespace.clone(),
            workspace: snapshot.workspace.clone(),
            visibility: snapshot.visibility.clone(),
            source_path: Some(source_path.to_string()),
            confidence: Some(0.72),
            ttl_seconds: Some(3_600),
            tag: vec![
                "auto-short-term".to_string(),
                "bundle-refresh".to_string(),
                source_path.to_string(),
            ],
            content: Some(content),
            input: None,
            stdin: false,
            auto_commit: false,
            roadmap_set: vec![],
        },
        base_url,
    )
    .await?;
    Ok(())
}

pub(crate) async fn auto_checkpoint_compaction_packet(
    packet: &CompactionPacket,
    base_url: &str,
) -> anyhow::Result<()> {
    let Some(output) = resolve_default_bundle_root()? else {
        return Ok(());
    };
    if read_bundle_runtime_config(&output)?.is_none() {
        return Ok(());
    }
    if !bundle_auto_short_term_capture_enabled(&output)? {
        return Ok(());
    }

    let Some(content) = render_compaction_checkpoint_content(packet) else {
        return Ok(());
    };

    let response = checkpoint_with_bundle_defaults(
        &CheckpointArgs {
            output: output.clone(),
            project: packet.session.project.clone(),
            namespace: None,
            workspace: None,
            visibility: None,
            source_path: Some("compaction".to_string()),
            confidence: Some(0.85),
            ttl_seconds: Some(3_600),
            tag: vec!["compaction".to_string(), "auto-checkpoint".to_string()],
            content: Some(content),
            input: None,
            stdin: false,
            auto_commit: false,
            roadmap_set: vec![],
        },
        base_url,
    )
    .await?;

    let snapshot = read_bundle_resume(
        &ResumeArgs {
            output,
            project: packet.session.project.clone(),
            namespace: None,
            agent: packet.session.agent.clone(),
            workspace: None,
            visibility: None,
            route: None,
            intent: Some("current_task".to_string()),
            limit: Some(8),
            rehydration_limit: Some(4),
            semantic: false,
            prompt: false,
            summary: false,
        },
        base_url,
    )
    .await?;
    write_bundle_memory_files(
        &snapshot_bundle_root(&response, &snapshot),
        &snapshot,
        None,
        false,
    )
    .await?;
    Ok(())
}

pub(crate) fn snapshot_bundle_root(
    _response: &memd_schema::StoreMemoryResponse,
    _snapshot: &ResumeSnapshot,
) -> PathBuf {
    resolve_default_bundle_root()
        .ok()
        .flatten()
        .unwrap_or_else(|| PathBuf::from(".memd"))
}

pub(crate) fn render_compaction_checkpoint_content(packet: &CompactionPacket) -> Option<String> {
    let mut lines = Vec::new();

    if !packet.session.task.trim().is_empty() {
        lines.push(format!("task: {}", packet.session.task.trim()));
    }
    if !packet.goal.trim().is_empty() {
        lines.push(format!("goal: {}", packet.goal.trim()));
    }
    if !packet.active_work.is_empty() {
        lines.push(format!(
            "active: {}",
            packet
                .active_work
                .iter()
                .take(3)
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    if !packet.next_actions.is_empty() {
        lines.push(format!(
            "next: {}",
            packet
                .next_actions
                .iter()
                .take(3)
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    if !packet.do_not_drop.is_empty() {
        lines.push(format!(
            "keep: {}",
            packet
                .do_not_drop
                .iter()
                .take(2)
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }

    let content = lines.join("\n");
    if content.trim().is_empty() {
        None
    } else {
        Some(content)
    }
}

pub(crate) fn checkpoint_as_remember_args(args: &CheckpointArgs) -> RememberArgs {
    let mut tags = vec!["checkpoint".to_string(), "current-task".to_string()];
    for tag in &args.tag {
        if !tags.iter().any(|existing| existing == tag) {
            tags.push(tag.clone());
        }
    }

    RememberArgs {
        output: args.output.clone(),
        project: args.project.clone(),
        namespace: args.namespace.clone(),
        workspace: args.workspace.clone(),
        visibility: args.visibility.clone(),
        kind: Some("status".to_string()),
        scope: Some("project".to_string()),
        source_agent: None,
        source_system: Some("memd-short-term".to_string()),
        source_path: args.source_path.clone(),
        source_quality: Some("derived".to_string()),
        confidence: args.confidence.or(Some(0.8)),
        ttl_seconds: args.ttl_seconds.or(Some(86_400)),
        tag: tags,
        supersede: Vec::new(),
        content: args.content.clone(),
        input: args.input.clone(),
        stdin: args.stdin,
    }
}

/// Update key-value pairs inside the `<!-- ROADMAP_STATE ... -->` block in ROADMAP.md.
///
/// If `repo_root` is Some, uses that directory. Otherwise finds the git repo root via
/// `git rev-parse --show-toplevel`. Parses the `<!-- ROADMAP_STATE` block,
/// patches matching keys (or appends new ones), and writes back.
/// Returns `Ok(true)` if changes were written, `Ok(false)` if nothing changed.
pub(crate) fn update_roadmap_state_in(
    updates: &[(String, String)],
    repo_root: Option<&std::path::Path>,
) -> anyhow::Result<bool> {
    if updates.is_empty() {
        return Ok(false);
    }

    let repo_dir_buf;
    let repo_dir: &std::path::Path = if let Some(root) = repo_root {
        root
    } else {
        // Find git root from CWD
        let toplevel = std::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output();
        repo_dir_buf = match toplevel {
            Ok(out) if out.status.success() => {
                std::path::PathBuf::from(String::from_utf8_lossy(&out.stdout).trim().to_string())
            }
            _ => anyhow::bail!("not in a git repository — cannot locate ROADMAP.md"),
        };
        &repo_dir_buf
    };

    let roadmap_path = repo_dir.join("ROADMAP.md");
    if !roadmap_path.exists() {
        anyhow::bail!("ROADMAP.md not found at {}", roadmap_path.display());
    }

    let content = fs::read_to_string(&roadmap_path)
        .with_context(|| format!("read {}", roadmap_path.display()))?;

    let start_marker = "<!-- ROADMAP_STATE";
    let end_marker = "-->";

    let Some(start_pos) = content.find(start_marker) else {
        anyhow::bail!("no <!-- ROADMAP_STATE block found in ROADMAP.md");
    };

    let block_content_start = start_pos + start_marker.len();
    let Some(end_offset) = content[block_content_start..].find(end_marker) else {
        anyhow::bail!("unterminated <!-- ROADMAP_STATE block (missing -->)");
    };
    let block_content_end = block_content_start + end_offset;

    // Parse existing key-value lines
    let block_text = &content[block_content_start..block_content_end];
    let mut lines: Vec<(String, String)> = Vec::new();

    for line in block_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Split on first ": " to preserve colons in values
        if let Some(colon_pos) = trimmed.find(": ") {
            let key = trimmed[..colon_pos].trim().to_string();
            let value = trimmed[colon_pos + 2..].trim().to_string();
            lines.push((key, value));
        }
    }

    // Apply updates: patch existing keys or append new ones
    let mut changed = false;
    for (update_key, update_value) in updates {
        if let Some(existing) = lines.iter_mut().find(|(k, _)| k == update_key) {
            if existing.1 != *update_value {
                existing.1 = update_value.clone();
                changed = true;
            }
        } else {
            lines.push((update_key.clone(), update_value.clone()));
            changed = true;
        }
    }

    if !changed {
        return Ok(false);
    }

    // Rebuild block
    let mut new_block = String::from("\n");
    for (key, value) in &lines {
        new_block.push_str(&format!("{}: {}\n", key, value));
    }

    let mut new_content = String::new();
    new_content.push_str(&content[..start_pos]);
    new_content.push_str(start_marker);
    new_content.push_str(&new_block);
    new_content.push_str(&content[block_content_end..]);

    fs::write(&roadmap_path, &new_content)
        .with_context(|| format!("write {}", roadmap_path.display()))?;

    Ok(true)
}

/// Convenience wrapper — calls `update_roadmap_state_in` with CWD-based git detection.
pub(crate) fn update_roadmap_state(updates: &[(String, String)]) -> anyhow::Result<bool> {
    update_roadmap_state_in(updates, None)
}

/// Auto-commit tracked dirty files before checkpointing.
///
/// If `repo_root` is Some, uses that as the git repo. Otherwise detects via CWD.
/// Returns `Ok(Some(hash))` if a commit was made, `Ok(None)` if the tree was clean.
/// Uses `git add -u` (tracked files only) to avoid staging secrets or binaries.
/// Refuses broad dirty trees by default; override with
/// `MEMD_AUTO_COMMIT_MAX_TRACKED_FILES` when a wider scoped commit is intentional.
pub(crate) fn git_auto_commit_if_dirty_in(
    message: &str,
    repo_root: Option<&std::path::Path>,
) -> anyhow::Result<Option<String>> {
    let repo_dir = if let Some(root) = repo_root {
        root.to_string_lossy().to_string()
    } else {
        // Find the git root from CWD
        let toplevel = std::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output();
        match toplevel {
            Ok(out) if out.status.success() => {
                String::from_utf8_lossy(&out.stdout).trim().to_string()
            }
            _ => return Ok(None), // not a git repo — nothing to commit
        }
    };

    // Check if working tree is dirty
    let status = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&repo_dir)
        .output()?;

    let status_text = String::from_utf8_lossy(&status.stdout);
    if status_text.trim().is_empty() {
        return Ok(None); // clean tree
    }
    let tracked_dirty_count = count_tracked_dirty_status_lines(&status_text);
    if tracked_dirty_count == 0 {
        return Ok(None); // only untracked files exist
    }
    let max_tracked_files = auto_commit_max_tracked_files();
    if tracked_dirty_count > max_tracked_files {
        anyhow::bail!(
            "refusing memd auto-commit: {tracked_dirty_count} tracked file(s) dirty exceeds limit {max_tracked_files}; make an atomic commit manually or raise MEMD_AUTO_COMMIT_MAX_TRACKED_FILES"
        );
    }

    // Stage tracked files only (git add -u avoids untracked/secrets)
    let add = std::process::Command::new("git")
        .args(["add", "-u"])
        .current_dir(&repo_dir)
        .output()?;

    if !add.status.success() {
        anyhow::bail!(
            "git add -u failed: {}",
            String::from_utf8_lossy(&add.stderr)
        );
    }

    // Verify something is actually staged (git add -u might be a no-op
    // if all changes were untracked)
    let diff_staged = std::process::Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .current_dir(&repo_dir)
        .output()?;

    if diff_staged.status.success() {
        // Nothing staged — only untracked files exist
        return Ok(None);
    }

    // Commit
    let commit = std::process::Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(&repo_dir)
        .output()?;

    if !commit.status.success() {
        let stderr = String::from_utf8_lossy(&commit.stderr);
        // "nothing to commit" is not an error for us
        if stderr.contains("nothing to commit") {
            return Ok(None);
        }
        anyhow::bail!("git commit failed: {}", stderr);
    }

    // Extract commit hash
    let rev = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(&repo_dir)
        .output()?;

    let hash = String::from_utf8_lossy(&rev.stdout).trim().to_string();
    Ok(Some(hash))
}

/// Convenience wrapper — calls `git_auto_commit_if_dirty_in` with CWD-based detection.
pub(crate) fn git_auto_commit_if_dirty(message: &str) -> anyhow::Result<Option<String>> {
    git_auto_commit_if_dirty_in(message, None)
}

fn auto_commit_max_tracked_files() -> usize {
    std::env::var("MEMD_AUTO_COMMIT_MAX_TRACKED_FILES")
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(5)
}

fn count_tracked_dirty_status_lines(status_text: &str) -> usize {
    status_text
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            !trimmed.is_empty() && !trimmed.starts_with("??")
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offline_test_request(content: &str) -> memd_schema::StoreMemoryRequest {
        memd_schema::StoreMemoryRequest {
            content: content.to_string(),
            kind: MemoryKind::Fact,
            scope: MemoryScope::Project,
            project: Some("memd-offline-proof".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            visibility: Some(memd_schema::MemoryVisibility::Workspace),
            belief_branch: None,
            source_agent: Some("codex@test".to_string()),
            source_system: Some("offline-test".to_string()),
            source_path: None,
            source_quality: Some(memd_schema::SourceQuality::Canonical),
            confidence: Some(0.9),
            ttl_seconds: None,
            last_verified_at: None,
            supersedes: Vec::new(),
            tags: vec!["offline".to_string()],
            status: Some(MemoryStatus::Active),
            lane: None,
        }
    }

    fn offline_candidate_request(content: &str) -> memd_schema::CandidateMemoryRequest {
        memd_schema::CandidateMemoryRequest {
            content: content.to_string(),
            kind: MemoryKind::Status,
            scope: MemoryScope::Project,
            project: Some("memd-offline-proof".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            visibility: Some(memd_schema::MemoryVisibility::Workspace),
            belief_branch: None,
            source_agent: Some("codex@test".to_string()),
            source_system: Some("hook-spill".to_string()),
            source_path: Some("compaction-packet".to_string()),
            source_quality: Some(memd_schema::SourceQuality::Derived),
            confidence: Some(0.6),
            ttl_seconds: Some(86_400),
            last_verified_at: None,
            supersedes: Vec::new(),
            tags: vec!["hook-spill".to_string(), "offline".to_string()],
            lane: None,
        }
    }

    fn offline_remember_args(output: PathBuf, content: &str) -> RememberArgs {
        RememberArgs {
            output,
            project: Some("memd-offline-proof".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            kind: Some("fact".to_string()),
            scope: Some("project".to_string()),
            source_agent: Some("codex@test".to_string()),
            source_system: Some("offline-test".to_string()),
            source_path: None,
            source_quality: Some("canonical".to_string()),
            confidence: Some(0.9),
            ttl_seconds: None,
            tag: vec!["offline".to_string()],
            supersede: Vec::new(),
            content: Some(content.to_string()),
            input: None,
            stdin: false,
        }
    }

    async fn spawn_offline_store_server() -> String {
        use axum::{Json, Router, routing::post};

        async fn store(
            Json(req): Json<memd_schema::StoreMemoryRequest>,
        ) -> Json<memd_schema::StoreMemoryResponse> {
            let now = chrono::Utc::now();
            Json(memd_schema::StoreMemoryResponse {
                item: memd_schema::MemoryItem {
                    id: Uuid::new_v4(),
                    content: req.content,
                    redundancy_key: None,
                    belief_branch: req.belief_branch,
                    preferred: false,
                    kind: req.kind,
                    scope: req.scope,
                    project: req.project,
                    namespace: req.namespace,
                    workspace: req.workspace,
                    visibility: req.visibility.unwrap_or_default(),
                    source_agent: req.source_agent,
                    source_system: req.source_system,
                    source_path: req.source_path,
                    source_quality: req.source_quality,
                    confidence: req.confidence.unwrap_or(0.8),
                    ttl_seconds: req.ttl_seconds,
                    created_at: now,
                    updated_at: now,
                    last_verified_at: req.last_verified_at,
                    supersedes: req.supersedes,
                    tags: req.tags,
                    status: req.status.unwrap_or(MemoryStatus::Active),
                    stage: memd_schema::MemoryStage::Canonical,
                    lane: req.lane,
                    version: 1,
                    correction_meta: None,
                },
            })
        }

        let app = Router::new().route("/memory/store", post(store));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind offline store server");
        let addr = listener.local_addr().expect("offline store addr");
        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("serve offline store server");
        });
        format!("http://{}", addr)
    }

    async fn spawn_offline_sync_server(
        seen: std::sync::Arc<std::sync::Mutex<Vec<&'static str>>>,
    ) -> String {
        use axum::{Json, Router, extract::State, routing::post};

        #[derive(Clone)]
        struct SyncState {
            seen: std::sync::Arc<std::sync::Mutex<Vec<&'static str>>>,
        }

        async fn capabilities(
            State(state): State<SyncState>,
            Json(req): Json<memd_schema::CapabilitySyncRequest>,
        ) -> Json<memd_schema::CapabilitySyncResponse> {
            state.seen.lock().expect("seen lock").push("capabilities");
            Json(memd_schema::CapabilitySyncResponse {
                upserted: req.records.len(),
                total: req.records.len(),
                records: req.records,
            })
        }

        async fn access_routes(
            State(state): State<SyncState>,
            Json(req): Json<memd_schema::AccessRouteSyncRequest>,
        ) -> Json<memd_schema::AccessRouteSyncResponse> {
            state.seen.lock().expect("seen lock").push("access_routes");
            Json(memd_schema::AccessRouteSyncResponse {
                upserted: req.routes.len(),
                total: req.routes.len(),
                routes: req.routes,
            })
        }

        async fn token_savings(
            State(state): State<SyncState>,
            Json(req): Json<memd_schema::TokenSavingsSyncRequest>,
        ) -> Json<memd_schema::TokenSavingsSyncResponse> {
            state.seen.lock().expect("seen lock").push("token_savings");
            Json(memd_schema::TokenSavingsSyncResponse {
                upserted: req.records.len(),
                total: req.records.len(),
                records: req.records,
            })
        }

        async fn candidate(
            State(state): State<SyncState>,
            Json(req): Json<memd_schema::CandidateMemoryRequest>,
        ) -> Json<memd_schema::CandidateMemoryResponse> {
            let now = chrono::Utc::now();
            state.seen.lock().expect("seen lock").push("candidates");
            Json(memd_schema::CandidateMemoryResponse {
                item: memd_schema::MemoryItem {
                    id: Uuid::new_v4(),
                    content: req.content,
                    redundancy_key: Some("candidate".to_string()),
                    belief_branch: req.belief_branch,
                    preferred: false,
                    kind: req.kind,
                    scope: req.scope,
                    project: req.project,
                    namespace: req.namespace,
                    workspace: req.workspace,
                    visibility: req.visibility.unwrap_or_default(),
                    source_agent: req.source_agent,
                    source_system: req.source_system,
                    source_path: req.source_path,
                    source_quality: req.source_quality,
                    confidence: req.confidence.unwrap_or(0.6),
                    ttl_seconds: req.ttl_seconds,
                    created_at: now,
                    updated_at: now,
                    last_verified_at: req.last_verified_at,
                    supersedes: req.supersedes,
                    tags: req.tags,
                    status: MemoryStatus::Active,
                    stage: memd_schema::MemoryStage::Candidate,
                    lane: req.lane,
                    version: 1,
                    correction_meta: None,
                },
                duplicate_of: None,
            })
        }

        let app = Router::new()
            .route("/capabilities/sync", post(capabilities))
            .route("/access/routes/sync", post(access_routes))
            .route("/tokens/savings/sync", post(token_savings))
            .route("/memory/candidates", post(candidate))
            .with_state(SyncState { seen });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind offline sync server");
        let addr = listener.local_addr().expect("offline sync addr");
        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("serve offline sync server");
        });
        format!("http://{}", addr)
    }

    #[tokio::test]
    async fn remember_queues_offline_store_when_backend_down_and_dedupes() {
        let temp = tempfile::tempdir().expect("tempdir");
        let bundle = temp.path().join(".memd");
        fs::create_dir_all(&bundle).expect("create bundle");
        let args = offline_remember_args(
            bundle.clone(),
            "Offline queued memory survives backend outage.",
        );

        let first = remember_with_bundle_defaults(&args, "http://127.0.0.1:1")
            .await
            .expect("queue offline remember");
        let second = remember_with_bundle_defaults(&args, "http://127.0.0.1:1")
            .await
            .expect("dedupe offline remember");
        let entries = read_offline_store_queue(&bundle).expect("read offline queue");

        assert!(is_offline_queued_response(&first));
        assert!(is_offline_queued_response(&second));
        assert_eq!(first.item.id, second.item.id);
        assert_eq!(entries.len(), 1, "same offline write should dedupe");
        assert_eq!(entries[0].status, "pending");
    }

    #[tokio::test]
    async fn replay_offline_store_queue_syncs_pending_and_skips_synced() {
        let temp = tempfile::tempdir().expect("tempdir");
        let bundle = temp.path().join(".memd");
        fs::create_dir_all(&bundle).expect("create bundle");
        let req = offline_test_request("Replay this offline memory once.");
        let queued =
            queue_offline_store_request(&bundle, &req, "backend unavailable").expect("queue");
        let base_url = spawn_offline_store_server().await;
        let client = MemdClient::new(&base_url).expect("client");

        let report = replay_offline_store_queue(&bundle, &client)
            .await
            .expect("replay queue");
        let entries = read_offline_store_queue(&bundle).expect("read replayed queue");
        let second = replay_offline_store_queue(&bundle, &client)
            .await
            .expect("second replay");

        assert_eq!(report.attempted, 1);
        assert_eq!(report.synced, 1);
        assert_eq!(report.failed, 0);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, queued.id);
        assert_eq!(entries[0].status, "synced");
        assert!(entries[0].synced_item_id.is_some());
        assert_eq!(
            second.attempted, 0,
            "synced entries should not replay twice"
        );
    }

    #[tokio::test]
    async fn replay_offline_sync_queue_replays_candidate_spill_payloads() {
        let temp = tempfile::tempdir().expect("tempdir");
        let bundle = temp.path().join(".memd");
        fs::create_dir_all(&bundle).expect("create bundle");
        let req = offline_candidate_request("Replay this offline hook spill candidate once.");
        queue_offline_sync_payload(
            &bundle,
            OfflineSyncPayload::Candidates(vec![req.clone()]),
            "backend unavailable",
        )
        .expect("queue candidate spill");
        let status = offline_queue_status(&bundle).expect("offline status");
        assert_eq!(status.sync.total, 1);
        assert_eq!(status.sync.by_kind["candidates"].pending, 1);

        let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let base_url = spawn_offline_sync_server(seen.clone()).await;
        let client = MemdClient::new(&base_url).expect("client");
        let report = replay_offline_sync_queue(&bundle, &client)
            .await
            .expect("replay candidate queue");
        let replayed = read_offline_sync_queue(&bundle).expect("read replayed candidate queue");
        let second = replay_offline_sync_queue(&bundle, &client)
            .await
            .expect("second candidate replay");

        assert_eq!(report.attempted, 1);
        assert_eq!(report.synced, 1);
        assert_eq!(report.failed, 0);
        assert_eq!(replayed.len(), 1);
        assert_eq!(replayed[0].status, "synced");
        assert_eq!(second.attempted, 0);
        assert_eq!(seen.lock().expect("seen lock").as_slice(), &["candidates"]);
    }

    #[test]
    fn offline_sync_queue_dedupes_and_reports_status() {
        let temp = tempfile::tempdir().expect("tempdir");
        let bundle = temp.path().join(".memd");
        fs::create_dir_all(&bundle).expect("create bundle");
        let payload = OfflineSyncPayload::Capabilities(memd_schema::CapabilitySyncRequest {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            user_id: None,
            agent: Some("codex".to_string()),
            records: vec![memd_schema::CapabilityRecord {
                harness: "codex".to_string(),
                kind: "skill".to_string(),
                name: "browser".to_string(),
                status: "installed".to_string(),
                portability_class: "harness-native".to_string(),
                source_path: "/tmp/SKILL.md".to_string(),
                bridge_hint: None,
                hash: None,
                notes: Vec::new(),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                user_id: None,
                agent: Some("codex".to_string()),
                updated_at: None,
            }],
        });

        let first = queue_offline_sync_payload(&bundle, payload.clone(), "server down")
            .expect("queue sync payload");
        let second = queue_offline_sync_payload(&bundle, payload, "still down")
            .expect("dedupe sync payload");
        let status = offline_queue_status(&bundle).expect("offline status");

        assert_eq!(first.id, second.id);
        assert_eq!(status.store.total, 0);
        assert_eq!(status.sync.total, 1);
        assert_eq!(status.sync.pending, 1);
    }

    #[tokio::test]
    async fn replay_offline_sync_queue_reconciles_payloads_with_server_authority() {
        let temp = tempfile::tempdir().expect("tempdir");
        let bundle = temp.path().join(".memd");
        fs::create_dir_all(&bundle).expect("create bundle");
        let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let base_url = spawn_offline_sync_server(seen.clone()).await;
        let client = MemdClient::new(&base_url).expect("client");

        queue_offline_sync_payload(
            &bundle,
            OfflineSyncPayload::Capabilities(memd_schema::CapabilitySyncRequest {
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                user_id: None,
                agent: Some("codex@pc-a".to_string()),
                records: vec![memd_schema::CapabilityRecord {
                    harness: "codex".to_string(),
                    kind: "skill".to_string(),
                    name: "browser-use:browser".to_string(),
                    status: "installed".to_string(),
                    portability_class: "harness-native".to_string(),
                    source_path: "/Users/test/.codex/skills/browser/SKILL.md".to_string(),
                    bridge_hint: Some("PC-B can use this through target equivalent".to_string()),
                    hash: Some("sha256:cap".to_string()),
                    notes: vec!["queued while backend down".to_string()],
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("shared".to_string()),
                    user_id: None,
                    agent: Some("codex@pc-a".to_string()),
                    updated_at: None,
                }],
            }),
            "backend down",
        )
        .expect("queue capability sync");
        queue_offline_sync_payload(
            &bundle,
            OfflineSyncPayload::AccessRoutes(memd_schema::AccessRouteSyncRequest {
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                user_id: None,
                agent: Some("codex@pc-a".to_string()),
                routes: vec![memd_schema::AccessRouteRecord {
                    id: "bitwarden-login".to_string(),
                    provider: "bitwarden".to_string(),
                    status: "locked".to_string(),
                    scope: "user/project".to_string(),
                    secret_values_stored: false,
                    guidance: "Ask user to unlock Bitwarden; store refs only.".to_string(),
                    source: "bw status".to_string(),
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("shared".to_string()),
                    user_id: None,
                    agent: Some("codex@pc-a".to_string()),
                    updated_at: None,
                }],
            }),
            "backend down",
        )
        .expect("queue access route sync");
        queue_offline_sync_payload(
            &bundle,
            OfflineSyncPayload::TokenSavings(memd_schema::TokenSavingsSyncRequest {
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                user_id: None,
                agent: Some("codex@pc-a".to_string()),
                records: vec![memd_schema::TokenSavingsRecord {
                    id: uuid::Uuid::new_v4(),
                    operation: "context_packet".to_string(),
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("shared".to_string()),
                    user_id: None,
                    agent: Some("codex@pc-a".to_string()),
                    model_tier: Some("tiny".to_string()),
                    intent: Some("CurrentTask".to_string()),
                    source_records: 3,
                    baseline_input_tokens: 1200,
                    output_tokens: 280,
                    tokens_saved: 920,
                    reason: "offline packet compile avoided reread".to_string(),
                    ts: chrono::Utc::now(),
                    updated_at: None,
                }],
            }),
            "backend down",
        )
        .expect("queue token savings sync");

        let report = replay_offline_sync_queue(&bundle, &client)
            .await
            .expect("replay sync queue");
        let second = replay_offline_sync_queue(&bundle, &client)
            .await
            .expect("second replay skips synced");
        let status = offline_queue_status(&bundle).expect("offline status after sync");
        let seen = seen.lock().expect("seen lock").clone();

        assert_eq!(report.attempted, 3);
        assert_eq!(report.synced, 3);
        assert_eq!(report.failed, 0);
        assert_eq!(report.pending, 0);
        assert_eq!(second.attempted, 0);
        assert_eq!(status.sync.pending, 0);
        assert_eq!(status.sync.by_kind["capabilities"].synced, 1);
        assert_eq!(status.sync.by_kind["access_routes"].synced, 1);
        assert_eq!(status.sync.by_kind["token_savings"].synced, 1);
        assert!(seen.contains(&"capabilities"));
        assert!(seen.contains(&"access_routes"));
        assert!(seen.contains(&"token_savings"));
    }

    #[test]
    fn infer_bundle_identity_defaults_bind_repo_without_runtime_config() {
        let temp_root =
            std::env::temp_dir().join(format!("memd-checkpoint-defaults-{}", uuid::Uuid::new_v4()));
        let repo_root = temp_root.join("repo-b");
        let bundle_root = repo_root.join(".memd");

        fs::create_dir_all(repo_root.join(".git")).expect("create repo git dir");

        let (project, namespace) = infer_bundle_identity_defaults(&bundle_root);
        assert_eq!(project.as_deref(), Some("repo-b"));
        assert_eq!(namespace.as_deref(), Some("main"));

        fs::remove_dir_all(temp_root).expect("cleanup temp root");
    }

    #[test]
    fn git_auto_commit_clean_tree_returns_none() {
        let temp_root =
            std::env::temp_dir().join(format!("memd-auto-commit-clean-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_root).expect("create temp dir");

        // Init a git repo with one commit
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&temp_root)
            .output()
            .expect("git init");
        std::process::Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(&temp_root)
            .output()
            .expect("git config email");
        std::process::Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(&temp_root)
            .output()
            .expect("git config name");
        fs::write(temp_root.join("file.txt"), "content").expect("write file");
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&temp_root)
            .output()
            .expect("git add");
        std::process::Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(&temp_root)
            .output()
            .expect("git commit");

        // Pass explicit repo root — no set_current_dir needed
        let result = git_auto_commit_if_dirty_in("test: should not commit", Some(&temp_root));

        assert!(result.is_ok());
        assert!(result.unwrap().is_none(), "clean tree should return None");

        fs::remove_dir_all(temp_root).expect("cleanup temp root");
    }

    #[test]
    fn git_auto_commit_dirty_tree_commits_and_returns_hash() {
        let temp_root =
            std::env::temp_dir().join(format!("memd-auto-commit-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_root).expect("create temp dir");

        // Init a git repo
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&temp_root)
            .output()
            .expect("git init");

        // Configure git user for the test repo
        std::process::Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(&temp_root)
            .output()
            .expect("git config email");
        std::process::Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(&temp_root)
            .output()
            .expect("git config name");

        // Create and commit an initial file (need at least one commit)
        let file_path = temp_root.join("tracked.txt");
        fs::write(&file_path, "initial").expect("write file");
        std::process::Command::new("git")
            .args(["add", "tracked.txt"])
            .current_dir(&temp_root)
            .output()
            .expect("git add");
        std::process::Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(&temp_root)
            .output()
            .expect("git commit initial");

        // Modify the tracked file (makes tree dirty)
        fs::write(&file_path, "modified").expect("modify file");

        // Pass explicit repo root — no set_current_dir needed
        let result = git_auto_commit_if_dirty_in("test: auto-commit dirty tree", Some(&temp_root));

        assert!(result.is_ok());
        let hash = result.unwrap();
        assert!(hash.is_some(), "should have committed and returned a hash");
        assert!(!hash.unwrap().is_empty(), "hash should not be empty");

        fs::remove_dir_all(temp_root).expect("cleanup temp root");
    }

    #[test]
    fn git_auto_commit_refuses_broad_dirty_tree() {
        let temp_root =
            std::env::temp_dir().join(format!("memd-auto-commit-broad-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_root).expect("create temp dir");

        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&temp_root)
            .output()
            .expect("git init");
        std::process::Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(&temp_root)
            .output()
            .expect("git config email");
        std::process::Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(&temp_root)
            .output()
            .expect("git config name");

        for index in 0..6 {
            fs::write(temp_root.join(format!("tracked-{index}.txt")), "initial")
                .expect("write initial file");
        }
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&temp_root)
            .output()
            .expect("git add");
        std::process::Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(&temp_root)
            .output()
            .expect("git commit initial");

        for index in 0..6 {
            fs::write(temp_root.join(format!("tracked-{index}.txt")), "modified")
                .expect("modify tracked file");
        }

        let err = git_auto_commit_if_dirty_in("test: should refuse", Some(&temp_root))
            .expect_err("broad dirty tree should be rejected");
        assert!(
            err.to_string().contains("refusing memd auto-commit"),
            "unexpected error: {err}"
        );

        let staged = std::process::Command::new("git")
            .args(["diff", "--cached", "--quiet"])
            .current_dir(&temp_root)
            .status()
            .expect("git diff cached");
        assert!(
            staged.success(),
            "broad auto-commit guard should not stage files"
        );

        fs::remove_dir_all(temp_root).expect("cleanup temp root");
    }

    #[test]
    fn update_roadmap_state_patches_existing_keys() {
        let temp_root =
            std::env::temp_dir().join(format!("memd-roadmap-state-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_root).expect("create temp dir");

        // Write a ROADMAP.md with a state block (no git needed — pass explicit path)
        let roadmap = r#"# Roadmap

<!-- ROADMAP_STATE
current_phase: O2
phase_status: verified
next_step: P2 — do the thing: with colons
note: O2 done
-->

## Content
"#;
        fs::write(temp_root.join("ROADMAP.md"), roadmap).expect("write roadmap");

        let updates = vec![
            ("current_phase".to_string(), "P2".to_string()),
            ("phase_status".to_string(), "in_progress".to_string()),
        ];

        let result = update_roadmap_state_in(&updates, Some(&temp_root));

        assert!(result.is_ok());
        assert!(result.unwrap(), "should report changes made");

        let updated = fs::read_to_string(temp_root.join("ROADMAP.md")).expect("read updated");
        assert!(
            updated.contains("current_phase: P2"),
            "phase should be updated"
        );
        assert!(
            updated.contains("phase_status: in_progress"),
            "status should be updated"
        );
        // Colons in values must survive
        assert!(
            updated.contains("next_step: P2 — do the thing: with colons"),
            "colon-bearing values must be preserved"
        );
        assert!(updated.contains("## Content"), "rest of file preserved");

        fs::remove_dir_all(temp_root).expect("cleanup");
    }

    #[test]
    fn update_roadmap_state_appends_new_keys() {
        let temp_root =
            std::env::temp_dir().join(format!("memd-roadmap-append-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_root).expect("create temp dir");

        let roadmap = "<!-- ROADMAP_STATE\ncurrent_phase: O2\n-->\n";
        fs::write(temp_root.join("ROADMAP.md"), roadmap).expect("write roadmap");

        let updates = vec![("new_key".to_string(), "new_value".to_string())];
        let result = update_roadmap_state_in(&updates, Some(&temp_root));

        assert!(result.is_ok());
        assert!(result.unwrap(), "should report changes");

        let updated = fs::read_to_string(temp_root.join("ROADMAP.md")).expect("read");
        assert!(updated.contains("new_key: new_value"), "new key appended");
        assert!(
            updated.contains("current_phase: O2"),
            "existing key preserved"
        );

        fs::remove_dir_all(temp_root).expect("cleanup");
    }

    #[test]
    fn update_roadmap_state_no_changes_returns_false() {
        let temp_root =
            std::env::temp_dir().join(format!("memd-roadmap-noop-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_root).expect("create temp dir");

        let roadmap = "<!-- ROADMAP_STATE\ncurrent_phase: O2\n-->\n";
        fs::write(temp_root.join("ROADMAP.md"), roadmap).expect("write roadmap");

        // Same value — no change
        let updates = vec![("current_phase".to_string(), "O2".to_string())];
        let result = update_roadmap_state_in(&updates, Some(&temp_root));

        assert!(result.is_ok());
        assert!(!result.unwrap(), "no changes should return false");

        fs::remove_dir_all(temp_root).expect("cleanup");
    }
}

use std::{
    collections::HashSet,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use memd_rag::{
    RagBackendHealthResponse, RagClient, RagIngestRequest, RagIngestSource, RagRerankCandidate,
    RagRerankRequest, RagRetrieveMode, RagRetrieveRequest,
};
use memd_schema::{MemoryItem, RagHealthStatus, SearchMemoryRequest};
use tracing::warn;
use uuid::Uuid;

static RAG_FAILURES: AtomicU64 = AtomicU64::new(0);

fn record_failure() {
    RAG_FAILURES.fetch_add(1, Ordering::Relaxed);
}

pub(crate) fn rag_failure_count() -> u64 {
    RAG_FAILURES.load(Ordering::Relaxed)
}

const RAG_HEALTH_TIMEOUT: Duration = Duration::from_millis(500);
const RAG_DEFAULT_TIMEOUT_MS: u64 = 300;

fn rag_timeout() -> Duration {
    let millis = std::env::var("MEMD_RAG_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(RAG_DEFAULT_TIMEOUT_MS);
    Duration::from_millis(millis)
}

pub(crate) fn rag_timeout_ms() -> u64 {
    rag_timeout().as_millis() as u64
}

pub(crate) fn build_rag_client() -> Option<Arc<RagClient>> {
    let url = std::env::var("MEMD_RAG_URL").ok()?;
    let url = url.trim();
    if url.is_empty() {
        return None;
    }
    RagClient::new(url).ok().map(Arc::new)
}

pub(crate) fn rag_dense_enabled() -> bool {
    match std::env::var("MEMD_RETRIEVAL_RAG_DENSE") {
        Ok(value) => {
            let value = value.trim().to_ascii_lowercase();
            matches!(value.as_str(), "1" | "true" | "on" | "yes")
        }
        Err(_) => true,
    }
}

pub(crate) fn spawn_ingest(client: Arc<RagClient>, item: MemoryItem) {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.spawn(async move {
            if let Err(error) = ingest_item(&client, &item).await {
                warn!(
                    target: "memd::rag_bridge",
                    error = %format_args!("{error:#}"),
                    "sidecar ingest failed (non-fatal)"
                );
            }
        });
        return;
    }

    let _ = std::thread::Builder::new()
        .name(format!("memd-rag-ingest-{}", item.id))
        .spawn(move || {
            match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => {
                    rt.block_on(async move {
                        if let Err(error) = ingest_item(&client, &item).await {
                            warn!(
                                target: "memd::rag_bridge",
                                error = %format_args!("{error:#}"),
                                "sidecar ingest failed (non-fatal)"
                            );
                        }
                    });
                }
                Err(error) => warn!(
                    target: "memd::rag_bridge",
                    error = %format_args!("{error:#}"),
                    "failed to build runtime for detached sidecar ingest"
                ),
            }
        });
}

pub(crate) async fn ingest_item(client: &RagClient, item: &MemoryItem) -> anyhow::Result<()> {
    let request = RagIngestRequest {
        project: item.project.clone(),
        namespace: item.namespace.clone(),
        source: RagIngestSource {
            id: item.id,
            kind: format!("{:?}", item.kind).to_lowercase(),
            content: rag_enriched_content(item),
            mime: None,
            bytes: Some(item.content.len() as u64),
            source_quality: item.source_quality,
            source_agent: item.source_agent.clone(),
            source_path: Some(item.id.to_string()),
            tags: item.tags.clone(),
        },
    };
    let per_attempt = rag_timeout().saturating_mul(2);
    let backoffs = [Duration::from_millis(100), Duration::from_millis(500)];
    let mut last_error: Option<anyhow::Error> = None;
    for attempt in 0..=backoffs.len() {
        match tokio::time::timeout(per_attempt, client.ingest(&request)).await {
            Ok(Ok(_)) => return Ok(()),
            Ok(Err(error)) => last_error = Some(error),
            Err(_) => {
                last_error = Some(anyhow::anyhow!(
                    "rag ingest timed out after {}ms",
                    per_attempt.as_millis()
                ))
            }
        }
        if let Some(delay) = backoffs.get(attempt) {
            tokio::time::sleep(*delay).await;
        }
    }
    record_failure();
    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("rag ingest failed with no error recorded")))
}

fn rag_enriched_content(item: &MemoryItem) -> String {
    let facets = rag_semantic_facets(item);
    if facets.is_empty() {
        return item.content.clone();
    }
    format!(
        "{}\n\nmemd_semantic_facets: {}",
        item.content,
        facets.join("; ")
    )
}

fn rag_semantic_facets(item: &MemoryItem) -> Vec<&'static str> {
    let mut haystack = item.content.to_ascii_lowercase();
    haystack.push(' ');
    haystack.push_str(&item.tags.join(" ").to_ascii_lowercase());
    if let Some(path) = item.source_path.as_deref() {
        haystack.push(' ');
        haystack.push_str(&path.to_ascii_lowercase());
    }
    let mut facets = Vec::new();
    let mut add = |needle: &[&str], facet: &'static str| {
        if needle.iter().any(|term| haystack.contains(term)) && !facets.contains(&facet) {
            facets.push(facet);
        }
    };
    add(
        &["restart", "breadcrumb", "interrupted", "crash", "resume"],
        "resume crashed assistant session with prior context",
    );
    add(
        &[
            "claude", "codex", "opencode", "openclaw", "hermes", "ollama",
        ],
        "switch assistants while retaining shared memory across harnesses",
    );
    add(
        &["packet", "gateway", "guard", "local model", "ollama"],
        "safe compact context for local language models",
    );
    add(&["ollama"], "feed local models trusted memories safely");
    add(
        &["queue", "offline", "backend returns", "server is down"],
        "remember facts when the server is unavailable and replay later",
    );
    add(
        &["alias", "aliases", "file paths", "commands", "identifiers"],
        "find misspelled files commands names and ids",
    );
    add(
        &["trace", "lexical", "fuzzy", "dense", "recency", "selected"],
        "explain why a retrieval result was selected",
    );
    add(
        &["correction", "supersedes", "stale", "outdated"],
        "latest correction beats old mistaken fact and tracks replacement",
    );
    add(
        &["visibility", "private", "workspace", "leak"],
        "stop another assistant from seeing private notes",
    );
    add(
        &["sync", "devices", "sessions", "self hosted", "backend"],
        "one self hosted backend keeps agents synchronized",
    );
    add(
        &["procedure", "procedures", "runbook", "workflow"],
        "reuse a repeated operational workflow",
    );
    add(
        &["atlas", "entities", "sessions", "decisions", "aliases"],
        "connect related people projects sessions and decisions",
    );
    add(
        &[
            "firewall",
            "quarantine",
            "instruction attacks",
            "policy",
            "tools",
        ],
        "prevent memory from changing system policy or tools",
    );
    add(
        &["bench", "embedding", "qrels", "recall", "mrr"],
        "choose the best vector model from measured qrels",
    );
    add(
        &["sidecar", "dense", "vector", "optional"],
        "vector service boosts recall without becoming truth",
    );
    add(
        &["rerank", "reorders", "relevance"],
        "sort retrieved memories by strongest relevance",
    );
    add(
        &["bundle", "wake", "events", "config"],
        "local files boot memory without network",
    );
    add(
        &["duplicate", "dedupe", "noisy"],
        "avoid storing the same fact repeatedly",
    );
    add(
        &["event log", "capture", "promote", "retrieve"],
        "audit how memory changed over time",
    );
    add(
        &["scope", "global", "project", "workspace"],
        "keep global and project memories from leaking",
    );
    add(
        &["fusion", "fts", "lanes", "signals"],
        "combine many retrieval signals into one ranking",
    );
    facets
}

pub(crate) async fn fetch_dense_candidates(
    client: &RagClient,
    req: &SearchMemoryRequest,
) -> anyhow::Result<Vec<(Uuid, f64)>> {
    if !rag_dense_enabled() {
        return Ok(Vec::new());
    }

    let Some(query) = req
        .query
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(Vec::new());
    };

    let timeout = rag_timeout();
    let response = match tokio::time::timeout(
        timeout,
        client.retrieve(&RagRetrieveRequest {
            query: query.to_string(),
            project: req.project.clone(),
            namespace: req.namespace.clone(),
            mode: RagRetrieveMode::Auto,
            limit: Some(req.limit.unwrap_or(10).max(1)),
            include_cross_modal: false,
        }),
    )
    .await
    {
        Ok(Ok(response)) => response,
        Ok(Err(error)) => {
            record_failure();
            warn!(
                target: "memd::rag_bridge",
                error = %format_args!("{error:#}"),
                "rag retrieve failed; degrading to lexical"
            );
            return Ok(Vec::new());
        }
        Err(_) => {
            record_failure();
            warn!(
                target: "memd::rag_bridge",
                timeout_ms = timeout.as_millis() as u64,
                "rag retrieve timed out; degrading to lexical"
            );
            return Ok(Vec::new());
        }
    };

    let mut seen = HashSet::new();
    let mut candidates = Vec::new();
    for item in response.items {
        let Some(source) = item.source else {
            continue;
        };
        let Ok(id) = Uuid::parse_str(source.trim()) else {
            continue;
        };
        if seen.insert(id) {
            candidates.push((id, item.score as f64));
        }
    }
    Ok(candidates)
}

pub(crate) async fn rerank_candidates(
    client: &RagClient,
    query: &str,
    candidates: &[(Uuid, String)],
    top_k: usize,
) -> anyhow::Result<Vec<(Uuid, f64)>> {
    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    let timeout = rag_timeout();
    let response = match tokio::time::timeout(
        timeout,
        client.rerank(&RagRerankRequest {
            query: query.to_string(),
            candidates: candidates
                .iter()
                .map(|(id, text)| RagRerankCandidate {
                    id: id.to_string(),
                    text: text.clone(),
                })
                .collect(),
            top_k: Some(top_k.max(1).min(candidates.len())),
        }),
    )
    .await
    {
        Ok(Ok(response)) => response,
        Ok(Err(error)) => {
            record_failure();
            warn!(
                target: "memd::rag_bridge",
                error = %format_args!("{error:#}"),
                "rag rerank failed; skipping rerank"
            );
            return Ok(Vec::new());
        }
        Err(_) => {
            record_failure();
            warn!(
                target: "memd::rag_bridge",
                timeout_ms = timeout.as_millis() as u64,
                "rag rerank timed out; skipping rerank"
            );
            return Ok(Vec::new());
        }
    };

    let mut ranked = Vec::new();
    for item in response.items {
        let Ok(id) = Uuid::parse_str(item.id.trim()) else {
            continue;
        };
        ranked.push((id, item.score as f64));
    }
    Ok(ranked)
}

pub(crate) async fn health_surface(client: Option<&RagClient>) -> RagHealthStatus {
    let recent_failures = rag_failure_count();
    let Some(client) = client else {
        return RagHealthStatus {
            enabled: false,
            reachable: false,
            name: None,
            profile: None,
            indexed_count: None,
            timeout_ms: Some(rag_timeout_ms()),
            last_sync_status: Some("disabled".to_string()),
            recent_failures,
        };
    };

    match tokio::time::timeout(RAG_HEALTH_TIMEOUT, client.healthz()).await {
        Ok(Ok(response)) => health_from_response(response, recent_failures),
        Ok(Err(error)) => {
            warn!(
                target: "memd::rag_bridge",
                error = %format_args!("{error:#}"),
                "rag health check failed"
            );
            RagHealthStatus {
                enabled: true,
                reachable: false,
                name: None,
                profile: None,
                indexed_count: None,
                timeout_ms: Some(rag_timeout_ms()),
                last_sync_status: Some("health_error".to_string()),
                recent_failures,
            }
        }
        Err(_) => {
            warn!(target: "memd::rag_bridge", "rag health check timed out");
            RagHealthStatus {
                enabled: true,
                reachable: false,
                name: None,
                profile: None,
                indexed_count: None,
                timeout_ms: Some(rag_timeout_ms()),
                last_sync_status: Some("health_timeout".to_string()),
                recent_failures,
            }
        }
    }
}

fn health_from_response(
    response: RagBackendHealthResponse,
    recent_failures: u64,
) -> RagHealthStatus {
    RagHealthStatus {
        enabled: true,
        reachable: response.backend.connected,
        name: response.backend.name,
        profile: response.backend.profile,
        indexed_count: response.backend.indexed_count,
        timeout_ms: Some(rag_timeout_ms()),
        last_sync_status: Some("health_ok".to_string()),
        recent_failures,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use memd_schema::{MemoryKind, MemoryScope, MemoryStage, MemoryStatus, MemoryVisibility};
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn dummy_item() -> MemoryItem {
        MemoryItem {
            id: Uuid::new_v4(),
            content: "test".to_string(),
            redundancy_key: None,
            belief_branch: None,
            preferred: false,
            kind: MemoryKind::Fact,
            scope: MemoryScope::Project,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            visibility: MemoryVisibility::default(),
            source_agent: None,
            source_system: None,
            source_path: None,
            source_quality: None,
            confidence: 0.8,
            ttl_seconds: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_verified_at: None,
            supersedes: Vec::new(),
            tags: Vec::new(),
            status: MemoryStatus::Active,
            stage: MemoryStage::Canonical,
            lane: None,
            version: 1,
            correction_meta: None,
        }
    }

    #[test]
    fn rag_timeout_reads_env_with_floor_and_default() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe { std::env::remove_var("MEMD_RAG_TIMEOUT_MS") };
        assert_eq!(rag_timeout(), Duration::from_millis(RAG_DEFAULT_TIMEOUT_MS));

        unsafe { std::env::set_var("MEMD_RAG_TIMEOUT_MS", "42") };
        assert_eq!(rag_timeout(), Duration::from_millis(42));

        unsafe { std::env::set_var("MEMD_RAG_TIMEOUT_MS", "0") };
        assert_eq!(rag_timeout(), Duration::from_millis(RAG_DEFAULT_TIMEOUT_MS));

        unsafe { std::env::set_var("MEMD_RAG_TIMEOUT_MS", "not-a-number") };
        assert_eq!(rag_timeout(), Duration::from_millis(RAG_DEFAULT_TIMEOUT_MS));

        unsafe { std::env::remove_var("MEMD_RAG_TIMEOUT_MS") };
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn fetch_dense_degrades_to_empty_on_unreachable_sidecar() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe { std::env::set_var("MEMD_RAG_TIMEOUT_MS", "1") };
        let before = rag_failure_count();
        let client = RagClient::new("http://127.0.0.1:1").expect("client");
        let req = SearchMemoryRequest {
            query: Some("anything".to_string()),
            limit: Some(5),
            ..Default::default()
        };
        let result = fetch_dense_candidates(&client, &req).await;
        assert!(matches!(result, Ok(ref v) if v.is_empty()));
        assert!(rag_failure_count() > before);
        unsafe { std::env::remove_var("MEMD_RAG_TIMEOUT_MS") };
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn rerank_degrades_to_empty_on_unreachable_sidecar() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe { std::env::set_var("MEMD_RAG_TIMEOUT_MS", "1") };
        let before = rag_failure_count();
        let client = RagClient::new("http://127.0.0.1:1").expect("client");
        let candidates = vec![(Uuid::new_v4(), "hello".to_string())];
        let result = rerank_candidates(&client, "q", &candidates, 5).await;
        assert!(matches!(result, Ok(ref v) if v.is_empty()));
        assert!(rag_failure_count() > before);
        unsafe { std::env::remove_var("MEMD_RAG_TIMEOUT_MS") };
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn ingest_retries_then_records_failure() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe { std::env::set_var("MEMD_RAG_TIMEOUT_MS", "1") };
        let before = rag_failure_count();
        let client = RagClient::new("http://127.0.0.1:1").expect("client");
        let item = dummy_item();
        let result = ingest_item(&client, &item).await;
        assert!(
            result.is_err(),
            "ingest must surface final error after retries"
        );
        assert!(rag_failure_count() > before);
        unsafe { std::env::remove_var("MEMD_RAG_TIMEOUT_MS") };
    }
}

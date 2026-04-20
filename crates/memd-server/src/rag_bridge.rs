use std::{collections::HashSet, sync::Arc, time::Duration};

use memd_rag::{
    RagBackendHealthResponse, RagClient, RagIngestRequest, RagIngestSource, RagRetrieveMode,
    RagRetrieveRequest, RagRerankCandidate, RagRerankRequest,
};
use memd_schema::{MemoryItem, RagHealthStatus, SearchMemoryRequest};
use tracing::warn;
use uuid::Uuid;

const RAG_HEALTH_TIMEOUT: Duration = Duration::from_millis(500);

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
            content: item.content.clone(),
            mime: None,
            bytes: Some(item.content.len() as u64),
            source_quality: item.source_quality,
            source_agent: item.source_agent.clone(),
            source_path: Some(item.id.to_string()),
            tags: item.tags.clone(),
        },
    };
    client.ingest(&request).await.map(|_| ())
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

    let response = client
        .retrieve(&RagRetrieveRequest {
            query: query.to_string(),
            project: req.project.clone(),
            namespace: req.namespace.clone(),
            mode: RagRetrieveMode::Auto,
            limit: Some(req.limit.unwrap_or(10).max(1)),
            include_cross_modal: false,
        })
        .await?;

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

    let response = client
        .rerank(&RagRerankRequest {
            query: query.to_string(),
            candidates: candidates
                .iter()
                .map(|(id, text)| RagRerankCandidate {
                    id: id.to_string(),
                    text: text.clone(),
                })
                .collect(),
            top_k: Some(top_k.max(1).min(candidates.len())),
        })
        .await?;

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
    let Some(client) = client else {
        return RagHealthStatus {
            enabled: false,
            reachable: false,
            name: None,
        };
    };

    match tokio::time::timeout(RAG_HEALTH_TIMEOUT, client.healthz()).await {
        Ok(Ok(response)) => health_from_response(response),
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
            }
        }
        Err(_) => {
            warn!(target: "memd::rag_bridge", "rag health check timed out");
            RagHealthStatus {
                enabled: true,
                reachable: false,
                name: None,
            }
        }
    }
}

fn health_from_response(response: RagBackendHealthResponse) -> RagHealthStatus {
    RagHealthStatus {
        enabled: true,
        reachable: response.backend.connected,
        name: response.backend.name,
    }
}

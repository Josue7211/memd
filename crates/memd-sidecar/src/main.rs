use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use clap::Parser;
use serde::{Deserialize, Serialize};

use memd_sidecar::{
    SidecarBackendHealth, SidecarHealthResponse, SidecarIngestRequest, SidecarIngestResponse,
    SidecarRetrieveItem, SidecarRetrieveMode, SidecarRetrieveRequest, SidecarRetrieveResponse,
};

#[derive(Debug, Parser)]
#[command(name = "rag-sidecar")]
struct Cli {
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    #[arg(long, default_value_t = 9000)]
    port: u16,

    #[arg(long, default_value = ".memd/rag-sidecar.json")]
    state_file: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredRecord {
    project: Option<String>,
    namespace: Option<String>,
    source: SidecarIngestRequest,
}

#[derive(Clone)]
struct AppState {
    state_file: PathBuf,
    records: Arc<RwLock<Vec<StoredRecord>>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let state_file = cli.state_file.clone();
    let records = load_records(&state_file)?;
    let state = AppState {
        state_file,
        records: Arc::new(RwLock::new(records)),
    };

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/ingest", post(ingest))
        .route("/v1/retrieve", post(retrieve))
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", cli.host, cli.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("rag-sidecar listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn healthz() -> impl IntoResponse {
    Json(SidecarHealthResponse {
        status: "ok".to_string(),
        backend: SidecarBackendHealth {
            connected: true,
            name: Some("rag-sidecar".to_string()),
            multimodal: true,
        },
    })
}

async fn ingest(
    State(state): State<AppState>,
    Json(request): Json<SidecarIngestRequest>,
) -> Result<Json<SidecarIngestResponse>, (StatusCode, String)> {
    let mut records = state.records.write().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "state lock poisoned".to_string(),
        )
    })?;
    records.push(StoredRecord {
        project: request.project.clone(),
        namespace: request.namespace.clone(),
        source: request.clone(),
    });
    persist_records(&state.state_file, &records)
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    Ok(Json(SidecarIngestResponse {
        status: "ok".to_string(),
        track_id: request.source.id,
        items: 1,
    }))
}

async fn retrieve(
    State(state): State<AppState>,
    Json(request): Json<SidecarRetrieveRequest>,
) -> Result<Json<SidecarRetrieveResponse>, (StatusCode, String)> {
    let records = state.records.read().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "state lock poisoned".to_string(),
        )
    })?;

    let query_terms = tokenize(&request.query);
    let limit = request.limit.unwrap_or(8).max(1);
    let mut matches = records
        .iter()
        .filter(|record| matches_scope(record, &request))
        .map(|record| {
            let score = score_record(record, &query_terms, &request.mode);
            (score, record)
        })
        .filter(|(score, _)| *score > 0.0)
        .collect::<Vec<_>>();

    matches.sort_by(|left, right| right.0.total_cmp(&left.0));
    matches.truncate(limit);

    let items = matches
        .into_iter()
        .map(|(score, record)| SidecarRetrieveItem {
            content: record.source.source.content.clone(),
            source: record
                .source
                .source
                .source_path
                .clone()
                .or_else(|| record.source.source.source_agent.clone())
                .or_else(|| record.project.clone()),
            score,
        })
        .collect::<Vec<_>>();

    Ok(Json(SidecarRetrieveResponse {
        status: "ok".to_string(),
        mode: request.mode,
        items,
    }))
}

fn load_records(path: &Path) -> anyhow::Result<Vec<StoredRecord>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(path)?;
    let records = serde_json::from_str(&data)?;
    Ok(records)
}

fn persist_records(path: &Path, records: &[StoredRecord]) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(records)?;
    fs::write(path, data)?;
    Ok(())
}

fn matches_scope(record: &StoredRecord, request: &SidecarRetrieveRequest) -> bool {
    request
        .project
        .as_ref()
        .is_none_or(|project| record.project.as_deref() == Some(project.as_str()))
        && request
            .namespace
            .as_ref()
            .is_none_or(|namespace| record.namespace.as_deref() == Some(namespace.as_str()))
}

fn score_record(record: &StoredRecord, query_terms: &[String], mode: &SidecarRetrieveMode) -> f32 {
    if query_terms.is_empty() {
        return 0.25;
    }

    let mut haystack = String::new();
    haystack.push_str(&record.source.source.content.to_lowercase());
    haystack.push(' ');
    haystack.push_str(&record.source.source.kind.to_lowercase());
    haystack.push(' ');
    haystack.push_str(&record.source.source.tags.join(" ").to_lowercase());
    if let Some(path) = record.source.source.source_path.as_deref() {
        haystack.push(' ');
        haystack.push_str(&path.to_lowercase());
    }
    if let Some(agent) = record.source.source.source_agent.as_deref() {
        haystack.push(' ');
        haystack.push_str(&agent.to_lowercase());
    }

    let mut matches = 0.0;
    for term in query_terms {
        if haystack.contains(term) {
            matches += 1.0;
        }
    }

    if matches == 0.0 {
        return 0.0;
    }

    let base = matches / query_terms.len() as f32;
    let mode_bonus = match mode {
        SidecarRetrieveMode::Auto => 0.10,
        SidecarRetrieveMode::Text => 0.05,
        SidecarRetrieveMode::Multimodal => 0.15,
        SidecarRetrieveMode::Graph => 0.12,
    };
    (base + mode_bonus).min(1.0)
}

fn tokenize(query: &str) -> Vec<String> {
    query
        .split(|ch: char| !ch.is_alphanumeric())
        .filter(|term| term.len() > 1)
        .map(|term| term.to_lowercase())
        .collect()
}

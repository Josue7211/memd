use std::{
    collections::{BTreeSet, HashMap},
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use clap::{ArgAction, Parser};
use fastembed::{
    EmbeddingModel, InitOptions, RerankInitOptions, RerankerModel, TextEmbedding, TextRerank,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value as JsonValue, json};

use memd_sidecar::{
    SidecarBackendHealth, SidecarHealthResponse, SidecarIngestRequest, SidecarIngestResponse,
    SidecarRerankCandidate, SidecarRerankItem, SidecarRerankRequest, SidecarRerankResponse,
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

    #[arg(long = "persist", default_value_t = true, action = ArgAction::Set)]
    persist: bool,

    #[arg(long, value_enum, default_value_t = EmbeddingBackend::Sparse)]
    embedding_backend: EmbeddingBackend,

    #[arg(long, default_value = ".memd/models/fastembed")]
    embedding_cache_dir: PathBuf,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
enum EmbeddingBackend {
    Sparse,
    Fastembed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfiguredEmbeddingModel {
    AllMiniLML6V2,
    BGEBaseENV15,
    BGELargeENV15,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredRecord {
    project: Option<String>,
    namespace: Option<String>,
    source: SidecarIngestRequest,
    #[serde(default)]
    normalized_text: String,
    #[serde(default)]
    tokens: Vec<String>,
    #[serde(default)]
    semantic_terms: Vec<String>,
}

#[derive(Clone)]
struct AppState {
    state_file: PathBuf,
    persist: bool,
    records: Arc<RwLock<Vec<StoredRecord>>>,
    embeddings: EmbeddingRuntime,
    reranker: Option<Arc<RerankRuntime>>,
    embedding_profile: String,
}

#[derive(Clone)]
struct EmbeddingRuntime {
    fastembed: Option<Arc<FastembedRuntime>>,
}

struct FastembedRuntime {
    model: Mutex<TextEmbedding>,
    query_cache: Mutex<HashMap<String, Vec<f32>>>,
    record_cache: Mutex<HashMap<uuid::Uuid, Vec<f32>>>,
}

struct LocalRerankRuntime {
    model: Mutex<TextRerank>,
}

struct AnthropicRerankRuntime {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
}

enum RerankBackend {
    Local(Box<LocalRerankRuntime>),
    Anthropic(AnthropicRerankRuntime),
}

struct RerankRuntime {
    primary: RerankBackend,
    local_fallback: Option<Box<LocalRerankRuntime>>,
}

struct RerankRunResult {
    items: Vec<SidecarRerankItem>,
    model: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let state_file = cli.state_file.clone();
    let records = if cli.persist {
        load_records(&state_file)?
    } else {
        Vec::new()
    };
    let reranker = if sidecar_rerank_enabled() {
        RerankRuntime::try_new(&cli.embedding_cache_dir)
            .ok()
            .map(Arc::new)
    } else {
        None
    };
    let state = AppState {
        state_file,
        persist: cli.persist,
        records: Arc::new(RwLock::new(records)),
        embeddings: EmbeddingRuntime::try_new(cli.embedding_backend, &cli.embedding_cache_dir)?,
        reranker: reranker.clone(),
        embedding_profile: match cli.embedding_backend {
            EmbeddingBackend::Sparse => "sparse".to_string(),
            EmbeddingBackend::Fastembed => {
                format!("fastembed:{}", configured_embedding_model_from_env().code())
            }
        },
    };

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/ingest", post(ingest))
        .route("/v1/retrieve", post(retrieve))
        .route("/v1/rerank", post(rerank))
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", cli.host, cli.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!(
        "rag-sidecar listening on http://{} persist={}",
        listener.local_addr()?,
        cli.persist
    );
    axum::serve(listener, app).await?;
    Ok(())
}

async fn healthz(State(state): State<AppState>) -> impl IntoResponse {
    let indexed_count = state
        .records
        .read()
        .map(|records| records.len())
        .unwrap_or(0);
    Json(SidecarHealthResponse {
        status: "ok".to_string(),
        backend: SidecarBackendHealth {
            connected: true,
            name: Some("rag-sidecar".to_string()),
            multimodal: true,
            profile: Some(state.embedding_profile),
            indexed_count: Some(indexed_count),
        },
    })
}

async fn ingest(
    State(state): State<AppState>,
    Json(request): Json<SidecarIngestRequest>,
) -> Result<Json<SidecarIngestResponse>, (StatusCode, String)> {
    let record_id = request.source.id;
    if let Some(runtime) = state.embeddings.fastembed.as_deref() {
        runtime
            .record_cache
            .lock()
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "record embedding cache lock poisoned".to_string(),
                )
            })?
            .remove(&record_id);
    }
    let record = build_stored_record(request);
    if let Some(runtime) = state.embeddings.fastembed.as_deref() {
        runtime.embed_record(&record).map_err(internal_error)?;
    }

    let mut records = state.records.write().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "state lock poisoned".to_string(),
        )
    })?;
    records.retain(|record| record.source.source.id != record_id);
    records.push(record);
    if state.persist {
        persist_records(&state.state_file, &records)
            .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    }

    Ok(Json(SidecarIngestResponse {
        status: "ok".to_string(),
        track_id: record_id,
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
    let query_embedding = state
        .embeddings
        .query_embedding(&request.query)
        .map_err(internal_error)?;
    let limit = request.limit.unwrap_or(8).max(1);
    let mut matches = records
        .iter()
        .filter(|record| matches_scope(record, &request))
        .map(|record| -> Result<_, (StatusCode, String)> {
            let record_embedding = state
                .embeddings
                .record_embedding(record)
                .map_err(internal_error)?;
            let score = score_record(
                record,
                &query_terms,
                query_embedding.as_deref(),
                record_embedding.as_deref(),
                &request.mode,
            );
            Ok((score, record))
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
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

async fn rerank(
    State(state): State<AppState>,
    Json(request): Json<SidecarRerankRequest>,
) -> Result<Json<SidecarRerankResponse>, (StatusCode, String)> {
    let requested = request.top_k.unwrap_or(request.candidates.len());
    let top_k = requested.max(1).min(request.candidates.len().max(1));
    let (items, model) = match state.reranker.as_deref() {
        Some(runtime) => match runtime
            .rerank(&request.query, &request.candidates, top_k)
            .await
        {
            Ok(result) => (result.items, result.model),
            Err(_) => (
                fallback_rerank_candidates(&request.query, &request.candidates, top_k),
                "fallback-heuristic".to_string(),
            ),
        },
        None => (
            fallback_rerank_candidates(&request.query, &request.candidates, top_k),
            "fallback-heuristic".to_string(),
        ),
    };
    Ok(Json(SidecarRerankResponse {
        status: "ok".to_string(),
        model,
        items,
    }))
}

fn load_records(path: &Path) -> anyhow::Result<Vec<StoredRecord>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(path)?;
    let mut records = serde_json::from_str::<Vec<StoredRecord>>(&data)?;
    for record in &mut records {
        if record.normalized_text.is_empty() {
            record.normalized_text = build_record_haystack(&record.source);
        }
        if record.tokens.is_empty() {
            record.tokens = tokenize(&record.normalized_text);
        }
        if record.semantic_terms.is_empty() {
            record.semantic_terms = build_semantic_terms(&record.normalized_text);
        }
    }
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

fn internal_error(error: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

impl EmbeddingRuntime {
    fn try_new(backend: EmbeddingBackend, cache_dir: &Path) -> anyhow::Result<Self> {
        let fastembed = match backend {
            EmbeddingBackend::Sparse => None,
            EmbeddingBackend::Fastembed => {
                let configured_model = configured_embedding_model_from_env();
                let model = TextEmbedding::try_new(
                    InitOptions::new(configured_model.fastembed_model())
                        .with_cache_dir(cache_dir.to_path_buf())
                        .with_show_download_progress(false),
                )?;
                Some(Arc::new(FastembedRuntime {
                    model: Mutex::new(model),
                    query_cache: Mutex::new(HashMap::new()),
                    record_cache: Mutex::new(HashMap::new()),
                }))
            }
        };
        let _ = backend;
        Ok(Self { fastembed })
    }

    fn query_embedding(&self, query: &str) -> anyhow::Result<Option<Vec<f32>>> {
        match &self.fastembed {
            Some(runtime) => runtime.embed_query(query).map(Some),
            None => Ok(None),
        }
    }

    fn record_embedding(&self, record: &StoredRecord) -> anyhow::Result<Option<Vec<f32>>> {
        match &self.fastembed {
            Some(runtime) => runtime.embed_record(record).map(Some),
            None => Ok(None),
        }
    }
}

impl ConfiguredEmbeddingModel {
    fn code(self) -> &'static str {
        match self {
            Self::AllMiniLML6V2 => "all-minilm-l6-v2",
            Self::BGEBaseENV15 => "bge-base-en-v1.5",
            Self::BGELargeENV15 => "bge-large-en-v1.5",
        }
    }

    fn fastembed_model(self) -> EmbeddingModel {
        match self {
            Self::AllMiniLML6V2 => EmbeddingModel::AllMiniLML6V2,
            Self::BGEBaseENV15 => EmbeddingModel::BGEBaseENV15,
            Self::BGELargeENV15 => EmbeddingModel::BGELargeENV15,
        }
    }
}

fn configured_embedding_model_from_env() -> ConfiguredEmbeddingModel {
    match std::env::var("MEMD_EMBED_MODEL")
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .as_deref()
    {
        Some("bge-base-en-v1.5") => ConfiguredEmbeddingModel::BGEBaseENV15,
        Some("bge-large-en-v1.5") => ConfiguredEmbeddingModel::BGELargeENV15,
        _ => ConfiguredEmbeddingModel::AllMiniLML6V2,
    }
}

fn sidecar_rerank_enabled() -> bool {
    match std::env::var("MEMD_SIDECAR_RERANK") {
        Ok(value) => {
            let value = value.trim().to_ascii_lowercase();
            !matches!(value.as_str(), "0" | "false" | "off" | "no" | "heuristic")
        }
        Err(_) => true,
    }
}

impl FastembedRuntime {
    fn embed_query(&self, query: &str) -> anyhow::Result<Vec<f32>> {
        if let Some(cached) = self
            .query_cache
            .lock()
            .map_err(|_| anyhow::anyhow!("query embedding cache lock poisoned"))?
            .get(query)
            .cloned()
        {
            return Ok(cached);
        }
        let query_input = format!("query: {}", query.trim());
        let embedding = self.embed_one(&query_input)?;
        self.query_cache
            .lock()
            .map_err(|_| anyhow::anyhow!("query embedding cache lock poisoned"))?
            .insert(query.to_string(), embedding.clone());
        Ok(embedding)
    }

    fn embed_record(&self, record: &StoredRecord) -> anyhow::Result<Vec<f32>> {
        let record_id = record.source.source.id;
        if let Some(cached) = self
            .record_cache
            .lock()
            .map_err(|_| anyhow::anyhow!("record embedding cache lock poisoned"))?
            .get(&record_id)
            .cloned()
        {
            return Ok(cached);
        }
        let embedding_input = format!("passage: {}", record.normalized_text.trim());
        let embedding = self.embed_one(&embedding_input)?;
        self.record_cache
            .lock()
            .map_err(|_| anyhow::anyhow!("record embedding cache lock poisoned"))?
            .insert(record_id, embedding.clone());
        Ok(embedding)
    }

    fn embed_one(&self, input: &str) -> anyhow::Result<Vec<f32>> {
        let mut model = self
            .model
            .lock()
            .map_err(|_| anyhow::anyhow!("embedding model lock poisoned"))?;
        let embeddings = model.embed(vec![input.to_string()], None)?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("embedding model returned no vectors"))
    }
}

impl LocalRerankRuntime {
    fn try_new(cache_dir: &Path) -> anyhow::Result<Self> {
        std::fs::create_dir_all(cache_dir).ok();
        let model = TextRerank::try_new(
            RerankInitOptions::new(RerankerModel::BGERerankerBase)
                .with_cache_dir(cache_dir.to_path_buf())
                .with_show_download_progress(false),
        )?;
        Ok(Self {
            model: Mutex::new(model),
        })
    }

    fn rerank(
        &self,
        query: &str,
        candidates: &[SidecarRerankCandidate],
        top_k: usize,
    ) -> anyhow::Result<Vec<SidecarRerankItem>> {
        if candidates.is_empty() {
            return Ok(Vec::new());
        }
        let docs = candidates
            .iter()
            .map(|candidate| candidate.text.as_str())
            .collect::<Vec<_>>();
        let mut model = self
            .model
            .lock()
            .map_err(|_| anyhow::anyhow!("reranker model lock poisoned"))?;
        let ranked = model.rerank(query, docs, false, None)?;
        Ok(ranked
            .into_iter()
            .take(top_k)
            .map(|result| SidecarRerankItem {
                id: candidates[result.index].id.clone(),
                score: result.score,
                text: None,
            })
            .collect())
    }
}

impl AnthropicRerankRuntime {
    fn from_env() -> anyhow::Result<Option<Self>> {
        let Some(api_key) = std::env::var("ANTHROPIC_API_KEY")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
        else {
            return Ok(None);
        };
        let base_url = std::env::var("ANTHROPIC_BASE_URL")
            .unwrap_or_else(|_| "https://api.anthropic.com".to_string());
        let model =
            std::env::var("MEMD_RERANK_MODEL").unwrap_or_else(|_| "claude-haiku-4-5".to_string());
        Ok(Some(Self::try_new(base_url, api_key, model)?))
    }

    fn try_new(base_url: String, api_key: String, model: String) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(45))
            .connect_timeout(Duration::from_secs(10))
            .build()?;
        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            model,
        })
    }

    async fn rerank(
        &self,
        query: &str,
        candidates: &[SidecarRerankCandidate],
        top_k: usize,
    ) -> anyhow::Result<Vec<SidecarRerankItem>> {
        if candidates.is_empty() {
            return Ok(Vec::new());
        }
        let payload = json!({
            "query": query,
            "top_k": top_k,
            "candidates": candidates
                .iter()
                .map(|candidate| json!({
                    "id": candidate.id,
                    "text": candidate.text,
                }))
                .collect::<Vec<_>>(),
        });
        let prompt = format!(
            "Rank the candidates for answering the query. Return strict JSON only in the form \
             {{\"items\":[{{\"id\":\"candidate-id\",\"score\":0.0}}]}} sorted best to worst. \
             Include at most {top_k} items.\n\n{}",
            serde_json::to_string_pretty(&payload)?
        );
        let response = self
            .client
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&json!({
                "model": self.model,
                "max_tokens": 768,
                "temperature": 0,
                "messages": [{"role": "user", "content": prompt}],
            }))
            .send()
            .await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("anthropic rerank request failed with {status}: {body}");
        }
        let body = response.json::<JsonValue>().await?;
        let text = body
            .get("content")
            .and_then(JsonValue::as_array)
            .map(|blocks| {
                blocks
                    .iter()
                    .filter_map(|block| {
                        (block.get("type").and_then(JsonValue::as_str) == Some("text"))
                            .then(|| block.get("text").and_then(JsonValue::as_str))
                            .flatten()
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default();
        let json_payload = extract_json_payload(&text)
            .ok_or_else(|| anyhow::anyhow!("anthropic rerank returned no JSON payload"))?;
        let ranked = serde_json::from_str::<AnthropicRerankOutput>(json_payload)?;
        let mut items = Vec::new();
        for item in ranked.items {
            if candidates.iter().any(|candidate| candidate.id == item.id)
                && !items
                    .iter()
                    .any(|existing: &SidecarRerankItem| existing.id == item.id)
            {
                items.push(SidecarRerankItem {
                    id: item.id,
                    score: item.score,
                    text: None,
                });
            }
            if items.len() >= top_k {
                break;
            }
        }
        if items.is_empty() {
            anyhow::bail!("anthropic rerank returned no valid candidate ids");
        }
        Ok(items)
    }
}

impl RerankBackend {
    fn profile(&self) -> &str {
        match self {
            Self::Local(_) => "bge-reranker-base",
            Self::Anthropic(runtime) => runtime.model.as_str(),
        }
    }
}

impl RerankRuntime {
    fn try_new(cache_dir: &Path) -> anyhow::Result<Self> {
        let local = LocalRerankRuntime::try_new(cache_dir).ok().map(Box::new);
        if let Some(runtime) = AnthropicRerankRuntime::from_env()? {
            return Ok(Self {
                primary: RerankBackend::Anthropic(runtime),
                local_fallback: local,
            });
        }
        let Some(local_runtime) = local else {
            anyhow::bail!("no anthropic reranker configured and local reranker unavailable");
        };
        Ok(Self {
            primary: RerankBackend::Local(local_runtime),
            local_fallback: None,
        })
    }

    #[cfg(test)]
    fn anthropic_for_tests(base_url: String, api_key: String, model: String) -> Self {
        Self {
            primary: RerankBackend::Anthropic(
                AnthropicRerankRuntime::try_new(base_url, api_key, model)
                    .expect("build anthropic test runtime"),
            ),
            local_fallback: None,
        }
    }

    async fn rerank(
        &self,
        query: &str,
        candidates: &[SidecarRerankCandidate],
        top_k: usize,
    ) -> anyhow::Result<RerankRunResult> {
        let primary = match &self.primary {
            RerankBackend::Local(runtime) => runtime.rerank(query, candidates, top_k),
            RerankBackend::Anthropic(runtime) => runtime.rerank(query, candidates, top_k).await,
        };
        match primary {
            Ok(items) => Ok(RerankRunResult {
                items,
                model: self.primary.profile().to_string(),
            }),
            Err(primary_error) => {
                let Some(local) = self.local_fallback.as_ref() else {
                    return Err(primary_error);
                };
                Ok(RerankRunResult {
                    items: local.rerank(query, candidates, top_k)?,
                    model: "bge-reranker-base".to_string(),
                })
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct AnthropicRerankOutput {
    items: Vec<AnthropicRerankOutputItem>,
}

#[derive(Debug, Deserialize)]
struct AnthropicRerankOutputItem {
    id: String,
    score: f32,
}

fn extract_json_payload(raw: &str) -> Option<&str> {
    let object = raw
        .find('{')
        .zip(raw.rfind('}'))
        .filter(|(start, end)| start <= end)
        .map(|(start, end)| &raw[start..=end]);
    if object.is_some() {
        return object;
    }
    raw.find('[')
        .zip(raw.rfind(']'))
        .filter(|(start, end)| start <= end)
        .map(|(start, end)| &raw[start..=end])
}

fn score_record(
    record: &StoredRecord,
    query_terms: &[String],
    query_embedding: Option<&[f32]>,
    record_embedding: Option<&[f32]>,
    mode: &SidecarRetrieveMode,
) -> f32 {
    if query_terms.is_empty() {
        return 0.25;
    }

    let query_token_set = query_terms.iter().cloned().collect::<BTreeSet<_>>();
    let query_keywords = extract_keywords(query_terms);
    let query_bigrams = build_query_bigrams(query_terms);
    let name_tokens = extract_name_tokens(&record.source.source.content);
    let record_token_set = record.tokens.iter().cloned().collect::<BTreeSet<_>>();
    let overlap = query_token_set.intersection(&record_token_set).count() as f32;
    let lexical = overlap / query_token_set.len().max(1) as f32;

    let token_frequency = record.tokens.iter().fold(HashMap::new(), |mut acc, token| {
        *acc.entry(token.as_str()).or_insert(0usize) += 1;
        acc
    });
    let bm25ish = query_keywords
        .iter()
        .map(|keyword| {
            let frequency = token_frequency.get(keyword.as_str()).copied().unwrap_or(0) as f32;
            if frequency == 0.0 {
                0.0
            } else {
                frequency / (frequency + 1.2)
            }
        })
        .sum::<f32>()
        / query_keywords.len().max(1) as f32;

    let query_semantic_terms = build_semantic_terms(&query_terms.join(" "));
    let semantic = cosine_similarity(&query_semantic_terms, &record.semantic_terms);
    let concept_bonus = memory_concept_overlap_bonus(&query_keywords, record);
    let dense_semantic = match (query_embedding, record_embedding) {
        (Some(query), Some(record)) => dense_cosine_similarity(query, record),
        _ => 0.0,
    };

    let phrase_bonus = if query_terms.len() >= 2 {
        let query_phrase = query_terms.join(" ");
        if record.normalized_text.contains(&query_phrase) {
            0.35
        } else {
            0.0
        }
    } else {
        0.0
    };

    let keyword_bonus = query_keywords
        .iter()
        .filter(|keyword| record.normalized_text.contains(keyword.as_str()))
        .count() as f32
        / query_keywords.len().max(1) as f32;
    let bigram_bonus = query_bigrams
        .iter()
        .filter(|bigram| record.normalized_text.contains(bigram.as_str()))
        .count() as f32
        / query_bigrams.len().max(1) as f32;

    let name_bonus = query_keywords
        .iter()
        .filter(|keyword| name_tokens.contains(keyword.as_str()))
        .count() as f32
        / query_keywords.len().max(1) as f32;
    let path_lower = record
        .source
        .source
        .source_path
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let path_bonus = query_keywords
        .iter()
        .filter(|keyword| path_lower.contains(keyword.as_str()))
        .count() as f32
        / query_keywords.len().max(1) as f32;
    let tag_bonus = query_keywords
        .iter()
        .filter(|keyword| {
            record
                .source
                .source
                .tags
                .iter()
                .any(|tag| tag.to_ascii_lowercase().contains(keyword.as_str()))
        })
        .count() as f32
        / query_keywords.len().max(1) as f32;

    let mut score = if dense_semantic > 0.0 {
        lexical * 0.22 + bm25ish * 0.18 + semantic * 0.20 + dense_semantic * 0.40
    } else {
        lexical * 0.35 + bm25ish * 0.25 + semantic * 0.40
    };
    if lexical == 0.0
        && bm25ish == 0.0
        && semantic == 0.0
        && dense_semantic == 0.0
        && concept_bonus == 0.0
    {
        return 0.0;
    }

    score += phrase_bonus;
    score += keyword_bonus * 0.20;
    score += bigram_bonus * 0.18;
    score += name_bonus * 0.10;
    score += path_bonus * 0.16;
    score += tag_bonus * 0.12;
    score += concept_bonus * 0.42;

    let mode_bonus = match mode {
        SidecarRetrieveMode::Auto => {
            0.10 + keyword_bonus * 0.10 + bigram_bonus * 0.08 + dense_semantic * 0.05
        }
        SidecarRetrieveMode::Text => 0.05,
        SidecarRetrieveMode::Multimodal => 0.12 + phrase_bonus * 0.25 + dense_semantic * 0.05,
        SidecarRetrieveMode::Graph => {
            0.10 + name_bonus * 0.15 + path_bonus * 0.10 + dense_semantic * 0.05
        }
    };
    (score + mode_bonus).clamp(0.0, 4.0)
}

fn memory_concept_overlap_bonus(query_keywords: &[String], record: &StoredRecord) -> f32 {
    if query_keywords.is_empty() {
        return 0.0;
    }
    let query_tokens = query_keywords
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let record_tokens = record
        .tokens
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let clusters: &[&[&str]] = &[
        &[
            "restart",
            "resume",
            "crashed",
            "crash",
            "interrupted",
            "session",
            "context",
            "breadcrumb",
            "breadcrumbs",
        ],
        &[
            "harness",
            "switch",
            "assistant",
            "assistants",
            "shared",
            "claude",
            "codex",
            "ollama",
            "hermes",
        ],
        &[
            "gateway", "packet", "compact", "context", "local", "model", "models", "evidence",
            "guard", "trusted",
        ],
        &[
            "offline",
            "unavailable",
            "backend",
            "server",
            "queue",
            "queued",
            "failed",
            "returns",
            "fallback",
            "network",
        ],
        &[
            "alias",
            "aliases",
            "misspelled",
            "fuzzy",
            "path",
            "paths",
            "file",
            "files",
            "command",
            "commands",
            "identifier",
            "identifiers",
            "ids",
            "names",
        ],
        &[
            "trace", "explain", "selected", "evidence", "lexical", "dense", "trust", "recency",
            "rerank",
        ],
        &[
            "truth",
            "correction",
            "corrected",
            "latest",
            "stale",
            "old",
            "outdated",
            "supersedes",
            "replaced",
            "provenance",
        ],
        &[
            "privacy",
            "private",
            "visibility",
            "leak",
            "leaking",
            "blocks",
            "filter",
            "workspace",
            "agent",
            "agents",
        ],
        &[
            "sync",
            "synchronized",
            "devices",
            "sessions",
            "self",
            "hosted",
            "backend",
            "authority",
            "central",
        ],
        &[
            "procedure",
            "procedures",
            "workflow",
            "workflows",
            "runbook",
            "runbooks",
            "repeated",
            "reuse",
            "operational",
            "invocation",
        ],
        &[
            "atlas",
            "entity",
            "entities",
            "related",
            "people",
            "projects",
            "sessions",
            "decisions",
            "links",
        ],
        &[
            "firewall",
            "policy",
            "policies",
            "tools",
            "instruction",
            "instructions",
            "attacks",
            "quarantine",
            "quarantines",
            "prevent",
        ],
        &[
            "bench",
            "benchmark",
            "model",
            "models",
            "vector",
            "embedding",
            "profiles",
            "qrels",
            "recall",
            "mrr",
            "latency",
            "cost",
        ],
        &[
            "sidecar",
            "vector",
            "dense",
            "boost",
            "booster",
            "candidate",
            "candidates",
            "optional",
            "truth",
        ],
        &[
            "rerank",
            "reranker",
            "sort",
            "relevance",
            "reorder",
            "candidate",
            "candidates",
            "retrieved",
        ],
        &[
            "local",
            "localfirst",
            "files",
            "boot",
            "bundle",
            "wake",
            "mem",
            "events",
            "config",
            "network",
        ],
        &[
            "ollama", "local", "models", "trusted", "evidence", "source", "ids", "guard", "prompt",
        ],
        &[
            "dedupe",
            "duplicate",
            "same",
            "fact",
            "facts",
            "repeatedly",
            "reinforces",
            "noisy",
        ],
        &[
            "events", "event", "audit", "changed", "time", "capture", "promote", "correct",
            "retrieve",
        ],
        &[
            "scope",
            "global",
            "project",
            "workspace",
            "private",
            "leak",
            "leaking",
            "rules",
            "separate",
        ],
        &[
            "ranking", "combine", "fusion", "fts", "fuzzy", "atlas", "dense", "truth", "rerank",
            "signals", "lanes",
        ],
    ];
    let mut matched = 0usize;
    let mut possible = 0usize;
    for cluster in clusters {
        let query_hit = cluster.iter().any(|term| query_tokens.contains(term));
        if !query_hit {
            continue;
        }
        possible += 1;
        if cluster.iter().any(|term| record_tokens.contains(term)) {
            matched += 1;
        }
    }
    if possible == 0 {
        0.0
    } else {
        (matched as f32 / possible as f32).min(1.0)
    }
}

fn fallback_rerank_candidates(
    query: &str,
    candidates: &[SidecarRerankCandidate],
    top_k: usize,
) -> Vec<SidecarRerankItem> {
    let query_terms = tokenize(query);
    let query_token_set = query_terms.iter().cloned().collect::<BTreeSet<_>>();
    let query_keywords = extract_keywords(&query_terms);
    let query_bigrams = build_query_bigrams(&query_terms);
    let query_phrase = query_terms.join(" ");

    let mut ranked = candidates
        .iter()
        .map(|candidate| {
            let normalized = candidate.text.trim().to_ascii_lowercase();
            let tokens = tokenize(&candidate.text);
            let token_set = tokens.iter().cloned().collect::<BTreeSet<_>>();
            let lexical = query_token_set.intersection(&token_set).count() as f32
                / query_token_set.len().max(1) as f32;
            let keyword_bonus = if query_keywords.is_empty() {
                0.0
            } else {
                query_keywords
                    .iter()
                    .filter(|keyword| normalized.contains(keyword.as_str()))
                    .count() as f32
                    / query_keywords.len() as f32
            };
            let bigram_bonus = if query_bigrams.is_empty() {
                0.0
            } else {
                query_bigrams
                    .iter()
                    .filter(|bigram| normalized.contains(bigram.as_str()))
                    .count() as f32
                    / query_bigrams.len() as f32
            };
            let phrase_bonus = if query_terms.len() >= 2 && normalized.contains(&query_phrase) {
                0.4
            } else {
                0.0
            };
            let score = lexical + keyword_bonus * 0.4 + bigram_bonus * 0.5 + phrase_bonus;
            (candidate, score)
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| right.1.total_cmp(&left.1));
    ranked
        .into_iter()
        .take(top_k)
        .map(|(candidate, score)| SidecarRerankItem {
            id: candidate.id.clone(),
            score,
            text: None,
        })
        .collect()
}

fn tokenize(query: &str) -> Vec<String> {
    query
        .split(|ch: char| !ch.is_alphanumeric())
        .filter(|term| term.len() > 1)
        .map(|term| term.to_lowercase())
        .collect()
}

fn build_semantic_terms(text: &str) -> Vec<String> {
    let tokens = tokenize(text);
    let mut features = Vec::new();
    for token in &tokens {
        features.push(format!("tok:{token}"));
        if token.len() >= 4 {
            for trigram in token.as_bytes().windows(3) {
                if let Ok(fragment) = std::str::from_utf8(trigram) {
                    features.push(format!("tri:{fragment}"));
                }
            }
        }
    }
    for pair in tokens.windows(2) {
        features.push(format!("bi:{}_{}", pair[0], pair[1]));
    }
    features
}

fn build_query_bigrams(query_terms: &[String]) -> Vec<String> {
    query_terms
        .windows(2)
        .map(|pair| format!("{} {}", pair[0], pair[1]))
        .collect()
}

fn cosine_similarity(left: &[String], right: &[String]) -> f32 {
    if left.is_empty() || right.is_empty() {
        return 0.0;
    }
    let left_freq = feature_frequency(left);
    let right_freq = feature_frequency(right);
    let mut dot = 0.0f32;
    for (feature, left_weight) in &left_freq {
        if let Some(right_weight) = right_freq.get(feature) {
            dot += left_weight * right_weight;
        }
    }
    if dot == 0.0 {
        return 0.0;
    }
    let left_norm = left_freq
        .values()
        .map(|weight| weight * weight)
        .sum::<f32>()
        .sqrt();
    let right_norm = right_freq
        .values()
        .map(|weight| weight * weight)
        .sum::<f32>()
        .sqrt();
    if left_norm == 0.0 || right_norm == 0.0 {
        0.0
    } else {
        dot / (left_norm * right_norm)
    }
}

fn dense_cosine_similarity(left: &[f32], right: &[f32]) -> f32 {
    if left.is_empty() || right.is_empty() || left.len() != right.len() {
        return 0.0;
    }
    let dot = left
        .iter()
        .zip(right.iter())
        .map(|(l, r)| l * r)
        .sum::<f32>();
    if dot == 0.0 {
        return 0.0;
    }
    let left_norm = left.iter().map(|value| value * value).sum::<f32>().sqrt();
    let right_norm = right.iter().map(|value| value * value).sum::<f32>().sqrt();
    if left_norm == 0.0 || right_norm == 0.0 {
        0.0
    } else {
        dot / (left_norm * right_norm)
    }
}

fn feature_frequency(features: &[String]) -> HashMap<&str, f32> {
    let mut frequency = HashMap::new();
    for feature in features {
        *frequency.entry(feature.as_str()).or_insert(0.0) += 1.0;
    }
    frequency
}

fn build_record_haystack(request: &SidecarIngestRequest) -> String {
    let mut haystack = String::new();
    haystack.push_str(&request.source.content.to_lowercase());
    haystack.push(' ');
    haystack.push_str(&request.source.kind.to_lowercase());
    haystack.push(' ');
    haystack.push_str(&request.source.tags.join(" ").to_lowercase());
    if let Some(path) = request.source.source_path.as_deref() {
        haystack.push(' ');
        haystack.push_str(&path.to_lowercase());
    }
    if let Some(agent) = request.source.source_agent.as_deref() {
        haystack.push(' ');
        haystack.push_str(&agent.to_lowercase());
    }
    haystack
}

fn build_stored_record(request: SidecarIngestRequest) -> StoredRecord {
    let normalized_text = build_record_haystack(&request);
    let tokens = tokenize(&normalized_text);
    let semantic_terms = build_semantic_terms(&normalized_text);
    StoredRecord {
        project: request.project.clone(),
        namespace: request.namespace.clone(),
        source: request,
        normalized_text,
        tokens,
        semantic_terms,
    }
}

fn extract_keywords(query_terms: &[String]) -> Vec<String> {
    let stop_words = [
        "what", "when", "where", "who", "how", "which", "did", "do", "was", "were", "have", "has",
        "had", "is", "are", "the", "a", "an", "my", "me", "i", "you", "your", "their", "it", "its",
        "in", "on", "at", "to", "for", "of", "with", "by", "from", "ago", "last", "that", "this",
        "there", "about", "get", "got", "give", "gave", "buy", "bought", "made", "make", "said",
        "would", "could", "should", "might", "can", "will", "shall", "kind", "type", "like",
        "prefer", "enjoy", "think", "feel",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    query_terms
        .iter()
        .filter(|token| token.len() >= 3 && !stop_words.contains(token.as_str()))
        .cloned()
        .collect()
}

fn extract_name_tokens(content: &str) -> BTreeSet<String> {
    content
        .split(|ch: char| !ch.is_ascii_alphabetic())
        .filter(|token| {
            token.len() >= 3
                && token
                    .chars()
                    .next()
                    .is_some_and(|first| first.is_ascii_uppercase())
        })
        .map(|token| token.to_ascii_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use memd_sidecar::{SidecarIngestSource, SidecarRerankRequest, SidecarRetrieveMode};
    use tower::util::ServiceExt;

    fn build_record(content: &str, path: Option<&str>, tags: &[&str], kind: &str) -> StoredRecord {
        let request = SidecarIngestRequest {
            project: Some("bench".to_string()),
            namespace: Some("ns".to_string()),
            source: SidecarIngestSource {
                id: uuid::Uuid::new_v4(),
                kind: kind.to_string(),
                content: content.to_string(),
                mime: None,
                bytes: Some(content.len() as u64),
                source_quality: None,
                source_agent: Some("public-benchmark".to_string()),
                source_path: path.map(str::to_string),
                tags: tags.iter().map(|tag| (*tag).to_string()).collect(),
            },
        };
        let normalized_text = build_record_haystack(&request);
        let tokens = tokenize(&normalized_text);
        let semantic_terms = build_semantic_terms(&normalized_text);
        StoredRecord {
            project: request.project.clone(),
            namespace: request.namespace.clone(),
            source: request,
            normalized_text,
            tokens,
            semantic_terms,
        }
    }

    #[test]
    fn score_record_prefers_phrase_and_keyword_overlap() {
        let query = tokenize("What did Brenda document about qualification workflow");
        let weak = build_record(
            "Brenda scheduled the demo and sent the invite.",
            Some("session_0_mid_1"),
            &["public-benchmark"],
            "turn",
        );
        let strong = build_record(
            "Brenda documented the MEDDIC qualification workflow for the deal review.",
            Some("session_0_mid_2"),
            &["public-benchmark"],
            "turn",
        );
        let weak_score = score_record(&weak, &query, None, None, &SidecarRetrieveMode::Auto);
        let strong_score = score_record(&strong, &query, None, None, &SidecarRetrieveMode::Auto);
        assert!(strong_score > weak_score);
    }

    #[test]
    fn score_record_uses_source_path_and_tags_as_retrieval_signal() {
        let query = tokenize("longmemeval turn 3");
        let record = build_record(
            "neutral content",
            Some("longmemeval_turn_3"),
            &["public-benchmark", "longmemeval"],
            "turn",
        );
        let score = score_record(&record, &query, None, None, &SidecarRetrieveMode::Text);
        assert!(score > 0.0);
    }

    #[test]
    fn cosine_similarity_rewards_related_surface_forms() {
        let left = build_semantic_terms("qualification workflow documented");
        let right = build_semantic_terms("qualifying workflows and documentation");
        let unrelated = build_semantic_terms("camping by the lake at sunrise");
        assert!(cosine_similarity(&left, &right) > cosine_similarity(&left, &unrelated));
    }

    #[test]
    fn score_record_prefers_adjacent_bigrams_and_field_hits() {
        let query = tokenize("target thread handoff");
        let weak = build_record(
            "send the packet somewhere later",
            Some("session_misc"),
            &["coordination"],
            "turn",
        );
        let strong = build_record(
            "send the handoff to the target thread now",
            Some("target_thread_handoff"),
            &["coordination", "handoff"],
            "turn",
        );
        let weak_score = score_record(&weak, &query, None, None, &SidecarRetrieveMode::Auto);
        let strong_score = score_record(&strong, &query, None, None, &SidecarRetrieveMode::Auto);
        assert!(strong_score > weak_score);
    }

    #[test]
    fn dense_cosine_similarity_rewards_related_vectors() {
        let aligned = dense_cosine_similarity(&[1.0, 0.5, 0.0], &[0.9, 0.45, 0.0]);
        let opposed = dense_cosine_similarity(&[1.0, 0.5, 0.0], &[0.0, -0.2, 1.0]);
        assert!(aligned > opposed);
    }

    #[test]
    fn score_record_preserves_dense_score_separation() {
        let query = tokenize("safe compact context for a local model");
        let record = build_record(
            "Cedar packet carries labeled evidence into local models while blocking raw instruction dumps.",
            Some("cedar_context_packet"),
            &["context", "ollama"],
            "fact",
        );
        let strong = score_record(
            &record,
            &query,
            Some(&[1.0, 0.0]),
            Some(&[0.98, 0.02]),
            &SidecarRetrieveMode::Auto,
        );
        let weak = score_record(
            &record,
            &query,
            Some(&[1.0, 0.0]),
            Some(&[0.10, 0.90]),
            &SidecarRetrieveMode::Auto,
        );
        assert!(strong > weak + 0.20);
        assert!(strong > 1.0);
    }

    #[test]
    fn memory_concept_overlap_bridges_memory_domain_synonyms() {
        let query = tokenize("resume a crashed assistant session with prior context");
        let expected = build_record(
            "Aster capsule preserves restart breadcrumbs after interrupted agent work.",
            Some("restart_capsule"),
            &["restart"],
            "fact",
        );
        let unrelated = build_record(
            "Quartz bench compares embedding profiles by recall MRR latency and cost.",
            Some("model_bench"),
            &["modelbench"],
            "fact",
        );
        let expected_score =
            score_record(&expected, &query, None, None, &SidecarRetrieveMode::Auto);
        let unrelated_score =
            score_record(&unrelated, &query, None, None, &SidecarRetrieveMode::Auto);
        assert!(expected_score > unrelated_score);
    }

    #[tokio::test]
    async fn rerank_route_prefers_stronger_phrase_match() {
        let state = AppState {
            state_file: PathBuf::from("/tmp/memd-sidecar-rerank-test.json"),
            persist: false,
            records: Arc::new(RwLock::new(Vec::new())),
            embeddings: EmbeddingRuntime { fastembed: None },
            reranker: None,
            embedding_profile: "sparse".to_string(),
        };
        let app = Router::new()
            .route("/v1/rerank", post(rerank))
            .with_state(state);
        let response = app
            .oneshot(
                Request::post("/v1/rerank")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&SidecarRerankRequest {
                            query: "qualification workflow".to_string(),
                            candidates: vec![
                                SidecarRerankCandidate {
                                    id: "weak".to_string(),
                                    text: "Brenda scheduled the demo and sent the invite."
                                        .to_string(),
                                },
                                SidecarRerankCandidate {
                                    id: "strong".to_string(),
                                    text:
                                        "Brenda documented the MEDDIC qualification workflow for the deal review."
                                            .to_string(),
                                },
                            ],
                            top_k: Some(2),
                        })
                        .expect("serialize rerank request"),
                    ))
                    .expect("build rerank request"),
            )
            .await
            .expect("run rerank route");
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read rerank body");
        let payload: SidecarRerankResponse =
            serde_json::from_slice(&body).expect("decode rerank response");
        assert_eq!(
            payload.items.first().map(|item| item.id.as_str()),
            Some("strong")
        );
    }

    #[tokio::test]
    async fn rerank_route_uses_anthropic_runtime_when_configured() {
        let anthropic_app = Router::new().route(
            "/v1/messages",
            post(|headers: axum::http::HeaderMap, Json(body): Json<serde_json::Value>| async move {
                assert_eq!(
                    headers
                        .get("x-api-key")
                        .and_then(|value| value.to_str().ok()),
                    Some("test-key")
                );
                assert_eq!(
                    headers
                        .get("anthropic-version")
                        .and_then(|value| value.to_str().ok()),
                    Some("2023-06-01")
                );
                assert_eq!(body.get("model").and_then(|value| value.as_str()), Some("claude-haiku-4-5"));
                Json(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": "{\"items\":[{\"id\":\"strong\",\"score\":0.97},{\"id\":\"weak\",\"score\":0.14}]}"
                        }
                    ]
                }))
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind anthropic test server");
        let base_url = format!("http://{}", listener.local_addr().expect("listener addr"));
        tokio::spawn(async move {
            axum::serve(listener, anthropic_app)
                .await
                .expect("serve anthropic test server");
        });

        let state = AppState {
            state_file: PathBuf::from("/tmp/memd-sidecar-anthropic-rerank-test.json"),
            persist: false,
            records: Arc::new(RwLock::new(Vec::new())),
            embeddings: EmbeddingRuntime { fastembed: None },
            reranker: Some(Arc::new(RerankRuntime::anthropic_for_tests(
                base_url,
                "test-key".to_string(),
                "claude-haiku-4-5".to_string(),
            ))),
            embedding_profile: "sparse".to_string(),
        };
        let app = Router::new()
            .route("/v1/rerank", post(rerank))
            .with_state(state);
        let response = app
            .oneshot(
                Request::post("/v1/rerank")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&SidecarRerankRequest {
                            query: "qualification workflow".to_string(),
                            candidates: vec![
                                SidecarRerankCandidate {
                                    id: "weak".to_string(),
                                    text: "Brenda scheduled the demo and sent the invite."
                                        .to_string(),
                                },
                                SidecarRerankCandidate {
                                    id: "strong".to_string(),
                                    text:
                                        "Brenda documented the MEDDIC qualification workflow for the deal review."
                                            .to_string(),
                                },
                            ],
                            top_k: Some(2),
                        })
                        .expect("serialize rerank request"),
                    ))
                    .expect("build rerank request"),
            )
            .await
            .expect("run rerank route");
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read rerank body");
        let payload: SidecarRerankResponse =
            serde_json::from_slice(&body).expect("decode rerank response");
        assert_eq!(payload.model, "claude-haiku-4-5");
        assert_eq!(
            payload.items.first().map(|item| item.id.as_str()),
            Some("strong")
        );
    }
}

use anyhow::Context;
use chrono::{DateTime, Utc};
use memd_schema::{MemoryKind, MemoryScope, MemoryStage, MemoryStatus, SourceQuality};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone)]
pub struct RagClient {
    base_url: String,
    http: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagRecord {
    pub id: Uuid,
    pub content: String,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub kind: MemoryKind,
    pub scope: MemoryScope,
    pub stage: MemoryStage,
    pub status: MemoryStatus,
    pub source_quality: Option<SourceQuality>,
    pub source_agent: Option<String>,
    pub source_path: Option<String>,
    pub redundancy_key: Option<String>,
    pub canonical_key: Option<String>,
    pub confidence: f32,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RagQuery {
    pub query: Option<String>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub kind: Option<MemoryKind>,
    pub scope: Option<MemoryScope>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagSearchHit {
    pub id: Uuid,
    pub content: String,
    pub score: f32,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub kind: MemoryKind,
    pub scope: MemoryScope,
    pub stage: MemoryStage,
    pub status: MemoryStatus,
    pub source_quality: Option<SourceQuality>,
    pub source_agent: Option<String>,
    pub source_path: Option<String>,
    pub redundancy_key: Option<String>,
    pub canonical_key: Option<String>,
    pub confidence: f32,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagSearchResponse {
    pub items: Vec<RagSearchHit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagUpsertResponse {
    pub item: RagRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagHealthResponse {
    pub status: String,
    pub items: Option<usize>,
}

impl RagClient {
    pub fn new(base_url: impl AsRef<str>) -> anyhow::Result<Self> {
        let base_url = normalize_base_url(base_url.as_ref())?;
        let http = reqwest::Client::builder()
            .build()
            .context("build rag http client")?;
        Ok(Self { base_url, http })
    }

    pub async fn healthz(&self) -> anyhow::Result<RagHealthResponse> {
        self.get_json("/healthz").await
    }

    pub async fn upsert(&self, record: &RagRecord) -> anyhow::Result<RagUpsertResponse> {
        self.post_json("/records/upsert", record).await
    }

    pub async fn search(&self, query: &RagQuery) -> anyhow::Result<RagSearchResponse> {
        self.post_json("/records/search", query).await
    }

    async fn get_json<T>(&self, path: &str) -> anyhow::Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, path);
        let response = self.http.get(url).send().await.context("send rag get")?;
        decode_response(response).await
    }

    async fn post_json<T, B>(&self, path: &str, body: &B) -> anyhow::Result<T>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize + ?Sized,
    {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .post(url)
            .json(body)
            .send()
            .await
            .context("send rag post")?;
        decode_response(response).await
    }
}

impl From<&memd_schema::MemoryItem> for RagRecord {
    fn from(item: &memd_schema::MemoryItem) -> Self {
        Self {
            id: item.id,
            content: item.content.clone(),
            project: item.project.clone(),
            namespace: item.namespace.clone(),
            kind: item.kind,
            scope: item.scope,
            stage: item.stage,
            status: item.status,
            source_quality: item.source_quality,
            source_agent: item.source_agent.clone(),
            source_path: item.source_path.clone(),
            redundancy_key: item.redundancy_key.clone(),
            canonical_key: None,
            confidence: item.confidence,
            updated_at: item.updated_at,
            tags: item.tags.clone(),
        }
    }
}

async fn decode_response<T>(response: reqwest::Response) -> anyhow::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.context("read rag response body")?;
        anyhow::bail!("rag request failed with {status}: {body}");
    }

    response
        .json::<T>()
        .await
        .context("decode rag response payload")
}

fn normalize_base_url(input: &str) -> anyhow::Result<String> {
    let mut url = url::Url::parse(input)
        .or_else(|_| url::Url::parse(&format!("http://{input}")))
        .context("parse rag base url")?;

    if url.path() != "/" {
        let path = url.path().trim_end_matches('/');
        if !path.is_empty() && path != "/" {
            anyhow::bail!("rag base url must not include a path: {input}");
        }
    }

    url.set_path("");
    Ok(url.to_string().trim_end_matches('/').to_string())
}

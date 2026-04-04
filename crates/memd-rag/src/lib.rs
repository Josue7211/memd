use anyhow::Context;
use memd_schema::{MemoryItem, SourceQuality};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone)]
pub struct RagClient {
    base_url: String,
    http: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagBackendHealthResponse {
    pub status: String,
    pub backend: RagBackendHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagBackendHealth {
    pub connected: bool,
    pub name: Option<String>,
    pub multimodal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagIngestSource {
    pub id: Uuid,
    pub kind: String,
    pub content: String,
    pub source_quality: Option<SourceQuality>,
    pub source_agent: Option<String>,
    pub source_path: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagIngestRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub source: RagIngestSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagIngestResponse {
    pub status: String,
    pub track_id: Uuid,
    pub items: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RagRetrieveMode {
    Auto,
    Text,
    Multimodal,
    Graph,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagRetrieveRequest {
    pub query: String,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub mode: RagRetrieveMode,
    pub limit: Option<usize>,
    pub include_cross_modal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagRetrieveItem {
    pub content: String,
    pub source: Option<String>,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagRetrieveResponse {
    pub status: String,
    pub mode: RagRetrieveMode,
    pub items: Vec<RagRetrieveItem>,
}

impl RagClient {
    pub fn new(base_url: impl AsRef<str>) -> anyhow::Result<Self> {
        let base_url = normalize_base_url(base_url.as_ref())?;
        let http = reqwest::Client::builder()
            .build()
            .context("build rag http client")?;
        Ok(Self { base_url, http })
    }

    pub async fn healthz(&self) -> anyhow::Result<RagBackendHealthResponse> {
        self.get_json("/healthz").await
    }

    pub async fn ingest(&self, req: &RagIngestRequest) -> anyhow::Result<RagIngestResponse> {
        self.post_json("/v1/ingest", req).await
    }

    pub async fn retrieve(&self, req: &RagRetrieveRequest) -> anyhow::Result<RagRetrieveResponse> {
        self.post_json("/v1/retrieve", req).await
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

impl From<&MemoryItem> for RagIngestSource {
    fn from(item: &MemoryItem) -> Self {
        Self {
            id: item.id,
            kind: format!("{:?}", item.kind).to_lowercase(),
            content: item.content.clone(),
            source_quality: item.source_quality,
            source_agent: item.source_agent.clone(),
            source_path: item.source_path.clone(),
            tags: item.tags.clone(),
        }
    }
}

impl From<&MemoryItem> for RagIngestRequest {
    fn from(item: &MemoryItem) -> Self {
        Self {
            project: item.project.clone(),
            namespace: item.namespace.clone(),
            source: RagIngestSource::from(item),
        }
    }
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

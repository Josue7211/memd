use anyhow::Context;
use memd_schema::{MemoryItem, SourceQuality};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone)]
pub struct SidecarClient {
    base_url: String,
    http: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarHealthResponse {
    pub status: String,
    pub backend: SidecarBackendHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarBackendHealth {
    pub connected: bool,
    pub name: Option<String>,
    pub multimodal: bool,
    #[serde(default)]
    pub profile: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarIngestSource {
    pub id: Uuid,
    pub kind: String,
    pub content: String,
    pub mime: Option<String>,
    pub bytes: Option<u64>,
    pub source_quality: Option<SourceQuality>,
    pub source_agent: Option<String>,
    pub source_path: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarIngestRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub source: SidecarIngestSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarIngestResponse {
    pub status: String,
    pub track_id: Uuid,
    pub items: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SidecarRetrieveMode {
    Auto,
    Text,
    Multimodal,
    Graph,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarRetrieveRequest {
    pub query: String,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub mode: SidecarRetrieveMode,
    pub limit: Option<usize>,
    pub include_cross_modal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarRetrieveItem {
    pub content: String,
    pub source: Option<String>,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarRetrieveResponse {
    pub status: String,
    pub mode: SidecarRetrieveMode,
    pub items: Vec<SidecarRetrieveItem>,
}

pub mod fixtures;

impl SidecarClient {
    pub fn new(base_url: impl AsRef<str>) -> anyhow::Result<Self> {
        let base_url = normalize_base_url(base_url.as_ref())?;
        let http = reqwest::Client::builder()
            .build()
            .context("build sidecar http client")?;
        Ok(Self { base_url, http })
    }

    pub async fn healthz(&self) -> anyhow::Result<SidecarHealthResponse> {
        self.get_json("/healthz").await
    }

    pub async fn ingest(
        &self,
        req: &SidecarIngestRequest,
    ) -> anyhow::Result<SidecarIngestResponse> {
        self.post_json("/v1/ingest", req).await
    }

    pub async fn retrieve(
        &self,
        req: &SidecarRetrieveRequest,
    ) -> anyhow::Result<SidecarRetrieveResponse> {
        self.post_json("/v1/retrieve", req).await
    }

    async fn get_json<T>(&self, path: &str) -> anyhow::Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .get(url)
            .send()
            .await
            .context("send sidecar get")?;
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
            .context("send sidecar post")?;
        decode_response(response).await
    }
}

impl From<&MemoryItem> for SidecarIngestSource {
    fn from(item: &MemoryItem) -> Self {
        Self {
            id: item.id,
            kind: format!("{:?}", item.kind).to_lowercase(),
            content: item.content.clone(),
            mime: None,
            bytes: None,
            source_quality: item.source_quality,
            source_agent: item.source_agent.clone(),
            source_path: item.source_path.clone(),
            tags: item.tags.clone(),
        }
    }
}

impl From<&MemoryItem> for SidecarIngestRequest {
    fn from(item: &MemoryItem) -> Self {
        Self {
            project: item.project.clone(),
            namespace: item.namespace.clone(),
            source: SidecarIngestSource::from(item),
        }
    }
}

fn normalize_base_url(input: &str) -> anyhow::Result<String> {
    let mut url = url::Url::parse(input)
        .or_else(|_| url::Url::parse(&format!("http://{input}")))
        .context("parse sidecar base url")?;

    if url.path() != "/" {
        let path = url.path().trim_end_matches('/');
        if !path.is_empty() && path != "/" {
            anyhow::bail!("sidecar base url must not include a path: {input}");
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
        let body = response
            .text()
            .await
            .context("read sidecar response body")?;
        anyhow::bail!("sidecar request failed with {status}: {body}");
    }

    response
        .json::<T>()
        .await
        .context("decode sidecar response payload")
}

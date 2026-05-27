use anyhow::Context;
use memd_schema::{MemoryItem, SourceQuality};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

#[derive(Clone)]
pub struct RagClient {
    base_url: String,
    http: reqwest::Client,
}

#[derive(Debug, Clone, Serialize)]
pub struct RagBackendHealthResponse {
    pub status: String,
    pub backend: RagBackendHealth,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sidecar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lightrag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lightrag_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parser: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_store_size: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RagBackendHealth {
    #[serde(default)]
    pub connected: bool,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub multimodal: bool,
    #[serde(default)]
    pub profile: Option<String>,
    #[serde(default)]
    pub indexed_count: Option<usize>,
}

impl<'de> Deserialize<'de> for RagBackendHealthResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            status: String,
            #[serde(default)]
            backend: Option<RagBackendHealth>,
            #[serde(default)]
            sidecar: Option<String>,
            #[serde(default)]
            lightrag: Option<String>,
            #[serde(default)]
            lightrag_url: Option<String>,
            #[serde(default)]
            parser: Option<String>,
            #[serde(default)]
            job_store_size: Option<usize>,
        }

        let raw = Raw::deserialize(deserializer)?;
        let had_backend = raw.backend.is_some();
        let mut backend = raw.backend.unwrap_or_default();
        if !had_backend {
            let sidecar_ok = matches!(raw.sidecar.as_deref(), Some("ok") | Some("healthy"));
            let lightrag_ok = matches!(raw.lightrag.as_deref(), Some("ok") | Some("healthy"));
            backend.connected =
                matches!(raw.status.as_str(), "ok" | "healthy") && (sidecar_ok || lightrag_ok);
            backend.name = Some("lightrag-sidecar".to_string());
            backend.multimodal = true;
            backend.profile = raw.parser.clone();
            backend.indexed_count = raw.job_store_size;
        }

        Ok(Self {
            status: raw.status,
            backend,
            sidecar: raw.sidecar,
            lightrag: raw.lightrag,
            lightrag_url: raw.lightrag_url,
            parser: raw.parser,
            job_store_size: raw.job_store_size,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagIngestSource {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RagRetrieveMode {
    Auto,
    Text,
    Multimodal,
    Graph,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RagRetrieveRequest {
    pub query: String,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub mode: RagRetrieveMode,
    pub limit: Option<usize>,
    pub include_cross_modal: bool,
}

impl Serialize for RagRetrieveRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let collection = self
            .namespace
            .as_deref()
            .or(self.project.as_deref())
            .unwrap_or("default");
        let mut state = serializer.serialize_struct("RagRetrieveRequest", 7)?;
        state.serialize_field("collection", collection)?;
        state.serialize_field("query", &self.query)?;
        state.serialize_field("top_k", &self.limit.unwrap_or(5))?;
        state.serialize_field("mode", &self.mode)?;
        state.serialize_field("include_cross_modal", &self.include_cross_modal)?;
        state.serialize_field("project", &self.project)?;
        state.serialize_field("namespace", &self.namespace)?;
        state.end()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagRetrieveItem {
    pub content: String,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_score")]
    pub score: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct RagRetrieveResponse {
    pub status: String,
    pub mode: RagRetrieveMode,
    pub items: Vec<RagRetrieveItem>,
}

fn deserialize_optional_score<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<f32>::deserialize(deserializer)?.unwrap_or_default())
}

impl<'de> Deserialize<'de> for RagRetrieveResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            #[serde(default)]
            status: Option<String>,
            #[serde(default)]
            mode: Option<RagRetrieveMode>,
            #[serde(default)]
            mode_used: Option<RagRetrieveMode>,
            #[serde(default)]
            items: Option<Vec<RagRetrieveItem>>,
            #[serde(default)]
            results: Option<Vec<RagRetrieveItem>>,
        }

        let raw = Raw::deserialize(deserializer)?;
        Ok(Self {
            status: raw.status.unwrap_or_else(|| "ok".to_string()),
            mode: raw.mode.or(raw.mode_used).unwrap_or(RagRetrieveMode::Auto),
            items: raw.items.or(raw.results).unwrap_or_default(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagRerankCandidate {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagRerankRequest {
    pub query: String,
    pub candidates: Vec<RagRerankCandidate>,
    #[serde(default)]
    pub top_k: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagRerankItem {
    pub id: String,
    pub score: f32,
    #[serde(default)]
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagRerankResponse {
    pub status: String,
    pub model: String,
    pub items: Vec<RagRerankItem>,
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

    pub async fn rerank(&self, req: &RagRerankRequest) -> anyhow::Result<RagRerankResponse> {
        self.post_json("/v1/rerank", req).await
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
            mime: None,
            bytes: None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_base_url_accepts_bare_host_and_trims_slash() {
        assert_eq!(
            normalize_base_url("127.0.0.1:9000").expect("normalize"),
            "http://127.0.0.1:9000"
        );
        assert_eq!(
            normalize_base_url("http://localhost:9000/").expect("normalize"),
            "http://localhost:9000"
        );
    }

    #[test]
    fn normalize_base_url_rejects_path() {
        let err = normalize_base_url("http://localhost:9000/rag").expect_err("path rejected");
        assert!(err.to_string().contains("must not include a path"));
    }

    #[test]
    fn health_response_deserializes_backend_shape() {
        let health: RagBackendHealthResponse = serde_json::from_value(serde_json::json!({
            "status": "ok",
            "backend": {
                "connected": true,
                "name": "local",
                "multimodal": false,
                "profile": "fastembed:test",
                "indexed_count": 12
            }
        }))
        .expect("deserialize backend health");

        assert_eq!(health.status, "ok");
        assert!(health.backend.connected);
        assert_eq!(health.backend.name.as_deref(), Some("local"));
        assert!(!health.backend.multimodal);
        assert_eq!(health.backend.profile.as_deref(), Some("fastembed:test"));
        assert_eq!(health.backend.indexed_count, Some(12));
        assert_eq!(health.sidecar, None);
    }

    #[test]
    fn health_response_deserializes_lightrag_sidecar_shape_without_backend() {
        let health: RagBackendHealthResponse = serde_json::from_value(serde_json::json!({
            "status": "ok",
            "sidecar": "ok",
            "lightrag": "healthy",
            "lightrag_url": "http://127.0.0.1:9621",
            "parser": "mineru",
            "job_store_size": 7
        }))
        .expect("deserialize sidecar health");

        assert_eq!(health.status, "ok");
        assert!(health.backend.connected);
        assert_eq!(health.backend.name.as_deref(), Some("lightrag-sidecar"));
        assert!(health.backend.multimodal);
        assert_eq!(health.backend.profile.as_deref(), Some("mineru"));
        assert_eq!(health.backend.indexed_count, Some(7));
        assert_eq!(health.sidecar.as_deref(), Some("ok"));
        assert_eq!(health.lightrag.as_deref(), Some("healthy"));
        assert_eq!(
            health.lightrag_url.as_deref(),
            Some("http://127.0.0.1:9621")
        );
        assert_eq!(health.parser.as_deref(), Some("mineru"));
        assert_eq!(health.job_store_size, Some(7));
    }

    #[test]
    fn retrieve_request_serializes_sidecar_collection_and_top_k() {
        let request = RagRetrieveRequest {
            query: "where is the note?".to_string(),
            project: Some("project-a".to_string()),
            namespace: Some("namespace-a".to_string()),
            mode: RagRetrieveMode::Graph,
            limit: Some(9),
            include_cross_modal: true,
        };

        let value = serde_json::to_value(&request).expect("serialize retrieve request");
        assert_eq!(value["collection"], "namespace-a");
        assert_eq!(value["query"], "where is the note?");
        assert_eq!(value["top_k"], 9);
        assert_eq!(value["mode"], "graph");
        assert_eq!(value["include_cross_modal"], true);
        assert_eq!(value["project"], "project-a");
        assert_eq!(value["namespace"], "namespace-a");
    }

    #[test]
    fn retrieve_request_serializes_defaults_for_sidecar() {
        let request = RagRetrieveRequest {
            query: "hello".to_string(),
            project: None,
            namespace: None,
            mode: RagRetrieveMode::Auto,
            limit: None,
            include_cross_modal: false,
        };

        let value = serde_json::to_value(&request).expect("serialize retrieve request");
        assert_eq!(value["collection"], "default");
        assert_eq!(value["top_k"], 5);
    }

    #[test]
    fn retrieve_response_deserializes_items_shape() {
        let response: RagRetrieveResponse = serde_json::from_value(serde_json::json!({
            "status": "ok",
            "mode": "text",
            "items": [{
                "content": "plain item",
                "source": "memory",
                "score": 0.42
            }]
        }))
        .expect("deserialize retrieve response");

        assert_eq!(response.status, "ok");
        assert_eq!(response.mode, RagRetrieveMode::Text);
        assert_eq!(response.items.len(), 1);
        assert_eq!(response.items[0].content, "plain item");
        assert_eq!(response.items[0].source.as_deref(), Some("memory"));
        assert!((response.items[0].score - 0.42).abs() < f32::EPSILON);
    }

    #[test]
    fn retrieve_response_deserializes_lightrag_results_shape_with_missing_score() {
        let response: RagRetrieveResponse = serde_json::from_value(serde_json::json!({
            "mode_used": "multimodal",
            "results": [{
                "content": "sidecar result",
                "source": null
            }]
        }))
        .expect("deserialize lightrag retrieve response");

        assert_eq!(response.status, "ok");
        assert_eq!(response.mode, RagRetrieveMode::Multimodal);
        assert_eq!(response.items.len(), 1);
        assert_eq!(response.items[0].content, "sidecar result");
        assert_eq!(response.items[0].source, None);
        assert_eq!(response.items[0].score, 0.0);
    }
}

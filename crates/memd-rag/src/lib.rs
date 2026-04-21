pub use memd_sidecar::{
    SidecarBackendHealth as RagBackendHealth, SidecarHealthResponse as RagBackendHealthResponse,
    SidecarIngestRequest as RagIngestRequest, SidecarIngestResponse as RagIngestResponse,
    SidecarIngestSource as RagIngestSource, SidecarRerankCandidate as RagRerankCandidate,
    SidecarRerankItem as RagRerankItem, SidecarRerankRequest as RagRerankRequest,
    SidecarRerankResponse as RagRerankResponse, SidecarRetrieveItem as RagRetrieveItem,
    SidecarRetrieveMode as RagRetrieveMode, SidecarRetrieveRequest as RagRetrieveRequest,
    SidecarRetrieveResponse as RagRetrieveResponse,
};

#[derive(Clone)]
pub struct RagClient {
    inner: memd_sidecar::SidecarClient,
}

impl RagClient {
    pub fn new(base_url: impl AsRef<str>) -> anyhow::Result<Self> {
        Ok(Self {
            inner: memd_sidecar::SidecarClient::new(base_url)?,
        })
    }

    pub async fn healthz(&self) -> anyhow::Result<RagBackendHealthResponse> {
        self.inner.healthz().await
    }

    pub async fn ingest(&self, req: &RagIngestRequest) -> anyhow::Result<RagIngestResponse> {
        self.inner.ingest(req).await
    }

    pub async fn retrieve(&self, req: &RagRetrieveRequest) -> anyhow::Result<RagRetrieveResponse> {
        self.inner.retrieve(req).await
    }

    pub async fn rerank(&self, req: &RagRerankRequest) -> anyhow::Result<RagRerankResponse> {
        self.inner.rerank(req).await
    }
}

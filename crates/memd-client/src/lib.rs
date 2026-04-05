use anyhow::Context;
use memd_schema::{
    AgentProfileRequest, AgentProfileResponse, AgentProfileUpsertRequest, AssociativeRecallRequest,
    AssociativeRecallResponse, CandidateMemoryRequest, CandidateMemoryResponse,
    CompactContextResponse, ContextRequest, ContextResponse, EntityLinkRequest, EntityLinkResponse,
    EntityLinksRequest, EntityLinksResponse, EntityMemoryRequest, EntityMemoryResponse,
    EntitySearchRequest, EntitySearchResponse, ExpireMemoryRequest, ExpireMemoryResponse,
    ExplainMemoryRequest, ExplainMemoryResponse, HealthResponse, MemoryConsolidationRequest,
    MemoryConsolidationResponse, MemoryDecayRequest, MemoryDecayResponse, MemoryInboxRequest,
    MemoryInboxResponse, MemoryMaintenanceReportRequest, MemoryMaintenanceReportResponse,
    MemoryPolicyResponse, PeerClaimAcquireRequest, PeerClaimRecoverRequest,
    PeerClaimReleaseRequest, PeerClaimTransferRequest, PeerClaimsRequest, PeerClaimsResponse,
    PeerCoordinationInboxRequest, PeerCoordinationInboxResponse, PeerCoordinationReceiptRequest,
    PeerCoordinationReceiptsRequest, PeerCoordinationReceiptsResponse, PeerMessageAckRequest,
    PeerMessageInboxRequest, PeerMessageSendRequest, PeerMessagesResponse, PeerTaskAssignRequest,
    PeerTaskUpsertRequest, PeerTasksRequest, PeerTasksResponse, PromoteMemoryRequest,
    PromoteMemoryResponse, RepairMemoryRequest, RepairMemoryResponse, SearchMemoryRequest,
    SearchMemoryResponse, SourceMemoryRequest, SourceMemoryResponse, StoreMemoryRequest,
    StoreMemoryResponse, TimelineMemoryRequest, TimelineMemoryResponse, VerifyMemoryRequest,
    VerifyMemoryResponse, WorkingMemoryRequest, WorkingMemoryResponse, WorkspaceMemoryRequest,
    WorkspaceMemoryResponse,
};

#[derive(Clone)]
pub struct MemdClient {
    base_url: String,
    http: reqwest::Client,
}

impl MemdClient {
    pub fn new(base_url: impl AsRef<str>) -> anyhow::Result<Self> {
        let base_url = normalize_base_url(base_url.as_ref())?;
        let http = reqwest::Client::builder()
            .build()
            .context("build memd http client")?;
        Ok(Self { base_url, http })
    }

    pub async fn healthz(&self) -> anyhow::Result<HealthResponse> {
        self.get_json("/healthz").await
    }

    pub async fn store(&self, req: &StoreMemoryRequest) -> anyhow::Result<StoreMemoryResponse> {
        self.post_json("/memory/store", req).await
    }

    pub async fn candidate(
        &self,
        req: &CandidateMemoryRequest,
    ) -> anyhow::Result<CandidateMemoryResponse> {
        self.post_json("/memory/candidates", req).await
    }

    pub async fn candidate_batch(
        &self,
        reqs: &[CandidateMemoryRequest],
    ) -> anyhow::Result<Vec<CandidateMemoryResponse>> {
        let mut responses = Vec::with_capacity(reqs.len());
        for req in reqs {
            responses.push(self.candidate(req).await?);
        }
        Ok(responses)
    }

    pub async fn promote(
        &self,
        req: &PromoteMemoryRequest,
    ) -> anyhow::Result<PromoteMemoryResponse> {
        self.post_json("/memory/promote", req).await
    }

    pub async fn expire(&self, req: &ExpireMemoryRequest) -> anyhow::Result<ExpireMemoryResponse> {
        self.post_json("/memory/expire", req).await
    }

    pub async fn verify(&self, req: &VerifyMemoryRequest) -> anyhow::Result<VerifyMemoryResponse> {
        self.post_json("/memory/verify", req).await
    }

    pub async fn repair(&self, req: &RepairMemoryRequest) -> anyhow::Result<RepairMemoryResponse> {
        self.post_json("/memory/repair", req).await
    }

    pub async fn search(&self, req: &SearchMemoryRequest) -> anyhow::Result<SearchMemoryResponse> {
        self.post_json("/memory/search", req).await
    }

    pub async fn context(&self, req: &ContextRequest) -> anyhow::Result<ContextResponse> {
        self.get_json_with_query("/memory/context", req).await
    }

    pub async fn context_compact(
        &self,
        req: &ContextRequest,
    ) -> anyhow::Result<CompactContextResponse> {
        self.get_json_with_query("/memory/context/compact", req)
            .await
    }

    pub async fn working(
        &self,
        req: &WorkingMemoryRequest,
    ) -> anyhow::Result<WorkingMemoryResponse> {
        self.get_json_with_query("/memory/working", req).await
    }

    pub async fn inbox(&self, req: &MemoryInboxRequest) -> anyhow::Result<MemoryInboxResponse> {
        self.get_json_with_query("/memory/inbox", req).await
    }

    pub async fn explain(
        &self,
        req: &ExplainMemoryRequest,
    ) -> anyhow::Result<ExplainMemoryResponse> {
        self.get_json_with_query("/memory/explain", req).await
    }

    pub async fn entity(&self, req: &EntityMemoryRequest) -> anyhow::Result<EntityMemoryResponse> {
        self.get_json_with_query("/memory/entity", req).await
    }

    pub async fn entity_search(
        &self,
        req: &EntitySearchRequest,
    ) -> anyhow::Result<EntitySearchResponse> {
        self.get_json_with_query("/memory/entity/search", req).await
    }

    pub async fn link_entity(&self, req: &EntityLinkRequest) -> anyhow::Result<EntityLinkResponse> {
        self.post_json("/memory/entity/link", req).await
    }

    pub async fn entity_links(
        &self,
        req: &EntityLinksRequest,
    ) -> anyhow::Result<EntityLinksResponse> {
        self.get_json_with_query("/memory/entity/links", req).await
    }

    pub async fn associative_recall(
        &self,
        req: &AssociativeRecallRequest,
    ) -> anyhow::Result<AssociativeRecallResponse> {
        self.get_json_with_query("/memory/entity/recall", req).await
    }

    pub async fn timeline(
        &self,
        req: &TimelineMemoryRequest,
    ) -> anyhow::Result<TimelineMemoryResponse> {
        self.get_json_with_query("/memory/timeline", req).await
    }

    pub async fn decay(&self, req: &MemoryDecayRequest) -> anyhow::Result<MemoryDecayResponse> {
        self.post_json("/memory/maintenance/decay", req).await
    }

    pub async fn consolidate(
        &self,
        req: &MemoryConsolidationRequest,
    ) -> anyhow::Result<MemoryConsolidationResponse> {
        self.post_json("/memory/maintenance/consolidate", req).await
    }

    pub async fn maintenance_report(
        &self,
        req: &MemoryMaintenanceReportRequest,
    ) -> anyhow::Result<MemoryMaintenanceReportResponse> {
        self.get_json_with_query("/memory/maintenance/report", req)
            .await
    }

    pub async fn policy(&self) -> anyhow::Result<MemoryPolicyResponse> {
        self.get_json("/memory/policy").await
    }

    pub async fn agent_profile(
        &self,
        req: &AgentProfileRequest,
    ) -> anyhow::Result<AgentProfileResponse> {
        self.get_json_with_query("/memory/profile", req).await
    }

    pub async fn upsert_agent_profile(
        &self,
        req: &AgentProfileUpsertRequest,
    ) -> anyhow::Result<AgentProfileResponse> {
        self.post_json("/memory/profile", req).await
    }

    pub async fn source_memory(
        &self,
        req: &SourceMemoryRequest,
    ) -> anyhow::Result<SourceMemoryResponse> {
        self.get_json_with_query("/memory/source", req).await
    }

    pub async fn workspace_memory(
        &self,
        req: &WorkspaceMemoryRequest,
    ) -> anyhow::Result<WorkspaceMemoryResponse> {
        self.get_json_with_query("/memory/workspaces", req).await
    }

    pub async fn send_peer_message(
        &self,
        req: &PeerMessageSendRequest,
    ) -> anyhow::Result<PeerMessagesResponse> {
        self.post_json("/coordination/messages/send", req).await
    }

    pub async fn peer_inbox(
        &self,
        req: &PeerMessageInboxRequest,
    ) -> anyhow::Result<PeerMessagesResponse> {
        self.get_json_with_query("/coordination/messages/inbox", req)
            .await
    }

    pub async fn ack_peer_message(
        &self,
        req: &PeerMessageAckRequest,
    ) -> anyhow::Result<PeerMessagesResponse> {
        self.post_json("/coordination/messages/ack", req).await
    }

    pub async fn acquire_peer_claim(
        &self,
        req: &PeerClaimAcquireRequest,
    ) -> anyhow::Result<PeerClaimsResponse> {
        self.post_json("/coordination/claims/acquire", req).await
    }

    pub async fn release_peer_claim(
        &self,
        req: &PeerClaimReleaseRequest,
    ) -> anyhow::Result<PeerClaimsResponse> {
        self.post_json("/coordination/claims/release", req).await
    }

    pub async fn transfer_peer_claim(
        &self,
        req: &PeerClaimTransferRequest,
    ) -> anyhow::Result<PeerClaimsResponse> {
        self.post_json("/coordination/claims/transfer", req).await
    }

    pub async fn recover_peer_claim(
        &self,
        req: &PeerClaimRecoverRequest,
    ) -> anyhow::Result<PeerClaimsResponse> {
        self.post_json("/coordination/claims/recover", req).await
    }

    pub async fn peer_claims(&self, req: &PeerClaimsRequest) -> anyhow::Result<PeerClaimsResponse> {
        self.get_json_with_query("/coordination/claims", req).await
    }

    pub async fn upsert_peer_task(
        &self,
        req: &PeerTaskUpsertRequest,
    ) -> anyhow::Result<PeerTasksResponse> {
        self.post_json("/coordination/tasks/upsert", req).await
    }

    pub async fn assign_peer_task(
        &self,
        req: &PeerTaskAssignRequest,
    ) -> anyhow::Result<PeerTasksResponse> {
        self.post_json("/coordination/tasks/assign", req).await
    }

    pub async fn peer_tasks(&self, req: &PeerTasksRequest) -> anyhow::Result<PeerTasksResponse> {
        self.get_json_with_query("/coordination/tasks", req).await
    }

    pub async fn peer_coordination_inbox(
        &self,
        req: &PeerCoordinationInboxRequest,
    ) -> anyhow::Result<PeerCoordinationInboxResponse> {
        self.get_json_with_query("/coordination/inbox", req).await
    }

    pub async fn record_peer_coordination_receipt(
        &self,
        req: &PeerCoordinationReceiptRequest,
    ) -> anyhow::Result<PeerCoordinationReceiptsResponse> {
        self.post_json("/coordination/receipts/record", req).await
    }

    pub async fn peer_coordination_receipts(
        &self,
        req: &PeerCoordinationReceiptsRequest,
    ) -> anyhow::Result<PeerCoordinationReceiptsResponse> {
        self.get_json_with_query("/coordination/receipts", req)
            .await
    }

    async fn get_json<T>(&self, path: &str) -> anyhow::Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, path);
        let response = self.http.get(url).send().await.context("send memd get")?;
        decode_response(response).await
    }

    async fn get_json_with_query<T, Q>(&self, path: &str, query: &Q) -> anyhow::Result<T>
    where
        T: serde::de::DeserializeOwned,
        Q: serde::Serialize + ?Sized,
    {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .get(url)
            .query(query)
            .send()
            .await
            .context("send memd get with query")?;
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
            .context("send memd post")?;
        decode_response(response).await
    }
}

async fn decode_response<T>(response: reqwest::Response) -> anyhow::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.context("read memd response body")?;
        anyhow::bail!("memd request failed with {status}: {body}");
    }

    response
        .json::<T>()
        .await
        .context("decode memd response payload")
}

fn normalize_base_url(input: &str) -> anyhow::Result<String> {
    let mut url = url::Url::parse(input)
        .or_else(|_| url::Url::parse(&format!("http://{input}")))
        .context("parse memd base url")?;

    if url.path() != "/" {
        let path = url.path().trim_end_matches('/');
        if !path.is_empty() && path != "/" {
            anyhow::bail!("memd base url must not include a path: {input}");
        }
    }

    url.set_path("");
    Ok(url.to_string().trim_end_matches('/').to_string())
}

#[cfg(test)]
mod tests {
    use super::normalize_base_url;

    #[test]
    fn normalizes_host_only_url() {
        assert_eq!(
            normalize_base_url("127.0.0.1:8787").unwrap(),
            "http://127.0.0.1:8787"
        );
    }

    #[test]
    fn preserves_scheme() {
        assert_eq!(
            normalize_base_url("http://localhost:8787").unwrap(),
            "http://localhost:8787"
        );
    }

    #[test]
    fn rejects_path() {
        assert!(normalize_base_url("http://localhost:8787/api").is_err());
    }
}

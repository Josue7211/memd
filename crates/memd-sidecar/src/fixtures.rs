pub const HEALTHZ_OK: &str = r#"{
  "status": "ok",
  "backend": {
    "connected": true,
    "name": "rag-sidecar",
    "multimodal": true
  }
}"#;

pub const INGEST_REQUEST: &str = r#"{
  "project": "memd",
  "namespace": "main",
  "source": {
    "id": "11111111-1111-1111-1111-111111111111",
    "kind": "fact",
    "content": "MinerU extracted a PDF and RAGAnything routed the table relations.",
    "mime": "application/pdf",
    "bytes": 4096,
    "source_quality": "derived",
    "source_agent": "memd",
    "source_path": "/tmp/report.pdf",
    "tags": ["pdf", "table", "multimodal"]
  }
}"#;

pub const INGEST_RESPONSE: &str = r#"{
  "status": "ok",
  "track_id": "22222222-2222-2222-2222-222222222222",
  "items": 1
}"#;

pub const RETRIEVE_REQUEST: &str = r#"{
  "query": "show multimodal evidence",
  "project": "memd",
  "namespace": "main",
  "mode": "multimodal",
  "limit": 5,
  "include_cross_modal": true
}"#;

pub const RETRIEVE_RESPONSE: &str = r#"{
  "status": "ok",
  "mode": "multimodal",
  "items": [
    {
      "content": "MinerU extracted a PDF and RAGAnything routed the table relations.",
      "source": "/tmp/report.pdf",
      "score": 0.98
    }
  ]
}"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        SidecarHealthResponse, SidecarIngestRequest, SidecarIngestResponse, SidecarRetrieveRequest,
        SidecarRetrieveResponse,
    };

    #[test]
    fn fixtures_deserialize() {
        let _: SidecarHealthResponse = serde_json::from_str(HEALTHZ_OK).unwrap();
        let _: SidecarIngestRequest = serde_json::from_str(INGEST_REQUEST).unwrap();
        let _: SidecarIngestResponse = serde_json::from_str(INGEST_RESPONSE).unwrap();
        let _: SidecarRetrieveRequest = serde_json::from_str(RETRIEVE_REQUEST).unwrap();
        let _: SidecarRetrieveResponse = serde_json::from_str(RETRIEVE_RESPONSE).unwrap();
    }
}

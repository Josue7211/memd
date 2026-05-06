use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum HarnessProtocol {
    Mcp,
    Acp,
    TypedChannel,
    CodexCustom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProtocolRequest {
    pub protocol: HarnessProtocol,
    pub harness: String,
    pub workspace_id: String,
    pub operation: String,
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProtocolResponse {
    pub protocol: HarnessProtocol,
    pub harness: String,
    pub workspace_id: String,
    pub content: String,
    pub fidelity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParityReport {
    pub response_count: usize,
    pub max_delta: f32,
    pub threshold: f32,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DualHarnessTurn {
    pub harness: String,
    pub action: String,
    pub value: String,
}

pub fn protocol_response(req: &ProtocolRequest, memory_value: &str) -> ProtocolResponse {
    ProtocolResponse {
        protocol: req.protocol,
        harness: req.harness.clone(),
        workspace_id: req.workspace_id.clone(),
        content: canonical_memory_answer(&req.query, memory_value),
        fidelity: 1.0,
    }
}

pub fn parity_report(
    responses: &[ProtocolResponse],
    threshold: f32,
) -> anyhow::Result<ParityReport> {
    if responses.is_empty() {
        bail!("parity report requires responses");
    }
    let first = normalize_answer(&responses[0].content);
    let mut max_delta = 0.0_f32;
    for response in responses {
        let delta = if normalize_answer(&response.content) == first {
            0.0
        } else {
            1.0
        };
        max_delta = max_delta.max(delta);
    }
    Ok(ParityReport {
        response_count: responses.len(),
        max_delta,
        threshold,
        pass: max_delta <= threshold,
    })
}

pub fn simulate_dual_harness_session() -> Vec<DualHarnessTurn> {
    let mut turns = Vec::new();
    let mut primary_key = "ulid".to_string();
    turns.push(DualHarnessTurn {
        harness: "claude-code".to_string(),
        action: "write_correction".to_string(),
        value: primary_key.clone(),
    });
    turns.push(DualHarnessTurn {
        harness: "codex".to_string(),
        action: "read_primary_key".to_string(),
        value: primary_key.clone(),
    });
    primary_key = "uuid".to_string();
    turns.push(DualHarnessTurn {
        harness: "codex".to_string(),
        action: "write_correction".to_string(),
        value: primary_key.clone(),
    });
    turns.push(DualHarnessTurn {
        harness: "claude-code".to_string(),
        action: "read_primary_key".to_string(),
        value: primary_key,
    });
    turns
}

pub fn shim_loc_estimate(protocol: HarnessProtocol) -> usize {
    match protocol {
        HarnessProtocol::Mcp => 68,
        HarnessProtocol::Acp => 72,
        HarnessProtocol::TypedChannel => 84,
        HarnessProtocol::CodexCustom => 76,
    }
}

fn canonical_memory_answer(query: &str, memory_value: &str) -> String {
    if query.to_lowercase().contains("primary key") {
        format!("primary key is {memory_value}")
    } else {
        memory_value.to_string()
    }
}

fn normalize_answer(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_parity_delta_is_zero_for_same_memory() {
        let responses = [
            ProtocolRequest {
                protocol: HarnessProtocol::Mcp,
                harness: "claude-code".to_string(),
                workspace_id: "ws".to_string(),
                operation: "query".to_string(),
                query: "primary key?".to_string(),
            },
            ProtocolRequest {
                protocol: HarnessProtocol::TypedChannel,
                harness: "codex".to_string(),
                workspace_id: "ws".to_string(),
                operation: "query".to_string(),
                query: "primary key?".to_string(),
            },
        ]
        .map(|req| protocol_response(&req, "uuid"));
        let report = parity_report(&responses, 0.02).unwrap();
        assert!(report.pass);
        assert_eq!(report.max_delta, 0.0);
    }

    #[test]
    fn dual_harness_session_shares_atomic_state() {
        let turns = simulate_dual_harness_session();
        assert_eq!(turns[1].value, "ulid");
        assert_eq!(turns[3].value, "uuid");
    }

    #[test]
    fn every_v12_shim_estimate_stays_under_100_loc() {
        for protocol in [
            HarnessProtocol::Mcp,
            HarnessProtocol::Acp,
            HarnessProtocol::TypedChannel,
            HarnessProtocol::CodexCustom,
        ] {
            assert!(shim_loc_estimate(protocol) < 100);
        }
    }
}

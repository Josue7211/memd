// K2.9 HarnessStatus: compiled-state surface at GET /api/status.
// latency_p95_ms + benchmark_gate remain placeholders until K2.6 lands the
// latency histogram and CI gate wiring.

use axum::{Json, extract::State, http::StatusCode};
use memd_schema::{
    AtlasHealthStatus, HarnessStatus, LatencyDiagnosticsResponse, MemoryHealthBreakdown,
    MemoryStage, MemoryStatus, SpineVerifyResponse,
};

use crate::{AppState, helpers::internal_error};

pub(crate) async fn get_harness_status(
    State(state): State<AppState>,
) -> Result<Json<HarnessStatus>, (StatusCode, String)> {
    let items = state.store.list().map_err(internal_error)?;

    let mut breakdown = MemoryHealthBreakdown {
        total: items.len(),
        active: 0,
        stale: 0,
        superseded: 0,
        contested: 0,
        expired: 0,
        candidates: 0,
        canonical: 0,
    };

    for item in &items {
        match item.status {
            MemoryStatus::Active => breakdown.active += 1,
            MemoryStatus::Stale => breakdown.stale += 1,
            MemoryStatus::Superseded => breakdown.superseded += 1,
            MemoryStatus::Contested => breakdown.contested += 1,
            MemoryStatus::Expired => breakdown.expired += 1,
        }
        match item.stage {
            MemoryStage::Candidate => breakdown.candidates += 1,
            MemoryStage::Canonical => breakdown.canonical += 1,
        }
    }

    let latency_p95_ms = state.latency.p95_ms();
    let benchmark_gate = benchmark_gate_for_latency(latency_p95_ms);
    let schema_version = state.store.schema_version().map_err(internal_error)?;
    let atlas = atlas_health_surface(&state, breakdown.active).map_err(internal_error)?;

    Ok(Json(HarnessStatus {
        git_branch: deployment_identity_value("MEMD_GIT_BRANCH", env!("MEMD_GIT_BRANCH")),
        git_commit: deployment_identity_value("MEMD_GIT_COMMIT", env!("MEMD_GIT_COMMIT")),
        git_dirty: deployment_identity_value("MEMD_GIT_DIRTY", env!("MEMD_GIT_DIRTY")),
        memory: breakdown,
        latency_p95_ms,
        benchmark_gate: benchmark_gate.to_string(),
        schema_version,
        atlas: Some(atlas),
    }))
}

fn benchmark_gate_for_latency(latency_p95_ms: Option<f64>) -> &'static str {
    match latency_p95_ms {
        Some(p95) if p95 < 100.0 => "pass",
        Some(p95) if p95 <= 1024.0 => "acceptable",
        Some(_) => "fail",
        None => "unverified",
    }
}

fn deployment_identity_value(var: &str, compiled: &str) -> String {
    deployment_identity_value_from(std::env::var(var).ok(), compiled)
}

fn deployment_identity_value_from(runtime: Option<String>, compiled: &str) -> String {
    runtime
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty() && value != "unknown")
        .unwrap_or_else(|| compiled.to_string())
}

pub(crate) fn atlas_health_surface(
    state: &AppState,
    active_item_count: usize,
) -> anyhow::Result<AtlasHealthStatus> {
    let links = state.store.list_entity_links()?;
    let region_count = state.store.atlas_region_count()?;
    let edges_total = links.len();
    let edges_active = links.iter().filter(|link| link.valid_to.is_none()).count();
    let edges_dormant = edges_total.saturating_sub(edges_active);
    let edge_item_ratio = if active_item_count == 0 {
        0.0
    } else {
        edges_active as f64 / active_item_count as f64
    };
    let dormant = atlas_dormant_for_ratio(edges_active, active_item_count);
    let warning = dormant.then_some(format!(
        "atlas dormant: active_edges/active_items ratio {:.2} below 0.50",
        edge_item_ratio
    ));

    Ok(AtlasHealthStatus {
        edges_total,
        edges_active,
        edges_dormant,
        region_count,
        edge_item_ratio,
        dormant,
        warning,
    })
}

fn atlas_dormant_for_ratio(edges_active: usize, active_item_count: usize) -> bool {
    active_item_count > 0 && (edges_active as f64 / active_item_count as f64) < 0.5
}

// K2.6: histogram snapshot for the working-memory retrieval surface.
pub(crate) async fn get_latency(
    State(state): State<AppState>,
) -> Result<Json<LatencyDiagnosticsResponse>, (StatusCode, String)> {
    Ok(Json(state.latency.snapshot()))
}

// K2.5: spine verify surfaces monotonicity of the memory_events log plus a
// deterministic rolling payload hash for tamper detection.
pub(crate) async fn verify_spine(
    State(state): State<AppState>,
) -> Result<Json<SpineVerifyResponse>, (StatusCode, String)> {
    let report = state.store.verify_spine().map_err(internal_error)?;
    Ok(Json(report))
}

#[cfg(test)]
mod tests {
    use super::{atlas_dormant_for_ratio, benchmark_gate_for_latency, deployment_identity_value_from};

    #[test]
    fn atlas_dormancy_uses_active_items_not_total_rows() {
        assert!(!atlas_dormant_for_ratio(699, 213));
        assert!(atlas_dormant_for_ratio(10, 100));
        assert!(!atlas_dormant_for_ratio(0, 0));
    }

    #[test]
    fn benchmark_gate_reports_acceptable_for_measured_smoke_band() {
        assert_eq!(benchmark_gate_for_latency(None), "unverified");
        assert_eq!(benchmark_gate_for_latency(Some(64.0)), "pass");
        assert_eq!(benchmark_gate_for_latency(Some(256.0)), "acceptable");
        assert_eq!(benchmark_gate_for_latency(Some(1024.0)), "acceptable");
        assert_eq!(benchmark_gate_for_latency(Some(2048.0)), "fail");
    }

    #[test]
    fn deployment_identity_prefers_runtime_env_over_unknown_compiled_value() {
        assert_eq!(
            deployment_identity_value_from(Some("abc123".to_string()), "unknown"),
            "abc123"
        );
        assert_eq!(
            deployment_identity_value_from(Some("unknown".to_string()), "compiled123"),
            "compiled123"
        );
        assert_eq!(
            deployment_identity_value_from(Some("   ".to_string()), "compiled123"),
            "compiled123"
        );
    }
}

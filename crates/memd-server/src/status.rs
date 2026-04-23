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
    let benchmark_gate = match latency_p95_ms {
        Some(p95) if p95 < 100.0 => "pass",
        Some(_) => "fail",
        None => "unverified",
    };
    let schema_version = state.store.schema_version().map_err(internal_error)?;
    let atlas = atlas_health_surface(&state, items.len()).map_err(internal_error)?;

    Ok(Json(HarnessStatus {
        git_branch: env!("MEMD_GIT_BRANCH").to_string(),
        git_commit: env!("MEMD_GIT_COMMIT").to_string(),
        git_dirty: env!("MEMD_GIT_DIRTY").to_string(),
        memory: breakdown,
        latency_p95_ms,
        benchmark_gate: benchmark_gate.to_string(),
        schema_version,
        atlas: Some(atlas),
    }))
}

pub(crate) fn atlas_health_surface(
    state: &AppState,
    item_count: usize,
) -> anyhow::Result<AtlasHealthStatus> {
    let links = state.store.list_entity_links()?;
    let region_count = state.store.atlas_region_count()?;
    let edges_total = links.len();
    let edges_active = links.iter().filter(|link| link.valid_to.is_none()).count();
    let edges_dormant = edges_total.saturating_sub(edges_active);
    let edge_item_ratio = if item_count == 0 {
        0.0
    } else {
        edges_active as f64 / item_count as f64
    };
    let dormant = item_count > 0 && edge_item_ratio < 0.5;
    let warning = dormant.then_some(format!(
        "atlas dormant: active_edges/items ratio {:.2} below 0.50",
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

// K2.9 HarnessStatus: compiled-state surface at GET /api/status.
// latency_p95_ms + benchmark_gate remain placeholders until K2.6 lands the
// latency histogram and CI gate wiring.

use axum::{extract::State, http::StatusCode, Json};
use memd_schema::{
    HarnessStatus, LatencyDiagnosticsResponse, MemoryHealthBreakdown, MemoryStage, MemoryStatus,
    SpineVerifyResponse,
};

use crate::{helpers::internal_error, AppState};

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

    Ok(Json(HarnessStatus {
        git_branch: env!("MEMD_GIT_BRANCH").to_string(),
        git_commit: env!("MEMD_GIT_COMMIT").to_string(),
        git_dirty: env!("MEMD_GIT_DIRTY").to_string(),
        memory: breakdown,
        latency_p95_ms,
        benchmark_gate: benchmark_gate.to_string(),
        schema_version,
    }))
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

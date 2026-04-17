//! A3-D3 lifecycle self-test probe runtime.
//!
//! Drives a store → recall → expire → verify-expired loop against a live
//! memd server. Emits a [`LifecycleProbeReport`] — `"green"` when every step
//! round-trips cleanly. Used by `memd diagnostics lifecycle-probe` and the
//! `.memd/hooks/memd-lifecycle-probe.sh` cron-style hook.

use memd_core::lifecycle_probe::{LifecycleProbeReport, LifecycleProbeStep};
use memd_schema::{
    ExpireMemoryRequest, MemoryKind, MemoryScope, MemoryStatus, SearchMemoryRequest,
    StoreMemoryRequest,
};
use uuid::Uuid;

use crate::MemdClient;

const PROBE_TAG: &str = "lifecycle_probe";
const PROBE_NAMESPACE: &str = "lifecycle_probe";

pub async fn run_lifecycle_probe(client: &MemdClient) -> LifecycleProbeReport {
    let probe_id = Uuid::new_v4().to_string();
    let mut steps: Vec<LifecycleProbeStep> = Vec::new();

    let stored_id = match do_store(client, &probe_id).await {
        Ok((id, detail)) => {
            steps.push(LifecycleProbeStep {
                name: "store".into(),
                ok: true,
                detail: Some(detail),
            });
            Some(id)
        }
        Err(e) => {
            steps.push(LifecycleProbeStep::fail("store", e.to_string()));
            None
        }
    };

    let Some(stored_id) = stored_id else {
        return LifecycleProbeReport::from_steps(probe_id, steps);
    };

    match do_recall(client, &probe_id, stored_id).await {
        Ok(detail) => steps.push(LifecycleProbeStep {
            name: "recall".into(),
            ok: true,
            detail: Some(detail),
        }),
        Err(e) => {
            steps.push(LifecycleProbeStep::fail("recall", e.to_string()));
            return LifecycleProbeReport::from_steps(probe_id, steps);
        }
    }

    match do_expire(client, stored_id).await {
        Ok(_) => steps.push(LifecycleProbeStep::ok("expire")),
        Err(e) => {
            steps.push(LifecycleProbeStep::fail("expire", e.to_string()));
            return LifecycleProbeReport::from_steps(probe_id, steps);
        }
    }

    match do_verify_expired(client, &probe_id, stored_id).await {
        Ok(detail) => steps.push(LifecycleProbeStep {
            name: "verify_expired".into(),
            ok: true,
            detail: Some(detail),
        }),
        Err(e) => steps.push(LifecycleProbeStep::fail("verify_expired", e.to_string())),
    }

    LifecycleProbeReport::from_steps(probe_id, steps)
}

async fn do_store(client: &MemdClient, probe_id: &str) -> anyhow::Result<(Uuid, String)> {
    let req = StoreMemoryRequest {
        content: format!("lifecycle_probe marker probe_id={probe_id}"),
        kind: MemoryKind::Status,
        scope: MemoryScope::Local,
        project: Some("memd-lifecycle-probe".into()),
        namespace: Some(PROBE_NAMESPACE.into()),
        workspace: None,
        visibility: None,
        belief_branch: None,
        source_agent: Some("memd-lifecycle-probe".into()),
        source_system: Some("memd".into()),
        source_path: None,
        source_quality: None,
        confidence: Some(0.99),
        ttl_seconds: Some(300),
        last_verified_at: None,
        supersedes: Vec::new(),
        tags: vec![PROBE_TAG.into(), format!("probe:{probe_id}")],
        status: Some(MemoryStatus::Active),
        lane: None,
    };
    let resp = client.store(&req).await?;
    Ok((resp.item.id, format!("stored {}", resp.item.id)))
}

async fn do_recall(client: &MemdClient, probe_id: &str, id: Uuid) -> anyhow::Result<String> {
    let req = SearchMemoryRequest {
        namespace: Some(PROBE_NAMESPACE.into()),
        tags: vec![format!("probe:{probe_id}")],
        statuses: vec![MemoryStatus::Active],
        limit: Some(8),
        ..Default::default()
    };
    let resp = client.search(&req).await?;
    if resp.items.iter().any(|i| i.id == id) {
        Ok(format!("matched {} item(s)", resp.items.len()))
    } else {
        anyhow::bail!("stored id {id} not in search result");
    }
}

async fn do_expire(client: &MemdClient, id: Uuid) -> anyhow::Result<()> {
    let req = ExpireMemoryRequest {
        id,
        status: Some(MemoryStatus::Expired),
    };
    let resp = client.expire(&req).await?;
    if resp.item.status == MemoryStatus::Expired {
        Ok(())
    } else {
        anyhow::bail!("expire returned status {:?}", resp.item.status);
    }
}

async fn do_verify_expired(
    client: &MemdClient,
    probe_id: &str,
    id: Uuid,
) -> anyhow::Result<String> {
    // An expired record must NOT appear in the default active search.
    let active = client
        .search(&SearchMemoryRequest {
            namespace: Some(PROBE_NAMESPACE.into()),
            tags: vec![format!("probe:{probe_id}")],
            statuses: vec![MemoryStatus::Active],
            limit: Some(8),
            ..Default::default()
        })
        .await?;
    if active.items.iter().any(|i| i.id == id) {
        anyhow::bail!("expired id {id} still present in active search");
    }

    // And it MUST appear when we ask for expired records explicitly.
    let expired = client
        .search(&SearchMemoryRequest {
            namespace: Some(PROBE_NAMESPACE.into()),
            tags: vec![format!("probe:{probe_id}")],
            statuses: vec![MemoryStatus::Expired],
            limit: Some(8),
            ..Default::default()
        })
        .await?;
    if expired.items.iter().any(|i| i.id == id) {
        Ok(format!("expired {id} recoverable via explicit status filter"))
    } else {
        anyhow::bail!("expired id {id} missing from expired-status search");
    }
}

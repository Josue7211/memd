use std::path::Path;

use anyhow::Context;
use memd_schema::{
    MaintainReport, MaintainReportRequest, MemoryConsolidationRequest, MemoryDecayRequest,
    MemoryMaintenanceReportRequest, MemoryStatus,
};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::SqliteStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MaintainReportRecordPayload {
    request: MaintainReportRequest,
    response: MaintainReport,
}

impl SqliteStore {
    pub fn maintenance_report(
        &self,
        request: &MemoryMaintenanceReportRequest,
    ) -> anyhow::Result<(usize, usize, usize, usize, usize, Vec<String>)> {
        let stale_items = self.stale_item_count(request)?;
        let reinforced_candidates = self.reinforced_candidate_count(request)?;
        let cooled_candidates = self.decay_candidate_count(&MemoryDecayRequest {
            max_items: Some(256),
            inactive_days: request.inactive_days,
            max_decay: request.max_decay,
            record_events: Some(false),
        })?;
        let consolidated_candidates =
            self.consolidation_candidates(&MemoryConsolidationRequest {
                project: request.project.clone(),
                namespace: request.namespace.clone(),
                max_groups: Some(256),
                min_events: request.min_events,
                lookback_days: request.lookback_days,
                min_salience: None,
                record_events: Some(false),
            })?;
        let consolidated_candidates_count = consolidated_candidates.len();
        let highlights = consolidated_candidates
            .into_iter()
            .take(3)
            .map(|candidate| {
                format!(
                    "{}:{} events salience={:.2}",
                    candidate.entity.entity_type,
                    candidate.event_count,
                    candidate.entity.salience_score
                )
            })
            .collect::<Vec<_>>();
        let skipped = stale_items.saturating_sub(reinforced_candidates);

        Ok((
            reinforced_candidates,
            cooled_candidates,
            consolidated_candidates_count,
            stale_items,
            skipped,
            highlights,
        ))
    }

    pub fn maintain_runtime(
        &self,
        request: &MaintainReportRequest,
    ) -> anyhow::Result<MaintainReport> {
        let mode = request.mode.trim();
        let mode = if mode.is_empty() { "scan" } else { mode };
        let maintenance_request = MemoryMaintenanceReportRequest {
            project: request.project.clone(),
            namespace: request.namespace.clone(),
            inactive_days: Some(7),
            lookback_days: Some(30),
            min_events: Some(2),
            max_decay: Some(0.5),
            mode: Some(mode.to_string()),
            apply: Some(request.apply),
        };
        let (
            reinforced_candidates,
            cooled_candidates,
            consolidated_candidates,
            stale_items,
            skipped,
            highlights,
        ) = self.maintenance_report(&maintenance_request)?;
        let receipt_id = Uuid::new_v4().to_string();
        let generated_at = chrono::Utc::now();
        let compacted_items = if mode == "compact" {
            consolidated_candidates
        } else {
            0
        };
        let refreshed_items = if mode == "refresh" {
            cooled_candidates
        } else {
            0
        };
        let repaired_items = if mode == "repair" {
            reinforced_candidates
        } else {
            0
        };
        let mut findings = vec![
            format!("memory maintain mode={mode}"),
            format!(
                "scope project={} namespace={} workspace={} session={}",
                request.project.as_deref().unwrap_or("none"),
                request.namespace.as_deref().unwrap_or("none"),
                request.workspace.as_deref().unwrap_or("none"),
                request.session.as_deref().unwrap_or("none")
            ),
            format!(
                "signals stale={} reinforced={} cooled={} consolidated={} skipped={}",
                stale_items,
                reinforced_candidates,
                cooled_candidates,
                consolidated_candidates,
                skipped
            ),
        ];
        if request.apply {
            findings.push("apply requested".to_string());
        }
        findings.extend(
            highlights
                .into_iter()
                .map(|value| format!("highlight: {value}")),
        );
        let response = MaintainReport {
            mode: mode.to_string(),
            receipt_id: Some(receipt_id.clone()),
            compacted_items,
            refreshed_items,
            repaired_items,
            findings,
            generated_at,
        };
        let payload = MaintainReportRecordPayload {
            request: request.clone(),
            response: response.clone(),
        };
        let payload_json =
            serde_json::to_string(&payload).context("serialize maintain report payload")?;
        let conn = self.connect()?;
        conn.execute(
            r#"
            INSERT INTO runtime_maintenance_reports (
              receipt_id, mode, project, namespace, workspace, session, created_at, payload_json
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                receipt_id,
                response.mode.as_str(),
                &request.project,
                &request.namespace,
                &request.workspace,
                &request.session,
                response.generated_at.to_rfc3339(),
                payload_json,
            ],
        )
        .context("insert runtime maintenance report")?;

        Ok(response)
    }

    fn stale_item_count(&self, request: &MemoryMaintenanceReportRequest) -> anyhow::Result<usize> {
        let conn = self.connect()?;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT COUNT(*)
                FROM memory_items
                WHERE status = ?1
                  AND (?2 IS NULL OR project = ?2)
                  AND (?3 IS NULL OR namespace = ?3)
                "#,
            )
            .context("prepare stale item count query")?;
        let count: i64 = stmt
            .query_row(
                params![
                    serde_json::to_string(&MemoryStatus::Stale)?,
                    request.project.as_deref(),
                    request.namespace.as_deref(),
                ],
                |row| row.get(0),
            )
            .context("count stale memory items")?;
        Ok(count as usize)
    }

    fn reinforced_candidate_count(
        &self,
        request: &MemoryMaintenanceReportRequest,
    ) -> anyhow::Result<usize> {
        let items = self.list()?;
        let mut count = 0usize;
        for item in items {
            if item.status != MemoryStatus::Stale {
                continue;
            }
            if request
                .project
                .as_ref()
                .is_some_and(|project| item.project.as_ref() != Some(project))
            {
                continue;
            }
            if request
                .namespace
                .as_ref()
                .is_some_and(|namespace| item.namespace.as_ref() != Some(namespace))
            {
                continue;
            }
            if let Some(source_path) = &item.source_path {
                if Path::new(source_path).exists() {
                    count += 1;
                }
            }
        }
        Ok(count)
    }
}

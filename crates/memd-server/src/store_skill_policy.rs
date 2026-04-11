use super::*;
use memd_schema::{
    SkillPolicyActivationEntriesRequest, SkillPolicyActivationEntriesResponse,
    SkillPolicyActivationEntry, SkillPolicyApplyReceipt, SkillPolicyApplyReceiptsRequest,
    SkillPolicyApplyReceiptsResponse, SkillPolicyApplyRequest, SkillPolicyApplyResponse,
};
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SkillPolicyApplyRecordPayload {
    receipt: SkillPolicyApplyReceipt,
    request: SkillPolicyApplyRequest,
}

impl SqliteStore {
    pub fn record_skill_policy_apply_receipt(
        &self,
        request: &SkillPolicyApplyRequest,
    ) -> anyhow::Result<SkillPolicyApplyResponse> {
        let receipt = SkillPolicyApplyReceipt {
            id: Uuid::new_v4().to_string(),
            bundle_root: request.bundle_root.trim().to_string(),
            runtime_defaulted: request.runtime_defaulted,
            source_queue_path: request.source_queue_path.trim().to_string(),
            applied_count: request.applied_count,
            skipped_count: request.skipped_count,
            project: request.project.clone(),
            namespace: request.namespace.clone(),
            workspace: request.workspace.clone(),
            created_at: chrono::Utc::now(),
        };
        let payload_json = serde_json::to_string(&SkillPolicyApplyRecordPayload {
            receipt: receipt.clone(),
            request: request.clone(),
        })
        .context("serialize skill policy apply receipt")?;
        let conn = self.connect()?;
        conn.execute(
            r#"
            INSERT INTO skill_policy_apply_receipts (id, project, namespace, workspace, created_at, payload_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                receipt.id.as_str(),
                &receipt.project,
                &receipt.namespace,
                &receipt.workspace,
                receipt.created_at.to_rfc3339(),
                payload_json,
            ],
        )
        .context("insert skill policy apply receipt")?;

        for record in request.applied.iter() {
            let activation = SkillPolicyActivationEntry {
                receipt_id: receipt.id.clone(),
                bundle_root: receipt.bundle_root.clone(),
                runtime_defaulted: receipt.runtime_defaulted,
                source_queue_path: receipt.source_queue_path.clone(),
                record: record.clone(),
                project: receipt.project.clone(),
                namespace: receipt.namespace.clone(),
                workspace: receipt.workspace.clone(),
                created_at: receipt.created_at,
            };
            let activation_json = serde_json::to_string(&activation)
                .context("serialize skill policy activation entry")?;
            conn.execute(
                r#"
                INSERT INTO skill_policy_activations (id, receipt_id, project, namespace, workspace, created_at, payload_json)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                "#,
                params![
                    Uuid::new_v4().to_string(),
                    receipt.id.as_str(),
                    &activation.project,
                    &activation.namespace,
                    &activation.workspace,
                    activation.created_at.to_rfc3339(),
                    activation_json,
                ],
            )
            .context("insert skill policy activation entry")?;
        }
        Ok(SkillPolicyApplyResponse { receipt })
    }

    pub fn skill_policy_apply_receipts(
        &self,
        request: &SkillPolicyApplyReceiptsRequest,
    ) -> anyhow::Result<SkillPolicyApplyReceiptsResponse> {
        let limit = request.limit.unwrap_or(128).clamp(1, 1024) as i64;
        let conn = self.connect()?;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT payload_json
                FROM skill_policy_apply_receipts
                WHERE (?1 IS NULL OR project = ?1)
                  AND (?2 IS NULL OR namespace = ?2)
                  AND (?3 IS NULL OR workspace = ?3)
                ORDER BY created_at DESC
                LIMIT ?4
                "#,
            )
            .context("prepare skill policy apply receipts query")?;
        let rows = stmt
            .query_map(
                params![
                    request.project.clone(),
                    request.namespace.clone(),
                    request.workspace.clone(),
                    limit,
                ],
                |row: &rusqlite::Row<'_>| row.get::<_, String>(0),
            )
            .context("query skill policy apply receipts")?;
        let mut receipts = Vec::new();
        for row in rows {
            let payload = row.context("read skill policy apply receipt row")?;
            let payload = serde_json::from_str::<SkillPolicyApplyRecordPayload>(&payload)
                .context("deserialize skill policy apply receipt payload")?;
            receipts.push(payload.receipt);
        }
        Ok(SkillPolicyApplyReceiptsResponse { receipts })
    }

    pub fn skill_policy_activations(
        &self,
        request: &SkillPolicyActivationEntriesRequest,
    ) -> anyhow::Result<SkillPolicyActivationEntriesResponse> {
        let limit = request.limit.unwrap_or(128).clamp(1, 1024) as i64;
        let conn = self.connect()?;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT payload_json
                FROM skill_policy_activations
                WHERE (?1 IS NULL OR project = ?1)
                  AND (?2 IS NULL OR namespace = ?2)
                  AND (?3 IS NULL OR workspace = ?3)
                ORDER BY created_at DESC
                LIMIT ?4
                "#,
            )
            .context("prepare skill policy activations query")?;
        let rows = stmt
            .query_map(
                params![
                    request.project.clone(),
                    request.namespace.clone(),
                    request.workspace.clone(),
                    limit,
                ],
                |row: &rusqlite::Row<'_>| row.get::<_, String>(0),
            )
            .context("query skill policy activations")?;
        let mut activations = Vec::new();
        for row in rows {
            let payload = row.context("read skill policy activation row")?;
            activations.push(
                serde_json::from_str::<SkillPolicyActivationEntry>(&payload)
                    .context("deserialize skill policy activation payload")?,
            );
        }
        Ok(SkillPolicyActivationEntriesResponse { activations })
    }
}

use super::*;

/// L2.5: build a `DivergenceSummary` from `memory_items`.
///
/// Strategy: scan all `kind = "decision"` rows with a non-null belief branch
/// (matched by `MemoryItem.belief_branch`), group by branch, take the 3
/// most recently updated decisions per branch, then keep the 2 most
/// recently active branches overall.
///
/// Normalization: trim, lowercase, collapse internal whitespace. This
/// collapses noise that would otherwise hide that two branches made the
/// *same* decision with different formatting.
impl SqliteStore {
    pub fn hive_divergence(
        &self,
        request: &DivergenceRequest,
    ) -> anyhow::Result<DivergenceSummary> {
        let conn = self.connect()?;
        // Pull all decisions with a belief_branch. The table is typically
        // small relative to the full corpus; caps applied in-memory.
        let mut stmt = conn
            .prepare(
                r#"
                SELECT id, payload_json, updated_at
                FROM memory_items
                WHERE kind = '"decision"'
                  AND (?1 IS NULL OR project IS ?1 OR project = ?1)
                  AND (?2 IS NULL OR namespace IS ?2 OR namespace = ?2)
                ORDER BY updated_at DESC
                "#,
            )
            .context("prepare divergence decisions query")?;
        let rows = stmt
            .query_map(params![request.project, request.namespace], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .context("query divergence decisions")?;

        let mut by_branch: std::collections::BTreeMap<String, Vec<DivergenceDecision>> =
            std::collections::BTreeMap::new();
        let mut branch_last_seen: std::collections::BTreeMap<
            String,
            chrono::DateTime<chrono::Utc>,
        > = std::collections::BTreeMap::new();
        let mut normalized_seen: std::collections::BTreeMap<
            String,
            std::collections::HashSet<String>,
        > = std::collections::BTreeMap::new();
        for row in rows {
            let (_id_raw, payload, _updated_raw) = row.context("read divergence row")?;
            let item: MemoryItem = match serde_json::from_str(&payload) {
                Ok(value) => value,
                Err(_) => continue,
            };
            let Some(branch) = item
                .belief_branch
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
            else {
                continue;
            };
            if let Some(workspace) = request.workspace.as_deref()
                && item.workspace.as_deref() != Some(workspace)
            {
                continue;
            }
            let text = item.content.chars().take(280).collect::<String>();
            let normalized = normalize_decision(&text);
            if normalized.is_empty() {
                continue;
            }
            let seen = normalized_seen.entry(branch.clone()).or_default();
            if !seen.insert(normalized.clone()) {
                continue;
            }
            let bucket = by_branch.entry(branch.clone()).or_default();
            if bucket.len() < DivergenceBranch::MAX_DECISIONS {
                bucket.push(DivergenceDecision {
                    id: item.id,
                    text,
                    normalized,
                    updated_at: item.updated_at,
                });
            }
            branch_last_seen
                .entry(branch)
                .and_modify(|ts| {
                    if *ts < item.updated_at {
                        *ts = item.updated_at;
                    }
                })
                .or_insert(item.updated_at);
        }

        // Order branches by most-recent-decision desc, then name asc for
        // ties. Keep up to MAX_BRANCHES.
        let mut ordered: Vec<(String, chrono::DateTime<chrono::Utc>)> =
            branch_last_seen.into_iter().collect();
        ordered.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let total_branches = ordered.len();
        ordered.truncate(DivergenceSummary::MAX_BRANCHES);

        let branches = ordered
            .into_iter()
            .map(|(name, _)| {
                let decisions = by_branch.remove(&name).unwrap_or_default();
                let truncated_decisions = decisions.len() == DivergenceBranch::MAX_DECISIONS;
                DivergenceBranch {
                    branch_name: name,
                    decisions,
                    truncated_decisions,
                }
            })
            .collect();

        Ok(DivergenceSummary {
            branches,
            truncated_branches: total_branches > DivergenceSummary::MAX_BRANCHES,
        })
    }
}

fn normalize_decision(text: &str) -> String {
    let lower = text.trim().to_lowercase();
    let mut out = String::with_capacity(lower.len());
    let mut last_space = true;
    for ch in lower.chars() {
        if ch.is_whitespace() {
            if !last_space {
                out.push(' ');
                last_space = true;
            }
        } else {
            out.push(ch);
            last_space = false;
        }
    }
    out.trim().to_string()
}

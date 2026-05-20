fn count_recoverable_decision_records(
    context: &memd_schema::CompactContextResponse,
    working: &memd_schema::WorkingMemoryResponse,
    preferences: &[String],
) -> usize {
    recoverable_signal_counts(context, working, preferences).decisions
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct RecoverableSignalCounts {
    facts: usize,
    decisions: usize,
    total: usize,
}

fn recoverable_signal_counts(
    context: &memd_schema::CompactContextResponse,
    working: &memd_schema::WorkingMemoryResponse,
    preferences: &[String],
) -> RecoverableSignalCounts {
    let mut seen = std::collections::HashSet::new();
    let mut counts = RecoverableSignalCounts::default();
    for record in preferences
        .iter()
        .map(|record| record.as_str())
        .chain(context.records.iter().map(|record| record.record.as_str()))
        .chain(working.records.iter().map(|record| record.record.as_str()))
        .chain(
            working
                .rehydration_queue
                .iter()
                .map(|record| record.summary.as_str()),
        )
    {
        let normalized = normalize_resume_record(record);
        if normalized.is_empty() || !seen.insert(normalized) {
            continue;
        }
        let is_fact = is_fact_record_text(record) || is_recoverable_status_fact(record);
        let is_decision = is_decision_record_text(record) || is_recoverable_status_decision(record);
        let is_procedural = is_procedural_record_text(record);
        if is_fact {
            counts.facts += 1;
        }
        if is_decision {
            counts.decisions += 1;
        }
        if is_fact || is_decision || is_procedural {
            counts.total += 1;
        }
    }
    counts
}

pub(crate) fn count_decision_records(records: &[String]) -> usize {
    count_decision_record_texts(records.iter().map(|record| record.as_str()))
}

fn count_decision_record_texts<'a, I>(records: I) -> usize
where
    I: IntoIterator<Item = &'a str>,
{
    let mut seen = std::collections::HashSet::new();
    records
        .into_iter()
        .filter(|record| is_decision_record_text(record))
        .filter(|record| seen.insert(normalize_resume_record(record)))
        .count()
}

fn is_decision_record_text(record: &str) -> bool {
    let normalized = record.to_ascii_lowercase();
    normalized.contains("| kind=decision |")
        || normalized.contains(" kind=decision ")
        || normalized.starts_with("decision:")
        || normalized.contains("decision: ")
}

fn is_fact_record_text(record: &str) -> bool {
    let normalized = record.to_ascii_lowercase();
    normalized.contains("| kind=fact |")
        || normalized.contains(" kind=fact ")
        || normalized.starts_with("fact:")
        || normalized.contains("fact: ")
}

fn is_status_record_text(record: &str) -> bool {
    let normalized = record.to_ascii_lowercase();
    normalized.contains("| kind=status |") || normalized.contains(" kind=status ")
}

fn is_recoverable_status_decision(record: &str) -> bool {
    if !is_status_record_text(record) {
        return false;
    }
    let normalized = record.to_ascii_lowercase();
    normalized.contains("current next action")
        || normalized.contains("next action")
        || normalized.contains("next=")
        || normalized.contains("fix partial handoff quality")
        || normalized.contains("current-task")
        || normalized.contains("resume_state")
        || normalized.contains("session_state")
}

fn is_recoverable_status_fact(record: &str) -> bool {
    if !is_status_record_text(record) {
        return false;
    }
    let normalized = record.to_ascii_lowercase();
    normalized.contains("proof_blockers=")
        || normalized.contains("live_state_blockers=")
        || normalized.contains("clawcontrol")
        || normalized.contains("auth_required")
        || normalized.contains("git_commit")
        || normalized.contains("tree clean")
        || normalized.contains("status: wake")
        || normalized.contains("bundle-refresh")
        || normalized.contains("resume_state")
        || normalized.contains("session_state")
}

fn is_procedural_record_text(record: &str) -> bool {
    let normalized = record.to_ascii_lowercase();
    normalized.contains("| kind=procedural |")
        || normalized.contains(" kind=procedural ")
        || normalized.starts_with("procedure:")
        || normalized.contains("procedure: ")
}

fn is_next_action_record(record: &str) -> bool {
    let normalized = record.to_ascii_lowercase();
    if is_auto_bundle_refresh_status_record(&normalized) {
        return false;
    }
    is_decision_record_text(record)
        || normalized.contains("next-agent")
        || normalized.contains("next action")
        || normalized.contains("next_action")
}

fn is_auto_bundle_refresh_status_record(normalized: &str) -> bool {
    (normalized.contains("status: wake project=")
        || normalized.contains("source_path=wake")
        || normalized.contains("source_path=checkpoint")
        || normalized.contains("source_path=handoff"))
        && (normalized.contains("auto-short-term") || normalized.contains("bundle-refresh"))
}

fn best_next_action_record(records: Vec<String>) -> Option<String> {
    records
        .into_iter()
        .filter(|record| is_next_action_record(record))
        .max_by_key(|record| record_updated_at(record).unwrap_or(0))
}

fn blocker_from_next_action(next_action: &str) -> Option<String> {
    let markers = [
        "Remaining blockers are ",
        "Remaining blocker pair: ",
        "Remaining blockers: ",
        "blockers are ",
        "blockers: ",
    ];
    markers.iter().find_map(|marker| {
        let (_, tail) = next_action.split_once(marker)?;
        let blocker = tail
            .split(['.', '\n'])
            .next()
            .unwrap_or(tail)
            .trim()
            .trim_end_matches(';')
            .trim();
        (!blocker.is_empty()).then(|| blocker.to_string())
    })
}

fn latest_raw_spine_next_action(output: &Path) -> Option<String> {
    let path = output.join("state").join("raw-spine.jsonl");
    let raw = std::fs::read_to_string(path).ok()?;
    let candidates = raw
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter_map(|value| {
            let content = value
                .get("content_preview")
                .and_then(|value| value.as_str())
                .or_else(|| value.get("content").and_then(|value| value.as_str()))?;
            let tags = value
                .get("tags")
                .and_then(|value| value.as_array())
                .map(|tags| {
                    tags.iter()
                        .filter_map(|tag| tag.as_str())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if tags
                .iter()
                .any(|tag| tag.starts_with("security:") || tag.starts_with("quarantine:"))
            {
                return None;
            }
            let content_lower = content.to_ascii_lowercase();
            let looks_like_current_checkpoint =
                content_lower.contains("current checkpoint")
                    && tags.iter().any(|tag| *tag == "current-task")
                    && tags.iter().any(|tag| *tag == "checkpoint")
                    && !tags.iter().any(|tag| *tag == "auto-short-term");
            let looks_like_next = content_lower.contains("current next action")
                || looks_like_current_checkpoint
                || tags.iter().any(|tag| *tag == "next-agent");
            if !looks_like_next {
                return None;
            }
            let stage = value
                .get("stage")
                .and_then(|value| value.as_str())
                .unwrap_or("raw");
            let status = value
                .get("status")
                .and_then(|value| value.as_str())
                .unwrap_or("active");
            if matches!(status, "archived" | "superseded" | "rejected") {
                return None;
            }
            let id = value
                .get("id")
                .and_then(|value| value.as_str())
                .unwrap_or("raw-next-action");
            let upd = value
                .get("recorded_at")
                .and_then(|value| value.as_str())
                .and_then(parse_record_timestamp)
                .unwrap_or(0);
            let tag_text = if tags.is_empty() {
                "next-agent".to_string()
            } else {
                tags.join(",")
            };
            Some((
                stage == "canonical",
                upd,
                format!(
                    "id={id} | stage={stage} | kind=decision | status={status} | tags={tag_text} | upd={upd} | c={content}"
                ),
            ))
        })
        .collect::<Vec<_>>();
    candidates
        .iter()
        .filter(|(canonical, _, _)| *canonical)
        .max_by_key(|(_, upd, _)| *upd)
        .or_else(|| candidates.iter().max_by_key(|(_, upd, _)| *upd))
        .map(|(_, _, record)| record.clone())
}

fn record_updated_at(record: &str) -> Option<i64> {
    let (_, tail) = record.split_once("| upd=")?;
    let value = tail
        .split(|ch: char| !ch.is_ascii_digit())
        .next()
        .unwrap_or_default();
    value.parse::<i64>().ok()
}

fn parse_record_timestamp(value: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|timestamp| timestamp.timestamp())
}

pub(crate) fn truth_status_label(snapshot: &ResumeSnapshot) -> String {
    if snapshot.refresh_recommended {
        "aging".to_string()
    } else if snapshot.redundant_context_items() > 0 {
        "contested".to_string()
    } else if !snapshot.event_spine().is_empty() {
        "current".to_string()
    } else if !snapshot.compact_working_records().is_empty() {
        "working".to_string()
    } else {
        "fallback".to_string()
    }
}

pub(crate) fn truth_epistemic_state_label(snapshot: &ResumeSnapshot) -> String {
    if !snapshot.event_spine().is_empty() {
        "verified".to_string()
    } else if !snapshot.compact_working_records().is_empty() {
        "claimed".to_string()
    } else if !snapshot.compact_rehydration_summaries().is_empty()
        || !snapshot.compact_context_records().is_empty()
        || !snapshot.compact_semantic_items().is_empty()
        || !snapshot.compact_inbox_items().is_empty()
    {
        "inferred".to_string()
    } else {
        "unknown".to_string()
    }
}

pub(crate) fn choose_retrieval_tier(snapshot: &ResumeSnapshot) -> RetrievalTier {
    if !snapshot.event_spine().is_empty() {
        RetrievalTier::Hot
    } else if !snapshot.compact_working_records().is_empty()
        || !snapshot.compact_context_records().is_empty()
    {
        RetrievalTier::Working
    } else if !snapshot.compact_rehydration_summaries().is_empty() {
        RetrievalTier::Rehydration
    } else if !snapshot.compact_semantic_items().is_empty() {
        RetrievalTier::Evidence
    } else {
        RetrievalTier::RawFallback
    }
}

pub(crate) fn top_source_provenance(snapshot: &ResumeSnapshot) -> String {
    snapshot
        .sources
        .sources
        .iter()
        .max_by(|left, right| left.trust_score.total_cmp(&right.trust_score))
        .map(|source| {
            ResumeSnapshot::source_label(
                source.source_agent.as_deref(),
                source.source_system.as_deref(),
                None,
            )
        })
        .unwrap_or_else(|| "bundle / compact".to_string())
}

pub(crate) fn top_source_confidence(snapshot: &ResumeSnapshot) -> f32 {
    snapshot
        .sources
        .sources
        .iter()
        .map(|source| source.avg_confidence)
        .max_by(|left, right| left.total_cmp(right))
        .unwrap_or(0.92)
}

pub(crate) fn build_truth_record_summary(
    lane: &str,
    truth: &str,
    epistemic_state: &str,
    freshness: &str,
    retrieval_tier: RetrievalTier,
    confidence: f32,
    provenance: &str,
    preview: &str,
) -> TruthRecordSummary {
    TruthRecordSummary {
        lane: lane.to_string(),
        truth: truth.to_string(),
        epistemic_state: epistemic_state.to_string(),
        freshness: freshness.to_string(),
        retrieval_tier,
        confidence,
        provenance: provenance.to_string(),
        preview: compact_inline(preview, 120),
    }
}

pub(crate) fn build_truth_summary(snapshot: &ResumeSnapshot) -> TruthSummary {
    let freshness = truth_freshness_label(snapshot);
    let truth = truth_status_label(snapshot);
    let epistemic_state = truth_epistemic_state_label(snapshot);
    let retrieval_tier = choose_retrieval_tier(snapshot);
    let provenance = top_source_provenance(snapshot);
    let confidence = top_source_confidence(snapshot);
    let mut records = Vec::new();

    if let Some(event) = snapshot.event_spine().first() {
        records.push(build_truth_record_summary(
            "live_truth",
            "current",
            "verified",
            &freshness,
            RetrievalTier::Hot,
            confidence.max(0.95),
            "event_spine / compact",
            event,
        ));
    }
    if let Some(record) = snapshot.compact_working_records().first() {
        records.push(build_truth_record_summary(
            "working_set",
            if snapshot.redundant_context_items() > 0 {
                "contested"
            } else {
                "working"
            },
            if snapshot.redundant_context_items() > 0 {
                "inferred"
            } else {
                "claimed"
            },
            &freshness,
            RetrievalTier::Working,
            confidence,
            &provenance,
            record,
        ));
    }
    if let Some(item) = snapshot.compact_rehydration_summaries().first() {
        records.push(build_truth_record_summary(
            "rehydration",
            "pending",
            "inferred",
            &freshness,
            RetrievalTier::Rehydration,
            (confidence - 0.08).max(0.5),
            "rehydration / deferred",
            item,
        ));
    }
    if let Some(item) = snapshot.compact_semantic_items().first() {
        records.push(build_truth_record_summary(
            "evidence",
            "evidence",
            "verified",
            &freshness,
            RetrievalTier::Evidence,
            confidence,
            &provenance,
            item,
        ));
    }
    if let Some(item) = snapshot.compact_inbox_items().first() {
        records.push(build_truth_record_summary(
            "inbox",
            "candidate",
            "inferred",
            &freshness,
            RetrievalTier::Working,
            (confidence - 0.12).max(0.45),
            "inbox / unmerged",
            item,
        ));
    }

    records.truncate(4);

    TruthSummary {
        retrieval_tier,
        truth,
        epistemic_state,
        freshness,
        confidence,
        action_hint: snapshot.memory_action_hint().to_string(),
        source_count: snapshot.sources.sources.len(),
        contested_sources: snapshot
            .sources
            .sources
            .iter()
            .filter(|source| source.contested_count > 0)
            .count(),
        compact_records: snapshot.compact_context_records().len()
            + snapshot.compact_working_records().len()
            + snapshot.compact_rehydration_summaries().len()
            + snapshot.compact_inbox_items().len(),
        records,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HandoffSnapshot {
    pub(crate) generated_at: DateTime<Utc>,
    pub(crate) resume: ResumeSnapshot,
    pub(crate) sources: memd_schema::SourceMemoryResponse,
    #[serde(default = "default_voice_mode")]
    pub(crate) voice_mode: String,
    pub(crate) target_session: Option<String>,
    pub(crate) target_bundle: Option<String>,
}

pub(crate) fn build_resume_snapshot_cache_key(
    args: &ResumeArgs,
    runtime: Option<&BundleRuntimeConfig>,
    base_url: &str,
) -> String {
    let project = args
        .project
        .as_deref()
        .or_else(|| runtime.and_then(|config| config.project.as_deref()));
    let namespace = args
        .namespace
        .as_deref()
        .or_else(|| runtime.and_then(|config| config.namespace.as_deref()));
    let agent = args
        .agent
        .as_deref()
        .or_else(|| runtime.and_then(|config| config.agent.as_deref()));
    let session = runtime.and_then(|config| config.session.as_deref());
    let tab_id = runtime.and_then(|config| config.tab_id.as_deref());
    let workspace = args
        .workspace
        .as_deref()
        .or_else(|| runtime.and_then(|config| config.workspace.as_deref()));
    let visibility = args
        .visibility
        .as_deref()
        .or_else(|| runtime.and_then(|config| config.visibility.as_deref()));
    let route = args
        .route
        .as_deref()
        .or_else(|| runtime.and_then(|config| config.route.as_deref()))
        .unwrap_or("auto");
    let intent = args
        .intent
        .as_deref()
        .or_else(|| runtime.and_then(|config| config.intent.as_deref()))
        .unwrap_or("general");
    let limit = args.limit.unwrap_or(8);
    let rehydration_limit = args.rehydration_limit.unwrap_or(4);
    let semantic = if args.semantic { "true" } else { "false" };
    let query = format!(
        "session={}|tab={}|workspace={}|visibility={}|route={}|intent={}|base_url={}|limit={}|rehydration_limit={}|semantic={}",
        session.unwrap_or("none"),
        tab_id.unwrap_or("none"),
        workspace.unwrap_or("none"),
        visibility.unwrap_or("none"),
        route,
        intent,
        base_url,
        limit,
        rehydration_limit,
        semantic
    );
    cache::build_turn_key(project, namespace, agent, "resume", &query)
}

pub(crate) fn invalidate_bundle_runtime_caches(output: &Path) -> anyhow::Result<()> {
    for path in [
        cache::resume_snapshot_cache_path(output),
        cache::handoff_snapshot_cache_path(output),
    ] {
        match fs::remove_file(&path) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => {
                return Err(err).with_context(|| format!("remove {}", path.display()));
            }
        }
    }
    Ok(())
}

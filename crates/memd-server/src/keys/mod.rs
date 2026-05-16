use chrono::{Duration, Utc};
use memd_schema::{MemoryItem, MemoryKind, MemoryStatus};
use sha2::{Digest, Sha256};

/// Per-kind freshness windows from lifecycle contract §3.
fn freshness_window_days(kind: MemoryKind) -> i64 {
    match kind {
        MemoryKind::LiveTruth => 1,
        MemoryKind::Status => 2,
        MemoryKind::Pattern => 7,
        MemoryKind::Fact | MemoryKind::Decision | MemoryKind::Procedural => 14,
        MemoryKind::Preference
        | MemoryKind::Constraint
        | MemoryKind::Runbook
        | MemoryKind::SelfModel
        | MemoryKind::Topology => 30,
        MemoryKind::Correction => 30,
        MemoryKind::Skill => 30, // Phase 1: stub freshness window
    }
}

pub fn validate_source_quality(
    source_quality: Option<memd_schema::SourceQuality>,
) -> anyhow::Result<()> {
    if matches!(source_quality, Some(memd_schema::SourceQuality::Synthetic)) {
        anyhow::bail!("synthetic source quality is not allowed");
    }
    Ok(())
}

pub fn canonical_key(item: &MemoryItem) -> String {
    let normalized_content = item
        .content
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    let normalized_tags = normalized_tags(&item.tags);
    let project = item.project.as_deref().unwrap_or("");
    let namespace = item.namespace.as_deref().unwrap_or("");
    let belief_branch = item.belief_branch.as_deref().unwrap_or("");

    format!(
        "{:?}|{:?}|{}|{}|{}|{}|{}",
        item.kind,
        item.scope,
        project,
        namespace,
        belief_branch,
        normalized_content,
        normalized_tags
    )
}

pub fn redundancy_key(item: &MemoryItem) -> String {
    let mut words: Vec<String> = item
        .content
        .split(|c: char| !c.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .filter_map(normalize_redundancy_token)
        .collect();
    words.sort();
    words.dedup();

    let project = item.project.as_deref().unwrap_or("");
    let namespace = item.namespace.as_deref().unwrap_or("");
    let belief_branch = item.belief_branch.as_deref().unwrap_or("");
    format!(
        "{:?}|{:?}|{}|{}|{}|{}",
        item.kind,
        item.scope,
        project.to_ascii_lowercase(),
        namespace.to_ascii_lowercase(),
        belief_branch.to_ascii_lowercase(),
        words.join("|")
    )
}

fn normalize_redundancy_token(token: &str) -> Option<String> {
    let token = token.to_ascii_lowercase();
    if matches!(
        token.as_str(),
        "the"
            | "a"
            | "an"
            | "and"
            | "or"
            | "to"
            | "of"
            | "in"
            | "on"
            | "for"
            | "with"
            | "by"
            | "is"
            | "are"
            | "was"
            | "were"
            | "be"
            | "been"
            | "being"
            | "it"
            | "this"
            | "that"
            | "these"
            | "those"
    ) {
        return None;
    }

    Some(stem_redundancy_token(&token))
}

fn stem_redundancy_token(token: &str) -> String {
    let mut stemmed = token.to_string();
    if stemmed.len() > 5 && stemmed.ends_with("ing") {
        stemmed.truncate(stemmed.len() - 3);
    } else if stemmed.len() > 4 && (stemmed.ends_with("ed") || stemmed.ends_with("es")) {
        stemmed.truncate(stemmed.len() - 2);
    } else if stemmed.len() > 3 && stemmed.ends_with('s') {
        stemmed.truncate(stemmed.len() - 1);
    }
    stemmed
}

pub fn apply_lifecycle(mut item: MemoryItem) -> (MemoryItem, bool) {
    let now = Utc::now();

    if item.status != MemoryStatus::Expired
        && let Some(ttl_seconds) = item.ttl_seconds
    {
        let ttl_expired = item.created_at + Duration::seconds(ttl_seconds as i64) <= now;
        if ttl_expired {
            item.status = MemoryStatus::Expired;
            item.updated_at = now;
            return (item, true);
        }
    }

    let window = freshness_window_days(item.kind);
    let reference_time = item.last_verified_at.unwrap_or(item.updated_at);

    if item.status == MemoryStatus::Active
        && item.stage == memd_schema::MemoryStage::Canonical
        && reference_time + Duration::days(window) <= now
    {
        item.status = MemoryStatus::Stale;
        item.updated_at = now;
        return (item, true);
    }

    if item.status == MemoryStatus::Stale && reference_time + Duration::days(window * 2) <= now {
        item.status = MemoryStatus::Expired;
        item.updated_at = now;
        return (item, true);
    }

    (item, false)
}

fn normalized_tags(tags: &[String]) -> String {
    let mut tags = tags
        .iter()
        .map(|tag| {
            tag.split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .to_ascii_lowercase()
        })
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();
    tags.sort();
    tags.dedup();
    tags.join(",")
}

/// Normalize content for hashing: trim, strip leading "- ", lowercase, collapse whitespace.
pub fn normalize_for_hash(s: &str) -> String {
    let s = s.trim();
    let s = s.strip_prefix("- ").unwrap_or(s);
    s.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// SHA-256 content hash, first 16 hex chars. Used for ingestion dedup.
pub fn content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(normalize_for_hash(content).as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..8])
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use memd_schema::{
        MemoryKind, MemoryScope, MemoryStage, MemoryStatus, MemoryVisibility, SourceQuality,
    };
    use uuid::Uuid;

    fn test_item(content: &str) -> MemoryItem {
        MemoryItem {
            id: Uuid::nil(),
            content: content.to_string(),
            redundancy_key: None,
            belief_branch: None,
            preferred: false,
            kind: MemoryKind::Fact,
            scope: MemoryScope::Project,
            project: Some("memd".into()),
            namespace: Some("test".into()),
            workspace: Some("core".into()),
            visibility: MemoryVisibility::Workspace,
            source_agent: Some("codex".into()),
            source_system: Some("memd".into()),
            source_path: None,
            source_quality: Some(SourceQuality::Canonical),
            confidence: 0.9,
            ttl_seconds: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_verified_at: None,
            supersedes: Vec::new(),
            tags: vec!["alpha".into(), "beta".into()],
            status: MemoryStatus::Active,
            stage: MemoryStage::Candidate,
            lane: None,
            version: 1,
            correction_meta: None,
        }
    }

    #[test]
    fn redundancy_key_collapses_paraphrases() {
        let a = test_item("Cloudflare Tunnel protects rag.example.com");
        let b = test_item("rag.example.com is protected by Cloudflare Tunnel");
        assert_eq!(redundancy_key(&a), redundancy_key(&b));
    }

    #[test]
    fn canonical_key_keeps_surface_differences_out_of_the_hot_path() {
        let a = test_item("Cloudflare Tunnel protects rag.example.com");
        let b = test_item("rag.example.com is protected by Cloudflare Tunnel");
        assert_ne!(canonical_key(&a), canonical_key(&b));
    }

    #[test]
    fn branch_key_separates_competing_beliefs() {
        let mut mainline = test_item("rag uses bundle-first config");
        mainline.belief_branch = Some("mainline".into());
        let mut fallback = test_item("rag uses bundle-first config");
        fallback.belief_branch = Some("fallback".into());
        assert_ne!(redundancy_key(&mainline), redundancy_key(&fallback));
        assert_ne!(canonical_key(&mainline), canonical_key(&fallback));
    }

    #[test]
    fn synthetic_source_quality_is_rejected() {
        assert!(validate_source_quality(Some(SourceQuality::Synthetic)).is_err());
        assert!(validate_source_quality(Some(SourceQuality::Derived)).is_ok());
    }

    #[test]
    fn freshness_window_matches_lifecycle_contract() {
        assert_eq!(freshness_window_days(MemoryKind::LiveTruth), 1);
        assert_eq!(freshness_window_days(MemoryKind::Status), 2);
        assert_eq!(freshness_window_days(MemoryKind::Pattern), 7);
        assert_eq!(freshness_window_days(MemoryKind::Fact), 14);
        assert_eq!(freshness_window_days(MemoryKind::Decision), 14);
        assert_eq!(freshness_window_days(MemoryKind::Procedural), 14);
        assert_eq!(freshness_window_days(MemoryKind::Preference), 30);
        assert_eq!(freshness_window_days(MemoryKind::Constraint), 30);
        assert_eq!(freshness_window_days(MemoryKind::Runbook), 30);
        assert_eq!(freshness_window_days(MemoryKind::SelfModel), 30);
        assert_eq!(freshness_window_days(MemoryKind::Topology), 30);
    }

    #[test]
    fn staleness_marks_fact_stale_after_14_days() {
        let mut item = test_item("the server runs debian");
        item.stage = MemoryStage::Canonical;
        item.kind = MemoryKind::Fact;
        item.updated_at = Utc::now() - Duration::days(15);
        let (result, changed) = apply_lifecycle(item);
        assert!(changed);
        assert_eq!(result.status, MemoryStatus::Stale);
    }

    #[test]
    fn staleness_does_not_trigger_within_window() {
        let mut item = test_item("the server runs debian");
        item.stage = MemoryStage::Canonical;
        item.kind = MemoryKind::Fact;
        item.updated_at = Utc::now() - Duration::days(10);
        let (result, changed) = apply_lifecycle(item);
        assert!(!changed);
        assert_eq!(result.status, MemoryStatus::Active);
    }

    #[test]
    fn staleness_uses_last_verified_at_when_present() {
        let mut item = test_item("the server runs debian");
        item.stage = MemoryStage::Canonical;
        item.kind = MemoryKind::Fact;
        item.updated_at = Utc::now() - Duration::days(30);
        item.last_verified_at = Some(Utc::now() - Duration::days(5));
        let (result, changed) = apply_lifecycle(item);
        assert!(!changed);
        assert_eq!(result.status, MemoryStatus::Active);
    }

    #[test]
    fn live_truth_goes_stale_after_1_day() {
        let mut item = test_item("build is green");
        item.stage = MemoryStage::Canonical;
        item.kind = MemoryKind::LiveTruth;
        item.updated_at = Utc::now() - Duration::days(2);
        let (result, changed) = apply_lifecycle(item);
        assert!(changed);
        assert_eq!(result.status, MemoryStatus::Stale);
    }

    #[test]
    fn double_window_expires_stale_fact() {
        let mut item = test_item("the server runs debian");
        item.stage = MemoryStage::Canonical;
        item.kind = MemoryKind::Fact;
        item.status = MemoryStatus::Stale;
        item.updated_at = Utc::now() - Duration::days(29);
        let (result, changed) = apply_lifecycle(item);
        assert!(changed);
        assert_eq!(result.status, MemoryStatus::Expired);
    }

    #[test]
    fn double_window_does_not_expire_within_range() {
        let mut item = test_item("the server runs debian");
        item.stage = MemoryStage::Canonical;
        item.kind = MemoryKind::Fact;
        item.status = MemoryStatus::Stale;
        item.updated_at = Utc::now() - Duration::days(20);
        let (result, changed) = apply_lifecycle(item);
        assert!(!changed);
        assert_eq!(result.status, MemoryStatus::Stale);
    }

    #[test]
    fn ttl_takes_precedence_over_staleness() {
        let mut item = test_item("ephemeral note");
        item.stage = MemoryStage::Canonical;
        item.kind = MemoryKind::Fact;
        item.ttl_seconds = Some(60);
        item.created_at = Utc::now() - Duration::seconds(120);
        item.updated_at = Utc::now();
        let (result, changed) = apply_lifecycle(item);
        assert!(changed);
        assert_eq!(result.status, MemoryStatus::Expired);
    }

    #[test]
    fn candidate_stage_skips_staleness() {
        let mut item = test_item("candidate note");
        item.stage = MemoryStage::Candidate;
        item.kind = MemoryKind::Fact;
        item.updated_at = Utc::now() - Duration::days(30);
        let (result, changed) = apply_lifecycle(item);
        assert!(!changed);
        assert_eq!(result.status, MemoryStatus::Active);
    }
}

use chrono::{Duration, Utc};
use memd_schema::{MemoryItem, MemoryStatus};

const STALE_AFTER_DAYS: i64 = 30;

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
    } else if stemmed.len() > 4 && stemmed.ends_with("ed") {
        stemmed.truncate(stemmed.len() - 2);
    } else if stemmed.len() > 4 && stemmed.ends_with("es") {
        stemmed.truncate(stemmed.len() - 2);
    } else if stemmed.len() > 3 && stemmed.ends_with('s') {
        stemmed.truncate(stemmed.len() - 1);
    }
    stemmed
}

pub fn apply_lifecycle(mut item: MemoryItem) -> (MemoryItem, bool) {
    let now = Utc::now();

    if item.status != MemoryStatus::Expired {
        if let Some(ttl_seconds) = item.ttl_seconds {
            let ttl_expired = item.created_at + Duration::seconds(ttl_seconds as i64) <= now;
            if ttl_expired {
                item.status = MemoryStatus::Expired;
                item.updated_at = now;
                return (item, true);
            }
        }
    }

    if item.status == MemoryStatus::Active && item.stage == memd_schema::MemoryStage::Canonical {
        let reference_time = item.last_verified_at.unwrap_or(item.updated_at);
        if reference_time + Duration::days(STALE_AFTER_DAYS) <= now {
            item.status = MemoryStatus::Stale;
            item.updated_at = now;
            return (item, true);
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use memd_schema::{MemoryKind, MemoryScope, MemoryStage, MemoryStatus, SourceQuality};
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
}

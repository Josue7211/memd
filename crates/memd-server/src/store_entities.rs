use std::path::Path;

use memd_schema::{
    EntitySearchRequest, MemoryContextFrame, MemoryEntityRecord, MemoryItem, SourceQuality,
};
use uuid::Uuid;

pub(crate) fn derive_entity_key(item: &MemoryItem, canonical_key: &str) -> String {
    if let Some(source_path) = item.source_path.as_deref() {
        return format!(
            "path|{:?}|{:?}|{}",
            item.project.as_deref().unwrap_or(""),
            item.namespace.as_deref().unwrap_or(""),
            source_path
        );
    }

    if let Some(source_system) = item.source_system.as_deref() {
        return format!(
            "system|{:?}|{:?}|{:?}|{}",
            item.project.as_deref().unwrap_or(""),
            item.namespace.as_deref().unwrap_or(""),
            source_system,
            canonical_key
        );
    }

    format!(
        "entity|{:?}|{:?}|{:?}|{}",
        item.project.as_deref().unwrap_or(""),
        item.namespace.as_deref().unwrap_or(""),
        item.kind,
        canonical_key
    )
}

pub(crate) fn new_entity_record(item: &MemoryItem) -> MemoryEntityRecord {
    let now = chrono::Utc::now();
    MemoryEntityRecord {
        id: Uuid::new_v4(),
        entity_type: format!("{:?}", item.kind).to_lowercase(),
        aliases: entity_aliases(item),
        current_state: Some(compact_entity_state(item)),
        state_version: 1,
        confidence: item.confidence,
        salience_score: item.confidence.clamp(0.0, 1.0),
        rehearsal_count: 1,
        created_at: now,
        updated_at: now,
        last_accessed_at: Some(now),
        last_seen_at: Some(item.updated_at),
        valid_from: Some(item.updated_at),
        valid_to: None,
        tags: item.tags.clone(),
        context: Some(MemoryContextFrame {
            at: Some(item.updated_at),
            project: item.project.clone(),
            namespace: item.namespace.clone(),
            workspace: item.workspace.clone(),
            repo: item.source_system.clone(),
            host: None,
            branch: None,
            agent: item.source_agent.clone(),
            location: item.source_path.clone(),
        }),
    }
}

pub(crate) fn update_entity_record(
    mut record: MemoryEntityRecord,
    item: &MemoryItem,
) -> MemoryEntityRecord {
    let now = chrono::Utc::now();
    let previous = record.context.clone();
    let previous_project = previous
        .as_ref()
        .and_then(|context| context.project.clone());
    let previous_namespace = previous
        .as_ref()
        .and_then(|context| context.namespace.clone());
    let previous_repo = previous.as_ref().and_then(|context| context.repo.clone());
    let previous_host = previous.as_ref().and_then(|context| context.host.clone());
    let previous_branch = previous.as_ref().and_then(|context| context.branch.clone());
    let previous_agent = previous.as_ref().and_then(|context| context.agent.clone());
    let previous_location = previous
        .as_ref()
        .and_then(|context| context.location.clone());

    record.aliases = merge_aliases(&record.aliases, &entity_aliases(item));
    record.current_state = Some(compact_entity_state(item));
    record.state_version = record.state_version.saturating_add(1);
    record.confidence = record.confidence.max(item.confidence).clamp(0.0, 1.0);
    record.salience_score = (record.salience_score + 0.05).min(1.0);
    record.rehearsal_count = record.rehearsal_count.saturating_add(1);
    record.updated_at = now;
    record.last_accessed_at = Some(now);
    record.last_seen_at = Some(item.updated_at);
    if record.valid_from.is_none() {
        record.valid_from = Some(item.updated_at);
    }
    record.valid_to = None;
    record.tags = merge_tags(&record.tags, &item.tags);
    record.context = Some(MemoryContextFrame {
        at: Some(item.updated_at),
        project: item.project.clone().or(previous_project),
        namespace: item.namespace.clone().or(previous_namespace),
        workspace: item
            .workspace
            .clone()
            .or(previous.and_then(|context| context.workspace)),
        repo: item.source_system.clone().or(previous_repo),
        host: previous_host,
        branch: previous_branch,
        agent: item.source_agent.clone().or(previous_agent),
        location: item.source_path.clone().or(previous_location),
    });
    record
}

fn entity_aliases(item: &MemoryItem) -> Vec<String> {
    let mut aliases = Vec::new();
    if let Some(project) = &item.project {
        aliases.push(project.clone());
    }
    if let Some(namespace) = &item.namespace {
        aliases.push(namespace.clone());
    }
    if let Some(agent) = &item.source_agent {
        aliases.push(agent.clone());
    }
    if let Some(system) = &item.source_system {
        aliases.push(system.clone());
    }
    if let Some(path) = &item.source_path {
        aliases.push(path.clone());
        if let Some(file_name) = Path::new(path).file_name().and_then(|value| value.to_str()) {
            aliases.push(file_name.to_string());
        }
    }
    aliases.push(format!("{:?}", item.kind).to_lowercase());
    aliases.sort();
    aliases.dedup();
    aliases
}

fn merge_aliases(existing: &[String], incoming: &[String]) -> Vec<String> {
    let mut aliases = existing.to_vec();
    aliases.extend(incoming.iter().cloned());
    aliases.sort();
    aliases.dedup();
    aliases
}

fn merge_tags(existing: &[String], incoming: &[String]) -> Vec<String> {
    let mut tags = existing.to_vec();
    tags.extend(incoming.iter().cloned());
    tags.sort();
    tags.dedup();
    tags
}

pub(crate) type SourceKey = (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    memd_schema::MemoryVisibility,
);

pub(crate) type WorkspaceKey = (
    Option<String>,
    Option<String>,
    Option<String>,
    memd_schema::MemoryVisibility,
);

#[derive(Default)]
pub(crate) struct SourceAggregate {
    pub item_count: usize,
    pub active_count: usize,
    pub candidate_count: usize,
    pub derived_count: usize,
    pub synthetic_count: usize,
    pub contested_count: usize,
    pub confidence_sum: f32,
    pub last_seen_at: Option<chrono::DateTime<chrono::Utc>>,
    pub tag_counts: std::collections::BTreeMap<String, usize>,
}

impl SourceAggregate {
    pub(crate) fn observe(&mut self, item: &MemoryItem) {
        self.item_count = self.item_count.saturating_add(1);
        if item.stage == memd_schema::MemoryStage::Canonical {
            self.active_count = self.active_count.saturating_add(1);
        } else {
            self.candidate_count = self.candidate_count.saturating_add(1);
        }
        if item.source_quality == Some(SourceQuality::Derived) {
            self.derived_count = self.derived_count.saturating_add(1);
        }
        if item.source_quality == Some(SourceQuality::Synthetic) {
            self.synthetic_count = self.synthetic_count.saturating_add(1);
        }
        if item.status == memd_schema::MemoryStatus::Contested {
            self.contested_count = self.contested_count.saturating_add(1);
        }
        self.confidence_sum += item.confidence.clamp(0.0, 1.0);
        self.last_seen_at = match self.last_seen_at {
            Some(current) if current >= item.updated_at => Some(current),
            _ => Some(item.updated_at),
        };
        for tag in &item.tags {
            *self.tag_counts.entry(tag.clone()).or_insert(0) += 1;
        }
    }

    pub(crate) fn avg_confidence(&self) -> f32 {
        if self.item_count == 0 {
            0.0
        } else {
            (self.confidence_sum / self.item_count as f32).clamp(0.0, 1.0)
        }
    }

    pub(crate) fn tags(&self, limit: usize) -> Vec<String> {
        let mut tags = self
            .tag_counts
            .iter()
            .map(|(tag, count)| (tag.clone(), *count))
            .collect::<Vec<_>>();
        tags.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        tags.into_iter().take(limit).map(|(tag, _)| tag).collect()
    }
}

#[derive(Default)]
pub(crate) struct WorkspaceAggregate {
    pub source: SourceAggregate,
    pub source_lanes: std::collections::BTreeSet<(Option<String>, Option<String>)>,
}

impl WorkspaceAggregate {
    pub(crate) fn observe(&mut self, item: &MemoryItem) {
        self.source.observe(item);
        self.source_lanes
            .insert((item.source_agent.clone(), item.source_system.clone()));
    }
}

pub(crate) fn source_trust_score(
    item_count: usize,
    active_count: usize,
    candidate_count: usize,
    derived_count: usize,
    synthetic_count: usize,
    contested_count: usize,
    avg_confidence: f32,
) -> f32 {
    if item_count == 0 {
        return 0.0;
    }

    let active_ratio = active_count as f32 / item_count as f32;
    let derived_ratio = derived_count as f32 / item_count as f32;
    let candidate_ratio = candidate_count as f32 / item_count as f32;
    let synthetic_ratio = synthetic_count as f32 / item_count as f32;
    let contested_ratio = contested_count as f32 / item_count as f32;

    let score = avg_confidence * 0.58 + active_ratio * 0.18 + derived_ratio * 0.12
        - candidate_ratio * 0.05
        - synthetic_ratio * 0.18
        - contested_ratio * 0.14;
    score.clamp(0.0, 1.0)
}

pub(crate) fn normalize_search_text(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn tokenize_search_text(value: &str) -> Vec<String> {
    value
        .split_whitespace()
        .map(|value| value.to_string())
        .filter(|value| !value.is_empty())
        .collect()
}

pub(crate) fn score_entity_search(
    request: &EntitySearchRequest,
    query: &str,
    query_tokens: &[String],
    entity: &MemoryEntityRecord,
) -> (f32, Vec<String>) {
    let mut score = 0.0f32;
    let mut reasons = Vec::new();
    let haystacks = entity_search_haystacks(entity);

    for haystack in &haystacks {
        if haystack == query {
            score += 1.0;
            reasons.push("exact match".to_string());
        } else if haystack.starts_with(query) {
            score += 0.7;
            reasons.push("prefix match".to_string());
        } else if haystack.contains(query) {
            score += 0.5;
            reasons.push("substring match".to_string());
        }
    }

    for token in query_tokens {
        if haystacks.iter().any(|haystack| haystack.contains(token)) {
            score += 0.2;
            reasons.push(format!("token:{token}"));
        }
    }

    if query_tokens.len() > 1 {
        let joined = query_tokens.join(" ");
        if haystacks.iter().any(|haystack| haystack.contains(&joined)) {
            score += 0.28;
            reasons.push("phrase match".to_string());
        }
    }

    if entity.salience_score > 0.0 {
        score += entity.salience_score * 0.08;
    }
    if entity.rehearsal_count > 0 {
        score += (entity.rehearsal_count as f32).ln_1p() * 0.03;
    }
    if entity.valid_from.is_some() {
        score += 0.08;
        reasons.push("validity window".to_string());
    }
    if request.project.is_some()
        && entity
            .context
            .as_ref()
            .and_then(|context| context.project.as_ref())
            .is_some()
    {
        score += 0.05;
        reasons.push("project context".to_string());
    }
    if request.namespace.is_some()
        && entity
            .context
            .as_ref()
            .and_then(|context| context.namespace.as_ref())
            .is_some()
    {
        score += 0.05;
        reasons.push("namespace context".to_string());
    }
    if request.host.is_some()
        && entity
            .context
            .as_ref()
            .and_then(|context| context.host.as_ref())
            .is_some()
    {
        score += 0.05;
        reasons.push("host context".to_string());
    }
    if request.branch.is_some()
        && entity
            .context
            .as_ref()
            .and_then(|context| context.branch.as_ref())
            .is_some()
    {
        score += 0.05;
        reasons.push("branch context".to_string());
    }
    if request.location.is_some()
        && entity
            .context
            .as_ref()
            .and_then(|context| context.location.as_ref())
            .is_some()
    {
        score += 0.05;
        reasons.push("location context".to_string());
    }
    if request.at.is_some() {
        score += 0.05;
        reasons.push("timestamp context".to_string());
    }

    score = score.min(1.0);
    reasons.sort();
    reasons.dedup();
    (score, reasons)
}

fn entity_search_haystacks(entity: &MemoryEntityRecord) -> Vec<String> {
    let mut haystacks = Vec::new();
    haystacks.push(normalize_search_text(&entity.entity_type));
    haystacks.extend(
        entity
            .aliases
            .iter()
            .map(|alias| normalize_search_text(alias)),
    );
    if let Some(state) = &entity.current_state {
        haystacks.push(normalize_search_text(state));
    }
    if let Some(context) = &entity.context {
        if let Some(project) = &context.project {
            haystacks.push(normalize_search_text(project));
        }
        if let Some(namespace) = &context.namespace {
            haystacks.push(normalize_search_text(namespace));
        }
        if let Some(repo) = &context.repo {
            haystacks.push(normalize_search_text(repo));
        }
        if let Some(agent) = &context.agent {
            haystacks.push(normalize_search_text(agent));
        }
        if let Some(location) = &context.location {
            haystacks.push(normalize_search_text(location));
            if let Some(file_name) = Path::new(location)
                .file_name()
                .and_then(|value| value.to_str())
            {
                haystacks.push(normalize_search_text(file_name));
            }
        }
    }
    haystacks.extend(entity.tags.iter().map(|tag| normalize_search_text(tag)));
    haystacks.sort();
    haystacks.dedup();
    haystacks
}

pub(crate) fn entity_matches_context(
    entity: &MemoryEntityRecord,
    request: &EntitySearchRequest,
) -> bool {
    if let Some(at) = request.at {
        if entity.valid_from.is_some_and(|valid_from| at < valid_from) {
            return false;
        }
        if entity.valid_to.is_some_and(|valid_to| at > valid_to) {
            return false;
        }
    }

    let context = entity.context.as_ref();
    if request.project.as_ref().is_some_and(|project| {
        context
            .and_then(|context| context.project.as_ref())
            .is_none_or(|entity_project| entity_project != project)
    }) {
        return false;
    }
    if request.namespace.as_ref().is_some_and(|namespace| {
        context
            .and_then(|context| context.namespace.as_ref())
            .is_none_or(|entity_namespace| entity_namespace != namespace)
    }) {
        return false;
    }
    if request.host.as_ref().is_some_and(|host| {
        context
            .and_then(|context| context.host.as_ref())
            .is_none_or(|entity_host| entity_host != host)
    }) {
        return false;
    }
    if request.branch.as_ref().is_some_and(|branch| {
        context
            .and_then(|context| context.branch.as_ref())
            .is_none_or(|entity_branch| entity_branch != branch)
    }) {
        return false;
    }
    if request.location.as_ref().is_some_and(|location| {
        context
            .and_then(|context| context.location.as_ref())
            .is_none_or(|entity_location| entity_location != location)
    }) {
        return false;
    }

    true
}

fn compact_entity_state(item: &MemoryItem) -> String {
    let mut state = item
        .content
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if state.len() > 240 {
        state.truncate(240);
        state.push('…');
    }
    state
}

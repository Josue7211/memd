fn intrinsic_rerank_enabled() -> bool {
    match std::env::var("MEMD_RETRIEVAL_RERANK") {
        Ok(value) => {
            let value = value.trim().to_ascii_lowercase();
            !matches!(value.as_str(), "0" | "false" | "off" | "no")
        }
        Err(_) => true,
    }
}

const INTRINSIC_RERANK_WINDOW: usize = 20;

#[derive(Debug, Clone)]
struct SearchRankLane {
    name: &'static str,
    weight: f64,
    ranks: Vec<(Uuid, f64)>,
}

impl SearchRankLane {
    fn new(name: &'static str, weight: f64, ranks: Vec<(Uuid, f64)>) -> Self {
        Self {
            name,
            weight,
            ranks,
        }
    }
}

fn fuse_search_rank_lanes(lanes: &[SearchRankLane]) -> Vec<(Uuid, f64)> {
    const RRF_K: f64 = 60.0;
    let mut fused: std::collections::HashMap<Uuid, f64> = std::collections::HashMap::new();
    for lane in lanes.iter().filter(|lane| !lane.ranks.is_empty()) {
        let max_score = lane
            .ranks
            .iter()
            .map(|(_, score)| score.abs())
            .fold(0.0_f64, f64::max)
            .max(1.0);
        for (rank, (id, score)) in lane.ranks.iter().enumerate() {
            let rank_signal = 1.0 / (RRF_K + rank as f64);
            let score_signal = (score / max_score).clamp(0.0, 1.0) * 0.12;
            *fused.entry(*id).or_insert(0.0) += lane.weight * (rank_signal + score_signal);
        }
    }
    let mut ranks = fused.into_iter().collect::<Vec<_>>();
    ranks.sort_by(|a, b| b.1.total_cmp(&a.1));
    ranks
}

fn truth_guard_search_candidates(
    items: &[MemoryViewItem],
    candidate_ranks: &[(Uuid, f64)],
) -> Vec<(Uuid, f64)> {
    if candidate_ranks.is_empty() {
        return Vec::new();
    }
    let candidate_ids = candidate_ranks
        .iter()
        .map(|(id, _)| *id)
        .collect::<std::collections::HashSet<_>>();
    let now = Utc::now();
    let mut ranks = items
        .iter()
        .filter(|entry| candidate_ids.contains(&entry.item.id))
        .map(|entry| {
            let item = &entry.item;
            let mut score = item.confidence as f64;
            score += match item.stage {
                MemoryStage::Canonical => 1.0,
                MemoryStage::Candidate => -0.5,
            };
            score += match item.status {
                MemoryStatus::Active => 1.2,
                MemoryStatus::Stale => -2.0,
                MemoryStatus::Superseded => -3.0,
                MemoryStatus::Contested => -1.5,
                MemoryStatus::Expired => -5.0,
            };
            score += match item.source_quality {
                Some(SourceQuality::Canonical) => 0.5,
                Some(SourceQuality::Derived) => 0.1,
                Some(SourceQuality::Synthetic) => -0.5,
                None => 0.0,
            };
            score += durable_truth_rank_adjustment(item) as f64;
            score += source_linked_provenance_rank_adjustment(item);
            score += temporal_recency_rank_adjustment(now, item.updated_at);
            score += trust_rank_adjustment(entry.source_trust_score) as f64;
            if let Some(verified_at) = item.last_verified_at {
                let verified_days = now.signed_duration_since(verified_at).num_days().max(0);
                score += if verified_days <= 7 {
                    0.45
                } else if verified_days <= 30 {
                    0.2
                } else if verified_days <= 90 {
                    0.05
                } else {
                    -0.15
                };
            }
            (item.id, score)
        })
        .collect::<Vec<_>>();
    ranks.sort_by(|a, b| b.1.total_cmp(&a.1));
    ranks
}

fn source_linked_provenance_rank_adjustment(item: &MemoryItem) -> f64 {
    let mut score = 0.0;
    if item
        .source_path
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
    {
        score += 0.25;
    }
    if item
        .source_system
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
    {
        score += 0.10;
    }
    if item
        .source_agent
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
    {
        score += 0.05;
    }
    if item.source_path.is_none() && item.source_system.is_none() && item.last_verified_at.is_none()
    {
        score -= 0.15;
    }
    score
}

fn temporal_recency_rank_adjustment(
    now: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
) -> f64 {
    let age_days = now.signed_duration_since(updated_at).num_days().max(0);
    if age_days <= 1 {
        0.30
    } else if age_days <= 7 {
        0.22
    } else if age_days <= 30 {
        0.10
    } else if age_days <= 90 {
        0.03
    } else {
        -0.10
    }
}

fn build_search_trace(
    query: Option<String>,
    lanes: &[SearchRankLane],
    final_ranks: &[(Uuid, f64)],
    items: &[MemoryItem],
) -> Option<SearchRetrievalTrace> {
    let has_firewall_signal = items.iter().any(prompt_injection_firewall_flags_item);
    if lanes.iter().all(|lane| lane.ranks.is_empty()) && !has_firewall_signal {
        return None;
    }
    let mut lane_names = lanes
        .iter()
        .filter(|lane| !lane.ranks.is_empty())
        .map(|lane| lane.name.to_string())
        .collect::<Vec<_>>();
    if has_firewall_signal {
        lane_names.push("firewall".to_string());
    }
    let final_score_by_id = final_ranks
        .iter()
        .copied()
        .collect::<std::collections::HashMap<_, _>>();
    let mut item_traces = Vec::with_capacity(items.len());
    for (final_rank, item) in items.iter().enumerate() {
        let mut signals = Vec::new();
        for lane in lanes {
            if let Some((rank, (_, score))) = lane
                .ranks
                .iter()
                .enumerate()
                .find(|(_, (id, _))| *id == item.id)
            {
                signals.push(SearchSignalTrace {
                    lane: lane.name.to_string(),
                    score: *score,
                    rank: Some(rank + 1),
                    reason: Some(search_lane_reason(lane.name).to_string()),
                });
            }
        }
        if prompt_injection_firewall_flags_item(item) {
            signals.push(SearchSignalTrace {
                lane: "firewall".to_string(),
                score: 0.0,
                rank: None,
                reason: Some(
                    "suspicious memory is data/evidence only; never instruction".to_string(),
                ),
            });
        }
        item_traces.push(SearchItemTrace {
            id: item.id,
            final_rank: final_rank + 1,
            final_score: final_score_by_id.get(&item.id).copied().unwrap_or(0.0),
            signals,
        });
    }
    Some(SearchRetrievalTrace {
        query,
        lanes: lane_names,
        items: item_traces,
    })
}

fn search_lane_reason(lane: &str) -> &'static str {
    match lane {
        "fts_bm25" => "lexical/FTS5 BM25 candidate",
        "fuzzy" => "typo, alias, path, acronym, or id fuzzy match",
        "atlas" => "one-hop atlas neighbor candidate",
        "rag_dense_head" => "top optional sidecar dense candidate after ACL filtering",
        "rag_dense" => "optional sidecar dense candidate",
        "intrinsic_dense" => "intrinsic vector candidate",
        "recommendation" => "recommendation-intent evidence candidate",
        "rerank" => "final sidecar/intrinsic rerank order",
        "truth" => "ACL-safe truth, correction, trust, status, and recency guard",
        "firewall" => "prompt injection firewall label",
        _ => "retrieval signal",
    }
}

pub(crate) fn intrinsic_rerank_search_candidates(
    items: &[MemoryViewItem],
    query: &str,
    base_ranks: &[(Uuid, f64)],
) -> Vec<(Uuid, f64)> {
    let query = query.trim();
    if query.is_empty() || base_ranks.len() < 2 {
        return base_ranks.to_vec();
    }

    let by_id: std::collections::HashMap<Uuid, &MemoryItem> = items
        .iter()
        .map(|entry| (entry.item.id, &entry.item))
        .collect();
    let head_len = base_ranks.len().min(INTRINSIC_RERANK_WINDOW);
    let (head, tail) = base_ranks.split_at(head_len);

    let mut reranked = head
        .iter()
        .enumerate()
        .map(|(index, (id, _base_score))| {
            let base_norm = 1.0 / (10.0 + index as f64);
            let local_score = by_id
                .get(id)
                .map(|item| intrinsic_local_rerank_score(item, query))
                .unwrap_or(0.0);
            let blended = local_score * 0.82 + base_norm * 0.18;
            (*id, blended, index)
        })
        .collect::<Vec<_>>();

    reranked.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.2.cmp(&b.2)));

    let mut ordered = reranked
        .into_iter()
        .map(|(id, score, _)| (id, score))
        .collect::<Vec<_>>();
    ordered.extend_from_slice(tail);
    ordered
}

async fn rerank_search_candidates(
    state: &AppState,
    items: &[MemoryViewItem],
    query: &str,
    base_ranks: &[(Uuid, f64)],
) -> Vec<(Uuid, f64)> {
    let query = query.trim();
    if query.is_empty() || base_ranks.len() < 2 {
        return base_ranks.to_vec();
    }

    let by_id: std::collections::HashMap<Uuid, &MemoryItem> = items
        .iter()
        .map(|entry| (entry.item.id, &entry.item))
        .collect();
    let head_len = base_ranks.len().min(INTRINSIC_RERANK_WINDOW);
    let (head, tail) = base_ranks.split_at(head_len);

    if let Some(rag) = state.rag.as_deref() {
        let candidates = head
            .iter()
            .filter_map(|(id, _)| by_id.get(id).map(|item| (*id, rerank_item_haystack(item))))
            .collect::<Vec<_>>();
        if !candidates.is_empty() {
            match crate::rag_bridge::rerank_candidates(rag, query, &candidates, candidates.len())
                .await
            {
                Ok(reranked) if !reranked.is_empty() => {
                    let mut ordered = reranked;
                    let seen = ordered
                        .iter()
                        .map(|(id, _)| *id)
                        .collect::<std::collections::HashSet<_>>();
                    for (index, (id, _base_score)) in head.iter().enumerate() {
                        if !seen.contains(id) {
                            ordered.push((*id, 1.0 / (10.0 + index as f64)));
                        }
                    }
                    ordered.extend_from_slice(tail);
                    return ordered;
                }
                Ok(_) => {}
                Err(error) => {
                    warn!(error = %format_args!("{error:#}"), "sidecar rerank failed; falling back to intrinsic rerank")
                }
            }
        }
    }

    intrinsic_rerank_search_candidates(items, query, base_ranks)
}

fn intrinsic_local_rerank_score(item: &MemoryItem, query: &str) -> f64 {
    let query_terms = rerank_tokenize(query);
    if query_terms.is_empty() {
        return 0.0;
    }
    let query_term_set = query_terms
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    let query_keywords = rerank_keyword_tokens(&query_terms);
    let query_bigrams = rerank_query_bigrams(&query_terms);
    let haystack = rerank_item_haystack(item);
    let record_tokens = rerank_tokenize(&haystack);
    let name_tokens = rerank_extract_name_tokens(&item.content);
    let record_term_set = record_tokens
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    let overlap = query_term_set.intersection(&record_term_set).count() as f64;
    let lexical = overlap / query_term_set.len().max(1) as f64;
    let token_frequency =
        record_tokens
            .iter()
            .fold(std::collections::HashMap::new(), |mut acc, token| {
                *acc.entry(token.as_str()).or_insert(0usize) += 1;
                acc
            });
    let bm25ish = if query_keywords.is_empty() {
        0.0
    } else {
        query_keywords
            .iter()
            .map(|keyword| {
                let frequency = token_frequency.get(keyword.as_str()).copied().unwrap_or(0) as f64;
                if frequency == 0.0 {
                    0.0
                } else {
                    frequency / (frequency + 1.2)
                }
            })
            .sum::<f64>()
            / query_keywords.len() as f64
    };
    let semantic = rerank_cosine_similarity(
        &rerank_semantic_terms(query),
        &rerank_semantic_terms(&haystack),
    );
    let phrase_bonus = if query_terms.len() >= 2 && haystack.contains(&query_terms.join(" ")) {
        0.35
    } else {
        0.0
    };
    let keyword_bonus = if query_keywords.is_empty() {
        0.0
    } else {
        query_keywords
            .iter()
            .filter(|keyword| haystack.contains(keyword.as_str()))
            .count() as f64
            / query_keywords.len() as f64
    };
    let bigram_bonus = if query_bigrams.is_empty() {
        0.0
    } else {
        query_bigrams
            .iter()
            .filter(|bigram| haystack.contains(bigram.as_str()))
            .count() as f64
            / query_bigrams.len() as f64
    };
    let name_bonus = if query_keywords.is_empty() {
        0.0
    } else {
        query_keywords
            .iter()
            .filter(|keyword| name_tokens.contains(keyword.as_str()))
            .count() as f64
            / query_keywords.len() as f64
    };
    let path_lower = item
        .source_path
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let path_bonus = if query_keywords.is_empty() {
        0.0
    } else {
        query_keywords
            .iter()
            .filter(|keyword| path_lower.contains(keyword.as_str()))
            .count() as f64
            / query_keywords.len() as f64
    };
    let tag_bonus = if query_keywords.is_empty() {
        0.0
    } else {
        query_keywords
            .iter()
            .filter(|keyword| {
                item.tags
                    .iter()
                    .any(|tag| tag.to_ascii_lowercase().contains(keyword.as_str()))
            })
            .count() as f64
            / query_keywords.len() as f64
    };
    let recommendation_bonus = recommendation_intent_bonus(&query_terms, &haystack);
    let recommendation_mismatch_penalty =
        recommendation_intent_mismatch_penalty(&query_terms, &haystack);

    let score = lexical * 0.22
        + bm25ish * 0.18
        + semantic * 0.25
        + phrase_bonus
        + keyword_bonus * 0.20
        + bigram_bonus * 0.18
        + name_bonus * 0.08
        + path_bonus * 0.14
        + tag_bonus * 0.10
        + recommendation_bonus
        - recommendation_mismatch_penalty;
    score.clamp(0.0, 1.0)
}

fn recommendation_query_intent(query_terms: &[String]) -> bool {
    query_terms.iter().any(|term| {
        matches!(
            term.as_str(),
            "recommend"
                | "recommended"
                | "recommends"
                | "recommendation"
                | "recommendations"
                | "suggest"
                | "suggested"
                | "suggestion"
                | "suggestions"
        )
    })
}

fn recommendation_intent_bonus(query_terms: &[String], haystack: &str) -> f64 {
    if !recommendation_query_intent(query_terms) {
        return 0.0;
    }
    let mut bonus: f64 = 0.0;
    if haystack.contains("assistant recommendation turn") {
        bonus = bonus.max(0.34);
    }
    if haystack.contains("recommend") || haystack.contains("recommended") {
        bonus = bonus.max(0.30);
    }
    if haystack.contains("worth checking out") || haystack.contains("you should try") {
        bonus = bonus.max(0.24);
    }
    bonus
}

fn recommendation_intent_mismatch_penalty(query_terms: &[String], haystack: &str) -> f64 {
    if !recommendation_query_intent(query_terms) {
        return 0.0;
    }
    let has_recommendation_evidence = haystack.contains("assistant recommendation turn")
        || haystack.contains("recommend")
        || haystack.contains("worth checking out")
        || haystack.contains("you should try");
    if has_recommendation_evidence {
        return 0.0;
    }
    if haystack.contains("i'm really into")
        || haystack.contains("im really into")
        || haystack.contains("i recently read")
        || haystack.contains("i enjoyed")
        || haystack.contains("overall experience of the book")
    {
        return 0.16;
    }
    0.0
}

fn rerank_item_haystack(item: &MemoryItem) -> String {
    let mut haystack = item.content.to_ascii_lowercase();
    haystack.push(' ');
    haystack.push_str(format!("{:?}", item.kind).to_ascii_lowercase().as_str());
    haystack.push(' ');
    haystack.push_str(&item.tags.join(" ").to_ascii_lowercase());
    if let Some(path) = item.source_path.as_deref() {
        haystack.push(' ');
        haystack.push_str(&path.to_ascii_lowercase());
    }
    if let Some(agent) = item.source_agent.as_deref() {
        haystack.push(' ');
        haystack.push_str(&agent.to_ascii_lowercase());
    }
    haystack
}

fn rerank_tokenize(text: &str) -> Vec<String> {
    text.split(|ch: char| !ch.is_alphanumeric())
        .flat_map(split_identifier_token)
        .filter(|token| token.len() > 1)
        .map(|token| token.to_ascii_lowercase())
        .collect()
}

fn split_identifier_token(token: &str) -> Vec<String> {
    let chars = token.chars().collect::<Vec<_>>();
    let mut out = Vec::new();
    let mut current = String::new();
    for (idx, ch) in chars.iter().copied().enumerate() {
        let previous = idx.checked_sub(1).and_then(|prev| chars.get(prev)).copied();
        let next = chars.get(idx + 1).copied();
        let starts_new_word = previous.is_some_and(|prev| {
            (prev.is_ascii_lowercase() && ch.is_ascii_uppercase())
                || (prev.is_ascii_uppercase()
                    && ch.is_ascii_uppercase()
                    && next.is_some_and(|next| next.is_ascii_lowercase()))
                || (prev.is_ascii_alphabetic() && ch.is_ascii_digit())
                || (prev.is_ascii_digit() && ch.is_ascii_alphabetic())
        });
        if starts_new_word && !current.is_empty() {
            out.push(std::mem::take(&mut current));
        }
        current.push(ch);
    }
    if !current.is_empty() {
        out.push(current);
    }
    if out.len() <= 1 {
        vec![token.to_string()]
    } else {
        let mut expanded = Vec::with_capacity(out.len() + 1);
        expanded.push(token.to_string());
        expanded.extend(out);
        expanded
    }
}

fn rerank_keyword_tokens(query_terms: &[String]) -> Vec<String> {
    let stop_words = [
        "what", "when", "where", "who", "how", "which", "did", "do", "was", "were", "have", "has",
        "had", "is", "are", "the", "a", "an", "my", "me", "i", "you", "your", "their", "it", "its",
        "in", "on", "at", "to", "for", "of", "with", "by", "from", "ago", "last", "that", "this",
        "there", "about", "get", "got", "give", "gave", "buy", "bought", "made", "make", "said",
        "would", "could", "should", "might", "can", "will", "shall", "kind", "type", "like",
        "prefer", "enjoy", "think", "feel",
    ]
    .into_iter()
    .collect::<std::collections::BTreeSet<_>>();
    query_terms
        .iter()
        .filter(|token| token.len() >= 3 && !stop_words.contains(token.as_str()))
        .cloned()
        .collect()
}

fn rerank_query_bigrams(query_terms: &[String]) -> Vec<String> {
    query_terms
        .windows(2)
        .map(|pair| format!("{} {}", pair[0], pair[1]))
        .collect()
}

fn rerank_semantic_terms(text: &str) -> Vec<String> {
    let tokens = rerank_tokenize(text);
    let mut features = Vec::new();
    for token in &tokens {
        features.push(format!("tok:{token}"));
        if token.len() >= 4 {
            for trigram in token.as_bytes().windows(3) {
                if let Ok(fragment) = std::str::from_utf8(trigram) {
                    features.push(format!("tri:{fragment}"));
                }
            }
        }
    }
    for pair in tokens.windows(2) {
        features.push(format!("bi:{}_{}", pair[0], pair[1]));
    }
    features
}

fn rerank_cosine_similarity(left: &[String], right: &[String]) -> f64 {
    if left.is_empty() || right.is_empty() {
        return 0.0;
    }
    let left_freq = rerank_feature_frequency(left);
    let right_freq = rerank_feature_frequency(right);
    let mut dot = 0.0f64;
    for (feature, left_weight) in &left_freq {
        if let Some(right_weight) = right_freq.get(feature) {
            dot += left_weight * right_weight;
        }
    }
    if dot == 0.0 {
        return 0.0;
    }
    let left_norm = left_freq
        .values()
        .map(|weight| weight * weight)
        .sum::<f64>()
        .sqrt();
    let right_norm = right_freq
        .values()
        .map(|weight| weight * weight)
        .sum::<f64>()
        .sqrt();
    if left_norm == 0.0 || right_norm == 0.0 {
        0.0
    } else {
        dot / (left_norm * right_norm)
    }
}

fn rerank_feature_frequency(features: &[String]) -> std::collections::HashMap<&str, f64> {
    let mut frequency = std::collections::HashMap::new();
    for feature in features {
        *frequency.entry(feature.as_str()).or_insert(0.0) += 1.0;
    }
    frequency
}

fn rerank_extract_name_tokens(content: &str) -> std::collections::BTreeSet<String> {
    content
        .split(|ch: char| !ch.is_ascii_alphabetic())
        .filter(|token| {
            token.len() >= 3
                && token
                    .chars()
                    .next()
                    .is_some_and(|first| first.is_ascii_uppercase())
        })
        .map(|token| token.to_ascii_lowercase())
        .collect()
}

fn fuzzy_search_candidates(
    items: &[MemoryViewItem],
    query: &str,
    allowed_ids: Option<&std::collections::HashSet<Uuid>>,
) -> Vec<(Uuid, f64)> {
    let query = query.trim();
    if query.len() < 2 {
        return Vec::new();
    }
    let mut scored = items
        .iter()
        .filter(|entry| allowed_ids.is_none_or(|ids| ids.contains(&entry.item.id)))
        .filter_map(|entry| {
            let score = fuzzy_item_score(&entry.item, query);
            (score >= 0.18).then_some((entry.item.id, score))
        })
        .collect::<Vec<_>>();
    scored.sort_by(|a, b| b.1.total_cmp(&a.1));
    scored.truncate(200);
    scored
}

fn recommendation_search_candidates(
    items: &[MemoryViewItem],
    query: &str,
    allowed_ids: Option<&std::collections::HashSet<Uuid>>,
) -> Vec<(Uuid, f64)> {
    let query_terms = rerank_tokenize(query);
    if !recommendation_query_intent(&query_terms) {
        return Vec::new();
    }
    let mut scored = items
        .iter()
        .filter(|entry| allowed_ids.is_none_or(|ids| ids.contains(&entry.item.id)))
        .filter_map(|entry| {
            let haystack = rerank_item_haystack(&entry.item);
            let mut score: f64 = 0.0;
            if haystack.contains("assistant recommendation turn") {
                score += 1.0;
            }
            if haystack.contains("recommend") || haystack.contains("recommended") {
                score += 0.9;
            }
            if haystack.contains("worth checking out") || haystack.contains("you should try") {
                score += 0.55;
            }
            if haystack.contains("looking for a good") {
                score += 0.45;
            }
            if haystack.contains("what's so special")
                || haystack.contains("what is so special")
                || haystack.contains("book you're suggesting")
                || haystack.contains("book youre suggesting")
            {
                score -= 0.55;
            }
            if haystack.contains("i'm really into")
                || haystack.contains("im really into")
                || haystack.contains("i recently read")
                || haystack.contains("i enjoyed")
                || haystack.contains("overall experience of the book")
            {
                score -= 0.55;
            }
            (score >= 0.45).then_some((entry.item.id, score))
        })
        .collect::<Vec<_>>();
    scored.sort_by(|a, b| b.1.total_cmp(&a.1));
    scored.truncate(100);
    scored
}

fn fuzzy_item_score(item: &MemoryItem, query: &str) -> f64 {
    let query_lower = query.to_ascii_lowercase();
    let haystack = rerank_item_haystack(item);
    let query_tokens = rerank_keyword_tokens(&rerank_tokenize(&query_lower));
    let record_tokens = rerank_tokenize(&haystack);
    if query_lower.trim().len() >= 4 && item.id.to_string().starts_with(query_lower.trim()) {
        return 1.0;
    }
    if query_tokens.is_empty() || record_tokens.is_empty() {
        return 0.0;
    }

    let contains_bonus = query_tokens
        .iter()
        .filter(|token| haystack.contains(token.as_str()))
        .count() as f64
        / query_tokens.len() as f64;
    let edit_bonus = query_tokens
        .iter()
        .map(|query_token| {
            record_tokens
                .iter()
                .map(|record_token| normalized_edit_similarity(query_token, record_token))
                .fold(0.0_f64, f64::max)
        })
        .sum::<f64>()
        / query_tokens.len() as f64;
    let trigram_bonus = char_ngram_jaccard(&query_lower, &haystack, 3);
    let acronym_bonus = acronym_match_score(&query_tokens, &record_tokens);
    let path_bonus = item
        .source_path
        .as_deref()
        .map(|path| char_ngram_jaccard(&query_lower, &path.to_ascii_lowercase(), 3))
        .unwrap_or(0.0);
    let path_token_bonus = item
        .source_path
        .as_deref()
        .map(|path| path_token_match_score(&query_tokens, path))
        .unwrap_or(0.0);
    let id_bonus = if item.id.to_string().starts_with(query_lower.trim()) {
        1.0
    } else {
        0.0
    };

    (contains_bonus * 0.30
        + edit_bonus * 0.34
        + trigram_bonus * 0.18
        + acronym_bonus * 0.10
        + path_bonus * 0.04
        + path_token_bonus * 0.16
        + id_bonus * 0.12)
        .clamp(0.0, 1.0)
}

fn path_token_match_score(query_tokens: &[String], path: &str) -> f64 {
    if query_tokens.is_empty() {
        return 0.0;
    }
    let path_tokens = rerank_tokenize(path);
    if path_tokens.is_empty() {
        return 0.0;
    }
    query_tokens
        .iter()
        .map(|query_token| {
            path_tokens
                .iter()
                .map(|path_token| {
                    if path_token == query_token
                        || path_token.contains(query_token.as_str())
                        || query_token.contains(path_token.as_str())
                    {
                        1.0
                    } else {
                        normalized_edit_similarity(query_token, path_token)
                    }
                })
                .fold(0.0_f64, f64::max)
        })
        .sum::<f64>()
        / query_tokens.len() as f64
}

fn normalized_edit_similarity(left: &str, right: &str) -> f64 {
    if left == right {
        return 1.0;
    }
    if left.len() < 3 || right.len() < 3 {
        return 0.0;
    }
    let distance = levenshtein_distance(left, right) as f64;
    let max_len = left.chars().count().max(right.chars().count()) as f64;
    (1.0 - distance / max_len).clamp(0.0, 1.0)
}

fn levenshtein_distance(left: &str, right: &str) -> usize {
    let right_chars = right.chars().collect::<Vec<_>>();
    let mut prev = (0..=right_chars.len()).collect::<Vec<_>>();
    let mut curr = vec![0; right_chars.len() + 1];
    for (i, left_ch) in left.chars().enumerate() {
        curr[0] = i + 1;
        for (j, right_ch) in right_chars.iter().enumerate() {
            let cost = usize::from(left_ch != *right_ch);
            curr[j + 1] = (curr[j] + 1).min(prev[j + 1] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[right_chars.len()]
}

fn char_ngram_jaccard(left: &str, right: &str, n: usize) -> f64 {
    let left = char_ngrams(left, n);
    let right = char_ngrams(right, n);
    if left.is_empty() || right.is_empty() {
        return 0.0;
    }
    let intersection = left.intersection(&right).count() as f64;
    let union = left.union(&right).count() as f64;
    if union == 0.0 {
        0.0
    } else {
        intersection / union
    }
}

fn char_ngrams(text: &str, n: usize) -> std::collections::BTreeSet<String> {
    let chars = text
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<Vec<_>>();
    if chars.len() < n {
        return chars.into_iter().map(|ch| ch.to_string()).collect();
    }
    chars
        .windows(n)
        .map(|window| window.iter().collect::<String>())
        .collect()
}

fn acronym_match_score(query_tokens: &[String], record_tokens: &[String]) -> f64 {
    if query_tokens.is_empty() || record_tokens.len() < 2 {
        return 0.0;
    }
    let acronym = record_tokens
        .iter()
        .filter_map(|token| token.chars().next())
        .collect::<String>();
    query_tokens
        .iter()
        .filter(|token| token.len() >= 2 && acronym.contains(token.as_str()))
        .count() as f64
        / query_tokens.len() as f64
}

pub(crate) async fn healthz(
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>, (StatusCode, String)> {
    if crate::store_runtime_maintenance::expired_item_gc_enabled() {
        let _ = state.store.gc_expired_items(3600);
    }
    let items = state.store.count().map_err(internal_error)?;

    // Compute pressure metrics from store
    let all_items = state.store.list().unwrap_or_default();
    let inbox_count = all_items
        .iter()
        .filter(|i| i.stage == MemoryStage::Candidate || i.status != MemoryStatus::Active)
        .filter(|i| i.status != MemoryStatus::Expired)
        .count();
    let candidates = all_items
        .iter()
        .filter(|i| i.stage == MemoryStage::Candidate)
        .count();
    let stale = all_items
        .iter()
        .filter(|i| i.status == MemoryStatus::Stale)
        .count();
    let expired = all_items
        .iter()
        .filter(|i| i.status == MemoryStatus::Expired)
        .count();
    let active = all_items
        .iter()
        .filter(|i| i.status == MemoryStatus::Active)
        .count();
    let rag = state.rag_health_surface().await;
    let atlas = crate::status::atlas_health_surface(&state, active).map_err(internal_error)?;

    Ok(Json(HealthResponse {
        status: "ok".to_string(),
        items,
        eval_score: None,
        pressure: Some(PressureMetrics {
            inbox: inbox_count,
            candidates,
            stale,
            expired,
        }),
        rag: Some(rag),
        atlas: Some(atlas),
    }))
}

pub(crate) async fn dashboard(
    State(state): State<AppState>,
) -> Result<Html<String>, (StatusCode, String)> {
    let snapshot = ui::build_visible_memory_snapshot(&state).map_err(internal_error)?;
    Ok(Html(ui::dashboard_html(&snapshot, ui::UiPage::Home)))
}

pub(crate) async fn get_visible_memory_snapshot(
    State(state): State<AppState>,
) -> Result<Json<VisibleMemorySnapshotResponse>, (StatusCode, String)> {
    let response = ui::build_visible_memory_snapshot(&state).map_err(internal_error)?;
    Ok(Json(response))
}

#[derive(Deserialize)]
pub(crate) struct VisibleMemoryArtifactQuery {
    pub(crate) id: Uuid,
}

pub(crate) async fn get_visible_memory_artifact(
    State(state): State<AppState>,
    Query(req): Query<VisibleMemoryArtifactQuery>,
) -> Result<Json<VisibleMemoryArtifactDetailResponse>, (StatusCode, String)> {
    let response = ui::build_visible_memory_artifact_detail(&state, req.id)?;
    Ok(Json(response))
}

pub(crate) async fn post_visible_memory_action(
    State(state): State<AppState>,
    Json(req): Json<VisibleMemoryUiActionRequest>,
) -> Result<Json<VisibleMemoryUiActionResponse>, (StatusCode, String)> {
    let response = ui::perform_visible_memory_action(&state, req)?;
    Ok(Json(response))
}

pub(crate) async fn store_memory(
    State(state): State<AppState>,
    Json(req): Json<StoreMemoryRequest>,
) -> Result<Json<StoreMemoryResponse>, (StatusCode, String)> {
    if req.content.trim().is_empty() {
        return Err(MemdError::validation("content", "must not be empty").into_wire());
    }

    let (req, stage) = apply_prompt_injection_firewall(req, MemoryStage::Canonical);
    let item = state.store_item(req, stage).map_err(internal_error)?;
    let (item, duplicate) = item;
    Ok(Json(StoreMemoryResponse {
        item: duplicate.map_or(item, |found| found.item),
    }))
}

pub(crate) async fn store_candidate(
    State(state): State<AppState>,
    Json(req): Json<CandidateMemoryRequest>,
) -> Result<Json<CandidateMemoryResponse>, (StatusCode, String)> {
    if req.content.trim().is_empty() {
        return Err(MemdError::validation("content", "must not be empty").into_wire());
    }

    let store_req = StoreMemoryRequest {
        content: req.content,
        kind: req.kind,
        scope: req.scope,
        project: req.project,
        namespace: req.namespace,
        workspace: req.workspace,
        visibility: req.visibility,
        belief_branch: req.belief_branch,
        source_agent: req.source_agent,
        source_system: req.source_system,
        source_path: req.source_path,
        source_quality: req.source_quality,
        confidence: req.confidence,
        ttl_seconds: req.ttl_seconds,
        last_verified_at: req.last_verified_at,
        supersedes: req.supersedes,
        tags: req.tags,
        status: Some(MemoryStatus::Active),
        lane: req.lane,
    };

    let (store_req, stage) = apply_prompt_injection_firewall(store_req, MemoryStage::Candidate);
    let (item, duplicate) = state.store_item(store_req, stage).map_err(internal_error)?;
    Ok(Json(CandidateMemoryResponse {
        item: duplicate.as_ref().map_or(item, |found| found.item.clone()),
        duplicate_of: duplicate.map(|found| found.id),
    }))
}

pub(crate) async fn promote_memory(
    State(state): State<AppState>,
    Json(req): Json<PromoteMemoryRequest>,
) -> Result<Json<PromoteMemoryResponse>, (StatusCode, String)> {
    let (item, duplicate) = state.promote_item(req).map_err(internal_error)?;
    Ok(Json(PromoteMemoryResponse {
        item: duplicate.as_ref().map_or(item, |found| found.item.clone()),
        duplicate_of: duplicate.map(|found| found.id),
    }))
}

pub(crate) async fn expire_memory(
    State(state): State<AppState>,
    Json(req): Json<ExpireMemoryRequest>,
) -> Result<Json<ExpireMemoryResponse>, (StatusCode, String)> {
    let item = repair::expire_item(&state, req)?;
    Ok(Json(ExpireMemoryResponse { item }))
}

pub(crate) async fn verify_memory(
    State(state): State<AppState>,
    Json(req): Json<VerifyMemoryRequest>,
) -> Result<Json<VerifyMemoryResponse>, (StatusCode, String)> {
    let item = repair::verify_item(&state, req)?;
    Ok(Json(VerifyMemoryResponse { item }))
}

pub(crate) async fn repair_memory(
    State(state): State<AppState>,
    Json(req): Json<RepairMemoryRequest>,
) -> Result<Json<RepairMemoryResponse>, (StatusCode, String)> {
    let response = repair::repair_item(&state, req)?;
    Ok(Json(response))
}

pub(crate) async fn correct_memory(
    State(state): State<AppState>,
    Json(req): Json<CorrectMemoryRequest>,
) -> Result<Json<CorrectMemoryResponse>, (StatusCode, String)> {
    let response = repair::correct_item(&state, req)?;
    Ok(Json(response))
}

pub(crate) async fn get_working_memory(
    State(state): State<AppState>,
    Query(req): Query<WorkingMemoryRequest>,
) -> Result<Json<WorkingMemoryResponse>, (StatusCode, String)> {
    // K2.6: record wall-clock ms into the process-global histogram so
    // /api/diagnostics/latency and HarnessStatus.latency_p95_ms reflect
    // the true tail of working-memory retrieval.
    let started = std::time::Instant::now();
    let result = working::working_memory(&state, req);
    state
        .latency
        .record_ms(started.elapsed().as_millis() as u64);
    let response = result?;
    Ok(Json(response))
}

pub(crate) async fn get_explain(
    State(state): State<AppState>,
    Query(req): Query<ExplainMemoryRequest>,
) -> Result<Json<ExplainMemoryResponse>, (StatusCode, String)> {
    let plan = RetrievalPlan::resolve(req.route, req.intent);
    let response = inspection::explain_memory(&state, req)?;
    state
        .record_retrieval_feedback(
            std::slice::from_ref(&response.item),
            1,
            "retrieved_explain",
            &plan,
        )
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn get_memory_policy() -> Json<MemoryPolicyResponse> {
    Json(working::memory_policy_snapshot())
}

#[derive(Deserialize)]
pub(crate) struct GetSearchQuery {
    pub(crate) query: Option<String>,
    pub(crate) tag: Option<String>,
    pub(crate) kind: Option<MemoryKind>,
    pub(crate) status: Option<MemoryStatus>,
    pub(crate) stage: Option<MemoryStage>,
    pub(crate) project: Option<String>,
    pub(crate) namespace: Option<String>,
    pub(crate) workspace: Option<String>,
    pub(crate) limit: Option<usize>,
}

// K2.4: GET surface for ergonomic curl/dashboard search.
// Delegates to the same filter pipeline as POST /memory/search so ranking,
// FTS, and retrieval-feedback stay consistent between the two entry points.
pub(crate) async fn search_memory_get(
    state: State<AppState>,
    Query(q): Query<GetSearchQuery>,
) -> Result<Json<SearchMemoryResponse>, (StatusCode, String)> {
    let req = SearchMemoryRequest {
        query: q.query,
        tags: q.tag.into_iter().collect(),
        kinds: q.kind.into_iter().collect(),
        statuses: q.status.into_iter().collect(),
        stages: q.stage.into_iter().collect(),
        project: q.project,
        namespace: q.namespace,
        workspace: q.workspace,
        limit: q.limit,
        ..SearchMemoryRequest::default()
    };
    search_memory(state, Json(req)).await
}

pub(crate) async fn search_memory(
    State(state): State<AppState>,
    Json(req): Json<SearchMemoryRequest>,
) -> Result<Json<SearchMemoryResponse>, (StatusCode, String)> {
    let policy = working::memory_policy_snapshot();
    let feedback_limit = policy.retrieval_feedback.max_items_per_request;
    // B3-part2 prereq: scope snapshot to project+namespace when set so
    // per-question bench namespaces don't trigger a full-corpus scan.
    let snapshot = state
        .snapshot_for_scope(req.project.as_deref(), req.namespace.as_deref())
        .map_err(internal_error)?;
    let region_member_ids = resolve_region_member_filter(
        &state,
        req.region.as_deref(),
        req.project.as_deref(),
        req.namespace.as_deref(),
    )
    .map_err(internal_error)?;
    let mut items = enrich_with_entities(&state, snapshot).map_err(internal_error)?;
    if let Some(allowed_ids) = region_member_ids.as_ref() {
        items.retain(|entry| allowed_ids.contains(&entry.item.id));
    }
    let plan = RetrievalPlan::resolve(req.route, req.intent);
    // B3-T2: sanitize + atlas-synonym expand before FTS.
    let mut fts_ranks = req
        .query
        .as_ref()
        .and_then(|q| {
            let sanitized = crate::query_sanitize::sanitize_query(q);
            let aliases = state
                .store
                .entity_aliases_for_query(&sanitized.clean, 4)
                .unwrap_or_default();
            let fts_expr = crate::query_sanitize::build_fts_match(&sanitized.clean, &aliases);
            state.store.fts_search(&fts_expr, 100).ok()
        })
        .unwrap_or_default();
    if let Some(allowed_ids) = region_member_ids.as_ref() {
        fts_ranks.retain(|(id, _)| allowed_ids.contains(id));
    }
    let fts_direct_ranks = fts_ranks.clone();
    let mut rank_lanes = vec![SearchRankLane::new(
        "fts_bm25",
        1.25,
        fts_direct_ranks.clone(),
    )];
    if let Some(query_text) = req
        .query
        .as_deref()
        .map(str::trim)
        .filter(|q| !q.is_empty())
    {
        rank_lanes.push(SearchRankLane::new(
            "fuzzy",
            0.95,
            fuzzy_search_candidates(&items, query_text, region_member_ids.as_ref()),
        ));
        let recommendation_ranks =
            recommendation_search_candidates(&items, query_text, region_member_ids.as_ref());
        if !recommendation_ranks.is_empty() {
            rank_lanes.push(SearchRankLane::new(
                "recommendation",
                2.4,
                recommendation_ranks,
            ));
        }
    }

    // B3-T6: atlas-at-recall 1-hop entity expansion (item-space). Top K
    // FTS hits seed a 1-hop neighbor lookup; neighbors are injected into
    // fts_ranks with a small score bonus so they survive filter_items.
    // Default on; opt out with MEMD_RETRIEVAL_ATLAS_RECALL=0.
    if atlas_recall_enabled() && !fts_direct_ranks.is_empty() {
        let seed_k = fts_direct_ranks.len().min(5);
        let seeds: Vec<uuid::Uuid> = fts_direct_ranks
            .iter()
            .take(seed_k)
            .map(|(id, _)| *id)
            .collect();
        let neighbors = state.store.one_hop_neighbors_for_items(&seeds, 10);
        if !neighbors.is_empty() {
            // Bonus is small (0.15) so direct FTS matches retain priority.
            let tail_score = fts_direct_ranks
                .last()
                .map(|(_, s)| *s * 0.5)
                .unwrap_or(0.15)
                .max(0.15);
            let existing: std::collections::HashSet<uuid::Uuid> =
                fts_direct_ranks.iter().map(|(id, _)| *id).collect();
            let atlas_ranks = neighbors
                .into_iter()
                .filter(|nid| !existing.contains(nid))
                .filter(|nid| {
                    region_member_ids
                        .as_ref()
                        .is_none_or(|ids| ids.contains(nid))
                })
                .map(|nid| (nid, tail_score))
                .collect::<Vec<_>>();
            rank_lanes.push(SearchRankLane::new("atlas", 0.35, atlas_ranks));
        }
    }
    if state.rag.is_some() && rag_dense_enabled() {
        match state.rag_dense_candidates(&req).await {
            Ok(dense) if !dense.is_empty() => {
                let dense_ranks = dense
                    .into_iter()
                    .filter(|(id, _)| {
                        region_member_ids
                            .as_ref()
                            .is_none_or(|ids| ids.contains(id))
                    })
                    .collect::<Vec<_>>();
                if let Some(head) = dense_ranks.first().copied() {
                    rank_lanes.push(SearchRankLane::new("rag_dense_head", 4.0, vec![head]));
                }
                rank_lanes.push(SearchRankLane::new("rag_dense", 3.2, dense_ranks));
            }
            Ok(_) => {}
            Err(error) => warn!(error = %format_args!("{error:#}"), "rag dense retrieval failed"),
        }
    }

    // B3-Part2: intrinsic dense blend (in-process hashed embeddings). Enabled
    // by default and opt-out via MEMD_INTRINSIC_DENSE=0. Scopes the vector scan to the same project+
    // namespace slice as `snapshot_for_scope`, so per-question bench
    // namespaces never trigger a global-corpus vector scan. Dense scores
    // replace fts scores rather than tail-append, so purely-semantic matches
    // (zero lexical overlap) surface into the top-K.
    if let Some(embedder) = state.embedder.as_deref()
        && crate::embed::intrinsic_dense_enabled()
        && let Some(query_text) = req
            .query
            .as_deref()
            .map(str::trim)
            .filter(|q| !q.is_empty())
    {
        match embedder.embed_query_normalized(query_text) {
            Ok(q_vec) => {
                match state.store.list_vectors_for_scope(
                    req.project.as_deref(),
                    req.namespace.as_deref(),
                    embedder.model_code(),
                ) {
                    Ok(candidates) if !candidates.is_empty() => {
                        // Per-chunk scoring; one memory_id can appear multiple
                        // times if content was chunked on store. Group by
                        // memory_id taking MAX so a single strong chunk
                        // lifts the session it belongs to.
                        let mut by_id: std::collections::HashMap<Uuid, f64> =
                            std::collections::HashMap::with_capacity(candidates.len());
                        for (id, bytes) in candidates {
                            let v = crate::embed::bytes_to_vec(&bytes);
                            let score = crate::embed::cosine_on_unit(&q_vec, &v) as f64;
                            let entry = by_id.entry(id).or_insert(f64::NEG_INFINITY);
                            if score > *entry {
                                *entry = score;
                            }
                        }
                        let mut scored: Vec<(Uuid, f64)> = by_id.into_iter().collect();
                        scored.sort_by(|a, b| {
                            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
                        });
                        scored.truncate(200);
                        let mut dense_ranks = scored
                            .into_iter()
                            .filter(|(id, _)| {
                                region_member_ids
                                    .as_ref()
                                    .is_none_or(|ids| ids.contains(id))
                            })
                            .collect::<Vec<_>>();
                        dense_ranks.sort_by(|a, b| {
                            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
                        });
                        rank_lanes.push(SearchRankLane::new("intrinsic_dense", 1.0, dense_ranks));
                    }
                    Ok(_) => {}
                    Err(error) => {
                        warn!(error = %format_args!("{error:#}"), "list_vectors_for_scope failed")
                    }
                }
            }
            Err(error) => warn!(error = %format_args!("{error:#}"), "query embed failed"),
        }
    }
    fts_ranks = fuse_search_rank_lanes(&rank_lanes);
    let truth_ranks = truth_guard_search_candidates(&items, &fts_ranks);
    if !truth_ranks.is_empty() {
        rank_lanes.push(SearchRankLane::new("truth", 1.4, truth_ranks));
        fts_ranks = fuse_search_rank_lanes(&rank_lanes);
    }
    if intrinsic_rerank_enabled()
        && let Some(query_text) = req
            .query
            .as_deref()
            .map(str::trim)
            .filter(|q| !q.is_empty())
        && fts_ranks.len() > 1
    {
        let rerank_ranks = rerank_search_candidates(&state, &items, query_text, &fts_ranks).await;
        rank_lanes.push(SearchRankLane::new("rerank", 1.0, rerank_ranks.clone()));
        let final_truth_ranks = truth_guard_search_candidates(&items, &rerank_ranks);
        let final_recommendation_ranks =
            recommendation_search_candidates(&items, query_text, region_member_ids.as_ref());
        fts_ranks = fuse_search_rank_lanes(&[
            SearchRankLane::new("rerank", 8.0, rerank_ranks),
            SearchRankLane::new("truth", 3.0, final_truth_ranks),
            SearchRankLane::new("recommendation", 6.0, final_recommendation_ranks),
        ]);
    }
    let items = filter_items(&items, &req, &plan, &fts_ranks);
    let trace = build_search_trace(req.query.clone(), &rank_lanes, &fts_ranks, &items);
    state
        .rehearse_items(&items, feedback_limit)
        .map_err(internal_error)?;
    state
        .record_retrieval_feedback(&items, feedback_limit, "retrieved_search", &plan)
        .map_err(internal_error)?;
    Ok(Json(SearchMemoryResponse {
        route: plan.route,
        intent: plan.intent,
        items,
        trace,
    }))
}


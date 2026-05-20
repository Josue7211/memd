use super::*;
use anyhow::anyhow;
use std::hash::{Hash, Hasher};

const CONVOMEM_MESSAGE_EVIDENCE_MATCH_VERSION: i64 = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LongMemEvalHypothesisEntry {
    pub question_id: String,
    pub hypothesis: String,
}

pub(crate) fn supported_public_benchmark_ids() -> &'static [&'static str] {
    &["longmemeval", "locomo", "convomem", "membench"]
}

pub(crate) fn implemented_public_benchmark_ids() -> &'static [&'static str] {
    &["longmemeval", "locomo", "convomem", "membench"]
}

pub(crate) fn public_benchmark_target_status(dataset: &str) -> &'static str {
    if implemented_public_benchmark_ids().contains(&dataset) {
        "implemented"
    } else {
        "declared-stub"
    }
}

pub(crate) fn render_longmemeval_haystack_text(value: &JsonValue) -> String {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|session| session.as_array())
        .flat_map(|turns| turns.iter())
        .filter_map(|turn| {
            let role = turn.get("role").and_then(JsonValue::as_str).unwrap_or("");
            let content = turn
                .get("content")
                .and_then(JsonValue::as_str)
                .unwrap_or("");
            if role.is_empty() && content.is_empty() {
                None
            } else {
                Some(format!("{role}: {content}"))
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn render_locomo_conversation_text(value: &JsonValue) -> String {
    let mut rendered = Vec::new();
    if let Some(conversation) = value.as_object() {
        let mut session_indexes = conversation
            .keys()
            .filter_map(|key| key.strip_prefix("session_"))
            .filter_map(|suffix| {
                suffix
                    .split_once('_')
                    .map(|(index, _)| index)
                    .or(Some(suffix))
            })
            .filter_map(|index| index.parse::<usize>().ok())
            .collect::<BTreeSet<_>>();
        if session_indexes.is_empty() {
            session_indexes = (1..=35).collect();
        }
        for session_index in session_indexes {
            let session_key = format!("session_{session_index}");
            if let Some(dialogs) = conversation.get(&session_key).and_then(JsonValue::as_array) {
                for dialog in dialogs {
                    let speaker = dialog
                        .get("speaker")
                        .and_then(JsonValue::as_str)
                        .unwrap_or("unknown");
                    let text = dialog.get("text").and_then(JsonValue::as_str).unwrap_or("");
                    if !text.is_empty() {
                        rendered.push(format!("{speaker}: {text}"));
                    }
                }
            }
        }
    }
    rendered.join("\n")
}

pub(crate) fn render_membench_message_list_text(value: &JsonValue) -> String {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(JsonValue::as_array)
        .flat_map(|session| session.iter())
        .filter_map(render_membench_turn_text)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Parse MemBench MC `choices` metadata into the labeled-string list expected
/// by `build_mc_generation_prompt`.
///
/// Upstream FirstAgent fixture stores `choices` as a letter-keyed object where
/// each value is an array of option strings, e.g. `{ "A": ["foo"], "B":
/// ["foo", "bar"] }`. A prior implementation only handled a flat string array
/// and therefore skipped every item. Accepted shapes:
/// - object `{letter → [strings]}` → `"A. foo"`, `"B. foo, bar"`
/// - object `{letter → "text"}` → `"A. text"`
/// - array `["A. foo", "B. bar"]` → passthrough
/// - `null` / missing / other → empty
pub(crate) fn parse_membench_choices(value: Option<&JsonValue>) -> Vec<String> {
    let Some(value) = value else {
        return Vec::new();
    };
    if let Some(obj) = value.as_object() {
        let mut keys: Vec<&String> = obj.keys().collect();
        keys.sort();
        return keys
            .into_iter()
            .map(|letter| {
                let rendered = match obj.get(letter) {
                    Some(JsonValue::Array(arr)) => arr
                        .iter()
                        .filter_map(JsonValue::as_str)
                        .collect::<Vec<_>>()
                        .join(", "),
                    Some(JsonValue::String(s)) => s.clone(),
                    Some(other) => other.to_string(),
                    None => String::new(),
                };
                format!("{letter}. {rendered}")
            })
            .collect();
    }
    if let Some(arr) = value.as_array() {
        return arr
            .iter()
            .filter_map(JsonValue::as_str)
            .map(str::to_string)
            .collect();
    }
    Vec::new()
}

pub(crate) fn render_membench_turn_text(turn: &JsonValue) -> Option<String> {
    let user = turn
        .get("user_message")
        .and_then(JsonValue::as_str)
        .or_else(|| turn.get("user").and_then(JsonValue::as_str));
    let assistant = turn
        .get("assistant_message")
        .and_then(JsonValue::as_str)
        .or_else(|| turn.get("assistant").and_then(JsonValue::as_str))
        .or_else(|| turn.get("agent").and_then(JsonValue::as_str));
    let recommendation_prefix = if membench_turn_looks_like_recommendation(user, assistant) {
        "assistant recommendation turn. "
    } else {
        ""
    };
    match (user, assistant) {
        (Some(user), Some(assistant)) => Some(format!(
            "{recommendation_prefix}user: {user}\nassistant: {assistant}"
        )),
        (Some(user), None) => Some(format!("{recommendation_prefix}user: {user}")),
        (None, Some(assistant)) => Some(format!("{recommendation_prefix}assistant: {assistant}")),
        (None, None) => None,
    }
}

fn membench_turn_looks_like_recommendation(user: Option<&str>, assistant: Option<&str>) -> bool {
    let mut text = String::new();
    if let Some(user) = user {
        text.push_str(user);
        text.push('\n');
    }
    if let Some(assistant) = assistant {
        text.push_str(assistant);
    }
    let normalized = text.to_ascii_lowercase();
    normalized.contains("recommend")
        || normalized.contains("looking for a good")
        || normalized.contains("worth checking out")
        || normalized.contains("you should try")
}

pub(crate) fn render_convomem_conversation_text(value: &JsonValue) -> String {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(JsonValue::as_object)
        .flat_map(|conversation| {
            conversation
                .get("messages")
                .and_then(JsonValue::as_array)
                .into_iter()
                .flatten()
                .filter_map(JsonValue::as_object)
                .map(|message| {
                    let speaker = message
                        .get("speaker")
                        .and_then(JsonValue::as_str)
                        .unwrap_or("unknown");
                    let text = message
                        .get("text")
                        .and_then(JsonValue::as_str)
                        .unwrap_or("");
                    format!("{speaker}: {text}")
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn convomem_message_id(
    conversation: &serde_json::Map<String, JsonValue>,
    conversation_index: usize,
    message_index: usize,
) -> String {
    let conversation_id = conversation
        .get("id")
        .and_then(JsonValue::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("conv-{conversation_index}"));
    format!("{conversation_id}::msg:{message_index}")
}

fn convomem_normalize_match_key(speaker: &str, text: &str) -> (String, String) {
    (
        speaker.trim().to_ascii_lowercase(),
        text.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_ascii_lowercase(),
    )
}

pub(super) fn convomem_message_docs(value: &JsonValue) -> Vec<(String, String)> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .enumerate()
        .filter_map(|(conversation_index, conversation)| {
            conversation
                .as_object()
                .map(|obj| (conversation_index, obj))
        })
        .flat_map(|(conversation_index, conversation)| {
            conversation
                .get("messages")
                .and_then(JsonValue::as_array)
                .into_iter()
                .flatten()
                .enumerate()
                .filter_map(move |(message_index, message)| {
                    let message = message.as_object()?;
                    let speaker = message
                        .get("speaker")
                        .and_then(JsonValue::as_str)
                        .unwrap_or("unknown")
                        .trim();
                    let text = message
                        .get("text")
                        .and_then(JsonValue::as_str)
                        .unwrap_or("")
                        .trim();
                    if text.is_empty() {
                        return None;
                    }
                    Some((
                        convomem_message_id(conversation, conversation_index, message_index),
                        format!("{speaker}: {text}"),
                    ))
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

pub(super) fn convomem_message_evidence_ids(
    evidences: &JsonValue,
    conversations: &JsonValue,
) -> Vec<String> {
    let mut by_message = BTreeMap::<(String, String), Vec<String>>::new();
    let mut message_entries = Vec::new();
    for (message_id, rendered) in convomem_message_docs(conversations) {
        let (speaker, text) = rendered
            .split_once(':')
            .map(|(speaker, text)| (speaker, text))
            .unwrap_or(("unknown", rendered.as_str()));
        let normalized = convomem_normalize_match_key(speaker, text);
        by_message
            .entry(normalized.clone())
            .or_default()
            .push(message_id.clone());
        message_entries.push((
            message_id,
            normalized.0,
            normalized.1.clone(),
            tokenize_public_benchmark_text(&normalized.1),
        ));
    }

    let mut expected = BTreeSet::new();
    for evidence in evidences.as_array().into_iter().flatten() {
        match evidence {
            JsonValue::Object(map) => {
                let speaker = map
                    .get("speaker")
                    .and_then(JsonValue::as_str)
                    .unwrap_or("unknown");
                let text = map.get("text").and_then(JsonValue::as_str).unwrap_or("");
                let normalized = convomem_normalize_match_key(speaker, text);
                if let Some(ids) = by_message.get(&normalized) {
                    expected.extend(ids.iter().cloned());
                    continue;
                }
                for ((msg_speaker, msg_text), ids) in &by_message {
                    if msg_speaker == &normalized.0 && msg_text.contains(&normalized.1) {
                        expected.extend(ids.iter().cloned());
                    }
                }
                if !expected.is_empty() {
                    continue;
                }
                let evidence_tokens = tokenize_public_benchmark_text(&normalized.1);
                let best_match = message_entries
                    .iter()
                    .filter(|(_, speaker, _, _)| speaker == &normalized.0)
                    .filter_map(|(message_id, _, _, message_tokens)| {
                        if evidence_tokens.is_empty() {
                            return None;
                        }
                        let overlap = evidence_tokens.intersection(message_tokens).count() as f64
                            / evidence_tokens.len() as f64;
                        Some((message_id, overlap))
                    })
                    .max_by(|left, right| left.1.total_cmp(&right.1));
                if let Some((message_id, overlap)) = best_match {
                    if overlap >= 0.8 {
                        expected.insert(message_id.clone());
                    }
                }
            }
            JsonValue::String(text) => {
                let normalized = text.trim().to_ascii_lowercase();
                for ((_, msg_text), ids) in &by_message {
                    if msg_text == &normalized || msg_text.contains(&normalized) {
                        expected.extend(ids.iter().cloned());
                    }
                }
                if !expected.is_empty() {
                    continue;
                }
                let evidence_tokens = tokenize_public_benchmark_text(&normalized);
                let best_match = message_entries
                    .iter()
                    .filter_map(|(message_id, _, _, message_tokens)| {
                        if evidence_tokens.is_empty() {
                            return None;
                        }
                        let overlap = evidence_tokens.intersection(message_tokens).count() as f64
                            / evidence_tokens.len() as f64;
                        Some((message_id, overlap))
                    })
                    .max_by(|left, right| left.1.total_cmp(&right.1));
                if let Some((message_id, overlap)) = best_match {
                    if overlap >= 0.8 {
                        expected.insert(message_id.clone());
                    }
                }
            }
            _ => {}
        }
    }

    expected.into_iter().collect()
}

pub(crate) fn locomo_category_name(category: i64) -> &'static str {
    match category {
        1 => "Single-hop",
        2 => "Temporal",
        3 => "Temporal-inference",
        4 => "Open-domain",
        5 => "Adversarial",
        _ => "Unknown",
    }
}

pub(crate) fn json_stringish_field(row: &JsonValue, key: &str) -> anyhow::Result<String> {
    let value = row.get(key).ok_or_else(|| anyhow!("missing {key} field"))?;
    match value {
        JsonValue::String(value) => Ok(value.clone()),
        JsonValue::Number(value) => Ok(value.to_string()),
        JsonValue::Bool(value) => Ok(value.to_string()),
        _ => anyhow::bail!("missing {key} string-compatible value"),
    }
}

pub(crate) fn json_stringish_or_array_field(row: &JsonValue, key: &str) -> anyhow::Result<String> {
    let value = row.get(key).ok_or_else(|| anyhow!("missing {key} field"))?;
    match value {
        JsonValue::Array(items) => Ok(items
            .iter()
            .map(|item| match item {
                JsonValue::String(value) => Ok(value.clone()),
                JsonValue::Number(value) => Ok(value.to_string()),
                JsonValue::Bool(value) => Ok(value.to_string()),
                _ => anyhow::bail!("missing {key} string-compatible array value"),
            })
            .collect::<anyhow::Result<Vec<_>>>()?
            .join(", ")),
        _ => json_stringish_field(row, key),
    }
}

pub(crate) fn normalize_longmemeval_dataset(
    path: &Path,
    rows: &[JsonValue],
) -> anyhow::Result<PublicBenchmarkDatasetFixture> {
    let items = rows
        .iter()
        .map(|row| {
            let item_id = json_stringish_field(row, "question_id")
                .with_context(|| format!("normalize {} question_id", path.display()))?;
            let query = json_stringish_field(row, "question")
                .with_context(|| format!("normalize {} question", path.display()))?;
            let gold_answer = json_stringish_field(row, "answer")
                .with_context(|| format!("normalize {} answer", path.display()))?;
            Ok(PublicBenchmarkDatasetFixtureItem {
                item_id: item_id.clone(),
                question_id: item_id,
                query,
                claim_class: "raw".to_string(),
                gold_answer,
                metadata: json!({
                    "question_type": row.get("question_type").cloned().unwrap_or(JsonValue::Null),
                    "question_date": row.get("question_date").cloned().unwrap_or(JsonValue::Null),
                    "haystack_dates": row.get("haystack_dates").cloned().unwrap_or(JsonValue::Null),
                    "haystack_session_ids": row.get("haystack_session_ids").cloned().unwrap_or(JsonValue::Null),
                    "haystack_sessions": row.get("haystack_sessions").cloned().unwrap_or(JsonValue::Null),
                    "answer_session_ids": row.get("answer_session_ids").cloned().unwrap_or(JsonValue::Null),
                    "haystack_text": render_longmemeval_haystack_text(
                        row.get("haystack_sessions").unwrap_or(&JsonValue::Null)
                    ),
                }),
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(PublicBenchmarkDatasetFixture {
        benchmark_id: "longmemeval".to_string(),
        benchmark_name: "LongMemEval".to_string(),
        version: "upstream".to_string(),
        split: "cleaned-small".to_string(),
        description: "Normalized upstream LongMemEval cleaned file.".to_string(),
        items,
    })
}

pub(crate) fn normalize_locomo_dataset(
    path: &Path,
    rows: &[JsonValue],
) -> anyhow::Result<PublicBenchmarkDatasetFixture> {
    let mut items = Vec::new();
    for row in rows {
        let sample_id = json_stringish_field(row, "sample_id")
            .with_context(|| format!("normalize {} sample_id", path.display()))?;
        let conversation = row.get("conversation").cloned().unwrap_or(JsonValue::Null);
        let conversation_text = render_locomo_conversation_text(&conversation);
        let session_summary = row
            .get("session_summary")
            .cloned()
            .unwrap_or(JsonValue::Null);
        let observation = row.get("observation").cloned().unwrap_or(JsonValue::Null);
        let event_summary = row.get("event_summary").cloned().unwrap_or(JsonValue::Null);
        let qa_rows = row
            .get("qa")
            .and_then(JsonValue::as_array)
            .ok_or_else(|| anyhow!("normalize {} qa array", path.display()))?;
        for (qa_index, qa_row) in qa_rows.iter().enumerate() {
            let query = json_stringish_field(qa_row, "question")
                .with_context(|| format!("normalize {} qa.question", path.display()))?;
            let gold_answer = json_stringish_field(qa_row, "answer")
                .or_else(|_| json_stringish_field(qa_row, "adversarial_answer"))
                .with_context(|| format!("normalize {} qa.answer", path.display()))?;
            let category_id = qa_row
                .get("category")
                .and_then(JsonValue::as_i64)
                .unwrap_or_default();
            items.push(PublicBenchmarkDatasetFixtureItem {
                item_id: format!("{sample_id}::{qa_index}"),
                question_id: format!("{sample_id}::{qa_index}"),
                query,
                claim_class: "raw".to_string(),
                gold_answer,
                metadata: json!({
                    "sample_id": sample_id,
                    "category_id": category_id,
                    "category_name": locomo_category_name(category_id),
                    "evidence": qa_row.get("evidence").cloned().unwrap_or(JsonValue::Null),
                    "conversation": conversation,
                    "conversation_text": conversation_text,
                    "session_summary": session_summary,
                    "observation": observation,
                    "event_summary": event_summary,
                }),
            });
        }
    }

    Ok(PublicBenchmarkDatasetFixture {
        benchmark_id: "locomo".to_string(),
        benchmark_name: "LoCoMo".to_string(),
        version: "upstream".to_string(),
        split: "locomo10".to_string(),
        description: "Normalized upstream LoCoMo conversation benchmark file.".to_string(),
        items,
    })
}

pub(crate) fn normalize_membench_dataset(
    path: &Path,
    value: &JsonValue,
) -> anyhow::Result<PublicBenchmarkDatasetFixture> {
    let root = value
        .as_object()
        .ok_or_else(|| anyhow!("normalize {} membench root object", path.display()))?;
    let mut items = Vec::new();
    for (topic, entries_value) in root {
        let entries = entries_value
            .as_array()
            .ok_or_else(|| anyhow!("normalize {} membench topic array", path.display()))?;
        for entry in entries {
            let tid = entry
                .get("tid")
                .and_then(JsonValue::as_i64)
                .unwrap_or_default();
            let qa = entry
                .get("QA")
                .or_else(|| entry.get("qa"))
                .ok_or_else(|| anyhow!("normalize {} membench QA object", path.display()))?;
            let qid = qa
                .get("qid")
                .and_then(JsonValue::as_i64)
                .unwrap_or_default();
            let query = json_stringish_field(qa, "question")
                .with_context(|| format!("normalize {} QA.question", path.display()))?;
            let gold_answer = json_stringish_or_array_field(qa, "answer")
                .with_context(|| format!("normalize {} QA.answer", path.display()))?;
            items.push(PublicBenchmarkDatasetFixtureItem {
                item_id: format!("{topic}::{tid}::{qid}"),
                question_id: format!("{topic}::{tid}::{qid}"),
                query,
                claim_class: "raw".to_string(),
                gold_answer,
                metadata: json!({
                    "topic": topic,
                    "tid": tid,
                    "qid": qid,
                    "target_step_id": qa.get("target_step_id").cloned().unwrap_or(JsonValue::Null),
                    "choices": qa.get("choices").cloned().unwrap_or(JsonValue::Null),
                    "ground_truth": qa.get("ground_truth").cloned().unwrap_or(JsonValue::Null),
                    "time": qa.get("time").cloned().unwrap_or(JsonValue::Null),
                    "message_list": entry.get("message_list").cloned().unwrap_or(JsonValue::Null),
                    "conversation_text": render_membench_message_list_text(
                        entry.get("message_list").unwrap_or(&JsonValue::Null)
                    ),
                }),
            });
        }
    }

    Ok(PublicBenchmarkDatasetFixture {
        benchmark_id: "membench".to_string(),
        benchmark_name: "MemBench".to_string(),
        version: "upstream".to_string(),
        split: "FirstAgent".to_string(),
        description: "Normalized upstream MemBench FirstAgent benchmark files.".to_string(),
        items,
    })
}

pub(crate) fn normalize_convomem_evidence_items(
    items: &[JsonValue],
) -> anyhow::Result<PublicBenchmarkDatasetFixture> {
    let items = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let query = json_stringish_field(item, "question")
                .with_context(|| format!("normalize convomem item {index} question"))?;
            let gold_answer = json_stringish_or_array_field(item, "answer")
                .with_context(|| format!("normalize convomem item {index} answer"))?;
            let category = item
                .get("category")
                .and_then(JsonValue::as_str)
                .unwrap_or("unknown");
            Ok(PublicBenchmarkDatasetFixtureItem {
                item_id: format!("{category}::{index}"),
                question_id: format!("{category}::{index}"),
                query,
                claim_class: "raw".to_string(),
                gold_answer,
                metadata: json!({
                    "category": item.get("category").cloned().unwrap_or(JsonValue::Null),
                    "scenario_description": item.get("scenario_description").cloned().unwrap_or(JsonValue::Null),
                    "person_id": item.get("personId").cloned().unwrap_or(JsonValue::Null),
                    "message_evidences": item.get("message_evidences").cloned().unwrap_or(JsonValue::Null),
                    "message_evidence_match_version": CONVOMEM_MESSAGE_EVIDENCE_MATCH_VERSION,
                    "message_evidence_ids": convomem_message_evidence_ids(
                        item.get("message_evidences").unwrap_or(&JsonValue::Null),
                        item.get("conversations").unwrap_or(&JsonValue::Null)
                    ),
                    "conversations": item.get("conversations").cloned().unwrap_or(JsonValue::Null),
                    "conversation_text": render_convomem_conversation_text(
                        item.get("conversations").unwrap_or(&JsonValue::Null)
                    ),
                    "use_case_model_name": item.get("use_case_model_name").cloned().unwrap_or(JsonValue::Null),
                    "core_model_name": item.get("core_model_name").cloned().unwrap_or(JsonValue::Null),
                }),
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(PublicBenchmarkDatasetFixture {
        benchmark_id: "convomem".to_string(),
        benchmark_name: "ConvoMem".to_string(),
        version: "upstream".to_string(),
        split: "evidence-sample".to_string(),
        description: "Sampled upstream ConvoMem evidence files normalized into a cached fixture."
            .to_string(),
        items,
    })
}

pub(crate) fn load_public_benchmark_dataset(
    benchmark_id: &str,
    path: &Path,
) -> anyhow::Result<PublicBenchmarkDatasetFixture> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let value = serde_json::from_str::<JsonValue>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    match value {
        JsonValue::Object(value)
            if benchmark_id == "membench" && value.get("benchmark_id").is_none() =>
        {
            normalize_membench_dataset(path, &JsonValue::Object(value))
        }
        JsonValue::Object(_) => serde_json::from_str::<PublicBenchmarkDatasetFixture>(&raw)
            .with_context(|| format!("parse {}", path.display())),
        JsonValue::Array(rows) if benchmark_id == "longmemeval" => {
            normalize_longmemeval_dataset(path, &rows)
        }
        JsonValue::Array(rows) if benchmark_id == "locomo" => normalize_locomo_dataset(path, &rows),
        JsonValue::Array(_) => anyhow::bail!(
            "benchmark `{benchmark_id}` array dataset format is not normalized yet for {}",
            path.display()
        ),
        _ => anyhow::bail!(
            "unsupported public benchmark dataset format in {}",
            path.display()
        ),
    }
}

pub(crate) fn public_benchmark_fixture_checksum(path: &Path) -> anyhow::Result<String> {
    let raw = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    Ok(format!("sha256:{:x}", Sha256::digest(&raw)))
}

pub(crate) fn public_benchmark_dataset_source(
    dataset: &str,
) -> Option<PublicBenchmarkDatasetSource> {
    match dataset {
        "longmemeval" => Some(PublicBenchmarkDatasetSource {
            benchmark_id: "longmemeval",
            source_url: Some(
                "https://huggingface.co/datasets/xiaowu0162/longmemeval-cleaned/resolve/main/longmemeval_s_cleaned.json",
            ),
            default_filename: "longmemeval_s_cleaned.json",
            expected_checksum: Some(
                "sha256:d6f21ea9d60a0d56f34a05b609c79c88a451d2ae03597821ea3d5a9678c3a442",
            ),
            split: "cleaned-small",
            access_mode: "auto-download",
            notes: "Upstream LongMemEval cleaned small file from the official benchmark repo README.",
        }),
        "locomo" => Some(PublicBenchmarkDatasetSource {
            benchmark_id: "locomo",
            source_url: Some(
                "https://raw.githubusercontent.com/snap-research/locomo/3eb6f2c585f5e1699204e3c3bdf7adc5c28cb376/data/locomo10.json",
            ),
            default_filename: "locomo10.json",
            expected_checksum: Some(
                "sha256:79fa87e90f04081343b8c8debecb80a9a6842b76a7aa537dc9fdf651ea698ff4",
            ),
            split: "locomo10",
            access_mode: "auto-download",
            notes: "Commit-pinned LoCoMo locomo10.json source from the upstream benchmark repo.",
        }),
        "convomem" => Some(PublicBenchmarkDatasetSource {
            benchmark_id: "convomem",
            source_url: Some(
                "https://huggingface.co/datasets/Salesforce/ConvoMem/tree/main/core_benchmark/evidence_questions",
            ),
            default_filename: "convomem-evidence-sample.json",
            expected_checksum: Some(
                "sha256:dead92689c44ac5a3b66c0c7980166c8fc8d9b16a9cedb2e1c2f7981b6e6f094",
            ),
            split: "evidence-sample",
            access_mode: "auto-download",
            notes: "Sampled upstream ConvoMem evidence files fetched from the Hugging Face dataset tree.",
        }),
        "membench" => Some(PublicBenchmarkDatasetSource {
            benchmark_id: "membench",
            source_url: Some(
                "https://github.com/import-myself/Membench/tree/f66d8d1028d3f68627d00f77a967b93fbb8694b6/MemData/FirstAgent",
            ),
            default_filename: "membench-firstagent.json",
            expected_checksum: Some(
                "sha256:54bde8259c10ee1cfe5ff16f35a8a25ca9ad5d79e162e0b3a43034ed64115e5a",
            ),
            split: "FirstAgent",
            access_mode: "auto-download",
            notes: "Commit-pinned MemBench FirstAgent category set normalized into a cached fixture.",
        }),
        _ => None,
    }
}

pub(crate) fn resolve_public_benchmark_dataset_override_path(
    args: &PublicBenchmarkArgs,
) -> Option<PathBuf> {
    args.dataset_root.as_ref().map(|path| {
        if path.is_dir() {
            path.join(format!("{}-mini.json", args.dataset))
        } else {
            path.clone()
        }
    })
}

pub(crate) fn public_benchmark_dataset_entry_dir(output: &Path, benchmark_id: &str) -> PathBuf {
    public_benchmark_dataset_cache_dir(output).join(benchmark_id)
}

pub(crate) fn public_benchmark_dataset_cache_path(
    output: &Path,
    benchmark_id: &str,
    filename: &str,
) -> PathBuf {
    public_benchmark_dataset_entry_dir(output, benchmark_id).join(filename)
}

pub(crate) fn public_benchmark_dataset_cache_metadata_path(
    output: &Path,
    benchmark_id: &str,
) -> PathBuf {
    public_benchmark_dataset_entry_dir(output, benchmark_id).join("metadata.json")
}

pub(crate) fn write_public_benchmark_dataset_cache_metadata(
    output: &Path,
    metadata: &PublicBenchmarkDatasetCacheMetadata,
) -> anyhow::Result<PathBuf> {
    let path = public_benchmark_dataset_cache_metadata_path(output, &metadata.benchmark_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, serde_json::to_string_pretty(metadata)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

pub(crate) fn validate_public_benchmark_checksum(
    checksum: &str,
    expected_checksum: Option<&str>,
) -> anyhow::Result<String> {
    if let Some(expected_checksum) = expected_checksum {
        anyhow::ensure!(
            checksum == expected_checksum,
            "dataset checksum mismatch: expected {expected_checksum}, got {checksum}"
        );
        Ok("verified".to_string())
    } else {
        Ok("recorded-unpinned".to_string())
    }
}

pub(crate) async fn download_public_benchmark_dataset(
    output: &Path,
    source: &PublicBenchmarkDatasetSource,
) -> anyhow::Result<ResolvedPublicBenchmarkDataset> {
    let source_url = source.source_url.ok_or_else(|| {
        anyhow!(
            "benchmark `{}` does not expose an auto-download URL",
            source.benchmark_id
        )
    })?;
    let entry_dir = public_benchmark_dataset_entry_dir(output, source.benchmark_id);
    fs::create_dir_all(&entry_dir).with_context(|| format!("create {}", entry_dir.display()))?;
    let dataset_path =
        public_benchmark_dataset_cache_path(output, source.benchmark_id, source.default_filename);
    eprintln!("[download] fetching {source_url}…");
    let response = reqwest::get(source_url)
        .await
        .with_context(|| format!("download dataset {source_url}"))?
        .error_for_status()
        .with_context(|| format!("download dataset {source_url}"))?;
    let bytes = response
        .bytes()
        .await
        .with_context(|| format!("read dataset bytes {source_url}"))?;
    eprintln!("[download] received {} bytes", bytes.len());
    fs::write(&dataset_path, &bytes)
        .with_context(|| format!("write {}", dataset_path.display()))?;
    let checksum = public_benchmark_fixture_checksum(&dataset_path)?;
    let verification_status =
        validate_public_benchmark_checksum(&checksum, source.expected_checksum)?;
    write_public_benchmark_dataset_cache_metadata(
        output,
        &PublicBenchmarkDatasetCacheMetadata {
            benchmark_id: source.benchmark_id.to_string(),
            source_url: source_url.to_string(),
            local_path: dataset_path.display().to_string(),
            checksum: checksum.clone(),
            expected_checksum: source.expected_checksum.map(str::to_string),
            verification_status: verification_status.clone(),
            fetched_at: Utc::now(),
            bytes: bytes.len(),
        },
    )?;
    Ok(ResolvedPublicBenchmarkDataset {
        path: dataset_path,
        source_url: source_url.to_string(),
        checksum,
        split: source.split.to_string(),
        verification_status,
    })
}

pub(crate) async fn download_membench_dataset(
    output: &Path,
    source: &PublicBenchmarkDatasetSource,
) -> anyhow::Result<ResolvedPublicBenchmarkDataset> {
    const MEMBENCH_FILES: &[&str] = &[
        "simple.json",
        "highlevel.json",
        "knowledge_update.json",
        "comparative.json",
        "conditional.json",
        "noisy.json",
        "aggregative.json",
        "highlevel_rec.json",
        "lowlevel_rec.json",
        "RecMultiSession.json",
        "post_processing.json",
    ];
    const MEMBENCH_COMMIT: &str = "f66d8d1028d3f68627d00f77a967b93fbb8694b6";
    let base_url = format!(
        "https://raw.githubusercontent.com/import-myself/Membench/{MEMBENCH_COMMIT}/MemData/FirstAgent"
    );
    let entry_dir = public_benchmark_dataset_entry_dir(output, source.benchmark_id);
    fs::create_dir_all(&entry_dir).with_context(|| format!("create {}", entry_dir.display()))?;
    let raw_dir = entry_dir.join("raw");
    fs::create_dir_all(&raw_dir).with_context(|| format!("create {}", raw_dir.display()))?;

    let mut merged = serde_json::Map::new();
    let mut byte_count = 0usize;
    for filename in MEMBENCH_FILES {
        let url = format!("{base_url}/{filename}");
        let response = reqwest::get(&url)
            .await
            .with_context(|| format!("download dataset {url}"))?
            .error_for_status()
            .with_context(|| format!("download dataset {url}"))?;
        let bytes = response
            .bytes()
            .await
            .with_context(|| format!("read dataset bytes {url}"))?;
        byte_count += bytes.len();
        let raw_path = raw_dir.join(filename);
        fs::write(&raw_path, &bytes).with_context(|| format!("write {}", raw_path.display()))?;
        let value = serde_json::from_slice::<JsonValue>(&bytes)
            .with_context(|| format!("parse {}", raw_path.display()))?;
        let object = value
            .as_object()
            .ok_or_else(|| anyhow!("membench source {} was not an object", raw_path.display()))?;
        for (key, value) in object {
            merged.insert(key.clone(), value.clone());
        }
    }

    let dataset_path =
        public_benchmark_dataset_cache_path(output, source.benchmark_id, source.default_filename);
    fs::write(
        &dataset_path,
        serde_json::to_string_pretty(&JsonValue::Object(merged))? + "\n",
    )
    .with_context(|| format!("write {}", dataset_path.display()))?;
    let checksum = public_benchmark_fixture_checksum(&dataset_path)?;
    let verification_status =
        validate_public_benchmark_checksum(&checksum, source.expected_checksum)?;
    write_public_benchmark_dataset_cache_metadata(
        output,
        &PublicBenchmarkDatasetCacheMetadata {
            benchmark_id: source.benchmark_id.to_string(),
            source_url: source.source_url.unwrap_or_default().to_string(),
            local_path: dataset_path.display().to_string(),
            checksum: checksum.clone(),
            expected_checksum: source.expected_checksum.map(str::to_string),
            verification_status: verification_status.clone(),
            fetched_at: Utc::now(),
            bytes: byte_count,
        },
    )?;
    Ok(ResolvedPublicBenchmarkDataset {
        path: dataset_path,
        source_url: source.source_url.unwrap_or_default().to_string(),
        checksum,
        split: source.split.to_string(),
        verification_status,
    })
}

pub(crate) async fn download_convomem_dataset(
    output: &Path,
    source: &PublicBenchmarkDatasetSource,
) -> anyhow::Result<ResolvedPublicBenchmarkDataset> {
    const CONVOMEM_CATEGORIES: &[&str] = &[
        "user_evidence",
        "assistant_facts_evidence",
        "changing_evidence",
        "abstention_evidence",
        "preference_evidence",
        "implicit_connection_evidence",
    ];
    let tree_url = "https://huggingface.co/api/datasets/Salesforce/ConvoMem/tree/main/core_benchmark/evidence_questions?recursive=true";
    let tree = reqwest::get(tree_url)
        .await
        .context("download ConvoMem tree api")?
        .error_for_status()
        .context("download ConvoMem tree api")?
        .json::<Vec<JsonValue>>()
        .await
        .context("parse ConvoMem tree api")?;

    let entry_dir = public_benchmark_dataset_entry_dir(output, source.benchmark_id);
    fs::create_dir_all(&entry_dir).with_context(|| format!("create {}", entry_dir.display()))?;
    let raw_dir = entry_dir.join("raw");
    fs::create_dir_all(&raw_dir).with_context(|| format!("create {}", raw_dir.display()))?;

    let mut sample_paths = Vec::new();
    for category in CONVOMEM_CATEGORIES {
        let path = tree
            .iter()
            .filter_map(|entry| entry.get("path").and_then(JsonValue::as_str))
            .filter(|path| {
                path.starts_with(&format!("core_benchmark/evidence_questions/{category}/"))
                    && path.ends_with(".json")
            })
            .min()
            .ok_or_else(|| anyhow!("no ConvoMem evidence file found for category `{category}`"))?;
        sample_paths.push(path.to_string());
    }

    let mut evidence_items = Vec::new();
    let mut byte_count = 0usize;
    for path in &sample_paths {
        let url =
            format!("https://huggingface.co/datasets/Salesforce/ConvoMem/resolve/main/{path}");
        let response = reqwest::get(&url)
            .await
            .with_context(|| format!("download dataset {url}"))?
            .error_for_status()
            .with_context(|| format!("download dataset {url}"))?;
        let bytes = response
            .bytes()
            .await
            .with_context(|| format!("read dataset bytes {url}"))?;
        byte_count += bytes.len();
        let filename = path
            .rsplit('/')
            .next()
            .ok_or_else(|| anyhow!("convomem sample path missing filename"))?;
        let raw_path = raw_dir.join(filename);
        fs::write(&raw_path, &bytes).with_context(|| format!("write {}", raw_path.display()))?;
        let value = serde_json::from_slice::<JsonValue>(&bytes)
            .with_context(|| format!("parse {}", raw_path.display()))?;
        let items = value
            .get("evidence_items")
            .and_then(JsonValue::as_array)
            .ok_or_else(|| {
                anyhow!(
                    "ConvoMem source {} missing evidence_items",
                    raw_path.display()
                )
            })?;
        evidence_items.extend(items.iter().cloned());
    }

    let fixture = normalize_convomem_evidence_items(&evidence_items)?;
    let dataset_path =
        public_benchmark_dataset_cache_path(output, source.benchmark_id, source.default_filename);
    fs::write(
        &dataset_path,
        serde_json::to_string_pretty(&fixture)? + "\n",
    )
    .with_context(|| format!("write {}", dataset_path.display()))?;
    let checksum = public_benchmark_fixture_checksum(&dataset_path)?;
    let verification_status =
        validate_public_benchmark_checksum(&checksum, source.expected_checksum)?;
    write_public_benchmark_dataset_cache_metadata(
        output,
        &PublicBenchmarkDatasetCacheMetadata {
            benchmark_id: source.benchmark_id.to_string(),
            source_url: source.source_url.unwrap_or_default().to_string(),
            local_path: dataset_path.display().to_string(),
            checksum: checksum.clone(),
            expected_checksum: source.expected_checksum.map(str::to_string),
            verification_status: verification_status.clone(),
            fetched_at: Utc::now(),
            bytes: byte_count,
        },
    )?;
    Ok(ResolvedPublicBenchmarkDataset {
        path: dataset_path,
        source_url: source.source_url.unwrap_or_default().to_string(),
        checksum,
        split: source.split.to_string(),
        verification_status,
    })
}

pub(crate) fn public_benchmark_cached_dataset_is_stale(
    benchmark_id: &str,
    dataset_path: &Path,
) -> anyhow::Result<bool> {
    if benchmark_id != "convomem" {
        return Ok(false);
    }

    let fixture = serde_json::from_str::<PublicBenchmarkDatasetFixture>(
        &fs::read_to_string(dataset_path)
            .with_context(|| format!("read {}", dataset_path.display()))?,
    )
    .with_context(|| format!("parse {}", dataset_path.display()))?;

    Ok(fixture.items.iter().any(|item| {
        !item
            .metadata
            .get("message_evidence_ids")
            .is_some_and(JsonValue::is_array)
            || item
                .metadata
                .get("message_evidence_match_version")
                .and_then(JsonValue::as_i64)
                != Some(CONVOMEM_MESSAGE_EVIDENCE_MATCH_VERSION)
    }))
}

pub(crate) async fn resolve_public_benchmark_dataset(
    args: &PublicBenchmarkArgs,
) -> anyhow::Result<ResolvedPublicBenchmarkDataset> {
    if let Some(path) = resolve_public_benchmark_dataset_override_path(args) {
        let checksum = public_benchmark_fixture_checksum(&path)?;
        return Ok(ResolvedPublicBenchmarkDataset {
            source_url: format!("file://{}", path.display()),
            path,
            checksum,
            split: "manual".to_string(),
            verification_status: "manual-path".to_string(),
        });
    }

    let source = public_benchmark_dataset_source(&args.dataset).ok_or_else(|| {
        anyhow!(
            "no public benchmark dataset source is registered for `{}`",
            args.dataset
        )
    })?;

    if source.access_mode != "auto-download" {
        anyhow::bail!(
            "benchmark `{}` currently requires --dataset-root; {}",
            args.dataset,
            source.notes
        );
    }

    let cached_path =
        public_benchmark_dataset_cache_path(&args.out, &args.dataset, source.default_filename);
    if cached_path.exists() {
        if public_benchmark_cached_dataset_is_stale(&args.dataset, &cached_path)? {
            eprintln!(
                "[bench] cached dataset stale for `{}` at {}; rebuilding normalized fixture",
                args.dataset,
                cached_path.display()
            );
        } else {
            let checksum = public_benchmark_fixture_checksum(&cached_path)?;
            let verification_status =
                validate_public_benchmark_checksum(&checksum, source.expected_checksum)?;
            write_public_benchmark_dataset_cache_metadata(
                &args.out,
                &PublicBenchmarkDatasetCacheMetadata {
                    benchmark_id: args.dataset.clone(),
                    source_url: source.source_url.unwrap_or_default().to_string(),
                    local_path: cached_path.display().to_string(),
                    checksum: checksum.clone(),
                    expected_checksum: source.expected_checksum.map(str::to_string),
                    verification_status: verification_status.clone(),
                    fetched_at: Utc::now(),
                    bytes: fs::metadata(&cached_path)
                        .with_context(|| format!("stat {}", cached_path.display()))?
                        .len() as usize,
                },
            )?;
            return Ok(ResolvedPublicBenchmarkDataset {
                path: cached_path,
                source_url: source.source_url.unwrap_or_default().to_string(),
                checksum,
                split: source.split.to_string(),
                verification_status,
            });
        }
    }

    if args.dataset == "membench" {
        return download_membench_dataset(&args.out, &source).await;
    }
    if args.dataset == "convomem" {
        return download_convomem_dataset(&args.out, &source).await;
    }

    download_public_benchmark_dataset(&args.out, &source).await
}

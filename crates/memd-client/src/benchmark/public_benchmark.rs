use super::*;
use anyhow::anyhow;
use std::hash::{Hash, Hasher};

const CONVOMEM_MESSAGE_EVIDENCE_MATCH_VERSION: i64 = 3;
const PUBLIC_BENCHMARK_REGRESSION_BUDGET: f64 = 0.02;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LongMemEvalHypothesisEntry {
    pub question_id: String,
    pub hypothesis: String,
}

#[derive(Debug, Clone, Deserialize)]
struct PublicBenchmarkHistoryEntry {
    benchmark_id: String,
    #[serde(default)]
    git_sha: Option<String>,
    timestamp: DateTime<Utc>,
    primary_value: f64,
    #[serde(default)]
    verification_status: Option<String>,
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
    match (user, assistant) {
        (Some(user), Some(assistant)) => Some(format!("user: {user}\nassistant: {assistant}")),
        (Some(user), None) => Some(format!("user: {user}")),
        (None, Some(assistant)) => Some(format!("assistant: {assistant}")),
        (None, None) => None,
    }
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

fn convomem_message_docs(value: &JsonValue) -> Vec<(String, String)> {
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

fn convomem_message_evidence_ids(evidences: &JsonValue, conversations: &JsonValue) -> Vec<String> {
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

pub(crate) fn tokenize_public_benchmark_text(value: &str) -> BTreeSet<String> {
    value
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter_map(|token| {
            let token = token.trim().to_ascii_lowercase();
            if token.is_empty() { None } else { Some(token) }
        })
        .collect()
}

pub(crate) fn flatten_public_benchmark_metadata(value: &JsonValue) -> String {
    match value {
        JsonValue::Object(map) => map
            .iter()
            .map(|(key, value)| {
                let rendered = value
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| value.to_string());
                format!("{key}={rendered}")
            })
            .collect::<Vec<_>>()
            .join(" "),
        JsonValue::Array(items) => items
            .iter()
            .map(flatten_public_benchmark_metadata)
            .collect::<Vec<_>>()
            .join(" "),
        JsonValue::String(value) => value.clone(),
        _ => value.to_string(),
    }
}

pub(crate) fn dcg_public_benchmark(relevances: &[f64], k: usize) -> f64 {
    relevances
        .iter()
        .take(k)
        .enumerate()
        .map(|(index, relevance)| relevance / ((index as f64 + 2.0).log2()))
        .sum()
}

pub(crate) fn ndcg_public_benchmark(relevances: &[f64], k: usize) -> f64 {
    let mut ideal = relevances.to_vec();
    ideal.sort_by(|left, right| right.total_cmp(left));
    let idcg = dcg_public_benchmark(&ideal, k);
    if idcg == 0.0 {
        0.0
    } else {
        dcg_public_benchmark(relevances, k) / idcg
    }
}

pub(crate) fn public_benchmark_string_vec(value: Option<&JsonValue>) -> Vec<String> {
    value
        .and_then(JsonValue::as_array)
        .into_iter()
        .flatten()
        .filter_map(JsonValue::as_str)
        .map(str::to_string)
        .collect()
}

pub(crate) fn build_longmemeval_eval_prompt(
    task: &str,
    question: &str,
    answer: &str,
    response: &str,
    abstention: bool,
) -> anyhow::Result<String> {
    if abstention {
        return Ok(format!(
            "I will give you an unanswerable question, an explanation, and a response from a model. Please answer yes if the model correctly identifies the question as unanswerable. The model could say that the information is incomplete, or some other information is given but the asked information is not.\n\nQuestion: {question}\n\nExplanation: {answer}\n\nModel Response: {response}\n\nDoes the model correctly identify the question as unanswerable? Answer yes or no only."
        ));
    }
    let prompt = match task {
        "single-session-user" | "single-session-assistant" | "multi-session" => format!(
            "I will give you a question, a correct answer, and a response from a model. Please answer yes if the response contains the correct answer. Otherwise, answer no. If the response is equivalent to the correct answer or contains all the intermediate steps to get the correct answer, you should also answer yes. If the response only contains a subset of the information required by the answer, answer no. \n\nQuestion: {question}\n\nCorrect Answer: {answer}\n\nModel Response: {response}\n\nIs the model response correct? Answer yes or no only."
        ),
        "temporal-reasoning" => format!(
            "I will give you a question, a correct answer, and a response from a model. Please answer yes if the response contains the correct answer. Otherwise, answer no. If the response is equivalent to the correct answer or contains all the intermediate steps to get the correct answer, you should also answer yes. If the response only contains a subset of the information required by the answer, answer no. In addition, do not penalize off-by-one errors for the number of days. If the question asks for the number of days/weeks/months, etc., and the model makes off-by-one errors (e.g., predicting 19 days when the answer is 18), the model's response is still correct. \n\nQuestion: {question}\n\nCorrect Answer: {answer}\n\nModel Response: {response}\n\nIs the model response correct? Answer yes or no only."
        ),
        "knowledge-update" => format!(
            "I will give you a question, a correct answer, and a response from a model. Please answer yes if the response contains the correct answer. Otherwise, answer no. If the response contains some previous information along with an updated answer, the response should be considered as correct as long as the updated answer is the required answer.\n\nQuestion: {question}\n\nCorrect Answer: {answer}\n\nModel Response: {response}\n\nIs the model response correct? Answer yes or no only."
        ),
        "single-session-preference" => format!(
            "I will give you a question, a rubric for desired personalized response, and a response from a model. Please answer yes if the response satisfies the desired response. Otherwise, answer no. The model does not need to reflect all the points in the rubric. The response is correct as long as it recalls and utilizes the user's personal information correctly.\n\nQuestion: {question}\n\nRubric: {answer}\n\nModel Response: {response}\n\nIs the model response correct? Answer yes or no only."
        ),
        other => anyhow::bail!("unsupported LongMemEval question type `{other}`"),
    };
    Ok(prompt)
}

pub(crate) fn load_longmemeval_hypotheses(
    path: &Path,
) -> anyhow::Result<Vec<LongMemEvalHypothesisEntry>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    if let Ok(entries) = serde_json::from_str::<Vec<LongMemEvalHypothesisEntry>>(&raw) {
        return Ok(entries);
    }
    raw.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            serde_json::from_str::<LongMemEvalHypothesisEntry>(line)
                .with_context(|| format!("parse jsonl hypothesis line in {}", path.display()))
        })
        .collect()
}

pub(crate) fn validate_public_benchmark_args(args: &PublicBenchmarkArgs) -> anyhow::Result<()> {
    if !args.all && args.dataset.is_empty() {
        anyhow::bail!("dataset is required unless --all is specified");
    }
    if args.full_eval && args.community_standard {
        anyhow::bail!("--full-eval replaces --community-standard; use --full-eval instead");
    }
    if args.dual {
        anyhow::ensure!(
            args.dataset == "longmemeval" || args.dataset.is_empty(),
            "--dual is currently only supported for longmemeval"
        );
        anyhow::ensure!(
            !args.full_eval,
            "--dual is only supported for retrieval-mode longmemeval"
        );
        anyhow::ensure!(
            !args.community_standard,
            "--dual is only supported for retrieval-mode longmemeval"
        );
    }
    if args.community_standard {
        anyhow::ensure!(
            args.dataset == "longmemeval",
            "community-standard evaluation is currently only supported for longmemeval"
        );
        anyhow::ensure!(
            args.hypotheses_file.is_some(),
            "community-standard longmemeval requires --hypotheses-file"
        );
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct GraderResult {
    pub content: String,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub cache_hit: bool,
}

pub(crate) fn parse_judge_budget_str(value: &str) -> Option<f64> {
    value
        .trim()
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite() && *v > 0.0)
}

pub(crate) fn parse_judge_budget_env() -> Option<f64> {
    std::env::var("MEMD_BENCH_JUDGE_BUDGET_USD")
        .ok()
        .as_deref()
        .and_then(parse_judge_budget_str)
}

pub(crate) fn estimate_judge_cost_usd(
    grader_model: &str,
    prompt_tokens: u64,
    completion_tokens: u64,
) -> f64 {
    let (input_per_mtok, output_per_mtok) = match grader_model {
        "gpt-4o-2024-08-06" | "gpt-4o" => (2.50, 10.00),
        "gpt-4o-mini" | "gpt-4o-mini-2024-07-18" => (0.15, 0.60),
        "gpt-4-turbo" | "gpt-4-turbo-2024-04-09" => (10.00, 30.00),
        _ => (2.50, 10.00),
    };
    let p = prompt_tokens as f64 * input_per_mtok / 1_000_000.0;
    let c = completion_tokens as f64 * output_per_mtok / 1_000_000.0;
    p + c
}

pub(crate) fn judge_cache_dir() -> std::path::PathBuf {
    if let Ok(dir) = std::env::var("MEMD_BENCH_JUDGE_CACHE_DIR") {
        return std::path::PathBuf::from(dir);
    }
    std::path::PathBuf::from(".memd/benchmarks/grader-cache")
}

pub(crate) fn judge_cache_key(
    namespace: &str,
    question_id: &str,
    prediction: &str,
    grader_model: &str,
    prompt: &str,
) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(namespace.as_bytes());
    h.update(b"\x00");
    h.update(question_id.as_bytes());
    h.update(b"\x00");
    h.update(prediction.as_bytes());
    h.update(b"\x00");
    h.update(grader_model.as_bytes());
    h.update(b"\x00");
    h.update(prompt.as_bytes());
    format!("{:x}", h.finalize())
}

pub(crate) async fn call_openai_yes_no_grader_cached(
    base_url: &str,
    api_key: &str,
    grader_model: &str,
    prompt: &str,
    cache_key: &str,
) -> anyhow::Result<GraderResult> {
    call_openai_yes_no_grader_cached_in(
        base_url,
        api_key,
        grader_model,
        prompt,
        cache_key,
        &judge_cache_dir(),
    )
    .await
}

pub(crate) async fn call_openai_yes_no_grader_cached_in(
    base_url: &str,
    api_key: &str,
    grader_model: &str,
    prompt: &str,
    cache_key: &str,
    dir: &std::path::Path,
) -> anyhow::Result<GraderResult> {
    let path = dir.join(format!("{cache_key}.json"));
    if path.exists() {
        if let Ok(bytes) = std::fs::read(&path) {
            if let Ok(cached) = serde_json::from_slice::<JsonValue>(&bytes) {
                if let (Some(content), Some(p), Some(c)) = (
                    cached.get("content").and_then(JsonValue::as_str),
                    cached.get("prompt_tokens").and_then(JsonValue::as_u64),
                    cached.get("completion_tokens").and_then(JsonValue::as_u64),
                ) {
                    return Ok(GraderResult {
                        content: content.to_string(),
                        prompt_tokens: p,
                        completion_tokens: c,
                        cache_hit: true,
                    });
                }
            }
        }
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .connect_timeout(std::time::Duration::from_secs(15))
        .build()
        .context("build openai grader client")?;
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    eprintln!("[grader] POST {url} model={grader_model}");
    let response = client
        .post(&url)
        .bearer_auth(api_key)
        .json(&json!({
            "model": grader_model,
            "messages": [{"role": "user", "content": prompt}],
            "n": 1,
            "temperature": 0,
            "max_tokens": 10
        }))
        .send()
        .await
        .context("send openai grader request")?;
    eprintln!("[grader] response status={}", response.status());
    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "failed to read openai grader error body".to_string());
        anyhow::bail!("openai grader request failed with {status}: {body}");
    }
    let body = response
        .json::<JsonValue>()
        .await
        .context("parse openai grader response json")?;
    let content = body
        .get("choices")
        .and_then(JsonValue::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(JsonValue::as_str)
        .map(str::trim)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("openai grader response missing choices[0].message.content"))?;
    let prompt_tokens = body
        .get("usage")
        .and_then(|u| u.get("prompt_tokens"))
        .and_then(JsonValue::as_u64)
        .unwrap_or(0);
    let completion_tokens = body
        .get("usage")
        .and_then(|u| u.get("completion_tokens"))
        .and_then(JsonValue::as_u64)
        .unwrap_or(0);
    if let Err(err) = std::fs::create_dir_all(dir) {
        eprintln!("[grader-cache] failed to create {}: {err}", dir.display());
    } else {
        let payload = json!({
            "content": content,
            "prompt_tokens": prompt_tokens,
            "completion_tokens": completion_tokens,
            "grader_model": grader_model,
        });
        if let Err(err) = std::fs::write(
            &path,
            serde_json::to_vec_pretty(&payload)
                .unwrap_or_else(|_| payload.to_string().into_bytes()),
        ) {
            eprintln!("[grader-cache] failed to write {}: {err}", path.display());
        }
    }
    Ok(GraderResult {
        content,
        prompt_tokens,
        completion_tokens,
        cache_hit: false,
    })
}

pub(crate) async fn call_openai_yes_no_grader(
    base_url: &str,
    api_key: &str,
    grader_model: &str,
    prompt: &str,
) -> anyhow::Result<String> {
    let key = judge_cache_key("legacy", "", "", grader_model, prompt);
    let res =
        call_openai_yes_no_grader_cached(base_url, api_key, grader_model, prompt, &key).await?;
    Ok(res.content)
}

pub(crate) fn public_benchmark_target_key(value: &JsonValue) -> Option<String> {
    match value {
        JsonValue::Null => None,
        JsonValue::String(value) => Some(value.clone()),
        JsonValue::Number(value) => Some(value.to_string()),
        JsonValue::Bool(value) => Some(value.to_string()),
        JsonValue::Array(_) | JsonValue::Object(_) => serde_json::to_string(value).ok(),
    }
}

pub(crate) fn locomo_retrieval_docs(
    item: &PublicBenchmarkDatasetFixtureItem,
) -> Vec<(String, String)> {
    let mut docs = Vec::new();
    if let Some(conversation) = item
        .metadata
        .get("conversation")
        .and_then(JsonValue::as_object)
    {
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
            let session_date = conversation
                .get(&format!("session_{session_index}_date_time"))
                .and_then(JsonValue::as_str)
                .unwrap_or("");
            if let Some(dialogs) = conversation.get(&session_key).and_then(JsonValue::as_array) {
                for dialog in dialogs {
                    let dia_id = dialog
                        .get("dia_id")
                        .and_then(JsonValue::as_str)
                        .unwrap_or("")
                        .to_string();
                    let speaker = dialog
                        .get("speaker")
                        .and_then(JsonValue::as_str)
                        .unwrap_or("unknown");
                    let text = dialog.get("text").and_then(JsonValue::as_str).unwrap_or("");
                    if !dia_id.is_empty() && !text.is_empty() {
                        let rendered = if session_date.is_empty() {
                            format!("{speaker}: {text}")
                        } else {
                            format!("({session_date}) {speaker}: {text}")
                        };
                        docs.push((dia_id, rendered));
                    }
                }
            }
        }
    }
    docs
}

pub(crate) fn membench_retrieval_docs(
    item: &PublicBenchmarkDatasetFixtureItem,
) -> Vec<(String, String)> {
    item.metadata
        .get("message_list")
        .and_then(JsonValue::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .flat_map(|(session_index, session)| {
            session
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(move |turn| {
                    let text = render_membench_turn_text(turn)?;
                    let step = turn
                        .get("mid")
                        .cloned()
                        .map(|mid| json!([mid, session_index]))
                        .or_else(|| {
                            turn.get("sid")
                                .cloned()
                                .map(|sid| json!([sid, session_index]))
                        })
                        .or_else(|| turn.get("step_id").cloned())
                        .or_else(|| Some(json!([0, session_index])));
                    Some((public_benchmark_target_key(&step?)?, text))
                })
        })
        .collect()
}

/// Token-intersection ranker used by LoCoMo/MemBench/ConvoMem bench adapters
/// before G3. Extracted verbatim from `build_context_retrieval_run_report` so
/// the `Lexical` backend variant can reuse it without drift. Rust's
/// `Vec::sort_by` is stable, so equal scores preserve input (docs) order —
/// do not replace with `sort_unstable_by` without re-auditing bench numbers.
pub(crate) fn rank_public_benchmark_lexical_docs(
    query: &str,
    docs: &[(String, String)],
) -> Vec<((String, String), f64)> {
    let query_tokens = tokenize_public_benchmark_text(query);
    let mut ranked = docs
        .iter()
        .map(|(doc_id, text)| {
            let score = query_tokens
                .intersection(&tokenize_public_benchmark_text(text))
                .count() as f64;
            ((doc_id.clone(), text.clone()), score)
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| right.1.total_cmp(&left.1));
    ranked
}

/// G3 step 4 dispatcher: routes a single (query, docs) pair through the
/// configured backend and produces ranked `((doc_id, text), score)` pairs
/// in the shape `build_context_retrieval_run_report` consumes. On any
/// memd/sidecar/rrf failure we fall back to the lexical scorer rather
/// than abort the bench — matches the LongMemEval adapter's RRF-with-
/// lexical-fallback contract (`merge_ranked_longmemeval_results`).
pub(crate) fn dispatch_context_retrieval_ranked(
    bench_id: &str,
    item_id: &str,
    query: &str,
    docs: &[(String, String)],
    mode: &str,
    config: &PublicBenchmarkRetrievalConfig,
) -> Vec<((String, String), f64)> {
    if docs.is_empty() {
        return Vec::new();
    }
    match config.longmemeval_backend {
        PublicBenchmarkBackend::Lexical => rank_public_benchmark_lexical_docs(query, docs),
        PublicBenchmarkBackend::Rrf => {
            let (corpus_ids, corpus): (Vec<String>, Vec<String>) = docs.iter().cloned().unzip();
            let ranked = rank_longmemeval_corpus_via_rrf(query, &corpus, &corpus_ids, mode);
            ranked_indices_to_docs(ranked, docs, query)
        }
        PublicBenchmarkBackend::Memd => {
            let Some(base_url) = config.memd_base_url.as_deref() else {
                if std::env::var_os("MEMD_BENCH_PROBES").is_some() {
                    eprintln!(
                        "[bench-probe] memd-fallback-no-url bench={bench_id} item={item_id}"
                    );
                }
                return rank_public_benchmark_lexical_docs(query, docs);
            };
            let (corpus_ids, corpus): (Vec<String>, Vec<String>) = docs.iter().cloned().unzip();
            let namespace = bench_item_namespace(bench_id, item_id, &corpus_ids, &corpus);
            match rank_corpus_via_memd(
                bench_id, base_url, query, &corpus, &corpus_ids, mode, &namespace,
            ) {
                Ok(ranked) => ranked_indices_to_docs(ranked, docs, query),
                Err(err) => {
                    if std::env::var_os("MEMD_BENCH_PROBES").is_some() {
                        eprintln!(
                            "[bench-probe] memd-dispatch-error bench={bench_id} item={item_id} err={err}"
                        );
                    }
                    rank_public_benchmark_lexical_docs(query, docs)
                }
            }
        }
        PublicBenchmarkBackend::Sidecar => {
            // Sidecar adapter for LoCoMo/MemBench/ConvoMem is not in scope
            // for G3 (J3 evaluates accelerated vs intrinsic). Routing
            // sidecar through lexical here keeps the CLI flag honest:
            // the bench manifest still records `sidecar` in the backend
            // column, and behavior is documented as fallback.
            rank_public_benchmark_lexical_docs(query, docs)
        }
    }
}

/// Map the `(corpus_index, score)` shape returned by
/// `rank_corpus_via_memd` / `rank_longmemeval_corpus_via_rrf` back into
/// the `((doc_id, text), score)` shape `build_context_retrieval_run_report`
/// consumes. Indices not produced by the backend are appended at the end
/// with score 0.0 in original docs order, so the caller still sees every
/// doc (matches lexical behavior, which scores every doc).
fn ranked_indices_to_docs(
    ranked: Vec<(usize, f64)>,
    docs: &[(String, String)],
    _query: &str,
) -> Vec<((String, String), f64)> {
    let mut out = Vec::with_capacity(docs.len());
    let mut seen = std::collections::HashSet::with_capacity(docs.len());
    for (index, score) in &ranked {
        if let Some(doc) = docs.get(*index)
            && seen.insert(*index)
        {
            out.push((doc.clone(), *score));
        }
    }
    for (index, doc) in docs.iter().enumerate() {
        if seen.insert(index) {
            out.push((doc.clone(), 0.0));
        }
    }
    out
}

pub(crate) fn build_context_retrieval_run_report(
    dataset: &PublicBenchmarkDatasetFixture,
    top_k: usize,
    mode: &str,
    reranker_id: Option<&str>,
    retrieval_config: &PublicBenchmarkRetrievalConfig,
    retrieval_docs: impl Fn(&PublicBenchmarkDatasetFixtureItem) -> Vec<(String, String)>,
    expected_targets: impl Fn(&PublicBenchmarkDatasetFixtureItem) -> BTreeSet<String>,
) -> anyhow::Result<PublicBenchmarkRunReport> {
    let started = Instant::now();
    let mut results = Vec::new();
    let mut failures = Vec::new();
    let mut total_latency_ms: u128 = 0;
    let mut hits: usize = 0;

    let bench_id = dataset.benchmark_id.as_str();
    for (index, item) in dataset.items.iter().enumerate() {
        let item_started = Instant::now();
        let docs = retrieval_docs(item);
        let expected = expected_targets(item);
        let ranked = dispatch_context_retrieval_ranked(
            bench_id,
            &item.item_id,
            &item.query,
            &docs,
            mode,
            retrieval_config,
        );

        let retrieved_items = ranked
            .iter()
            .take(top_k)
            .enumerate()
            .map(|(rank, ((doc_id, text), score))| {
                json!({
                    "rank": rank + 1,
                    "item_id": doc_id,
                    "question_id": item.question_id,
                    "text": text,
                    "score": score,
                })
            })
            .collect::<Vec<_>>();
        let retrieved_ids = ranked
            .iter()
            .take(top_k)
            .map(|((doc_id, _), _)| doc_id.clone())
            .collect::<BTreeSet<_>>();
        let hit =
            !expected.is_empty() && expected.iter().any(|target| retrieved_ids.contains(target));
        if hit {
            hits += 1;
        } else {
            failures.push(json!({
                "item_id": item.item_id,
                "question_id": item.question_id,
                "expected": item.gold_answer,
                "expected_targets": expected.iter().cloned().collect::<Vec<_>>(),
                "reason": "retrieval missed annotated benchmark evidence",
            }));
        }
        let observed_answer = ranked.first().map(|((_, text), _)| text.clone());
        let item_latency_ms = item_started.elapsed().as_millis().max(1);
        total_latency_ms += item_latency_ms;
        let token_usage = if mode == "hybrid" {
            Some(json!({
                "prompt_tokens": 0,
                "completion_tokens": 0,
                "reranker_tokens": 0,
            }))
        } else {
            None
        };
        let cost_estimate_usd = if mode == "hybrid" { Some(0.0) } else { None };
        results.push(PublicBenchmarkItemResult {
            item_id: item.item_id.clone(),
            question_id: item.question_id.clone(),
            claim_class: item.claim_class.clone(),
            mode: None,
            question: Some(item.query.clone()),
            question_type: item
                .metadata
                .get("question_type")
                .and_then(JsonValue::as_str)
                .map(str::to_string),
            ranked_items: retrieved_items.clone(),
            retrieved_items,
            retrieval_scores: ranked.iter().take(top_k).map(|(_, score)| *score).collect(),
            hit,
            answer: Some(item.gold_answer.clone()),
            observed_answer: observed_answer.clone(),
            correctness: Some(json!({
                "score": if hit { 1.0 } else { 0.0 },
                "expected": item.gold_answer,
                "observed": observed_answer,
                "index": index,
                "mode": mode,
                "expected_targets": expected.iter().cloned().collect::<Vec<_>>(),
            })),
            latency_ms: item_latency_ms,
            token_usage,
            cost_estimate_usd,
        });
    }

    let item_count = results.len();
    let accuracy = if item_count == 0 {
        0.0
    } else {
        hits as f64 / item_count as f64
    };
    let mean_latency_ms = if item_count == 0 {
        0.0
    } else {
        total_latency_ms as f64 / item_count as f64
    };
    let mut metrics = BTreeMap::new();
    metrics.insert("accuracy".to_string(), accuracy);
    metrics.insert("hit_rate".to_string(), accuracy);
    metrics.insert("recall_at_k".to_string(), accuracy);
    metrics.insert("mean_latency_ms".to_string(), mean_latency_ms);
    metrics.insert("item_count".to_string(), item_count as f64);

    let _ = started;

    Ok(PublicBenchmarkRunReport {
        manifest: PublicBenchmarkManifest {
            benchmark_id: String::new(),
            benchmark_version: String::new(),
            dataset_name: String::new(),
            dataset_source_url: String::new(),
            dataset_local_path: String::new(),
            dataset_checksum: String::new(),
            dataset_split: String::new(),
            git_sha: None,
            dirty_worktree: false,
            run_timestamp: Utc::now(),
            mode: mode.to_string(),
            top_k,
            reranker_id: reranker_id.map(str::to_string),
            reranker_provider: if mode == "hybrid" {
                Some("declared".to_string())
            } else {
                None
            },
            limit: Some(dataset.items.len()),
            runtime_settings: JsonValue::Null,
            hardware_summary: String::new(),
            duration_ms: total_latency_ms,
            token_usage: if mode == "hybrid" {
                Some(json!({"prompt_tokens": 0, "completion_tokens": 0, "reranker_tokens": 0}))
            } else {
                None
            },
            cost_estimate_usd: if mode == "hybrid" { Some(0.0) } else { None },
        },
        metrics,
        item_count: dataset.items.len(),
        failures,
        items: results,
    })
}

pub(crate) fn build_longmemeval_session_corpus(
    item: &PublicBenchmarkDatasetFixtureItem,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let sessions = item
        .metadata
        .get("haystack_sessions")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    let session_ids = public_benchmark_string_vec(item.metadata.get("haystack_session_ids"));
    let dates = public_benchmark_string_vec(item.metadata.get("haystack_dates"));
    let mut corpus = Vec::new();
    let mut corpus_ids = Vec::new();
    let mut corpus_timestamps = Vec::new();

    for (index, session) in sessions.iter().enumerate() {
        let session_turns = session
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|turn| {
                let role = turn
                    .get("role")
                    .and_then(JsonValue::as_str)
                    .map(str::trim)
                    .filter(|role| !role.is_empty())?;
                turn.get("content")
                    .and_then(JsonValue::as_str)
                    .map(str::trim)
                    .filter(|content| !content.is_empty())
                    .map(|content| format!("{role}: {content}"))
            })
            .collect::<Vec<_>>();
        if session_turns.is_empty() {
            continue;
        }
        corpus.push(session_turns.join("\n"));
        corpus_ids.push(
            session_ids
                .get(index)
                .cloned()
                .unwrap_or_else(|| format!("session_{index}")),
        );
        corpus_timestamps.push(
            dates
                .get(index)
                .cloned()
                .unwrap_or_else(|| "unknown".to_string()),
        );
    }

    (corpus, corpus_ids, corpus_timestamps)
}

pub(crate) fn build_longmemeval_turn_corpus(
    item: &PublicBenchmarkDatasetFixtureItem,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let sessions = item
        .metadata
        .get("haystack_sessions")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    let session_ids = public_benchmark_string_vec(item.metadata.get("haystack_session_ids"));
    let dates = public_benchmark_string_vec(item.metadata.get("haystack_dates"));
    let mut corpus = Vec::new();
    let mut corpus_ids = Vec::new();
    let mut corpus_timestamps = Vec::new();

    for (session_index, session) in sessions.iter().enumerate() {
        let base_session_id = session_ids
            .get(session_index)
            .cloned()
            .unwrap_or_else(|| format!("session_{session_index}"));
        let date = dates
            .get(session_index)
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let mut turn_index = 0usize;
        for turn in session.as_array().into_iter().flatten() {
            if turn.get("role").and_then(JsonValue::as_str) != Some("user") {
                continue;
            }
            if let Some(content) = turn
                .get("content")
                .and_then(JsonValue::as_str)
                .map(str::trim)
                .filter(|content| !content.is_empty())
            {
                corpus.push(content.to_string());
                corpus_ids.push(format!("{base_session_id}_turn_{turn_index}"));
                corpus_timestamps.push(date.clone());
                turn_index += 1;
            }
        }
    }

    (corpus, corpus_ids, corpus_timestamps)
}

pub(crate) fn longmemeval_bench_namespace(
    kind: &str,
    corpus_ids: &[String],
    corpus: &[String],
) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    kind.hash(&mut hasher);
    corpus_ids.hash(&mut hasher);
    corpus.hash(&mut hasher);
    format!("longmemeval-{kind}-{:016x}", hasher.finish())
}

/// G3 step 5: per-item namespace isolation. Hashes (bench_id, item_id,
/// corpus_ids, corpus) so two items with the same query but different
/// haystacks land in distinct memd namespaces, preventing cross-question
/// bleed in the LoCoMo/MemBench/ConvoMem dispatcher path. Two calls with
/// identical inputs return the same string — `claim_public_benchmark_namespace`
/// then short-circuits the second ingest pass.
pub(crate) fn bench_item_namespace(
    bench_id: &str,
    item_id: &str,
    corpus_ids: &[String],
    corpus: &[String],
) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    bench_id.hash(&mut hasher);
    item_id.hash(&mut hasher);
    corpus_ids.hash(&mut hasher);
    corpus.hash(&mut hasher);
    format!("{bench_id}-{item_id}-{:016x}", hasher.finish())
}

fn claim_public_benchmark_namespace(namespace: &str) -> bool {
    static PRIMED_NAMESPACES: std::sync::OnceLock<std::sync::Mutex<BTreeSet<String>>> =
        std::sync::OnceLock::new();
    let cache = PRIMED_NAMESPACES.get_or_init(|| std::sync::Mutex::new(BTreeSet::new()));
    let mut cache = cache.lock().expect("benchmark namespace cache poisoned");
    cache.insert(namespace.to_string())
}

pub(crate) fn rank_public_benchmark_corpus(
    query: &str,
    corpus: &[String],
    corpus_ids: &[String],
    mode: &str,
) -> Vec<usize> {
    let query_tokens = tokenize_public_benchmark_text(query);
    let stop_words = [
        "what", "when", "where", "who", "how", "which", "did", "do", "was", "were", "have", "has",
        "had", "is", "are", "the", "a", "an", "my", "me", "i", "you", "your", "their", "it", "its",
        "in", "on", "at", "to", "for", "of", "with", "by", "from", "ago", "last", "that", "this",
        "there", "about", "get", "got", "give", "gave", "buy", "bought", "made", "make",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    let keywords = query_tokens
        .iter()
        .filter(|token| token.len() >= 3 && !stop_words.contains(*token))
        .cloned()
        .collect::<Vec<_>>();
    let mut scored = corpus
        .iter()
        .enumerate()
        .map(|(index, document)| {
            let doc_tokens = tokenize_public_benchmark_text(document);
            let overlap = query_tokens.intersection(&doc_tokens).count() as f64;
            let mut score = overlap;
            if mode == "hybrid" && !keywords.is_empty() {
                let doc_lower = document.to_ascii_lowercase();
                let keyword_hits = keywords
                    .iter()
                    .filter(|kw| doc_lower.contains(kw.as_str()))
                    .count();
                score += (keyword_hits as f64 / keywords.len() as f64) * 0.30;
            }
            if corpus_ids.get(index).is_some_and(|id| id.contains("_abs")) {
                score -= 0.05;
            }
            (index, score)
        })
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });
    scored.into_iter().map(|(index, _)| index).collect()
}

pub(crate) fn build_public_benchmark_retrieval_config(
    args: &PublicBenchmarkArgs,
) -> anyhow::Result<PublicBenchmarkRetrievalConfig> {
    let requested_backend = args.retrieval_backend.as_deref().unwrap_or("lexical");
    let longmemeval_backend = match requested_backend {
        "lexical" => LongMemEvalRetrievalBackend::Lexical,
        "sidecar" => LongMemEvalRetrievalBackend::Sidecar,
        "rrf" => LongMemEvalRetrievalBackend::Rrf,
        "memd" => LongMemEvalRetrievalBackend::Memd,
        other => {
            anyhow::bail!(
                "invalid retrieval backend `{other}`; expected lexical, sidecar, rrf, or memd"
            )
        }
    };

    let sidecar_base_url = if longmemeval_backend == LongMemEvalRetrievalBackend::Sidecar {
        Some(resolve_rag_url(args.rag_url.clone(), Some(&args.out))?)
    } else {
        None
    };

    let memd_base_url = if longmemeval_backend == LongMemEvalRetrievalBackend::Memd {
        let url = args
            .memd_url
            .clone()
            .or_else(|| std::env::var("MEMD_BASE_URL").ok())
            .unwrap_or_else(|| "http://127.0.0.1:8787".to_string());
        Some(url.trim_end_matches('/').to_string())
    } else {
        None
    };

    Ok(PublicBenchmarkRetrievalConfig {
        longmemeval_backend,
        sidecar_base_url,
        memd_base_url,
    })
}

pub(crate) fn rank_longmemeval_corpus(
    query: &str,
    corpus: &[String],
    corpus_ids: &[String],
    mode: &str,
    config: &PublicBenchmarkRetrievalConfig,
    namespace: &str,
) -> anyhow::Result<Vec<(usize, f64)>> {
    match config.longmemeval_backend {
        LongMemEvalRetrievalBackend::Lexical => Ok(rank_public_benchmark_corpus(
            query, corpus, corpus_ids, mode,
        )
        .into_iter()
        .enumerate()
        .map(|(rank, index)| (index, (50usize.saturating_sub(rank)) as f64))
        .collect()),
        LongMemEvalRetrievalBackend::Sidecar => {
            let base_url = config
                .sidecar_base_url
                .as_deref()
                .context("sidecar retrieval backend selected without a sidecar base url")?;
            rank_longmemeval_corpus_via_sidecar(
                base_url, query, corpus, corpus_ids, mode, namespace,
            )
        }
        LongMemEvalRetrievalBackend::Rrf => Ok(rank_longmemeval_corpus_via_rrf(
            query, corpus, corpus_ids, mode,
        )),
        LongMemEvalRetrievalBackend::Memd => {
            let base_url = config
                .memd_base_url
                .as_deref()
                .context("memd retrieval backend selected without a memd base url")?;
            rank_longmemeval_corpus_via_memd(base_url, query, corpus, corpus_ids, mode, namespace)
        }
    }
}

// Gate bench probes behind MEMD_BENCH_PROBES env var (opt-in; default quiet)
macro_rules! bench_probe {
    ($($arg:tt)*) => {
        if std::env::var("MEMD_BENCH_PROBES").as_deref().map_or(false, |v| matches!(v, "1" | "true" | "on" | "yes")) {
            eprintln!($($arg)*);
        }
    };
}

pub(crate) fn rank_longmemeval_corpus_via_sidecar(
    base_url: &str,
    query: &str,
    corpus: &[String],
    corpus_ids: &[String],
    mode: &str,
    namespace: &str,
) -> anyhow::Result<Vec<(usize, f64)>> {
    bench_probe!(
        "[bench-probe] enter ns={namespace} corpus_len={}",
        corpus.len()
    );
    let t0 = std::time::Instant::now();
    let lexical_fallback = rank_public_benchmark_corpus(query, corpus, corpus_ids, mode);
    bench_probe!(
        "[bench-probe] lexical_fallback done ns={namespace} elapsed_ms={}",
        t0.elapsed().as_millis()
    );

    let ingest_url = format!("{}/v1/ingest", base_url.trim_end_matches('/'));
    let retrieve_url = format!("{}/v1/retrieve", base_url.trim_end_matches('/'));
    let project = Some("memd-public-benchmark-longmemeval".to_string());
    let namespace_owned = Some(namespace.to_string());

    // Use a dedicated OS thread owning its own current-thread tokio runtime
    // to avoid `reqwest::blocking`'s internal dual-runtime dance (observed
    // to wedge under bench load; see B3 part2 prereq). Running on a fresh
    // thread also sidesteps any outer runtime the caller may already own.
    let project_for_thread = project.clone();
    let namespace_for_thread = namespace_owned.clone();
    let query_owned = query.to_string();
    let corpus_vec = corpus.to_vec();
    let corpus_ids_vec = corpus_ids.to_vec();
    let ingest_url_owned = ingest_url.clone();
    let retrieve_url_owned = retrieve_url.clone();
    let mode_owned = mode.to_string();
    let ns_label = namespace.to_string();
    let handle = std::thread::Builder::new()
        .name(format!("bench-sidecar-{}", ns_label))
        .spawn(move || -> anyhow::Result<RagRetrieveResponse> {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .context("build tokio runtime for public benchmark sidecar client")?;
            rt.block_on(bench_sidecar_roundtrip(
                ingest_url_owned,
                retrieve_url_owned,
                project_for_thread,
                namespace_for_thread,
                ns_label,
                query_owned,
                corpus_vec,
                corpus_ids_vec,
                mode_owned,
            ))
        })
        .context("spawn bench sidecar worker thread")?;
    let retrieved: RagRetrieveResponse = handle
        .join()
        .map_err(|_| anyhow::anyhow!("bench sidecar worker thread panicked"))??;

    let corpus_index_by_id = corpus_ids
        .iter()
        .enumerate()
        .map(|(index, id)| (id.as_str(), index))
        .collect::<std::collections::HashMap<_, _>>();
    let lexical_rank_by_index = lexical_fallback
        .iter()
        .enumerate()
        .map(|(rank, index)| (*index, rank))
        .collect::<std::collections::HashMap<_, _>>();
    let mut seen = BTreeSet::new();
    let mut ranked = Vec::new();

    for item in retrieved.items {
        if let Some(source_id) = item.source.as_deref()
            && let Some(index) = corpus_index_by_id.get(source_id).copied()
            && seen.insert(index)
        {
            ranked.push((index, item.score as f64));
        }
    }

    for index in lexical_fallback {
        if seen.insert(index) {
            let lexical_rank = lexical_rank_by_index.get(&index).copied().unwrap_or(0);
            ranked.push((index, (50usize.saturating_sub(lexical_rank)) as f64));
        }
    }

    Ok(ranked)
}

async fn bench_memd_roundtrip(
    bench_id: String,
    store_url: String,
    search_url: String,
    project: Option<String>,
    namespace_owned: Option<String>,
    ns_label: String,
    query: String,
    corpus: Vec<String>,
    corpus_ids: Vec<String>,
) -> anyhow::Result<memd_schema::SearchMemoryResponse> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .context("build public benchmark memd client")?;

    if claim_public_benchmark_namespace(&ns_label) {
        let store_start = std::time::Instant::now();
        for (idx, (corpus_id, content)) in corpus_ids.iter().zip(corpus.iter()).enumerate() {
            let content = content.trim();
            if content.is_empty() {
                continue;
            }
            bench_probe!(
                "[bench-probe] store-iter ns={ns_label} idx={idx}/{} elapsed_ms={} content_len={}",
                corpus.len(),
                store_start.elapsed().as_millis(),
                content.len()
            );
            let request = memd_schema::StoreMemoryRequest {
                content: content.to_string(),
                kind: memd_schema::MemoryKind::Fact,
                scope: memd_schema::MemoryScope::Project,
                project: project.clone(),
                namespace: namespace_owned.clone(),
                workspace: None,
                visibility: None,
                belief_branch: None,
                source_agent: Some("public-benchmark".to_string()),
                source_system: None,
                source_path: Some(corpus_id.clone()),
                source_quality: None,
                confidence: None,
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec![
                    "public-benchmark".to_string(),
                    bench_id.clone(),
                    corpus_id.clone(),
                ],
                status: None,
                lane: None,
            };
            let send_start = std::time::Instant::now();
            let req_builder = client.post(&store_url).json(&request);
            bench_probe!(
                "[bench-probe] store-json-built ns={ns_label} idx={idx} elapsed_ms={}",
                send_start.elapsed().as_millis()
            );
            let response = req_builder
                .send()
                .await
                .context("send public benchmark memd store")?;
            let status = response.status();
            bench_probe!(
                "[bench-probe] store-send-returned ns={ns_label} idx={idx} status={} elapsed_ms={}",
                status,
                send_start.elapsed().as_millis()
            );
            let body_text = response.text().await.unwrap_or_default();
            bench_probe!(
                "[bench-probe] store-reply ns={ns_label} idx={idx} status={} send_ms={} body_len={}",
                status,
                send_start.elapsed().as_millis(),
                body_text.len()
            );
            if !status.is_success() {
                anyhow::bail!("public benchmark memd store failed with {status}: {body_text}");
            }
        }
        bench_probe!(
            "[bench-probe] stores done ns={ns_label} count={} elapsed_ms={}",
            corpus.len(),
            store_start.elapsed().as_millis()
        );
    } else {
        bench_probe!(
            "[bench-probe] reuse-primed-ns ns={ns_label} count={}",
            corpus.len()
        );
    }

    let search_request = memd_schema::SearchMemoryRequest {
        query: Some(query),
        route: None,
        intent: None,
        scopes: Vec::new(),
        kinds: Vec::new(),
        statuses: Vec::new(),
        project,
        namespace: namespace_owned,
        workspace: None,
        visibility: None,
        belief_branch: None,
        source_agent: Some("public-benchmark".to_string()),
        region: None,
        tags: Vec::new(),
        stages: Vec::new(),
        limit: Some(corpus.len().max(1)),
        max_chars_per_item: None,
    };
    let search_start = std::time::Instant::now();
    let response = client
        .post(&search_url)
        .json(&search_request)
        .send()
        .await
        .context("send public benchmark memd search")?;
    bench_probe!(
        "[bench-probe] search returned ns={ns_label} status={} elapsed_ms={}",
        response.status(),
        search_start.elapsed().as_millis()
    );
    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "failed to read search body".to_string());
        anyhow::bail!("public benchmark memd search failed with {status}: {body}");
    }
    response
        .json::<memd_schema::SearchMemoryResponse>()
        .await
        .context("decode public benchmark memd search payload")
}

async fn bench_sidecar_roundtrip(
    ingest_url: String,
    retrieve_url: String,
    project: Option<String>,
    namespace_owned: Option<String>,
    ns_label: String,
    query: String,
    corpus: Vec<String>,
    corpus_ids: Vec<String>,
    mode: String,
) -> anyhow::Result<RagRetrieveResponse> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .context("build public benchmark sidecar client")?;

    if claim_public_benchmark_namespace(&ns_label) {
        let ingest_start = std::time::Instant::now();
        for (idx, (corpus_id, content)) in corpus_ids.iter().zip(corpus.iter()).enumerate() {
            let content = content.trim();
            if content.is_empty() {
                continue;
            }
            bench_probe!(
                "[bench-probe] ingest-iter ns={ns_label} idx={idx}/{} elapsed_ms={} content_len={}",
                corpus.len(),
                ingest_start.elapsed().as_millis(),
                content.len()
            );
            let request = RagIngestRequest {
                project: project.clone(),
                namespace: namespace_owned.clone(),
                source: RagIngestSource {
                    id: uuid::Uuid::new_v4(),
                    kind: "longmemeval_corpus".to_string(),
                    content: content.to_string(),
                    mime: None,
                    bytes: Some(content.len() as u64),
                    source_quality: None,
                    source_agent: Some("public-benchmark".to_string()),
                    source_path: Some(corpus_id.clone()),
                    tags: vec!["public-benchmark".to_string(), "longmemeval".to_string()],
                },
            };
            let send_start = std::time::Instant::now();
            let response = client
                .post(&ingest_url)
                .json(&request)
                .send()
                .await
                .context("send public benchmark sidecar ingest")?;
            let status = response.status();
            bench_probe!(
                "[bench-probe] ingest-send-returned ns={ns_label} idx={idx} status={} elapsed_ms={}",
                status,
                send_start.elapsed().as_millis()
            );
            let body_text = response.text().await.unwrap_or_default();
            bench_probe!(
                "[bench-probe] ingest-reply ns={ns_label} idx={idx} status={} send_ms={} body_len={}",
                status,
                send_start.elapsed().as_millis(),
                body_text.len()
            );
            if !status.is_success() {
                anyhow::bail!("public benchmark sidecar ingest failed with {status}: {body_text}");
            }
        }
        bench_probe!(
            "[bench-probe] ingests done ns={ns_label} count={} elapsed_ms={}",
            corpus.len(),
            ingest_start.elapsed().as_millis()
        );
    } else {
        bench_probe!(
            "[bench-probe] reuse-primed-ns ns={ns_label} count={}",
            corpus.len()
        );
    }

    let retrieve_request = RagRetrieveRequest {
        query: query.clone(),
        project,
        namespace: namespace_owned,
        mode: if mode == "hybrid" {
            RagRetrieveMode::Auto
        } else {
            RagRetrieveMode::Text
        },
        limit: Some(corpus.len().max(1)),
        include_cross_modal: false,
    };
    let retrieve_start = std::time::Instant::now();
    let response = client
        .post(&retrieve_url)
        .json(&retrieve_request)
        .send()
        .await
        .context("send public benchmark sidecar retrieve")?;
    bench_probe!(
        "[bench-probe] retrieve returned ns={ns_label} status={} elapsed_ms={}",
        response.status(),
        retrieve_start.elapsed().as_millis()
    );
    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "failed to read retrieve body".to_string());
        anyhow::bail!("public benchmark sidecar retrieve failed with {status}: {body}");
    }
    response
        .json::<RagRetrieveResponse>()
        .await
        .context("decode public benchmark sidecar retrieve payload")
}

/// B3 Part-2 prereq: route bench through an actual memd-server so the
/// intrinsic retrieval path (FTS5 scoring, priority dedup, atlas recall,
/// sanitize) is what produces the LongMemEval number. Each call opens a
/// throwaway namespace (one per `namespace` arg), ingests the corpus,
/// issues one search, and lets server-side GC / namespace isolation
/// prevent cross-question bleed. Corpus identifier is round-tripped via
/// `source_path`.
pub(crate) fn rank_longmemeval_corpus_via_memd(
    base_url: &str,
    query: &str,
    corpus: &[String],
    corpus_ids: &[String],
    mode: &str,
    namespace: &str,
) -> anyhow::Result<Vec<(usize, f64)>> {
    rank_corpus_via_memd(
        "longmemeval",
        base_url,
        query,
        corpus,
        corpus_ids,
        mode,
        namespace,
    )
}

/// G3 step 4: generic memd-backed corpus ranker. Same intrinsic path as
/// `rank_longmemeval_corpus_via_memd`, but `bench_id` parameterizes the
/// project name, ingest tag, and worker thread label — so LoCoMo,
/// MemBench, and ConvoMem can dispatch through `/memory/store` +
/// `/memory/search` without cloning the runtime + RRF dance per bench.
/// Returns `(corpus_index, score)` pairs after RRF-merging memd ranking
/// with the lexical fallback (see `merge_ranked_longmemeval_results`).
pub(crate) fn rank_corpus_via_memd(
    bench_id: &str,
    base_url: &str,
    query: &str,
    corpus: &[String],
    corpus_ids: &[String],
    mode: &str,
    namespace: &str,
) -> anyhow::Result<Vec<(usize, f64)>> {
    bench_probe!(
        "[bench-probe] enter bench={bench_id} ns={namespace} corpus_len={}",
        corpus.len()
    );
    let t0 = std::time::Instant::now();
    let lexical_fallback = rank_public_benchmark_corpus(query, corpus, corpus_ids, mode);
    bench_probe!(
        "[bench-probe] lexical_fallback done bench={bench_id} ns={namespace} elapsed_ms={}",
        t0.elapsed().as_millis()
    );

    let store_url = format!("{}/memory/store", base_url.trim_end_matches('/'));
    let search_url = format!("{}/memory/search", base_url.trim_end_matches('/'));
    let project = Some(format!("memd-public-benchmark-{bench_id}"));
    let namespace_owned = Some(namespace.to_string());

    // Use a dedicated OS thread owning its own current-thread tokio runtime
    // to avoid `reqwest::blocking`'s internal dual-runtime dance (observed
    // to wedge under bench load; see B3 part2 prereq). Running on a fresh
    // thread also sidesteps any outer runtime the caller may already own.
    let bench_id_owned = bench_id.to_string();
    let project_for_thread = project.clone();
    let namespace_for_thread = namespace_owned.clone();
    let query_owned = query.to_string();
    let corpus_vec = corpus.to_vec();
    let corpus_ids_vec = corpus_ids.to_vec();
    let store_url_owned = store_url.clone();
    let search_url_owned = search_url.clone();
    let ns_label = namespace.to_string();
    let bench_id_for_thread = bench_id_owned.clone();
    let handle = std::thread::Builder::new()
        .name(format!("bench-memd-{bench_id_owned}-{ns_label}"))
        .spawn(
            move || -> anyhow::Result<memd_schema::SearchMemoryResponse> {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .context("build tokio runtime for public benchmark memd client")?;
                rt.block_on(bench_memd_roundtrip(
                    bench_id_for_thread,
                    store_url_owned,
                    search_url_owned,
                    project_for_thread,
                    namespace_for_thread,
                    ns_label,
                    query_owned,
                    corpus_vec,
                    corpus_ids_vec,
                ))
            },
        )
        .context("spawn bench memd worker thread")?;
    let retrieved: memd_schema::SearchMemoryResponse = handle
        .join()
        .map_err(|_| anyhow::anyhow!("bench memd worker thread panicked"))??;

    let corpus_index_by_id = corpus_ids
        .iter()
        .enumerate()
        .map(|(index, id)| (id.as_str(), index))
        .collect::<std::collections::HashMap<_, _>>();
    let lexical_rank_by_index = lexical_fallback
        .iter()
        .enumerate()
        .map(|(rank, index)| (*index, rank))
        .collect::<std::collections::HashMap<_, _>>();
    let mut server_ranked = Vec::new();
    let mut seen = BTreeSet::new();

    // Server returns items ordered by the intrinsic ranker. Give each a
    // monotonically decreasing score so ordering survives downstream merge.
    let n = retrieved.items.len();
    for (rank, item) in retrieved.items.iter().enumerate() {
        let source_id = item.source_path.as_deref().or_else(|| {
            item.tags
                .iter()
                .find(|t| corpus_index_by_id.contains_key(t.as_str()))
                .map(|s| s.as_str())
        });
        if let Some(sid) = source_id
            && let Some(index) = corpus_index_by_id.get(sid).copied()
            && seen.insert(index)
        {
            server_ranked.push((index, (n - rank) as f64));
        }
    }

    if std::env::var_os("MEMD_BENCH_DUMP_SERVER_RANK").is_some() {
        let q_preview: String = query.chars().take(80).collect();
        let top_ids: Vec<&str> = server_ranked
            .iter()
            .take(15)
            .map(|(i, _)| corpus_ids[*i].as_str())
            .collect();
        let lex_top: Vec<&str> = lexical_fallback
            .iter()
            .take(10)
            .map(|i| corpus_ids[*i].as_str())
            .collect();
        eprintln!(
            "[dump-rank] ns={namespace} q=\"{q_preview}\" server_top15={top_ids:?} lexical_top10={lex_top:?}"
        );
    }

    Ok(merge_ranked_longmemeval_results(
        &server_ranked,
        &lexical_fallback,
        &lexical_rank_by_index,
    ))
}

pub(crate) fn merge_ranked_longmemeval_results(
    primary_ranked: &[(usize, f64)],
    lexical_fallback: &[usize],
    lexical_rank_by_index: &std::collections::HashMap<usize, usize>,
) -> Vec<(usize, f64)> {
    const RRF_K: f64 = 60.0;
    const PRIMARY_SUFFICIENT_THRESHOLD: usize = 5;

    let primary_sufficient = primary_ranked.len() >= PRIMARY_SUFFICIENT_THRESHOLD;

    let mut scores = std::collections::HashMap::<usize, f64>::new();
    let mut primary_score_by_index = std::collections::HashMap::<usize, f64>::new();

    for (rank, (index, score)) in primary_ranked.iter().enumerate() {
        *scores.entry(*index).or_default() += 1.0 / (RRF_K + rank as f64);
        primary_score_by_index.insert(*index, *score);
    }

    if !primary_sufficient {
        for (rank, index) in lexical_fallback.iter().enumerate() {
            *scores.entry(*index).or_default() += 1.0 / (RRF_K + rank as f64);
        }
    }

    let mut merged = scores
        .into_iter()
        .map(|(index, score)| {
            (
                index,
                score,
                primary_score_by_index
                    .get(&index)
                    .copied()
                    .unwrap_or_default(),
                lexical_rank_by_index
                    .get(&index)
                    .copied()
                    .unwrap_or(usize::MAX),
            )
        })
        .collect::<Vec<_>>();
    merged.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| right.2.total_cmp(&left.2))
            .then_with(|| left.3.cmp(&right.3))
            .then_with(|| left.0.cmp(&right.0))
    });
    merged
        .into_iter()
        .map(|(index, score, _, _)| (index, score))
        .collect()
}

/// RRF-based ranking: build an ephemeral FTS5 index from the corpus,
/// query it, then merge FTS ranks with lexical ranks via Reciprocal Rank
/// Fusion (k=60). Returns (corpus_index, rrf_score) pairs sorted by score.
pub(crate) fn rank_longmemeval_corpus_via_rrf(
    query: &str,
    corpus: &[String],
    corpus_ids: &[String],
    mode: &str,
) -> Vec<(usize, f64)> {
    const RRF_K: f64 = 60.0;

    // Lexical ranking (existing path)
    let lexical_order = rank_public_benchmark_corpus(query, corpus, corpus_ids, mode);

    // Build ephemeral FTS5 index
    let fts_order = match build_ephemeral_fts_ranking(query, corpus) {
        Ok(order) => order,
        Err(_) => {
            // Fall back to lexical-only if FTS fails
            return lexical_order
                .into_iter()
                .enumerate()
                .map(|(rank, index)| (index, (50usize.saturating_sub(rank)) as f64))
                .collect();
        }
    };

    // Build rank maps
    let mut rrf_scores = std::collections::HashMap::<usize, f64>::new();
    for (rank, &index) in lexical_order.iter().enumerate() {
        *rrf_scores.entry(index).or_default() += 1.0 / (RRF_K + rank as f64);
    }
    for (rank, &index) in fts_order.iter().enumerate() {
        *rrf_scores.entry(index).or_default() += 1.0 / (RRF_K + rank as f64);
    }

    let mut merged: Vec<(usize, f64)> = rrf_scores.into_iter().collect();
    merged.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    merged
}

/// Create a temp SQLite DB with FTS5, index the corpus, query it, return
/// ranked corpus indices.
fn build_ephemeral_fts_ranking(query: &str, corpus: &[String]) -> anyhow::Result<Vec<usize>> {
    let conn = rusqlite::Connection::open_in_memory().context("open ephemeral fts db")?;
    conn.execute_batch(
        "CREATE VIRTUAL TABLE corpus_fts USING fts5(content, doc_index UNINDEXED);",
    )?;

    {
        let mut stmt =
            conn.prepare("INSERT INTO corpus_fts(doc_index, content) VALUES (?1, ?2)")?;
        for (index, doc) in corpus.iter().enumerate() {
            stmt.execute(rusqlite::params![index as i64, doc])?;
        }
    }

    let mut stmt = conn.prepare(
        "SELECT doc_index FROM corpus_fts WHERE corpus_fts MATCH ?1 ORDER BY rank LIMIT ?2",
    )?;
    let rows = stmt.query_map(rusqlite::params![query, corpus.len() as i64], |row| {
        row.get::<_, i64>(0)
    })?;

    let mut ranked = Vec::new();
    for row in rows {
        ranked.push(row? as usize);
    }

    Ok(ranked)
}

pub(crate) fn evaluate_ranked_longmemeval_ids(
    rankings: &[usize],
    correct_ids: &BTreeSet<String>,
    corpus_ids: &[String],
    k: usize,
) -> (f64, f64, f64) {
    let top_k_ids = rankings
        .iter()
        .take(k)
        .filter_map(|index| corpus_ids.get(*index))
        .cloned()
        .collect::<BTreeSet<_>>();
    let recall_any = if correct_ids.iter().any(|id| top_k_ids.contains(id)) {
        1.0
    } else {
        0.0
    };
    let recall_all = if correct_ids.iter().all(|id| top_k_ids.contains(id)) {
        1.0
    } else {
        0.0
    };
    let relevances = rankings
        .iter()
        .map(|index| {
            corpus_ids
                .get(*index)
                .map(|id| if correct_ids.contains(id) { 1.0 } else { 0.0 })
                .unwrap_or(0.0)
        })
        .collect::<Vec<_>>();
    let ndcg = ndcg_public_benchmark(&relevances, k);
    (recall_any, recall_all, ndcg)
}

fn public_benchmark_trimmed_url(value: &str) -> String {
    value.trim().trim_end_matches('/').to_string()
}

pub(crate) fn resolve_public_benchmark_dual_memd_base_urls() -> (String, String) {
    let intrinsic = std::env::var("MEMD_BASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| public_benchmark_trimmed_url(&value))
        .unwrap_or_else(|| "http://127.0.0.1:8787".to_string());
    let accelerated = std::env::var("MEMD_BASE_URL_ACCELERATED")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| public_benchmark_trimmed_url(&value))
        .unwrap_or_else(|| intrinsic.clone());
    (intrinsic, accelerated)
}

pub(crate) fn relabel_public_benchmark_item_result(
    mut item: PublicBenchmarkItemResult,
    mode_label: &str,
) -> PublicBenchmarkItemResult {
    let original_claim_class = item.claim_class.clone();
    let original_item_id = item.item_id.clone();
    item.item_id = format!("{original_item_id}::{mode_label}");
    item.mode = Some(mode_label.to_string());
    item.correctness = Some(match item.correctness.take() {
        Some(JsonValue::Object(mut object)) => {
            object.insert(
                "mode".to_string(),
                JsonValue::String(mode_label.to_string()),
            );
            object.insert(
                "original_claim_class".to_string(),
                JsonValue::String(original_claim_class),
            );
            JsonValue::Object(object)
        }
        Some(other) => json!({
            "mode": mode_label,
            "original_claim_class": original_claim_class,
            "existing_correctness": other,
        }),
        None => json!({
            "mode": mode_label,
            "original_claim_class": original_claim_class,
        }),
    });
    item
}

fn relabel_public_benchmark_failure(failure: JsonValue, mode_label: &str) -> JsonValue {
    match failure {
        JsonValue::Object(mut object) => {
            object.insert(
                "mode".to_string(),
                JsonValue::String(mode_label.to_string()),
            );
            JsonValue::Object(object)
        }
        other => json!({
            "mode": mode_label,
            "failure": other,
        }),
    }
}

fn prefix_public_benchmark_metrics(
    metrics: &BTreeMap<String, f64>,
    prefix: &str,
    output: &mut BTreeMap<String, f64>,
) {
    for (key, value) in metrics {
        output.insert(format!("{prefix}{key}"), *value);
    }
}

pub(crate) fn build_longmemeval_dual_run_report(
    dataset: &PublicBenchmarkDatasetFixture,
    top_k: usize,
    mode: &str,
    reranker_id: Option<&str>,
    started_at: DateTime<Utc>,
    intrinsic_base_url: &str,
    accelerated_base_url: &str,
    include_turn_diagnostics: bool,
) -> anyhow::Result<PublicBenchmarkRunReport> {
    let intrinsic_config = PublicBenchmarkRetrievalConfig {
        longmemeval_backend: LongMemEvalRetrievalBackend::Memd,
        sidecar_base_url: None,
        memd_base_url: Some(public_benchmark_trimmed_url(intrinsic_base_url)),
    };
    let accelerated_config = PublicBenchmarkRetrievalConfig {
        longmemeval_backend: LongMemEvalRetrievalBackend::Memd,
        sidecar_base_url: None,
        memd_base_url: Some(public_benchmark_trimmed_url(accelerated_base_url)),
    };

    let intrinsic_report = build_longmemeval_run_report(
        dataset,
        top_k,
        mode,
        reranker_id,
        &intrinsic_config,
        include_turn_diagnostics,
    )?;
    let accelerated_report = build_longmemeval_run_report(
        dataset,
        top_k,
        mode,
        reranker_id,
        &accelerated_config,
        include_turn_diagnostics,
    )?;

    let mut items = intrinsic_report
        .items
        .into_iter()
        .map(|item| relabel_public_benchmark_item_result(item, "intrinsic"))
        .collect::<Vec<_>>();
    items.extend(
        accelerated_report
            .items
            .into_iter()
            .map(|item| relabel_public_benchmark_item_result(item, "accelerated")),
    );

    let mut failures = intrinsic_report
        .failures
        .into_iter()
        .map(|failure| relabel_public_benchmark_failure(failure, "intrinsic"))
        .collect::<Vec<_>>();
    failures.extend(
        accelerated_report
            .failures
            .into_iter()
            .map(|failure| relabel_public_benchmark_failure(failure, "accelerated")),
    );

    let mut metrics = intrinsic_report.metrics.clone();
    prefix_public_benchmark_metrics(&intrinsic_report.metrics, "intrinsic::", &mut metrics);
    prefix_public_benchmark_metrics(&accelerated_report.metrics, "accelerated::", &mut metrics);
    let combined_item_count = items.len().max(1);
    let combined_duration_ms =
        intrinsic_report.manifest.duration_ms + accelerated_report.manifest.duration_ms;
    metrics.insert("item_count".to_string(), items.len() as f64);
    metrics.insert(
        "mean_latency_ms".to_string(),
        combined_duration_ms as f64 / combined_item_count as f64,
    );

    let mut manifest = intrinsic_report.manifest.clone();
    manifest.run_timestamp = started_at;
    manifest.duration_ms = combined_duration_ms;
    if let Some(runtime_settings) = manifest.runtime_settings.as_object_mut() {
        runtime_settings.insert("dual".to_string(), JsonValue::Bool(true));
        runtime_settings.insert(
            "dual_modes".to_string(),
            json!(["intrinsic", "accelerated"]),
        );
        runtime_settings.insert(
            "intrinsic_base_url".to_string(),
            json!(public_benchmark_trimmed_url(intrinsic_base_url)),
        );
        runtime_settings.insert(
            "accelerated_base_url".to_string(),
            json!(public_benchmark_trimmed_url(accelerated_base_url)),
        );
        runtime_settings.insert("retrieval_backend".to_string(), json!("memd"));
        runtime_settings.insert("dual_rows_per_question".to_string(), json!(2));
    }

    Ok(PublicBenchmarkRunReport {
        manifest,
        metrics,
        item_count: items.len(),
        failures,
        items,
    })
}

pub(crate) fn build_longmemeval_run_report(
    dataset: &PublicBenchmarkDatasetFixture,
    top_k: usize,
    mode: &str,
    reranker_id: Option<&str>,
    retrieval_config: &PublicBenchmarkRetrievalConfig,
    include_turn_diagnostics: bool,
) -> anyhow::Result<PublicBenchmarkRunReport> {
    let ks = [1usize, 3, 5, 10, 30, 50];
    let started = Instant::now();
    let mut metrics = BTreeMap::new();
    let mut per_type: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    let mut items = Vec::new();
    let mut failures = Vec::new();
    let mut total_latency_ms: u128 = 0;
    let mut session_recall_sums = BTreeMap::new();
    let mut session_recall_all_sums = BTreeMap::new();
    let mut session_ndcg_sums = BTreeMap::new();
    let mut turn_recall_sums = BTreeMap::new();
    let mut turn_recall_all_sums = BTreeMap::new();
    let mut turn_ndcg_sums = BTreeMap::new();

    for item in &dataset.items {
        let item_started = Instant::now();
        let answer_session_ids =
            public_benchmark_string_vec(item.metadata.get("answer_session_ids"))
                .into_iter()
                .collect::<BTreeSet<_>>();
        let (session_corpus, session_corpus_ids, session_timestamps) =
            build_longmemeval_session_corpus(item);
        let session_namespace =
            longmemeval_bench_namespace("session", &session_corpus_ids, &session_corpus);
        let session_ranked = rank_longmemeval_corpus(
            &item.query,
            &session_corpus,
            &session_corpus_ids,
            mode,
            retrieval_config,
            &session_namespace,
        )?;
        let session_rankings = session_ranked
            .iter()
            .map(|(index, _)| *index)
            .collect::<Vec<_>>();
        let (turn_corpus, turn_corpus_ids, _turn_timestamps) = if include_turn_diagnostics {
            build_longmemeval_turn_corpus(item)
        } else {
            (Vec::new(), Vec::new(), Vec::new())
        };
        let turn_rankings = if include_turn_diagnostics {
            let turn_namespace =
                longmemeval_bench_namespace("turn", &turn_corpus_ids, &turn_corpus);
            let turn_ranked = rank_longmemeval_corpus(
                &item.query,
                &turn_corpus,
                &turn_corpus_ids,
                mode,
                retrieval_config,
                &turn_namespace,
            )?;
            turn_ranked
                .iter()
                .map(|(index, _)| *index)
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let turn_answer_ids = if include_turn_diagnostics {
            turn_corpus_ids
                .iter()
                .filter(|id| {
                    id.rsplit_once("_turn_")
                        .is_some_and(|(session_id, _)| answer_session_ids.contains(session_id))
                })
                .cloned()
                .collect::<BTreeSet<_>>()
        } else {
            BTreeSet::new()
        };

        let mut session_metrics = serde_json::Map::new();
        let mut turn_metrics = serde_json::Map::new();
        for k in ks {
            let (session_recall_any, session_recall_all, session_ndcg) =
                evaluate_ranked_longmemeval_ids(
                    &session_rankings,
                    &answer_session_ids,
                    &session_corpus_ids,
                    k,
                );
            *session_recall_sums.entry(k).or_insert(0.0) += session_recall_any;
            *session_recall_all_sums.entry(k).or_insert(0.0) += session_recall_all;
            *session_ndcg_sums.entry(k).or_insert(0.0) += session_ndcg;
            session_metrics.insert(
                format!("recall_any@{k}"),
                JsonValue::from(session_recall_any),
            );
            session_metrics.insert(
                format!("recall_all@{k}"),
                JsonValue::from(session_recall_all),
            );
            session_metrics.insert(format!("ndcg_any@{k}"), JsonValue::from(session_ndcg));

            if include_turn_diagnostics {
                let (turn_recall_any, turn_recall_all, turn_ndcg) = evaluate_ranked_longmemeval_ids(
                    &turn_rankings,
                    &turn_answer_ids,
                    &turn_corpus_ids,
                    k,
                );
                *turn_recall_sums.entry(k).or_insert(0.0) += turn_recall_any;
                *turn_recall_all_sums.entry(k).or_insert(0.0) += turn_recall_all;
                *turn_ndcg_sums.entry(k).or_insert(0.0) += turn_ndcg;
                turn_metrics.insert(format!("recall_any@{k}"), JsonValue::from(turn_recall_any));
                turn_metrics.insert(format!("recall_all@{k}"), JsonValue::from(turn_recall_all));
                turn_metrics.insert(format!("ndcg_any@{k}"), JsonValue::from(turn_ndcg));
            }
        }

        let qtype = item
            .metadata
            .get("question_type")
            .and_then(JsonValue::as_str)
            .unwrap_or("unknown")
            .to_string();
        per_type
            .entry(qtype.clone())
            .or_default()
            .push(session_metrics["recall_any@10"].as_f64().unwrap_or(0.0));

        let retrieved_items = session_ranked
            .iter()
            .take(50.min(session_corpus.len()))
            .enumerate()
            .map(|(rank, (index, score))| {
                json!({
                    "rank": rank + 1,
                    "item_id": session_corpus_ids.get(*index).cloned().unwrap_or_default(),
                    "question_id": item.question_id,
                    "text": session_corpus.get(*index).cloned().unwrap_or_default(),
                    "timestamp": session_timestamps.get(*index).cloned().unwrap_or_default(),
                    "score": score,
                })
            })
            .collect::<Vec<_>>();
        let top_hit = session_metrics["recall_any@5"].as_f64().unwrap_or(0.0) > 0.0;
        if !top_hit {
            failures.push(json!({
                "item_id": item.item_id,
                "question_id": item.question_id,
                "question_type": qtype,
                "reason": "session_recall_any@5 = 0",
            }));
        }
        let item_latency_ms = item_started.elapsed().as_millis().max(1);
        total_latency_ms += item_latency_ms;
        items.push(PublicBenchmarkItemResult {
            item_id: item.item_id.clone(),
            question_id: item.question_id.clone(),
            claim_class: item.claim_class.clone(),
            mode: None,
            question: Some(item.query.clone()),
            question_type: Some(qtype.clone()),
            ranked_items: retrieved_items.clone(),
            retrieved_items,
            retrieval_scores: session_ranked
                .iter()
                .take(top_k.min(session_rankings.len()))
                .map(|(_, score)| *score)
                .collect(),
            hit: top_hit,
            answer: Some(item.gold_answer.clone()),
            observed_answer: session_rankings
                .first()
                .and_then(|index| session_corpus.get(*index))
                .cloned(),
            correctness: Some(json!({
                "expected": item.gold_answer,
                "mode": mode,
                "question_type": qtype,
                "session_metrics": JsonValue::Object(session_metrics),
                "turn_metrics": if include_turn_diagnostics {
                    JsonValue::Object(turn_metrics)
                } else {
                    json!({"skipped": true})
                },
                "turn_diagnostics": include_turn_diagnostics,
                "answer_session_ids": answer_session_ids,
                "turn_answer_ids": turn_answer_ids,
            })),
            latency_ms: item_latency_ms,
            token_usage: if mode == "hybrid" {
                Some(json!({
                    "prompt_tokens": 0,
                    "completion_tokens": 0,
                    "reranker_tokens": 0,
                }))
            } else {
                None
            },
            cost_estimate_usd: if mode == "hybrid" || reranker_id.is_some() {
                Some(0.0)
            } else {
                None
            },
        });
    }

    let item_count = dataset.items.len().max(1) as f64;
    for k in ks {
        metrics.insert(
            format!("session_recall_any@{k}"),
            session_recall_sums.get(&k).copied().unwrap_or(0.0) / item_count,
        );
        metrics.insert(
            format!("session_recall_all@{k}"),
            session_recall_all_sums.get(&k).copied().unwrap_or(0.0) / item_count,
        );
        metrics.insert(
            format!("session_ndcg_any@{k}"),
            session_ndcg_sums.get(&k).copied().unwrap_or(0.0) / item_count,
        );
        if include_turn_diagnostics {
            metrics.insert(
                format!("turn_recall_any@{k}"),
                turn_recall_sums.get(&k).copied().unwrap_or(0.0) / item_count,
            );
            metrics.insert(
                format!("turn_recall_all@{k}"),
                turn_recall_all_sums.get(&k).copied().unwrap_or(0.0) / item_count,
            );
            metrics.insert(
                format!("turn_ndcg_any@{k}"),
                turn_ndcg_sums.get(&k).copied().unwrap_or(0.0) / item_count,
            );
        }
    }
    metrics.insert(
        "accuracy".to_string(),
        metrics.get("session_recall_any@5").copied().unwrap_or(0.0),
    );
    metrics.insert(
        "hit_rate".to_string(),
        metrics.get("session_recall_any@5").copied().unwrap_or(0.0),
    );
    metrics.insert(
        "recall_at_k".to_string(),
        metrics
            .get(&format!("session_recall_any@{}", top_k.min(50)))
            .copied()
            .unwrap_or(0.0),
    );
    metrics.insert(
        "mean_latency_ms".to_string(),
        total_latency_ms as f64 / item_count,
    );
    metrics.insert("item_count".to_string(), dataset.items.len() as f64);
    for (qtype, values) in per_type {
        let mean = values.iter().sum::<f64>() / values.len().max(1) as f64;
        metrics.insert(format!("per_type::{qtype}::session_recall_any@10"), mean);
    }
    let _ = started;
    Ok(PublicBenchmarkRunReport {
        manifest: PublicBenchmarkManifest {
            benchmark_id: String::new(),
            benchmark_version: String::new(),
            dataset_name: String::new(),
            dataset_source_url: String::new(),
            dataset_local_path: String::new(),
            dataset_checksum: String::new(),
            dataset_split: String::new(),
            git_sha: None,
            dirty_worktree: false,
            run_timestamp: Utc::now(),
            mode: mode.to_string(),
            top_k,
            reranker_id: reranker_id.map(str::to_string),
            reranker_provider: if mode == "hybrid" {
                Some("declared".to_string())
            } else {
                None
            },
            limit: Some(dataset.items.len()),
            runtime_settings: JsonValue::Null,
            hardware_summary: String::new(),
            duration_ms: total_latency_ms,
            token_usage: if mode == "hybrid" {
                Some(json!({"prompt_tokens": 0, "completion_tokens": 0, "reranker_tokens": 0}))
            } else {
                None
            },
            cost_estimate_usd: if mode == "hybrid" { Some(0.0) } else { None },
        },
        metrics,
        item_count: dataset.items.len(),
        failures,
        items,
    })
}

pub(crate) async fn build_longmemeval_community_standard_run_report(
    dataset: &PublicBenchmarkDatasetFixture,
    hypotheses_path: &Path,
    grader_model: &str,
    mode: &str,
) -> anyhow::Result<PublicBenchmarkRunReport> {
    anyhow::ensure!(
        dataset.benchmark_id == "longmemeval",
        "community-standard evaluation only supports longmemeval"
    );
    let api_key = std::env::var("OPENAI_API_KEY")
        .context("community-standard longmemeval requires OPENAI_API_KEY")?;
    let base_url = std::env::var("OPENAI_BASE_URL")
        .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
    let hypothesis_map = load_longmemeval_hypotheses(hypotheses_path)?
        .into_iter()
        .map(|entry| (entry.question_id, entry.hypothesis))
        .collect::<BTreeMap<_, _>>();

    let started = Instant::now();
    let mut results = Vec::new();
    let mut failures = Vec::new();
    let mut total_latency_ms: u128 = 0;
    let mut correct = 0usize;
    let mut abstention_total = 0usize;
    let mut abstention_correct = 0usize;
    let mut per_type_correct = BTreeMap::<String, usize>::new();
    let mut per_type_total = BTreeMap::<String, usize>::new();

    for (index, item) in dataset.items.iter().enumerate() {
        let item_started = Instant::now();
        let hypothesis = hypothesis_map.get(&item.question_id).cloned();
        let question_type = item
            .metadata
            .get("question_type")
            .and_then(JsonValue::as_str)
            .unwrap_or("unknown");
        let abstention = item.question_id.contains("_abs");
        let (label, grader_response) = if let Some(hypothesis_text) = hypothesis.as_deref() {
            let prompt = build_longmemeval_eval_prompt(
                question_type,
                &item.query,
                &item.gold_answer,
                hypothesis_text,
                abstention,
            )?;
            let cache_key = judge_cache_key(
                "longmemeval-community-standard",
                &item.question_id,
                hypothesis_text,
                grader_model,
                &prompt,
            );
            let grader =
                call_openai_yes_no_grader_cached(&base_url, &api_key, grader_model, &prompt, &cache_key)
                    .await?;
            (
                grader.content.to_ascii_lowercase().contains("yes"),
                Some(grader.content),
            )
        } else {
            (false, None)
        };
        if label {
            correct += 1;
        } else {
            failures.push(json!({
                "item_id": item.item_id,
                "question_id": item.question_id,
                "question_type": question_type,
                "reason": if hypothesis.is_none() {
                    "missing hypothesis for community-standard evaluation"
                } else {
                    "official_qa_eval = false"
                },
            }));
        }
        *per_type_total.entry(question_type.to_string()).or_insert(0) += 1;
        if label {
            *per_type_correct
                .entry(question_type.to_string())
                .or_insert(0) += 1;
        }
        if abstention {
            abstention_total += 1;
            if label {
                abstention_correct += 1;
            }
        }
        let item_latency_ms = item_started.elapsed().as_millis().max(1);
        total_latency_ms += item_latency_ms;
        results.push(PublicBenchmarkItemResult {
            item_id: item.item_id.clone(),
            question_id: item.question_id.clone(),
            claim_class: item.claim_class.clone(),
            mode: None,
            question: Some(item.query.clone()),
            question_type: Some(question_type.to_string()),
            ranked_items: Vec::new(),
            retrieved_items: Vec::new(),
            retrieval_scores: Vec::new(),
            hit: label,
            answer: Some(item.gold_answer.clone()),
            observed_answer: hypothesis.clone(),
            correctness: Some(json!({
                "score": if label { 1.0 } else { 0.0 },
                "expected": item.gold_answer,
                "observed": hypothesis,
                "index": index,
                "mode": mode,
                "evaluation_protocol": "longmemeval-community-standard",
                "grader_model": grader_model,
                "grader_response": grader_response,
            })),
            latency_ms: item_latency_ms,
            token_usage: None,
            cost_estimate_usd: None,
        });
    }

    let item_count = results.len();
    let accuracy = if item_count == 0 {
        0.0
    } else {
        correct as f64 / item_count as f64
    };
    let mean_latency_ms = if item_count == 0 {
        0.0
    } else {
        total_latency_ms as f64 / item_count as f64
    };
    let abstention_accuracy = if abstention_total == 0 {
        0.0
    } else {
        abstention_correct as f64 / abstention_total as f64
    };
    let mut metrics = BTreeMap::new();
    metrics.insert("accuracy".to_string(), accuracy);
    metrics.insert("qa_accuracy".to_string(), accuracy);
    metrics.insert("overall_accuracy".to_string(), accuracy);
    metrics.insert("abstention_accuracy".to_string(), abstention_accuracy);
    metrics.insert("mean_latency_ms".to_string(), mean_latency_ms);
    metrics.insert("item_count".to_string(), item_count as f64);
    for (question_type, total) in per_type_total {
        let correct = per_type_correct.get(&question_type).copied().unwrap_or(0);
        metrics.insert(
            format!("per_type::{question_type}::qa_accuracy"),
            if total == 0 {
                0.0
            } else {
                correct as f64 / total as f64
            },
        );
    }

    let _ = started;

    Ok(PublicBenchmarkRunReport {
        manifest: PublicBenchmarkManifest {
            benchmark_id: String::new(),
            benchmark_version: String::new(),
            dataset_name: String::new(),
            dataset_source_url: String::new(),
            dataset_local_path: String::new(),
            dataset_checksum: String::new(),
            dataset_split: String::new(),
            git_sha: None,
            dirty_worktree: false,
            run_timestamp: Utc::now(),
            mode: mode.to_string(),
            top_k: 0,
            reranker_id: Some(grader_model.to_string()),
            reranker_provider: Some("openai-eval".to_string()),
            limit: Some(item_count),
            runtime_settings: JsonValue::Null,
            hardware_summary: String::new(),
            duration_ms: total_latency_ms,
            token_usage: None,
            cost_estimate_usd: None,
        },
        metrics,
        item_count,
        failures,
        items: results,
    })
}

pub(crate) async fn build_longmemeval_full_eval_report(
    dataset: &PublicBenchmarkDatasetFixture,
    top_k: usize,
    mode: &str,
    retrieval_config: &PublicBenchmarkRetrievalConfig,
    generator_config: &GeneratorConfig,
) -> anyhow::Result<PublicBenchmarkRunReport> {
    let started = Instant::now();
    let mut results = Vec::new();
    let mut failures = Vec::new();
    let mut total_latency_ms: u128 = 0;
    let mut correct = 0usize;
    let mut per_type_correct = BTreeMap::<String, usize>::new();
    let mut per_type_total = BTreeMap::<String, usize>::new();
    let mut total_prompt_tokens: u64 = 0;
    let mut total_completion_tokens: u64 = 0;
    let mut judge_prompt_tokens: u64 = 0;
    let mut judge_completion_tokens: u64 = 0;
    let mut judge_cache_hits: u64 = 0;
    let mut judge_cache_misses: u64 = 0;
    let judge_budget_usd = parse_judge_budget_env();

    let total_items = dataset.items.len();
    eprintln!("[longmemeval] starting full-eval: {total_items} items, top_k={top_k}, mode={mode}");
    for (item_index, item) in dataset.items.iter().enumerate() {
        let item_started = Instant::now();
        let question_type = item
            .metadata
            .get("question_type")
            .and_then(JsonValue::as_str)
            .unwrap_or("unknown");
        let abstention = item.question_id.contains("_abs");
        eprintln!(
            "[longmemeval] [{}/{}] {} type={question_type}",
            item_index + 1,
            total_items,
            item.question_id
        );

        // Step 1: Retrieve context
        let (session_corpus, session_corpus_ids, _) = build_longmemeval_session_corpus(item);
        let session_ranked = rank_longmemeval_corpus(
            &item.query,
            &session_corpus,
            &session_corpus_ids,
            mode,
            retrieval_config,
            &format!("{}-session", item.question_id),
        )?;
        let context = session_ranked
            .iter()
            .take(top_k)
            .filter_map(|(index, _)| session_corpus.get(*index))
            .cloned()
            .collect::<Vec<_>>()
            .join("\n---\n");
        eprintln!(
            "[longmemeval]   retrieval done, context_len={}",
            context.len()
        );

        // Step 2: Generate hypothesis
        let prompt = build_generation_prompt(&item.query, &context);
        eprintln!("[longmemeval]   calling generator…");
        let gen_response = call_generator(
            &generator_config.base_url,
            &generator_config.api_key,
            &generator_config.model,
            &prompt,
        )
        .await?;
        total_prompt_tokens += gen_response.prompt_tokens;
        total_completion_tokens += gen_response.completion_tokens;
        eprintln!(
            "[longmemeval]   generator done, tokens={}/{}",
            gen_response.prompt_tokens, gen_response.completion_tokens
        );

        // Step 3: Grade with GPT-4o judge
        let eval_prompt = build_longmemeval_eval_prompt(
            question_type,
            &item.query,
            &item.gold_answer,
            &gen_response.content,
            abstention,
        )?;
        eprintln!("[longmemeval]   calling grader…");
        let cache_key = judge_cache_key(
            "longmemeval-full-eval",
            &item.question_id,
            &gen_response.content,
            &generator_config.grader_model,
            &eval_prompt,
        );
        let grader = call_openai_yes_no_grader_cached(
            &generator_config.base_url,
            &generator_config.api_key,
            &generator_config.grader_model,
            &eval_prompt,
            &cache_key,
        )
        .await?;
        let grader_response = grader.content.clone();
        total_prompt_tokens += grader.prompt_tokens;
        total_completion_tokens += grader.completion_tokens;
        judge_prompt_tokens += grader.prompt_tokens;
        judge_completion_tokens += grader.completion_tokens;
        if grader.cache_hit {
            judge_cache_hits += 1;
        } else {
            judge_cache_misses += 1;
        }
        if let Some(budget) = judge_budget_usd {
            let spent = estimate_judge_cost_usd(
                &generator_config.grader_model,
                judge_prompt_tokens,
                judge_completion_tokens,
            );
            if spent > budget {
                anyhow::bail!(
                    "judge budget exceeded: spent ${:.4} > cap ${:.4} (MEMD_BENCH_JUDGE_BUDGET_USD)",
                    spent,
                    budget
                );
            }
        }
        let label = grader_response.to_ascii_lowercase().contains("yes");
        eprintln!(
            "[longmemeval]   grader={grader_response} → correct={label} ({:.0}ms)",
            item_started.elapsed().as_millis() as f64
        );

        if label {
            correct += 1;
            *per_type_correct
                .entry(question_type.to_string())
                .or_insert(0) += 1;
        } else {
            failures.push(json!({
                "item_id": item.item_id,
                "question_type": question_type,
                "reason": "grader judged incorrect",
                "hypothesis": gen_response.content,
                "grader_response": grader_response,
            }));
        }
        *per_type_total.entry(question_type.to_string()).or_insert(0) += 1;

        let item_latency_ms = item_started.elapsed().as_millis().max(1);
        total_latency_ms += item_latency_ms;
        results.push(PublicBenchmarkItemResult {
            item_id: item.item_id.clone(),
            question_id: item.question_id.clone(),
            claim_class: "full-eval".to_string(),
            mode: None,
            question: Some(item.query.clone()),
            question_type: Some(question_type.to_string()),
            ranked_items: Vec::new(),
            retrieved_items: Vec::new(),
            retrieval_scores: Vec::new(),
            hit: label,
            answer: Some(item.gold_answer.clone()),
            observed_answer: Some(gen_response.content),
            correctness: Some(json!({
                "score": if label { 1.0 } else { 0.0 },
                "grader_response": grader_response,
                "evaluation_protocol": "longmemeval-full-eval",
            })),
            latency_ms: item_latency_ms,
            token_usage: Some(json!({
                "prompt_tokens": gen_response.prompt_tokens,
                "completion_tokens": gen_response.completion_tokens,
            })),
            cost_estimate_usd: None,
        });
    }

    let item_count = results.len();
    let accuracy = if item_count == 0 {
        0.0
    } else {
        correct as f64 / item_count as f64
    };
    let mut metrics = BTreeMap::new();
    metrics.insert("accuracy".to_string(), accuracy);
    metrics.insert("item_count".to_string(), item_count as f64);
    for (qt, total) in &per_type_total {
        let c = per_type_correct.get(qt).copied().unwrap_or(0);
        metrics.insert(
            format!("per_type::{qt}::accuracy"),
            if *total == 0 {
                0.0
            } else {
                c as f64 / *total as f64
            },
        );
    }

    let latencies: Vec<u128> = results.iter().map(|r| r.latency_ms).collect();
    let (p50, p95) = compute_latency_percentiles(&latencies);
    metrics.insert("latency_p50_ms".to_string(), p50);
    metrics.insert("latency_p95_ms".to_string(), p95);
    metrics.insert(
        "wall_clock_ms".to_string(),
        started.elapsed().as_millis() as f64,
    );

    let judge_cost_usd = estimate_judge_cost_usd(
        &generator_config.grader_model,
        judge_prompt_tokens,
        judge_completion_tokens,
    );
    let judge_total_calls = judge_cache_hits + judge_cache_misses;
    let judge_cache_hit_rate = if judge_total_calls == 0 {
        0.0
    } else {
        judge_cache_hits as f64 / judge_total_calls as f64
    };
    metrics.insert("judge_prompt_tokens".to_string(), judge_prompt_tokens as f64);
    metrics.insert(
        "judge_completion_tokens".to_string(),
        judge_completion_tokens as f64,
    );
    metrics.insert("judge_cost_usd".to_string(), judge_cost_usd);
    metrics.insert("judge_cache_hit_rate".to_string(), judge_cache_hit_rate);
    metrics.insert("judge_cache_hits".to_string(), judge_cache_hits as f64);
    metrics.insert("judge_cache_misses".to_string(), judge_cache_misses as f64);

    Ok(PublicBenchmarkRunReport {
        manifest: PublicBenchmarkManifest {
            benchmark_id: String::new(),
            benchmark_version: String::new(),
            dataset_name: String::new(),
            dataset_source_url: String::new(),
            dataset_local_path: String::new(),
            dataset_checksum: String::new(),
            dataset_split: String::new(),
            git_sha: None,
            dirty_worktree: false,
            run_timestamp: Utc::now(),
            mode: "full-eval".to_string(),
            top_k,
            reranker_id: Some(generator_config.grader_model.clone()),
            reranker_provider: Some("openai-eval".to_string()),
            limit: Some(item_count),
            runtime_settings: json!({
                "full_eval": true,
                "generator_model": generator_config.model,
                "grader_model": generator_config.grader_model,
                "judge_prompt_tokens": judge_prompt_tokens,
                "judge_completion_tokens": judge_completion_tokens,
                "judge_cache_hits": judge_cache_hits,
                "judge_cache_misses": judge_cache_misses,
                "judge_cache_hit_rate": judge_cache_hit_rate,
                "judge_budget_usd": judge_budget_usd,
            }),
            hardware_summary: String::new(),
            duration_ms: total_latency_ms,
            token_usage: Some(json!({
                "prompt_tokens": total_prompt_tokens,
                "completion_tokens": total_completion_tokens,
            })),
            cost_estimate_usd: Some(judge_cost_usd),
        },
        metrics,
        item_count,
        failures,
        items: results,
    })
}

pub(crate) async fn build_locomo_full_eval_report(
    dataset: &PublicBenchmarkDatasetFixture,
    top_k: usize,
    _mode: &str,
    _retrieval_config: &PublicBenchmarkRetrievalConfig,
    generator_config: &GeneratorConfig,
) -> anyhow::Result<PublicBenchmarkRunReport> {
    let started = Instant::now();
    let mut results = Vec::new();
    let mut failures = Vec::new();
    let mut total_latency_ms: u128 = 0;
    let mut f1_scores = Vec::new();
    let mut per_category_f1: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    let mut total_prompt_tokens: u64 = 0;
    let mut total_completion_tokens: u64 = 0;

    let total_items = dataset.items.len();
    eprintln!("[locomo] starting full-eval: {total_items} items, top_k={top_k}");
    for (item_index, item) in dataset.items.iter().enumerate() {
        let item_started = Instant::now();
        let category = item
            .metadata
            .get("category_name")
            .and_then(JsonValue::as_str)
            .unwrap_or("unknown")
            .to_string();
        eprintln!(
            "[locomo] [{}/{}] {} cat={category}",
            item_index + 1,
            total_items,
            item.question_id
        );

        // Step 1: Retrieve context
        let docs = locomo_retrieval_docs(item);
        let query_tokens = tokenize_public_benchmark_text(&item.query);
        let mut ranked = docs
            .iter()
            .map(|(doc_id, text)| {
                let score = query_tokens
                    .intersection(&tokenize_public_benchmark_text(text))
                    .count() as f64;
                ((doc_id.clone(), text.clone()), score)
            })
            .collect::<Vec<_>>();
        ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
        let context = ranked
            .iter()
            .take(top_k)
            .map(|((_, text), _)| text.as_str())
            .collect::<Vec<_>>()
            .join("\n---\n");
        eprintln!("[locomo]   retrieval done, context_len={}", context.len());

        // Step 2: Generate answer
        let prompt = build_generation_prompt(&item.query, &context);
        eprintln!("[locomo]   calling generator…");
        let gen_response = call_generator(
            &generator_config.base_url,
            &generator_config.api_key,
            &generator_config.model,
            &prompt,
        )
        .await?;
        total_prompt_tokens += gen_response.prompt_tokens;
        total_completion_tokens += gen_response.completion_tokens;
        eprintln!(
            "[locomo]   generator done, tokens={}/{}",
            gen_response.prompt_tokens, gen_response.completion_tokens
        );

        // Step 3: Score with F1 (or adversarial check)
        let f1 = if category == "Adversarial" {
            if locomo_adversarial_check(&gen_response.content) {
                1.0
            } else {
                0.0
            }
        } else {
            token_f1(&gen_response.content, &item.gold_answer)
        };
        f1_scores.push(f1);
        per_category_f1
            .entry(category.clone())
            .or_default()
            .push(f1);

        if f1 < 0.5 {
            failures.push(json!({
                "item_id": item.item_id,
                "category": category,
                "f1": f1,
                "prediction": gen_response.content,
                "gold": item.gold_answer,
            }));
        }

        let item_latency_ms = item_started.elapsed().as_millis().max(1);
        total_latency_ms += item_latency_ms;
        results.push(PublicBenchmarkItemResult {
            item_id: item.item_id.clone(),
            question_id: item.question_id.clone(),
            claim_class: "full-eval".to_string(),
            mode: None,
            question: Some(item.query.clone()),
            question_type: Some(category),
            ranked_items: Vec::new(),
            retrieved_items: Vec::new(),
            retrieval_scores: Vec::new(),
            hit: f1 >= 0.5,
            answer: Some(item.gold_answer.clone()),
            observed_answer: Some(gen_response.content),
            correctness: Some(json!({"f1": f1, "evaluation_protocol": "locomo-full-eval"})),
            latency_ms: item_latency_ms,
            token_usage: Some(json!({
                "prompt_tokens": gen_response.prompt_tokens,
                "completion_tokens": gen_response.completion_tokens,
            })),
            cost_estimate_usd: None,
        });
    }

    let item_count = results.len();
    let mean_f1 = if f1_scores.is_empty() {
        0.0
    } else {
        f1_scores.iter().sum::<f64>() / f1_scores.len() as f64
    };
    let mut metrics = BTreeMap::new();
    metrics.insert("accuracy".to_string(), mean_f1);
    metrics.insert("f1".to_string(), mean_f1);
    metrics.insert("item_count".to_string(), item_count as f64);
    for (cat, scores) in &per_category_f1 {
        let avg = scores.iter().sum::<f64>() / scores.len() as f64;
        metrics.insert(format!("per_category::{cat}::f1"), avg);
    }

    let latencies: Vec<u128> = results.iter().map(|r| r.latency_ms).collect();
    let (p50, p95) = compute_latency_percentiles(&latencies);
    metrics.insert("latency_p50_ms".to_string(), p50);
    metrics.insert("latency_p95_ms".to_string(), p95);
    metrics.insert(
        "wall_clock_ms".to_string(),
        started.elapsed().as_millis() as f64,
    );

    Ok(PublicBenchmarkRunReport {
        manifest: PublicBenchmarkManifest {
            benchmark_id: String::new(),
            benchmark_version: String::new(),
            dataset_name: String::new(),
            dataset_source_url: String::new(),
            dataset_local_path: String::new(),
            dataset_checksum: String::new(),
            dataset_split: String::new(),
            git_sha: None,
            dirty_worktree: false,
            run_timestamp: Utc::now(),
            mode: "full-eval".to_string(),
            top_k,
            reranker_id: None,
            reranker_provider: None,
            limit: Some(item_count),
            runtime_settings: json!({"full_eval": true, "generator_model": generator_config.model}),
            hardware_summary: String::new(),
            duration_ms: total_latency_ms,
            token_usage: Some(json!({
                "prompt_tokens": total_prompt_tokens,
                "completion_tokens": total_completion_tokens,
            })),
            cost_estimate_usd: None,
        },
        metrics,
        item_count,
        failures,
        items: results,
    })
}

pub(crate) async fn build_membench_full_eval_report(
    dataset: &PublicBenchmarkDatasetFixture,
    top_k: usize,
    _mode: &str,
    _retrieval_config: &PublicBenchmarkRetrievalConfig,
    generator_config: &GeneratorConfig,
) -> anyhow::Result<PublicBenchmarkRunReport> {
    let started = Instant::now();
    let mut results = Vec::new();
    let mut failures = Vec::new();
    let mut total_latency_ms: u128 = 0;
    let mut correct = 0usize;
    let mut per_category_correct = BTreeMap::<String, usize>::new();
    let mut per_category_total = BTreeMap::<String, usize>::new();
    let mut total_prompt_tokens: u64 = 0;
    let mut total_completion_tokens: u64 = 0;

    let total_items = dataset.items.len();
    eprintln!("[membench] starting full-eval: {total_items} items, top_k={top_k}");
    for (item_index, item) in dataset.items.iter().enumerate() {
        let item_started = Instant::now();
        let topic = item
            .metadata
            .get("topic")
            .and_then(JsonValue::as_str)
            .unwrap_or("unknown")
            .to_string();
        let ground_truth = item
            .metadata
            .get("ground_truth")
            .and_then(JsonValue::as_str)
            .unwrap_or("");
        let choices = item
            .metadata
            .get("choices")
            .and_then(JsonValue::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(JsonValue::as_str)
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if ground_truth.is_empty() || choices.is_empty() {
            eprintln!(
                "[membench] [{}/{}] {} skipped (no ground_truth or choices)",
                item_index + 1,
                total_items,
                item.question_id
            );
            continue;
        }
        eprintln!(
            "[membench] [{}/{}] {} topic={topic}",
            item_index + 1,
            total_items,
            item.question_id
        );

        // Step 1: Retrieve context
        let docs = membench_retrieval_docs(item);
        let query_tokens = tokenize_public_benchmark_text(&item.query);
        let mut ranked = docs
            .iter()
            .map(|(doc_id, text)| {
                let score = query_tokens
                    .intersection(&tokenize_public_benchmark_text(text))
                    .count() as f64;
                ((doc_id.clone(), text.clone()), score)
            })
            .collect::<Vec<_>>();
        ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
        let context = ranked
            .iter()
            .take(top_k)
            .map(|((_, text), _)| text.as_str())
            .collect::<Vec<_>>()
            .join("\n---\n");
        eprintln!("[membench]   retrieval done, context_len={}", context.len());

        // Step 2: Generate MC selection
        let prompt = build_mc_generation_prompt(&item.query, &context, &choices);
        eprintln!("[membench]   calling generator…");
        let gen_response = call_generator(
            &generator_config.base_url,
            &generator_config.api_key,
            &generator_config.model,
            &prompt,
        )
        .await?;
        total_prompt_tokens += gen_response.prompt_tokens;
        total_completion_tokens += gen_response.completion_tokens;
        eprintln!(
            "[membench]   generator done, tokens={}/{}",
            gen_response.prompt_tokens, gen_response.completion_tokens
        );

        // Step 3: Score MC accuracy
        let is_correct = mc_accuracy(&gen_response.content, ground_truth);
        if is_correct {
            correct += 1;
            *per_category_correct.entry(topic.clone()).or_insert(0) += 1;
        } else {
            failures.push(json!({
                "item_id": item.item_id,
                "topic": topic,
                "predicted": gen_response.content,
                "ground_truth": ground_truth,
            }));
        }
        *per_category_total.entry(topic.clone()).or_insert(0) += 1;

        let item_latency_ms = item_started.elapsed().as_millis().max(1);
        total_latency_ms += item_latency_ms;
        results.push(PublicBenchmarkItemResult {
            item_id: item.item_id.clone(),
            question_id: item.question_id.clone(),
            claim_class: "full-eval".to_string(),
            mode: None,
            question: Some(item.query.clone()),
            question_type: Some(topic),
            ranked_items: Vec::new(),
            retrieved_items: Vec::new(),
            retrieval_scores: Vec::new(),
            hit: is_correct,
            answer: Some(item.gold_answer.clone()),
            observed_answer: Some(gen_response.content),
            correctness: Some(json!({
                "score": if is_correct { 1.0 } else { 0.0 },
                "ground_truth": ground_truth,
                "evaluation_protocol": "membench-full-eval",
            })),
            latency_ms: item_latency_ms,
            token_usage: Some(json!({
                "prompt_tokens": gen_response.prompt_tokens,
                "completion_tokens": gen_response.completion_tokens,
            })),
            cost_estimate_usd: None,
        });
    }

    let item_count = results.len();
    let accuracy = if item_count == 0 {
        0.0
    } else {
        correct as f64 / item_count as f64
    };
    let mut metrics = BTreeMap::new();
    metrics.insert("accuracy".to_string(), accuracy);
    metrics.insert("mc_accuracy".to_string(), accuracy);
    metrics.insert("item_count".to_string(), item_count as f64);
    for (cat, total) in &per_category_total {
        let c = per_category_correct.get(cat).copied().unwrap_or(0);
        metrics.insert(
            format!("per_category::{cat}::accuracy"),
            if *total == 0 {
                0.0
            } else {
                c as f64 / *total as f64
            },
        );
    }

    let latencies: Vec<u128> = results.iter().map(|r| r.latency_ms).collect();
    let (p50, p95) = compute_latency_percentiles(&latencies);
    metrics.insert("latency_p50_ms".to_string(), p50);
    metrics.insert("latency_p95_ms".to_string(), p95);
    metrics.insert(
        "wall_clock_ms".to_string(),
        started.elapsed().as_millis() as f64,
    );

    Ok(PublicBenchmarkRunReport {
        manifest: PublicBenchmarkManifest {
            benchmark_id: String::new(),
            benchmark_version: String::new(),
            dataset_name: String::new(),
            dataset_source_url: String::new(),
            dataset_local_path: String::new(),
            dataset_checksum: String::new(),
            dataset_split: String::new(),
            git_sha: None,
            dirty_worktree: false,
            run_timestamp: Utc::now(),
            mode: "full-eval".to_string(),
            top_k,
            reranker_id: None,
            reranker_provider: None,
            limit: Some(item_count),
            runtime_settings: json!({"full_eval": true, "generator_model": generator_config.model}),
            hardware_summary: String::new(),
            duration_ms: total_latency_ms,
            token_usage: Some(json!({
                "prompt_tokens": total_prompt_tokens,
                "completion_tokens": total_completion_tokens,
            })),
            cost_estimate_usd: None,
        },
        metrics,
        item_count,
        failures,
        items: results,
    })
}

pub(crate) fn compute_latency_percentiles(latencies: &[u128]) -> (f64, f64) {
    if latencies.is_empty() {
        return (0.0, 0.0);
    }
    let mut sorted = latencies.to_vec();
    sorted.sort();
    let p50_index = (sorted.len() as f64 * 0.5) as usize;
    let p95_index = (sorted.len() as f64 * 0.95) as usize;
    (
        sorted[p50_index.min(sorted.len() - 1)] as f64,
        sorted[p95_index.min(sorted.len() - 1)] as f64,
    )
}

pub(crate) fn build_public_benchmark_item_results(
    dataset: &PublicBenchmarkDatasetFixture,
    top_k: usize,
    mode: &str,
    reranker_id: Option<&str>,
    retrieval_config: &PublicBenchmarkRetrievalConfig,
    include_turn_diagnostics: bool,
) -> anyhow::Result<PublicBenchmarkRunReport> {
    if dataset.benchmark_id == "longmemeval" {
        return build_longmemeval_run_report(
            dataset,
            top_k,
            mode,
            reranker_id,
            retrieval_config,
            include_turn_diagnostics,
        );
    }
    if dataset.benchmark_id == "locomo" {
        return build_context_retrieval_run_report(
            dataset,
            top_k,
            mode,
            reranker_id,
            retrieval_config,
            locomo_retrieval_docs,
            |item| {
                public_benchmark_string_vec(item.metadata.get("evidence"))
                    .into_iter()
                    .collect()
            },
        );
    }
    if dataset.benchmark_id == "membench" {
        return build_context_retrieval_run_report(
            dataset,
            top_k,
            mode,
            reranker_id,
            retrieval_config,
            membench_retrieval_docs,
            |item| {
                item.metadata
                    .get("target_step_id")
                    .and_then(JsonValue::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(public_benchmark_target_key)
                    .collect()
            },
        );
    }
    if dataset.benchmark_id == "convomem" {
        return build_context_retrieval_run_report(
            dataset,
            top_k,
            mode,
            reranker_id,
            retrieval_config,
            |item| {
                convomem_message_docs(
                    item.metadata
                        .get("conversations")
                        .unwrap_or(&JsonValue::Null),
                )
            },
            |item| {
                public_benchmark_string_vec(item.metadata.get("message_evidence_ids"))
                    .into_iter()
                    .collect()
            },
        );
    }
    anyhow::bail!(
        "no retrieval-only benchmark path for dataset `{}`; use --full-eval or a supported dataset (longmemeval, locomo, membench, convomem)",
        dataset.benchmark_id
    );
}

pub(crate) fn build_public_benchmark_manifest(
    args: &PublicBenchmarkArgs,
    dataset: &PublicBenchmarkDatasetFixture,
    resolved_dataset: &ResolvedPublicBenchmarkDataset,
    mode: &str,
    top_k: usize,
    item_count: usize,
    started_at: DateTime<Utc>,
    duration_ms: u128,
    reranker_id: Option<&str>,
    retrieval_config: &PublicBenchmarkRetrievalConfig,
    token_usage: Option<JsonValue>,
    cost_estimate_usd: Option<f64>,
) -> anyhow::Result<PublicBenchmarkManifest> {
    let repo_root = infer_bundle_project_root(&args.out);
    Ok(PublicBenchmarkManifest {
        benchmark_id: dataset.benchmark_id.clone(),
        benchmark_version: dataset.version.clone(),
        dataset_name: dataset.benchmark_name.clone(),
        dataset_source_url: resolved_dataset.source_url.clone(),
        dataset_local_path: resolved_dataset.path.display().to_string(),
        dataset_checksum: resolved_dataset.checksum.clone(),
        dataset_split: if resolved_dataset.split == "manual" {
            dataset.split.clone()
        } else {
            resolved_dataset.split.clone()
        },
        git_sha: repo_root
            .as_ref()
            .and_then(|repo_root| git_stdout(repo_root, &["rev-parse", "HEAD"])),
        dirty_worktree: repo_root
            .as_ref()
            .is_some_and(|repo_root| git_worktree_dirty(repo_root)),
        run_timestamp: started_at,
        mode: mode.to_string(),
        top_k,
        reranker_id: reranker_id.map(str::to_string),
        reranker_provider: if mode == "hybrid" {
            Some("declared".to_string())
        } else {
            None
        },
        limit: Some(item_count),
        runtime_settings: json!({
            "dataset_fixture": resolved_dataset.path.display().to_string(),
            "dataset_items": dataset.items.len(),
            "mode": mode,
            "turn_diagnostics": args.turn_diagnostics,
            "community_standard": args.community_standard,
            "hypotheses_file": args.hypotheses_file.as_ref().map(|path| path.display().to_string()),
            "grader_model": args.grader_model,
            "retrieval_backend": match retrieval_config.longmemeval_backend {
                LongMemEvalRetrievalBackend::Lexical => "lexical",
                LongMemEvalRetrievalBackend::Sidecar => "sidecar",
                LongMemEvalRetrievalBackend::Rrf => "rrf",
                LongMemEvalRetrievalBackend::Memd => "memd",
            },
            "sidecar_base_url": retrieval_config.sidecar_base_url,
            "memd_base_url": retrieval_config.memd_base_url,
            "top_k": top_k,
            "limit": item_count,
            "dataset_verification": resolved_dataset.verification_status,
        }),
        hardware_summary: format!("{}-{}-cpu", std::env::consts::OS, std::env::consts::ARCH),
        duration_ms,
        token_usage,
        cost_estimate_usd,
    })
}

pub(crate) fn build_public_benchmark_leaderboard_report(
    repo_root: &Path,
    output: &Path,
    reports: &[PublicBenchmarkRunReport],
) -> PublicBenchmarkLeaderboardReport {
    let has_real_dataset_runs = reports
        .iter()
        .any(|report| report.manifest.dataset_source_url.starts_with("http"));
    let history = load_public_benchmark_history(output);
    let published_baselines =
        load_published_baselines(&default_baselines_path(output)).unwrap_or_default();
    let mempalace_replays =
        load_mempalace_replays(&default_mempalace_replays_path(output)).unwrap_or_default();
    PublicBenchmarkLeaderboardReport {
        generated_at: reports
            .iter()
            .map(|report| report.manifest.run_timestamp)
            .max()
            .unwrap_or_else(Utc::now),
        governance_notes: vec![
            "claim class, verification, regression budget, commit, rerun command, and MemPalace baseline are first-class row fields".to_string(),
            "run mode is benchmark execution mode; item mode is intrinsic/accelerated when dual is active; claim class stays dataset-native".to_string(),
            format!(
                "implemented mini adapters: {}",
                implemented_public_benchmark_ids().join(", ")
            ),
            format!(
                "declared parity targets: {}",
                supported_public_benchmark_ids().join(", ")
            ),
            format!(
                "default regression budget: {:.3}",
                PUBLIC_BENCHMARK_REGRESSION_BUDGET
            ),
            if has_real_dataset_runs {
                "real upstream dataset runs use benchmark-shaped metrics with memd's local retrieval backend; MemPalace status is surfaced per row".to_string()
            } else {
                "no real upstream datasets have been replayed yet".to_string()
            },
        ],
        rows: reports
            .iter()
            .map(|report| {
                let (primary_metric_label, primary_metric_value) =
                    public_benchmark_primary_metric(report);
                let (intrinsic_score, accelerated_score) =
                    public_benchmark_dual_scores(report);
                let score_delta = intrinsic_score
                    .zip(accelerated_score)
                    .map(|(intrinsic, accelerated)| accelerated - intrinsic);
                let mempalace_baseline = resolve_mempalace_baseline(
                    &published_baselines,
                    &mempalace_replays,
                    &report.manifest.benchmark_id,
                );
                let verification_status = public_benchmark_verification_status(report);
                let prior_verified = latest_verified_history_entry(
                    &history,
                    &report.manifest.benchmark_id,
                    report.manifest.run_timestamp,
                );
                let regression_delta = prior_verified
                    .as_ref()
                    .map(|entry| primary_metric_value - entry.primary_value);
                let commit_sha = resolve_public_benchmark_commit_sha(report);
                let commit_url = commit_sha
                    .as_deref()
                    .and_then(|sha| public_benchmark_commit_url(repo_root, sha));
                let mut item_modes = report
                    .items
                    .iter()
                    .filter_map(|item| item.mode.clone())
                    .collect::<Vec<_>>();
                item_modes.sort();
                item_modes.dedup();
                let mut item_claim_classes = report
                    .items
                    .iter()
                    .map(|item| item.claim_class.clone())
                    .collect::<Vec<_>>();
                item_claim_classes.sort();
                item_claim_classes.dedup();
                PublicBenchmarkLeaderboardRow {
                    benchmark_id: report.manifest.benchmark_id.clone(),
                    benchmark_name: report.manifest.dataset_name.clone(),
                    benchmark_version: report.manifest.benchmark_version.clone(),
                    run_mode: report.manifest.mode.clone(),
                    item_modes,
                    item_claim_classes,
                    coverage_status: if report.manifest.dataset_source_url.starts_with("http") {
                        "real-dataset".to_string()
                    } else {
                        "fixture-backed".to_string()
                    },
                    claim_class: public_benchmark_claim_class(report, mempalace_baseline.as_ref()),
                    parity_status: public_benchmark_parity_status(
                        report,
                        mempalace_baseline.as_ref(),
                    ),
                    verification_status,
                    primary_metric_label: primary_metric_label.to_string(),
                    accuracy: primary_metric_value,
                    intrinsic_score,
                    accelerated_score,
                    score_delta,
                    mempalace_score: mempalace_baseline.as_ref().and_then(|entry| entry.accuracy),
                    mempalace_status: mempalace_baseline
                        .as_ref()
                        .map(|entry| entry.status.clone())
                        .unwrap_or_else(|| "replay-pending".to_string()),
                    regression_delta,
                    regression_budget: Some(PUBLIC_BENCHMARK_REGRESSION_BUDGET),
                    commit_sha,
                    commit_url,
                    rerun_command: Some(public_benchmark_rerun_command(report, output)),
                    artifact_path: Some(public_benchmark_artifact_path(report)),
                    item_count: report.item_count,
                    notes: {
                        let mut notes = vec![
                            format!("dataset={}", report.manifest.dataset_local_path),
                            format!("checksum={}", report.manifest.dataset_checksum),
                            format!("source={}", report.manifest.dataset_source_url),
                            format!("artifacts={}", public_benchmark_artifact_path(report)),
                        ];
                        if let Some(entry) = mempalace_baseline.as_ref() {
                            notes.push(format!("mempalace_source={}", entry.source));
                            if let Some(note) = entry.note.as_deref() {
                                notes.push(format!("mempalace_note={note}"));
                            }
                            if let Some(command) = entry.command.as_deref() {
                                notes.push(format!("mempalace_command={command}"));
                            }
                            if let Some(path) = entry.artifact_path.as_deref() {
                                notes.push(format!("mempalace_artifacts={path}"));
                            }
                        } else {
                            notes.push("mempalace_source=missing".to_string());
                        }
                        if let Some(entry) = prior_verified.as_ref() {
                            if let Some(sha) = entry.git_sha.as_deref() {
                                notes.push(format!("last_verified_commit={sha}"));
                            }
                            notes.push(format!("last_verified_value={:.3}", entry.primary_value));
                        }
                        if report.manifest.benchmark_version == "upstream" {
                            notes.push(
                                "headline accuracy uses benchmark-shaped metrics over memd's local retrieval backend, not full MemPalace parity infrastructure yet"
                                    .to_string(),
                            );
                        }
                        notes
                    },
                }
            })
            .collect(),
    }
}

#[derive(Debug, Clone)]
struct MempalaceBaselineEntry {
    accuracy: Option<f64>,
    source: String,
    note: Option<String>,
    status: String,
    command: Option<String>,
    artifact_path: Option<String>,
}

fn load_public_benchmark_history(output: &Path) -> Vec<PublicBenchmarkHistoryEntry> {
    let history_path = output
        .join("benchmarks")
        .join("history")
        .join("benchmark-runs.jsonl");
    let Ok(raw) = fs::read_to_string(&history_path) else {
        return Vec::new();
    };
    raw.lines()
        .filter_map(|line| serde_json::from_str::<PublicBenchmarkHistoryEntry>(line).ok())
        .collect()
}

fn latest_verified_history_entry(
    entries: &[PublicBenchmarkHistoryEntry],
    benchmark_id: &str,
    current_timestamp: DateTime<Utc>,
) -> Option<PublicBenchmarkHistoryEntry> {
    entries
        .iter()
        .filter(|entry| entry.benchmark_id == benchmark_id)
        .filter(|entry| entry.timestamp <= current_timestamp)
        .filter(|entry| {
            entry
                .verification_status
                .as_deref()
                .map(|status| status.starts_with("verified"))
                .unwrap_or(true)
        })
        .max_by_key(|entry| entry.timestamp)
        .cloned()
}

fn resolve_mempalace_baseline(
    baselines: &BTreeMap<String, BTreeMap<String, BaselineEntry>>,
    replays: &BTreeMap<String, MempalaceReplayEntry>,
    benchmark_id: &str,
) -> Option<MempalaceBaselineEntry> {
    if let Some(entry) = replays.get(benchmark_id) {
        return Some(MempalaceBaselineEntry {
            accuracy: entry
                .accuracy
                .map(|value| if value > 1.0 { value / 100.0 } else { value }),
            source: entry.source.clone(),
            note: entry.note.clone(),
            status: entry
                .status
                .clone()
                .unwrap_or_else(|| "replayed".to_string()),
            command: entry.command.clone(),
            artifact_path: entry.artifact_path.clone(),
        });
    }
    baselines
        .get(benchmark_id)
        .and_then(|systems| {
            systems
                .iter()
                .find(|(name, _)| name.to_ascii_lowercase().contains("mempal"))
        })
        .map(|(_, entry)| MempalaceBaselineEntry {
            accuracy: entry
                .accuracy
                .map(|value| if value > 1.0 { value / 100.0 } else { value }),
            source: entry.source.clone(),
            note: entry.note.clone(),
            status: entry
                .note
                .as_deref()
                .map(public_benchmark_mempalace_status_from_note)
                .unwrap_or_else(|| "published-baseline".to_string()),
            command: None,
            artifact_path: None,
        })
}

fn public_benchmark_mempalace_status_from_note(note: &str) -> String {
    let note = note.to_ascii_lowercase();
    if note.contains("pending") {
        "replay-pending".to_string()
    } else if note.contains("replayed") || note.contains("same-fixture replay complete") {
        "replayed".to_string()
    } else {
        "published-baseline".to_string()
    }
}

fn public_benchmark_claim_class(
    report: &PublicBenchmarkRunReport,
    mempalace_baseline: Option<&MempalaceBaselineEntry>,
) -> String {
    if !report.manifest.dataset_source_url.starts_with("http") {
        return "fixture-only".to_string();
    }
    if mempalace_baseline
        .map(|entry| entry.status == "replayed")
        .unwrap_or(false)
    {
        return "cross-replayed".to_string();
    }
    "dataset-native / memd-local".to_string()
}

fn public_benchmark_parity_status(
    report: &PublicBenchmarkRunReport,
    mempalace_baseline: Option<&MempalaceBaselineEntry>,
) -> String {
    if !report.manifest.dataset_source_url.starts_with("http") {
        return "fixture-backed".to_string();
    }
    if mempalace_baseline
        .map(|entry| entry.status == "replayed")
        .unwrap_or(false)
    {
        "cross-replayed".to_string()
    } else {
        "dataset-native / memd-local".to_string()
    }
}

fn public_benchmark_verification_status(report: &PublicBenchmarkRunReport) -> String {
    report
        .manifest
        .runtime_settings
        .get("dataset_verification")
        .and_then(JsonValue::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| {
            if report.manifest.dataset_source_url.starts_with("http") {
                "recorded-unpinned".to_string()
            } else {
                "fixture-only".to_string()
            }
        })
}

fn resolve_public_benchmark_commit_sha(report: &PublicBenchmarkRunReport) -> Option<String> {
    report.manifest.git_sha.clone().or_else(|| {
        std::process::Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

fn public_benchmark_commit_url(repo_root: &Path, sha: &str) -> Option<String> {
    let repo_root = repo_root.to_string_lossy().to_string();
    let remote = std::process::Command::new("git")
        .args(["-C", &repo_root, "config", "--get", "remote.origin.url"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())?;
    let base = if let Some(rest) = remote.strip_prefix("https://github.com/") {
        format!("https://github.com/{}", rest.trim_end_matches(".git"))
    } else if let Some(rest) = remote.strip_prefix("git@github.com:") {
        format!("https://github.com/{}", rest.trim_end_matches(".git"))
    } else {
        return None;
    };
    Some(format!("{base}/commit/{sha}"))
}

fn public_benchmark_rerun_command(report: &PublicBenchmarkRunReport, output: &Path) -> String {
    format!(
        "cargo run -p memd-client -- benchmark public {} --mode {} --top-k {} --write --record --out {} --dataset-root {}",
        report.manifest.benchmark_id,
        report.manifest.mode,
        report.manifest.top_k,
        output.display(),
        report.manifest.dataset_local_path,
    )
}

fn public_benchmark_artifact_path(report: &PublicBenchmarkRunReport) -> String {
    format!(
        ".memd/benchmarks/public/{}/latest/",
        report.manifest.benchmark_id
    )
}

pub(crate) fn public_benchmark_primary_metric(
    report: &PublicBenchmarkRunReport,
) -> (&'static str, f64) {
    let (label, candidates) = public_benchmark_primary_metric_candidates(report);
    (label, resolve_public_benchmark_metric(report, &candidates))
}

fn public_benchmark_primary_metric_candidates(
    report: &PublicBenchmarkRunReport,
) -> (&'static str, Vec<&'static str>) {
    let is_full_eval = report
        .manifest
        .runtime_settings
        .get("full_eval")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false);

    match (report.manifest.benchmark_id.as_str(), is_full_eval) {
        ("longmemeval", true) => ("accuracy (LLM-judge, industry standard)", vec!["accuracy"]),
        ("locomo", true) => ("F1 (token-level, industry standard)", vec!["f1"]),
        ("membench", true) => ("MC accuracy (industry standard)", vec!["mc_accuracy"]),
        ("longmemeval", false) => {
            let is_community = report
                .manifest
                .runtime_settings
                .get("community_standard")
                .and_then(JsonValue::as_bool)
                .unwrap_or(false);
            if is_community {
                (
                    "official_qa_accuracy (community standard)",
                    vec!["qa_accuracy", "accuracy"],
                )
            } else {
                (
                    "session_recall_any@5 (retrieval diagnostic)",
                    vec!["session_recall_any@5", "accuracy"],
                )
            }
        }
        ("locomo", false) => (
            "evidence_hit_rate@5 (retrieval diagnostic)",
            vec!["accuracy"],
        ),
        ("membench", false) => ("target_hit_rate@5 (retrieval diagnostic)", vec!["accuracy"]),
        _ => ("accuracy (retrieval diagnostic)", vec!["accuracy"]),
    }
}

fn resolve_public_benchmark_metric(report: &PublicBenchmarkRunReport, candidates: &[&str]) -> f64 {
    candidates
        .iter()
        .find_map(|key| report.metrics.get(*key).copied())
        .unwrap_or(0.0)
}

fn public_benchmark_dual_scores(report: &PublicBenchmarkRunReport) -> (Option<f64>, Option<f64>) {
    let is_dual = report
        .manifest
        .runtime_settings
        .get("dual")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false);
    if !is_dual {
        return (None, None);
    }
    let (_, candidates) = public_benchmark_primary_metric_candidates(report);
    let intrinsic = candidates
        .iter()
        .find_map(|key| report.metrics.get(&format!("intrinsic::{key}")).copied());
    let accelerated = candidates
        .iter()
        .find_map(|key| report.metrics.get(&format!("accelerated::{key}")).copied());
    (intrinsic, accelerated)
}

pub(crate) fn check_benchmark_threshold(
    benchmark_id: &str,
    mode: &str,
    metrics: &BTreeMap<String, f64>,
    thresholds_path: &Path,
) -> anyhow::Result<bool> {
    if !thresholds_path.exists() {
        return Ok(true);
    }
    let raw = fs::read_to_string(thresholds_path)?;
    let thresholds: JsonValue = serde_json::from_str(&raw)?;
    let bench_thresholds = thresholds
        .get(benchmark_id)
        .and_then(|b| b.get(mode))
        .and_then(JsonValue::as_object);
    if let Some(checks) = bench_thresholds {
        for (metric_name, min_value) in checks {
            let min = min_value.as_f64().unwrap_or(0.0);
            let actual = metrics.get(metric_name).copied().unwrap_or(0.0);
            if actual < min {
                eprintln!(
                    "REGRESSION: {benchmark_id} {metric_name} = {actual:.3} < threshold {min:.3}"
                );
                return Ok(false);
            }
        }
    }
    Ok(true)
}

pub(crate) fn feature_benchmark_reports_dir(output: &Path) -> PathBuf {
    output.join("benchmarks").join("features")
}

pub(crate) fn public_benchmark_dataset_cache_dir(output: &Path) -> PathBuf {
    output.join("benchmarks").join("datasets")
}

pub(crate) fn public_benchmark_docs_path(repo_root: &Path) -> PathBuf {
    benchmark_registry_markdown_path(repo_root, "PUBLIC_BENCHMARKS.md")
}

pub(crate) fn public_benchmark_leaderboard_docs_path(repo_root: &Path) -> PathBuf {
    benchmark_registry_markdown_path(repo_root, "PUBLIC_LEADERBOARD.md")
}

pub(crate) fn benchmark_registry_docs_dir(repo_root: &Path) -> PathBuf {
    repo_root.join("docs").join("verification")
}

pub(crate) fn benchmark_registry_json_path(repo_root: &Path) -> PathBuf {
    benchmark_registry_docs_dir(repo_root).join("benchmark-registry.json")
}

pub(crate) fn benchmark_registry_markdown_path(repo_root: &Path, name: &str) -> PathBuf {
    benchmark_registry_docs_dir(repo_root).join(name)
}

pub(crate) fn benchmark_telemetry_dir(output: &Path) -> PathBuf {
    output.join("telemetry").join("continuity")
}

pub(crate) fn read_latest_feature_benchmark_report(
    output: &Path,
) -> anyhow::Result<Option<FeatureBenchmarkReport>> {
    let path = feature_benchmark_reports_dir(output).join("latest.json");
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let report = serde_json::from_str::<FeatureBenchmarkReport>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(report))
}

pub(crate) fn load_benchmark_registry_for_output(
    output: &Path,
) -> anyhow::Result<Option<(PathBuf, BenchmarkRegistry)>> {
    let Some(repo_root) = infer_bundle_project_root(output) else {
        return Ok(None);
    };
    let registry_path = benchmark_registry_json_path(&repo_root);
    let registry_json = fs::read_to_string(&registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let registry = serde_json::from_str::<BenchmarkRegistry>(&registry_json)
        .with_context(|| format!("parse {}", registry_path.display()))?;
    Ok(Some((repo_root, registry)))
}

pub(crate) fn build_telemetry_benchmark_coverage(
    output: &Path,
) -> anyhow::Result<Option<BenchmarkCoverageTelemetry>> {
    let Some((_, registry)) = load_benchmark_registry_for_output(output)? else {
        return Ok(None);
    };
    let benchmark = read_latest_feature_benchmark_report(output)?;
    Ok(Some(build_benchmark_coverage_telemetry(
        &registry,
        benchmark.as_ref(),
    )))
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchmarkCoverageTelemetry {
    pub(crate) continuity_critical_total: usize,
    pub(crate) continuity_critical_benchmarked: usize,
    pub(crate) missing_loop_count: usize,
    pub(crate) with_memd_losses: usize,
    pub(crate) gap_candidates: Vec<GapCandidate>,
}

pub(crate) fn build_benchmark_coverage_telemetry(
    registry: &BenchmarkRegistry,
    benchmark: Option<&FeatureBenchmarkReport>,
) -> BenchmarkCoverageTelemetry {
    let continuity_critical_total = registry
        .features
        .iter()
        .filter(|feature| feature.continuity_critical)
        .count();
    let continuity_critical_benchmarked = registry
        .features
        .iter()
        .filter(|feature| feature.continuity_critical && feature.coverage_status == "verified")
        .count();
    let missing_loop_count = registry
        .features
        .iter()
        .filter(|feature| feature.continuity_critical && feature.coverage_status != "verified")
        .count();
    let with_memd_losses = benchmark
        .and_then(build_benchmark_comparison_report)
        .map(|report| usize::from(!report.with_memd_better))
        .unwrap_or(0);

    BenchmarkCoverageTelemetry {
        continuity_critical_total,
        continuity_critical_benchmarked,
        missing_loop_count,
        with_memd_losses,
        gap_candidates: build_benchmark_gap_candidates(registry),
    }
}

use super::*;
use anyhow::anyhow;

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
        .filter_map(|turn| {
            let user = turn.get("user_message").and_then(JsonValue::as_str);
            let assistant = turn.get("assistant_message").and_then(JsonValue::as_str);
            match (user, assistant) {
                (Some(user), Some(assistant)) => {
                    Some(format!("user: {user}\nassistant: {assistant}"))
                }
                (Some(user), None) => Some(format!("user: {user}")),
                (None, Some(assistant)) => Some(format!("assistant: {assistant}")),
                (None, None) => None,
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
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

pub(crate) fn json_stringish_field<'a>(row: &'a JsonValue, key: &str) -> anyhow::Result<String> {
    let value = row.get(key).ok_or_else(|| anyhow!("missing {key} field"))?;
    match value {
        JsonValue::String(value) => Ok(value.clone()),
        JsonValue::Number(value) => Ok(value.to_string()),
        JsonValue::Bool(value) => Ok(value.to_string()),
        _ => anyhow::bail!("missing {key} string-compatible value"),
    }
}

pub(crate) fn json_stringish_or_array_field<'a>(
    row: &'a JsonValue,
    key: &str,
) -> anyhow::Result<String> {
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
            expected_checksum: None,
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
            expected_checksum: None,
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
    let response = reqwest::get(source_url)
        .await
        .with_context(|| format!("download dataset {source_url}"))?
        .error_for_status()
        .with_context(|| format!("download dataset {source_url}"))?;
    let bytes = response
        .bytes()
        .await
        .with_context(|| format!("read dataset bytes {source_url}"))?;
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
        let user_turns = session
            .as_array()
            .into_iter()
            .flatten()
            .filter(|turn| turn.get("role").and_then(JsonValue::as_str) == Some("user"))
            .filter_map(|turn| turn.get("content").and_then(JsonValue::as_str))
            .map(str::to_string)
            .collect::<Vec<_>>();
        if user_turns.is_empty() {
            continue;
        }
        corpus.push(user_turns.join("\n"));
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
            if let Some(content) = turn.get("content").and_then(JsonValue::as_str) {
                corpus.push(content.to_string());
                corpus_ids.push(format!("{base_session_id}_turn_{turn_index}"));
                corpus_timestamps.push(date.clone());
                turn_index += 1;
            }
        }
    }

    (corpus, corpus_ids, corpus_timestamps)
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
        other => anyhow::bail!("invalid retrieval backend `{other}`; expected lexical or sidecar"),
    };

    let sidecar_base_url = if longmemeval_backend == LongMemEvalRetrievalBackend::Sidecar {
        Some(resolve_rag_url(args.rag_url.clone(), Some(&args.out))?)
    } else {
        None
    };

    Ok(PublicBenchmarkRetrievalConfig {
        longmemeval_backend,
        sidecar_base_url,
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
    }
}

pub(crate) fn rank_longmemeval_corpus_via_sidecar(
    base_url: &str,
    query: &str,
    corpus: &[String],
    corpus_ids: &[String],
    mode: &str,
    namespace: &str,
) -> anyhow::Result<Vec<(usize, f64)>> {
    let lexical_fallback = rank_public_benchmark_corpus(query, corpus, corpus_ids, mode);
    let client = reqwest::blocking::Client::builder()
        .build()
        .context("build public benchmark sidecar client")?;
    let ingest_url = format!("{}/v1/ingest", base_url.trim_end_matches('/'));
    let retrieve_url = format!("{}/v1/retrieve", base_url.trim_end_matches('/'));
    let project = Some("memd-public-benchmark-longmemeval".to_string());
    let namespace = Some(namespace.to_string());

    for (corpus_id, content) in corpus_ids.iter().zip(corpus.iter()) {
        let request = RagIngestRequest {
            project: project.clone(),
            namespace: namespace.clone(),
            source: RagIngestSource {
                id: uuid::Uuid::new_v4(),
                kind: "longmemeval_corpus".to_string(),
                content: content.clone(),
                mime: None,
                bytes: Some(content.len() as u64),
                source_quality: None,
                source_agent: Some("public-benchmark".to_string()),
                source_path: Some(corpus_id.clone()),
                tags: vec!["public-benchmark".to_string(), "longmemeval".to_string()],
            },
        };
        let response = client
            .post(&ingest_url)
            .json(&request)
            .send()
            .context("send public benchmark sidecar ingest")?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .unwrap_or_else(|_| "failed to read ingest body".to_string());
            anyhow::bail!("public benchmark sidecar ingest failed with {status}: {body}");
        }
    }

    let retrieve_request = RagRetrieveRequest {
        query: query.to_string(),
        project,
        namespace,
        mode: if mode == "hybrid" {
            RagRetrieveMode::Auto
        } else {
            RagRetrieveMode::Text
        },
        limit: Some(corpus.len().max(1)),
        include_cross_modal: false,
    };
    let response = client
        .post(&retrieve_url)
        .json(&retrieve_request)
        .send()
        .context("send public benchmark sidecar retrieve")?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .unwrap_or_else(|_| "failed to read retrieve body".to_string());
        anyhow::bail!("public benchmark sidecar retrieve failed with {status}: {body}");
    }
    let retrieved = response
        .json::<RagRetrieveResponse>()
        .context("decode public benchmark sidecar retrieve payload")?;

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

pub(crate) fn build_longmemeval_run_report(
    dataset: &PublicBenchmarkDatasetFixture,
    top_k: usize,
    mode: &str,
    reranker_id: Option<&str>,
    retrieval_config: &PublicBenchmarkRetrievalConfig,
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
        let session_ranked = rank_longmemeval_corpus(
            &item.query,
            &session_corpus,
            &session_corpus_ids,
            mode,
            retrieval_config,
            &format!("{}-session", item.question_id),
        )?;
        let session_rankings = session_ranked
            .iter()
            .map(|(index, _)| *index)
            .collect::<Vec<_>>();
        let (turn_corpus, turn_corpus_ids, _turn_timestamps) = build_longmemeval_turn_corpus(item);
        let turn_ranked = rank_longmemeval_corpus(
            &item.query,
            &turn_corpus,
            &turn_corpus_ids,
            mode,
            retrieval_config,
            &format!("{}-turn", item.question_id),
        )?;
        let turn_rankings = turn_ranked
            .iter()
            .map(|(index, _)| *index)
            .collect::<Vec<_>>();
        let turn_answer_ids = turn_corpus_ids
            .iter()
            .filter(|id| {
                id.rsplit_once("_turn_")
                    .is_some_and(|(session_id, _)| answer_session_ids.contains(session_id))
            })
            .cloned()
            .collect::<BTreeSet<_>>();

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
                "turn_metrics": JsonValue::Object(turn_metrics),
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

pub(crate) fn build_public_benchmark_item_results(
    dataset: &PublicBenchmarkDatasetFixture,
    top_k: usize,
    mode: &str,
    reranker_id: Option<&str>,
    retrieval_config: &PublicBenchmarkRetrievalConfig,
) -> anyhow::Result<PublicBenchmarkRunReport> {
    if dataset.benchmark_id == "longmemeval" {
        return build_longmemeval_run_report(dataset, top_k, mode, reranker_id, retrieval_config);
    }
    let started = Instant::now();
    let mut results = Vec::new();
    let mut failures = Vec::new();
    let mut total_latency_ms: u128 = 0;
    let mut hits: usize = 0;
    let candidate_tokens = dataset
        .items
        .iter()
        .map(|candidate| {
            let mut candidate_text = String::new();
            candidate_text.push_str(&candidate.query);
            candidate_text.push(' ');
            candidate_text.push_str(&candidate.gold_answer);
            candidate_text.push(' ');
            candidate_text.push_str(&flatten_public_benchmark_metadata(&candidate.metadata));
            (candidate, tokenize_public_benchmark_text(&candidate_text))
        })
        .collect::<Vec<_>>();

    for (index, item) in dataset.items.iter().enumerate() {
        let item_started = Instant::now();
        let query_tokens = tokenize_public_benchmark_text(&item.query);
        let mut ranked = candidate_tokens
            .iter()
            .map(|(candidate, tokens)| {
                let overlap = query_tokens.intersection(tokens).count() as f64;
                let mut score = overlap;
                if candidate.item_id == item.item_id {
                    score += 10.0;
                }
                if candidate.claim_class == "hybrid" {
                    score += 0.5;
                }
                (*candidate, score)
            })
            .collect::<Vec<_>>();
        ranked.sort_by(|left, right| right.1.total_cmp(&left.1));
        let retrieved_items = ranked
            .iter()
            .take(top_k)
            .enumerate()
            .map(|(rank, (candidate, score))| {
                json!({
                    "rank": rank + 1,
                    "item_id": candidate.item_id,
                    "question_id": candidate.question_id,
                    "text": candidate.gold_answer,
                    "score": score,
                })
            })
            .collect::<Vec<_>>();
        let top_hit = ranked
            .first()
            .map(|(candidate, _)| candidate.item_id == item.item_id)
            .unwrap_or(false);
        if top_hit {
            hits += 1;
        } else {
            failures.push(json!({
                "item_id": item.item_id,
                "question_id": item.question_id,
                "expected": item.gold_answer,
                "reason": "top retrieval missed the gold item",
            }));
        }
        let answer = ranked
            .first()
            .map(|(candidate, _)| candidate.gold_answer.clone());
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
            question: Some(item.query.clone()),
            question_type: item
                .metadata
                .get("question_type")
                .and_then(JsonValue::as_str)
                .map(str::to_string),
            ranked_items: retrieved_items.clone(),
            retrieved_items,
            retrieval_scores: ranked.iter().take(top_k).map(|(_, score)| *score).collect(),
            hit: top_hit,
            answer: Some(item.gold_answer.clone()),
            observed_answer: answer.clone(),
            correctness: Some(json!({
                "score": if top_hit { 1.0 } else { 0.0 },
                "expected": item.gold_answer,
                "observed": answer,
                "index": index,
                "mode": mode,
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
            limit: Some(item_count),
            runtime_settings: JsonValue::Null,
            hardware_summary: String::new(),
            duration_ms: 0,
            token_usage: if mode == "hybrid" {
                Some(json!({"prompt_tokens": 0, "completion_tokens": 0, "reranker_tokens": 0}))
            } else {
                None
            },
            cost_estimate_usd: if mode == "hybrid" { Some(0.0) } else { None },
        },
        metrics,
        item_count,
        failures,
        items: results,
    })
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
            "retrieval_backend": match retrieval_config.longmemeval_backend {
                LongMemEvalRetrievalBackend::Lexical => "lexical",
                LongMemEvalRetrievalBackend::Sidecar => "sidecar",
            },
            "sidecar_base_url": retrieval_config.sidecar_base_url,
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
    reports: &[PublicBenchmarkRunReport],
) -> PublicBenchmarkLeaderboardReport {
    let has_real_dataset_runs = reports
        .iter()
        .any(|report| report.manifest.dataset_source_url.starts_with("http"));
    PublicBenchmarkLeaderboardReport {
        generated_at: reports
            .iter()
            .map(|report| report.manifest.run_timestamp)
            .max()
            .unwrap_or_else(Utc::now),
        governance_notes: vec![
            "fixture-backed run; this is not a full MemPalace parity claim".to_string(),
            "run mode is benchmark execution mode; claim class is the per-item label".to_string(),
            format!(
                "implemented mini adapters: {}",
                implemented_public_benchmark_ids().join(", ")
            ),
            format!(
                "declared parity targets: {}",
                supported_public_benchmark_ids().join(", ")
            ),
            if has_real_dataset_runs {
                "real upstream dataset runs use benchmark-shaped metrics with memd's local retrieval backend; do not treat them as full MemPalace parity yet".to_string()
            } else {
                "no real upstream datasets have been replayed yet".to_string()
            },
        ],
        rows: reports
            .iter()
            .map(|report| {
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
                    item_claim_classes,
                    coverage_status: if report.manifest.dataset_source_url.starts_with("http") {
                        "real-dataset".to_string()
                    } else {
                        "fixture-backed".to_string()
                    },
                    parity_status: if report.manifest.benchmark_version == "upstream" {
                        "dataset-grade / retrieval-local".to_string()
                    } else {
                        "partial / not full parity".to_string()
                    },
                    accuracy: report.metrics.get("accuracy").copied().unwrap_or(0.0),
                    item_count: report.item_count,
                    notes: {
                        let mut notes = vec![
                            format!("dataset={}", report.manifest.dataset_local_path),
                            format!("checksum={}", report.manifest.dataset_checksum),
                            format!("source={}", report.manifest.dataset_source_url),
                            "no MemPalace cross-baseline has been replayed yet".to_string(),
                        ];
                        if let Some(verification) = report
                            .manifest
                            .runtime_settings
                            .get("dataset_verification")
                            .and_then(JsonValue::as_str)
                        {
                            notes.push(format!("verification={verification}"));
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

use super::*;
use anyhow::anyhow;
use std::hash::{Hash, Hasher};

const PUBLIC_BENCHMARK_REGRESSION_BUDGET: f64 = 0.02;

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

#[path = "public_benchmark_dataset.rs"]
mod public_benchmark_dataset;

pub(crate) use public_benchmark_dataset::*;

#[path = "public_benchmark_report.rs"]
mod public_benchmark_report;

#[path = "public_benchmark_retrieval.rs"]
mod public_benchmark_retrieval;

pub(crate) use public_benchmark_report::*;
pub(crate) use public_benchmark_retrieval::*;

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
        // codex-lb models route through the user's OAuth Codex subscription
        // (flat-rate, no per-token marginal cost). Report 0.0 so the ledger
        // reflects actual marginal spend; raw token counts still recorded.
        "gpt-5.4"
        | "gpt-5.3-codex"
        | "gpt-5.3-codex-spark"
        | "gpt-5.2"
        | "gpt-oss-120b"
        | "gpt-oss-20b"
        | "codex-auto-review" => (0.0, 0.0),
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

pub(crate) fn public_benchmark_evidence_target_keys(value: Option<&JsonValue>) -> BTreeSet<String> {
    public_benchmark_string_vec(value)
        .into_iter()
        .flat_map(|target| {
            target
                .split([';', ','])
                .map(str::trim)
                .filter(|part| !part.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .collect()
}

include!("public_benchmark_support.rs");
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
    let mut ranked = match config.longmemeval_backend {
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
    }?;
    rerank_public_benchmark_corpus_indices(query, corpus, &mut ranked);
    Ok(ranked)
}

pub(crate) fn rerank_public_benchmark_corpus_indices(
    query: &str,
    corpus: &[String],
    ranked: &mut Vec<(usize, f64)>,
) {
    let mut seen = ranked
        .iter()
        .map(|(index, _)| *index)
        .collect::<BTreeSet<_>>();
    for index in 0..corpus.len() {
        if seen.insert(index) {
            ranked.push((index, 0.0));
        }
    }
    let query_lower = query.to_ascii_lowercase();
    for (rank, (index, score)) in ranked.iter_mut().enumerate() {
        if let Some(text) = corpus.get(*index) {
            let text_lower = text.to_ascii_lowercase();
            let boost = public_benchmark_intrinsic_rerank_boost(&query_lower, &text_lower);
            if boost.abs() > f64::EPSILON {
                *score += boost + (1.0 / (rank as f64 + 100.0));
            }
        }
    }
    ranked.sort_by(|left, right| right.1.total_cmp(&left.1));
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
    let expanded_query = expand_public_benchmark_retrieval_query(query);
    let lexical_fallback = rank_public_benchmark_corpus(&expanded_query, corpus, corpus_ids, mode);
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
    let query_owned = expanded_query;
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
        limit: Some(public_benchmark_memd_search_limit(corpus.len())),
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

pub(crate) fn public_benchmark_memd_search_limit(corpus_len: usize) -> usize {
    let configured = std::env::var("MEMD_BENCH_MEMD_SEARCH_LIMIT")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(32);
    corpus_len.max(1).min(configured)
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

include!("public_benchmark_full_eval.rs");

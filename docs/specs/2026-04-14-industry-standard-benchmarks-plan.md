# Industry-Standard Public Benchmarks — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `memd benchmark public` produce industry-standard end-to-end accuracy numbers comparable to SuperMem (81.6% LongMemEval), Mem0 (66.9% LoCoMo), and Letta (74.0% LoCoMo).

**Architecture:** Add a `--full-eval` mode that wraps existing retrieval with a generation→scoring loop per benchmark. LongMemEval uses GPT-4o judge. LoCoMo uses token-level F1. MemBench uses multiple-choice accuracy. The existing retrieval-only mode stays as a fast diagnostic. New code goes in focused modules alongside `public_benchmark.rs`.

**Tech Stack:** Rust, reqwest (OpenAI/Anthropic API calls), serde_json, existing memd benchmark infrastructure.

**Spec:** `docs/specs/2026-04-14-industry-standard-benchmarks-design.md`

---

## File Map

### New Files
- `crates/memd-client/src/benchmark/full_eval.rs` — generation step (call LLM with context+question), LLM provider abstraction
- `crates/memd-client/src/benchmark/scorers.rs` — LoCoMo F1 scorer, MemBench MC scorer, Porter stemmer, scoring utilities
- `crates/memd-client/src/benchmark/baselines.rs` — published competitor baselines, comparison table generator
- `.memd/benchmarks/baselines/published_baselines.json` — static competitor scores

### Modified Files
- `crates/memd-client/src/cli/args.rs` — add `--full-eval`, `--generator-model`, `--sample`, `--all`, `--dry-run` flags
- `crates/memd-client/src/benchmark/mod.rs` — register new modules
- `crates/memd-client/src/benchmark/runtime.rs` — wire `--full-eval` into `run_public_benchmark_command`
- `crates/memd-client/src/benchmark/public_benchmark.rs` — delete generic fallback path, update primary metric labels
- `crates/memd-client/src/bundle/models.rs` — extend `PublicBenchmarkItemResult` and `PublicBenchmarkManifest` with generation fields

### Test Files
- `crates/memd-client/src/main_tests/public_benchmark_tests/mod.rs` — add full-eval integration tests

---

## Task 1: CLI Flags

**Files:**
- Modify: `crates/memd-client/src/cli/args.rs:2349-2395`

- [ ] **Step 1: Add new flags to PublicBenchmarkArgs**

In `args.rs`, add these fields to `PublicBenchmarkArgs` after the existing `grader_model` field:

```rust
#[arg(long, default_value_t = false)]
pub(crate) full_eval: bool,

#[arg(long)]
pub(crate) generator_model: Option<String>,

#[arg(long)]
pub(crate) sample: Option<usize>,

#[arg(long, default_value_t = false)]
pub(crate) dry_run: bool,

#[arg(long, default_value_t = false)]
pub(crate) all: bool,
```

- [ ] **Step 2: Update validate_public_benchmark_args**

In `public_benchmark.rs`, update `validate_public_benchmark_args` to handle new flags:

```rust
if args.full_eval && args.community_standard {
    anyhow::bail!("--full-eval replaces --community-standard; use --full-eval instead");
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check -p memd-client`
Expected: compiles with no errors

- [ ] **Step 4: Commit**

```bash
git add crates/memd-client/src/cli/args.rs crates/memd-client/src/benchmark/public_benchmark.rs
git commit -m "feat(benchmark): add --full-eval, --generator-model, --sample, --dry-run CLI flags"
```

---

## Task 2: Scorers Module (F1 + MC)

**Files:**
- Create: `crates/memd-client/src/benchmark/scorers.rs`
- Modify: `crates/memd-client/src/benchmark/mod.rs`

- [ ] **Step 1: Write failing tests for F1 scorer**

At the bottom of `scorers.rs`, add tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f1_exact_match() {
        let score = token_f1("the cat sat on the mat", "the cat sat on the mat");
        assert!((score - 1.0).abs() < 0.001);
    }

    #[test]
    fn f1_partial_match() {
        let score = token_f1("the cat sat", "the cat sat on the mat");
        // precision=3/3=1.0, recall=3/6=0.5, f1=2*1.0*0.5/1.5=0.667
        assert!((score - 0.667).abs() < 0.01);
    }

    #[test]
    fn f1_no_match() {
        let score = token_f1("dog runs fast", "the cat sat on the mat");
        assert!((score - 0.0).abs() < 0.001);
    }

    #[test]
    fn f1_empty_prediction() {
        let score = token_f1("", "the cat sat on the mat");
        assert!((score - 0.0).abs() < 0.001);
    }

    #[test]
    fn mc_exact_match() {
        assert!(mc_accuracy("A", "A"));
    }

    #[test]
    fn mc_no_match() {
        assert!(!mc_accuracy("A", "B"));
    }

    #[test]
    fn mc_extracts_from_text() {
        assert!(mc_accuracy("The answer is B", "B"));
    }

    #[test]
    fn stem_basic() {
        assert_eq!(stem_token("running"), "run");
        assert_eq!(stem_token("cats"), "cat");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p memd-client scorers::tests -- --nocapture`
Expected: compilation error, module doesn't exist

- [ ] **Step 3: Implement scorers module**

Create `scorers.rs`:

```rust
use super::*;

/// Porter-style stemmer (simplified English suffix stripping).
/// Matches LoCoMo paper's stemming normalization.
pub(crate) fn stem_token(word: &str) -> String {
    let w = word.to_ascii_lowercase();
    // Step 1: plurals and -ed/-ing
    if let Some(base) = w.strip_suffix("ies") {
        return format!("{base}i");
    }
    if let Some(base) = w.strip_suffix("sses") {
        return format!("{base}ss");
    }
    if let Some(base) = w.strip_suffix("ness") {
        return base.to_string();
    }
    if w.ends_with('s') && !w.ends_with("ss") && w.len() > 3 {
        return w[..w.len() - 1].to_string();
    }
    if let Some(base) = w.strip_suffix("eed") {
        if base.len() > 1 {
            return format!("{base}ee");
        }
    }
    if let Some(base) = w.strip_suffix("ing") {
        if base.len() > 2 {
            return base.to_string();
        }
    }
    if let Some(base) = w.strip_suffix("ed") {
        if base.len() > 2 {
            return base.to_string();
        }
    }
    w
}

/// Tokenize and stem a string for F1 computation.
/// Matches LoCoMo paper: lowercase, split on whitespace+punctuation, stem.
pub(crate) fn tokenize_and_stem(text: &str) -> Vec<String> {
    text.to_ascii_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| stem_token(t))
        .collect()
}

/// Token-level F1 score with stemming (LoCoMo paper protocol).
/// Uses frequency-aware multiset matching, not set-based.
/// precision = |pred ∩ gold| / |pred|
/// recall = |pred ∩ gold| / |gold|
/// F1 = 2 * precision * recall / (precision + recall)
pub(crate) fn token_f1(prediction: &str, gold: &str) -> f64 {
    let pred_tokens = tokenize_and_stem(prediction);
    let gold_tokens = tokenize_and_stem(gold);
    if pred_tokens.is_empty() || gold_tokens.is_empty() {
        return 0.0;
    }
    // Frequency-aware: count occurrences of each token
    let mut pred_counts: BTreeMap<&str, usize> = BTreeMap::new();
    for t in &pred_tokens {
        *pred_counts.entry(t.as_str()).or_insert(0) += 1;
    }
    let mut gold_counts: BTreeMap<&str, usize> = BTreeMap::new();
    for t in &gold_tokens {
        *gold_counts.entry(t.as_str()).or_insert(0) += 1;
    }
    // Intersection = sum of min(pred_count, gold_count) per token
    let intersection: usize = pred_counts
        .iter()
        .map(|(token, count)| {
            count.min(gold_counts.get(token).unwrap_or(&0))
        })
        .sum();
    let precision = intersection as f64 / pred_tokens.len() as f64;
    let recall = intersection as f64 / gold_tokens.len() as f64;
    if precision + recall == 0.0 {
        return 0.0;
    }
    2.0 * precision * recall / (precision + recall)
}

/// LoCoMo adversarial category: check if model correctly abstains.
pub(crate) fn locomo_adversarial_check(prediction: &str) -> bool {
    let lower = prediction.to_ascii_lowercase();
    lower.contains("no information")
        || lower.contains("not mentioned")
        || lower.contains("cannot answer")
        || lower.contains("not available")
        || lower.contains("no relevant")
        || lower.contains("don't have")
        || lower.contains("do not have")
        || lower.contains("unanswerable")
}

/// Multiple-choice accuracy: extract choice letter from LLM response,
/// compare against ground_truth.
pub(crate) fn mc_accuracy(response: &str, ground_truth: &str) -> bool {
    let gt = ground_truth.trim().to_ascii_uppercase();
    let resp = response.trim().to_ascii_uppercase();
    // Direct match
    if resp == gt {
        return true;
    }
    // Extract single letter if response is verbose ("The answer is B")
    for pattern in ["ANSWER IS ", "ANSWER: ", "CHOICE: ", "OPTION: "] {
        if let Some(after) = resp.find(pattern) {
            let rest = &resp[after + pattern.len()..];
            if let Some(letter) = rest.chars().next() {
                if letter.to_string() == gt {
                    return true;
                }
            }
        }
    }
    // Check if gt letter appears as standalone in response
    if gt.len() == 1 {
        let gt_char = gt.chars().next().unwrap();
        // Match "B" or "B." or "(B)" patterns
        if resp.starts_with(gt_char)
            && resp.get(1..2).map_or(true, |c| !c.chars().next().unwrap().is_alphanumeric())
        {
            return true;
        }
    }
    false
}
```

- [ ] **Step 4: Register module in mod.rs**

In `crates/memd-client/src/benchmark/mod.rs`, add:

```rust
mod scorers;
pub(crate) use scorers::*;
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p memd-client scorers::tests -- --nocapture`
Expected: all tests pass

- [ ] **Step 6: Commit**

```bash
git add crates/memd-client/src/benchmark/scorers.rs crates/memd-client/src/benchmark/mod.rs
git commit -m "feat(benchmark): add LoCoMo F1 scorer and MemBench MC accuracy scorer"
```

---

## Task 3: Full-Eval Generation Step

**Files:**
- Create: `crates/memd-client/src/benchmark/full_eval.rs`
- Modify: `crates/memd-client/src/benchmark/mod.rs`

- [ ] **Step 1: Write failing test for generation call**

At the bottom of `full_eval.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_generation_prompt_includes_context_and_question() {
        let prompt = build_generation_prompt("What color is the sky?", "User said: the sky is blue");
        assert!(prompt.contains("What color is the sky?"));
        assert!(prompt.contains("the sky is blue"));
    }

    #[test]
    fn build_mc_generation_prompt_includes_choices() {
        let choices = vec!["A. Red".to_string(), "B. Blue".to_string()];
        let prompt = build_mc_generation_prompt("What color?", "sky is blue", &choices);
        assert!(prompt.contains("A. Red"));
        assert!(prompt.contains("B. Blue"));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p memd-client full_eval::tests -- --nocapture`
Expected: compilation error

- [ ] **Step 3: Implement full_eval module**

Create `full_eval.rs`:

```rust
use super::*;

/// Build a generation prompt for LongMemEval / LoCoMo.
/// Context is the retrieved memory content. Question is the benchmark question.
pub(crate) fn build_generation_prompt(question: &str, retrieved_context: &str) -> String {
    format!(
        "You are a helpful assistant with access to the user's conversation history.\n\n\
         Relevant conversation history:\n{retrieved_context}\n\n\
         Question: {question}\n\n\
         Answer the question based only on the conversation history above. \
         If the information is not available, say so. Be concise."
    )
}

/// Build a generation prompt for MemBench multiple-choice questions.
pub(crate) fn build_mc_generation_prompt(
    question: &str,
    retrieved_context: &str,
    choices: &[String],
) -> String {
    let choices_text = choices.join("\n");
    format!(
        "You are a helpful assistant with access to the user's conversation history.\n\n\
         Relevant conversation history:\n{retrieved_context}\n\n\
         Question: {question}\n\n\
         Choices:\n{choices_text}\n\n\
         Select the correct answer. Reply with ONLY the choice letter (A, B, C, etc.)."
    )
}

/// Call an OpenAI-compatible API to generate a response.
/// Works with OpenAI, Anthropic (via proxy), or any compatible endpoint.
pub(crate) fn call_generator(
    base_url: &str,
    api_key: &str,
    model: &str,
    prompt: &str,
) -> anyhow::Result<GeneratorResponse> {
    let client = reqwest::blocking::Client::builder()
        .build()
        .context("build generator client")?;
    let response = client
        .post(format!("{}/chat/completions", base_url.trim_end_matches('/')))
        .bearer_auth(api_key)
        .json(&json!({
            "model": model,
            "messages": [{"role": "user", "content": prompt}],
            "n": 1,
            "temperature": 0,
            "max_tokens": 512
        }))
        .send()
        .context("send generator request")?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        anyhow::bail!("generator request failed with {status}: {body}");
    }
    let body = response.json::<JsonValue>().context("parse generator response")?;
    let content = body
        .get("choices")
        .and_then(JsonValue::as_array)
        .and_then(|c| c.first())
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(JsonValue::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let usage = body.get("usage").cloned();
    let prompt_tokens = usage
        .as_ref()
        .and_then(|u| u.get("prompt_tokens"))
        .and_then(JsonValue::as_u64)
        .unwrap_or(0);
    let completion_tokens = usage
        .as_ref()
        .and_then(|u| u.get("completion_tokens"))
        .and_then(JsonValue::as_u64)
        .unwrap_or(0);
    Ok(GeneratorResponse {
        content,
        prompt_tokens,
        completion_tokens,
    })
}

#[derive(Debug, Clone)]
pub(crate) struct GeneratorResponse {
    pub content: String,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
}

/// Resolve generator configuration from CLI args + env vars.
pub(crate) fn resolve_generator_config(
    args: &PublicBenchmarkArgs,
) -> anyhow::Result<GeneratorConfig> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .context("--full-eval requires OPENAI_API_KEY (or ANTHROPIC_API_KEY with --generator-model claude-*)")?;
    let base_url = std::env::var("OPENAI_BASE_URL")
        .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
    let model = args
        .generator_model
        .as_deref()
        .unwrap_or("gpt-4o-mini")
        .to_string();
    let grader_model = args
        .grader_model
        .as_deref()
        .unwrap_or("gpt-4o-2024-08-06")
        .to_string();
    Ok(GeneratorConfig {
        base_url,
        api_key,
        model,
        grader_model,
    })
}

#[derive(Debug, Clone)]
pub(crate) struct GeneratorConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub grader_model: String,
}
```

- [ ] **Step 4: Register module in mod.rs**

Add to `crates/memd-client/src/benchmark/mod.rs`:

```rust
mod full_eval;
pub(crate) use full_eval::*;
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p memd-client full_eval::tests -- --nocapture`
Expected: all tests pass

- [ ] **Step 6: Commit**

```bash
git add crates/memd-client/src/benchmark/full_eval.rs crates/memd-client/src/benchmark/mod.rs
git commit -m "feat(benchmark): add LLM generation step for full-eval mode"
```

---

## Task 4: LongMemEval Full-Eval Pipeline

**Files:**
- Modify: `crates/memd-client/src/benchmark/public_benchmark.rs`
- Modify: `crates/memd-client/src/benchmark/runtime.rs`

- [ ] **Step 1: Add `build_longmemeval_full_eval_report` function**

In `public_benchmark.rs`, add a new function that combines retrieval → generation → grading. This replaces the need for external `--hypotheses-file`:

```rust
pub(crate) fn build_longmemeval_full_eval_report(
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

    for item in &dataset.items {
        let item_started = Instant::now();
        let question_type = item
            .metadata
            .get("question_type")
            .and_then(JsonValue::as_str)
            .unwrap_or("unknown");
        let abstention = item.question_id.contains("_abs");

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

        // Step 2: Generate hypothesis
        let prompt = build_generation_prompt(&item.query, &context);
        let gen_response = call_generator(
            &generator_config.base_url,
            &generator_config.api_key,
            &generator_config.model,
            &prompt,
        )?;
        total_prompt_tokens += gen_response.prompt_tokens;
        total_completion_tokens += gen_response.completion_tokens;

        // Step 3: Grade with GPT-4o judge
        let eval_prompt = build_longmemeval_eval_prompt(
            question_type,
            &item.query,
            &item.gold_answer,
            &gen_response.content,
            abstention,
        )?;
        let grader_response = call_openai_yes_no_grader(
            &generator_config.base_url,
            &generator_config.api_key,
            &generator_config.grader_model,
            &eval_prompt,
        )?;
        let label = grader_response.to_ascii_lowercase().contains("yes");

        if label {
            correct += 1;
            *per_type_correct.entry(question_type.to_string()).or_insert(0) += 1;
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
    let accuracy = if item_count == 0 { 0.0 } else { correct as f64 / item_count as f64 };
    let mut metrics = BTreeMap::new();
    metrics.insert("accuracy".to_string(), accuracy);
    metrics.insert("item_count".to_string(), item_count as f64);
    for (qt, total) in &per_type_total {
        let c = per_type_correct.get(qt).copied().unwrap_or(0);
        metrics.insert(
            format!("per_type::{qt}::accuracy"),
            if *total == 0 { 0.0 } else { c as f64 / *total as f64 },
        );
    }

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
```

- [ ] **Step 2: Wire into run_public_benchmark_command**

In `runtime.rs`, update `run_public_benchmark_command` (around line 1283) to dispatch full-eval:

```rust
let evaluation = if args.full_eval {
    let generator_config = resolve_generator_config(args)?;
    match args.dataset.as_str() {
        "longmemeval" => build_longmemeval_full_eval_report(
            &selected_dataset, top_k, mode, &retrieval_config, &generator_config,
        )?,
        "locomo" => build_locomo_full_eval_report(
            &selected_dataset, top_k, mode, &retrieval_config, &generator_config,
        )?,
        "membench" => build_membench_full_eval_report(
            &selected_dataset, top_k, mode, &retrieval_config, &generator_config,
        )?,
        other => anyhow::bail!("--full-eval not yet supported for {other}"),
    }
} else if args.community_standard {
    // legacy path — kept for backwards compat
    // ...existing code...
} else {
    build_public_benchmark_item_results(/* ...existing... */)?
};
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check -p memd-client`
Expected: compiles (locomo/membench functions don't exist yet — use stubs or bail)

- [ ] **Step 4: Commit**

```bash
git add crates/memd-client/src/benchmark/public_benchmark.rs crates/memd-client/src/benchmark/runtime.rs
git commit -m "feat(benchmark): add LongMemEval full-eval pipeline (retrieve → generate → judge)"
```

---

## Task 5: LoCoMo Full-Eval Pipeline

**Files:**
- Modify: `crates/memd-client/src/benchmark/public_benchmark.rs`

- [ ] **Step 1: Add `build_locomo_full_eval_report` function**

Same pattern as LongMemEval but uses F1 scoring instead of LLM judge:

```rust
pub(crate) fn build_locomo_full_eval_report(
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
    let mut f1_scores = Vec::new();
    let mut per_category_f1: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    let mut total_prompt_tokens: u64 = 0;
    let mut total_completion_tokens: u64 = 0;

    for item in &dataset.items {
        let item_started = Instant::now();
        let category = item
            .metadata
            .get("category_name")
            .and_then(JsonValue::as_str)
            .unwrap_or("unknown")
            .to_string();

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

        // Step 2: Generate answer
        let prompt = build_generation_prompt(&item.query, &context);
        let gen_response = call_generator(
            &generator_config.base_url,
            &generator_config.api_key,
            &generator_config.model,
            &prompt,
        )?;
        total_prompt_tokens += gen_response.prompt_tokens;
        total_completion_tokens += gen_response.completion_tokens;

        // Step 3: Score with F1 (or adversarial check)
        let f1 = if category == "Adversarial" {
            if locomo_adversarial_check(&gen_response.content) { 1.0 } else { 0.0 }
        } else {
            token_f1(&gen_response.content, &item.gold_answer)
        };
        f1_scores.push(f1);
        per_category_f1.entry(category.clone()).or_default().push(f1);

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
    let mean_f1 = if f1_scores.is_empty() { 0.0 } else {
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
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p memd-client`

- [ ] **Step 3: Commit**

```bash
git add crates/memd-client/src/benchmark/public_benchmark.rs
git commit -m "feat(benchmark): add LoCoMo full-eval pipeline (retrieve → generate → F1 score)"
```

---

## Task 6: MemBench Full-Eval Pipeline

**Files:**
- Modify: `crates/memd-client/src/benchmark/public_benchmark.rs`

- [ ] **Step 1: Add `build_membench_full_eval_report` function**

Multiple-choice pipeline — simplest of the three:

```rust
pub(crate) fn build_membench_full_eval_report(
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
    let mut per_category_correct = BTreeMap::<String, usize>::new();
    let mut per_category_total = BTreeMap::<String, usize>::new();
    let mut total_prompt_tokens: u64 = 0;
    let mut total_completion_tokens: u64 = 0;

    for item in &dataset.items {
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
            // Skip items without MC data — they're not scoreable
            continue;
        }

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

        // Step 2: Generate MC selection
        let prompt = build_mc_generation_prompt(&item.query, &context, &choices);
        let gen_response = call_generator(
            &generator_config.base_url,
            &generator_config.api_key,
            &generator_config.model,
            &prompt,
        )?;
        total_prompt_tokens += gen_response.prompt_tokens;
        total_completion_tokens += gen_response.completion_tokens;

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
    let accuracy = if item_count == 0 { 0.0 } else { correct as f64 / item_count as f64 };
    let mut metrics = BTreeMap::new();
    metrics.insert("accuracy".to_string(), accuracy);
    metrics.insert("mc_accuracy".to_string(), accuracy);
    metrics.insert("item_count".to_string(), item_count as f64);
    for (cat, total) in &per_category_total {
        let c = per_category_correct.get(cat).copied().unwrap_or(0);
        metrics.insert(
            format!("per_category::{cat}::accuracy"),
            if *total == 0 { 0.0 } else { c as f64 / *total as f64 },
        );
    }

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
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p memd-client`

- [ ] **Step 3: Commit**

```bash
git add crates/memd-client/src/benchmark/public_benchmark.rs
git commit -m "feat(benchmark): add MemBench full-eval pipeline (retrieve → generate → MC accuracy)"
```

---

## Task 7: Published Baselines + Comparison Table

**Files:**
- Create: `crates/memd-client/src/benchmark/baselines.rs`
- Create: `.memd/benchmarks/baselines/published_baselines.json`
- Modify: `crates/memd-client/src/benchmark/mod.rs`

- [ ] **Step 1: Create published baselines JSON**

```json
{
  "longmemeval": {
    "SuperMem": {"accuracy": 81.6, "source": "supermemory.ai/research", "date": "2026"},
    "GPT-4o (oracle)": {"accuracy": 91.8, "source": "arxiv:2410.10813", "date": "2024"},
    "GPT-4o (tested)": {"accuracy": 60.6, "source": "arxiv:2410.10813", "date": "2024"},
    "Llama-3.1-70B": {"accuracy": 33.4, "source": "arxiv:2410.10813", "date": "2024"}
  },
  "locomo": {
    "Mem0": {"accuracy": 66.9, "source": "mem0.ai/research", "date": "2025"},
    "Letta": {"accuracy": 74.0, "source": "letta.com/blog/benchmarking", "date": "2025"},
    "RAG top-5": {"accuracy": 41.4, "source": "arxiv:2402.17753", "date": "2024"},
    "GPT-3.5-turbo-16K": {"accuracy": 37.8, "source": "arxiv:2402.17753", "date": "2024"}
  },
  "membench": {}
}
```

- [ ] **Step 2: Implement baselines module**

Create `baselines.rs` with functions to load baselines and render comparison markdown table.

- [ ] **Step 3: Wire comparison table into full-eval report output**

After a `--full-eval` run, append comparison table to the markdown report.

- [ ] **Step 4: Commit**

```bash
git add .memd/benchmarks/baselines/ crates/memd-client/src/benchmark/baselines.rs crates/memd-client/src/benchmark/mod.rs
git commit -m "feat(benchmark): add published baselines and competitive comparison table"
```

---

## Task 8: Update Primary Metric Labels + Delete Dead Code

**Files:**
- Modify: `crates/memd-client/src/benchmark/public_benchmark.rs`

- [ ] **Step 1: Update `public_benchmark_primary_metric` for full-eval**

```rust
pub(crate) fn public_benchmark_primary_metric(
    report: &PublicBenchmarkRunReport,
) -> (&'static str, f64) {
    let is_full_eval = report
        .manifest
        .runtime_settings
        .get("full_eval")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false);

    match (report.manifest.benchmark_id.as_str(), is_full_eval) {
        ("longmemeval", true) => (
            "accuracy (LLM-judge, industry standard)",
            report.metrics.get("accuracy").copied().unwrap_or(0.0),
        ),
        ("locomo", true) => (
            "F1 (token-level, industry standard)",
            report.metrics.get("f1").copied().unwrap_or(0.0),
        ),
        ("membench", true) => (
            "MC accuracy (industry standard)",
            report.metrics.get("mc_accuracy").copied().unwrap_or(0.0),
        ),
        ("longmemeval", false) => (
            "session_recall_any@5 (retrieval diagnostic)",
            report.metrics.get("session_recall_any@5").copied()
                .or_else(|| report.metrics.get("accuracy").copied())
                .unwrap_or(0.0),
        ),
        _ => (
            "accuracy (retrieval diagnostic)",
            report.metrics.get("accuracy").copied().unwrap_or(0.0),
        ),
    }
}
```

- [ ] **Step 2: Delete the generic fallback path**

Remove the `build_public_benchmark_item_results` fallback (lines 2092-2257) that contains:
- `candidate.item_id == item.item_id` score bonus
- `gold_answer` in candidate text
- Self-match ranking

Replace with a clean bail for unsupported benchmarks.

- [ ] **Step 3: Run full test suite**

Run: `cargo test -p memd-client -- --nocapture`
Expected: all existing tests pass, no regressions

- [ ] **Step 4: Commit**

```bash
git add crates/memd-client/src/benchmark/public_benchmark.rs
git commit -m "fix(benchmark): update primary metric labels, delete inflated generic fallback path"
```

---

## Task 9: Latency Percentiles + Token Efficiency

**Files:**
- Modify: `crates/memd-client/src/benchmark/public_benchmark.rs`

- [ ] **Step 1: Add percentile + efficiency helpers**

```rust
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

pub(crate) fn compute_token_efficiency(
    retrieved_tokens: u64,
    full_context_tokens: u64,
) -> f64 {
    if full_context_tokens == 0 {
        return 0.0;
    }
    1.0 - (retrieved_tokens as f64 / full_context_tokens as f64)
}
```

- [ ] **Step 2: Add these metrics to full-eval reports**

In each `build_*_full_eval_report` function, after computing results, add:

```rust
let latencies: Vec<u128> = results.iter().map(|r| r.latency_ms).collect();
let (p50, p95) = compute_latency_percentiles(&latencies);
metrics.insert("latency_p50_ms".to_string(), p50);
metrics.insert("latency_p95_ms".to_string(), p95);
metrics.insert("token_efficiency".to_string(),
    compute_token_efficiency(total_prompt_tokens, full_context_token_count));
```

- [ ] **Step 3: Commit**

```bash
git add crates/memd-client/src/benchmark/public_benchmark.rs
git commit -m "feat(benchmark): add latency percentiles and token efficiency metrics"
```

---

## Task 10: Sample Mode + Dry Run

**Files:**
- Modify: `crates/memd-client/src/benchmark/runtime.rs`

- [ ] **Step 1: Wire `--sample` into item selection**

In `run_public_benchmark_command`, update item selection to respect `--sample`:

```rust
let item_count = args.sample
    .or(args.limit)
    .unwrap_or(dataset.items.len())
    .max(1)
    .min(dataset.items.len());
```

- [ ] **Step 2: Add dry-run cost estimation**

Before the evaluation dispatch, if `args.dry_run`:

```rust
if args.dry_run {
    let est_calls = item_count * if args.dataset == "longmemeval" { 2 } else { 1 };
    let est_tokens = est_calls as f64 * 2200.0; // ~2K input + 200 output avg
    let est_cost_4o_mini = est_tokens * 0.00000015; // $0.15/1M input
    let est_cost_4o = est_tokens * 0.0000025; // $2.50/1M input
    println!("Dry run: {item_count} items, ~{est_calls} API calls");
    println!("Estimated cost: ${:.2} (gpt-4o-mini) / ${:.2} (gpt-4o)", est_cost_4o_mini, est_cost_4o);
    return Ok(/* empty report */);
}
```

- [ ] **Step 3: Commit**

```bash
git add crates/memd-client/src/benchmark/runtime.rs
git commit -m "feat(benchmark): add --sample and --dry-run support for cost control"
```

---

## Task 11: Integration Test

**Files:**
- Modify: `crates/memd-client/src/main_tests/public_benchmark_tests/mod.rs`

- [ ] **Step 1: Add scorer unit test coverage**

Expand F1 and MC tests with edge cases from real benchmark data.

- [ ] **Step 2: Add integration test for full-eval pipeline**

Test the full pipeline with mini datasets (already exist at `.memd/benchmarks/datasets/*-mini.json`). Mock the LLM API calls to avoid real API spend in CI.

- [ ] **Step 3: Run full test suite**

Run: `cargo test -p memd-client -- --nocapture`
Expected: all tests pass

- [ ] **Step 4: Commit**

```bash
git add crates/memd-client/src/main_tests/
git commit -m "test(benchmark): add full-eval integration tests with mocked LLM"
```

---

## Task 12: `--all` Dispatch + Multi-Benchmark Runs

**Files:**
- Modify: `crates/memd-client/src/benchmark/runtime.rs`

- [ ] **Step 1: Add multi-benchmark dispatch**

In `run_public_benchmark_command`, before the single-benchmark path, handle `--all`:

```rust
if args.all {
    let mut all_reports = Vec::new();
    for dataset_id in implemented_public_benchmark_ids() {
        let mut sub_args = args.clone();
        sub_args.dataset = dataset_id.to_string();
        sub_args.all = false;
        match run_public_benchmark_command(&sub_args).await {
            Ok(report) => all_reports.push(report),
            Err(err) => eprintln!("warning: {dataset_id} benchmark failed: {err}"),
        }
    }
    // Return the last report for now; write all reports to artifacts
    // The leaderboard/comparison table uses all_reports
    return all_reports.pop().ok_or_else(|| anyhow!("no benchmarks completed"));
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p memd-client`

- [ ] **Step 3: Commit**

```bash
git add crates/memd-client/src/benchmark/runtime.rs
git commit -m "feat(benchmark): add --all flag for running all benchmarks sequentially"
```

---

## Task 13: CI Regression Gate

**Files:**
- Create: `.memd/benchmarks/thresholds.json`
- Modify: `crates/memd-client/src/benchmark/runtime.rs`

- [ ] **Step 1: Create threshold config**

```json
{
  "longmemeval": {
    "retrieval": {"session_recall_any@5": 0.80},
    "full_eval": {"accuracy": 0.0}
  },
  "locomo": {
    "retrieval": {"accuracy": 0.35},
    "full_eval": {"f1": 0.0}
  },
  "membench": {
    "retrieval": {"accuracy": 0.30},
    "full_eval": {"mc_accuracy": 0.0}
  }
}
```

Note: `full_eval` thresholds start at 0.0 (no gate) until we have baseline numbers. Update after first real run.

- [ ] **Step 2: Add threshold check to benchmark run**

After computing results, compare against thresholds. If `--ci-gate` flag is set and score drops below threshold, exit with non-zero status:

```rust
pub(crate) fn check_benchmark_threshold(
    benchmark_id: &str,
    mode: &str,
    metrics: &BTreeMap<String, f64>,
    thresholds_path: &Path,
) -> anyhow::Result<bool> {
    if !thresholds_path.exists() {
        return Ok(true); // no thresholds = pass
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
```

- [ ] **Step 3: Add historical tracking**

After each run, append a summary to `.memd/benchmarks/history/{benchmark_id}.jsonl`:

```rust
let history_entry = json!({
    "timestamp": Utc::now().to_rfc3339(),
    "git_sha": manifest.git_sha,
    "mode": manifest.mode,
    "metrics": report.metrics,
    "item_count": report.item_count,
});
```

- [ ] **Step 4: Commit**

```bash
git add .memd/benchmarks/thresholds.json crates/memd-client/src/benchmark/runtime.rs
git commit -m "feat(benchmark): add CI regression gate with threshold config and historical tracking"
```

---

## Execution Order

Task 1 must come first (CLI flags). Tasks 2, 3, 7 can parallelize after Task 1. Tasks 4-6 depend on 2+3. Task 8 depends on 4-6. Tasks 9-10, 12-13 are polish. Task 11 is final verification.

Recommended parallel groups:
- **Wave 1:** Task 1 (CLI flags — everything else needs these)
- **Wave 2:** Tasks 2, 3, 7 (scorers, generation, baselines — independent)
- **Wave 3:** Tasks 4, 5, 6, 12 (per-benchmark pipelines + --all dispatch)
- **Wave 4:** Tasks 8, 9, 10, 13 (cleanup, metrics, CI gate)
- **Wave 5:** Task 11 (final integration test)

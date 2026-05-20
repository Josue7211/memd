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
            let grader = call_openai_yes_no_grader_cached(
                &base_url,
                &api_key,
                grader_model,
                &prompt,
                &cache_key,
            )
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
    metrics.insert(
        "judge_prompt_tokens".to_string(),
        judge_prompt_tokens as f64,
    );
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

        // Step 1: Retrieve context via dispatched backend (lexical/memd/rrf/sidecar).
        // j3-prep-1: honor --retrieval-backend for LoCoMo full-eval. Prior
        // hardcoded lexical intersection ignored `retrieval_config`.
        let docs = locomo_retrieval_docs(item);
        let ranked = dispatch_context_retrieval_ranked(
            "locomo",
            &item.item_id,
            &item.query,
            &docs,
            mode,
            retrieval_config,
        );
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
    metrics.insert("token_f1_avg".to_string(), mean_f1);
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
        let choices = parse_membench_choices(item.metadata.get("choices"));

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

        // Step 1: Retrieve context via dispatched backend (lexical/memd/rrf/sidecar).
        // j3-prep-2: honor --retrieval-backend for MemBench full-eval. Prior
        // hardcoded lexical intersection ignored `retrieval_config`.
        let docs = membench_retrieval_docs(item);
        let ranked = dispatch_context_retrieval_ranked(
            "membench",
            &item.item_id,
            &item.query,
            &docs,
            mode,
            retrieval_config,
        );
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

/// k3-convomem: cheap fallback normalized-match scorer, retained for unit tests
/// and judge-unavailable paths. Canonical ConvoMem scoring uses the
/// `build_convomem_judge_prompt` LLM judge mirroring upstream
/// `DefaultAnsweringEvaluation` (RIGHT/WRONG).
pub(crate) fn convomem_exact_match(predicted: &str, gold: &str) -> bool {
    fn normalize(s: &str) -> String {
        let trimmed = s
            .trim()
            .trim_matches(|c: char| matches!(c, '"' | '\'' | '`' | '.' | '!' | '?' | ',' | ';'))
            .trim();
        let lower = trimmed.to_lowercase();
        lower.split_whitespace().collect::<Vec<_>>().join(" ")
    }
    let n_pred = normalize(predicted);
    let n_gold = normalize(gold);
    if n_pred.is_empty() || n_gold.is_empty() {
        return false;
    }
    if n_pred == n_gold {
        return true;
    }
    // Allow "the answer is X" style wrappers around the gold value.
    n_pred.contains(&n_gold) || n_gold.contains(&n_pred)
}

pub(crate) fn convomem_retrieval_docs(
    item: &PublicBenchmarkDatasetFixtureItem,
) -> Vec<(String, String)> {
    convomem_message_docs(
        item.metadata
            .get("conversations")
            .unwrap_or(&JsonValue::Null),
    )
}

/// k3-convomem: mirror of upstream `DefaultAnsweringEvaluation.getJudgePromptTemplate`.
/// Upstream (Salesforce/ConvoMem, Scala) scores factual evidence via an LLM judge
/// returning RIGHT/WRONG, not substring/exact match. We use the same prompt wording
/// verbatim so judge-swap (ours = gpt-5.4, upstream = Gemini 2.5 flash) is the
/// only deviation. Disclose the judge-swap in the PUBLIC_LEADERBOARD method card.
pub(crate) fn build_convomem_judge_prompt(
    question: &str,
    gold: &str,
    model_answer: &str,
) -> String {
    let guidelines = "**Crucial Guidelines for your judgment:**\n\n1.  **Core Information is Key**: The Model's Response must contain all the **essential factual information** that directly answers the Question.\n2.  **Equivalence Counts**: Phrasing doesn't need to be identical. If the Model's Response conveys the exact same core meaning and details as the Correct Answer, even if paraphrased or structured differently, consider it correct.\n3.  **Superfluous (but Accurate) Information**: If the Model's Response includes additional details that were *not explicitly asked for* by the Question, but these details are **accurate and do not contradict** the Correct Answer, you should **still count it as correct** if the core question is fully answered.\n4.  **Partial Answers are Incorrect**: If the Model's Response is missing any essential information directly requested by the Question (even if the Correct Answer provides more detail), it is incorrect.\n5.  **Focus on the Question**: Your primary focus should be whether the Model's Response adequately addresses the Question using information that aligns with the Correct Answer. Do not penalize the model for not reiterating every single word or incidental detail from the Correct Answer if it wasn't requested.";
    format!(
        "I will provide you with a **Question**, a **Correct Answer**, and a **Model's Response**. Your sole task is to determine if the Model's Response is **sufficiently correct and complete** to answer the Question, when compared against the Correct Answer.\n\n{guidelines}\n\n**Answer only \"RIGHT\" or \"WRONG\". Do not provide any additional text, explanations, or reasoning.**\n\nQuestion: {question}\nCorrect Answer: {gold}\nModel Response: {model_answer}\n\nAnswer (RIGHT/WRONG):"
    )
}

pub(crate) async fn build_convomem_full_eval_report(
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

    let total_items = dataset.items.len();
    eprintln!("[convomem] starting full-eval: {total_items} items, top_k={top_k}");
    for (item_index, item) in dataset.items.iter().enumerate() {
        let item_started = Instant::now();
        let category = item
            .metadata
            .get("category")
            .and_then(JsonValue::as_str)
            .unwrap_or("unknown")
            .to_string();
        eprintln!(
            "[convomem] [{}/{}] {} cat={category}",
            item_index + 1,
            total_items,
            item.question_id
        );

        let docs = convomem_retrieval_docs(item);
        let ranked = dispatch_context_retrieval_ranked(
            "convomem",
            &item.item_id,
            &item.query,
            &docs,
            mode,
            retrieval_config,
        );
        let context = ranked
            .iter()
            .take(top_k)
            .map(|((_, text), _)| text.as_str())
            .collect::<Vec<_>>()
            .join("\n---\n");
        eprintln!("[convomem]   retrieval done, context_len={}", context.len());

        let prompt = build_generation_prompt(&item.query, &context);
        eprintln!("[convomem]   calling generator…");
        let gen_response = call_generator(
            &generator_config.base_url,
            &generator_config.api_key,
            &generator_config.model,
            &prompt,
        )
        .await?;
        total_prompt_tokens += gen_response.prompt_tokens;
        total_completion_tokens += gen_response.completion_tokens;

        let judge_prompt =
            build_convomem_judge_prompt(&item.query, &item.gold_answer, &gen_response.content);
        let cache_key = judge_cache_key(
            "convomem-full-eval",
            &item.question_id,
            &gen_response.content,
            &generator_config.grader_model,
            &judge_prompt,
        );
        eprintln!("[convomem]   calling judge…");
        let judge = call_openai_yes_no_grader_cached(
            &generator_config.base_url,
            &generator_config.api_key,
            &generator_config.grader_model,
            &judge_prompt,
            &cache_key,
        )
        .await?;
        total_prompt_tokens += judge.prompt_tokens;
        total_completion_tokens += judge.completion_tokens;
        let judge_verdict = judge.content.trim().to_ascii_uppercase();
        let is_correct = judge_verdict.contains("RIGHT") && !judge_verdict.contains("WRONG");
        eprintln!("[convomem]   judge={judge_verdict} → correct={is_correct}");
        if is_correct {
            correct += 1;
            *per_category_correct.entry(category.clone()).or_insert(0) += 1;
        } else {
            failures.push(json!({
                "item_id": item.item_id,
                "category": category,
                "predicted": gen_response.content,
                "gold": item.gold_answer,
                "judge_verdict": judge.content,
            }));
        }
        *per_category_total.entry(category.clone()).or_insert(0) += 1;

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
            hit: is_correct,
            answer: Some(item.gold_answer.clone()),
            observed_answer: Some(gen_response.content),
            correctness: Some(json!({
                "score": if is_correct { 1.0 } else { 0.0 },
                "evaluation_protocol": "convomem-full-eval-llm-judge",
                "judge_model": generator_config.grader_model,
                "judge_verdict": judge.content,
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
    metrics.insert("judge_accuracy".to_string(), accuracy);
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
            runtime_settings: json!({
                "full_eval": true,
                "generator_model": generator_config.model,
                "judge_model": generator_config.grader_model,
                "judge_prompt": "DefaultAnsweringEvaluation (upstream ConvoMem)",
            }),
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

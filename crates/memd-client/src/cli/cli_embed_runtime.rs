use super::*;
use anyhow::Context;
use memd_rag::{
    RagClient, RagRerankCandidate, RagRerankRequest, RagRetrieveMode, RagRetrieveRequest,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, time::Instant};

pub(crate) async fn run_embed_mode(args: EmbedArgs) -> anyhow::Result<()> {
    match args.mode {
        EmbedMode::Models(args) => run_embed_models(args),
        EmbedMode::Bench(args) => run_embed_bench(args).await,
    }
}

#[derive(Debug, Clone, Deserialize)]
struct EmbedBenchFile {
    corpus: Option<String>,
    qrels: Vec<EmbedBenchCase>,
}

#[derive(Debug, Clone, Deserialize)]
struct EmbedBenchCase {
    query: String,
    relevant_id: Option<String>,
    #[serde(default)]
    relevant_ids: Vec<String>,
    #[serde(default)]
    project: Option<String>,
    #[serde(default)]
    namespace: Option<String>,
    #[serde(default)]
    candidates: Vec<EmbedBenchCandidate>,
    #[serde(default)]
    scores: BTreeMap<String, f64>,
    #[serde(default)]
    latency_ms: BTreeMap<String, f64>,
    #[serde(default)]
    cost_units: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Deserialize)]
struct EmbedBenchCandidate {
    id: String,
    text: String,
}

#[derive(Debug, Clone, Serialize)]
struct EmbedBenchReport {
    corpus: Option<String>,
    cases: usize,
    live: bool,
    selected_model: Option<String>,
    results: Vec<EmbedBenchModelResult>,
}

#[derive(Debug, Clone, Serialize)]
struct EmbedBenchModelResult {
    model_id: String,
    provider: String,
    role: String,
    cases: usize,
    mean_score: f64,
    mean_latency_ms: Option<f64>,
    mean_cost_units: Option<f64>,
    recall_at_1: Option<f64>,
    mrr: Option<f64>,
    selection_score: f64,
}

fn run_embed_models(args: EmbedModelsArgs) -> anyhow::Result<()> {
    let registry = memd_core::embedding_registry::EmbeddingModelRegistry::builtin();
    if args.json {
        print_json(&registry)?;
        return Ok(());
    }

    if let Some(target) = args.target.as_deref() {
        if let Some(profile) = registry.recommended_for(target) {
            println!(
                "{} provider={:?} role={:?} dim={} quality={} cost={} latency={}",
                profile.id,
                profile.provider,
                profile.role,
                profile
                    .dimensions
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "n/a".to_string()),
                profile.quality_tier,
                profile.cost_tier,
                profile.latency_tier
            );
            println!("{}", profile.notes);
            return Ok(());
        }
        anyhow::bail!("unknown embed target '{target}'; expected cloud, local, or hybrid");
    }

    println!("memd embedding model registry");
    println!("default_cloud={}", registry.default_cloud);
    println!("default_local={}", registry.default_local);
    println!("default_hybrid={}", registry.default_hybrid);
    for profile in registry.profiles {
        println!(
            "- {} provider={:?} role={:?} dim={} quality={} cost={} latency={} local={} cloud={} :: {}",
            profile.id,
            profile.provider,
            profile.role,
            profile
                .dimensions
                .map(|value| value.to_string())
                .unwrap_or_else(|| "n/a".to_string()),
            profile.quality_tier,
            profile.cost_tier,
            profile.latency_tier,
            profile.local,
            profile.cloud,
            profile.notes
        );
    }
    Ok(())
}

async fn run_embed_bench(args: EmbedBenchArgs) -> anyhow::Result<()> {
    let raw = fs::read_to_string(&args.input)
        .with_context(|| format!("read embed bench input {}", args.input.display()))?;
    let bench: EmbedBenchFile = serde_json::from_str(&raw)
        .with_context(|| format!("parse embed bench input {}", args.input.display()))?;
    let registry = memd_core::embedding_registry::EmbeddingModelRegistry::builtin();
    let mut report = evaluate_embed_bench(&registry, &bench, args.target.as_deref())?;
    if let Some(rag_url) = args.rag_url.as_ref() {
        let rag_url = resolve_rag_url(
            Some(rag_url.clone()),
            resolve_default_bundle_root()?.as_deref(),
        )?;
        let rag = RagClient::new(&rag_url)?;
        let live_results = evaluate_live_sidecar_bench(&rag, &bench, &args).await?;
        report.live = true;
        report.results.extend(live_results);
        sort_embed_bench_results(&mut report.results);
        report.selected_model = report.results.first().map(|result| result.model_id.clone());
    }
    if args.json {
        print_json(&report)?;
    } else {
        println!(
            "embed bench corpus={} cases={} live={} selected={}",
            report.corpus.as_deref().unwrap_or("unknown"),
            report.cases,
            report.live,
            report.selected_model.as_deref().unwrap_or("none")
        );
        for result in &report.results {
            println!(
                "- {} score={:.4} selection={:.4} latency={} cost={} recall@1={} mrr={} cases={}",
                result.model_id,
                result.mean_score,
                result.selection_score,
                result
                    .mean_latency_ms
                    .map(|value| format!("{value:.2}ms"))
                    .unwrap_or_else(|| "n/a".to_string()),
                result
                    .mean_cost_units
                    .map(|value| format!("{value:.4}"))
                    .unwrap_or_else(|| "n/a".to_string()),
                result
                    .recall_at_1
                    .map(|value| format!("{value:.4}"))
                    .unwrap_or_else(|| "n/a".to_string()),
                result
                    .mrr
                    .map(|value| format!("{value:.4}"))
                    .unwrap_or_else(|| "n/a".to_string()),
                result.cases
            );
        }
    }
    Ok(())
}

fn evaluate_embed_bench(
    registry: &memd_core::embedding_registry::EmbeddingModelRegistry,
    bench: &EmbedBenchFile,
    target: Option<&str>,
) -> anyhow::Result<EmbedBenchReport> {
    if bench.qrels.is_empty() {
        anyhow::bail!("embed bench input has no qrels");
    }
    let allowed = registry
        .profiles
        .iter()
        .filter(|profile| match target {
            Some("cloud") => profile.cloud,
            Some("local") => profile.local,
            Some("hybrid") => {
                profile.id == registry.default_hybrid
                    || matches!(
                        profile.role,
                        memd_core::embedding_registry::EmbeddingRole::Hybrid
                    )
            }
            Some(_) => false,
            None => true,
        })
        .collect::<Vec<_>>();
    if allowed.is_empty() {
        anyhow::bail!("unknown or empty embed bench target; expected cloud, local, or hybrid");
    }

    let mut results = Vec::new();
    for profile in allowed {
        let mut scores = Vec::new();
        let mut latencies = Vec::new();
        let mut costs = Vec::new();
        for case in &bench.qrels {
            let _ = (&case.query, &case.relevant_id);
            if let Some(score) = case.scores.get(&profile.id) {
                scores.push(score.clamp(0.0, 1.0));
                if let Some(latency) = case.latency_ms.get(&profile.id) {
                    latencies.push(*latency);
                }
                if let Some(cost) = case.cost_units.get(&profile.id) {
                    costs.push(*cost);
                }
            }
        }
        if scores.is_empty() {
            continue;
        }
        let mean_score = mean(&scores);
        let mean_latency_ms = (!latencies.is_empty()).then(|| mean(&latencies));
        let mean_cost_units = (!costs.is_empty()).then(|| mean(&costs));
        let latency_penalty = mean_latency_ms
            .map(|value| (value / 10_000.0).clamp(0.0, 0.20))
            .unwrap_or_else(|| (6_u8.saturating_sub(profile.latency_tier) as f64) * 0.01);
        let cost_penalty = mean_cost_units
            .map(|value| value.clamp(0.0, 0.20))
            .unwrap_or_else(|| (6_u8.saturating_sub(profile.cost_tier) as f64) * 0.01);
        results.push(EmbedBenchModelResult {
            model_id: profile.id.clone(),
            provider: format!("{:?}", profile.provider),
            role: format!("{:?}", profile.role),
            cases: scores.len(),
            mean_score,
            mean_latency_ms,
            mean_cost_units,
            recall_at_1: None,
            mrr: None,
            selection_score: (mean_score - latency_penalty - cost_penalty).max(0.0),
        });
    }
    sort_embed_bench_results(&mut results);
    let selected_model = results.first().map(|result| result.model_id.clone());
    Ok(EmbedBenchReport {
        corpus: bench.corpus.clone(),
        cases: bench.qrels.len(),
        live: false,
        selected_model,
        results,
    })
}

async fn evaluate_live_sidecar_bench(
    rag: &RagClient,
    bench: &EmbedBenchFile,
    args: &EmbedBenchArgs,
) -> anyhow::Result<Vec<EmbedBenchModelResult>> {
    let health = rag.healthz().await.ok();
    let profile = health
        .as_ref()
        .and_then(|health| health.backend.profile.as_deref())
        .unwrap_or("unknown");
    let mut results = Vec::new();
    if let Some(result) = evaluate_live_sidecar_retrieve(rag, bench, args, profile).await? {
        results.push(result);
    }
    if let Some(result) = evaluate_live_sidecar_rerank(rag, bench, profile).await? {
        results.push(result);
    }
    if results.is_empty() {
        anyhow::bail!(
            "live sidecar bench had no scorable qrels; provide relevant_id/relevant_ids and indexed retrieve data or candidates"
        );
    }
    Ok(results)
}

async fn evaluate_live_sidecar_retrieve(
    rag: &RagClient,
    bench: &EmbedBenchFile,
    args: &EmbedBenchArgs,
    profile: &str,
) -> anyhow::Result<Option<EmbedBenchModelResult>> {
    let mut scores = Vec::new();
    let mut latencies = Vec::new();
    let mut top1 = 0usize;
    for case in &bench.qrels {
        let relevant = relevant_ids(case);
        if relevant.is_empty() {
            continue;
        }
        let started = Instant::now();
        let response = rag
            .retrieve(&RagRetrieveRequest {
                query: case.query.clone(),
                project: case.project.clone().or_else(|| args.project.clone()),
                namespace: case.namespace.clone().or_else(|| args.namespace.clone()),
                mode: RagRetrieveMode::Auto,
                limit: Some(args.limit.max(1)),
                include_cross_modal: false,
            })
            .await?;
        latencies.push(started.elapsed().as_secs_f64() * 1000.0);
        let rank = response
            .items
            .iter()
            .position(|item| {
                item.source
                    .as_deref()
                    .is_some_and(|source| relevant.iter().any(|id| id == source))
            })
            .map(|index| index + 1);
        if rank == Some(1) {
            top1 += 1;
        }
        scores.push(rank.map(|rank| 1.0 / rank as f64).unwrap_or(0.0));
    }
    if scores.is_empty() {
        return Ok(None);
    }
    let mean_score = mean(&scores);
    let mean_latency_ms = mean(&latencies);
    let recall_at_1 = top1 as f64 / scores.len() as f64;
    Ok(Some(EmbedBenchModelResult {
        model_id: format!("rag-sidecar:{profile}:retrieve"),
        provider: "Sidecar".to_string(),
        role: "LiveRetrieve".to_string(),
        cases: scores.len(),
        mean_score,
        mean_latency_ms: Some(mean_latency_ms),
        mean_cost_units: Some(0.0),
        recall_at_1: Some(recall_at_1),
        mrr: Some(mean_score),
        selection_score: live_selection_score(mean_score, mean_latency_ms),
    }))
}

async fn evaluate_live_sidecar_rerank(
    rag: &RagClient,
    bench: &EmbedBenchFile,
    profile: &str,
) -> anyhow::Result<Option<EmbedBenchModelResult>> {
    let mut scores = Vec::new();
    let mut latencies = Vec::new();
    let mut top1 = 0usize;
    let mut model = None;
    for case in &bench.qrels {
        let relevant = relevant_ids(case);
        if relevant.is_empty() || case.candidates.is_empty() {
            continue;
        }
        let started = Instant::now();
        let response = rag
            .rerank(&RagRerankRequest {
                query: case.query.clone(),
                candidates: case
                    .candidates
                    .iter()
                    .map(|candidate| RagRerankCandidate {
                        id: candidate.id.clone(),
                        text: candidate.text.clone(),
                    })
                    .collect(),
                top_k: Some(case.candidates.len()),
            })
            .await?;
        latencies.push(started.elapsed().as_secs_f64() * 1000.0);
        model = Some(response.model);
        let rank = response
            .items
            .iter()
            .position(|item| relevant.iter().any(|id| id == &item.id))
            .map(|index| index + 1);
        if rank == Some(1) {
            top1 += 1;
        }
        scores.push(rank.map(|rank| 1.0 / rank as f64).unwrap_or(0.0));
    }
    if scores.is_empty() {
        return Ok(None);
    }
    let mean_score = mean(&scores);
    let mean_latency_ms = mean(&latencies);
    let recall_at_1 = top1 as f64 / scores.len() as f64;
    Ok(Some(EmbedBenchModelResult {
        model_id: format!(
            "rag-sidecar:{}:rerank",
            model.unwrap_or_else(|| profile.to_string())
        ),
        provider: "Sidecar".to_string(),
        role: "LiveRerank".to_string(),
        cases: scores.len(),
        mean_score,
        mean_latency_ms: Some(mean_latency_ms),
        mean_cost_units: Some(0.0),
        recall_at_1: Some(recall_at_1),
        mrr: Some(mean_score),
        selection_score: live_selection_score(mean_score, mean_latency_ms),
    }))
}

fn relevant_ids(case: &EmbedBenchCase) -> Vec<String> {
    let mut ids = case.relevant_ids.clone();
    if let Some(id) = case.relevant_id.as_ref()
        && !ids.iter().any(|existing| existing == id)
    {
        ids.push(id.clone());
    }
    ids
}

fn live_selection_score(mean_score: f64, mean_latency_ms: f64) -> f64 {
    (mean_score - (mean_latency_ms / 10_000.0).clamp(0.0, 0.20)).max(0.0)
}

fn sort_embed_bench_results(results: &mut [EmbedBenchModelResult]) {
    results.sort_by(|left, right| {
        right
            .selection_score
            .total_cmp(&left.selection_score)
            .then_with(|| right.mean_score.total_cmp(&left.mean_score))
            .then_with(|| left.model_id.cmp(&right.model_id))
    });
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len().max(1) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embed_bench_selects_best_scored_model_with_latency_penalty() {
        let registry = memd_core::embedding_registry::EmbeddingModelRegistry::builtin();
        let bench = EmbedBenchFile {
            corpus: Some("demo".to_string()),
            qrels: vec![
                EmbedBenchCase {
                    query: "who owns the correction?".to_string(),
                    relevant_id: Some("m1".to_string()),
                    relevant_ids: Vec::new(),
                    project: None,
                    namespace: None,
                    candidates: Vec::new(),
                    scores: BTreeMap::from([
                        ("text-embedding-3-large".to_string(), 0.92),
                        ("text-embedding-3-small".to_string(), 0.86),
                    ]),
                    latency_ms: BTreeMap::from([
                        ("text-embedding-3-large".to_string(), 300.0),
                        ("text-embedding-3-small".to_string(), 80.0),
                    ]),
                    cost_units: BTreeMap::new(),
                },
                EmbedBenchCase {
                    query: "find path typo".to_string(),
                    relevant_id: Some("m2".to_string()),
                    relevant_ids: Vec::new(),
                    project: None,
                    namespace: None,
                    candidates: Vec::new(),
                    scores: BTreeMap::from([
                        ("text-embedding-3-large".to_string(), 0.90),
                        ("text-embedding-3-small".to_string(), 0.84),
                    ]),
                    latency_ms: BTreeMap::from([
                        ("text-embedding-3-large".to_string(), 320.0),
                        ("text-embedding-3-small".to_string(), 70.0),
                    ]),
                    cost_units: BTreeMap::new(),
                },
            ],
        };

        let report = evaluate_embed_bench(&registry, &bench, Some("cloud")).expect("bench");

        assert_eq!(
            report.selected_model.as_deref(),
            Some("text-embedding-3-large")
        );
        assert_eq!(report.results[0].cases, 2);
    }

    async fn mock_embed_bench_healthz() -> axum::Json<memd_rag::RagBackendHealthResponse> {
        axum::Json(memd_rag::RagBackendHealthResponse {
            status: "ok".to_string(),
            backend: memd_rag::RagBackendHealth {
                connected: true,
                name: Some("rag-sidecar".to_string()),
                multimodal: false,
                profile: Some("fastembed:test".to_string()),
                indexed_count: Some(2),
            },
        })
    }

    async fn mock_embed_bench_retrieve(
        axum::Json(req): axum::Json<memd_rag::RagRetrieveRequest>,
    ) -> axum::Json<memd_rag::RagRetrieveResponse> {
        assert_eq!(req.project.as_deref(), Some("memd"));
        axum::Json(memd_rag::RagRetrieveResponse {
            status: "ok".to_string(),
            mode: memd_rag::RagRetrieveMode::Text,
            items: vec![
                memd_rag::RagRetrieveItem {
                    content: "correct semantic match".to_string(),
                    source: Some("target".to_string()),
                    score: 0.98,
                },
                memd_rag::RagRetrieveItem {
                    content: "distractor".to_string(),
                    source: Some("distractor".to_string()),
                    score: 0.20,
                },
            ],
        })
    }

    async fn mock_embed_bench_rerank(
        axum::Json(req): axum::Json<memd_rag::RagRerankRequest>,
    ) -> axum::Json<memd_rag::RagRerankResponse> {
        let mut items = req
            .candidates
            .iter()
            .map(|candidate| memd_rag::RagRerankItem {
                id: candidate.id.clone(),
                score: if candidate.id == "target" { 0.99 } else { 0.10 },
                text: None,
            })
            .collect::<Vec<_>>();
        items.sort_by(|left, right| right.score.total_cmp(&left.score));
        axum::Json(memd_rag::RagRerankResponse {
            status: "ok".to_string(),
            model: "mock-reranker".to_string(),
            items,
        })
    }

    async fn spawn_mock_embed_bench_rag() -> String {
        let app = axum::Router::new()
            .route("/healthz", axum::routing::get(mock_embed_bench_healthz))
            .route(
                "/v1/retrieve",
                axum::routing::post(mock_embed_bench_retrieve),
            )
            .route("/v1/rerank", axum::routing::post(mock_embed_bench_rerank));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind mock embed bench rag");
        let addr = listener.local_addr().expect("mock embed bench addr");
        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("serve mock embed bench rag");
        });
        format!("http://{}", addr)
    }

    #[tokio::test]
    async fn embed_bench_scores_live_sidecar_retrieve_and_rerank() {
        let rag_url = spawn_mock_embed_bench_rag().await;
        let rag = RagClient::new(&rag_url).expect("rag client");
        let bench = EmbedBenchFile {
            corpus: Some("live-demo".to_string()),
            qrels: vec![EmbedBenchCase {
                query: "semantic continuity recall".to_string(),
                relevant_id: Some("target".to_string()),
                relevant_ids: Vec::new(),
                project: None,
                namespace: None,
                candidates: vec![
                    EmbedBenchCandidate {
                        id: "distractor".to_string(),
                        text: "wrong memory".to_string(),
                    },
                    EmbedBenchCandidate {
                        id: "target".to_string(),
                        text: "right semantic memory".to_string(),
                    },
                ],
                scores: BTreeMap::new(),
                latency_ms: BTreeMap::new(),
                cost_units: BTreeMap::new(),
            }],
        };
        let args = EmbedBenchArgs {
            input: std::path::PathBuf::from("unused.json"),
            target: None,
            rag_url: Some(rag_url),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            limit: 5,
            json: true,
        };

        let mut results = evaluate_live_sidecar_bench(&rag, &bench, &args)
            .await
            .expect("live sidecar bench");
        sort_embed_bench_results(&mut results);

        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|result| result.model_id
            == "rag-sidecar:fastembed:test:retrieve"
            && result.recall_at_1 == Some(1.0)
            && result.mrr == Some(1.0)));
        assert!(results.iter().any(
            |result| result.model_id == "rag-sidecar:mock-reranker:rerank"
                && result.recall_at_1 == Some(1.0)
                && result.mrr == Some(1.0)
        ));
    }
}

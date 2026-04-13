use std::path::Path;

use anyhow::Context;
use clap::Parser;
use memd_client::MemdClient;
use memd_schema::{
    ExpireMemoryRequest, MemoryConsolidationRequest, MemoryDecayRequest, MemoryKind, MemoryScope,
    MemoryStage, MemoryStatus, ProcedureDetectRequest, SearchMemoryRequest, VerifyMemoryRequest,
};
use tokio::time::{Duration, sleep};

#[derive(Debug, Parser)]
#[command(name = "memd-worker")]
#[command(about = "Background verification worker for memd")]
struct Args {
    #[arg(long, default_value = "http://127.0.0.1:8787")]
    base_url: String,

    #[arg(long, default_value_t = 300)]
    interval_secs: u64,

    #[arg(long, default_value_t = 64)]
    batch_size: usize,

    #[arg(long, default_value_t = 0.05)]
    confidence_bump: f32,

    #[arg(long)]
    report: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let client = MemdClient::new(&args.base_url)?;

    let health = client.healthz().await.context("check memd health")?;
    println!(
        "memd-worker connected to {} with {} persisted items",
        args.base_url, health.items
    );

    loop {
        let result = run_once(&client, &args).await?;
        if args.report {
            println!(
                "learning report: reinforced={} cooled={} consolidated={} procedures={} stale_checked={} skipped={}",
                result.verified,
                result.decayed,
                result.consolidated,
                result.procedures_detected,
                result.expired,
                result.skipped
            );
        } else {
            println!(
                "verification pass complete: verified={}, expired={}, decayed={}, consolidated={}, procedures={}, skipped={}",
                result.verified,
                result.expired,
                result.decayed,
                result.consolidated,
                result.procedures_detected,
                result.skipped
            );
        }
        sleep(Duration::from_secs(args.interval_secs)).await;
    }
}

struct WorkerResult {
    verified: usize,
    expired: usize,
    decayed: usize,
    consolidated: usize,
    procedures_detected: usize,
    skipped: usize,
}

async fn run_once(client: &MemdClient, args: &Args) -> anyhow::Result<WorkerResult> {
    let stale_items = client
        .search(&SearchMemoryRequest {
            query: None,
            route: None,
            intent: None,
            scopes: vec![
                MemoryScope::Local,
                MemoryScope::Synced,
                MemoryScope::Project,
                MemoryScope::Global,
            ],
            kinds: vec![
                MemoryKind::Fact,
                MemoryKind::Decision,
                MemoryKind::Preference,
                MemoryKind::Runbook,
                MemoryKind::Procedural,
                MemoryKind::SelfModel,
                MemoryKind::Topology,
                MemoryKind::Status,
                MemoryKind::Pattern,
                MemoryKind::Constraint,
            ],
            statuses: vec![MemoryStatus::Stale],
            project: None,
            namespace: None,
            workspace: None,
            visibility: None,
            belief_branch: None,
            source_agent: None,
            tags: Vec::new(),
            stages: vec![MemoryStage::Canonical],
            limit: Some(args.batch_size),
            max_chars_per_item: Some(160),
        })
        .await
        .context("load stale items")?;

    let mut verified = 0usize;
    let mut expired = 0usize;

    for item in stale_items.items {
        match verify_or_expire(client, &item.content, args.confidence_bump, &item).await? {
            VerificationAction::Verified => verified += 1,
            VerificationAction::Expired => expired += 1,
        }
    }

    let decay = client
        .decay(&MemoryDecayRequest {
            max_items: Some(args.batch_size),
            inactive_days: Some(21),
            max_decay: Some(0.12),
            record_events: Some(true),
        })
        .await
        .context("decay memory entities")?;

    let consolidation = client
        .consolidate(&MemoryConsolidationRequest {
            project: None,
            namespace: None,
            max_groups: Some(args.batch_size),
            min_events: Some(3),
            lookback_days: Some(14),
            min_salience: Some(0.22),
            record_events: Some(true),
        })
        .await
        .context("consolidate semantic memory")?;

    let procedures = client
        .procedure_detect(&ProcedureDetectRequest {
            project: None,
            namespace: None,
            min_events: Some(3),
            lookback_days: Some(14),
            max_candidates: Some(5),
        })
        .await
        .context("detect procedures")?;

    Ok(WorkerResult {
        verified,
        expired,
        decayed: decay.updated,
        consolidated: consolidation.consolidated,
        procedures_detected: procedures.created,
        skipped: 0,
    })
}

enum VerificationAction {
    Verified,
    Expired,
}

async fn verify_or_expire(
    client: &MemdClient,
    _content: &str,
    confidence_bump: f32,
    item: &memd_schema::MemoryItem,
) -> anyhow::Result<VerificationAction> {
    if let Some(source_path) = &item.source_path
        && Path::new(source_path).exists()
    {
        let confidence = (item.confidence + confidence_bump).min(1.0);
        client
            .verify(&VerifyMemoryRequest {
                id: item.id,
                confidence: Some(confidence),
                status: Some(MemoryStatus::Active),
            })
            .await
            .context("verify memory item")?;
        return Ok(VerificationAction::Verified);
    }

    client
        .expire(&ExpireMemoryRequest {
            id: item.id,
            status: Some(MemoryStatus::Expired),
        })
        .await
        .context("expire unverifiable memory item")?;
    Ok(VerificationAction::Expired)
}

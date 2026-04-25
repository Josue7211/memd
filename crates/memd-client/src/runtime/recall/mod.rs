use super::*;

pub(crate) mod depth;
pub(crate) mod escalation;
pub(crate) mod telemetry;

pub(crate) use depth::{depth_flag_enabled, escalation_hint_enabled, RecallDepth};

use std::time::Instant;

/// Hard cap on records returned at `--depth lookup`, per
/// `docs/contracts/recall-depth.md` ("1–3 records").
pub(crate) const LOOKUP_DEPTH_RECORD_CAP: usize = 3;

pub(crate) async fn dispatch_lookup_with_depth(
    client: &MemdClient,
    base_url: &str,
    args: LookupArgs,
) -> anyhow::Result<()> {
    if !depth_flag_enabled() && args.depth != RecallDepth::Lookup {
        anyhow::bail!("--depth flag is disabled (set MEMD_E4_DEPTH_FLAG=1 to enable)");
    }

    let bundle_root = args.output.clone();
    let session_id = read_bundle_runtime_config(&bundle_root)
        .ok()
        .flatten()
        .and_then(|c| c.session);
    let query = args.query.clone();
    let depth = args.depth;
    let started = Instant::now();

    let (result, records, tokens, hint) = match depth {
        RecallDepth::Wake => {
            let res = run_wake_arm(&args, base_url).await;
            (res, 0_usize, 0_usize, None)
        }
        RecallDepth::Lookup => {
            let outcome = run_lookup_arm_inner(client, args).await;
            match outcome {
                Ok(out) => {
                    let hint = out.escalation_hint.clone();
                    if let Some(h) = hint.as_deref() {
                        eprintln!("{h}");
                    }
                    let render_result: anyhow::Result<()> = if out.json {
                        crate::print_json(&out.response)
                    } else {
                        println!("{}", out.markdown);
                        Ok(())
                    };
                    let records = out.response.items.len();
                    let tokens = telemetry::approx_tokens(out.markdown.len());
                    (render_result, records, tokens, hint)
                }
                Err(err) => (Err(err), 0, 0, None),
            }
        }
        RecallDepth::Resume => {
            let res = run_resume_arm(&args, base_url).await;
            (res, 0, 0, None)
        }
    };

    let _ = telemetry::record(telemetry::RecordOpts {
        bundle_root: &bundle_root,
        session_id: session_id.as_deref(),
        query: &query,
        depth,
        records_returned: records,
        tokens_returned: tokens,
        latency_ms: started.elapsed().as_millis() as u64,
        escalation_hint: hint.as_deref(),
    });

    result
}

async fn run_wake_arm(args: &LookupArgs, base_url: &str) -> anyhow::Result<()> {
    let wake_args = synth_wake_args(args);
    crate::run_bundle_wake_command(&wake_args, base_url).await
}

async fn run_resume_arm(args: &LookupArgs, base_url: &str) -> anyhow::Result<()> {
    let resume_args = synth_resume_args(args);
    let snapshot = read_bundle_resume(&resume_args, base_url).await?;
    crate::print_json(&snapshot)
}

pub(crate) struct LookupArmOutcome {
    pub(crate) response: memd_schema::SearchMemoryResponse,
    pub(crate) markdown: String,
    pub(crate) json: bool,
    pub(crate) escalation_hint: Option<String>,
}

pub(crate) async fn run_lookup_arm_inner(
    client: &MemdClient,
    args: LookupArgs,
) -> anyhow::Result<LookupArmOutcome> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    let mut args = crate::cli::apply_lookup_bundle_defaults(args, runtime.as_ref());
    args.limit = Some(clamp_lookup_limit(args.limit));
    let req = build_lookup_request(&args, runtime.as_ref())?;
    let response = lookup_with_fallbacks(client, &req, &args.query).await?;
    let escalation_hint = (response.items.is_empty()
        && escalation_hint_enabled()
        && escalation::detect(&args.query))
    .then(|| escalation::hint_line(&args.query));
    let markdown = render_lookup_markdown(&args.query, &req, &response, args.verbose);
    Ok(LookupArmOutcome {
        response,
        markdown,
        json: args.json,
        escalation_hint,
    })
}

pub(crate) fn clamp_lookup_limit(limit: Option<usize>) -> usize {
    let raw = limit.unwrap_or(LOOKUP_DEPTH_RECORD_CAP);
    raw.min(LOOKUP_DEPTH_RECORD_CAP).max(1)
}

pub(crate) fn synth_wake_args(args: &LookupArgs) -> WakeArgs {
    WakeArgs {
        output: args.output.clone(),
        project: args.project.clone(),
        namespace: args.namespace.clone(),
        agent: None,
        workspace: args.workspace.clone(),
        visibility: args.visibility.clone(),
        route: args.route.clone(),
        intent: args.intent.clone(),
        limit: args.limit,
        rehydration_limit: None,
        semantic: false,
        verbose: args.verbose,
        write: false,
        summary: false,
        raw: false,
        budget_tokens: 0,
        include_bucket: Vec::new(),
        exclude_bucket: Vec::new(),
    }
}

pub(crate) fn synth_resume_args(args: &LookupArgs) -> ResumeArgs {
    ResumeArgs {
        output: args.output.clone(),
        project: args.project.clone(),
        namespace: args.namespace.clone(),
        agent: None,
        workspace: args.workspace.clone(),
        visibility: args.visibility.clone(),
        route: args.route.clone(),
        intent: args.intent.clone(),
        limit: args.limit,
        rehydration_limit: None,
        semantic: false,
        prompt: false,
        summary: false,
    }
}

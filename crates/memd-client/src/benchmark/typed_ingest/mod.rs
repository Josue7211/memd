//! V6 typed-ingest module root.
//!
//! A6 lands episodic adapters across the four public benches (LME, LoCoMo,
//! MemBench, ConvoMem). B6 layers semantic distillation on top; C6 promotes
//! to canonical. See `docs/phases/v6/phase-a6-plan.md` and
//! `docs/phases/v6/V6-INTEGRATION.md`.

pub(crate) mod episodic;
pub(crate) mod bench_loaders;
pub(crate) mod ingest_card;
pub(crate) mod distiller;
pub(crate) mod dedupe;
pub(crate) mod candidate_store;
pub(crate) mod promotion;
pub(crate) mod canonical_index;
pub(crate) mod compiler;

pub(crate) use episodic::{EpisodicAdapter, EpisodicProvenance};

use std::path::Path;

use anyhow::{anyhow, Result};

use bench_loaders::{
    convomem::ConvomemAdapter, lme::LmeAdapter, locomo::LocomoAdapter,
    membench::MembenchAdapter,
};

/// Format the user-visible notice emitted by the runtime when
/// `--typed-ingest=…` is set. Pure — runtime calls this and forwards
/// to eprintln; tests exercise it directly. `distill_model` is the
/// already-env-resolved value (see `distiller::effective_distill_model`).
/// `promotion_dry_run` is the env-resolved C6 dry-run flag (see
/// `promotion_dry_run_active`); only surfaced when the mode includes
/// `+canonical`.
pub(crate) fn typed_ingest_runtime_notice(
    mode: &str,
    env_active: bool,
    distill_model: &str,
    budget_milli_usd: u64,
    cache_enabled: bool,
    promotion_dry_run: bool,
) -> String {
    let semantic_on = mode == "episodic+semantic" || mode == "episodic+semantic+canonical";
    let canonical_on = mode == "episodic+semantic+canonical";
    let distill_note = if semantic_on {
        format!(
            " distill_model={} budget_milli_usd={} cache={}",
            distill_model,
            budget_milli_usd,
            if cache_enabled { "on" } else { "off" }
        )
    } else {
        String::new()
    };
    let canonical_note = if canonical_on {
        format!(
            " promotion_rule={} dry_run={}",
            promotion::PROMOTION_RULE_VERSION,
            if promotion_dry_run { "on" } else { "off" }
        )
    } else {
        String::new()
    };
    let activation = if env_active {
        "ACTIVE (preview)"
    } else {
        "gated — flag is a no-op until A6.9"
    };
    format!(
        "[bench] --typed-ingest={} recognised;{}{} runtime activation {} (env MEMD_V6_TYPED_INGEST=1 graduates in A6.9; C6 promotion shares the same gate)",
        mode, distill_note, canonical_note, activation
    )
}

/// C6 dry-run resolution: env `MEMD_V6_PROMOTION_DRY_RUN=1` always
/// forces dry-run; otherwise falls back to the CLI flag. Pure read.
pub(crate) fn promotion_dry_run_active(cli_flag: bool) -> bool {
    if std::env::var("MEMD_V6_PROMOTION_DRY_RUN")
        .ok()
        .as_deref()
        == Some("1")
    {
        return true;
    }
    cli_flag
}

/// D6 compiler resolution: env `MEMD_V6_COMPILER=1` always forces the
/// compiler ON; otherwise the CLI flag wins. `cli_value` is the raw
/// `--compiler=on|off` value. Pure read.
pub(crate) fn compiler_active(cli_value: &str) -> bool {
    if std::env::var("MEMD_V6_COMPILER").ok().as_deref() == Some("1") {
        return true;
    }
    cli_value == "on"
}

/// User-visible notice emitted when `--compiler=…` is set on a public
/// benchmark run. Pure — runtime forwards to eprintln. The `off` mode
/// must produce no compiler-specific text other than the resolution
/// echo so the flat-RAG path stays observably unchanged (test 6).
pub(crate) fn compiler_runtime_notice(mode: &str, env_active: bool) -> String {
    let active = compiler_active(mode);
    let resolution = if env_active && mode != "on" {
        " (env MEMD_V6_COMPILER=1 overrode --compiler=off)"
    } else {
        ""
    };
    if active {
        format!(
            "[bench] --compiler=on engaged; budgets={}{}",
            compiler::default_budgets_path(),
            resolution
        )
    } else {
        format!(
            "[bench] --compiler={} (off-path: legacy flat-RAG prompt unchanged)",
            mode
        )
    }
}

/// Outcome of a typed-ingest dispatch — counts and provenance hashes
/// (deterministic enough for ingest-card baseline locks in A6.8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TypedIngestReport {
    pub bench_id: &'static str,
    pub turn_count: usize,
    pub session_count: usize,
}

/// Maps a public-benchmark dataset id to its episodic adapter and walks
/// the full turn stream, producing a `TypedIngestReport`. Pure: no
/// network, no server. Runtime activation (CLI → live ingest) graduates
/// in A6.9 once the calendar gate clears.
pub(crate) fn dispatch_typed_ingest_episodic(
    dataset_id: &str,
    path: &Path,
) -> Result<TypedIngestReport> {
    let mut sessions = std::collections::BTreeSet::<String>::new();
    let mut turn_count = 0usize;
    let bench_id: &'static str = match dataset_id {
        "longmemeval" => {
            let mut a = LmeAdapter::from_path(path)?;
            while let Some(t) = a.next_turn() {
                sessions.insert(t.provenance.session_id);
                turn_count += 1;
            }
            bench_loaders::lme::BENCH_ID
        }
        "locomo" => {
            let mut a = LocomoAdapter::from_path(path)?;
            while let Some(t) = a.next_turn() {
                sessions.insert(t.provenance.session_id);
                turn_count += 1;
            }
            bench_loaders::locomo::BENCH_ID
        }
        "membench" => {
            let mut a = MembenchAdapter::from_path(path)?;
            while let Some(t) = a.next_turn() {
                sessions.insert(t.provenance.session_id);
                turn_count += 1;
            }
            bench_loaders::membench::BENCH_ID
        }
        "convomem" => {
            let mut a = ConvomemAdapter::from_path(path)?;
            while let Some(t) = a.next_turn() {
                sessions.insert(t.provenance.session_id);
                turn_count += 1;
            }
            bench_loaders::convomem::BENCH_ID
        }
        other => {
            return Err(anyhow!(
                "typed-ingest=episodic does not support dataset `{}`",
                other
            ));
        }
    };
    Ok(TypedIngestReport {
        bench_id,
        turn_count,
        session_count: sessions.len(),
    })
}

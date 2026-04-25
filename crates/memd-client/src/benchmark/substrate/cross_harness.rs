//! C5 cross-harness runner.
//!
//! Drives a write-side harness through a deterministic `Script`, then a
//! read-side harness through the matching read script. Records:
//!   * `truth_conservation_rate` — share of project-scope reads whose
//!     hits include the writer's tag.
//!   * `visibility_leak_count` — hard 0 floor: any `Local`-scope hit
//!     on the reader that originated from another harness counts as a
//!     leak (real cross-harness contamination), and any `Project`-scope
//!     hit whose source project does not match the reader's project
//!     also counts as a leak (cross-project contamination).
//!
//! Scenarios are seed-deterministic; the runner reuses the perfect-recall
//! `InMemoryGateway` for in-process runs, the same way B5 reuses
//! `InProcessB5Backend`.

use crate::benchmark::substrate::harness_adapter::{
    HarnessAdapter, HarnessRunOutcome, InMemoryGateway, MemdGateway, ReadResult, Scope, Script,
    ScriptStep,
};
use crate::benchmark::substrate::report::{append_ndjson, ScenarioRecord};
use chrono::Utc;
use std::path::{Path, PathBuf};
use std::time::Instant;
use uuid::Uuid;

/// Pass-gate per `phase-c5-plan.md` §2.
#[derive(Debug, Clone, Copy)]
pub(crate) struct C5PassGate {
    pub(crate) truth_conservation_rate: f64,
    pub(crate) visibility_leak_count: u64,
    pub(crate) latency_p95_ms: u64,
}

impl Default for C5PassGate {
    fn default() -> Self {
        Self {
            truth_conservation_rate: 0.95,
            visibility_leak_count: 0,
            latency_p95_ms: 2_000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum C5Scenario {
    FactRoundtrip,
    PreferenceRoundtrip,
    CorrectionRoundtrip,
}

impl C5Scenario {
    pub(crate) fn id(&self) -> &'static str {
        match self {
            C5Scenario::FactRoundtrip => "fact_roundtrip",
            C5Scenario::PreferenceRoundtrip => "preference_roundtrip",
            C5Scenario::CorrectionRoundtrip => "correction_roundtrip",
        }
    }

    pub(crate) fn kind(&self) -> &'static str {
        match self {
            C5Scenario::FactRoundtrip => "fact",
            C5Scenario::PreferenceRoundtrip => "preference",
            C5Scenario::CorrectionRoundtrip => "correction",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct C5RunConfig {
    pub(crate) seed: u64,
    pub(crate) pairs: Vec<(String, String)>,
    pub(crate) scenarios: Vec<C5Scenario>,
    pub(crate) per_scenario_facts: usize,
    pub(crate) project: String,
    pub(crate) pass_gate: C5PassGate,
    pub(crate) results_dir: PathBuf,
}

impl C5RunConfig {
    pub(crate) fn default_with_results_dir(results_dir: PathBuf) -> Self {
        Self {
            seed: 44,
            pairs: vec![
                ("claude_code".into(), "codex".into()),
                ("codex".into(), "claude_code".into()),
            ],
            scenarios: vec![
                C5Scenario::FactRoundtrip,
                C5Scenario::PreferenceRoundtrip,
                C5Scenario::CorrectionRoundtrip,
            ],
            per_scenario_facts: 10,
            project: "c5-demo".into(),
            pass_gate: C5PassGate::default(),
            results_dir,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct VisibilityLeak {
    pub(crate) reader_harness: String,
    pub(crate) source_harness: String,
    pub(crate) source_scope: Scope,
    pub(crate) tag: String,
}

#[derive(Debug, Clone)]
pub(crate) struct C5Outcome {
    pub(crate) records: Vec<ScenarioRecord>,
    pub(crate) ndjson_path: PathBuf,
    pub(crate) overall_pass: bool,
    pub(crate) leaks: Vec<VisibilityLeak>,
}

/// Drive every `(writer, reader)` × scenario combination through the
/// supplied adapters and gateway. The shared gateway is the canonical
/// arrangement: production memd is the gateway, claude+codex are
/// independent clients of it.
pub(crate) fn run_c5_with_adapters(
    config: &C5RunConfig,
    adapters: &[(&str, &dyn HarnessAdapter)],
    gateway: &dyn MemdGateway,
) -> std::io::Result<C5Outcome> {
    let run_id = Uuid::new_v4().to_string();
    let ts_ms = Utc::now().timestamp_millis();

    let lookup_adapter = |name: &str| -> Option<&dyn HarnessAdapter> {
        adapters.iter().find(|(n, _)| *n == name).map(|(_, a)| *a)
    };

    let mut records = Vec::new();
    let mut leaks = Vec::new();
    let mut overall_pass = true;
    let mut pair_idx = 0usize;

    for (writer_name, reader_name) in &config.pairs {
        let writer = lookup_adapter(writer_name)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, format!("missing adapter: {writer_name}")))?;
        let reader = lookup_adapter(reader_name)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, format!("missing adapter: {reader_name}")))?;

        for scenario in &config.scenarios {
            let (write_script, read_script) =
                build_scripts(*scenario, &config.project, config.seed, config.per_scenario_facts, pair_idx);

            let t0 = Instant::now();
            let write_outcome = writer.run_script(&write_script, gateway)?;
            let read_outcome = reader.run_script(&read_script, gateway)?;
            let elapsed_ms = t0.elapsed().as_millis() as u64;

            let truth = score_truth_conservation(&read_outcome);
            let pair_leaks =
                score_visibility_leaks(reader_name, &config.project, &read_outcome);
            for l in &pair_leaks {
                leaks.push(l.clone());
            }
            let leak_count = pair_leaks.len() as u64;

            let pass = truth >= config.pass_gate.truth_conservation_rate
                && leak_count <= config.pass_gate.visibility_leak_count
                && elapsed_ms <= config.pass_gate.latency_p95_ms;
            if !pass {
                overall_pass = false;
            }

            records.push(ScenarioRecord {
                suite: format!("cross-harness::{}::{}->{}", scenario.id(), writer_name, reader_name),
                run_id: run_id.clone(),
                ts_ms,
                seed: config.seed,
                fact_count: write_outcome.writes.len(),
                cut_k: pair_idx,
                recall_at_1: truth,
                recall_at_3: leak_count as f64,
                answer_exact_match: truth,
                tokens_per_recall: 0,
                latency_ms_p50: 0,
                latency_ms_p95: elapsed_ms,
                pass,
            });
        }
        pair_idx += 1;
    }

    let ndjson_path = ndjson_path_for(&config.results_dir, ts_ms);
    if !records.is_empty() {
        append_ndjson(&ndjson_path, &records)?;
    }
    write_run_metadata(&config.results_dir, &run_id, ts_ms, config)?;

    Ok(C5Outcome {
        records,
        ndjson_path,
        overall_pass,
        leaks,
    })
}

/// Default in-process driver: spins up the perfect-recall gateway +
/// real claude-code/codex adapters. Used by `run_substrate_command`
/// in the dispatcher.
pub(crate) fn run_c5_in_process(config: &C5RunConfig) -> std::io::Result<C5Outcome> {
    use crate::benchmark::substrate::harness_adapter::{
        claude_code::ClaudeCodeAdapter, codex::CodexAdapter,
    };
    let gateway = InMemoryGateway::new();
    let claude = ClaudeCodeAdapter::from_home();
    let codex = CodexAdapter::from_home();
    let adapters: Vec<(&str, &dyn HarnessAdapter)> = vec![
        ("claude_code", &claude),
        ("codex", &codex),
    ];
    let allow_skip = allow_skip_from_env();
    run_c5_with_skip(config, &adapters, &gateway, allow_skip)
}

fn allow_skip_from_env() -> bool {
    // CI defaults to allow-skip per `phase-c5-plan.md` §7. Locally the
    // operator must opt in explicitly so missing harness configs surface
    // as failures rather than silent zero-record runs.
    match std::env::var("MEMD_SUBSTRATE_C5_HARNESS_ALLOW_SKIP") {
        Ok(v) => v != "0",
        Err(_) => std::env::var("CI").is_ok(),
    }
}

/// Filter pairs by adapter availability before driving. When a harness
/// is unavailable AND `allow_skip` is true, every pair touching it is
/// dropped; if no pairs survive, the outcome is an empty-but-passing
/// record set (CI graceful skip per §7).
pub(crate) fn run_c5_with_skip(
    config: &C5RunConfig,
    adapters: &[(&str, &dyn HarnessAdapter)],
    gateway: &dyn MemdGateway,
    allow_skip: bool,
) -> std::io::Result<C5Outcome> {
    use std::collections::HashMap;
    let availability: HashMap<String, bool> = adapters
        .iter()
        .map(|(n, a)| ((*n).to_string(), a.is_available()))
        .collect();

    let referenced: std::collections::BTreeSet<&str> = config
        .pairs
        .iter()
        .flat_map(|(w, r)| [w.as_str(), r.as_str()])
        .collect();
    let unavailable: Vec<&str> = referenced
        .iter()
        .filter(|n| !availability.get(**n).copied().unwrap_or(false))
        .copied()
        .collect();

    if !unavailable.is_empty() && !allow_skip {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "cross-harness: required harnesses unavailable: {unavailable:?}; \
                 set MEMD_SUBSTRATE_C5_HARNESS_ALLOW_SKIP=1 to skip"
            ),
        ));
    }

    let mut filtered = config.clone();
    let original_pair_count = filtered.pairs.len();
    if allow_skip {
        filtered.pairs.retain(|(w, r)| {
            availability.get(w).copied().unwrap_or(false)
                && availability.get(r).copied().unwrap_or(false)
        });
    }

    let skipped = original_pair_count - filtered.pairs.len();
    if filtered.pairs.is_empty() {
        // Empty passing run; record a `runs.jsonl` skip note so CI
        // logs surface the reason instead of pretending all is well.
        let ts_ms = Utc::now().timestamp_millis();
        let run_id = Uuid::new_v4().to_string();
        write_skip_metadata(&filtered.results_dir, &run_id, ts_ms, &availability, skipped)?;
        return Ok(C5Outcome {
            records: Vec::new(),
            ndjson_path: ndjson_path_for(&filtered.results_dir, ts_ms),
            overall_pass: true,
            leaks: Vec::new(),
        });
    }

    run_c5_with_adapters(&filtered, adapters, gateway)
}

fn write_skip_metadata(
    results_dir: &Path,
    run_id: &str,
    ts_ms: i64,
    availability: &std::collections::HashMap<String, bool>,
    skipped_pairs: usize,
) -> std::io::Result<()> {
    use std::io::Write;
    std::fs::create_dir_all(results_dir)?;
    let runs_jsonl = results_dir.join("runs.jsonl");
    let row = serde_json::json!({
        "suite": "cross-harness",
        "run_id": run_id,
        "ts_ms": ts_ms,
        "skipped": true,
        "skipped_pair_count": skipped_pairs,
        "availability": availability,
    });
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&runs_jsonl)?;
    f.write_all(format!("{row}\n").as_bytes())
}

fn ndjson_path_for(results_dir: &Path, ts_ms: i64) -> PathBuf {
    let date = chrono::DateTime::<Utc>::from_timestamp_millis(ts_ms)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "unknown-date".into());
    results_dir.join(format!("cross-harness-{date}.ndjson"))
}

fn write_run_metadata(
    results_dir: &Path,
    run_id: &str,
    ts_ms: i64,
    config: &C5RunConfig,
) -> std::io::Result<()> {
    use std::io::Write;
    std::fs::create_dir_all(results_dir)?;
    let runs_jsonl = results_dir.join("runs.jsonl");
    let row = serde_json::json!({
        "suite": "cross-harness",
        "run_id": run_id,
        "ts_ms": ts_ms,
        "seed": config.seed,
        "pairs": config.pairs,
        "scenarios": config.scenarios.iter().map(|s| s.id()).collect::<Vec<_>>(),
        "per_scenario_facts": config.per_scenario_facts,
        "project": config.project,
    });
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&runs_jsonl)?;
    let line = format!("{row}\n");
    f.write_all(line.as_bytes())
}

/// Build a deterministic `(write_script, read_script)` pair for a
/// scenario. Each fact is planted at both `Project` and `Local` scope
/// so the read side can audit both: project must round-trip,
/// local must NOT.
fn build_scripts(
    scenario: C5Scenario,
    project: &str,
    seed: u64,
    n: usize,
    pair_idx: usize,
) -> (Script, Script) {
    let kind = scenario.kind();
    let mut write_steps = Vec::with_capacity(n * 2);
    let mut read_steps = Vec::with_capacity(n * 2);
    for i in 0..n {
        let tag = format!("{}-{seed}-p{pair_idx}-{i:03}", scenario.id());
        let proj_tag = format!("{tag}-proj");
        let local_tag = format!("{tag}-local");
        write_steps.push(ScriptStep::Write {
            kind: kind.into(),
            content: format!("{tag} payload {i}"),
            scope: Scope::Project,
            tag: proj_tag.clone(),
        });
        write_steps.push(ScriptStep::Write {
            kind: kind.into(),
            content: format!("{tag} secret {i}"),
            scope: Scope::Local,
            tag: local_tag.clone(),
        });
        read_steps.push(ScriptStep::Read {
            query: proj_tag.clone(),
            scope: Scope::Project,
            expect_tag: proj_tag.clone(),
        });
        read_steps.push(ScriptStep::Read {
            query: local_tag.clone(),
            scope: Scope::Local,
            expect_tag: local_tag.clone(),
        });
    }
    let write_script = Script {
        project: project.into(),
        steps: write_steps,
    };
    let read_script = Script {
        project: project.into(),
        steps: read_steps,
    };
    (write_script, read_script)
}

/// Truth conservation: share of `Project`-scope reads where at least
/// one returned hit carries the expected tag. Local-scope reads are
/// excluded because they SHOULD return zero hits — counting them as
/// "missed truth" would conflate isolation with availability.
fn score_truth_conservation(outcome: &HarnessRunOutcome) -> f64 {
    let project_reads: Vec<&ReadResult> = outcome
        .reads
        .iter()
        .filter(|r| r.requested_scope == Scope::Project)
        .collect();
    if project_reads.is_empty() {
        return 0.0;
    }
    let mut hits = 0usize;
    for r in &project_reads {
        if r.hits.iter().any(|h| h.tag == r.expect_tag) {
            hits += 1;
        }
    }
    hits as f64 / project_reads.len() as f64
}

/// Visibility audit. A leak is any reader hit that should not have
/// crossed the harness boundary:
///   * Local-scope hit whose source harness ≠ reader.
///   * Any-scope hit whose source project ≠ reader's project.
fn score_visibility_leaks(
    reader_harness: &str,
    reader_project: &str,
    outcome: &HarnessRunOutcome,
) -> Vec<VisibilityLeak> {
    let mut leaks = Vec::new();
    for r in &outcome.reads {
        for h in &r.hits {
            let cross_project = h
                .source_project
                .as_deref()
                .map(|p| p != reader_project)
                .unwrap_or(false);
            let cross_harness_local =
                h.source_scope == Scope::Local && h.source_harness != reader_harness;
            if cross_harness_local || cross_project {
                leaks.push(VisibilityLeak {
                    reader_harness: reader_harness.to_string(),
                    source_harness: h.source_harness.clone(),
                    source_scope: h.source_scope,
                    tag: h.tag.clone(),
                });
            }
        }
    }
    leaks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::benchmark::substrate::harness_adapter::{
        claude_code::ClaudeCodeAdapter, codex::CodexAdapter, InMemoryGateway,
    };
    use std::fs;
    use tempfile::tempdir;

    fn ready_adapters(dir: &Path) -> (ClaudeCodeAdapter, CodexAdapter) {
        let claude_settings = dir.join("settings.json");
        let codex_hooks = dir.join("hooks.json");
        fs::write(&claude_settings, "{}").unwrap();
        fs::write(&codex_hooks, "{}").unwrap();
        (
            ClaudeCodeAdapter::with_config_path(claude_settings),
            CodexAdapter::with_config_path(codex_hooks),
        )
    }

    /// C5 Test 4 — `runner_roundtrips_fact_claude_to_codex`.
    /// Project-scope writes from claude survive a codex read; local
    /// writes do not.
    #[test]
    fn runner_roundtrips_fact_claude_to_codex() {
        let dir = tempdir().unwrap();
        let (claude, codex) = ready_adapters(dir.path());
        let gateway = InMemoryGateway::new();

        let cfg = C5RunConfig {
            pairs: vec![("claude_code".into(), "codex".into())],
            scenarios: vec![C5Scenario::FactRoundtrip],
            per_scenario_facts: 5,
            results_dir: dir.path().to_path_buf(),
            ..C5RunConfig::default_with_results_dir(dir.path().to_path_buf())
        };
        let adapters: Vec<(&str, &dyn HarnessAdapter)> =
            vec![("claude_code", &claude), ("codex", &codex)];
        let outcome = run_c5_with_adapters(&cfg, &adapters, &gateway).unwrap();

        assert_eq!(outcome.records.len(), 1);
        let record = &outcome.records[0];
        assert!(
            (record.recall_at_1 - 1.0).abs() < f64::EPSILON,
            "perfect gateway must conserve truth on project scope: got {:.3}",
            record.recall_at_1
        );
        assert_eq!(outcome.leaks.len(), 0);
        assert!(outcome.overall_pass);
    }

    /// C5 Test 5 — `runner_visibility_leak_zero_on_project_scope`.
    /// Full pair matrix + all scenarios: zero leaks under perfect gateway.
    #[test]
    fn runner_visibility_leak_zero_on_project_scope() {
        let dir = tempdir().unwrap();
        let (claude, codex) = ready_adapters(dir.path());
        let gateway = InMemoryGateway::new();

        let cfg = C5RunConfig {
            results_dir: dir.path().to_path_buf(),
            per_scenario_facts: 5,
            ..C5RunConfig::default_with_results_dir(dir.path().to_path_buf())
        };
        let adapters: Vec<(&str, &dyn HarnessAdapter)> =
            vec![("claude_code", &claude), ("codex", &codex)];
        let outcome = run_c5_with_adapters(&cfg, &adapters, &gateway).unwrap();

        assert_eq!(outcome.records.len(), cfg.pairs.len() * cfg.scenarios.len());
        assert_eq!(outcome.leaks.len(), 0, "perfect gateway must produce zero leaks");
        for r in &outcome.records {
            assert!(r.pass, "scenario {} failed", r.suite);
            assert!(
                (r.recall_at_1 - 1.0).abs() < f64::EPSILON,
                "{} truth conservation = {}",
                r.suite,
                r.recall_at_1
            );
            assert!((r.recall_at_3 - 0.0).abs() < f64::EPSILON, "{} leaks > 0", r.suite);
        }
        assert!(outcome.overall_pass);
    }

    /// C5 Test 6 — `runner_visibility_leak_detected_on_planted_breach`.
    /// Gateway with `leak_local=true` returns local-scope writes across
    /// harnesses; auditor must catch every planted breach and the run
    /// must fail the hard floor.
    #[test]
    fn runner_visibility_leak_detected_on_planted_breach() {
        let dir = tempdir().unwrap();
        let (claude, codex) = ready_adapters(dir.path());
        let leaky = InMemoryGateway::with_leak_local();

        let cfg = C5RunConfig {
            pairs: vec![("claude_code".into(), "codex".into())],
            scenarios: vec![C5Scenario::FactRoundtrip],
            per_scenario_facts: 4,
            results_dir: dir.path().to_path_buf(),
            ..C5RunConfig::default_with_results_dir(dir.path().to_path_buf())
        };
        let adapters: Vec<(&str, &dyn HarnessAdapter)> =
            vec![("claude_code", &claude), ("codex", &codex)];
        let outcome = run_c5_with_adapters(&cfg, &adapters, &leaky).unwrap();

        assert!(!outcome.overall_pass, "leaky gateway must fail the hard floor");
        assert!(
            !outcome.leaks.is_empty(),
            "auditor must flag every cross-harness local-scope read"
        );
        for leak in &outcome.leaks {
            assert_eq!(leak.reader_harness, "codex");
            assert_eq!(leak.source_harness, "claude_code");
            assert_eq!(leak.source_scope, Scope::Local);
        }
        let leak_record = outcome
            .records
            .iter()
            .find(|r| r.suite.contains("fact_roundtrip"))
            .unwrap();
        assert!(leak_record.recall_at_3 > 0.0);
        assert!(!leak_record.pass);
    }
}

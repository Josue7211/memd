use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::Context;
use memd_core::{
    self_tuning::{
        AbBenchResult, CompilerMode, DEFAULT_MAX_BUDGET_REGRESSION_PCT,
        DEFAULT_MAX_QUALITY_REGRESSION, DEFAULT_MIN_QUALITY_SCORE, QualityGuard, TuningProfile,
        TuningTelemetryPoint, build_ab_bench_result, build_tuning_profile,
    },
    telemetry::{TelemetryEvent, read_telemetry_events},
};
use serde::Serialize;
use serde_json::Value as JsonValue;

use crate::{CompilerCommand, CompilerTuneArgs};

pub(crate) fn run_v15_compiler_command(
    output: &Path,
    global_json: bool,
    command: Option<&CompilerCommand>,
) -> anyhow::Result<()> {
    match command {
        Some(CompilerCommand::Tune(args)) => {
            let report = tune_self_tuning_profiles(output, args)?;
            if global_json || args.json {
                crate::print_json(&report)?;
            } else {
                println!(
                    "compiler profiles={} accepted={} path={}",
                    report.profile_count,
                    report.accepted_count,
                    report.profiles_path.display()
                );
            }
        }
        Some(CompilerCommand::Profiles(args)) => {
            let mut profiles = read_self_tuning_profiles(output)?;
            if args.accepted_only {
                profiles.retain(|profile| profile.accepted);
            }
            if global_json || args.json {
                crate::print_json(&profiles)?;
            } else if profiles.is_empty() {
                println!("compiler profiles=0");
            } else {
                for profile in profiles {
                    println!(
                        "user={} harness={} accepted={} baseline={} tuned={} savings_pct={:.2} quality_delta={:.3}",
                        profile.user_hash,
                        profile.harness,
                        profile.accepted,
                        profile.baseline_budget,
                        profile.tuned_budget,
                        profile.token_savings_pct,
                        profile.quality_delta
                    );
                }
            }
        }
        Some(CompilerCommand::AbBench(args)) => {
            let results =
                build_self_tuning_ab_bench(output, args.static_budget, args.dynamic_budget)?;
            if global_json || args.json {
                crate::print_json(&results)?;
            } else {
                for result in results {
                    println!(
                        "user={} harness={} static={} dynamic={} self_tuning={} savings_vs_dynamic_pct={:.2} quality_delta={:.3} accepted={}",
                        result.user_hash,
                        result.harness,
                        result.static_budget,
                        result.dynamic_budget,
                        result.self_tuning_budget,
                        result.token_savings_vs_dynamic_pct,
                        result.quality_delta_vs_dynamic,
                        result.accepted
                    );
                }
            }
        }
        None => {
            let status = compiler_status(output)?;
            if global_json {
                crate::print_json(&status)?;
            } else {
                println!(
                    "compiler mode={} profiles={} accepted={} path={}",
                    status.mode,
                    status.profile_count,
                    status.accepted_count,
                    status.profiles_path.display()
                );
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SelfTuningTuneReport {
    pub(crate) schema_version: u32,
    pub(crate) profile_count: usize,
    pub(crate) accepted_count: usize,
    pub(crate) profiles_path: PathBuf,
    pub(crate) min_token_savings_pct: f64,
    pub(crate) min_quality_delta: f64,
    pub(crate) mode: String,
    pub(crate) profiles: Vec<TuningProfile>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CompilerStatus {
    pub(crate) mode: String,
    pub(crate) profile_count: usize,
    pub(crate) accepted_count: usize,
    pub(crate) profiles_path: PathBuf,
}

pub(crate) fn tune_self_tuning_profiles(
    output: &Path,
    args: &CompilerTuneArgs,
) -> anyhow::Result<SelfTuningTuneReport> {
    let guard = QualityGuard {
        min_samples: args.min_samples,
        min_quality_score: args.min_quality_score,
        max_quality_regression: DEFAULT_MAX_QUALITY_REGRESSION,
        max_budget_regression_pct: DEFAULT_MAX_BUDGET_REGRESSION_PCT,
        tuning_headroom: args.tuning_headroom,
    };
    let profiles = build_profiles_from_telemetry(output, args.baseline_budget, guard)?;
    write_self_tuning_profiles(output, &profiles)?;
    let accepted: Vec<_> = profiles.iter().filter(|profile| profile.accepted).collect();
    let min_token_savings_pct = accepted
        .iter()
        .map(|profile| profile.token_savings_pct)
        .fold(f64::INFINITY, f64::min);
    let min_quality_delta = accepted
        .iter()
        .map(|profile| profile.quality_delta)
        .fold(f64::INFINITY, f64::min);
    Ok(SelfTuningTuneReport {
        schema_version: 1,
        profile_count: profiles.len(),
        accepted_count: accepted.len(),
        profiles_path: self_tuning_profiles_path(output),
        min_token_savings_pct: if accepted.is_empty() {
            0.0
        } else {
            min_token_savings_pct
        },
        min_quality_delta: if accepted.is_empty() {
            0.0
        } else {
            min_quality_delta
        },
        mode: compiler_mode_from_config(output).as_str().to_string(),
        profiles,
    })
}

pub(crate) fn compiler_status(output: &Path) -> anyhow::Result<CompilerStatus> {
    let profiles = read_self_tuning_profiles(output)?;
    Ok(CompilerStatus {
        mode: compiler_mode_from_config(output).as_str().to_string(),
        profile_count: profiles.len(),
        accepted_count: profiles.iter().filter(|profile| profile.accepted).count(),
        profiles_path: self_tuning_profiles_path(output),
    })
}

pub(crate) fn build_profiles_from_telemetry(
    output: &Path,
    baseline_budget: u64,
    guard: QualityGuard,
) -> anyhow::Result<Vec<TuningProfile>> {
    let mut grouped: BTreeMap<(String, String), Vec<TuningTelemetryPoint>> = BTreeMap::new();
    for event in read_telemetry_events(output)? {
        if let Some(point) = telemetry_event_to_tuning_point(&event, baseline_budget) {
            grouped
                .entry((point.user_hash.clone(), point.harness.clone()))
                .or_default()
                .push(point);
        }
    }
    Ok(grouped
        .into_iter()
        .map(|((user_hash, harness), points)| {
            build_tuning_profile(&user_hash, &harness, &points, baseline_budget, guard)
        })
        .collect())
}

pub(crate) fn write_self_tuning_profiles(
    output: &Path,
    profiles: &[TuningProfile],
) -> anyhow::Result<PathBuf> {
    let target = self_tuning_profiles_path(output);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&target, serde_json::to_string_pretty(profiles)? + "\n")
        .with_context(|| format!("write {}", target.display()))?;
    Ok(target)
}

pub(crate) fn read_self_tuning_profiles(output: &Path) -> anyhow::Result<Vec<TuningProfile>> {
    let target = self_tuning_profiles_path(output);
    if !target.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&target).with_context(|| format!("read {}", target.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", target.display()))
}

pub(crate) fn build_self_tuning_ab_bench(
    output: &Path,
    static_budget: u64,
    dynamic_budget: u64,
) -> anyhow::Result<Vec<AbBenchResult>> {
    Ok(read_self_tuning_profiles(output)?
        .iter()
        .map(|profile| build_ab_bench_result(profile, static_budget, dynamic_budget))
        .collect())
}

pub(crate) fn self_tuning_profiles_path(output: &Path) -> PathBuf {
    output.join("compiler").join("tuning-profiles.json")
}

pub(crate) fn compiler_mode_from_config(output: &Path) -> CompilerMode {
    let path = output.join("config.json");
    let raw = fs::read_to_string(path).ok();
    let Some(raw) = raw else {
        return CompilerMode::default();
    };
    let Ok(doc) = serde_json::from_str::<JsonValue>(&raw) else {
        return CompilerMode::default();
    };
    doc.get("compiler")
        .and_then(|compiler| compiler.get("mode"))
        .and_then(JsonValue::as_str)
        .and_then(|mode| CompilerMode::from_str(mode).ok())
        .unwrap_or_default()
}

fn telemetry_event_to_tuning_point(
    event: &TelemetryEvent,
    default_budget: u64,
) -> Option<TuningTelemetryPoint> {
    let quality_score = metadata_f64(&event.metadata, "quality_score")?;
    if !quality_score.is_finite() {
        return None;
    }
    Some(TuningTelemetryPoint {
        user_hash: event.user_hash.clone(),
        harness: event.harness.clone(),
        token_count: event.token_count,
        budget_target: metadata_u64(&event.metadata, "budget_target").unwrap_or(default_budget),
        quality_score,
        baseline_quality_score: metadata_f64(&event.metadata, "baseline_quality_score")
            .unwrap_or(DEFAULT_MIN_QUALITY_SCORE),
    })
}

fn metadata_f64(map: &serde_json::Map<String, JsonValue>, key: &str) -> Option<f64> {
    map.get(key).and_then(JsonValue::as_f64)
}

fn metadata_u64(map: &serde_json::Map<String, JsonValue>, key: &str) -> Option<u64> {
    map.get(key).and_then(JsonValue::as_u64)
}

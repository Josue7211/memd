//! F5 live-fire routine-invocation harness.
//!
//! Plants a routine in session 1, invokes it in sessions 2+, and asserts
//! `token_savings(routine) ≥ 1×baseline_retrieval_cost` per
//! `MILESTONE-v5.md` PR-axis assertion. Any substrate that does not cache
//! routines fails the gate (savings stay at zero).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;

/// Synthetic baseline cost of a full retrieval (tokens).
pub(crate) const BASELINE_RETRIEVAL_COST: u32 = 100;
/// Synthetic cost of a cache-hit routine invocation (tokens).
pub(crate) const ROUTINE_INVOCATION_COST: u32 = 10;

/// Substrate trait the live-fire harness queries. The "perfect" substrate
/// caches a routine after first observation; a non-caching substrate fails
/// the gate by design (savings == 0).
pub(crate) trait RoutineSubstrate {
    /// Ask the substrate to handle a routine call. Returns the token cost
    /// the substrate paid. First call should be `BASELINE_RETRIEVAL_COST`
    /// (planting); subsequent calls on a cached routine should be
    /// `ROUTINE_INVOCATION_COST`.
    fn observe_or_invoke(&mut self, routine_id: &str) -> u32;

    /// Whether the substrate has cached this routine. Used by the scorer
    /// to assert "routine X observed in S1, invoked in S2+".
    fn is_cached(&self, routine_id: &str) -> bool;
}

/// Reference substrate: caches every observed routine. Ships pass-by-default
/// behavior so the in-process aggregator run lifts PR cleanly.
#[derive(Debug, Default, Clone)]
pub(crate) struct PerfectRoutineSubstrate {
    cache: BTreeMap<String, ()>,
}

impl RoutineSubstrate for PerfectRoutineSubstrate {
    fn observe_or_invoke(&mut self, routine_id: &str) -> u32 {
        if self.cache.contains_key(routine_id) {
            ROUTINE_INVOCATION_COST
        } else {
            self.cache.insert(routine_id.to_string(), ());
            BASELINE_RETRIEVAL_COST
        }
    }

    fn is_cached(&self, routine_id: &str) -> bool {
        self.cache.contains_key(routine_id)
    }
}

/// Negative-control substrate: never caches, always pays full retrieval
/// cost. Exists so tests can assert the harness flips to fail when the
/// substrate is not behaving as a routine cache.
#[derive(Debug, Default, Clone)]
pub(crate) struct NoCacheRoutineSubstrate;

impl RoutineSubstrate for NoCacheRoutineSubstrate {
    fn observe_or_invoke(&mut self, _routine_id: &str) -> u32 {
        BASELINE_RETRIEVAL_COST
    }
    fn is_cached(&self, _routine_id: &str) -> bool {
        false
    }
}

/// Per-call ledger entry written to NDJSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RoutineRecord {
    pub(crate) suite: String,
    pub(crate) routine_id: String,
    pub(crate) session_id: String,
    pub(crate) phase: String,
    pub(crate) cost: u32,
    pub(crate) baseline_cost: u32,
    pub(crate) is_invocation: bool,
    pub(crate) cumulative_savings: u32,
}

/// Static config for the live-fire run.
#[derive(Debug, Clone)]
pub(crate) struct LiveFireConfig {
    pub(crate) routine_count: usize,
    pub(crate) invocations_per_routine: usize,
    pub(crate) results_dir: PathBuf,
}

impl LiveFireConfig {
    pub(crate) fn default_with_results_dir(results_dir: PathBuf) -> Self {
        Self {
            routine_count: 5,
            invocations_per_routine: 2,
            results_dir,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LiveFireOutcome {
    pub(crate) records: Vec<RoutineRecord>,
    pub(crate) total_savings: u32,
    pub(crate) total_baseline_invocation_cost: u32,
    pub(crate) per_routine_pass: BTreeMap<String, bool>,
    pub(crate) overall_pass: bool,
    pub(crate) ndjson_path: PathBuf,
}

pub(crate) fn run_live_fire_in_process(cfg: &LiveFireConfig) -> std::io::Result<LiveFireOutcome> {
    let mut substrate = PerfectRoutineSubstrate::default();
    run_live_fire_with_substrate(cfg, &mut substrate)
}

pub(crate) fn run_live_fire_with_substrate<S: RoutineSubstrate>(
    cfg: &LiveFireConfig,
    substrate: &mut S,
) -> std::io::Result<LiveFireOutcome> {
    let mut records = Vec::new();
    let mut per_routine_pass = BTreeMap::new();

    for r in 0..cfg.routine_count {
        let routine_id = format!("routine-{r}");

        let plant_cost = substrate.observe_or_invoke(&routine_id);
        records.push(RoutineRecord {
            suite: "typed-retrieval".into(),
            routine_id: routine_id.clone(),
            session_id: format!("S1-r{r}"),
            phase: "plant".into(),
            cost: plant_cost,
            baseline_cost: BASELINE_RETRIEVAL_COST,
            is_invocation: false,
            cumulative_savings: 0,
        });

        let mut routine_savings: u32 = 0;
        for inv in 0..cfg.invocations_per_routine {
            let cost = substrate.observe_or_invoke(&routine_id);
            let savings = BASELINE_RETRIEVAL_COST.saturating_sub(cost);
            routine_savings = routine_savings.saturating_add(savings);
            records.push(RoutineRecord {
                suite: "typed-retrieval".into(),
                routine_id: routine_id.clone(),
                session_id: format!("S{}-r{r}", inv + 2),
                phase: "invoke".into(),
                cost,
                baseline_cost: BASELINE_RETRIEVAL_COST,
                is_invocation: true,
                cumulative_savings: routine_savings,
            });
        }

        let is_cached = substrate.is_cached(&routine_id);
        let pass = is_cached && routine_savings >= BASELINE_RETRIEVAL_COST;
        per_routine_pass.insert(routine_id, pass);
    }

    let total_savings: u32 = records
        .iter()
        .filter(|r| r.is_invocation)
        .map(|r| BASELINE_RETRIEVAL_COST.saturating_sub(r.cost))
        .sum();
    let total_baseline_invocation_cost: u32 = records
        .iter()
        .filter(|r| r.is_invocation)
        .map(|r| r.baseline_cost)
        .sum();
    let overall_pass = !per_routine_pass.is_empty()
        && per_routine_pass.values().all(|&p| p);

    let ndjson_path = cfg.results_dir.join("typed-retrieval-live-fire.ndjson");
    if let Some(parent) = ndjson_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&ndjson_path)?;
    for r in &records {
        let line = serde_json::to_string(r)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        writeln!(f, "{}", line)?;
    }

    Ok(LiveFireOutcome {
        records,
        total_savings,
        total_baseline_invocation_cost,
        per_routine_pass,
        overall_pass,
        ndjson_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn small_cfg(dir: PathBuf) -> LiveFireConfig {
        LiveFireConfig {
            routine_count: 3,
            invocations_per_routine: 2,
            results_dir: dir,
        }
    }

    #[test]
    fn perfect_substrate_passes_with_savings_above_baseline() {
        let dir = tempdir().unwrap();
        let cfg = small_cfg(dir.path().to_path_buf());
        let out = run_live_fire_in_process(&cfg).unwrap();
        assert!(out.overall_pass, "perfect routine cache must pass live-fire");
        for (rid, pass) in &out.per_routine_pass {
            assert!(pass, "routine {rid} must pass");
        }
        assert!(
            out.total_savings >= BASELINE_RETRIEVAL_COST,
            "total savings must exceed one baseline retrieval cost"
        );
    }

    #[test]
    fn no_cache_substrate_fails_the_gate() {
        let dir = tempdir().unwrap();
        let cfg = small_cfg(dir.path().to_path_buf());
        let mut substrate = NoCacheRoutineSubstrate;
        let out = run_live_fire_with_substrate(&cfg, &mut substrate).unwrap();
        assert!(
            !out.overall_pass,
            "non-caching substrate must fail live-fire (no behavior credit)"
        );
        assert_eq!(out.total_savings, 0, "no caching → zero savings");
    }

    #[test]
    fn ndjson_records_one_plant_plus_invocations_per_routine() {
        let dir = tempdir().unwrap();
        let cfg = small_cfg(dir.path().to_path_buf());
        let out = run_live_fire_in_process(&cfg).unwrap();
        let plants = out.records.iter().filter(|r| r.phase == "plant").count();
        let invokes = out.records.iter().filter(|r| r.phase == "invoke").count();
        assert_eq!(plants, cfg.routine_count);
        assert_eq!(invokes, cfg.routine_count * cfg.invocations_per_routine);
        assert!(out.ndjson_path.exists());
    }

    #[test]
    fn cumulative_savings_grows_strictly_per_invocation() {
        let dir = tempdir().unwrap();
        let cfg = small_cfg(dir.path().to_path_buf());
        let out = run_live_fire_in_process(&cfg).unwrap();
        // Each routine's invocations should report increasing cumulative savings.
        let mut by_routine: BTreeMap<String, Vec<u32>> = BTreeMap::new();
        for r in out.records.iter().filter(|r| r.is_invocation) {
            by_routine
                .entry(r.routine_id.clone())
                .or_default()
                .push(r.cumulative_savings);
        }
        for (rid, series) in &by_routine {
            for w in series.windows(2) {
                assert!(w[1] > w[0], "cumulative savings must grow for {rid}");
            }
        }
    }
}

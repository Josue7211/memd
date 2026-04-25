//! Per-turn drift tick (Phase F4.5).
//!
//! A tiny rate limiter for the F4 drift detector. Each agent turn calls
//! [`record_turn`]; once every `n_turns` calls the outcome flags
//! `should_fire = true` and the caller (typically the PostToolUse hook
//! path) invokes the LLM-judge drift detector.
//!
//! State persists to `.memd/state/preference-drift-tick.json` so the
//! counter survives across CLI invocations.
//!
//! Feature flags (read by [`drift_tick_enabled`] and
//! [`n_turns_from_env`]):
//! - `MEMD_F4_PREF_DRIFT` (default off until F4.7 dogfood) — master gate.
//! - `MEMD_F4_DRIFT_N_TURNS` (default 10) — turn interval.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// On-disk shape of `preference-drift-tick.json`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DriftTickState {
    /// Total turns observed since file creation.
    #[serde(default)]
    pub counter: u32,
    /// Counter value at the most recent fire. `0` = never fired.
    #[serde(default)]
    pub last_fire_counter: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickOutcome {
    pub counter: u32,
    pub should_fire: bool,
}

pub fn drift_tick_state_path(memd_dir: &Path) -> PathBuf {
    memd_dir.join("state").join("preference-drift-tick.json")
}

pub fn read_tick_state(path: &Path) -> Result<DriftTickState> {
    match fs::read_to_string(path) {
        Ok(body) if body.trim().is_empty() => Ok(DriftTickState::default()),
        Ok(body) => Ok(serde_json::from_str(&body)?),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(DriftTickState::default()),
        Err(e) => Err(e.into()),
    }
}

pub fn write_tick_state(path: &Path, state: &DriftTickState) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(state)?)?;
    Ok(())
}

/// Increment the counter and report whether the next tick should fire.
///
/// `n_turns == 0` disables firing entirely (drift-detector off).
pub fn record_turn(path: &Path, n_turns: u32) -> Result<TickOutcome> {
    let mut state = read_tick_state(path)?;
    state.counter = state.counter.saturating_add(1);
    let should_fire = n_turns > 0 && state.counter % n_turns == 0;
    if should_fire {
        state.last_fire_counter = state.counter;
    }
    write_tick_state(path, &state)?;
    Ok(TickOutcome {
        counter: state.counter,
        should_fire,
    })
}

/// Read the master gate. Default off until F4.7 dogfood completes.
pub fn drift_tick_enabled() -> bool {
    matches!(
        std::env::var("MEMD_F4_PREF_DRIFT")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "on" | "yes"
    )
}

/// Read `MEMD_F4_DRIFT_N_TURNS`; default 10. Zero or unparseable → 10.
pub fn n_turns_from_env() -> u32 {
    std::env::var("MEMD_F4_DRIFT_N_TURNS")
        .ok()
        .and_then(|v| v.trim().parse::<u32>().ok())
        .filter(|n| *n > 0)
        .unwrap_or(10)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Test 12 — per-turn pipeline fires the drift detector every N turns.
    #[test]
    fn per_turn_pipeline_invokes_drift_every_n_turns() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("tick.json");
        let n = 3;
        let mut fires = 0;
        for i in 1..=10 {
            let outcome = record_turn(&path, n).unwrap();
            assert_eq!(outcome.counter, i, "counter monotonically increments");
            if outcome.should_fire {
                fires += 1;
            }
        }
        // Turns 3, 6, 9 → 3 fires across 10 turns.
        assert_eq!(fires, 3);

        let state = read_tick_state(&path).unwrap();
        assert_eq!(state.counter, 10);
        assert_eq!(state.last_fire_counter, 9);
    }

    #[test]
    fn record_turn_n_zero_never_fires() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("tick.json");
        for _ in 0..50 {
            assert!(!record_turn(&path, 0).unwrap().should_fire);
        }
    }

    #[test]
    fn n_turns_from_env_defaults_to_ten() {
        // Avoid asserting on the env var; the helper's default branch is
        // covered by feeding zero, which the helper rewrites to 10.
        unsafe {
            std::env::set_var("MEMD_F4_DRIFT_N_TURNS", "0");
        }
        assert_eq!(n_turns_from_env(), 10);
        unsafe {
            std::env::set_var("MEMD_F4_DRIFT_N_TURNS", "7");
        }
        assert_eq!(n_turns_from_env(), 7);
        unsafe {
            std::env::remove_var("MEMD_F4_DRIFT_N_TURNS");
        }
        assert_eq!(n_turns_from_env(), 10);
    }
}

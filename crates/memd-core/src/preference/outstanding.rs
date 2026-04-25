//! Outstanding preference-drift state (Phase F4.2).
//!
//! When the drift detector returns a `Drift` verdict the result is
//! persisted to `.memd/state/preference-drift-outstanding.json`. The
//! D4 wake compiler reads this file on every wake and prepends a
//! one-line note to the Preferences section. `memd preference confirm`
//! clears the entry.
//!
//! `Aligned` and `Unknown` verdicts also clear any outstanding entry
//! for the same preference id — the agent recovered, no surface needed.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::drift::{DriftCheck, DriftVerdict};

/// On-disk shape of `preference-drift-outstanding.json`. Maps
/// `preference_id` to the most-recent unacknowledged drift signal.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct OutstandingDriftState {
    #[serde(default)]
    pub entries: HashMap<String, OutstandingDriftEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutstandingDriftEntry {
    pub preference_id: String,
    pub verdict: DriftVerdict,
    pub confidence: f32,
    pub violation_count: u32,
    pub rationale: String,
    pub recorded_at_ms: i64,
    pub checked_turns: u32,
}

impl OutstandingDriftEntry {
    /// One-line drift note prepended to the Preferences wake section.
    /// Hard cap ≤80 chars per F4 contract.
    pub fn render_line(&self) -> String {
        let line = format!(
            "⚠ drift: {} ({} violation{} in last {} turns)",
            self.preference_id,
            self.violation_count,
            if self.violation_count == 1 { "" } else { "s" },
            self.checked_turns,
        );
        if line.chars().count() > 80 {
            line.chars().take(79).chain(std::iter::once('…')).collect()
        } else {
            line
        }
    }
}

/// State-file path for the bundle.
pub fn outstanding_state_path(memd_dir: &Path) -> PathBuf {
    memd_dir
        .join("state")
        .join("preference-drift-outstanding.json")
}

pub fn read_outstanding(path: &Path) -> Result<OutstandingDriftState> {
    match fs::read_to_string(path) {
        Ok(body) if body.trim().is_empty() => Ok(OutstandingDriftState::default()),
        Ok(body) => Ok(serde_json::from_str(&body)?),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(OutstandingDriftState::default()),
        Err(e) => Err(e.into()),
    }
}

pub fn write_outstanding(path: &Path, state: &OutstandingDriftState) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(state)?)?;
    Ok(())
}

/// Record a drift verdict. `Drift` upserts; `Aligned`/`Unknown` clear
/// any existing entry for the same id (agent recovered).
pub fn record_drift(
    path: &Path,
    check: &DriftCheck,
    now_ms: i64,
) -> Result<OutstandingDriftState> {
    let mut state = read_outstanding(path)?;
    match check.verdict {
        DriftVerdict::Drift => {
            state.entries.insert(
                check.preference_id.clone(),
                OutstandingDriftEntry {
                    preference_id: check.preference_id.clone(),
                    verdict: check.verdict,
                    confidence: check.confidence,
                    violation_count: check.violation_count,
                    rationale: check.rationale.clone(),
                    recorded_at_ms: now_ms,
                    checked_turns: check.checked_turns,
                },
            );
        }
        DriftVerdict::Aligned | DriftVerdict::Unknown => {
            state.entries.remove(&check.preference_id);
        }
    }
    write_outstanding(path, &state)?;
    Ok(state)
}

/// Clear outstanding state for one preference id. Used by `memd
/// preference confirm`.
pub fn clear_outstanding(path: &Path, preference_id: &str) -> Result<OutstandingDriftState> {
    let mut state = read_outstanding(path)?;
    state.entries.remove(preference_id);
    write_outstanding(path, &state)?;
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn drift_check_terse() -> DriftCheck {
        DriftCheck {
            preference_id: "pref-voice-terse".into(),
            verdict: DriftVerdict::Drift,
            confidence: 0.83,
            violation_count: 3,
            rationale: "agent went verbose".into(),
            cache_hit: false,
            cost_usd: 0.01,
            checked_turns: 10,
        }
    }

    /// Test 4 — drift outstanding state persists, round-trips, and clears
    /// on confirm or aligned re-check.
    #[test]
    fn drift_outstanding_state_persists_and_clears_on_confirm() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("state.json");
        let drift = drift_check_terse();

        let s1 = record_drift(&path, &drift, 1_700_000_000_000).unwrap();
        assert_eq!(s1.entries.len(), 1);
        assert!(s1.entries.contains_key("pref-voice-terse"));

        let s2 = read_outstanding(&path).unwrap();
        assert_eq!(s2, s1);
        assert_eq!(
            s2.entries["pref-voice-terse"].render_line(),
            "⚠ drift: pref-voice-terse (3 violations in last 10 turns)"
        );

        let s3 = clear_outstanding(&path, "pref-voice-terse").unwrap();
        assert!(s3.entries.is_empty());

        let _ = record_drift(&path, &drift, 1).unwrap();
        let aligned = DriftCheck {
            verdict: DriftVerdict::Aligned,
            violation_count: 0,
            ..drift.clone()
        };
        let s4 = record_drift(&path, &aligned, 2).unwrap();
        assert!(s4.entries.is_empty());
    }

    #[test]
    fn render_line_truncates_to_80_chars() {
        let entry = OutstandingDriftEntry {
            preference_id: "pref-".to_string() + &"x".repeat(120),
            verdict: DriftVerdict::Drift,
            confidence: 0.9,
            violation_count: 5,
            rationale: "y".into(),
            recorded_at_ms: 0,
            checked_turns: 12,
        };
        let line = entry.render_line();
        assert!(line.chars().count() <= 80, "len={}", line.chars().count());
        assert!(line.ends_with('…'));
    }
}

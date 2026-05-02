//! Phase F4 — `memd preference` CLI verbs.
//!
//! Local-first: no server required. `drift` records a verdict (manual or
//! injected) into `preference-drift.ndjson` + outstanding state. `confirm`
//! clears the outstanding entry, optionally promoting via C4. `promote`
//! always writes through the C4 `corrections.ndjson` path so the
//! correction lane sees preference re-pins as first-class corrections.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use memd_core::preference::drift::{DriftCheck, DriftVerdict};
use memd_core::preference::outstanding::{self, OutstandingDriftState, outstanding_state_path};
use memd_core::preference::tick::{
    self, drift_tick_enabled, drift_tick_state_path, n_turns_from_env,
};
use serde::{Deserialize, Serialize};

use super::args::{
    PreferenceConfirmArgs, PreferenceDriftArgs, PreferenceListArgs, PreferencePromoteArgs,
    PreferenceTickArgs,
};
use super::{CorrectionCaptureArgs, run_correction_capture};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PreferenceDriftLogRow {
    pub ts_ms: i64,
    pub session_id: Option<String>,
    pub preference_id: String,
    pub checked_turns: u32,
    pub violation_count: u32,
    pub judge_verdict: String,
    pub judge_confidence: f32,
    pub rationale: Option<String>,
    pub surfaced: bool,
    pub source: String,
}

pub(crate) fn run_preference_list(args: &PreferenceListArgs) -> Result<()> {
    let path = outstanding_state_path(&args.output);
    let state = outstanding::read_outstanding(&path)?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&state)?);
    } else if state.entries.is_empty() {
        println!("no outstanding preference drift");
    } else {
        for entry in state.entries.values() {
            println!("{}", entry.render_line());
        }
    }
    Ok(())
}

pub(crate) fn run_preference_drift(args: &PreferenceDriftArgs) -> Result<()> {
    let verdict = parse_verdict_arg(args)?;

    if !args.no_judge && args.verdict.is_none() {
        return Err(anyhow!(
            "live judge invocation not yet wired into CLI; pass --verdict (or --no-judge) for now",
        ));
    }

    let confidence = args.confidence.unwrap_or(match verdict {
        DriftVerdict::Drift => 0.85,
        DriftVerdict::Aligned => 0.9,
        DriftVerdict::Unknown => 0.5,
    });
    let violation_count = args.violation_count.unwrap_or(0);
    let checked_turns = args.checked_turns.unwrap_or_else(|| {
        args.turns_json
            .as_deref()
            .and_then(|raw| serde_json::from_str::<Vec<String>>(raw).ok())
            .map(|v| v.len() as u32)
            .unwrap_or(0)
    });
    let rationale = args
        .rationale
        .clone()
        .unwrap_or_else(|| "manual override".to_string());

    let check = DriftCheck {
        preference_id: args.preference_id.clone(),
        verdict,
        confidence,
        violation_count,
        rationale: rationale.clone(),
        cache_hit: false,
        cost_usd: 0.0,
        checked_turns,
    };
    let now_ms = Utc::now().timestamp_millis();
    let state = outstanding::record_drift(&outstanding_state_path(&args.output), &check, now_ms)?;

    let surfaced = matches!(verdict, DriftVerdict::Drift);
    append_drift_log(
        &args.output,
        &PreferenceDriftLogRow {
            ts_ms: now_ms,
            session_id: args.session_id.clone(),
            preference_id: args.preference_id.clone(),
            checked_turns,
            violation_count,
            judge_verdict: verdict_str(verdict).to_string(),
            judge_confidence: confidence,
            rationale: Some(rationale),
            surfaced,
            source: if args.verdict.is_some() {
                "manual".into()
            } else {
                "judge".into()
            },
        },
    )?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&state)?);
    } else {
        println!(
            "preference={} verdict={} confidence={:.2} violations={} surfaced={}",
            args.preference_id,
            verdict_str(verdict),
            confidence,
            violation_count,
            surfaced,
        );
    }
    Ok(())
}

pub(crate) fn run_preference_confirm(args: &PreferenceConfirmArgs) -> Result<()> {
    let cleared =
        outstanding::clear_outstanding(&outstanding_state_path(&args.output), &args.preference_id)?;
    if args.promote {
        let content = args
            .preference_content
            .clone()
            .ok_or_else(|| anyhow!("--promote requires --preference-content"))?;
        run_preference_promote(&PreferencePromoteArgs {
            preference_id: args.preference_id.clone(),
            preference_content: content,
            confidence: args.confidence,
            session_id: None,
            output: args.output.clone(),
        })?;
    }
    println!(
        "confirmed preference={} outstanding_remaining={}",
        args.preference_id,
        cleared.entries.len()
    );
    Ok(())
}

pub(crate) fn run_preference_tick(args: &PreferenceTickArgs) -> Result<()> {
    let enabled = args.force_enabled || drift_tick_enabled();
    if !enabled {
        if args.json {
            println!(
                r#"{{"enabled":false,"counter":null,"fire":false,"reason":"MEMD_F4_PREF_DRIFT off"}}"#
            );
        } else {
            println!("drift-tick disabled (MEMD_F4_PREF_DRIFT off)");
        }
        return Ok(());
    }
    let n = args.n_turns.unwrap_or_else(n_turns_from_env);
    let path = drift_tick_state_path(&args.output);
    let outcome = tick::record_turn(&path, n)?;

    // F4.7 driver — emit a tick-fire row to preference-drift.ndjson on
    // every Nth turn so the dogfood log accumulates real per-turn
    // telemetry. Verdict is `tick_fire` (a non-judge marker) since this
    // is the lightweight cadence signal; full LLM-judge invocation is a
    // separate verb (`memd preference drift`) and is not auto-fired here
    // to keep tokens at zero in the dogfood path.
    if outcome.should_fire {
        append_drift_log(
            &args.output,
            &PreferenceDriftLogRow {
                ts_ms: Utc::now().timestamp_millis(),
                session_id: args.session_id.clone(),
                preference_id: "tick-fire".into(),
                checked_turns: n,
                violation_count: 0,
                judge_verdict: "tick_fire".into(),
                judge_confidence: 0.0,
                rationale: Some(format!(
                    "per-turn driver fired at counter={}",
                    outcome.counter
                )),
                surfaced: false,
                source: "preference-tick".into(),
            },
        )?;
    }

    if args.json {
        println!(
            "{}",
            serde_json::json!({
                "enabled": true,
                "counter": outcome.counter,
                "n_turns": n,
                "fire": outcome.should_fire,
            })
        );
    } else {
        println!(
            "drift-tick counter={} n={} fire={}",
            outcome.counter, n, outcome.should_fire
        );
    }
    Ok(())
}

pub(crate) fn run_preference_promote(args: &PreferencePromoteArgs) -> Result<()> {
    if args.confidence < 0.0 || args.confidence > 1.0 {
        return Err(anyhow!(
            "--confidence must be in [0.0, 1.0]; got {}",
            args.confidence
        ));
    }
    run_correction_capture(&CorrectionCaptureArgs {
        content: args.preference_content.clone(),
        corrects_id: Some(args.preference_id.clone()),
        source_turn: None,
        confidence: args.confidence,
        captured_by: "preference-promote".into(),
        session_id: args.session_id.clone(),
        output: args.output.clone(),
    })
    .with_context(|| "promote -> correction capture")?;

    println!(
        "promoted preference={} confidence={:.2}",
        args.preference_id, args.confidence
    );
    Ok(())
}

fn parse_verdict_arg(args: &PreferenceDriftArgs) -> Result<DriftVerdict> {
    let raw = args
        .verdict
        .as_deref()
        .map(|s| s.trim().to_ascii_lowercase());
    match raw.as_deref() {
        Some("drift") => Ok(DriftVerdict::Drift),
        Some("aligned") | Some("ok") => Ok(DriftVerdict::Aligned),
        Some("unknown") => Ok(DriftVerdict::Unknown),
        Some(other) => Err(anyhow!(
            "--verdict must be drift|aligned|unknown; got `{other}`"
        )),
        None => Ok(DriftVerdict::Unknown),
    }
}

fn verdict_str(v: DriftVerdict) -> &'static str {
    match v {
        DriftVerdict::Drift => "drift",
        DriftVerdict::Aligned => "aligned",
        DriftVerdict::Unknown => "unknown",
    }
}

pub(crate) fn preference_drift_log_path(output: &Path) -> std::path::PathBuf {
    output.join("logs").join("preference-drift.ndjson")
}

fn append_drift_log(output: &Path, row: &PreferenceDriftLogRow) -> Result<()> {
    let path = preference_drift_log_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    writeln!(file, "{}", serde_json::to_string(row)?)?;
    Ok(())
}

#[allow(dead_code)]
pub(crate) fn read_outstanding_state(output: &Path) -> Result<OutstandingDriftState> {
    outstanding::read_outstanding(&outstanding_state_path(output))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn cli_preference_drift_force_check() {
        let tmp = TempDir::new().unwrap();
        let args = PreferenceDriftArgs {
            preference_id: "pref-voice-terse".into(),
            preference_content: Some("voice=terse".into()),
            turns_json: Some(r#"["short ok","let me explain at length, you see..."]"#.into()),
            verdict: Some("drift".into()),
            confidence: Some(0.85),
            violation_count: Some(3),
            rationale: Some("agent went verbose".into()),
            checked_turns: Some(10),
            session_id: Some("s-1".into()),
            no_judge: true,
            output: tmp.path().to_path_buf(),
            json: false,
        };
        run_preference_drift(&args).unwrap();

        let state = read_outstanding_state(tmp.path()).unwrap();
        assert_eq!(state.entries.len(), 1);
        let entry = &state.entries["pref-voice-terse"];
        assert_eq!(entry.violation_count, 3);
        assert_eq!(entry.checked_turns, 10);

        let log_body = fs::read_to_string(preference_drift_log_path(tmp.path())).unwrap();
        assert!(log_body.contains("\"judge_verdict\":\"drift\""));
        assert!(log_body.contains("\"surfaced\":true"));
        assert!(log_body.contains("pref-voice-terse"));
    }

    #[test]
    fn cli_preference_confirm_clears_outstanding() {
        let tmp = TempDir::new().unwrap();
        run_preference_drift(&PreferenceDriftArgs {
            preference_id: "pref-x".into(),
            preference_content: None,
            turns_json: None,
            verdict: Some("drift".into()),
            confidence: Some(0.9),
            violation_count: Some(2),
            rationale: Some("r".into()),
            checked_turns: Some(8),
            session_id: None,
            no_judge: true,
            output: tmp.path().to_path_buf(),
            json: false,
        })
        .unwrap();
        assert_eq!(read_outstanding_state(tmp.path()).unwrap().entries.len(), 1);

        run_preference_confirm(&PreferenceConfirmArgs {
            preference_id: "pref-x".into(),
            promote: false,
            preference_content: None,
            confidence: 0.95,
            output: tmp.path().to_path_buf(),
        })
        .unwrap();

        assert!(
            read_outstanding_state(tmp.path())
                .unwrap()
                .entries
                .is_empty()
        );
    }

    #[test]
    fn cli_preference_promote_writes_correction_record_via_c4_path() {
        let tmp = TempDir::new().unwrap();
        run_preference_promote(&PreferencePromoteArgs {
            preference_id: "pref-voice-terse".into(),
            preference_content: "voice=terse".into(),
            confidence: 0.95,
            session_id: Some("s-2".into()),
            output: tmp.path().to_path_buf(),
        })
        .unwrap();

        let path = tmp.path().join("logs").join("corrections.ndjson");
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains("\"action\":\"capture\""));
        assert!(body.contains("\"corrects_id\":\"pref-voice-terse\""));
        assert!(body.contains("\"captured_by\":\"preference-promote\""));
    }

    #[test]
    fn cli_preference_drift_rejects_live_judge_until_wired() {
        let tmp = TempDir::new().unwrap();
        let args = PreferenceDriftArgs {
            preference_id: "p".into(),
            preference_content: Some("c".into()),
            turns_json: Some(r#"["t"]"#.into()),
            verdict: None,
            confidence: None,
            violation_count: None,
            rationale: None,
            checked_turns: None,
            session_id: None,
            no_judge: false,
            output: tmp.path().to_path_buf(),
            json: false,
        };
        assert!(run_preference_drift(&args).is_err());
    }

    #[test]
    fn cli_preference_tick_fires_every_n_turns_when_forced() {
        let tmp = TempDir::new().unwrap();
        let mut fires = 0;
        for _ in 0..6 {
            run_preference_tick(&PreferenceTickArgs {
                n_turns: Some(2),
                force_enabled: true,
                session_id: None,
                output: tmp.path().to_path_buf(),
                json: false,
            })
            .unwrap();
        }
        let state = tick::read_tick_state(&drift_tick_state_path(tmp.path())).unwrap();
        assert_eq!(state.counter, 6);
        // Turns 2, 4, 6 → 3 fires; last_fire_counter = 6.
        assert_eq!(state.last_fire_counter, 6);
        // Sanity: count fires manually too.
        for _ in 0..4 {
            let outcome = tick::record_turn(&drift_tick_state_path(tmp.path()), 3).unwrap();
            if outcome.should_fire {
                fires += 1;
            }
        }
        assert!(fires >= 1, "fires accumulated across n=3 sequence");
    }

    #[test]
    fn cli_preference_tick_no_op_when_gate_off() {
        let tmp = TempDir::new().unwrap();
        // Force disabled by leaving force_enabled = false; env may be unset.
        unsafe {
            std::env::remove_var("MEMD_F4_PREF_DRIFT");
        }
        run_preference_tick(&PreferenceTickArgs {
            n_turns: Some(1),
            force_enabled: false,
            session_id: None,
            output: tmp.path().to_path_buf(),
            json: false,
        })
        .unwrap();
        // No state file written.
        assert!(!drift_tick_state_path(tmp.path()).exists());
    }

    /// F4.7 driver: tick that fires emits a `tick_fire` row to
    /// `preference-drift.ndjson` so dogfood NDJSON harvest is feasible.
    #[test]
    fn cli_preference_tick_emits_ndjson_row_when_fires() {
        let tmp = TempDir::new().unwrap();
        for _ in 0..3 {
            run_preference_tick(&PreferenceTickArgs {
                n_turns: Some(3),
                force_enabled: true,
                session_id: Some("sess-f47".into()),
                output: tmp.path().to_path_buf(),
                json: false,
            })
            .unwrap();
        }
        // Three ticks at n=3 → fires once at counter=3 → exactly one
        // row in preference-drift.ndjson.
        let log = preference_drift_log_path(tmp.path());
        let body = fs::read_to_string(&log).expect("ndjson exists after fire");
        let lines: Vec<&str> = body.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(lines.len(), 1, "exactly one tick_fire row");
        let parsed: PreferenceDriftLogRow = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(parsed.judge_verdict, "tick_fire");
        assert_eq!(parsed.preference_id, "tick-fire");
        assert_eq!(parsed.checked_turns, 3);
        assert_eq!(parsed.session_id.as_deref(), Some("sess-f47"));
        assert_eq!(parsed.source, "preference-tick");
        assert!(!parsed.surfaced);
    }

    /// F4.7 driver: ticks that do NOT fire must not append to the
    /// drift log — log size stays zero across pre-fire turns.
    #[test]
    fn cli_preference_tick_no_ndjson_row_when_pre_fire() {
        let tmp = TempDir::new().unwrap();
        // Two ticks at n=5 → no fire yet (counter=2 < 5).
        for _ in 0..2 {
            run_preference_tick(&PreferenceTickArgs {
                n_turns: Some(5),
                force_enabled: true,
                session_id: None,
                output: tmp.path().to_path_buf(),
                json: false,
            })
            .unwrap();
        }
        let log = preference_drift_log_path(tmp.path());
        assert!(
            !log.exists(),
            "no NDJSON row when tick has not fired yet (counter < n)"
        );
    }

    #[test]
    fn cli_preference_drift_aligned_clears_existing() {
        let tmp = TempDir::new().unwrap();
        // seed
        run_preference_drift(&PreferenceDriftArgs {
            preference_id: "pref-a".into(),
            preference_content: None,
            turns_json: None,
            verdict: Some("drift".into()),
            confidence: Some(0.9),
            violation_count: Some(1),
            rationale: None,
            checked_turns: Some(5),
            session_id: None,
            no_judge: true,
            output: tmp.path().to_path_buf(),
            json: false,
        })
        .unwrap();
        // recovery — aligned removes outstanding entry per F4.2 contract.
        run_preference_drift(&PreferenceDriftArgs {
            preference_id: "pref-a".into(),
            preference_content: None,
            turns_json: None,
            verdict: Some("aligned".into()),
            confidence: Some(0.92),
            violation_count: Some(0),
            rationale: None,
            checked_turns: Some(5),
            session_id: None,
            no_judge: true,
            output: tmp.path().to_path_buf(),
            json: false,
        })
        .unwrap();
        assert!(
            read_outstanding_state(tmp.path())
                .unwrap()
                .entries
                .is_empty()
        );
    }
}

//! G4.3 — cross-V4 assertions module.
//!
//! Six pure-function asserters covering A4, B4, C4, D4, E4, F4. Each asserter
//! takes a typed view of the relevant V4 phase output and returns `Ok(())` on
//! a healthy run or `Err(reason)` when a regression is observed. Tests 3–8 of
//! `phase-g4-plan.md §4` pair each asserter with its `inject-faults/*.json`
//! mutation: healthy passes, faulted fails with the expected error class.
//!
//! Asserters are intentionally decoupled from the live harness — they consume
//! synthetic input structs. G4.5 wires the real harness output (`.memd/logs`,
//! `.memd/state`) into the same asserter signatures.

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub(crate) struct WakeBriefView {
    pub token_count: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct HookTraceView {
    pub ordered_events: Vec<String>,
    pub post_tool_use_count: usize,
    pub ends_with_seal: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct CorrectionRecord {
    pub id: String,
    pub current_value: String,
    pub provenance: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct LookupResult {
    pub query: String,
    pub returned_value: String,
}

#[derive(Debug, Clone)]
pub(crate) struct PreferenceDriftState {
    pub outstanding_count: usize,
}

pub(crate) fn assert_a4_postcompact_restore_ran(restored_before_first_tool: bool) -> Result<(), String> {
    if restored_before_first_tool {
        Ok(())
    } else {
        Err("A4 regression: PostCompact restore did not run before first session-2 tool call".into())
    }
}

pub(crate) fn assert_b4_hook_trace(
    trace: &HookTraceView,
    expected_events: &[&str],
    min_post_tool_use_count: usize,
) -> Result<(), String> {
    let actual: Vec<&str> = trace.ordered_events.iter().map(String::as_str).collect();
    for needle in expected_events {
        if !actual.contains(needle) {
            return Err(format!(
                "B4 regression: hook trace missing event `{needle}` (saw {actual:?})"
            ));
        }
    }
    if trace.post_tool_use_count < min_post_tool_use_count {
        return Err(format!(
            "B4 regression: PostToolUse count {} below minimum {}",
            trace.post_tool_use_count, min_post_tool_use_count
        ));
    }
    if !trace.ends_with_seal {
        return Err("B4 regression: hook trace does not end with PreCompact seal".into());
    }
    Ok(())
}

pub(crate) fn assert_c4_correction_provenance(
    corrections: &[CorrectionRecord],
) -> Result<(), String> {
    for c in corrections {
        if c.provenance.is_none() {
            return Err(format!(
                "C4 regression: correction `{}` (current_value={}) is missing provenance",
                c.id, c.current_value
            ));
        }
    }
    Ok(())
}

pub(crate) fn assert_d4_wake_within_budget(
    wake: &WakeBriefView,
    max_tokens: usize,
) -> Result<(), String> {
    if wake.token_count > max_tokens {
        return Err(format!(
            "D4 regression: wake brief {} tokens exceeds budget {}",
            wake.token_count, max_tokens
        ));
    }
    Ok(())
}

pub(crate) fn assert_e4_lookup_returns_corrected(
    lookup: &LookupResult,
    must_contain: &str,
    must_not_contain: &str,
) -> Result<(), String> {
    if !lookup.returned_value.contains(must_contain) {
        return Err(format!(
            "E4 regression: lookup `{}` returned `{}`, missing required value `{}`",
            lookup.query, lookup.returned_value, must_contain
        ));
    }
    if lookup.returned_value.contains(must_not_contain) {
        return Err(format!(
            "E4 regression: lookup `{}` returned stale value containing `{}`",
            lookup.query, must_not_contain
        ));
    }
    Ok(())
}

pub(crate) fn assert_f4_drift_detected(
    drift: &PreferenceDriftState,
    min_outstanding: usize,
) -> Result<(), String> {
    if drift.outstanding_count < min_outstanding {
        return Err(format!(
            "F4 regression: outstanding drift count {} below expected minimum {}",
            drift.outstanding_count, min_outstanding
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn fault_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/g4/inject-faults")
            .join(name)
    }

    fn load_fault(name: &str) -> Value {
        let text = std::fs::read_to_string(fault_path(name))
            .unwrap_or_else(|err| panic!("read fault {name}: {err}"));
        serde_json::from_str(&text).unwrap_or_else(|err| panic!("parse fault {name}: {err}"))
    }

    fn mutation(fault: &Value) -> &Value {
        fault.get("mutation").expect("fault file has mutation block")
    }

    /// Test 3 — A4 PostCompact restore silently no-ops.
    #[test]
    fn t3_a4_skip_postcompact_restore_fires() {
        assert_a4_postcompact_restore_ran(true).expect("healthy run passes");

        let fault = load_fault("a4-skip-postcompact-restore.json");
        let force_restored = mutation(&fault)["force_restored"].as_bool().unwrap();
        let err = assert_a4_postcompact_restore_ran(force_restored)
            .expect_err("fault must be detected");
        assert!(err.contains("A4 regression"));
        assert!(err.contains("PostCompact restore did not run"));
    }

    /// Test 4 — B4 hook trace omits a PostToolUse line.
    #[test]
    fn t4_b4_silent_hook_swallow_fires() {
        let healthy = HookTraceView {
            ordered_events: vec!["SessionStart".into(), "PostToolUse".into(), "PreCompact".into()],
            post_tool_use_count: 3,
            ends_with_seal: true,
        };
        assert_b4_hook_trace(&healthy, &["SessionStart", "PostToolUse", "PreCompact"], 3)
            .expect("healthy hook trace passes");

        let fault = load_fault("b4-silent-hook-swallow.json");
        let drop = mutation(&fault)["drop_post_tool_use_count"]
            .as_u64()
            .unwrap() as usize;
        let mut faulted = healthy.clone();
        faulted.post_tool_use_count = faulted.post_tool_use_count.saturating_sub(drop) - 2; // force below floor

        let err = assert_b4_hook_trace(
            &faulted,
            &["SessionStart", "PostToolUse", "PreCompact"],
            3,
        )
        .expect_err("dropped PostToolUse must be detected");
        assert!(err.contains("B4 regression"));
        assert!(err.contains("PostToolUse count"));
    }

    /// Test 5 — C4 correction stored without provenance.
    #[test]
    fn t5_c4_correction_missing_provenance_fires() {
        let healthy = vec![
            CorrectionRecord {
                id: "fact-A".into(),
                current_value: "ulid".into(),
                provenance: Some("S1-T3".into()),
            },
            CorrectionRecord {
                id: "fact-B".into(),
                current_value: "2026-05-15".into(),
                provenance: Some("S2-T18".into()),
            },
        ];
        assert_c4_correction_provenance(&healthy).expect("healthy corrections pass");

        let fault = load_fault("c4-correction-missing-provenance.json");
        let strip_id = mutation(&fault)["strip_provenance_for_id"]
            .as_str()
            .unwrap();
        let faulted: Vec<CorrectionRecord> = healthy
            .into_iter()
            .map(|mut c| {
                if c.id == strip_id {
                    c.provenance = None;
                }
                c
            })
            .collect();
        let err =
            assert_c4_correction_provenance(&faulted).expect_err("missing provenance detected");
        assert!(err.contains("C4 regression"));
        assert!(err.contains(strip_id));
    }

    /// Test 6 — D4 wake brief exceeds budget.
    #[test]
    fn t6_d4_wake_exceeds_budget_fires() {
        let healthy = WakeBriefView { token_count: 1850 };
        assert_d4_wake_within_budget(&healthy, 2000).expect("under-budget passes");

        let fault = load_fault("d4-wake-exceeds-budget.json");
        let over = mutation(&fault)["override_token_count"].as_u64().unwrap() as usize;
        let faulted = WakeBriefView { token_count: over };
        let err =
            assert_d4_wake_within_budget(&faulted, 2000).expect_err("over-budget must be detected");
        assert!(err.contains("D4 regression"));
        assert!(err.contains("exceeds budget 2000"));
    }

    /// Test 7 — E4 lookup returns stale (uuid) instead of corrected (ulid).
    #[test]
    fn t7_e4_lookup_stale_fires() {
        let healthy = LookupResult {
            query: "primary ID".into(),
            returned_value: "ulid".into(),
        };
        assert_e4_lookup_returns_corrected(&healthy, "ulid", "uuid").expect("healthy lookup passes");

        let fault = load_fault("e4-lookup-stale.json");
        let stale = mutation(&fault)["override_returned_value"]
            .as_str()
            .unwrap()
            .to_string();
        let faulted = LookupResult {
            query: "primary ID".into(),
            returned_value: stale,
        };
        let err = assert_e4_lookup_returns_corrected(&faulted, "ulid", "uuid")
            .expect_err("stale lookup must be detected");
        assert!(err.contains("E4 regression"));
    }

    /// Test 8 — F4 drift detector silently skips verbose-drift turn.
    #[test]
    fn t8_f4_drift_undetected_fires() {
        let healthy = PreferenceDriftState { outstanding_count: 1 };
        assert_f4_drift_detected(&healthy, 1).expect("healthy drift state passes");

        let fault = load_fault("f4-drift-undetected.json");
        let zero = mutation(&fault)["override_outstanding_count"]
            .as_u64()
            .unwrap() as usize;
        let faulted = PreferenceDriftState {
            outstanding_count: zero,
        };
        let err =
            assert_f4_drift_detected(&faulted, 1).expect_err("missed drift must be detected");
        assert!(err.contains("F4 regression"));
        assert!(err.contains("outstanding drift count 0"));
    }
}

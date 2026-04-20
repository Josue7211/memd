//! Working-memory lifecycle self-test probe (A3-D3).
//!
//! Pure data model. Runtime lives in `memd-client` (`cli_lifecycle_probe_runtime`)
//! because it needs `MemdClient` and the server HTTP surface.

use serde::{Deserialize, Serialize};

/// Final probe verdict. `status` is `"green"` when every step reports `ok`,
/// otherwise `"red"`. The string form is deliberately stable for hooks and
/// downstream validators (Part 2 cross-harness validator consumes this).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LifecycleProbeReport {
    pub status: String,
    pub probe_id: String,
    pub steps: Vec<LifecycleProbeStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LifecycleProbeStep {
    pub name: String,
    pub ok: bool,
    pub detail: Option<String>,
}

impl LifecycleProbeReport {
    pub fn from_steps(probe_id: impl Into<String>, steps: Vec<LifecycleProbeStep>) -> Self {
        let all_ok = steps.iter().all(|s| s.ok);
        let status = if all_ok { "green" } else { "red" }.to_string();
        LifecycleProbeReport {
            status,
            probe_id: probe_id.into(),
            steps,
        }
    }

    pub fn is_green(&self) -> bool {
        self.status == "green"
    }
}

impl LifecycleProbeStep {
    pub fn ok(name: impl Into<String>) -> Self {
        LifecycleProbeStep {
            name: name.into(),
            ok: true,
            detail: None,
        }
    }

    pub fn fail(name: impl Into<String>, detail: impl Into<String>) -> Self {
        LifecycleProbeStep {
            name: name.into(),
            ok: false,
            detail: Some(detail.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_ok_steps_yield_green_report() {
        let report = LifecycleProbeReport::from_steps(
            "probe-1",
            vec![
                LifecycleProbeStep::ok("store"),
                LifecycleProbeStep::ok("recall"),
                LifecycleProbeStep::ok("expire"),
                LifecycleProbeStep::ok("verify_expired"),
            ],
        );
        assert_eq!(report.status, "green");
        assert!(report.is_green());
    }

    #[test]
    fn any_failing_step_yields_red_report() {
        let report = LifecycleProbeReport::from_steps(
            "probe-2",
            vec![
                LifecycleProbeStep::ok("store"),
                LifecycleProbeStep::fail("recall", "no match"),
            ],
        );
        assert_eq!(report.status, "red");
        assert!(!report.is_green());
    }

    #[test]
    fn report_round_trips_through_json() {
        let report =
            LifecycleProbeReport::from_steps("probe-3", vec![LifecycleProbeStep::ok("store")]);
        let json = serde_json::to_string(&report).unwrap();
        let parsed: LifecycleProbeReport = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, report);
    }
}

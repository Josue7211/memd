use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnforcementPolicy {
    Off,
    #[default]
    Warn,
    Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateDecision {
    Allow,
    Warn { path: String, reason: String },
    Deny { path: String, reason: String },
}

pub fn gate_decision(
    policy: EnforcementPolicy,
    target_path: &str,
    sealed_paths: &[String],
    fresh_read_paths: &[String],
) -> GateDecision {
    if matches!(policy, EnforcementPolicy::Off) {
        return GateDecision::Allow;
    }
    let in_sealed = sealed_paths.iter().any(|p| p == target_path);
    if !in_sealed {
        return GateDecision::Allow;
    }
    let is_fresh = fresh_read_paths.iter().any(|p| p == target_path);
    if is_fresh {
        return GateDecision::Allow;
    }
    let reason = format!(
        "continuity: {target_path} was touched in a prior session (sealed ledger) but has not been Read in THIS session. Read it before editing."
    );
    match policy {
        EnforcementPolicy::Warn => GateDecision::Warn {
            path: target_path.into(),
            reason,
        },
        EnforcementPolicy::Block => GateDecision::Deny {
            path: target_path.into(),
            reason,
        },
        EnforcementPolicy::Off => unreachable!(),
    }
}

/// Render a GateDecision as the JSON string the PreToolUse hook should print,
/// or `None` when no output is needed (Allow). Extracted so the CLI arm and
/// tests share the same formatter.
pub fn format_gate_output(decision: GateDecision) -> Option<String> {
    match decision {
        GateDecision::Allow => None,
        GateDecision::Warn { reason, .. } => Some(
            serde_json::json!({ "systemMessage": reason }).to_string(),
        ),
        GateDecision::Deny { reason, .. } => Some(
            serde_json::json!({
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "deny",
                    "permissionDecisionReason": reason
                }
            })
            .to_string(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_round_trips_through_serde() {
        for p in [
            EnforcementPolicy::Off,
            EnforcementPolicy::Warn,
            EnforcementPolicy::Block,
        ] {
            let s = serde_json::to_string(&p).unwrap();
            let r: EnforcementPolicy = serde_json::from_str(&s).unwrap();
            assert_eq!(p, r);
        }
    }

    #[test]
    fn gate_decision_passes_when_path_not_in_sealed_set() {
        let sealed: &[String] = &["a.rs".into()];
        let fresh: &[String] = &[];
        assert_eq!(
            gate_decision(EnforcementPolicy::Block, "other.rs", sealed, fresh),
            GateDecision::Allow
        );
    }

    #[test]
    fn gate_decision_denies_when_block_and_sealed_path_not_fresh() {
        let sealed: &[String] = &["a.rs".into()];
        let fresh: &[String] = &[];
        assert!(matches!(
            gate_decision(EnforcementPolicy::Block, "a.rs", sealed, fresh),
            GateDecision::Deny { .. }
        ));
    }

    #[test]
    fn gate_decision_warns_when_warn_and_sealed_path_not_fresh() {
        let sealed: &[String] = &["a.rs".into()];
        let fresh: &[String] = &[];
        assert!(matches!(
            gate_decision(EnforcementPolicy::Warn, "a.rs", sealed, fresh),
            GateDecision::Warn { .. }
        ));
    }

    #[test]
    fn gate_decision_allows_when_sealed_path_is_fresh_read() {
        let sealed: &[String] = &["a.rs".into()];
        let fresh: &[String] = &["a.rs".into()];
        assert_eq!(
            gate_decision(EnforcementPolicy::Block, "a.rs", sealed, fresh),
            GateDecision::Allow
        );
    }

    #[test]
    fn gate_decision_allows_when_policy_off() {
        let sealed: &[String] = &["a.rs".into()];
        let fresh: &[String] = &[];
        assert_eq!(
            gate_decision(EnforcementPolicy::Off, "a.rs", sealed, fresh),
            GateDecision::Allow
        );
    }
}

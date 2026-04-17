//! Live memory contract (A3-D5).
//!
//! Part 1 ships a **minimal** contract with one typed guarantee so the shape
//! is stable (`{version, guarantees: {...}}`) and downstream validators can
//! start consuming it. Part 2 expands `ContractGuarantees` as more surfacing
//! guarantees graduate from implementation detail to published contract.

use serde::{Deserialize, Serialize};

pub const CONTRACT_FILE_NAME: &str = "contract.json";
pub const CURRENT_VERSION: &str = "0.2.0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemdContract {
    pub version: String,
    pub guarantees: ContractGuarantees,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractGuarantees {
    /// A3-D1 surfacing half: whenever a prior session sealed a
    /// file-interaction ledger under the bundle, the wake packet must be
    /// able to list those paths via `collect_files_touched`.
    pub surfaces_files_touched_when_sealed_ledger_exists: bool,
    pub seals_session_ledger_on_precompact: bool,
    pub enforces_continuity_gate_when_configured: bool,
    pub replays_preferences_on_cold_boot: bool,
}

impl Default for MemdContract {
    fn default() -> Self {
        MemdContract {
            version: CURRENT_VERSION.to_string(),
            guarantees: ContractGuarantees {
                surfaces_files_touched_when_sealed_ledger_exists: true,
                seals_session_ledger_on_precompact: true,
                enforces_continuity_gate_when_configured: true,
                replays_preferences_on_cold_boot: true,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractViolation {
    pub guarantee: String,
    pub detail: String,
}

/// Inputs captured from the bundle at verify time. Keeping this struct flat
/// makes the verifier a pure function (easy to unit-test without I/O).
#[derive(Debug, Clone)]
pub struct ContractEvidence<'a> {
    pub sealed_ledger_exists: bool,
    pub files_touched: &'a [String],
    pub live_ledger_exists: bool,
    pub sealed_dir_empty: bool,
    pub enforcement_policy_configured: bool,
    pub enforcement_hook_wired: bool,
    /// Tri-state: Some(true)=green, Some(false)=red, None=not exercised.
    pub preference_recall_on_cold_boot_green: Option<bool>,
}

pub fn verify_contract(
    contract: &MemdContract,
    evidence: &ContractEvidence<'_>,
) -> Vec<ContractViolation> {
    let mut violations = Vec::new();
    let g = &contract.guarantees;

    if g.surfaces_files_touched_when_sealed_ledger_exists
        && evidence.sealed_ledger_exists
        && evidence.files_touched.is_empty()
    {
        violations.push(ContractViolation {
            guarantee: "surfaces_files_touched_when_sealed_ledger_exists".into(),
            detail: "sealed ledger exists but files_touched is empty".into(),
        });
    }
    if g.seals_session_ledger_on_precompact
        && evidence.live_ledger_exists
        && evidence.sealed_dir_empty
    {
        violations.push(ContractViolation {
            guarantee: "seals_session_ledger_on_precompact".into(),
            detail: "live ledger exists but no sealed copy present".into(),
        });
    }
    if g.enforces_continuity_gate_when_configured
        && evidence.enforcement_policy_configured
        && !evidence.enforcement_hook_wired
    {
        violations.push(ContractViolation {
            guarantee: "enforces_continuity_gate_when_configured".into(),
            detail: "continuity.enforcement is configured but PreToolUse gate hook is not wired".into(),
        });
    }
    if g.replays_preferences_on_cold_boot
        && matches!(evidence.preference_recall_on_cold_boot_green, Some(false))
    {
        violations.push(ContractViolation {
            guarantee: "replays_preferences_on_cold_boot".into(),
            detail: "preferences stored via `memd remember --kind preference` did not surface in cold-boot wake".into(),
        });
    }
    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_contract_round_trips_through_json() {
        let contract = MemdContract::default();
        let json = serde_json::to_string_pretty(&contract).unwrap();
        let parsed: MemdContract = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, contract);
        assert_eq!(parsed.version, CURRENT_VERSION);
        assert!(
            parsed
                .guarantees
                .surfaces_files_touched_when_sealed_ledger_exists
        );
    }

    #[test]
    fn verify_clean_when_no_sealed_ledger() {
        let contract = MemdContract::default();
        let evidence = ContractEvidence {
            sealed_ledger_exists: false,
            files_touched: &[],
            live_ledger_exists: false,
            sealed_dir_empty: false,
            enforcement_policy_configured: false,
            enforcement_hook_wired: false,
            preference_recall_on_cold_boot_green: None,
        };
        assert!(verify_contract(&contract, &evidence).is_empty());
    }

    #[test]
    fn verify_clean_when_sealed_ledger_with_surfaced_files() {
        let contract = MemdContract::default();
        let files = vec!["a.rs".to_string(), "b.rs".to_string()];
        let evidence = ContractEvidence {
            sealed_ledger_exists: true,
            files_touched: &files,
            live_ledger_exists: false,
            sealed_dir_empty: false,
            enforcement_policy_configured: false,
            enforcement_hook_wired: false,
            preference_recall_on_cold_boot_green: None,
        };
        assert!(verify_contract(&contract, &evidence).is_empty());
    }

    #[test]
    fn verify_flags_missing_surfacing_when_sealed_ledger_exists() {
        let contract = MemdContract::default();
        let evidence = ContractEvidence {
            sealed_ledger_exists: true,
            files_touched: &[],
            live_ledger_exists: false,
            sealed_dir_empty: false,
            enforcement_policy_configured: false,
            enforcement_hook_wired: false,
            preference_recall_on_cold_boot_green: None,
        };
        let violations = verify_contract(&contract, &evidence);
        assert_eq!(violations.len(), 1);
        assert_eq!(
            violations[0].guarantee,
            "surfaces_files_touched_when_sealed_ledger_exists"
        );
    }

    #[test]
    fn default_contract_has_four_guarantees() {
        let c = MemdContract::default();
        assert!(c.guarantees.surfaces_files_touched_when_sealed_ledger_exists);
        assert!(c.guarantees.seals_session_ledger_on_precompact);
        assert!(c.guarantees.enforces_continuity_gate_when_configured);
        assert!(c.guarantees.replays_preferences_on_cold_boot);
    }

    #[test]
    fn verify_flags_missing_seal_when_ledger_exists_but_sealed_dir_empty() {
        let c = MemdContract::default();
        let evidence = ContractEvidence {
            sealed_ledger_exists: false,
            files_touched: &[],
            live_ledger_exists: true,
            sealed_dir_empty: true,
            enforcement_policy_configured: false,
            enforcement_hook_wired: false,
            preference_recall_on_cold_boot_green: None,
        };
        let v = verify_contract(&c, &evidence);
        assert!(v.iter().any(|x| x.guarantee == "seals_session_ledger_on_precompact"));
    }

    #[test]
    fn verify_flags_missing_enforcement_wiring_when_policy_configured() {
        let c = MemdContract::default();
        let evidence = ContractEvidence {
            sealed_ledger_exists: true,
            files_touched: &["a.rs".into()],
            live_ledger_exists: true,
            sealed_dir_empty: false,
            enforcement_policy_configured: true,
            enforcement_hook_wired: false,
            preference_recall_on_cold_boot_green: None,
        };
        let v = verify_contract(&c, &evidence);
        assert!(v.iter().any(|x| x.guarantee == "enforces_continuity_gate_when_configured"));
    }

    #[test]
    fn verify_flags_preference_replay_regression_when_evidence_is_red() {
        let c = MemdContract::default();
        let evidence = ContractEvidence {
            sealed_ledger_exists: false,
            files_touched: &[],
            live_ledger_exists: false,
            sealed_dir_empty: false,
            enforcement_policy_configured: false,
            enforcement_hook_wired: false,
            preference_recall_on_cold_boot_green: Some(false),
        };
        let v = verify_contract(&c, &evidence);
        assert!(v.iter().any(|x| x.guarantee == "replays_preferences_on_cold_boot"));
    }

    #[test]
    fn verify_does_not_flag_preference_replay_when_evidence_is_none() {
        let c = MemdContract::default();
        let evidence = ContractEvidence {
            sealed_ledger_exists: false,
            files_touched: &[],
            live_ledger_exists: false,
            sealed_dir_empty: false,
            enforcement_policy_configured: false,
            enforcement_hook_wired: false,
            preference_recall_on_cold_boot_green: None,
        };
        let v = verify_contract(&c, &evidence);
        assert!(v.iter().all(|x| x.guarantee != "replays_preferences_on_cold_boot"));
    }
}

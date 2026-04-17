//! Live memory contract (A3-D5).
//!
//! Part 1 ships a **minimal** contract with one typed guarantee so the shape
//! is stable (`{version, guarantees: {...}}`) and downstream validators can
//! start consuming it. Part 2 expands `ContractGuarantees` as more surfacing
//! guarantees graduate from implementation detail to published contract.

use serde::{Deserialize, Serialize};

pub const CONTRACT_FILE_NAME: &str = "contract.json";
pub const CURRENT_VERSION: &str = "0.1.0";

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
}

impl Default for MemdContract {
    fn default() -> Self {
        MemdContract {
            version: CURRENT_VERSION.to_string(),
            guarantees: ContractGuarantees {
                surfaces_files_touched_when_sealed_ledger_exists: true,
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
}

pub fn verify_contract(
    contract: &MemdContract,
    evidence: &ContractEvidence<'_>,
) -> Vec<ContractViolation> {
    let mut violations = Vec::new();

    if contract
        .guarantees
        .surfaces_files_touched_when_sealed_ledger_exists
        && evidence.sealed_ledger_exists
        && evidence.files_touched.is_empty()
    {
        violations.push(ContractViolation {
            guarantee: "surfaces_files_touched_when_sealed_ledger_exists".into(),
            detail: "sealed session ledger exists under bundle but files_touched is empty"
                .into(),
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
        };
        assert!(verify_contract(&contract, &evidence).is_empty());
    }

    #[test]
    fn verify_flags_missing_surfacing_when_sealed_ledger_exists() {
        let contract = MemdContract::default();
        let evidence = ContractEvidence {
            sealed_ledger_exists: true,
            files_touched: &[],
        };
        let violations = verify_contract(&contract, &evidence);
        assert_eq!(violations.len(), 1);
        assert_eq!(
            violations[0].guarantee,
            "surfaces_files_touched_when_sealed_ledger_exists"
        );
    }
}

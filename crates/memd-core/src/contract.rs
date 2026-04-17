//! Live memory contract (A3-D5).
//!
//! v0.3.0 grows the contract with a `file_layout` section + an
//! `enforces_file_layout_contract` guarantee — memd's file-structure is now
//! hardcoded into the runtime, not relied on as prose convention. The
//! `classify_write_path` helper is what the gate uses to accept/deny/warn
//! on Edit/Write targets.

use serde::{Deserialize, Serialize};

pub const CONTRACT_FILE_NAME: &str = "contract.json";
pub const CURRENT_VERSION: &str = "0.3.0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemdContract {
    pub version: String,
    pub guarantees: ContractGuarantees,
    #[serde(default)]
    pub file_layout: FileLayoutSchema,
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
    /// A3 Part 3: writes to non-canonical / denylisted paths are caught by
    /// the PreToolUse gate. Schema in `MemdContract::file_layout`.
    #[serde(default)]
    pub enforces_file_layout_contract: bool,
}

/// Canonical paths per artifact kind + denylist. Paths are prefix patterns
/// (trailing `/` matches any descendant). Matches are longest-prefix-wins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileLayoutSchema {
    pub plans: Vec<String>,
    pub specs: Vec<String>,
    pub backlog: Vec<String>,
    pub phases: Vec<String>,
    pub handoff: Vec<String>,
    pub theory: Vec<String>,
    pub policy: Vec<String>,
    pub verification: Vec<String>,
    pub hooks: Vec<String>,
    pub denylist: Vec<String>,
}

impl Default for FileLayoutSchema {
    fn default() -> Self {
        Self {
            plans: vec!["docs/plans/".into()],
            specs: vec!["docs/specs/".into()],
            backlog: vec!["docs/backlog/".into()],
            phases: vec!["docs/phases/".into()],
            handoff: vec!["docs/handoff/".into()],
            theory: vec!["docs/theory/".into()],
            policy: vec!["docs/policy/".into()],
            verification: vec!["docs/verification/".into()],
            hooks: vec![".memd/hooks/".into()],
            denylist: vec!["docs/superpowers/".into()],
        }
    }
}

/// Result of classifying a write target against the `FileLayoutSchema`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteClassification {
    /// Path is inside a canonical location for the given kind.
    Canonical { kind: &'static str },
    /// Path is explicitly denied — writing here is a contract violation.
    /// `canonical_hint` points to the nearest kind the user probably meant.
    Denied {
        canonical_hint: Option<&'static str>,
    },
    /// Path is not covered by any canonical or deny rule (e.g. `crates/`,
    /// `scripts/`, `integrations/`) — writes allowed.
    Unmanaged,
}

/// Classify a write target (relative to repo root) against the schema.
/// Longest-prefix match wins; denylist beats canonical if equal length.
pub fn classify_write_path(schema: &FileLayoutSchema, path: &str) -> WriteClassification {
    let norm = path.trim_start_matches("./");

    // Check denylist first (deny always wins on overlap).
    for deny in &schema.denylist {
        if norm.starts_with(deny.as_str()) {
            // Hint at the closest docs/ kind by filename pattern.
            let hint = if norm.contains("plans/") {
                Some("plans (→ docs/plans/)")
            } else if norm.contains("specs/") {
                Some("specs (→ docs/specs/)")
            } else if norm.contains("backlog/") {
                Some("backlog (→ docs/backlog/)")
            } else {
                Some("plans (→ docs/plans/) or specs (→ docs/specs/)")
            };
            return WriteClassification::Denied {
                canonical_hint: hint,
            };
        }
    }

    let kinds: [(&'static str, &Vec<String>); 9] = [
        ("plans", &schema.plans),
        ("specs", &schema.specs),
        ("backlog", &schema.backlog),
        ("phases", &schema.phases),
        ("handoff", &schema.handoff),
        ("theory", &schema.theory),
        ("policy", &schema.policy),
        ("verification", &schema.verification),
        ("hooks", &schema.hooks),
    ];
    for (kind, prefixes) in kinds.iter() {
        for p in prefixes.iter() {
            if norm.starts_with(p.as_str()) {
                return WriteClassification::Canonical { kind };
            }
        }
    }

    WriteClassification::Unmanaged
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
                enforces_file_layout_contract: true,
            },
            file_layout: FileLayoutSchema::default(),
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
    /// A3 Part 3 file-layout gate wired evidence:
    /// Some(true)=gate widens to write paths and blocks denylist, Some(false)=schema
    /// present but gate not yet widened, None=not exercised.
    pub file_layout_gate_wired: Option<bool>,
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
    if g.enforces_file_layout_contract
        && matches!(evidence.file_layout_gate_wired, Some(false))
    {
        violations.push(ContractViolation {
            guarantee: "enforces_file_layout_contract".into(),
            detail: "file_layout schema present in contract but PreToolUse gate does not yet widen to Edit/Write paths".into(),
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
            file_layout_gate_wired: None,
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
            file_layout_gate_wired: None,
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
            file_layout_gate_wired: None,
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
            file_layout_gate_wired: None,
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
            file_layout_gate_wired: None,
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
            file_layout_gate_wired: None,
        };
        let v = verify_contract(&c, &evidence);
        assert!(v.iter().any(|x| x.guarantee == "replays_preferences_on_cold_boot"));
    }

    #[test]
    fn classify_recognizes_canonical_plans_dir() {
        let schema = FileLayoutSchema::default();
        assert_eq!(
            classify_write_path(&schema, "docs/plans/A3-EXECUTION-PLAN.md"),
            WriteClassification::Canonical { kind: "plans" }
        );
    }

    #[test]
    fn classify_recognizes_canonical_specs_dir() {
        let schema = FileLayoutSchema::default();
        assert_eq!(
            classify_write_path(&schema, "docs/specs/2026-04-17-foo-plan.md"),
            WriteClassification::Canonical { kind: "specs" }
        );
    }

    #[test]
    fn classify_denies_superpowers_plans() {
        let schema = FileLayoutSchema::default();
        match classify_write_path(&schema, "docs/superpowers/plans/2026-04-17-foo.md") {
            WriteClassification::Denied { canonical_hint } => {
                let hint = canonical_hint.unwrap();
                assert!(hint.contains("plans"), "hint should point at plans: {hint}");
            }
            other => panic!("expected Denied, got {other:?}"),
        }
    }

    #[test]
    fn classify_allows_unmanaged_paths() {
        let schema = FileLayoutSchema::default();
        assert_eq!(
            classify_write_path(&schema, "crates/memd-core/src/lib.rs"),
            WriteClassification::Unmanaged
        );
        assert_eq!(
            classify_write_path(&schema, "scripts/foo.sh"),
            WriteClassification::Unmanaged
        );
    }

    #[test]
    fn classify_recognizes_hooks_dir() {
        let schema = FileLayoutSchema::default();
        assert_eq!(
            classify_write_path(&schema, ".memd/hooks/memd-bootstrap.sh"),
            WriteClassification::Canonical { kind: "hooks" }
        );
    }

    #[test]
    fn contract_round_trips_file_layout_through_json() {
        let c = MemdContract::default();
        let json = serde_json::to_string_pretty(&c).unwrap();
        let parsed: MemdContract = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.file_layout, c.file_layout);
        assert!(parsed.file_layout.denylist.iter().any(|p| p == "docs/superpowers/"));
    }

    #[test]
    fn contract_default_version_is_030() {
        assert_eq!(MemdContract::default().version, "0.3.0");
    }

    #[test]
    fn verify_flags_file_layout_gate_unwired_when_evidence_is_red() {
        let c = MemdContract::default();
        let evidence = ContractEvidence {
            sealed_ledger_exists: false,
            files_touched: &[],
            live_ledger_exists: false,
            sealed_dir_empty: false,
            enforcement_policy_configured: false,
            enforcement_hook_wired: false,
            preference_recall_on_cold_boot_green: None,
            file_layout_gate_wired: Some(false),
        };
        let v = verify_contract(&c, &evidence);
        assert!(v.iter().any(|x| x.guarantee == "enforces_file_layout_contract"));
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
            file_layout_gate_wired: None,
        };
        let v = verify_contract(&c, &evidence);
        assert!(v.iter().all(|x| x.guarantee != "replays_preferences_on_cold_boot"));
    }
}

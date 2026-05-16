use crate::file_ledger::{FileInteractionLedger, FileOp, ledger_path};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

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

/// Layered decision for write operations. First runs the continuity check
/// (same sealed/fresh semantics as `gate_decision`) then, if still Allow,
/// classifies the target path against the `FileLayoutSchema` from the live
/// contract. Denylist hits become Deny (under Warn policy they become Warn
/// + systemMessage nudge); unmanaged / canonical targets pass.
pub fn gate_write_decision(
    policy: EnforcementPolicy,
    target_path: &str,
    sealed_paths: &[String],
    fresh_read_paths: &[String],
    schema: &crate::contract::FileLayoutSchema,
) -> GateDecision {
    // Continuity check first — a write to a sealed-but-unread path is
    // blocked the same way as a continuity Edit gate would.
    let cont = gate_decision(policy, target_path, sealed_paths, fresh_read_paths);
    if !matches!(cont, GateDecision::Allow) {
        return cont;
    }
    if matches!(policy, EnforcementPolicy::Off) {
        return GateDecision::Allow;
    }
    match crate::contract::classify_write_path(schema, target_path) {
        crate::contract::WriteClassification::Denied { canonical_hint } => {
            let hint = canonical_hint.unwrap_or("canonical docs/ kind");
            let reason = format!(
                "file-layout: {target_path} is under a denylisted path. Canonical location: {hint}. See .memd/contract.json file_layout schema."
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
        crate::contract::WriteClassification::Canonical { .. }
        | crate::contract::WriteClassification::Unmanaged => GateDecision::Allow,
    }
}

/// Render a GateDecision as the JSON string the PreToolUse hook should print,
/// or `None` when no output is needed (Allow). Extracted so the CLI arm and
/// tests share the same formatter.
pub fn format_gate_output(decision: GateDecision) -> Option<String> {
    match decision {
        GateDecision::Allow => None,
        GateDecision::Warn { reason, .. } => {
            Some(serde_json::json!({ "systemMessage": reason }).to_string())
        }
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

pub struct FreshReadIndex {
    paths: Vec<String>,
}

impl FreshReadIndex {
    pub fn for_session(output: &Path, session_id: &str) -> Self {
        let lp = ledger_path(output, session_id);
        let paths = if lp.exists() {
            FileInteractionLedger::load_from_path(&lp)
                .map(|l| {
                    l.entries
                        .into_iter()
                        .filter(|e| e.op == FileOp::Read)
                        .map(|e| e.path)
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        Self { paths }
    }

    pub fn contains(&self, path: &str) -> bool {
        self.paths.iter().any(|p| p == path)
    }

    pub fn paths(&self) -> &[String] {
        &self.paths
    }
}

/// A3 Part 3: cross-harness pre-send validator signals.
///
/// Pure data. The validator answers one question: is the session ready to
/// emit a completion message (handoff / end-of-turn) given its memd state?
#[derive(Debug, Clone, Copy)]
pub struct CompletionSignals {
    /// Session has at least one checkpoint written since its last wake.
    pub has_recent_checkpoint: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionDecision {
    Ready,
    Warn { reason: String },
    Block { reason: String },
}

/// One check, deliberately narrow: a completion attempt with no recent
/// checkpoint is the most common cross-harness drift. Under `Block` we
/// stop the assistant from declaring done; under `Warn` we surface a
/// reminder; under `Off` we get out of the way.
pub fn verify_completion_ready(
    policy: EnforcementPolicy,
    signals: CompletionSignals,
) -> CompletionDecision {
    if matches!(policy, EnforcementPolicy::Off) || signals.has_recent_checkpoint {
        return CompletionDecision::Ready;
    }
    let reason = "continuity: session has no checkpoint since wake — run `memd checkpoint` before declaring the turn complete.".to_string();
    match policy {
        EnforcementPolicy::Warn => CompletionDecision::Warn { reason },
        EnforcementPolicy::Block => CompletionDecision::Block { reason },
        EnforcementPolicy::Off => unreachable!(),
    }
}

pub fn load_latest_sealed_paths(output: &Path) -> Vec<String> {
    let state = output.join("state");
    let Ok(rd) = fs::read_dir(&state) else {
        return Vec::new();
    };
    let mut latest: Option<(std::time::SystemTime, PathBuf)> = None;
    for entry in rd.flatten() {
        if !entry.file_name().to_string_lossy().starts_with("session-") {
            continue;
        }
        let sealed = entry.path().join("sealed");
        let Ok(sd) = fs::read_dir(&sealed) else {
            continue;
        };
        for s in sd.flatten() {
            let p = s.path();
            if let Ok(meta) = fs::metadata(&p)
                && let Ok(mt) = meta.modified()
                && latest.as_ref().is_none_or(|(l, _)| mt > *l)
            {
                latest = Some((mt, p));
            }
        }
    }
    latest
        .and_then(|(_, p)| FileInteractionLedger::load_from_path(&p).ok())
        .map(|l| l.distinct_paths())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::contract::FileLayoutSchema;

    #[test]
    fn gate_write_block_denies_superpowers_plans_under_block_policy() {
        let schema = FileLayoutSchema::default();
        let decision = gate_write_decision(
            EnforcementPolicy::Block,
            "docs/superpowers/plans/2026-04-17-foo.md",
            &[],
            &[],
            &schema,
        );
        match decision {
            GateDecision::Deny { reason, .. } => {
                assert!(reason.contains("file-layout"));
                assert!(reason.contains("docs/superpowers"));
            }
            other => panic!("expected Deny, got {other:?}"),
        }
    }

    #[test]
    fn gate_write_warn_nudges_on_denylist_under_warn_policy() {
        let schema = FileLayoutSchema::default();
        let decision = gate_write_decision(
            EnforcementPolicy::Warn,
            "docs/superpowers/plans/foo.md",
            &[],
            &[],
            &schema,
        );
        assert!(matches!(decision, GateDecision::Warn { .. }));
    }

    #[test]
    fn gate_write_allows_canonical_plans() {
        let schema = FileLayoutSchema::default();
        assert_eq!(
            gate_write_decision(
                EnforcementPolicy::Block,
                "docs/plans/A3-EXECUTION-PLAN.md",
                &[],
                &[],
                &schema,
            ),
            GateDecision::Allow
        );
    }

    #[test]
    fn gate_write_allows_unmanaged() {
        let schema = FileLayoutSchema::default();
        assert_eq!(
            gate_write_decision(
                EnforcementPolicy::Block,
                "crates/memd-core/src/lib.rs",
                &[],
                &[],
                &schema,
            ),
            GateDecision::Allow
        );
    }

    #[test]
    fn gate_write_off_bypasses_everything() {
        let schema = FileLayoutSchema::default();
        assert_eq!(
            gate_write_decision(
                EnforcementPolicy::Off,
                "docs/superpowers/plans/foo.md",
                &[],
                &[],
                &schema,
            ),
            GateDecision::Allow
        );
    }

    #[test]
    fn gate_write_continuity_check_runs_before_layout() {
        // Sealed-but-unread path AND denylisted — continuity fires first
        // (same failure class we already warn on).
        let schema = FileLayoutSchema::default();
        let sealed: &[String] = &["docs/superpowers/plans/foo.md".into()];
        let decision = gate_write_decision(
            EnforcementPolicy::Block,
            "docs/superpowers/plans/foo.md",
            sealed,
            &[],
            &schema,
        );
        match decision {
            GateDecision::Deny { reason, .. } => {
                // Continuity message wins (sealed ledger override).
                assert!(reason.contains("continuity"));
            }
            other => panic!("expected continuity Deny, got {other:?}"),
        }
    }

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

    #[test]
    fn fresh_read_index_surfaces_only_reads_from_live_ledger() {
        use crate::file_ledger::{FileInteractionLedger, FileOp, ledger_path};
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path();
        let mut lg = FileInteractionLedger::new("sess-live");
        lg.record("a.rs", FileOp::Read, 1);
        lg.record("b.rs", FileOp::Edit, 2);
        lg.save_to_path(&ledger_path(out, "sess-live")).unwrap();
        let index = FreshReadIndex::for_session(out, "sess-live");
        assert!(index.contains("a.rs"));
        assert!(!index.contains("b.rs"), "Edit does not count as fresh Read");
    }

    #[test]
    fn verify_completion_blocks_when_block_policy_and_no_checkpoint() {
        let out = verify_completion_ready(
            EnforcementPolicy::Block,
            CompletionSignals {
                has_recent_checkpoint: false,
            },
        );
        match out {
            CompletionDecision::Block { reason } => {
                assert!(reason.contains("memd checkpoint"));
            }
            other => panic!("expected Block, got {other:?}"),
        }
    }

    #[test]
    fn verify_completion_ready_when_checkpoint_present_under_block() {
        assert_eq!(
            verify_completion_ready(
                EnforcementPolicy::Block,
                CompletionSignals {
                    has_recent_checkpoint: true
                },
            ),
            CompletionDecision::Ready
        );
    }

    #[test]
    fn verify_completion_warns_under_warn_policy_without_checkpoint() {
        assert!(matches!(
            verify_completion_ready(
                EnforcementPolicy::Warn,
                CompletionSignals {
                    has_recent_checkpoint: false
                },
            ),
            CompletionDecision::Warn { .. }
        ));
    }

    #[test]
    fn verify_completion_off_policy_bypasses_check() {
        assert_eq!(
            verify_completion_ready(
                EnforcementPolicy::Off,
                CompletionSignals {
                    has_recent_checkpoint: false
                },
            ),
            CompletionDecision::Ready
        );
    }

    #[test]
    fn load_latest_sealed_paths_returns_distinct_paths_across_sessions() {
        use crate::file_ledger::{FileInteractionLedger, FileOp, seal_session_ledger};
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path();
        // Session A: seal ledger with a.rs, b.rs
        let mut la = FileInteractionLedger::new("sess-a");
        la.record("a.rs", FileOp::Edit, 1);
        la.record("b.rs", FileOp::Read, 2);
        la.save_to_path(&ledger_path(out, "sess-a")).unwrap();
        seal_session_ledger("sess-a", out).unwrap();
        let loaded = load_latest_sealed_paths(out);
        assert!(loaded.contains(&"a.rs".to_string()));
        assert!(loaded.contains(&"b.rs".to_string()));
    }
}

use serde::{Deserialize, Serialize};
use std::{fs, path::{Path, PathBuf}};
use crate::file_ledger::{FileInteractionLedger, FileOp, ledger_path};

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

pub fn load_latest_sealed_paths(output: &Path) -> Vec<String> {
    let state = output.join("state");
    let Ok(rd) = fs::read_dir(&state) else { return Vec::new(); };
    let mut latest: Option<(std::time::SystemTime, PathBuf)> = None;
    for entry in rd.flatten() {
        if !entry.file_name().to_string_lossy().starts_with("session-") { continue; }
        let sealed = entry.path().join("sealed");
        let Ok(sd) = fs::read_dir(&sealed) else { continue; };
        for s in sd.flatten() {
            let p = s.path();
            if let Ok(meta) = fs::metadata(&p) {
                if let Ok(mt) = meta.modified() {
                    if latest.as_ref().map_or(true, |(l, _)| mt > *l) {
                        latest = Some((mt, p));
                    }
                }
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

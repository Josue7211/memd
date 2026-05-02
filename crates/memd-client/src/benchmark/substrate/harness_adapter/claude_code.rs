//! Claude Code harness adapter.
//!
//! Detection: presence + JSON-parse of `~/.claude/settings.json`. The
//! `phase-c5-plan.md` §3 note explicitly inverts the historical
//! `HARNESS_BRIDGES.md` doc — read the harness's own config file
//! directly, not the generic bridge schema.

use crate::benchmark::substrate::harness_adapter::{
    HarnessAdapter, HarnessRunOutcome, MemdGateway, Script, drive_script_via_gateway,
};
use std::path::{Path, PathBuf};

pub(crate) struct ClaudeCodeAdapter {
    config_path: PathBuf,
}

impl ClaudeCodeAdapter {
    pub(crate) fn from_home() -> Self {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_default();
        Self {
            config_path: home.join(".claude").join("settings.json"),
        }
    }

    pub(crate) fn with_config_path(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    pub(crate) fn config_path(&self) -> &Path {
        &self.config_path
    }
}

impl HarnessAdapter for ClaudeCodeAdapter {
    fn name(&self) -> &'static str {
        "claude_code"
    }

    fn is_available(&self) -> bool {
        let bytes = match std::fs::read(&self.config_path) {
            Ok(b) => b,
            Err(_) => return false,
        };
        // Parse-as-JSON guard. A missing file or malformed config
        // disqualifies the harness — graceful skip path.
        serde_json::from_slice::<serde_json::Value>(&bytes).is_ok()
    }

    fn run_script(
        &self,
        script: &Script,
        gateway: &dyn MemdGateway,
    ) -> std::io::Result<HarnessRunOutcome> {
        drive_script_via_gateway(self.name(), script, gateway)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// C5 Test 1 — `adapter_claude_code_detects_availability_via_settings_json`.
    /// Adapter must report `is_available() = true` when settings.json
    /// exists and parses, false otherwise.
    #[test]
    fn adapter_claude_code_detects_availability_via_settings_json() {
        let dir = tempdir().unwrap();
        let settings = dir.path().join("settings.json");

        let adapter = ClaudeCodeAdapter::with_config_path(settings.clone());
        assert!(
            !adapter.is_available(),
            "missing settings.json must disqualify"
        );

        fs::write(&settings, "not json").unwrap();
        assert!(
            !adapter.is_available(),
            "malformed settings.json must disqualify"
        );

        fs::write(&settings, r#"{"hooks":{}}"#).unwrap();
        assert!(adapter.is_available(), "valid settings.json must qualify");
        assert_eq!(adapter.name(), "claude_code");
    }
}

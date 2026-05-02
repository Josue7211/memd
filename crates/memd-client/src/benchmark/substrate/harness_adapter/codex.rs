//! Codex harness adapter.
//!
//! Detection: presence + JSON-parse of `~/.codex/hooks.json`. Same
//! shape as the claude-code adapter; the only difference is the
//! detection path. Both share `drive_script_via_gateway` so the
//! cross-harness runner sees a single contract.

use crate::benchmark::substrate::harness_adapter::{
    HarnessAdapter, HarnessRunOutcome, MemdGateway, Script, drive_script_via_gateway,
};
use std::path::{Path, PathBuf};

pub(crate) struct CodexAdapter {
    config_path: PathBuf,
}

impl CodexAdapter {
    pub(crate) fn from_home() -> Self {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_default();
        Self {
            config_path: home.join(".codex").join("hooks.json"),
        }
    }

    pub(crate) fn with_config_path(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    pub(crate) fn config_path(&self) -> &Path {
        &self.config_path
    }
}

impl HarnessAdapter for CodexAdapter {
    fn name(&self) -> &'static str {
        "codex"
    }

    fn is_available(&self) -> bool {
        let bytes = match std::fs::read(&self.config_path) {
            Ok(b) => b,
            Err(_) => return false,
        };
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

    /// C5 Test 2 — `adapter_codex_detects_availability_via_hooks_json`.
    #[test]
    fn adapter_codex_detects_availability_via_hooks_json() {
        let dir = tempdir().unwrap();
        let hooks = dir.path().join("hooks.json");

        let adapter = CodexAdapter::with_config_path(hooks.clone());
        assert!(
            !adapter.is_available(),
            "missing hooks.json must disqualify"
        );

        fs::write(&hooks, "garbage").unwrap();
        assert!(
            !adapter.is_available(),
            "malformed hooks.json must disqualify"
        );

        fs::write(&hooks, r#"{"hooks":{}}"#).unwrap();
        assert!(adapter.is_available(), "valid hooks.json must qualify");
        assert_eq!(adapter.name(), "codex");
    }
}

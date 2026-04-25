//! Codex harness adapter — placeholder.
//!
//! C5.1 only ships the trait + claude-code adapter to keep the commit
//! atomic. The real codex impl (Test 2) lands in C5.2.

use crate::benchmark::substrate::harness_adapter::{
    HarnessAdapter, HarnessRunOutcome, MemdGateway, Script,
};

pub(crate) struct CodexAdapter;

impl HarnessAdapter for CodexAdapter {
    fn name(&self) -> &'static str {
        "codex"
    }

    fn is_available(&self) -> bool {
        false
    }

    fn run_script(
        &self,
        _script: &Script,
        _gateway: &dyn MemdGateway,
    ) -> std::io::Result<HarnessRunOutcome> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "codex adapter not yet implemented (C5.2)",
        ))
    }
}

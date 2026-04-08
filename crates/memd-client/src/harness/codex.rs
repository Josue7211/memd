use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CodexHarnessPack {
    pub(crate) agent: String,
    pub(crate) project: String,
    pub(crate) namespace: String,
    pub(crate) bundle_root: PathBuf,
    pub(crate) files: Vec<PathBuf>,
    pub(crate) commands: Vec<String>,
    pub(crate) behaviors: Vec<String>,
}

pub(crate) fn build_codex_harness_pack(
    bundle_root: &Path,
    project: &str,
    namespace: &str,
) -> CodexHarnessPack {
    CodexHarnessPack {
        agent: "codex".to_string(),
        project: project.to_string(),
        namespace: namespace.to_string(),
        bundle_root: bundle_root.to_path_buf(),
        files: vec![
            bundle_root.join("MEMD_WAKEUP.md"),
            bundle_root.join("MEMD_MEMORY.md"),
            bundle_root.join("agents").join("CODEX_WAKEUP.md"),
            bundle_root.join("agents").join("CODEX_MEMORY.md"),
        ],
        commands: vec![
            "memd wake --output .memd --intent current_task --write".to_string(),
            "memd resume --output .memd --intent current_task".to_string(),
            "memd hook capture --output .memd --stdin --summary".to_string(),
        ],
        behaviors: vec![
            "recall before turn".to_string(),
            "capture after turn".to_string(),
            "turn-scoped cache".to_string(),
        ],
    }
}

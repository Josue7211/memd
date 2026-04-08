use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OpenClawHarnessPack {
    pub(crate) agent: String,
    pub(crate) project: String,
    pub(crate) namespace: String,
    pub(crate) bundle_root: PathBuf,
    pub(crate) files: Vec<PathBuf>,
    pub(crate) commands: Vec<String>,
    pub(crate) behaviors: Vec<String>,
}

pub(crate) fn build_openclaw_harness_pack(
    bundle_root: &Path,
    project: &str,
    namespace: &str,
) -> OpenClawHarnessPack {
    OpenClawHarnessPack {
        agent: "openclaw".to_string(),
        project: project.to_string(),
        namespace: namespace.to_string(),
        bundle_root: bundle_root.to_path_buf(),
        files: vec![
            bundle_root.join("MEMD_WAKEUP.md"),
            bundle_root.join("MEMD_MEMORY.md"),
            bundle_root.join("agents").join("OPENCLAW_WAKEUP.md"),
            bundle_root.join("agents").join("OPENCLAW_MEMORY.md"),
        ],
        commands: vec![
            "memd context --project <project> --agent openclaw --compact".to_string(),
            "memd resume --output .memd --intent current_task".to_string(),
            "memd hook spill --output .memd --stdin --apply".to_string(),
        ],
        behaviors: vec![
            "fetch compact context before the task".to_string(),
            "spill after compaction boundary".to_string(),
            "turn-scoped cache".to_string(),
        ],
    }
}

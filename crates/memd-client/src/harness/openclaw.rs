use std::path::Path;

use crate::harness::shared::HarnessPackData;

pub(crate) type OpenClawHarnessPack = HarnessPackData;

pub(crate) fn build_openclaw_harness_pack(
    bundle_root: &Path,
    project: &str,
    namespace: &str,
) -> OpenClawHarnessPack {
    HarnessPackData {
        name: "OpenClaw",
        role: "compact context/spill pack",
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

use std::path::Path;

use crate::harness::shared::HarnessPackData;

pub(crate) type CodexHarnessPack = HarnessPackData;

pub(crate) fn build_codex_harness_pack(
    bundle_root: &Path,
    project: &str,
    namespace: &str,
) -> CodexHarnessPack {
    HarnessPackData {
        name: "Codex",
        role: "turn-first recall/capture/spill pack",
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
            "memd wake --output .memd --write".to_string(),
            "memd resume --output .memd".to_string(),
            "memd lookup --output .memd --query \"what did we already decide about this?\""
                .to_string(),
            "memd hook capture --output .memd --stdin --summary".to_string(),
            "memd hook spill --output .memd --stdin --apply".to_string(),
        ],
        behaviors: vec![
            "recall before turn".to_string(),
            "pre-answer lookup before memory-dependent responses".to_string(),
            "capture after turn".to_string(),
            "spill at compaction boundaries".to_string(),
            "turn-scoped cache".to_string(),
        ],
    }
}

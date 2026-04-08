use std::path::Path;

use crate::harness::shared::HarnessPackData;

pub(crate) type OpenCodeHarnessPack = HarnessPackData;

pub(crate) fn build_opencode_harness_pack(
    bundle_root: &Path,
    project: &str,
    namespace: &str,
) -> OpenCodeHarnessPack {
    HarnessPackData {
        name: "OpenCode",
        role: "resume/remember/handoff pack",
        agent: "opencode".to_string(),
        project: project.to_string(),
        namespace: namespace.to_string(),
        bundle_root: bundle_root.to_path_buf(),
        files: vec![
            bundle_root.join("MEMD_WAKEUP.md"),
            bundle_root.join("MEMD_MEMORY.md"),
            bundle_root.join("agents").join("OPENCODE_WAKEUP.md"),
            bundle_root.join("agents").join("OPENCODE_MEMORY.md"),
        ],
        commands: vec![
            "memd resume --output .memd".to_string(),
            "memd remember --output .memd --kind decision --content \"Keep the shared lane current.\""
                .to_string(),
            "memd handoff --output .memd --prompt".to_string(),
        ],
        behaviors: vec![
            "resume before starting work".to_string(),
            "write durable outcomes back".to_string(),
            "emit a shared handoff".to_string(),
            "keep the visible bundle in sync".to_string(),
        ],
    }
}

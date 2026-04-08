use std::path::Path;

use crate::harness::shared::HarnessPackData;

pub(crate) type HermesHarnessPack = HarnessPackData;

pub(crate) fn build_hermes_harness_pack(
    bundle_root: &Path,
    project: &str,
    namespace: &str,
) -> HermesHarnessPack {
    HarnessPackData {
        name: "Hermes",
        role: "adoption-focused wake/capture pack",
        agent: "hermes".to_string(),
        project: project.to_string(),
        namespace: namespace.to_string(),
        bundle_root: bundle_root.to_path_buf(),
        files: vec![
            bundle_root.join("MEMD_WAKEUP.md"),
            bundle_root.join("MEMD_MEMORY.md"),
            bundle_root.join("agents").join("HERMES_WAKEUP.md"),
            bundle_root.join("agents").join("HERMES_MEMORY.md"),
        ],
        commands: vec![
            "memd wake --output .memd --intent current_task --write".to_string(),
            "memd resume --output .memd --intent current_task --semantic".to_string(),
            "memd hook capture --output .memd --stdin --summary".to_string(),
        ],
        behaviors: vec![
            "onboarding-friendly wake before the turn".to_string(),
            "capture after the turn".to_string(),
            "turn-scoped cache".to_string(),
            "cloud-first reach with self-host later".to_string(),
        ],
    }
}

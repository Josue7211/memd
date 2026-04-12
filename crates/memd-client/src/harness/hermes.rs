use std::path::Path;

use crate::harness::preset::SHARED_VISIBLE_SURFACES;
use crate::harness::shared::HarnessPackData;

pub(crate) type HermesHarnessPack = HarnessPackData;

pub(crate) fn build_hermes_harness_pack(
    bundle_root: &Path,
    project: &str,
    namespace: &str,
) -> HermesHarnessPack {
    HarnessPackData {
        name: "Hermes",
        role: "adoption-focused wake/capture/spill pack",
        agent: "hermes".to_string(),
        project: project.to_string(),
        namespace: namespace.to_string(),
        bundle_root: bundle_root.to_path_buf(),
        files: SHARED_VISIBLE_SURFACES
            .iter()
            .map(|surface| bundle_root.join(surface))
            .collect(),
        commands: vec![
            "memd wake --output .memd --write".to_string(),
            "memd resume --output .memd --semantic".to_string(),
            "memd hook capture --output .memd --stdin --summary".to_string(),
            "memd hook spill --output .memd --stdin --apply".to_string(),
        ],
        behaviors: vec![
            "onboarding-friendly wake before the turn".to_string(),
            "capture after the turn".to_string(),
            "spill at compaction boundaries".to_string(),
            "turn-scoped cache".to_string(),
            "cloud-first reach with self-host later".to_string(),
        ],
    }
}

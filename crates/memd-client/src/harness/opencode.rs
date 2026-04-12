use std::path::Path;

use crate::harness::preset::SHARED_VISIBLE_SURFACES;
use crate::harness::shared::HarnessPackData;

pub(crate) type OpenCodeHarnessPack = HarnessPackData;

pub(crate) fn build_opencode_harness_pack(
    bundle_root: &Path,
    project: &str,
    namespace: &str,
) -> OpenCodeHarnessPack {
    HarnessPackData {
        name: "OpenCode",
        role: "resume/remember/handoff/spill pack",
        agent: "opencode".to_string(),
        project: project.to_string(),
        namespace: namespace.to_string(),
        bundle_root: bundle_root.to_path_buf(),
        files: SHARED_VISIBLE_SURFACES
            .iter()
            .map(|surface| bundle_root.join(surface))
            .collect(),
        commands: vec![
            "memd resume --output .memd".to_string(),
            "memd remember --output .memd --kind decision --content \"Keep the shared lane current.\""
                .to_string(),
            "memd handoff --output .memd --prompt".to_string(),
            "memd hook spill --output .memd --stdin --apply".to_string(),
        ],
        behaviors: vec![
            "resume before starting work".to_string(),
            "write durable outcomes back".to_string(),
            "emit a shared handoff".to_string(),
            "spill at compaction boundaries".to_string(),
            "keep the visible bundle in sync".to_string(),
        ],
    }
}

use std::path::Path;

use crate::harness::shared::HarnessPackData;

pub(crate) type AgentZeroHarnessPack = HarnessPackData;

pub(crate) fn build_agent_zero_harness_pack(
    bundle_root: &Path,
    project: &str,
    namespace: &str,
) -> AgentZeroHarnessPack {
    HarnessPackData {
        name: "Agent Zero",
        role: "zero-friction resume/remember/handoff/spill pack",
        agent: "agent-zero".to_string(),
        project: project.to_string(),
        namespace: namespace.to_string(),
        bundle_root: bundle_root.to_path_buf(),
        files: vec![
            bundle_root.join("MEMD_WAKEUP.md"),
            bundle_root.join("MEMD_MEMORY.md"),
            bundle_root.join("agents").join("AGENT_ZERO_WAKEUP.md"),
            bundle_root.join("agents").join("AGENT_ZERO_MEMORY.md"),
        ],
        commands: vec![
            "memd resume --output .memd".to_string(),
            "memd remember --output .memd --kind decision --content \"Keep the zero-friction lane current.\""
                .to_string(),
            "memd handoff --output .memd --prompt".to_string(),
            "memd hook spill --output .memd --stdin --apply".to_string(),
        ],
        behaviors: vec![
            "zero-friction resume before starting work".to_string(),
            "write durable outcomes back".to_string(),
            "emit a shared handoff".to_string(),
            "spill at compaction boundaries".to_string(),
            "keep the visible bundle in sync".to_string(),
        ],
    }
}

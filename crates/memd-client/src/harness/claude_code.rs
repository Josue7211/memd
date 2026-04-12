use std::path::Path;

use crate::harness::preset::WAKE_ONLY_SURFACES;
use crate::harness::shared::HarnessPackData;

pub(crate) type ClaudeCodeHarnessPack = HarnessPackData;

pub(crate) fn build_claude_code_harness_pack(
    bundle_root: &Path,
    project: &str,
    namespace: &str,
) -> ClaudeCodeHarnessPack {
    let mut files: Vec<_> = WAKE_ONLY_SURFACES
        .iter()
        .map(|surface| bundle_root.join(surface))
        .collect();
    files.push(bundle_root.join("agents").join("CLAUDE_IMPORTS.md"));
    files.push(bundle_root.join("agents").join("CLAUDE.md.example"));
    HarnessPackData {
        name: "Claude Code",
        role: "native import bridge pack",
        agent: "claude-code".to_string(),
        project: project.to_string(),
        namespace: namespace.to_string(),
        bundle_root: bundle_root.to_path_buf(),
        files,
        commands: vec![
            "memd wake --output .memd --write".to_string(),
            "memd resume --output .memd".to_string(),
            "memd lookup --output .memd --query \"what did we already decide about this?\""
                .to_string(),
        ],
        behaviors: vec![
            "native Claude import bridge".to_string(),
            "pre-answer lookup before memory-dependent responses".to_string(),
            "wake-only boot path with explicit deeper recall".to_string(),
        ],
    }
}

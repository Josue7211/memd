use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{codex::CodexHarnessPack, openclaw::OpenClawHarnessPack};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HarnessPackIndex {
    pub(crate) root: String,
    pub(crate) project: String,
    pub(crate) namespace: String,
    pub(crate) pack_count: usize,
    pub(crate) packs: Vec<HarnessPackIndexEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HarnessPackIndexEntry {
    pub(crate) name: String,
    pub(crate) role: String,
    pub(crate) project: String,
    pub(crate) namespace: String,
    pub(crate) bundle_root: String,
    pub(crate) files: Vec<String>,
    pub(crate) commands: Vec<String>,
    pub(crate) behaviors: Vec<String>,
}

pub(crate) fn build_harness_pack_index(
    bundle_root: &Path,
    project: Option<&str>,
    namespace: Option<&str>,
) -> HarnessPackIndex {
    let project = project.unwrap_or("none").trim().to_string();
    let namespace = namespace.unwrap_or("none").trim().to_string();
    let codex = super::codex::build_codex_harness_pack(bundle_root, &project, &namespace);
    let openclaw = super::openclaw::build_openclaw_harness_pack(bundle_root, &project, &namespace);
    let packs = vec![
        HarnessPackIndexEntry::from(&codex),
        HarnessPackIndexEntry::from(&openclaw),
    ];

    HarnessPackIndex {
        root: bundle_root.display().to_string(),
        project,
        namespace,
        pack_count: packs.len(),
        packs,
    }
}

pub(crate) fn filter_harness_pack_index(
    index: HarnessPackIndex,
    query: Option<&str>,
) -> HarnessPackIndex {
    let Some(query) = query
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
    else {
        return index;
    };

    let packs = index
        .packs
        .into_iter()
        .filter(|pack| harness_pack_matches(pack, &query))
        .collect::<Vec<_>>();

    HarnessPackIndex {
        pack_count: packs.len(),
        packs,
        ..index
    }
}

fn harness_pack_matches(pack: &HarnessPackIndexEntry, query: &str) -> bool {
    let mut fields = vec![
        pack.name.to_lowercase(),
        pack.role.to_lowercase(),
        pack.project.to_lowercase(),
        pack.namespace.to_lowercase(),
        pack.bundle_root.to_lowercase(),
    ];
    fields.extend(pack.files.iter().map(|value| value.to_lowercase()));
    fields.extend(pack.commands.iter().map(|value| value.to_lowercase()));
    fields.extend(pack.behaviors.iter().map(|value| value.to_lowercase()));
    fields.into_iter().any(|field| field.contains(query))
}

impl From<&CodexHarnessPack> for HarnessPackIndexEntry {
    fn from(pack: &CodexHarnessPack) -> Self {
        Self {
            name: "Codex".to_string(),
            role: "turn-first recall/capture pack".to_string(),
            project: pack.project.clone(),
            namespace: pack.namespace.clone(),
            bundle_root: pack.bundle_root.display().to_string(),
            files: pack
                .files
                .iter()
                .map(|path| path.display().to_string())
                .collect(),
            commands: pack.commands.clone(),
            behaviors: pack.behaviors.clone(),
        }
    }
}

impl From<&OpenClawHarnessPack> for HarnessPackIndexEntry {
    fn from(pack: &OpenClawHarnessPack) -> Self {
        Self {
            name: "OpenClaw".to_string(),
            role: "compact context/spill pack".to_string(),
            project: pack.project.clone(),
            namespace: pack.namespace.clone(),
            bundle_root: pack.bundle_root.display().to_string(),
            files: pack
                .files
                .iter()
                .map(|path| path.display().to_string())
                .collect(),
            commands: pack.commands.clone(),
            behaviors: pack.behaviors.clone(),
        }
    }
}

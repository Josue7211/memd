use std::path::Path;

use serde::{Deserialize, Serialize};

use super::shared::pack_index_entry_from_view;

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
    let claude_code =
        super::claude_code::build_claude_code_harness_pack(bundle_root, &project, &namespace);
    let agent_zero =
        super::agent_zero::build_agent_zero_harness_pack(bundle_root, &project, &namespace);
    let hermes = super::hermes::build_hermes_harness_pack(bundle_root, &project, &namespace);
    let opencode = super::opencode::build_opencode_harness_pack(bundle_root, &project, &namespace);
    let openclaw = super::openclaw::build_openclaw_harness_pack(bundle_root, &project, &namespace);
    let packs = vec![
        HarnessPackIndexEntry::from(&codex),
        HarnessPackIndexEntry::from(&claude_code),
        HarnessPackIndexEntry::from(&agent_zero),
        HarnessPackIndexEntry::from(&hermes),
        HarnessPackIndexEntry::from(&opencode),
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

impl<T: super::shared::HarnessPackView> From<&T> for HarnessPackIndexEntry {
    fn from(pack: &T) -> Self {
        pack_index_entry_from_view(pack)
    }
}

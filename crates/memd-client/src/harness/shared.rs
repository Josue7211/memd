use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HarnessPackData {
    pub(crate) name: &'static str,
    pub(crate) role: &'static str,
    pub(crate) agent: String,
    pub(crate) project: String,
    pub(crate) namespace: String,
    pub(crate) bundle_root: PathBuf,
    pub(crate) files: Vec<PathBuf>,
    pub(crate) commands: Vec<String>,
    pub(crate) behaviors: Vec<String>,
}

#[allow(dead_code)]
pub(crate) trait HarnessPackView {
    fn name(&self) -> &'static str;
    fn role(&self) -> &'static str;
    fn agent(&self) -> &str;
    fn project(&self) -> &str;
    fn namespace(&self) -> &str;
    fn bundle_root(&self) -> &Path;
    fn files(&self) -> &[PathBuf];
    fn commands(&self) -> &[String];
    fn behaviors(&self) -> &[String];
}

impl HarnessPackView for HarnessPackData {
    fn name(&self) -> &'static str {
        self.name
    }

    fn role(&self) -> &'static str {
        self.role
    }

    fn agent(&self) -> &str {
        &self.agent
    }

    fn project(&self) -> &str {
        &self.project
    }

    fn namespace(&self) -> &str {
        &self.namespace
    }

    fn bundle_root(&self) -> &Path {
        &self.bundle_root
    }

    fn files(&self) -> &[PathBuf] {
        &self.files
    }

    fn commands(&self) -> &[String] {
        &self.commands
    }

    fn behaviors(&self) -> &[String] {
        &self.behaviors
    }
}

#[allow(dead_code)]
pub(crate) fn render_harness_pack_markdown(pack: &impl HarnessPackView) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!("# {} Harness Pack\n\n", pack.name()));
    markdown.push_str(&format!("- agent: `{}`\n", pack.agent()));
    markdown.push_str(&format!("- project: `{}`\n", pack.project()));
    markdown.push_str(&format!("- namespace: `{}`\n", pack.namespace()));
    markdown.push_str(&format!(
        "- bundle root: `{}`\n\n",
        pack.bundle_root().display()
    ));

    markdown.push_str("## Files\n");
    for file in pack.files() {
        markdown.push_str(&format!("- `{}`\n", file.display()));
    }
    markdown.push('\n');

    markdown.push_str("## Behaviors\n");
    for behavior in pack.behaviors() {
        markdown.push_str(&format!("- {}\n", behavior));
    }
    markdown.push('\n');

    markdown.push_str("## Commands\n");
    for command in pack.commands() {
        markdown.push_str(&format!("- `{}`\n", command));
    }

    markdown
}

pub(crate) fn pack_index_entry_from_view(
    pack: &impl HarnessPackView,
) -> crate::harness::index::HarnessPackIndexEntry {
    crate::harness::index::HarnessPackIndexEntry {
        name: pack.name().to_string(),
        role: pack.role().to_string(),
        project: pack.project().to_string(),
        namespace: pack.namespace().to_string(),
        bundle_root: pack.bundle_root().display().to_string(),
        files: pack
            .files()
            .iter()
            .map(|path| path.display().to_string())
            .collect(),
        commands: pack.commands().to_vec(),
        behaviors: pack.behaviors().to_vec(),
    }
}

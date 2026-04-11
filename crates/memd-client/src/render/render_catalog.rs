use super::*;

#[allow(dead_code)]
pub(crate) fn render_harness_preset_markdown(preset: &HarnessPreset) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!("# {} Harness Pack\n\n", preset.display_name));
    markdown.push_str(&format!("- pack id: `{}`\n", preset.pack_id));
    markdown.push_str(&format!("- entrypoint: `{}`\n", preset.entrypoint));
    markdown.push_str(&format!("- cache policy: {}\n", preset.cache_policy));
    markdown.push_str(&format!("- tone: {}\n\n", preset.copy_tone));

    markdown.push_str("## Surface Set\n");
    for surface in preset.surface_set {
        markdown.push_str(&format!("- `{}`\n", surface));
    }
    markdown.push('\n');

    markdown.push_str("## Default Verbs\n");
    for verb in preset.default_verbs {
        markdown.push_str(&format!("- `{}`\n", verb));
    }
    markdown.push('\n');

    markdown.push_str("## Shared Core\n");
    markdown.push_str(
        "memd owns the same memory control plane, compiled pages, and turn-scoped cache.\n",
    );
    markdown
}

pub(crate) fn render_harness_pack_index_summary(
    bundle_root: &std::path::Path,
    index: &HarnessPackIndex,
    query: Option<&str>,
) -> String {
    let mut summary = format!(
        "pack index root={} project={} namespace={} packs={}",
        bundle_root.display(),
        index.project,
        index.namespace,
        index.pack_count
    );
    if let Some(query) = query {
        summary.push_str(&format!(" query={}", compact_inline(query, 48)));
    }
    if !index.packs.is_empty() {
        summary.push_str(&format!(
            " names={}",
            index
                .packs
                .iter()
                .map(|pack| pack.name.as_str())
                .collect::<Vec<_>>()
                .join("|")
        ));
    }
    if !index.preset_names.is_empty() {
        summary.push_str(&format!(" presets={}", index.preset_names.join("|")));
    }
    summary
}

pub(crate) fn render_harness_pack_index_markdown(
    bundle_root: &std::path::Path,
    index: &HarnessPackIndex,
) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd harness packs\n\n");
    markdown.push_str(&format!("- Root: `{}`\n", bundle_root.display()));
    markdown.push_str(&format!("- Project: `{}`\n", index.project));
    markdown.push_str(&format!("- Namespace: `{}`\n", index.namespace));
    markdown.push_str(&format!("- Packs: `{}`\n\n", index.pack_count));
    if !index.preset_names.is_empty() {
        markdown.push_str("## Harness Presets\n");
        for preset in &index.preset_names {
            markdown.push_str(&format!("- `{}`\n", preset));
        }
        markdown.push('\n');
    }

    if index.packs.is_empty() {
        markdown.push_str("No visible harness packs found.\n");
        return markdown;
    }

    for pack in &index.packs {
        render_harness_pack_section(&mut markdown, pack);
    }

    markdown
}

pub(crate) fn render_harness_pack_index_json(index: &HarnessPackIndex) -> HarnessPackIndexJson {
    HarnessPackIndexJson {
        root: index.root.clone(),
        project: index.project.clone(),
        namespace: index.namespace.clone(),
        pack_count: index.pack_count,
        preset_names: index.preset_names.clone(),
        packs: index
            .packs
            .iter()
            .map(HarnessPackIndexEntryJson::from)
            .collect(),
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct HarnessPackIndexJson {
    pub(crate) root: String,
    pub(crate) project: String,
    pub(crate) namespace: String,
    pub(crate) pack_count: usize,
    pub(crate) preset_names: Vec<String>,
    pub(crate) packs: Vec<HarnessPackIndexEntryJson>,
}

#[derive(Debug, Serialize)]
pub(crate) struct HarnessPackIndexEntryJson {
    pub(crate) name: String,
    pub(crate) role: String,
    pub(crate) project: String,
    pub(crate) namespace: String,
    pub(crate) bundle_root: String,
    pub(crate) files: Vec<String>,
    pub(crate) commands: Vec<String>,
    pub(crate) behaviors: Vec<String>,
}

impl From<&HarnessPackIndexEntry> for HarnessPackIndexEntryJson {
    fn from(value: &HarnessPackIndexEntry) -> Self {
        Self {
            name: value.name.clone(),
            role: value.role.clone(),
            project: value.project.clone(),
            namespace: value.namespace.clone(),
            bundle_root: value.bundle_root.clone(),
            files: value.files.clone(),
            commands: value.commands.clone(),
            behaviors: value.behaviors.clone(),
        }
    }
}

fn render_harness_pack_section(markdown: &mut String, pack: &HarnessPackIndexEntry) {
    markdown.push_str(&format!("## {}\n\n", pack.name));
    markdown.push_str(&format!("- role: `{}`\n", pack.role));
    markdown.push_str(&format!("- project: `{}`\n", pack.project));
    markdown.push_str(&format!("- namespace: `{}`\n", pack.namespace));
    markdown.push_str(&format!("- bundle root: `{}`\n\n", pack.bundle_root));

    markdown.push_str("### Files\n");
    for file in &pack.files {
        markdown.push_str(&format!("- `{}`\n", file));
    }
    markdown.push('\n');

    markdown.push_str("### Commands\n");
    for command in &pack.commands {
        markdown.push_str(&format!("- `{}`\n", command));
    }
    markdown.push('\n');

    markdown.push_str("### Behaviors\n");
    for behavior in &pack.behaviors {
        markdown.push_str(&format!("- {}\n", behavior));
    }
    markdown.push('\n');
}

pub(crate) fn render_command_catalog_summary(
    catalog: &CommandCatalog,
    query: Option<&str>,
) -> String {
    let mut summary = format!(
        "commands root={} commands={}",
        catalog.root, catalog.command_count
    );
    let native_count = catalog
        .commands
        .iter()
        .filter(|command| command.role == "native-cli")
        .count();
    let external_count = catalog
        .commands
        .iter()
        .filter(|command| command.role == "bridge-surface")
        .count();
    let helper_count = catalog
        .commands
        .iter()
        .filter(|command| command.role == "bundle-helper")
        .count();
    summary.push_str(&format!(
        " native={} external={} helpers={}",
        native_count, external_count, helper_count
    ));
    if let Some(query) = query {
        summary.push_str(&format!(" query={}", compact_inline(query, 48)));
    }
    if !catalog.commands.is_empty() {
        summary.push_str(&format!(
            " names={}",
            catalog
                .commands
                .iter()
                .map(|command| command.name.as_str())
                .collect::<Vec<_>>()
                .join("|")
        ));
    }
    summary
}

pub(crate) fn render_command_catalog_markdown(catalog: &CommandCatalog) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd commands\n\n");
    markdown.push_str(&format!("- Root: `{}`\n", catalog.root));
    markdown.push_str(&format!("- Commands: `{}`\n\n", catalog.command_count));
    if catalog.commands.is_empty() {
        markdown.push_str("No commands found.\n");
        return markdown;
    }
    markdown.push_str("memd owns the native CLI surfaces. External bridge surfaces stay listed so they can be migrated, swapped, or reimplemented on other harnesses without pretending memd owns them.\n\n");
    render_command_catalog_section(
        &mut markdown,
        "Native memd CLI",
        &catalog.commands,
        |command| command.role == "native-cli",
    );
    render_command_catalog_section(
        &mut markdown,
        "Bridge surfaces",
        &catalog.commands,
        |command| command.role == "bridge-surface",
    );
    render_command_catalog_section(
        &mut markdown,
        "Bundle helpers",
        &catalog.commands,
        |command| command.role == "bundle-helper",
    );
    markdown
}

pub(crate) fn render_command_catalog_json(catalog: &CommandCatalog) -> CommandCatalogJson {
    CommandCatalogJson {
        root: catalog.root.clone(),
        command_count: catalog.command_count,
        commands: catalog
            .commands
            .iter()
            .map(CommandCatalogEntryJson::from)
            .collect(),
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CommandCatalogJson {
    pub(crate) root: String,
    pub(crate) command_count: usize,
    pub(crate) commands: Vec<CommandCatalogEntryJson>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CommandCatalogEntryJson {
    pub(crate) name: String,
    pub(crate) surface: String,
    pub(crate) kind: String,
    pub(crate) ownership: String,
    pub(crate) role: String,
    pub(crate) compatibility_flags: Vec<String>,
    pub(crate) command: String,
    pub(crate) purpose: String,
    pub(crate) path: Option<String>,
}

impl From<&CommandCatalogEntry> for CommandCatalogEntryJson {
    fn from(value: &CommandCatalogEntry) -> Self {
        Self {
            name: value.name.clone(),
            surface: value.surface.clone(),
            kind: value.kind.clone(),
            ownership: value.ownership.clone(),
            role: value.role.clone(),
            compatibility_flags: value.compatibility_flags.clone(),
            command: value.command.clone(),
            purpose: value.purpose.clone(),
            path: value.path.clone(),
        }
    }
}

fn render_command_catalog_entry(markdown: &mut String, command: &CommandCatalogEntry) {
    markdown.push_str(&format!("## {}\n\n", command.name));
    markdown.push_str(&format!("- surface: `{}`\n", command.surface));
    markdown.push_str(&format!("- kind: `{}`\n", command.kind));
    markdown.push_str(&format!("- ownership: `{}`\n", command.ownership));
    markdown.push_str(&format!("- role: `{}`\n", command.role));
    if !command.compatibility_flags.is_empty() {
        markdown.push_str(&format!(
            "- compatibility: `{}`\n",
            command.compatibility_flags.join("`, `")
        ));
    }
    markdown.push_str(&format!("- command: `{}`\n", command.command));
    markdown.push_str(&format!("- purpose: {}\n", command.purpose));
    if let Some(path) = command.path.as_deref() {
        markdown.push_str(&format!("- path: `{}`\n", path));
    }
    markdown.push('\n');
}

fn render_command_catalog_section<F>(
    markdown: &mut String,
    title: &str,
    commands: &[CommandCatalogEntry],
    matches: F,
) where
    F: Fn(&CommandCatalogEntry) -> bool,
{
    let filtered = commands
        .iter()
        .filter(|command| matches(command))
        .collect::<Vec<_>>();
    if filtered.is_empty() {
        return;
    }
    markdown.push_str(&format!("## {}\n\n", title));
    for command in filtered {
        render_command_catalog_entry(markdown, command);
    }
}

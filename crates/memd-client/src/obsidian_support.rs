use super::*;

pub(crate) fn obsidian_path_is_internal(path: &Path) -> bool {
    path.components().any(|component| {
        matches!(
            component,
            std::path::Component::Normal(name)
                if name == ".memd" || name == ".obsidian" || name == ".git"
        )
    })
}

pub(crate) fn workspace_path_is_internal(path: &Path) -> bool {
    path.components().any(|component| {
        matches!(
            component,
            std::path::Component::Normal(name)
                if name == ".memd"
                    || name == ".git"
                    || name == "target"
                    || name == "node_modules"
                    || name == "watch.out"
                    || name == "memd-watch.log"
                    || name == "memd-watch.err"
        )
    })
}

pub(crate) fn workspace_path_should_trigger(path: &Path) -> bool {
    if workspace_path_is_internal(path) {
        return false;
    }

    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    if matches!(
        file_name,
        "Cargo.toml"
            | "Cargo.lock"
            | "Makefile"
            | "Dockerfile"
            | "README"
            | "README.md"
            | "AGENTS.md"
            | "CLAUDE.md"
            | "ROADMAP.md"
            | "DESIGN.md"
            | "CONTRIBUTING.md"
            | "CHANGELOG.md"
    ) {
        return true;
    }

    match path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
    {
        Some(ext)
            if matches!(
                ext.as_str(),
                "rs" | "toml"
                    | "md"
                    | "sh"
                    | "ps1"
                    | "json"
                    | "yml"
                    | "yaml"
                    | "js"
                    | "ts"
                    | "tsx"
                    | "py"
                    | "go"
                    | "c"
                    | "h"
                    | "cpp"
                    | "css"
                    | "html"
                    | "txt"
                    | "lock"
            ) =>
        {
            true
        }
        _ => false,
    }
}

pub(crate) fn count_obsidian_mirrors(vault: &Path, kind: &str) -> anyhow::Result<usize> {
    let root = vault.join(".memd").join("writeback").join(kind);
    if !root.exists() {
        return Ok(0);
    }
    let mut count = 0usize;
    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if entry.file_type().is_file() {
            count += 1;
        }
    }
    Ok(count)
}

pub(crate) fn resolve_pack_bundle_root(explicit: Option<&Path>) -> anyhow::Result<PathBuf> {
    if let Some(explicit) = explicit {
        return Ok(explicit.to_path_buf());
    }

    Ok(resolve_default_bundle_root()?.unwrap_or_else(default_bundle_root_path))
}

pub(crate) fn read_request<T>(input: &RequestInput) -> anyhow::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let json = if let Some(json) = &input.json {
        json.clone()
    } else if let Some(path) = &input.input {
        fs::read_to_string(path).with_context(|| format!("read request file {}", path.display()))?
    } else if input.stdin {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("read request from stdin")?;
        buffer
    } else {
        anyhow::bail!("provide --json, --input, or --stdin");
    };

    serde_json::from_str(&json).context("parse request json")
}

pub(crate) fn print_json<T>(value: &T) -> anyhow::Result<()>
where
    T: serde::Serialize,
{
    let json = serde_json::to_string_pretty(value).context("serialize response json")?;
    println!("{json}");
    Ok(())
}

#[derive(Debug, Clone, Args)]
pub(crate) struct RequestInput {
    #[arg(long)]
    pub(crate) json: Option<String>,

    #[arg(long)]
    pub(crate) input: Option<PathBuf>,

    #[arg(long)]
    pub(crate) stdin: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct ObsidianImportOutput {
    pub(crate) preview: ObsidianImportPreview,
    pub(crate) submitted: usize,
    pub(crate) attachment_submitted: usize,
    pub(crate) duplicates: usize,
    pub(crate) attachment_duplicates: usize,
    pub(crate) note_failures: usize,
    pub(crate) attachment_failures: usize,
    pub(crate) links_created: usize,
    pub(crate) attachment_links_created: usize,
    pub(crate) mirrored_notes: usize,
    pub(crate) mirrored_attachments: usize,
    pub(crate) attachments: Option<MultimodalIngestOutput>,
    pub(crate) attachment_unchanged_count: usize,
    pub(crate) dry_run: bool,
}

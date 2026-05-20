use super::*;

#[derive(Debug, Clone, Args)]
pub(crate) struct SkillArgs {
    #[command(subcommand)]
    pub(crate) command: SkillSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum SkillSubcommand {
    Add(SkillAddArgs),
    List(SkillListArgs),
    Show(SkillShowArgs),
    Retire(SkillRetireArgs),
    Sync(SkillSyncArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct SkillAddArgs {
    /// Skill name (required).
    #[arg(long, value_name = "NAME")]
    pub(crate) name: String,

    /// Skill description (required).
    #[arg(long, value_name = "DESC")]
    pub(crate) description: String,

    /// Skill body text (can be provided via --body, --body-file, or --stdin).
    #[arg(long, value_name = "TEXT")]
    pub(crate) body: Option<String>,

    /// Path to file containing skill body.
    #[arg(long, value_name = "PATH")]
    pub(crate) body_file: Option<PathBuf>,

    /// Read skill body from stdin.
    #[arg(long, default_value_t = false)]
    pub(crate) stdin: bool,

    /// Bundle root (defaults to .memd).
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    /// Memory scope (defaults to project).
    #[arg(long)]
    pub(crate) scope: Option<String>,

    /// Tags to attach to the memory record (repeatable).
    #[arg(long, value_name = "TEXT")]
    pub(crate) tag: Vec<String>,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct SkillListArgs {
    /// Bundle root (defaults to .memd).
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    /// Print full JSON instead of summary line.
    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct SkillShowArgs {
    /// Skill name (required).
    #[arg(long, value_name = "NAME")]
    pub(crate) name: String,

    /// Bundle root (defaults to .memd).
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct SkillRetireArgs {
    /// Skill name (required).
    #[arg(long, value_name = "NAME")]
    pub(crate) name: String,

    /// Bundle root (defaults to .memd).
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    /// Only delete disk mirror, leave record alive in memd.
    #[arg(long, default_value_t = false)]
    pub(crate) keep_record: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct SkillSyncArgs {
    /// Bundle root (defaults to .memd).
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    /// Print would-write set without touching the filesystem.
    #[arg(long, default_value_t = false)]
    pub(crate) dry_run: bool,

    /// Remove mirror dirs whose record is missing or retired.
    #[arg(long, default_value_t = false)]
    pub(crate) prune: bool,
}

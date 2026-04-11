use super::*;

pub(crate) fn snapshot_bundle_for_reversion(output: &Path) -> anyhow::Result<PathBuf> {
    let snapshot_root =
        std::env::temp_dir().join(format!("memd-experiment-backup-{}", uuid::Uuid::new_v4()));
    copy_dir_contents(output, &snapshot_root)?;
    Ok(snapshot_root)
}

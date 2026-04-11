use super::*;

pub(crate) fn git_stdout(root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(crate) fn detect_git_worktree_root(root: &Path) -> Option<PathBuf> {
    git_stdout(root, &["rev-parse", "--show-toplevel"]).map(PathBuf::from)
}

pub(crate) fn detect_git_repo_root(root: &Path) -> Option<PathBuf> {
    let common_dir = git_stdout(
        root,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )
    .map(PathBuf::from)?;
    if common_dir
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value == ".git")
    {
        common_dir.parent().map(Path::to_path_buf)
    } else {
        detect_git_worktree_root(root)
    }
}

pub(crate) fn git_worktree_dirty(root: &Path) -> bool {
    git_dirty_paths(root).is_some_and(|paths| !paths.is_empty())
}

pub(crate) fn git_dirty_paths(root: &Path) -> Option<BTreeSet<String>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("status")
        .arg("--porcelain")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(parse_git_status_path)
            .filter(|path| !is_bundle_generated_path(path))
            .collect(),
    )
}

pub(crate) fn git_branch_changed_paths(
    root: &Path,
    base_branch: &str,
    branch: &str,
) -> Option<BTreeSet<String>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("diff")
        .arg("--name-only")
        .arg(format!("{base_branch}..{branch}"))
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .filter(|path| !path.is_empty())
            .map(str::to_string)
            .collect(),
    )
}

pub(crate) fn git_worktree_conflicts_with_branch(
    root: &Path,
    base_branch: &str,
    branch: &str,
) -> bool {
    let Some(dirty_paths) = git_dirty_paths(root) else {
        return git_worktree_dirty(root);
    };
    if dirty_paths.is_empty() {
        return false;
    }
    let Some(branch_paths) = git_branch_changed_paths(root, base_branch, branch) else {
        return true;
    };
    if branch_paths.is_empty() {
        return false;
    }
    dirty_paths.iter().any(|path| branch_paths.contains(path))
}

pub(crate) fn parse_git_status_path(line: &str) -> Option<String> {
    if line.len() < 4 {
        return None;
    }
    let path = line.get(3..)?.trim();
    if path.is_empty() {
        return None;
    }
    if let Some((_, renamed)) = path.split_once(" -> ") {
        return Some(renamed.trim().to_string());
    }
    Some(path.to_string())
}

pub(crate) fn is_bundle_generated_path(path: &str) -> bool {
    let normalized = path.trim_start_matches("./");
    normalized == ".memd" || normalized.starts_with(".memd/") || normalized.contains("/.memd/")
}

pub(crate) fn branch_prefix_from_branch_name(branch: &str) -> String {
    branch
        .rsplit_once('/')
        .map(|(prefix, _)| prefix.to_string())
        .unwrap_or_else(|| branch.to_string())
}

pub(crate) fn git_branch_exists(root: &Path, branch: &str) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("show-ref")
        .arg("--verify")
        .arg(format!("refs/heads/{branch}"))
        .output()
        .ok()
        .is_some_and(|output| output.status.success())
}

pub(crate) fn git_branch_has_diff(root: &Path, base_branch: &str, branch: &str) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("diff")
        .arg("--quiet")
        .arg(format!("{base_branch}..{branch}"))
        .status()
        .ok()
        .is_some_and(|status| !status.success())
}

pub(crate) fn git_branch_tip_ancestor_of_head(root: &Path, branch: &str) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("merge-base")
        .arg("--is-ancestor")
        .arg(branch)
        .arg("HEAD")
        .status()
        .ok()
        .is_some_and(|status| status.success())
}

pub(crate) fn display_path_nonempty(path: &Path) -> String {
    let rendered = path.display().to_string();
    if rendered.is_empty() {
        ".".to_string()
    } else {
        rendered
    }
}

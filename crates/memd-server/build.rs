use std::process::Command;

fn main() {
    // Re-run whenever HEAD moves or the working tree is checked out.
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/index");

    let branch = git(&["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_else(|| "unknown".into());
    let commit = git(&["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "unknown".into());
    let dirty = match git(&["status", "--porcelain"]) {
        Some(s) if !s.is_empty() => "dirty",
        Some(_) => "clean",
        None => "unknown",
    };

    println!("cargo:rustc-env=MEMD_GIT_BRANCH={branch}");
    println!("cargo:rustc-env=MEMD_GIT_COMMIT={commit}");
    println!("cargo:rustc-env=MEMD_GIT_DIRTY={dirty}");
}

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

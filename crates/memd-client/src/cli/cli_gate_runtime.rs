use std::io::Read;
use memd_core::contract::{CONTRACT_FILE_NAME, FileLayoutSchema, MemdContract};
use memd_core::enforcement::{
    EnforcementPolicy, FreshReadIndex, format_gate_output, gate_write_decision,
    load_latest_sealed_paths,
};

use crate::cli::args::HookGateArgs;

/// Testable core: compute the gate decision and return the rendered output
/// (or None for Allow). Does not touch stdout.
pub(crate) async fn run_gate(args: &HookGateArgs) -> anyhow::Result<Option<String>> {
    let payload_raw = if let Some(c) = &args.content {
        c.clone()
    } else if args.stdin {
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        s
    } else {
        return Ok(None);
    };
    let v: serde_json::Value = serde_json::from_str(&payload_raw)?;
    let tool = v.get("tool_name").and_then(|s| s.as_str()).unwrap_or("");
    // Only gate Edit/Write/NotebookEdit. Read passes through unchanged.
    if !matches!(tool, "Edit" | "Write" | "NotebookEdit") { return Ok(None); }
    let path = v.pointer("/tool_input/file_path")
        .and_then(|s| s.as_str())
        .or_else(|| v.pointer("/tool_input/notebook_path").and_then(|s| s.as_str()))
        .unwrap_or("");
    if path.is_empty() { return Ok(None); }
    let session_id = args.session_id.clone()
        .or_else(|| v.get("session_id").and_then(|s| s.as_str().map(String::from)))
        .unwrap_or_else(|| "unknown".into());
    let policy = resolve_policy(args, &args.output);
    let sealed = load_latest_sealed_paths(&args.output);
    let fresh = FreshReadIndex::for_session(&args.output, &session_id);
    let schema = load_file_layout_schema(&args.output);
    // Normalize path to repo-relative (strip leading $CWD if present). The
    // contract schema uses "docs/..." style, so absolute host paths would
    // otherwise fall through as Unmanaged.
    let rel = normalize_to_repo_rel(path, &args.output);
    let decision = gate_write_decision(
        policy,
        &rel,
        &sealed,
        fresh.paths(),
        &schema,
    );
    // Mark the file-layout gate as wired so `memd contract verify` can
    // surface the guarantee as green-on-evidence.
    let _ = std::fs::create_dir_all(args.output.join("state"));
    let _ = std::fs::write(
        args.output.join("state/file-layout-gate.green"),
        b"wired\n",
    );
    Ok(format_gate_output(decision))
}

fn load_file_layout_schema(output: &std::path::Path) -> FileLayoutSchema {
    let path = output.join(CONTRACT_FILE_NAME);
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return FileLayoutSchema::default();
    };
    let Ok(c) = serde_json::from_str::<MemdContract>(&raw) else {
        return FileLayoutSchema::default();
    };
    c.file_layout
}

/// Given an absolute file_path and the bundle output dir, return a path
/// relative to the repo root (the parent of the output dir). Falls back to
/// the input unchanged if normalization fails.
fn normalize_to_repo_rel(path: &str, output: &std::path::Path) -> String {
    let Some(repo_root) = output.parent() else {
        return path.to_string();
    };
    let p = std::path::Path::new(path);
    if let Ok(stripped) = p.strip_prefix(repo_root) {
        return stripped.to_string_lossy().to_string();
    }
    path.to_string()
}

/// CLI wrapper: call run_gate, print the Some(s) result.
pub(crate) async fn run_gate_cli(args: &HookGateArgs) -> anyhow::Result<()> {
    if let Some(s) = run_gate(args).await? {
        println!("{s}");
    }
    Ok(())
}

fn resolve_policy(args: &HookGateArgs, output: &std::path::Path) -> EnforcementPolicy {
    if let Some(s) = &args.policy {
        return match s.as_str() {
            "off" => EnforcementPolicy::Off,
            "warn" => EnforcementPolicy::Warn,
            "block" => EnforcementPolicy::Block,
            _ => EnforcementPolicy::default(),
        };
    }
    let cfg = output.join("config.json");
    if let Ok(bytes) = std::fs::read(&cfg) {
        if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            if let Some(s) = v.pointer("/continuity/enforcement").and_then(|s| s.as_str()) {
                return match s {
                    "off" => EnforcementPolicy::Off,
                    "warn" => EnforcementPolicy::Warn,
                    "block" => EnforcementPolicy::Block,
                    _ => EnforcementPolicy::default(),
                };
            }
        }
    }
    EnforcementPolicy::default()
}

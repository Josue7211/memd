use std::io::Read;
use memd_core::enforcement::{
    EnforcementPolicy, FreshReadIndex, format_gate_output, gate_decision,
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
    let decision = gate_decision(policy, path, &sealed, fresh.paths());
    Ok(format_gate_output(decision))
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

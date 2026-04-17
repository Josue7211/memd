//! A3-D5 live memory contract CLI runtime.
//!
//! Two subcommands:
//! - `memd contract verify` — check the bundle against `.memd/contract.json`.
//! - `memd contract generate` — write the default contract shape to disk.

use std::path::Path;

use anyhow::Context;
use memd_core::contract::{
    CONTRACT_FILE_NAME, ContractEvidence, MemdContract, verify_contract,
};

use crate::cli::args::{ContractGenerateArgs, ContractVerifyArgs};
use crate::runtime::collect_files_touched;

pub fn run_contract_verify(args: &ContractVerifyArgs) -> anyhow::Result<()> {
    let contract = load_contract(&args.output)?;
    let sealed = any_sealed_ledger_exists(&args.output);
    let files = collect_files_touched(&args.output);
    let evidence = ContractEvidence {
        sealed_ledger_exists: sealed,
        files_touched: &files,
        live_ledger_exists: false,
        sealed_dir_empty: false,
        enforcement_policy_configured: false,
        enforcement_hook_wired: false,
        preference_recall_on_cold_boot_green: None,
    };
    let violations = verify_contract(&contract, &evidence);

    if args.json {
        println!("{}", serde_json::to_string_pretty(&violations)?);
    } else if violations.is_empty() {
        println!(
            "contract verify ok — version {} (sealed_ledger={} files_touched={})",
            contract.version,
            sealed,
            files.len()
        );
    } else {
        println!("contract verify FAILED — {} violation(s):", violations.len());
        for v in &violations {
            println!("  - {}: {}", v.guarantee, v.detail);
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        anyhow::bail!("{} contract violation(s)", violations.len());
    }
}

pub fn run_contract_generate(args: &ContractGenerateArgs) -> anyhow::Result<()> {
    let contract = MemdContract::default();
    let path = args.output.join(CONTRACT_FILE_NAME);
    if path.exists() && !args.force {
        anyhow::bail!("{} already exists — pass --force to overwrite", path.display());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(&contract)?;
    std::fs::write(&path, json).with_context(|| format!("write {}", path.display()))?;
    println!("wrote {}", path.display());
    Ok(())
}

fn load_contract(output: &Path) -> anyhow::Result<MemdContract> {
    let path = output.join(CONTRACT_FILE_NAME);
    if !path.exists() {
        // Missing contract is a green verify against the current default —
        // keep the ergonomics friendly (no hard error), but note it.
        eprintln!(
            "note: {} missing — using built-in default contract (run `memd contract generate` to persist)",
            path.display()
        );
        return Ok(MemdContract::default());
    }
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let contract: MemdContract =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    Ok(contract)
}

fn any_sealed_ledger_exists(output: &Path) -> bool {
    let state = output.join("state");
    let Ok(rd) = std::fs::read_dir(&state) else {
        return false;
    };
    for entry in rd.flatten() {
        if !entry.file_name().to_string_lossy().starts_with("session-") {
            continue;
        }
        let sealed = entry.path().join("sealed");
        if let Ok(sd) = std::fs::read_dir(&sealed) {
            if sd.flatten().any(|f| f.path().extension().is_some_and(|e| e == "json")) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use memd_core::file_ledger::{FileOp, append_file_interaction, seal_session_ledger};

    #[test]
    fn sealed_ledger_detector_sees_sealed_file() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path();
        assert!(!any_sealed_ledger_exists(output));
        let payload = serde_json::json!({
            "session_id": "sess-x",
            "tool_name": "Read",
            "tool_input": { "file_path": "a.rs" },
        });
        append_file_interaction(&payload, None, output, 1).unwrap();
        seal_session_ledger("sess-x", output).unwrap();
        assert!(any_sealed_ledger_exists(output));
    }
}

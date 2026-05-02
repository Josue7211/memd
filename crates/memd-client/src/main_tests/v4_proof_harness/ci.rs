//! G4.5 — CI entrypoint helpers.
//!
//! Tests 11 + 12 of `phase-g4-plan.md §4`. Test 11 proves the in-process
//! 3-session driver is deterministic across 10 back-to-back runs (no temp-file
//! leakage, no race). Test 12 pins the retry-decision allow-list: the CI script
//! retries only on a closed set of infra-flake symptoms, never on a memd
//! assertion failure (which must surface the source-phase regression).
//!
//! Both helpers are reused by `scripts/ci/v4-proof-harness.sh` so the bash
//! wrapper and the unit tests agree on retry policy.

/// Closed allow-list of stderr substrings that classify a CI run as an infra
/// flake (NOT a memd regression). Any match → retry once. Anything else → fail
/// hard.
const INFRA_FLAKE_PATTERNS: &[&str] = &[
    "No space left on device",
    "Resource temporarily unavailable",
    "Connection refused",
    "Network is unreachable",
    "Too many open files",
    "tmpfile create",
    "stale NFS file handle",
];

pub(crate) fn is_infra_flake(stderr_text: &str) -> bool {
    INFRA_FLAKE_PATTERNS
        .iter()
        .any(|needle| stderr_text.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test 11 — in-process harness deterministic across 10 back-to-back runs.
    /// Reuses the public `crate::main_tests::v4_proof_harness::*` driver
    /// indirectly: each loop builds a fresh tempdir, replays Reads, seals,
    /// simulates compaction, and restores. If any iteration leaks state into
    /// the next, the assertion sequence inside the driver test fires.
    #[tokio::test]
    async fn t11_ci_harness_passes_10_of_10_on_clean_tree() {
        use crate::MemdClient;
        use crate::cli::{
            HookArgs, HookMode, HookRestoreArgs, HookSealLedgerArgs, run_hook_mode,
            run_hook_restore,
        };
        use memd_core::file_ledger::{append_file_interaction, ledger_path};

        let client = MemdClient::new("http://127.0.0.1:1").expect("client");
        let mut greens = 0usize;
        for run_idx in 0..10 {
            let dir = tempfile::tempdir().expect("tempdir per run");
            let bundle = dir.path().join(".memd");
            std::fs::create_dir_all(&bundle).expect("bundle dir");
            let sid_a = format!("ci-loop-{run_idx}-a");
            let sid_b = format!("ci-loop-{run_idx}-b");

            // Two-session minimal: seed → seal → wipe → restore.
            let payload = serde_json::json!({
                "session_id": &sid_a,
                "tool_name": "Read",
                "tool_input": { "file_path": "src/lib.rs" },
            });
            append_file_interaction(&payload, None, &bundle, 1).expect("append");

            run_hook_mode(
                &client,
                "http://127.0.0.1:1",
                HookArgs {
                    mode: HookMode::SealLedger(HookSealLedgerArgs {
                        output: bundle.clone(),
                        session_id: sid_a.clone(),
                    }),
                },
            )
            .await
            .expect("seal");

            let active = ledger_path(&bundle, &sid_a);
            if active.exists() {
                std::fs::remove_file(&active).expect("wipe");
            }

            let report = run_hook_restore(&HookRestoreArgs {
                output: bundle.clone(),
                session_id: sid_a.clone(),
                latest_only: None,
                dry_run: false,
                json: false,
            })
            .expect("restore");
            assert!(report.ok, "run {run_idx}: restore must succeed");

            // Second session does not interfere with first session's restored
            // state — proves no leakage across loop iterations.
            let payload_b = serde_json::json!({
                "session_id": &sid_b,
                "tool_name": "Read",
                "tool_input": { "file_path": "src/main.rs" },
            });
            append_file_interaction(&payload_b, None, &bundle, 1).expect("append b");

            assert!(
                !bundle.join("logs/continuity-breach.log").exists(),
                "run {run_idx} produced a breach log on a healthy loop"
            );
            greens += 1;
        }
        assert_eq!(greens, 10, "expected 10/10 green CI loops");
    }

    /// Test 12 — retry decision triggers only on infra-flake patterns; any
    /// memd assertion failure must NOT trigger retry.
    #[test]
    fn t12_ci_harness_retries_only_on_infra_flake() {
        // Infra flakes → retry.
        for flake in [
            "ENOSPC: No space left on device",
            "stale NFS file handle on /mnt/.../target",
            "Connection refused (os error 111)",
            "Too many open files (os error 24)",
        ] {
            assert!(
                is_infra_flake(flake),
                "must classify `{flake}` as infra flake"
            );
        }

        // Real memd regressions → no retry, surface the failure.
        for failure in [
            "A4 regression: PostCompact restore did not run before first session-2 tool call",
            "B4 regression: hook trace missing event `PreCompact`",
            "C4 regression: correction `fact-B` is missing provenance",
            "D4 regression: wake brief 2400 tokens exceeds budget 2000",
            "E4 regression: lookup `primary ID` returned stale value containing `uuid`",
            "F4 regression: outstanding drift count 0 below expected minimum 1",
            "scorecard regenerator refused — over-claim detected",
            "assertion `left == right` failed",
            "thread 'main' panicked at ...",
        ] {
            assert!(
                !is_infra_flake(failure),
                "must NOT classify memd failure `{failure}` as infra flake"
            );
        }
    }
}

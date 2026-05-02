//! G4 — V4 proof harness driver.
//!
//! Tests 1 + 2 of `phase-g4-plan.md §4`. Test 1 parses the G4.1 fixtures into
//! a typed `Scenario`. Test 2 replays the three sessions against an in-process
//! memd state under a tempdir, simulating PreCompact between sessions and
//! invoking PostCompact restore on session-2 wake. Phase-semantic assertions
//! live in G4.3 (`assertions.rs`); this module only proves the driver can move
//! through the scenario and produce on-disk artifacts.

mod assertions;
mod ci;
mod scorecard;

use super::*;
use crate::MemdClient;
use crate::cli::{
    HookArgs, HookMode, HookRestoreArgs, HookSealLedgerArgs, run_hook_mode, run_hook_restore,
};
use memd_core::file_ledger::{append_file_interaction, ledger_path, restore::RestoreSource};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const FIXTURE_ROOT: &str = "fixtures/g4";

#[derive(Debug, serde::Deserialize)]
struct SeedState {
    workspace_id: String,
    agent_id: String,
    namespace: String,
    project: String,
    bundle_root_relative: String,
    #[serde(default)]
    memd_env: BTreeMap<String, String>,
}

#[derive(Debug, serde::Deserialize)]
struct ScriptRecord {
    turn: String,
    role: String,
    #[serde(default)]
    event: Option<String>,
    #[serde(default)]
    tool: Option<String>,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    memd: Option<String>,
    #[serde(flatten)]
    _rest: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug)]
struct Scenario {
    seed: SeedState,
    sessions: Vec<Session>,
}

#[derive(Debug)]
struct Session {
    id: String,
    records: Vec<ScriptRecord>,
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_ROOT)
}

fn load_scenario() -> Scenario {
    let root = fixture_dir();
    let seed_text =
        std::fs::read_to_string(root.join("seed-state.json")).expect("read g4 seed-state.json");
    let seed: SeedState =
        serde_json::from_str(&seed_text).expect("seed-state.json matches SeedState");

    let sessions = ["session-1", "session-2", "session-3"]
        .iter()
        .map(|name| {
            let text = std::fs::read_to_string(root.join(format!("{name}.jsonl")))
                .unwrap_or_else(|err| panic!("read {name}.jsonl: {err}"));
            let records: Vec<ScriptRecord> = text
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(|l| {
                    serde_json::from_str(l)
                        .unwrap_or_else(|err| panic!("parse {name} record `{l}`: {err}"))
                })
                .collect();
            Session {
                id: (*name).into(),
                records,
            }
        })
        .collect();

    Scenario { seed, sessions }
}

fn dummy_client() -> MemdClient {
    MemdClient::new("http://127.0.0.1:1").expect("build memd client for proof harness")
}

async fn seal(output: &Path, sid: &str) {
    let args = HookArgs {
        mode: HookMode::SealLedger(HookSealLedgerArgs {
            output: output.to_path_buf(),
            session_id: sid.to_string(),
        }),
    };
    run_hook_mode(&dummy_client(), "http://127.0.0.1:1", args)
        .await
        .expect("seal-ledger hook in proof harness");
}

fn restore(output: &Path, sid: &str) {
    let args = HookRestoreArgs {
        output: output.to_path_buf(),
        session_id: sid.to_string(),
        latest_only: None,
        dry_run: false,
        json: false,
    };
    let report = run_hook_restore(&args).expect("restore hook in proof harness");
    assert!(report.ok, "PostCompact restore must succeed at {sid}");
    assert_eq!(report.source, RestoreSource::Postcompact);
}

fn replay_reads(output: &Path, sid: &str, records: &[ScriptRecord]) {
    let mut seq: i64 = 1;
    for rec in records {
        if rec.role != "assistant" {
            continue;
        }
        let Some(tool) = rec.tool.as_deref() else {
            continue;
        };
        if tool != "Read" {
            continue;
        }
        let Some(path) = rec.path.as_deref() else {
            continue;
        };
        let payload = serde_json::json!({
            "session_id": sid,
            "tool_name": "Read",
            "tool_input": { "file_path": path },
        });
        append_file_interaction(&payload, None, output, seq).expect("append file interaction");
        seq += 1;
    }
}

#[test]
fn harness_parses_fixture_script() {
    let scenario = load_scenario();
    assert_eq!(scenario.seed.workspace_id, "v4-dogfood");
    assert_eq!(scenario.seed.agent_id, "g4-runner");
    assert_eq!(scenario.seed.namespace, "main");
    assert_eq!(scenario.seed.project, "memd-v4-proof");
    assert_eq!(scenario.seed.bundle_root_relative, ".memd");

    // Env block must include both default-off flags so a fresh per-run bundle
    // exercises A4 restore + C4 correction-detect.
    assert_eq!(
        scenario
            .seed
            .memd_env
            .get("MEMD_A4_LEDGER_SURVIVAL")
            .map(String::as_str),
        Some("1"),
        "seed-state must enable A4 ledger survival"
    );
    assert_eq!(
        scenario
            .seed
            .memd_env
            .get("MEMD_C4_CORRECTION_DETECT")
            .map(String::as_str),
        Some("1"),
        "seed-state must enable C4 correction detection"
    );

    assert_eq!(scenario.sessions.len(), 3, "3-session scenario");
    let ids: Vec<&str> = scenario.sessions.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids, vec!["session-1", "session-2", "session-3"]);

    for session in &scenario.sessions {
        assert!(
            !session.records.is_empty(),
            "{} must contain script records",
            session.id
        );
        for rec in &session.records {
            assert!(
                !rec.turn.is_empty(),
                "every record needs a turn label: {:?}",
                rec
            );
            // Allowed roles match what G4.1 fixtures emit. Anything else is a
            // shape regression the asserter should never have to handle.
            let role_ok = matches!(rec.role.as_str(), "user" | "assistant" | "system");
            assert!(role_ok, "unexpected role `{}` at {}", rec.role, rec.turn);
        }
    }

    // Session-2 + session-3 must each begin with a SessionStart event so
    // PostCompact restore + wake compilation have a deterministic anchor.
    for sess in &scenario.sessions[1..] {
        let first = &sess.records[0];
        assert_eq!(
            first.role, "system",
            "{} must open with system event",
            sess.id
        );
        assert_eq!(
            first.event.as_deref(),
            Some("SessionStart"),
            "{} must open with SessionStart event",
            sess.id
        );
    }

    // At least one memd CLI invocation in session-2 (lookup) and session-3
    // (preference confirm) — proves the script exercises live verbs, not
    // only chat turns.
    let s2_has_memd = scenario.sessions[1]
        .records
        .iter()
        .any(|r| r.memd.is_some());
    let s3_has_memd = scenario.sessions[2]
        .records
        .iter()
        .any(|r| r.memd.is_some());
    assert!(s2_has_memd, "session-2 must invoke at least one memd verb");
    assert!(s3_has_memd, "session-3 must invoke at least one memd verb");
}

#[tokio::test]
async fn harness_runs_3_sessions_in_sequence_with_simulated_compaction() {
    let scenario = load_scenario();
    let dir = tempfile::tempdir().expect("tempdir for proof harness");
    let bundle = dir.path().join(&scenario.seed.bundle_root_relative);
    std::fs::create_dir_all(&bundle).expect("create bundle root");

    // Session 1: replay reads, then PreCompact seal.
    let s1 = &scenario.sessions[0];
    replay_reads(&bundle, &s1.id, &s1.records);
    seal(&bundle, &s1.id).await;

    // Simulate compaction wiping session-1's active ledger.
    let s1_ledger = ledger_path(&bundle, &s1.id);
    if s1_ledger.exists() {
        std::fs::remove_file(&s1_ledger).expect("simulate s1 ledger wipe");
    }
    assert!(
        !s1_ledger.exists(),
        "session-1 active ledger must be gone after simulated compaction"
    );

    // Session 2: PostCompact restore against session-1 sealed state, then
    // replay reads, then seal.
    let s2 = &scenario.sessions[1];
    restore(&bundle, &s1.id);
    assert!(
        s1_ledger.exists(),
        "PostCompact restore must rehydrate session-1 ledger before session-2 work"
    );
    replay_reads(&bundle, &s2.id, &s2.records);
    seal(&bundle, &s2.id).await;

    let s2_ledger = ledger_path(&bundle, &s2.id);
    if s2_ledger.exists() {
        std::fs::remove_file(&s2_ledger).expect("simulate s2 ledger wipe");
    }

    // Session 3: PostCompact restore against session-2, replay, seal.
    let s3 = &scenario.sessions[2];
    restore(&bundle, &s2.id);
    replay_reads(&bundle, &s3.id, &s3.records);
    seal(&bundle, &s3.id).await;

    // Driver-level invariant only: each session left a sealed-ledger artifact
    // somewhere under the bundle. G4.3 will read these to score axes.
    let sealed_root = bundle.join("state");
    assert!(
        sealed_root.exists(),
        "bundle/state must exist after a 3-session run"
    );
    let any_sealed = walkdir(&sealed_root)
        .into_iter()
        .any(|p| p.to_string_lossy().contains("sealed"));
    assert!(
        any_sealed,
        "expected at least one sealed-ledger artifact under {}",
        sealed_root.display()
    );

    // Continuity-breach log must not exist on a healthy 3-session run.
    let breach_log = bundle.join("logs/continuity-breach.log");
    assert!(
        !breach_log.exists(),
        "healthy 3-session run must not write continuity-breach.log"
    );
}

fn walkdir(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(read) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in read.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                out.push(path);
            }
        }
    }
    out
}

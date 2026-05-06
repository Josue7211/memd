use super::*;
use memd_core::self_tuning::CompilerMode;

fn write_v15_bundle_config(output: &Path, compiler_mode: &str) {
    std::fs::create_dir_all(output).unwrap();
    std::fs::write(
        output.join("config.json"),
        format!(
            r#"{{
  "schema_version": 2,
  "project": "memd-test",
  "namespace": "main",
  "agent": "codex",
  "session": "session-test",
  "base_url": "http://127.0.0.1:8787",
  "telemetry": {{
    "enabled": true,
    "retention_days": 30,
    "export_scope": "local"
  }},
  "compiler": {{
    "mode": "{compiler_mode}",
    "self_tuning": {{
      "min_samples": 3,
      "min_quality_score": 0.90,
      "max_quality_regression": 0.0,
      "max_budget_regression_pct": 0.0
    }}
  }}
}}
"#
        ),
    )
    .unwrap();
}

fn record_quality_event(output: &Path, user: &str, harness: &str, tokens: u64, quality: f64) {
    record_telemetry_event(
        output,
        &TelemetryRecordArgs {
            user: Some(user.to_string()),
            harness: Some(harness.to_string()),
            source: "v15-test".to_string(),
            event_kind: "compiler_usage".to_string(),
            tokens,
            cost_usd: 0.0,
            session_id: None,
            model_family: Some("gpt-5.4".to_string()),
            metadata_json: Some(format!(
                r#"{{"quality_score":{quality},"baseline_quality_score":0.92,"budget_target":1500}}"#
            )),
            force: false,
        },
    )
    .unwrap();
}

#[test]
fn self_tuning_v15_builds_three_guarded_profiles_from_v14_telemetry() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path();
    write_v15_bundle_config(output, "self_tuning");

    for (user, harness, base_tokens) in [
        ("alice@example.com", "codex", 900),
        ("bob@example.com", "claude", 930),
        ("carol@example.com", "gemini", 960),
    ] {
        record_quality_event(output, user, harness, base_tokens, 0.94);
        record_quality_event(output, user, harness, base_tokens + 25, 0.95);
        record_quality_event(output, user, harness, base_tokens + 50, 0.94);
    }

    let report = tune_self_tuning_profiles(
        output,
        &CompilerTuneArgs {
            baseline_budget: 1500,
            min_samples: 3,
            min_quality_score: 0.90,
            tuning_headroom: 1.10,
            json: true,
        },
    )
    .unwrap();

    assert_eq!(report.profile_count, 3);
    assert_eq!(report.accepted_count, 3);
    assert!(report.min_token_savings_pct >= 20.0);
    assert!(report.min_quality_delta >= 0.0);
    let profiles = read_self_tuning_profiles(output).unwrap();
    assert_eq!(profiles.len(), 3);
    assert!(profiles.iter().all(|profile| profile.accepted));

    let bench = build_self_tuning_ab_bench(output, 4000, 1500).unwrap();
    assert_eq!(bench.len(), 3);
    assert!(
        bench
            .iter()
            .all(|row| row.accepted && row.token_savings_vs_dynamic_pct >= 20.0)
    );
}

#[test]
fn self_tuning_v15_honors_manual_compiler_mode_override() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path();
    write_v15_bundle_config(output, "static");

    assert_eq!(compiler_mode_from_config(output), CompilerMode::Static);
}

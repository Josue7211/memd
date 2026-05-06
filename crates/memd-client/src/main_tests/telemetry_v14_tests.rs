use super::*;
use serde_json::Value as JsonValue;

fn write_min_bundle_config(output: &Path) {
    std::fs::create_dir_all(output).unwrap();
    std::fs::write(
        output.join("config.json"),
        r#"{
  "schema_version": 2,
  "project": "memd-test",
  "namespace": "main",
  "agent": "codex",
  "session": "session-test",
  "base_url": "http://127.0.0.1:8787",
  "telemetry": {
    "enabled": false,
    "retention_days": 30,
    "export_scope": "local"
  }
}
"#,
    )
    .unwrap();
}

#[test]
fn telemetry_enable_report_export_round_trip() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path();
    write_min_bundle_config(output);

    run_v14_telemetry_command(output, false, &TelemetryCommand::Enable).unwrap();
    let status = telemetry_status(output).unwrap();
    assert!(status.enabled);

    record_telemetry_event(
        output,
        &TelemetryRecordArgs {
            user: Some("alice@example.com".to_string()),
            harness: Some("codex".to_string()),
            source: "unit-test".to_string(),
            event_kind: "usage".to_string(),
            tokens: 1200,
            cost_usd: 0.004,
            session_id: Some("session-alice".to_string()),
            model_family: Some("gpt-5.4".to_string()),
            metadata_json: Some(
                r#"{"note":"email bob@example.com path /Users/alice/x"}"#.to_string(),
            ),
            force: false,
        },
    )
    .unwrap();

    let report = build_telemetry_usage_report(
        output,
        &TelemetryReportArgs {
            window: "30d".to_string(),
            json: true,
        },
    )
    .unwrap();
    assert_eq!(report.event_count, 1);
    assert_eq!(report.total_tokens, 1200);
    assert_eq!(report.users.len(), 1);
    let user = report.users.values().next().unwrap();
    assert_eq!(user.harnesses["codex"].event_count, 1);

    let export_path = export_telemetry(
        output,
        &TelemetryExportArgs {
            output_file: None,
            scope: "bench".to_string(),
            window: "30d".to_string(),
        },
    )
    .unwrap();
    let exported = std::fs::read_to_string(export_path).unwrap();
    assert!(!exported.contains("alice@example.com"));
    assert!(!exported.contains("/Users/alice"));
    assert!(!exported.contains("session-alice"));
    assert!(exported.contains("[redacted-email]"));
}

#[test]
fn disabled_telemetry_blocks_cost_ledger_backend() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path();
    write_min_bundle_config(output);

    crate::runtime::resume::compiler::ledger::write_cost_line(
        output,
        Some("session-x"),
        800,
        1200,
        "gpt-5.4",
    )
    .unwrap();
    assert!(
        !memd_core::telemetry::telemetry_events_path(output).exists(),
        "disabled telemetry must not create orphaned telemetry state"
    );

    run_v14_telemetry_command(output, false, &TelemetryCommand::Enable).unwrap();
    crate::runtime::resume::compiler::ledger::write_cost_line(
        output,
        Some("session-x"),
        800,
        1200,
        "gpt-5.4",
    )
    .unwrap();
    let body =
        std::fs::read_to_string(memd_core::telemetry::telemetry_events_path(output)).unwrap();
    let line: JsonValue = serde_json::from_str(body.lines().next().unwrap()).unwrap();
    assert_eq!(line["event_kind"], "wake_cost");
    assert_eq!(line["token_count"], 800);
}

mod tests {
    use super::*;

    #[test]
    fn live_state_rejects_unapproved_private_messages_payload() {
        let root = tempfile::tempdir().expect("tempdir");
        let args = LiveStateIngestArgs {
            output: root.path().join(".memd"),
            source: "clawcontrol".to_string(),
            module: "messages".to_string(),
            scope: "inbox".to_string(),
            visibility: "private".to_string(),
            privacy: "redacted".to_string(),
            approved: false,
            agentsecrets_approved: false,
            freshness_secs: 300,
            label: vec![],
            summary: "latest message from approved contact".to_string(),
            payload_json: Some(r#"{"latest":"redacted"}"#.to_string()),
            payload_file: None,
            json: false,
        };
        let err = ingest_live_state(&args).expect_err("must reject");
        assert!(err.to_string().contains("requires --approved"));
    }

    #[test]
    fn live_state_allows_messages_metadata_without_full_chat_access() {
        let root = tempfile::tempdir().expect("tempdir");
        let args = LiveStateIngestArgs {
            output: root.path().join(".memd"),
            source: "clawcontrol".to_string(),
            module: "messages".to_string(),
            scope: "idea-context".to_string(),
            visibility: "private".to_string(),
            privacy: "metadata".to_string(),
            approved: false,
            agentsecrets_approved: false,
            freshness_secs: 300,
            label: vec!["messages".to_string()],
            summary: "approved text metadata says the user is discussing a launch idea".to_string(),
            payload_json: Some(r#"{"topic":"launch idea","contact":"approved"}"#.to_string()),
            payload_file: None,
            json: false,
        };
        let report = ingest_live_state(&args).expect("metadata ingest");
        assert_eq!(report.total, 1);
        assert_eq!(report.records[0].privacy, "metadata");
    }

    #[test]
    fn live_state_requirement_ignores_unapproved_messages_for_required_freshness() {
        let root = tempfile::tempdir().expect("tempdir");
        let output = root.path().join(".memd");
        let args = LiveStateIngestArgs {
            output: output.clone(),
            source: "clawcontrol".to_string(),
            module: "messages".to_string(),
            scope: "current".to_string(),
            visibility: "private".to_string(),
            privacy: "metadata".to_string(),
            approved: false,
            agentsecrets_approved: false,
            freshness_secs: 300,
            label: vec!["messages".to_string()],
            summary: "unapproved message metadata fixture".to_string(),
            payload_json: Some(
                r#"{"mode":"metadata-only","threads":[],"raw_media_stored":false}"#.to_string(),
            ),
            payload_file: None,
            json: false,
        };

        let report = ingest_live_state(&args).expect("metadata ingest");
        let messages_requirement = report
            .requirements
            .iter()
            .find(|requirement| requirement.module == "messages")
            .expect("messages requirement");
        assert_eq!(report.total, 1);
        assert_eq!(report.fresh, 1);
        assert_eq!(messages_requirement.status, "missing");
        assert_eq!(messages_requirement.matched_scope, None);
        assert!(report.sync_required);
        assert!(
            report
                .sync_actions
                .iter()
                .any(|action| action.contains("approved_communications:messages status=missing"))
        );
    }

    #[test]
    fn live_state_rejects_public_personal_calendar_state() {
        let root = tempfile::tempdir().expect("tempdir");
        let args = LiveStateIngestArgs {
            output: root.path().join(".memd"),
            source: "clawcontrol".to_string(),
            module: "calendar".to_string(),
            scope: "primary".to_string(),
            visibility: "public".to_string(),
            privacy: "public".to_string(),
            approved: true,
            agentsecrets_approved: false,
            freshness_secs: 300,
            label: vec!["calendar".to_string()],
            summary: "next event Dentist".to_string(),
            payload_json: Some(r#"{"events":[{"title":"Dentist"}]}"#.to_string()),
            payload_file: None,
            json: false,
        };
        let err = ingest_live_state(&args).expect_err("must reject");
        assert!(err.to_string().contains("must use --visibility private"));
    }

    #[test]
    fn live_state_rejects_message_media_without_agentsecrets_approval() {
        let root = tempfile::tempdir().expect("tempdir");
        let args = LiveStateIngestArgs {
            output: root.path().join(".memd"),
            source: "clawcontrol".to_string(),
            module: "messages".to_string(),
            scope: "attachments".to_string(),
            visibility: "private".to_string(),
            privacy: "metadata".to_string(),
            approved: false,
            agentsecrets_approved: false,
            freshness_secs: 300,
            label: vec!["image".to_string()],
            summary: "thread contains an image attachment".to_string(),
            payload_json: Some(r#"{"attachments":[{"type":"image"}]}"#.to_string()),
            payload_file: None,
            json: false,
        };
        let err = ingest_live_state(&args).expect_err("must reject");
        assert!(err.to_string().contains("--agentsecrets-approved"));
    }

    #[test]
    fn live_state_rejects_raw_message_media_even_with_agentsecrets_approval() {
        let root = tempfile::tempdir().expect("tempdir");
        let args = LiveStateIngestArgs {
            output: root.path().join(".memd"),
            source: "clawcontrol".to_string(),
            module: "messages".to_string(),
            scope: "attachments".to_string(),
            visibility: "private".to_string(),
            privacy: "metadata".to_string(),
            approved: false,
            agentsecrets_approved: true,
            freshness_secs: 300,
            label: vec!["image".to_string()],
            summary: "thread contains an image attachment".to_string(),
            payload_json: Some(r#"{"data_url":"data:image/png;base64,AAAA"}"#.to_string()),
            payload_file: None,
            json: false,
        };
        let err = ingest_live_state(&args).expect_err("must reject");
        assert!(err.to_string().contains("must stay behind AgentSecrets"));
    }

    #[test]
    fn live_state_ingests_calendar_and_renders_context_section() {
        let root = tempfile::tempdir().expect("tempdir");
        let output = root.path().join(".memd");
        let args = LiveStateIngestArgs {
            output: output.clone(),
            source: "ClawControl".to_string(),
            module: "Calendar".to_string(),
            scope: "primary".to_string(),
            visibility: "private".to_string(),
            privacy: "approved".to_string(),
            approved: true,
            agentsecrets_approved: false,
            freshness_secs: 300,
            label: vec!["calendar".to_string()],
            summary: "next event Dentist at 2026-05-17T14:00:00Z".to_string(),
            payload_json: Some(r#"{"events":[{"title":"Dentist"}]}"#.to_string()),
            payload_file: None,
            json: false,
        };
        let report = ingest_live_state(&args).expect("ingest");
        assert_eq!(report.total, 1);
        assert_eq!(report.fresh, 1);

        let section = render_live_app_state_section(&output, 4);
        assert!(section.contains("memd:calendar"));
        assert!(section.contains("required:memd:calendar"));
        assert!(section.contains("status=fresh"));
        assert!(section.contains("required:approved_communications:messages"));
        assert!(section.contains("no unrestricted chat access"));
        assert!(section.contains("privacy=approved"));
        assert!(section.contains("Dentist"));
    }

    #[test]
    fn live_state_requirement_report_accepts_clawcontrol_current_scope() {
        let root = tempfile::tempdir().expect("tempdir");
        let output = root.path().join(".memd");
        let args = LiveStateIngestArgs {
            output: output.clone(),
            source: "clawcontrol".to_string(),
            module: "calendar".to_string(),
            scope: "current".to_string(),
            visibility: "private".to_string(),
            privacy: "approved".to_string(),
            approved: true,
            agentsecrets_approved: false,
            freshness_secs: 300,
            label: vec!["calendar".to_string()],
            summary: "calendar: loaded; upcoming_events=1".to_string(),
            payload_json: Some(r#"{"events":[{"title":"Dentist"}]}"#.to_string()),
            payload_file: None,
            json: false,
        };
        let report = ingest_live_state(&args).expect("ingest current-scope calendar");
        let calendar_requirement = report
            .requirements
            .iter()
            .find(|requirement| requirement.module == "calendar")
            .expect("calendar requirement");
        assert_eq!(calendar_requirement.status, "fresh");
        assert_eq!(
            calendar_requirement.matched_scope.as_deref(),
            Some("current")
        );
        assert_eq!(calendar_requirement.canonical_scope, "primary");
        assert_eq!(report.requirement_fresh, 1);

        let summary = render_live_state_summary(&report);
        assert!(summary.contains("requirement_fresh=1"));
        assert!(summary.contains("required:memd:calendar"));
        assert!(summary.contains("matched_scope=current"));
    }

    #[test]
    fn live_state_ingest_batch_accepts_clawcontrol_body_shape() {
        let root = tempfile::tempdir().expect("tempdir");
        let output = root.path().join(".memd");
        let batch = serde_json::json!({
            "records": [
                {
                    "sourceApp": "memd",
                    "module": "visible_page",
                    "scope": "current",
                    "visibility": "private",
                    "privacy": "metadata",
                    "freshnessSecs": 300,
                    "labels": ["live-app-state", "visible_page"],
                    "summary": "visible page: route=/calendar title=Calendar",
                    "payload": {"route": "/calendar", "title": "Calendar", "facts": []}
                },
                {
                    "sourceApp": "memd",
                    "module": "calendar",
                    "scope": "current",
                    "visibility": "private",
                    "privacy": "approved",
                    "approved": true,
                    "freshnessSecs": 300,
                    "labels": ["live-app-state", "calendar"],
                    "summary": "calendar: loaded; upcoming_events=1",
                    "payload": {"events": [{"title": "Dentist"}]}
                },
                {
                    "sourceApp": "memd",
                    "module": "reminders",
                    "scope": "current",
                    "visibility": "private",
                    "privacy": "approved",
                    "approved": true,
                    "freshnessSecs": 300,
                    "labels": ["live-app-state", "reminders"],
                    "summary": "reminders: loaded; open=0 total=0",
                    "payload": {"reminders": []}
                },
                {
                    "sourceApp": "memd",
                    "module": "todos",
                    "scope": "current",
                    "visibility": "private",
                    "privacy": "approved",
                    "approved": true,
                    "freshnessSecs": 300,
                    "labels": ["live-app-state", "todos"],
                    "summary": "todos: loaded; open=0 total=0",
                    "payload": {"todos": []}
                },
                {
                    "sourceApp": "approved_communications",
                    "module": "messages",
                    "scope": "current",
                    "visibility": "private",
                    "privacy": "metadata",
                    "approved": true,
                    "agentsecretsApproved": false,
                    "freshnessSecs": 300,
                    "labels": ["live-app-state", "messages"],
                    "summary": "messages: loaded; conversations=0",
                    "payload": {"summary": "messages: loaded; conversations=0"}
                },
                {
                    "sourceApp": "approved_communications",
                    "module": "email",
                    "scope": "current",
                    "visibility": "private",
                    "privacy": "metadata",
                    "approved": true,
                    "agentsecretsApproved": false,
                    "freshnessSecs": 300,
                    "labels": ["live-app-state", "email"],
                    "summary": "email: loaded; inbox_items=0",
                    "payload": {"summary": "email: loaded; inbox_items=0"}
                }
            ]
        });
        let args = LiveStateIngestBatchArgs {
            output,
            stdin: false,
            input_json: Some(batch.to_string()),
            input_file: None,
            json: false,
        };

        let report = ingest_live_state_batch(&args).expect("batch ingest");
        assert_eq!(report.total, 6);
        assert_eq!(report.status, "fresh");
        assert_eq!(report.requirement_missing, 0);
        assert_eq!(report.requirement_stale, 0);
        assert!(!report.sync_required);
        assert!(report.records.iter().any(|record| {
            record.id == "memd:visible_page:current" && record.summary.contains("route=/calendar")
        }));
    }

    #[test]
    fn live_state_import_copies_composite_bundle_records() {
        let root = tempfile::tempdir().expect("tempdir");
        let source_output = root.path().join("clawcontrol/.memd");
        let dest_output = root.path().join("memd/.memd");
        let template = live_state_report(&dest_output).expect("empty status");
        let batch_args = LiveStateIngestBatchArgs {
            output: source_output.clone(),
            stdin: false,
            input_json: Some(render_live_state_batch_template(&template)),
            input_file: None,
            json: false,
        };
        ingest_live_state_batch(&batch_args).expect("source batch ingest");

        let import_args = LiveStateImportArgs {
            output: dest_output,
            from_output: source_output,
            source: None,
            fresh_only: true,
            json: false,
        };
        let report = import_live_state(&import_args).expect("import live state");

        assert_eq!(report.total, 6);
        assert_eq!(report.status, "fresh");
        assert_eq!(report.requirement_missing, 0);
        assert!(!report.sync_required);
        assert!(
            report
                .records
                .iter()
                .any(|record| record.source_app == "memd")
        );
        assert!(
            report
                .records
                .iter()
                .any(|record| record.source_app == "approved_communications")
        );
    }

    #[test]
    fn live_state_sync_imports_only_when_authority_needs_refresh() {
        let root = tempfile::tempdir().expect("tempdir");
        let source_output = root.path().join("clawcontrol/.memd");
        let dest_output = root.path().join("memd/.memd");
        let template = live_state_report(&dest_output).expect("empty status");
        let batch_args = LiveStateIngestBatchArgs {
            output: source_output.clone(),
            stdin: false,
            input_json: Some(render_live_state_batch_template(&template)),
            input_file: None,
            json: false,
        };
        ingest_live_state_batch(&batch_args).expect("source batch ingest");

        let sync_args = LiveStateSyncArgs {
            output: dest_output.clone(),
            from_output: source_output.clone(),
            source: "all".to_string(),
            due_within_secs: 0,
            allow_stale: false,
            json: false,
        };
        let synced = sync_live_state(&sync_args).expect("sync imports missing authority");
        assert_eq!(synced.status, "fresh");
        assert_eq!(synced.requirement_missing, 0);
        assert_eq!(synced.total, 6);

        let source_path = live_app_state_path(&source_output);
        std::fs::remove_file(source_path).expect("remove source map");
        let no_op = sync_live_state(&sync_args).expect("fresh authority does not need source");
        assert_eq!(no_op.status, "fresh");
        assert_eq!(no_op.total, 6);
    }

    #[test]
    fn live_state_check_can_warn_before_next_refresh() {
        let root = tempfile::tempdir().expect("tempdir");
        let output = root.path().join(".memd");
        let mut report = live_state_report(&output).expect("empty status");
        for (module, scope, approved, payload_json) in [
            (
                "visible_page",
                "current",
                false,
                r#"{"route":"/calendar","title":"Calendar","facts":[]}"#,
            ),
            ("calendar", "primary", false, r#"{"events":[]}"#),
            ("reminders", "default", false, r#"{"reminders":[]}"#),
            ("todos", "default", false, r#"{"todos":[]}"#),
            (
                "messages",
                "approved",
                true,
                r#"{"mode":"metadata-only","threads":[],"raw_media_stored":false}"#,
            ),
            (
                "email",
                "approved",
                true,
                r#"{"mode":"approved-metadata","messages":[],"raw_body_stored":false}"#,
            ),
        ] {
            let args = LiveStateIngestArgs {
                output: output.clone(),
                source: if sensitive_communication_module(module) {
                    "approved_communications".to_string()
                } else {
                    "clawcontrol".to_string()
                },
                module: module.to_string(),
                scope: scope.to_string(),
                visibility: "private".to_string(),
                privacy: "metadata".to_string(),
                approved,
                agentsecrets_approved: false,
                freshness_secs: 60,
                label: vec![module.to_string()],
                summary: format!("{module} fixture fresh"),
                payload_json: Some(payload_json.to_string()),
                payload_file: None,
                json: false,
            };
            report = ingest_live_state(&args).expect("ingest fixture");
        }

        assert_eq!(report.status, "fresh");
        assert_eq!(report.requirement_missing, 0);
        assert_eq!(report.requirement_stale, 0);
        assert!(!report.sync_required);
        assert!(report.next_refresh_at > report.checked_at);
        assert!(!live_state_check_required(&report, 5));
        assert!(live_state_check_required(&report, 120));
    }

    #[test]
    fn live_state_empty_map_renders_required_sync_surface() {
        let root = tempfile::tempdir().expect("tempdir");
        let output = root.path().join(".memd");
        let section = render_live_app_state_section(&output, 8);
        assert!(section.contains("no fresh live app state"));
        assert!(section.contains("refresh_policy contract=1"));
        assert!(section.contains("reason=\"missing required live-state surface"));
        assert!(section.contains("default_ttl=86400s"));
        assert!(section.contains("required:memd:visible_page"));
        assert!(section.contains("required:memd:calendar"));
        assert!(section.contains("required:memd:reminders"));
        assert!(section.contains("required:approved_communications:messages"));
        assert!(section.contains("sync_required=true sync_tasks=6"));
        assert!(section.contains("sync_task:approved_communications:messages"));
        assert!(section.contains("sync_task:approved_communications:messages scope=approved"));
        assert!(section.contains("approved_required=true"));
        assert!(section.contains("media_agentsecrets=true"));
        assert!(section.contains("status=missing"));
        assert!(section.contains("private metadata/redacted; no unrestricted chat access"));

        let report = live_state_report(&output).expect("status report");
        assert_eq!(report.status, "missing_requirements");
        assert_eq!(
            report.requirement_missing,
            LIVE_APP_STATE_REQUIREMENTS.len()
        );
        assert_eq!(report.requirement_fresh, 0);
        assert!(report.sync_required);
        assert_eq!(report.checked_at, report.next_refresh_at);
        assert_eq!(report.producer_contract_version, 1);
        assert!(report.refresh_reason.contains("missing required"));
        assert!(
            report
                .refresh_policy
                .contains("default producer ttl=86400s")
        );
        assert_eq!(report.sync_actions.len(), LIVE_APP_STATE_REQUIREMENTS.len());
        assert_eq!(report.sync_tasks.len(), LIVE_APP_STATE_REQUIREMENTS.len());
        assert!(
            report
                .sync_actions
                .iter()
                .any(|action| action.contains("memd:visible_page status=missing"))
        );
        let messages_task = report
            .sync_tasks
            .iter()
            .find(|task| task.module == "messages")
            .expect("messages sync task");
        assert_eq!(messages_task.required_scope, "approved");
        assert_eq!(messages_task.privacy, "metadata");
        assert!(messages_task.approved_required);
        assert!(messages_task.agentsecrets_required_for_media);
        assert!(
            messages_task
                .ingest_argv
                .iter()
                .any(|arg| arg == "--approved")
        );
        assert!(
            messages_task
                .ingest_argv
                .windows(2)
                .any(|items| items == ["--privacy", "metadata"])
        );
        let summary = render_live_state_summary(&report);
        assert!(summary.contains("sync_required=true"));
        assert!(summary.contains("next_refresh_at="));
        assert!(summary.contains("contract=1"));
        assert!(summary.contains("sync_action:memd:calendar status=missing"));
        assert!(summary.contains("sync_task:approved_communications:messages"));

        let tasks = render_live_state_task_lines(&report);
        assert!(tasks.contains("live_state_tasks sync_required=true count=6"));
        assert!(tasks.contains("task source=memd module=visible_page"));
        assert!(tasks.contains("task source=approved_communications module=messages"));
        assert!(tasks.contains("approved_required=true"));
        assert!(tasks.contains("media_agentsecrets=true"));
        assert!(tasks.contains(approved_communications_access_route_command()));
        assert!(
            tasks.contains(
                "producer_route=\"scripts/live-state-capture-approved-communications.mjs\""
            )
        );
        assert!(tasks.contains(r#"approved_zero_route="APPROVED_COMMUNICATIONS_EMPTY_APPROVED=1 scripts/live-state-capture-approved-communications.mjs""#));
        assert!(
            tasks.contains("approval_request=.memd/state/approved-communications-request.json")
        );
        assert!(
            tasks.contains("approved_template=.memd/state/approved-communications-template.json")
        );

        let commands = render_live_state_command_lines(&report);
        assert!(commands.contains("# live_state_commands sync_required=true count=6"));
        assert!(commands.contains("memd live-state ingest"));
        assert!(commands.contains("--module messages"));
        assert!(commands.contains("--scope approved"));
        assert!(commands.contains("--approved"));
        assert!(commands.contains("'approved text-message metadata"));

        let batch_template = render_live_state_batch_template(&report);
        assert!(batch_template.contains(r#""records""#));
        assert!(batch_template.contains(r#""sourceApp": "approved_communications""#));
        assert!(batch_template.contains(r#""module": "messages""#));
        assert!(batch_template.contains(r#""scope": "approved""#));
        assert!(batch_template.contains(r#""approved": true"#));
        assert!(batch_template.contains(r#""agentsecretsApproved": false"#));
        assert!(batch_template.contains("agentsecretsApproved=true"));
        assert!(batch_template.contains(r#""raw_media_stored": false"#));
    }

    #[test]
    fn live_state_status_surfaces_source_unavailable_diagnostics() {
        let root = tempfile::tempdir().expect("tempdir");
        let output = root.path().join(".memd");
        let source_status_path = live_app_source_status_path(&output);
        std::fs::create_dir_all(source_status_path.parent().expect("state dir"))
            .expect("create state dir");
        std::fs::write(
            &source_status_path,
            r#"{
  "version": 1,
  "updated_at": "2026-05-17T06:00:00Z",
  "sources": [
    {
      "source_app": "memd",
      "status": "unavailable",
      "checked_at": "2026-05-17T06:00:00Z",
      "api_base": "http://127.0.0.1:3000",
      "api_bases": ["http://127.0.0.1:3010", "http://127.0.0.1:3000"],
      "auth_configured": false,
      "visible_page": "missing",
      "produced": [],
      "missing": ["visible_page", "calendar", "todos", "reminders", "messages", "email"],
      "record_count": 0,
      "endpoints": [
        {"module": "calendar", "path": "/api/calendar", "ok": false, "status": 0, "error": "unreachable"}
      ],
      "last_error": "missing live-state surfaces: visible_page, calendar, todos, reminders, messages, email"
    }
  ]
}"#,
        )
        .expect("write source status");

        let report = live_state_report(&output).expect("status report");
        assert_eq!(report.source_unavailable, 1);
        assert_eq!(report.source_fresh, 0);
        assert_eq!(report.source_stale, 1);
        assert_eq!(
            report.source_status_path,
            source_status_path.display().to_string()
        );
        assert_eq!(report.source_statuses[0].source_app, "memd");
        assert_eq!(
            report.source_statuses[0].api_bases,
            vec![
                "http://127.0.0.1:3010".to_string(),
                "http://127.0.0.1:3000".to_string()
            ]
        );
        assert!(!report.source_statuses[0].auth_configured);
        assert_eq!(report.source_statuses[0].missing.len(), 6);

        let summary = render_live_state_summary(&report);
        assert!(summary.contains("source_unavailable=1"));
        assert!(summary.contains("source_stale=1"));
        assert!(summary.contains("source_status:memd status=unavailable"));
        assert!(summary.contains("api_bases=http://127.0.0.1:3010,http://127.0.0.1:3000"));
        assert!(summary.contains("auth_configured=false"));
        assert!(summary.contains("freshness=stale"));
        assert!(summary.contains("missing=visible_page,calendar,todos,reminders,messages,email"));
        assert!(summary.contains("state_map_fresh=none"));
        assert!(summary.contains("state_map_unmet=visible_page,calendar,reminders,todos"));

        let section = render_live_app_state_section(&output, 8);
        assert!(section.contains("source_status:memd status=unavailable"));
        assert!(section.contains("freshness=stale"));
        assert!(section.contains("api_base=http://127.0.0.1:3000"));

        let detail = live_state_blocker_detail_from_report(&report).expect("blocker detail");
        assert!(detail.contains("memd:status=unavailable"), "{detail}");
        assert!(
            detail.contains(r#"producer_route="scripts/live-state-sync-memd.sh""#),
            "{detail}"
        );
        assert!(
            detail.contains("memd-owned producers only; does not launch ClawControl"),
            "{detail}"
        );
        assert!(!detail.contains("external_source_route="), "{detail}");
    }

    #[test]
    fn live_state_blocker_detail_surfaces_clawcontrol_access_route() {
        let root = tempfile::tempdir().expect("tempdir");
        let output = root.path().join(".memd");
        let source_status_path = live_app_source_status_path(&output);
        std::fs::create_dir_all(source_status_path.parent().expect("state dir"))
            .expect("create state dir");
        std::fs::write(
            &source_status_path,
            r#"{
  "version": 1,
  "updated_at": "2026-05-17T06:00:00Z",
  "sources": [
    {
      "source_app": "memd",
      "status": "auth_required",
      "checked_at": "2026-05-17T06:00:00Z",
      "api_base": "http://127.0.0.1:3010",
      "api_bases": ["http://127.0.0.1:3010", "http://127.0.0.1:3000"],
      "auth_configured": false,
      "visible_page": "missing",
      "produced": [],
      "missing": ["visible_page", "calendar", "todos", "reminders", "messages", "email"],
      "record_count": 0,
      "endpoints": [
        {"module": "calendar", "path": "/api/calendar", "ok": false, "status": 401, "error": "HTTP 401"}
      ],
      "last_error": "provide CLAWCONTROL_API_KEY or MC_API_KEY for X-API-Key auth"
    }
  ]
}"#,
        )
        .expect("write source status");

        let report = live_state_report(&output).expect("status report");
        let detail = live_state_blocker_detail_from_report(&report).expect("blocker detail");

        assert!(detail.contains("memd:status=auth_required"), "{detail}");
        assert!(
            detail.contains("missing=visible_page,calendar,reminders,todos"),
            "{detail}"
        );
        assert!(
            !detail.contains(clawcontrol_api_key_access_route_command()),
            "{detail}"
        );
        assert!(
            detail.contains(r#"producer_route="scripts/live-state-sync-memd.sh""#),
            "{detail}"
        );
        assert!(
            detail.contains("memd-owned producers only; does not launch ClawControl"),
            "{detail}"
        );
        assert!(!detail.contains("CLAWCONTROL_API_KEY="), "{detail}");
        assert!(!detail.contains("MC_API_KEY="), "{detail}");
    }

    #[test]
    fn live_state_blocker_detail_uses_unmet_requirements_after_partial_fallback() {
        let root = tempfile::tempdir().expect("tempdir");
        let output = root.path().join(".memd");
        let source_status_path = live_app_source_status_path(&output);
        std::fs::create_dir_all(source_status_path.parent().expect("state dir"))
            .expect("create state dir");
        std::fs::write(
            &source_status_path,
            r#"{
  "version": 1,
  "updated_at": "2026-05-17T08:00:00Z",
  "sources": [
    {
      "source_app": "memd",
      "status": "auth_required",
      "checked_at": "2026-05-17T08:00:00Z",
      "api_base": "http://127.0.0.1:3010",
      "api_bases": ["http://127.0.0.1:3010"],
      "auth_configured": false,
      "visible_page": "ok",
      "produced": [],
      "missing": ["messages", "email"],
      "record_count": 0,
      "endpoints": [],
      "last_error": "provide approved communications file for messages/email"
    },
    {
      "source_app": "approved_communications",
      "status": "missing_approval",
      "checked_at": "2026-05-17T08:00:00Z",
      "api_base": "approved-communications",
      "api_bases": ["approved-communications"],
      "auth_configured": false,
      "visible_page": "not_applicable",
      "produced": [],
      "missing": ["messages", "email"],
      "record_count": 0,
      "endpoints": [],
      "last_error": "no approved communications file configured",
      "approval_request_path": ".memd/state/approved-communications-request.json"
    }
  ]
}"#,
        )
        .expect("write source status");
        let batch = LiveStateIngestBatchArgs {
            output: output.clone(),
            stdin: false,
            input_json: Some(
                r#"{
  "records": [
    {"sourceApp":"clawcontrol","module":"visible_page","scope":"current","visibility":"private","privacy":"metadata","approved":true,"summary":"visible page fallback fresh","payload":{"producer":"mac-bridge-fallback","title":"workspace"}},
    {"sourceApp":"clawcontrol","module":"calendar","scope":"primary","visibility":"private","privacy":"metadata","approved":true,"summary":"calendar fallback fresh","payload":{"producer":"mac-bridge","events":[]}},
    {"sourceApp":"clawcontrol","module":"reminders","scope":"default","visibility":"private","privacy":"metadata","approved":true,"summary":"reminders fallback fresh","payload":{"producer":"mac-bridge","reminders":[]}},
    {"sourceApp":"clawcontrol","module":"todos","scope":"default","visibility":"private","privacy":"metadata","approved":true,"summary":"todos fallback fresh","payload":{"producer":"mac-bridge","todos":[]}}
  ]
}"#
                .to_string(),
            ),
            input_file: None,
            json: false,
        };
        let report = ingest_live_state_batch(&batch).expect("ingest fallback records");
        let detail = live_state_blocker_detail_from_report(&report).expect("blocker detail");

        assert!(!detail.contains("status=auth_required"), "{detail}");
        assert!(detail.contains("status=missing_approval"), "{detail}");
        assert!(detail.contains("missing=messages,email"), "{detail}");
        assert!(
            !detail.contains(clawcontrol_api_key_access_route_command()),
            "{detail}"
        );
        assert!(
            detail.contains(approved_communications_access_route_command()),
            "{detail}"
        );
        assert!(
            detail.contains(
                r#"producer_route="scripts/live-state-capture-approved-communications.mjs""#
            ),
            "{detail}"
        );
        assert!(
            detail.contains(
                r#"approved_zero_route="APPROVED_COMMUNICATIONS_EMPTY_APPROVED=1 scripts/live-state-capture-approved-communications.mjs""#
            ),
            "{detail}"
        );
        assert!(
            detail.contains("explicitly approves zero message/email metadata"),
            "{detail}"
        );
        assert!(
            detail
                .contains(r#"approval_request=".memd/state/approved-communications-request.json""#),
            "{detail}"
        );
        let summary = render_live_state_summary(&report);
        assert!(summary.contains("missing=messages,email"), "{summary}");
        assert!(
            summary.contains("approval_request=.memd/state/approved-communications-request.json"),
            "{summary}"
        );
        assert!(
            summary.contains("state_map_fresh=visible_page,calendar,reminders,todos"),
            "{summary}"
        );
        assert!(
            summary.contains("state_map_unmet=messages,email"),
            "{summary}"
        );
    }
}

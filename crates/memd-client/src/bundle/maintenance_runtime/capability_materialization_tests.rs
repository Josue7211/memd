use super::*;

#[cfg(test)]
mod capability_materialization_tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    static HOST_CLI_INSTALL_ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn lock_host_cli_install_env() -> std::sync::MutexGuard<'static, ()> {
        HOST_CLI_INSTALL_ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("host CLI install env lock poisoned")
    }

    fn capability(
        harness: &str,
        kind: &str,
        name: &str,
        portability_class: &str,
        source_path: &str,
    ) -> CapabilityRecord {
        CapabilityRecord {
            harness: harness.to_string(),
            kind: kind.to_string(),
            name: name.to_string(),
            status: "available-server".to_string(),
            portability_class: portability_class.to_string(),
            source_path: source_path.to_string(),
            bridge_hint: None,
            hash: None,
            notes: Vec::new(),
        }
    }

    #[test]
    fn capability_sync_uses_persisted_registry_without_forced_refresh() {
        let bundle = std::env::temp_dir().join(format!(
            "memd-capability-sync-persisted-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(bundle.join("state")).expect("create bundle state");
        let registry = CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: None,
            capabilities: vec![capability(
                "codex",
                "skill",
                "persisted-only",
                "universal",
                "/tmp/persisted-only/SKILL.md",
            )],
        };
        write_bundle_capability_registry(&bundle, &registry).expect("write registry");

        let report = run_capabilities_command(&CapabilitiesArgs {
            command: Some(CapabilitiesSubcommand::Sync(CapabilitiesSyncArgs {
                output: bundle.clone(),
                json: false,
            })),
            output: bundle.clone(),
            harness: None,
            kind: None,
            portability: None,
            query: None,
            limit: 12,
            summary: false,
            json: false,
            materialize_plan: false,
            materialize: false,
        })
        .expect("capability sync report");

        assert_eq!(report.records.len(), 1);
        assert_eq!(report.records[0].name, "persisted-only");
        fs::remove_dir_all(bundle).expect("cleanup capability temp");
    }

    #[test]
    fn host_cli_install_plan_materializes_as_installer_ready() {
        let bundle = std::env::temp_dir().join(format!(
            "memd-host-cli-install-plan-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(bundle.join("state")).expect("create bundle state");
        let mut record = capability(
            "local",
            "cli",
            "memd-test-gh",
            "host-local",
            "/usr/local/bin/memd-test-gh",
        );
        record.notes = vec![
            "PATH inventory; executable availability is host-local".to_string(),
            "memd:host-cli-install-plan:#!/bin/sh\necho install gh\nexit 2\n".to_string(),
        ];
        let registry = CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: None,
            capabilities: vec![record],
        };
        write_bundle_capability_registry(&bundle, &registry).expect("write registry");

        let report = run_capabilities_command(&CapabilitiesArgs {
            command: None,
            output: bundle.clone(),
            harness: None,
            kind: None,
            portability: None,
            query: None,
            limit: 12,
            summary: false,
            json: false,
            materialize_plan: false,
            materialize: true,
        })
        .expect("capability report")
        .materialization
        .expect("materialization report");

        assert_eq!(report.status, "partial-applied");
        assert!(report.applied >= 1);
        assert_eq!(report.missing, 0);
        assert!(report.host_local >= 1);
        assert!(!report.fresh_machine_ready);
        let action = report
            .actions
            .iter()
            .find(|action| action.name == "memd-test-gh")
            .expect("memd-test-gh action");
        assert_eq!(action.status, "installer-ready");
        assert_eq!(action.action, "write-host-cli-install-plan");
        assert!(action.reason.contains("MEMD_HOST_CLI_INSTALL_APPROVED=1"));
        let plan_path = bundle
            .join("install")
            .join("host-cli")
            .join("memd-test-gh.sh");
        let plan = fs::read_to_string(&plan_path).expect("read install plan");
        assert!(plan.contains("echo install gh"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(plan_path)
                .expect("stat install plan")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o755);
        }

        let second_report = run_capabilities_command(&CapabilitiesArgs {
            command: None,
            output: bundle.clone(),
            harness: None,
            kind: None,
            portability: None,
            query: None,
            limit: 12,
            summary: false,
            json: false,
            materialize_plan: false,
            materialize: true,
        })
        .expect("second capability report")
        .materialization
        .expect("second materialization report");
        let second_action = second_report
            .actions
            .iter()
            .find(|action| action.name == "memd-test-gh")
            .expect("second memd-test-gh action");
        assert_eq!(second_action.status, "installer-ready");
        assert!(second_action.reason.contains("already materialized"));
        assert_eq!(second_report.applied, 0);
        assert_eq!(second_report.missing, 0);

        fs::remove_dir_all(bundle).ok();
    }

    #[test]
    fn host_cli_auth_gap_reports_next_auth_check() {
        let _guard = lock_host_cli_install_env();
        let old_path = std::env::var_os("PATH");
        unsafe {
            std::env::remove_var("PATH");
        }
        let bundle =
            std::env::temp_dir().join(format!("memd-host-cli-auth-gap-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(bundle.join("state")).expect("create bundle state");
        let mut record = capability("local", "cli", "gh", "host-local", "host-cli:gh");
        record.notes = vec![
            "PATH inventory; executable availability is host-local".to_string(),
            "memd:host-cli-install-plan:#!/bin/sh\necho install gh\n".to_string(),
        ];
        let registry = CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: None,
            capabilities: vec![record],
        };

        let report = build_capabilities_response_from_registry(
            &CapabilitiesArgs {
                command: None,
                output: bundle.clone(),
                harness: None,
                kind: None,
                portability: None,
                query: None,
                limit: 12,
                summary: false,
                json: false,
                materialize_plan: true,
                materialize: false,
            },
            &bundle,
            &registry,
            &CapabilityBridgeRegistry {
                generated_at: Utc::now(),
                actions: Vec::new(),
            },
            "status",
            false,
            12,
        )
        .expect("capability report")
        .materialization
        .expect("materialization report");

        assert_eq!(report.auth_gaps, 1);
        assert_eq!(report.auth_unknown, 1);
        assert_eq!(report.auth_authenticated, 0);
        assert_eq!(report.auth_unauthenticated, 0);
        let action = report.actions.first().expect("host CLI action");
        assert_eq!(action.auth_status.as_deref(), Some("unknown"));
        assert_eq!(action.auth_check.as_deref(), Some("gh auth status"));
        assert!(action.reason.contains("auth_status=unknown"));
        assert!(action.reason.contains("auth_check=gh auth status"));

        unsafe {
            match old_path {
                Some(value) => std::env::set_var("PATH", value),
                None => std::env::remove_var("PATH"),
            }
        }
        fs::remove_dir_all(bundle).ok();
    }

    #[test]
    fn host_cli_auth_probe_marks_authenticated_without_storing_output() {
        let _guard = lock_host_cli_install_env();
        let old_path = std::env::var_os("PATH");
        let old_auth_probe = std::env::var_os("MEMD_CAPABILITIES_AUTH_PROBE");
        let root =
            std::env::temp_dir().join(format!("memd-host-cli-auth-probe-{}", uuid::Uuid::new_v4()));
        let bundle = root.join(".memd");
        let bin = root.join("bin");
        fs::create_dir_all(bundle.join("state")).expect("create bundle state");
        fs::create_dir_all(&bin).expect("create fake bin");
        let gh = bin.join("gh");
        fs::write(&gh, "#!/bin/sh\nexit 0\n").expect("write fake gh");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&gh).expect("fake gh metadata").permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&gh, permissions).expect("chmod fake gh");
        }
        unsafe {
            std::env::set_var("PATH", &bin);
            std::env::set_var("MEMD_CAPABILITIES_AUTH_PROBE", "1");
        }

        let mut record = capability(
            "local",
            "cli",
            "gh",
            "host-local",
            &gh.display().to_string(),
        );
        record.notes = vec![
            "PATH inventory; executable availability is host-local".to_string(),
            "memd:host-cli-install-plan:#!/bin/sh\necho install gh\n".to_string(),
        ];
        let registry = CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: None,
            capabilities: vec![record],
        };

        let report = build_capabilities_response_from_registry(
            &CapabilitiesArgs {
                command: None,
                output: bundle.clone(),
                harness: None,
                kind: None,
                portability: None,
                query: None,
                limit: 12,
                summary: false,
                json: false,
                materialize_plan: true,
                materialize: false,
            },
            &bundle,
            &registry,
            &CapabilityBridgeRegistry {
                generated_at: Utc::now(),
                actions: Vec::new(),
            },
            "status",
            false,
            12,
        )
        .expect("capability report")
        .materialization
        .expect("materialization report");

        assert_eq!(report.auth_gaps, 0);
        assert_eq!(report.auth_unknown, 0);
        assert_eq!(report.auth_authenticated, 1);
        assert_eq!(report.auth_unauthenticated, 0);
        let action = report.actions.first().expect("host CLI action");
        assert_eq!(action.auth_status.as_deref(), Some("authenticated"));
        assert_eq!(action.auth_check.as_deref(), Some("gh auth status"));
        assert!(!action.reason.contains("stdout="));
        assert!(!action.reason.contains("stderr="));

        unsafe {
            match old_path {
                Some(value) => std::env::set_var("PATH", value),
                None => std::env::remove_var("PATH"),
            }
            match old_auth_probe {
                Some(value) => std::env::set_var("MEMD_CAPABILITIES_AUTH_PROBE", value),
                None => std::env::remove_var("MEMD_CAPABILITIES_AUTH_PROBE"),
            }
        }
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn host_cli_auth_probe_marks_unauthenticated_as_gap() {
        let _guard = lock_host_cli_install_env();
        let old_path = std::env::var_os("PATH");
        let old_auth_probe = std::env::var_os("MEMD_CAPABILITIES_AUTH_PROBE");
        let root = std::env::temp_dir().join(format!(
            "memd-host-cli-auth-probe-fail-{}",
            uuid::Uuid::new_v4()
        ));
        let bundle = root.join(".memd");
        let bin = root.join("bin");
        fs::create_dir_all(bundle.join("state")).expect("create bundle state");
        fs::create_dir_all(&bin).expect("create fake bin");
        let gh = bin.join("gh");
        fs::write(&gh, "#!/bin/sh\nexit 1\n").expect("write fake gh");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&gh).expect("fake gh metadata").permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&gh, permissions).expect("chmod fake gh");
        }
        unsafe {
            std::env::set_var("PATH", &bin);
            std::env::set_var("MEMD_CAPABILITIES_AUTH_PROBE", "1");
        }

        let mut record = capability(
            "local",
            "cli",
            "gh",
            "host-local",
            &gh.display().to_string(),
        );
        record.notes = vec![
            "PATH inventory; executable availability is host-local".to_string(),
            "memd:host-cli-install-plan:#!/bin/sh\necho install gh\n".to_string(),
        ];
        let registry = CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: None,
            capabilities: vec![record],
        };

        let report = build_capabilities_response_from_registry(
            &CapabilitiesArgs {
                command: None,
                output: bundle.clone(),
                harness: None,
                kind: None,
                portability: None,
                query: None,
                limit: 12,
                summary: false,
                json: false,
                materialize_plan: true,
                materialize: false,
            },
            &bundle,
            &registry,
            &CapabilityBridgeRegistry {
                generated_at: Utc::now(),
                actions: Vec::new(),
            },
            "status",
            false,
            12,
        )
        .expect("capability report")
        .materialization
        .expect("materialization report");

        assert_eq!(report.auth_gaps, 1);
        assert_eq!(report.auth_unknown, 0);
        assert_eq!(report.auth_authenticated, 0);
        assert_eq!(report.auth_unauthenticated, 1);
        let action = report.actions.first().expect("host CLI action");
        assert_eq!(action.auth_status.as_deref(), Some("unauthenticated"));
        assert_eq!(action.auth_check.as_deref(), Some("gh auth status"));

        unsafe {
            match old_path {
                Some(value) => std::env::set_var("PATH", value),
                None => std::env::remove_var("PATH"),
            }
            match old_auth_probe {
                Some(value) => std::env::set_var("MEMD_CAPABILITIES_AUTH_PROBE", value),
                None => std::env::remove_var("MEMD_CAPABILITIES_AUTH_PROBE"),
            }
        }
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn host_cli_auth_notes_are_syncable_without_secret_output() {
        let _guard = lock_host_cli_install_env();
        let old_path = std::env::var_os("PATH");
        let old_auth_probe = std::env::var_os("MEMD_CAPABILITIES_AUTH_PROBE");
        let root = std::env::temp_dir().join(format!(
            "memd-host-cli-auth-sync-note-{}",
            uuid::Uuid::new_v4()
        ));
        let bundle = root.join(".memd");
        let bin = root.join("bin");
        fs::create_dir_all(bundle.join("state")).expect("create bundle state");
        fs::create_dir_all(&bin).expect("create fake bin");
        let gh = bin.join("gh");
        fs::write(&gh, "#!/bin/sh\necho secret-ish-output\nexit 1\n").expect("write fake gh");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&gh).expect("fake gh metadata").permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&gh, permissions).expect("chmod fake gh");
        }
        unsafe {
            std::env::set_var("PATH", &bin);
            std::env::set_var("MEMD_CAPABILITIES_AUTH_PROBE", "1");
        }

        let mut record = capability(
            "local",
            "cli",
            "gh",
            "host-local",
            &gh.display().to_string(),
        );
        record.notes = vec![
            "PATH inventory; executable availability is host-local".to_string(),
            "memd:host-auth-status:stale".to_string(),
            "memd:host-auth-output-stored:true".to_string(),
        ];
        let registry = CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: None,
            capabilities: vec![record],
        };

        let response = build_capabilities_response_from_registry(
            &CapabilitiesArgs {
                command: None,
                output: bundle.clone(),
                harness: None,
                kind: None,
                portability: None,
                query: None,
                limit: 12,
                summary: false,
                json: false,
                materialize_plan: false,
                materialize: false,
            },
            &bundle,
            &registry,
            &CapabilityBridgeRegistry {
                generated_at: Utc::now(),
                actions: Vec::new(),
            },
            "status",
            false,
            12,
        )
        .expect("capability response");

        let record = response
            .records
            .iter()
            .find(|record| record.name == "gh")
            .expect("gh record");
        assert!(
            record
                .notes
                .iter()
                .any(|note| note == "memd:host-auth-status:unauthenticated")
        );
        assert!(
            record
                .notes
                .iter()
                .any(|note| note == "memd:host-auth-check:gh auth status")
        );
        assert!(
            record
                .notes
                .iter()
                .any(|note| note == "memd:host-auth-proof:local-probe")
        );
        assert!(
            record
                .notes
                .iter()
                .any(|note| note == "memd:host-auth-output-stored:false")
        );
        assert!(
            record
                .notes
                .iter()
                .any(|note| note == "memd:host-cli-path-status:on-path")
        );
        let serialized = serde_json::to_string(record).expect("serialize record");
        assert!(!serialized.contains("secret-ish-output"));
        assert!(!serialized.contains("stdout="));
        assert!(!serialized.contains("stderr="));

        unsafe {
            match old_path {
                Some(value) => std::env::set_var("PATH", value),
                None => std::env::remove_var("PATH"),
            }
            match old_auth_probe {
                Some(value) => std::env::set_var("MEMD_CAPABILITIES_AUTH_PROBE", value),
                None => std::env::remove_var("MEMD_CAPABILITIES_AUTH_PROBE"),
            }
        }
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn capability_sync_persists_host_cli_auth_notes_to_registry() {
        let _guard = lock_host_cli_install_env();
        let old_path = std::env::var_os("PATH");
        let old_auth_probe = std::env::var_os("MEMD_CAPABILITIES_AUTH_PROBE");
        let root = std::env::temp_dir().join(format!(
            "memd-host-cli-auth-sync-registry-{}",
            uuid::Uuid::new_v4()
        ));
        let bundle = root.join(".memd");
        let bin = root.join("bin");
        fs::create_dir_all(bundle.join("state")).expect("create bundle state");
        fs::create_dir_all(&bin).expect("create fake bin");
        let gh = bin.join("gh");
        fs::write(&gh, "#!/bin/sh\nexit 0\n").expect("write fake gh");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&gh).expect("fake gh metadata").permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&gh, permissions).expect("chmod fake gh");
        }
        unsafe {
            std::env::set_var("PATH", &bin);
            std::env::set_var("MEMD_CAPABILITIES_AUTH_PROBE", "1");
        }

        run_capabilities_command(&CapabilitiesArgs {
            command: Some(CapabilitiesSubcommand::Sync(
                crate::cli::args::CapabilitiesSyncArgs {
                    output: bundle.clone(),
                    json: false,
                },
            )),
            output: PathBuf::from("ignored-by-sync"),
            harness: None,
            kind: None,
            portability: None,
            query: None,
            limit: 12,
            summary: false,
            json: false,
            materialize_plan: false,
            materialize: false,
        })
        .expect("capability sync");

        let persisted = read_bundle_capability_registry(&bundle)
            .expect("read registry")
            .expect("registry");
        let record = persisted
            .capabilities
            .iter()
            .find(|record| record.name == "gh")
            .expect("gh record");
        assert!(
            record
                .notes
                .iter()
                .any(|note| note == "memd:host-auth-status:authenticated")
        );
        assert!(
            record
                .notes
                .iter()
                .any(|note| note == "memd:host-auth-proof:local-probe")
        );
        assert!(
            record
                .notes
                .iter()
                .any(|note| note == "memd:host-auth-output-stored:false")
        );
        assert!(
            record
                .notes
                .iter()
                .any(|note| note == "memd:host-cli-path-status:on-path")
        );

        unsafe {
            match old_path {
                Some(value) => std::env::set_var("PATH", value),
                None => std::env::remove_var("PATH"),
            }
            match old_auth_probe {
                Some(value) => std::env::set_var("MEMD_CAPABILITIES_AUTH_PROBE", value),
                None => std::env::remove_var("MEMD_CAPABILITIES_AUTH_PROBE"),
            }
        }
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn host_cli_auth_notes_skip_non_host_local_cli_records() {
        let root = std::env::temp_dir().join(format!(
            "memd-host-cli-auth-sync-skip-{}",
            uuid::Uuid::new_v4()
        ));
        let bundle = root.join(".memd");
        fs::create_dir_all(bundle.join("state")).expect("create bundle state");

        let mut record = capability(
            "codex",
            "plugin",
            "github:7955f1db",
            "harness-native",
            "/tmp/plugin.json",
        );
        record.notes = vec!["memd:host-auth-status:stale".to_string()];
        let registry = CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: None,
            capabilities: vec![record],
        };

        let response = build_capabilities_response_from_registry(
            &CapabilitiesArgs {
                command: None,
                output: bundle.clone(),
                harness: None,
                kind: None,
                portability: None,
                query: None,
                limit: 12,
                summary: false,
                json: false,
                materialize_plan: false,
                materialize: false,
            },
            &bundle,
            &registry,
            &CapabilityBridgeRegistry {
                generated_at: Utc::now(),
                actions: Vec::new(),
            },
            "status",
            false,
            12,
        )
        .expect("capability response");

        let record = response
            .records
            .iter()
            .find(|record| record.name == "github:7955f1db")
            .expect("plugin record");
        assert_eq!(record.notes, vec!["memd:host-auth-status:stale"]);

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn approved_host_cli_install_plan_runs_and_rechecks_path() {
        let _guard = lock_host_cli_install_env();
        let old_approved = std::env::var_os("MEMD_HOST_CLI_INSTALL_APPROVED");
        let old_path = std::env::var_os("PATH");
        let root = std::env::temp_dir().join(format!(
            "memd-host-cli-approved-install-{}",
            uuid::Uuid::new_v4()
        ));
        let bundle = root.join(".memd");
        let bin = root.join("bin");
        fs::create_dir_all(bundle.join("state")).expect("create bundle state");
        fs::create_dir_all(&bin).expect("create fake bin");
        let cli_path = bin.join("memd-test-runner");
        let plan = format!(
            "#!/bin/sh\nset -eu\ncat > '{}' <<'EOF'\n#!/bin/sh\nexit 0\nEOF\nchmod +x '{}'\n",
            cli_path.display(),
            cli_path.display()
        );
        let mut record = capability(
            "local",
            "cli",
            "memd-test-runner",
            "host-local",
            "host-cli:memd-test-runner",
        );
        record.notes = vec![
            "PATH inventory; executable availability is host-local".to_string(),
            format!("memd:host-cli-install-plan:{plan}"),
        ];
        let registry = CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: None,
            capabilities: vec![record],
        };
        write_bundle_capability_registry(&bundle, &registry).expect("write registry");

        let mut paths = vec![bin.clone()];
        if let Some(old_path) = old_path.as_ref() {
            paths.extend(std::env::split_paths(old_path));
        }
        let joined_path = std::env::join_paths(paths).expect("join PATH");
        unsafe {
            std::env::set_var("MEMD_HOST_CLI_INSTALL_APPROVED", "1");
            std::env::set_var("PATH", &joined_path);
        }

        let report = build_capabilities_response_from_registry(
            &CapabilitiesArgs {
                command: None,
                output: bundle.clone(),
                harness: None,
                kind: None,
                portability: None,
                query: None,
                limit: 12,
                summary: false,
                json: false,
                materialize_plan: false,
                materialize: true,
            },
            &bundle,
            &registry,
            &CapabilityBridgeRegistry {
                generated_at: Utc::now(),
                actions: Vec::new(),
            },
            "status",
            false,
            12,
        )
        .expect("capability report")
        .materialization
        .expect("materialization report");

        unsafe {
            match old_approved {
                Some(value) => std::env::set_var("MEMD_HOST_CLI_INSTALL_APPROVED", value),
                None => std::env::remove_var("MEMD_HOST_CLI_INSTALL_APPROVED"),
            }
            match old_path {
                Some(value) => std::env::set_var("PATH", value),
                None => std::env::remove_var("PATH"),
            }
        }

        let action = report
            .actions
            .iter()
            .find(|action| action.name == "memd-test-runner")
            .expect("memd-test-runner action");
        assert_eq!(action.status, "present");
        assert!(action.reason.contains("now available on PATH"));
        assert!(cli_path.is_file());

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn host_cli_on_path_keeps_fresh_machine_materialization_partial() {
        if !host_cli_available_on_path("sh") {
            return;
        }
        let bundle =
            std::env::temp_dir().join(format!("memd-host-cli-on-path-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(bundle.join("state")).expect("create bundle state");
        let mut record = capability("local", "cli", "sh", "host-local", "/bin/sh");
        record.notes = vec![
            "PATH inventory; executable availability is host-local".to_string(),
            "memd:host-cli-install-plan:#!/bin/sh\necho install sh\n".to_string(),
        ];
        let registry = CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: None,
            capabilities: vec![record],
        };
        write_bundle_capability_registry(&bundle, &registry).expect("write registry");

        let report = build_capabilities_response_from_registry(
            &CapabilitiesArgs {
                command: None,
                output: bundle.clone(),
                harness: None,
                kind: None,
                portability: None,
                query: None,
                limit: 12,
                summary: false,
                json: false,
                materialize_plan: true,
                materialize: false,
            },
            &bundle,
            &registry,
            &CapabilityBridgeRegistry {
                generated_at: Utc::now(),
                actions: Vec::new(),
            },
            "status",
            false,
            12,
        )
        .expect("capability report")
        .materialization
        .expect("materialization report");

        assert_eq!(report.status, "partial-host-local");
        assert_eq!(report.missing, 0);
        assert_eq!(report.host_local, 1);
        assert_eq!(report.auth_gaps, 0);
        assert_eq!(report.auth_unknown, 0);
        assert_eq!(report.auth_authenticated, 0);
        assert_eq!(report.auth_unauthenticated, 0);
        assert!(!report.fresh_machine_ready);
        let action = report.actions.first().expect("host CLI action");
        assert_eq!(action.status, "present");
        assert_eq!(action.action, "host-cli-on-path");
        assert!(action.reason.contains("fresh machines still need"));

        fs::remove_dir_all(bundle).ok();
    }

    #[test]
    fn materialize_restores_text_payload_assets() {
        let root =
            std::env::temp_dir().join(format!("memd-payload-apply-{}", uuid::Uuid::new_v4()));
        let bundle = root.join(".memd");
        fs::create_dir_all(bundle.join("state")).expect("create bundle state");
        let target = root.join("skills").join("demo").join("SKILL.md");
        let mut record = capability(
            "codex",
            "skill",
            "demo",
            "harness-native",
            "skills/demo/SKILL.md",
        );
        record.notes = vec!["memd:payload-text:# Demo\n".to_string()];
        let registry = CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: Some(root.display().to_string()),
            capabilities: vec![record],
        };
        write_bundle_capability_registry(&bundle, &registry).expect("write registry");

        let report = run_capabilities_command(&CapabilitiesArgs {
            command: None,
            output: bundle.clone(),
            harness: None,
            kind: None,
            portability: None,
            query: None,
            limit: 12,
            summary: false,
            json: false,
            materialize_plan: false,
            materialize: true,
        })
        .expect("capability report")
        .materialization
        .expect("materialization report");

        assert_eq!(report.applied, 1);
        assert_eq!(
            fs::read_to_string(&target).expect("read payload"),
            "# Demo\n"
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn materialize_restores_text_payload_file_sets() {
        let root =
            std::env::temp_dir().join(format!("memd-payload-set-apply-{}", uuid::Uuid::new_v4()));
        let bundle = root.join(".memd");
        fs::create_dir_all(bundle.join("state")).expect("create bundle state");
        let mut record = capability(
            "codex",
            "skill",
            "demo",
            "harness-native",
            "skills/demo/SKILL.md",
        );
        record.notes = vec![
            "memd:payload-text:# Demo\n".to_string(),
            format!(
                "memd:payload-file-json:{}",
                serde_json::json!({"path": "SKILL.md", "content": "# Demo\n"})
            ),
            format!(
                "memd:payload-file-json:{}",
                serde_json::json!({"path": "scripts/run.sh", "content": "#!/bin/sh\necho demo\n"})
            ),
            format!(
                "memd:payload-file-json:{}",
                serde_json::json!({"path": "../escape.sh", "content": "nope\n"})
            ),
        ];
        let registry = CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: Some(root.display().to_string()),
            capabilities: vec![record],
        };
        write_bundle_capability_registry(&bundle, &registry).expect("write registry");

        let report = run_capabilities_command(&CapabilitiesArgs {
            command: None,
            output: bundle.clone(),
            harness: None,
            kind: None,
            portability: None,
            query: None,
            limit: 12,
            summary: false,
            json: false,
            materialize_plan: false,
            materialize: true,
        })
        .expect("capability report")
        .materialization
        .expect("materialization report");

        assert_eq!(report.applied, 1);
        let action = report
            .actions
            .iter()
            .find(|action| {
                action.harness == "codex" && action.kind == "skill" && action.name == "demo"
            })
            .expect("payload set action");
        assert_eq!(action.action, "restore-from-payload-set");
        assert_eq!(action.status, "present");
        assert_eq!(
            fs::read_to_string(root.join("skills/demo/SKILL.md")).expect("read skill"),
            "# Demo\n"
        );
        assert_eq!(
            fs::read_to_string(root.join("skills/demo/scripts/run.sh")).expect("read script"),
            "#!/bin/sh\necho demo\n"
        );
        assert!(!root.join("skills/escape.sh").exists());

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn materialize_plan_does_not_serialize_payload_body() {
        let root = std::env::temp_dir().join(format!("memd-payload-plan-{}", uuid::Uuid::new_v4()));
        let bundle = root.join(".memd");
        fs::create_dir_all(bundle.join("state")).expect("create bundle state");
        let mut record = capability(
            "codex",
            "skill",
            "demo",
            "harness-native",
            "skills/demo/SKILL.md",
        );
        record.notes = vec!["memd:payload-text:# Secret-ish payload\n".to_string()];
        let registry = CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: Some(root.display().to_string()),
            capabilities: vec![record],
        };
        write_bundle_capability_registry(&bundle, &registry).expect("write registry");

        let report = run_capabilities_command(&CapabilitiesArgs {
            command: None,
            output: bundle.clone(),
            harness: None,
            kind: None,
            portability: None,
            query: None,
            limit: 12,
            summary: false,
            json: false,
            materialize_plan: true,
            materialize: false,
        })
        .expect("capability report");
        let json = serde_json::to_string(&report.materialization).expect("serialize report");

        assert!(json.contains("restore-from-payload"));
        assert!(!json.contains("Secret-ish payload"));

        let full_json = serde_json::to_string(&report).expect("serialize full report");
        assert!(full_json.contains("memd:payload-text:<omitted"));
        assert!(!full_json.contains("Secret-ish payload"));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn pull_mode_materialize_flag_restores_payload_assets() {
        let root = std::env::temp_dir().join(format!("memd-payload-pull-{}", uuid::Uuid::new_v4()));
        let bundle = root.join(".memd");
        fs::create_dir_all(bundle.join("state")).expect("create bundle state");
        let target = root.join("skills").join("demo").join("SKILL.md");
        let mut record = capability(
            "codex",
            "skill",
            "demo",
            "harness-native",
            "skills/demo/SKILL.md",
        );
        record.notes = vec!["memd:payload-text:# Pull Demo\n".to_string()];
        let registry = CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: Some(root.display().to_string()),
            capabilities: vec![record],
        };

        let report = build_capabilities_response_from_registry(
            &CapabilitiesArgs {
                command: Some(CapabilitiesSubcommand::Pull(
                    crate::cli::args::CapabilitiesPullArgs {
                        output: bundle.clone(),
                        json: false,
                        materialize_plan: false,
                        materialize: true,
                    },
                )),
                output: PathBuf::from("ignored-by-pull"),
                harness: None,
                kind: None,
                portability: None,
                query: None,
                limit: 12,
                summary: false,
                json: false,
                materialize_plan: false,
                materialize: false,
            },
            &bundle,
            &registry,
            &CapabilityBridgeRegistry {
                generated_at: Utc::now(),
                actions: Vec::new(),
            },
            "pull",
            true,
            12,
        )
        .expect("capability response")
        .materialization
        .expect("materialization report");

        assert_eq!(report.applied, 1);
        assert_eq!(
            fs::read_to_string(&target).expect("read payload"),
            "# Pull Demo\n"
        );

        fs::remove_dir_all(root).ok();
    }
}

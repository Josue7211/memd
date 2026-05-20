use super::*;

fn capability(harness: &str, kind: &str, name: &str, status: &str) -> CapabilityRecord {
    CapabilityRecord {
        harness: harness.to_string(),
        kind: kind.to_string(),
        name: name.to_string(),
        status: status.to_string(),
        portability_class: "harness-native".to_string(),
        source_path: format!("/{harness}/{kind}/{name}"),
        bridge_hint: None,
        hash: None,
        notes: Vec::new(),
    }
}

#[test]
fn capability_merge_keeps_server_shadow_and_prefers_local_state() {
    let persisted = CapabilityRegistry {
        generated_at: Utc::now(),
        project_root: Some("/server-shadow".to_string()),
        capabilities: vec![
            capability("codex", "skill", "remote-only", "available-server"),
            capability("codex", "skill", "shared", "available-server"),
        ],
    };
    let local = CapabilityRegistry {
        generated_at: Utc::now(),
        project_root: Some("/local".to_string()),
        capabilities: vec![capability("codex", "skill", "shared", "installed")],
    };

    let merged = merge_capability_registries(local, persisted);

    assert_eq!(merged.capabilities.len(), 2);
    assert!(
        merged
            .capabilities
            .iter()
            .any(|record| { record.name == "remote-only" && record.status == "available-server" })
    );
    assert!(
        merged
            .capabilities
            .iter()
            .any(|record| { record.name == "shared" && record.status == "installed" })
    );
}

#[test]
fn capability_merge_keeps_server_payload_over_unwired_local_stub() {
    let mut persisted = capability("hermes", "harness-pack", "Hermes", "wired");
    persisted.notes = vec!["memd:payload-text:#!/bin/sh\n".to_string()];
    let local = capability("hermes", "harness-pack", "Hermes", "available");

    let merged = merge_capability_registries(
        CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: Some("/fresh".to_string()),
            capabilities: vec![local],
        },
        CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: Some("/server".to_string()),
            capabilities: vec![persisted],
        },
    );

    let record = merged
        .capabilities
        .iter()
        .find(|record| record.harness == "hermes")
        .expect("merged hermes capability");
    assert_eq!(record.status, "wired");
    assert!(capability_payload_text(record).is_some());
}

#[test]
fn capability_merge_keeps_server_host_cli_plan_over_local_path_stub() {
    let mut persisted = capability("local", "cli", "gh", "available-server");
    persisted.portability_class = "host-local".to_string();
    persisted.notes = vec!["memd:host-cli-install-plan:#!/bin/sh\necho install gh\n".to_string()];
    let mut local = capability("local", "cli", "gh", "available");
    local.portability_class = "host-local".to_string();

    let merged = merge_capability_registries(
        CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: Some("/fresh".to_string()),
            capabilities: vec![local],
        },
        CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: Some("/server".to_string()),
            capabilities: vec![persisted],
        },
    );

    let record = merged
        .capabilities
        .iter()
        .find(|record| record.name == "gh")
        .expect("merged gh capability");
    assert_eq!(record.status, "available-server");
    assert!(host_cli_install_plan_text(record).is_some());
}

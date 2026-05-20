use super::*;

#[test]
fn prune_bundle_compiled_memory_outputs_removes_stale_tree() {
    let output = std::env::temp_dir().join(format!("memd-compiled-prune-{}", uuid::Uuid::new_v4()));
    let compiled = bundle_compiled_memory_dir(&output);
    let stale = compiled.join("items").join("working").join("stale.md");
    std::fs::create_dir_all(stale.parent().expect("stale parent"))
        .expect("create stale compiled tree");
    std::fs::write(&stale, "# stale\n").expect("write stale compiled item");

    prune_bundle_compiled_memory_outputs(&output).expect("prune compiled memory");

    assert!(!compiled.exists());
    let _ = std::fs::remove_dir_all(output);
}

#[test]
fn bundle_memory_write_lock_releases_on_drop() {
    let output = std::env::temp_dir().join(format!("memd-memory-lock-{}", uuid::Uuid::new_v4()));
    let lock_path = output.join("state").join("write-memory.lock");

    let lock = acquire_bundle_memory_write_lock(&output).expect("acquire memory write lock");
    assert!(lock_path.exists());
    drop(lock);

    assert!(!lock_path.exists());
    let _ = std::fs::remove_dir_all(output);
}

#[test]
fn capability_registry_includes_codex_system_skills_and_plugin_manifests() {
    let home = std::env::temp_dir().join(format!("memd-capability-home-{}", uuid::Uuid::new_v4()));
    let system_skill = home
        .join(".codex")
        .join("skills")
        .join(".system")
        .join("imagegen");
    std::fs::create_dir_all(&system_skill).expect("create system skill");
    std::fs::write(system_skill.join("SKILL.md"), "# imagegen\n").expect("write skill");
    let plugin_root = home
        .join(".codex")
        .join("plugins")
        .join("cache")
        .join("openai-curated")
        .join("cloudflare")
        .join("abc123");
    let plugin_skill = plugin_root.join("skills").join("workers-best-practices");
    std::fs::create_dir_all(&plugin_skill).expect("create plugin skill");
    std::fs::write(plugin_skill.join("SKILL.md"), "# workers\n").expect("write plugin skill");
    let manifest_dir = plugin_root.join(".codex-plugin");
    std::fs::create_dir_all(&manifest_dir).expect("create plugin manifest dir");
    std::fs::write(manifest_dir.join("plugin.json"), "{}\n").expect("write manifest");

    let registry = build_bundle_capability_registry_with_home(None, Some(&home));

    assert!(registry.capabilities.iter().any(|record| {
        record.harness == "codex" && record.kind == "skill" && record.name == "imagegen"
    }));
    assert!(registry.capabilities.iter().any(|record| {
        record.harness == "codex"
            && record.kind == "plugin-skill"
            && record.name == "abc123:workers-best-practices"
    }));
    assert!(registry.capabilities.iter().any(|record| {
        record.harness == "codex" && record.kind == "plugin" && record.name == "cloudflare:abc123"
    }));

    let _ = std::fs::remove_dir_all(home);
}

#[test]
fn capability_registry_includes_harness_packs_and_claude_settings() {
    let home = std::env::temp_dir().join(format!(
        "memd-capability-home-harness-{}",
        uuid::Uuid::new_v4()
    ));
    let project = std::env::temp_dir().join(format!(
        "memd-capability-project-harness-{}",
        uuid::Uuid::new_v4()
    ));
    let claude = home.join(".claude");
    std::fs::create_dir_all(&claude).expect("create claude home");
    std::fs::write(claude.join("settings.json"), "{\"hooks\":{}}\n")
        .expect("write claude settings");
    let agents = project.join(".memd").join("agents");
    std::fs::create_dir_all(&agents).expect("create bundle agents");
    for profile in [
        "agent-zero.sh",
        "claude-code.sh",
        "codex.sh",
        "hermes.sh",
        "openclaw.sh",
        "opencode.sh",
    ] {
        std::fs::write(agents.join(profile), "#!/bin/sh\n").expect("write profile");
    }

    let registry = build_bundle_capability_registry_with_home(Some(&project), Some(&home));

    for harness in [
        "agent-zero",
        "claude-code",
        "codex",
        "hermes",
        "openclaw",
        "opencode",
    ] {
        assert!(
            registry.capabilities.iter().any(|record| {
                record.harness == harness
                    && record.kind == "harness-pack"
                    && record.status == "wired"
            }),
            "missing harness-pack for {harness}"
        );
    }
    assert!(registry.capabilities.iter().any(|record| {
        record.harness == "claude"
            && record.kind == "claude-config"
            && record.name == "settings.json"
    }));

    let _ = std::fs::remove_dir_all(home);
    let _ = std::fs::remove_dir_all(project);
}

#[test]
fn host_cli_install_plan_is_machine_approved_runner() {
    let script = render_host_cli_install_plan("wrangler", None);

    assert!(script.contains("MEMD_HOST_CLI_INSTALL_APPROVED=1"));
    assert!(script.contains("npm install -g wrangler"));
    assert!(script.contains("dry-run only; no host changes made"));
    assert!(script.contains("memd does not copy host-local binaries across machines"));
}

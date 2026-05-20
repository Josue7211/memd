fn collect_skill_files_recursive(root: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if depth > 8 || out.len() >= 500 || !root.is_dir() {
        return;
    }
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_file()
            && path
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value == "SKILL.md")
            && path
                .parent()
                .and_then(|parent| parent.parent())
                .and_then(|parent| parent.file_name())
                .and_then(|value| value.to_str())
                == Some("skills")
        {
            out.push(path);
            if out.len() >= 500 {
                return;
            }
        } else if path.is_dir() {
            collect_skill_files_recursive(&path, depth + 1, out);
            if out.len() >= 500 {
                return;
            }
        }
    }
}

pub(crate) fn collect_claude_family_capabilities(
    harness_root: &HarnessRoot,
    home: &Path,
) -> Vec<CapabilityRecord> {
    let mut records = Vec::new();
    let portability_class = if harness_root.harness == "claude" {
        "universal".to_string()
    } else {
        "claude-family".to_string()
    };
    records.extend(
        collect_named_file_sources(
            &harness_root.root,
            &[
                "AGENTS.md",
                "TEAMS.md",
                "MEMORY.md",
                "USER.md",
                "IDENTITY.md",
                "SOUL.md",
                "TOOLS.md",
                "BOOTSTRAP.md",
                "HEARTBEAT.md",
                "settings.json",
            ],
        )
        .into_iter()
        .map(|path| CapabilityRecord {
            harness: harness_root.harness.clone(),
            kind: source_kind_from_path(&path),
            name: path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("unknown")
                .to_string(),
            status: "discovered".to_string(),
            portability_class: portability_class.clone(),
            source_path: path.display().to_string(),
            bridge_hint: None,
            hash: file_sha256(&path),
            notes: notes_with_text_payload(
                vec!["detected from Claude-family harness root".to_string()],
                &path,
            ),
        }),
    );
    collect_directory_entry_capabilities(
        &mut records,
        &harness_root.harness,
        &harness_root.root,
        "agents",
        "agent",
        &portability_class,
        Some("agent"),
        &["md", "json", "yml", "yaml"],
    );
    collect_directory_entry_capabilities(
        &mut records,
        &harness_root.harness,
        &harness_root.root,
        "teams",
        "team",
        &portability_class,
        Some("team"),
        &["md", "json", "yml", "yaml"],
    );
    collect_directory_entry_capabilities(
        &mut records,
        &harness_root.harness,
        &harness_root.root,
        "hooks",
        "hook",
        &portability_class,
        Some("hook"),
        &["js", "mjs", "ts", "cts", "json"],
    );
    collect_directory_entry_capabilities(
        &mut records,
        &harness_root.harness,
        &harness_root.root,
        "command",
        "command",
        &portability_class,
        Some("command"),
        &["md", "json", "yml", "yaml", "sh", "js"],
    );
    collect_directory_entry_capabilities(
        &mut records,
        &harness_root.harness,
        &harness_root.root,
        "commands",
        "command",
        &portability_class,
        Some("command"),
        &["md", "json", "yml", "yaml", "sh", "js"],
    );
    records.extend(collect_claude_plugin_capabilities(
        &harness_root.root.join("settings.json"),
        &harness_root.harness,
        home,
    ));
    records
}

pub(crate) fn collect_openclaw_capabilities(workspace_root: &Path) -> Vec<CapabilityRecord> {
    collect_harness_root_directory_capabilities(
        workspace_root,
        "openclaw",
        "harness-native",
        &[
            "AGENTS.md",
            "TEAMS.md",
            "MEMORY.md",
            "USER.md",
            "IDENTITY.md",
            "SOUL.md",
            "TOOLS.md",
            "BOOTSTRAP.md",
            "HEARTBEAT.md",
        ],
    )
}

pub(crate) fn collect_opencode_capabilities(root: &Path) -> Vec<CapabilityRecord> {
    collect_harness_root_directory_capabilities(
        root,
        "opencode",
        "harness-native",
        &[
            "AGENTS.md",
            "TEAMS.md",
            "MEMORY.md",
            "USER.md",
            "IDENTITY.md",
            "SOUL.md",
            "TOOLS.md",
            "BOOTSTRAP.md",
            "HEARTBEAT.md",
            "opencode.json",
            "settings.json",
        ],
    )
}

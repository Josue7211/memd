use super::*;
use memd_schema::IngestLanesRequest;

pub(crate) async fn run_bundle_setup_command(args: &SetupArgs) -> anyhow::Result<()> {
    let init_args = normalize_init_args(setup_args_to_init_args(args))?;
    let decision = resolve_bootstrap_authority(init_args).await?;
    let init_args = decision.init_args;
    write_init_bundle(&init_args)?;

    // F2: Ingest lane source files into DB after setup.
    if let Ok(memd) = MemdClient::new(&init_args.base_url) {
        let root = init_args
            .output
            .parent()
            .unwrap_or(&init_args.output)
            .to_path_buf();
        let _ = memd
            .ingest_lanes(&IngestLanesRequest {
                root: root.display().to_string(),
                project: init_args.project.clone(),
                namespace: init_args.namespace.clone(),
            })
            .await;
    }

    if decision.fallback_activated {
        set_bundle_localhost_read_only_authority_state(
            &init_args.output,
            &decision.shared_base_url,
            "setup",
            "shared authority unavailable during bootstrap",
        )?;
        write_agent_profiles(&init_args.output)?;
        write_native_agent_bridge_files(&init_args.output)?;
    }
    if args.json {
        print_json(&json!({
            "bundle": init_args.output,
            "project": init_args.project,
            "namespace": init_args.namespace,
            "agent": init_args.agent,
            "base_url": init_args.base_url,
            "shared_base_url": decision.shared_base_url,
            "authority_mode": if decision.fallback_activated { "localhost_read_only" } else { "shared" },
            "route": init_args.route,
            "intent": init_args.intent,
            "voice_mode": init_args.voice_mode,
            "workspace": init_args.workspace,
            "visibility": init_args.visibility,
            "setup_ready": true,
        }))?;
    } else if args.summary {
        println!(
            "setup bundle={} project={} namespace={} agent={} voice={} authority={} ready=true",
            init_args.output.display(),
            init_args.project.as_deref().unwrap_or("none"),
            init_args.namespace.as_deref().unwrap_or("none"),
            init_args.agent,
            init_args.voice_mode.as_deref().unwrap_or("caveman-ultra"),
            if decision.fallback_activated {
                "localhost_read_only"
            } else {
                "shared"
            },
        );
    } else {
        println!("Initialized memd bundle at {}", init_args.output.display());
        if decision.fallback_activated {
            eprintln!("memd authority warning:");
            eprintln!("- shared authority unavailable");
            eprintln!("- localhost fallback is lower trust");
            eprintln!("- prompt-injection and split-brain risk increased");
            eprintln!("- coordination writes blocked");
        }
    }
    Ok(())
}

pub(crate) async fn run_bundle_doctor_command(
    args: &DoctorArgs,
    base_url: &str,
) -> anyhow::Result<()> {
    let bundle_root = resolve_doctor_bundle_root(args.output.as_deref())?;
    let mut status = read_bundle_status(&bundle_root, base_url).await?;
    let setup_ready = status
        .get("setup_ready")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    if args.repair && !setup_ready {
        let project_root = args.project_root.clone().or(detect_current_project_root()?);
        let setup_args = doctor_args_to_setup_args(args, bundle_root.clone(), project_root);
        let decision = resolve_bootstrap_authority(setup_args_to_init_args(&setup_args)).await?;
        write_init_bundle(&decision.init_args)?;
        if decision.fallback_activated {
            set_bundle_localhost_read_only_authority_state(
                &decision.init_args.output,
                &decision.shared_base_url,
                "doctor",
                "shared authority unavailable during repair bootstrap",
            )?;
            write_agent_profiles(&decision.init_args.output)?;
            write_native_agent_bridge_files(&decision.init_args.output)?;
        }
        status = read_bundle_status(&bundle_root, base_url).await?;
    } else if args.repair {
        let repaired_worker_env = repair_bundle_worker_name_env(&bundle_root)?;
        if repaired_worker_env {
            write_agent_profiles(&bundle_root)?;
        }
        status = read_bundle_status(&bundle_root, base_url).await?;
    }
    if args.json {
        print_json(&status)?;
    } else if args.summary {
        println!("{}", render_bundle_status_summary(&status));
    } else {
        println!("{}", render_doctor_status_markdown(&bundle_root, &status));
    }
    Ok(())
}

pub(crate) async fn run_bundle_config_command(
    args: &ConfigArgs,
    base_url: &str,
) -> anyhow::Result<()> {
    let bundle_root = resolve_setup_bundle_root(args.output.as_deref())?;
    if let Some(command) = &args.command {
        return run_bundle_config_subcommand(&bundle_root, command);
    }
    for setting in &args.set {
        apply_bundle_config_setting(&bundle_root, setting)?;
    }
    let project_root = args.project_root.clone().or(detect_current_project_root()?);
    let runtime = read_bundle_runtime_config(&bundle_root)?;
    let status = read_bundle_status(&bundle_root, base_url).await.ok();
    let config = render_bundle_config_snapshot(
        &bundle_root,
        project_root.as_deref(),
        runtime.as_ref(),
        status.as_ref(),
    );
    if args.json {
        print_json(&config)?;
    } else if args.summary {
        println!("{}", render_bundle_config_summary(&config));
    } else {
        println!("{}", render_bundle_config_markdown(&config));
    }
    Ok(())
}

fn run_bundle_config_subcommand(bundle_root: &Path, command: &ConfigCommand) -> anyhow::Result<()> {
    match command {
        ConfigCommand::List(args) => {
            let doc = read_config_json(bundle_root)?;
            let rows = config_setting_rows(&doc);
            if args.json {
                print_json(&rows)?;
            } else {
                for row in rows {
                    println!(
                        "{}={} default={} owner={}",
                        row.key, row.value, row.default, row.owner
                    );
                }
            }
        }
        ConfigCommand::Get(args) => {
            let key = normalize_config_key(&args.key)?;
            let doc = read_config_json(bundle_root)?;
            let row = config_setting_rows(&doc)
                .into_iter()
                .find(|row| row.key == key)
                .expect("validated key exists");
            if args.json {
                print_json(&row)?;
            } else {
                println!("{}", row.value);
            }
        }
        ConfigCommand::Set(args) => {
            let (key, value) = args
                .setting
                .split_once('=')
                .ok_or_else(|| anyhow::anyhow!("config setting must be KEY=VALUE"))?;
            let key = normalize_config_key(key)?;
            let mut doc = read_config_json(bundle_root)?;
            set_config_value(&mut doc, &key, value.trim())?;
            write_config_json(bundle_root, &doc)?;
            let row = config_setting_rows(&doc)
                .into_iter()
                .find(|row| row.key == key)
                .expect("validated key exists");
            if args.json {
                print_json(&row)?;
            } else {
                println!("{}={}", row.key, row.value);
            }
        }
        ConfigCommand::Reset(args) => {
            let mut doc = read_config_json(bundle_root)?;
            if let Some(key) = &args.key {
                let key = normalize_config_key(key)?;
                set_default_config_value(&mut doc, &key)?;
            } else {
                for key in CONFIG_KEYS {
                    set_default_config_value(&mut doc, key)?;
                }
            }
            write_config_json(bundle_root, &doc)?;
            let rows = config_setting_rows(&doc);
            if args.json {
                print_json(&rows)?;
            } else if let Some(key) = &args.key {
                let key = normalize_config_key(key)?;
                let row = rows
                    .into_iter()
                    .find(|row| row.key == key)
                    .expect("validated key exists");
                println!("{}={}", row.key, row.value);
            } else {
                println!("reset {} settings", CONFIG_KEYS.len());
            }
        }
        ConfigCommand::ShowSchema(args) => {
            let schema = config_schema_json();
            if args.json {
                print_json(&schema)?;
            } else {
                println!("{}", serde_json::to_string_pretty(&schema)?);
            }
        }
    }
    Ok(())
}

fn apply_bundle_config_setting(bundle_root: &Path, setting: &str) -> anyhow::Result<()> {
    let (key, value) = setting
        .split_once('=')
        .ok_or_else(|| anyhow::anyhow!("config setting must be KEY=VALUE"))?;
    match key.trim() {
        "auto_commit.enabled" => {
            set_bundle_auto_commit_enabled(bundle_root, parse_config_bool(value.trim())?)
        }
        other => anyhow::bail!("unknown config key '{other}'"),
    }
}

#[derive(Debug, Clone, Serialize)]
struct ConfigSettingRow {
    key: &'static str,
    value: JsonValue,
    default: JsonValue,
    ty: &'static str,
    owner: &'static str,
}

const CONFIG_KEYS: &[&str] = &[
    "auto_commit.enabled",
    "cost_ledger.budget_tokens",
    "cost_ledger.per_turn_warn",
    "provenance.drilldown_depth_max",
    "voice.mode",
    "visibility.default_scope",
    "feature_flags.v11_compiler",
    "feature_flags.v13_sota_guard",
];

fn read_config_json(bundle_root: &Path) -> anyhow::Result<JsonValue> {
    let config_path = bundle_root.join("config.json");
    if !config_path.exists() {
        anyhow::bail!(
            "{} does not exist; initialize the bundle first",
            config_path.display()
        );
    }
    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))
}

fn write_config_json(bundle_root: &Path, doc: &JsonValue) -> anyhow::Result<()> {
    let config_path = bundle_root.join("config.json");
    let tmp_path = config_path.with_extension("json.tmp");
    fs::write(&tmp_path, serde_json::to_string_pretty(doc)? + "\n")
        .with_context(|| format!("write {}", tmp_path.display()))?;
    fs::rename(&tmp_path, &config_path)
        .with_context(|| format!("replace {}", config_path.display()))?;
    Ok(())
}

fn config_setting_rows(doc: &JsonValue) -> Vec<ConfigSettingRow> {
    CONFIG_KEYS
        .iter()
        .map(|key| ConfigSettingRow {
            key,
            value: config_value(doc, key),
            default: config_default_value(key),
            ty: config_type(key),
            owner: config_owner(key),
        })
        .collect()
}

pub(crate) fn config_budget_tokens_from_bundle(output: &Path) -> Option<usize> {
    read_config_json(output).ok().and_then(|doc| {
        config_value(&doc, "cost_ledger.budget_tokens")
            .as_u64()
            .map(|value| value as usize)
    })
}

fn normalize_config_key(key: &str) -> anyhow::Result<&'static str> {
    let key = key.trim();
    if let Some(found) = CONFIG_KEYS.iter().find(|candidate| **candidate == key) {
        return Ok(found);
    }
    let hint = CONFIG_KEYS
        .iter()
        .min_by_key(|candidate| levenshtein(key, candidate))
        .copied()
        .unwrap_or("auto_commit.enabled");
    eprintln!("unknown config key '{key}'; did you mean '{hint}'?");
    Err(anyhow::Error::new(HookEnforceExitCode(2)))
}

fn config_default_value(key: &str) -> JsonValue {
    match key {
        "auto_commit.enabled" => json!(true),
        "cost_ledger.budget_tokens" => json!(4000),
        "cost_ledger.per_turn_warn" => json!(1200),
        "provenance.drilldown_depth_max" => json!(3),
        "voice.mode" => json!(default_voice_mode()),
        "visibility.default_scope" => json!("project"),
        "feature_flags.v11_compiler" => json!(false),
        "feature_flags.v13_sota_guard" => json!(false),
        _ => JsonValue::Null,
    }
}

fn config_type(key: &str) -> &'static str {
    match key {
        "auto_commit.enabled" | "feature_flags.v11_compiler" | "feature_flags.v13_sota_guard" => {
            "bool"
        }
        "cost_ledger.budget_tokens"
        | "cost_ledger.per_turn_warn"
        | "provenance.drilldown_depth_max" => "number",
        "voice.mode" | "visibility.default_scope" => "string",
        _ => "unknown",
    }
}

fn config_owner(key: &str) -> &'static str {
    match key {
        "auto_commit.enabled" => "V7 H7",
        "cost_ledger.budget_tokens" | "cost_ledger.per_turn_warn" => "V8 E8/G8",
        "provenance.drilldown_depth_max" => "V8 D8/G8",
        "voice.mode" => "memd voice bootstrap",
        "visibility.default_scope" => "V9 reserved",
        "feature_flags.v11_compiler" => "V11 reserved",
        "feature_flags.v13_sota_guard" => "V13 reserved",
        _ => "unknown",
    }
}

fn config_value(doc: &JsonValue, key: &str) -> JsonValue {
    match key {
        "voice.mode" => doc
            .get("voice_mode")
            .cloned()
            .unwrap_or_else(|| config_default_value(key)),
        _ => read_nested(doc, key).unwrap_or_else(|| config_default_value(key)),
    }
}

fn read_nested(doc: &JsonValue, key: &str) -> Option<JsonValue> {
    let mut current = doc;
    for part in key.split('.') {
        current = current.get(part)?;
    }
    Some(current.clone())
}

fn set_default_config_value(doc: &mut JsonValue, key: &str) -> anyhow::Result<()> {
    let value = config_default_value(key);
    write_config_value(doc, key, value)
}

fn set_config_value(doc: &mut JsonValue, key: &str, raw: &str) -> anyhow::Result<()> {
    let value = match config_type(key) {
        "bool" => json!(parse_config_bool(raw)?),
        "number" => {
            let parsed: u64 = raw
                .parse()
                .with_context(|| format!("parse numeric config value for {key}"))?;
            json!(parsed)
        }
        "string" => json!(raw.trim().to_string()),
        _ => anyhow::bail!("unsupported config key '{key}'"),
    };
    write_config_value(doc, key, value)
}

fn write_config_value(doc: &mut JsonValue, key: &str, value: JsonValue) -> anyhow::Result<()> {
    if !doc.is_object() {
        *doc = json!({});
    }
    if key == "voice.mode" {
        doc.as_object_mut()
            .expect("object checked")
            .insert("voice_mode".to_string(), value);
        return Ok(());
    }
    let mut current = doc;
    let mut parts = key.split('.').peekable();
    while let Some(part) = parts.next() {
        if parts.peek().is_none() {
            current
                .as_object_mut()
                .expect("object checked")
                .insert(part.to_string(), value);
            return Ok(());
        }
        let obj = current.as_object_mut().expect("object checked");
        current = obj.entry(part).or_insert_with(|| json!({}));
        if !current.is_object() {
            *current = json!({});
        }
    }
    Ok(())
}

fn config_schema_json() -> JsonValue {
    let properties = config_setting_rows(&json!({}))
        .into_iter()
        .map(|row| {
            (
                row.key.to_string(),
                json!({
                    "type": row.ty,
                    "default": row.default,
                    "owner": row.owner,
                }),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "memd runtime settings",
        "schema_version": "0.3",
        "properties": properties,
        "additionalProperties": true
    })
}

fn levenshtein(a: &str, b: &str) -> usize {
    let mut costs = (0..=b.chars().count()).collect::<Vec<_>>();
    for (i, ca) in a.chars().enumerate() {
        let mut last = i;
        costs[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let old = costs[j + 1];
            let substitution = last + usize::from(ca != cb);
            costs[j + 1] = (costs[j + 1] + 1).min(costs[j] + 1).min(substitution);
            last = old;
        }
    }
    costs[b.chars().count()]
}

fn parse_config_bool(value: &str) -> anyhow::Result<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" | "enabled" => Ok(true),
        "0" | "false" | "no" | "off" | "disabled" => Ok(false),
        other => anyhow::bail!("invalid boolean '{other}'"),
    }
}

pub(crate) async fn run_bundle_init_command(args: InitArgs) -> anyhow::Result<()> {
    let args = normalize_init_args(args)?;
    let decision = resolve_bootstrap_authority(args).await?;
    write_init_bundle(&decision.init_args)?;
    if decision.fallback_activated {
        set_bundle_localhost_read_only_authority_state(
            &decision.init_args.output,
            &decision.shared_base_url,
            "init",
            "shared authority unavailable during bootstrap",
        )?;
        write_agent_profiles(&decision.init_args.output)?;
        write_native_agent_bridge_files(&decision.init_args.output)?;
    }
    println!(
        "Initialized memd bundle at {}",
        decision.init_args.output.display()
    );
    if decision.fallback_activated {
        eprintln!("memd authority warning:");
        eprintln!("- shared authority unavailable");
        eprintln!("- localhost fallback is lower trust");
        eprintln!("- prompt-injection and split-brain risk increased");
        eprintln!("- coordination writes blocked");
    }
    Ok(())
}

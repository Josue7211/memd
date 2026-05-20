pub(crate) fn build_skill_lifecycle_report(policy: &MemoryPolicyResponse) -> SkillLifecycleReport {
    let registry = build_bundle_capability_registry(None);
    let bridges = detect_capability_bridges();
    let bridge_lookup = bridges
        .actions
        .iter()
        .map(|action| {
            (
                (action.harness.clone(), action.capability.clone()),
                (action.status.as_str(), action.target_path.as_str()),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let runtime_is_defaulted = is_default_runtime(&policy.runtime);
    let low_risk_threshold = 0.25_f32;

    let mut records = Vec::new();
    let mut proposed = 0usize;
    let mut sandbox_passed = 0usize;
    let mut sandbox_review = 0usize;
    let mut sandbox_blocked = 0usize;
    let mut activation_candidates = 0usize;
    let mut activated = 0usize;
    let mut review_queue = Vec::new();
    let mut activate_queue = Vec::new();

    for capability in registry
        .capabilities
        .iter()
        .filter(|capability| capability.kind == "skill" || capability.kind == "skill-bridge")
    {
        proposed += 1;
        let proposal = if capability.status == "installed" || capability.status == "enabled" {
            "proposed"
        } else {
            "staged"
        };

        let bridge_state = bridge_lookup
            .get(&(capability.harness.clone(), capability.name.clone()))
            .copied();
        let (sandbox, sandbox_risk, sandbox_reason) = score_skill_sandbox(capability, bridge_state);
        if sandbox == "pass" {
            sandbox_passed += 1;
        } else if sandbox == "review" {
            sandbox_review += 1;
        } else if sandbox == "block" {
            sandbox_blocked += 1;
        }

        let policy_allows_activation =
            !runtime_is_defaulted && policy.runtime.skill_gating.gated_activation;
        let activation = if !policy_allows_activation {
            "review"
        } else if sandbox == "pass"
            && policy.runtime.skill_gating.sandboxed_evaluation
            && (!policy.runtime.skill_gating.auto_activate_low_risk_only
                || sandbox_risk <= low_risk_threshold)
        {
            activated += 1;
            activation_candidates += 1;
            "activate"
        } else if sandbox == "pass" {
            activation_candidates += 1;
            "candidate"
        } else {
            "hold"
        };
        let activation_reason = match activation {
            "activate" => "low-risk sandbox passed and policy allowed auto-activation",
            "candidate" => "sandbox passed but policy still wants explicit activation",
            "review" if runtime_is_defaulted => {
                "legacy backend defaults require review before activation"
            }
            "review" => "policy gate requires review before activation",
            _ => "sandbox did not pass",
        };

        let record = SkillLifecycleRecord {
            harness: capability.harness.clone(),
            name: capability.name.clone(),
            kind: capability.kind.clone(),
            portability_class: capability.portability_class.clone(),
            proposal: proposal.to_string(),
            sandbox: sandbox.to_string(),
            sandbox_risk,
            sandbox_reason,
            activation: activation.to_string(),
            activation_reason: activation_reason.to_string(),
            source_path: capability.source_path.clone(),
            target_path: bridge_state.map(|state| state.1.to_string()),
            notes: capability.notes.clone(),
        };
        if activation == "activate" {
            activate_queue.push(record.clone());
        } else {
            review_queue.push(record.clone());
        }
        records.push(record);
    }

    records.sort_by(|a, b| {
        a.harness
            .cmp(&b.harness)
            .then(a.kind.cmp(&b.kind))
            .then(a.name.cmp(&b.name))
    });

    SkillLifecycleReport {
        generated_at: Utc::now(),
        proposed,
        sandbox_passed,
        sandbox_review,
        sandbox_blocked,
        activation_candidates,
        activated,
        review_queue,
        activate_queue,
        records,
    }
}

pub(crate) fn score_skill_sandbox(
    capability: &CapabilityRecord,
    bridge_state: Option<(&str, &str)>,
) -> (&'static str, f32, String) {
    let mut risk: f32;
    let mut reasons = Vec::new();

    match capability.portability_class.as_str() {
        "universal" => {
            risk = 0.05;
            reasons.push("portable".to_string());
        }
        class if class.contains("bridgeable") => {
            risk = 0.20;
            reasons.push("bridgeable".to_string());
            match bridge_state.map(|state| state.0) {
                Some("bridged") | Some("already-bridged") => {
                    risk -= 0.12;
                    reasons.push("bridge_ready".to_string());
                }
                Some("blocked") => {
                    risk += 0.20;
                    reasons.push("bridge_blocked".to_string());
                }
                _ => {
                    reasons.push("bridge_pending".to_string());
                }
            }
        }
        "harness-native" => {
            risk = 0.38;
            reasons.push("harness_native".to_string());
        }
        other => {
            risk = 0.82;
            reasons.push(format!("portability={other}"));
        }
    }

    if capability.status == "installed" || capability.status == "enabled" {
        risk -= 0.03;
        reasons.push("present".to_string());
    }
    if capability.hash.is_some() {
        risk -= 0.01;
        reasons.push("hashed".to_string());
    }
    if capability
        .notes
        .iter()
        .any(|note| note.contains("active Codex bridge"))
    {
        risk -= 0.04;
        reasons.push("active_bridge".to_string());
    }

    risk = risk.clamp(0.0, 1.0);
    let sandbox = if risk <= 0.15 {
        "pass"
    } else if risk <= 0.5 {
        "review"
    } else {
        "block"
    };

    (sandbox, risk, reasons.join(";"))
}

pub(crate) fn render_skill_lifecycle_report(report: &SkillLifecycleReport, follow: bool) -> String {
    let mut markdown = String::new();
    markdown.push_str("## Skill Lifecycle\n\n");
    markdown.push_str(&format!(
        "- proposed: {}\n- sandbox_passed: {}\n- sandbox_review: {}\n- sandbox_blocked: {}\n- review_queue: {}\n- activate_queue: {}\n- activation_candidates: {}\n- activated: {}\n",
        report.proposed,
        report.sandbox_passed,
        report.sandbox_review,
        report.sandbox_blocked,
        report.review_queue.len(),
        report.activate_queue.len(),
        report.activation_candidates,
        report.activated
    ));

    if !report.activate_queue.is_empty() {
        markdown.push_str("\n### Activate Queue\n\n");
        for record in report
            .activate_queue
            .iter()
            .take(if follow { 12 } else { 8 })
        {
            markdown.push_str(&format!(
                "- {} / {} / {} risk={:.2} sandbox={} activation={} reason={}",
                record.harness,
                record.kind,
                record.name,
                record.sandbox_risk,
                record.sandbox,
                record.activation,
                record.activation_reason
            ));
            if let Some(target) = record.target_path.as_deref() {
                markdown.push_str(&format!(" -> {}", target));
            }
            if follow && !record.notes.is_empty() {
                markdown.push_str(&format!(" notes={}", record.notes.join(" | ")));
            }
            markdown.push('\n');
        }
    }

    if !report.review_queue.is_empty() {
        markdown.push_str("\n### Review Queue\n\n");
        for record in report.review_queue.iter().take(if follow { 12 } else { 8 }) {
            markdown.push_str(&format!(
                "- {} / {} / {} risk={:.2} sandbox={} activation={} reason={}",
                record.harness,
                record.kind,
                record.name,
                record.sandbox_risk,
                record.sandbox,
                record.activation,
                record.activation_reason
            ));
            if let Some(target) = record.target_path.as_deref() {
                markdown.push_str(&format!(" -> {}", target));
            }
            if follow && !record.notes.is_empty() {
                markdown.push_str(&format!(" notes={}", record.notes.join(" | ")));
            }
            markdown.push('\n');
        }
    }

    if follow && !report.records.is_empty() {
        markdown.push_str("\n### Lifecycle records\n\n");
        for record in report.records.iter().take(if follow { 12 } else { 8 }) {
            markdown.push_str(&format!(
                "- {} / {} / {} [{}] proposal={} sandbox={} risk={:.2} activation={}",
                record.harness,
                record.kind,
                record.name,
                record.portability_class,
                record.proposal,
                record.sandbox,
                record.sandbox_risk,
                record.activation
            ));
            if let Some(target) = record.target_path.as_deref() {
                markdown.push_str(&format!(" -> {}", target));
            }
            markdown.push_str(&format!(" reason={}", record.sandbox_reason));
            markdown.push_str(&format!(" activation_reason={}", record.activation_reason));
            if follow && !record.notes.is_empty() {
                markdown.push_str(&format!(" notes={}", record.notes.join(" | ")));
            }
            markdown.push('\n');
        }
    }

    markdown
}

pub(crate) fn render_skill_policy_batch_markdown(batch: &SkillPolicyBatchArtifact) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd skill policy batch\n\n");
    markdown.push_str(&format!(
        "- generated_at: {}\n- bundle_root: {}\n- runtime_defaulted: {}\n- proposed: {}\n- sandbox_passed: {}\n- sandbox_review: {}\n- sandbox_blocked: {}\n- review_queue: {}\n- activate_queue: {}\n- activation_candidates: {}\n- activated: {}\n",
        batch.generated_at.to_rfc3339(),
        batch.bundle_root,
        batch.runtime_defaulted,
        batch.report.proposed,
        batch.report.sandbox_passed,
        batch.report.sandbox_review,
        batch.report.sandbox_blocked,
        batch.report.review_queue.len(),
        batch.report.activate_queue.len(),
        batch.report.activation_candidates,
        batch.report.activated
    ));
    markdown.push_str("\n## Apply Flow\n\n");
    markdown.push_str(
        "Use the activate queue after sandbox review. Keep review queue as the manual follow-up set.\n",
    );
    if !batch.report.activate_queue.is_empty() {
        markdown.push_str("\n### Activate Queue\n\n");
        for record in batch.report.activate_queue.iter().take(12) {
            markdown.push_str(&format!(
                "- {} / {} / {} risk={:.2} sandbox={} activation={} reason={}",
                record.harness,
                record.kind,
                record.name,
                record.sandbox_risk,
                record.sandbox,
                record.activation,
                record.activation_reason
            ));
            if let Some(target) = record.target_path.as_deref() {
                markdown.push_str(&format!(" -> {}", target));
            }
            markdown.push('\n');
        }
    }
    if !batch.report.review_queue.is_empty() {
        markdown.push_str("\n### Review Queue\n\n");
        for record in batch.report.review_queue.iter().take(12) {
            markdown.push_str(&format!(
                "- {} / {} / {} risk={:.2} sandbox={} activation={} reason={}",
                record.harness,
                record.kind,
                record.name,
                record.sandbox_risk,
                record.sandbox,
                record.activation,
                record.activation_reason
            ));
            if let Some(target) = record.target_path.as_deref() {
                markdown.push_str(&format!(" -> {}", target));
            }
            markdown.push('\n');
        }
    }
    markdown
}

pub(crate) fn render_skill_policy_queue_markdown(queue: &SkillPolicyQueueArtifact) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!(
        "# memd skill policy {} queue\n\n- generated_at: {}\n- bundle_root: {}\n- runtime_defaulted: {}\n- records: {}\n",
        queue.queue,
        queue.generated_at.to_rfc3339(),
        queue.bundle_root,
        queue.runtime_defaulted,
        queue.records.len()
    ));
    if !queue.records.is_empty() {
        markdown.push_str("\n## Records\n\n");
        for record in queue.records.iter().take(16) {
            markdown.push_str(&format!(
                "- {} / {} / {} risk={:.2} sandbox={} activation={} reason={}",
                record.harness,
                record.kind,
                record.name,
                record.sandbox_risk,
                record.sandbox,
                record.activation,
                record.activation_reason
            ));
            if let Some(target) = record.target_path.as_deref() {
                markdown.push_str(&format!(" -> {}", target));
            }
            markdown.push('\n');
        }
    }
    markdown
}

pub(crate) fn render_skill_policy_apply_markdown(receipt: &SkillPolicyApplyArtifact) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd skill policy apply receipt\n\n");
    markdown.push_str(&format!(
        "- generated_at: {}\n- bundle_root: {}\n- runtime_defaulted: {}\n- source_queue_path: {}\n- applied_count: {}\n- skipped_count: {}\n",
        receipt.generated_at.to_rfc3339(),
        receipt.bundle_root,
        receipt.runtime_defaulted,
        receipt.source_queue_path,
        receipt.applied_count,
        receipt.skipped_count
    ));
    if !receipt.applied.is_empty() {
        markdown.push_str("\n## Applied\n\n");
        for record in receipt.applied.iter().take(16) {
            markdown.push_str(&format!(
                "- {} / {} / {} -> {}",
                record.harness, record.kind, record.name, record.activation_reason
            ));
            if let Some(target) = record.target_path.as_deref() {
                markdown.push_str(&format!(" -> {}", target));
            }
            markdown.push('\n');
        }
    }
    if !receipt.skipped.is_empty() {
        markdown.push_str("\n## Skipped\n\n");
        for record in receipt.skipped.iter().take(16) {
            markdown.push_str(&format!(
                "- {} / {} / {} -> {}",
                record.harness, record.kind, record.name, record.activation_reason
            ));
            if let Some(target) = record.target_path.as_deref() {
                markdown.push_str(&format!(" -> {}", target));
            }
            markdown.push('\n');
        }
    }
    markdown
}

/// Wire memd bootstrap hooks into every detected Claude-family harness settings.json.
///
/// For each harness root (e.g. `~/.claude`), ensures `settings.json` contains a
/// `UserPromptSubmit` hook entry that runs `memd-hook-bootstrap`. This is the
/// firmware-level enforcement: the hook fires before the model sees any message,
/// so the agent cannot skip memd wake.
pub(crate) fn ensure_harness_bootstrap_hooks(output: &Path) -> Vec<String> {
    let home = match home_dir() {
        Some(home) => home,
        None => return vec!["skip: could not determine home directory".to_string()],
    };

    let roots = detect_claude_family_harness_roots(&home);
    let mut results = Vec::new();

    for harness_root in &roots {
        match ensure_settings_bootstrap_hook(output, &harness_root.root) {
            Ok(action) => results.push(format!("{}: {}", harness_root.harness, action)),
            Err(err) => results.push(format!("{}: error: {}", harness_root.harness, err)),
        }
    }

    results
}

fn ensure_settings_bootstrap_hook(
    bundle_output: &Path,
    harness_root: &Path,
) -> anyhow::Result<String> {
    let settings_path = harness_root.join("settings.json");
    let hook_script = bundle_output.join("hooks").join("memd-bootstrap.sh");
    let hook_command = format!("bash \"{}\"", hook_script.display());

    let mut doc: serde_json::Value = if settings_path.is_file() {
        let content = fs::read_to_string(&settings_path)
            .with_context(|| format!("read {}", settings_path.display()))?;
        serde_json::from_str(&content)
            .with_context(|| format!("parse {}", settings_path.display()))?
    } else {
        serde_json::json!({})
    };

    let hooks = doc
        .as_object_mut()
        .context("settings root is not an object")?
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));

    let events = hooks
        .as_object_mut()
        .context("hooks is not an object")?
        .entry("UserPromptSubmit")
        .or_insert_with(|| serde_json::json!([]));

    let event_array = events
        .as_array_mut()
        .context("UserPromptSubmit is not an array")?;

    let already_wired = event_array.iter().any(|entry| {
        entry
            .get("hooks")
            .and_then(|h| h.as_array())
            .map(|hooks| {
                hooks.iter().any(|hook| {
                    hook.get("command")
                        .and_then(|c| c.as_str())
                        .map(|c| c.contains("memd-bootstrap") || c.contains("memd-hook-bootstrap"))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    });

    if already_wired {
        return Ok("already wired".to_string());
    }

    let hook_entry = serde_json::json!({
        "hooks": [{
            "type": "command",
            "command": hook_command,
            "timeout": 15
        }]
    });
    event_array.push(hook_entry);

    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let formatted = serde_json::to_string_pretty(&doc)?;
    fs::write(&settings_path, format!("{formatted}\n"))
        .with_context(|| format!("write {}", settings_path.display()))?;

    Ok("wired".to_string())
}

pub(crate) fn render_bootstrap_hook_summary(results: &[String]) -> String {
    if results.is_empty() {
        return String::new();
    }
    let mut output = String::from("bootstrap hooks:\n");
    for result in results {
        output.push_str(&format!("  - {result}\n"));
    }
    output
}

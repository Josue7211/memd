use super::*;

pub(crate) async fn lookup_with_fallbacks(
    client: &MemdClient,
    req: &SearchMemoryRequest,
    query: &str,
) -> anyhow::Result<memd_schema::SearchMemoryResponse> {
    let response = client.search(req).await?;
    if !response.items.is_empty() {
        return Ok(response);
    }

    for candidate_query in supersede_query_candidates(query).into_iter().skip(1) {
        let fallback = client
            .search(&SearchMemoryRequest {
                query: Some(candidate_query.clone()),
                ..req.clone()
            })
            .await?;
        if !fallback.items.is_empty() {
            return Ok(fallback);
        }
    }

    if !should_use_broad_lookup_fallback(req, query) {
        return Ok(response);
    }

    let broad_response = client
        .search(&SearchMemoryRequest {
            query: None,
            ..req.clone()
        })
        .await?;
    let broad_fallback = rerank_broad_lookup_fallback(&broad_response, req, query);
    if !broad_fallback.items.is_empty() {
        return Ok(broad_fallback);
    }
    if !req.tags.is_empty() && !broad_response.items.is_empty() {
        return Ok(broad_response);
    }

    Ok(response)
}

pub(crate) fn build_lookup_request(
    args: &LookupArgs,
    runtime: Option<&BundleRuntimeConfig>,
) -> anyhow::Result<SearchMemoryRequest> {
    let project = args
        .project
        .clone()
        .or_else(|| runtime.and_then(|config| config.project.clone()));
    let namespace = args
        .namespace
        .clone()
        .or_else(|| runtime.and_then(|config| config.namespace.clone()));
    let workspace = args
        .workspace
        .clone()
        .or_else(|| runtime.and_then(|config| config.workspace.clone()));
    let visibility = args
        .visibility
        .clone()
        .or_else(|| runtime.and_then(|config| config.visibility.clone()))
        .as_deref()
        .map(parse_memory_visibility_value)
        .transpose()?;
    let route = parse_retrieval_route(args.route.clone().or(Some("project_first".to_string())))?;
    let intent = parse_retrieval_intent(args.intent.clone().or(Some("general".to_string())))?
        .unwrap_or(RetrievalIntent::General);
    let kinds = if args.kind.is_empty() {
        default_kinds_for_intent(intent)
    } else {
        args.kind
            .iter()
            .map(|value| parse_memory_kind_value(value))
            .collect::<anyhow::Result<Vec<_>>>()?
    };
    let mut statuses = vec![MemoryStatus::Active];
    if args.include_stale {
        statuses.push(MemoryStatus::Stale);
        statuses.push(MemoryStatus::Contested);
    }

    Ok(SearchMemoryRequest {
        query: Some(args.query.clone()),
        route,
        intent: Some(intent),
        scopes: vec![
            MemoryScope::Project,
            MemoryScope::Synced,
            MemoryScope::Global,
        ],
        kinds,
        statuses,
        project,
        namespace,
        workspace,
        visibility,
        belief_branch: None,
        source_agent: None,
        tags: args.tag.clone(),
        stages: vec![MemoryStage::Canonical, MemoryStage::Candidate],
        limit: args.limit.or(Some(6)),
        max_chars_per_item: Some(280),
    })
}

pub(crate) fn should_use_broad_lookup_fallback(req: &SearchMemoryRequest, query: &str) -> bool {
    !req.tags.is_empty() || lookup_query_terms(query).len() >= 3
}

pub(crate) fn lookup_query_terms(query: &str) -> Vec<String> {
    let mut terms = query
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .map(|term| term.trim().to_ascii_lowercase())
        .filter(|term| term.len() >= 3)
        .collect::<Vec<_>>();
    terms.sort();
    terms.dedup();
    terms
}

fn lookup_item_search_text(item: &memd_schema::MemoryItem) -> String {
    let mut text = item.content.to_ascii_lowercase();
    if let Some(source_path) = item.source_path.as_deref() {
        text.push(' ');
        text.push_str(&source_path.to_ascii_lowercase());
    }
    for tag in &item.tags {
        text.push(' ');
        text.push_str(&tag.to_ascii_lowercase());
    }
    text
}

fn broad_lookup_score(
    item: &memd_schema::MemoryItem,
    req: &SearchMemoryRequest,
    query: &str,
) -> i64 {
    let query_lower = query.to_ascii_lowercase();
    let item_text = lookup_item_search_text(item);
    let mut score = 0i64;

    if !query_lower.trim().is_empty() && item_text.contains(query_lower.trim()) {
        score += 8;
    }

    for term in lookup_query_terms(query) {
        if item_text.contains(&term) {
            score += 2;
        }
    }

    for tag in &req.tags {
        if item
            .tags
            .iter()
            .any(|item_tag| item_tag.eq_ignore_ascii_case(tag))
        {
            score += 4;
        }
    }

    if req.kinds.iter().any(|kind| *kind == item.kind) {
        score += 1;
    }

    score
}

pub(crate) fn rerank_broad_lookup_fallback(
    response: &memd_schema::SearchMemoryResponse,
    req: &SearchMemoryRequest,
    query: &str,
) -> memd_schema::SearchMemoryResponse {
    let mut ranked = response
        .items
        .iter()
        .cloned()
        .filter_map(|item| {
            let score = broad_lookup_score(&item, req, query);
            (score > 0).then_some((score, item))
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|(left_score, left_item), (right_score, right_item)| {
        right_score.cmp(left_score).then_with(|| {
            right_item
                .confidence
                .partial_cmp(&left_item.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });

    memd_schema::SearchMemoryResponse {
        route: response.route,
        intent: response.intent,
        items: ranked
            .into_iter()
            .map(|(_, item)| item)
            .take(req.limit.unwrap_or(6))
            .collect(),
    }
}

pub(crate) fn render_lookup_markdown(
    query: &str,
    request: &SearchMemoryRequest,
    response: &memd_schema::SearchMemoryResponse,
    verbose: bool,
) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd lookup\n\n");
    markdown.push_str(&format!("- query: {}\n", query));
    markdown.push_str(&format!(
        "- plan: intent={} route={} scopes={} types={}\n",
        enum_label_intent(request.intent.unwrap_or(response.intent)),
        enum_label_route(request.route.unwrap_or(response.route)),
        request
            .scopes
            .iter()
            .map(|scope| enum_label_scope(*scope))
            .collect::<Vec<_>>()
            .join(" -> "),
        typed_targets_for_request(request).join(", ")
    ));
    markdown.push_str(&format!("- matches: {}\n\n", response.items.len()));

    if response.items.is_empty() {
        markdown.push_str("No durable memory matched.\n");
        return markdown;
    }

    markdown.push_str("## Matches\n\n");
    let limit = if verbose { 6 } else { 3 };
    for item in response.items.iter().take(limit) {
        markdown.push_str(&format!(
            "- [{}] {} ({}, type={}, {}, {:.2})\n",
            short_uuid(item.id),
            item.content.replace('\n', " "),
            enum_label_kind(item.kind),
            typed_memory_label(item.kind, item.stage),
            enum_label_status(item.status),
            item.confidence
        ));
    }

    markdown
        .push_str("\n- Use recalled items before answering; correct memory if they conflict.\n");
    markdown
}

pub(crate) fn enum_label_kind(kind: MemoryKind) -> &'static str {
    match kind {
        MemoryKind::Fact => "fact",
        MemoryKind::Decision => "decision",
        MemoryKind::Preference => "preference",
        MemoryKind::Runbook => "runbook",
        MemoryKind::Procedural => "procedural",
        MemoryKind::SelfModel => "self_model",
        MemoryKind::Topology => "topology",
        MemoryKind::Status => "status",
        MemoryKind::LiveTruth => "live_truth",
        MemoryKind::Pattern => "pattern",
        MemoryKind::Constraint => "constraint",
    }
}

pub(crate) fn enum_label_scope(scope: MemoryScope) -> &'static str {
    match scope {
        MemoryScope::Local => "local",
        MemoryScope::Synced => "synced",
        MemoryScope::Project => "project",
        MemoryScope::Global => "global",
    }
}

pub(crate) fn enum_label_route(route: RetrievalRoute) -> &'static str {
    match route {
        RetrievalRoute::Auto => "auto",
        RetrievalRoute::LocalOnly => "local_only",
        RetrievalRoute::SyncedOnly => "synced_only",
        RetrievalRoute::ProjectOnly => "project_only",
        RetrievalRoute::GlobalOnly => "global_only",
        RetrievalRoute::LocalFirst => "local_first",
        RetrievalRoute::SyncedFirst => "synced_first",
        RetrievalRoute::ProjectFirst => "project_first",
        RetrievalRoute::GlobalFirst => "global_first",
        RetrievalRoute::All => "all",
    }
}

pub(crate) fn enum_label_intent(intent: RetrievalIntent) -> &'static str {
    match intent {
        RetrievalIntent::General => "general",
        RetrievalIntent::CurrentTask => "current_task",
        RetrievalIntent::Decision => "decision",
        RetrievalIntent::Runbook => "runbook",
        RetrievalIntent::Procedural => "procedural",
        RetrievalIntent::SelfModel => "self_model",
        RetrievalIntent::Topology => "topology",
        RetrievalIntent::Preference => "preference",
        RetrievalIntent::Fact => "fact",
        RetrievalIntent::Pattern => "pattern",
    }
}

pub(crate) fn default_kinds_for_intent(intent: RetrievalIntent) -> Vec<MemoryKind> {
    match intent {
        RetrievalIntent::CurrentTask => vec![
            MemoryKind::Status,
            MemoryKind::Decision,
            MemoryKind::Constraint,
            MemoryKind::Pattern,
            MemoryKind::LiveTruth,
        ],
        RetrievalIntent::Decision => vec![MemoryKind::Decision, MemoryKind::Constraint],
        RetrievalIntent::Runbook => vec![MemoryKind::Runbook, MemoryKind::Procedural],
        RetrievalIntent::Procedural => vec![MemoryKind::Procedural, MemoryKind::Runbook],
        RetrievalIntent::SelfModel => vec![MemoryKind::SelfModel, MemoryKind::Preference],
        RetrievalIntent::Topology => vec![MemoryKind::Topology, MemoryKind::Decision],
        RetrievalIntent::Preference => vec![MemoryKind::Preference, MemoryKind::Decision],
        RetrievalIntent::Fact => vec![
            MemoryKind::Fact,
            MemoryKind::LiveTruth,
            MemoryKind::Constraint,
        ],
        RetrievalIntent::Pattern => vec![MemoryKind::Pattern, MemoryKind::Fact],
        RetrievalIntent::General => vec![
            MemoryKind::Decision,
            MemoryKind::Preference,
            MemoryKind::Fact,
            MemoryKind::Constraint,
            MemoryKind::Runbook,
            MemoryKind::Procedural,
            MemoryKind::Status,
            MemoryKind::Pattern,
        ],
    }
}

pub(crate) fn typed_memory_axes(kind: MemoryKind, stage: MemoryStage) -> Vec<&'static str> {
    let mut axes = Vec::new();
    match kind {
        MemoryKind::Runbook | MemoryKind::Procedural => axes.push("procedural"),
        MemoryKind::Status => axes.push("session_continuity"),
        MemoryKind::Pattern => axes.push("episodic"),
        MemoryKind::Fact
        | MemoryKind::Decision
        | MemoryKind::Preference
        | MemoryKind::SelfModel
        | MemoryKind::Topology
        | MemoryKind::LiveTruth
        | MemoryKind::Constraint => axes.push("semantic"),
    }
    match stage {
        MemoryStage::Candidate => axes.push("candidate"),
        MemoryStage::Canonical => axes.push("canonical"),
    }
    axes
}

pub(crate) fn typed_memory_label(kind: MemoryKind, stage: MemoryStage) -> String {
    typed_memory_axes(kind, stage).join("+")
}

pub(crate) fn typed_targets_for_request(req: &SearchMemoryRequest) -> Vec<String> {
    let mut targets = req
        .kinds
        .iter()
        .flat_map(|kind| {
            req.stages
                .iter()
                .map(|stage| typed_memory_label(*kind, *stage))
        })
        .collect::<Vec<_>>();
    targets.sort();
    targets.dedup();
    targets
}

pub(crate) fn enum_label_status(status: MemoryStatus) -> &'static str {
    match status {
        MemoryStatus::Active => "active",
        MemoryStatus::Stale => "stale",
        MemoryStatus::Superseded => "superseded",
        MemoryStatus::Contested => "contested",
        MemoryStatus::Expired => "expired",
    }
}

pub(crate) fn resolve_default_bundle_root() -> anyhow::Result<Option<PathBuf>> {
    if let Ok(value) = std::env::var("MEMD_BUNDLE_ROOT") {
        let value = value.trim();
        if !value.is_empty() {
            return Ok(Some(PathBuf::from(value)));
        }
    }

    let cwd = std::env::current_dir().context("read current directory")?;
    let bundle_root = cwd.join(".memd");
    if bundle_root.join("config.json").exists() {
        return Ok(Some(bundle_root));
    }

    if let Some(project_root) = find_project_root(&cwd) {
        let project_bundle = project_root.join(".memd");
        if project_bundle.join("config.json").exists() {
            return Ok(Some(project_bundle));
        }
        return Ok(None);
    }

    let global_root = default_global_bundle_root();
    if global_root.join("config.json").exists() {
        return Ok(Some(global_root));
    }

    Ok(None)
}

pub(crate) fn resolve_rag_url(
    explicit: Option<String>,
    bundle_root: Option<&Path>,
) -> anyhow::Result<String> {
    if let Some(value) = explicit
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return Ok(value);
    }

    if let Some(bundle_root) = bundle_root
        && let Some(config) = read_bundle_rag_config(bundle_root)?
        && config.enabled
    {
        if let Some(url) = config.url {
            return Ok(url);
        }
        anyhow::bail!(
            "rag backend is enabled in {} but no url was configured",
            bundle_root.display()
        );
    }

    if let Ok(value) = std::env::var("MEMD_RAG_URL") {
        let value = value.trim().to_string();
        if !value.is_empty() {
            return Ok(value);
        }
    }

    anyhow::bail!("provide --rag-url, configure rag_url in the bundle, or set MEMD_RAG_URL")
}

pub(crate) fn maybe_rag_client_from_bundle_or_env() -> anyhow::Result<Option<RagClient>> {
    if let Some(bundle_root) = resolve_default_bundle_root()?
        && let Some(config) = read_bundle_rag_config(bundle_root.as_path())?
        && config.enabled
    {
        let rag_url = config.url.with_context(|| {
            format!(
                "rag backend is enabled in {} but no url was configured",
                bundle_root.display()
            )
        })?;
        return Ok(Some(RagClient::new(rag_url)?));
    }

    match std::env::var("MEMD_RAG_URL") {
        Ok(value) if !value.trim().is_empty() => Ok(Some(RagClient::new(value)?)),
        _ => Ok(None),
    }
}

pub(crate) fn maybe_rag_client_for_bundle(output: &Path) -> anyhow::Result<Option<RagClient>> {
    if let Some(config) = read_bundle_rag_config(output)?
        && config.enabled
    {
        let rag_url = config.url.with_context(|| {
            format!(
                "rag backend is enabled in {} but no url was configured",
                output.display()
            )
        })?;
        return Ok(Some(RagClient::new(rag_url)?));
    }

    match std::env::var("MEMD_RAG_URL") {
        Ok(value) if !value.trim().is_empty() => Ok(Some(RagClient::new(value)?)),
        _ => Ok(None),
    }
}

pub(crate) fn build_resume_rag_query(
    project: Option<&str>,
    workspace: Option<&str>,
    intent: &str,
    working: &memd_schema::WorkingMemoryResponse,
    context: &memd_schema::CompactContextResponse,
) -> String {
    let mut parts = Vec::new();

    if let Some(project) = project.filter(|value| !value.trim().is_empty()) {
        parts.push(format!("project: {project}"));
    }
    if let Some(workspace) = workspace.filter(|value| !value.trim().is_empty()) {
        parts.push(format!("workspace: {workspace}"));
    }
    if !intent.trim().is_empty() {
        parts.push(format!("intent: {intent}"));
    }

    for record in working.records.iter().take(2) {
        let value = compact_resume_rag_text(&record.record, 180);
        if !value.is_empty() {
            parts.push(format!("working: {value}"));
        }
    }

    for record in context.records.iter().take(2) {
        let value = compact_resume_rag_text(&record.record, 180);
        if !value.is_empty() {
            parts.push(format!("context: {value}"));
        }
    }

    compact_resume_rag_text(&parts.join(" | "), 700)
}

pub(crate) fn compact_resume_rag_text(input: &str, max_chars: usize) -> String {
    let collapsed = input.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.len() <= max_chars {
        return collapsed;
    }

    let mut output = String::new();
    for ch in collapsed.chars() {
        if output.chars().count() >= max_chars.saturating_sub(1) {
            break;
        }
        output.push(ch);
    }
    output.push('…');
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    static CWD_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn lock_env_mutation() -> std::sync::MutexGuard<'static, ()> {
        ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("env mutation lock poisoned")
    }

    fn lock_cwd_mutation() -> std::sync::MutexGuard<'static, ()> {
        CWD_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("cwd mutation lock poisoned")
    }

    #[test]
    fn resolve_default_bundle_root_prefers_repo_bundle_over_global_bundle() {
        let _env_lock = lock_env_mutation();
        let _cwd_lock = lock_cwd_mutation();
        let root =
            std::env::temp_dir().join(format!("memd-default-bundle-root-{}", uuid::Uuid::new_v4()));
        let home = root.join("home");
        let repo = root.join("repo");
        let repo_bundle = repo.join(".memd");
        let global_bundle = home.join(".memd");
        let original_home = std::env::var_os("HOME");
        let original_bundle_root = std::env::var_os("MEMD_BUNDLE_ROOT");
        let original_cwd = std::env::current_dir().expect("read current dir");

        fs::create_dir_all(&repo_bundle).expect("create repo bundle");
        fs::create_dir_all(&global_bundle).expect("create global bundle");
        fs::write(repo_bundle.join("config.json"), "{}\n").expect("write repo config");
        fs::write(global_bundle.join("config.json"), "{}\n").expect("write global config");

        unsafe {
            std::env::set_var("HOME", &home);
            std::env::remove_var("MEMD_BUNDLE_ROOT");
        }
        std::env::set_current_dir(&repo).expect("set current dir to repo");

        let resolved = resolve_default_bundle_root()
            .expect("resolve bundle root")
            .expect("bundle root should exist");
        assert_eq!(resolved, repo_bundle);

        std::env::set_current_dir(&original_cwd).expect("restore current dir");
        unsafe {
            match original_home {
                Some(value) => std::env::set_var("HOME", value),
                None => std::env::remove_var("HOME"),
            }
            match original_bundle_root {
                Some(value) => std::env::set_var("MEMD_BUNDLE_ROOT", value),
                None => std::env::remove_var("MEMD_BUNDLE_ROOT"),
            }
        }
        fs::remove_dir_all(root).expect("cleanup temp bundle roots");
    }

    #[test]
    fn typed_memory_label_maps_kind_and_stage_into_top_level_model() {
        assert_eq!(
            typed_memory_label(MemoryKind::Decision, MemoryStage::Canonical),
            "semantic+canonical"
        );
        assert_eq!(
            typed_memory_label(MemoryKind::Procedural, MemoryStage::Candidate),
            "procedural+candidate"
        );
        assert_eq!(
            typed_memory_label(MemoryKind::Status, MemoryStage::Candidate),
            "session_continuity+candidate"
        );
        assert_eq!(
            typed_memory_label(MemoryKind::Pattern, MemoryStage::Canonical),
            "episodic+canonical"
        );
    }

    #[test]
    fn default_kinds_for_current_task_include_continuity_and_episode_signal() {
        let kinds = default_kinds_for_intent(RetrievalIntent::CurrentTask);
        assert!(kinds.contains(&MemoryKind::Status));
        assert!(kinds.contains(&MemoryKind::Pattern));
        assert!(kinds.contains(&MemoryKind::Decision));
    }

    #[test]
    fn default_kinds_for_procedural_focus_on_runbooks_and_procedures() {
        let kinds = default_kinds_for_intent(RetrievalIntent::Procedural);
        assert_eq!(kinds, vec![MemoryKind::Procedural, MemoryKind::Runbook]);
    }

    #[test]
    fn default_kinds_for_fact_keep_truth_signal_over_workflow_noise() {
        let kinds = default_kinds_for_intent(RetrievalIntent::Fact);
        assert!(kinds.contains(&MemoryKind::Fact));
        assert!(kinds.contains(&MemoryKind::LiveTruth));
        assert!(!kinds.contains(&MemoryKind::Runbook));
    }

    #[test]
    fn resolve_default_bundle_root_does_not_fall_back_to_global_inside_repo_without_local_bundle() {
        let _env_lock = lock_env_mutation();
        let _cwd_lock = lock_cwd_mutation();
        let root =
            std::env::temp_dir().join(format!("memd-no-global-fallback-{}", uuid::Uuid::new_v4()));
        let home = root.join("home");
        let repo = root.join("repo");
        let nested = repo.join("src").join("feature");
        let global_bundle = home.join(".memd");
        let original_home = std::env::var_os("HOME");
        let original_bundle_root = std::env::var_os("MEMD_BUNDLE_ROOT");
        let original_cwd = std::env::current_dir().expect("read current dir");

        fs::create_dir_all(&nested).expect("create nested repo dir");
        fs::create_dir_all(repo.join(".git")).expect("create repo git dir");
        fs::create_dir_all(&global_bundle).expect("create global bundle");
        fs::write(global_bundle.join("config.json"), "{}\n").expect("write global config");

        unsafe {
            std::env::set_var("HOME", &home);
            std::env::remove_var("MEMD_BUNDLE_ROOT");
        }
        std::env::set_current_dir(&nested).expect("set current dir to nested repo");

        let resolved = resolve_default_bundle_root().expect("resolve bundle root");
        assert_eq!(resolved, None);

        std::env::set_current_dir(&original_cwd).expect("restore current dir");
        unsafe {
            match original_home {
                Some(value) => std::env::set_var("HOME", value),
                None => std::env::remove_var("HOME"),
            }
            match original_bundle_root {
                Some(value) => std::env::set_var("MEMD_BUNDLE_ROOT", value),
                None => std::env::remove_var("MEMD_BUNDLE_ROOT"),
            }
        }
        fs::remove_dir_all(root).expect("cleanup temp bundle roots");
    }

    #[test]
    fn lookup_defaults_include_candidate_memory() {
        let req = build_lookup_request(
            &LookupArgs {
                output: PathBuf::from(".memd"),
                query: "what did we already decide about this?".to_string(),
                project: None,
                namespace: None,
                workspace: None,
                visibility: None,
                route: None,
                intent: None,
                kind: Vec::new(),
                tag: Vec::new(),
                include_stale: false,
                limit: None,
                verbose: false,
                json: false,
            },
            None,
        )
        .expect("build lookup request");

        assert!(req.kinds.contains(&MemoryKind::Status));
        assert!(req.kinds.contains(&MemoryKind::Decision));
        assert!(req.kinds.contains(&MemoryKind::Fact));
        assert!(req.kinds.contains(&MemoryKind::Pattern));
        assert_eq!(
            req.stages,
            vec![MemoryStage::Canonical, MemoryStage::Candidate]
        );
    }

    #[test]
    fn broad_lookup_fallback_uses_tags_or_rich_query_terms() {
        let tagged_req = SearchMemoryRequest {
            query: Some("memory".to_string()),
            route: Some(RetrievalRoute::ProjectFirst),
            intent: Some(RetrievalIntent::General),
            scopes: vec![MemoryScope::Project],
            kinds: vec![MemoryKind::Fact],
            statuses: vec![MemoryStatus::Active],
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            visibility: None,
            belief_branch: None,
            source_agent: None,
            tags: vec!["checkpoint".to_string()],
            stages: vec![MemoryStage::Canonical],
            limit: Some(6),
            max_chars_per_item: Some(280),
        };
        let untagged_req = SearchMemoryRequest {
            tags: Vec::new(),
            ..tagged_req.clone()
        };

        assert!(should_use_broad_lookup_fallback(&tagged_req, "memory"));
        assert!(!should_use_broad_lookup_fallback(&untagged_req, "memory"));
        assert!(should_use_broad_lookup_fallback(
            &untagged_req,
            "ralph roadmap progress state current phase next step"
        ));
    }

    #[test]
    fn rerank_broad_lookup_fallback_prefers_term_overlap_and_source_path() {
        let req = SearchMemoryRequest {
            query: Some("ralph roadmap progress state current phase next step".to_string()),
            route: Some(RetrievalRoute::ProjectFirst),
            intent: Some(RetrievalIntent::General),
            scopes: vec![MemoryScope::Project],
            kinds: vec![MemoryKind::Status],
            statuses: vec![MemoryStatus::Active],
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            visibility: None,
            belief_branch: None,
            source_agent: None,
            tags: Vec::new(),
            stages: vec![MemoryStage::Canonical],
            limit: Some(6),
            max_chars_per_item: Some(280),
        };
        let response = memd_schema::SearchMemoryResponse {
            route: RetrievalRoute::ProjectFirst,
            intent: RetrievalIntent::General,
            items: vec![
                memd_schema::MemoryItem {
                    id: uuid::Uuid::new_v4(),
                    content: "hive resume state".to_string(),
                    redundancy_key: None,
                    belief_branch: None,
                    preferred: false,
                    kind: MemoryKind::Status,
                    scope: MemoryScope::Project,
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: None,
                    visibility: memd_schema::MemoryVisibility::Workspace,
                    source_agent: None,
                    source_system: None,
                    source_path: None,
                    source_quality: None,
                    confidence: 0.9,
                    ttl_seconds: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    last_verified_at: None,
                    supersedes: Vec::new(),
                    tags: vec!["resume_state".to_string()],
                    status: MemoryStatus::Active,
                    stage: MemoryStage::Canonical,
                },
                memd_schema::MemoryItem {
                    id: uuid::Uuid::new_v4(),
                    content: "ralph roadmap progress state: current phase Phase E. next step cross-harness wake-packet proof.".to_string(),
                    redundancy_key: None,
                    belief_branch: None,
                    preferred: false,
                    kind: MemoryKind::Status,
                    scope: MemoryScope::Project,
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: None,
                    visibility: memd_schema::MemoryVisibility::Workspace,
                    source_agent: None,
                    source_system: None,
                    source_path: Some("ralph-roadmap-progress-state".to_string()),
                    source_quality: None,
                    confidence: 0.8,
                    ttl_seconds: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    last_verified_at: None,
                    supersedes: Vec::new(),
                    tags: vec!["roadmap".to_string(), "phase-e".to_string(), "ralph".to_string()],
                    status: MemoryStatus::Active,
                    stage: MemoryStage::Canonical,
                },
            ],
        };

        let reranked = rerank_broad_lookup_fallback(
            &response,
            &req,
            "ralph roadmap progress state current phase next step",
        );

        assert_eq!(reranked.items.len(), 2);
        assert!(
            reranked.items[0]
                .content
                .contains("ralph roadmap progress state")
        );
    }
}

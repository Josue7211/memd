use super::*;

pub(crate) fn enrich_with_entities(
    state: &AppState,
    items: Vec<MemoryItem>,
) -> anyhow::Result<Vec<MemoryViewItem>> {
    items
        .into_iter()
        .map(|item| {
            let entity = state.store.entity_for_item(item.id)?;
            let source_trust_score = state.store.trust_score_for_item(&item)?;
            Ok(MemoryViewItem {
                item,
                entity,
                source_trust_score,
            })
        })
        .collect()
}

pub(crate) struct BuildContextResult {
    pub plan: RetrievalPlan,
    pub retrieval_order: Vec<MemoryScope>,
    pub items: Vec<MemoryItem>,
}

pub(crate) fn build_context(
    state: &AppState,
    req: &ContextRequest,
) -> Result<BuildContextResult, (StatusCode, String)> {
    let plan = RetrievalPlan::resolve(req.route, req.intent);
    let limit = req.limit.unwrap_or(8).min(32);
    let max_chars = req.max_chars_per_item.unwrap_or(280).clamp(80, 2000);
    let items = enrich_with_entities(state, state.snapshot().map_err(internal_error)?)
        .map_err(internal_error)?;
    let retrieval_order = plan.scopes();

    let mut scoped: Vec<MemoryItem> = Vec::new();
    let mut live_truth: Vec<MemoryViewItem> = items
        .iter()
        .filter(|entry| entry.item.kind == MemoryKind::LiveTruth)
        .filter(|entry| entry.item.status == MemoryStatus::Active)
        .filter(|entry| plan.allows(entry.item.scope))
        .filter(
            |entry| match (&req.project, &entry.item.project, entry.item.scope) {
                (Some(project), Some(item_project), MemoryScope::Project | MemoryScope::Synced) => {
                    item_project == project
                }
                (Some(_), None, MemoryScope::Project | MemoryScope::Synced) => false,
                _ => true,
            },
        )
        .filter(|entry| {
            req.visibility
                .is_none_or(|visibility| entry.item.visibility == visibility)
        })
        .cloned()
        .collect();
    live_truth.sort_by(|a, b| b.item.updated_at.cmp(&a.item.updated_at));

    for entry in live_truth {
        let mut item = entry.item;
        item.content = compact_content(&item.content, max_chars);
        scoped.push(item);
        if scoped.len() >= limit {
            scoped.truncate(limit);
            return Ok(BuildContextResult {
                plan,
                retrieval_order,
                items: scoped,
            });
        }
    }

    let mut ranked_items: Vec<MemoryViewItem> = items
        .iter()
        .filter(|entry| plan.allows(entry.item.scope))
        .filter(|entry| entry.item.kind != MemoryKind::LiveTruth)
        .filter(|entry| entry.item.status == MemoryStatus::Active)
        .filter(
            |entry| match (&req.project, &entry.item.project, entry.item.scope) {
                (Some(project), Some(item_project), MemoryScope::Project) => {
                    item_project == project
                }
                (Some(project), Some(item_project), MemoryScope::Synced) => item_project == project,
                (Some(_), None, MemoryScope::Project | MemoryScope::Synced) => false,
                _ => true,
            },
        )
        .filter(|entry| {
            req.visibility
                .is_none_or(|visibility| entry.item.visibility == visibility)
        })
        .cloned()
        .collect();

    ranked_items.sort_by(|a, b| {
        context_score(&b.item, b.entity.as_ref(), b.source_trust_score, req, &plan)
            .partial_cmp(&context_score(
                &a.item,
                a.entity.as_ref(),
                a.source_trust_score,
                req,
                &plan,
            ))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.item.updated_at.cmp(&a.item.updated_at))
    });

    scoped.extend(ranked_items.into_iter().map(|entry| entry.item));

    for item in &mut scoped {
        item.content = compact_content(&item.content, max_chars);
    }
    scoped.truncate(limit);

    Ok(BuildContextResult {
        plan,
        retrieval_order,
        items: scoped,
    })
}

pub(crate) fn apply_agent_profile_defaults(
    state: &AppState,
    mut req: ContextRequest,
) -> anyhow::Result<ContextRequest> {
    let Some(agent) = req.agent.clone() else {
        return Ok(req);
    };

    let profile = state.store.agent_profile(&AgentProfileRequest {
        agent,
        project: req.project.clone(),
        namespace: None,
    })?;
    if let Some(profile) = profile {
        if req.route.is_none() {
            req.route = profile.preferred_route;
        }
        if req.intent.is_none() {
            req.intent = profile.preferred_intent;
        }
        if req.max_chars_per_item.is_none() {
            req.max_chars_per_item = profile.summary_chars;
        }
        if req.limit.is_none() && profile.recall_depth.is_some() {
            req.limit = profile.recall_depth;
        }
    }

    Ok(req)
}

pub(crate) fn filter_items(
    items: &[MemoryViewItem],
    req: &SearchMemoryRequest,
    plan: &RetrievalPlan,
) -> Vec<MemoryItem> {
    let query = req.query.as_ref().map(|q| q.to_ascii_lowercase());
    let limit = req.limit.unwrap_or(10).min(100);
    let max_chars = req.max_chars_per_item.unwrap_or(420).clamp(120, 4000);

    let mut filtered: Vec<MemoryViewItem> = items
        .iter()
        .filter(|entry| req.scopes.is_empty() || req.scopes.contains(&entry.item.scope))
        .filter(|entry| plan.allows(entry.item.scope))
        .filter(|entry| req.kinds.is_empty() || req.kinds.contains(&entry.item.kind))
        .filter(|entry| req.statuses.is_empty() || req.statuses.contains(&entry.item.status))
        .filter(|entry| req.stages.is_empty() || req.stages.contains(&entry.item.stage))
        .filter(|entry| {
            req.project
                .as_ref()
                .is_none_or(|project| entry.item.project.as_ref() == Some(project))
        })
        .filter(|entry| {
            req.namespace
                .as_ref()
                .is_none_or(|namespace| entry.item.namespace.as_ref() == Some(namespace))
        })
        .filter(|entry| {
            req.workspace
                .as_ref()
                .is_none_or(|workspace| entry.item.workspace.as_ref() == Some(workspace))
        })
        .filter(|entry| {
            req.visibility
                .is_none_or(|visibility| entry.item.visibility == visibility)
        })
        .filter(|entry| {
            req.belief_branch
                .as_ref()
                .is_none_or(|branch| entry.item.belief_branch.as_ref() == Some(branch))
        })
        .filter(|entry| {
            req.source_agent
                .as_ref()
                .is_none_or(|agent| entry.item.source_agent.as_ref() == Some(agent))
        })
        .filter(|entry| {
            req.tags.is_empty()
                || req
                    .tags
                    .iter()
                    .all(|tag| entry.item.tags.iter().any(|item_tag| item_tag == tag))
        })
        .filter(|entry| {
            query.as_ref().is_none_or(|query| {
                entry.item.content.to_ascii_lowercase().contains(query)
                    || entry
                        .item
                        .tags
                        .iter()
                        .any(|tag| tag.to_ascii_lowercase().contains(query))
            })
        })
        .cloned()
        .collect();

    filtered.sort_by(|a, b| {
        search_score(
            &b.item,
            b.entity.as_ref(),
            b.source_trust_score,
            &query,
            plan,
        )
        .partial_cmp(&search_score(
            &a.item,
            a.entity.as_ref(),
            a.source_trust_score,
            &query,
            plan,
        ))
        .unwrap_or(std::cmp::Ordering::Equal)
        .then_with(|| {
            b.item
                .confidence
                .partial_cmp(&a.item.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .then_with(|| b.item.updated_at.cmp(&a.item.updated_at))
    });
    for item in &mut filtered {
        item.item.content = compact_content(&item.item.content, max_chars);
    }
    filtered.truncate(limit);
    filtered.into_iter().map(|entry| entry.item).collect()
}

pub(crate) fn compact_content(content: &str, max_chars: usize) -> String {
    let normalized = content.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.chars().count() <= max_chars {
        return normalized;
    }

    let mut compact = String::with_capacity(max_chars + 3);
    for ch in normalized.chars().take(max_chars.saturating_sub(3)) {
        compact.push(ch);
    }
    compact.push_str("...");
    compact
}

pub(crate) fn event_type_for_stage(stage: MemoryStage) -> &'static str {
    match stage {
        MemoryStage::Candidate => "candidate_created",
        MemoryStage::Canonical => "canonical_created",
    }
}

pub(crate) fn entity_context_frame(entity: &MemoryEntityRecord, item: &MemoryItem) -> MemoryContextFrame {
    entity.context.clone().unwrap_or(MemoryContextFrame {
        at: Some(item.updated_at),
        project: item.project.clone(),
        namespace: item.namespace.clone(),
        workspace: item.workspace.clone(),
        repo: item.source_system.clone(),
        host: None,
        branch: None,
        agent: item.source_agent.clone(),
        location: item.source_path.clone(),
    })
}

pub(crate) fn consolidation_content(
    entity: &MemoryEntityRecord,
    event_count: usize,
    first_recorded_at: chrono::DateTime<chrono::Utc>,
    last_recorded_at: chrono::DateTime<chrono::Utc>,
) -> String {
    let state = compact_content(
        entity
            .current_state
            .as_deref()
            .unwrap_or("state unavailable"),
        220,
    );
    let span_days = (last_recorded_at - first_recorded_at).num_days().max(0);
    format!(
        "stable {} state after {} events over {}d: {}",
        entity.entity_type, event_count, span_days, state
    )
}

pub(crate) fn consolidation_scope(entity: &MemoryEntityRecord) -> MemoryScope {
    let context = entity.context.as_ref();
    if context
        .and_then(|context| context.project.as_ref())
        .is_some()
    {
        MemoryScope::Project
    } else if context
        .and_then(|context| context.namespace.as_ref())
        .is_some()
    {
        MemoryScope::Synced
    } else {
        MemoryScope::Local
    }
}

pub(crate) fn consolidation_kind(entity_type: &str) -> MemoryKind {
    match entity_type {
        "fact" => MemoryKind::Fact,
        "decision" => MemoryKind::Decision,
        "preference" => MemoryKind::Preference,
        "runbook" => MemoryKind::Runbook,
        "procedural" => MemoryKind::Procedural,
        "self_model" => MemoryKind::SelfModel,
        "topology" => MemoryKind::Topology,
        "status" => MemoryKind::Status,
        "live_truth" => MemoryKind::LiveTruth,
        "pattern" => MemoryKind::Pattern,
        "constraint" => MemoryKind::Constraint,
        _ => MemoryKind::Pattern,
    }
}

pub(crate) fn consolidation_tags(entity: &MemoryEntityRecord, event_count: usize) -> Vec<String> {
    let mut tags = entity.tags.clone();
    tags.push("consolidated".to_string());
    tags.push(format!("events:{}", event_count));
    tags.push(entity.entity_type.clone());
    tags.sort();
    tags.dedup();
    tags
}

pub(crate) fn internal_error(error: anyhow::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

pub(crate) fn compact_record(item: &MemoryItem) -> String {
    let mut parts = Vec::new();
    parts.push(format!("id={}", item.id));
    parts.push(format!("stage={}", enum_label_stage(item.stage)));
    parts.push(format!("scope={}", enum_label_scope(item.scope)));
    parts.push(format!("kind={}", enum_label_kind(item.kind)));
    parts.push(format!("status={}", enum_label_status(item.status)));

    if let Some(project) = &item.project {
        if !project.is_empty() {
            parts.push(format!("project={}", sanitize_value(project)));
        }
    }
    if let Some(namespace) = &item.namespace {
        if !namespace.is_empty() {
            parts.push(format!("ns={}", sanitize_value(namespace)));
        }
    }
    if let Some(workspace) = &item.workspace {
        if !workspace.is_empty() {
            parts.push(format!("ws={}", sanitize_value(workspace)));
        }
    }
    parts.push(format!("vis={}", enum_label_visibility(item.visibility)));
    if let Some(branch) = &item.belief_branch {
        if !branch.is_empty() {
            parts.push(format!("belief_branch={}", sanitize_value(branch)));
        }
    }
    if let Some(agent) = &item.source_agent {
        if !agent.is_empty() {
            parts.push(format!("agent={}", sanitize_value(agent)));
        }
    }
    if !item.tags.is_empty() {
        let tags = item
            .tags
            .iter()
            .map(|tag| sanitize_value(tag))
            .collect::<Vec<_>>()
            .join(",");
        parts.push(format!("tags={}", tags));
    }
    parts.push(format!("cf={:.2}", item.confidence));
    parts.push(format!("upd={}", item.updated_at.timestamp()));
    parts.push(format!("c={}", sanitize_value(&item.content)));

    parts.join(" | ")
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

pub(crate) fn associative_recall_score(
    entity: &MemoryEntityRecord,
    link: &MemoryEntityLinkRecord,
    depth: usize,
    root: &MemoryEntityRecord,
) -> f32 {
    let relation_weight = match link.relation_kind {
        memd_schema::EntityRelationKind::SameAs => 1.0,
        memd_schema::EntityRelationKind::Supersedes => 0.92,
        memd_schema::EntityRelationKind::DerivedFrom => 0.88,
        memd_schema::EntityRelationKind::Related => 0.7,
        memd_schema::EntityRelationKind::Contradicts => 0.62,
    };
    let depth_penalty = 1.0 / (depth as f32 + 1.0);
    let salience = entity.salience_score.clamp(0.0, 1.0);
    let rehearsal = (entity.rehearsal_count as f32).ln_1p().min(3.0) / 3.0;
    let context_bonus = if entity
        .context
        .as_ref()
        .and_then(|context| context.project.as_ref())
        == root
            .context
            .as_ref()
            .and_then(|context| context.project.as_ref())
    {
        0.08
    } else {
        0.0
    };
    ((relation_weight * 0.42)
        + (salience * 0.34)
        + (rehearsal * 0.12)
        + (depth_penalty * 0.08)
        + context_bonus)
        .clamp(0.0, 1.0)
}

pub(crate) fn associative_recall_reasons(
    entity: &MemoryEntityRecord,
    link: &MemoryEntityLinkRecord,
    depth: usize,
) -> Vec<String> {
    let mut reasons = Vec::new();
    reasons.push(format!("{:?}", link.relation_kind).to_lowercase());
    reasons.push(format!("depth={depth}"));
    reasons.push(format!("salience={:.2}", entity.salience_score));
    if entity.rehearsal_count > 1 {
        reasons.push(format!("rehearsal={}", entity.rehearsal_count));
    }
    if !entity.aliases.is_empty() {
        reasons.push(format!("aliases={}", entity.aliases.len()));
    }
    reasons
}

#[allow(dead_code)]
pub(crate) fn legacy_dashboard_html() -> String {
    r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>memd</title>
  <style>
    :root {
      color-scheme: dark;
      --bg: #0b0d10;
      --panel: #11151b;
      --panel-2: #161b23;
      --text: #e7eef8;
      --muted: #93a4ba;
      --line: #243041;
      --accent: #69a8ff;
      --accent-2: #7bf1c8;
      --warn: #ffbd59;
      --bad: #ff6b6b;
    }
    * { box-sizing: border-box; }
    body {
      margin: 0;
      font: 14px/1.5 Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, sans-serif;
      background:
        radial-gradient(circle at top left, rgba(105,168,255,0.12), transparent 32%),
        radial-gradient(circle at top right, rgba(123,241,200,0.10), transparent 28%),
        linear-gradient(180deg, #090b0e, var(--bg));
      color: var(--text);
    }
    header {
      padding: 28px 24px 16px;
      border-bottom: 1px solid var(--line);
      background: rgba(10, 13, 17, 0.92);
      position: sticky;
      top: 0;
      backdrop-filter: blur(14px);
      z-index: 2;
    }
    .shell {
      max-width: 1400px;
      margin: 0 auto;
    }
    h1 {
      margin: 0 0 6px;
      font-size: 28px;
      letter-spacing: -0.02em;
    }
    .sub {
      color: var(--muted);
      margin: 0;
    }
    main {
      max-width: 1400px;
      margin: 0 auto;
      padding: 20px 24px 32px;
      display: grid;
      grid-template-columns: 360px 1fr;
      gap: 18px;
      align-items: start;
    }
    .panel {
      background: linear-gradient(180deg, rgba(17,21,27,0.95), rgba(13,17,22,0.95));
      border: 1px solid var(--line);
      border-radius: 18px;
      box-shadow: 0 24px 60px rgba(0,0,0,0.25);
      overflow: hidden;
    }
    .panel h2 {
      margin: 0;
      padding: 16px 16px 12px;
      border-bottom: 1px solid var(--line);
      font-size: 14px;
      letter-spacing: 0.08em;
      text-transform: uppercase;
      color: var(--muted);
    }
    .content {
      padding: 16px;
    }
    label {
      display: block;
      margin: 0 0 10px;
      color: var(--muted);
      font-size: 12px;
      text-transform: uppercase;
      letter-spacing: 0.08em;
    }
    input, select, textarea, button {
      width: 100%;
      border-radius: 12px;
      border: 1px solid var(--line);
      background: var(--panel-2);
      color: var(--text);
      padding: 11px 12px;
      font: inherit;
    }
    textarea {
      min-height: 120px;
      resize: vertical;
    }
    button {
      cursor: pointer;
      background: linear-gradient(180deg, rgba(105,168,255,0.95), rgba(76,131,245,0.95));
      border: 0;
      font-weight: 650;
    }
    button.secondary {
      background: var(--panel-2);
      border: 1px solid var(--line);
      color: var(--text);
      font-weight: 600;
    }
    .stack {
      display: grid;
      gap: 10px;
    }
    .grid-2 {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 10px;
    }
    .meta {
      display: flex;
      flex-wrap: wrap;
      gap: 8px;
      color: var(--muted);
      font-size: 12px;
      margin-bottom: 12px;
    }
    .pill {
      border: 1px solid var(--line);
      border-radius: 999px;
      padding: 6px 10px;
      background: rgba(255,255,255,0.02);
    }
    pre {
      margin: 0;
      white-space: pre-wrap;
      word-break: break-word;
      color: #dce7f4;
      background: #0b0f14;
      border: 1px solid var(--line);
      border-radius: 14px;
      padding: 14px;
      min-height: 240px;
      max-height: 68vh;
      overflow: auto;
    }
    .section {
      display: grid;
      gap: 12px;
    }
    .toolbar {
      display: flex;
      gap: 8px;
      flex-wrap: wrap;
    }
    .toolbar button {
      width: auto;
      padding: 10px 14px;
    }
    .note {
      color: var(--muted);
      font-size: 12px;
    }
    @media (max-width: 1040px) {
      main { grid-template-columns: 1fr; }
    }
  </style>
</head>
<body>
  <header>
    <div class="shell">
      <h1>memd</h1>
      <p class="sub">Memory manager, retrieval router, inbox, and explain surface.</p>
    </div>
  </header>
  <main>
    <section class="panel">
      <h2>Controls</h2>
      <div class="content stack">
        <div class="grid-2">
          <div>
            <label>Project</label>
            <input id="project" placeholder="demo">
          </div>
          <div>
            <label>Agent</label>
            <input id="agent" placeholder="codex">
          </div>
        </div>
        <div class="grid-2">
          <div>
            <label>Workspace</label>
            <input id="workspace" placeholder="team-alpha">
          </div>
          <div>
            <label>Visibility</label>
            <select id="visibility">
              <option value="">all</option>
              <option value="private">private</option>
              <option value="workspace">workspace</option>
              <option value="public">public</option>
            </select>
          </div>
        </div>
        <div class="grid-2">
          <div>
            <label>Route</label>
            <select id="route">
              <option value="auto">auto</option>
              <option value="local_only">local_only</option>
              <option value="synced_only">synced_only</option>
              <option value="project_only">project_only</option>
              <option value="global_only">global_only</option>
              <option value="local_first">local_first</option>
              <option value="synced_first">synced_first</option>
              <option value="project_first">project_first</option>
              <option value="global_first">global_first</option>
              <option value="all">all</option>
            </select>
          </div>
          <div>
            <label>Intent</label>
            <select id="intent">
              <option value="general">general</option>
              <option value="current_task">current_task</option>
              <option value="decision">decision</option>
              <option value="runbook">runbook</option>
              <option value="procedural">procedural</option>
              <option value="self_model">self_model</option>
              <option value="topology">topology</option>
              <option value="preference">preference</option>
              <option value="fact">fact</option>
              <option value="pattern">pattern</option>
            </select>
          </div>
        </div>
        <div>
          <label>Search query</label>
          <input id="query" placeholder="postgres, routing, memory, etc.">
        </div>
        <div>
          <label>Explain id</label>
          <input id="id" placeholder="UUID">
        </div>
        <div class="toolbar">
          <button onclick="loadHealth()">Refresh health</button>
          <button onclick="loadContext()">Load context</button>
          <button onclick="loadInbox()">Load inbox</button>
          <button onclick="loadSearch()">Search</button>
          <button onclick="loadWorkspaces()">Workspaces</button>
          <button class="secondary" onclick="loadSources()">Sources</button>
          <button class="secondary" onclick="loadExplain()">Explain</button>
        </div>
        <div class="note" id="healthNote">Loading health...</div>
      </div>
    </section>
    <section class="panel">
      <h2>Output</h2>
      <div class="content section">
        <pre id="output">{}</pre>
      </div>
    </section>
  </main>
  <script>
    const output = document.getElementById('output');
    const healthNote = document.getElementById('healthNote');
    const qs = () => ({
      project: document.getElementById('project').value.trim(),
      workspace: document.getElementById('workspace').value.trim(),
      visibility: document.getElementById('visibility').value,
      agent: document.getElementById('agent').value.trim(),
      route: document.getElementById('route').value,
      intent: document.getElementById('intent').value,
      query: document.getElementById('query').value.trim(),
      id: document.getElementById('id').value.trim(),
    });
    function pretty(data) {
      output.textContent = JSON.stringify(data, null, 2);
    }
    async function loadHealth() {
      const res = await fetch('/healthz');
      const data = await res.json();
      healthNote.textContent = `status=${data.status} items=${data.items}`;
      pretty(data);
    }
    async function loadContext() {
      const q = qs();
      const params = new URLSearchParams();
      if (q.project) params.set('project', q.project);
      if (q.workspace) params.set('workspace', q.workspace);
      if (q.visibility) params.set('visibility', q.visibility);
      if (q.agent) params.set('agent', q.agent);
      if (q.route !== 'auto') params.set('route', q.route);
      if (q.intent !== 'general') params.set('intent', q.intent);
      const res = await fetch('/memory/context/compact?' + params.toString());
      pretty(await res.json());
    }
    async function loadInbox() {
      const q = qs();
      const params = new URLSearchParams();
      if (q.project) params.set('project', q.project);
      if (q.workspace) params.set('workspace', q.workspace);
      if (q.visibility) params.set('visibility', q.visibility);
      if (q.agent) params.set('agent', q.agent);
      if (q.route !== 'auto') params.set('route', q.route);
      if (q.intent !== 'general') params.set('intent', q.intent);
      const res = await fetch('/memory/inbox?' + params.toString());
      pretty(await res.json());
    }
    async function loadSearch() {
      const q = qs();
      const body = {
        query: q.query || undefined,
        project: q.project || undefined,
        workspace: q.workspace || undefined,
        visibility: q.visibility || undefined,
        route: q.route,
        intent: q.intent,
      };
      const res = await fetch('/memory/search', {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify(body),
      });
      pretty(await res.json());
    }
    async function loadWorkspaces() {
      const q = qs();
      const params = new URLSearchParams();
      if (q.project) params.set('project', q.project);
      if (q.workspace) params.set('workspace', q.workspace);
      if (q.visibility) params.set('visibility', q.visibility);
      const res = await fetch('/memory/workspaces?' + params.toString());
      pretty(await res.json());
    }
    async function loadSources() {
      const q = qs();
      const params = new URLSearchParams();
      if (q.project) params.set('project', q.project);
      if (q.workspace) params.set('workspace', q.workspace);
      if (q.visibility) params.set('visibility', q.visibility);
      const res = await fetch('/memory/source?' + params.toString());
      pretty(await res.json());
    }
    async function loadExplain() {
      const q = qs();
      const params = new URLSearchParams();
      params.set('id', q.id);
      if (q.route !== 'auto') params.set('route', q.route);
      if (q.intent !== 'general') params.set('intent', q.intent);
      const res = await fetch('/memory/explain?' + params.toString());
      pretty(await res.json());
    }
    loadHealth().catch(err => {
      healthNote.textContent = `health check failed: ${err}`;
      output.textContent = JSON.stringify({error: String(err)}, null, 2);
    });
    setInterval(() => { loadHealth().catch(() => {}); }, 5000);
  </script>
</body>
</html>"#
        .to_string()
}

pub(crate) fn sanitize_value(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace('|', "/")
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

pub(crate) fn enum_label_visibility(visibility: MemoryVisibility) -> &'static str {
    match visibility {
        MemoryVisibility::Private => "private",
        MemoryVisibility::Workspace => "workspace",
        MemoryVisibility::Public => "public",
    }
}

pub(crate) fn enum_label_stage(stage: MemoryStage) -> &'static str {
    match stage {
        MemoryStage::Candidate => "candidate",
        MemoryStage::Canonical => "canonical",
    }
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

pub(crate) fn context_score(
    item: &MemoryItem,
    entity: Option<&MemoryEntityRecord>,
    source_trust_score: f32,
    req: &ContextRequest,
    plan: &RetrievalPlan,
) -> f32 {
    let mut score = item.confidence;

    score += match item.stage {
        MemoryStage::Canonical => 1.25,
        MemoryStage::Candidate => 0.25,
    };

    score += plan.scope_rank_bonus(item.scope);
    score += plan.intent_scope_bonus(item.scope);
    score += entity_attention_bonus(item, entity);

    if let Some(project) = &req.project {
        if item.project.as_ref() == Some(project) {
            score += 1.5;
        }
    }

    if let Some(agent) = &req.agent {
        if item.source_agent.as_ref() == Some(agent) {
            score += 0.75;
        }
    }

    score += workspace_rank_adjustment(req.workspace.as_ref(), item.workspace.as_ref());

    score += entity_context_bonus(entity, req.project.as_ref(), req.agent.as_ref());
    score += trust_rank_adjustment(source_trust_score);
    score += epistemic_rank_adjustment(item);

    if item.status == MemoryStatus::Stale {
        score -= 1.5;
    }

    if item.status == MemoryStatus::Contested {
        score -= 2.0;
    }

    score -= age_penalty(item.updated_at);
    score
}

pub(crate) fn search_score(
    item: &MemoryItem,
    entity: Option<&MemoryEntityRecord>,
    source_trust_score: f32,
    query: &Option<String>,
    plan: &RetrievalPlan,
) -> f32 {
    let mut score = item.confidence;

    score += match item.stage {
        MemoryStage::Canonical => 1.0,
        MemoryStage::Candidate => 0.2,
    };

    score += match item.status {
        MemoryStatus::Active => 1.0,
        MemoryStatus::Stale => -1.0,
        MemoryStatus::Superseded => -2.0,
        MemoryStatus::Contested => -1.5,
        MemoryStatus::Expired => -4.0,
    };

    score += match item.scope {
        MemoryScope::Project => 0.75,
        MemoryScope::Synced => 0.5,
        MemoryScope::Local => 0.4,
        MemoryScope::Global => 0.1,
    };
    score += plan.scope_rank_bonus(item.scope) * 0.5;
    score += plan.intent_scope_bonus(item.scope) * 0.75;
    score += entity_attention_bonus(item, entity) * 0.75;
    score += trust_rank_adjustment(source_trust_score) * 0.8;
    score += epistemic_rank_adjustment(item) * 0.85;

    if let Some(query) = query {
        let content = item.content.to_ascii_lowercase();
        if content.contains(query) {
            score += 2.0;
        }
        let tag_hits = item
            .tags
            .iter()
            .filter(|tag| tag.to_ascii_lowercase().contains(query))
            .count();
        score += tag_hits as f32 * 0.5;
    }

    score -= age_penalty(item.updated_at);
    score
}

pub(crate) fn trust_rank_adjustment(source_trust_score: f32) -> f32 {
    if source_trust_score < 0.35 {
        -1.1
    } else if source_trust_score < 0.5 {
        -0.65
    } else if source_trust_score < 0.6 {
        -0.3
    } else if source_trust_score >= 0.9 {
        0.22
    } else if source_trust_score >= 0.75 {
        0.12
    } else {
        0.0
    }
}

pub(crate) fn epistemic_rank_adjustment(item: &MemoryItem) -> f32 {
    let mut score = match item.source_quality {
        Some(SourceQuality::Canonical) => 0.4,
        Some(SourceQuality::Derived) => 0.1,
        Some(SourceQuality::Synthetic) => -0.4,
        None => 0.0,
    };

    score += match item.last_verified_at {
        Some(verified_at) => {
            let verified_days = Utc::now()
                .signed_duration_since(verified_at)
                .num_days()
                .max(0);
            if verified_days <= 7 {
                0.45
            } else if verified_days <= 30 {
                0.2
            } else if verified_days <= 90 {
                0.05
            } else {
                -0.15
            }
        }
        None => -0.2,
    };

    if item.confidence < 0.6 {
        score -= 0.25;
    } else if item.confidence >= 0.9 {
        score += 0.08;
    }

    score
}

pub(crate) fn workspace_rank_adjustment(
    requested_workspace: Option<&String>,
    item_workspace: Option<&String>,
) -> f32 {
    match (requested_workspace, item_workspace) {
        (Some(requested), Some(item)) if requested == item => 0.85,
        (Some(_), Some(_)) => -0.18,
        (Some(_), None) => -0.08,
        _ => 0.0,
    }
}

pub(crate) fn age_penalty(updated_at: chrono::DateTime<Utc>) -> f32 {
    let age_days = (Utc::now() - updated_at).num_days().max(0) as f32;
    (age_days / 14.0).min(3.0)
}

pub(crate) fn inbox_score(
    item: &MemoryItem,
    entity: Option<&MemoryEntityRecord>,
    plan: &RetrievalPlan,
) -> f32 {
    let mut score = item.confidence;
    score += plan.scope_rank_bonus(item.scope);
    score += plan.intent_scope_bonus(item.scope);
    score += match item.stage {
        MemoryStage::Candidate => 2.0,
        MemoryStage::Canonical => 0.5,
    };
    score += match item.status {
        MemoryStatus::Contested => 2.5,
        MemoryStatus::Stale => 2.0,
        MemoryStatus::Superseded => 1.5,
        MemoryStatus::Expired => 1.0,
        MemoryStatus::Active => 0.0,
    };
    score += entity_attention_bonus(item, entity);
    score -= age_penalty(item.updated_at) * 0.75;
    score
}

pub(crate) fn entity_attention_bonus(item: &MemoryItem, entity: Option<&MemoryEntityRecord>) -> f32 {
    let Some(entity) = entity else {
        return 0.0;
    };

    let salience = entity.salience_score.clamp(0.0, 1.0);
    let rehearsal = (entity.rehearsal_count as f32 + 1.0).ln_1p();
    let recency = entity
        .last_accessed_at
        .map(|at| {
            let age_days = (Utc::now() - at).num_days().max(0) as f32;
            (1.0 - (age_days / 30.0)).clamp(0.0, 1.0)
        })
        .unwrap_or(0.0);
    let state_alignment = entity
        .context
        .as_ref()
        .map(|context| {
            let mut bonus = 0.0;
            if context.project.as_ref() == item.project.as_ref() {
                bonus += 0.2;
            }
            if context.namespace.as_ref() == item.namespace.as_ref() {
                bonus += 0.1;
            }
            if context.agent.as_ref() == item.source_agent.as_ref() {
                bonus += 0.1;
            }
            bonus
        })
        .unwrap_or(0.0);

    salience * 0.9 + rehearsal * 0.25 + recency * 0.25 + state_alignment
}

pub(crate) fn entity_context_bonus(
    entity: Option<&MemoryEntityRecord>,
    project: Option<&String>,
    agent: Option<&String>,
) -> f32 {
    let Some(entity) = entity else {
        return 0.0;
    };

    let mut bonus = 0.0;
    if let Some(context) = &entity.context {
        if context.project.as_ref() == project {
            bonus += 0.35;
        }
        if context.agent.as_ref() == agent {
            bonus += 0.2;
        }
    }
    bonus
}

pub(crate) fn inbox_reasons(item: &MemoryItem) -> Vec<String> {
    let mut reasons = Vec::new();
    if item.preferred {
        reasons.push("preferred-branch".to_string());
    }
    if item.stage == MemoryStage::Candidate {
        reasons.push("candidate".to_string());
    }
    match item.status {
        MemoryStatus::Contested => reasons.push("contested".to_string()),
        MemoryStatus::Stale => reasons.push("stale".to_string()),
        MemoryStatus::Superseded => reasons.push("superseded".to_string()),
        MemoryStatus::Expired => reasons.push("expired".to_string()),
        MemoryStatus::Active => {}
    }
    if item.source_quality == Some(SourceQuality::Derived) {
        reasons.push("derived".to_string());
    }
    if item.source_quality == Some(SourceQuality::Synthetic) {
        reasons.push("rejected-source".to_string());
    }
    if item.confidence < 0.75 {
        reasons.push("low-confidence".to_string());
    }
    if item.ttl_seconds.is_some() {
        reasons.push("ttl".to_string());
    }
    if item.belief_branch.is_some() && !item.preferred && item.status == MemoryStatus::Contested {
        reasons.push("unresolved-contradiction".to_string());
    }
    reasons
}

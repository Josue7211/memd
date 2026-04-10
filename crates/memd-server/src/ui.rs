use axum::http::StatusCode;
use chrono::Utc;
use memd_schema::{
    ExplainMemoryRequest, HiveClaimsRequest, HiveSessionsRequest, HiveTasksRequest,
    MemoryEventRecord, MemoryItem, MemoryRepairMode, MemoryStage, MemoryStatus, MemoryVisibility,
    RepairMemoryRequest, SourceMemoryRequest, TimelineMemoryResponse, VisibleMemoryArtifact,
    VisibleMemoryArtifactDetailResponse, VisibleMemoryGraphEdge, VisibleMemoryGraphNode,
    VisibleMemoryHome, VisibleMemoryKnowledgeMap, VisibleMemoryProvenance,
    VisibleMemorySnapshotResponse, VisibleMemoryStatus, VisibleMemoryUiActionKind,
    VisibleMemoryUiActionRequest, VisibleMemoryUiActionResponse, WorkspaceMemoryRecord,
    WorkspaceMemoryRequest,
};
use uuid::Uuid;

use crate::{AppState, internal_error, keys::canonical_key, keys::redundancy_key};

pub(crate) fn dashboard_html(snapshot: &VisibleMemorySnapshotResponse) -> String {
    let focus = &snapshot.home.focus_artifact;
    let source_path = focus.provenance.source_path.as_deref().unwrap_or("none");
    let source_system = focus.provenance.source_system.as_deref().unwrap_or("memd");
    let producer = focus.provenance.producer.as_deref().unwrap_or("memd");
    let detail_href = format!("/ui/artifact?id={}", focus.id);
    let focus_id = focus.id;
    let obsidian_bridge = if focus.provenance.source_path.is_some() {
        format!(
            r#"<div class="bridge">
  <div class="eyebrow">Obsidian bridge</div>
  <p>Vault source: <code>{}</code></p>
  <div class="actions">
    <button class="ghost">Open in Obsidian</button>
    <button class="ghost">Open vault source</button>
  </div>
</div>"#,
            escape_html(source_path)
        )
    } else {
        String::new()
    };
    let knowledge_rows = if snapshot.knowledge_map.nodes.is_empty() {
        "<li class=\"muted\">No graph nodes available yet.</li>".to_string()
    } else {
        snapshot
            .knowledge_map
            .nodes
            .iter()
            .take(8)
            .map(|node| {
                format!(
                    "<li><button class=\"artifact-link\" data-artifact-id=\"{}\"><strong>{}</strong><span>{} · {:?}</span></button></li>",
                    node.artifact_id,
                    escape_html(&node.title),
                    escape_html(&node.artifact_kind),
                    node.status
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };
    let timeline_nodes = count_nodes_by_kind(&snapshot.knowledge_map, "timeline_event");
    let workspace_nodes = count_nodes_by_kind(&snapshot.knowledge_map, "workspace_lane");
    let related_nodes = snapshot
        .knowledge_map
        .nodes
        .len()
        .saturating_sub(timeline_nodes + workspace_nodes);

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>memd visible memories</title>
  <style>
    :root {{
      color-scheme: dark;
      --bg: #0b0813;
      --panel: #14111f;
      --panel-2: #1a1628;
      --text: #f3efff;
      --muted: #a79cbe;
      --line: rgba(157, 124, 216, 0.18);
      --accent: #9d7cd8;
      --accent-2: #68d391;
    }}
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0;
      color: var(--text);
      background:
        radial-gradient(circle at 14% 18%, rgba(157, 124, 216, 0.14), transparent 24%),
        radial-gradient(circle at 84% 16%, rgba(104, 211, 145, 0.08), transparent 18%),
        linear-gradient(180deg, #0e0b18 0%, var(--bg) 100%);
      font: 14px/1.5 "Manrope", ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, sans-serif;
      min-height: 100vh;
    }}
    .shell {{
      display: grid;
      grid-template-columns: 260px minmax(0, 1fr) 320px;
      min-height: 100vh;
    }}
    aside {{
      padding: 24px;
      background: rgba(20, 17, 31, 0.88);
      border-right: 1px solid var(--line);
    }}
    .detail {{
      border-right: 0;
      border-left: 1px solid var(--line);
    }}
    main {{
      padding: 24px;
      display: grid;
      gap: 18px;
      align-content: start;
    }}
    h1, h2, h3 {{
      margin: 0;
      line-height: 1;
      letter-spacing: -0.04em;
    }}
    h1 {{
      font-size: clamp(2.6rem, 5vw, 4.4rem);
      margin-bottom: 0.5rem;
    }}
    h2 {{
      font-size: 1.45rem;
      margin-bottom: 0.7rem;
    }}
    p {{ color: var(--muted); margin: 0; }}
    .eyebrow {{
      font-size: 0.72rem;
      letter-spacing: 0.18em;
      text-transform: uppercase;
      color: rgba(167, 156, 190, 0.82);
      margin-bottom: 0.55rem;
    }}
    .panel {{
      background: linear-gradient(180deg, rgba(255,255,255,0.03), rgba(255,255,255,0.015));
      border: 1px solid var(--line);
      border-radius: 18px;
      padding: 18px;
      box-shadow: 0 18px 48px rgba(0, 0, 0, 0.22);
    }}
    .hero {{
      display: grid;
      gap: 1rem;
    }}
    .meta {{
      display: flex;
      flex-wrap: wrap;
      gap: 0.5rem;
      font-size: 0.76rem;
      letter-spacing: 0.12em;
      text-transform: uppercase;
      color: rgba(235, 230, 255, 0.84);
    }}
    .meta span, .tag {{
      border: 1px solid rgba(255,255,255,0.08);
      border-radius: 999px;
      padding: 0.34rem 0.7rem;
      background: rgba(255,255,255,0.03);
    }}
    .grid {{
      display: grid;
      gap: 12px;
      grid-template-columns: repeat(3, minmax(0, 1fr));
    }}
    .block {{
      border-radius: 16px;
      padding: 14px;
      background: rgba(255,255,255,0.03);
      border: 1px solid rgba(255,255,255,0.06);
    }}
    .block strong {{
      display: block;
      font-size: 1.6rem;
      margin-bottom: 0.2rem;
    }}
    .actions {{
      display: flex;
      flex-wrap: wrap;
      gap: 0.5rem;
    }}
    .artifact-link {{
      width: 100%;
      border: 0;
      background: transparent;
      padding: 0;
      text-align: left;
      display: grid;
      gap: 0.25rem;
      border-radius: 12px;
    }}
    .artifact-link strong {{
      color: var(--text);
    }}
    .artifact-link span {{
      color: var(--muted);
    }}
    .bridge {{
      display: grid;
      gap: 0.6rem;
      margin-top: 0.3rem;
      padding: 0.9rem;
      border-radius: 16px;
      border: 1px solid rgba(157, 124, 216, 0.2);
      background: rgba(255, 255, 255, 0.02);
    }}
    .bridge p {{
      color: var(--text);
    }}
    code {{
      font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
      font-size: 0.92em;
      color: var(--accent-2);
    }}
    button {{
      appearance: none;
      border: 1px solid rgba(255,255,255,0.1);
      background: rgba(255,255,255,0.03);
      color: var(--text);
      border-radius: 999px;
      padding: 0.65rem 0.9rem;
      font: inherit;
    }}
    button.primary {{
      background: linear-gradient(180deg, rgba(157,124,216,0.32), rgba(157,124,216,0.18));
      border-color: rgba(157,124,216,0.35);
    }}
    ul {{
      list-style: none;
      padding: 0;
      margin: 0;
      display: grid;
      gap: 0.7rem;
    }}
    li {{
      padding: 0.8rem;
      border-radius: 14px;
      border: 1px solid rgba(255,255,255,0.06);
      background: rgba(255,255,255,0.02);
    }}
    li strong {{
      display: block;
      margin-bottom: 0.2rem;
      color: var(--text);
    }}
    .muted {{
      color: var(--muted);
    }}
    .section-spacer {{
      display: grid;
      gap: 0.8rem;
    }}
    .label {{
      text-transform: uppercase;
      letter-spacing: 0.16em;
      font-size: 0.72rem;
      color: rgba(167, 156, 190, 0.78);
    }}
    .truth {{
      display: grid;
      gap: 0.65rem;
    }}
    .truth-row {{
      display: flex;
      justify-content: space-between;
      gap: 1rem;
      padding: 0.55rem 0;
      border-bottom: 1px solid rgba(255,255,255,0.06);
    }}
    .truth-row:last-child {{
      border-bottom: 0;
    }}
    .truth-row span:last-child {{
      color: var(--text);
      text-align: right;
    }}
    .footer-note {{
      font-size: 0.8rem;
      color: rgba(167, 156, 190, 0.7);
    }}
    .status-note {{
      color: var(--accent-2);
      min-height: 1.2rem;
      font-size: 0.88rem;
    }}
    @media (max-width: 1100px) {{
      .shell {{ grid-template-columns: 1fr; }}
      aside, .detail {{ border-left: 0; border-right: 0; border-top: 1px solid var(--line); }}
      .grid {{ grid-template-columns: 1fr; }}
    }}
  </style>
</head>
<body>
  <div class="shell">
    <aside>
      <div class="eyebrow">Memory Home</div>
      <h2>Visible memories</h2>
      <p>Workspaces, lanes, and connected sources stay visible here.</p>
      <div style="margin-top: 1rem;" class="section-spacer">
        <div class="label">Navigation</div>
        <div class="tag">Memory Home</div>
        <div class="tag">Knowledge Map</div>
        <div class="tag">Working Memory</div>
        <div class="tag">Inbox</div>
        <div class="tag">Repair</div>
      </div>
    </aside>
    <main>
      <section class="panel hero">
        <div class="eyebrow">Selected Artifact</div>
        <h1 id="artifact-title">{title}</h1>
        <p id="artifact-body">{body}</p>
        <div class="meta" id="artifact-meta">
          <span id="artifact-kind">{artifact_kind}</span>
          <span id="artifact-status">{status}</span>
          <span id="artifact-freshness">{freshness}</span>
          <span id="artifact-workspace">workspace: {workspace}</span>
        </div>
        <p><a id="artifact-detail-link" href="{detail_href}">Open selected artifact detail</a></p>
        <p class="status-note" id="action-status">Snapshot loaded.</p>
        <div class="actions">
          <button class="primary" data-action="inspect" data-artifact-id="{focus_id}">Inspect</button>
          <button data-action="explain" data-artifact-id="{focus_id}">Explain</button>
          <button data-action="verify_current" data-artifact-id="{focus_id}">Verify Current</button>
          <button data-action="mark_stale" data-artifact-id="{focus_id}">Mark Stale</button>
          <button data-action="promote" data-artifact-id="{focus_id}">Promote</button>
          <button data-action="open_source" data-artifact-id="{focus_id}">Open Source</button>
          {obsidian_bridge}
        </div>
      </section>
      <section class="panel">
        <div class="eyebrow">Knowledge Map</div>
        <h2>Linked artifacts</h2>
        <div class="meta" style="margin-bottom: 0.75rem;">
          <span>{related_nodes} linked artifacts</span>
          <span>{timeline_nodes} timeline events</span>
          <span>{workspace_nodes} workspace lanes</span>
        </div>
        <ul>{knowledge_rows}</ul>
      </section>
      <section class="panel">
        <div class="eyebrow">Artifact Drilldown</div>
        <h2 id="detail-heading">Selected artifact detail</h2>
        <div class="grid">
          <div class="block">
            <div class="label">Timeline</div>
            <strong id="detail-timeline-count">0</strong>
            <p id="detail-timeline-copy">No timeline loaded</p>
          </div>
          <div class="block">
            <div class="label">Sources</div>
            <strong id="detail-source-count">0</strong>
            <p id="detail-source-copy">No source lanes loaded</p>
          </div>
          <div class="block">
            <div class="label">Linked Artifacts</div>
            <strong id="detail-related-count">0</strong>
            <p id="detail-related-copy">No related artifacts loaded</p>
          </div>
        </div>
      </section>
      <section class="panel">
        <div class="eyebrow">Hive Board</div>
        <h2>Live bees</h2>
        <div class="meta" style="margin-bottom: 0.75rem;">
          <span id="hive-queen">queen: loading</span>
          <span id="hive-active-count">active: 0</span>
          <span id="hive-review-count">review: 0</span>
        </div>
        <div class="grid">
          <div class="block">
            <div class="label">Active Bees</div>
            <strong id="hive-active-total">0</strong>
            <p id="hive-active-copy">No hive board loaded</p>
          </div>
          <div class="block">
            <div class="label">Overlap Risks</div>
            <strong id="hive-risk-total">0</strong>
            <p id="hive-risk-copy">No overlap data loaded</p>
          </div>
          <div class="block">
            <div class="label">Stale Bees</div>
            <strong id="hive-stale-total">0</strong>
            <p id="hive-stale-copy">No stale hive data loaded</p>
          </div>
        </div>
        <div class="actions" style="margin-top: 1rem;">
          <button class="primary" type="button" data-hive-queen-action="auto-retire">Auto Retire Stale</button>
          <button type="button" data-hive-queen-action="retire-focused">Retire Focused Bee</button>
          <button type="button" data-hive-queen-action="deny-focused">Deny Focused Bee</button>
          <button type="button" data-hive-queen-action="reroute-focused">Reroute Focused Bee</button>
          <button type="button" data-hive-queen-action="handoff-focused">Handoff Scope</button>
        </div>
        <div class="section-spacer" style="margin-top: 1rem;">
          <div>
            <div class="label">Roster</div>
            <ul id="hive-roster-list"><li class="muted">Loading hive roster…</li></ul>
          </div>
          <div>
            <div class="label">Focused Bee</div>
            <ul id="hive-follow-list"><li class="muted">Loading hive focus…</li></ul>
          </div>
        </div>
      </section>
      <section class="grid">
        <div class="block">
          <div class="label">Memory Home</div>
          <strong>{inbox_count}</strong>
          <p>Inbox items</p>
        </div>
        <div class="block">
          <div class="label">Repair Pressure</div>
          <strong>{repair_count}</strong>
          <p>Items needing attention</p>
        </div>
        <div class="block">
          <div class="label">Workspace Awareness</div>
          <strong>{awareness_count}</strong>
          <p>Visible workspace records</p>
        </div>
      </section>
    </main>
    <aside class="detail">
      <div class="eyebrow">Truth</div>
      <div class="truth">
        <div class="truth-row"><span>Source system</span><span id="truth-source-system">{source_system}</span></div>
        <div class="truth-row"><span>Source path</span><span id="truth-source-path">{source_path}</span></div>
        <div class="truth-row"><span>Producer</span><span id="truth-producer">{producer}</span></div>
        <div class="truth-row"><span>Confidence</span><span id="truth-confidence">{confidence}</span></div>
        <div class="truth-row"><span>Repair state</span><span id="truth-repair-state">{repair_state}</span></div>
        <div class="truth-row"><span>Sessions</span><span id="truth-session-count">0</span></div>
        <div class="truth-row"><span>Tasks</span><span id="truth-task-count">0</span></div>
        <div class="truth-row"><span>Claims</span><span id="truth-claim-count">0</span></div>
      </div>
      <p class="footer-note" style="margin-top: 1rem;">The shell is snapshot-backed. Obsidian is a bridge, not a rebuild.</p>
    </aside>
  </div>
  <script>
    const selectedState = {{
      artifactId: "{focus_id}",
    }};

    function text(id, value) {{
      const el = document.getElementById(id);
      if (el) el.textContent = value;
    }}

    function actionButtons() {{
      return Array.from(document.querySelectorAll('[data-action]'));
    }}

    function renderList(id, items) {{
      const el = document.getElementById(id);
      if (!el) return;
      if (!items.length) {{
        el.innerHTML = '<li class="muted">none</li>';
        return;
      }}
      el.innerHTML = items.map((item) => `<li>${{item}}</li>`).join('');
    }}

    function setActionArtifact(id) {{
      selectedState.artifactId = id;
      for (const button of actionButtons()) {{
        button.dataset.artifactId = id;
      }}
      const link = document.getElementById('artifact-detail-link');
      if (link) link.href = `/ui/artifact?id=${{id}}`;
    }}

    function renderArtifactDetail(detail) {{
      const artifact = detail.artifact;
      text('artifact-title', artifact.title);
      text('artifact-body', artifact.body);
      text('artifact-kind', artifact.artifact_kind);
      text('artifact-status', artifact.status);
      text('artifact-freshness', artifact.freshness);
      text('artifact-workspace', `workspace: ${{artifact.workspace || 'none'}}`);
      text('detail-heading', artifact.title);
      text('truth-source-system', artifact.provenance.source_system || 'memd');
      text('truth-source-path', artifact.provenance.source_path || 'none');
      text('truth-producer', artifact.provenance.producer || 'memd');
      text('truth-confidence', artifact.confidence.toFixed(2));
      text('truth-repair-state', artifact.repair_state);
      text('truth-session-count', String(detail.sessions.sessions.length));
      text('truth-task-count', String(detail.tasks.tasks.length));
      text('truth-claim-count', String(detail.claims.claims.length));
      text('detail-timeline-count', String(detail.timeline ? detail.timeline.events.length : 0));
      text('detail-timeline-copy', detail.timeline ? 'Event history loaded' : 'No timeline loaded');
      text('detail-source-count', String(detail.sources.sources.length));
      text('detail-source-copy', detail.sources.sources.length ? 'Source lanes loaded' : 'No source lanes loaded');
      text('detail-related-count', String(detail.related_artifacts.length));
      text('detail-related-copy', detail.related_artifacts.length ? 'Related artifacts loaded' : 'No related artifacts loaded');
      setActionArtifact(artifact.id);
    }}

    async function loadArtifactDetail(id) {{
      const response = await fetch(`/ui/artifact?id=${{id}}`);
      if (!response.ok) throw new Error(`artifact detail failed: ${{response.status}}`);
      const detail = await response.json();
      renderArtifactDetail(detail);
      text('action-status', `Loaded artifact ${{detail.artifact.title}}`);
    }}

    async function runAction(action, id) {{
      const response = await fetch('/ui/action', {{
        method: 'POST',
        headers: {{ 'Content-Type': 'application/json' }},
        body: JSON.stringify({{ id, action }}),
      }});
      if (!response.ok) throw new Error(`action failed: ${{response.status}}`);
      const result = await response.json();
      if (result.detail) {{
        renderArtifactDetail(result.detail);
      }}
      text('action-status', result.message || `${{action}} complete`);
      if (result.open_uri) {{
        window.open(result.open_uri, '_blank');
      }}
    }}

    const selectedHiveBoardState = {{
      queenSession: null,
      project: null,
      namespace: null,
      workspace: null,
    }};
    let hiveRefreshInFlight = {{ value: false }};
    const hiveRefreshIntervalMs = 5000;

    function focusedHiveBeeLabel() {{
      return selectedHiveFollowSession.value || 'none';
    }}

    async function postHiveAction(path, payload) {{
      const response = await fetch(path, {{
        method: 'POST',
        headers: {{ 'Content-Type': 'application/json' }},
        body: JSON.stringify(payload),
      }});
      if (!response.ok) throw new Error(`queen action failed: ${{response.status}}`);
      return response.json();
    }}

    async function runHiveQueenAction(action) {{
      if (action === 'auto-retire') {{
        const response = await fetch('/coordination/sessions/auto-retire', {{
          method: 'POST',
          headers: {{ 'Content-Type': 'application/json' }},
          body: JSON.stringify({{}}),
        }});
        if (!response.ok) throw new Error(`queen action failed: ${{response.status}}`);
        const result = await response.json();
        await reloadHiveBoard(selectedHiveFollowSession.value);
        text('action-status', result.retired && result.retired.length
          ? `Retired stale bees: ${{result.retired.join(', ')}}`
          : 'No stale bees to retire');
        return;
      }}

      if (!selectedHiveBoardState.queenSession) {{
        throw new Error('no queen session available');
      }}

      if (action === 'retire-focused') {{
        if (!selectedHiveFollowSession.value) {{
          throw new Error('no focused bee selected');
        }}
        const response = await fetch('/coordination/sessions/retire', {{
          method: 'POST',
          headers: {{ 'Content-Type': 'application/json' }},
          body: JSON.stringify({{ session: selectedHiveFollowSession.value }}),
        }});
        if (!response.ok) throw new Error(`queen action failed: ${{response.status}}`);
        const result = await response.json();
        const retired = (result.sessions || []).map((session) => session.worker_name || session.session);
        selectedHiveFollowSession.value = null;
        await reloadHiveBoard(null);
        text('action-status', retired.length
          ? `Retired bee: ${{retired.join(', ')}}`
          : 'No bee retired');
        return;
      }}

      if (!selectedHiveFollowSession.value) {{
        throw new Error('no focused bee selected');
      }}

      if (action === 'deny-focused') {{
        const result = await postHiveAction('/hive/queen/deny', {{
          queen_session: selectedHiveBoardState.queenSession,
          target_session: selectedHiveFollowSession.value,
          project: selectedHiveBoardState.project,
          namespace: selectedHiveBoardState.namespace,
          workspace: selectedHiveBoardState.workspace,
        }});
        await reloadHiveBoard(selectedHiveFollowSession.value);
        text('action-status', result.summary || `Denied focused bee: ${{focusedHiveBeeLabel()}}`);
        return;
      }}

      if (action === 'reroute-focused') {{
        const result = await postHiveAction('/hive/queen/reroute', {{
          queen_session: selectedHiveBoardState.queenSession,
          target_session: selectedHiveFollowSession.value,
          project: selectedHiveBoardState.project,
          namespace: selectedHiveBoardState.namespace,
          workspace: selectedHiveBoardState.workspace,
        }});
        await reloadHiveBoard(selectedHiveFollowSession.value);
        text('action-status', result.summary || `Reroute recorded for: ${{focusedHiveBeeLabel()}}`);
        return;
      }}

      if (action === 'handoff-focused') {{
        const scope = window.prompt('Scope to hand off', '');
        if (!scope || !scope.trim()) {{
          throw new Error('handoff scope required');
        }}
        const note = window.prompt('Optional handoff note', '') || '';
        const result = await postHiveAction('/hive/queen/handoff', {{
          queen_session: selectedHiveBoardState.queenSession,
          target_session: selectedHiveFollowSession.value,
          scope: scope.trim(),
          project: selectedHiveBoardState.project,
          namespace: selectedHiveBoardState.namespace,
          workspace: selectedHiveBoardState.workspace,
          note: note.trim() || null,
        }});
        await reloadHiveBoard(selectedHiveFollowSession.value);
        text('action-status', result.summary || `Handoff recorded for: ${{focusedHiveBeeLabel()}} on ${{scope.trim()}}`);
      }}
    }}

    async function loadHiveBoard() {{
      const response = await fetch('/hive/board');
      if (!response.ok) throw new Error(`hive board failed: ${{response.status}}`);
      const board = await response.json();
      selectedHiveBoardState.queenSession = board.queen_session || null;
      selectedHiveBoardState.project = board.project || null;
      selectedHiveBoardState.namespace = board.namespace || null;
      selectedHiveBoardState.workspace = board.workspace || null;
      text('hive-queen', `queen: ${{board.queen_session || 'none'}}`);
      text('hive-active-count', `active: ${{board.active_bees.length}}`);
      text('hive-review-count', `review: ${{board.review_queue.length}}`);
      text('hive-active-total', String(board.active_bees.length));
      text('hive-active-copy', board.active_bees.length ? 'Live hive board connected' : 'No active bees');
      text('hive-risk-total', String(board.overlap_risks.length));
      text('hive-risk-copy', board.overlap_risks.length ? board.overlap_risks[0] : 'No overlap data loaded');
      text('hive-stale-total', String(board.stale_bees.length));
      text('hive-stale-copy', board.stale_bees.length ? board.stale_bees[0] : 'No stale hive data loaded');
      return board;
    }}

    let selectedHiveFollowSession = {{ value: null }};

    function beeLabel(bee) {{
      return bee.worker_name || bee.display_name || bee.agent || bee.session || 'unnamed';
    }}

    function renderHiveRosterItems(bees) {{
      return (bees || []).slice(0, 8).map((bee) => {{
        const worker = beeLabel(bee);
        const role = bee.role || bee.hive_role || 'worker';
        const lane = bee.lane_id || bee.branch || 'none';
        const task = bee.task_id || 'none';
        const selected = selectedHiveFollowSession.value === bee.session ? ' selected' : '';
        return `<button type="button" class="artifact-link hive-bee${{selected}}" data-hive-follow-session="${{bee.session}}"><strong>${{worker}}</strong><span>${{role}} · ${{lane}} · task=${{task}}</span></button>`;
      }});
    }}

    async function loadHiveRoster() {{
      const response = await fetch('/hive/roster');
      if (!response.ok) throw new Error(`hive roster failed: ${{response.status}}`);
      const roster = await response.json();
      renderList('hive-roster-list', renderHiveRosterItems(roster.bees));
      return roster;
    }}

    async function loadHiveFollow(session) {{
      if (!session) {{
        selectedHiveFollowSession.value = null;
        renderList('hive-follow-list', []);
        return null;
      }}
      selectedHiveFollowSession.value = session;
      const response = await fetch(`/hive/follow?session=${{encodeURIComponent(session)}}`);
      if (!response.ok) throw new Error(`hive follow failed: ${{response.status}}`);
      const follow = await response.json();
      const lane = follow.target.lane_id || follow.target.branch || 'none';
      const role = follow.target.role || follow.target.hive_role || 'worker';
      const task = follow.target.task_id || 'none';
      const nextAction = follow.next_action || 'none';
      const latestMessage = (follow.messages || [])[0];
      const latestReceipt = (follow.recent_receipts || [])[0];
      const items = [
        `<strong>${{follow.target.worker_name || follow.target.agent || follow.target.session}}</strong><span>role=${{role}} · lane=${{lane}} · task=${{task}}</span>`,
        `<strong>work</strong><span>${{follow.work_summary || 'none'}}</span>`,
        `<strong>touches</strong><span>${{(follow.touch_points || []).join(',') || 'none'}}</span>`,
        `<strong>action</strong><span>${{follow.recommended_action}}</span>`,
        `<strong>next</strong><span>${{nextAction}}</span>`,
      ];
      if (follow.overlap_risk) {{
        items.push(`<strong>overlap</strong><span>${{follow.overlap_risk}}</span>`);
      }}
      if (latestMessage) {{
        items.push(`<strong>latest message</strong><span>${{latestMessage.kind}} · ${{latestMessage.from_agent || latestMessage.from_session}} · ${{latestMessage.content.replace(/\\s+/g, ' ').slice(0, 120)}}</span>`);
      }}
      if (latestReceipt) {{
        items.push(`<strong>latest receipt</strong><span>${{latestReceipt.kind}} · ${{latestReceipt.summary}}</span>`);
      }}
      renderList('hive-follow-list', items);
      return follow;
    }}

    async function reloadHiveBoard(preferredSession) {{
      if (hiveRefreshInFlight.value) {{
        return null;
      }}
      hiveRefreshInFlight.value = true;
      try {{
      const [board, roster] = await Promise.all([loadHiveBoard(), loadHiveRoster()]);
      const rosterSessions = (roster && roster.bees ? roster.bees : []).map((bee) => bee.session);
      const nextSession = preferredSession && rosterSessions.includes(preferredSession)
        ? preferredSession
        : ((board && board.active_bees && board.active_bees[0] && board.active_bees[0].session) || null);
      await loadHiveFollow(nextSession);
      return {{ board, roster, nextSession }};
      }} finally {{
        hiveRefreshInFlight.value = false;
      }}
    }}

    async function refreshHiveBoardIfVisible() {{
      if (document.hidden) {{
        return;
      }}
      try {{
        await reloadHiveBoard(selectedHiveFollowSession.value);
      }} catch (error) {{
        text('action-status', `hive refresh failed: ${{String(error)}}`);
      }}
    }}

    document.addEventListener('click', (event) => {{
      const actionButton = event.target.closest('[data-action]');
      if (actionButton) {{
        event.preventDefault();
        runAction(actionButton.dataset.action, actionButton.dataset.artifactId).catch((error) => {{
          text('action-status', String(error));
        }});
        return;
      }}
      const hiveQueenAction = event.target.closest('[data-hive-queen-action]');
      if (hiveQueenAction) {{
        event.preventDefault();
        runHiveQueenAction(hiveQueenAction.dataset.hiveQueenAction).catch((error) => {{
          text('action-status', String(error));
        }});
        return;
      }}
      const hiveFollowButton = event.target.closest('[data-hive-follow-session]');
      if (hiveFollowButton) {{
        event.preventDefault();
        loadHiveFollow(hiveFollowButton.dataset.hiveFollowSession)
          .then(() => loadHiveRoster())
          .catch((error) => {{
            renderList('hive-follow-list', [`<span class="muted">${{String(error)}}</span>`]);
          }});
        return;
      }}
      const artifactLink = event.target.closest('.artifact-link');
      if (artifactLink) {{
        event.preventDefault();
        loadArtifactDetail(artifactLink.dataset.artifactId).catch((error) => {{
          text('action-status', String(error));
        }});
        return;
      }}
    }});

    loadArtifactDetail(selectedState.artifactId).catch((error) => {{
      text('action-status', String(error));
    }});
    reloadHiveBoard(selectedHiveFollowSession.value)
      .catch((error) => {{
        renderList('hive-roster-list', [`<span class="muted">${{String(error)}}</span>`]);
        renderList('hive-follow-list', [`<span class="muted">${{String(error)}}</span>`]);
      }});
    window.setInterval(refreshHiveBoardIfVisible, hiveRefreshIntervalMs);
  </script>
</body>
</html>
"#,
        title = escape_html(&focus.title),
        body = escape_html(&focus.body),
        artifact_kind = escape_html(&focus.artifact_kind),
        status = format!("{:?}", focus.status),
        freshness = escape_html(&focus.freshness),
        workspace = escape_html(focus.workspace.as_deref().unwrap_or("none")),
        inbox_count = snapshot.home.inbox_count,
        repair_count = snapshot.home.repair_count,
        awareness_count = snapshot.home.awareness_count,
        source_system = escape_html(source_system),
        source_path = escape_html(source_path),
        producer = escape_html(producer),
        confidence = format!("{:.2}", focus.confidence),
        repair_state = escape_html(&focus.repair_state),
        obsidian_bridge = obsidian_bridge,
        related_nodes = related_nodes,
        timeline_nodes = timeline_nodes,
        workspace_nodes = workspace_nodes,
        detail_href = detail_href,
        focus_id = focus_id,
    )
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub(crate) fn build_visible_memory_snapshot(
    state: &AppState,
) -> anyhow::Result<VisibleMemorySnapshotResponse> {
    let items = state.snapshot()?;
    let focus_item = items
        .iter()
        .find(|item| item.preferred)
        .or_else(|| items.first())
        .ok_or_else(|| anyhow::anyhow!("no memory items available"))?;

    let inbox_items = inbox_items(&items);
    let repair_items = repair_items(&items);
    let workspace_records = workspace_records(state)?;
    let timeline_events = timeline_events(state, focus_item.id, 4)?;
    let inbox_count = inbox_items.len();
    let repair_count = repair_items.len();
    let awareness_count = workspace_records.len();

    let focus_artifact = visible_artifact(focus_item);
    let knowledge_map =
        build_knowledge_map(focus_item, &items, &timeline_events, &workspace_records);

    Ok(VisibleMemorySnapshotResponse {
        generated_at: Utc::now(),
        home: VisibleMemoryHome {
            focus_artifact,
            inbox_count,
            repair_count,
            awareness_count,
        },
        knowledge_map,
    })
}

pub(crate) fn build_visible_memory_artifact_detail(
    state: &AppState,
    id: Uuid,
) -> Result<VisibleMemoryArtifactDetailResponse, (StatusCode, String)> {
    let items = state.snapshot().map_err(internal_error)?;
    let focus_item = items
        .iter()
        .find(|item| item.id == id)
        .cloned()
        .ok_or_else(|| (StatusCode::NOT_FOUND, "memory item not found".to_string()))?;
    build_visible_memory_artifact_detail_from_item(state, focus_item, &items)
}

pub(crate) fn perform_visible_memory_action(
    state: &AppState,
    req: VisibleMemoryUiActionRequest,
) -> Result<VisibleMemoryUiActionResponse, (StatusCode, String)> {
    let item = state
        .store
        .get(req.id)
        .map_err(internal_error)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "memory item not found".to_string()))?;

    let action = req.action;
    let snapshot = state.snapshot().map_err(internal_error)?;
    let (artifact, outcome, message, detail, open_uri, source_path) = match action {
        VisibleMemoryUiActionKind::Inspect => {
            let detail =
                build_visible_memory_artifact_detail_from_item(state, item.clone(), &snapshot)?;
            (
                item,
                "inspected".to_string(),
                "built selected artifact detail".to_string(),
                Some(detail),
                None,
                None,
            )
        }
        VisibleMemoryUiActionKind::Explain => {
            let detail =
                build_visible_memory_artifact_detail_from_item(state, item.clone(), &snapshot)?;
            (
                item,
                "explained".to_string(),
                "reused explain, timeline, source, workspace, and coordination data".to_string(),
                Some(detail),
                None,
                None,
            )
        }
        VisibleMemoryUiActionKind::VerifyCurrent => {
            let updated = crate::repair::verify_item(
                state,
                memd_schema::VerifyMemoryRequest {
                    id: item.id,
                    confidence: Some(item.confidence),
                    status: Some(MemoryStatus::Active),
                },
            )?;
            let detail = build_visible_memory_artifact_detail_from_item(
                state,
                updated.clone(),
                &state.snapshot().map_err(internal_error)?,
            )?;
            (
                updated,
                "verified".to_string(),
                "marked memory current and updated verification state".to_string(),
                Some(detail),
                None,
                None,
            )
        }
        VisibleMemoryUiActionKind::MarkStale => {
            let updated = crate::repair::repair_item(
                state,
                RepairMemoryRequest {
                    id: item.id,
                    mode: MemoryRepairMode::Expire,
                    confidence: Some(item.confidence),
                    status: Some(MemoryStatus::Stale),
                    workspace: None,
                    visibility: None,
                    source_agent: None,
                    source_system: None,
                    source_path: None,
                    source_quality: None,
                    content: None,
                    tags: None,
                    supersedes: Vec::new(),
                },
            )?
            .item;
            let detail = build_visible_memory_artifact_detail_from_item(
                state,
                updated.clone(),
                &state.snapshot().map_err(internal_error)?,
            )?;
            (
                updated,
                "marked_stale".to_string(),
                "marked artifact stale".to_string(),
                Some(detail),
                None,
                None,
            )
        }
        VisibleMemoryUiActionKind::Promote => {
            let (updated, duplicate): (MemoryItem, Option<crate::store::DuplicateMatch>) = state
                .promote_item(memd_schema::PromoteMemoryRequest {
                    id: item.id,
                    scope: None,
                    project: None,
                    namespace: None,
                    workspace: None,
                    visibility: None,
                    belief_branch: None,
                    confidence: None,
                    ttl_seconds: None,
                    tags: None,
                    status: None,
                })
                .map_err(internal_error)?;
            let effective = duplicate
                .as_ref()
                .map_or(updated.clone(), |found| found.item.clone());
            let detail = build_visible_memory_artifact_detail_from_item(
                state,
                effective.clone(),
                &state.snapshot().map_err(internal_error)?,
            )?;
            (
                effective,
                "promoted".to_string(),
                if duplicate.is_some() {
                    "promoted artifact and resolved duplicate".to_string()
                } else {
                    "promoted artifact to canonical stage".to_string()
                },
                Some(detail),
                None,
                None,
            )
        }
        VisibleMemoryUiActionKind::OpenSource => {
            let detail =
                build_visible_memory_artifact_detail_from_item(state, item.clone(), &snapshot)?;
            (
                item.clone(),
                "metadata".to_string(),
                "returned source metadata for the selected artifact".to_string(),
                Some(detail),
                None,
                item.source_path.clone(),
            )
        }
        VisibleMemoryUiActionKind::OpenInObsidian => {
            let detail =
                build_visible_memory_artifact_detail_from_item(state, item.clone(), &snapshot)?;
            let open_uri = item
                .source_path
                .as_ref()
                .map(|path| build_obsidian_uri(path));
            (
                item.clone(),
                "metadata".to_string(),
                if open_uri.is_some() {
                    "generated Obsidian open URI".to_string()
                } else {
                    "no Obsidian path available".to_string()
                },
                Some(detail),
                open_uri,
                item.source_path.clone(),
            )
        }
    };

    Ok(VisibleMemoryUiActionResponse {
        action,
        artifact_id: artifact.id,
        outcome,
        message,
        detail,
        open_uri,
        source_path,
    })
}

fn workspace_records(state: &AppState) -> anyhow::Result<Vec<WorkspaceMemoryRecord>> {
    let response = state.store.workspace_memory(&WorkspaceMemoryRequest {
        project: None,
        namespace: None,
        workspace: None,
        visibility: None,
        source_agent: None,
        source_system: None,
        limit: Some(32),
    })?;
    Ok(response.workspaces)
}

fn timeline_events(
    state: &AppState,
    item_id: Uuid,
    limit: usize,
) -> anyhow::Result<Vec<MemoryEventRecord>> {
    let (entity, events) = state.entity_view(item_id, limit)?;
    Ok(if entity.is_some() { events } else { Vec::new() })
}

fn build_visible_memory_artifact_detail_from_item(
    state: &AppState,
    item: MemoryItem,
    items: &[MemoryItem],
) -> Result<VisibleMemoryArtifactDetailResponse, (StatusCode, String)> {
    let artifact = visible_artifact(&item);
    let explain = Some(crate::inspection::explain_memory(
        state,
        ExplainMemoryRequest {
            id: item.id,
            belief_branch: item.belief_branch.clone(),
            route: None,
            intent: None,
        },
    )?);

    let timeline: Option<TimelineMemoryResponse> = {
        let limit = 8;
        let (entity, events): (
            Option<memd_schema::MemoryEntityRecord>,
            Vec<MemoryEventRecord>,
        ) = state.entity_view(item.id, limit).map_err(internal_error)?;
        entity.map(|entity| TimelineMemoryResponse {
            route: memd_schema::RetrievalRoute::Auto,
            intent: memd_schema::RetrievalIntent::General,
            entity: Some(entity),
            events,
        })
    };

    let sources = state
        .store
        .source_memory(&SourceMemoryRequest {
            project: item.project.clone(),
            namespace: item.namespace.clone(),
            workspace: item.workspace.clone(),
            visibility: Some(item.visibility),
            source_agent: item.source_agent.clone(),
            source_system: item.source_system.clone(),
            limit: Some(8),
        })
        .map_err(internal_error)?;

    let workspaces = state
        .store
        .workspace_memory(&WorkspaceMemoryRequest {
            project: item.project.clone(),
            namespace: item.namespace.clone(),
            workspace: item.workspace.clone(),
            visibility: Some(item.visibility),
            source_agent: item.source_agent.clone(),
            source_system: item.source_system.clone(),
            limit: Some(8),
        })
        .map_err(internal_error)?;

    let sessions = state
        .store
        .hive_sessions(&HiveSessionsRequest {
            session: None,
            project: item.project.clone(),
            namespace: item.namespace.clone(),
            repo_root: None,
            worktree_root: None,
            branch: None,
            workspace: item.workspace.clone(),
            hive_system: None,
            hive_role: None,
            host: None,
            hive_group: None,
            active_only: Some(true),
            limit: Some(8),
        })
        .map_err(internal_error)?;

    let tasks = state
        .store
        .hive_tasks(&HiveTasksRequest {
            session: None,
            project: item.project.clone(),
            namespace: item.namespace.clone(),
            workspace: item.workspace.clone(),
            active_only: Some(true),
            limit: Some(8),
        })
        .map_err(internal_error)?;

    let claims = state
        .store
        .hive_claims(&HiveClaimsRequest {
            session: None,
            project: item.project.clone(),
            namespace: item.namespace.clone(),
            workspace: item.workspace.clone(),
            active_only: Some(true),
            limit: Some(8),
        })
        .map_err(internal_error)?;

    let timeline_events = timeline
        .as_ref()
        .map(|entry| entry.events.clone())
        .unwrap_or_default();
    let related_map = build_knowledge_map(&item, items, &timeline_events, &workspaces.workspaces);
    let related_artifacts = related_visible_items(&item, items);

    Ok(VisibleMemoryArtifactDetailResponse {
        generated_at: Utc::now(),
        artifact,
        explain,
        timeline,
        sources,
        workspaces,
        sessions,
        tasks,
        claims,
        related_artifacts,
        related_map,
        actions: visible_actions_for_item(&item),
    })
}

fn related_visible_items(item: &MemoryItem, items: &[MemoryItem]) -> Vec<VisibleMemoryArtifact> {
    let mut related = items
        .iter()
        .filter(|candidate| candidate.id != item.id)
        .filter(|candidate| {
            candidate.workspace == item.workspace
                || candidate.project == item.project
                || candidate.supersedes.contains(&item.id)
                || item.supersedes.contains(&candidate.id)
                || candidate.source_path == item.source_path
                || candidate.source_system == item.source_system
                || candidate.source_agent == item.source_agent
        })
        .map(visible_artifact)
        .collect::<Vec<_>>();
    related.sort_by(|a, b| a.title.cmp(&b.title));
    related.truncate(12);
    related
}

fn visible_actions_for_item(item: &MemoryItem) -> Vec<VisibleMemoryUiActionKind> {
    let mut actions = vec![
        VisibleMemoryUiActionKind::Inspect,
        VisibleMemoryUiActionKind::Explain,
        VisibleMemoryUiActionKind::VerifyCurrent,
        VisibleMemoryUiActionKind::MarkStale,
        VisibleMemoryUiActionKind::Promote,
    ];
    if item.source_path.is_some() {
        actions.push(VisibleMemoryUiActionKind::OpenSource);
        actions.push(VisibleMemoryUiActionKind::OpenInObsidian);
    }
    actions
}

fn build_obsidian_uri(path: &str) -> String {
    format!("obsidian://open?path={}", percent_encode_path(path))
}

fn percent_encode_path(path: &str) -> String {
    let mut encoded = String::with_capacity(path.len());
    for byte in path.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                encoded.push(byte as char);
            }
            _ => {
                use std::fmt::Write as _;
                let _ = write!(encoded, "%{:02X}", byte);
            }
        }
    }
    encoded
}

fn build_knowledge_map(
    focus_item: &MemoryItem,
    items: &[MemoryItem],
    timeline_events: &[MemoryEventRecord],
    workspace_records: &[WorkspaceMemoryRecord],
) -> VisibleMemoryKnowledgeMap {
    let mut nodes = items
        .iter()
        .map(|item| VisibleMemoryGraphNode {
            artifact_id: item.id,
            title: artifact_title(item),
            artifact_kind: artifact_kind(item),
            status: visible_status(item),
        })
        .collect::<Vec<_>>();

    let mut edges = Vec::new();
    for artifact in items.iter() {
        if artifact.id == focus_item.id {
            continue;
        }
        if artifact.workspace == focus_item.workspace || artifact.project == focus_item.project {
            edges.push(VisibleMemoryGraphEdge {
                from: focus_item.id,
                to: artifact.id,
                relation: "related".to_string(),
            });
        }
    }

    for event in timeline_events {
        edges.push(VisibleMemoryGraphEdge {
            from: focus_item.id,
            to: event.id,
            relation: "timeline".to_string(),
        });
    }

    for workspace in workspace_records.iter().take(6) {
        let node_id = workspace_node_id(workspace);
        edges.push(VisibleMemoryGraphEdge {
            from: focus_item.id,
            to: node_id,
            relation: format!("workspace:{}", workspace_title(workspace)),
        });
    }

    for event in timeline_events {
        nodes.push(VisibleMemoryGraphNode {
            artifact_id: event.id,
            title: event.summary.clone(),
            artifact_kind: "timeline_event".to_string(),
            status: if event.confidence >= 0.75 {
                VisibleMemoryStatus::Current
            } else {
                VisibleMemoryStatus::Candidate
            },
        });
    }

    for workspace in workspace_records.iter().take(6) {
        let node_id = workspace_node_id(workspace);
        nodes.push(VisibleMemoryGraphNode {
            artifact_id: node_id,
            title: workspace_title(workspace),
            artifact_kind: "workspace_lane".to_string(),
            status: workspace_status(workspace),
        });
    }

    VisibleMemoryKnowledgeMap { nodes, edges }
}

fn visible_artifact(item: &MemoryItem) -> VisibleMemoryArtifact {
    let status = visible_status(item);
    let freshness = freshness_label(item, status);
    let repair_state = repair_state_label(item, status);
    let title = artifact_title(item);
    let mut actions = vec![
        "inspect".to_string(),
        "explain".to_string(),
        "verify_current".to_string(),
        "mark_stale".to_string(),
        "supersede".to_string(),
        "promote".to_string(),
    ];
    if item.source_path.is_some() {
        actions.push("open_in_obsidian".to_string());
    }

    VisibleMemoryArtifact {
        id: item.id,
        title,
        body: item.content.clone(),
        artifact_kind: artifact_kind(item),
        memory_kind: Some(item.kind),
        scope: Some(item.scope),
        visibility: Some(item.visibility),
        workspace: item.workspace.clone(),
        status,
        freshness,
        confidence: item.confidence,
        provenance: VisibleMemoryProvenance {
            source_system: item.source_system.clone(),
            source_path: item.source_path.clone(),
            producer: item.source_agent.clone(),
            trust_reason: trust_reason(item, status),
            last_verified_at: item.last_verified_at,
        },
        sources: item.source_path.clone().into_iter().collect(),
        linked_artifact_ids: item.supersedes.clone(),
        linked_sessions: item.workspace.clone().into_iter().collect(),
        linked_agents: item.source_agent.clone().into_iter().collect(),
        repair_state,
        actions,
    }
}

fn artifact_title(item: &MemoryItem) -> String {
    let candidate = item
        .source_path
        .as_deref()
        .and_then(|path| std::path::Path::new(path).file_stem())
        .and_then(|stem| stem.to_str())
        .unwrap_or_else(|| item.content.lines().next().unwrap_or("memory item"));
    let title = candidate.trim();
    if title.is_empty() {
        "memory item".to_string()
    } else {
        title.to_string()
    }
}

fn artifact_kind(item: &MemoryItem) -> String {
    if item.stage == MemoryStage::Candidate {
        "candidate_memory".to_string()
    } else {
        "memory_item".to_string()
    }
}

fn visible_status(item: &MemoryItem) -> VisibleMemoryStatus {
    if item.stage == MemoryStage::Candidate {
        return VisibleMemoryStatus::Candidate;
    }
    match item.status {
        MemoryStatus::Active => VisibleMemoryStatus::Current,
        MemoryStatus::Stale => VisibleMemoryStatus::Stale,
        MemoryStatus::Superseded => VisibleMemoryStatus::Superseded,
        MemoryStatus::Contested => VisibleMemoryStatus::Conflicted,
        MemoryStatus::Expired => VisibleMemoryStatus::Archived,
    }
}

fn freshness_label(item: &MemoryItem, status: VisibleMemoryStatus) -> String {
    match status {
        VisibleMemoryStatus::Current => {
            if item.last_verified_at.is_some() {
                "verified".to_string()
            } else {
                "current".to_string()
            }
        }
        VisibleMemoryStatus::Candidate => "candidate".to_string(),
        VisibleMemoryStatus::Stale => "stale".to_string(),
        VisibleMemoryStatus::Superseded => "superseded".to_string(),
        VisibleMemoryStatus::Conflicted => "conflicted".to_string(),
        VisibleMemoryStatus::Archived => "archived".to_string(),
    }
}

fn repair_state_label(item: &MemoryItem, status: VisibleMemoryStatus) -> String {
    match status {
        VisibleMemoryStatus::Current if item.stage == MemoryStage::Canonical => {
            "healthy".to_string()
        }
        VisibleMemoryStatus::Current => "needs_review".to_string(),
        VisibleMemoryStatus::Candidate => "needs_promotion".to_string(),
        VisibleMemoryStatus::Stale
        | VisibleMemoryStatus::Superseded
        | VisibleMemoryStatus::Conflicted
        | VisibleMemoryStatus::Archived => "needs_attention".to_string(),
    }
}

fn trust_reason(item: &MemoryItem, status: VisibleMemoryStatus) -> String {
    let origin = item.source_system.as_deref().unwrap_or("memd").to_string();
    let verified = if item.last_verified_at.is_some() {
        "verified"
    } else {
        "unverified"
    };
    format!("{origin} {status:?} {verified}")
}

fn is_inbox_item(item: &MemoryItem) -> bool {
    item.stage == MemoryStage::Candidate || item.status != MemoryStatus::Active
}

fn is_repair_item(item: &MemoryItem) -> bool {
    matches!(
        item.status,
        MemoryStatus::Stale
            | MemoryStatus::Superseded
            | MemoryStatus::Contested
            | MemoryStatus::Expired
    ) || item.stage == MemoryStage::Candidate
}

fn inbox_items(items: &[MemoryItem]) -> Vec<&MemoryItem> {
    items.iter().filter(|item| is_inbox_item(item)).collect()
}

fn repair_items(items: &[MemoryItem]) -> Vec<&MemoryItem> {
    items
        .iter()
        .filter(|item| {
            is_repair_item(item)
                || item.last_verified_at.is_none()
                || item.source_quality == Some(memd_schema::SourceQuality::Derived)
        })
        .collect()
}

fn workspace_title(record: &WorkspaceMemoryRecord) -> String {
    let project = record.project.as_deref().unwrap_or("shared");
    let namespace = record.namespace.as_deref().unwrap_or("default");
    let workspace = record.workspace.as_deref().unwrap_or("workspace");
    format!("{project} / {namespace} / {workspace}")
}

fn workspace_status(record: &WorkspaceMemoryRecord) -> VisibleMemoryStatus {
    if record.contested_count > 0 {
        VisibleMemoryStatus::Conflicted
    } else if record.candidate_count > 0 {
        VisibleMemoryStatus::Candidate
    } else {
        VisibleMemoryStatus::Current
    }
}

fn count_nodes_by_kind(map: &VisibleMemoryKnowledgeMap, kind: &str) -> usize {
    map.nodes
        .iter()
        .filter(|node| node.artifact_kind == kind)
        .count()
}

fn workspace_node_id(record: &WorkspaceMemoryRecord) -> Uuid {
    use std::hash::{Hash, Hasher};

    let title = workspace_title(record);
    let mut left = std::collections::hash_map::DefaultHasher::new();
    title.hash(&mut left);
    let mut right = std::collections::hash_map::DefaultHasher::new();
    "memd-workspace".hash(&mut right);
    title.hash(&mut right);
    let high = left.finish() as u128;
    let low = right.finish() as u128;
    Uuid::from_u128((high << 64) | low)
}

pub(crate) fn test_insert_visible_item(
    state: &AppState,
    content: &str,
    preferred: bool,
) -> anyhow::Result<MemoryItem> {
    let req = memd_schema::StoreMemoryRequest {
        content: content.to_string(),
        kind: memd_schema::MemoryKind::Decision,
        scope: memd_schema::MemoryScope::Project,
        project: Some("memd".to_string()),
        namespace: Some("core".to_string()),
        workspace: Some("team-alpha".to_string()),
        visibility: Some(MemoryVisibility::Workspace),
        belief_branch: None,
        source_agent: Some("codex".to_string()),
        source_system: Some("obsidian".to_string()),
        source_path: Some(format!("wiki/{}.md", content.replace(' ', "-"))),
        source_quality: Some(memd_schema::SourceQuality::Derived),
        confidence: Some(0.9),
        ttl_seconds: None,
        last_verified_at: Some(Utc::now()),
        supersedes: Vec::new(),
        tags: vec!["visible-memory".to_string()],
        status: Some(MemoryStatus::Active),
    };
    let (mut item, _) = state.store_item(req, MemoryStage::Canonical)?;
    item.preferred = preferred;
    item.updated_at = Utc::now();
    let canonical_key = canonical_key(&item);
    let redundancy_key = redundancy_key(&item);
    state.store.update(&item, &canonical_key, &redundancy_key)?;
    Ok(item)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_state() -> AppState {
        let path = std::env::temp_dir().join(format!("memd-visible-ui-{}.db", Uuid::new_v4()));
        AppState {
            store: crate::store::SqliteStore::open(&path).unwrap(),
        }
    }

    #[test]
    fn builds_visible_memory_snapshot_from_stored_state() {
        let state = test_state();
        let preferred = test_insert_visible_item(&state, "runtime spine", true).unwrap();
        let newer = test_insert_visible_item(&state, "awareness lane", false).unwrap();
        let candidate = test_insert_candidate_item(&state, "candidate note").unwrap();
        let stale = test_insert_stale_item(&state, "stale belief").unwrap();
        assert!(newer.updated_at >= preferred.updated_at);

        let snapshot = build_visible_memory_snapshot(&state).unwrap();
        assert_eq!(snapshot.home.focus_artifact.id, preferred.id);
        assert_eq!(snapshot.home.focus_artifact.body, "runtime spine");
        assert_eq!(
            snapshot.home.focus_artifact.status,
            VisibleMemoryStatus::Current
        );
        assert_eq!(snapshot.home.focus_artifact.repair_state, "healthy");
        assert_eq!(
            snapshot.home.focus_artifact.provenance.source_system,
            Some("obsidian".to_string())
        );
        assert_eq!(snapshot.home.inbox_count, 2);
        assert_eq!(snapshot.home.repair_count, 4);
        assert_eq!(snapshot.home.awareness_count, 1);
        assert!(
            snapshot
                .knowledge_map
                .nodes
                .iter()
                .any(|node| node.artifact_kind == "timeline_event")
        );
        assert!(
            snapshot
                .knowledge_map
                .nodes
                .iter()
                .any(|node| node.artifact_kind == "workspace_lane")
        );
        assert!(snapshot.knowledge_map.nodes.len() >= 5);
        assert!(snapshot.knowledge_map.edges.len() >= 4);
        assert_eq!(candidate.status, MemoryStatus::Active);
        assert_eq!(stale.status, MemoryStatus::Stale);
    }

    #[test]
    fn dashboard_html_contains_memory_home_sections() {
        let state = test_state();
        test_insert_visible_item(&state, "runtime spine", true).unwrap();
        test_insert_candidate_item(&state, "candidate note").unwrap();
        let snapshot = build_visible_memory_snapshot(&state).unwrap();

        let html = dashboard_html(&snapshot);
        assert!(html.contains("Memory Home"));
        assert!(html.contains("Knowledge Map"));
        assert!(html.contains("Truth"));
        assert!(html.contains("Obsidian bridge"));
        assert!(html.contains("Open in Obsidian"));
        assert!(html.contains("wiki/runtime-spine.md"));
        assert!(html.contains("timeline events"));
        assert!(html.contains("workspace lanes"));
        assert!(html.contains("Hive Board"));
        assert!(html.contains("/hive/board"));
        assert!(html.contains("/hive/roster"));
        assert!(html.contains("/hive/follow?session="));
        assert!(html.contains("data-hive-follow-session"));
        assert!(html.contains("data-hive-queen-action=\"auto-retire\""));
        assert!(html.contains("data-hive-queen-action=\"retire-focused\""));
        assert!(html.contains("data-hive-queen-action=\"deny-focused\""));
        assert!(html.contains("data-hive-queen-action=\"reroute-focused\""));
        assert!(html.contains("data-hive-queen-action=\"handoff-focused\""));
    }

    #[test]
    fn builds_visible_memory_artifact_detail_from_stored_state() {
        let state = test_state();
        let item = test_insert_visible_item(&state, "runtime spine", true).unwrap();
        let _related = test_insert_candidate_item(&state, "candidate note").unwrap();

        let detail = build_visible_memory_artifact_detail(&state, item.id).unwrap();

        assert_eq!(detail.artifact.id, item.id);
        assert_eq!(detail.artifact.title, "runtime-spine");
        assert!(detail.explain.is_some());
        assert!(detail.sources.sources.len() >= 1);
        assert_eq!(detail.workspaces.workspaces.len(), 1);
        assert_eq!(detail.sessions.sessions.len(), 0);
        assert_eq!(detail.tasks.tasks.len(), 0);
        assert_eq!(detail.claims.claims.len(), 0);
        assert!(
            detail
                .related_artifacts
                .iter()
                .any(|artifact| artifact.title == "candidate-note")
        );
        assert!(!detail.actions.is_empty());
    }

    #[test]
    fn visible_memory_action_response_builds_obsidian_metadata() {
        let state = test_state();
        let item = test_insert_visible_item(&state, "runtime spine", true).unwrap();

        let response = perform_visible_memory_action(
            &state,
            VisibleMemoryUiActionRequest {
                id: item.id,
                action: VisibleMemoryUiActionKind::OpenInObsidian,
            },
        )
        .unwrap();

        assert_eq!(response.artifact_id, item.id);
        assert_eq!(response.action, VisibleMemoryUiActionKind::OpenInObsidian);
        assert_eq!(response.outcome, "metadata");
        assert!(
            response
                .open_uri
                .as_deref()
                .is_some_and(|uri| uri.starts_with("obsidian://open?path="))
        );
        assert_eq!(
            response.source_path.as_deref(),
            Some("wiki/runtime-spine.md")
        );
        assert!(response.detail.is_some());
    }

    #[test]
    fn visible_memory_action_can_verify_and_mark_stale() {
        let state = test_state();
        let item = test_insert_stale_item(&state, "stale belief").unwrap();

        let verified = perform_visible_memory_action(
            &state,
            VisibleMemoryUiActionRequest {
                id: item.id,
                action: VisibleMemoryUiActionKind::VerifyCurrent,
            },
        )
        .unwrap();
        assert_eq!(verified.outcome, "verified");
        assert!(verified.detail.is_some());

        let marked_stale = perform_visible_memory_action(
            &state,
            VisibleMemoryUiActionRequest {
                id: item.id,
                action: VisibleMemoryUiActionKind::MarkStale,
            },
        )
        .unwrap();
        assert_eq!(marked_stale.outcome, "marked_stale");
        assert!(marked_stale.detail.is_some());
    }

    #[test]
    fn visible_memory_action_can_promote() {
        let state = test_state();
        let item = test_insert_candidate_item(&state, "candidate note").unwrap();

        let promoted = perform_visible_memory_action(
            &state,
            VisibleMemoryUiActionRequest {
                id: item.id,
                action: VisibleMemoryUiActionKind::Promote,
            },
        )
        .unwrap();

        assert_eq!(promoted.outcome, "promoted");
        assert!(promoted.detail.is_some());
    }

    fn test_insert_candidate_item(state: &AppState, content: &str) -> anyhow::Result<MemoryItem> {
        let mut item = state
            .store_item(
                memd_schema::StoreMemoryRequest {
                    content: content.to_string(),
                    kind: memd_schema::MemoryKind::Decision,
                    scope: memd_schema::MemoryScope::Project,
                    project: Some("memd".to_string()),
                    namespace: Some("core".to_string()),
                    workspace: Some("team-alpha".to_string()),
                    visibility: Some(MemoryVisibility::Workspace),
                    belief_branch: None,
                    source_agent: Some("codex".to_string()),
                    source_system: Some("obsidian".to_string()),
                    source_path: Some(format!("wiki/{}.md", content.replace(' ', "-"))),
                    source_quality: Some(memd_schema::SourceQuality::Derived),
                    confidence: Some(0.72),
                    ttl_seconds: None,
                    last_verified_at: None,
                    supersedes: Vec::new(),
                    tags: vec!["visible-memory".to_string()],
                    status: Some(MemoryStatus::Active),
                },
                MemoryStage::Candidate,
            )?
            .0;
        item.updated_at = Utc::now();
        let canonical_key = canonical_key(&item);
        let redundancy_key = redundancy_key(&item);
        state.store.update(&item, &canonical_key, &redundancy_key)?;
        Ok(item)
    }

    fn test_insert_stale_item(state: &AppState, content: &str) -> anyhow::Result<MemoryItem> {
        let mut item = state
            .store_item(
                memd_schema::StoreMemoryRequest {
                    content: content.to_string(),
                    kind: memd_schema::MemoryKind::Decision,
                    scope: memd_schema::MemoryScope::Project,
                    project: Some("memd".to_string()),
                    namespace: Some("core".to_string()),
                    workspace: Some("team-alpha".to_string()),
                    visibility: Some(MemoryVisibility::Workspace),
                    belief_branch: None,
                    source_agent: Some("codex".to_string()),
                    source_system: Some("obsidian".to_string()),
                    source_path: Some(format!("wiki/{}.md", content.replace(' ', "-"))),
                    source_quality: Some(memd_schema::SourceQuality::Derived),
                    confidence: Some(0.6),
                    ttl_seconds: None,
                    last_verified_at: None,
                    supersedes: Vec::new(),
                    tags: vec!["visible-memory".to_string()],
                    status: Some(MemoryStatus::Stale),
                },
                MemoryStage::Canonical,
            )?
            .0;
        item.updated_at = Utc::now();
        let canonical_key = canonical_key(&item);
        let redundancy_key = redundancy_key(&item);
        state.store.update(&item, &canonical_key, &redundancy_key)?;
        Ok(item)
    }
}

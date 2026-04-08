use memd_schema::{
    CandidateMemoryRequest, CompactContextResponse, CompactionDecision, CompactionOpenLoop,
    CompactionPacket, CompactionReference, CompactionSession, CompactionSpillBatch,
    CompactionSpillOptions, MemoryKind, MemoryScope, RetrievalIntent, RetrievalRoute,
    SourceQuality,
};
use std::cmp::Ordering;

const MAX_SESSION_CHARS: usize = 120;
const MAX_LIST_ITEMS: usize = 4;
const MAX_RECORDS: usize = 4;
const MAX_LINE_CHARS: usize = 240;

pub struct BuildCompactionPacketArgs {
    pub session: CompactionSession,
    pub goal: String,
    pub hard_constraints: Vec<String>,
    pub active_work: Vec<String>,
    pub decisions: Vec<CompactionDecision>,
    pub open_loops: Vec<CompactionOpenLoop>,
    pub exact_refs: Vec<CompactionReference>,
    pub next_actions: Vec<String>,
    pub do_not_drop: Vec<String>,
    pub memory: CompactContextResponse,
}

pub fn build_compaction_packet(args: BuildCompactionPacketArgs) -> CompactionPacket {
    CompactionPacket {
        session: args.session,
        goal: args.goal,
        hard_constraints: args.hard_constraints,
        active_work: args.active_work,
        decisions: args.decisions,
        open_loops: args.open_loops,
        exact_refs: args.exact_refs,
        next_actions: args.next_actions,
        do_not_drop: args.do_not_drop,
        memory: args.memory,
    }
}

pub fn compact_string(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn split_compaction_list(values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(|value| compact_string(value).trim().to_string())
        .filter(|value| !value.is_empty())
        .collect()
}

pub fn render_compaction_wire(packet: &CompactionPacket) -> String {
    let mut lines = Vec::new();
    lines.push("memd-compaction".to_string());

    let mut session = Vec::new();
    if let Some(project) = packet.session.project.as_deref() {
        session.push(format!("p={}", compact_string(project)));
    }
    if let Some(agent) = packet.session.agent.as_deref() {
        session.push(format!("a={}", compact_string(agent)));
    }
    session.push(format!("t={}", compact_string(&packet.session.task)));
    lines.push(render_budgeted_line(
        "s",
        session.join(" "),
        MAX_SESSION_CHARS,
    ));

    lines.push(render_budgeted_line(
        "g",
        compact_string(&packet.goal),
        MAX_LINE_CHARS,
    ));
    if !packet.hard_constraints.is_empty() {
        lines.push(render_budgeted_line(
            "hc",
            join_budgeted(&packet.hard_constraints, " || ", MAX_LIST_ITEMS),
            MAX_LINE_CHARS,
        ));
    }
    if !packet.active_work.is_empty() {
        lines.push(render_budgeted_line(
            "aw",
            join_budgeted(&packet.active_work, " || ", MAX_LIST_ITEMS),
            MAX_LINE_CHARS,
        ));
    }
    for decision in packet.decisions.iter().take(MAX_LIST_ITEMS) {
        lines.push(render_budgeted_line(
            "d",
            format!(
                "{} {}",
                compact_string(&decision.id),
                compact_string(&decision.text)
            ),
            MAX_LINE_CHARS,
        ));
    }
    if packet.decisions.len() > MAX_LIST_ITEMS {
        lines.push(format!(
            "d ... +{}",
            packet.decisions.len() - MAX_LIST_ITEMS
        ));
    }
    for open_loop in packet.open_loops.iter().take(MAX_LIST_ITEMS) {
        lines.push(render_budgeted_line(
            "l",
            format!(
                "{}[{}] {}",
                compact_string(&open_loop.id),
                compact_string(&open_loop.status),
                compact_string(&open_loop.text)
            ),
            MAX_LINE_CHARS,
        ));
    }
    if packet.open_loops.len() > MAX_LIST_ITEMS {
        lines.push(format!(
            "l ... +{}",
            packet.open_loops.len() - MAX_LIST_ITEMS
        ));
    }
    for reference in packet.exact_refs.iter().take(MAX_LIST_ITEMS) {
        lines.push(render_budgeted_line(
            "x",
            format!(
                "{} {}",
                compact_string(&reference.kind),
                compact_string(&reference.value)
            ),
            MAX_LINE_CHARS,
        ));
    }
    if packet.exact_refs.len() > MAX_LIST_ITEMS {
        lines.push(format!(
            "x ... +{}",
            packet.exact_refs.len() - MAX_LIST_ITEMS
        ));
    }
    if !packet.next_actions.is_empty() {
        for action in packet.next_actions.iter().take(MAX_LIST_ITEMS) {
            lines.push(render_budgeted_line(
                "n",
                compact_string(action),
                MAX_LINE_CHARS,
            ));
        }
        if packet.next_actions.len() > MAX_LIST_ITEMS {
            lines.push(format!(
                "n ... +{}",
                packet.next_actions.len() - MAX_LIST_ITEMS
            ));
        }
    }
    if !packet.do_not_drop.is_empty() {
        lines.push(render_budgeted_line(
            "k",
            join_budgeted(&packet.do_not_drop, " ", MAX_LIST_ITEMS),
            MAX_LINE_CHARS,
        ));
    }
    if !packet.memory.records.is_empty() {
        lines.push(render_budgeted_line(
            "r",
            enum_label_route(packet.memory.route).to_string(),
            MAX_SESSION_CHARS,
        ));
        lines.push(render_budgeted_line(
            "i",
            enum_label_intent(packet.memory.intent).to_string(),
            MAX_SESSION_CHARS,
        ));
        lines.push(render_budgeted_line(
            "o",
            packet
                .memory
                .retrieval_order
                .iter()
                .map(|scope| format!("{scope:?}").to_lowercase())
                .collect::<Vec<_>>()
                .join(" "),
            MAX_SESSION_CHARS,
        ));
        for record in packet.memory.records.iter().take(MAX_RECORDS) {
            lines.push(render_budgeted_line(
                "m",
                compact_string(&record.record),
                MAX_LINE_CHARS,
            ));
        }
        if packet.memory.records.len() > MAX_RECORDS {
            lines.push(format!(
                "m ... +{}",
                packet.memory.records.len() - MAX_RECORDS
            ));
        }
    }

    lines.join("\n")
}

pub fn derive_compaction_spill(packet: &CompactionPacket) -> CompactionSpillBatch {
    derive_compaction_spill_with_options(
        packet,
        CompactionSpillOptions {
            include_transient_state: false,
        },
    )
}

pub fn derive_compaction_spill_with_options(
    packet: &CompactionPacket,
    options: CompactionSpillOptions,
) -> CompactionSpillBatch {
    let mut items = Vec::new();
    let mut dropped = Vec::new();
    let project_scope = packet
        .session
        .project
        .as_ref()
        .map(|_| MemoryScope::Project)
        .unwrap_or(MemoryScope::Global);
    let working_scope = packet
        .session
        .project
        .as_ref()
        .map(|_| MemoryScope::Synced)
        .unwrap_or(MemoryScope::Local);
    let source_agent = packet.session.agent.clone();

    if options.include_transient_state {
        push_session_item(
            &mut items,
            packet,
            SessionItemConfig {
                label: "session",
                kind: MemoryKind::Status,
                scope: working_scope,
                source_agent: source_agent.clone(),
                confidence: 0.55,
                ttl_seconds: Some(3 * 24 * 60 * 60),
                content: format!("session {}", compact_string(&packet.session.task)),
                tags: Vec::new(),
            },
        );

        push_session_item(
            &mut items,
            packet,
            SessionItemConfig {
                label: "goal",
                kind: MemoryKind::Status,
                scope: working_scope,
                source_agent: source_agent.clone(),
                confidence: 0.65,
                ttl_seconds: Some(7 * 24 * 60 * 60),
                content: format!("goal {}", compact_string(&packet.goal)),
                tags: Vec::new(),
            },
        );
    } else {
        dropped.push("session".to_string());
        dropped.push("goal".to_string());
    }

    for constraint in packet.hard_constraints.iter().take(MAX_LIST_ITEMS) {
        push_session_item(
            &mut items,
            packet,
            SessionItemConfig {
                label: "constraint",
                kind: MemoryKind::Constraint,
                scope: project_scope,
                source_agent: source_agent.clone(),
                confidence: 0.88,
                ttl_seconds: None,
                content: compact_string(constraint),
                tags: vec!["constraint".to_string()],
            },
        );
    }

    if options.include_transient_state {
        for work in packet.active_work.iter().take(MAX_LIST_ITEMS) {
            push_session_item(
                &mut items,
                packet,
                SessionItemConfig {
                    label: "active_work",
                    kind: MemoryKind::Status,
                    scope: working_scope,
                    source_agent: source_agent.clone(),
                    confidence: 0.4,
                    ttl_seconds: Some(3 * 24 * 60 * 60),
                    content: compact_string(work),
                    tags: Vec::new(),
                },
            );
        }
    } else if !packet.active_work.is_empty() {
        dropped.push("active_work".to_string());
    }

    for decision in packet.decisions.iter().take(MAX_LIST_ITEMS) {
        push_session_item(
            &mut items,
            packet,
            SessionItemConfig {
                label: "decision",
                kind: MemoryKind::Decision,
                scope: project_scope,
                source_agent: source_agent.clone(),
                confidence: 0.92,
                ttl_seconds: None,
                content: format!(
                    "{} {}",
                    compact_string(&decision.id),
                    compact_string(&decision.text)
                ),
                tags: Vec::new(),
            },
        );
    }

    for open_loop in packet.open_loops.iter().take(MAX_LIST_ITEMS) {
        push_session_item(
            &mut items,
            packet,
            SessionItemConfig {
                label: "open_loop",
                kind: MemoryKind::Status,
                scope: working_scope,
                source_agent: source_agent.clone(),
                confidence: 0.75,
                ttl_seconds: Some(14 * 24 * 60 * 60),
                content: format!(
                    "{}[{}] {}",
                    compact_string(&open_loop.id),
                    compact_string(&open_loop.status),
                    compact_string(&open_loop.text)
                ),
                tags: Vec::new(),
            },
        );
    }

    for reference in packet.exact_refs.iter().take(MAX_LIST_ITEMS) {
        let kind = match reference.kind.trim().to_ascii_lowercase().as_str() {
            "file" => MemoryKind::Topology,
            "command" => MemoryKind::Pattern,
            "host" | "hostname" | "ip" | "id" => MemoryKind::Fact,
            _ => MemoryKind::Fact,
        };
        push_session_item(
            &mut items,
            packet,
            SessionItemConfig {
                label: "anchor",
                kind,
                scope: project_scope,
                source_agent: source_agent.clone(),
                confidence: 0.86,
                ttl_seconds: None,
                content: format!(
                    "{} {}",
                    compact_string(&reference.kind),
                    compact_string(&reference.value)
                ),
                tags: vec!["anchor".to_string()],
            },
        );
        if let Some(last) = items.last_mut()
            && reference.kind.trim().eq_ignore_ascii_case("file")
        {
            last.source_path = Some(compact_string(&reference.value));
        }
    }

    if options.include_transient_state {
        for action in packet.next_actions.iter().take(MAX_LIST_ITEMS) {
            push_session_item(
                &mut items,
                packet,
                SessionItemConfig {
                    label: "next_action",
                    kind: MemoryKind::Status,
                    scope: working_scope,
                    source_agent: source_agent.clone(),
                    confidence: 0.35,
                    ttl_seconds: Some(3 * 24 * 60 * 60),
                    content: compact_string(action),
                    tags: Vec::new(),
                },
            );
        }
    } else if !packet.next_actions.is_empty() {
        dropped.push("next_actions".to_string());
    }

    for keep in packet.do_not_drop.iter().take(MAX_LIST_ITEMS) {
        push_session_item(
            &mut items,
            packet,
            SessionItemConfig {
                label: "do_not_drop",
                kind: MemoryKind::Constraint,
                scope: project_scope,
                source_agent: source_agent.clone(),
                confidence: 0.9,
                ttl_seconds: None,
                content: compact_string(keep),
                tags: vec!["do_not_drop".to_string()],
            },
        );
    }

    items.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.content.cmp(&b.content))
    });

    CompactionSpillBatch { items, dropped }
}

struct SessionItemConfig {
    label: &'static str,
    kind: MemoryKind,
    scope: MemoryScope,
    source_agent: Option<String>,
    confidence: f32,
    ttl_seconds: Option<u64>,
    content: String,
    tags: Vec<String>,
}

fn push_session_item(
    items: &mut Vec<CandidateMemoryRequest>,
    packet: &CompactionPacket,
    config: SessionItemConfig,
) {
    let mut tags_out = vec!["compaction".to_string(), config.label.to_string()];
    let mut tags = config.tags;
    tags_out.append(&mut tags);
    let project = packet.session.project.clone();

    items.push(CandidateMemoryRequest {
        content: config.content,
        kind: config.kind,
        scope: config.scope,
        project,
        namespace: Some("compaction".to_string()),
        workspace: None,
        visibility: None,
        belief_branch: None,
        source_agent: config.source_agent,
        source_system: Some("memd".to_string()),
        source_path: None,
        source_quality: Some(SourceQuality::Derived),
        confidence: Some(config.confidence),
        ttl_seconds: config.ttl_seconds,
        last_verified_at: None,
        supersedes: Vec::new(),
        tags: tags_out,
    });
}

fn render_budgeted_line(prefix: &str, value: String, max_chars: usize) -> String {
    let mut compacted = compact_string(&value);
    if compacted.chars().count() > max_chars {
        compacted = truncate_with_marker(&compacted, max_chars);
    }
    format!("{prefix} {compacted}")
}

fn join_budgeted(values: &[String], separator: &str, limit: usize) -> String {
    let mut items = split_compaction_list(values);
    let total = items.len();
    items.truncate(limit);
    let mut joined = items.join(separator);
    if total > limit {
        if !joined.is_empty() {
            joined.push_str(separator);
        }
        joined.push_str(&format!("... +{}", total - limit));
    }
    joined
}

fn truncate_with_marker(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    if max_chars <= 3 {
        return "...".chars().take(max_chars).collect();
    }

    let mut out = String::with_capacity(max_chars);
    for ch in value.chars().take(max_chars - 3) {
        out.push(ch);
    }
    out.push_str("...");
    out
}

fn enum_label_route(route: RetrievalRoute) -> &'static str {
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

fn enum_label_intent(intent: RetrievalIntent) -> &'static str {
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
#[cfg(test)]
mod tests {
    use super::*;
    use memd_schema::{CompactMemoryRecord, MemoryKind, MemoryScope};
    use uuid::Uuid;

    #[test]
    fn builds_packet_without_losing_fields() {
        let packet = build_compaction_packet(BuildCompactionPacketArgs {
            session: CompactionSession {
                project: Some("proj".into()),
                agent: Some("codex".into()),
                task: "task".into(),
            },
            goal: "goal".to_string(),
            hard_constraints: vec!["constraint".into()],
            active_work: vec!["work".into()],
            decisions: vec![CompactionDecision {
                id: "d1".into(),
                text: "keep".into(),
            }],
            open_loops: vec![CompactionOpenLoop {
                id: "l1".into(),
                text: "check".into(),
                status: "open".into(),
            }],
            exact_refs: vec![CompactionReference {
                kind: "file".into(),
                value: "/tmp/file".into(),
            }],
            next_actions: vec!["next".into()],
            do_not_drop: vec!["drop".into()],
            memory: CompactContextResponse {
                route: RetrievalRoute::ProjectFirst,
                intent: RetrievalIntent::General,
                retrieval_order: vec![MemoryScope::Project],
                records: vec![CompactMemoryRecord {
                    id: Uuid::nil(),
                    record: "id=1 | c=test".into(),
                }],
            },
        });

        assert_eq!(packet.goal, "goal");
        assert_eq!(packet.session.agent.as_deref(), Some("codex"));
        assert_eq!(packet.memory.records.len(), 1);
    }

    #[test]
    fn compaction_helpers_preserve_anchors_while_normalizing_whitespace() {
        assert_eq!(
            compact_string("  cargo   check   --workspace  "),
            "cargo check --workspace"
        );
        assert_eq!(
            compact_string("/tmp/My File.md  &&  ./run.sh"),
            "/tmp/My File.md && ./run.sh"
        );

        let values = vec![
            "  keep   this  ".to_string(),
            "".to_string(),
            "   ".to_string(),
            "\nnext\nstep\n".to_string(),
        ];

        assert_eq!(
            split_compaction_list(&values),
            vec!["keep this".to_string(), "next step".to_string()]
        );
    }

    #[test]
    fn compaction_packet_roundtrips_through_json_without_losing_state() {
        let packet = build_compaction_packet(BuildCompactionPacketArgs {
            session: CompactionSession {
                project: Some("memd".into()),
                agent: Some("codex".into()),
                task: "inspect compaction".into(),
            },
            goal: "preserve context".to_string(),
            hard_constraints: vec![
                "do not lose exact refs".into(),
                "do not flatten open loops".into(),
            ],
            active_work: vec!["build the packet".into()],
            decisions: vec![CompactionDecision {
                id: "decision-1".into(),
                text: "keep the packet structured".into(),
            }],
            open_loops: vec![CompactionOpenLoop {
                id: "loop-1".into(),
                text: "should compaction preserve command text verbatim?".into(),
                status: "open".into(),
            }],
            exact_refs: vec![
                CompactionReference {
                    kind: "file".into(),
                    value: "crates/memd-core/src/lib.rs".into(),
                },
                CompactionReference {
                    kind: "command".into(),
                    value: "cargo check".into(),
                },
            ],
            next_actions: vec!["serialize the packet".into()],
            do_not_drop: vec!["do not drop anchors".into()],
            memory: CompactContextResponse {
                route: RetrievalRoute::All,
                intent: RetrievalIntent::General,
                retrieval_order: vec![MemoryScope::Local, MemoryScope::Project],
                records: vec![
                    CompactMemoryRecord {
                        id: Uuid::nil(),
                        record: "id=... | stage=canonical | scope=project | kind=fact | status=active | project=memd | c=keep anchors".into(),
                    },
                    CompactMemoryRecord {
                        id: Uuid::new_v4(),
                        record: "id=... | stage=candidate | scope=local | kind=decision | status=active | c=preserve open loops".into(),
                    },
                ],
            },
        });

        let encoded = serde_json::to_string(&packet).expect("serialize packet");
        let decoded: CompactionPacket = serde_json::from_str(&encoded).expect("deserialize packet");

        assert_eq!(decoded.session.project.as_deref(), Some("memd"));
        assert_eq!(decoded.open_loops[0].text, packet.open_loops[0].text);
        assert_eq!(decoded.exact_refs[0].value, packet.exact_refs[0].value);
        assert_eq!(decoded.memory.records.len(), 2);
        assert!(
            decoded
                .exact_refs
                .iter()
                .any(|reference| reference.value.contains("cargo check"))
        );
    }

    #[test]
    fn wire_format_is_smaller_than_json_and_keeps_anchors() {
        let packet = build_compaction_packet(BuildCompactionPacketArgs {
            session: CompactionSession {
                project: Some("memd".into()),
                agent: Some("codex".into()),
                task: "build memory manager".into(),
            },
            goal: "Preserve memory without token waste".to_string(),
            hard_constraints: vec![
                "compact retrieval only".into(),
                "no transcript dumps".into(),
                "cross-project reuse must stay scoped".into(),
            ],
            active_work: vec!["verification worker scans stale canonical items".into()],
            decisions: vec![CompactionDecision {
                id: "decision-1".into(),
                text: "keep the packet structured".into(),
            }],
            open_loops: vec![CompactionOpenLoop {
                id: "loop-1".into(),
                text: "Should compaction preserve command text verbatim?".into(),
                status: "open".into(),
            }],
            exact_refs: vec![
                CompactionReference {
                    kind: "file".into(),
                    value: "crates/memd-server/src/main.rs".into(),
                },
                CompactionReference {
                    kind: "command".into(),
                    value: "cargo check".into(),
                },
            ],
            next_actions: vec!["Define the promotion boundary for compaction output".into()],
            do_not_drop: vec!["scope".into(), "exact refs".into(), "open loops".into()],
            memory: CompactContextResponse {
                route: RetrievalRoute::All,
                intent: RetrievalIntent::General,
                retrieval_order: vec![
                    MemoryScope::Local,
                    MemoryScope::Synced,
                    MemoryScope::Project,
                    MemoryScope::Global,
                ],
                records: vec![CompactMemoryRecord {
                    id: Uuid::nil(),
                    record: "id=... | stage=canonical | scope=project | kind=fact | status=active | c=keep anchors".into(),
                }],
            },
        });

        let json = serde_json::to_string(&packet).expect("serialize json");
        let wire = render_compaction_wire(&packet);

        assert!(wire.len() < json.len());
        assert!(wire.contains("crates/memd-server/src/main.rs"));
        assert!(wire.contains("cargo check"));
        assert!(wire.contains("Should compaction preserve command text verbatim?"));
        assert!(!wire.contains('{'));
        assert!(!wire.contains('\"'));
    }

    #[test]
    fn wire_format_truncates_with_explicit_markers() {
        let packet = build_compaction_packet(BuildCompactionPacketArgs {
            session: CompactionSession {
                project: Some("memd".into()),
                agent: Some("codex".into()),
                task: "build memory manager with a very long task description that should still preserve the first anchor and then stop".into(),
            },
            goal: "goal text that is intentionally very long so that it has to be truncated in the wire output while keeping the start intact".to_string(),
            hard_constraints: (0..10)
                .map(|i| format!("hard constraint number {i} with extra filler text to force truncation"))
                .collect(),
            active_work: (0..10)
                .map(|i| format!("active work item {i} with extra filler text to force truncation"))
                .collect(),
            decisions: (0..10)
                .map(|i| CompactionDecision {
                    id: format!("decision-{i}"),
                    text: format!("decision text {i} with extra filler text to force truncation"),
                })
                .collect(),
            open_loops: (0..10)
                .map(|i| CompactionOpenLoop {
                    id: format!("loop-{i}"),
                    text: format!("open loop text {i} with extra filler text to force truncation"),
                    status: "open".into(),
                })
                .collect(),
            exact_refs: (0..10)
                .map(|i| CompactionReference {
                    kind: "file".into(),
                    value: format!("/tmp/example-{i}.md"),
                })
                .collect(),
            next_actions: (0..10)
                .map(|i| format!("next action {i} with extra filler text to force truncation"))
                .collect(),
            do_not_drop: vec![
                "scope".into(),
                "project".into(),
                "exact refs".into(),
                "open loops".into(),
                "hard constraints".into(),
                "more items to force truncation".into(),
            ],
            memory: CompactContextResponse {
                route: RetrievalRoute::All,
                intent: RetrievalIntent::General,
                retrieval_order: vec![
                    MemoryScope::Local,
                    MemoryScope::Synced,
                    MemoryScope::Project,
                    MemoryScope::Global,
                ],
                records: (0..10)
                    .map(|i| CompactMemoryRecord {
                        id: Uuid::new_v4(),
                        record: format!(
                            "id={i} | stage=canonical | scope=project | kind=fact | status=active | c=memory record {i} with extra filler text to force truncation"
                        ),
                    })
                    .collect(),
            },
        });

        let wire = render_compaction_wire(&packet);

        assert!(wire.contains("memd-compaction"));
        assert!(wire.contains("p=memd"));
        assert!(wire.contains("a=codex"));
        assert!(wire.contains("d decision-0"));
        assert!(wire.contains("l loop-0[open]"));
        assert!(wire.contains("x file /tmp/example-0.md"));
        assert!(wire.contains("m id=0 | stage=canonical"));
        assert!(wire.contains("... +"));
        assert!(wire.lines().count() <= 40);
    }

    #[test]
    fn spill_batch_extracts_durable_state_with_scopes_and_tags() {
        let packet = build_compaction_packet(BuildCompactionPacketArgs {
            session: CompactionSession {
                project: Some("memd".into()),
                agent: Some("codex".into()),
                task: "stabilize memory".into(),
            },
            goal: "keep both short-term and long-term recall".to_string(),
            hard_constraints: vec!["do not lose exact refs".into()],
            active_work: vec!["working on spill layer".into()],
            decisions: vec![CompactionDecision {
                id: "decision-1".into(),
                text: "use a structured spill".into(),
            }],
            open_loops: vec![CompactionOpenLoop {
                id: "loop-1".into(),
                text: "should active work be synced or project scoped?".into(),
                status: "open".into(),
            }],
            exact_refs: vec![
                CompactionReference {
                    kind: "file".into(),
                    value: "crates/memd-client/src/main.rs".into(),
                },
                CompactionReference {
                    kind: "command".into(),
                    value: "cargo check".into(),
                },
            ],
            next_actions: vec!["make spill durable".into()],
            do_not_drop: vec!["open loops".into(), "anchors".into()],
            memory: CompactContextResponse {
                route: RetrievalRoute::All,
                intent: RetrievalIntent::General,
                retrieval_order: vec![
                    MemoryScope::Local,
                    MemoryScope::Synced,
                    MemoryScope::Project,
                    MemoryScope::Global,
                ],
                records: vec![],
            },
        });

        let spill = derive_compaction_spill(&packet);

        assert!(!spill.items.is_empty());
        assert!(
            spill
                .items
                .iter()
                .any(|item| item.kind == MemoryKind::Decision
                    && item.content.contains("structured spill"))
        );
        assert!(spill.items.iter().any(|item| {
            item.kind == MemoryKind::Topology
                && item.content.contains("crates/memd-client/src/main.rs")
        }));
        assert!(
            spill
                .items
                .iter()
                .all(|item| item.namespace.as_deref() == Some("compaction"))
        );
        assert!(
            spill
                .items
                .iter()
                .any(|item| item.tags.iter().any(|tag| tag == "anchor"))
        );
        assert!(spill.dropped.contains(&"session".to_string()));
        assert!(spill.dropped.contains(&"goal".to_string()));
        assert!(spill.dropped.contains(&"active_work".to_string()));
        assert!(spill.dropped.contains(&"next_actions".to_string()));
    }

    #[test]
    fn spill_batch_omits_transient_state_by_default() {
        let packet = build_compaction_packet(BuildCompactionPacketArgs {
            session: CompactionSession {
                project: Some("memd".into()),
                agent: Some("codex".into()),
                task: "stabilize memory".into(),
            },
            goal: "keep both short-term and long-term recall".to_string(),
            hard_constraints: vec!["do not lose exact refs".into()],
            active_work: vec!["working on spill layer".into()],
            decisions: vec![CompactionDecision {
                id: "decision-1".into(),
                text: "use a structured spill".into(),
            }],
            open_loops: vec![CompactionOpenLoop {
                id: "loop-1".into(),
                text: "should active work be synced or project scoped?".into(),
                status: "open".into(),
            }],
            exact_refs: vec![
                CompactionReference {
                    kind: "file".into(),
                    value: "crates/memd-client/src/main.rs".into(),
                },
                CompactionReference {
                    kind: "command".into(),
                    value: "cargo check".into(),
                },
            ],
            next_actions: vec!["make spill durable".into()],
            do_not_drop: vec!["open loops".into(), "anchors".into()],
            memory: CompactContextResponse {
                route: RetrievalRoute::All,
                intent: RetrievalIntent::General,
                retrieval_order: vec![
                    MemoryScope::Local,
                    MemoryScope::Synced,
                    MemoryScope::Project,
                    MemoryScope::Global,
                ],
                records: vec![],
            },
        });

        let spill = derive_compaction_spill(&packet);
        let contents = spill
            .items
            .iter()
            .map(|item| item.content.clone())
            .collect::<Vec<_>>();

        assert!(
            contents
                .iter()
                .any(|content| content == "do not lose exact refs")
        );
        assert!(
            !contents
                .iter()
                .any(|content| content.contains("working on spill layer"))
        );
        assert!(
            !contents
                .iter()
                .any(|content| content.contains("make spill durable"))
        );
        assert!(spill.dropped.contains(&"session".to_string()));
        assert!(spill.dropped.contains(&"goal".to_string()));
        assert!(spill.dropped.contains(&"active_work".to_string()));
        assert!(spill.dropped.contains(&"next_actions".to_string()));
    }

    #[test]
    fn file_anchors_carry_source_path_for_verification() {
        let packet = build_compaction_packet(BuildCompactionPacketArgs {
            session: CompactionSession {
                project: Some("memd".into()),
                agent: Some("codex".into()),
                task: "anchor file".into(),
            },
            goal: "keep source path".to_string(),
            hard_constraints: vec![],
            active_work: vec![],
            decisions: vec![],
            open_loops: vec![],
            exact_refs: vec![CompactionReference {
                kind: "file".into(),
                value: "crates/memd-client/src/main.rs".into(),
            }],
            next_actions: vec![],
            do_not_drop: vec![],
            memory: CompactContextResponse {
                route: RetrievalRoute::All,
                intent: RetrievalIntent::General,
                retrieval_order: vec![MemoryScope::Project],
                records: vec![],
            },
        });

        let spill = derive_compaction_spill(&packet);
        let anchor = spill
            .items
            .iter()
            .find(|item| item.kind == MemoryKind::Topology)
            .expect("anchor spill item");

        assert_eq!(
            anchor.source_path.as_deref(),
            Some("crates/memd-client/src/main.rs")
        );
    }
}

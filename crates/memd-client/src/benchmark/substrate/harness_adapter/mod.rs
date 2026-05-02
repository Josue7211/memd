//! C5 harness adapter contract.
//!
//! Each adapter wraps one real harness (claude-code, codex) and translates
//! a deterministic `Script` of write+read steps into calls against memd.
//! Production binaries shell out to `memd`; tests inject `InMemoryGateway`
//! so the C5 suite is exercisable without harness/memd binaries on PATH.
//!
//! Visibility rules enforced by the gateway:
//! * `Scope::Project` — visible to any harness sharing the same project key.
//! * `Scope::Local`   — visible only to the writer harness (per-harness sandbox).
//! * `Scope::Global`  — visible to every harness regardless of project.
//!
//! A misbehaving gateway can flip `leak_local=true`; the visibility
//! auditor in `cross_harness.rs` catches the resulting cross-harness leak.

pub(crate) mod claude_code;
pub(crate) mod codex;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) enum Scope {
    Project,
    Local,
    Global,
}

impl Scope {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Scope::Project => "project",
            Scope::Local => "local",
            Scope::Global => "global",
        }
    }

    pub(crate) fn from_str(s: &str) -> Option<Self> {
        match s {
            "project" => Some(Scope::Project),
            "local" => Some(Scope::Local),
            "global" => Some(Scope::Global),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Script {
    pub(crate) project: String,
    pub(crate) steps: Vec<ScriptStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub(crate) enum ScriptStep {
    Write {
        kind: String,
        content: String,
        scope: Scope,
        tag: String,
    },
    Read {
        query: String,
        scope: Scope,
        expect_tag: String,
    },
}

#[derive(Debug, Clone)]
pub(crate) struct WriteResult {
    pub(crate) id: String,
    pub(crate) tag: String,
    pub(crate) scope: Scope,
}

#[derive(Debug, Clone)]
pub(crate) struct ReadResult {
    pub(crate) query: String,
    pub(crate) requested_scope: Scope,
    pub(crate) hits: Vec<ReadHit>,
    pub(crate) expect_tag: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ReadHit {
    pub(crate) id: String,
    pub(crate) content: String,
    pub(crate) tag: String,
    pub(crate) source_harness: String,
    pub(crate) source_scope: Scope,
    pub(crate) source_project: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct HarnessRunOutcome {
    pub(crate) writes: Vec<WriteResult>,
    pub(crate) reads: Vec<ReadResult>,
}

/// Bridge between adapter and memd. Production = subprocess to `memd`
/// CLI; tests = `InMemoryGateway`.
pub(crate) trait MemdGateway {
    fn remember(
        &self,
        harness: &str,
        project: &str,
        kind: &str,
        content: &str,
        tag: &str,
        scope: Scope,
    ) -> std::io::Result<WriteResult>;

    fn lookup(
        &self,
        harness: &str,
        project: &str,
        query: &str,
        scope: Scope,
    ) -> std::io::Result<Vec<ReadHit>>;
}

/// Adapter for a single harness. Implementations live in `claude_code.rs`,
/// `codex.rs`. Detection uses the harness's own config file (per
/// `phase-c5-plan.md` §3 — `~/.claude/settings.json`, `~/.codex/hooks.json`).
pub(crate) trait HarnessAdapter {
    fn name(&self) -> &'static str;
    fn is_available(&self) -> bool;
    fn run_script(
        &self,
        script: &Script,
        gateway: &dyn MemdGateway,
    ) -> std::io::Result<HarnessRunOutcome>;
}

/// Default `run_script` body shared by both adapters: walk steps,
/// translate each into a gateway call, collect results.
pub(crate) fn drive_script_via_gateway(
    harness_name: &str,
    script: &Script,
    gateway: &dyn MemdGateway,
) -> std::io::Result<HarnessRunOutcome> {
    let mut outcome = HarnessRunOutcome::default();
    for step in &script.steps {
        match step {
            ScriptStep::Write {
                kind,
                content,
                scope,
                tag,
            } => {
                let r =
                    gateway.remember(harness_name, &script.project, kind, content, tag, *scope)?;
                outcome.writes.push(r);
            }
            ScriptStep::Read {
                query,
                scope,
                expect_tag,
            } => {
                let hits = gateway.lookup(harness_name, &script.project, query, *scope)?;
                outcome.reads.push(ReadResult {
                    query: query.clone(),
                    requested_scope: *scope,
                    hits,
                    expect_tag: expect_tag.clone(),
                });
            }
        }
    }
    Ok(outcome)
}

#[derive(Debug, Clone)]
struct StoredRecord {
    id: String,
    harness: String,
    project: String,
    kind: String,
    content: String,
    tag: String,
    scope: Scope,
}

/// Perfect-recall in-memory gateway used by tests + integration suite.
/// Enforces the documented scope visibility rules unless `leak_local`
/// is set, in which case `Local` writes leak across harnesses (the
/// fault-injection knob for Test 6).
#[derive(Default)]
pub(crate) struct InMemoryGateway {
    state: Mutex<GatewayState>,
}

#[derive(Default)]
struct GatewayState {
    records: Vec<StoredRecord>,
    counter: u64,
    /// Scoped per-harness counters used to render stable record ids.
    per_harness: HashMap<String, u64>,
    leak_local: bool,
}

impl InMemoryGateway {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn with_leak_local() -> Self {
        let g = Self::default();
        g.state.lock().unwrap().leak_local = true;
        g
    }
}

impl MemdGateway for InMemoryGateway {
    fn remember(
        &self,
        harness: &str,
        project: &str,
        kind: &str,
        content: &str,
        tag: &str,
        scope: Scope,
    ) -> std::io::Result<WriteResult> {
        let mut st = self.state.lock().unwrap();
        st.counter += 1;
        let n = st.counter;
        *st.per_harness.entry(harness.to_string()).or_insert(0) += 1;
        let id = format!("c5-{harness}-{n:04}");
        st.records.push(StoredRecord {
            id: id.clone(),
            harness: harness.to_string(),
            project: project.to_string(),
            kind: kind.to_string(),
            content: content.to_string(),
            tag: tag.to_string(),
            scope,
        });
        Ok(WriteResult {
            id,
            tag: tag.to_string(),
            scope,
        })
    }

    fn lookup(
        &self,
        harness: &str,
        project: &str,
        query: &str,
        scope: Scope,
    ) -> std::io::Result<Vec<ReadHit>> {
        let st = self.state.lock().unwrap();
        let leak_local = st.leak_local;
        let mut out = Vec::new();
        for r in &st.records {
            if !content_matches(&r.content, query) && !tag_matches(&r.tag, query) {
                continue;
            }
            // Scope filter: which writes can be returned given a lookup
            // requested at `scope`?
            //
            // - When the caller asks for `Project`, return writes that
            //   were stored with project visibility AND share the project.
            // - When the caller asks for `Local`, return only writes the
            //   *same* harness made (unless the gateway is misbehaving).
            // - When the caller asks for `Global`, return global writes
            //   from any harness.
            let visible = match (scope, r.scope) {
                (Scope::Project, Scope::Project) => r.project == project,
                (Scope::Local, Scope::Local) => r.harness == harness || leak_local,
                (Scope::Global, Scope::Global) => true,
                _ => false,
            };
            if !visible {
                continue;
            }
            out.push(ReadHit {
                id: r.id.clone(),
                content: r.content.clone(),
                tag: r.tag.clone(),
                source_harness: r.harness.clone(),
                source_scope: r.scope,
                source_project: Some(r.project.clone()),
            });
        }
        Ok(out)
    }
}

fn content_matches(content: &str, query: &str) -> bool {
    content.to_lowercase().contains(&query.to_lowercase())
}

fn tag_matches(tag: &str, query: &str) -> bool {
    tag.eq_ignore_ascii_case(query)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::benchmark::substrate::harness_adapter::claude_code::ClaudeCodeAdapter;
    use std::fs;
    use tempfile::tempdir;

    /// C5 Test 3 — `adapter_run_script_writes_and_reads_via_memd_cli`.
    /// In production the adapter shells out to the `memd` CLI; under
    /// test we inject `InMemoryGateway`. Either path obeys the same
    /// `MemdGateway` contract, so this test exercises the adapter's
    /// translation logic without needing the binary.
    #[test]
    fn adapter_run_script_writes_and_reads_via_memd_cli() {
        let dir = tempdir().unwrap();
        let settings = dir.path().join("settings.json");
        fs::write(&settings, "{}").unwrap();
        let adapter = ClaudeCodeAdapter::with_config_path(settings);
        let gateway = InMemoryGateway::new();

        let script = Script {
            project: "demo".into(),
            steps: vec![
                ScriptStep::Write {
                    kind: "fact".into(),
                    content: "alice lives in berlin".into(),
                    scope: Scope::Project,
                    tag: "alice-loc".into(),
                },
                ScriptStep::Read {
                    query: "alice".into(),
                    scope: Scope::Project,
                    expect_tag: "alice-loc".into(),
                },
            ],
        };

        let outcome = adapter.run_script(&script, &gateway).unwrap();
        assert_eq!(outcome.writes.len(), 1);
        assert_eq!(outcome.reads.len(), 1);
        let hits = &outcome.reads[0].hits;
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].tag, "alice-loc");
        assert_eq!(hits[0].source_harness, "claude_code");
        assert_eq!(hits[0].source_scope, Scope::Project);
    }

    #[test]
    fn in_memory_gateway_isolates_local_scope_per_harness() {
        let g = InMemoryGateway::new();
        g.remember("claude_code", "p", "fact", "secret", "secret", Scope::Local)
            .unwrap();
        let codex_view = g.lookup("codex", "p", "secret", Scope::Local).unwrap();
        assert!(
            codex_view.is_empty(),
            "local scope must isolate per harness"
        );
        let claude_view = g
            .lookup("claude_code", "p", "secret", Scope::Local)
            .unwrap();
        assert_eq!(claude_view.len(), 1);
    }

    #[test]
    fn in_memory_gateway_leak_mode_breaches_local_scope() {
        let g = InMemoryGateway::with_leak_local();
        g.remember("claude_code", "p", "fact", "secret", "secret", Scope::Local)
            .unwrap();
        let codex_view = g.lookup("codex", "p", "secret", Scope::Local).unwrap();
        assert_eq!(codex_view.len(), 1, "leak mode should breach local scope");
        assert_eq!(codex_view[0].source_harness, "claude_code");
    }

    #[test]
    fn in_memory_gateway_project_scope_crosses_harnesses() {
        let g = InMemoryGateway::new();
        g.remember(
            "claude_code",
            "p",
            "fact",
            "shared",
            "shared",
            Scope::Project,
        )
        .unwrap();
        let codex_view = g.lookup("codex", "p", "shared", Scope::Project).unwrap();
        assert_eq!(codex_view.len(), 1);
        let other_project = g
            .lookup("codex", "other", "shared", Scope::Project)
            .unwrap();
        assert!(
            other_project.is_empty(),
            "project scope must not leak across projects"
        );
    }
}

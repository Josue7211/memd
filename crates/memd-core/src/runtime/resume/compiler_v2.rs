//! V11/D11 dynamic per-turn compiler.

use crate::cost::ledger::CostTarget;
use crate::isolation::{ProjectScope, ScopedMemoryRecord, filter_project_records};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntentClass {
    Lookup,
    Recall,
    Create,
    Refine,
    Synthesize,
    General,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DepthDecision {
    pub immediate: usize,
    pub procedural: usize,
    pub background: usize,
}

impl DepthDecision {
    pub fn render(&self) -> String {
        format!(
            "immediate:{}, procedural:{}, background:{}",
            self.immediate, self.procedural, self.background
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompilerInput {
    pub session_id: String,
    pub turn_seq: u64,
    pub scope: ProjectScope,
    pub user_text: String,
    pub target_token_budget: usize,
    pub cost_target: Option<CostTarget>,
    pub records: Vec<ScopedMemoryRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompilerContextRow {
    pub compiler_context_id: String,
    pub session_id: String,
    pub turn_seq: u64,
    pub intent_class: IntentClass,
    pub target_token_budget: usize,
    pub actual_tokens: usize,
    pub depth_decision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompilerDecision {
    pub row: CompilerContextRow,
    pub selected: Vec<ScopedMemoryRecord>,
}

pub fn classify_intent(text: &str) -> IntentClass {
    let lower = text.to_ascii_lowercase();
    if lower.contains("summarize") || lower.contains("remind") || lower.contains("learned") {
        IntentClass::Recall
    } else if lower.contains("what") || lower.contains("lookup") || lower.contains("which") {
        IntentClass::Lookup
    } else if lower.contains("draft") || lower.contains("create") || lower.contains("write") {
        IntentClass::Create
    } else if lower.contains("revise") || lower.contains("refine") || lower.contains("fix") {
        IntentClass::Refine
    } else if lower.contains("compare") || lower.contains("synthesize") {
        IntentClass::Synthesize
    } else {
        IntentClass::General
    }
}

pub fn depth_for_intent(intent: IntentClass) -> DepthDecision {
    match intent {
        IntentClass::Lookup => DepthDecision {
            immediate: 3,
            procedural: 0,
            background: 0,
        },
        IntentClass::Recall => DepthDecision {
            immediate: 4,
            procedural: 2,
            background: 0,
        },
        IntentClass::Create | IntentClass::Refine => DepthDecision {
            immediate: 4,
            procedural: 3,
            background: 1,
        },
        IntentClass::Synthesize => DepthDecision {
            immediate: 4,
            procedural: 2,
            background: 2,
        },
        IntentClass::General => DepthDecision {
            immediate: 3,
            procedural: 1,
            background: 0,
        },
    }
}

pub fn compile_turn(input: CompilerInput) -> CompilerDecision {
    let intent = classify_intent(&input.user_text);
    let depth = depth_for_intent(intent);
    let budget = input
        .cost_target
        .as_ref()
        .map(|target| target.token_budget(input.target_token_budget))
        .unwrap_or(input.target_token_budget)
        .min(input.target_token_budget);

    let scoped = filter_project_records(&input.scope, &input.records);
    let mut immediate = scoped
        .iter()
        .filter(|record| record.kind != "procedural" && record.kind != "background")
        .cloned()
        .collect::<Vec<_>>();
    immediate.sort_by_key(|record| (!record.correction_active, record.id.clone()));

    let mut procedural = scoped
        .iter()
        .filter(|record| record.kind == "procedural")
        .cloned()
        .collect::<Vec<_>>();
    procedural.sort_by_key(|record| record.id.clone());

    let mut background = scoped
        .iter()
        .filter(|record| record.kind == "background")
        .cloned()
        .collect::<Vec<_>>();
    background.sort_by_key(|record| record.id.clone());

    let candidates = immediate
        .into_iter()
        .take(depth.immediate)
        .chain(procedural.into_iter().take(depth.procedural))
        .chain(background.into_iter().take(depth.background));

    let mut selected = Vec::new();
    let mut actual_tokens = 0usize;
    for record in candidates {
        let tokens = record.token_count.max(1);
        if actual_tokens + tokens > budget {
            continue;
        }
        actual_tokens += tokens;
        selected.push(record);
    }

    CompilerDecision {
        row: CompilerContextRow {
            compiler_context_id: format!("compiler-{}-{}", input.session_id, input.turn_seq),
            session_id: input.session_id,
            turn_seq: input.turn_seq,
            intent_class: intent,
            target_token_budget: budget,
            actual_tokens,
            depth_decision: depth.render(),
        },
        selected,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn records(scope: &ProjectScope) -> Vec<ScopedMemoryRecord> {
        vec![
            ScopedMemoryRecord::scoped("correction", scope, "correction", "cache is Redis")
                .active_correction("t4")
                .with_tokens(30),
            ScopedMemoryRecord::scoped("fact", scope, "fact", "primary storage PostgreSQL")
                .with_tokens(30),
            ScopedMemoryRecord::scoped("proc", scope, "procedural", "read schema first")
                .with_tokens(30),
        ]
    }

    #[test]
    fn recall_intent_uses_immediate_plus_procedural_depth() {
        let scope = ProjectScope::new("project-a", "workspace-1");
        let decision = compile_turn(CompilerInput {
            session_id: "s1".into(),
            turn_seq: 13,
            scope: scope.clone(),
            user_text: "Summarize what you learned".into(),
            target_token_budget: 1500,
            cost_target: None,
            records: records(&scope),
        });
        assert_eq!(decision.row.intent_class, IntentClass::Recall);
        assert_eq!(
            decision.row.depth_decision,
            "immediate:4, procedural:2, background:0"
        );
        assert_eq!(decision.selected.len(), 3);
        assert_eq!(decision.selected[0].id, "correction");
    }

    #[test]
    fn cost_target_caps_actual_budget() {
        let scope = ProjectScope::new("project-a", "workspace-1");
        let decision = compile_turn(CompilerInput {
            session_id: "s1".into(),
            turn_seq: 14,
            scope: scope.clone(),
            user_text: "What cache backend?".into(),
            target_token_budget: 4000,
            cost_target: Some(CostTarget {
                project_id: scope.project_id.clone(),
                per_turn_cents: 0.5,
            }),
            records: records(&scope),
        });
        assert_eq!(decision.row.target_token_budget, 1500);
        assert!(decision.row.actual_tokens <= 1500);
    }
}

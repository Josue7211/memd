use serde::Serialize;

/// Selective expansion stages for lookup synthesis. The ladder starts at a
/// small exact hit set (needle), then broadens only when the query asks for an
/// executive packet or raw proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ExpansionStage {
    Needle,
    Thread,
    Ceo,
    Forensics,
}

impl ExpansionStage {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            ExpansionStage::Needle => "needle",
            ExpansionStage::Thread => "thread",
            ExpansionStage::Ceo => "ceo",
            ExpansionStage::Forensics => "forensics",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ExpansionPlan {
    pub(crate) stages: Vec<ExpansionStage>,
    pub(crate) ceo_mode: bool,
    pub(crate) forensics: bool,
    pub(crate) rationale: &'static str,
}

impl ExpansionPlan {
    pub(crate) fn normal() -> Self {
        Self {
            stages: vec![ExpansionStage::Needle],
            ceo_mode: false,
            forensics: false,
            rationale: "normal lookup: needle only",
        }
    }

    pub(crate) fn ceo() -> Self {
        Self {
            stages: vec![
                ExpansionStage::Needle,
                ExpansionStage::Thread,
                ExpansionStage::Ceo,
            ],
            ceo_mode: true,
            forensics: false,
            rationale: "CEO request: needle -> thread -> ceo",
        }
    }

    pub(crate) fn ceo_forensics() -> Self {
        Self {
            stages: vec![
                ExpansionStage::Needle,
                ExpansionStage::Thread,
                ExpansionStage::Ceo,
                ExpansionStage::Forensics,
            ],
            ceo_mode: true,
            forensics: true,
            rationale: "CEO request with raw proof/debug/conflict need: include forensics",
        }
    }

    pub(crate) fn stage_names(&self) -> Vec<&'static str> {
        self.stages.iter().map(|stage| stage.as_str()).collect()
    }
}

pub(crate) fn plan_for_lookup(query: &str) -> ExpansionPlan {
    let normalized = query.to_ascii_lowercase();
    let ceo = explicit_ceo_request(&normalized) || inferred_ceo_request(&normalized);
    let forensics = raw_history_or_proof_requested(&normalized);

    match (ceo, forensics) {
        (true, true) => ExpansionPlan::ceo_forensics(),
        (true, false) => ExpansionPlan::ceo(),
        (false, _) => ExpansionPlan::normal(),
    }
}

fn explicit_ceo_request(query: &str) -> bool {
    query.contains("ceo")
        || query.contains("executive")
        || query.contains("synthesis packet")
        || query.contains("decision packet")
}

fn inferred_ceo_request(query: &str) -> bool {
    let decision_terms = [
        "what should i do",
        "what should we do",
        "recommendation",
        "recommend",
        "priority",
        "prioritize",
        "strategy",
        "bottleneck",
        "next moves",
        "next steps",
        "tradeoff",
        "decision",
    ];
    decision_terms.iter().any(|term| query.contains(term))
}

fn raw_history_or_proof_requested(query: &str) -> bool {
    let proof_terms = [
        "raw history",
        "history",
        "conflict",
        "conflicts",
        "debug",
        "forensics",
        "audit trail",
        "proof",
        "evidence",
        "show your work",
    ];
    proof_terms.iter().any(|term| query.contains(term))
}

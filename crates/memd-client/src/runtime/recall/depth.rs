use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum RecallDepth {
    Wake,
    Lookup,
    Resume,
}

impl RecallDepth {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            RecallDepth::Wake => "wake",
            RecallDepth::Lookup => "lookup",
            RecallDepth::Resume => "resume",
        }
    }
}

impl std::fmt::Display for RecallDepth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

pub(crate) fn depth_flag_enabled() -> bool {
    flag_on("MEMD_E4_DEPTH_FLAG")
}

pub(crate) fn escalation_hint_enabled() -> bool {
    flag_on("MEMD_E4_ESCALATION_HINT")
}

fn flag_on(var: &str) -> bool {
    let raw = std::env::var(var).unwrap_or_default();
    let normalized = raw.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return true;
    }
    matches!(normalized.as_str(), "1" | "true" | "on" | "yes")
}

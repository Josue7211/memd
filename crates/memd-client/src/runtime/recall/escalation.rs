use regex::RegexSet;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelectiveExpansionMode {
    Normal,
    CeoExplicit,
    CeoInferred,
}

impl SelectiveExpansionMode {
    pub(crate) fn is_ceo(self) -> bool {
        !matches!(self, SelectiveExpansionMode::Normal)
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            SelectiveExpansionMode::Normal => "normal",
            SelectiveExpansionMode::CeoExplicit => "ceo_explicit",
            SelectiveExpansionMode::CeoInferred => "ceo_inferred",
        }
    }
}

/// Specifier patterns that, on a zero-hit `--depth lookup`, mean the user
/// is referring to a specific past artifact. The dispatcher prints a
/// hint suggesting `--depth resume`. Patterns are case-insensitive and
/// matched against the post-sanitization query (server-side sanitizer
/// runs first; `escalation::detect` operates on what the user typed).
///
/// Source: `docs/contracts/recall-depth.md` §"Specifier regex set".
static SPECIFIER_PATTERNS: LazyLock<RegexSet> = LazyLock::new(|| {
    RegexSet::new([
        r"(?i)\b(the|my|our)\b\s+.+\s+\b(task|plan|issue|decision|bug|feature)\b",
        r"(?i)\bwhat\s+(was|were)\s+(i|we)\b\s+(doing|working\s+on|trying)\b",
        r"(?i)\bwhere\s+did\s+(i|we)\s+leave\s+off\b",
    ])
    .expect("compile escalation specifier regex set")
});

pub(crate) fn detect(query: &str) -> bool {
    SPECIFIER_PATTERNS.is_match(query)
}

pub(crate) fn hint_line(query: &str) -> String {
    format!(
        "hint: zero results at lookup depth. Escalate with `memd lookup --query \"{}\" --depth resume` (cost ~6k tokens).",
        query.replace('"', "\\\"")
    )
}

/// Explicit CEO-mode triggers. These are user phrases that ask the agent to
/// expand above ordinary lookup into strategic synthesis while staying compact.
static CEO_EXPLICIT_PATTERNS: LazyLock<RegexSet> = LazyLock::new(|| {
    RegexSet::new([
        r"(?i)\bceo\s*mode\b",
        r"(?i)\b10\s*star\b",
        r"(?i)\b25\s*/\s*5\s*star\b",
        r"(?i)\b25\s*out\s*of\s*5\b",
        r"(?i)\bmake\s+this\s+(?:great|excellent|exceptional|world[- ]class)\b",
        r"(?i)\bthink\s+bigger\b",
    ])
    .expect("compile CEO explicit regex set")
});

/// Inferred CEO-mode triggers. These are high-level strategy / quality asks
/// where the user likely wants synthesis, bottlenecks, moves, and proof rather
/// than raw memory matches.
static CEO_INFERRED_PATTERNS: LazyLock<RegexSet> = LazyLock::new(|| {
    RegexSet::new([
        r"(?i)\bhow\s+(?:can|do)\s+we\s+make\s+.+\b(?:better|great|excellent|stronger)\b",
        r"(?i)\bwhat\s+are\s+we\s+missing\b",
        r"(?i)\bwhat\s+should\s+(?:happen|we\s+do|the\s+agent\s+do)\b",
        r"(?i)\bwhy\s+is\s+this\s+(?:failing|not\s+working|weak)\b",
        r"(?i)\bwhat\s+is\s+the\s+(?:bottleneck|leverage|strategy|best\s+move)\b",
        r"(?i)\bwhat\s+would\s+make\s+this\s+.+\b(?:better|great|excellent|stronger)\b",
    ])
    .expect("compile CEO inferred regex set")
});

pub(crate) fn selective_expansion_mode(query: &str) -> SelectiveExpansionMode {
    if CEO_EXPLICIT_PATTERNS.is_match(query) {
        SelectiveExpansionMode::CeoExplicit
    } else if CEO_INFERRED_PATTERNS.is_match(query) {
        SelectiveExpansionMode::CeoInferred
    } else {
        SelectiveExpansionMode::Normal
    }
}

pub(crate) fn ceo_mode_guidance_markdown(mode: SelectiveExpansionMode) -> Option<String> {
    if !mode.is_ceo() {
        return None;
    }
    Some(format!(
        "## Selective expansion: CEO mode

- trigger: {}
- ladder: needle -> thread -> CEO -> forensics only if needed
- answer shape: Read, Prize, Bottleneck, Moves, Recommendation, Proof
- memory rule: use approved decisions/preferences/outcomes; avoid raw chat noise

",
        mode.label()
    ))
}

pub(crate) fn ceo_mode_hint_line(query: &str, mode: SelectiveExpansionMode) -> Option<String> {
    if !mode.is_ceo() {
        return None;
    }
    Some(format!(
        "hint: selective expansion CEO mode ({}) active. If 1-3 records are not enough, escalate with `memd lookup --query \"{}\" --depth resume` for thread reconstruction.",
        mode.label(),
        query.replace('"', "\\\"")
    ))
}

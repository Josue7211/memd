use regex::RegexSet;
use std::sync::LazyLock;

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

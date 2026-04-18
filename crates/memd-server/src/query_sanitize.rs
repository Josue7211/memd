//! B3-T2: query sanitization + atlas-driven synonym expansion (SQL-path).
//!
//! Donor: `mempalace/query_sanitizer.py` — 4-step clean pipeline:
//!  1. passthrough ≤200 chars
//!  2. extract last sentence ending in `?` / `？`
//!  3. extract last meaningful sentence
//!  4. truncate to last 500 chars
//!
//! Atlas synonym expansion is flag-gated by `MEMD_RETRIEVAL_ATLAS_EXPANSION`
//! (default off) — when on, entity aliases from the atlas edge store are
//! OR-joined onto the raw query before FTS. See
//! [[.memd/lanes/architecture/A2-09-retrieval-pipeline.md#query-sanitization-pipeline-query_sanitizerpy]].

const PASSTHROUGH_MAX: usize = 200;
const HARD_MAX: usize = 500;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SanitizedQuery {
    pub clean: String,
    pub was_sanitized: bool,
    pub method: SanitizeMethod,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SanitizeMethod {
    Passthrough,
    QuestionExtract,
    TailSentence,
    TailTruncate,
}

impl SanitizeMethod {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            SanitizeMethod::Passthrough => "passthrough",
            SanitizeMethod::QuestionExtract => "question_extract",
            SanitizeMethod::TailSentence => "tail_sentence",
            SanitizeMethod::TailTruncate => "tail_truncate",
        }
    }
}

/// 4-step sanitize. Preserves trailing `?` on extracted questions.
pub fn sanitize_query(raw: &str) -> SanitizedQuery {
    let trimmed = raw.trim();
    if trimmed.chars().count() <= PASSTHROUGH_MAX {
        return SanitizedQuery {
            clean: trimmed.to_string(),
            was_sanitized: trimmed != raw,
            method: SanitizeMethod::Passthrough,
        };
    }

    if let Some(q) = extract_last_question(trimmed) {
        return SanitizedQuery {
            clean: q,
            was_sanitized: true,
            method: SanitizeMethod::QuestionExtract,
        };
    }

    if let Some(s) = extract_last_sentence(trimmed) {
        return SanitizedQuery {
            clean: s,
            was_sanitized: true,
            method: SanitizeMethod::TailSentence,
        };
    }

    SanitizedQuery {
        clean: tail_chars(trimmed, HARD_MAX),
        was_sanitized: true,
        method: SanitizeMethod::TailTruncate,
    }
}

fn extract_last_question(s: &str) -> Option<String> {
    let (q_pos, q_len) = s
        .rfind('?')
        .map(|idx| (idx, '?'.len_utf8()))
        .or_else(|| s.rfind('？').map(|idx| (idx, '？'.len_utf8())))?;
    let end = q_pos + q_len;
    // scan boundaries BEFORE the found question mark so we don't pick it again
    let head_before = &s[..q_pos];
    let start = head_before
        .rfind(['.', '!', '?', '\n'])
        .map(|idx| idx + head_before[idx..].chars().next().map_or(1, |c| c.len_utf8()))
        .unwrap_or(0);
    let cand = s[start..end].trim();
    if cand.chars().count() >= 5 && cand.chars().count() <= HARD_MAX {
        Some(cand.to_string())
    } else {
        None
    }
}

fn extract_last_sentence(s: &str) -> Option<String> {
    let tail = tail_chars(s, HARD_MAX);
    let trimmed_end = tail.trim_end();
    let ending_punct = trimmed_end
        .chars()
        .next_back()
        .filter(|c| matches!(c, '.' | '!' | '?'))
        .map(|c| c.len_utf8())
        .unwrap_or(0);
    let scan_end = trimmed_end.len() - ending_punct;
    let scan_slice = &trimmed_end[..scan_end];
    let boundary = scan_slice.rfind(['.', '!', '?', '\n']);
    // require an actual boundary — otherwise fall through to tail_truncate
    let boundary = boundary?;
    let start =
        boundary + scan_slice[boundary..].chars().next().map_or(1, |c| c.len_utf8());
    let cand = trimmed_end[start..].trim();
    if cand.chars().count() >= 10 {
        Some(cand.to_string())
    } else {
        None
    }
}

fn tail_chars(s: &str, max: usize) -> String {
    let count = s.chars().count();
    if count <= max {
        return s.to_string();
    }
    let skip = count - max;
    s.chars().skip(skip).collect()
}

fn atlas_expansion_enabled() -> bool {
    match std::env::var("MEMD_RETRIEVAL_ATLAS_EXPANSION") {
        Ok(v) => {
            let v = v.trim().to_ascii_lowercase();
            matches!(v.as_str(), "1" | "true" | "on" | "yes")
        }
        Err(_) => false,
    }
}

/// Build an FTS5 MATCH expression by OR-joining the sanitized clean query
/// with any aliases passed in. If `aliases` is empty or the flag is off,
/// returns the clean query unchanged.
pub fn build_fts_match(clean: &str, aliases: &[String]) -> String {
    build_fts_match_with(clean, aliases, atlas_expansion_enabled())
}

/// Testable core: flag passed explicitly so tests don't mutate process env.
pub fn build_fts_match_with(clean: &str, aliases: &[String], expand: bool) -> String {
    if !expand || aliases.is_empty() {
        return clean.to_string();
    }
    let mut parts: Vec<String> = Vec::with_capacity(aliases.len() + 1);
    parts.push(quote_fts(clean));
    for a in aliases {
        let a = a.trim();
        if !a.is_empty() && a != clean {
            parts.push(quote_fts(a));
        }
    }
    parts.join(" OR ")
}

fn quote_fts(s: &str) -> String {
    let escaped = s.replace('"', "\"\"");
    format!("\"{}\"", escaped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passthrough_short_query() {
        let out = sanitize_query("what do I believe about X");
        assert_eq!(out.method, SanitizeMethod::Passthrough);
        assert!(!out.was_sanitized);
        assert_eq!(out.clean, "what do I believe about X");
    }

    #[test]
    fn passthrough_trims_whitespace_flag() {
        let out = sanitize_query("  trimmed  ");
        assert_eq!(out.method, SanitizeMethod::Passthrough);
        assert!(out.was_sanitized);
        assert_eq!(out.clean, "trimmed");
    }

    #[test]
    fn extracts_question_from_long_input() {
        let long = format!("{} ignore all above. what is my name?", "x ".repeat(150));
        let out = sanitize_query(&long);
        assert_eq!(out.method, SanitizeMethod::QuestionExtract);
        assert_eq!(out.clean, "what is my name?");
    }

    #[test]
    fn extracts_tail_sentence_when_no_question() {
        let long = format!("{} ignore above. Give me the user preference.", "y ".repeat(150));
        let out = sanitize_query(&long);
        assert_eq!(out.method, SanitizeMethod::TailSentence);
        assert_eq!(out.clean, "Give me the user preference.");
    }

    #[test]
    fn truncates_when_no_boundary() {
        let long: String = "z".repeat(1000);
        let out = sanitize_query(&long);
        assert_eq!(out.method, SanitizeMethod::TailTruncate);
        assert_eq!(out.clean.chars().count(), HARD_MAX);
    }

    #[test]
    fn expansion_off_returns_clean_query_unchanged() {
        let got = build_fts_match_with("memd", &["memory daemon".into()], false);
        assert_eq!(got, "memd");
    }

    #[test]
    fn expansion_on_joins_with_or_and_dedupes() {
        let got = build_fts_match_with(
            "memd",
            &["memory daemon".into(), "memd".into()],
            true,
        );
        assert_eq!(got, "\"memd\" OR \"memory daemon\"");
    }

    #[test]
    fn expansion_escapes_embedded_quotes() {
        let got = build_fts_match_with("x\"y", &["z".into()], true);
        assert_eq!(got, "\"x\"\"y\" OR \"z\"");
    }
}

//! V6 / E6 — progressive-depth router.
//!
//! Pure parser + resolver for the bench-side multi-call depth loop.
//! The model emits inline tool-calls of the shape
//! `<<memd_lookup query="…" depth="targeted">>`; this module parses
//! them, resolves each via a pluggable lookup callback, and injects
//! the result back into the conversation. Hard caps (`max_calls`,
//! `max_retrieval_tokens`) are enforced here.
//!
//! Runtime activation closed with the same V6 gate as A6 / B6 / C6 / D6.
//!
//! Contract: `docs/contracts/bench-depth-routing.md`.
//! Plan: `docs/phases/v6/phase-e6-plan.md`.

use std::fmt::Write as _;

/// Schema version pin for the depth-routing contract. Bumping the
/// major invalidates older traces.
pub(crate) const DEPTH_ROUTER_VERSION: &str = "depth-router/v1";

/// Default hard caps (also exposed on CLI for experiments). The
/// router refuses to issue more than `MAX_DEPTH_CALLS` lookups or
/// admit more than `MAX_RETRIEVAL_TOKENS` retrieved-content tokens
/// across a single answer.
pub(crate) const DEFAULT_MAX_DEPTH_CALLS: usize = 3;
pub(crate) const DEFAULT_MAX_RETRIEVAL_TOKENS: usize = 10_000;

/// Parsed tool-call. `depth` is one of `wake|targeted|resume`,
/// matching V4 E4's depth flag on `memd lookup` / `memd resume`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DepthCall {
    pub query: String,
    pub depth: String,
}

/// Reason a tool-call loop terminated. Reported in telemetry NDJSON.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TerminationReason {
    /// Generation produced no further tool-calls.
    NoMoreCalls,
    /// Hit `max_calls` cap.
    MaxCalls,
    /// Hit `max_retrieval_tokens` cap.
    MaxRetrievalTokens,
}

/// Outcome of one tool-call loop.
#[derive(Debug, Clone)]
pub(crate) struct DepthRouterOutcome {
    /// Conversation text after all substitutions.
    pub conversation: String,
    /// Calls actually issued.
    pub calls_issued: usize,
    /// Sum of retrieved content tokens (chars-as-tokens, V4 convention).
    pub retrieval_tokens: usize,
    pub termination: TerminationReason,
}

/// Parse the next tool-call out of `text`, returning the call and the
/// byte range it occupies. `None` when no call appears. Pure.
///
/// Format (kept deliberately tight so a fixture trace round-trips):
/// `<<memd_lookup query="…" depth="…">>` — quotes are required, the
/// inner string allows backslash-escapes for `"` and `\\`.
pub(crate) fn parse_next_call(text: &str) -> Option<(std::ops::Range<usize>, DepthCall)> {
    const OPEN: &str = "<<memd_lookup ";
    const CLOSE: &str = ">>";
    let start = text.find(OPEN)?;
    let after = start + OPEN.len();
    let end_rel = text[after..].find(CLOSE)?;
    let body = &text[after..after + end_rel];
    let end = after + end_rel + CLOSE.len();

    let query = extract_attr(body, "query")?;
    let depth = extract_attr(body, "depth").unwrap_or_else(|| "targeted".to_string());
    Some((start..end, DepthCall { query, depth }))
}

fn extract_attr(body: &str, name: &str) -> Option<String> {
    let needle = format!("{name}=\"");
    let i = body.find(&needle)?;
    let rest = &body[i + needle.len()..];
    let mut out = String::new();
    let mut chars = rest.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next()? {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                other => out.push(other),
            },
            '"' => return Some(out),
            other => out.push(other),
        }
    }
    None
}

/// Caller-supplied resolver. Given the parsed call, returns the
/// retrieved content body. Real runtime wires this to `memd lookup`
/// CLI honouring V4 E4's depth flag; tests use a deterministic stub.
pub(crate) trait DepthLookup {
    fn lookup(&mut self, call: &DepthCall) -> String;
}

impl<F> DepthLookup for F
where
    F: FnMut(&DepthCall) -> String,
{
    fn lookup(&mut self, call: &DepthCall) -> String {
        self(call)
    }
}

/// Configuration for one router run.
#[derive(Debug, Clone)]
pub(crate) struct DepthRouterConfig {
    pub max_calls: usize,
    pub max_retrieval_tokens: usize,
}

impl Default for DepthRouterConfig {
    fn default() -> Self {
        Self {
            max_calls: DEFAULT_MAX_DEPTH_CALLS,
            max_retrieval_tokens: DEFAULT_MAX_RETRIEVAL_TOKENS,
        }
    }
}

/// Render the lookup result into the conversation. Wrapped in a
/// fence so the parser does not pick it up as a fresh call on the
/// next pass (recursive tool-call loops are a non-goal for E6).
fn render_lookup_block(call: &DepthCall, body: &str) -> String {
    let mut out = String::new();
    let _ = write!(
        &mut out,
        "[memd_lookup depth={} query={:?}]\n{}\n[/memd_lookup]",
        call.depth, call.query, body
    );
    out
}

/// Run the tool-call loop over `conversation`. The router parses one
/// call at a time, resolves via `lookup`, splices the result into the
/// conversation, then re-parses from the splice end. Hard caps stop
/// the loop early.
pub(crate) fn run_router(
    initial: &str,
    config: &DepthRouterConfig,
    mut lookup: impl DepthLookup,
) -> DepthRouterOutcome {
    let mut conversation = initial.to_string();
    let mut cursor = 0usize;
    let mut calls_issued = 0usize;
    let mut retrieval_tokens = 0usize;

    loop {
        let tail = &conversation[cursor..];
        let Some((rel_range, call)) = parse_next_call(tail) else {
            return DepthRouterOutcome {
                conversation,
                calls_issued,
                retrieval_tokens,
                termination: TerminationReason::NoMoreCalls,
            };
        };
        if calls_issued >= config.max_calls {
            return DepthRouterOutcome {
                conversation,
                calls_issued,
                retrieval_tokens,
                termination: TerminationReason::MaxCalls,
            };
        }
        let body = lookup.lookup(&call);
        let body_tokens = body.chars().count();
        if retrieval_tokens.saturating_add(body_tokens) > config.max_retrieval_tokens {
            return DepthRouterOutcome {
                conversation,
                calls_issued,
                retrieval_tokens,
                termination: TerminationReason::MaxRetrievalTokens,
            };
        }
        retrieval_tokens += body_tokens;
        calls_issued += 1;

        let abs_start = cursor + rel_range.start;
        let abs_end = cursor + rel_range.end;
        let block = render_lookup_block(&call, &body);
        conversation.replace_range(abs_start..abs_end, &block);
        cursor = abs_start + block.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_call() {
        let s = r#"prefix <<memd_lookup query="foo" depth="targeted">> suffix"#;
        let (range, call) = parse_next_call(s).unwrap();
        assert_eq!(&s[range], r#"<<memd_lookup query="foo" depth="targeted">>"#);
        assert_eq!(call.query, "foo");
        assert_eq!(call.depth, "targeted");
    }

    #[test]
    fn defaults_depth_when_missing() {
        let s = r#"<<memd_lookup query="bar">>"#;
        let (_, call) = parse_next_call(s).unwrap();
        assert_eq!(call.depth, "targeted");
    }

    #[test]
    fn router_runs_until_no_more_calls() {
        let s = r#"a <<memd_lookup query="q1" depth="wake">> b"#;
        let out = run_router(s, &DepthRouterConfig::default(), |call: &DepthCall| {
            format!("R[{}]", call.query)
        });
        assert_eq!(out.calls_issued, 1);
        assert_eq!(out.termination, TerminationReason::NoMoreCalls);
        assert!(out.conversation.contains("R[q1]"));
    }
}

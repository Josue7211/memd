//! V6 / F6 — public benchmark report regenerator (scaffold-symmetric).
//!
//! Pure renderer: takes a per-bench `BenchScorecard` set, formats the
//! canonical PUBLIC_BENCHMARKS.md V6 section. Method-card links are
//! preserved verbatim so reorganisation doesn't break the docs cross
//! references.
//!
//! Plan: `docs/phases/v6/phase-f6-plan.md` §1, §3.

use std::fmt::Write as _;

pub(crate) const REPORT_VERSION: &str = "public-bench-report/v6";

/// One bench's V6 scorecard line, fully resolved against the
/// per-bench primary metric.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BenchScorecard {
    pub bench_id: &'static str,
    pub display_name: &'static str,
    pub metric: &'static str,
    pub value: f64,
    pub target: f64,
    pub method_card: &'static str,
}

/// Render the V6 PUBLIC_BENCHMARKS.md section as a freestanding
/// markdown chunk. Stable string — golden-file tests in
/// `typed_ingest_f6_tests` lock the schema.
pub(crate) fn render_v6_report(cards: &[BenchScorecard]) -> String {
    let mut out = String::new();
    let _ = writeln!(&mut out, "<!-- {REPORT_VERSION} -->");
    let _ = writeln!(&mut out, "## V6 canonical scorecard");
    let _ = writeln!(&mut out);
    let _ = writeln!(
        &mut out,
        "| Bench | Metric | Value | Target | Method card |"
    );
    let _ = writeln!(&mut out, "| --- | --- | --- | --- | --- |");
    for c in cards {
        let _ = writeln!(
            &mut out,
            "| {} | {} | {:.3} | {:.3} | [{}]({}) |",
            c.display_name,
            c.metric,
            c.value,
            c.target,
            method_card_label(c.bench_id),
            c.method_card
        );
    }
    out
}

fn method_card_label(bench_id: &str) -> String {
    format!("{bench_id}-v6")
}

/// Verify every bench in a fixed expected list is present in the
/// rendered report. Used by the canonical-link test.
pub(crate) fn report_contains_all_method_cards(report: &str, cards: &[BenchScorecard]) -> bool {
    cards.iter().all(|c| report.contains(c.method_card))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Vec<BenchScorecard> {
        vec![BenchScorecard {
            bench_id: "lme",
            display_name: "LongMemEval",
            metric: "qa_accuracy",
            value: 0.85,
            target: 0.85,
            method_card: "docs/verification/method-cards/lme-v6.md",
        }]
    }

    #[test]
    fn renders_table_header() {
        let r = render_v6_report(&fixture());
        assert!(r.contains("| Bench | Metric | Value | Target | Method card |"));
    }

    #[test]
    fn round_trips_method_card_link() {
        let r = render_v6_report(&fixture());
        assert!(report_contains_all_method_cards(&r, &fixture()));
    }
}

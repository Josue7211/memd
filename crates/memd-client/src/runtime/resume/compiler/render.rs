//! Markdown emitter — D4.5.
//!
//! Walks admitted buckets in priority order, emits `## <Section>`
//! headers, lists records as `- <truncated record>` lines, and tacks a
//! `## Demoted (use \`memd lookup\`)` section when overflow exists. The
//! returned `tokens` is `markdown.len()` — exact char-count parity with
//! `compute_wake_token_metrics` so per-wake accounting stays consistent.

use std::collections::HashMap;

use super::buckets::AdmittedBuckets;
use super::priority::PRIORITY_ORDER;
use super::{BucketKind, BucketReport, CompiledWake, DemotionHint, WakeBudget};

const RECORD_LINE_MAX: usize = 220;

pub fn emit(
    admitted: AdmittedBuckets,
    budget: &WakeBudget,
    drift_notes: &[String],
) -> CompiledWake {
    let mut markdown = String::new();
    let mut bucket_report: HashMap<BucketKind, BucketReport> = HashMap::new();
    let mut demotion_hints: Vec<DemotionHint> = Vec::new();

    let demoted_lookup: HashMap<BucketKind, usize> = admitted.demoted.iter().copied().collect();
    let admitted_lookup: HashMap<BucketKind, &Vec<memd_schema::CompactMemoryRecord>> = admitted
        .buckets
        .iter()
        .map(|(k, recs)| (*k, recs))
        .collect();

    for kind in PRIORITY_ORDER {
        let admitted_recs = admitted_lookup.get(&kind).copied();
        let admitted_n = admitted_recs.map(|v| v.len()).unwrap_or(0);
        let demoted_n = demoted_lookup.get(&kind).copied().unwrap_or(0);
        let total = admitted_n + demoted_n;
        let fill_ratio = if total == 0 {
            0.0
        } else {
            admitted_n as f64 / total as f64
        };
        bucket_report.insert(
            kind,
            BucketReport {
                admitted: admitted_n,
                demoted: demoted_n,
                fill_ratio,
            },
        );

        if admitted_n == 0 && demoted_n == 0 {
            continue;
        }

        markdown.push_str(&format!("## {}\n\n", kind.section_header()));
        if matches!(kind, BucketKind::Preference) && !drift_notes.is_empty() {
            for note in drift_notes {
                markdown.push_str(note);
                markdown.push('\n');
            }
            markdown.push('\n');
        }
        if let Some(recs) = admitted_recs {
            for record in recs {
                markdown.push_str(&format!(
                    "- {}\n",
                    truncate_line(record.record.trim(), RECORD_LINE_MAX)
                ));
            }
        }
        if demoted_n > 0 {
            markdown.push_str(&format!(
                "- + {} more in this bucket via `memd lookup --query <topic>`\n",
                demoted_n
            ));
            demotion_hints.push(DemotionHint {
                bucket: kind,
                count: demoted_n,
                reason: "budget overflow".to_string(),
            });
        }
        markdown.push('\n');
    }

    if !demotion_hints.is_empty() {
        markdown.push_str("## Demoted (use `memd lookup`)\n\n");
        for hint in &demotion_hints {
            markdown.push_str(&format!(
                "- {}: {} record(s) demoted ({})\n",
                hint.bucket.section_header(),
                hint.count,
                hint.reason
            ));
        }
        markdown.push('\n');
    }

    let total_admitted_chars: usize = admitted_lookup
        .values()
        .flat_map(|recs| recs.iter())
        .map(|r| r.record.len())
        .sum();
    markdown.push_str(&format!(
        "_compiled wake: budget={} chars, admitted_records={}_\n",
        budget.tokens, total_admitted_chars,
    ));

    let tokens = markdown.len();
    CompiledWake {
        markdown,
        tokens,
        bucket_report,
        demotion_hints,
    }
}

fn truncate_line(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    let mut out = String::with_capacity(max_chars);
    for ch in value.chars().take(max_chars.saturating_sub(1)) {
        out.push(ch);
    }
    out.push('…');
    out
}

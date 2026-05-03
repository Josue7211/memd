//! G5.4 — strict-mode regenerator for `docs/verification/MEMD-10-STAR.md`.
//!
//! Rules (V5-INTEGRATION §9b):
//! 1. Axis credit ceiling: PR ≤ 4, CH ≤ 4, RR ≤ 6.
//! 2. Owned-axis-only: V5 writes only PR, CH, RR rows. SC, CR, TE, TP rows
//!    are preserved verbatim from the prior file.
//! 3. Composite refuses to write below 4.20 unless `allow_below_target`.
//!
//! The regenerator parses the existing axis table, mutates only the three
//! V5-owned rows (score column + status column when caller supplied a new
//! status), recomputes composite from the merged 7-axis row, and rewrites
//! the `**Composite: …**` line. Prose evidence blocks are left untouched.

use std::path::Path;

use crate::benchmark::substrate::aggregator::SuiteSummary;

/// V5-owned axis scores. Non-owned axes are read from the existing file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AxisScores {
    pub(crate) pr: u8,
    pub(crate) ch: u8,
    pub(crate) rr: u8,
}

/// V5 composite gate (V5-INTEGRATION §9b).
pub(crate) const COMPOSITE_TARGET: f64 = 4.20;

/// Per-axis ceilings the V5 regenerator may write (V5-INTEGRATION §9b.1).
const PR_CEILING: u8 = 4;
const CH_CEILING: u8 = 4;
const RR_CEILING: u8 = 6;

/// Canonical axis labels — must match the markdown table column 1 verbatim.
const AXIS_SC: &str = "Session continuity";
const AXIS_CR: &str = "Correction retention";
const AXIS_PR: &str = "Procedural reuse";
const AXIS_CH: &str = "Cross-harness continuity";
const AXIS_RR: &str = "Raw retrieval strength";
const AXIS_TE: &str = "Token efficiency";
const AXIS_TP: &str = "Trust + provenance";

#[derive(Debug)]
pub(crate) enum RegenError {
    /// V5 caller tried to score an axis above its V5 ceiling.
    CeilingExceeded {
        axis: &'static str,
        score: u8,
        ceiling: u8,
    },
    /// Computed composite below the V5 gate and `allow_below_target` is off.
    CompositeBelowTarget { composite: f64, target: f64 },
    /// Existing 10-STAR file is missing or malformed.
    ParseError(String),
    /// Filesystem I/O.
    IoError(std::io::Error),
}

impl std::fmt::Display for RegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CeilingExceeded {
                axis,
                score,
                ceiling,
            } => {
                write!(
                    f,
                    "10-STAR ceiling exceeded: {axis} = {score}/10 > V5 ceiling {ceiling}/10"
                )
            }
            Self::CompositeBelowTarget { composite, target } => {
                write!(
                    f,
                    "10-STAR composite {composite:.2} < V5 gate {target:.2} (use --allow-below-target to override)"
                )
            }
            Self::ParseError(msg) => write!(f, "10-STAR parse error: {msg}"),
            Self::IoError(e) => write!(f, "10-STAR io error: {e}"),
        }
    }
}

impl std::error::Error for RegenError {}

impl From<std::io::Error> for RegenError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

/// Map V5 suite summaries to V5-owned axis scores (PR/CH/RR).
///
/// Mapping (per V5-INTEGRATION §9b axis-lift contract + MILESTONE-v5
/// PR-axis assertion):
/// - PR (procedural-reuse) ← typed-retrieval pass AND `live_fire_pass`
///   metric == 1.0 (routine plant in S1 → invocation in S2+ with
///   token_savings ≥ baseline_retrieval_cost). Else 1.
/// - CH (cross-harness)    ← cross-harness pass: 2 → 4 on C5 lift, else 2.
/// - RR (raw-retrieval)    ← floor 4; +2 if A5+D5+E5+F5+G5 all pass.
pub(crate) fn axis_scores_from_summaries(summaries: &[SuiteSummary]) -> AxisScores {
    let pass = |id: &str| summaries.iter().any(|s| s.id == id && s.pass);
    let metric = |id: &str, key: &str| -> Option<f64> {
        summaries
            .iter()
            .find(|s| s.id == id)
            .and_then(|s| s.metrics.get(key).copied())
    };

    let typed_pass = pass("typed-retrieval");
    let live_fire_credit = metric("typed-retrieval", "live_fire_pass")
        .map(|v| (v - 1.0).abs() < 1e-9)
        .unwrap_or(false);
    let pr = if typed_pass && live_fire_credit { 4 } else { 1 };
    let ch = if pass("cross-harness") { 4 } else { 2 };

    let rr_aggregate = pass("cross-session-recall")
        && pass("progressive-depth")
        && pass("provenance-integrity")
        && pass("typed-retrieval")
        && pass("adversarial-noise");
    let rr = if rr_aggregate { 6 } else { 4 };

    AxisScores { pr, ch, rr }
}

/// Regenerate the canonical 10-STAR doc with the V5-owned axes patched.
/// Returns the recomputed composite on success.
pub(crate) fn regenerate_10star_md(
    doc_path: &Path,
    scores: &AxisScores,
    allow_below_target: bool,
) -> Result<f64, RegenError> {
    enforce_ceiling(AXIS_PR, scores.pr, PR_CEILING)?;
    enforce_ceiling(AXIS_CH, scores.ch, CH_CEILING)?;
    enforce_ceiling(AXIS_RR, scores.rr, RR_CEILING)?;

    let body = std::fs::read_to_string(doc_path)?;
    let mut table = parse_axis_table(&body)?;

    // Patch only V5-owned rows.
    set_axis_score(&mut table, AXIS_PR, scores.pr);
    set_axis_score(&mut table, AXIS_CH, scores.ch);
    set_axis_score(&mut table, AXIS_RR, scores.rr);

    let composite = compute_composite(&table)?;

    if composite + 1e-9 < COMPOSITE_TARGET && !allow_below_target {
        return Err(RegenError::CompositeBelowTarget {
            composite,
            target: COMPOSITE_TARGET,
        });
    }

    let new_body = rewrite_doc(&body, &table, composite)?;
    std::fs::write(doc_path, new_body)?;
    Ok(composite)
}

fn enforce_ceiling(axis: &'static str, score: u8, ceiling: u8) -> Result<(), RegenError> {
    if score > ceiling {
        Err(RegenError::CeilingExceeded {
            axis,
            score,
            ceiling,
        })
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct AxisRow {
    axis: String,
    weight_pct: f64,
    score: u8,
    status: String,
    line_idx: usize,
}

const REQUIRED_AXES: &[(&str, f64)] = &[
    (AXIS_SC, 20.0),
    (AXIS_CR, 15.0),
    (AXIS_PR, 15.0),
    (AXIS_CH, 15.0),
    (AXIS_RR, 15.0),
    (AXIS_TE, 10.0),
    (AXIS_TP, 10.0),
];

fn parse_axis_table(body: &str) -> Result<Vec<AxisRow>, RegenError> {
    let mut rows = Vec::new();
    for (idx, line) in body.lines().enumerate() {
        let trim = line.trim_start();
        if !trim.starts_with('|') {
            continue;
        }
        // skip header + separator rows
        if trim.contains("Axis") && trim.contains("Weight") {
            continue;
        }
        if trim.contains("---") {
            continue;
        }
        let cols: Vec<&str> = trim
            .trim_start_matches('|')
            .trim_end_matches('|')
            .split('|')
            .map(|c| c.trim())
            .collect();
        if cols.len() < 4 {
            continue;
        }
        let axis = cols[0].to_string();
        if !REQUIRED_AXES.iter().any(|(name, _)| *name == axis) {
            continue;
        }
        let weight_pct = cols[1].trim_end_matches('%').parse::<f64>().map_err(|_| {
            RegenError::ParseError(format!("bad weight col on line {idx}: {line:?}"))
        })?;
        let score = cols[2]
            .split('/')
            .next()
            .and_then(|n| n.trim().parse::<u8>().ok())
            .ok_or_else(|| {
                RegenError::ParseError(format!("bad score col on line {idx}: {line:?}"))
            })?;
        let status = cols[3..].join(" | ");
        rows.push(AxisRow {
            axis,
            weight_pct,
            score,
            status,
            line_idx: idx,
        });
    }

    for (name, _) in REQUIRED_AXES {
        if !rows.iter().any(|r| r.axis == *name) {
            return Err(RegenError::ParseError(format!("axis row missing: {name}")));
        }
    }
    Ok(rows)
}

fn set_axis_score(rows: &mut [AxisRow], axis: &str, score: u8) {
    if let Some(row) = rows.iter_mut().find(|r| r.axis == axis) {
        row.score = score;
    }
}

fn compute_composite(rows: &[AxisRow]) -> Result<f64, RegenError> {
    let mut total = 0.0;
    let mut weight_sum = 0.0;
    for (name, expected_pct) in REQUIRED_AXES {
        let row = rows.iter().find(|r| r.axis == *name).ok_or_else(|| {
            RegenError::ParseError(format!("axis missing during compute: {name}"))
        })?;
        if (row.weight_pct - expected_pct).abs() > 1e-6 {
            return Err(RegenError::ParseError(format!(
                "weight mismatch for {name}: file={}, expected={}",
                row.weight_pct, expected_pct
            )));
        }
        total += row.weight_pct / 100.0 * row.score as f64;
        weight_sum += row.weight_pct;
    }
    if (weight_sum - 100.0).abs() > 1e-6 {
        return Err(RegenError::ParseError(format!(
            "axis weights must sum to 100, got {weight_sum}"
        )));
    }
    Ok(round2(total))
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn rewrite_doc(body: &str, rows: &[AxisRow], composite: f64) -> Result<String, RegenError> {
    let mut lines: Vec<String> = body.lines().map(|s| s.to_string()).collect();

    // Patch each owned axis row in place.
    for axis in [AXIS_PR, AXIS_CH, AXIS_RR] {
        let row = rows
            .iter()
            .find(|r| r.axis == axis)
            .ok_or_else(|| RegenError::ParseError(format!("missing row {axis} during rewrite")))?;
        let new_line = format!(
            "| {axis} | {weight}% | {score}/10 | {status} |",
            axis = row.axis,
            weight = format_weight(row.weight_pct),
            score = row.score,
            status = row.status,
        );
        lines[row.line_idx] = new_line;
    }

    // Replace the first `**Composite: …**` line.
    let composite_line = format!(
        "**Composite: {composite:.2}/10 (G5 regenerated {date} — V5 substrate aggregator)**",
        date = chrono::Utc::now().format("%Y-%m-%d"),
    );
    let mut replaced = false;
    for line in lines.iter_mut() {
        if line.trim_start().starts_with("**Composite:") {
            *line = composite_line.clone();
            replaced = true;
            break;
        }
    }
    if !replaced {
        return Err(RegenError::ParseError(
            "no `**Composite: …**` line found to update".into(),
        ));
    }

    let mut out = lines.join("\n");
    if !out.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}

fn format_weight(pct: f64) -> String {
    if (pct.fract()).abs() < 1e-6 {
        format!("{}", pct as u32)
    } else {
        format!("{pct}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn fixture_body(scores: [(&'static str, u8); 7]) -> String {
        let mut s = String::from(
            "# memd 10-Star Target\n\n## 10-Star Composite Scorecard\n\n\
             | Axis | Weight | Score | Status |\n\
             |------|--------|-------|--------|\n",
        );
        for (axis, score) in scores {
            let weight = match axis {
                AXIS_SC => 20,
                AXIS_CR | AXIS_PR | AXIS_CH | AXIS_RR => 15,
                AXIS_TE | AXIS_TP => 10,
                _ => unreachable!(),
            };
            s.push_str(&format!(
                "| {axis} | {weight}% | {score}/10 | placeholder status |\n"
            ));
        }
        s.push_str("\n**Composite: 0.00/10 (placeholder)**\n\nEvidence prose follows.\n");
        s
    }

    fn baseline_at_v4_post() -> [(&'static str, u8); 7] {
        [
            (AXIS_SC, 4),
            (AXIS_CR, 4),
            (AXIS_PR, 1),
            (AXIS_CH, 4),
            (AXIS_RR, 4),
            (AXIS_TE, 4),
            (AXIS_TP, 3),
        ]
    }

    #[test]
    fn axis_scores_from_summaries_lifts_only_when_suite_passes() {
        fn pass(id: &str) -> SuiteSummary {
            SuiteSummary::passed(id, BTreeMap::new())
        }
        fn pass_typed_with_live_fire() -> SuiteSummary {
            let mut m = BTreeMap::new();
            m.insert("live_fire_pass".to_string(), 1.0);
            SuiteSummary::passed("typed-retrieval", m)
        }
        fn fail(id: &str) -> SuiteSummary {
            SuiteSummary::failed(id, "x", BTreeMap::new())
        }

        let all_pass = vec![
            pass("cross-session-recall"),
            pass("correction-propagation"),
            pass("cross-harness"),
            pass("progressive-depth"),
            pass("provenance-integrity"),
            pass_typed_with_live_fire(),
            pass("adversarial-noise"),
        ];
        let s = axis_scores_from_summaries(&all_pass);
        assert_eq!(
            s,
            AxisScores {
                pr: 4,
                ch: 4,
                rr: 6
            }
        );

        let mut mixed = all_pass.clone();
        mixed[2] = fail("cross-harness");
        let s = axis_scores_from_summaries(&mixed);
        assert_eq!(s.ch, 2, "CH falls to 2 without C5 pass");
        assert_eq!(s.pr, 4, "PR independent of CH");
        assert_eq!(s.rr, 6, "RR aggregate doesn't depend on C5/B5");

        let mut no_g5 = all_pass.clone();
        no_g5[6] = fail("adversarial-noise");
        let s = axis_scores_from_summaries(&no_g5);
        assert_eq!(s.rr, 4, "RR drops if G5 fails");

        // PR=4 requires live_fire_pass metric. Typed-retrieval pass alone
        // (raw-retrieval credit only) leaves PR at floor.
        let mut no_live_fire = all_pass.clone();
        no_live_fire[5] = pass("typed-retrieval");
        let s = axis_scores_from_summaries(&no_live_fire);
        assert_eq!(
            s.pr, 1,
            "PR stays at 1 without live_fire_pass metric — typed-retrieval alone is RR credit"
        );
    }

    #[test]
    fn ceiling_exceeded_blocks_write() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("MEMD-10-STAR.md");
        std::fs::write(&path, fixture_body(baseline_at_v4_post())).unwrap();
        let bad = AxisScores {
            pr: 5,
            ch: 4,
            rr: 6,
        };
        let err = regenerate_10star_md(&path, &bad, true).unwrap_err();
        assert!(matches!(err, RegenError::CeilingExceeded { axis, .. } if axis == AXIS_PR));
    }

    #[test]
    fn composite_below_target_blocks_without_flag() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("MEMD-10-STAR.md");
        // CR=1 floor (V4 not closed) means even max V5 lift can't reach 4.20.
        let mut baseline = baseline_at_v4_post();
        baseline[1].1 = 1;
        let body_before = fixture_body(baseline);
        std::fs::write(&path, &body_before).unwrap();

        let v5_max = AxisScores {
            pr: 4,
            ch: 4,
            rr: 6,
        };
        let err = regenerate_10star_md(&path, &v5_max, false).unwrap_err();
        assert!(matches!(err, RegenError::CompositeBelowTarget { .. }));

        // File untouched on refusal.
        let body_after = std::fs::read_to_string(&path).unwrap();
        assert_eq!(body_before, body_after);

        // With override flag: writes and returns the (sub-target) composite.
        let composite = regenerate_10star_md(&path, &v5_max, true).unwrap();
        assert!(composite < COMPOSITE_TARGET);
    }

    #[test]
    fn happy_path_writes_composite_4_20() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("MEMD-10-STAR.md");
        std::fs::write(&path, fixture_body(baseline_at_v4_post())).unwrap();

        let scores = AxisScores {
            pr: 4,
            ch: 4,
            rr: 6,
        };
        let composite = regenerate_10star_md(&path, &scores, false).unwrap();
        assert!(
            (composite - 4.20).abs() < 1e-6,
            "composite must equal exactly 4.20, got {composite}"
        );

        let body = std::fs::read_to_string(&path).unwrap();
        assert!(body.contains("| Procedural reuse | 15% | 4/10"));
        assert!(body.contains("| Cross-harness continuity | 15% | 4/10"));
        assert!(body.contains("| Raw retrieval strength | 15% | 6/10"));
        assert!(body.contains("**Composite: 4.20/10"));
    }

    #[test]
    fn non_owned_axes_preserved_verbatim() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("MEMD-10-STAR.md");
        // Idiosyncratic non-owned values to prove they are NOT touched.
        let baseline = [
            (AXIS_SC, 4),
            (AXIS_CR, 1),
            (AXIS_PR, 1),
            (AXIS_CH, 2),
            (AXIS_RR, 4),
            (AXIS_TE, 2),
            (AXIS_TP, 3),
        ];
        std::fs::write(&path, fixture_body(baseline)).unwrap();
        let scores = AxisScores {
            pr: 4,
            ch: 4,
            rr: 6,
        };
        let _ = regenerate_10star_md(&path, &scores, true).unwrap();
        let body = std::fs::read_to_string(&path).unwrap();
        assert!(body.contains("| Session continuity | 20% | 4/10"));
        assert!(body.contains("| Correction retention | 15% | 1/10"));
        assert!(body.contains("| Token efficiency | 10% | 2/10"));
        assert!(body.contains("| Trust + provenance | 10% | 3/10"));
    }

    #[test]
    fn parse_error_when_axis_row_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("MEMD-10-STAR.md");
        std::fs::write(
            &path,
            "# 10-Star\n\n| Axis | Weight | Score | Status |\n|--|--|--|--|\n| Session continuity | 20% | 4/10 | x |\n",
        )
        .unwrap();
        let scores = AxisScores {
            pr: 4,
            ch: 4,
            rr: 6,
        };
        let err = regenerate_10star_md(&path, &scores, true).unwrap_err();
        assert!(matches!(err, RegenError::ParseError(_)));
    }
}

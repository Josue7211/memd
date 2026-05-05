//! V6 / F6 — `docs/verification/MEMD-10-STAR.md` regenerator for the
//! V6 typed pipeline.
//!
//! V5 substrate writer (`benchmark::substrate::ten_star_writer`) caps
//! V5-owned axes at PR≤4 / CH≤4 / RR≤6 and writes composite ≥4.20.
//! V6 lifts RR 6→7 (real-corpus typed pipeline gates pass) and
//! TP 3→4 (method cards + reproducibility script land). The V6 writer
//! patches only RR and TP rows and refuses to drop below
//! `V6_COMPOSITE_TARGET = 4.45` unless `allow_below_target` is set.
//!
//! Plan: `docs/phases/v6/phase-f6-plan.md` §3, §6;
//! contract row from `docs/verification/0.1.0-CONTRACT.md` §V6.

use std::path::Path;

use crate::benchmark::typed_ingest::report_aggregator::BenchScorecard;

/// V6 milestone composite gate (`MILESTONE-v6.md`).
pub(crate) const V6_COMPOSITE_TARGET: f64 = 4.45;

/// Per-axis ceilings the V6 regenerator may write.
const RR_CEILING_V6: u8 = 7;
const TP_CEILING_V6: u8 = 4;

/// V5 floors the V6 writer must preserve when the V6 lift is missed.
const RR_FLOOR_V5: u8 = 6;
const TP_FLOOR_V4: u8 = 3;

const AXIS_SC: &str = "Session continuity";
const AXIS_CR: &str = "Correction retention";
const AXIS_PR: &str = "Procedural reuse";
const AXIS_CH: &str = "Cross-harness continuity";
const AXIS_RR: &str = "Raw retrieval strength";
const AXIS_TE: &str = "Token efficiency";
const AXIS_TP: &str = "Trust + provenance";

const REQUIRED_AXES: &[(&str, f64)] = &[
    (AXIS_SC, 20.0),
    (AXIS_CR, 15.0),
    (AXIS_PR, 15.0),
    (AXIS_CH, 15.0),
    (AXIS_RR, 15.0),
    (AXIS_TE, 10.0),
    (AXIS_TP, 10.0),
];

/// V6-owned axis scores. Only RR and TP move under V6.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct V6AxisScores {
    pub rr: u8,
    pub tp: u8,
}

#[derive(Debug)]
pub(crate) enum V6RegenError {
    CeilingExceeded {
        axis: &'static str,
        score: u8,
        ceiling: u8,
    },
    CompositeBelowTarget {
        composite: f64,
        target: f64,
    },
    ParseError(String),
    IoError(std::io::Error),
}

impl std::fmt::Display for V6RegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CeilingExceeded {
                axis,
                score,
                ceiling,
            } => write!(
                f,
                "10-STAR V6 ceiling exceeded: {axis} = {score}/10 > V6 ceiling {ceiling}/10"
            ),
            Self::CompositeBelowTarget { composite, target } => write!(
                f,
                "10-STAR V6 composite {composite:.2} < V6 gate {target:.2} (use --allow-below-target to override)"
            ),
            Self::ParseError(msg) => write!(f, "10-STAR V6 parse error: {msg}"),
            Self::IoError(e) => write!(f, "10-STAR V6 io error: {e}"),
        }
    }
}

impl std::error::Error for V6RegenError {}

impl From<std::io::Error> for V6RegenError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

/// Map V6 per-bench scorecards (post real-corpus run) to V6-owned
/// axis scores. RR lifts to 7 only when all four canonical bench
/// gates pass (`value >= target`). TP lifts to 4 only when both the
/// method-cards-on-disk gate and the reproducibility-script-on-disk
/// gate pass. Floors preserve the V5 baseline.
pub(crate) fn axis_scores_from_v6_scorecards(
    cards: &[BenchScorecard],
    method_cards_present: bool,
    reproducibility_script_present: bool,
) -> V6AxisScores {
    let four_canonical = ["lme", "locomo", "membench", "convomem"];
    let all_pass = four_canonical.iter().all(|id| {
        cards
            .iter()
            .find(|c| c.bench_id == *id)
            .map(|c| c.value >= c.target)
            .unwrap_or(false)
    });
    let rr = if all_pass { RR_CEILING_V6 } else { RR_FLOOR_V5 };
    let tp = if method_cards_present && reproducibility_script_present {
        TP_CEILING_V6
    } else {
        TP_FLOOR_V4
    };
    V6AxisScores { rr, tp }
}

/// Regenerate the canonical 10-STAR doc with V6-owned axes patched.
/// Returns the recomputed composite on success.
pub(crate) fn regenerate_10star_md_v6(
    doc_path: &Path,
    scores: &V6AxisScores,
    allow_below_target: bool,
) -> Result<f64, V6RegenError> {
    enforce_ceiling(AXIS_RR, scores.rr, RR_CEILING_V6)?;
    enforce_ceiling(AXIS_TP, scores.tp, TP_CEILING_V6)?;

    let body = std::fs::read_to_string(doc_path)?;
    let mut table = parse_axis_table(&body)?;

    set_axis_score(&mut table, AXIS_RR, scores.rr);
    set_axis_score(&mut table, AXIS_TP, scores.tp);

    let composite = compute_composite(&table)?;
    if composite + 1e-9 < V6_COMPOSITE_TARGET && !allow_below_target {
        return Err(V6RegenError::CompositeBelowTarget {
            composite,
            target: V6_COMPOSITE_TARGET,
        });
    }

    let new_body = rewrite_doc(&body, &table, composite)?;
    std::fs::write(doc_path, new_body)?;
    Ok(composite)
}

fn enforce_ceiling(axis: &'static str, score: u8, ceiling: u8) -> Result<(), V6RegenError> {
    if score > ceiling {
        Err(V6RegenError::CeilingExceeded {
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

fn parse_axis_table(body: &str) -> Result<Vec<AxisRow>, V6RegenError> {
    let mut rows = Vec::new();
    for (idx, line) in body.lines().enumerate() {
        let trim = line.trim_start();
        if !trim.starts_with('|') {
            continue;
        }
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
            V6RegenError::ParseError(format!("bad weight col on line {idx}: {line:?}"))
        })?;
        let score = cols[2]
            .split('/')
            .next()
            .and_then(|n| n.trim().parse::<u8>().ok())
            .ok_or_else(|| {
                V6RegenError::ParseError(format!("bad score col on line {idx}: {line:?}"))
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
            return Err(V6RegenError::ParseError(format!(
                "axis row missing: {name}"
            )));
        }
    }
    Ok(rows)
}

fn set_axis_score(rows: &mut [AxisRow], axis: &str, score: u8) {
    if let Some(row) = rows.iter_mut().find(|r| r.axis == axis) {
        row.score = score;
    }
}

fn compute_composite(rows: &[AxisRow]) -> Result<f64, V6RegenError> {
    let mut total = 0.0;
    let mut weight_sum = 0.0;
    for (name, expected_pct) in REQUIRED_AXES {
        let row = rows.iter().find(|r| r.axis == *name).ok_or_else(|| {
            V6RegenError::ParseError(format!("axis missing during compute: {name}"))
        })?;
        if (row.weight_pct - expected_pct).abs() > 1e-6 {
            return Err(V6RegenError::ParseError(format!(
                "weight mismatch for {name}: file={}, expected={expected_pct}",
                row.weight_pct
            )));
        }
        total += row.weight_pct / 100.0 * row.score as f64;
        weight_sum += row.weight_pct;
    }
    if (weight_sum - 100.0).abs() > 1e-6 {
        return Err(V6RegenError::ParseError(format!(
            "axis weights must sum to 100, got {weight_sum}"
        )));
    }
    Ok(round2(total))
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn rewrite_doc(body: &str, rows: &[AxisRow], composite: f64) -> Result<String, V6RegenError> {
    let mut lines: Vec<String> = body.lines().map(|s| s.to_string()).collect();

    for axis in [AXIS_RR, AXIS_TP] {
        let row = rows
            .iter()
            .find(|r| r.axis == axis)
            .ok_or_else(|| V6RegenError::ParseError(format!("missing row {axis}")))?;
        let new_line = format!(
            "| {axis} | {weight}% | {score}/10 | {status} |",
            axis = row.axis,
            weight = format_weight(row.weight_pct),
            score = row.score,
            status = row.status,
        );
        lines[row.line_idx] = new_line;
    }

    let composite_line = format!(
        "**Composite: {composite:.2}/10 (F6 regenerated {date} — V6 typed pipeline)**",
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
        return Err(V6RegenError::ParseError(
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
    if pct.fract().abs() < 1e-6 {
        format!("{}", pct as u32)
    } else {
        format!("{pct}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_body() -> String {
        let mut s = String::from(
            "# memd 10-Star Target\n\n## 10-Star Composite Scorecard\n\n\
             | Axis | Weight | Score | Status |\n\
             |------|--------|-------|--------|\n",
        );
        let rows: [(&'static str, u8); 7] = [
            (AXIS_SC, 4),
            (AXIS_CR, 4),
            (AXIS_PR, 4),
            (AXIS_CH, 4),
            (AXIS_RR, 6),
            (AXIS_TE, 4),
            (AXIS_TP, 3),
        ];
        for (axis, score) in rows {
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
        s.push_str("\n**Composite: 4.20/10 (placeholder)**\n\nEvidence prose follows.\n");
        s
    }

    fn pass_card(bench_id: &'static str, value: f64, target: f64) -> BenchScorecard {
        BenchScorecard {
            bench_id,
            display_name: bench_id,
            metric: "x",
            value,
            target,
            method_card: "x",
        }
    }

    #[test]
    fn axis_mapper_lifts_when_all_gates_pass_and_artifacts_present() {
        let cards = vec![
            pass_card("lme", 0.86, 0.85),
            pass_card("locomo", 0.76, 0.75),
            pass_card("membench", 0.76, 0.75),
            pass_card("convomem", 0.91, 0.90),
        ];
        let s = axis_scores_from_v6_scorecards(&cards, true, true);
        assert_eq!(s, V6AxisScores { rr: 7, tp: 4 });
    }

    #[test]
    fn axis_mapper_holds_floor_when_one_gate_fails() {
        let cards = vec![
            pass_card("lme", 0.84, 0.85),
            pass_card("locomo", 0.76, 0.75),
            pass_card("membench", 0.76, 0.75),
            pass_card("convomem", 0.91, 0.90),
        ];
        let s = axis_scores_from_v6_scorecards(&cards, true, true);
        assert_eq!(s.rr, 6, "RR drops to V5 floor when any bench gate fails");
        assert_eq!(s.tp, 4);
    }

    #[test]
    fn axis_mapper_drops_tp_when_artifacts_missing() {
        let cards = vec![
            pass_card("lme", 0.86, 0.85),
            pass_card("locomo", 0.76, 0.75),
            pass_card("membench", 0.76, 0.75),
            pass_card("convomem", 0.91, 0.90),
        ];
        assert_eq!(
            axis_scores_from_v6_scorecards(&cards, false, true),
            V6AxisScores { rr: 7, tp: 3 }
        );
        assert_eq!(
            axis_scores_from_v6_scorecards(&cards, true, false),
            V6AxisScores { rr: 7, tp: 3 }
        );
    }

    #[test]
    fn ceiling_blocks_rr_above_7() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("MEMD-10-STAR.md");
        std::fs::write(&path, baseline_body()).unwrap();
        let err = regenerate_10star_md_v6(&path, &V6AxisScores { rr: 8, tp: 4 }, true).unwrap_err();
        assert!(matches!(err, V6RegenError::CeilingExceeded { axis, .. } if axis == AXIS_RR));
    }

    #[test]
    fn ceiling_blocks_tp_above_4() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("MEMD-10-STAR.md");
        std::fs::write(&path, baseline_body()).unwrap();
        let err = regenerate_10star_md_v6(&path, &V6AxisScores { rr: 7, tp: 5 }, true).unwrap_err();
        assert!(matches!(err, V6RegenError::CeilingExceeded { axis, .. } if axis == AXIS_TP));
    }

    #[test]
    fn happy_path_writes_composite_4_45() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("MEMD-10-STAR.md");
        std::fs::write(&path, baseline_body()).unwrap();
        let composite =
            regenerate_10star_md_v6(&path, &V6AxisScores { rr: 7, tp: 4 }, false).unwrap();
        assert!(
            (composite - 4.45).abs() < 1e-6,
            "composite must equal 4.45, got {composite}"
        );
        let body = std::fs::read_to_string(&path).unwrap();
        assert!(body.contains("| Raw retrieval strength | 15% | 7/10"));
        assert!(body.contains("| Trust + provenance | 10% | 4/10"));
        assert!(body.contains("**Composite: 4.45/10"));
        assert!(body.contains("V6 typed pipeline"));
    }

    #[test]
    fn refuses_below_target_without_override() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("MEMD-10-STAR.md");
        std::fs::write(&path, baseline_body()).unwrap();
        let err =
            regenerate_10star_md_v6(&path, &V6AxisScores { rr: 6, tp: 3 }, false).unwrap_err();
        assert!(matches!(err, V6RegenError::CompositeBelowTarget { .. }));
        let body_after = std::fs::read_to_string(&path).unwrap();
        assert!(
            body_after.contains("**Composite: 4.20/10 (placeholder)"),
            "file untouched on refusal"
        );
    }

    #[test]
    fn override_writes_below_target() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("MEMD-10-STAR.md");
        std::fs::write(&path, baseline_body()).unwrap();
        let composite =
            regenerate_10star_md_v6(&path, &V6AxisScores { rr: 6, tp: 3 }, true).unwrap();
        assert!(composite < V6_COMPOSITE_TARGET);
        let body = std::fs::read_to_string(&path).unwrap();
        assert!(body.contains("V6 typed pipeline"));
    }

    #[test]
    fn non_owned_axes_preserved() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("MEMD-10-STAR.md");
        std::fs::write(&path, baseline_body()).unwrap();
        let _ = regenerate_10star_md_v6(&path, &V6AxisScores { rr: 7, tp: 4 }, false).unwrap();
        let body = std::fs::read_to_string(&path).unwrap();
        assert!(body.contains("| Session continuity | 20% | 4/10"));
        assert!(body.contains("| Correction retention | 15% | 4/10"));
        assert!(body.contains("| Procedural reuse | 15% | 4/10"));
        assert!(body.contains("| Cross-harness continuity | 15% | 4/10"));
        assert!(body.contains("| Token efficiency | 10% | 4/10"));
    }
}

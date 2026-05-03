//! G4.4 — 10-STAR scorecard regenerator.
//!
//! Tests 9 + 10 of `phase-g4-plan.md §4`. Strict mode (per §G4 plan revision
//! 2026-04-22): the regenerator refuses to write when any axis is over-claimed
//! relative to the milestone target, exits non-zero with a diff, and does not
//! mutate the scorecard. Healthy mode (observed ≤ target across all axes)
//! returns the rewritten table content.
//!
//! Inputs are typed maps so the asserter is decoupled from the actual
//! MEMD-10-STAR.md location — G4.5 wires the live file path in.

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AxisRow {
    pub axis: String,
    pub weight_pct: u32,
    pub score_out_of_10: u32,
    pub status: String,
}

/// Parse the `## 10-Star Composite Scorecard` markdown table into typed rows.
/// Recognises the shape `| Axis | Weight | Score | Status |` followed by a
/// separator and one row per axis. Stops at the first blank line after the
/// rows.
pub(crate) fn parse_scorecard_table(markdown: &str) -> Result<Vec<AxisRow>, String> {
    let mut in_table = false;
    let mut header_seen = false;
    let mut rows: Vec<AxisRow> = Vec::new();

    for line in markdown.lines() {
        let trimmed = line.trim();
        if !in_table {
            if trimmed.starts_with("| Axis")
                && trimmed.contains("Weight")
                && trimmed.contains("Score")
            {
                in_table = true;
            }
            continue;
        }
        if !header_seen {
            // Separator line `|---|...|`.
            if trimmed.starts_with("|") && trimmed.contains("---") {
                header_seen = true;
            }
            continue;
        }
        if trimmed.is_empty() || !trimmed.starts_with("|") {
            break;
        }
        let cells: Vec<&str> = trimmed
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        if cells.len() < 4 {
            return Err(format!(
                "scorecard row has {} cells, expected ≥4: {trimmed}",
                cells.len()
            ));
        }
        let axis = cells[0].to_string();
        let weight_pct = cells[1]
            .trim_end_matches('%')
            .trim()
            .parse::<u32>()
            .map_err(|err| format!("weight `{}` not a u32: {err}", cells[1]))?;
        let (num, _denom) = cells[2]
            .split_once('/')
            .ok_or_else(|| format!("score `{}` missing /10", cells[2]))?;
        let score_out_of_10 = num
            .trim()
            .parse::<u32>()
            .map_err(|err| format!("score num `{num}` not a u32: {err}"))?;
        let status = cells[3..].join(" | ");
        rows.push(AxisRow {
            axis,
            weight_pct,
            score_out_of_10,
            status,
        });
    }

    if rows.is_empty() {
        return Err("scorecard table not found or empty".into());
    }
    Ok(rows)
}

/// Strict-mode regenerate. Returns the rewritten markdown on success.
/// Returns Err with the over-claim diff on any `observed > target` axis.
pub(crate) fn regenerate_scorecard(
    markdown: &str,
    observed: &BTreeMap<String, u32>,
    targets: &BTreeMap<String, u32>,
    evidence_pointer: &str,
) -> Result<String, String> {
    let rows = parse_scorecard_table(markdown)?;
    let mut over_claims: Vec<String> = Vec::new();
    for row in &rows {
        if let Some(obs) = observed.get(&row.axis) {
            let tgt = targets.get(&row.axis).copied().unwrap_or(0);
            if *obs > tgt {
                over_claims.push(format!(
                    "axis `{}`: observed {obs} > target {tgt}",
                    row.axis
                ));
            }
        }
    }
    if !over_claims.is_empty() {
        return Err(format!(
            "scorecard regenerator refused — over-claim detected:\n  {}",
            over_claims.join("\n  ")
        ));
    }

    // Rewrite each known row's Score cell with the observed value (when
    // present); leave others untouched. Append `evidence_pointer` to the
    // status cell so audit can trace each lift to its NDJSON.
    let mut out = String::with_capacity(markdown.len() + 64);
    let mut in_table = false;
    let mut header_seen = false;
    for line in markdown.lines() {
        let trimmed = line.trim();
        if !in_table {
            if trimmed.starts_with("| Axis")
                && trimmed.contains("Weight")
                && trimmed.contains("Score")
            {
                in_table = true;
            }
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if !header_seen {
            if trimmed.starts_with("|") && trimmed.contains("---") {
                header_seen = true;
            }
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if trimmed.is_empty() || !trimmed.starts_with("|") {
            in_table = false;
            out.push_str(line);
            out.push('\n');
            continue;
        }
        let cells: Vec<&str> = trimmed
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        if cells.len() < 4 {
            out.push_str(line);
            out.push('\n');
            continue;
        }
        let axis = cells[0];
        if let Some(obs) = observed.get(axis) {
            let weight = cells[1];
            let status = cells[3..].join(" | ");
            let status = if status.contains(evidence_pointer) {
                status
            } else {
                format!("{status} | evidence: {evidence_pointer}")
            };
            let rewritten = format!("| {axis} | {weight} | {obs}/10 | {status} |");
            out.push_str(&rewritten);
            out.push('\n');
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    // Trim a single trailing newline if input did not end with one.
    if !markdown.ends_with('\n') && out.ends_with('\n') {
        out.pop();
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
# memd 10-Star Target

## 10-Star Composite Scorecard

| Axis | Weight | Score | Status |
|------|--------|-------|--------|
| Session continuity | 20% | 3/10 | A4 ledger survives compaction |
| Correction retention | 15% | 1/10 | mechanics exist |
| Procedural reuse | 15% | 1/10 | unreachable |
| Cross-harness continuity | 15% | 2/10 | 6 presets |
| Raw retrieval strength | 15% | 4/10 | search works |
| Token efficiency | 10% | 2/10 | budget enforced |
| Trust + provenance | 10% | 3/10 | explain works |

**Composite: 2.30/10**

## Next section
";

    fn targets() -> BTreeMap<String, u32> {
        BTreeMap::from([
            ("Session continuity".into(), 4),
            ("Correction retention".into(), 4),
            ("Procedural reuse".into(), 2),
            ("Cross-harness continuity".into(), 3),
            ("Raw retrieval strength".into(), 4),
            ("Token efficiency".into(), 4),
            ("Trust + provenance".into(), 3),
        ])
    }

    /// Test 9 — regenerator updates score column in place when observed ≤ targets.
    #[test]
    fn t9_scorecard_regenerator_updates_in_place() {
        let observed = BTreeMap::from([
            ("Session continuity".into(), 4u32),
            ("Correction retention".into(), 4u32),
            ("Cross-harness continuity".into(), 3u32),
            ("Raw retrieval strength".into(), 4u32),
            ("Token efficiency".into(), 4u32),
            ("Trust + provenance".into(), 3u32),
        ]);
        let out = regenerate_scorecard(
            SAMPLE,
            &observed,
            &targets(),
            "v4-proof-runs/2026-05-01T00-00Z.ndjson",
        )
        .expect("regenerator must succeed when observed ≤ targets");

        let parsed = parse_scorecard_table(&out).expect("output table re-parses");
        let by_axis: BTreeMap<&str, u32> = parsed
            .iter()
            .map(|r| (r.axis.as_str(), r.score_out_of_10))
            .collect();
        assert_eq!(by_axis["Session continuity"], 4);
        assert_eq!(by_axis["Correction retention"], 4);
        assert_eq!(by_axis["Cross-harness continuity"], 3);
        assert_eq!(by_axis["Raw retrieval strength"], 4);
        assert_eq!(by_axis["Token efficiency"], 4);
        assert_eq!(by_axis["Trust + provenance"], 3);
        // Untouched axis stays at original score.
        assert_eq!(by_axis["Procedural reuse"], 1);

        // Evidence pointer threaded into status of every updated axis.
        let updated_with_evidence = parsed
            .iter()
            .filter(|r| observed.contains_key(&r.axis))
            .all(|r| r.status.contains("v4-proof-runs/2026-05-01T00-00Z.ndjson"));
        assert!(
            updated_with_evidence,
            "every updated row carries evidence pointer"
        );

        // Sections outside the table preserved verbatim.
        assert!(out.contains("**Composite: 2.30/10**"));
        assert!(out.contains("## Next section"));
    }

    /// Test 10 — regenerator refuses + emits diff when any axis over-claims.
    #[test]
    fn t10_scorecard_regenerator_refuses_overclaim() {
        let observed = BTreeMap::from([
            ("Session continuity".into(), 4u32), // ok: target 4
            ("Token efficiency".into(), 7u32),   // over: target 4
        ]);
        let err = regenerate_scorecard(SAMPLE, &observed, &targets(), "v4-proof-runs/over.ndjson")
            .expect_err("over-claim must refuse");
        assert!(err.contains("scorecard regenerator refused"));
        assert!(err.contains("Token efficiency"));
        assert!(err.contains("observed 7 > target 4"));
        // Session-continuity axis was within budget — must NOT appear in the diff.
        assert!(!err.contains("Session continuity"));
    }

    /// G4.7 close gate — composite ≥ 3.45 floor on the live `MEMD-10-STAR.md`,
    /// with each axis bounded by milestone-union ceilings (V4 targets +
    /// already-banked V5 owner deltas — see C5's +1 bank documented in the
    /// 2026-04-25 note). Strict-mode over-claim refusal property is preserved
    /// for V4-attributed work; cross-axis bank flips that V5 already landed
    /// are recognized as not-an-over-claim.
    #[test]
    fn t13_v4_close_axes_match_milestone_targets() {
        let live = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../docs/verification/MEMD-10-STAR.md"),
        )
        .expect("read live MEMD-10-STAR.md");
        let rows = parse_scorecard_table(&live).expect("live scorecard parses");
        // V4 milestone targets per MILESTONE-v4.md axis table, plus
        // already-landed V5 axis deltas per 0.1.0-AXIS-OWNERSHIP:
        //   - Cross-harness +1 from V5 C5 (banked 2026-04-25, materializes
        //     atomically on V4 G4 close per MEMD-10-STAR.md line 93).
        //   - Procedural reuse +2 from V5 F5 live-fire (PR 2→4, gated by
        //     ten_star_writer live_fire_pass=1.0 metric).
        //   - Raw retrieval +2 from V5 RR aggregate (RR 4→6, gated by
        //     ten_star_writer rr_aggregate clause across A5/D5/E5/F5/G5).
        let mut ceilings = targets();
        ceilings.insert("Cross-harness continuity".into(), 4);
        ceilings.insert("Procedural reuse".into(), 4);
        ceilings.insert("Raw retrieval strength".into(), 6);
        for row in &rows {
            let ceiling = ceilings
                .get(&row.axis)
                .copied()
                .expect("every live axis has a milestone-union ceiling");
            assert!(
                row.score_out_of_10 <= ceiling,
                "V4 close over-claim: axis `{}` observed {} > ceiling {}",
                row.axis,
                row.score_out_of_10,
                ceiling,
            );
        }
        // Composite floor: weighted sum ≥ 3.45 (V4 close gate).
        let weights = std::collections::BTreeMap::from([
            ("Session continuity", 20u32),
            ("Correction retention", 15u32),
            ("Procedural reuse", 15u32),
            ("Cross-harness continuity", 15u32),
            ("Raw retrieval strength", 15u32),
            ("Token efficiency", 10u32),
            ("Trust + provenance", 10u32),
        ]);
        let mut weighted_sum_x100: u32 = 0;
        for row in &rows {
            let w = weights
                .get(row.axis.as_str())
                .copied()
                .expect("every axis has a weight");
            weighted_sum_x100 += row.score_out_of_10 * w;
        }
        // composite = sum(score · weight%) / 100 — gate ≥ 3.45 → 345.
        assert!(
            weighted_sum_x100 >= 345,
            "V4 close composite under gate: observed {}.{:02} < 3.45 floor",
            weighted_sum_x100 / 100,
            weighted_sum_x100 % 100,
        );
    }
}

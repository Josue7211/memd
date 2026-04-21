//! E3-D3: decay calibration harness.
//!
//! Deliberately offline. This module never touches sqlite — the point
//! is to sweep (half_life × reinforcement) against a synthetic event
//! stream before touching production parameters. Run via the unit tests
//! (`cargo test decay_calibration`) or lift into an ad-hoc binary when
//! ready to actually pick numbers.
//!
//! The simulator mirrors the production decay shape in
//! `SqliteStore::decay_entities`:
//!
//!   idle_days       = turn - last_access_turn
//!   over_days       = max(0, idle_days - inactive_days)
//!   rehearsal_scale = 1 / (ln(1 + rehearsal_count + 1) + 1)
//!   decay_step      = min(1, over_days / half_life_days) * max_decay * rehearsal_scale
//!   salience       -= decay_step
//!
//! Reinforcement adds `reinforcement_bump` to salience on every access
//! and increments rehearsal_count.

#[derive(Debug, Clone, Copy)]
pub struct DecayEvent {
    /// Turn index at which the item was accessed.
    pub turn: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct FixtureItem {
    pub initial_salience: f32,
    /// Inclusive turns at which this item is reinforced (rehearsed).
    pub access_turns: &'static [u64],
}

#[derive(Debug, Clone, Copy)]
pub struct CalibrationKnobs {
    pub half_life_days: u64,
    pub reinforcement_bump: f32,
    pub inactive_days: u64,
    pub max_decay: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct CalibrationPoint {
    pub half_life_days: u64,
    pub reinforcement_bump: f32,
    /// Mean salience across fixture at `measure_turn`.
    pub retention_at_measure: f32,
    /// Fraction of items that dropped to ~0 by `measure_turn`.
    pub zero_fraction: f32,
}

pub fn simulate(
    knobs: CalibrationKnobs,
    fixture: &[FixtureItem],
    measure_turn: u64,
) -> CalibrationPoint {
    let mut salience_sum = 0.0f32;
    let mut zero_count = 0usize;

    for item in fixture {
        let salience = simulate_one(knobs, item, measure_turn);
        salience_sum += salience;
        if salience <= 0.01 {
            zero_count += 1;
        }
    }

    let n = fixture.len().max(1) as f32;
    CalibrationPoint {
        half_life_days: knobs.half_life_days,
        reinforcement_bump: knobs.reinforcement_bump,
        retention_at_measure: salience_sum / n,
        zero_fraction: zero_count as f32 / n,
    }
}

fn simulate_one(knobs: CalibrationKnobs, item: &FixtureItem, measure_turn: u64) -> f32 {
    let mut salience = item.initial_salience;
    let mut last_access: u64 = 0;
    let mut rehearsal_count: u32 = 0;

    // Build the ordered event stream (accesses) + the final measurement.
    // We step one turn at a time to let decay accumulate correctly.
    for turn in 1..=measure_turn {
        let is_access = item.access_turns.contains(&turn);

        let idle = turn.saturating_sub(last_access);
        if idle >= knobs.inactive_days {
            let over = (idle - knobs.inactive_days) as f32;
            let rehearsal_scale = 1.0 / ((rehearsal_count as f32 + 1.0).ln_1p() + 1.0);
            let step =
                (over / knobs.half_life_days.max(1) as f32).min(1.0) * knobs.max_decay * rehearsal_scale;
            if step > 0.001 {
                salience = (salience - step).max(0.0);
            }
        }

        if is_access {
            salience = (salience + knobs.reinforcement_bump).min(1.0);
            rehearsal_count += 1;
            last_access = turn;
        }
    }

    salience
}

pub fn sweep(
    half_life_grid: &[u64],
    reinforcement_grid: &[f32],
    inactive_days: u64,
    max_decay: f32,
    fixture: &[FixtureItem],
    measure_turn: u64,
) -> Vec<CalibrationPoint> {
    let mut out = Vec::with_capacity(half_life_grid.len() * reinforcement_grid.len());
    for &hl in half_life_grid {
        for &bump in reinforcement_grid {
            out.push(simulate(
                CalibrationKnobs {
                    half_life_days: hl,
                    reinforcement_bump: bump,
                    inactive_days,
                    max_decay,
                },
                fixture,
                measure_turn,
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const NEVER_ACCESSED: FixtureItem = FixtureItem {
        initial_salience: 0.6,
        access_turns: &[],
    };
    const SPARSELY_ACCESSED: FixtureItem = FixtureItem {
        initial_salience: 0.6,
        access_turns: &[10, 20, 30],
    };
    const FIXTURE: [FixtureItem; 2] = [NEVER_ACCESSED, SPARSELY_ACCESSED];

    #[test]
    fn larger_half_life_means_higher_retention() {
        let short = simulate(
            CalibrationKnobs {
                half_life_days: 7,
                reinforcement_bump: 0.05,
                inactive_days: 7,
                max_decay: 0.12,
            },
            &FIXTURE,
            35,
        );
        let long = simulate(
            CalibrationKnobs {
                half_life_days: 30,
                reinforcement_bump: 0.05,
                inactive_days: 7,
                max_decay: 0.12,
            },
            &FIXTURE,
            35,
        );
        assert!(
            long.retention_at_measure > short.retention_at_measure,
            "longer half-life must retain more: long={:.3} short={:.3}",
            long.retention_at_measure,
            short.retention_at_measure
        );
    }

    #[test]
    fn reinforcement_bump_raises_retention_for_accessed_items() {
        let no_bump = simulate_one(
            CalibrationKnobs {
                half_life_days: 14,
                reinforcement_bump: 0.0,
                inactive_days: 7,
                max_decay: 0.12,
            },
            &SPARSELY_ACCESSED,
            35,
        );
        let bumped = simulate_one(
            CalibrationKnobs {
                half_life_days: 14,
                reinforcement_bump: 0.1,
                inactive_days: 7,
                max_decay: 0.12,
            },
            &SPARSELY_ACCESSED,
            35,
        );
        assert!(
            bumped > no_bump,
            "reinforcement must raise retention for accessed items"
        );
    }

    #[test]
    fn sweep_populates_every_grid_cell() {
        let grid = sweep(
            &[7, 14, 30],
            &[0.0, 0.05, 0.1],
            7,
            0.12,
            &FIXTURE,
            35,
        );
        assert_eq!(grid.len(), 9);
        for pt in &grid {
            assert!(pt.retention_at_measure.is_finite());
            assert!(pt.zero_fraction.is_finite());
            assert!(pt.zero_fraction <= 1.0);
        }
    }

    #[test]
    fn never_accessed_item_decays_more_than_sparsely_accessed() {
        let knobs = CalibrationKnobs {
            half_life_days: 14,
            reinforcement_bump: 0.05,
            inactive_days: 7,
            max_decay: 0.12,
        };
        let cold = simulate_one(knobs, &NEVER_ACCESSED, 35);
        let warm = simulate_one(knobs, &SPARSELY_ACCESSED, 35);
        assert!(
            warm > cold,
            "accessed item should retain more than never-accessed: warm={warm:.3} cold={cold:.3}"
        );
    }
}

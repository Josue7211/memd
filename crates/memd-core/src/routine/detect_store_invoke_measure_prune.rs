//! V10/C10 routine detect-store-invoke-measure-prune loop.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileTouch {
    pub turn_id: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoutineCandidate {
    pub routine_id: String,
    pub pattern: String,
    pub observed_count: usize,
    pub invoked_count: usize,
    pub success_count: usize,
    pub noise_count: usize,
    pub status: RoutineStatus,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoutineStatus {
    Candidate,
    Invoked,
    Pruned,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoutineMetrics {
    pub routine_candidates_observed: usize,
    pub invoked_count: usize,
    pub accuracy: f32,
    pub pruned_count: usize,
}

pub fn detect_routines(touches: &[FileTouch], min_observed: usize) -> Vec<RoutineCandidate> {
    let now = Utc::now();
    let mut counts = BTreeMap::<String, usize>::new();
    for touch in touches {
        if let Some(pattern) = path_pattern(&touch.path) {
            *counts.entry(pattern).or_default() += 1;
        }
    }

    counts
        .into_iter()
        .filter(|(_, count)| *count >= min_observed)
        .map(|(pattern, count)| RoutineCandidate {
            routine_id: format!("routine:{}", pattern.replace(['/', '*'], "_")),
            pattern,
            observed_count: count,
            invoked_count: 0,
            success_count: 0,
            noise_count: 0,
            status: RoutineStatus::Candidate,
            first_seen: now,
            last_seen: now,
        })
        .collect()
}

pub fn invoke_candidate(candidate: &mut RoutineCandidate) {
    if candidate.status != RoutineStatus::Pruned {
        candidate.invoked_count += 1;
        candidate.status = RoutineStatus::Invoked;
        candidate.last_seen = Utc::now();
    }
}

pub fn measure_invocation(candidate: &mut RoutineCandidate, success: bool) {
    if success {
        candidate.success_count += 1;
    } else {
        candidate.noise_count += 1;
    }
    candidate.last_seen = Utc::now();
}

pub fn prune_noisy(candidate: &mut RoutineCandidate, min_invocations: usize, max_noise_rate: f32) {
    if candidate.invoked_count < min_invocations {
        return;
    }
    let total = candidate.success_count + candidate.noise_count;
    if total == 0 {
        return;
    }
    let noise_rate = candidate.noise_count as f32 / total as f32;
    if noise_rate > max_noise_rate {
        candidate.status = RoutineStatus::Pruned;
    }
}

pub fn summarize_routines(candidates: &[RoutineCandidate]) -> RoutineMetrics {
    let invoked_count = candidates.iter().map(|item| item.invoked_count).sum();
    let success_count: usize = candidates.iter().map(|item| item.success_count).sum();
    let noise_count: usize = candidates.iter().map(|item| item.noise_count).sum();
    let total = success_count + noise_count;
    RoutineMetrics {
        routine_candidates_observed: candidates.iter().map(|item| item.observed_count).sum(),
        invoked_count,
        accuracy: if total == 0 {
            0.0
        } else {
            success_count as f32 / total as f32
        },
        pruned_count: candidates
            .iter()
            .filter(|item| item.status == RoutineStatus::Pruned)
            .count(),
    }
}

fn path_pattern(path: &str) -> Option<String> {
    let mut parts = path.split('/').collect::<Vec<_>>();
    if parts.len() < 2 {
        return None;
    }
    let file = parts.pop().unwrap_or_default();
    let ext = file.rsplit_once('.').map(|(_, ext)| ext).unwrap_or("*");
    let dir = parts.join("/");
    Some(format!("{dir}/*.{ext}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(turn_id: &str, path: &str) -> FileTouch {
        FileTouch {
            turn_id: turn_id.to_string(),
            path: path.to_string(),
        }
    }

    #[test]
    fn detects_repeated_file_touch_pattern() {
        let touches = vec![
            touch("t13", "migrations/001.sql"),
            touch("t14", "migrations/002.sql"),
            touch("t15", "migrations/003.sql"),
            touch("t16", "docs/readme.md"),
        ];
        let routines = detect_routines(&touches, 3);
        assert_eq!(routines.len(), 1);
        assert_eq!(routines[0].pattern, "migrations/*.sql");
        assert_eq!(routines[0].observed_count, 3);
    }

    #[test]
    fn invoke_measure_and_keep_accurate_routine() {
        let mut routine = detect_routines(
            &[
                touch("t13", "scripts/a.sh"),
                touch("t14", "scripts/b.sh"),
                touch("t15", "scripts/c.sh"),
            ],
            3,
        )
        .remove(0);
        for _ in 0..5 {
            invoke_candidate(&mut routine);
            measure_invocation(&mut routine, true);
        }
        prune_noisy(&mut routine, 5, 0.25);
        let metrics = summarize_routines(&[routine.clone()]);
        assert_eq!(routine.status, RoutineStatus::Invoked);
        assert_eq!(metrics.routine_candidates_observed, 3);
        assert_eq!(metrics.accuracy, 1.0);
    }

    #[test]
    fn prunes_noisy_routine_after_floor() {
        let mut routine = detect_routines(
            &[
                touch("t13", "crates/a.rs"),
                touch("t14", "crates/b.rs"),
                touch("t15", "crates/c.rs"),
            ],
            3,
        )
        .remove(0);
        for _ in 0..4 {
            invoke_candidate(&mut routine);
            measure_invocation(&mut routine, false);
        }
        prune_noisy(&mut routine, 4, 0.25);
        assert_eq!(routine.status, RoutineStatus::Pruned);
    }
}

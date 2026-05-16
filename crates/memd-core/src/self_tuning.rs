use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub const DEFAULT_MIN_TUNING_SAMPLES: usize = 3;
pub const DEFAULT_MIN_QUALITY_SCORE: f64 = 0.90;
pub const DEFAULT_MAX_QUALITY_REGRESSION: f64 = 0.0;
pub const DEFAULT_MAX_BUDGET_REGRESSION_PCT: f64 = 0.0;
pub const DEFAULT_TUNING_HEADROOM: f64 = 1.10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum CompilerMode {
    Static,
    #[default]
    Dynamic,
    SelfTuning,
}

impl CompilerMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Dynamic => "dynamic",
            Self::SelfTuning => "self_tuning",
        }
    }
}

impl FromStr for CompilerMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "static" => Ok(Self::Static),
            "dynamic" => Ok(Self::Dynamic),
            "self_tuning" | "self-tuning" | "selftuning" => Ok(Self::SelfTuning),
            other => Err(format!("unknown compiler mode '{other}'")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TuningTelemetryPoint {
    pub user_hash: String,
    pub harness: String,
    pub token_count: u64,
    pub budget_target: u64,
    pub quality_score: f64,
    #[serde(default = "default_min_quality_score")]
    pub baseline_quality_score: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct QualityGuard {
    pub min_samples: usize,
    pub min_quality_score: f64,
    pub max_quality_regression: f64,
    pub max_budget_regression_pct: f64,
    pub tuning_headroom: f64,
}

impl Default for QualityGuard {
    fn default() -> Self {
        Self {
            min_samples: DEFAULT_MIN_TUNING_SAMPLES,
            min_quality_score: DEFAULT_MIN_QUALITY_SCORE,
            max_quality_regression: DEFAULT_MAX_QUALITY_REGRESSION,
            max_budget_regression_pct: DEFAULT_MAX_BUDGET_REGRESSION_PCT,
            tuning_headroom: DEFAULT_TUNING_HEADROOM,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TuningProfile {
    pub user_hash: String,
    pub harness: String,
    pub mode: CompilerMode,
    pub sample_count: usize,
    pub baseline_budget: u64,
    pub tuned_budget: u64,
    pub average_tokens: f64,
    pub average_quality_score: f64,
    pub baseline_quality_score: f64,
    pub quality_delta: f64,
    pub token_savings_pct: f64,
    pub budget_regression_pct: f64,
    pub accepted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AbBenchResult {
    pub user_hash: String,
    pub harness: String,
    pub static_budget: u64,
    pub dynamic_budget: u64,
    pub self_tuning_budget: u64,
    pub token_savings_vs_dynamic_pct: f64,
    pub quality_delta_vs_dynamic: f64,
    pub accepted: bool,
}

pub fn build_tuning_profile(
    user_hash: &str,
    harness: &str,
    points: &[TuningTelemetryPoint],
    baseline_budget: u64,
    guard: QualityGuard,
) -> TuningProfile {
    let sample_count = points.len();
    let effective_baseline = baseline_budget.max(
        points
            .iter()
            .map(|point| point.budget_target)
            .max()
            .unwrap_or(1),
    );
    if sample_count == 0 {
        return TuningProfile {
            user_hash: user_hash.to_string(),
            harness: harness.to_string(),
            mode: CompilerMode::SelfTuning,
            sample_count,
            baseline_budget: effective_baseline,
            tuned_budget: effective_baseline,
            average_tokens: 0.0,
            average_quality_score: 0.0,
            baseline_quality_score: guard.min_quality_score,
            quality_delta: -guard.min_quality_score,
            token_savings_pct: 0.0,
            budget_regression_pct: 0.0,
            accepted: false,
            rejected_reason: Some("no_samples".to_string()),
        };
    }

    let average_tokens = points
        .iter()
        .map(|point| point.token_count as f64)
        .sum::<f64>()
        / sample_count as f64;
    let average_quality_score =
        points.iter().map(|point| point.quality_score).sum::<f64>() / sample_count as f64;
    let baseline_quality_score = points
        .iter()
        .map(|point| point.baseline_quality_score)
        .sum::<f64>()
        / sample_count as f64;
    let headroom = if guard.tuning_headroom.is_finite() && guard.tuning_headroom >= 1.0 {
        guard.tuning_headroom
    } else {
        DEFAULT_TUNING_HEADROOM
    };
    let candidate_budget = ((average_tokens * headroom).ceil() as u64)
        .max(1)
        .min(effective_baseline);
    let quality_delta = average_quality_score - baseline_quality_score;
    let token_savings_pct = percent_delta(effective_baseline, candidate_budget);
    let budget_regression_pct = percent_delta(candidate_budget, effective_baseline);

    let rejected_reason = if sample_count < guard.min_samples {
        Some("insufficient_samples")
    } else if average_quality_score < guard.min_quality_score {
        Some("quality_below_minimum")
    } else if quality_delta < -guard.max_quality_regression {
        Some("quality_regression")
    } else if budget_regression_pct > guard.max_budget_regression_pct {
        Some("budget_regression")
    } else if candidate_budget >= effective_baseline {
        Some("no_savings")
    } else {
        None
    };

    let accepted = rejected_reason.is_none();
    TuningProfile {
        user_hash: user_hash.to_string(),
        harness: harness.to_string(),
        mode: CompilerMode::SelfTuning,
        sample_count,
        baseline_budget: effective_baseline,
        tuned_budget: if accepted {
            candidate_budget
        } else {
            effective_baseline
        },
        average_tokens,
        average_quality_score,
        baseline_quality_score,
        quality_delta,
        token_savings_pct: if accepted { token_savings_pct } else { 0.0 },
        budget_regression_pct,
        accepted,
        rejected_reason: rejected_reason.map(str::to_string),
    }
}

pub fn select_compiler_budget(
    mode: CompilerMode,
    static_budget: u64,
    dynamic_budget: u64,
    profile: Option<&TuningProfile>,
) -> u64 {
    match mode {
        CompilerMode::Static => static_budget,
        CompilerMode::Dynamic => dynamic_budget,
        CompilerMode::SelfTuning => profile
            .filter(|profile| profile.accepted)
            .map(|profile| profile.tuned_budget)
            .unwrap_or(dynamic_budget),
    }
}

pub fn build_ab_bench_result(
    profile: &TuningProfile,
    static_budget: u64,
    dynamic_budget: u64,
) -> AbBenchResult {
    let self_tuning_budget = select_compiler_budget(
        CompilerMode::SelfTuning,
        static_budget,
        dynamic_budget,
        Some(profile),
    );
    AbBenchResult {
        user_hash: profile.user_hash.clone(),
        harness: profile.harness.clone(),
        static_budget,
        dynamic_budget,
        self_tuning_budget,
        token_savings_vs_dynamic_pct: percent_delta(dynamic_budget, self_tuning_budget),
        quality_delta_vs_dynamic: profile.quality_delta,
        accepted: profile.accepted,
    }
}

fn percent_delta(from: u64, to: u64) -> f64 {
    if from == 0 || to >= from {
        return 0.0;
    }
    ((from - to) as f64 / from as f64) * 100.0
}

fn default_min_quality_score() -> f64 {
    DEFAULT_MIN_QUALITY_SCORE
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(tokens: u64, quality_score: f64) -> TuningTelemetryPoint {
        TuningTelemetryPoint {
            user_hash: "user-1".to_string(),
            harness: "codex".to_string(),
            token_count: tokens,
            budget_target: 1500,
            quality_score,
            baseline_quality_score: 0.92,
        }
    }

    #[test]
    fn self_tuning_accepts_quality_preserving_savings() {
        let points = vec![point(900, 0.94), point(930, 0.95), point(960, 0.94)];
        let profile =
            build_tuning_profile("user-1", "codex", &points, 1500, QualityGuard::default());
        assert!(profile.accepted);
        assert!(profile.tuned_budget <= 1056);
        assert!(profile.token_savings_pct >= 20.0);
        assert!(profile.quality_delta >= 0.0);
    }

    #[test]
    fn quality_guard_rejects_regression() {
        let points = vec![point(800, 0.86), point(820, 0.87), point(830, 0.86)];
        let profile =
            build_tuning_profile("user-1", "codex", &points, 1500, QualityGuard::default());
        assert!(!profile.accepted);
        assert_eq!(profile.tuned_budget, 1500);
        assert_eq!(
            profile.rejected_reason.as_deref(),
            Some("quality_below_minimum")
        );
    }

    #[test]
    fn manual_mode_override_selects_static_or_dynamic() {
        let points = vec![point(900, 0.94), point(930, 0.95), point(960, 0.94)];
        let profile =
            build_tuning_profile("user-1", "codex", &points, 1500, QualityGuard::default());
        assert_eq!(
            select_compiler_budget(CompilerMode::Static, 4000, 1500, Some(&profile)),
            4000
        );
        assert_eq!(
            select_compiler_budget(CompilerMode::Dynamic, 4000, 1500, Some(&profile)),
            1500
        );
        assert_eq!(
            select_compiler_budget(CompilerMode::SelfTuning, 4000, 1500, Some(&profile)),
            profile.tuned_budget
        );
    }
}

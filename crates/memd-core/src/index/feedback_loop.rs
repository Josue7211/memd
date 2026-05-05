//! V10/D10 retrieval-feedback loop.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetrievalFeedbackEvent {
    pub item_id: String,
    pub route: String,
    pub useful: bool,
    pub noisy: bool,
    pub score_delta: f32,
    pub observed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeedbackAggregate {
    pub route: String,
    pub event_count: usize,
    pub useful_count: usize,
    pub noisy_count: usize,
    pub mean_score_delta: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeightUpdate {
    pub route: String,
    pub previous_weight: f32,
    pub new_weight: f32,
    pub delta: f32,
}

pub fn aggregate_feedback_30day(
    events: &[RetrievalFeedbackEvent],
    now: DateTime<Utc>,
) -> Vec<FeedbackAggregate> {
    let cutoff = now - Duration::days(30);
    let mut grouped = BTreeMap::<String, Vec<&RetrievalFeedbackEvent>>::new();
    for event in events {
        if event.observed_at >= cutoff && event.observed_at <= now {
            grouped.entry(event.route.clone()).or_default().push(event);
        }
    }

    grouped
        .into_iter()
        .map(|(route, events)| {
            let event_count = events.len();
            let useful_count = events.iter().filter(|event| event.useful).count();
            let noisy_count = events.iter().filter(|event| event.noisy).count();
            let mean_score_delta =
                events.iter().map(|event| event.score_delta).sum::<f32>() / event_count as f32;
            FeedbackAggregate {
                route,
                event_count,
                useful_count,
                noisy_count,
                mean_score_delta,
            }
        })
        .collect()
}

pub fn apply_weight_updates(
    aggregates: &[FeedbackAggregate],
    current_weights: &BTreeMap<String, f32>,
    max_abs_delta: f32,
) -> Vec<WeightUpdate> {
    aggregates
        .iter()
        .map(|aggregate| {
            let previous_weight = current_weights
                .get(&aggregate.route)
                .copied()
                .unwrap_or(1.0);
            let quality = if aggregate.event_count == 0 {
                0.0
            } else {
                (aggregate.useful_count as f32 - aggregate.noisy_count as f32)
                    / aggregate.event_count as f32
            };
            let raw_delta = (quality * 0.04) + aggregate.mean_score_delta.clamp(-0.02, 0.02);
            let delta = raw_delta.clamp(-max_abs_delta, max_abs_delta);
            let new_weight = (previous_weight + delta).clamp(0.1, 3.0);
            WeightUpdate {
                route: aggregate.route.clone(),
                previous_weight,
                new_weight,
                delta: new_weight - previous_weight,
            }
        })
        .collect()
}

pub fn all_deltas_within_limit(updates: &[WeightUpdate], max_abs_delta: f32) -> bool {
    updates
        .iter()
        .all(|update| update.delta.abs() <= max_abs_delta + f32::EPSILON)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(route: &str, useful: bool, noisy: bool, days_old: i64) -> RetrievalFeedbackEvent {
        RetrievalFeedbackEvent {
            item_id: format!("{route}-{days_old}"),
            route: route.to_string(),
            useful,
            noisy,
            score_delta: if useful { 0.02 } else { -0.02 },
            observed_at: Utc::now() - Duration::days(days_old),
        }
    }

    #[test]
    fn aggregates_only_30day_window() {
        let now = Utc::now();
        let events = vec![
            event("wake", true, false, 1),
            event("wake", false, true, 2),
            event("wake", true, false, 31),
            event("targeted", true, false, 3),
        ];
        let aggregates = aggregate_feedback_30day(&events, now);
        let wake = aggregates.iter().find(|item| item.route == "wake").unwrap();
        assert_eq!(wake.event_count, 2);
        assert_eq!(wake.useful_count, 1);
        assert_eq!(wake.noisy_count, 1);
    }

    #[test]
    fn caps_weight_delta_at_five_percent() {
        let aggregates = vec![FeedbackAggregate {
            route: "targeted".into(),
            event_count: 10,
            useful_count: 10,
            noisy_count: 0,
            mean_score_delta: 0.50,
        }];
        let weights = BTreeMap::from([("targeted".to_string(), 1.0)]);
        let updates = apply_weight_updates(&aggregates, &weights, 0.05);
        assert_eq!(updates.len(), 1);
        assert!(all_deltas_within_limit(&updates, 0.05));
        assert!((updates[0].delta - 0.05).abs() < 0.0001);
    }

    #[test]
    fn penalizes_noisy_route() {
        let aggregates = vec![FeedbackAggregate {
            route: "resume".into(),
            event_count: 4,
            useful_count: 0,
            noisy_count: 4,
            mean_score_delta: -0.02,
        }];
        let weights = BTreeMap::from([("resume".to_string(), 1.0)]);
        let updates = apply_weight_updates(&aggregates, &weights, 0.05);
        assert!(updates[0].new_weight < 1.0);
    }
}

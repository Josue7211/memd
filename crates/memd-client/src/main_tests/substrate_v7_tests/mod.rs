//! V7 substrate tests — correction behavior-change across a session boundary.

use crate::benchmark::substrate::correction_behavior::{
    InProcessV7BehaviorBackend, OriginalValueV7Backend, V7Event, V7RunConfig, corrected_value,
    run_v7_correction_behavior_in_process, run_v7_correction_behavior_with_backend,
};
use crate::benchmark::substrate::fixtures::{KindMix, generate_corpus};
use std::path::PathBuf;
use tempfile::tempdir;

fn small_config(results_dir: PathBuf) -> V7RunConfig {
    let mut cfg = V7RunConfig::default_with_results_dir(results_dir);
    cfg.fact_count = 5;
    cfg.kind_mix = KindMix::default();
    cfg
}

#[test]
fn v7_correction_behavior_happy_path() {
    let dir = tempdir().unwrap();
    let cfg = small_config(dir.path().to_path_buf());
    let outcome = run_v7_correction_behavior_in_process(&cfg).unwrap();

    assert!(outcome.overall_pass);
    assert_eq!(outcome.next_session_behavior_rate, 1.0);
    assert_eq!(outcome.chain_completeness_rate, 1.0);
    assert_eq!(outcome.rollback_behavior_rate, 1.0);
    assert_eq!(outcome.rollback_chain_completeness_rate, 1.0);
    assert!(outcome.ndjson_path.exists());

    let body = std::fs::read_to_string(&outcome.ndjson_path).unwrap();
    assert!(body.contains("\"suite\":\"correction-behavior-change\""));
    assert!(body.contains("\"recall_at_1\":1.0"));
    assert!(body.contains("\"recall_at_3\":1.0"));
}

#[test]
fn v7_original_value_backend_fails_behavior_gate() {
    let dir = tempdir().unwrap();
    let cfg = small_config(dir.path().to_path_buf());
    let outcome =
        run_v7_correction_behavior_with_backend(&cfg, &OriginalValueV7Backend::default()).unwrap();

    assert!(!outcome.overall_pass);
    assert_eq!(outcome.next_session_behavior_rate, 0.0);
    assert_eq!(outcome.chain_completeness_rate, 0.0);
    assert_eq!(outcome.rollback_chain_completeness_rate, 0.0);
    assert!(!outcome.records[0].pass);
}

#[test]
fn v7_s2_query_does_not_repeat_corrected_value() {
    let dir = tempdir().unwrap();
    let cfg = small_config(dir.path().to_path_buf());
    let backend = InProcessV7BehaviorBackend::default();
    let facts = generate_corpus(cfg.seed, cfg.fact_count, &cfg.kind_mix);

    let outcome = run_v7_correction_behavior_with_backend(&cfg, &backend).unwrap();
    assert!(outcome.overall_pass);

    let corrected_values: Vec<String> = facts.iter().map(corrected_value).collect();
    let query_events: Vec<V7Event> = backend
        .events()
        .into_iter()
        .filter(|e| matches!(e, V7Event::BehaviorQuery { session, .. } if session.contains("-s2")))
        .collect();

    assert_eq!(query_events.len(), facts.len());
    for event in query_events {
        if let V7Event::BehaviorQuery {
            subject, predicate, ..
        } = event
        {
            for corrected in &corrected_values {
                assert!(
                    !subject.contains(corrected) && !predicate.contains(corrected),
                    "S2 behavior query leaked corrected value {corrected:?}"
                );
            }
        }
    }
}

#[test]
fn v7_writes_results_dir_tree() {
    let dir = tempdir().unwrap();
    let cfg = small_config(dir.path().to_path_buf());
    let outcome = run_v7_correction_behavior_in_process(&cfg).unwrap();

    assert!(outcome.ndjson_path.exists());
    let runs_jsonl = dir.path().join("runs.jsonl");
    assert!(runs_jsonl.exists());
    let runs_body = std::fs::read_to_string(&runs_jsonl).unwrap();
    assert!(runs_body.contains("correction-behavior-change"));
    assert!(runs_body.contains("chain_completeness_rate"));
    assert!(runs_body.contains("rollback_chain_completeness_rate"));
}

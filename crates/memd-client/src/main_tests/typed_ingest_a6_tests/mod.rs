//! A6 integration tests — `--typed-ingest=episodic` round-trip across
//! the four public benches (LME, LoCoMo, MemBench, ConvoMem). Tests
//! 1–10 per `phase-a6-plan.md` §4. Bodies land in tasks A6.2–A6.8.

#[allow(unused_imports)]
use crate::benchmark::typed_ingest::{EpisodicAdapter, EpisodicProvenance};

use std::io::Write as _;

use crate::benchmark::typed_ingest::bench_loaders::lme::LmeAdapter;
use crate::benchmark::typed_ingest::bench_loaders::locomo::LocomoAdapter;

/// A6 Test 1 — `bench_loader_lme_yields_typed_episodic`.
/// LME loader yields `EpisodicTurn` records with bench_id="longmemeval"
/// and fully-populated provenance (session_id, turn_index, speaker,
/// source_hash, captured_at).
#[test]
fn bench_loader_lme_yields_typed_episodic() {
    let fixture = serde_json::json!([
        {
            "question_id": "q0",
            "question_type": "single-session-user",
            "question": "what did I say about the river puzzle?",
            "question_date": "2024-01-01",
            "answer": "irrelevant",
            "answer_session_ids": ["sess_a"],
            "haystack_dates": ["2024-01-01", "2024-01-02"],
            "haystack_session_ids": ["sess_a", "sess_b"],
            "haystack_sessions": [
                [
                    {"role": "user", "content": "hello"},
                    {"role": "assistant", "content": "hi back"}
                ],
                [
                    {"role": "user", "content": "second session"}
                ]
            ]
        }
    ]);

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    tmp.write_all(fixture.to_string().as_bytes()).unwrap();

    let mut adapter = LmeAdapter::from_path(tmp.path()).expect("loader");
    assert_eq!(adapter.bench_id(), "longmemeval");

    let mut turns = Vec::new();
    while let Some(t) = adapter.next_turn() {
        turns.push(t);
    }

    assert_eq!(turns.len(), 3, "expect 2+1 turns across two sessions");

    let t0 = &turns[0];
    assert_eq!(t0.content, "hello");
    assert_eq!(t0.provenance.bench_id, "longmemeval");
    assert_eq!(t0.provenance.session_id, "sess_a");
    assert_eq!(t0.provenance.turn_index, 0);
    assert_eq!(t0.provenance.speaker, "user");
    assert_eq!(t0.provenance.captured_at, "2024-01-01");
    assert_eq!(t0.provenance.source_hash.len(), 64, "sha256 hex");
    assert!(
        t0.provenance.source_hash.chars().all(|c| c.is_ascii_hexdigit()),
        "source_hash must be hex"
    );

    let t1 = &turns[1];
    assert_eq!(t1.provenance.turn_index, 1);
    assert_eq!(t1.provenance.speaker, "assistant");

    let t2 = &turns[2];
    assert_eq!(t2.provenance.session_id, "sess_b");
    assert_eq!(t2.provenance.turn_index, 0);
    assert_eq!(t2.provenance.captured_at, "2024-01-02");

    // source_hash deterministic + unique per turn
    assert_ne!(t0.provenance.source_hash, t1.provenance.source_hash);
}

/// A6 Test 2 — `bench_loader_locomo_yields_typed_episodic`.
/// LoCoMo loader walks `conversation.session_N` keys in numeric order,
/// yields `EpisodicTurn` per `{speaker, dia_id, text}`, with `session_id`
/// scoped by `sample_id` and `captured_at` from `session_N_date_time`.
#[test]
fn bench_loader_locomo_yields_typed_episodic() {
    let fixture = serde_json::json!([
        {
            "sample_id": "loco_0",
            "conversation": {
                "speaker_a": "Alice",
                "speaker_b": "Bob",
                "session_1_date_time": "1:00 pm on 1 Jan, 2024",
                "session_1": [
                    {"speaker": "Alice", "dia_id": "D1:1", "text": "hi"},
                    {"speaker": "Bob",   "dia_id": "D1:2", "text": "hello"}
                ],
                "session_2_date_time": "2:00 pm on 2 Jan, 2024",
                "session_2": [
                    {"speaker": "Alice", "dia_id": "D2:1", "text": "later"}
                ]
            },
            "qa": [],
            "event_summary": {},
            "observation": {},
            "session_summary": {}
        }
    ]);
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    tmp.write_all(fixture.to_string().as_bytes()).unwrap();

    let mut adapter = LocomoAdapter::from_path(tmp.path()).expect("loader");
    assert_eq!(adapter.bench_id(), "locomo");

    let mut turns = Vec::new();
    while let Some(t) = adapter.next_turn() {
        turns.push(t);
    }
    assert_eq!(turns.len(), 3);

    assert_eq!(turns[0].content, "hi");
    assert_eq!(turns[0].provenance.session_id, "loco_0::session_1");
    assert_eq!(turns[0].provenance.turn_index, 0);
    assert_eq!(turns[0].provenance.speaker, "Alice");
    assert_eq!(turns[0].provenance.captured_at, "1:00 pm on 1 Jan, 2024");
    assert_eq!(turns[0].provenance.source_hash.len(), 64);

    assert_eq!(turns[1].provenance.turn_index, 1);
    assert_eq!(turns[1].provenance.speaker, "Bob");

    assert_eq!(turns[2].provenance.session_id, "loco_0::session_2");
    assert_eq!(turns[2].provenance.captured_at, "2:00 pm on 2 Jan, 2024");
}

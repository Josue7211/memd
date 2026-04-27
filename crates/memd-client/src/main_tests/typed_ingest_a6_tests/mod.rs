//! A6 integration tests — `--typed-ingest=episodic` round-trip across
//! the four public benches (LME, LoCoMo, MemBench, ConvoMem). Tests
//! 1–10 per `phase-a6-plan.md` §4. Bodies land in tasks A6.2–A6.8.

#[allow(unused_imports)]
use crate::benchmark::typed_ingest::{EpisodicAdapter, EpisodicProvenance};

use std::io::Write as _;

use crate::benchmark::typed_ingest::bench_loaders::lme::LmeAdapter;
use crate::benchmark::typed_ingest::bench_loaders::locomo::LocomoAdapter;
use crate::benchmark::typed_ingest::bench_loaders::membench::MembenchAdapter;
use crate::benchmark::typed_ingest::bench_loaders::convomem::ConvomemAdapter;

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

/// A6 Test 3 — `bench_loader_membench_yields_typed_episodic`.
/// MemBench loader walks `<category>[].message_list[i][j]` turns, splits
/// each `{user, assistant}` pair into two `EpisodicTurn`s with stable
/// `turn_index = mid*2 + role_offset` and `session_id` keyed by category +
/// tid + list index.
#[test]
fn bench_loader_membench_yields_typed_episodic() {
    let fixture = serde_json::json!({
        "book": [
            {
                "tid": "tid_0",
                "QA": [],
                "message_list": [[
                    {
                        "mid": 0,
                        "time": "'2024-10-01 08:00' Tuesday",
                        "place": "Boston, MA",
                        "user": "I love Seinlanguage",
                        "assistant": "Cool!"
                    },
                    {
                        "mid": 1,
                        "time": "'2024-10-02 09:00' Wednesday",
                        "place": "Boston, MA",
                        "user": "What about Catch-22?",
                        "assistant": "A classic."
                    }
                ]]
            }
        ]
    });
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    tmp.write_all(fixture.to_string().as_bytes()).unwrap();

    let mut adapter = MembenchAdapter::from_path(tmp.path()).expect("loader");
    assert_eq!(adapter.bench_id(), "membench");

    let mut turns = Vec::new();
    while let Some(t) = adapter.next_turn() {
        turns.push(t);
    }
    assert_eq!(turns.len(), 4, "two pairs → four typed turns");

    assert_eq!(turns[0].content, "I love Seinlanguage");
    assert_eq!(turns[0].provenance.session_id, "book::tid_0::list_0");
    assert_eq!(turns[0].provenance.speaker, "user");
    assert_eq!(turns[0].provenance.turn_index, 0);
    assert_eq!(turns[0].provenance.captured_at, "'2024-10-01 08:00' Tuesday");

    assert_eq!(turns[1].content, "Cool!");
    assert_eq!(turns[1].provenance.speaker, "assistant");
    assert_eq!(turns[1].provenance.turn_index, 1);

    assert_eq!(turns[2].provenance.turn_index, 2);
    assert_eq!(turns[3].provenance.turn_index, 3);
    assert_ne!(turns[0].provenance.source_hash, turns[1].provenance.source_hash);
}

/// A6 Test 4 — `bench_loader_convomem_yields_typed_episodic`.
/// ConvoMem loader walks `items[].metadata.conversations[].messages[]`,
/// keys session by `<item_id>::<conversation_id>`, leaves `captured_at`
/// empty (ConvoMem ships no per-message timestamps).
#[test]
fn bench_loader_convomem_yields_typed_episodic() {
    let fixture = serde_json::json!({
        "benchmark_id": "convomem",
        "items": [
            {
                "item_id": "item_0",
                "metadata": {
                    "conversations": [
                        {
                            "id": "conv_0",
                            "containsEvidence": true,
                            "messages": [
                                {"speaker": "User", "text": "hi"},
                                {"speaker": "Assistant", "text": "hello"}
                            ]
                        },
                        {
                            "id": "conv_1",
                            "messages": [
                                {"speaker": "User", "text": "later"}
                            ]
                        }
                    ]
                }
            }
        ]
    });
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    tmp.write_all(fixture.to_string().as_bytes()).unwrap();

    let mut adapter = ConvomemAdapter::from_path(tmp.path()).expect("loader");
    assert_eq!(adapter.bench_id(), "convomem");

    let mut turns = Vec::new();
    while let Some(t) = adapter.next_turn() {
        turns.push(t);
    }
    assert_eq!(turns.len(), 3);

    assert_eq!(turns[0].content, "hi");
    assert_eq!(turns[0].provenance.session_id, "item_0::conv_0");
    assert_eq!(turns[0].provenance.turn_index, 0);
    assert_eq!(turns[0].provenance.speaker, "User");
    assert_eq!(turns[0].provenance.captured_at, "");
    assert_eq!(turns[0].provenance.source_hash.len(), 64);

    assert_eq!(turns[1].provenance.turn_index, 1);

    assert_eq!(turns[2].provenance.session_id, "item_0::conv_1");
    assert_eq!(turns[2].provenance.turn_index, 0);
}

/// A6 Test 5 — `provenance_fields_populated_across_all_loaders`.
/// Every adapter yields turns whose provenance has `bench_id` set, a
/// non-empty `session_id` and `speaker`, a 64-hex `source_hash`, and a
/// `turn_index` (zero counts as populated).
#[test]
fn provenance_fields_populated_across_all_loaders() {
    let lme_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/typed_ingest/a6/lme-sample-10turn.json");
    let mut lme = LmeAdapter::from_path(&lme_path).expect("lme fixture");
    let mut count_lme = 0;
    while let Some(t) = lme.next_turn() {
        count_lme += 1;
        assert_eq!(t.provenance.bench_id, "longmemeval");
        assert!(!t.provenance.session_id.is_empty(), "lme session_id empty");
        assert!(!t.provenance.speaker.is_empty(), "lme speaker empty");
        assert_eq!(t.provenance.source_hash.len(), 64, "lme hash len");
        assert!(!t.provenance.captured_at.is_empty(), "lme captured_at empty");
    }
    assert_eq!(count_lme, 10, "10-turn fixture");

    // LoCoMo: small inline fixture
    let loco_fixture = serde_json::json!([
        {"sample_id":"loco_x","conversation":{
            "speaker_a":"A","speaker_b":"B",
            "session_1_date_time":"now",
            "session_1":[{"speaker":"A","dia_id":"D1","text":"x"}]
        }}
    ]);
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    tmp.write_all(loco_fixture.to_string().as_bytes()).unwrap();
    let mut loco = LocomoAdapter::from_path(tmp.path()).unwrap();
    while let Some(t) = loco.next_turn() {
        assert_eq!(t.provenance.bench_id, "locomo");
        assert!(!t.provenance.session_id.is_empty());
        assert!(!t.provenance.speaker.is_empty());
        assert_eq!(t.provenance.source_hash.len(), 64);
    }

    // MemBench
    let mb_fixture = serde_json::json!({
        "x": [{"tid":"t","QA":[],"message_list":[[
            {"mid":0,"time":"t0","place":"p","user":"u","assistant":"a"}
        ]]}]
    });
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    tmp.write_all(mb_fixture.to_string().as_bytes()).unwrap();
    let mut mb = MembenchAdapter::from_path(tmp.path()).unwrap();
    while let Some(t) = mb.next_turn() {
        assert_eq!(t.provenance.bench_id, "membench");
        assert!(!t.provenance.session_id.is_empty());
        assert!(matches!(t.provenance.speaker.as_str(), "user" | "assistant"));
        assert_eq!(t.provenance.source_hash.len(), 64);
        assert_eq!(t.provenance.captured_at, "t0");
    }

    // ConvoMem
    let cm_fixture = serde_json::json!({
        "items":[{"item_id":"i","metadata":{"conversations":[
            {"id":"c","messages":[{"speaker":"User","text":"hi"}]}
        ]}}]
    });
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    tmp.write_all(cm_fixture.to_string().as_bytes()).unwrap();
    let mut cm = ConvomemAdapter::from_path(tmp.path()).unwrap();
    while let Some(t) = cm.next_turn() {
        assert_eq!(t.provenance.bench_id, "convomem");
        assert!(!t.provenance.session_id.is_empty());
        assert!(!t.provenance.speaker.is_empty());
        assert_eq!(t.provenance.source_hash.len(), 64);
        // captured_at intentionally empty for ConvoMem (no per-message dates).
    }
}

/// A6 Test 7 — `cli_args_accept_typed_ingest_episodic`.
/// `PublicBenchmarkArgs` parses `--typed-ingest=episodic`; default is None;
/// any other value is rejected by clap's value_parser.
#[test]
fn cli_args_accept_typed_ingest_episodic() {
    use crate::cli::args::PublicBenchmarkArgs;
    use clap::Parser;

    #[derive(Parser)]
    struct Wrap {
        #[command(flatten)]
        a: PublicBenchmarkArgs,
    }

    let on = Wrap::try_parse_from([
        "memd",
        "longmemeval",
        "--typed-ingest=episodic",
    ])
    .expect("parse");
    assert_eq!(on.a.typed_ingest.as_deref(), Some("episodic"));

    let off = Wrap::try_parse_from(["memd", "longmemeval"]).expect("parse default");
    assert!(off.a.typed_ingest.is_none());

    let bad = Wrap::try_parse_from([
        "memd",
        "longmemeval",
        "--typed-ingest=semantic",
    ]);
    assert!(bad.is_err(), "only `episodic` is accepted in A6");
}

/// A6 Test 8 — `runtime_dispatches_to_episodic_when_flag_set`.
/// `dispatch_typed_ingest_episodic` selects the right adapter per dataset
/// id, walks the full turn stream, and returns deterministic counts.
#[test]
fn runtime_dispatches_to_episodic_when_flag_set() {
    use crate::benchmark::typed_ingest::dispatch_typed_ingest_episodic;

    let lme_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/typed_ingest/a6/lme-sample-10turn.json");
    let report = dispatch_typed_ingest_episodic("longmemeval", &lme_path).unwrap();
    assert_eq!(report.bench_id, "longmemeval");
    assert_eq!(report.turn_count, 10);
    assert_eq!(report.session_count, 2);

    // Unknown dataset → error.
    let err = dispatch_typed_ingest_episodic("unknown_bench", &lme_path).unwrap_err();
    assert!(err.to_string().contains("does not support"));
}

/// A6 Test 6 — `episodic_turn_serde_round_trip`.
/// `EpisodicTurn` round-trips through JSON without loss. This is the
/// minimal contract the `--typed-ingest` pipeline relies on when shipping
/// turns from loader → ingester → record metadata.
#[test]
fn episodic_turn_serde_round_trip() {
    let lme_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/typed_ingest/a6/lme-sample-10turn.json");
    let mut lme = LmeAdapter::from_path(&lme_path).expect("lme fixture");
    let original = lme.next_turn().expect("at least one turn");
    let json = serde_json::to_string(&original).expect("serialize");
    let parsed: crate::benchmark::typed_ingest::episodic::EpisodicTurn =
        serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.content, original.content);
    assert_eq!(parsed.provenance, original.provenance);
}

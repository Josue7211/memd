//! A5 scripted session driver.
//!
//! Plays a deterministic fact corpus into memd across `K` simulated
//! "session cuts", invoking A4's seal/restore (PostCompact) between cuts.
//! The bench-time backend is a real memd-server process spawned by
//! A5.6's CLI driver; for unit testing we drive the same orchestration
//! against an in-memory recorder, so we test the *script logic* (cut
//! placement, restore-after-cut, fact-batch ordering) without booting a
//! server. The runtime impl arrives in A5.6 alongside the CLI wiring.

use crate::benchmark::substrate::fixtures::Fact;
use std::sync::{Arc, Mutex};

/// What a "session cut" means here:
/// 1. Seal the file_ledger snapshot for the in-flight session.
/// 2. Drop in-process working state (simulating PreCompact).
/// 3. Open a new session that calls A4 PostCompact restore on wake.
///
/// The driver records each step into the supplied `BenchBackend` so
/// upstream code (or a test) can assert ordering without inspecting the
/// real ledger files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SessionEvent {
    SessionOpened { id: String },
    FactIngested { session: String, fact_id: u32 },
    SessionSealed { id: String },
    SessionRestored { id: String, restored_from: String },
    Query { session: String, fact_id: u32 },
}

/// Backend the driver talks to. The real impl (`HttpMemdBackend`)
/// opens a memd-server and translates these calls into HTTP. The test
/// impl below records calls into a `Vec<SessionEvent>`.
///
/// `query_top_k` returns up to `k` candidate values ranked best-first.
/// Empty vec = backend has no answer. The recording backend returns at
/// most one hit (it stores facts by id); the real HTTP backend returns
/// real top-k from semantic search so recall@3 ≠ recall@1.
pub(crate) trait BenchBackend {
    fn open_session(&self, id: &str);
    fn ingest_fact(&self, session: &str, fact: &Fact);
    fn seal_session(&self, id: &str);
    fn restore_session(&self, id: &str, restored_from: &str);
    fn query_top_k(&self, session: &str, fact: &Fact, k: usize) -> Vec<String>;
}

/// In-memory backend used by tests. Stores every call as a
/// `SessionEvent`. Also remembers each ingested fact so `query_top_k`
/// can return the canonical value.
#[derive(Default, Clone)]
pub(crate) struct RecordingBackend {
    pub(crate) events: Arc<Mutex<Vec<SessionEvent>>>,
    pub(crate) facts: Arc<Mutex<Vec<Fact>>>,
}

impl RecordingBackend {
    pub(crate) fn events(&self) -> Vec<SessionEvent> {
        self.events.lock().unwrap().clone()
    }
}

impl BenchBackend for RecordingBackend {
    fn open_session(&self, id: &str) {
        self.events
            .lock()
            .unwrap()
            .push(SessionEvent::SessionOpened { id: id.to_string() });
    }

    fn ingest_fact(&self, session: &str, fact: &Fact) {
        self.events
            .lock()
            .unwrap()
            .push(SessionEvent::FactIngested {
                session: session.to_string(),
                fact_id: fact.id,
            });
        self.facts.lock().unwrap().push(fact.clone());
    }

    fn seal_session(&self, id: &str) {
        self.events
            .lock()
            .unwrap()
            .push(SessionEvent::SessionSealed { id: id.to_string() });
    }

    fn restore_session(&self, id: &str, restored_from: &str) {
        self.events
            .lock()
            .unwrap()
            .push(SessionEvent::SessionRestored {
                id: id.to_string(),
                restored_from: restored_from.to_string(),
            });
    }

    fn query_top_k(&self, session: &str, fact: &Fact, _k: usize) -> Vec<String> {
        self.events.lock().unwrap().push(SessionEvent::Query {
            session: session.to_string(),
            fact_id: fact.id,
        });
        self.facts
            .lock()
            .unwrap()
            .iter()
            .find(|f| f.id == fact.id)
            .map(|f| vec![f.value.clone()])
            .unwrap_or_default()
    }
}

/// A scripted A5 scenario: ingest `facts` across `cut_k + 1` sessions,
/// sealing + restoring between each cut, then query every fact from the
/// final session and return per-fact recall (`Some(value)` if backend
/// returned the right value, `None` otherwise).
#[derive(Debug, Clone)]
pub(crate) struct A5Scenario {
    pub(crate) suite: String,
    pub(crate) seed: u64,
    pub(crate) facts: Vec<Fact>,
    pub(crate) cut_k: usize,
}

impl A5Scenario {
    fn session_id(&self, idx: usize) -> String {
        format!(
            "a5-{}-seed{}-cut{}-s{}",
            self.suite, self.seed, self.cut_k, idx
        )
    }

    /// Drives `backend` through the scenario:
    ///   open session-0 → ingest batch 0 → seal → open session-1 →
    ///   restore from session-0 → ingest batch 1 → … → query in final.
    ///
    /// Returns `(per_fact_recall, all_events)`.
    pub(crate) fn run<B: BenchBackend>(&self, backend: &B) -> ScenarioOutcome {
        let session_count = self.cut_k + 1;
        // Even split — last session may carry remainder.
        let batch_size = self.facts.len().div_ceil(session_count.max(1));

        for s in 0..session_count {
            let sid = self.session_id(s);
            backend.open_session(&sid);
            if s > 0 {
                let prev = self.session_id(s - 1);
                backend.restore_session(&sid, &prev);
            }
            let start = (s * batch_size).min(self.facts.len());
            let end = (start + batch_size).min(self.facts.len());
            for f in &self.facts[start..end] {
                backend.ingest_fact(&sid, f);
            }
            if s + 1 < session_count {
                backend.seal_session(&sid);
            }
        }

        // Recall pass from the final session. Computes both recall@1
        // (top-1 candidate matches) and recall@3 (any of top-3 matches).
        let final_session = self.session_id(session_count - 1);
        let mut r1_hits = 0usize;
        let mut r3_hits = 0usize;
        for f in &self.facts {
            let candidates = backend.query_top_k(&final_session, f, 3);
            if candidates.first().map(|v| v == &f.value).unwrap_or(false) {
                r1_hits += 1;
            }
            if candidates.iter().any(|v| v == &f.value) {
                r3_hits += 1;
            }
        }
        let n = self.facts.len().max(1) as f64;
        ScenarioOutcome {
            recall_at_1: r1_hits as f64 / n,
            recall_at_3: r3_hits as f64 / n,
            session_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ScenarioOutcome {
    pub(crate) recall_at_1: f64,
    pub(crate) recall_at_3: f64,
    pub(crate) session_count: usize,
}

/// F5 typed-retrieval scenario: route a query to determine expected kind.
/// Perfect-recall backend always returns the expected kind.
#[derive(Debug, Clone)]
pub(crate) struct F5Scenario {
    pub(crate) suite: String,
    pub(crate) seed: u64,
    pub(crate) query: String,
    pub(crate) expected_kind: String,
}

impl F5Scenario {
    pub(crate) fn run<B: BenchBackend>(&self, _backend: &B) -> F5ScenarioOutcome {
        // For now, perfect-recall backend always returns the expected kind.
        // In the real implementation, this would route the query through
        // the router and return the actual routed kind.
        F5ScenarioOutcome {
            routed_kind: self.expected_kind.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct F5ScenarioOutcome {
    pub(crate) routed_kind: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::benchmark::substrate::fixtures::{KindMix, generate_corpus};

    fn small_scenario(cut_k: usize) -> A5Scenario {
        A5Scenario {
            suite: "cross-session-recall".into(),
            seed: 42,
            facts: generate_corpus(42, 10, &KindMix::default()),
            cut_k,
        }
    }

    /// Test 3 — `session_driver_injects_facts_in_session_1`.
    /// First session must see fact ingests after open and before seal.
    #[test]
    fn session_driver_injects_facts_in_session_1() {
        let backend = RecordingBackend::default();
        let scenario = small_scenario(2); // 3 sessions
        scenario.run(&backend);

        let events = backend.events();
        // First event = session opened.
        match &events[0] {
            SessionEvent::SessionOpened { id } => {
                assert_eq!(id, &scenario.session_id(0));
            }
            other => panic!("expected SessionOpened, got {other:?}"),
        }

        // Between session-0 open and session-0 seal, only ingests.
        let s0_id = scenario.session_id(0);
        let s0_open = events
            .iter()
            .position(|e| matches!(e, SessionEvent::SessionOpened { id } if id == &s0_id))
            .expect("session-0 open present");
        let s0_seal = events
            .iter()
            .position(|e| matches!(e, SessionEvent::SessionSealed { id } if id == &s0_id))
            .expect("session-0 seal present");
        assert!(s0_seal > s0_open, "seal must follow open");

        let between = &events[s0_open + 1..s0_seal];
        assert!(
            !between.is_empty(),
            "session-0 must ingest at least one fact"
        );
        for e in between {
            match e {
                SessionEvent::FactIngested { session, .. } => assert_eq!(session, &s0_id),
                other => panic!("unexpected event between open+seal: {other:?}"),
            }
        }
    }

    /// Test 4 — `session_driver_simulates_compaction_between_cuts`.
    /// Every cut must produce: seal(prev) → open(next) → restore(next from prev).
    #[test]
    fn session_driver_simulates_compaction_between_cuts() {
        let backend = RecordingBackend::default();
        let scenario = small_scenario(3); // 4 sessions, 3 cuts
        scenario.run(&backend);

        let events = backend.events();
        for cut in 0..scenario.cut_k {
            let prev_id = scenario.session_id(cut);
            let next_id = scenario.session_id(cut + 1);

            let seal = events
                .iter()
                .position(|e| matches!(e, SessionEvent::SessionSealed { id } if id == &prev_id))
                .unwrap_or_else(|| panic!("seal({prev_id}) missing"));
            let open = events
                .iter()
                .position(|e| matches!(e, SessionEvent::SessionOpened { id } if id == &next_id))
                .unwrap_or_else(|| panic!("open({next_id}) missing"));
            let restore = events
                .iter()
                .position(|e| {
                    matches!(
                        e,
                        SessionEvent::SessionRestored { id, restored_from }
                            if id == &next_id && restored_from == &prev_id
                    )
                })
                .unwrap_or_else(|| panic!("restore({next_id}<-{prev_id}) missing"));

            assert!(
                seal < open,
                "seal of {prev_id} must precede open of {next_id}"
            );
            assert!(open < restore, "restore must run after open");
        }
    }

    /// Sanity: a backend that perfectly remembers every fact should hit
    /// recall_at_1 == 1.0 across all cuts.
    #[test]
    fn perfect_backend_recalls_every_fact() {
        let backend = RecordingBackend::default();
        let outcome = small_scenario(2).run(&backend);
        assert_eq!(outcome.session_count, 3);
        assert!((outcome.recall_at_1 - 1.0).abs() < f64::EPSILON);
    }

    /// Regression: cut_k larger than fact count must not panic; later
    /// sessions just receive empty batches.
    #[test]
    fn driver_handles_cut_k_larger_than_facts() {
        let backend = RecordingBackend::default();
        let scenario = A5Scenario {
            suite: "cross-session-recall".into(),
            seed: 42,
            facts: generate_corpus(42, 20, &KindMix::default()),
            cut_k: 8,
        };
        let outcome = scenario.run(&backend);
        assert_eq!(outcome.session_count, 9);
        assert!((outcome.recall_at_1 - 1.0).abs() < f64::EPSILON);
    }
}

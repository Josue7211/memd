//! Preference drift detector (Phase F4.1).
//!
//! Builds a prompt from a stored preference + recent agent turns, calls
//! the cached LLM judge (shared C4 infrastructure), and returns a
//! [`DriftCheck`] verdict. Cache is keyed by `(preference_id,
//! behavior_hash)` so re-running with identical input is free.
//!
//! Budget guard reads/writes the same monthly state file as C4 so the
//! two lanes share one $/month pool.
//!
//! Outstanding-drift state (Phase F4.2) lives in
//! [`crate::preference::outstanding`].

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::correction::judge::{
    DEFAULT_COST_PER_CALL_USD, JudgeTransport, ensure_current_month, read_budget, write_budget,
};

use super::PreferenceRecord;

/// Verdict tier returned by the drift detector.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DriftVerdict {
    /// Recent behavior matches the stored preference.
    Aligned,
    /// Recent behavior diverges from the stored preference.
    Drift,
    /// Detector could not classify (no input, parse error, judge
    /// short-circuit). No surface is emitted on `Unknown`.
    Unknown,
}

/// One detector outcome for a single `(preference, recent_turns)` pair.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DriftCheck {
    pub preference_id: String,
    pub verdict: DriftVerdict,
    pub confidence: f32,
    pub violation_count: u32,
    pub rationale: String,
    pub cache_hit: bool,
    pub cost_usd: f32,
    /// Number of turns the verdict considered. Persisted in NDJSON
    /// telemetry as `checked_turns`.
    pub checked_turns: u32,
}

/// Detector configuration. Cache directory is F4-scoped; the budget
/// file is the same path C4 uses so both lanes draw from a single pool.
#[derive(Debug, Clone)]
pub struct DriftConfig {
    pub cache_dir: PathBuf,
    pub budget_file: PathBuf,
    pub model: String,
    pub budget_usd: f32,
}

impl DriftConfig {
    /// Build configuration from environment + bundle root.
    ///
    /// Defaults match the F4 plan §7:
    /// - `MEMD_F4_JUDGE_MODEL` falls back to `MEMD_C4_JUDGE_MODEL` then `gpt-5.4`.
    /// - `MEMD_F4_JUDGE_BUDGET_USD` defaults to 5.0 (shared with C4 cap).
    /// - cache lives at `<memd>/benchmarks/grader-cache/f4`.
    /// - budget file is `<memd>/logs/c4-cost.json` — shared with C4.
    pub fn from_env(memd_dir: &Path) -> Self {
        let model = std::env::var("MEMD_F4_JUDGE_MODEL")
            .or_else(|_| std::env::var("MEMD_C4_JUDGE_MODEL"))
            .unwrap_or_else(|_| "gpt-5.4".to_string());
        let budget_usd = std::env::var("MEMD_F4_JUDGE_BUDGET_USD")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5.0_f32);
        Self {
            cache_dir: memd_dir.join("benchmarks/grader-cache/f4"),
            budget_file: memd_dir.join("logs/c4-cost.json"),
            model,
            budget_usd,
        }
    }
}

/// Detector handle. Generic over transport so tests can inject a
/// deterministic stub.
pub struct DriftDetector<T: JudgeTransport> {
    pub transport: T,
    pub config: DriftConfig,
}

impl<T: JudgeTransport> DriftDetector<T> {
    pub fn new(transport: T, config: DriftConfig) -> Self {
        Self { transport, config }
    }

    /// Run a drift check for one preference against `recent_turns`.
    ///
    /// Cache hit returns synthesized zero-cost verdict. Cache miss
    /// passes the budget guard before calling the proxy and writes
    /// budget + cache on success.
    pub fn detect(
        &self,
        preference: &PreferenceRecord,
        recent_turns: &[String],
    ) -> Result<DriftCheck> {
        if recent_turns.is_empty() {
            return Ok(DriftCheck {
                preference_id: preference.id.clone(),
                verdict: DriftVerdict::Unknown,
                confidence: 0.0,
                violation_count: 0,
                rationale: "no recent turns".into(),
                cache_hit: false,
                cost_usd: 0.0,
                checked_turns: 0,
            });
        }

        let prompt = build_prompt(preference, recent_turns);
        let key = cache_key(&preference.id, recent_turns, &self.config.model);
        let cache_path = self.config.cache_dir.join(format!("{key}.json"));

        if let Some(cached) = read_cache(&cache_path)? {
            return Ok(DriftCheck {
                cache_hit: true,
                cost_usd: 0.0,
                ..cached
            });
        }

        let mut budget = read_budget(&self.config.budget_file)?;
        ensure_current_month(&mut budget);
        if budget.usd_spent + DEFAULT_COST_PER_CALL_USD > self.config.budget_usd {
            return Err(anyhow!(
                "F4_JUDGE_BUDGET_USD exceeded ({:.4} + {:.4} > {:.4})",
                budget.usd_spent,
                DEFAULT_COST_PER_CALL_USD,
                self.config.budget_usd
            ));
        }

        let raw = self.transport.call(&prompt, &self.config.model)?;
        if !(200..300).contains(&raw.status) {
            return Err(anyhow!(
                "drift judge upstream returned status {} body={}",
                raw.status,
                truncate(&raw.body, 256)
            ));
        }

        let mut check = parse_drift(&raw.body, &preference.id, recent_turns.len() as u32)?;
        check.cache_hit = false;
        check.cost_usd = DEFAULT_COST_PER_CALL_USD;

        write_cache(&cache_path, &check)?;
        budget.usd_spent += DEFAULT_COST_PER_CALL_USD;
        budget.call_count += 1;
        write_budget(&self.config.budget_file, &budget)?;

        Ok(check)
    }
}

/// Build the judge prompt. Keep deterministic so the cache key is
/// stable across runs.
pub fn build_prompt(preference: &PreferenceRecord, recent_turns: &[String]) -> String {
    let mut turns = String::new();
    for (i, turn) in recent_turns.iter().enumerate() {
        turns.push_str(&format!("[{i}] {turn}\n"));
    }
    format!(
        "You are checking whether the assistant's recent turns drift from a stored user preference.\n\
         Reply with strict JSON: {{\"verdict\":\"aligned|drift\",\"confidence\":0..1,\"violation_count\":N,\"rationale\":\"...\"}}.\n\
         Preference id: {id}\n\
         Preference: {pref}\n\
         Recent turns ({n}):\n{turns}",
        id = preference.id,
        pref = preference.content,
        n = recent_turns.len(),
    )
}

/// Cache key over `(pref_id, model, behavior_hash)`. Two checks with
/// the same preference id + identical turn window collide in the cache.
pub fn cache_key(pref_id: &str, recent_turns: &[String], model: &str) -> String {
    let mut h = Sha256::new();
    h.update(pref_id.as_bytes());
    h.update(b"|");
    h.update(model.as_bytes());
    for turn in recent_turns {
        h.update(b"|");
        h.update(turn.as_bytes());
    }
    let digest = h.finalize();
    let mut out = String::with_capacity(digest.len() * 2);
    for b in digest.iter() {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

fn read_cache(path: &Path) -> Result<Option<DriftCheck>> {
    match fs::read_to_string(path) {
        Ok(body) => Ok(Some(serde_json::from_str(&body)?)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn write_cache(path: &Path, check: &DriftCheck) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(check)?)?;
    Ok(())
}

fn parse_drift(body: &str, preference_id: &str, checked_turns: u32) -> Result<DriftCheck> {
    let v: serde_json::Value = serde_json::from_str(body)?;
    let verdict = match v.get("verdict").and_then(|x| x.as_str()) {
        Some("aligned") => DriftVerdict::Aligned,
        Some("drift") => DriftVerdict::Drift,
        Some(other) => return Err(anyhow!("unknown verdict: {other}")),
        None => return Err(anyhow!("missing verdict field")),
    };
    let confidence = v
        .get("confidence")
        .and_then(|x| x.as_f64())
        .map(|x| x as f32)
        .unwrap_or(0.5);
    let violation_count = v
        .get("violation_count")
        .and_then(|x| x.as_u64())
        .unwrap_or(0) as u32;
    let rationale = v
        .get("rationale")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    Ok(DriftCheck {
        preference_id: preference_id.to_string(),
        verdict,
        confidence,
        violation_count,
        rationale,
        cache_hit: false,
        cost_usd: 0.0,
        checked_turns,
    })
}

fn truncate(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::correction::CorrectionCandidate;
    use crate::correction::judge::{JudgeBudgetState, JudgeClient, JudgeConfig, RawJudgeResponse};
    use std::sync::Mutex;
    use tempfile::TempDir;

    struct StubTransport {
        responses: Mutex<Vec<RawJudgeResponse>>,
        calls: Mutex<u32>,
        last_prompt: Mutex<Option<String>>,
    }

    impl StubTransport {
        fn new(responses: Vec<RawJudgeResponse>) -> Self {
            Self {
                responses: Mutex::new(responses),
                calls: Mutex::new(0),
                last_prompt: Mutex::new(None),
            }
        }
        fn call_count(&self) -> u32 {
            *self.calls.lock().unwrap()
        }
        fn last_prompt(&self) -> Option<String> {
            self.last_prompt.lock().unwrap().clone()
        }
    }

    impl JudgeTransport for StubTransport {
        fn call(&self, prompt: &str, _model: &str) -> Result<RawJudgeResponse> {
            *self.calls.lock().unwrap() += 1;
            *self.last_prompt.lock().unwrap() = Some(prompt.to_string());
            let mut q = self.responses.lock().unwrap();
            if q.is_empty() {
                return Err(anyhow!("no stub response queued"));
            }
            Ok(q.remove(0))
        }
    }

    fn drift_cfg(dir: &Path) -> DriftConfig {
        DriftConfig {
            cache_dir: dir.join("cache-f4"),
            budget_file: dir.join("budget.json"),
            model: "gpt-5.4".into(),
            budget_usd: 5.0,
        }
    }

    fn pref_terse() -> PreferenceRecord {
        PreferenceRecord::new(
            "pref-voice-terse",
            "voice=terse: replies must be short, no trailing summaries",
        )
    }

    /// Test 1 — drift detector flags verbose behavior against terse pref.
    #[test]
    fn drift_detector_detects_verbose_against_terse_preference() {
        let tmp = TempDir::new().unwrap();
        let cfg = drift_cfg(tmp.path());
        let stub = StubTransport::new(vec![RawJudgeResponse {
            status: 200,
            body: r#"{"verdict":"drift","confidence":0.83,"violation_count":3,"rationale":"agent wrote 3 long-form summaries"}"#.into(),
        }]);
        let detector = DriftDetector::new(stub, cfg);
        let pref = pref_terse();
        let turns = vec![
            "Sure! Here is a long-winded explanation of every step I took, with bullet points..."
                .to_string(),
            "Now let me also recap what we just did and outline next steps in detail..."
                .to_string(),
            "Finally, a summary table comparing approaches and a list of follow-ups...".to_string(),
        ];

        let check = detector.detect(&pref, &turns).unwrap();
        assert_eq!(check.verdict, DriftVerdict::Drift);
        assert!(check.confidence > 0.7, "confidence={}", check.confidence);
        assert_eq!(check.violation_count, 3);
        assert!(!check.cache_hit);
        assert_eq!(check.checked_turns, 3);
    }

    /// Test 2 — aligned behavior produces aligned verdict.
    #[test]
    fn drift_detector_passes_on_aligned_behavior() {
        let tmp = TempDir::new().unwrap();
        let cfg = drift_cfg(tmp.path());
        let stub = StubTransport::new(vec![RawJudgeResponse {
            status: 200,
            body: r#"{"verdict":"aligned","confidence":0.91,"violation_count":0,"rationale":"all replies are short"}"#.into(),
        }]);
        let detector = DriftDetector::new(stub, cfg);
        let turns = vec!["fix.".into(), "done.".into(), "ok.".into()];

        let check = detector.detect(&pref_terse(), &turns).unwrap();
        assert_eq!(check.verdict, DriftVerdict::Aligned);
        assert_eq!(check.violation_count, 0);
    }

    /// Test 3 — second check with same preference + behavior hits the cache.
    #[test]
    fn drift_detector_caches_verdict_by_preference_id_behavior_hash() {
        let tmp = TempDir::new().unwrap();
        let cfg = drift_cfg(tmp.path());
        let stub = StubTransport::new(vec![RawJudgeResponse {
            status: 200,
            body: r#"{"verdict":"drift","confidence":0.8,"violation_count":2,"rationale":"x"}"#
                .into(),
        }]);
        let detector = DriftDetector::new(stub, cfg);
        let turns = vec!["long reply".into(), "another long reply".into()];

        let v1 = detector.detect(&pref_terse(), &turns).unwrap();
        assert!(!v1.cache_hit);
        let v2 = detector.detect(&pref_terse(), &turns).unwrap();
        assert!(v2.cache_hit, "second call should hit cache");
        assert_eq!(detector.transport.call_count(), 1);

        // Different turn window should miss cache.
        let other = vec!["completely different turn".into()];
        // Queue another response so the call succeeds.
        detector
            .transport
            .responses
            .lock()
            .unwrap()
            .push(RawJudgeResponse {
                status: 200,
                body:
                    r#"{"verdict":"aligned","confidence":0.9,"violation_count":0,"rationale":"y"}"#
                        .into(),
            });
        let v3 = detector.detect(&pref_terse(), &other).unwrap();
        assert!(!v3.cache_hit);
        assert_eq!(detector.transport.call_count(), 2);
    }

    /// Test 5 — prompt includes preference + every recent turn.
    #[test]
    fn drift_prompt_includes_recent_turn_window() {
        let tmp = TempDir::new().unwrap();
        let cfg = drift_cfg(tmp.path());
        let stub = StubTransport::new(vec![RawJudgeResponse {
            status: 200,
            body: r#"{"verdict":"aligned","confidence":0.9,"violation_count":0,"rationale":"x"}"#
                .into(),
        }]);
        let detector = DriftDetector::new(stub, cfg);
        let turns = vec![
            "alpha-marker".into(),
            "beta-marker".into(),
            "gamma-marker".into(),
        ];
        detector.detect(&pref_terse(), &turns).unwrap();
        let prompt = detector.transport.last_prompt().expect("prompt captured");
        assert!(
            prompt.contains("voice=terse"),
            "prompt missing pref: {prompt}"
        );
        for t in &turns {
            assert!(prompt.contains(t), "prompt missing turn {t}: {prompt}");
        }
        assert!(prompt.contains("[0]") && prompt.contains("[1]") && prompt.contains("[2]"));
    }

    /// Test 6 — F4 detector + C4 judge share one budget pool.
    #[test]
    fn judge_budget_shared_with_c4() {
        let tmp = TempDir::new().unwrap();
        let shared_budget = tmp.path().join("shared-budget.json");

        // C4 client and F4 detector both point at `shared_budget`.
        let c4_cfg = JudgeConfig {
            cache_dir: tmp.path().join("c4-cache"),
            budget_file: shared_budget.clone(),
            model: "gpt-5.4".into(),
            budget_usd: 0.015,
            disabled: false,
        };
        let c4_stub =
            crate::correction::judge::tests_support::StubTransport::new(vec![RawJudgeResponse {
                status: 200,
                body: r#"{"decision":"confirmed","confidence":0.9,"rationale":"x"}"#.into(),
            }]);
        let c4_client = JudgeClient::new(c4_stub, c4_cfg);
        let candidate = CorrectionCandidate {
            score: 0.7,
            reasons: vec!["wait_actually".into()],
            references_prior: true,
            corrects_id: Some("rec-1".into()),
            source_turn: Some("t-1".into()),
        };
        // First C4 call drains 0.01 of 0.015 budget.
        c4_client.verdict(&candidate, "wait actually").unwrap();
        let budget_after_c4: JudgeBudgetState =
            serde_json::from_str(&fs::read_to_string(&shared_budget).unwrap()).unwrap();
        assert!(
            (budget_after_c4.usd_spent - DEFAULT_COST_PER_CALL_USD).abs() < 1e-5,
            "c4 spend not recorded: {:?}",
            budget_after_c4
        );

        // F4 detector now should fail because 0.01 + 0.01 > 0.015.
        let f4_cfg = DriftConfig {
            cache_dir: tmp.path().join("f4-cache"),
            budget_file: shared_budget.clone(),
            model: "gpt-5.4".into(),
            budget_usd: 0.015,
        };
        let f4_stub = StubTransport::new(vec![RawJudgeResponse {
            status: 200,
            body: r#"{"verdict":"drift","confidence":0.8,"violation_count":1,"rationale":"y"}"#
                .into(),
        }]);
        let f4 = DriftDetector::new(f4_stub, f4_cfg);
        let err = f4
            .detect(&pref_terse(), &vec!["very long verbose".into()])
            .unwrap_err();
        assert!(
            err.to_string().contains("BUDGET"),
            "expected shared-budget refusal, got {err}"
        );
        assert_eq!(f4.transport.call_count(), 0);
    }
}

//! LLM-judge client with on-disk cache + monthly budget guard (Phase C4.3).
//!
//! Defaults to the codex-lb proxy at 127.0.0.1:2455 with model gpt-5.4.
//! All transport is behind the [`JudgeTransport`] trait so tests inject a
//! deterministic stub instead of standing up a hyper server.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::CorrectionCandidate;

/// Average cost per judge call. Approximation; refine when the proxy
/// reports usage in headers.
pub const DEFAULT_COST_PER_CALL_USD: f32 = 0.01;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JudgeDecision {
    Confirmed,
    Rejected,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JudgeVerdict {
    pub decision: JudgeDecision,
    pub confidence: f32,
    pub rationale: String,
    pub cache_hit: bool,
    pub cost_usd: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JudgeBudgetState {
    pub month: String,
    pub usd_spent: f32,
    pub call_count: u64,
}

/// Raw transport contract — anything that can answer "judge this prompt".
pub trait JudgeTransport: Send + Sync {
    fn call(&self, prompt: &str, model: &str) -> Result<RawJudgeResponse>;
}

#[derive(Debug, Clone)]
pub struct RawJudgeResponse {
    pub status: u16,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct JudgeConfig {
    pub cache_dir: PathBuf,
    pub budget_file: PathBuf,
    pub model: String,
    pub budget_usd: f32,
    pub disabled: bool,
}

impl JudgeConfig {
    pub fn from_env(memd_dir: &Path) -> Self {
        let model = std::env::var("MEMD_C4_JUDGE_MODEL").unwrap_or_else(|_| "gpt-5.4".to_string());
        let budget_usd = std::env::var("MEMD_C4_JUDGE_BUDGET_USD")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5.0_f32);
        let disabled = matches!(
            std::env::var("MEMD_C4_JUDGE_DISABLED").as_deref(),
            Ok("1") | Ok("true") | Ok("TRUE")
        );
        Self {
            cache_dir: memd_dir.join("benchmarks/grader-cache/c4"),
            budget_file: memd_dir.join("logs/c4-cost.json"),
            model,
            budget_usd,
            disabled,
        }
    }
}

pub struct JudgeClient<T: JudgeTransport> {
    pub transport: T,
    pub config: JudgeConfig,
}

impl<T: JudgeTransport> JudgeClient<T> {
    pub fn new(transport: T, config: JudgeConfig) -> Self {
        Self { transport, config }
    }

    pub fn verdict(
        &self,
        candidate: &CorrectionCandidate,
        turn_text: &str,
    ) -> Result<JudgeVerdict> {
        if self.config.disabled {
            return Ok(JudgeVerdict {
                decision: JudgeDecision::Skipped,
                confidence: candidate.score,
                rationale: "judge disabled (MEMD_C4_JUDGE_DISABLED=1)".into(),
                cache_hit: false,
                cost_usd: 0.0,
            });
        }

        let prompt = build_prompt(candidate, turn_text);
        let key = cache_key(&prompt, &self.config.model, candidate.score);
        let cache_path = self.config.cache_dir.join(format!("{key}.json"));

        if let Some(verdict) = read_cache(&cache_path)? {
            return Ok(JudgeVerdict {
                cache_hit: true,
                cost_usd: 0.0,
                ..verdict
            });
        }

        // Budget guard fires before any network spend.
        let mut budget = read_budget(&self.config.budget_file)?;
        ensure_current_month(&mut budget);
        if budget.usd_spent + DEFAULT_COST_PER_CALL_USD > self.config.budget_usd {
            return Err(anyhow!(
                "MEMD_C4_JUDGE_BUDGET_USD exceeded ({:.4} + {:.4} > {:.4})",
                budget.usd_spent,
                DEFAULT_COST_PER_CALL_USD,
                self.config.budget_usd
            ));
        }

        let raw = self.transport.call(&prompt, &self.config.model)?;
        if !(200..300).contains(&raw.status) {
            return Err(anyhow!(
                "judge upstream returned status {} body={}",
                raw.status,
                truncate(&raw.body, 256)
            ));
        }

        let verdict = parse_verdict(&raw.body, candidate.score)
            .with_context(|| format!("parse judge body: {}", truncate(&raw.body, 256)))?;
        let verdict = JudgeVerdict {
            cache_hit: false,
            cost_usd: DEFAULT_COST_PER_CALL_USD,
            ..verdict
        };

        write_cache(&cache_path, &verdict)?;
        budget.usd_spent += DEFAULT_COST_PER_CALL_USD;
        budget.call_count += 1;
        write_budget(&self.config.budget_file, &budget)?;

        Ok(verdict)
    }
}

fn build_prompt(candidate: &CorrectionCandidate, turn_text: &str) -> String {
    format!(
        "You are reviewing whether a turn corrects a prior claim in a chat.\n\
         Reply with strict JSON: {{\"decision\":\"confirmed|rejected\",\"confidence\":0..1,\"rationale\":\"...\"}}.\n\
         Detector reasons: {reasons:?}\n\
         Detector score: {score:.3}\n\
         References prior id: {id:?}\n\
         Turn:\n{turn}\n",
        reasons = candidate.reasons,
        score = candidate.score,
        id = candidate.corrects_id,
        turn = turn_text,
    )
}

fn cache_key(prompt: &str, model: &str, score: f32) -> String {
    let mut h = Sha256::new();
    h.update(prompt.as_bytes());
    h.update(b"|");
    h.update(model.as_bytes());
    h.update(b"|");
    h.update(format!("{score:.4}").as_bytes());
    let digest = h.finalize();
    hex_lower(&digest)
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

fn read_cache(path: &Path) -> Result<Option<JudgeVerdict>> {
    match fs::read_to_string(path) {
        Ok(body) => Ok(Some(serde_json::from_str(&body)?)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn write_cache(path: &Path, verdict: &JudgeVerdict) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let serialized = serde_json::to_string_pretty(verdict)?;
    fs::write(path, serialized)?;
    Ok(())
}

pub fn read_budget(path: &Path) -> Result<JudgeBudgetState> {
    match fs::read_to_string(path) {
        Ok(body) => Ok(serde_json::from_str(&body).unwrap_or_default()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(JudgeBudgetState::default()),
        Err(e) => Err(e.into()),
    }
}

pub fn write_budget(path: &Path, state: &JudgeBudgetState) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(state)?)?;
    Ok(())
}

pub fn ensure_current_month(state: &mut JudgeBudgetState) {
    let now = chrono::Utc::now().format("%Y-%m").to_string();
    if state.month != now {
        state.month = now;
        state.usd_spent = 0.0;
        state.call_count = 0;
    }
}

fn parse_verdict(body: &str, fallback_confidence: f32) -> Result<JudgeVerdict> {
    let v: serde_json::Value = serde_json::from_str(body)?;
    let decision = match v.get("decision").and_then(|s| s.as_str()) {
        Some("confirmed") => JudgeDecision::Confirmed,
        Some("rejected") => JudgeDecision::Rejected,
        Some(other) => return Err(anyhow!("unknown decision: {other}")),
        None => return Err(anyhow!("missing decision field")),
    };
    let confidence = v
        .get("confidence")
        .and_then(|x| x.as_f64())
        .map(|x| x as f32)
        .unwrap_or(fallback_confidence);
    let rationale = v
        .get("rationale")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    Ok(JudgeVerdict {
        decision,
        confidence,
        rationale,
        cache_hit: false,
        cost_usd: 0.0,
    })
}

fn truncate(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// Reqwest-backed transport for production callers.
pub struct ReqwestTransport {
    pub base_url: String,
    pub api_key: Option<String>,
}

impl ReqwestTransport {
    pub fn from_env() -> Self {
        Self {
            base_url: std::env::var("CODEX_LB_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:2455".to_string()),
            api_key: std::env::var("CODEX_LB_API_KEY").ok(),
        }
    }
}

impl JudgeTransport for ReqwestTransport {
    fn call(&self, prompt: &str, model: &str) -> Result<RawJudgeResponse> {
        let body = serde_json::json!({
            "model": model,
            "temperature": 0,
            "messages": [{"role": "user", "content": prompt}],
        });
        let mut req = reqwest::blocking::Client::new()
            .post(format!(
                "{}/v1/chat/completions",
                self.base_url.trim_end_matches('/')
            ))
            .json(&body);
        if let Some(key) = self.api_key.as_deref() {
            req = req.bearer_auth(key);
        }
        let resp = req.send()?;
        let status = resp.status().as_u16();
        let text = resp.text()?;
        // OpenAI-style envelope: choices[0].message.content carries our JSON
        let parsed: serde_json::Value =
            serde_json::from_str(&text).unwrap_or(serde_json::Value::Null);
        let inner = parsed
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|s| s.as_str())
            .unwrap_or(&text)
            .to_string();
        Ok(RawJudgeResponse {
            status,
            body: inner,
        })
    }
}

#[cfg(test)]
pub mod tests_support {
    //! Public stub transport shared with F4's preference-drift tests.
    use super::*;
    use std::sync::Mutex;

    pub struct StubTransport {
        pub responses: Mutex<Vec<RawJudgeResponse>>,
        pub calls: Mutex<u32>,
    }

    impl StubTransport {
        pub fn new(responses: Vec<RawJudgeResponse>) -> Self {
            Self {
                responses: Mutex::new(responses),
                calls: Mutex::new(0),
            }
        }
        pub fn call_count(&self) -> u32 {
            *self.calls.lock().unwrap()
        }
    }

    impl JudgeTransport for StubTransport {
        fn call(&self, _prompt: &str, _model: &str) -> Result<RawJudgeResponse> {
            *self.calls.lock().unwrap() += 1;
            let mut q = self.responses.lock().unwrap();
            if q.is_empty() {
                return Err(anyhow!("no stub response queued"));
            }
            Ok(q.remove(0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::tests_support::StubTransport;
    use super::*;
    use tempfile::TempDir;

    fn cfg(dir: &Path) -> JudgeConfig {
        JudgeConfig {
            cache_dir: dir.join("cache"),
            budget_file: dir.join("budget.json"),
            model: "gpt-5.4".into(),
            budget_usd: 5.0,
            disabled: false,
        }
    }

    fn cand() -> CorrectionCandidate {
        CorrectionCandidate {
            score: 0.7,
            reasons: vec!["wait_actually".into()],
            references_prior: true,
            corrects_id: Some("rec-1".into()),
            source_turn: Some("t-3".into()),
        }
    }

    #[test]
    fn judge_cache_hit_returns_without_network() {
        let tmp = TempDir::new().unwrap();
        let cfg = cfg(tmp.path());
        let stub = StubTransport::new(vec![RawJudgeResponse {
            status: 200,
            body: r#"{"decision":"confirmed","confidence":0.9,"rationale":"clear"}"#.into(),
        }]);
        let client = JudgeClient::new(stub, cfg);
        let cand = cand();
        let v1 = client.verdict(&cand, "wait actually it's beta").unwrap();
        assert_eq!(v1.decision, JudgeDecision::Confirmed);
        assert!(!v1.cache_hit);

        let v2 = client.verdict(&cand, "wait actually it's beta").unwrap();
        assert!(v2.cache_hit, "second call should hit cache");
        assert_eq!(client.transport.call_count(), 1);
    }

    #[test]
    fn judge_cache_miss_calls_proxy_and_writes_cache() {
        let tmp = TempDir::new().unwrap();
        let cfg = cfg(tmp.path());
        let cache_dir = cfg.cache_dir.clone();
        let stub = StubTransport::new(vec![RawJudgeResponse {
            status: 200,
            body: r#"{"decision":"confirmed","confidence":0.88,"rationale":"matches prior"}"#
                .into(),
        }]);
        let client = JudgeClient::new(stub, cfg);
        client.verdict(&cand(), "wait actually it's beta").unwrap();
        assert_eq!(client.transport.call_count(), 1);
        let entries: Vec<_> = fs::read_dir(&cache_dir).unwrap().collect();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn judge_budget_guard_refuses_when_budget_exceeded() {
        let tmp = TempDir::new().unwrap();
        let mut cfg = cfg(tmp.path());
        cfg.budget_usd = 0.001;
        let stub = StubTransport::new(vec![RawJudgeResponse {
            status: 200,
            body: r#"{"decision":"confirmed","confidence":0.9,"rationale":"x"}"#.into(),
        }]);
        let client = JudgeClient::new(stub, cfg);
        let err = client
            .verdict(&cand(), "wait actually it's beta")
            .unwrap_err();
        assert!(err.to_string().contains("BUDGET"), "msg={}", err);
        assert_eq!(client.transport.call_count(), 0);
    }

    #[test]
    fn judge_rejects_non_2xx_upstream_gracefully() {
        let tmp = TempDir::new().unwrap();
        let cfg = cfg(tmp.path());
        let stub = StubTransport::new(vec![RawJudgeResponse {
            status: 503,
            body: "service unavailable".into(),
        }]);
        let client = JudgeClient::new(stub, cfg);
        let err = client.verdict(&cand(), "x").unwrap_err();
        assert!(err.to_string().contains("503"), "msg={}", err);
    }

    #[test]
    fn judge_disabled_short_circuits_to_skipped() {
        let tmp = TempDir::new().unwrap();
        let mut cfg = cfg(tmp.path());
        cfg.disabled = true;
        let stub = StubTransport::new(vec![]);
        let client = JudgeClient::new(stub, cfg);
        let v = client.verdict(&cand(), "x").unwrap();
        assert_eq!(v.decision, JudgeDecision::Skipped);
        assert_eq!(client.transport.call_count(), 0);
    }
}

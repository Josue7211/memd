// L2.6: per-agent write rate limiting.
//
// Token-bucket-ish via fixed 60s rolling window. Two thresholds on the same
// counter:
//   soft = 100 writes/min → 429 + Retry-After; client should back off
//   hard = 200 writes/min → 429 + Retry-After; client is rejected outright
//
// Agent key is extracted from X-Memd-Agent. When absent, we fall back to the
// synthetic key "_anon" so a runaway agent that forgets the header still shares
// one bucket with its peers rather than getting unlimited budget.
//
// Writes = any non-GET/HEAD/OPTIONS request. Reads are never throttled.

use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderName, HeaderValue, Method, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::AppState;

pub const SOFT_LIMIT: u32 = 100;
pub const HARD_LIMIT: u32 = 200;
pub const WINDOW: Duration = Duration::from_secs(60);

pub(crate) const HEADER_AGENT: HeaderName = HeaderName::from_static("x-memd-agent");
pub(crate) const HEADER_REMAINING: HeaderName =
    HeaderName::from_static("x-memd-ratelimit-remaining");

#[derive(Debug, Clone, Copy)]
struct WindowState {
    started: Instant,
    count: u32,
}

#[derive(Debug)]
pub struct RateLimiter {
    state: Mutex<HashMap<String, WindowState>>,
    soft: u32,
    hard: u32,
    window: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Allow { remaining: u32 },
    Soft { retry_after_secs: u64 },
    Hard { retry_after_secs: u64 },
}

impl RateLimiter {
    pub fn new() -> Self {
        Self::with(SOFT_LIMIT, HARD_LIMIT, WINDOW)
    }

    pub fn with(soft: u32, hard: u32, window: Duration) -> Self {
        Self {
            state: Mutex::new(HashMap::new()),
            soft,
            hard,
            window,
        }
    }

    pub fn check(&self, key: &str) -> Verdict {
        self.check_at(key, Instant::now())
    }

    fn check_at(&self, key: &str, now: Instant) -> Verdict {
        let mut map = match self.state.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        let entry = map.entry(key.to_string()).or_insert(WindowState {
            started: now,
            count: 0,
        });
        if now.duration_since(entry.started) >= self.window {
            *entry = WindowState {
                started: now,
                count: 0,
            };
        }
        let elapsed = now.duration_since(entry.started);
        let retry_after = self.window.saturating_sub(elapsed).as_secs().max(1);
        if entry.count >= self.hard {
            return Verdict::Hard {
                retry_after_secs: retry_after,
            };
        }
        if entry.count >= self.soft {
            entry.count += 1;
            return Verdict::Soft {
                retry_after_secs: retry_after,
            };
        }
        entry.count += 1;
        Verdict::Allow {
            remaining: self.soft.saturating_sub(entry.count),
        }
    }
}

fn agent_key(req: &Request) -> String {
    req.headers()
        .get(&HEADER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .map(|v| v.to_string())
        .unwrap_or_else(|| "_anon".to_string())
}

fn is_write(method: &Method) -> bool {
    !matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS)
}

pub(crate) async fn rate_limit_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    if !is_write(req.method()) {
        return next.run(req).await;
    }
    let key = agent_key(&req);
    match state.rate_limiter.check(&key) {
        Verdict::Allow { remaining } => {
            let mut response = next.run(req).await;
            if let Ok(v) = HeaderValue::from_str(&remaining.to_string()) {
                response.headers_mut().insert(HEADER_REMAINING, v);
            }
            response
        }
        Verdict::Soft { retry_after_secs } => too_many(retry_after_secs, "soft", &key),
        Verdict::Hard { retry_after_secs } => too_many(retry_after_secs, "hard", &key),
    }
}

fn too_many(retry_after_secs: u64, tier: &str, key: &str) -> Response {
    tracing::warn!(agent = key, tier, retry_after_secs, "rate limit triggered");
    let body = format!(
        r#"{{"error":"rate_limited","tier":"{tier}","retry_after_secs":{retry_after_secs}}}"#
    );
    let mut response = Response::new(Body::from(body));
    *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    if let Ok(v) = HeaderValue::from_str(&retry_after_secs.to_string()) {
        response
            .headers_mut()
            .insert(axum::http::header::RETRY_AFTER, v);
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn under_soft_limit_allows_and_decrements_remaining() {
        let rl = RateLimiter::with(5, 10, Duration::from_secs(60));
        let v1 = rl.check("agent-a");
        let v2 = rl.check("agent-a");
        match (v1, v2) {
            (Verdict::Allow { remaining: r1 }, Verdict::Allow { remaining: r2 }) => {
                assert!(
                    r1 > r2,
                    "remaining should monotonically decrease: {r1} -> {r2}"
                );
            }
            other => panic!("expected two Allow verdicts, got {other:?}"),
        }
    }

    #[test]
    fn soft_tier_triggers_between_soft_and_hard() {
        let rl = RateLimiter::with(3, 6, Duration::from_secs(60));
        for _ in 0..3 {
            assert!(matches!(rl.check("a"), Verdict::Allow { .. }));
        }
        // slots 4..=6 are soft
        for _ in 0..3 {
            assert!(matches!(rl.check("a"), Verdict::Soft { .. }));
        }
        // slot 7+ is hard
        assert!(matches!(rl.check("a"), Verdict::Hard { .. }));
    }

    #[test]
    fn hard_tier_does_not_increment_counter() {
        let rl = RateLimiter::with(1, 2, Duration::from_secs(60));
        assert!(matches!(rl.check("a"), Verdict::Allow { .. }));
        assert!(matches!(rl.check("a"), Verdict::Soft { .. }));
        // hard blocked, counter already at 2 == hard so it stays hard
        assert!(matches!(rl.check("a"), Verdict::Hard { .. }));
        assert!(matches!(rl.check("a"), Verdict::Hard { .. }));
    }

    #[test]
    fn window_rolls_over_and_resets_counter() {
        let rl = RateLimiter::with(1, 2, Duration::from_millis(25));
        let now = Instant::now();
        assert!(matches!(rl.check_at("a", now), Verdict::Allow { .. }));
        assert!(matches!(
            rl.check_at("a", now + Duration::from_millis(10)),
            Verdict::Soft { .. }
        ));
        // past window
        assert!(matches!(
            rl.check_at("a", now + Duration::from_millis(200)),
            Verdict::Allow { .. }
        ));
    }

    #[test]
    fn per_agent_buckets_are_independent() {
        let rl = RateLimiter::with(1, 2, Duration::from_secs(60));
        assert!(matches!(rl.check("a"), Verdict::Allow { .. }));
        assert!(matches!(rl.check("a"), Verdict::Soft { .. }));
        // bucket b is untouched
        assert!(matches!(rl.check("b"), Verdict::Allow { .. }));
    }
}

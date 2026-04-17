// K2.10: per-request token headers. M3 measures budget in characters
// rather than tokens (tokenizers vary), so this middleware emits:
//   X-Memd-Response-Chars       — exact response body length
//   X-Memd-Response-Tokens-Est  — heuristic estimate = ceil(chars / 4)
// Clients that want a true tokenizer can substitute their own, but this
// header gives them the measurement without recomputing it.

use axum::{
    body::{Body, to_bytes},
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};

const MAX_BODY_BYTES: usize = 64 * 1024 * 1024;

pub(crate) const HEADER_CHARS: HeaderName = HeaderName::from_static("x-memd-response-chars");
pub(crate) const HEADER_TOKENS_EST: HeaderName =
    HeaderName::from_static("x-memd-response-tokens-est");

pub(crate) async fn token_headers_middleware(request: Request, next: Next) -> Response {
    let response = next.run(request).await;
    let (mut parts, body) = response.into_parts();

    let bytes = match to_bytes(body, MAX_BODY_BYTES).await {
        Ok(b) => b,
        Err(err) => {
            tracing::warn!(
                error = %err,
                "token_headers middleware: failed to buffer body, returning empty"
            );
            return Response::from_parts(parts, Body::empty());
        }
    };

    let chars = bytes.len();
    let tokens_est = chars.div_ceil(4);

    if let Ok(v) = HeaderValue::from_str(&chars.to_string()) {
        parts.headers.insert(HEADER_CHARS, v);
    }
    if let Ok(v) = HeaderValue::from_str(&tokens_est.to_string()) {
        parts.headers.insert(HEADER_TOKENS_EST, v);
    }

    Response::from_parts(parts, Body::from(bytes))
}

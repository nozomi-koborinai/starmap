//! Shared retry/backoff helpers for GitHub's rate limits.
//!
//! Both the GraphQL client and the REST README fetcher hit the same limits —
//! HTTP 403 (secondary rate limit) and HTTP 429 (primary rate limit) — so the
//! "should I retry, and how long do I wait" logic lives here rather than being
//! duplicated per call site.

use std::time::Duration;

/// Maximum number of retries before giving up on a rate-limited request.
pub const MAX_RETRIES: u32 = 4;

/// True for the HTTP statuses GitHub uses to signal rate limiting:
/// 403 = secondary rate limit, 429 = primary rate limit.
pub fn is_rate_limit_status(status: reqwest::StatusCode) -> bool {
    status == reqwest::StatusCode::FORBIDDEN || status == reqwest::StatusCode::TOO_MANY_REQUESTS
}

/// True for statuses worth retrying: rate limits plus transient server errors
/// (500/502/503/504), which GitHub returns under load — e.g. a 504 Gateway
/// Timeout on a heavy GraphQL query.
pub fn is_retryable_status(status: reqwest::StatusCode) -> bool {
    is_rate_limit_status(status) || status.is_server_error()
}

/// How long to wait before the next retry: honor the `Retry-After` header when
/// GitHub provides one, otherwise fall back to exponential backoff.
pub fn retry_delay(resp: &reqwest::Response, attempt: u32) -> Duration {
    let secs = parse_retry_after(resp).unwrap_or_else(|| backoff_seconds(attempt));
    Duration::from_secs(secs)
}

/// Parse the `Retry-After` header value (in seconds). Returns None when the
/// header is missing or unparseable.
fn parse_retry_after(resp: &reqwest::Response) -> Option<u64> {
    resp.headers()
        .get(reqwest::header::RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()
}

/// Exponential backoff: 2s, 4s, 8s, 16s.
fn backoff_seconds(attempt: u32) -> u64 {
    2u64.saturating_pow(attempt + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_grows_exponentially() {
        assert_eq!(backoff_seconds(0), 2);
        assert_eq!(backoff_seconds(1), 4);
        assert_eq!(backoff_seconds(2), 8);
        assert_eq!(backoff_seconds(3), 16);
    }

    #[test]
    fn rate_limit_statuses() {
        assert!(is_rate_limit_status(reqwest::StatusCode::FORBIDDEN));
        assert!(is_rate_limit_status(reqwest::StatusCode::TOO_MANY_REQUESTS));
        assert!(!is_rate_limit_status(reqwest::StatusCode::OK));
        assert!(!is_rate_limit_status(reqwest::StatusCode::NOT_FOUND));
    }

    #[test]
    fn retryable_statuses() {
        // Rate limits and transient 5xx are retryable.
        assert!(is_retryable_status(reqwest::StatusCode::FORBIDDEN));
        assert!(is_retryable_status(reqwest::StatusCode::TOO_MANY_REQUESTS));
        assert!(is_retryable_status(reqwest::StatusCode::BAD_GATEWAY));
        assert!(is_retryable_status(
            reqwest::StatusCode::SERVICE_UNAVAILABLE
        ));
        assert!(is_retryable_status(reqwest::StatusCode::GATEWAY_TIMEOUT));
        // Success and ordinary client errors are not.
        assert!(!is_retryable_status(reqwest::StatusCode::OK));
        assert!(!is_retryable_status(reqwest::StatusCode::NOT_FOUND));
    }
}

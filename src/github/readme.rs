use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::Deserialize;
use std::time::Duration;

const MAX_RETRIES: u32 = 4;

/// Fetch README content via REST API. Returns Some(content) on success,
/// None on 404 / unavailable, propagates other errors.
///
/// Retries with exponential backoff on 403 / 429 (rate-limit responses).
pub async fn fetch_readme(
    client: &reqwest::Client,
    token: &str,
    owner: &str,
    name: &str,
) -> Result<Option<String>> {
    let url = format!("https://api.github.com/repos/{owner}/{name}/readme");
    let mut attempt: u32 = 0;
    loop {
        let resp = client
            .get(&url)
            .bearer_auth(token)
            .header("User-Agent", "starmap-cli")
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .context("Failed to send readme request")?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        // Handle rate-limit responses with exponential backoff.
        // 403 = secondary rate limit, 429 = primary rate limit.
        if (resp.status() == reqwest::StatusCode::FORBIDDEN
            || resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS)
            && attempt < MAX_RETRIES
        {
            let retry_after = parse_retry_after(&resp);
            let wait = retry_after.unwrap_or_else(|| backoff_seconds(attempt));
            eprintln!(
                "  rate-limited on {owner}/{name} (attempt {}/{}); waiting {}s",
                attempt + 1,
                MAX_RETRIES,
                wait
            );
            tokio::time::sleep(Duration::from_secs(wait)).await;
            attempt += 1;
            continue;
        }

        if !resp.status().is_success() {
            anyhow::bail!("README fetch failed: {}", resp.status());
        }

        #[derive(Deserialize)]
        struct ReadmeResponse {
            content: String,
            encoding: String,
        }

        let body: ReadmeResponse = resp.json().await.context("Parse README response")?;
        if body.encoding != "base64" {
            return Ok(None);
        }
        let cleaned: String = body
            .content
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        let bytes = STANDARD.decode(&cleaned).context("Decode README base64")?;
        let text = String::from_utf8(bytes).context("README is not UTF-8")?;
        return Ok(Some(text));
    }
}

/// Parse the `Retry-After` header value (in seconds). Returns None when
/// the header is missing or unparseable.
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

/// Truncate `s` to at most `max_bytes`, ending on a UTF-8 char boundary.
pub fn truncate_utf8(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut idx = max_bytes;
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    &s[..idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_ascii() {
        assert_eq!(truncate_utf8("hello world", 5), "hello");
        assert_eq!(truncate_utf8("hi", 100), "hi");
    }

    #[test]
    fn truncate_multibyte_safe() {
        // "あいうえお" is 5 chars × 3 bytes = 15 bytes
        let s = "あいうえお";
        let t = truncate_utf8(s, 5); // mid-character
        assert_eq!(t, "あ"); // backed off to char boundary
        assert!(t.is_char_boundary(t.len()));
    }

    #[test]
    fn truncate_zero() {
        assert_eq!(truncate_utf8("hello", 0), "");
    }

    #[test]
    fn backoff_grows_exponentially() {
        assert_eq!(backoff_seconds(0), 2);
        assert_eq!(backoff_seconds(1), 4);
        assert_eq!(backoff_seconds(2), 8);
        assert_eq!(backoff_seconds(3), 16);
    }
}

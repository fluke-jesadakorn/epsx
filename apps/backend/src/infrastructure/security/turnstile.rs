// Cloudflare Turnstile Server-Side Verification
// Validates Turnstile CAPTCHA tokens against the Cloudflare siteverify API

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{info, warn, error};

/// Cloudflare siteverify endpoint
const SITEVERIFY_URL: &str = "https://challenges.cloudflare.com/turnstile/v0/siteverify";

/// Global Turnstile config loaded from env at startup
static TURNSTILE_SECRET: Lazy<Option<String>> = Lazy::new(|| {
    env::var("TURNSTILE_SECRET_KEY").ok().filter(|s| !s.is_empty())
});

/// Request payload sent to Cloudflare siteverify
#[derive(Debug, Serialize)]
struct SiteverifyRequest<'a> {
    secret: &'a str,
    response: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    remoteip: Option<&'a str>,
}

/// Response from Cloudflare siteverify
#[derive(Debug, Deserialize)]
pub struct SiteverifyResponse {
    pub success: bool,
    #[serde(default)]
    pub challenge_ts: Option<String>,
    #[serde(default)]
    pub hostname: Option<String>,
    #[serde(default, rename = "error-codes")]
    pub error_codes: Vec<String>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub cdata: Option<String>,
}

/// Verify a Turnstile token against the Cloudflare API.
///
/// Returns `Ok(response)` on success, `Err(message)` on failure.
/// If `TURNSTILE_SECRET_KEY` is not set, verification is **skipped** (dev mode).
pub async fn verify_turnstile_token(
    token: &str,
    remote_ip: Option<&str>,
) -> Result<SiteverifyResponse, String> {
    let secret = match TURNSTILE_SECRET.as_deref() {
        Some(s) => s,
        None => {
            warn!("TURNSTILE_SECRET_KEY not configured – skipping Turnstile verification (dev mode)");
            return Ok(SiteverifyResponse {
                success: true,
                challenge_ts: None,
                hostname: None,
                error_codes: vec![],
                action: None,
                cdata: None,
            });
        }
    };

    // Allow explicit dev-skip token for local development without a key
    if token == "development-skip-token" {
        warn!("Development skip token received – accepting without verification");
        return Ok(SiteverifyResponse {
            success: true,
            challenge_ts: None,
            hostname: None,
            error_codes: vec!["dev-skip".to_string()],
            action: None,
            cdata: None,
        });
    }

    let payload = SiteverifyRequest {
        secret,
        response: token,
        remoteip: remote_ip,
    };

    let client = reqwest::Client::new();

    let response = client
        .post(SITEVERIFY_URL)
        .form(&payload)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| {
            error!("Turnstile siteverify request failed: {}", e);
            format!("Turnstile verification network error: {}", e)
        })?;

    if !response.status().is_success() {
        let status = response.status();
        error!("Turnstile siteverify returned HTTP {}", status);
        return Err(format!("Turnstile verification HTTP error: {}", status));
    }

    let result: SiteverifyResponse = response.json().await.map_err(|e| {
        error!("Failed to parse Turnstile siteverify response: {}", e);
        format!("Turnstile verification parse error: {}", e)
    })?;

    if result.success {
        info!(
            action = ?result.action,
            hostname = ?result.hostname,
            "Turnstile verification successful"
        );
    } else {
        warn!(
            error_codes = ?result.error_codes,
            "Turnstile verification failed"
        );
    }

    Ok(result)
}

/// Check whether Turnstile enforcement is enabled (i.e., secret key is present).
pub fn is_turnstile_enabled() -> bool {
    TURNSTILE_SECRET.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dev_skip_token() {
        let result = verify_turnstile_token("development-skip-token", None).await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }
}

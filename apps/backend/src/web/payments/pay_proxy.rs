//! Pay proxy — forwards `/api/v1/pay/*` requests to the
//! `pay.epsx.io` microservice.
//!
//! wave49(slice-4): adds a thin reverse-proxy in the monolith
//! backend so the legacy `/api/payment/*` callers (and any new
//! client that wants to talk to pay through the gateway) can
//! reach the pay-svc without exposing it directly to the
//! internet.
//!
//! Behavior:
//! - Matches any path under `/api/v1/pay/{*rest}`
//! - Reads `PAY_URL` env var (defaults to
//!   `http://epsx-pay-svc:8103` in-cluster,
//!   `http://127.0.0.1:8103` in dev).
//! - Forwards method + path + body + query params to the upstream.
//! - Returns the upstream's status + body unchanged.
//! - Logs a `tracing::warn!` per call so we can spot when the
//!   legacy paths are still being hit. After slice-4 these calls
//!   should drop to zero as the frontend switches to direct
//!   `pay.epsx.io` BFF proxies.
//!
//! Implementation note: the existing `ServiceClient` in
//! `epsx_client` only exposes typed GET/POST/PUT/DELETE
//! helpers; for a generic method-agnostic proxy we use
//! `reqwest::Client` directly (already a workspace dep).

use axum::{
    body::{to_bytes, Body},
    extract::{Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::time::Duration;
use tracing::warn;

const PAY_PROXY_LOG_CATEGORY: &str = "legacy_pay_proxy";
const PAY_PROXY_ROUTE_TEMPLATE: &str = "/api/v1/pay/{*rest}";
const PAY_PROXY_UPSTREAM_ERROR_BODY: &str = "upstream pay service unavailable";

#[derive(Clone)]
pub struct PayProxyState {
    pub pay_base_url: String,
    pub http: reqwest::Client,
}

impl PayProxyState {
    pub fn from_env() -> Self {
        let environment = std::env::var("EPSX_ENV")
            .or_else(|_| std::env::var("ENV"))
            .or_else(|_| std::env::var("ENVIRONMENT"))
            .unwrap_or_default();
        let pay_base_url = std::env::var("PAY_URL")
            .unwrap_or_else(|_| default_pay_base_url(&environment).to_string());
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("reqwest client builder");
        Self { pay_base_url, http }
    }
}

fn default_pay_base_url(environment: &str) -> &'static str {
    if matches!(environment, "development" | "dev" | "local" | "test") {
        "http://127.0.0.1:8103"
    } else {
        "http://epsx-pay-svc:8103"
    }
}

fn pay_rest_path(path: &str) -> &str {
    path.strip_prefix("/api/v1/pay/")
        .or_else(|| path.strip_prefix("/api/v1/pay"))
        // Axum strips the prefix before dispatching into a nested router.
        .unwrap_or(path)
        .trim_start_matches('/')
}

fn log_method(method: &str) -> &'static str {
    match method {
        "GET" => "GET",
        "HEAD" => "HEAD",
        "POST" => "POST",
        "PUT" => "PUT",
        "PATCH" => "PATCH",
        "DELETE" => "DELETE",
        "OPTIONS" => "OPTIONS",
        _ => "OTHER",
    }
}

fn warn_legacy_pay_proxy(method: &str) {
    let method = log_method(method);
    warn!(
        target: "pay_proxy",
        category = PAY_PROXY_LOG_CATEGORY,
        route_template = PAY_PROXY_ROUTE_TEMPLATE,
        method,
        "legacy pay proxy request"
    );
}

fn upstream_unavailable_response() -> Response {
    (StatusCode::BAD_GATEWAY, PAY_PROXY_UPSTREAM_ERROR_BODY).into_response()
}

/// Catch-all pay proxy handler. Matches any path under
/// `/api/v1/pay/...`. The full path (after the `/api/v1/pay/`
/// prefix) is forwarded to the pay-svc.
pub async fn pay_proxy(
    State(state): State<PayProxyState>,
    req: Request,
) -> Result<Response, Response> {
    // Capture the parts we need BEFORE consuming req.
    let full_path = req.uri().path().to_string();
    let query = req.uri().query().unwrap_or("").to_string();
    let method_str = req.method().as_str().to_string();
    let headers = req.headers().clone();

    // Extract the path after `/api/v1/pay/`.
    let rest = pay_rest_path(&full_path);

    // Build the upstream URL.
    let upstream = format!(
        "{}/api/v1/pay/{}{}{}",
        state.pay_base_url.trim_end_matches('/'),
        rest,
        if query.is_empty() { "" } else { "?" },
        query,
    );

    warn_legacy_pay_proxy(&method_str);

    // Read the request body.
    let body_bytes = match to_bytes(req.into_body(), 16 * 1024 * 1024).await {
        Ok(b) => b,
        Err(e) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read request body: {}", e),
            )
                .into_response());
        }
    };

    // Build the upstream request.
    let method = reqwest::Method::from_bytes(method_str.as_bytes()).unwrap_or(reqwest::Method::GET);
    let mut upstream_req = state
        .http
        .request(method, &upstream)
        .body(body_bytes.to_vec());

    // Forward request headers (excluding hop-by-hop).
    for (k, v) in headers.iter() {
        if matches!(
            k.as_str(),
            "host"
                | "connection"
                | "keep-alive"
                | "transfer-encoding"
                | "upgrade"
                | "content-length"
        ) {
            continue;
        }
        if let (Ok(name), Ok(value)) = (
            reqwest::header::HeaderName::from_bytes(k.as_str().as_bytes()),
            reqwest::header::HeaderValue::from_bytes(v.as_bytes()),
        ) {
            upstream_req = upstream_req.header(name, value);
        }
    }

    // Send and translate the response.
    match upstream_req.send().await {
        Ok(resp) => {
            let status =
                StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
            let mut resp_headers = axum::http::HeaderMap::new();
            for (k, v) in resp.headers().iter() {
                if matches!(
                    k.as_str(),
                    "connection" | "keep-alive" | "transfer-encoding" | "upgrade"
                ) {
                    continue;
                }
                if let (Ok(name), Ok(value)) = (
                    axum::http::HeaderName::from_bytes(k.as_str().as_bytes()),
                    axum::http::HeaderValue::from_bytes(v.as_bytes()),
                ) {
                    resp_headers.insert(name, value);
                }
            }
            let body_bytes = resp.bytes().await.unwrap_or_default();
            Ok((status, resp_headers, Body::from(body_bytes.to_vec())).into_response())
        }
        Err(_) => Err(upstream_unavailable_response()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pay_proxy_warning_metadata_is_static_and_non_identifying() {
        assert_eq!(PAY_PROXY_LOG_CATEGORY, "legacy_pay_proxy");
        assert_eq!(PAY_PROXY_ROUTE_TEMPLATE, "/api/v1/pay/{*rest}");
        assert!(!PAY_PROXY_ROUTE_TEMPLATE.contains("history/0x"));
        assert!(!PAY_PROXY_ROUTE_TEMPLATE.contains('?'));
        assert!(!PAY_PROXY_ROUTE_TEMPLATE.contains("http"));
        assert_eq!(log_method("GET"), "GET");
        assert_eq!(
            log_method("0x1111111111111111111111111111111111111111"),
            "OTHER"
        );
        assert_eq!(default_pay_base_url("development"), "http://127.0.0.1:8103");
        assert_eq!(
            default_pay_base_url("production"),
            "http://epsx-pay-svc:8103"
        );
        assert_eq!(
            pay_rest_path("/api/v1/pay/history/0x1111"),
            "history/0x1111"
        );
        assert_eq!(pay_rest_path("/history/0x1111"), "history/0x1111");
        assert_eq!(pay_rest_path("/"), "");
    }

    #[tokio::test]
    async fn upstream_failure_body_never_exposes_topology_owner_or_query() {
        let response = upstream_unavailable_response();
        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
        let body = to_bytes(response.into_body(), 1_024).await.unwrap();
        assert_eq!(body.as_ref(), PAY_PROXY_UPSTREAM_ERROR_BODY.as_bytes());
        let text = String::from_utf8(body.to_vec()).unwrap();
        for sensitive in [
            "http://epsx-pay-svc:8103",
            "/api/v1/pay/history/0x1111111111111111111111111111111111111111",
            "limit=10&offset=0",
        ] {
            assert!(!text.contains(sensitive), "{sensitive}");
        }
    }
}

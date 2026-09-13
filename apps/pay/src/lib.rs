#![cfg(all(feature = "server", not(target_arch = "wasm32")))]
//! Pay BFF: isolated epsx-pay sessions, canonical plural APIs and Dioxus SSR.
//! Browser wallets sign prepared transactions. Status comes from the Pay service.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use epsx_bff::middleware::security_headers;
use epsx_client::ServiceClient;
use std::net::SocketAddr;
use std::sync::Arc;

mod auth;
mod fullstack;
mod fullstack_auth;
#[cfg(test)]
mod fullstack_fixture;
use epsx_bff::{
    cookies::{CookieClient, CookieEnvironment},
    session::{JwksVerifier, JwksVerifierConfig, PAY_CLIENT_ID},
    typed_session::TypedBffSession,
};

#[derive(Clone)]
struct AppState {
    pay: Arc<ServiceClient>,
    identity: Arc<ServiceClient>,
    api_url: String,
    pay_url: String,
    public_origin: String,
    cookie_environment: CookieEnvironment,
    verifier: Arc<JwksVerifier>,
}
impl AppState {
    fn session(&self) -> TypedBffSession {
        TypedBffSession::new(
            self.verifier.clone(),
            self.cookie_environment,
            CookieClient::Pay,
        )
    }
}

pub async fn run() {
    epsx_observability::Observability::init("bff-pay");

    let api_url = std::env::var("API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3002);
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    let pay_url =
        std::env::var("PAYMENT_SERVICE_URL").unwrap_or_else(|_| "http://127.0.0.1:8103".into());
    let cookie_environment = CookieEnvironment::from_env().expect("explicit EPSX_ENV required");
    let issuer = std::env::var("OIDC_ISSUER").expect("OIDC_ISSUER required");
    let public_origin =
        std::env::var("PAY_FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3002".into());
    for value in [&api_url, &pay_url] {
        validate_url(value, cookie_environment, true).expect("invalid internal URL");
    }
    for value in [&issuer, &public_origin] {
        validate_url(value, cookie_environment, false).expect("invalid public URL");
    }
    let jwks = std::env::var("OIDC_JWKS_URL")
        .unwrap_or_else(|_| format!("{}/.well-known/jwks.json", issuer.trim_end_matches('/')));
    validate_url(&jwks, cookie_environment, true).expect("invalid JWKS URL");
    let verifier = Arc::new(
        JwksVerifier::with_http(
            JwksVerifierConfig::new(
                jwks,
                issuer,
                PAY_CLIENT_ID,
                std::time::Duration::from_secs(300),
            )
            .expect("invalid verifier configuration"),
        )
        .expect("verifier HTTP client"),
    );
    let client = |base_url: String| {
        Arc::new(ServiceClient::new(epsx_client::ClientConfig {
            base_url,
            timeout: std::time::Duration::from_secs(15),
        }))
    };
    let state = AppState {
        pay: client(pay_url.clone()),
        identity: client(api_url.clone()),
        api_url: api_url.trim_end_matches('/').into(),
        pay_url: pay_url.trim_end_matches('/').into(),
        public_origin: public_origin.trim_end_matches('/').into(),
        cookie_environment,
        verifier,
    };
    let public_dir = std::env::var("EPSX_PUBLIC_DIR")
        .unwrap_or_else(|_| format!("{}/../frontend/public", env!("CARGO_MANIFEST_DIR")));
    let fullstack_app = fullstack::application(state.clone());
    let assets = std::env::var("DIOXUS_PUBLIC_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::current_exe()
                .expect("executable path")
                .parent()
                .expect("executable directory")
                .join("public")
        });
    let app = Router::new()
        .route("/api/wallet-config", get(wallet_config))
        .route(
            "/walletconnect-2.24.0.js",
            get(|| async {
                (
                    [
                        ("content-type", "text/javascript; charset=utf-8"),
                        ("cache-control", "no-cache"),
                    ],
                    include_str!("../vendor/walletconnect-2.24.0.js"),
                )
            }),
        )
        .route("/api/health", get(api_health))
        .route(
            "/brand-icon.svg",
            get(|| async {
                (
                    [("content-type", "image/svg+xml")],
                    include_str!("../../frontend/public/logos/epsx-icon.svg"),
                )
            }),
        )
        .route(
            "/merchant.css",
            get(|| async {
                (
                    [("content-type", "text/css; charset=utf-8")],
                    include_str!("merchant.css"),
                )
            }),
        )
        .route(
            "/checkout.css",
            get(|| async {
                (
                    [
                        ("content-type", "text/css; charset=utf-8"),
                        ("cache-control", "no-cache"),
                    ],
                    include_str!("checkout.css"),
                )
            }),
        )
        .route("/api/v1/auth/challenge", post(auth::challenge))
        .route("/api/v1/auth/siwe", post(auth::login))
        .route("/api/v1/auth/refresh", post(auth::refresh))
        .route("/api/v1/auth/logout", post(auth::logout))
        .route("/api/v1/auth/me", get(auth::me))
        .route("/api/v1/pay/{*path}", get(proxy_pay).post(proxy_pay))
        .nest_service("/public", tower_http::services::ServeDir::new(public_dir))
        .nest_service(
            "/assets",
            tower_http::services::ServeDir::new(assets.join("assets")),
        )
        .nest_service(
            "/wasm",
            tower_http::services::ServeDir::new(assets.join("wasm")),
        )
        .with_state(state)
        .merge(fullstack_app)
        .layer(axum::middleware::from_fn(security_headers))
        .layer(axum::middleware::from_fn(wallet_security))
        .layer(axum::middleware::from_fn(epsx_bff::fullstack::dev_no_cache));

    let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();
    tracing::info!("Pay BFF listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn wallet_config() -> impl IntoResponse {
    let project = std::env::var("WALLETCONNECT_PROJECT_ID").unwrap_or_default();
    let project = if project.len() == 32 && project.bytes().all(|b| b.is_ascii_hexdigit()) {
        project
    } else {
        String::new()
    };
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../vendor/manifest.json")).expect("pinned SDK manifest");
    (
        [("cache-control", "no-store")],
        axum::Json(serde_json::json!({
            "projectId":project,"sdkUrl":format!("/walletconnect-2.24.0.js?v={}",manifest["files"]["walletconnect-2.24.0.js"].as_str().unwrap()),"sdkIntegrity":manifest["sdkIntegrity"]
        })),
    )
}

// Pay-only additions for the WalletConnect relay and verification services.
async fn wallet_security(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    let mut response = next.run(request).await;
    if let Some(csp) = response
        .headers()
        .get("content-security-policy")
        .and_then(|v| v.to_str().ok())
    {
        let csp = csp.replace("connect-src 'self'", "connect-src 'self' https://*.walletconnect.com https://*.walletconnect.org https://*.reown.com");
        if let Ok(value) = csp.parse() {
            response
                .headers_mut()
                .insert("content-security-policy", value);
        }
    }
    response
}

async fn api_health() -> &'static str {
    "ok"
}

fn validate_url(
    value: &str,
    environment: CookieEnvironment,
    internal: bool,
) -> Result<(), &'static str> {
    let url = reqwest::Url::parse(value).map_err(|_| "absolute URL required")?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err("forbidden URL components");
    }
    let loopback = url.host_str().is_some_and(|host| {
        host.trim_matches(['[', ']'])
            .parse::<std::net::IpAddr>()
            .is_ok_and(|ip| ip.is_loopback())
    });
    if internal && url.scheme() == "http" && !loopback {
        return Err("internal HTTP requires numeric loopback");
    }
    if environment == CookieEnvironment::Production
        && !(internal && loopback)
        && (url.scheme() != "https" || loopback || url.host_str() == Some("localhost"))
    {
        return Err("public HTTPS required; internal HTTP must be numeric loopback");
    }
    Ok(())
}
// Historical singular REST URLs remain transport aliases for native intents.
// They use the same session and CSRF checks as the canonical plural endpoint.
fn compatibility_path(path: &str) -> String {
    if path == "/api/v1/pay/intent" {
        return "/api/v1/pay/intents".into();
    }
    if let Some(rest) = path.strip_prefix("/api/v1/pay/intent/") {
        let rest = rest.strip_suffix("/status").unwrap_or(rest);
        return format!("/api/v1/pay/intents/{rest}");
    }
    path.into()
}
fn compatibility_body(
    path: &str,
    method: &axum::http::Method,
    body: axum::body::Bytes,
) -> Result<axum::body::Bytes, StatusCode> {
    if path != "/api/v1/pay/intent" || *method != axum::http::Method::POST {
        return Ok(body);
    }
    let value: serde_json::Value =
        serde_json::from_slice(&body).map_err(|_| StatusCode::BAD_REQUEST)?;
    let amount = value
        .get("amount")
        .and_then(serde_json::Value::as_str)
        .ok_or(StatusCode::BAD_REQUEST)?;
    let currency = value
        .get("currency")
        .and_then(serde_json::Value::as_str)
        .ok_or(StatusCode::BAD_REQUEST)?;
    let chain = value
        .get("chain_id")
        .and_then(serde_json::Value::as_str)
        .map(|v| {
            if let Some(hex) = v.strip_prefix("0x") {
                u64::from_str_radix(hex, 16)
            } else {
                v.parse()
            }
        })
        .transpose()
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .unwrap_or(56);
    let zero = "0x0000000000000000000000000000000000000000";
    let payload = serde_json::json!({"amount":amount,"token":value.get("token").and_then(serde_json::Value::as_str).unwrap_or(currency),"chain_id":chain,"description":value.get("description"),"payer":value.get("payer").and_then(serde_json::Value::as_str).unwrap_or(zero),"payee":value.get("payee").or_else(||value.get("merchant")).and_then(serde_json::Value::as_str).unwrap_or(zero)});
    serde_json::to_vec(&payload)
        .map(Into::into)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn proxy_pay(
    axum::extract::State(state): axum::extract::State<AppState>,
    request: axum::extract::Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let path = parts.uri.path();
    let merchant = parts
        .headers
        .get("x-pay-api-version")
        .is_some_and(|v| v == "2026-09-08");
    let guest = merchant
        && (path.starts_with("/api/v1/pay/checkout-sessions/")
            || path.starts_with("/api/v1/pay/operations/mop_")
                && parts.headers.contains_key("x-pay-checkout-token")
            || path.starts_with("/api/v1/pay/links/plink_") && path.ends_with("/checkouts")
            || parts.method == axum::http::Method::GET && path.starts_with("/api/v1/pay/catalog/")
            || parts.method == axum::http::Method::POST
                && path.starts_with("/api/v1/pay/products/pkg_")
                && path.ends_with("/checkouts")
            || path == "/api/v1/pay/config");
    let public = guest
        || parts.method == axum::http::Method::GET
            && path
                .strip_prefix("/api/v1/pay/links/")
                .is_some_and(|slug| !slug.is_empty() && !slug.contains('/'));
    if parts.method == axum::http::Method::POST && !auth::same_origin(&state, &parts.headers) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let token = if public {
        None
    } else {
        match state.session().verified_access_token(&parts.headers).await {
            Some((token, _)) => Some(token),
            None => return StatusCode::UNAUTHORIZED.into_response(),
        }
    };
    let Ok(body) = axum::body::to_bytes(body, 64 * 1024).await else {
        return StatusCode::PAYLOAD_TOO_LARGE.into_response();
    };
    let body = match compatibility_body(path, &parts.method, body) {
        Ok(body) => body,
        Err(status) => return status.into_response(),
    };
    let upstream_path = compatibility_path(path);
    let query = parts
        .uri
        .query()
        .map(|query| format!("?{query}"))
        .unwrap_or_default();
    let mut request = state
        .pay
        .auth_client()
        .request(
            parts.method,
            format!("{}{}{}", state.pay_url, upstream_path, query),
        )
        .header("Content-Type", "application/json")
        .body(body);
    if let Some(token) = token {
        request = request.bearer_auth(token)
    }
    if let Some(key) = parts.headers.get("idempotency-key") {
        request = request.header("idempotency-key", key)
    }
    for name in [
        "x-pay-api-version",
        "x-pay-environment",
        "x-pay-checkout-token",
        "x-pay-guest-id",
    ] {
        if let Some(value) = parts.headers.get(name) {
            request = request.header(name, value);
        }
    }
    let Ok(upstream) = request.send().await else {
        return StatusCode::BAD_GATEWAY.into_response();
    };
    let status = upstream.status();
    let Ok(bytes) = upstream.bytes().await else {
        return StatusCode::BAD_GATEWAY.into_response();
    };
    (
        status,
        [
            (axum::http::header::CONTENT_TYPE, "application/json"),
            (axum::http::header::CACHE_CONTROL, "no-store"),
        ],
        bytes,
    )
        .into_response()
}

#[cfg(test)]
mod compatibility_tests {
    use super::*;
    #[test]
    fn historical_rest_aliases_keep_canonical_contracts() {
        assert_eq!(
            compatibility_path("/api/v1/pay/intent/abc/status"),
            "/api/v1/pay/intents/abc"
        );
        assert_eq!(
            compatibility_path("/api/v1/pay/intent/abc/execute"),
            "/api/v1/pay/intents/abc/execute"
        );
        assert_eq!(
            compatibility_path("/api/v1/pay/links/slug"),
            "/api/v1/pay/links/slug"
        );
        let input = serde_json::json!({"amount":"500","currency":"USDT","chain_id":"0x38","merchant":"merchant"});
        let result = compatibility_body(
            "/api/v1/pay/intent",
            &axum::http::Method::POST,
            serde_json::to_vec(&input).unwrap().into(),
        )
        .unwrap();
        let result: serde_json::Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(result["chain_id"], 56);
        assert_eq!(result["token"], "USDT");
        assert_eq!(result["payee"], "merchant");
    }
}

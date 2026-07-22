use axum::{
    extract::{Path as AxPath, RawQuery, Request, State},
    http::{header, HeaderMap, HeaderValue, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
    routing::{delete, get, post},
    Json, Router,
};
use epsx_bff::{
    cookies::CookieEnvironment,
    middleware::security_headers,
    session::{JwksVerifier, JwksVerifierConfig, ADMIN_CLIENT_ID, JWKS_PATH},
};
use epsx_client::{RequestContext, ServiceClient};
use epsx_dioxus_ui::pages::admin_pages::payments::decode_admin_payment_intent_list;
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

mod auth;
mod news_adapter;
mod notification_admin_adapter;
mod session_auth;
#[cfg(test)]
mod session_auth_tests;
mod ssr;

#[derive(Clone)]
struct AppState {
    identity: Arc<ServiceClient>,
    wallet: Arc<ServiceClient>,
    payment: Arc<ServiceClient>,
    subscription: Arc<ServiceClient>,
    content: Arc<ServiceClient>,
    notification: Arc<ServiceClient>,
    analytics: Arc<ServiceClient>,
    indexer: Arc<ServiceClient>,
    verifier: Arc<JwksVerifier>,
    cookie_environment: CookieEnvironment,
    api_url: String,
    demo_login_enabled: bool,
}

#[derive(Deserialize)]
struct SiweLoginBody {
    message: String,
    signature: String,
    #[serde(default)]
    #[serde(rename = "chain_id")]
    _chain_id: String,
    address: String,
    nonce: String,
}

#[derive(Deserialize)]
struct ChallengeBody {
    address: String,
}

#[derive(Deserialize)]
struct DemoLoginBody {
    #[serde(rename = "address")]
    _address: Option<String>,
    #[serde(rename = "chain_id")]
    _chain_id: Option<String>,
}

fn ctx_from(headers: &HeaderMap) -> RequestContext {
    RequestContext::from_headers(headers)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PaymentIntentQuery {
    pub(crate) payer: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) limit: u64,
    pub(crate) offset: u64,
}

/// Bound deep pagination well below the pay service's signed `i64` extractor
/// limit. Larger offsets are treated as invalid rather than being forwarded and
/// silently interpreted differently upstream.
const MAX_PAYMENT_INTENT_OFFSET: u64 = 10_000_000;

impl PaymentIntentQuery {
    pub(crate) fn from_raw(raw_query: &str) -> Result<Self, ()> {
        let mut parsed = Self {
            payer: None,
            status: None,
            limit: 20,
            offset: 0,
        };
        let mut payer_seen = false;
        let mut status_seen = false;
        let mut limit_seen = false;
        let mut offset_seen = false;
        let mut url = reqwest::Url::parse("http://admin.invalid/")
            .expect("the fixed payment query base URL is valid");
        url.set_query((!raw_query.is_empty()).then_some(raw_query));

        for (key, value) in url.query_pairs() {
            match key.as_ref() {
                "payer" => {
                    if payer_seen {
                        return Err(());
                    }
                    payer_seen = true;
                    parsed.payer = if value.is_empty() {
                        None
                    } else {
                        Some(safe_payment_query_value(&value, 128).ok_or(())?)
                    };
                }
                "status" => {
                    if status_seen {
                        return Err(());
                    }
                    status_seen = true;
                    parsed.status = if value.is_empty() {
                        None
                    } else {
                        Some(safe_payment_query_value(&value, 32).ok_or(())?)
                    };
                }
                "limit" => {
                    if limit_seen {
                        return Err(());
                    }
                    limit_seen = true;
                    let limit = value.parse::<u64>().map_err(|_| ())?;
                    if !(1..=100).contains(&limit) {
                        return Err(());
                    }
                    parsed.limit = limit;
                }
                "offset" => {
                    if offset_seen {
                        return Err(());
                    }
                    offset_seen = true;
                    let offset = value.parse::<u64>().map_err(|_| ())?;
                    if offset > MAX_PAYMENT_INTENT_OFFSET {
                        return Err(());
                    }
                    parsed.offset = offset;
                }
                _ => {}
            }
        }
        Ok(parsed)
    }

    pub(crate) fn upstream_path(&self) -> String {
        let mut pairs = Vec::with_capacity(4);
        if let Some(payer) = &self.payer {
            pairs.push(format!("payer={payer}"));
        }
        if let Some(status) = &self.status {
            pairs.push(format!("status={status}"));
        }
        pairs.push(format!("limit={}", self.limit));
        pairs.push(format!("offset={}", self.offset));
        format!("/api/v1/admin/pay/intents?{}", pairs.join("&"))
    }
}

fn safe_payment_query_value(value: &str, max_len: usize) -> Option<String> {
    (!value.is_empty()
        && value.len() <= max_len
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'.' | b'_' | b'-')))
    .then(|| value.to_string())
}

pub(crate) fn payment_tab(raw_query: &str) -> Result<&'static str, ()> {
    let mut url = reqwest::Url::parse("http://admin.invalid/")
        .expect("the fixed payment query base URL is valid");
    url.set_query((!raw_query.is_empty()).then_some(raw_query));
    let mut tab = None;
    for (key, value) in url.query_pairs() {
        if key != "tab" {
            continue;
        }
        if tab.is_some() {
            return Err(());
        }
        tab = Some(match value.as_ref() {
            "payments" => "payments",
            "user-access" => "user-access",
            "payment-links" => "payment-links",
            _ => return Err(()),
        });
    }
    Ok(tab.unwrap_or("payments"))
}

fn safe_upstream_status(status: u16) -> Option<StatusCode> {
    match status {
        400 => Some(StatusCode::BAD_REQUEST),
        401 => Some(StatusCode::UNAUTHORIZED),
        403 => Some(StatusCode::FORBIDDEN),
        404 => Some(StatusCode::NOT_FOUND),
        409 => Some(StatusCode::CONFLICT),
        422 => Some(StatusCode::UNPROCESSABLE_ENTITY),
        429 => Some(StatusCode::TOO_MANY_REQUESTS),
        502 => Some(StatusCode::BAD_GATEWAY),
        503 => Some(StatusCode::SERVICE_UNAVAILABLE),
        504 => Some(StatusCode::GATEWAY_TIMEOUT),
        _ => None,
    }
}

fn err_to_status(e: epsx_client::ClientError) -> StatusCode {
    use epsx_client::ClientError::*;
    match e {
        Unauthorized => StatusCode::UNAUTHORIZED,
        NotFound => StatusCode::NOT_FOUND,
        Timeout => StatusCode::GATEWAY_TIMEOUT,
        Http(error) if error.is_timeout() => StatusCode::GATEWAY_TIMEOUT,
        Http(error) if error.is_connect() => StatusCode::SERVICE_UNAVAILABLE,
        UpstreamStatus(status) => safe_upstream_status(status).unwrap_or(StatusCode::BAD_GATEWAY),
        Http(_) | Service(_) | Serde(_) => StatusCode::BAD_GATEWAY,
    }
}

#[cfg(test)]
mod upstream_error_mapping_tests {
    use super::*;
    use epsx_client::ClientError;

    #[test]
    fn preserves_the_closed_safe_upstream_status_set() {
        for (code, expected) in [
            ("400", StatusCode::BAD_REQUEST),
            ("401", StatusCode::UNAUTHORIZED),
            ("403", StatusCode::FORBIDDEN),
            ("404", StatusCode::NOT_FOUND),
            ("409", StatusCode::CONFLICT),
            ("422", StatusCode::UNPROCESSABLE_ENTITY),
            ("429", StatusCode::TOO_MANY_REQUESTS),
            ("502", StatusCode::BAD_GATEWAY),
            ("503", StatusCode::SERVICE_UNAVAILABLE),
            ("504", StatusCode::GATEWAY_TIMEOUT),
        ] {
            let error = ClientError::UpstreamStatus(code.parse().unwrap());
            assert_eq!(err_to_status(error), expected, "upstream status {code}");
        }
    }

    #[test]
    fn rejects_arbitrary_or_unsafe_upstream_statuses() {
        for code in [200, 402, 418, 500, 599, 4030] {
            assert_eq!(
                err_to_status(ClientError::UpstreamStatus(code)),
                StatusCode::BAD_GATEWAY,
                "{code}"
            );
        }
    }

    #[test]
    fn typed_auth_resource_and_timeout_errors_keep_their_semantics() {
        assert_eq!(
            err_to_status(ClientError::Unauthorized),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(err_to_status(ClientError::NotFound), StatusCode::NOT_FOUND);
        assert_eq!(
            err_to_status(ClientError::Timeout),
            StatusCode::GATEWAY_TIMEOUT
        );
    }

    #[test]
    fn legacy_service_details_are_never_parsed_or_exposed_as_statuses() {
        let sensitive = "Bearer secret\r\nset-cookie: stolen=yes";
        assert_eq!(
            err_to_status(ClientError::Service(format!(
                "status 403 Forbidden: {sensitive}"
            ))),
            StatusCode::BAD_GATEWAY
        );
        assert_eq!(
            err_to_status(ClientError::Service(format!(
                "status 500 Internal Server Error: {sensitive}"
            ))),
            StatusCode::BAD_GATEWAY
        );
    }
}

#[cfg(test)]
mod payment_intent_adapter_tests {
    use super::*;

    #[test]
    fn payment_query_forwards_only_the_read_allowlist() {
        let query = PaymentIntentQuery::from_raw(
            "payer=0xabc&status=pending&limit=50&offset=100&tab=payments&force=cancel",
        )
        .unwrap();
        assert_eq!(query.payer.as_deref(), Some("0xabc"));
        assert_eq!(query.status.as_deref(), Some("pending"));
        assert_eq!(query.limit, 50);
        assert_eq!(query.offset, 100);
        assert_eq!(
            query.upstream_path(),
            "/api/v1/admin/pay/intents?payer=0xabc&status=pending&limit=50&offset=100"
        );
        assert!(!query.upstream_path().contains("force"));
    }

    #[test]
    fn payment_query_rejects_invalid_recognized_values_instead_of_broadening() {
        for raw in [
            "payer=%26admin%3Dtrue",
            "status=%0D%0Aauthorization",
            "limit=0",
            "limit=999",
            "offset=-1",
            "offset=10000001",
        ] {
            assert!(PaymentIntentQuery::from_raw(raw).is_err(), "{raw}");
        }
        assert!(MAX_PAYMENT_INTENT_OFFSET < i64::MAX as u64);
    }

    #[test]
    fn payment_query_uses_bounded_defaults_only_when_values_are_absent() {
        let query = PaymentIntentQuery::from_raw("payer=&status=&tab=payments").unwrap();
        assert_eq!(query.payer, None);
        assert_eq!(query.status, None);
        assert_eq!(query.limit, 20);
        assert_eq!(query.offset, 0);
        assert_eq!(
            query.upstream_path(),
            "/api/v1/admin/pay/intents?limit=20&offset=0"
        );
    }

    #[test]
    fn payment_query_rejects_duplicate_recognized_parameters() {
        for raw in [
            "payer=0xfirst&payer=0xsecond",
            "status=pending&status=released",
            "limit=20&limit=50",
            "offset=0&offset=20",
        ] {
            assert!(PaymentIntentQuery::from_raw(raw).is_err(), "{raw}");
        }
    }

    #[test]
    fn payment_tab_accepts_only_known_read_surfaces() {
        assert_eq!(payment_tab(""), Ok("payments"));
        assert_eq!(payment_tab("tab=payments"), Ok("payments"));
        assert_eq!(payment_tab("tab=user-access"), Ok("user-access"));
        assert_eq!(payment_tab("tab=payment-links"), Ok("payment-links"));
        assert!(payment_tab("tab=create-link").is_err());
        assert!(payment_tab("tab=%0D%0Aevil").is_err());
        assert!(payment_tab("tab=payments&tab=payment-links").is_err());
    }
}

#[tokio::main]
async fn main() {
    epsx_observability::Observability::init("bff-admin");

    // Wave 21 — dev auth bypass banner. Always evaluated (cheap) so the
    // log line is honest about the process state. Default is OFF; the
    // env var must be set to "1" to flip it on.
    if epsx_bff::dev_bypass::is_dev_bypass_enabled() {
        tracing::warn!(
            "EPSX_DEV_AUTH_BYPASS=1 — every request is treated as logged in as dev admin (0x...d3v1). NEVER enable in production."
        );
    }

    let state = state_from_env().expect("valid admin authentication configuration");
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3001);
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let api_url = state.api_url.clone();
    let app = build_app(state);

    let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();
    tracing::info!("Admin BFF listening on http://{} (api={})", addr, api_url);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn state_from_env() -> Result<AppState, String> {
    let cookie_environment = CookieEnvironment::from_env().map_err(|error| error.to_string())?;
    let api_url = std::env::var("API_URL")
        .or_else(|_| std::env::var("BACKEND_URL"))
        .map_err(|_| "API_URL or BACKEND_URL is required".to_string())?;
    let issuer = std::env::var("OIDC_ISSUER")
        .or_else(|_| std::env::var("BACKEND_URL"))
        .map_err(|_| "OIDC_ISSUER or BACKEND_URL is required".to_string())?;
    validate_auth_url(&api_url, cookie_environment, "API_URL/BACKEND_URL")?;
    validate_auth_url(&issuer, cookie_environment, "OIDC_ISSUER/BACKEND_URL")?;

    let demo_login_enabled = std::env::var("EPSX_ENABLE_DEMO_LOGIN").ok().as_deref() == Some("1");
    let dev_bypass_enabled = std::env::var("EPSX_DEV_AUTH_BYPASS").ok().as_deref() == Some("1");
    if cookie_environment == CookieEnvironment::Production
        && (demo_login_enabled || dev_bypass_enabled)
    {
        return Err("demo login and auth bypass are forbidden in production".to_string());
    }

    let verifier_config = JwksVerifierConfig::new(
        format!("{}{}", api_url.trim_end_matches('/'), JWKS_PATH),
        issuer.trim_end_matches('/'),
        ADMIN_CLIENT_ID,
        Duration::from_secs(300),
    )
    .map_err(|error| error.to_string())?;
    let verifier =
        Arc::new(JwksVerifier::with_http(verifier_config).map_err(|error| error.to_string())?);
    let cfg = epsx_client::ClientConfig {
        base_url: api_url.clone(),
        timeout: Duration::from_secs(15),
    };
    Ok(AppState {
        identity: Arc::new(ServiceClient::new(cfg.clone())),
        wallet: Arc::new(ServiceClient::new(cfg.clone())),
        payment: Arc::new(ServiceClient::new(cfg.clone())),
        subscription: Arc::new(ServiceClient::new(cfg.clone())),
        content: Arc::new(ServiceClient::new(cfg.clone())),
        notification: Arc::new(ServiceClient::new(cfg.clone())),
        analytics: Arc::new(ServiceClient::new(cfg.clone())),
        indexer: Arc::new(ServiceClient::new(cfg)),
        verifier,
        cookie_environment,
        api_url,
        demo_login_enabled,
    })
}

fn validate_auth_url(
    value: &str,
    environment: CookieEnvironment,
    label: &str,
) -> Result<(), String> {
    let url = reqwest::Url::parse(value).map_err(|_| format!("{label} must be an absolute URL"))?;
    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(format!("{label} has forbidden URL components"));
    }
    if environment == CookieEnvironment::Production && url.scheme() != "https" {
        return Err(format!("{label} must use HTTPS in production"));
    }
    if environment == CookieEnvironment::Production {
        let host = url
            .host_str()
            .ok_or_else(|| format!("{label} must include a host"))?;
        let local_host = host.eq_ignore_ascii_case("localhost")
            || host.ends_with(".localhost")
            || host
                .parse::<std::net::IpAddr>()
                .is_ok_and(|address| address.is_loopback());
        if local_host {
            return Err(format!("{label} must not use a local host in production"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod configuration_tests {
    use super::*;

    #[test]
    fn production_requires_https_non_local_auth_urls() {
        assert!(validate_auth_url(
            "https://api.epsx.io",
            CookieEnvironment::Production,
            "API_URL"
        )
        .is_ok());
        assert!(validate_auth_url(
            "http://api.epsx.io",
            CookieEnvironment::Production,
            "API_URL"
        )
        .is_err());
        assert!(validate_auth_url(
            "https://localhost:8080",
            CookieEnvironment::Production,
            "API_URL"
        )
        .is_err());
    }

    #[test]
    fn local_mode_allows_http_but_rejects_ambiguous_components() {
        assert!(
            validate_auth_url("http://localhost:8080", CookieEnvironment::Local, "API_URL").is_ok()
        );
        assert!(validate_auth_url(
            "http://localhost:8080?issuer=other",
            CookieEnvironment::Local,
            "API_URL"
        )
        .is_err());
    }
}

fn build_app(state: AppState) -> Router {
    Router::new()
        // Health
        .route("/api/health", get(api_health))
        // Auth
        .route("/api/v1/auth/challenge", post(session_auth::auth_challenge))
        .route("/api/v1/auth/siwe", post(session_auth::siwe_login))
        .route("/api/v1/auth/login", post(session_auth::siwe_login))
        .route("/api/v1/auth/refresh", post(session_auth::refresh_token))
        .route("/api/v1/auth/demo", post(session_auth::demo_login))
        .route("/api/v1/auth/me", get(session_auth::auth_me))
        .route("/api/v1/auth/logout", post(session_auth::logout))
        // The legacy admin login page no longer owns authentication. Keep the
        // browser entry point deterministic while the canonical auth flow is
        // reconciled, and never let query input choose the redirect target.
        .route("/auth", get(admin_auth_redirect))
        // Wave 43 T1 A3 — prod-mirror middleware: redirect
        // `/wallet-management` → `/wallet-management/wallets` to
        // match prod's Vercel middleware (3-hop redirect chain).
        .route("/wallet-management", get(wallet_redirect_to_wallets))
        // Users (identity)
        .route("/api/v1/users", get(list_users).post(create_user))
        .route(
            "/api/v1/users/{id}",
            get(get_user).put(update_user).delete(delete_user),
        )
        // Payments: the migration slice exposes only the canonical admin-wide
        // read contract. Owner reads and financial mutations stay unregistered.
        .route("/api/v1/payments", get(list_payments))
        // Subscriptions
        .route("/api/v1/subscriptions", get(list_subscriptions))
        .route("/api/v1/subscriptions/{id}", get(get_subscription))
        .route(
            "/api/v1/subscriptions/{id}/cancel",
            post(cancel_subscription),
        )
        .route(
            "/api/v1/subscription/plans",
            get(list_plans).post(create_plan),
        )
        .route("/api/v1/subscription/plans/{id}", get(get_plan))
        // Content
        .route("/api/v1/pages", get(list_pages).post(create_page))
        .route("/api/v1/pages/{slug}", get(get_page).put(update_page))
        .route("/api/v1/pages/{slug}/publish", post(publish_page))
        .route("/api/v1/themes", get(list_themes).post(create_theme))
        .route("/api/v1/themes/{id}", get(get_theme).put(update_theme))
        .route("/api/v1/blocks", get(list_blocks))
        .route("/api/v1/content/navigation", get(content_navigation))
        .route("/api/v1/content/site", get(content_site))
        // Notifications
        .route("/api/v1/notifications", get(list_notifications))
        .route("/api/v1/notifications/{id}/read", post(mark_read))
        .route("/api/v1/notifications/{id}", delete(delete_notification))
        .route(
            "/api/v1/notifications/templates",
            get(list_templates).post(create_template),
        )
        .route(
            "/api/v1/notifications/templates/{id}",
            delete(delete_template),
        )
        .route("/api/v1/notifications/send", post(send_notification))
        // Analytics
        .route("/api/v1/analytics/events", get(list_events))
        .route("/api/v1/analytics/metrics/{metric}", get(get_metrics))
        .route("/api/v1/analytics/revenue", get(revenue))
        .route("/api/v1/analytics/track", post(track_event))
        // Indexer
        .route("/api/v1/indexer/status/{chain}", get(chain_status))
        .route("/api/v1/indexer/block/{chain}/{number}", get(get_block))
        .route("/api/v1/indexer/tx/{chain}/{hash}", get(get_tx))
        .route(
            "/api/v1/indexer/transfers/{chain}/{address}",
            get(get_transfers),
        )
        // Wallet
        .route("/api/v1/wallet/accounts", get(list_accounts))
        .route("/api/v1/wallet/accounts/{address}", get(get_account))
        // Static assets
        .nest_service(
            "/public",
            tower_http::services::ServeDir::new(format!("{}/public", env!("CARGO_MANIFEST_DIR")))
                .fallback(tower_http::services::ServeFile::new(format!(
                    "{}/public/index.html",
                    env!("CARGO_MANIFEST_DIR")
                ))),
        )
        // SSR fallback (Dioxus)
        .fallback(fallback_handler)
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            require_verified_admin_session,
        ))
        .layer(axum::middleware::from_fn(security_headers))
        .with_state(state)
}

async fn require_verified_admin_session(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();
    if is_api_path(path) && !is_known_admin_api_path(path) {
        return api_not_found_response();
    }
    if !is_known_protected_admin_api_path(path) {
        return next.run(request).await;
    }
    if !is_allowed_protected_admin_api_method(request.method(), path) {
        // The path is registered, but this method is not. Let Axum's
        // MethodRouter return its canonical 405 without demanding a session.
        return next.run(request).await;
    }

    let Some((token, _user)) = auth::verified_access_token(
        request.headers(),
        state.verifier.as_ref(),
        state.cookie_environment,
    )
    .await
    else {
        return session_auth::clear_session_response(
            &state,
            StatusCode::UNAUTHORIZED,
            "invalid_access_token",
        );
    };
    let Ok(value) = HeaderValue::from_str(&format!("Bearer {token}")) else {
        return session_auth::clear_session_response(
            &state,
            StatusCode::UNAUTHORIZED,
            "invalid_access_token",
        );
    };
    request.headers_mut().insert(header::AUTHORIZATION, value);
    next.run(request).await
}

fn is_api_path(path: &str) -> bool {
    path == "/api" || path.starts_with("/api/")
}

fn api_segments(path: &str) -> Option<Vec<&str>> {
    if !is_api_path(path) || path.ends_with('/') {
        return None;
    }
    let segments: Vec<_> = path.trim_start_matches('/').split('/').collect();
    segments.iter().all(|segment| !segment.is_empty()).then_some(segments)
}

fn is_known_public_admin_api_path(path: &str) -> bool {
    let Some(segments) = api_segments(path) else { return false; };
    matches!(
        segments.as_slice(),
        ["api", "health"]
            | ["api", "v1", "auth", "challenge"]
            | ["api", "v1", "auth", "siwe"]
            | ["api", "v1", "auth", "login"]
            | ["api", "v1", "auth", "refresh"]
            | ["api", "v1", "auth", "demo"]
            | ["api", "v1", "auth", "me"]
            | ["api", "v1", "auth", "logout"]
            | ["api", "v1", "blocks"]
            | ["api", "v1", "content", "navigation"]
            | ["api", "v1", "content", "site"]
    )
}

/// Exact path-shape allowlist for registered protected admin API routes. The
/// authentication middleware uses this before verification so an unknown API
/// miss cannot be converted into a 401 or SSR HTML response.
fn is_known_protected_admin_api_path(path: &str) -> bool {
    let Some(segments) = api_segments(path) else { return false; };
    matches!(
        segments.as_slice(),
        ["api", "v1", "users"]
            | ["api", "v1", "users", _]
            | ["api", "v1", "payments"]
            | ["api", "v1", "subscriptions"]
            | ["api", "v1", "subscriptions", _]
            | ["api", "v1", "subscriptions", _, "cancel"]
            | ["api", "v1", "subscription", "plans"]
            | ["api", "v1", "subscription", "plans", _]
            | ["api", "v1", "pages"]
            | ["api", "v1", "pages", _]
            | ["api", "v1", "pages", _, "publish"]
            | ["api", "v1", "themes"]
            | ["api", "v1", "themes", _]
            | ["api", "v1", "notifications"]
            | ["api", "v1", "notifications", "templates"]
            | ["api", "v1", "notifications", "templates", _]
            | ["api", "v1", "notifications", "send"]
            | ["api", "v1", "notifications", _]
            | ["api", "v1", "notifications", _, "read"]
            | ["api", "v1", "analytics", "events"]
            | ["api", "v1", "analytics", "metrics", _]
            | ["api", "v1", "analytics", "revenue"]
            | ["api", "v1", "analytics", "track"]
            | ["api", "v1", "indexer", "status", _]
            | ["api", "v1", "indexer", "block", _, _]
            | ["api", "v1", "indexer", "tx", _, _]
            | ["api", "v1", "indexer", "transfers", _, _]
            | ["api", "v1", "wallet", "accounts"]
            | ["api", "v1", "wallet", "accounts", _]
    )
}

fn is_allowed_protected_admin_api_method(method: &Method, path: &str) -> bool {
    let Some(segments) = api_segments(path) else { return false; };
    let is_read = method == Method::GET || method == Method::HEAD;
    match segments.as_slice() {
        ["api", "v1", "users"] => is_read || method == Method::POST,
        ["api", "v1", "users", _] => {
            is_read || method == Method::PUT || method == Method::DELETE
        }
        ["api", "v1", "payments"]
        | ["api", "v1", "subscriptions"]
        | ["api", "v1", "subscriptions", _]
        | ["api", "v1", "subscription", "plans", _]
        | ["api", "v1", "analytics", "events"]
        | ["api", "v1", "analytics", "metrics", _]
        | ["api", "v1", "analytics", "revenue"]
        | ["api", "v1", "indexer", "status", _]
        | ["api", "v1", "indexer", "block", _, _]
        | ["api", "v1", "indexer", "tx", _, _]
        | ["api", "v1", "indexer", "transfers", _, _]
        | ["api", "v1", "wallet", "accounts"]
        | ["api", "v1", "wallet", "accounts", _] => is_read,
        ["api", "v1", "subscriptions", _, "cancel"]
        | ["api", "v1", "pages", _, "publish"]
        | ["api", "v1", "notifications", _, "read"]
        | ["api", "v1", "notifications", "send"]
        | ["api", "v1", "analytics", "track"] => method == Method::POST,
        ["api", "v1", "subscription", "plans"]
        | ["api", "v1", "pages"]
        | ["api", "v1", "themes"]
        | ["api", "v1", "notifications", "templates"] => {
            is_read || method == Method::POST
        }
        ["api", "v1", "pages", _] | ["api", "v1", "themes", _] => {
            is_read || method == Method::PUT
        }
        ["api", "v1", "notifications"] => is_read,
        ["api", "v1", "notifications", "templates", _]
        | ["api", "v1", "notifications", _] => method == Method::DELETE,
        _ => false,
    }
}

fn is_known_admin_api_path(path: &str) -> bool {
    is_known_public_admin_api_path(path) || is_known_protected_admin_api_path(path)
}

fn api_not_found_response() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error": "not_found",
            "message": "API route not found"
        })),
    )
        .into_response()
}

async fn fallback_handler(State(state): State<AppState>, request: Request) -> Response {
    if is_api_path(request.uri().path()) {
        api_not_found_response()
    } else {
        ssr::ssr_handler(State(state), request).await
    }
}

#[cfg(test)]
mod routing_tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::{header, Method, Request, StatusCode},
    };
    use tower::ServiceExt;

    fn test_state() -> AppState {
        let base_url = "http://127.0.0.1:9";
        let config = epsx_client::ClientConfig {
            base_url: base_url.to_string(),
            timeout: Duration::from_millis(50),
        };
        let client = Arc::new(ServiceClient::new(config));
        let verifier = JwksVerifierConfig::new(
            format!("{base_url}{JWKS_PATH}"),
            "https://issuer.test",
            ADMIN_CLIENT_ID,
            Duration::from_secs(60),
        )
        .unwrap();
        AppState {
            identity: client.clone(),
            wallet: client.clone(),
            payment: client.clone(),
            subscription: client.clone(),
            content: client.clone(),
            notification: client.clone(),
            analytics: client.clone(),
            indexer: client,
            verifier: Arc::new(JwksVerifier::with_http(verifier).unwrap()),
            cookie_environment: CookieEnvironment::Local,
            api_url: base_url.to_string(),
            demo_login_enabled: false,
        }
    }

    async fn request(method: Method, uri: &str) -> Response {
        build_app(test_state())
            .oneshot(Request::builder().method(method).uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    #[test]
    fn protected_api_predicate_covers_every_registered_shape() {
        for path in [
            "/api/v1/users",
            "/api/v1/users/id",
            "/api/v1/payments",
            "/api/v1/subscriptions",
            "/api/v1/subscriptions/id",
            "/api/v1/subscriptions/id/cancel",
            "/api/v1/subscription/plans",
            "/api/v1/subscription/plans/id",
            "/api/v1/pages",
            "/api/v1/pages/slug",
            "/api/v1/pages/slug/publish",
            "/api/v1/themes",
            "/api/v1/themes/id",
            "/api/v1/notifications",
            "/api/v1/notifications/id",
            "/api/v1/notifications/id/read",
            "/api/v1/notifications/templates",
            "/api/v1/notifications/templates/id",
            "/api/v1/notifications/send",
            "/api/v1/analytics/events",
            "/api/v1/analytics/metrics/usage",
            "/api/v1/analytics/revenue",
            "/api/v1/analytics/track",
            "/api/v1/indexer/status/bsc",
            "/api/v1/indexer/block/bsc/1",
            "/api/v1/indexer/tx/bsc/hash",
            "/api/v1/indexer/transfers/bsc/address",
            "/api/v1/wallet/accounts",
            "/api/v1/wallet/accounts/address",
        ] {
            assert!(is_known_protected_admin_api_path(path), "{path}");
        }

        for path in [
            "/api/v1/users/id/extra",
            "/api/v1/auth/unknown",
            "/api/v1/indexer/block/bsc",
            "/api/v1/notifications/templates/id/extra",
            "/api/v1/payments/id",
            "/api/v1/payments/id/confirm",
            "/api/v1/payments/id/cancel",
            "/api/v1/escrows",
            "/api/v1/escrows/id/release",
        ] {
            assert!(!is_known_admin_api_path(path), "{path}");
        }
    }

    #[tokio::test]
    async fn ssr_unknown_and_malformed_routes_bypass_unauth_skeleton() {
        assert_eq!(request(Method::GET, "/").await.status(), StatusCode::OK);

        for path in [
            "/missing-page",
            "/chat/",
            "/chat/id/extra",
            "/news//edit",
            "/news/id/edit/extra",
            "/wallet-management/address/extra",
            "/wallet-management/access/plans/id/extra",
            "/wallet-management/wallets/address/disable/extra",
        ] {
            let response = request(Method::GET, path).await;
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
            assert_eq!(response.headers()[header::CONTENT_TYPE], "text/html; charset=utf-8", "{path}");
            let body = to_bytes(response.into_body(), 2 * 1024 * 1024).await.unwrap();
            let html = String::from_utf8_lossy(&body);
            assert!(html.contains("Page not found"), "{path}");
            assert!(!html.contains("admin-skeleton"), "{path}");
        }

        assert_eq!(request(Method::HEAD, "/missing-page").await.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn unknown_api_is_json_before_auth_and_known_route_remains_protected() {
        for path in [
            "/api",
            "/api/",
            "/api/v1/users/id/extra",
            "/api/v1/auth/unknown",
            "/api/v1/payments/id",
            "/api/v1/payments/id/confirm",
            "/api/v1/payments/id/cancel",
            "/api/v1/escrows",
            "/api/v1/escrows/id/release",
        ] {
            let response = request(Method::GET, path).await;
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
            assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json", "{path}");
            let body: serde_json::Value = serde_json::from_slice(
                &to_bytes(response.into_body(), 16 * 1024).await.unwrap(),
            )
            .unwrap();
            assert_eq!(body["error"], "not_found", "{path}");
        }

        let head = request(Method::HEAD, "/api/v1/users/id/extra").await;
        assert_eq!(head.status(), StatusCode::NOT_FOUND);
        assert_eq!(head.headers()[header::CONTENT_TYPE], "application/json");

        assert_eq!(request(Method::GET, "/api/v1/users").await.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(request(Method::HEAD, "/api/v1/users").await.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            request(Method::POST, "/api/v1/users/id").await.status(),
            StatusCode::METHOD_NOT_ALLOWED
        );
        assert_eq!(
            request(Method::GET, "/api/v1/auth/challenge").await.status(),
            StatusCode::METHOD_NOT_ALLOWED
        );
    }

    #[tokio::test]
    async fn admin_redirect_targets_are_fixed_but_transport_drift_remains_visible() {
        for uri in [
            "/auth",
            "/auth?next=https%3A%2F%2Fevil.example%2Fsteal",
            "/auth?return_url=%2F%2Fevil.example%2Fsteal",
            "/auth?return_url=%2Fwallet-management%2Fwallets&next=%2Fanalytics",
        ] {
            let auth = request(Method::GET, uri).await;
            assert_eq!(auth.status(), StatusCode::TEMPORARY_REDIRECT, "{uri}");
            assert_eq!(auth.headers()[header::LOCATION], "/", "{uri}");
            let body = to_bytes(auth.into_body(), 16 * 1024).await.unwrap();
            let body = String::from_utf8_lossy(&body);
            assert!(!body.contains("evil.example"), "{uri}");
            assert!(!body.contains("wallet-management"), "{uri}");
            assert!(!body.contains("analytics"), "{uri}");
        }

        let auth_head = request(Method::HEAD, "/auth?return_url=https%3A%2F%2Fevil.example").await;
        assert_eq!(auth_head.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(auth_head.headers()[header::LOCATION], "/");
        assert!(to_bytes(auth_head.into_body(), 16 * 1024)
            .await
            .unwrap()
            .is_empty());

        let auth_post = request(Method::POST, "/auth?return_url=%2Fanalytics").await;
        assert_eq!(auth_post.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert!(auth_post.headers().get(header::LOCATION).is_none());

        for uri in [
            "/wallet-management",
            "/wallet-management?next=https%3A%2F%2Fevil.example%2Fsteal",
        ] {
            let wallet = request(Method::GET, uri).await;
            assert_eq!(wallet.status(), StatusCode::PERMANENT_REDIRECT, "{uri}");
            assert_eq!(
                wallet.headers()[header::LOCATION],
                "/wallet-management/wallets",
                "{uri}"
            );
        }

        for uri in [
            "/notifications",
            "/notifications?next=https%3A%2F%2Fevil.example%2Fsteal",
        ] {
            let notifications = request(Method::GET, uri).await;
            assert_eq!(notifications.status(), StatusCode::OK, "{uri}");
            let body = to_bytes(notifications.into_body(), 2 * 1024 * 1024)
                .await
                .unwrap();
            let html = String::from_utf8_lossy(&body);
            assert!(
                html.contains("window.location.replace('/notifications/manage');"),
                "{uri}"
            );
            assert!(!html.contains("evil.example"), "{uri}");
        }
    }
}

async fn api_health() -> &'static str {
    "ok"
}

async fn admin_auth_redirect() -> Response {
    Redirect::temporary("/").into_response()
}

// Wave 43 T1 A3 — mirror prod's Vercel middleware:
// `/wallet-management` → `/wallet-management/wallets` (308 Permanent).
// Prod uses Vercel middleware that fires before the page handler;
// dev's `Router::fallback(ssr_handler)` would otherwise render
// `wallet_redirect::render()` which is NOT what prod shows for the
// bare `/wallet-management` URL (prod redirects to /wallets).
async fn wallet_redirect_to_wallets() -> Response {
    Redirect::permanent("/wallet-management/wallets").into_response()
}

// ===== Users =====
async fn list_users(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .identity
        .get_with_ctx("/api/v1/identity/users", &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn create_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .identity
        .post_with_ctx("/api/v1/identity/users", &body, &ctx)
        .await
        .map(|v| (StatusCode::CREATED, Json(v)).into_response())
        .map_err(err_to_status)
}

async fn get_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/identity/users/{}", id);
    state
        .identity
        .get_with_ctx(&path, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn update_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/identity/users/{}", id);
    state
        .identity
        .put_with_ctx(&path, &body, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn delete_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/identity/users/{}", id);
    state
        .identity
        .delete_with_ctx(&path, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

// ===== Payments =====
async fn list_payments(
    State(state): State<AppState>,
    headers: HeaderMap,
    RawQuery(raw_query): RawQuery,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let query = PaymentIntentQuery::from_raw(raw_query.as_deref().unwrap_or_default())
        .map_err(|()| StatusCode::BAD_REQUEST)?;
    let value = state
        .payment
        .get_with_ctx(&query.upstream_path(), &ctx)
        .await
        .map_err(err_to_status)?;
    let payload = decode_admin_payment_intent_list(value).ok_or(StatusCode::BAD_GATEWAY)?;
    Ok(Json(payload).into_response())
}

// ===== Subscriptions =====
async fn list_subscriptions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .subscription
        .get_with_ctx("/api/v1/subscription/subscriptions", &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn get_subscription(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/subscription/subscriptions/{}", id);
    state
        .subscription
        .get_with_ctx(&path, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn cancel_subscription(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/subscription/subscriptions/{}/cancel", id);
    state
        .subscription
        .post_with_ctx(&path, &serde_json::json!({}), &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn list_plans(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .subscription
        .get_with_ctx("/api/v1/subscription/plans", &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn create_plan(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .subscription
        .post_with_ctx("/api/v1/subscription/plans", &body, &ctx)
        .await
        .map(|v| (StatusCode::CREATED, Json(v)).into_response())
        .map_err(err_to_status)
}

async fn get_plan(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/subscription/plans/{}", id);
    state
        .subscription
        .get_with_ctx(&path, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

// ===== Content =====
async fn list_pages(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .content
        .get_with_ctx("/api/v1/content/pages", &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn create_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .content
        .post_with_ctx("/api/v1/content/pages", &body, &ctx)
        .await
        .map(|v| (StatusCode::CREATED, Json(v)).into_response())
        .map_err(err_to_status)
}

async fn get_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(slug): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/content/pages/{}", slug);
    state
        .content
        .get_with_ctx(&path, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn update_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(slug): AxPath<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/content/pages/{}", slug);
    state
        .content
        .put_with_ctx(&path, &body, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn publish_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(slug): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/content/pages/{}/publish", slug);
    state
        .content
        .post_with_ctx(&path, &serde_json::json!({}), &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn list_themes(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .content
        .get_with_ctx("/api/v1/content/themes", &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn create_theme(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .content
        .post_with_ctx("/api/v1/content/themes", &body, &ctx)
        .await
        .map(|v| (StatusCode::CREATED, Json(v)).into_response())
        .map_err(err_to_status)
}

async fn get_theme(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/content/themes/{}", id);
    state
        .content
        .get_with_ctx(&path, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn update_theme(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/content/themes/{}", id);
    state
        .content
        .put_with_ctx(&path, &body, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn list_blocks(State(state): State<AppState>) -> Result<Response, StatusCode> {
    state
        .content
        .get_plain("/api/v1/content/blocks")
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn content_navigation(State(state): State<AppState>) -> Result<Response, StatusCode> {
    state
        .content
        .get_plain("/api/v1/content/navigation")
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn content_site(State(state): State<AppState>) -> Result<Response, StatusCode> {
    state
        .content
        .get_plain("/api/v1/content/site")
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

// ===== Notifications =====
async fn list_notifications(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .notification
        .get_with_ctx("/api/v1/notification/list", &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn mark_read(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/notification/{}/read", id);
    state
        .notification
        .post_with_ctx(&path, &serde_json::json!({}), &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn delete_notification(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/notification/{}", id);
    state
        .notification
        .delete_with_ctx(&path, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn list_templates(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .notification
        .get_with_ctx("/api/v1/notification/templates", &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn create_template(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .notification
        .post_with_ctx("/api/v1/notification/templates", &body, &ctx)
        .await
        .map(|v| (StatusCode::CREATED, Json(v)).into_response())
        .map_err(err_to_status)
}

async fn delete_template(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/notification/templates/{}", id);
    state
        .notification
        .delete_with_ctx(&path, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn send_notification(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .notification
        .post_with_ctx("/api/v1/notification/send", &body, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

// ===== Analytics =====
async fn list_events(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .analytics
        .get_with_ctx("/api/v1/analytics/events", &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn get_metrics(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(metric): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/analytics/metrics/{}", metric);
    state
        .analytics
        .get_with_ctx(&path, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn revenue(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .analytics
        .get_with_ctx("/api/v1/analytics/revenue", &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn track_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .analytics
        .post_with_ctx("/api/v1/analytics/track", &body, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

// ===== Indexer =====
async fn chain_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(chain): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/indexer/status/{}", chain);
    state
        .indexer
        .get_with_ctx(&path, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn get_block(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath((chain, number)): AxPath<(String, String)>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/indexer/block/{}/{}", chain, number);
    state
        .indexer
        .get_with_ctx(&path, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn get_tx(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath((chain, hash)): AxPath<(String, String)>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/indexer/tx/{}/{}", chain, hash);
    state
        .indexer
        .get_with_ctx(&path, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn get_transfers(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath((chain, address)): AxPath<(String, String)>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/indexer/transfers/{}/{}", chain, address);
    state
        .indexer
        .get_with_ctx(&path, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

// ===== Wallet =====
async fn list_accounts(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .wallet
        .get_with_ctx("/api/v1/wallet/accounts", &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn get_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(address): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/wallet/accounts/{}", address);
    state
        .wallet
        .get_with_ctx(&path, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

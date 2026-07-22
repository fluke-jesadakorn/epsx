//! EPSX Frontend BFF.
//!
//! Dioxus 0.7 fullstack SSR + axum JSON API proxy.
//!
//! Strategy:
//! - The design system (Tailwind v2 CDN, glassmorphism, EPSX color tokens,
//!   global JS controllers, FOUC prevention) is injected by
//!   `epsx_templates::design_system_head` + `epsx_templates::global_js` —
//!   exactly the same as the Next.js frontend.
//! - The page body is rendered by Dioxus `rsx!` components from
//!   `epsx_dioxus_ui::pages` and serialized to HTML via `dioxus_ssr`.
//! - JSON API endpoints (`/api/*`) are kept on the same axum router and
//!   proxied to the gateway via `epsx_client::ServiceClient`.

use axum::{
    extract::{Request, State},
    response::{IntoResponse, Response},
    routing::{any, get, post},
    Json, Router,
};
use epsx_bff::{
    cookies::CookieEnvironment,
    middleware::security_headers,
    session::{JwksVerifier, JwksVerifierConfig, FRONTEND_CLIENT_ID, JWKS_PATH},
};
use epsx_client::ServiceClient;
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

mod api;
mod auth;
mod ssr;

use api::*;
#[derive(Clone)]
pub struct AppState {
    pub identity: Arc<ServiceClient>,
    pub notification: Arc<ServiceClient>,
    pub content: Arc<ServiceClient>,
    pub analytics: Arc<ServiceClient>,
    pub wallet: Arc<ServiceClient>,
    pub payment: Arc<ServiceClient>,
    pub subscription: Arc<ServiceClient>,
    pub verifier: Arc<JwksVerifier>,
    pub cookie_environment: CookieEnvironment,
    pub api_url: String,
    pub demo_login_enabled: bool,
}

#[derive(Deserialize)]
pub struct SavePageBody {
    pub title: Option<String>,
    pub blocks: Option<serde_json::Value>,
    pub seo: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct SiweLoginBody {
    pub message: String,
    pub signature: String,
    #[serde(default)]
    pub chain_id: String,
    /// Wallet address (lowercased) that produced the signature. Wave 50b —
    /// the monolithic backend's `SignatureVerificationRequest` requires
    /// `wallet_address`, so we propagate it from the auth-page JS.
    pub address: String,
    /// Challenge nonce returned by `/api/auth/web3/challenge`. Wave 50b —
    /// the monolithic backend requires it as a separate field.
    pub nonce: String,
}

#[derive(Deserialize)]
pub struct DemoLoginBody {
    pub address: Option<String>,
    pub chain_id: Option<String>,
}

#[derive(Deserialize)]
pub struct ChallengeBody {
    pub address: String,
}

#[derive(Deserialize)]
pub struct AnalyticsTrackBody {
    pub event_name: String,
    pub properties: Option<serde_json::Value>,
    pub user_id: Option<String>,
    pub chain_id: Option<String>,
}

#[tokio::main]
async fn main() {
    epsx_observability::Observability::init("bff-frontend");
    let state = state_from_env().expect("valid frontend authentication configuration");
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let api_url = state.api_url.clone();
    let app = build_app(state);

    let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();
    tracing::info!(
        "Frontend BFF listening on http://{} (api={})",
        addr,
        api_url
    );
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

    let jwks_url = format!("{}{}", api_url.trim_end_matches('/'), JWKS_PATH);
    let verifier_config = JwksVerifierConfig::new(
        jwks_url,
        issuer.trim_end_matches('/'),
        FRONTEND_CLIENT_ID,
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
        notification: Arc::new(ServiceClient::new(cfg.clone())),
        content: Arc::new(ServiceClient::new(cfg.clone())),
        analytics: Arc::new(ServiceClient::new(cfg.clone())),
        wallet: Arc::new(ServiceClient::new(cfg.clone())),
        payment: Arc::new(ServiceClient::new(cfg.clone())),
        subscription: Arc::new(ServiceClient::new(cfg)),
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
        assert!(validate_auth_url(
            "https://127.0.0.1:8080",
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

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(api_health))
        .route("/api/v1/pages/{slug}", any(get_page))
        .route("/api/v1/edit/{slug}/save", any(save_page))
        .route("/api/v1/edit/{slug}/publish", any(publish_page))
        .route("/api/v1/auth/siwe", post(siwe_login))
        .route("/api/v1/auth/challenge", post(auth_challenge))
        .route("/api/v1/auth/demo", post(demo_login))
        .route("/api/v1/auth/refresh", post(refresh_token))
        .route("/api/v1/auth/logout", post(logout))
        .route("/api/v1/auth/me", get(auth_me))
        // Wave 23 T3 — OAuth start route. The auth page links to
        // `/api/v1/auth/oauth/{provider}` (e.g. `google`) and the
        // dev BFF must respond with a real HTTP status (not 404) so
        // the click is observable. We 501 with a clear "not
        // implemented" JSON when the backend identity service has no
        // OAuth integration yet (current state of the Rust backend
        // — see `shared/rust/epsx-identity-shared`); a future wave
        // can wire the real provider redirect.
        .route("/api/v1/auth/oauth/{provider}", get(api_oauth_start))
        .route(
            "/api/v1/notifications",
            get(notifications_api).head(|| async { axum::http::StatusCode::METHOD_NOT_ALLOWED }),
        )
        .route(
            "/api/v1/notifications/unread-count",
            get(notification_unread_count)
                .head(|| async { axum::http::StatusCode::METHOD_NOT_ALLOWED }),
        )
        .route("/api/v1/notifications/{id}/read", post(notification_read))
        .route(
            "/api/v1/notifications/{id}/delete",
            post(notification_delete),
        )
        .route(
            "/api/v1/notifications/mark-all-read",
            post(notification_mark_all),
        )
        .route(
            "/api/v1/notifications/clear-all",
            post(notification_clear_all),
        )
        .route("/api/v1/analytics/track", post(track_event))
        .route("/api/v1/rankings", get(api_rankings))
        .route("/api/v1/plans", get(api_plans))
        .route("/api/v1/news", get(api_news))
        .route("/api/v1/news/{slug}", get(api_news_post))
        // Unowned dashboard and portfolio compatibility producers are
        // intentionally absent. Those pages fail closed until their
        // owner-scoped backend contracts exist.
        .route("/api/v1/wallet/chains", get(api_wallet_chains))
        .route("/api/v1/wallet/connect", post(api_wallet_connect))
        .route("/api/v1/subscription/plans", get(api_subscription_plans))
        .route(
            "/api/v1/subscription/merchant/{addr}",
            get(api_subscription_merchant),
        )
        .route(
            "/api/v1/subscription/subscribe",
            post(api_subscription_subscribe),
        )
        .route(
            "/api/v1/subscription/plans/create",
            post(api_subscription_create_plan),
        )
        .route("/service-worker.js", get(service_worker))
        .nest_service(
            "/public",
            tower_http::services::ServeDir::new(format!("{}/public", env!("CARGO_MANIFEST_DIR")))
                .fallback(tower_http::services::ServeFile::new(format!(
                    "{}/public/index.html",
                    env!("CARGO_MANIFEST_DIR")
                ))),
        )
        .fallback(fallback_handler)
        .layer(axum::middleware::from_fn(security_headers))
        .with_state(state)
}

/// Public, body-free service-worker entry point for the `/offline` recovery
/// shell. The worker script is constant, does not inspect credentials, and is
/// deliberately revalidated so deployments cannot strand an old cache policy.
async fn service_worker() -> Response {
    (
        [
            ("content-type", "text/javascript; charset=utf-8"),
            ("cache-control", "no-cache, no-store, must-revalidate"),
            ("service-worker-allowed", "/"),
        ],
        ssr::offline_service_worker_script(),
    )
        .into_response()
}

fn is_api_path(path: &str) -> bool {
    path == "/api" || path.starts_with("/api/")
}

fn api_not_found_response() -> Response {
    (
        axum::http::StatusCode::NOT_FOUND,
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
            FRONTEND_CLIENT_ID,
            Duration::from_secs(60),
        )
        .unwrap();
        AppState {
            identity: client.clone(),
            notification: client.clone(),
            content: client.clone(),
            analytics: client.clone(),
            wallet: client.clone(),
            payment: client.clone(),
            subscription: client,
            verifier: Arc::new(JwksVerifier::with_http(verifier).unwrap()),
            cookie_environment: CookieEnvironment::Local,
            api_url: base_url.to_string(),
            demo_login_enabled: false,
        }
    }

    async fn request(method: Method, uri: &str) -> Response {
        build_app(test_state())
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn ssr_known_unknown_and_malformed_routes_have_explicit_status() {
        assert_eq!(request(Method::GET, "/").await.status(), StatusCode::OK);

        for path in [
            "/missing-page",
            "/portfolio/",
            "/portfolio/address/extra",
            "/chat/id/extra",
            "/news/slug/extra",
            "/payment/intent",
            "/payment/intent/id/extra",
        ] {
            let response = request(Method::GET, path).await;
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
            assert_eq!(
                response.headers()[header::CONTENT_TYPE],
                "text/html; charset=utf-8",
                "{path}"
            );
            let body = to_bytes(response.into_body(), 2 * 1024 * 1024)
                .await
                .unwrap();
            assert!(
                String::from_utf8_lossy(&body).contains("Page not found"),
                "{path}"
            );
        }

        assert_eq!(
            request(Method::HEAD, "/missing-page").await.status(),
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn unknown_api_is_json_and_known_method_mismatch_stays_405() {
        for path in [
            "/api",
            "/api/",
            "/api/v1/plans/extra",
            "/api/v1/credits",
            "/api/v1/account",
            "/api/v1/developer",
            "/api/v1/developer/docs",
            "/api/v1/developer/usage",
            "/api/v1/analytics/summary",
            "/api/v1/dashboard",
            "/api/v1/dashboard/stats",
            "/api/v1/portfolio/0x0000000000000000000000000000000000000001",
            "/api/v1/payment/not-an-authorized-intent",
        ] {
            let response = request(Method::GET, path).await;
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
            assert_eq!(
                response.headers()[header::CONTENT_TYPE],
                "application/json",
                "{path}"
            );
            let body: serde_json::Value =
                serde_json::from_slice(&to_bytes(response.into_body(), 16 * 1024).await.unwrap())
                    .unwrap();
            assert_eq!(body["error"], "not_found", "{path}");
        }

        let head = request(Method::HEAD, "/api/v1/plans/extra").await;
        assert_eq!(head.status(), StatusCode::NOT_FOUND);
        assert_eq!(head.headers()[header::CONTENT_TYPE], "application/json");

        assert_eq!(
            request(Method::POST, "/api/v1/plans").await.status(),
            StatusCode::METHOD_NOT_ALLOWED
        );
        for method in [
            Method::HEAD,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ] {
            for path in [
                "/api/v1/notifications",
                "/api/v1/notifications/unread-count",
            ] {
                assert_eq!(
                    request(method.clone(), path).await.status(),
                    StatusCode::METHOD_NOT_ALLOWED,
                    "{method} {path}"
                );
            }
        }
    }

    #[test]
    fn dormant_nav_badge_stays_unavailable_and_never_counts_a_list_page() {
        let source = include_str!("ui.rs");
        assert!(source.contains("data-state=\"unavailable\""));
        assert!(!source.contains("fetch('/api/v1/notifications"));
        assert!(!source.contains("/api/v1/notifications?limit=1"));
        assert!(!source.contains("items.filter"));
        assert!(!source.contains(">0</span>"));
    }

    #[tokio::test]
    async fn pricing_redirect_status_and_target_are_preserved() {
        let response = request(Method::GET, "/pricing?ref=test").await;
        assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(response.headers()[header::LOCATION], "/plans?ref=test");
    }

    #[tokio::test]
    async fn offline_worker_is_public_static_and_revalidated() {
        let response = request(Method::GET, "/service-worker.js").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "text/javascript; charset=utf-8"
        );
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "no-cache, no-store, must-revalidate"
        );
        assert_eq!(response.headers()["service-worker-allowed"], "/");
        assert!(response.headers().get(header::SET_COOKIE).is_none());

        let body = to_bytes(response.into_body(), 128 * 1024).await.unwrap();
        let script = String::from_utf8_lossy(&body);
        assert!(script.contains("const OFFLINE_PATH = '/offline';"));
        assert!(script.contains("credentials: 'omit'"));
        assert!(script.contains("url.search !== ''"));
        assert!(script.contains("request.mode !== 'navigate'"));
        assert!(script.contains("cache.put(OFFLINE_PATH"));
        assert!(!script.contains("cache.addAll"));
        assert!(!script.contains("event.request.clone()"));

        assert_eq!(
            request(Method::POST, "/service-worker.js").await.status(),
            StatusCode::METHOD_NOT_ALLOWED
        );
    }

    #[tokio::test]
    async fn offline_shell_explicitly_declares_public_cache_contract() {
        let response = request(Method::GET, "/offline").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "public, max-age=0, must-revalidate"
        );
        assert_eq!(
            response.headers()["x-epsx-public-cache"],
            "offline-shell-v1"
        );
        assert!(response.headers().get(header::SET_COOKIE).is_none());

        let body = to_bytes(response.into_body(), 2 * 1024 * 1024)
            .await
            .unwrap();
        let html = String::from_utf8_lossy(&body);
        assert!(html.contains("data-epsx-offline-worker-registration"));
        assert!(html.contains("Open this offline help page"));
        assert!(!html.contains("View cached notifications"));
        assert!(!html.contains("Your data will sync"));
    }
}

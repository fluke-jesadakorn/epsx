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
    routing::{any, get, post},
    Router,
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
        .route("/api/v1/notifications", any(notifications_api))
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
        .route("/api/v1/portfolio/{addr}", get(api_portfolio))
        // Wave 23 T5 — new data_X endpoints for previously-unwired
        // data-bound pages (account, credits, developer, analytics,
        // payment). Each returns a canned payload shape matching
        // the dev page's typed struct.
        .route("/api/v1/account", get(api_account))
        .route("/api/v1/credits", get(api_credits))
        .route("/api/v1/developer", get(api_developer))
        .route("/api/v1/developer/usage", get(api_developer_usage))
        .route("/api/v1/developer/docs", get(api_developer_docs))
        .route("/api/v1/analytics/summary", get(api_analytics))
        .route("/api/v1/dashboard", get(api_dashboard))
        // Wave 31 T1 / Wave 32 T1 — `/api/v1/dashboard/stats` is
        // the explicit stats endpoint. Wave 31 T1 added the route
        // (returned the inner `data` sub-object). Wave 32 T1
        // changed the shape to the full envelope
        // `{success: true, data: {stats, recentActivity}}` per
        // the brief: "should return full envelope `{success, data:
        // {...}}` (brief's shape). My attempt returned only inner
        // `data` sub-object." The SSR layer still extracts the
        // inner `data` for the page's `ctx.params["data_dashboard"]`
        // lookup — see `ssr.rs::fetch_page_data`.
        .route("/api/v1/dashboard/stats", get(api_dashboard_stats))
        .route("/api/v1/payment/{id}", get(api_payment))
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
        .nest_service(
            "/public",
            tower_http::services::ServeDir::new(format!("{}/public", env!("CARGO_MANIFEST_DIR")))
                .fallback(tower_http::services::ServeFile::new(format!(
                    "{}/public/index.html",
                    env!("CARGO_MANIFEST_DIR")
                ))),
        )
        .fallback(ssr::ssr_handler)
        .layer(axum::middleware::from_fn(security_headers))
        .with_state(state)
}

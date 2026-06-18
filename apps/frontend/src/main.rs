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
    extract::{Path as AxPath, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{any, get, post},
    Json, Router,
};
use epsx_bff::middleware::security_headers;
use epsx_client::ServiceClient;
use epsx_dioxus_ui::pages::{PageContext, render_page};
use epsx_templates::page_shell_with_body_class;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

mod auth;
mod api;
mod ssr;

use api::*;
use auth::{build_set_cookie, build_clear_cookie, get_cookie};

#[derive(Clone)]
pub struct AppState {
    pub identity: Arc<ServiceClient>,
    pub notification: Arc<ServiceClient>,
    pub content: Arc<ServiceClient>,
    pub analytics: Arc<ServiceClient>,
    pub wallet: Arc<ServiceClient>,
    pub payment: Arc<ServiceClient>,
    pub subscription: Arc<ServiceClient>,
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
    pub chain_id: String,
}

#[derive(Deserialize)]
pub struct DemoLoginBody {
    pub address: Option<String>,
    pub chain_id: Option<String>,
}

#[derive(Deserialize)]
pub struct ChallengeBody {
    pub address: String,
    pub chain_id: String,
}

#[derive(Serialize)]
pub struct AuthApiResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    pub user: serde_json::Value,
    pub demo: bool,
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

    // Wave 21 — dev auth bypass banner. Always evaluated (cheap) so the
    // log line is honest about the process state. Default is OFF; the
    // env var must be set to "1" to flip it on.
    if epsx_bff::dev_bypass::is_dev_bypass_enabled() {
        tracing::warn!(
            "EPSX_DEV_AUTH_BYPASS=1 — every request is treated as logged in as dev admin (0x...d3v1). NEVER enable in production."
        );
    }

    let api_url = std::env::var("API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(3000);
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let demo_login_enabled = std::env::var("EPSX_ENABLE_DEMO_LOGIN").ok().as_deref() == Some("1");

    let cfg = epsx_client::ClientConfig { base_url: api_url.clone(), timeout: std::time::Duration::from_secs(15) };
    let state = AppState {
        identity: Arc::new(ServiceClient::new(cfg.clone())),
        notification: Arc::new(ServiceClient::new(cfg.clone())),
        content: Arc::new(ServiceClient::new(cfg.clone())),
        analytics: Arc::new(ServiceClient::new(cfg.clone())),
        wallet: Arc::new(ServiceClient::new(cfg.clone())),
        payment: Arc::new(ServiceClient::new(cfg.clone())),
        subscription: Arc::new(ServiceClient::new(cfg.clone())),
        api_url,
        demo_login_enabled,
    };

    let app = Router::new()
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
        .route("/api/v1/notifications/{id}/delete", post(notification_delete))
        .route("/api/v1/notifications/mark-all-read", post(notification_mark_all))
        .route("/api/v1/notifications/clear-all", post(notification_clear_all))
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
        // Wave 31 T1 — `/api/v1/dashboard/stats` is the explicit
        // stats endpoint (returns the prod `dashboardData.data`
        // shape directly). Mirrors the brief: "add `/api/v1/dashboard/stats`
        // BFF route" so the page's 3-card stats row has a
        // named-stat endpoint, not just the full dashboard blob.
        .route("/api/v1/dashboard/stats", get(api_dashboard_stats))
        .route("/api/v1/payment/{id}", get(api_payment))
        .route("/api/v1/wallet/chains", get(api_wallet_chains))
        .route("/api/v1/wallet/connect", post(api_wallet_connect))
        .route("/api/v1/subscription/plans", get(api_subscription_plans))
        .route("/api/v1/subscription/merchant/{addr}", get(api_subscription_merchant))
        .route("/api/v1/subscription/subscribe", post(api_subscription_subscribe))
        .route("/api/v1/subscription/plans/create", post(api_subscription_create_plan))
        .nest_service("/public", tower_http::services::ServeDir::new(format!("{}/public", env!("CARGO_MANIFEST_DIR"))).fallback(tower_http::services::ServeFile::new(format!("{}/public/index.html", env!("CARGO_MANIFEST_DIR")))))
        .fallback(ssr::ssr_handler)
        .layer(axum::middleware::from_fn(security_headers))
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();
    tracing::info!("Frontend BFF listening on http://{} (api={})", addr, std::env::var("API_URL").unwrap_or_default());
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

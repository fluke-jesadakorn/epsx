use axum::{
    extract::{Path as AxPath, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use epsx_bff::middleware::security_headers;
use epsx_client::{RequestContext, ServiceClient};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;

mod auth;
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
    api_url: String,
}

fn ctx_from(headers: &HeaderMap) -> RequestContext {
    RequestContext::from_headers(headers)
}

fn err_to_status(e: epsx_client::ClientError) -> StatusCode {
    use epsx_client::ClientError::*;
    match e {
        Unauthorized => StatusCode::UNAUTHORIZED,
        NotFound => StatusCode::NOT_FOUND,
        Service(s) if s.starts_with("status 4") => StatusCode::BAD_REQUEST,
        Service(s) if s.starts_with("status 5") => StatusCode::BAD_GATEWAY,
        _ => StatusCode::BAD_GATEWAY,
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

    let api_url = std::env::var("API_URL").unwrap_or_else(|_| "http://localhost:18081".to_string());
    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(3001);
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    let cfg = epsx_client::ClientConfig { base_url: api_url.clone(), timeout: std::time::Duration::from_secs(30) };
    let state = AppState {
        identity: Arc::new(ServiceClient::new(cfg.clone())),
        wallet: Arc::new(ServiceClient::new(cfg.clone())),
        payment: Arc::new(ServiceClient::new(cfg.clone())),
        subscription: Arc::new(ServiceClient::new(cfg.clone())),
        content: Arc::new(ServiceClient::new(cfg.clone())),
        notification: Arc::new(ServiceClient::new(cfg.clone())),
        analytics: Arc::new(ServiceClient::new(cfg.clone())),
        indexer: Arc::new(ServiceClient::new(cfg.clone())),
        api_url,
    };

    let app = Router::new()
        // Health
        .route("/api/health", get(api_health))
        // Auth
        .route("/api/v1/auth/login", post(siwe_login))
        .route("/api/v1/auth/refresh", post(refresh_token))
        .route("/api/v1/auth/demo", post(demo_login))
        .route("/api/v1/auth/me", get(current_user))
        // Users (identity)
        .route("/api/v1/users", get(list_users).post(create_user))
        .route("/api/v1/users/{id}", get(get_user).put(update_user).delete(delete_user))
        // Payments
        .route("/api/v1/payments", get(list_payments))
        .route("/api/v1/payments/{id}", get(get_payment))
        .route("/api/v1/payments/{id}/confirm", post(confirm_payment))
        .route("/api/v1/payments/{id}/cancel", post(cancel_payment))
        .route("/api/v1/escrows", get(list_escrows))
        .route("/api/v1/escrows/{id}/release", post(release_escrow))
        // Subscriptions
        .route("/api/v1/subscriptions", get(list_subscriptions))
        .route("/api/v1/subscriptions/{id}", get(get_subscription))
        .route("/api/v1/subscriptions/{id}/cancel", post(cancel_subscription))
        .route("/api/v1/subscription/plans", get(list_plans).post(create_plan))
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
        .route("/api/v1/notifications/templates", get(list_templates).post(create_template))
        .route("/api/v1/notifications/templates/{id}", delete(delete_template))
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
        .route("/api/v1/indexer/transfers/{chain}/{address}", get(get_transfers))
        // Wallet
        .route("/api/v1/wallet/accounts", get(list_accounts))
        .route("/api/v1/wallet/accounts/{address}", get(get_account))
        // Static assets
        .nest_service("/public", tower_http::services::ServeDir::new(format!("{}/public", env!("CARGO_MANIFEST_DIR"))).fallback(tower_http::services::ServeFile::new(format!("{}/public/index.html", env!("CARGO_MANIFEST_DIR")))))
        // SSR fallback (Dioxus)
        .fallback(ssr::ssr_handler)
        .layer(axum::middleware::from_fn(security_headers))
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();
    tracing::info!("Admin BFF listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn api_health() -> &'static str { "ok" }

// ===== Auth =====
async fn siwe_login(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    state.identity.post_plain("/api/v1/identity/auth/siwe", &body).await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn refresh_token(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    state.identity.post_plain("/api/v1/identity/auth/refresh", &body).await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn demo_login(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    state.identity.post_plain("/api/v1/identity/auth/demo", &body).await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn current_user(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state.identity.get_with_ctx("/api/v1/identity/auth/me", &ctx).await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

// ===== Users =====
async fn list_users(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state.identity.get_with_ctx("/api/v1/identity/users", &ctx).await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn create_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state.identity.post_with_ctx("/api/v1/identity/users", &body, &ctx).await
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
    state.identity.get_with_ctx(&path, &ctx).await
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
    state.identity.put_with_ctx(&path, &body, &ctx).await
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
    state.identity.delete_with_ctx(&path, &ctx).await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

// ===== Payments =====
async fn list_payments(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state.payment.get_with_ctx("/api/v1/payment/intents", &ctx).await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn get_payment(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/payment/intents/{}", id);
    state.payment.get_with_ctx(&path, &ctx).await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn confirm_payment(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/payment/intents/{}/confirm", id);
    state.payment.post_with_ctx(&path, &serde_json::json!({}), &ctx).await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn cancel_payment(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/payment/intents/{}/cancel", id);
    state.payment.post_with_ctx(&path, &serde_json::json!({}), &ctx).await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn list_escrows(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state.payment.get_with_ctx("/api/v1/payment/escrows", &ctx).await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn release_escrow(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/payment/escrows/{}/release", id);
    state.payment.post_with_ctx(&path, &serde_json::json!({}), &ctx).await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

// ===== Subscriptions =====
async fn list_subscriptions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state.subscription.get_with_ctx("/api/v1/subscription/subscriptions", &ctx).await
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
    state.subscription.get_with_ctx(&path, &ctx).await
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
    state.subscription.post_with_ctx(&path, &serde_json::json!({}), &ctx).await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn list_plans(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state.subscription.get_with_ctx("/api/v1/subscription/plans", &ctx).await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn create_plan(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state.subscription.post_with_ctx("/api/v1/subscription/plans", &body, &ctx).await
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
    state.subscription.get_with_ctx(&path, &ctx).await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

// ===== Content =====
async fn list_pages(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state.content.get_with_ctx("/api/v1/content/pages", &ctx).await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn create_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state.content.post_with_ctx("/api/v1/content/pages", &body, &ctx).await
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
    state.content.get_with_ctx(&path, &ctx).await
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
    state.content.put_with_ctx(&path, &body, &ctx).await
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
    state.content.post_with_ctx(&path, &serde_json::json!({}), &ctx).await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn list_themes(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state.content.get_with_ctx("/api/v1/content/themes", &ctx).await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn create_theme(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state.content.post_with_ctx("/api/v1/content/themes", &body, &ctx).await
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
    state.content.get_with_ctx(&path, &ctx).await
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
    state.content.put_with_ctx(&path, &body, &ctx).await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn list_blocks(
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    state.content.get_plain("/api/v1/content/blocks").await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn content_navigation(
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    state.content.get_plain("/api/v1/content/navigation").await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn content_site(
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    state.content.get_plain("/api/v1/content/site").await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

// ===== Notifications =====
async fn list_notifications(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state.notification.get_with_ctx("/api/v1/notification/list", &ctx).await
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
    state.notification.post_with_ctx(&path, &serde_json::json!({}), &ctx).await
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
    state.notification.delete_with_ctx(&path, &ctx).await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn list_templates(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state.notification.get_with_ctx("/api/v1/notification/templates", &ctx).await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn create_template(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state.notification.post_with_ctx("/api/v1/notification/templates", &body, &ctx).await
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
    state.notification.delete_with_ctx(&path, &ctx).await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn send_notification(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state.notification.post_with_ctx("/api/v1/notification/send", &body, &ctx).await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

// ===== Analytics =====
async fn list_events(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state.analytics.get_with_ctx("/api/v1/analytics/events", &ctx).await
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
    state.analytics.get_with_ctx(&path, &ctx).await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn revenue(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state.analytics.get_with_ctx("/api/v1/analytics/revenue", &ctx).await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn track_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state.analytics.post_with_ctx("/api/v1/analytics/track", &body, &ctx).await
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
    state.indexer.get_with_ctx(&path, &ctx).await
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
    state.indexer.get_with_ctx(&path, &ctx).await
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
    state.indexer.get_with_ctx(&path, &ctx).await
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
    state.indexer.get_with_ctx(&path, &ctx).await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

// ===== Wallet =====
async fn list_accounts(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state.wallet.get_with_ctx("/api/v1/wallet/accounts", &ctx).await
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
    state.wallet.get_with_ctx(&path, &ctx).await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}


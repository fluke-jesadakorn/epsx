use axum::{
    extract::{Path as AxPath, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use epsx_client::{RequestContext, ServiceClient};
use epsx_templates::{page_shell_with_body_class, theme_toggle_button, logo};
use std::net::SocketAddr;
use std::sync::Arc;

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
        .route("/api/health", get(api_health))
        .route("/api/v1/auth/login", post(siwe_login))
        .route("/api/v1/auth/refresh", post(refresh_token))
        .route("/api/v1/auth/demo", post(demo_login))
        .route("/api/v1/auth/me", get(current_user))
        .route("/api/v1/users", get(list_users).post(create_user))
        .route("/api/v1/users/{id}", get(get_user).put(update_user).delete(delete_user))
        .route("/api/v1/payments", get(list_payments))
        .route("/api/v1/payments/{id}", get(get_payment))
        .route("/api/v1/payments/{id}/confirm", post(confirm_payment))
        .route("/api/v1/payments/{id}/cancel", post(cancel_payment))
        .route("/api/v1/escrows", get(list_escrows))
        .route("/api/v1/escrows/{id}/release", post(release_escrow))
        .route("/api/v1/subscriptions", get(list_subscriptions))
        .route("/api/v1/subscriptions/{id}", get(get_subscription))
        .route("/api/v1/subscriptions/{id}/cancel", post(cancel_subscription))
        .route("/api/v1/subscription/plans", get(list_plans).post(create_plan))
        .route("/api/v1/subscription/plans/{id}", get(get_plan))
        .route("/api/v1/pages", get(list_pages).post(create_page))
        .route("/api/v1/pages/{slug}", get(get_page).put(update_page))
        .route("/api/v1/pages/{slug}/publish", post(publish_page))
        .route("/api/v1/themes", get(list_themes).post(create_theme))
        .route("/api/v1/themes/{id}", get(get_theme).put(update_theme))
        .route("/api/v1/blocks", get(list_blocks))
        .route("/api/v1/content/navigation", get(content_navigation))
        .route("/api/v1/content/site", get(content_site))
        .route("/api/v1/notifications", get(list_notifications))
        .route("/api/v1/notifications/{id}/read", post(mark_read))
        .route("/api/v1/notifications/{id}", delete(delete_notification))
        .route("/api/v1/notifications/templates", get(list_templates).post(create_template))
        .route("/api/v1/notifications/templates/{id}", delete(delete_template))
        .route("/api/v1/notifications/send", post(send_notification))
        .route("/api/v1/analytics/events", get(list_events))
        .route("/api/v1/analytics/metrics/{metric}", get(get_metrics))
        .route("/api/v1/analytics/revenue", get(revenue))
        .route("/api/v1/analytics/track", post(track_event))
        .route("/api/v1/indexer/status/{chain}", get(chain_status))
        .route("/api/v1/indexer/block/{chain}/{number}", get(get_block))
        .route("/api/v1/indexer/tx/{chain}/{hash}", get(get_tx))
        .route("/api/v1/indexer/transfers/{chain}/{address}", get(get_transfers))
        .route("/api/v1/wallet/accounts", get(list_accounts))
        .route("/api/v1/wallet/accounts/{address}", get(get_account))
        .fallback(ssr_fallback)
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();
    tracing::info!("Admin BFF listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn api_health() -> &'static str { "ok" }

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
    let mut payload = body;
    if !payload.get("recipient").map(|v| v.is_string()).unwrap_or(false) {
        payload["recipient"] = serde_json::Value::String(
            payload.get("user_id")
                .and_then(|v| v.as_str())
                .unwrap_or("0x0000000000000000000000000000000000000000")
                .to_string()
        );
    }
    if !payload.get("channel").map(|v| v.is_string()).unwrap_or(false) {
        payload["channel"] = serde_json::Value::String("in_app".to_string());
    }
    state.notification.post_with_ctx("/api/v1/notification/send", &payload, &ctx).await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

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

async fn ssr_fallback(uri: axum::http::Uri, _headers: HeaderMap) -> Response {
    let path = uri.path();
    let html = render_page(path);
    Html(html).into_response()
}

fn render_page(path: &str) -> String {
    let title = if path.starts_with("/users") { "User Management" }
        else if path.starts_with("/payments") { "Payment Management" }
        else if path.starts_with("/subscriptions") { "Subscription Management" }
        else if path.starts_with("/content") { "Content Management" }
        else if path.starts_with("/analytics") { "Analytics Dashboard" }
        else if path.starts_with("/settings") { "System Settings" }
        else if path.starts_with("/audit") { "Audit Log" }
        else { "Admin Dashboard" };

    let active = |p: &str| -> &'static str {
        if path == p || (p != "/" && path.starts_with(p)) { "active" } else { "" }
    };

    let nav_links = [
        ("/", "gauge-high", "Dashboard"),
        ("/users", "users", "Users"),
        ("/payments", "credit-card", "Payments"),
        ("/subscriptions", "vault", "Subscriptions"),
        ("/content", "newspaper", "Content"),
        ("/analytics", "chart-line", "Analytics"),
        ("/settings", "gear", "Settings"),
        ("/audit", "list-check", "Audit Log"),
    ];
    let mut links_html = String::new();
    for (href, icon, label) in nav_links {
        let cls = active(href);
        links_html.push_str(&format!(
            r#"<a href="{href}" class="nav-link {cls}" style="width:100%;justify-content:flex-start;padding:0.625rem 0.75rem;">
              <i data-lucide="{icon}" style="color:var(--epsx-orange);width:1.25rem;height:1.25rem;"></i>
              <span>{label}</span>
            </a>"#,
        ));
    }

    let body = format!(
        r##"<div style="display:flex;min-height:calc(100vh - 3.5rem);">
  <aside class="admin-sidebar" style="width:16rem;background:var(--bg-secondary);border-right:1px solid var(--border);padding:1.5rem 1rem;display:flex;flex-direction:column;gap:1rem;">
    <div style="display:flex;align-items:center;gap:0.5rem;padding:0 0.5rem 0.5rem;border-bottom:1px solid var(--border);">
      {logo}
      <span class="badge badge-primary">Admin</span>
    </div>
    <nav style="display:flex;flex-direction:column;gap:0.25rem;">{links}</nav>
    <div style="margin-top:auto;padding:0.75rem;border-top:1px solid var(--border);font-size:0.75rem;color:var(--text-subtle);">
      <i data-lucide="shield" style="color:var(--epsx-orange);"></i> Secured by EPSX
    </div>
  </aside>
  <main style="flex:1;padding:2rem;overflow:auto;">
    <div style="display:flex;justify-content:space-between;align-items:flex-end;flex-wrap:wrap;gap:1rem;margin-bottom:2rem;">
      <div>
        <span class="badge-pill"><i data-lucide="shield" style="color:var(--epsx-orange);"></i> Admin</span>
        <h1 style="font-size:2rem;font-weight:800;margin-top:0.75rem;">{title}</h1>
      </div>
      <div id="auth-pill" style="display:flex;align-items:center;gap:0.5rem;"></div>
    </div>
    <div id="app" data-route="{path}">Loading...</div>
  </main>
</div>
<script>
const route = document.getElementById('app').dataset.route;
let accessToken = localStorage.getItem('epsx_admin_token') || '';
let currentUser = null;
async function authedFetch(url, opts = {{}}) {{
  opts.headers = {{ 'content-type': 'application/json', ...(opts.headers || {{}}) }};
  if (accessToken) opts.headers.authorization = 'Bearer ' + accessToken;
  let r = await fetch(url, opts);
  if (r.status === 401) {{
    const refreshed = await tryRefresh();
    if (refreshed) {{
      opts.headers.authorization = 'Bearer ' + accessToken;
      r = await fetch(url, opts);
    }}
  }}
  return r;
}}
async function tryRefresh() {{
  const rt = localStorage.getItem('epsx_admin_refresh');
  if (!rt) return false;
  const r = await fetch('/api/v1/auth/refresh', {{ method: 'POST', headers: {{'content-type':'application/json'}}, body: JSON.stringify({{refresh_token: rt}}) }});
  if (!r.ok) return false;
  const j = await r.json();
  accessToken = j.access_token;
  localStorage.setItem('epsx_admin_token', j.access_token);
  return true;
}}
async function loadMe() {{
  const r = await authedFetch('/api/v1/auth/me');
  if (r.ok) currentUser = await r.json();
  document.getElementById('auth-pill').innerHTML = currentUser
    ? `<span style="font-size:0.75rem;color:var(--text-muted);">${{currentUser.address?.slice(0,8)}}...</span><button onclick="logout()" style="font-size:0.75rem;padding:0.25rem 0.5rem;border:1px solid var(--border);background:transparent;color:var(--text-muted);border-radius:0.25rem;cursor:pointer;">Logout</button>`
    : `<button onclick="demoLogin()" style="font-size:0.75rem;padding:0.25rem 0.75rem;background:var(--epsx-orange);color:#000;border:none;border-radius:0.25rem;cursor:pointer;font-weight:600;">Demo Login</button>`;
}}
function logout() {{ localStorage.removeItem('epsx_admin_token'); localStorage.removeItem('epsx_admin_refresh'); accessToken = ''; location.reload(); }}
async function demoLogin() {{
  const r = await fetch('/api/v1/auth/demo', {{ method: 'POST', headers: {{'content-type':'application/json'}}, body: JSON.stringify({{}}) }});
  if (!r.ok) return alert('Demo login disabled');
  const j = await r.json();
  accessToken = j.access_token;
  localStorage.setItem('epsx_admin_token', j.access_token);
  localStorage.setItem('epsx_admin_refresh', j.refresh_token);
  loadMe(); load();
}}

function pill(text, kind) {{
  const colors = {{ success: 'var(--epsx-green)', danger: 'var(--epsx-red)', warning: 'var(--epsx-yellow)', info: 'var(--epsx-blue-start)', primary: 'var(--epsx-orange)' }};
  return `<span class="badge" style="background:${{colors[kind] || colors.primary}}20;color:${{colors[kind] || colors.primary}};">${{text}}</span>`;
}}

async function load() {{
  if (!accessToken) {{
    document.getElementById('app').innerHTML = `<div class="card-insight" style="text-align:center;padding:3rem;"><i data-lucide="log-in" style="color:var(--epsx-orange);width:2rem;height:2rem;"></i><p style="margin-top:1rem;color:var(--text-muted);">Click "Demo Login" in the top-right to access the admin dashboard.</p></div>`;
    if (window.lucide) lucide.createIcons();
    return;
  }}
  if (route === '/') {{
    const [u, s, p, rev, e] = await Promise.all([
      authedFetch('/api/v1/users').then(r => r.json()).catch(() => ({{ users: [] }})),
      authedFetch('/api/v1/subscriptions').then(r => r.json()).catch(() => []),
      authedFetch('/api/v1/payments').then(r => r.json()).catch(() => []),
      authedFetch('/api/v1/analytics/revenue').then(r => r.json()).catch(() => ({{}})),
      authedFetch('/api/v1/wallet/accounts').then(r => r.json()).catch(() => []),
    ]);
    const users = u.users || u || [];
    const subs = s || [];
    const payments = p || [];
    const accounts = e || [];
    const revAmount = rev.total ?? rev.amount ?? 0;
    document.getElementById('app').innerHTML = `
      <div style="display:grid;grid-template-columns:repeat(auto-fit, minmax(200px, 1fr));gap:1.5rem;">
        <div class="card-insight">
          <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:0.5rem;">
            <span style="font-size:0.875rem;color:var(--text-muted);">Total Users</span>
            <i data-lucide="users" style="color:var(--epsx-blue-start);width:1.25rem;height:1.25rem;"></i>
          </div>
          <div style="font-size:2rem;font-weight:800;">${{users.length}}</div>
        </div>
        <div class="card-insight">
          <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:0.5rem;">
            <span style="font-size:0.875rem;color:var(--text-muted);">Revenue</span>
            <i data-lucide="dollar-sign" style="color:var(--epsx-green);width:1.25rem;height:1.25rem;"></i>
          </div>
          <div class="gradient-text" style="font-size:2rem;font-weight:800;">${{revAmount}}</div>
        </div>
        <div class="card-insight">
          <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:0.5rem;">
            <span style="font-size:0.875rem;color:var(--text-muted);">Active Subs</span>
            <i data-lucide="shield" style="color:var(--epsx-purple);width:1.25rem;height:1.25rem;"></i>
          </div>
          <div style="font-size:2rem;font-weight:800;color:var(--epsx-purple);">${{subs.length}}</div>
        </div>
        <div class="card-insight">
          <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:0.5rem;">
            <span style="font-size:0.875rem;color:var(--text-muted);">Payments</span>
            <i data-lucide="credit-card" style="color:var(--epsx-orange);width:1.25rem;height:1.25rem;"></i>
          </div>
          <div style="font-size:2rem;font-weight:800;color:var(--epsx-orange);">${{payments.length}}</div>
        </div>
        <div class="card-insight">
          <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:0.5rem;">
            <span style="font-size:0.875rem;color:var(--text-muted);">Wallet Accounts</span>
            <i data-lucide="wallet" style="color:var(--epsx-cyan);width:1.25rem;height:1.25rem;"></i>
          </div>
          <div style="font-size:2rem;font-weight:800;color:var(--epsx-cyan);">${{accounts.length}}</div>
        </div>
      </div>`;
  }} else if (route === '/users') {{
    const r = await authedFetch('/api/v1/users');
    if (!r.ok) {{ document.getElementById('app').innerHTML = `<div class="card-insight" style="color:var(--epsx-red);padding:2rem;">Failed: ${{r.status}} (admin role required)</div>`; return; }}
    const u = await r.json();
    const list = u.users || u || [];
    document.getElementById('app').innerHTML = `
      <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:1rem;">
        <input id="search" placeholder="Search by address..." style="flex:1;max-width:20rem;padding:0.5rem 0.75rem;background:var(--bg-secondary);border:1px solid var(--border);color:var(--text);border-radius:0.375rem;" oninput="renderUsers()">
        <button onclick="newUser()" style="padding:0.5rem 1rem;background:var(--epsx-orange);color:#000;border:none;border-radius:0.375rem;cursor:pointer;font-weight:600;">+ New User</button>
      </div>
      <div id="user-table" class="table-wrap"><table class="table">
        <thead><tr><th>Address</th><th>Roles</th><th>Chain</th><th>Created</th><th>Actions</th></tr></thead>
        <tbody>${{list.map(x => `<tr>
          <td style="font-family:monospace;">${{(x.address||'').slice(0,10)}}...${{(x.address||'').slice(-6)}}</td>
          <td>${{(x.roles||[]).map(r => pill(r, 'primary')).join(' ')}}</td>
          <td>${{x.chain_id}}</td>
          <td style="color:var(--text-subtle);">${{(x.created_at||'').slice(0,10)}}</td>
          <td><button onclick="editUser('${{x.id}}')" style="font-size:0.75rem;padding:0.25rem 0.5rem;background:transparent;color:var(--epsx-blue-start);border:1px solid var(--epsx-blue-start);border-radius:0.25rem;cursor:pointer;">Edit</button>
              <button onclick="delUser('${{x.id}}')" style="font-size:0.75rem;padding:0.25rem 0.5rem;background:transparent;color:var(--epsx-red);border:1px solid var(--epsx-red);border-radius:0.25rem;cursor:pointer;">Delete</button></td>
        </tr>`).join('') || '<tr><td colspan="5" style="text-align:center;color:var(--text-muted);">No users yet</td></tr>'}}</tbody>
      </table></div>`;
  }} else if (route === '/payments') {{
    const r = await authedFetch('/api/v1/payments');
    const p = r.ok ? await r.json() : [];
    const list = p || [];
    document.getElementById('app').innerHTML = `
      <div class="table-wrap"><table class="table">
        <thead><tr><th>Intent ID</th><th>Amount</th><th>Token</th><th>Status</th><th>Created</th><th>Actions</th></tr></thead>
        <tbody>${{list.map(x => `<tr>
          <td style="font-family:monospace;">${{(x.id||'').slice(0,10)}}...</td>
          <td style="text-align:right;font-weight:600;">${{x.amount}}</td>
          <td>${{x.token || 'USDT'}}</td>
          <td>${{pill(x.status||'pending', x.status==='confirmed'?'success':x.status==='failed'||x.status==='cancelled'?'danger':'warning')}}</td>
          <td style="color:var(--text-subtle);">${{(x.created_at||'').slice(0,10)}}</td>
          <td>${{x.status === 'pending' ? `<button onclick="payAction('${{x.id}}','confirm')" style="font-size:0.75rem;padding:0.25rem 0.5rem;background:transparent;color:var(--epsx-green);border:1px solid var(--epsx-green);border-radius:0.25rem;cursor:pointer;">Confirm</button> <button onclick="payAction('${{x.id}}','cancel')" style="font-size:0.75rem;padding:0.25rem 0.5rem;background:transparent;color:var(--epsx-red);border:1px solid var(--epsx-red);border-radius:0.25rem;cursor:pointer;">Cancel</button>` : '-'}}</td>
        </tr>`).join('') || '<tr><td colspan="6" style="text-align:center;color:var(--text-muted);">No payments yet</td></tr>'}}</tbody>
      </table></div>`;
  }} else if (route === '/subscriptions') {{
    const [s, p] = await Promise.all([
      authedFetch('/api/v1/subscriptions').then(r => r.json()).catch(() => []),
      authedFetch('/api/v1/subscription/plans').then(r => r.json()).catch(() => []),
    ]);
    const subs = s || [];
    const plans = p || [];
    document.getElementById('app').innerHTML = `
      <div style="display:grid;grid-template-columns:1fr 1fr;gap:1.5rem;">
        <section class="card-insight">
          <h3 style="font-weight:600;margin-bottom:0.75rem;display:flex;justify-content:space-between;align-items:center;">
            <span>Active Subscriptions</span>
            <span class="badge badge-primary">${{subs.length}}</span>
          </h3>
          <div class="table-wrap"><table class="table">
            <thead><tr><th>Sub</th><th>Plan</th><th>Status</th><th>Actions</th></tr></thead>
            <tbody>${{subs.map(x => `<tr>
              <td style="font-family:monospace;">${{(x.id||'').slice(0,8)}}</td>
              <td style="font-family:monospace;">${{(x.plan_id||'').slice(0,8)}}</td>
              <td>${{pill(x.status||'pending', x.status==='active'?'active':x.status==='cancelled'?'danger':'warning')}}</td>
              <td>${{x.status==='active' ? `<button onclick="subCancel('${{x.id}}')" style="font-size:0.75rem;padding:0.25rem 0.5rem;background:transparent;color:var(--epsx-red);border:1px solid var(--epsx-red);border-radius:0.25rem;cursor:pointer;">Cancel</button>` : '-'}}</td>
            </tr>`).join('') || '<tr><td colspan="4" style="text-align:center;color:var(--text-muted);">No subs</td></tr>'}}</tbody>
          </table></div>
        </section>
        <section class="card-insight">
          <h3 style="font-weight:600;margin-bottom:0.75rem;display:flex;justify-content:space-between;align-items:center;">
            <span>Plans</span>
            <button onclick="newPlan()" style="font-size:0.75rem;padding:0.25rem 0.5rem;background:var(--epsx-orange);color:#000;border:none;border-radius:0.25rem;cursor:pointer;">+ New</button>
          </h3>
          <div class="table-wrap"><table class="table">
            <thead><tr><th>Name</th><th>Amount</th><th>Interval</th></tr></thead>
            <tbody>${{plans.map(x => `<tr>
              <td>${{x.name}}</td>
              <td>${{x.amount}} ${{x.currency||''}}</td>
              <td>${{x.interval_secs ? Math.round(x.interval_secs/86400)+'d' : (x.interval ? Math.round(x.interval/86400)+'d' : '-')}}</td>
            </tr>`).join('') || '<tr><td colspan="3" style="text-align:center;color:var(--text-muted);">No plans</td></tr>'}}</tbody>
          </table></div>
        </section>
      </div>`;
  }} else if (route === '/content') {{
    const [p, b, t, n, s] = await Promise.all([
      authedFetch('/api/v1/pages').then(r => r.json()).catch(() => []),
      authedFetch('/api/v1/blocks').then(r => r.json()).catch(() => []),
      authedFetch('/api/v1/themes').then(r => r.json()).catch(() => []),
      authedFetch('/api/v1/content/navigation').then(r => r.json()).catch(() => null),
      authedFetch('/api/v1/content/site').then(r => r.json()).catch(() => null),
    ]);
    const pages = p || [];
    const blocks = b || [];
    const themes = t || [];
    document.getElementById('app').innerHTML = `
      <div style="display:grid;grid-template-columns:repeat(auto-fit, minmax(280px, 1fr));gap:1.5rem;">
        <section class="card-insight">
          <h3 style="font-weight:600;margin-bottom:0.75rem;display:flex;justify-content:space-between;align-items:center;">
            <span>Pages</span>
            <span class="badge badge-primary">${{pages.length}}</span>
          </h3>
          <ul style="list-style:none;padding:0;margin:0;display:flex;flex-direction:column;gap:0.5rem;font-size:0.875rem;">
            ${{pages.map(x => `<li style="padding:0.5rem;background:var(--bg-secondary);border-radius:0.375rem;display:flex;justify-content:space-between;align-items:center;">
              <span>${{x.slug}}</span>
              <span>${{pill(x.status || 'draft', x.status === 'published' ? 'success' : 'warning')}}</span>
            </li>`).join('') || '<li style="color:var(--text-muted);">No pages</li>'}}
          </ul>
        </section>
        <section class="card-insight">
          <h3 style="font-weight:600;margin-bottom:0.75rem;display:flex;justify-content:space-between;align-items:center;">
            <span>Block Types</span>
            <span class="badge badge-info">${{blocks.length}}</span>
          </h3>
          <ul style="list-style:none;padding:0;margin:0;display:flex;flex-direction:column;gap:0.5rem;font-size:0.875rem;">
            ${{blocks.map(x => `<li style="padding:0.5rem;background:var(--bg-secondary);border-radius:0.375rem;display:flex;justify-content:space-between;"><span>${{x.name || x.id}}</span><span style="color:var(--text-subtle);">${{x.category || ''}}</span></li>`).join('') || '<li style="color:var(--text-muted);">No blocks</li>'}}
          </ul>
        </section>
        <section class="card-insight">
          <h3 style="font-weight:600;margin-bottom:0.75rem;display:flex;justify-content:space-between;align-items:center;">
            <span>Themes</span>
            <span class="badge badge-purple">${{themes.length}}</span>
          </h3>
          <ul style="list-style:none;padding:0;margin:0;display:flex;flex-direction:column;gap:0.5rem;font-size:0.875rem;">
            ${{themes.map(x => `<li style="padding:0.5rem;background:var(--bg-secondary);border-radius:0.375rem;display:flex;justify-content:space-between;"><span>${{x.name}}</span>${{x.is_default ? '<span class="badge badge-primary" style="font-size:0.625rem;">default</span>' : ''}}</li>`).join('') || '<li style="color:var(--text-muted);">No themes</li>'}}
          </ul>
        </section>
        <section class="card-insight">
          <h3 style="font-weight:600;margin-bottom:0.75rem;">Site Settings</h3>
          <pre style="font-size:0.75rem;color:var(--text-muted);white-space:pre-wrap;word-break:break-all;">${{s ? JSON.stringify(s, null, 2) : 'Not loaded'}}</pre>
        </section>
        <section class="card-insight" style="grid-column:1 / -1;">
          <h3 style="font-weight:600;margin-bottom:0.75rem;">Navigation</h3>
          <pre style="font-size:0.75rem;color:var(--text-muted);white-space:pre-wrap;">${{n ? JSON.stringify(n, null, 2) : 'Not loaded'}}</pre>
        </section>
      </div>`;
  }} else if (route === '/analytics') {{
    const [rev, ev, m] = await Promise.all([
      authedFetch('/api/v1/analytics/revenue').then(r => r.json()).catch(() => ({{}})),
      authedFetch('/api/v1/analytics/events').then(r => r.json()).catch(() => []),
      authedFetch('/api/v1/analytics/metrics/revenue').then(r => r.json()).catch(() => null),
    ]);
    const events = ev.events || ev || [];
    const revAmount = rev.total ?? rev.amount ?? 0;
    document.getElementById('app').innerHTML = `
      <div style="display:grid;grid-template-columns:repeat(auto-fit, minmax(200px, 1fr));gap:1.5rem;">
        <div class="card-insight"><div style="font-size:0.875rem;color:var(--text-muted);margin-bottom:0.5rem;">Revenue</div><div class="gradient-text" style="font-size:2rem;font-weight:800;">${{revAmount}}</div></div>
        <div class="card-insight"><div style="font-size:0.875rem;color:var(--text-muted);margin-bottom:0.5rem;">Events</div><div style="font-size:2rem;font-weight:800;">${{events.length}}</div></div>
        <div class="card-insight"><div style="font-size:0.875rem;color:var(--text-muted);margin-bottom:0.5rem;">Conversion</div><div style="font-size:2rem;font-weight:800;color:var(--epsx-green);">${{m?.rate || '-'}}</div></div>
      </div>
      <h3 style="font-weight:600;margin:2rem 0 1rem;">Recent Events</h3>
      <div class="table-wrap"><table class="table">
        <thead><tr><th>Name</th><th>User</th><th>Properties</th><th>Time</th></tr></thead>
        <tbody>${{events.slice(0,20).map(e => `<tr>
          <td>${{pill(e.name||e.event, 'info')}}</td>
          <td style="font-family:monospace;">${{(e.user_id||'').slice(0,10)}}</td>
          <td style="font-size:0.75rem;color:var(--text-muted);">${{JSON.stringify(e.properties||e.props||{}).slice(0,80)}}</td>
          <td style="color:var(--text-subtle);font-size:0.75rem;">${{(e.timestamp||e.created_at||'').slice(0,19)}}</td>
        </tr>`).join('') || '<tr><td colspan="4" style="text-align:center;color:var(--text-muted);">No events</td></tr>'}}</tbody>
      </table></div>`;
  }} else if (route === '/settings') {{
    const [bsc, bsct] = await Promise.all([
      authedFetch('/api/v1/indexer/status/56').then(r => r.json()).catch(() => null),
      authedFetch('/api/v1/indexer/status/97').then(r => r.json()).catch(() => null),
    ]);
    document.getElementById('app').innerHTML = `
      <div class="card-insight">
        <h3 style="font-weight:600;margin-bottom:1rem;">System</h3>
        <dl style="display:grid;gap:0.75rem;font-size:0.875rem;">
          <div style="display:flex;justify-content:space-between;padding:0.5rem 0;border-bottom:1px solid var(--border);"><dt style="color:var(--text-muted);">Environment</dt><dd>production</dd></div>
          <div style="display:flex;justify-content:space-between;padding:0.5rem 0;border-bottom:1px solid var(--border);"><dt style="color:var(--text-muted);">Region</dt><dd>local</dd></div>
          <div style="display:flex;justify-content:space-between;padding:0.5rem 0;border-bottom:1px solid var(--border);"><dt style="color:var(--text-muted);">Database</dt><dd>PostgreSQL</dd></div>
          <div style="display:flex;justify-content:space-between;padding:0.5rem 0;border-bottom:1px solid var(--border);"><dt style="color:var(--text-muted);">Cache</dt><dd>Redis</dd></div>
          <div style="display:flex;justify-content:space-between;padding:0.5rem 0;border-bottom:1px solid var(--border);"><dt style="color:var(--text-muted);">BSC Mainnet Indexer</dt><dd>${{pill(bsc?.status || 'unknown', bsc?.status==='synced'?'success':'warning')}}</dd></div>
          <div style="display:flex;justify-content:space-between;padding:0.5rem 0;"><dt style="color:var(--text-muted);">BSC Testnet Indexer</dt><dd>${{pill(bsct?.status || 'unknown', bsct?.status==='synced'?'success':'warning')}}</dd></div>
        </dl>
      </div>`;
  }} else if (route === '/audit') {{
    const r = await authedFetch('/api/v1/analytics/events');
    const events = r.ok ? (await r.json()).events || (await r.json()) : [];
    document.getElementById('app').innerHTML = `
      <div class="table-wrap"><table class="table">
        <thead><tr><th>Time</th><th>Event</th><th>User</th><th>Properties</th></tr></thead>
        <tbody>${{events.slice(0,50).map(e => `<tr>
          <td style="color:var(--text-subtle);font-size:0.75rem;">${{(e.timestamp||e.created_at||'').slice(0,19)}}</td>
          <td>${{pill(e.name||e.event, 'info')}}</td>
          <td style="font-family:monospace;font-size:0.75rem;">${{(e.user_id||'').slice(0,10)}}</td>
          <td style="font-size:0.75rem;color:var(--text-muted);">${{JSON.stringify(e.properties||e.props||{}).slice(0,100)}}</td>
        </tr>`).join('') || '<tr><td colspan="4" style="text-align:center;color:var(--text-muted);">No events</td></tr>'}}</tbody>
      </table></div>`;
  }}
  if (window.lucide) lucide.createIcons();
}}

async function newUser() {{
  const address = prompt('Ethereum address (0x + 40 hex):');
  if (!address) return;
  const chain_id = prompt('Chain ID (56 for BSC mainnet, 97 for testnet):', '56') || '56';
  const r = await authedFetch('/api/v1/users', {{ method: 'POST', body: JSON.stringify({{ address, chain_id }}) }});
  if (r.ok) {{ load(); alert('User created'); }} else alert('Failed: ' + r.status);
}}
async function editUser(id) {{
  const roles = prompt('Comma-separated roles (user, editor, content_manager, designer, merchant, admin):');
  if (!roles) return;
  const r = await authedFetch('/api/v1/users/' + id, {{ method: 'PUT', body: JSON.stringify({{ roles: roles.split(',').map(s => s.trim()) }}) }});
  if (r.ok) load(); else alert('Failed: ' + r.status);
}}
async function delUser(id) {{
  if (!confirm('Delete user ' + id + '?')) return;
  const r = await authedFetch('/api/v1/users/' + id, {{ method: 'DELETE' }});
  if (r.ok) load(); else alert('Failed: ' + r.status);
}}

async function payAction(id, action) {{
  const r = await authedFetch('/api/v1/payments/' + id + '/' + action, {{ method: 'POST', body: JSON.stringify({{}}) }});
  if (r.ok) load(); else alert('Failed: ' + r.status);
}}

async function subCancel(id) {{
  if (!confirm('Cancel subscription ' + id + '?')) return;
  const r = await authedFetch('/api/v1/subscriptions/' + id + '/cancel', {{ method: 'POST', body: JSON.stringify({{}}) }});
  if (r.ok) load(); else alert('Failed: ' + r.status);
}}
async function newPlan() {{
  const name = prompt('Plan name:'); if (!name) return;
  const amount = prompt('Amount in token units (e.g. 1000000 for 1 USDT):'); if (!amount) return;
  const currency = prompt('Currency (USDT/USDC):', 'USDT') || 'USDT';
  const chain_id = prompt('Chain ID:', '56') || '56';
  const interval = prompt('Interval in seconds (2592000 = 30 days):', '2592000') || '2592000';
  const merchant_id = prompt('Merchant ID (0x address or merchant code):', '0x' + '0'.repeat(40)) || '';
  const r = await authedFetch('/api/v1/subscription/plans', {{ method: 'POST', body: JSON.stringify({{ name, amount, currency, chain_id, interval: parseInt(interval), merchant_id }}) }});
  if (r.ok) {{ load(); alert('Plan created'); }} else alert('Failed: ' + r.status);
}}

loadMe();
load();
</script>"##,
        title = title,
        path = path,
        logo = logo("/", "sm"),
        links = links_html,
    );

    let nav = format!(
        r#"<nav class="navbar"><div class="container-x flex items-center justify-between" style="height:3.5rem;">
  {logo}
  <div class="flex items-center gap-2">
    {toggle}
  </div>
</div></nav>"#,
        logo = logo("/", "sm"),
        toggle = theme_toggle_button(),
    );

    let full_title = format!("{} - EPSX Admin", title);
    page_shell_with_body_class(&full_title, "EPSX Admin Dashboard", &nav, &body, false, "page-bg")
}

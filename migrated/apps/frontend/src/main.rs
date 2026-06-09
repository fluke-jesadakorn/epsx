use axum::{
    extract::{Path as AxPath, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{any, get, post},
    Json, Router,
};
use epsx_client::ServiceClient;
use epsx_templates::{epsx_header, page_shell_with_body_class};
use epsx_templates::components::{Badge, Btn, Card, StatCard, BadgeKind};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;

mod ui;
mod pages;
mod playground;
mod auth;
mod widgets;
use ui::render_navbar;
use pages::*;
use auth::{current_user, build_set_cookie, build_clear_cookie, get_cookie, AuthUser};

#[derive(Clone)]
#[allow(dead_code)]
struct AppState {
    identity: Arc<ServiceClient>,
    notification: Arc<ServiceClient>,
    content: Arc<ServiceClient>,
    analytics: Arc<ServiceClient>,
    api_url: String,
    demo_login_enabled: bool,
}

#[derive(Deserialize)]
struct SavePageBody {
    title: Option<String>,
    blocks: Option<serde_json::Value>,
    seo: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct SiweLoginBody {
    message: String,
    signature: String,
    chain_id: String,
}

#[derive(Deserialize)]
struct DemoLoginBody {
    address: Option<String>,
    chain_id: Option<String>,
}

#[derive(Serialize)]
struct AuthApiResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
    user: serde_json::Value,
    demo: bool,
}

#[derive(Deserialize)]
struct AnalyticsTrackBody {
    event_name: String,
    properties: Option<serde_json::Value>,
    user_id: Option<String>,
    chain_id: Option<String>,
}

#[tokio::main]
async fn main() {
    epsx_observability::Observability::init("bff-frontend");

    let api_url = std::env::var("API_URL").unwrap_or_else(|_| "http://localhost:18081".to_string());
    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(3000);
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let demo_login_enabled = std::env::var("EPSX_ENABLE_DEMO_LOGIN").ok().as_deref() == Some("1");

    let cfg = epsx_client::ClientConfig { base_url: api_url.clone(), timeout: std::time::Duration::from_secs(15) };
    let state = AppState {
        identity: Arc::new(ServiceClient::new(cfg.clone())),
        notification: Arc::new(ServiceClient::new(cfg.clone())),
        content: Arc::new(ServiceClient::new(cfg.clone())),
        analytics: Arc::new(ServiceClient::new(cfg.clone())),
        api_url,
        demo_login_enabled,
    };

    let app = Router::new()
        .route("/api/health", get(api_health))
        .route("/api/v1/pages/{slug}", any(get_page))
        .route("/api/v1/edit/{slug}/save", any(save_page))
        .route("/api/v1/edit/{slug}/publish", any(publish_page))
        .route("/api/v1/auth/siwe", post(siwe_login))
        .route("/api/v1/auth/demo", post(demo_login))
        .route("/api/v1/auth/refresh", post(refresh_token))
        .route("/api/v1/auth/logout", post(logout))
        .route("/api/v1/auth/me", get(auth_me))
        .route("/api/v1/notifications", any(notifications_api))
        .route("/api/v1/notifications/{id}/read", post(notification_read))
        .route("/api/v1/notifications/{id}/delete", post(notification_delete))
        .route("/api/v1/notifications/mark-all-read", post(notification_mark_all))
        .route("/api/v1/notifications/clear-all", post(notification_clear_all))
        .route("/api/v1/analytics/track", post(track_event))
        .route("/api/v1/rankings", get(api_rankings))
        .route("/api/v1/plans", get(api_plans))
        .route("/api/v1/news", get(api_news))
        .route("/api/v1/portfolio/{addr}", get(api_portfolio))
        .route("/api/v1/news/{slug}", get(api_news_post))
        .fallback(ssr_fallback)
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();
    tracing::info!("Frontend BFF listening on http://{} (api={})", addr, std::env::var("API_URL").unwrap_or_default());
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn api_health() -> &'static str { "ok" }

// =====================================================================
// JSON API endpoints (proxied to gateway)
// =====================================================================

async fn get_page(
    State(state): State<AppState>,
    AxPath(slug): AxPath<String>,
) -> Result<Response, StatusCode> {
    let path = format!("/api/v1/content/pages/{}", slug);
    state.content.get_plain(&path).await
        .map(|v| Json(v).into_response())
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn save_page(
    State(state): State<AppState>,
    AxPath(slug): AxPath<String>,
    Json(body): Json<SavePageBody>,
) -> Result<Response, StatusCode> {
    let path = format!("/api/v1/content/pages/{}", slug);
    let payload = serde_json::json!({
        "title": body.title,
        "blocks_json": body.blocks.map(|v| v.to_string()).unwrap_or_else(|| "[]".to_string()),
        "seo_json": body.seo.map(|v| v.to_string()).unwrap_or_else(|| "{}".to_string()),
    });
    state.content.put_plain(&path, &payload).await
        .map(|v| Json(v).into_response())
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn publish_page(
    State(state): State<AppState>,
    AxPath(slug): AxPath<String>,
) -> Result<Response, StatusCode> {
    let path = format!("/api/v1/content/pages/{}/publish", slug);
    state.content.post_plain(&path, &serde_json::json!({})).await
        .map(|v| Json(v).into_response())
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn siwe_login(
    State(state): State<AppState>,
    _headers: HeaderMap,
    Json(body): Json<SiweLoginBody>,
) -> Result<Response, StatusCode> {
    let url = format!("{}/api/v1/identity/auth/siwe", state.api_url.trim_end_matches('/'));
    let resp = state.identity.clone_for_bearer()
        .post(&url)
        .json(&serde_json::json!({
            "message": body.message,
            "signature": body.signature,
            "chain_id": body.chain_id,
        }))
        .send().await.map_err(|e| { tracing::error!("siwe: {e}"); StatusCode::BAD_GATEWAY })?;
    let status = resp.status();
    let value: serde_json::Value = resp.json().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
    if !status.is_success() {
        return Ok((status, Json(value)).into_response());
    }
    let user_id = value.get("user").and_then(|u| u.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
    let address = value.get("user").and_then(|u| u.get("address")).and_then(|v| v.as_str()).unwrap_or("").to_string();
    let chain_id = value.get("user").and_then(|u| u.get("chain_id")).and_then(|v| v.as_str()).unwrap_or("56").to_string();
    let access = value.get("access_token").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let refresh = value.get("refresh_token").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let expires = value.get("expires_in").and_then(|v| v.as_u64()).unwrap_or(3600);

    let mut response = Json(AuthApiResponse {
        access_token: access.clone(),
        refresh_token: if refresh.is_empty() { None } else { Some(refresh.clone()) },
        expires_in: Some(expires),
        user: serde_json::json!({
            "id": user_id, "address": address, "chain_id": chain_id,
            "roles": value.get("user").and_then(|u| u.get("roles")).cloned().unwrap_or(serde_json::json!([])),
        }),
        demo: false,
    }).into_response();

    let cookie_max_age = expires as i64;
    let cookies = vec![
        build_set_cookie("epsx_token", &access, cookie_max_age),
        build_set_cookie("epsx_user_id", &user_id, cookie_max_age),
        build_set_cookie("epsx_user_address", &address, cookie_max_age),
    ];
    for c in cookies {
        if let Ok(v) = c.parse() {
            response.headers_mut().append("set-cookie", v);
        }
    }
    Ok(response)
}

async fn demo_login(
    State(state): State<AppState>,
    _headers: HeaderMap,
    body: Option<Json<DemoLoginBody>>,
) -> Result<Response, StatusCode> {
    if !state.demo_login_enabled {
        return Err(StatusCode::NOT_FOUND);
    }
    let address = body.as_ref().and_then(|b| b.address.clone())
        .unwrap_or_else(|| "0xDEMO0000000000000000000000000000000000".to_string());
    let chain_id = body.as_ref().and_then(|b| b.chain_id.clone())
        .unwrap_or_else(|| "56".to_string());
    let payload = serde_json::json!({ "address": address, "chain_id": chain_id });

    let url = format!("{}/api/v1/identity/auth/demo", state.api_url.trim_end_matches('/'));
    let resp = state.identity.clone_for_bearer()
        .post(&url)
        .json(&payload)
        .send().await.map_err(|e| { tracing::error!("demo-login: {e}"); StatusCode::BAD_GATEWAY })?;
    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Ok((status, Json(serde_json::json!({"error": text}))).into_response());
    }
    let value: serde_json::Value = resp.json().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
    let user_id = value.get("user").and_then(|u| u.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
    let address = value.get("user").and_then(|u| u.get("address")).and_then(|v| v.as_str()).unwrap_or("").to_string();
    let chain_id = value.get("user").and_then(|u| u.get("chain_id")).and_then(|v| v.as_str()).unwrap_or("56").to_string();
    let access = value.get("access_token").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let refresh = value.get("refresh_token").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let expires = value.get("expires_in").and_then(|v| v.as_u64()).unwrap_or(3600);

    let mut response = Json(AuthApiResponse {
        access_token: access.clone(),
        refresh_token: if refresh.is_empty() { None } else { Some(refresh.clone()) },
        expires_in: Some(expires),
        user: serde_json::json!({
            "id": user_id, "address": address, "chain_id": chain_id,
            "roles": value.get("user").and_then(|u| u.get("roles")).cloned().unwrap_or(serde_json::json!([])),
        }),
        demo: true,
    }).into_response();
    let cookie_max_age = expires as i64;
    let cookies = vec![
        build_set_cookie("epsx_token", &access, cookie_max_age),
        build_set_cookie("epsx_user_id", &user_id, cookie_max_age),
        build_set_cookie("epsx_user_address", &address, cookie_max_age),
    ];
    for c in cookies {
        if let Ok(v) = c.parse() {
            response.headers_mut().append("set-cookie", v);
        }
    }
    Ok(response)
}

async fn refresh_token(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let url = format!("{}/api/v1/identity/auth/refresh", state.api_url.trim_end_matches('/'));
    let resp = state.identity.clone_for_bearer()
        .post(&url).json(&body).send().await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let status = resp.status();
    let value: serde_json::Value = resp.json().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
    Ok((status, Json(value)).into_response())
}

async fn logout() -> Response {
    let mut response = Json(serde_json::json!({"ok": true})).into_response();
    for name in &["epsx_token", "epsx_user_id", "epsx_user_address"] {
        if let Ok(v) = build_clear_cookie(name).parse() {
            response.headers_mut().append("set-cookie", v);
        }
    }
    response
}

async fn auth_me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let user = current_user(&state.identity, &headers).await
        .ok_or(StatusCode::UNAUTHORIZED)?;
    Ok(Json(serde_json::json!({
        "id": user.id,
        "address": user.address,
        "chain_id": user.chain_id,
        "roles": user.roles,
    })).into_response())
}

// =====================================================================
// Notification API (proxies to notification service with auth)
// =====================================================================

#[derive(Deserialize, Default)]
struct NotifListQuery {
    user_id: Option<String>,
    status: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
    page: Option<i64>,
}

async fn notifications_api(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<NotifListQuery>,
) -> Result<Response, StatusCode> {
    let token = auth::bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let user_id = q.user_id.clone()
        .or_else(|| get_cookie(&headers, "epsx_user_id"))
        .unwrap_or_else(|| "demo".to_string());
    let limit = q.limit.unwrap_or(50);
    let offset = q.offset.or_else(|| q.page.map(|p| (p - 1).max(0) * limit)).unwrap_or(0);
    let status_q = q.status.as_deref().map(|s| format!("&status={}", s)).unwrap_or_default();
    let url = format!("{}/api/v1/notification/list?user_id={}&limit={}&offset={}{}",
        state.api_url.trim_end_matches('/'), user_id, limit, offset, status_q);
    match auth::authed_get_json(&state.notification, &url, &token).await {
        Ok(v) => Ok(Json(v).into_response()),
        Err(_) => Ok(Json(serde_json::json!({"items": [], "total": 0, "error": "fetch failed"})).into_response()),
    }
}

async fn notification_read(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let token = auth::bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let url = format!("{}/api/v1/notification/{}/read", state.api_url.trim_end_matches('/'), id);
    match auth::authed_post_json(&state.notification, &url, &token, &serde_json::json!({})).await {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Ok((StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": format!("{e}")}))).into_response()),
    }
}

async fn notification_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let token = auth::bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let url = format!("{}/api/v1/notification/{}", state.api_url.trim_end_matches('/'), id);
    match auth::authed_delete(&state.notification, &url, &token).await {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Ok((StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": format!("{e}")}))).into_response()),
    }
}
async fn notification_mark_all(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let token = auth::bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let user_id = get_cookie(&headers, "epsx_user_id").unwrap_or_else(|| "demo".to_string());
    let url = format!("{}/api/v1/notification/mark-all-read?user_id={}",
        state.api_url.trim_end_matches('/'), user_id);
    match auth::authed_post_json(&state.notification, &url, &token, &serde_json::json!({})).await {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Ok((StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": format!("{e}")}))).into_response()),
    }
}

async fn notification_clear_all(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let token = auth::bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let user_id = get_cookie(&headers, "epsx_user_id").unwrap_or_else(|| "demo".to_string());
    let url = format!("{}/api/v1/notification/clear-all?user_id={}",
        state.api_url.trim_end_matches('/'), user_id);
    match auth::authed_post_json(&state.notification, &url, &token, &serde_json::json!({})).await {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Ok((StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": format!("{e}")}))).into_response()),
    }
}

async fn track_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AnalyticsTrackBody>,
) -> Result<Response, StatusCode> {
    let url = format!("{}/api/v1/analytics/track", state.api_url.trim_end_matches('/'));
    let user_id = body.user_id.clone().or_else(|| get_cookie(&headers, "epsx_user_id"));
    let payload = serde_json::json!({
        "event_name": body.event_name,
        "properties": body.properties,
        "user_id": user_id,
        "chain_id": body.chain_id,
    });
    let token = auth::bearer_token(&headers);
    let mut req = state.analytics.clone_for_bearer().post(&url).json(&payload);
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    let resp = req.send().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
    let status = resp.status();
    let value: serde_json::Value = resp.json().await.unwrap_or(serde_json::json!({}));
    Ok((status, Json(value)).into_response())
}

// =====================================================================
// Public JSON APIs (for client-side fetching of rankings, plans, news)
// =====================================================================

#[derive(Serialize)]
struct CompanyRanking {
    rank: u32,
    ticker: String,
    price: String,
    growth: String,
    growth_pct: f64,
    next_action_days: u32,
    next_action_pct: f64,
    tradingview_url: String,
}

async fn api_rankings() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "companies": [
            { "rank": 100, "ticker": "GHC",  "price": "$5.40",  "growth": "+4650.00%", "growth_pct": 4650.00, "next_action_days": 158, "next_action_pct": 5.0,    "tradingview_url": "https://www.tradingview.com/symbols/GHC" },
            { "rank": 101, "ticker": "6535", "price": "$462.00","growth": "+4622.84%", "growth_pct": 4622.84, "next_action_days": 1,   "next_action_pct": 98.89,  "tradingview_url": "https://www.tradingview.com/symbols/6535" },
            { "rank": 102, "ticker": "4657", "price": "$427.00","growth": "+4612.47%", "growth_pct": 4612.47, "next_action_days": 65,  "next_action_pct": 27.78,  "tradingview_url": "https://www.tradingview.com/symbols/4657" }
        ],
        "as_of": "2026-06-09T00:00:00Z",
        "total": 100
    }))
}

#[derive(Serialize)]
struct Plan {
    id: String,
    category: String,
    title: String,
    price: String,
    price_usd: f64,
    original_price: String,
    original_usd: f64,
    discount_pct: u32,
    savings: String,
    features: Vec<String>,
    badge: String,
    countdown_hours: u32,
    sale_active: bool,
}

async fn api_plans() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "personal": [
            {
                "id": "1day", "category": "personal", "title": "1 Day Package",
                "price": "$1", "price_usd": 1.0, "original_price": "$5", "original_usd": 5.0,
                "discount_pct": 80, "savings": "Save $4", "badge": "SALE",
                "countdown_hours": 24, "sale_active": true,
                "features": ["Basic analytics view", "Rankings from position 6+", "Basic trading features", "24-hour access", "Explore the platform"]
            },
            {
                "id": "1month", "category": "personal", "title": "1 Month Package",
                "price": "$9.9", "price_usd": 9.9, "original_price": "$99", "original_usd": 99.0,
                "discount_pct": 90, "savings": "Save $89.1", "badge": "SALE",
                "countdown_hours": 168, "sale_active": true,
                "features": ["Advanced analytics view", "25 stock rankings", "Basic analytic features", "Price alerts", "Email support", "30-day access"]
            },
            {
                "id": "lifetime", "category": "personal", "title": "Lifetime Package",
                "price": "$4999", "price_usd": 4999.0, "original_price": "$9999", "original_usd": 9999.0,
                "discount_pct": 50, "savings": "Save $5000", "badge": "SALE",
                "countdown_hours": 720, "sale_active": true,
                "features": ["Advanced analytics suite", "Full rankings access (Rank 1+)", "API read access", "Basic & Pro trading", "Priority support", "Lifetime access"]
            }
        ],
        "api": [
            {
                "id": "api-personal", "category": "api", "title": "API Personal",
                "price": "$999", "price_usd": 999.0, "original_price": "$3999", "original_usd": 3999.0,
                "discount_pct": 75, "savings": "Save $3000", "badge": "SALE",
                "countdown_hours": 360, "sale_active": true,
                "features": ["Analytics view access", "API read access", "Data export capability", "Full developer documentation", "30-day access"]
            },
            {
                "id": "api-company", "category": "api", "title": "API Company",
                "price": "$2999", "price_usd": 2999.0, "original_price": "$6999", "original_usd": 6999.0,
                "discount_pct": 57, "savings": "Save $4000", "badge": "SALE",
                "countdown_hours": 360, "sale_active": true,
                "features": ["Advanced analytics suite", "Full trading suite (Basic, Pro & Advanced)", "API read & write access", "Data export", "Notifications management", "365-day company access", "Dedicated support"]
            }
        ],
        "custom": [
            {
                "id": "revenue-share", "category": "custom", "title": "Custom",
                "price": "Revenue Share", "price_usd": 0.0, "original_price": "", "original_usd": 0.0,
                "discount_pct": 0, "savings": "Volume-based", "badge": "",
                "countdown_hours": 0, "sale_active": false,
                "features": ["Custom feature set & permissions", "Dedicated support & SLA", "Volume-based pricing", "Custom API rate limits", "White-label options", "Priority onboarding"]
            }
        ]
    }))
}

#[derive(Serialize)]
struct NewsArticle {
    slug: String,
    href: String,
    title: String,
    excerpt: String,
    date: String,
    tag1: String,
    tag2: String,
    featured: bool,
    image: Option<String>,
}

async fn api_news() -> Json<serde_json::Value> {
    let articles = vec![
        NewsArticle {
            slug: "strategic-roadmap-future".to_string(),
            href: "/news/strategic-roadmap-future".to_string(),
            title: "Strategic Roadmap and Future Capabilities".to_string(),
            excerpt: "A preview of upcoming system enhancements, including automated alerts and expanded analytical depth.".to_string(),
            date: "May 9, 2026".to_string(),
            tag1: "roadmap".to_string(),
            tag2: "strategy".to_string(),
            featured: true,
            image: Some("https://aullybia7vqndzva.public.blob.vercel-storage.com/news/strategic-roadmap-future.jpg".to_string()),
        },
        NewsArticle {
            slug: "enhanced-portfolio-management".to_string(),
            href: "/news/enhanced-portfolio-management".to_string(),
            title: "Enhanced Portfolio Management Solutions".to_string(),
            excerpt: "Our asset tracking capabilities have been integrated into a unified portfolio view for streamlined oversight.".to_string(),
            date: "April 30, 2026".to_string(),
            tag1: "portfolio".to_string(),
            tag2: "oversight".to_string(),
            featured: false,
            image: Some("https://aullybia7vqndzva.public.blob.vercel-storage.com/news/enhanced-portfolio-management.jpg".to_string()),
        },
        NewsArticle {
            slug: "integrated-service-solutions".to_string(),
            href: "/news/integrated-service-solutions".to_string(),
            title: "Integrated Service Solutions: Professional Tier Alignment".to_string(),
            excerpt: "Our refined service structure is designed to align with various professional requirements and organizational scales.".to_string(),
            date: "April 19, 2026".to_string(),
            tag1: "solutions".to_string(),
            tag2: "tier-update".to_string(),
            featured: false,
            image: None,
        },
        NewsArticle {
            slug: "proprietary-performance-metrics".to_string(),
            href: "/news/proprietary-performance-metrics".to_string(),
            title: "Proprietary Performance Metrics and Strategic Positioning".to_string(),
            excerpt: "An overview of our methodology for identifying market momentum and prioritizing growth indicators.".to_string(),
            date: "April 6, 2026".to_string(),
            tag1: "methodology".to_string(),
            tag2: "insights".to_string(),
            featured: false,
            image: None,
        },
        NewsArticle {
            slug: "strategic-launch-epsx".to_string(),
            href: "/news/strategic-launch-epsx".to_string(),
            title: "Strategic Launch of EPSX: Institutional-Grade Market Insights".to_string(),
            excerpt: "EPSX is now operational, providing streamlined access to essential market metrics and professional portfolio management.".to_string(),
            date: "March 23, 2026".to_string(),
            tag1: "announcement".to_string(),
            tag2: "business".to_string(),
            featured: false,
            image: None,
        },
        NewsArticle {
            slug: "optimizing-high-throughput-analytics-rust".to_string(),
            href: "/news/optimizing-high-throughput-analytics-rust".to_string(),
            title: "Strategic Analysis Performance for Operational Excellence".to_string(),
            excerpt: "How EPSX leverages high-performance data processing to deliver precise rankings and insights for business decision-making.".to_string(),
            date: "March 15, 2026".to_string(),
            tag1: "strategy".to_string(),
            tag2: "performance".to_string(),
            featured: false,
            image: Some("https://aullybia7vqndzva.public.blob.vercel-storage.com/news/strategic-analysis-performance.jpg".to_string()),
        },
        NewsArticle {
            slug: "real-time-intelligence".to_string(),
            href: "/news/real-time-intelligence".to_string(),
            title: "Real-Time Intelligence: Capturing Market Opportunities as They Happen".to_string(),
            excerpt: "The EPSX dashboard removes the gap between on-chain events and your decision-making. Learn how our Redis-powered real-time data pipeline keeps you at the absolute forefront of the market.".to_string(),
            date: "March 10, 2026".to_string(),
            tag1: "redis".to_string(),
            tag2: "real-time".to_string(),
            featured: false,
            image: None,
        },
        NewsArticle {
            slug: "securing-the-future".to_string(),
            href: "/news/securing-the-future".to_string(),
            title: "Securing the Future: Enterprise-Grade Trust in a Web3 World".to_string(),
            excerpt: "EPSX leads the industry in user privacy and security by adopting wallet-first authentication. Learn how our implementation of SIWE protects your assets and identity.".to_string(),
            date: "March 10, 2026".to_string(),
            tag1: "web3".to_string(),
            tag2: "security".to_string(),
            featured: false,
            image: None,
        },
        NewsArticle {
            slug: "scalable-foundation".to_string(),
            href: "/news/scalable-foundation".to_string(),
            title: "Built for Ambition: A Scalable Foundation for Global Analytics".to_string(),
            excerpt: "Scaling a global analytics platform requires an industrial-strength architecture. Discover how EPSX manages billions of time-series records with ease.".to_string(),
            date: "March 10, 2026".to_string(),
            tag1: "postgresql".to_string(),
            tag2: "database".to_string(),
            featured: false,
            image: None,
        },
        NewsArticle {
            slug: "smarter-decisions-ai".to_string(),
            href: "/news/smarter-decisions-ai".to_string(),
            title: "Smarter Decisions: How EPSX AI Navigates Market Complexity".to_string(),
            excerpt: "Leverage the power of artificial intelligence to filter out noise and surface high-signal trends. Discover how the unique EPSX Sentiment Score provides actionable market intelligence.".to_string(),
            date: "March 10, 2026".to_string(),
            tag1: "ai".to_string(),
            tag2: "machine-learning".to_string(),
            featured: false,
            image: None,
        },
    ];
    Json(serde_json::json!({ "articles": articles, "total": articles.len() }))
}

async fn api_news_post(AxPath(slug): AxPath<String>) -> Json<serde_json::Value> {
    use epsx_renderer::render_markdown;

    let mdx_name = pages::slug_to_mdx(&slug);
    let (title, body_html) = match mdx_name.and_then(|n| std::fs::read_to_string(pages::mdx_path(n)).ok()) {
        Some(raw) => {
            let (t, _, _) = pages::mdx_frontmatter(&raw);
            let body = pages::mdx_body(&raw);
            (t, render_markdown(body))
        }
        None => (slug.replace('-', " "), format!("<p>Article for <code>{slug}</code> coming soon.</p>")),
    };

    Json(serde_json::json!({
        "slug": slug,
        "title": title,
        "body": body_html,
        "published": "2026-06-09T00:00:00Z"
    }))
}

async fn api_portfolio(AxPath(addr): AxPath<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "address": addr,
        "total_value_usd": 0.0,
        "watchlist": [],
        "subscriptions": [],
        "transactions": [],
        "auth_required": true,
        "message": "Sign in to view your portfolio"
    }))
}

// =====================================================================
// SSR fallback
// =====================================================================

async fn ssr_fallback(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: axum::http::Uri,
) -> Response {
    let path = uri.path();
    let query = uri.query();
    let user = current_user(&state.identity, &headers).await;
    let html = render_page(&state, path, query, user.as_ref());
    Html(html).into_response()
}

fn percent_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'+' {
            out.push(' ');
        } else if bytes[i] == b'%' && i + 2 < bytes.len() {
            let h = (bytes[i+1] as char).to_digit(16);
            let l = (bytes[i+2] as char).to_digit(16);
            if let (Some(h), Some(l)) = (h, l) {
                if let Some(c) = char::from_u32((h * 16 + l) as u32) {
                    out.push(c);
                    i += 3;
                    continue;
                }
            }
            out.push(bytes[i] as char);
        } else {
            out.push(bytes[i] as char);
        }
        i += 1;
    }
    out
}

fn render_page(state: &AppState, path: &str, query: Option<&str>, user: Option<&AuthUser>) -> String {
    let get_q = |k: &str| -> Option<String> {
        let q = query?;
        for pair in q.split('&') {
            if let Some(idx) = pair.find('=') {
                if &pair[..idx] == k {
                    return Some(percent_decode(&pair[idx+1..]));
                }
            }
        }
        None
    };

    let user_id_for_data = user.map(|u| u.id.to_string()).unwrap_or_else(|| "demo".to_string());
    let is_authed = user.is_some();

    let (title, description, body, include_footer) = match path {
        "/" => ("EPSX - Web3 Analytics Platform",
                "Real-time blockchain data, analytics, and payments on BSC",
                home_body(),
                false),
        "/about" => ("About - EPSX",
                     "Learn about EPSX, our mission, and architecture",
                     about_body(),
                     true),
        "/pricing" => ("Pricing - EPSX",
                       "Simple, transparent pricing for everyone",
                       pricing_body(),
                       true),
        "/plans" => ("Plans - EPSX",
                     "Choose the plan that's right for you",
                     plans_body(),
                     true),
        "/blog" => ("Blog - EPSX",
                    "Latest news, deep dives, and product updates from the EPSX team",
                    blog_index_body(),
                    true),
        "/news" => ("News - EPSX",
                    "Latest announcements and product updates",
                    news_index_body(),
                    true),
        "/contact" => ("Contact - EPSX",
                       "Get in touch with the EPSX team",
                       contact_body(),
                       true),
        "/dashboard" => ("Dashboard - EPSX",
                         "Your portfolio, subscriptions, and transactions at a glance",
                         dashboard_body(is_authed, user.map(|u| u.display())),
                         true),
        "/portfolio" => ("Portfolio - EPSX",
                         "Track your watchlist and on-chain assets",
                         portfolio_body(),
                         true),
        "/rankings" => ("Rankings - EPSX",
                        "EPS stock rankings by growth — top 100+ assets",
                        pages::rankings_body(),
                        true),
        "/analytics" => ("Analytics - EPSX",
                         "Top-ranked stocks and tokens by EPS growth",
                         analytics_body(),
                         true),
        "/permissions" => ("Permissions - EPSX",
                           "Your active and historical permissions",
                           permissions_body(),
                           true),
        "/chat" => ("Support - EPSX",
                    "Chat with our support team",
                    chat_body(),
                    false),
        "/chat/history" => ("Chat History - EPSX",
                            "Your past support conversations",
                            chat_body(),
                            false),
        "/developer" => ("Developer - EPSX",
                         "API keys, usage, and documentation",
                         developer_overview_body(),
                         true),
        "/developer/docs" => ("API Documentation - EPSX",
                              "Complete API reference",
                              developer_docs_body(),
                              true),
        "/developer/usage" => ("API Usage - EPSX",
                               "Monitor your API consumption",
                               developer_usage_body(),
                               true),
        "/account" => ("Account - EPSX",
                       "Manage your wallet, plan, and preferences",
                       account_body(),
                       true),
        "/account/credits" => ("Credits - EPSX",
                               "Your credit balance and history",
                               account_credits_body(),
                               true),
        "/profile" => ("Profile - EPSX",
                       "Manage your account, wallet, and preferences",
                       profile_body(),
                       true),
        "/notifications" => ("Notifications - EPSX",
                             "Your account notifications",
                             pages::notifications_body_server(&user_id_for_data)
                                + &notifications_client_script(&user_id_for_data),
                             true),
        "/auth" => ("Sign In - EPSX",
                    "Sign in to your EPSX account",
                    auth_body(state.demo_login_enabled),
                    false),
        "/access-denied" => {
            let reason = get_q("reason").unwrap_or_else(|| "You do not have permission to access this page.".to_string());
            let route = get_q("route").unwrap_or_default();
            let required = if !route.is_empty() { format!("{}:access", route.trim_start_matches('/')) } else { String::new() };
            ("Access Denied - EPSX", "Access denied", access_denied_body(&reason, &required), true)
        }
        "/offline" => ("Offline - EPSX", "You are offline", offline_body(), false),
        "/payment" => {
            let plan = get_q("planId").or_else(|| get_q("plan"));
            ("Payment - EPSX", "Complete your payment", payment_body(plan.as_deref()), false)
        }
        "/terms" => ("Terms of Service - EPSX",
                     "EPSX platform terms of service",
                     legal_body("Terms of Service", "Last updated: January 2025",
                                "These terms govern your use of the EPSX platform..."),
                     true),
        "/privacy" => ("Privacy Policy - EPSX",
                       "EPSX platform privacy policy",
                       legal_body("Privacy Policy", "Last updated: January 2025",
                                  "This policy describes what data EPSX collects and how it is used..."),
                       true),
        p if p.starts_with("/blog/") => {
            let slug = p.trim_start_matches("/blog/");
            ("Blog Post - EPSX",
             "EPSX blog post",
             blog_post_body(slug),
             true)
        }
        p if p.starts_with("/news/") => {
            let slug = p.trim_start_matches("/news/");
            ("News - EPSX", "EPSX news article", news_post_body(slug), true)
        }
        p if p.starts_with("/chat/") => {
            ("Support - EPSX", "Support conversation", chat_body(), false)
        }
        p if p.starts_with("/edit/") => {
            let slug = p.trim_start_matches("/edit/");
            ("Edit - EPSX", "Visual page editor", edit_body(slug), false)
        }
        _ => ("Not Found - EPSX", "Page not found", not_found_body(), true),
    };

    let nav = if path == "/" {
        // Use the EPSX.io-style header on the home page
        epsx_header()
    } else {
        render_navbar(path, is_authed, user)
    };
    let is_marketing = matches!(path, "/" | "/about" | "/pricing" | "/blog" | "/contact" | "/news" | "/plans" | "/rankings") || path.starts_with("/rankings/");
    let body_class = if path == "/" { "" } else if is_marketing { "page-bg" } else { "" };

    let mut html = page_shell_with_body_class(title, description, &nav, &body, include_footer, body_class);

    // Inject widgets (chat, auth modal, toaster) into the page
    if !path.starts_with("/auth") {
        html = html.replace("</body>", &format!(
            "{chat_widget}{auth_modal}{toaster}</body>",
            chat_widget = widgets::chat_widget(is_authed, &user_id_for_data),
            auth_modal = widgets::auth_modal(),
            toaster = widgets::toaster_init(),
        ));
    }

    html
}

// =====================================================================
// Stub bodies that get filled in below (dashboard, auth, etc).
// =====================================================================

fn home_body() -> String {
    // Hero — matches epsx.io (📈 Track Your / Performance Growth / Metrics ✨)
    let hero = r##"<section class="relative w-full min-h-[85vh] flex items-center justify-center overflow-hidden">
  <div class="relative text-center space-y-12 max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-16 sm:py-20 z-[1]">
    <div class="inline-block animate-slide-up">
      <div class="mb-4 inline-flex items-center gap-2 px-4 py-2 rounded-full" style="background:linear-gradient(90deg, rgba(59,130,246,0.1) 0%, rgba(168,85,247,0.1) 100%);border:1px solid rgba(59,130,246,0.2);backdrop-filter:blur(8px);-webkit-backdrop-filter:blur(8px);">
        <i data-lucide="trending-up" style="width:1rem;height:1rem;color:#3b82f6;"></i>
        <span style="font-size:0.875rem;font-weight:500;color:#3b82f6;">Performance Analytics Platform</span>
      </div>
    </div>

    <h1 class="text-4xl sm:text-5xl md:text-6xl lg:text-7xl xl:text-8xl font-bold leading-tight" style="color:var(--text);">
      <span class="block">📈 Track Your</span>
      <span class="block hero-gradient-text animate-gradient-x">Performance Growth</span>
      <span class="block mt-2">Metrics ✨</span>
    </h1>

    <p class="animate-slide-up-d1 text-lg sm:text-xl md:text-2xl max-w-4xl mx-auto leading-relaxed" style="color:var(--text-muted);">
      🚀 Discover comprehensive data insights with our advanced analytics platform!
      <span class="block mt-2 font-bold" style="background:linear-gradient(90deg,#3b82f6 0%,#a855f7 100%);-webkit-background-clip:text;background-clip:text;-webkit-text-fill-color:transparent;">
        Make informed decisions with real-time insights
        <span class="ml-2" style="-webkit-text-fill-color:initial;color:initial;">📈</span>
      </span>
    </p>

    <div class="animate-slide-up-d2 flex flex-col sm:flex-row gap-4 sm:gap-6 justify-center items-center">
      <a href="/analytics" class="epsx-connect-btn" style="width:auto;height:auto;padding:0.875rem 2rem;font-size:1rem;border-radius:0.875rem;">
        <i data-lucide="chart-line" style="width:1.5rem;height:1.5rem;"></i>
        🚀 Start Exploration
      </a>
      <button class="epsx-connect-btn" type="button" onclick="if(navigator.share){navigator.share({title:'EPSX',url:location.href,text:'Check out EPSX!'});}else{navigator.clipboard.writeText(location.href);epsx.toast('Link copied to clipboard!','success');}" style="width:auto;height:auto;padding:0.875rem 2rem;font-size:1rem;border-radius:0.875rem;">
        <i data-lucide="share" style="width:1.5rem;height:1.5rem;"></i>
        📤 Share Platform
      </button>
    </div>

    <div class="grid grid-cols-1 sm:grid-cols-3 gap-6 sm:gap-8 mt-16">
      <div class="stat-card animate-fade-in-d3" style="--c1:#3b82f6;--c2:#06b6d4;animation-delay:0s;">
        <div class="stat-overlay"></div>
        <div class="stat-content">
          <i data-lucide="zap" class="stat-icon" style="width:2.5rem;height:2.5rem;color:#f59e0b;"></i>
          <div class="stat-num" style="background:linear-gradient(90deg,#3b82f6,#06b6d4);-webkit-background-clip:text;background-clip:text;-webkit-text-fill-color:transparent;">24/7</div>
          <div class="stat-label">🔄 Latest Updates</div>
        </div>
      </div>
      <div class="stat-card animate-fade-in-d3" style="--c1:#eab308;--c2:#f97316;animation-delay:0.1s;">
        <div class="stat-overlay"></div>
        <div class="stat-content">
          <i data-lucide="trending-up" class="stat-icon" style="width:2.5rem;height:2.5rem;color:#f97316;"></i>
          <div class="stat-num" style="background:linear-gradient(90deg,#eab308,#f97316);-webkit-background-clip:text;background-clip:text;-webkit-text-fill-color:transparent;">100+</div>
          <div class="stat-label">📊 Stock Analytics</div>
        </div>
      </div>
      <div class="stat-card animate-fade-in-d3" style="--c1:#10b981;--c2:#34d399;animation-delay:0.2s;">
        <div class="stat-overlay"></div>
        <div class="stat-content">
          <i data-lucide="users" class="stat-icon" style="width:2.5rem;height:2.5rem;color:#10b981;"></i>
          <div class="stat-num" style="background:linear-gradient(90deg,#10b981,#34d399);-webkit-background-clip:text;background-clip:text;-webkit-text-fill-color:transparent;">&lt; 1s</div>
          <div class="stat-label">⚡ Response Time</div>
        </div>
      </div>
    </div>
  </div>
</section>"##;

    // Performance Companies
    let companies = r##"<section class="epsx-section">
  <div class="container mx-auto px-4" style="position:relative;">
    <div style="position:absolute;top:-2rem;left:-2rem;width:4rem;height:4rem;border-radius:9999px;background:linear-gradient(135deg,rgba(249,115,22,0.2) 0%,rgba(234,179,8,0.2) 100%);filter:blur(24px);"></div>
    <div style="position:absolute;right:-2rem;bottom:-2rem;width:5rem;height:5rem;border-radius:9999px;background:linear-gradient(135deg,rgba(59,130,246,0.2) 0%,rgba(6,182,212,0.2) 100%);filter:blur(24px);"></div>
    <div class="text-center mb-12 relative">
      <h2 class="epsx-h2" style="background:linear-gradient(90deg,#f97316 0%,#eab308 50%,#a855f7 100%);-webkit-background-clip:text;background-clip:text;-webkit-text-fill-color:transparent;">Performance Companies</h2>
      <p class="text-base max-w-2xl mx-auto mt-3" style="color:var(--text-muted);">Discover data-driven growth and performance metrics. Upgrade your plan to access top-ranked companies.</p>
      <div class="epsx-section-underline warm" style="background:linear-gradient(90deg,#f97316 0%,#eab308 50%,#a855f7 100%);"></div>
    </div>
    <div id="rankings-grid" class="grid grid-cols-1 gap-6 px-4 sm:grid-cols-2 lg:grid-cols-3">

      <div class="company-card">
        <div class="epsx-blob" style="top:0;right:0;width:8rem;height:8rem;background:rgba(59,130,246,0.1);transform:translate(2.5rem,-2.5rem);"></div>
        <div class="epsx-blob" style="bottom:0;left:0;width:6rem;height:6rem;background:rgba(168,85,247,0.1);transform:translate(-2.5rem,2.5rem);"></div>
        <div class="text-center mb-4">
          <h3 style="font-size:0.75rem;font-weight:700;color:var(--text-subtle);text-transform:uppercase;letter-spacing:0.1em;margin-bottom:0.25rem;">RANK #100</h3>
          <div class="flex items-center justify-center mb-0.5"><span class="text-4xl font-black tracking-tighter" style="color:#3b82f6;">GHC</span></div>
          <div style="font-size:0.875rem;font-weight:600;color:var(--text-muted);margin-top:0.125rem;">$5.48</div>
        </div>
        <div class="space-y-2 mb-4 flex-grow">
          <div class="row-card flex items-center justify-between">
            <div class="flex items-center gap-2">
              <span class="row-icon" style="background:rgba(16,185,129,0.2);color:#10b981;"><i data-lucide="trending-up" style="width:1rem;height:1rem;"></i></span>
              <span style="font-size:0.875rem;color:var(--text-muted);font-weight:500;">Growth</span>
            </div>
            <span style="font-weight:700;font-size:0.875rem;color:#10b981;">+4650.00%</span>
          </div>
          <div class="progress-track"><div class="progress-fill" style="width:100%;background:linear-gradient(90deg,#10b981,#34d399);"><div class="progress-shine"></div></div></div>
          <div class="row-card flex items-center justify-between">
            <div class="flex items-center gap-2">
              <span class="row-icon" style="background:rgba(59,130,246,0.2);color:#3b82f6;"><i data-lucide="calendar" style="width:0.875rem;height:0.875rem;"></i></span>
              <span style="font-size:0.75rem;color:var(--text-subtle);font-weight:500;text-transform:uppercase;letter-spacing:0.05em;">Next Action</span>
            </div>
            <span style="font-weight:700;font-size:0.875rem;color:var(--text);">158 Days</span>
          </div>
          <div class="progress-track"><div class="progress-fill" style="width:5%;background:linear-gradient(90deg,#3b82f6,#06b6d4);"><div class="progress-shine"></div></div></div>
        </div>
        <a href="https://www.tradingview.com/symbols/GHC" target="_blank" rel="noopener noreferrer" class="block w-full mt-auto">
          <button class="view-btn">View Details <i data-lucide="arrow-right" style="width:1rem;height:1rem;display:inline;margin-left:0.25rem;"></i></button>
        </a>
      </div>

      <div class="company-card">
        <div class="epsx-blob" style="top:0;right:0;width:8rem;height:8rem;background:rgba(59,130,246,0.1);transform:translate(2.5rem,-2.5rem);"></div>
        <div class="epsx-blob" style="bottom:0;left:0;width:6rem;height:6rem;background:rgba(168,85,247,0.1);transform:translate(-2.5rem,2.5rem);"></div>
        <div class="text-center mb-4">
          <h3 style="font-size:0.75rem;font-weight:700;color:var(--text-subtle);text-transform:uppercase;letter-spacing:0.1em;margin-bottom:0.25rem;">RANK #101</h3>
          <div class="flex items-center justify-center mb-0.5"><span class="text-4xl font-black tracking-tighter" style="color:#3b82f6;">6535</span></div>
          <div style="font-size:0.875rem;font-weight:600;color:var(--text-muted);margin-top:0.125rem;">$462.00</div>
        </div>
        <div class="space-y-2 mb-4 flex-grow">
          <div class="row-card flex items-center justify-between">
            <div class="flex items-center gap-2">
              <span class="row-icon" style="background:rgba(16,185,129,0.2);color:#10b981;"><i data-lucide="trending-up" style="width:1rem;height:1rem;"></i></span>
              <span style="font-size:0.875rem;color:var(--text-muted);font-weight:500;">Growth</span>
            </div>
            <span style="font-weight:700;font-size:0.875rem;color:#10b981;">+4622.84%</span>
          </div>
          <div class="progress-track"><div class="progress-fill" style="width:100%;background:linear-gradient(90deg,#10b981,#34d399);"><div class="progress-shine"></div></div></div>
          <div class="row-card flex items-center justify-between">
            <div class="flex items-center gap-2">
              <span class="row-icon" style="background:rgba(59,130,246,0.2);color:#3b82f6;"><i data-lucide="calendar" style="width:0.875rem;height:0.875rem;"></i></span>
              <span style="font-size:0.75rem;color:var(--text-subtle);font-weight:500;text-transform:uppercase;letter-spacing:0.05em;">Next Action</span>
            </div>
            <span style="font-weight:700;font-size:0.875rem;color:var(--text);">1 Days</span>
          </div>
          <div class="progress-track"><div class="progress-fill" style="width:98.89%;background:linear-gradient(90deg,#3b82f6,#06b6d4);"><div class="progress-shine"></div></div></div>
        </div>
        <a href="https://www.tradingview.com/symbols/6535" target="_blank" rel="noopener noreferrer" class="block w-full mt-auto">
          <button class="view-btn">View Details <i data-lucide="arrow-right" style="width:1rem;height:1rem;display:inline;margin-left:0.25rem;"></i></button>
        </a>
      </div>

      <div class="company-card">
        <div class="epsx-blob" style="top:0;right:0;width:8rem;height:8rem;background:rgba(59,130,246,0.1);transform:translate(2.5rem,-2.5rem);"></div>
        <div class="epsx-blob" style="bottom:0;left:0;width:6rem;height:6rem;background:rgba(168,85,247,0.1);transform:translate(-2.5rem,2.5rem);"></div>
        <div class="text-center mb-4">
          <h3 style="font-size:0.75rem;font-weight:700;color:var(--text-subtle);text-transform:uppercase;letter-spacing:0.1em;margin-bottom:0.25rem;">RANK #102</h3>
          <div class="flex items-center justify-center mb-0.5"><span class="text-4xl font-black tracking-tighter" style="color:#3b82f6;">4657</span></div>
          <div style="font-size:0.875rem;font-weight:600;color:var(--text-muted);margin-top:0.125rem;">$427.00</div>
        </div>
        <div class="space-y-2 mb-4 flex-grow">
          <div class="row-card flex items-center justify-between">
            <div class="flex items-center gap-2">
              <span class="row-icon" style="background:rgba(16,185,129,0.2);color:#10b981;"><i data-lucide="trending-up" style="width:1rem;height:1rem;"></i></span>
              <span style="font-size:0.875rem;color:var(--text-muted);font-weight:500;">Growth</span>
            </div>
            <span style="font-weight:700;font-size:0.875rem;color:#10b981;">+4612.47%</span>
          </div>
          <div class="progress-track"><div class="progress-fill" style="width:100%;background:linear-gradient(90deg,#10b981,#34d399);"><div class="progress-shine"></div></div></div>
          <div class="row-card flex items-center justify-between">
            <div class="flex items-center gap-2">
              <span class="row-icon" style="background:rgba(59,130,246,0.2);color:#3b82f6;"><i data-lucide="calendar" style="width:0.875rem;height:0.875rem;"></i></span>
              <span style="font-size:0.75rem;color:var(--text-subtle);font-weight:500;text-transform:uppercase;letter-spacing:0.05em;">Next Action</span>
            </div>
            <span style="font-weight:700;font-size:0.875rem;color:var(--text);">65 Days</span>
          </div>
          <div class="progress-track"><div class="progress-fill" style="width:27.78%;background:linear-gradient(90deg,#3b82f6,#06b6d4);"><div class="progress-shine"></div></div></div>
        </div>
        <a href="https://www.tradingview.com/symbols/4657" target="_blank" rel="noopener noreferrer" class="block w-full mt-auto">
          <button class="view-btn">View Details <i data-lucide="arrow-right" style="width:1rem;height:1rem;display:inline;margin-left:0.25rem;"></i></button>
        </a>
      </div>
    </div>
  </div>
</section>"##;

    // Personal Plans
    let personal = r##"<section class="epsx-section">
  <div class="container mx-auto px-4">
    <div class="text-center mb-12">
      <h2 class="epsx-h2-orange" style="background:linear-gradient(90deg,#f97316 0%,#eab308 50%,#f97316 100%);-webkit-background-clip:text;background-clip:text;-webkit-text-fill-color:transparent;">💰Personal Plans</h2>
      <p class="max-w-2xl mx-auto mt-4" style="color:var(--text-muted);">🚀 Choose the perfect plan for individual use and start your data journey</p>
      <div class="epsx-section-underline warm"></div>
    </div>
    <div class="grid grid-cols-1 sm:grid-cols-3 gap-6 max-w-5xl mx-auto">

      <div class="pricing-card">
        <span class="sale-badge">SALE</span>
        <h3 class="price-title">1 Day Package</h3>
        <div class="flex items-baseline gap-1" style="color:#3b82f6;font-weight:900;">
          <span style="font-size:2.5rem;line-height:1;">$1</span><span class="suffix" style="color:var(--text-subtle);">USD</span>
          <span class="promo-badge">80% OFF</span>
        </div>
        <div class="price-original">$5 USD</div>
        <div class="price-savings">Save $4</div>
        <div class="countdown" id="countdown-day" data-countdown-hours="24">Ends in 23h 59m 59s</div>
        <ul class="features">
          <li><i data-lucide="check" class="check"></i> Basic analytics view</li>
          <li><i data-lucide="check" class="check"></i> Rankings from position 6+</li>
          <li><i data-lucide="check" class="check"></i> Basic trading features</li>
          <li><i data-lucide="check" class="check"></i> 24-hour access</li>
          <li><i data-lucide="check" class="check"></i> Explore the platform</li>
        </ul>
        <button class="cta-btn" onclick="epsx.toast('Sign in to get started','info')">Get Started</button>
      </div>

      <div class="pricing-card">
        <span class="sale-badge">SALE</span>
        <h3 class="price-title">1 Month Package</h3>
        <div class="flex items-baseline gap-1" style="color:#3b82f6;font-weight:900;">
          <span style="font-size:2.5rem;line-height:1;">$9.9</span><span class="suffix" style="color:var(--text-subtle);">USD</span>
          <span class="promo-badge">90% OFF</span>
        </div>
        <div class="price-original">$99 USD</div>
        <div class="price-savings">Save $89.1</div>
        <div class="countdown" id="countdown-month" data-countdown-hours="168">Ends in 6d 23h 59m</div>
        <ul class="features">
          <li><i data-lucide="check" class="check"></i> Advanced analytics view</li>
          <li><i data-lucide="check" class="check"></i> 25 stock rankings</li>
          <li><i data-lucide="check" class="check"></i> Basic analytic features</li>
          <li><i data-lucide="check" class="check"></i> Price alerts</li>
          <li><i data-lucide="check" class="check"></i> Email support</li>
          <li><i data-lucide="check" class="check"></i> 30-day access</li>
        </ul>
        <button class="cta-btn" onclick="epsx.toast('Sign in to get started','info')">Get Started</button>
      </div>

      <div class="pricing-card">
        <span class="sale-badge">SALE</span>
        <h3 class="price-title">Lifetime Package</h3>
        <div class="flex items-baseline gap-1" style="color:#3b82f6;font-weight:900;">
          <span style="font-size:2.5rem;line-height:1;">$4999</span><span class="suffix" style="color:var(--text-subtle);">USD</span>
          <span class="promo-badge">50% OFF</span>
        </div>
        <div class="price-original">$9999 USD</div>
        <div class="price-savings">Save $5000</div>
        <div class="countdown" id="countdown-lifetime" data-countdown-hours="720">Ends in 29d 23h 59m</div>
        <ul class="features">
          <li><i data-lucide="check" class="check"></i> Advanced analytics suite</li>
          <li><i data-lucide="check" class="check"></i> Full rankings access (Rank 1+)</li>
          <li><i data-lucide="check" class="check"></i> API read access</li>
          <li><i data-lucide="check" class="check"></i> Basic &amp; Pro trading</li>
          <li><i data-lucide="check" class="check"></i> Priority support</li>
          <li><i data-lucide="check" class="check"></i> Lifetime access</li>
        </ul>
        <button class="cta-btn" onclick="epsx.toast('Sign in to get started','info')">Get Started</button>
      </div>
    </div>
  </div>
</section>"##;

    // API Plans
    let api = r##"<section class="epsx-section">
  <div class="container mx-auto px-4">
    <div class="text-center mb-12">
      <h2 class="epsx-h2-pink-purple">API Plans</h2>
      <p class="max-w-2xl mx-auto mt-4" style="color:var(--text-muted);">Integrate our powerful API into your systems</p>
      <div class="epsx-section-underline pink"></div>
    </div>
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-6 max-w-4xl mx-auto">

      <div class="pricing-card">
        <span class="sale-badge">SALE</span>
        <h3 class="price-title">API Personal</h3>
        <div class="flex items-baseline gap-1" style="color:#3b82f6;font-weight:900;">
          <span style="font-size:2.5rem;line-height:1;">$999</span><span class="suffix" style="color:var(--text-subtle);">USD</span>
          <span class="promo-badge">75% OFF</span>
        </div>
        <div class="price-original">$3999 USD</div>
        <div class="price-savings">Save $3000</div>
        <div class="countdown">Ends in 14d 23h 59m</div>
        <ul class="features">
          <li><i data-lucide="check" class="check"></i> Analytics view access</li>
          <li><i data-lucide="check" class="check"></i> API read access</li>
          <li><i data-lucide="check" class="check"></i> Data export capability</li>
          <li><i data-lucide="check" class="check"></i> Full developer documentation</li>
          <li><i data-lucide="check" class="check"></i> 30-day access</li>
        </ul>
        <button class="cta-btn" onclick="epsx.toast('Sign in to get started','info')">Get Started</button>
      </div>

      <div class="pricing-card">
        <span class="sale-badge">SALE</span>
        <h3 class="price-title">API Company</h3>
        <div class="flex items-baseline gap-1" style="color:#3b82f6;font-weight:900;">
          <span style="font-size:2.5rem;line-height:1;">$2999</span><span class="suffix" style="color:var(--text-subtle);">USD</span>
          <span class="promo-badge">57% OFF</span>
        </div>
        <div class="price-original">$6999 USD</div>
        <div class="price-savings">Save $4000</div>
        <div class="countdown">Ends in 14d 23h 59m</div>
        <ul class="features">
          <li><i data-lucide="check" class="check"></i> Advanced analytics suite</li>
          <li><i data-lucide="check" class="check"></i> Full trading suite (Basic, Pro &amp; Advanced)</li>
          <li><i data-lucide="check" class="check"></i> API read &amp; write access</li>
          <li><i data-lucide="check" class="check"></i> Data export</li>
          <li><i data-lucide="check" class="check"></i> Notifications management</li>
          <li><i data-lucide="check" class="check"></i> 365-day company access</li>
          <li><i data-lucide="check" class="check"></i> Dedicated support</li>
        </ul>
        <button class="cta-btn" onclick="epsx.toast('Sign in to get started','info')">Get Started</button>
      </div>
    </div>
  </div>
</section>"##;

    // Custom Plans
    let custom = r##"<section class="epsx-section">
  <div class="container mx-auto px-4">
    <div class="text-center mb-12">
      <h2 class="epsx-h2-purple">Custom Plans</h2>
      <p class="max-w-2xl mx-auto mt-4" style="color:var(--text-muted);">Tailored solutions for partners, corporate, and enterprise needs</p>
      <div class="epsx-section-underline purple"></div>
    </div>
    <div class="max-w-md mx-auto">
      <div class="pricing-card">
        <h3 class="price-title" style="text-align:center;">CUSTOM</h3>
        <div class="text-center">
          <div style="font-size:1.5rem;font-weight:700;background:linear-gradient(90deg,#a855f7 0%,#d946ef 100%);-webkit-background-clip:text;background-clip:text;-webkit-text-fill-color:transparent;">Revenue Share</div>
        </div>
        <ul class="features">
          <li><i data-lucide="check" class="check"></i> Custom feature set &amp; permissions</li>
          <li><i data-lucide="check" class="check"></i> Dedicated support &amp; SLA</li>
          <li><i data-lucide="check" class="check"></i> Volume-based pricing</li>
          <li><i data-lucide="check" class="check"></i> Custom API rate limits</li>
          <li><i data-lucide="check" class="check"></i> White-label options</li>
          <li><i data-lucide="check" class="check"></i> Priority onboarding</li>
        </ul>
        <button class="cta-btn" style="background:linear-gradient(90deg,#a855f7 0%,#d946ef 100%);" onclick="epsx.toast('Reach out via /contact','info')">
          <i data-lucide="message-circle" style="width:1rem;height:1rem;display:inline;margin-right:0.25rem;"></i>
          Get in Touch
        </button>
        <div class="text-center mt-3" style="font-size:0.75rem;color:var(--text-subtle);">We'll create a plan that fits your needs</div>
      </div>
    </div>
  </div>
</section>"##;

    // News
    let news = r##"<section class="epsx-section">
  <div class="container mx-auto px-4">
    <div class="mb-6 flex items-center justify-between">
      <div class="flex items-center gap-2">
        <i data-lucide="newspaper" style="width:1.25rem;height:1.25rem;color:#1fc7d4;"></i>
        <h2 style="font-size:1.25rem;font-weight:700;color:var(--text);">Latest News</h2>
      </div>
      <a href="/news" class="flex items-center gap-1 text-sm font-medium" style="color:#1fc7d4;">
        View all <i data-lucide="arrow-right" style="width:1rem;height:1rem;"></i>
      </a>
    </div>
    <div class="space-y-4">

      <a href="/news/optimizing-high-throughput-analytics-rust" class="block group" style="text-decoration:none;">
        <div class="news-featured">
          <img src="https://aullybia7vqndzva.public.blob.vercel-storage.com/news/strategic-analysis-performance.jpg" alt="Strategic Analysis Performance for Operational Excellence" />
          <div class="news-overlay"></div>
          <div class="news-caption">
            <div class="news-featured-tag"><i data-lucide="pin" style="width:0.875rem;height:0.875rem;"></i>Featured</div>
            <div class="flex flex-wrap gap-2 mb-3 mt-2">
              <span class="news-tag">strategy</span>
              <span class="news-tag">performance</span>
            </div>
            <h3 class="news-title">Strategic Analysis Performance for Operational Excellence</h3>
            <p class="news-excerpt">How EPSX leverages high-performance data processing to deliver precise rankings and insights for business decision-making.</p>
            <span class="news-date">Mar 15, 2026</span>
          </div>
        </div>
      </a>

      <div class="grid gap-4 grid-cols-2">
        <a href="/news/strategic-roadmap-future" class="block group" style="text-decoration:none;">
          <div class="news-small">
            <img src="https://aullybia7vqndzva.public.blob.vercel-storage.com/news/strategic-roadmap-future.jpg" alt="Strategic Roadmap and Future Capabilities" />
            <div class="news-overlay"></div>
            <div class="news-caption">
              <h3 class="news-title">Strategic Roadmap and Future Capabilities</h3>
              <span class="news-date">May 9, 2026</span>
            </div>
          </div>
        </a>
        <a href="/news/enhanced-portfolio-management" class="block group" style="text-decoration:none;">
          <div class="news-small">
            <img src="https://aullybia7vqndzva.public.blob.vercel-storage.com/news/enhanced-portfolio-management.jpg" alt="Enhanced Portfolio Management Solutions" />
            <div class="news-overlay"></div>
            <div class="news-caption">
              <h3 class="news-title">Enhanced Portfolio Management Solutions</h3>
              <span class="news-date">Apr 30, 2026</span>
            </div>
          </div>
        </a>
      </div>
    </div>
  </div>
</section>"##;

    // Countdown timer JS is now handled by startCountdowns() in the global epsx module
    let countdown_js = "";

    format!("{}{}{}{}{}{}{}", hero, companies, personal, api, custom, news, countdown_js)
}

// =====================================================================
// Dashboard, auth, edit, legal, not-found, about, pricing, blog, contact
// (server-rendered; reuse static layout from pages.rs where possible)
// =====================================================================

fn about_body() -> String {
    // Matches epsx.io's actual /about page: "Precision Analytics For Modern Teams"
    r##"<section style="max-width:80rem;margin:0 auto;padding:3rem 1.5rem;">
  <div class="mb-12 animate-fade-in">
    <div class="flex items-center gap-3 mb-8">
      <div class="rounded-2xl bg-gradient-to-br from-orange-500 to-purple-600 p-3 shadow-2xl shadow-orange-500/20 ring-1 ring-white/20 transition-transform hover:scale-105">
        <i data-lucide="cpu" style="width:2rem;height:2rem;color:white;"></i>
      </div>
      <span class="text-4xl font-black tracking-tighter uppercase italic" style="color:var(--text);">EPSX</span>
    </div>
    <h1 style="font-size:3rem;font-weight:700;line-height:1.1;letter-spacing:-0.02em;" class="xl:text-7xl">Precision <span style="background:linear-gradient(90deg,#fb923c 0%,#facc15 50%,#f97316 100%);-webkit-background-clip:text;background-clip:text;-webkit-text-fill-color:transparent;background-size:200% 200%;animation:gradient-x 3s ease infinite;">Analytics</span> <br>For Modern Teams</h1>
    <p style="margin-top:1.5rem;font-size:1.125rem;color:var(--text-muted);max-width:36rem;line-height:1.7;" class="xl:text-xl">Join the next generation of data intelligence. Real-time metrics, predictive modeling, and institutional-grade insights at your fingertips.</p>
  </div>

  <div class="grid grid-cols-1 md:grid-cols-3 gap-6 animate-slide-up">
    <div class="card-insight">
      <div style="width:3rem;height:3rem;border-radius:0.75rem;background:linear-gradient(135deg,#f97316,#a855f7);display:flex;align-items:center;justify-content:center;margin-bottom:1rem;">
        <i data-lucide="zap" style="color:white;width:1.5rem;height:1.5rem;"></i>
      </div>
      <h3 style="font-size:1.25rem;font-weight:700;margin-bottom:0.5rem;">Real-time Metrics</h3>
      <p style="color:var(--text-muted);line-height:1.6;">Sub-second updates on every metric that matters to your portfolio. Powered by Redis Streams and WebSockets.</p>
    </div>
    <div class="card-insight">
      <div style="width:3rem;height:3rem;border-radius:0.75rem;background:linear-gradient(135deg,#3b82f6,#06b6d4);display:flex;align-items:center;justify-content:center;margin-bottom:1rem;">
        <i data-lucide="brain" style="color:white;width:1.5rem;height:1.5rem;"></i>
      </div>
      <h3 style="font-size:1.25rem;font-weight:700;margin-bottom:0.5rem;">Predictive Modeling</h3>
      <p style="color:var(--text-muted);line-height:1.6;">EPSX AI scores trends and surfaces high-signal opportunities before they hit the mainstream news cycle.</p>
    </div>
    <div class="card-insight">
      <div style="width:3rem;height:3rem;border-radius:0.75rem;background:linear-gradient(135deg,#a855f7,#ec4899);display:flex;align-items:center;justify-content:center;margin-bottom:1rem;">
        <i data-lucide="shield" style="color:white;width:1.5rem;height:1.5rem;"></i>
      </div>
      <h3 style="font-size:1.25rem;font-weight:700;margin-bottom:0.5rem;">Institutional-Grade</h3>
      <p style="color:var(--text-muted);line-height:1.6;">The same caliber of insight relied upon by hedge funds, now accessible to individual investors and builders.</p>
    </div>
  </div>

  <h2 style="font-size:2rem;font-weight:700;margin:4rem 0 1.5rem;">Architecture</h2>
  <div class="card-insight" style="margin-bottom:1rem;">
    <ul style="list-style:none;padding:0;margin:0;display:grid;gap:0.75rem;">
      <li style="display:flex;gap:0.75rem;align-items:flex-start;"><i data-lucide="microchip" style="color:var(--epsx-orange);width:1.25rem;height:1.25rem;flex-shrink:0;margin-top:0.25rem;"></i><div><strong style="color:var(--text);">Rust microservices</strong> — Axum + SQLx + Alloy for predictable, low-latency infrastructure.</div></li>
      <li style="display:flex;gap:0.75rem;align-items:flex-start;"><i data-lucide="boxes" style="color:var(--epsx-orange);width:1.25rem;height:1.25rem;flex-shrink:0;margin-top:0.25rem;"></i><div><strong style="color:var(--text);">Plain axum BFFs</strong> — Server-rendered HTML with the new design system, fast on any device.</div></li>
      <li style="display:flex;gap:0.75rem;align-items:flex-start;"><i data-lucide="link" style="color:var(--epsx-orange);width:1.25rem;height:1.25rem;flex-shrink:0;margin-top:0.25rem;"></i><div><strong style="color:var(--text);">BSC mainnet + testnet</strong> — chain ids 56 and 97 with multi-chain support.</div></li>
      <li style="display:flex;gap:0.75rem;align-items:flex-start;"><i data-lucide="fuel" style="color:var(--epsx-orange);width:1.25rem;height:1.25rem;flex-shrink:0;margin-top:0.25rem;"></i><div><strong style="color:var(--text);">ERC-4337 paymaster</strong> — sponsor user gas, charge USDC.</div></li>
      <li style="display:flex;gap:0.75rem;align-items:flex-start;"><i data-lucide="vault" style="color:var(--epsx-orange);width:1.25rem;height:1.25rem;flex-shrink:0;margin-top:0.25rem;"></i><div><strong style="color:var(--text);">On-chain subscription vaults</strong> — per-merchant isolation, no Superfluid complexity.</div></li>
    </ul>
  </div>
</section>"##.to_string()
}

fn pricing_body() -> String {
    r##"<section class="section">
<div class="container-x" style="text-align:center;">
  <span class="badge-pill"><i data-lucide="tag" style="color:var(--epsx-orange);"></i> Pricing</span>
  <h1 style="font-size:3rem;font-weight:800;margin:1rem 0 1rem;">Simple, transparent pricing</h1>
  <p style="font-size:1.125rem;color:var(--text-muted);max-width:42rem;margin:0 auto 3rem;">Start free, scale as you grow. No hidden fees, ever.</p>
  <div style="display:grid;grid-template-columns:repeat(auto-fit, minmax(280px, 1fr));gap:1.5rem;max-width:72rem;margin:0 auto;text-align:left;">
    <div class="card-insight"><h3 style="font-size:1.5rem;font-weight:700;margin-bottom:0.5rem;">Free</h3><div style="font-size:2.5rem;font-weight:800;margin-bottom:1rem;">$0<span style="font-size:1rem;font-weight:500;color:var(--text-muted);">/month</span></div><ul style="list-style:none;padding:0;margin:0 0 1.5rem;display:grid;gap:0.5rem;"><li style="display:flex;gap:0.5rem;align-items:center;"><i data-lucide="check" style="color:var(--epsx-green);"></i> 100 API calls/day</li><li style="display:flex;gap:0.5rem;align-items:center;"><i data-lucide="check" style="color:var(--epsx-green);"></i> Basic analytics</li><li style="display:flex;gap:0.5rem;align-items:center;"><i data-lucide="check" style="color:var(--epsx-green);"></i> Community support</li></ul><a href="/auth" class="btn btn-outline btn-block">Get Started</a></div>
    <div class="card-insight" style="border:2px solid var(--epsx-orange);position:relative;"><span class="badge badge-primary" style="position:absolute;top:-0.75rem;left:50%;transform:translateX(-50%);"><i data-lucide="star"></i> POPULAR</span><h3 style="font-size:1.5rem;font-weight:700;margin-bottom:0.5rem;">Pro</h3><div class="gradient-text" style="font-size:2.5rem;font-weight:800;margin-bottom:1rem;">$29<span style="font-size:1rem;font-weight:500;color:var(--text-muted);-webkit-text-fill-color:var(--text-muted);">/month</span></div><ul style="list-style:none;padding:0;margin:0 0 1.5rem;display:grid;gap:0.5rem;"><li style="display:flex;gap:0.5rem;align-items:center;"><i data-lucide="check" style="color:var(--epsx-green);"></i> 10,000 API calls/day</li><li style="display:flex;gap:0.5rem;align-items:center;"><i data-lucide="check" style="color:var(--epsx-green);"></i> Advanced analytics</li><li style="display:flex;gap:0.5rem;align-items:center;"><i data-lucide="check" style="color:var(--epsx-green);"></i> Priority support</li><li style="display:flex;gap:0.5rem;align-items:center;"><i data-lucide="check" style="color:var(--epsx-green);"></i> Custom webhooks</li></ul><a href="/auth" class="btn btn-gradient btn-block">Subscribe</a></div>
    <div class="card-insight"><h3 style="font-size:1.5rem;font-weight:700;margin-bottom:0.5rem;">Enterprise</h3><div style="font-size:2.5rem;font-weight:800;margin-bottom:1rem;">Custom</div><ul style="list-style:none;padding:0;margin:0 0 1.5rem;display:grid;gap:0.5rem;"><li style="display:flex;gap:0.5rem;align-items:center;"><i data-lucide="check" style="color:var(--epsx-green);"></i> Unlimited API calls</li><li style="display:flex;gap:0.5rem;align-items:center;"><i data-lucide="check" style="color:var(--epsx-green);"></i> Custom integrations</li><li style="display:flex;gap:0.5rem;align-items:center;"><i data-lucide="check" style="color:var(--epsx-green);"></i> Dedicated support</li><li style="display:flex;gap:0.5rem;align-items:center;"><i data-lucide="check" style="color:var(--epsx-green);"></i> SLA guarantee</li></ul><a href="/contact" class="btn btn-outline btn-block">Contact Us</a></div>
  </div>
</div>
</section>"##.to_string()
}

fn blog_index_body() -> String {
    r##"<section class="section">
<div class="container-x">
  <div style="text-align:center;margin-bottom:3rem;">
    <span class="badge-pill"><i data-lucide="newspaper" style="color:var(--epsx-orange);"></i> Blog</span>
    <h1 style="font-size:3rem;font-weight:800;margin:1rem 0 1rem;">Latest from the team</h1>
    <p style="font-size:1.125rem;color:var(--text-muted);max-width:42rem;margin:0 auto;">News, deep dives, and product updates.</p>
  </div>
  <div style="display:grid;grid-template-columns:repeat(auto-fit, minmax(300px, 1fr));gap:1.5rem;">
    <a href="/blog/welcome" class="card-insight" style="text-decoration:none;color:inherit;display:block;"><span class="badge badge-primary" style="margin-bottom:0.75rem;">Product</span><h2 style="font-size:1.5rem;font-weight:700;margin-bottom:0.5rem;">Welcome to EPSX</h2><p style="color:var(--text-muted);margin-bottom:1rem;">Our first blog post about web3 analytics and stablecoin payments on BSC.</p><div style="font-size:0.875rem;color:var(--text-subtle);">5 min read</div></a>
    <a href="/blog/subscription-vaults" class="card-insight" style="text-decoration:none;color:inherit;display:block;"><span class="badge badge-purple" style="margin-bottom:0.75rem;">Engineering</span><h2 style="font-size:1.5rem;font-weight:700;margin-bottom:0.5rem;">Subscription Vaults Explained</h2><p style="color:var(--text-muted);margin-bottom:1rem;">How on-chain subscription vaults work and why they are better than Superfluid streams.</p><div style="font-size:0.875rem;color:var(--text-subtle);">8 min read</div></a>
    <a href="/blog/paymaster" class="card-insight" style="text-decoration:none;color:inherit;display:block;"><span class="badge badge-info" style="margin-bottom:0.75rem;">Engineering</span><h2 style="font-size:1.5rem;font-weight:700;margin-bottom:0.5rem;">Paymaster Gas Sponsorship</h2><p style="color:var(--text-muted);margin-bottom:1rem;">Sponsor user gas with stablecoins — no native gas required for end users.</p><div style="font-size:0.875rem;color:var(--text-subtle);">6 min read</div></a>
  </div>
</div>
</section>"##.to_string()
}

fn blog_post_body(slug: &str) -> String {
    format!(
        r##"<section class="section">
<div class="container-x" style="max-width:54rem;">
  <a href="/blog" class="nav-link" style="margin-bottom:1.5rem;display:inline-flex;">
    <i data-lucide="arrow-left"></i> Back to blog
  </a>
  <span class="badge badge-primary">Post</span>
  <h1 style="font-size:2.5rem;font-weight:800;margin:1rem 0 1.5rem;">{}</h1>
  <div class="card-insight" style="font-size:1rem;line-height:1.7;color:var(--text-muted);">
    <p>Loading post...</p>
  </div>
  <div id="post-content" data-slug="{}">Loading...</div>
</div>
</section>"##,
        slug, slug
    )
}

fn contact_body() -> String {
    r##"<section class="section">
<div class="container-x" style="max-width:42rem;">
  <div style="text-align:center;margin-bottom:3rem;">
    <span class="badge-pill"><i data-lucide="message-square" style="color:var(--epsx-orange);"></i> Contact</span>
    <h1 style="font-size:3rem;font-weight:800;margin:1rem 0 1rem;">Get in touch</h1>
    <p style="font-size:1.125rem;color:var(--text-muted);">Have a question, partnership idea, or feedback? We'd love to hear from you.</p>
  </div>
  <form class="card-insight" onsubmit="event.preventDefault(); epsx.toast('Message sent! We will respond within 24h.', 'success');" style="display:grid;gap:1rem;">
    <div><label class="label">Your name</label><input type="text" class="input" placeholder="Jane Doe" required /></div>
    <div><label class="label">Email</label><input type="email" class="input" placeholder="you@company.com" required /></div>
    <div><label class="label">Subject</label><select class="input" required><option>Sales inquiry</option><option>Partnership</option><option>Support</option><option>Other</option></select></div>
    <div><label class="label">Message</label><textarea class="input" rows="5" placeholder="Tell us what you're working on..." required></textarea></div>
    <button type="submit" class="btn btn-gradient btn-lg"><i data-lucide="send"></i> Send Message</button>
  </form>
</div>
</section>"##.to_string()
}

fn dashboard_body(is_authed: bool, display: Option<String>) -> String {
    if is_authed {
        let addr = display.unwrap_or_else(|| "0xDEMO...0000".to_string());
        let portfolio = StatCard::new("Portfolio Value", "$0.00")
            .icon("chart-line", "var(--epsx-green)")
            .change("Live in 1s", BadgeKind::Info)
            .href("/portfolio")
            .render();
        let subs = StatCard::new("Active Subscriptions", "0")
            .icon("vault", "var(--epsx-blue-start)")
            .change("0 expiring", BadgeKind::Info)
            .href("/account")
            .render();
        let notifs = StatCard::new("Notifications", "0")
            .icon("bell", "var(--epsx-purple)")
            .change("0 unread", BadgeKind::Info)
            .href("/notifications")
            .render();
        let cta = Btn::new("Open Full Settings").gradient().lg().icon_left("gear").href("/account").render();
        format!(
            r##"<section class="section">
<div class="container-x">
  <div style="display:flex;justify-content:space-between;align-items:flex-end;flex-wrap:wrap;gap:1rem;margin-bottom:2rem;">
    <div>
      <span class="badge-pill"><i data-lucide="wallet" style="color:var(--epsx-orange);"></i> {addr}</span>
      <h1 style="font-size:2.5rem;font-weight:800;margin-top:1rem;">Welcome back</h1>
      <p style="color:var(--text-muted);">Your portfolio, subscriptions, and notifications at a glance.</p>
    </div>
    {cta}
  </div>
  <div style="display:grid;grid-template-columns:repeat(auto-fit, minmax(220px, 1fr));gap:1.5rem;margin-bottom:2rem;">
    {portfolio}
    {subs}
    {notifs}
  </div>
  <div class="card-insight" style="text-align:center;padding:2.5rem;">
    <h2 style="font-size:1.5rem;font-weight:700;margin-bottom:0.5rem;">Browse the platform</h2>
    <p style="color:var(--text-muted);margin-bottom:1.5rem;">Jump into analytics, manage your portfolio, or check out the docs.</p>
    <div style="display:flex;flex-wrap:wrap;gap:0.75rem;justify-content:center;">
      <a href="/analytics" class="btn btn-gradient"><i data-lucide="line-chart"></i> Analytics</a>
      <a href="/portfolio" class="btn btn-outline"><i data-lucide="heart"></i> Portfolio</a>
      <a href="/developer" class="btn btn-outline"><i data-lucide="code"></i> Developer</a>
    </div>
  </div>
</div>
</section>"##,
            addr = addr,
            cta = cta,
            portfolio = portfolio,
            subs = subs,
            notifs = notifs
        )
    } else {
        let portfolio = StatCard::new("Portfolio Value", "$0.00")
            .icon("chart-line", "var(--epsx-green)")
            .change("Connect wallet to track", BadgeKind::Info)
            .href("/portfolio")
            .render();
        let subs = StatCard::new("Active Subscriptions", "0")
            .icon("vault", "var(--epsx-blue-start)")
            .change("Across all vaults", BadgeKind::Info)
            .href("/portfolio?tab=subscriptions")
            .render();
        let txs = StatCard::new("Transactions", "0")
            .icon("arrow-right-arrow-left", "var(--epsx-purple)")
            .change("Last 30 days", BadgeKind::Info)
            .href("/analytics")
            .render();
        let cta = Card::body_only(r##"<div style="text-align:center;padding:1rem 0;">
          <div style="width:4rem;height:4rem;border-radius:9999px;background:var(--gradient-warm);display:flex;align-items:center;justify-content:center;margin:0 auto 1rem;box-shadow:var(--shadow-orange);">
            <i data-lucide="wallet" style="color:white;font-size:1.25rem;"></i>
          </div>
          <h2 style="font-size:1.5rem;font-weight:700;margin-bottom:0.5rem;">Connect a BSC wallet</h2>
          <p style="color:var(--text-muted);margin-bottom:1.5rem;">Sign in with your wallet to track assets, subscriptions, and on-chain activity.</p>
        </div>"##.to_string())
            .insight()
            .footer(Btn::new("Sign In").gradient().lg().icon_left("arrow-right-to-bracket").href("/auth").block().render())
            .render();
        let header_badge = Badge::new("Dashboard").warm().pill().icon("gauge-high").render();
        let connect_btn = Btn::new("Connect Wallet").gradient().icon_left("wallet").href("/auth").render();
        format!(
            r##"<section class="section">
<div class="container-x">
  <div style="display:flex;justify-content:space-between;align-items:flex-end;flex-wrap:wrap;gap:1rem;margin-bottom:2rem;">
    <div>
      {header_badge}
      <h1 style="font-size:2.5rem;font-weight:800;margin-top:1rem;">Your portfolio at a glance</h1>
    </div>
    {connect_btn}
  </div>
  <div style="display:grid;grid-template-columns:repeat(auto-fit, minmax(220px, 1fr));gap:1.5rem;margin-bottom:2rem;">
    {portfolio}
    {subs}
    {txs}
  </div>
  {cta}
</div>
</section>"##,
            header_badge = header_badge,
            connect_btn = connect_btn,
            portfolio = portfolio,
            subs = subs,
            txs = txs,
            cta = cta,
        )
    }
}

fn auth_body(demo_enabled: bool) -> String {
    let wallet_btn = Btn::new("Connect Wallet (SIWE on BSC)").gradient().lg().block().icon_left("wallet").attr("id", "wallet-btn").render();
    let email_btn  = Btn::new("Sign in with Email").outline().lg().block().icon_left("envelope").render();
    let demo_btn = if demo_enabled {
        r##"<div style="margin-top:1.5rem;padding-top:1.5rem;border-top:1px solid var(--border);text-align:center;">
          <p style="font-size:0.8125rem;color:var(--text-muted);margin-bottom:0.75rem;">Or try the demo (no wallet required):</p>
          <button class="btn btn-outline btn-block" id="demo-btn"><i data-lucide="flask-conical"></i> Continue as Demo User</button>
        </div>"##
    } else { "" };

    let card = Card::body_only(format!(
        r##"<div style="text-align:center;margin-bottom:2rem;">
          <div style="width:4rem;height:4rem;border-radius:9999px;background:var(--gradient-warm);display:flex;align-items:center;justify-content:center;margin:0 auto 1rem;box-shadow:var(--shadow-orange);">
            <i data-lucide="log-in" style="color:white;font-size:1.25rem;"></i>
          </div>
          <h1 style="font-size:1.75rem;font-weight:800;margin-bottom:0.5rem;">Welcome back</h1>
          <p style="color:var(--text-muted);">Connect your Web3 wallet to continue</p>
        </div>
        {wallet}
        <div style="margin-bottom:1.5rem;"></div>
        {email}
        <p style="font-size:0.8125rem;color:var(--text-subtle);text-align:center;margin-top:1.5rem;">
          By signing in, you agree to our <a href="/terms" class="footer-link">Terms of Service</a>
          and <a href="/privacy" class="footer-link">Privacy Policy</a>.
        </p>
        {demo}"##,
        wallet = wallet_btn,
        email = email_btn,
        demo = demo_btn
    )).insight().cls("auth-card").render();

    let js = r##"<script>
document.getElementById('wallet-btn').onclick = async () => {
  if (typeof window.ethereum === 'undefined') { epsx.toast('Install MetaMask or another BSC wallet', 'warning'); return; }
  try {
    const accounts = await window.ethereum.request({ method: 'eth_requestAccounts' });
    const address = accounts[0];
    const chainId = await window.ethereum.request({ method: 'eth_chainId' });
    const domain = location.host;
    const uri = location.origin;
    const nonce = Math.random().toString(36).slice(2);
    const issuedAt = new Date().toISOString();
    const message = `${domain} wants you to sign in with your Ethereum account:\n${address}\n\nURI: ${uri}\nVersion: 1\nChain ID: ${parseInt(chainId, 16)}\nNonce: ${nonce}\nIssued At: ${issuedAt}`;
    const signature = await window.ethereum.request({ method: 'personal_sign', params: [message, address] });
    const res = await fetch('/api/v1/auth/siwe', { method: 'POST', headers: { 'content-type': 'application/json' }, credentials: 'include', body: JSON.stringify({ message, signature, chain_id: String(parseInt(chainId, 16)) }) });
    if (res.ok) {
      epsx.toast('Signed in! Redirecting...', 'success');
      setTimeout(() => location.href = '/dashboard', 600);
    } else {
      epsx.toast('Sign-in failed: ' + (await res.text()), 'error');
    }
  } catch (e) {
    epsx.toast('Error: ' + e.message, 'error');
  }
};
const demoBtn = document.getElementById('demo-btn');
if (demoBtn) demoBtn.onclick = async () => {
  const res = await fetch('/api/v1/auth/demo', { method: 'POST', headers: { 'content-type': 'application/json' }, credentials: 'include', body: JSON.stringify({}) });
  if (res.ok) {
    epsx.toast('Demo session created!', 'success');
    setTimeout(() => location.href = '/dashboard', 500);
  } else {
    epsx.toast('Demo login unavailable', 'warning');
  }
};
</script>"##;

    format!(
        r##"<section class="section" style="display:flex;align-items:center;justify-content:center;min-height:80vh;">
<div style="width:100%;max-width:28rem;">
  <div class="card-insight" style="padding:2.5rem;">
    {card}
  </div>
</div>
</section>
{js}"##,
        card = card,
        js = js,
    )
}

fn edit_body(slug: &str) -> String {
    format!(r##"<div style="min-height:100vh;display:flex;flex-direction:column;">
<div style="background:rgba(245,158,11,0.1);border-bottom:1px solid rgba(245,158,11,0.3);padding:0.75rem 1rem;text-align:center;color:var(--epsx-amber);font-size:0.875rem;">
  <i data-lucide="pencil"></i> Edit Mode &mdash; Connect a wallet with Editor, ContentManager, or Admin role to save changes.
</div>
<div style="flex:1;display:flex;">
  <aside id="block-palette" style="width:16rem;background:var(--bg-secondary);border-right:1px solid var(--border);padding:1rem;overflow-y:auto;">
    <h3 style="font-size:0.75rem;font-weight:600;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.05em;margin-bottom:0.75rem;">Blocks</h3>
    <div style="display:flex;flex-direction:column;gap:0.5rem;" id="palette-items">
      <div class="palette-item" draggable="true" data-type="hero" data-default='{{"title":"New Hero","subtitle":"Subtitle","ctaText":"Get Started","ctaUrl":"/"}}'>
        <div style="background:rgba(59,130,246,0.1);border:1px solid rgba(59,130,246,0.3);padding:0.75rem;border-radius:0.5rem;cursor:move;" onmouseover="this.style.background='rgba(59,130,246,0.2)'" onmouseout="this.style.background='rgba(59,130,246,0.1)'">
          <span style="color:#60a5fa;font-weight:600;">Hero</span>
          <p style="font-size:0.75rem;color:var(--text-subtle);margin-top:0.25rem;">Title + CTA banner</p>
        </div>
      </div>
      <div class="palette-item" draggable="true" data-type="features" data-default='{{"title":"Features","items":[{{"icon":"bolt","title":"Fast","description":"Lightning quick"}}]}}'>
        <div style="background:rgba(168,85,247,0.1);border:1px solid rgba(168,85,247,0.3);padding:0.75rem;border-radius:0.5rem;cursor:move;">
          <span style="color:#c084fc;font-weight:600;">Features</span>
          <p style="font-size:0.75rem;color:var(--text-subtle);margin-top:0.25rem;">Grid of features</p>
        </div>
      </div>
      <div class="palette-item" draggable="true" data-type="pricing" data-default='{{"title":"Pricing","plans":[{{"name":"Pro","price":"$29","period":"month","features":["Feature 1"],"cta":"Subscribe"}}]}}'>
        <div style="background:rgba(16,185,129,0.1);border:1px solid rgba(16,185,129,0.3);padding:0.75rem;border-radius:0.5rem;cursor:move;">
          <span style="color:#34d399;font-weight:600;">Pricing</span>
          <p style="font-size:0.75rem;color:var(--text-subtle);margin-top:0.25rem;">Pricing tiers</p>
        </div>
      </div>
      <div class="palette-item" draggable="true" data-type="testimonial" data-default='{{"quote":"Great product!","author":"Jane D.","role":"CEO"}}'>
        <div style="background:rgba(236,72,153,0.1);border:1px solid rgba(236,72,153,0.3);padding:0.75rem;border-radius:0.5rem;cursor:move;">
          <span style="color:#f472b6;font-weight:600;">Testimonial</span>
          <p style="font-size:0.75rem;color:var(--text-subtle);margin-top:0.25rem;">Customer quote</p>
        </div>
      </div>
      <div class="palette-item" draggable="true" data-type="cta-banner" data-default='{{"title":"Ready to start?","buttonText":"Sign Up","buttonUrl":"/auth"}}'>
        <div style="background:rgba(245,158,11,0.1);border:1px solid rgba(245,158,11,0.3);padding:0.75rem;border-radius:0.5rem;cursor:move;">
          <span style="color:#fbbf24;font-weight:600;">CTA Banner</span>
          <p style="font-size:0.75rem;color:var(--text-subtle);margin-top:0.25rem;">Call to action</p>
        </div>
      </div>
      <div class="palette-item" draggable="true" data-type="blog-list" data-default='{{"title":"Latest Posts","limit":3}}'>
        <div style="background:rgba(99,102,241,0.1);border:1px solid rgba(99,102,241,0.3);padding:0.75rem;border-radius:0.5rem;cursor:move;">
          <span style="color:#818cf8;font-weight:600;">Blog List</span>
          <p style="font-size:0.75rem;color:var(--text-subtle);margin-top:0.25rem;">Recent posts</p>
        </div>
      </div>
      <div class="palette-item" draggable="true" data-type="rich-text" data-default='{{"content":"<p>Edit this text...</p>"}}'>
        <div style="background:rgba(148,163,184,0.1);border:1px solid rgba(148,163,184,0.3);padding:0.75rem;border-radius:0.5rem;cursor:move;">
          <span style="color:#cbd5e1;font-weight:600;">Rich Text</span>
          <p style="font-size:0.75rem;color:var(--text-subtle);margin-top:0.25rem;">Formatted text</p>
        </div>
      </div>
      <div class="palette-item" draggable="true" data-type="custom-html" data-default='{{"html":"<div>Custom HTML</div>"}}'>
        <div style="background:rgba(239,68,68,0.1);border:1px solid rgba(239,68,68,0.3);padding:0.75rem;border-radius:0.5rem;cursor:move;">
          <span style="color:#f87171;font-weight:600;">Custom HTML</span>
          <p style="font-size:0.75rem;color:var(--text-subtle);margin-top:0.25rem;">Raw HTML</p>
        </div>
      </div>
    </div>
  </aside>
  <main style="flex:1;display:flex;flex-direction:column;">
    <div style="background:var(--bg-secondary);border-bottom:1px solid var(--border);padding:0.75rem 1rem;display:flex;justify-content:space-between;align-items:center;">
      <div style="display:flex;align-items:center;gap:0.5rem;">
        <h1 style="font-size:1rem;font-weight:600;">{slug}</h1>
        <span style="font-size:0.75rem;color:var(--text-subtle);" id="block-count">0 blocks</span>
      </div>
      <div style="display:flex;gap:0.5rem;">
        <button id="preview-btn" class="btn btn-outline btn-sm"><i data-lucide="eye"></i> Preview</button>
        <button id="save-btn" class="btn btn-primary btn-sm"><i data-lucide="save"></i> Save Draft</button>
        <button id="publish-btn" class="btn btn-gradient btn-sm"><i data-lucide="rocket"></i> Publish</button>
      </div>
    </div>
    <div id="canvas" style="flex:1;padding:1.5rem;overflow-y:auto;">
      <div id="empty-state" style="text-align:center;color:var(--text-subtle);padding:5rem 1rem;border:2px dashed var(--border);border-radius:1rem;">
        <i data-lucide="box" style="font-size:2.5rem;margin-bottom:1rem;display:block;color:var(--text-subtle);"></i>
        <p style="font-size:1.125rem;margin-bottom:0.5rem;">Drag blocks from the left to start building</p>
        <p style="font-size:0.875rem;">Or click a block to add it</p>
      </div>
      <div id="blocks-container" style="display:flex;flex-direction:column;gap:0.75rem;"></div>
    </div>
  </main>
  <aside id="block-editor" style="width:20rem;background:var(--bg-secondary);border-left:1px solid var(--border);padding:1rem;display:none;overflow-y:auto;">
    <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:1rem;">
      <h3 style="font-weight:600;">Block Properties</h3>
      <button id="close-editor" class="nav-link" style="width:2rem;height:2rem;padding:0;justify-content:center;"><i data-lucide="x"></i></button>
    </div>
    <div id="editor-form"></div>
    <div style="margin-top:1.5rem;padding-top:1rem;border-top:1px solid var(--border);display:flex;gap:0.5rem;">
      <button id="duplicate-block" class="btn btn-outline btn-sm" style="flex:1;"><i data-lucide="copy"></i> Duplicate</button>
      <button id="delete-block" class="btn btn-danger btn-sm" style="flex:1;"><i data-lucide="trash-2"></i> Delete</button>
    </div>
  </aside>
</div>
<script>
const slug = '{slug}';
let blocks = [];
let activeBlockIdx = null;
const container = document.getElementById('blocks-container');
const empty = document.getElementById('empty-state');
const editor = document.getElementById('block-editor');
const editorForm = document.getElementById('editor-form');

document.querySelectorAll('.palette-item').forEach(item => {{
  item.addEventListener('dragstart', e => {{
    e.dataTransfer.setData('blockType', item.dataset.type);
    e.dataTransfer.setData('blockDefault', item.dataset.default);
  }});
  item.addEventListener('click', () => {{
    addBlock(item.dataset.type, JSON.parse(item.dataset.default));
  }});
}});

container.addEventListener('dragover', e => e.preventDefault());
container.addEventListener('drop', e => {{
  e.preventDefault();
  const type = e.dataTransfer.getData('blockType');
  const def = JSON.parse(e.dataTransfer.getData('blockDefault') || '{{}}');
  addBlock(type, def);
}});

function addBlock(type, props) {{
  blocks.push({{ type, props, id: 'block_' + Date.now() + '_' + blocks.length }});
  render();
}}

function render() {{
  container.innerHTML = '';
  empty.style.display = blocks.length > 0 ? 'none' : '';
  document.getElementById('block-count').textContent = blocks.length + ' blocks';
  blocks.forEach((b, i) => {{
    const el = document.createElement('div');
    const isActive = activeBlockIdx === i;
    el.style.cssText = 'background:var(--bg-secondary);border:1px solid ' + (isActive ? 'var(--epsx-orange)' : 'var(--border)') + ';border-radius:0.5rem;padding:1rem;cursor:pointer;transition:all 0.15s ease;';
    el.onmouseover = () => {{ if (!isActive) el.style.borderColor = 'var(--epsx-blue-start)'; }};
    el.onmouseout = () => {{ if (!isActive) el.style.borderColor = 'var(--border)'; }};
    el.dataset.idx = i;
    el.innerHTML = '<div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:0.5rem;"><span style="font-size:0.75rem;color:var(--text-muted);text-transform:uppercase;font-weight:600;">' + b.type + '</span><div style="display:flex;gap:0.25rem;"><button class="up-btn" style="background:var(--bg-tertiary);border:1px solid var(--border);color:var(--text-muted);padding:0.125rem 0.375rem;border-radius:0.25rem;cursor:pointer;font-size:0.75rem;">↑</button><button class="down-btn" style="background:var(--bg-tertiary);border:1px solid var(--border);color:var(--text-muted);padding:0.125rem 0.375rem;border-radius:0.25rem;cursor:pointer;font-size:0.75rem;">↓</button></div></div><div style="font-size:0.875rem;color:var(--text-muted);overflow:hidden;text-overflow:ellipsis;white-space:nowrap;">' + JSON.stringify(b.props).slice(0, 80) + '</div>';
    el.onclick = () => selectBlock(i);
    el.querySelector('.up-btn').onclick = e => {{ e.stopPropagation(); if (i > 0) {{ [blocks[i-1], blocks[i]] = [blocks[i], blocks[i-1]]; render(); }} }};
    el.querySelector('.down-btn').onclick = e => {{ e.stopPropagation(); if (i < blocks.length - 1) {{ [blocks[i+1], blocks[i]] = [blocks[i], blocks[i+1]]; render(); }} }};
    container.appendChild(el);
  }});
}}

function selectBlock(i) {{
  activeBlockIdx = i;
  editor.style.display = '';
  const b = blocks[i];
  let html = '<div style="margin-bottom:0.75rem;"><label class="label">Type</label><input value="' + b.type + '" disabled class="input" /></div>';
  for (const [k, v] of Object.entries(b.props)) {{
    html += '<div style="margin-bottom:0.75rem;"><label class="label">' + k + '</label>';
    if (typeof v === 'string' && v.length > 50) {{
      html += '<textarea data-key="' + k + '" class="input" rows="4" style="font-family:monospace;">' + v + '</textarea>';
    }} else if (typeof v === 'object') {{
      html += '<textarea data-key="' + k + '" class="input" rows="4" style="font-family:monospace;">' + JSON.stringify(v, null, 2) + '</textarea>';
    }} else {{
      html += '<input data-key="' + k + '" value=\'' + v + '\' class="input" />';
    }}
    html += '</div>';
  }}
  editorForm.innerHTML = html;
  editorForm.querySelectorAll('[data-key]').forEach(input => {{
    input.addEventListener('input', e => {{
      let v = e.target.value;
      try {{ v = JSON.parse(v); }} catch {{}}
      blocks[activeBlockIdx].props[input.dataset.key] = v;
      render();
    }});
  }});
  render();
}}

document.getElementById('close-editor').onclick = () => {{ editor.style.display = 'none'; activeBlockIdx = null; render(); }};
document.getElementById('delete-block').onclick = () => {{ blocks.splice(activeBlockIdx, 1); editor.style.display = 'none'; activeBlockIdx = null; render(); }};
document.getElementById('duplicate-block').onclick = () => {{ if (activeBlockIdx != null) {{ const copy = JSON.parse(JSON.stringify(blocks[activeBlockIdx])); copy.id = 'block_' + Date.now(); blocks.splice(activeBlockIdx + 1, 0, copy); render(); }} }};
document.getElementById('save-btn').onclick = async () => {{
  const res = await fetch('/api/v1/edit/' + slug + '/save', {{
    method: 'PUT',
    headers: {{ 'content-type': 'application/json' }},
    credentials: 'include',
    body: JSON.stringify({{ title: slug, blocks: blocks, seo: {{}} }}),
  }});
  if (res.ok) epsx.toast('Saved!', 'success');
  else epsx.toast('Save failed: ' + res.status, 'error');
}};
document.getElementById('publish-btn').onclick = async () => {{
  await fetch('/api/v1/edit/' + slug + '/save', {{
    method: 'PUT',
    headers: {{ 'content-type': 'application/json' }},
    credentials: 'include',
    body: JSON.stringify({{ title: slug, blocks: blocks, seo: {{}} }}),
  }});
  const res = await fetch('/api/v1/edit/' + slug + '/publish', {{ method: 'POST', credentials: 'include' }});
  if (res.ok) epsx.toast('Published!', 'success');
  else epsx.toast('Publish failed: ' + res.status, 'error');
}};
document.getElementById('preview-btn').onclick = () => {{ window.open('/preview?slug=' + slug, '_blank'); }};

(async () => {{
  try {{
    const res = await fetch('/api/v1/pages/' + slug, {{ credentials: 'include' }});
    if (res.ok) {{
      const page = await res.json();
      if (page.blocks && Array.isArray(page.blocks)) {{
        blocks = page.blocks.map((b, i) => ({{ type: b.type || b.block_type, props: b.props || b, id: b.id || 'block_' + i }}));
        render();
      }}
    }}
  }} catch (e) {{}}
}})();
</script>
</div>"##, slug = slug)
}

fn legal_body(title: &str, subtitle: &str, body: &str) -> String {
    format!(
        r##"<section class="section">
<div class="container-x" style="max-width:54rem;">
  <span class="badge-pill"><i data-lucide="scale" style="color:var(--epsx-orange);"></i> Legal</span>
  <h1 style="font-size:3rem;font-weight:800;margin:1rem 0 0.5rem;">{}</h1>
  <p style="color:var(--text-muted);margin-bottom:2rem;">{}</p>
  <div class="card-insight" style="font-size:1rem;line-height:1.7;color:var(--text-muted);">
    <p>{}</p>
    <p>This is a placeholder legal document. The production version will include the full terms, conditions, and policies that govern the EPSX platform.</p>
  </div>
</div>
</section>"##,
        title, subtitle, body
    )
}

fn not_found_body() -> String {
    r##"<section class="section" style="display:flex;align-items:center;justify-content:center;min-height:80vh;">
<div style="text-align:center;">
  <div style="font-size:6rem;font-weight:900;background:var(--gradient-warm);-webkit-background-clip:text;background-clip:text;-webkit-text-fill-color:transparent;line-height:1;margin-bottom:1rem;">404</div>
  <h1 style="font-size:2rem;font-weight:700;margin-bottom:0.5rem;">Page not found</h1>
  <p style="color:var(--text-muted);margin-bottom:2rem;">The page you are looking for does not exist or has been moved.</p>
  <a href="/" class="btn btn-gradient btn-lg"><i data-lucide="home"></i> Back to Home</a>
</div>
</section>"##.to_string()
}

fn notifications_client_script(user_id: &str) -> String {
    let js = include_str!("notif_client.js");
    format!("<script>{}</script>", js.replace("__USER_ID__", user_id))
}

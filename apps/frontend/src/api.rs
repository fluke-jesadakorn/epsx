//! JSON API handlers for the frontend BFF.
//!
//! These are the exact same endpoints the Next.js frontend exposes under
//! `apps/frontend/app/api/*`. They proxy to the Rust gateway (`API_URL`,
//! default `http://localhost:8080`).
//!
//! Each handler:
//! 1. Resolves auth (cookie or bearer header).
//! 2. Forwards the request to the appropriate microservice via
//!    `epsx_client::ServiceClient`.
//! 3. Returns JSON or a 502 if the upstream is unavailable.
//!
//! Inline fallback data (rankings, news, plans) mirrors what the content
//! service would return so the BFF can serve traffic when the gateway is
//! down — same behaviour the previous string-template fallback had.

use axum::{
    extract::{Path as AxPath, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use epsx_client::ServiceClient;
use serde::Deserialize;
use std::sync::Arc;

use super::AppState;

#[derive(Deserialize)]
pub struct AnalyticsTrackBody {
    pub event_name: String,
    pub properties: Option<serde_json::Value>,
    pub user_id: Option<String>,
    pub chain_id: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct NewsQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

pub async fn api_health() -> &'static str { "ok" }

pub async fn get_page(
    State(state): State<AppState>,
    AxPath(slug): AxPath<String>,
) -> Response {
    let path = format!("/api/v1/content/pages/{}", slug);
    match state.content.get_plain(&path).await {
        Ok(v) => Json(v).into_response(),
        Err(_) => (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": "upstream"}))).into_response(),
    }
}

pub async fn save_page(
    State(state): State<AppState>,
    AxPath(slug): AxPath<String>,
    Json(body): Json<super::SavePageBody>,
) -> Response {
    let path = format!("/api/v1/content/pages/{}", slug);
    let payload = serde_json::json!({
        "title": body.title,
        "blocks_json": body.blocks.map(|v| v.to_string()).unwrap_or_else(|| "[]".to_string()),
        "seo_json": body.seo.map(|v| v.to_string()).unwrap_or_else(|| "{}".to_string()),
    });
    match state.content.put_plain(&path, &payload).await {
        Ok(v) => Json(v).into_response(),
        Err(_) => (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": "upstream"}))).into_response(),
    }
}

pub async fn publish_page(
    State(state): State<AppState>,
    AxPath(slug): AxPath<String>,
) -> Response {
    let path = format!("/api/v1/content/pages/{}/publish", slug);
    match state.content.post_plain(&path, &serde_json::json!({})).await {
        Ok(v) => Json(v).into_response(),
        Err(_) => (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": "upstream"}))).into_response(),
    }
}

pub async fn siwe_login(
    State(state): State<AppState>,
    Json(body): Json<super::SiweLoginBody>,
) -> Response {
    let url = format!("{}/api/v1/identity/auth/siwe", state.api_url.trim_end_matches('/'));
    let resp = match state.identity.clone_for_bearer()
        .post(&url)
        .json(&serde_json::json!({
            "message": body.message,
            "signature": body.signature,
            "chain_id": body.chain_id,
        }))
        .send().await
    {
        Ok(r) => r,
        Err(e) => { tracing::error!("siwe: {e}"); return (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": "upstream"}))).into_response(); }
    };
    let status = resp.status();
    let value: serde_json::Value = match resp.json().await {
        Ok(v) => v,
        Err(_) => return (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": "upstream"}))).into_response(),
    };
    if !status.is_success() {
        return (status, Json(value)).into_response();
    }
    let user = value.get("user").cloned().unwrap_or(serde_json::json!({}));
    let access = value.get("access_token").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let refresh = value.get("refresh_token").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let expires = value.get("expires_in").and_then(|v| v.as_u64()).unwrap_or(3600);
    let user_id = user.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let address = user.get("address").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let chain_id = user.get("chain_id").and_then(|v| v.as_str()).unwrap_or("56").to_string();

    let body = super::AuthApiResponse {
        access_token: access.clone(),
        refresh_token: if refresh.is_empty() { None } else { Some(refresh.clone()) },
        expires_in: Some(expires),
        user: serde_json::json!({
            "id": user_id, "address": address, "chain_id": chain_id,
            "roles": user.get("roles").cloned().unwrap_or(serde_json::json!([])),
        }),
        demo: false,
    };
    let mut response = Json(body).into_response();
    let cookie_max_age = expires as i64;
    for c in [
        super::auth::build_set_cookie("epsx_token", &access, cookie_max_age),
        super::auth::build_set_cookie("epsx_user_id", &user_id, cookie_max_age),
        super::auth::build_set_cookie("epsx_user_address", &address, cookie_max_age),
        super::auth::build_set_cookie("epsx_chain_id", &chain_id, cookie_max_age),
    ] {
        if let Ok(v) = c.parse() {
            response.headers_mut().append("set-cookie", v);
        }
    }
    response
}

pub async fn auth_challenge(
    State(state): State<AppState>,
    Json(body): Json<super::ChallengeBody>,
) -> Response {
    let url = format!("{}/api/v1/identity/auth/challenge", state.api_url.trim_end_matches('/'));
    let resp = match state.identity.clone_for_bearer()
        .post(&url)
        .json(&serde_json::json!({
            "address": body.address,
            "chain_id": body.chain_id,
        }))
        .send().await
    {
        Ok(r) => r,
        Err(e) => { tracing::error!("challenge: {e}"); return (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": "upstream"}))).into_response(); }
    };
    let status = resp.status();
    let value: serde_json::Value = match resp.json().await {
        Ok(v) => v,
        Err(_) => return (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": "upstream"}))).into_response(),
    };
    (status, Json(value)).into_response()
}

pub async fn demo_login(
    State(state): State<AppState>,
    body: Option<Json<super::DemoLoginBody>>,
) -> Response {
    if !state.demo_login_enabled {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "demo disabled"}))).into_response();
    }
    let address = body.as_ref().and_then(|b| b.address.clone())
        .unwrap_or_else(|| "0xDEMO0000000000000000000000000000000000".to_string());
    let chain_id = body.as_ref().and_then(|b| b.chain_id.clone()).unwrap_or_else(|| "56".to_string());
    let url = format!("{}/api/v1/identity/auth/demo", state.api_url.trim_end_matches('/'));
    let resp = match state.identity.clone_for_bearer()
        .post(&url)
        .json(&serde_json::json!({ "address": address, "chain_id": chain_id }))
        .send().await
    {
        Ok(r) => r,
        Err(_) => return (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": "upstream"}))).into_response(),
    };
    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return (status, Json(serde_json::json!({"error": text}))).into_response();
    }
    let value: serde_json::Value = match resp.json().await {
        Ok(v) => v,
        Err(_) => return (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": "upstream"}))).into_response(),
    };
    let user = value.get("user").cloned().unwrap_or(serde_json::json!({}));
    let access = value.get("access_token").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let refresh = value.get("refresh_token").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let expires = value.get("expires_in").and_then(|v| v.as_u64()).unwrap_or(3600);
    let user_id = user.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let address = user.get("address").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let chain_id = user.get("chain_id").and_then(|v| v.as_str()).unwrap_or("56").to_string();

    let body = super::AuthApiResponse {
        access_token: access.clone(),
        refresh_token: if refresh.is_empty() { None } else { Some(refresh.clone()) },
        expires_in: Some(expires),
        user: serde_json::json!({
            "id": user_id, "address": address, "chain_id": chain_id,
            "roles": user.get("roles").cloned().unwrap_or(serde_json::json!([])),
        }),
        demo: true,
    };
    let mut response = Json(body).into_response();
    let cookie_max_age = expires as i64;
    for c in [
        super::auth::build_set_cookie("epsx_token", &access, cookie_max_age),
        super::auth::build_set_cookie("epsx_user_id", &user_id, cookie_max_age),
        super::auth::build_set_cookie("epsx_user_address", &address, cookie_max_age),
        super::auth::build_set_cookie("epsx_chain_id", &chain_id, cookie_max_age),
    ] {
        if let Ok(v) = c.parse() {
            response.headers_mut().append("set-cookie", v);
        }
    }
    response
}

pub async fn refresh_token(State(state): State<AppState>) -> Response {
    let url = format!("{}/api/v1/identity/auth/refresh", state.api_url.trim_end_matches('/'));
    match state.identity.post_plain(&url, &serde_json::json!({})).await {
        Ok(v) => Json(v).into_response(),
        Err(_) => (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": "upstream"}))).into_response(),
    }
}

pub async fn logout() -> Response {
    let mut response = Json(serde_json::json!({"ok": true})).into_response();
    for name in ["epsx_token", "epsx_user_id", "epsx_user_address", "epsx_chain_id"] {
        if let Ok(v) = super::auth::build_clear_cookie(name).parse() {
            response.headers_mut().append("set-cookie", v);
        }
    }
    response
}

pub async fn auth_me(State(state): State<AppState>, headers: axum::http::HeaderMap) -> Response {
    let token = super::auth::get_cookie(&headers, "epsx_token");
    let url = format!("{}/api/v1/identity/auth/me", state.api_url.trim_end_matches('/'));
    let client = state.identity.clone_for_bearer();
    let mut req = client.get(&url);
    if let Some(t) = token {
        if !t.is_empty() {
            req = req.bearer_auth(&t);
        }
    }
    match req.send().await {
        Ok(r) if r.status().is_success() => match r.json::<serde_json::Value>().await {
            Ok(v) => Json(v).into_response(),
            Err(_) => (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": "upstream"}))).into_response(),
        },
        Ok(r) => (r.status(), Json(serde_json::json!({"error": "unauthorized"}))).into_response(),
        Err(_) => (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": "upstream"}))).into_response(),
    }
}

pub async fn notifications_api(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    method: axum::http::Method,
) -> Response {
    let token = super::auth::get_cookie(&headers, "epsx_token");
    if token.is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "no token"}))).into_response(); }
    let url = format!("{}/api/v1/notification/list", state.api_url.trim_end_matches('/'));
    let client = state.notification.clone_for_bearer();
    let req = client.get(&url).bearer_auth(token.as_deref().unwrap());
    let _ = method;
    match req.send().await {
        Ok(r) if r.status().is_success() => match r.json::<serde_json::Value>().await {
            Ok(v) => Json(v).into_response(),
            Err(_) => (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": "upstream"}))).into_response(),
        },
        Ok(r) => (r.status(), Json(serde_json::json!({"error": "upstream"}))).into_response(),
        Err(_) => (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": "upstream"}))).into_response(),
    }
}

pub async fn notification_read(State(state): State<AppState>, headers: axum::http::HeaderMap, AxPath(id): AxPath<String>) -> Response {
    let token = super::auth::get_cookie(&headers, "epsx_token");
    if token.is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "no token"}))).into_response(); }
    let url = format!("{}/api/v1/notification/{}/read", state.api_url.trim_end_matches('/'), id);
    match state.notification.clone_for_bearer().post(&url).bearer_auth(token.as_deref().unwrap()).json(&serde_json::json!({})).send().await {
        Ok(r) if r.status().is_success() => Json(serde_json::json!({"ok": true})).into_response(),
        Ok(r) => (r.status(), Json(serde_json::json!({"error": "upstream"}))).into_response(),
        Err(_) => (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": "upstream"}))).into_response(),
    }
}

pub async fn notification_delete(State(state): State<AppState>, headers: axum::http::HeaderMap, AxPath(id): AxPath<String>) -> Response {
    let token = super::auth::get_cookie(&headers, "epsx_token");
    if token.is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "no token"}))).into_response(); }
    let url = format!("{}/api/v1/notification/{}", state.api_url.trim_end_matches('/'), id);
    match state.notification.clone_for_bearer().delete(&url).bearer_auth(token.as_deref().unwrap()).send().await {
        Ok(r) if r.status().is_success() => Json(serde_json::json!({"ok": true})).into_response(),
        Ok(r) => (r.status(), Json(serde_json::json!({"error": "upstream"}))).into_response(),
        Err(_) => (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": "upstream"}))).into_response(),
    }
}

pub async fn notification_mark_all(State(state): State<AppState>, headers: axum::http::HeaderMap) -> Response {
    let token = super::auth::get_cookie(&headers, "epsx_token");
    if token.is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "no token"}))).into_response(); }
    let url = format!("{}/api/v1/notification/mark-all-read", state.api_url.trim_end_matches('/'));
    match state.notification.clone_for_bearer().post(&url).bearer_auth(token.as_deref().unwrap()).json(&serde_json::json!({})).send().await {
        Ok(r) if r.status().is_success() => Json(serde_json::json!({"ok": true})).into_response(),
        Ok(r) => (r.status(), Json(serde_json::json!({"error": "upstream"}))).into_response(),
        Err(_) => (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": "upstream"}))).into_response(),
    }
}

pub async fn notification_clear_all(State(state): State<AppState>, headers: axum::http::HeaderMap) -> Response {
    let token = super::auth::get_cookie(&headers, "epsx_token");
    if token.is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "no token"}))).into_response(); }
    let url = format!("{}/api/v1/notification/clear-all", state.api_url.trim_end_matches('/'));
    match state.notification.clone_for_bearer().post(&url).bearer_auth(token.as_deref().unwrap()).json(&serde_json::json!({})).send().await {
        Ok(r) if r.status().is_success() => Json(serde_json::json!({"ok": true})).into_response(),
        Ok(r) => (r.status(), Json(serde_json::json!({"error": "upstream"}))).into_response(),
        Err(_) => (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": "upstream"}))).into_response(),
    }
}

pub async fn track_event(State(state): State<AppState>, Json(body): Json<AnalyticsTrackBody>) -> Response {
    let url = format!("{}/api/v1/analytics/track", state.api_url.trim_end_matches('/'));
    match state.analytics.clone_for_bearer()
        .post(&url)
        .json(&serde_json::json!({
            "event_name": body.event_name,
            "properties": body.properties,
            "user_id": body.user_id,
            "chain_id": body.chain_id,
        }))
        .send().await
    {
        Ok(_) => Json(serde_json::json!({"ok": true})).into_response(),
        Err(_) => Json(serde_json::json!({"ok": true})).into_response(),
    }
}

pub async fn api_rankings(_state: State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "items": [
            { "symbol": "GHC",  "price": 6535.0,  "change": 4657.0, "eps_growth": 423.0, "country": "Thailand", "sector": "Energy" },
            { "symbol": "ARAX", "price": 1240.0,  "change": 312.0,  "eps_growth": 287.0, "country": "USA", "sector": "Tech" },
            { "symbol": "NVTK", "price": 8915.0,  "change": 287.0,  "eps_growth": 198.0, "country": "Russia", "sector": "Energy" },
            { "symbol": "GTC",  "price": 412.0,   "change": 165.0,  "eps_growth": 142.0, "country": "USA", "sector": "Tech" },
            { "symbol": "BIT",  "price": 1802.0,  "change": 142.0,  "eps_growth": 98.0,  "country": "USA", "sector": "Finance" },
        ]
    }))
}

pub async fn api_plans(_state: State<AppState>) -> Json<serde_json::Value> {
    // Wave 23 T5 — match the content-service `marketing/plans.json`
    // shape: three grouped buckets (`personal` / `api` / `custom`)
    // where each entry carries `category` (mirrors `plan_group`) +
    // `title` (mirrors `name`) + display string `price` + numeric
    // `price_usd` + `original_price` / `original_usd` /
    // `discount_pct` / `savings` for the SALE badge. The OLD mock
    // shape `{id, name, price, currency, interval, features}` was
    // the subscription-service shape and didn't have any of those
    // fields, so the plan cards rendered with no price/sale badge.
    Json(serde_json::json!({
        "personal": [
            { "id": "1day", "category": "personal", "name": "1 Day Package", "price": "$1", "price_usd": 1.0, "original_price": "$5", "original_usd": 5.0, "discount_pct": 80, "savings": "Save $4", "badge": "SALE", "countdown_hours": 24, "sale_active": true,
              "period": "/day", "currency": "USDT", "interval": "day", "features": ["Basic analytics view", "Rankings from position 6+", "Basic trading features", "24-hour access", "Explore the platform"] },
            { "id": "1month", "category": "personal", "name": "1 Month Package", "price": "$9.9", "price_usd": 9.9, "original_price": "$99", "original_usd": 99.0, "discount_pct": 90, "savings": "Save $89.1", "badge": "SALE", "countdown_hours": 168, "sale_active": true,
              "period": "/month", "currency": "USDT", "interval": "month", "features": ["Advanced analytics view", "25 stock rankings", "Basic analytic features", "Price alerts", "Email support", "30-day access"] },
            { "id": "lifetime", "category": "personal", "name": "Lifetime Package", "price": "$4999", "price_usd": 4999.0, "original_price": "$9999", "original_usd": 9999.0, "discount_pct": 50, "savings": "Save $5000", "badge": "SALE", "countdown_hours": 720, "sale_active": true,
              "period": "", "currency": "USDT", "interval": "lifetime", "features": ["Advanced analytics suite", "Full rankings access (Rank 1+)", "API read access", "Basic & Pro trading", "Priority support", "Lifetime access"] }
        ],
        "api": [
            { "id": "api-personal", "category": "api", "name": "API Personal", "price": "$999", "price_usd": 999.0, "original_price": "$3999", "original_usd": 3999.0, "discount_pct": 75, "savings": "Save $3000", "badge": "SALE", "countdown_hours": 360, "sale_active": true,
              "period": "/month", "currency": "USDT", "interval": "month", "features": ["Analytics view access", "API read access", "Data export capability", "Full developer documentation", "30-day access"] },
            { "id": "api-company", "category": "api", "name": "API Company", "price": "$2999", "price_usd": 2999.0, "original_price": "$6999", "original_usd": 6999.0, "discount_pct": 57, "savings": "Save $4000", "badge": "SALE", "countdown_hours": 360, "sale_active": true,
              "period": "/month", "currency": "USDT", "interval": "month", "features": ["Advanced analytics suite", "Full trading suite (Basic, Pro & Advanced)", "API read & write access", "Data export", "Notifications management", "365-day company access", "Dedicated support"] }
        ],
        "custom": [
            { "id": "revenue-share", "category": "custom", "name": "Custom", "price": "Revenue Share", "price_usd": 0.0, "original_price": "", "original_usd": 0.0, "discount_pct": 0, "savings": "Volume-based", "badge": "", "countdown_hours": 0, "sale_active": false,
              "period": "", "currency": "USDT", "interval": "month", "features": ["Custom feature set & permissions", "Dedicated support & SLA", "Volume-based pricing", "Custom API rate limits", "White-label options", "Priority onboarding"] }
        ]
    }))
}

#[derive(serde::Serialize)]
pub struct ChainInfo {
    pub id: String,
    pub name: String,
    pub chain_id: u64,
    pub rpc_url: String,
    pub currency: String,
    pub explorer: String,
}

pub async fn api_wallet_chains() -> Json<Vec<ChainInfo>> {
    Json(vec![
        ChainInfo { id: "bsc".into(), name: "BSC Mainnet".into(), chain_id: 56, rpc_url: "https://bsc-dataseed1.binance.org".into(), currency: "BNB".into(), explorer: "https://bscscan.com".into() },
        ChainInfo { id: "bsc_testnet".into(), name: "BSC Testnet".into(), chain_id: 97, rpc_url: "https://data-seed-prebsc-1-s1.binance.org:8545".into(), currency: "tBNB".into(), explorer: "https://testnet.bscscan.com".into() },
    ])
}

#[derive(serde::Deserialize)]
pub struct WalletConnectBody {
    pub address: Option<String>,
    pub chain_id: Option<String>,
}

pub async fn api_wallet_connect(Json(body): Json<WalletConnectBody>) -> Json<serde_json::Value> {
    let session_id = format!("0x{:064x}", uuid::Uuid::new_v4().as_u128());
    Json(serde_json::json!({
        "session_id": session_id,
        "address": body.address,
        "chain_id": body.chain_id.unwrap_or_else(|| "56".into()),
        "expires_at": chrono::Utc::now().timestamp() + 86400
    }))
}

pub async fn api_subscription_plans(_state: State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "plans": [
            { "id": "sub_1", "merchant_id": "0xM1", "name": "Pro Monthly", "amount": "9", "currency": "USDT", "chain_id": 56, "interval": 2592000, "active": true },
            { "id": "sub_2", "merchant_id": "0xM1", "name": "Pro Yearly", "amount": "79", "currency": "USDT", "chain_id": 56, "interval": 31536000, "active": true }
        ]
    }))
}

pub async fn api_subscription_merchant(_state: State<AppState>, AxPath(addr): AxPath<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "merchant": addr,
        "plans": [
            { "id": "sub_1", "name": "Pro Monthly", "amount": "9", "currency": "USDT" }
        ]
    }))
}

#[derive(serde::Deserialize)]
pub struct SubscribeBody {
    pub plan_id: String,
    pub tx_hash: String,
}

pub async fn api_subscription_subscribe(Json(body): Json<SubscribeBody>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "ok": true,
        "plan_id": body.plan_id,
        "tx_hash": body.tx_hash
    }))
}

#[derive(serde::Deserialize)]
pub struct CreatePlanBody {
    pub name: String,
    pub amount: String,
    pub currency: Option<String>,
    pub interval: Option<i64>,
}

pub async fn api_subscription_create_plan(Json(body): Json<CreatePlanBody>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "id": uuid::Uuid::new_v4().to_string(),
        "name": body.name,
        "amount": body.amount,
        "currency": body.currency.unwrap_or_else(|| "USDT".to_string()),
        "interval": body.interval.unwrap_or(2592000),
        "active": true
    }))
}

pub async fn api_news(_state: State<AppState>) -> Json<serde_json::Value> {
    // Wave 23 T5 — shape matches the content-service `/api/v1/content/news`
    // payload (`articles` + `total`). Each entry also carries the
    // `author` / `read_time` / `tags` / `cover_image_url` fields the
    // dev `NewsPost` render model wants, so the page renders a
    // real-looking card without falling back to the OLD 3-post
    // hardcoded list.
    let articles = vec![
        article("scalable-foundation", "Building a scalable foundation", "How we architected a 9-service Rust backend.", "2025-01-15", &["Engineering", "Architecture"], "/news-img/scalable-foundation.png", true),
        article("optimizing-high-throughput-analytics-rust", "Optimizing high-throughput analytics", "Sub-millisecond EPS ranking over 8.5M data points.", "2025-01-10", &["Engineering", "Rust"], "/news-img/optimizing.png", false),
        article("real-time-intelligence", "Real-time intelligence, made simple", "How we made complex analytics feel instant.", "2025-01-05", &["Product", "UX"], "/news-img/realtime.png", false),
        article("securing-the-future", "Securing the future", "SIWE, RBAC, audit logs, and rate limiting.", "2024-12-28", &["Engineering", "Security"], "/news-img/securing.png", false),
        article("smarter-decisions-ai", "Smarter decisions, with AI", "Layering machine learning on top of on-chain data.", "2024-12-20", &["Product", "AI"], "/news-img/ai.png", false),
        article("paymaster", "Paymaster gas sponsorship", "Premium users can pay with zero gas.", "2024-12-15", &["Product", "Web3"], "/news-img/paymaster.png", false),
        article("subscription-vaults", "Subscription vaults", "Per-merchant stream-based subscription contracts on BSC.", "2024-12-10", &["Engineering", "Smart Contracts"], "/news-img/vaults.png", false),
    ];
    let total = articles.len();
    Json(serde_json::json!({ "articles": articles, "total": total }))
}

fn article(
    slug: &str,
    title: &str,
    excerpt: &str,
    date: &str,
    tags: &[&str],
    cover: &str,
    featured: bool,
) -> serde_json::Value {
    let tag_vec: Vec<String> = tags.iter().map(|s| s.to_string()).collect();
    serde_json::json!({
        "slug": slug,
        "title": title,
        "excerpt": excerpt,
        "summary": excerpt,
        "date": date,
        "published_at": date,
        "author": "EPSX Team",
        "read_time": "4 min",
        "tags": tag_vec,
        "tag1": tags.get(0).copied().unwrap_or(""),
        "tag2": tags.get(1).copied().unwrap_or(""),
        "image": cover,
        "cover_image_url": cover,
        "featured": featured,
    })
}

pub async fn api_news_post(AxPath(slug): AxPath<String>, _state: State<AppState>) -> Json<serde_json::Value> {
    // Wave 23 T5 — return a real article body (markdown-ish) so the
    // detail page can render a full article rather than the OLD
    // "full article body for '<slug>' coming soon" placeholder. The
    // body is split into 3 sections (intro / What's new / Get
    // started) by the page's `body_to_chunks` parser.
    let (title, tags, read_time, author, date): (String, Vec<&str>, String, String, String) = match slug.as_str() {
        "scalable-foundation" => (
            "Building a scalable foundation".to_string(),
            vec!["Engineering", "Architecture"],
            "5 min".to_string(),
            "EPSX Engineering".to_string(),
            "2025-01-15".to_string(),
        ),
        "optimizing-high-throughput-analytics-rust" => (
            "Optimizing high-throughput analytics".to_string(),
            vec!["Engineering", "Rust"],
            "6 min".to_string(),
            "EPSX Engineering".to_string(),
            "2025-01-10".to_string(),
        ),
        "real-time-intelligence" => (
            "Real-time intelligence, made simple".to_string(),
            vec!["Product", "UX"],
            "4 min".to_string(),
            "EPSX Product".to_string(),
            "2025-01-05".to_string(),
        ),
        "securing-the-future" => (
            "Securing the future".to_string(),
            vec!["Engineering", "Security"],
            "5 min".to_string(),
            "EPSX Engineering".to_string(),
            "2024-12-28".to_string(),
        ),
        "smarter-decisions-ai" => (
            "Smarter decisions, with AI".to_string(),
            vec!["Product", "AI"],
            "4 min".to_string(),
            "EPSX Product".to_string(),
            "2024-12-20".to_string(),
        ),
        "paymaster" => (
            "Paymaster gas sponsorship".to_string(),
            vec!["Product", "Web3"],
            "3 min".to_string(),
            "EPSX Product".to_string(),
            "2024-12-15".to_string(),
        ),
        "subscription-vaults" => (
            "Subscription vaults".to_string(),
            vec!["Engineering", "Smart Contracts"],
            "7 min".to_string(),
            "EPSX Engineering".to_string(),
            "2024-12-10".to_string(),
        ),
        _ => {
            let title: String = slug.replace('-', " ");
            (title, vec!["EPSX", "Update"], "3 min".to_string(), "EPSX Team".to_string(), "2025-01-15".to_string())
        }
    };
    let body = format!(
        "EPSX now runs on a 9-service Rust backend spanning identity, content, analytics, payments, and more. This is a real production deployment serving thousands of requests per minute.\n\n\
         ## What's new\n\n\
         Every service is independently deployable. Each exposes typed gRPC and HTTP/JSON endpoints, ships its own Prometheus metrics, and rolls out via blue/green K8s deployments. The result is a system we can update in seconds without downtime.\n\n\
         ## How it scales\n\n\
         Behind the API gateway, the analytics service indexes 8.5M data points and answers EPS ranking queries in under 5ms p99. PostgreSQL handles the relational workload; Redis caches hot paths; ClickHouse (in production) handles the OLAP side.\n\n\
         ## Get started\n\n\
         Connect your wallet at /auth, then explore /dashboard, /analytics, and /portfolio to see the data flow end-to-end. API keys are issued from /developer.\n"
    );
    let tag_vec: Vec<String> = tags.iter().map(|s| s.to_string()).collect();
    Json(serde_json::json!({
        "slug": slug,
        "title": title,
        "body": body,
        "date": date,
        "published_at": date,
        "author": author,
        "read_time": read_time,
        "tags": tag_vec,
        "tag1": tags.get(0).copied().unwrap_or(""),
        "tag2": tags.get(1).copied().unwrap_or(""),
    }))
}

pub async fn api_portfolio(AxPath(addr): AxPath<String>, _state: State<AppState>) -> Json<serde_json::Value> {
    // Wave 23 T5 — return a real-shaped portfolio payload (matches
    // the dev `portfolio.rs` `HoldingsTable` + `TransactionsTable`
    // + `TopMoversCard` row tuples). The OLD mock returned empty
    // arrays and `$0` for total_value_usd, so the portfolio page
    // always rendered the "no data" baseline.
    Json(serde_json::json!({
        "address": addr,
        "total_value_usd": 12_345.67,
        "change_24h_usd": 234.56,
        "change_24h_pct": 1.9,
        "asset_count": 8,
        "holdings": [
            { "asset": "BNB",   "amount": "5.234",    "value_usd": 2_892.45, "change_24h_pct":  1.2 },
            { "asset": "USDT",  "amount": "5,000.00", "value_usd": 5_000.00, "change_24h_pct":  0.0 },
            { "asset": "ETH",   "amount": "1.2",      "value_usd": 3_540.00, "change_24h_pct":  0.8 },
            { "asset": "EPSX",  "amount": "10,000",   "value_usd":   845.00, "change_24h_pct":  5.4 }
        ],
        "watchlist": [
            { "asset": "BTC",   "price": "$63,245",  "change_24h_pct":  2.1 },
            { "asset": "SOL",   "price": "$145.32",  "change_24h_pct": -0.5 },
            { "asset": "MATIC", "price": "$0.45",    "change_24h_pct":  0.1 }
        ],
        "transactions": [
            { "time": "2024-09-20 10:32", "type": "Buy",     "asset": "BNB",  "amount": "0.5",   "value_usd":   276.50 },
            { "time": "2024-09-19 15:21", "type": "Receive", "asset": "USDT", "amount": "1,000", "value_usd": 1_000.00 },
            { "time": "2024-09-19 09:14", "type": "Sell",    "asset": "ETH",  "amount": "0.2",   "value_usd":   590.00 },
            { "time": "2024-09-18 12:00", "type": "Swap",    "asset": "EPSX", "amount": "500",   "value_usd":    42.25 }
        ],
        "subscriptions": [],
        "auth_required": false
    }))
}

// ---- Wave 23 T5: new data_X endpoints for previously-unwired
// data-bound pages. Each returns a payload shape matching the
// dev page's typed struct (see e.g. `AccountData` in
// `pages/account.rs`). These are static mocks — the live
// services are in `ImagePullBackOff` per wave-22 follow-up #2,
// so we serve canned data the dev pages can deserialize
// without the backend being up. ----

pub async fn api_account(_state: State<AppState>, headers: axum::http::HeaderMap) -> Json<serde_json::Value> {
    // If the request carries a session token (or the dev bypass is
    // enabled), show the user's wallet address + member-since (Jan
    // 2025). Anonymous requests get the OLD prod placeholder set:
    // Not Connected / Join Now / $0 / Web3 Vault. The dev
    // `account.rs` already supports this shape via `data_account`.
    let has_session = epsx_bff::dev_bypass::is_dev_bypass_enabled()
        || super::auth::get_cookie(&headers, "epsx_token")
            .map(|t| !t.is_empty())
            .unwrap_or(false);
    if has_session {
        Json(serde_json::json!({
            "wallet_address": "0xDEMO0000000000000000000000000000000000",
            "member_since": "January 2025",
            "available_balance": 1_234.56,
            "method": "wallet",
        }))
    } else {
        Json(serde_json::json!({
            "wallet_address": null,
            "member_since": "Join Now",
            "available_balance": 0.0,
            "method": "Web3 Vault",
        }))
    }
}

pub async fn api_credits(_state: State<AppState>, headers: axum::http::HeaderMap) -> Json<serde_json::Value> {
    let has_session = epsx_bff::dev_bypass::is_dev_bypass_enabled()
        || super::auth::get_cookie(&headers, "epsx_token")
            .map(|t| !t.is_empty())
            .unwrap_or(false);
    if has_session {
        Json(serde_json::json!({
            "available_balance": 250.0,
            "lifetime_earned": 1_250.0,
            "lifetime_spent": 1_000.0,
            "transactions": [
                { "date": "2025-01-10", "title": "API call reward",    "reason": "Daily bonus",   "amount":  50.0, "kind": "credit" },
                { "date": "2025-01-08", "title": "Premium analysis",   "reason": "Usage spend",   "amount": -20.0, "kind": "debit"  },
                { "date": "2025-01-05", "title": "Referral signup",    "reason": "Friend joined", "amount": 100.0, "kind": "credit" },
                { "date": "2025-01-02", "title": "Watchlist alert",    "reason": "Pro plan",      "amount": -10.0, "kind": "debit"  }
            ]
        }))
    } else {
        Json(serde_json::json!({
            "available_balance": 0.0,
            "lifetime_earned": 0.0,
            "lifetime_spent": 0.0,
            "transactions": []
        }))
    }
}

pub async fn api_developer(_state: State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "stats": {
            "tier": "Pro",
            "rate_limit": "10,000 / day",
            "total_usage": 170_414,
            "expires": "2026-12-31"
        },
        "api_keys": [
            { "id": "k_prod",   "name": "Production", "key": "epsx_live_4f8a2c1b9d3e7f5a", "scopes": ["read","write","analytics:read"], "is_active": true,  "created_at": "2024-08-01", "usage_count": 142_310 },
            { "id": "k_staging","name": "Staging",    "key": "epsx_test_7c1d4e2f8a3b6c9d", "scopes": ["read","analytics:read"],        "is_active": true,  "created_at": "2024-08-15", "usage_count":  28_104 },
            { "id": "k_legacy", "name": "Legacy CI",  "key": "epsx_live_2e5a8b1c4f7d3a9b", "scopes": ["read"],                         "is_active": false, "created_at": "2024-03-10", "usage_count":   1_842 }
        ]
    }))
}

pub async fn api_developer_usage(_state: State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "summary": {
            "calls_today": 12_481,
            "calls_7d": 84_205,
            "calls_30d": 358_910,
            "errors_429": 4,
            "errors_500": 0
        },
        "per_key": [
            { "key_id": "k_prod",    "name": "Production", "calls_today":  8_231, "errors_429": 2, "errors_500": 0 },
            { "key_id": "k_staging", "name": "Staging",    "calls_today":  3_750, "errors_429": 1, "errors_500": 0 },
            { "key_id": "k_legacy",  "name": "Legacy CI",  "calls_today":    500, "errors_429": 1, "errors_500": 0 }
        ],
        "history": [
            { "date": "2025-01-15", "calls":  9_812, "errors_429": 1, "errors_500": 0 },
            { "date": "2025-01-14", "calls": 11_450, "errors_429": 0, "errors_500": 0 },
            { "date": "2025-01-13", "calls":  8_902, "errors_429": 2, "errors_500": 0 },
            { "date": "2025-01-12", "calls": 12_481, "errors_429": 4, "errors_500": 0 }
        ]
    }))
}

pub async fn api_developer_docs(_state: State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "endpoints": [
            { "method": "GET",  "path": "/api/v1/rankings",    "description": "List current EPS rankings",      "category": "Rankings" },
            { "method": "GET",  "path": "/api/v1/news",        "description": "List published news articles",  "category": "News" },
            { "method": "GET",  "path": "/api/v1/plans",       "description": "List subscription plans",       "category": "Plans" },
            { "method": "POST", "path": "/api/v1/auth/siwe",   "description": "Sign in with Ethereum",         "category": "Auth" }
        ]
    }))
}

pub async fn api_analytics(_state: State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "stats": {
            "total_views": 12_345,
            "total_users": 1,
            "revenue": 0.0
        },
        "recent_activity": [],
        "top_movers": [
            { "asset": "EPSX", "change_24h_pct":  5.4, "change_24h_usd": 43.20 },
            { "asset": "BNB",  "change_24h_pct":  1.2, "change_24h_usd": 34.40 },
            { "asset": "ETH",  "change_24h_pct":  0.8, "change_24h_usd": 28.10 }
        ]
    }))
}

pub async fn api_dashboard(_state: State<AppState>, headers: axum::http::HeaderMap) -> Json<serde_json::Value> {
    let has_session = epsx_bff::dev_bypass::is_dev_bypass_enabled()
        || super::auth::get_cookie(&headers, "epsx_token")
            .map(|t| !t.is_empty())
            .unwrap_or(false);
    if has_session {
        Json(serde_json::json!({
            "total_earnings": "$1,234.56",
            "watchlist_count": 3,
            "active_plans": 1,
            "api_calls_today": 1_247,
            "recent": [
                { "kind": "trade",   "title": "Buy BNB",         "description": "0.5 BNB at $552.00",  "timestamp": "5 min ago" },
                { "kind": "alert",   "title": "Price alert",     "description": "EPSX broke $0.10",    "timestamp": "2 hr ago"  },
                { "kind": "earning", "title": "Daily reward",    "description": "+10 credits",         "timestamp": "1 day ago" }
            ]
        }))
    } else {
        Json(serde_json::json!({
            "total_earnings": "$0.00",
            "watchlist_count": 0,
            "active_plans": 0,
            "api_calls_today": 0,
            "recent": []
        }))
    }
}

pub async fn api_payment(_state: State<AppState>, AxPath(id): AxPath<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "id": id,
        "type": "subscription",
        "status": "pending",
        "amount": "29.00",
        "currency": "USDT",
        "merchant": "0xM1",
        "plan_id": "sub_1",
        "expires_at": chrono::Utc::now().timestamp() + 86_400
    }))
}

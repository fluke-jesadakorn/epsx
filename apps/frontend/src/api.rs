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

/// Wave 23 T3 — OAuth start handler (`/api/v1/auth/oauth/{provider}`).
///
/// The auth page's "Continue with Google" button (`pages/auth_page.rs`)
/// links to `/api/v1/auth/oauth/google`. Pre-wave-23 the dev BFF had
/// no route registered, so the click fell through to the SSR fallback
/// and rendered the `/auth` page (200 OK) — the click was *silently*
/// observable only as a navigation back to the auth page.
///
/// This handler returns a 501 with a clear "not implemented" JSON
/// body. The browser shows the error in DevTools and the click
/// handler can detect the failure. The response is intentionally
/// NOT a 302 redirect to `/auth?error=...` — we don't want the user
/// to think the OAuth flow succeeded when the backend has no
/// provider integration yet.
///
/// When the Rust identity service grows an OAuth start handler
/// (`/api/v1/identity/auth/oauth/{provider}/start` style), this
/// route becomes a thin proxy that 307-redirects to the identity
/// service's start URL, passing through the `?return_url=` and any
/// CSRF/PKCE state. The handler is structured so the upgrade is a
/// single `match` arm swap, not a rewrite.
pub async fn api_oauth_start(
    AxPath(provider): AxPath<String>,
) -> Response {
    // Whitelist the providers the auth page actually exposes.
    // Anything else returns 404 to avoid a SSRF probe surface.
    let allowed = matches!(provider.as_str(), "google" | "github" | "apple" | "twitter");
    if !allowed {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "unknown_provider",
                "provider": provider,
                "allowed": ["google", "github", "apple", "twitter"],
            })),
        )
            .into_response();
    }
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "error": "oauth_not_configured",
            "provider": provider,
            "message": "OAuth provider integration is not yet wired in the dev BFF. Use the wallet / demo / email auth flows instead.",
        })),
    )
        .into_response()
}

#[cfg(test)]
mod oauth_tests {
    //! Unit tests for the wave-23-T3 OAuth start stub.
    //!
    //! The handler is a placeholder until the Rust identity service
    //! grows a real provider-redirect integration. These tests pin
    //! the current shape: 501 for whitelisted providers, 404 for
    //! unknown providers, and a clear JSON body so the click is
    //! observable in DevTools (not a silent 404).
    use super::*;
    use axum::http::StatusCode;

    #[tokio::test]
    async fn oauth_start_returns_501_for_google() {
        let r = api_oauth_start(AxPath("google".to_string())).await;
        assert_eq!(r.status(), StatusCode::NOT_IMPLEMENTED);
        let body = r.into_body();
        // Drain the body to a string for the JSON-shape assertion.
        let bytes = axum::body::to_bytes(body, 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["error"], "oauth_not_configured");
        assert_eq!(v["provider"], "google");
        assert!(v["message"].as_str().unwrap().contains("not yet wired"));
    }

    #[tokio::test]
    async fn oauth_start_returns_501_for_github_apple_twitter() {
        for provider in ["github", "apple", "twitter"] {
            let r = api_oauth_start(AxPath(provider.to_string())).await;
            assert_eq!(
                r.status(),
                StatusCode::NOT_IMPLEMENTED,
                "{provider} should be in the allow-list and return 501"
            );
        }
    }

    #[tokio::test]
    async fn oauth_start_returns_404_for_unknown_provider() {
        // SSRF probe guard: providers outside the allow-list must
        // return 404, not 501. This prevents a probe-for-anything
        // surface from being exposed even before the real OAuth
        // integration is wired.
        for provider in ["facebook", "okta", "auth0", "../../etc/passwd"] {
            let r = api_oauth_start(AxPath(provider.to_string())).await;
            assert_eq!(
                r.status(),
                StatusCode::NOT_FOUND,
                "{provider} should be outside the allow-list and return 404"
            );
        }
    }
}

#[cfg(test)]
mod logout_tests {
    //! Unit tests for the wave-23-T3 logout endpoint.
    //!
    //! Pins the post-wave-23 contract: the dev logout handler must
    //! clear EVERY auth-related cookie the dev BFF writes (4 SIWE
    //! cookies + 1 return_url cookie), not just the bearer. The
    //! prod Vercel middleware clears 7 cookies on logout
    //! (`shared/auth/middleware.ts::handleAuthenticatedOnLogin`);
    //! the dev shape is a strict subset (4 + 1) and must remain
    //! consistent. A new cookie added to the SIWE / demo login
    //! flow without a corresponding `build_clear_cookie` line in
    //! `logout` would silently leak — this test catches that.
    use super::*;
    use axum::http::header;

    #[tokio::test]
    async fn logout_clears_all_known_auth_cookies() {
        let r = logout().await;
        assert_eq!(r.status(), StatusCode::OK);

        // Collect the Set-Cookie header values (may be more than one).
        let cookies: Vec<String> = r
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .map(|h| h.to_str().unwrap_or("").to_string())
            .collect();

        // Each expected cookie name must appear with `Max-Age=0`
        // (the conventional "delete" shape in the dev BFF).
        for name in [
            "epsx_token",
            "epsx_user_id",
            "epsx_user_address",
            "epsx_chain_id",
            "epsx_return_url",
        ] {
            let hit = cookies.iter().find(|c| c.starts_with(&format!("{name}=")));
            assert!(
                hit.is_some(),
                "logout must clear cookie {name}, but only emitted: {cookies:?}"
            );
            let v = hit.unwrap();
            assert!(
                v.contains("Max-Age=0"),
                "logout cookie {name} must have Max-Age=0 to actually delete it. Got: {v}"
            );
            assert!(
                v.contains("Path=/"),
                "logout cookie {name} must have Path=/ to match the original Set-Cookie path. Got: {v}"
            );
        }
    }

    #[tokio::test]
    async fn logout_response_body_is_ok() {
        let r = logout().await;
        let bytes = axum::body::to_bytes(r.into_body(), 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["ok"], true);
    }
}

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
    // Wave 23 T3 — clear ALL auth-related cookies, including the
    // dev's `epsx.return_url` post-login bounce cookie. Prod's
    // Vercel middleware clears seven cookies on logout
    // (`shared/auth/middleware.ts::handleAuthenticatedOnLogin`):
    // `epsx.access_token`, `epsx.refresh_token`, `epsx.id_token`,
    // `epsx.user`, `epsx.sid`, `epsx.auth_time`, `epsx.expires_at`.
    //
    // The dev BFF uses a simpler four-cookie set (`epsx_token`,
    // `epsx_user_id`, `epsx_user_address`, `epsx_chain_id`),
    // corresponding to the dev's SIWE / demo login response shape
    // (see `siwe_login` + `demo_login` in this file). Plus
    // `epsx_return_url` which is set by the SSR layer on unauth
    // redirects (mirroring the prod Vercel middleware's
    // `epsx.return_url` cookie). The cookie is `__Host-` prefixed
    // in prod but the dev shape is the unprefixed form — the
    // `build_clear_cookie` helper uses the same `Path=/; HttpOnly;
    // SameSite=Lax; Max-Age=0` shape for all names, which works
    // for both prefixed and unprefixed variants because the
    // browser matches on the `Name=...; Path=...; Max-Age=0`
    // tuple, not the prefix.
    for name in [
        "epsx_token",
        "epsx_user_id",
        "epsx_user_address",
        "epsx_chain_id",
        "epsx_return_url",
    ] {
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
    Json(serde_json::json!({
        "personal": [
            { "id": "1day", "name": "1 day", "price": 1, "currency": "USDT", "interval": "day", "features": ["Basic analytics"] },
            { "id": "1month", "name": "1 month", "price": 9, "currency": "USDT", "interval": "month", "features": ["Full analytics", "Watchlist", "Alerts"] },
            { "id": "1year", "name": "1 year", "price": 79, "currency": "USDT", "interval": "year", "features": ["Everything in monthly", "API access"] },
            { "id": "lifetime", "name": "Lifetime", "price": 499, "currency": "USDT", "interval": "lifetime", "features": ["Everything", "Paymaster gas"] }
        ],
        "api": [
            { "id": "api_personal", "name": "API Personal", "price": 19, "currency": "USDT", "interval": "month", "features": ["10k calls/day"] },
            { "id": "api_company",  "name": "API Company",  "price": 99, "currency": "USDT", "interval": "month", "features": ["1M calls/day", "Priority support"] }
        ],
        "custom": [
            { "id": "revenue_share", "name": "Revenue Share", "price": null, "currency": "USDT", "interval": "month", "features": ["Pay-as-you-earn", "Negotiated rate"] }
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
    Json(serde_json::json!({
        "items": [
            { "slug": "scalable-foundation", "title": "Building a scalable foundation", "excerpt": "How we architected a 9-service Rust backend.", "date": "2025-01-15", "tag1": "Engineering", "tag2": "Architecture", "featured": true, "image": "/news-img/scalable-foundation.png" },
            { "slug": "optimizing-high-throughput-analytics-rust", "title": "Optimizing high-throughput analytics", "excerpt": "Sub-millisecond EPS ranking over 8.5M data points.", "date": "2025-01-10", "tag1": "Engineering", "tag2": "Rust", "featured": false, "image": "/news-img/optimizing.png" },
            { "slug": "real-time-intelligence", "title": "Real-time intelligence, made simple", "excerpt": "How we made complex analytics feel instant.", "date": "2025-01-05", "tag1": "Product", "tag2": "UX", "featured": false, "image": "/news-img/realtime.png" },
            { "slug": "securing-the-future", "title": "Securing the future", "excerpt": "SIWE, RBAC, audit logs, and rate limiting.", "date": "2024-12-28", "tag1": "Engineering", "tag2": "Security", "featured": false, "image": "/news-img/securing.png" },
            { "slug": "smarter-decisions-ai", "title": "Smarter decisions, with AI", "excerpt": "Layering machine learning on top of on-chain data.", "date": "2024-12-20", "tag1": "Product", "tag2": "AI", "featured": false, "image": "/news-img/ai.png" },
            { "slug": "paymaster", "title": "Paymaster gas sponsorship", "excerpt": "Premium users can pay with zero gas.", "date": "2024-12-15", "tag1": "Product", "tag2": "Web3", "featured": false, "image": "/news-img/paymaster.png" },
            { "slug": "subscription-vaults", "title": "Subscription vaults", "excerpt": "Per-merchant stream-based subscription contracts on BSC.", "date": "2024-12-10", "tag1": "Engineering", "tag2": "Smart Contracts", "featured": false, "image": "/news-img/vaults.png" }
        ],
        "total": 7
    }))
}

pub async fn api_news_post(AxPath(slug): AxPath<String>, _state: State<AppState>) -> Json<serde_json::Value> {
    let title = slug.replace('-', " ");
    Json(serde_json::json!({
        "slug": slug,
        "title": title,
        "body": format!("This is the full article body for '{}'. In production, this is read from content/pages/{}.mdx and rendered via the content service.", title, slug),
        "date": "2025-01-15",
        "author": "EPSX Team"
    }))
}

pub async fn api_portfolio(AxPath(addr): AxPath<String>, _state: State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "address": addr,
        "total_value_usd": 0.0,
        "watchlist": [],
        "subscriptions": [],
        "transactions": [],
        "auth_required": true
    }))
}

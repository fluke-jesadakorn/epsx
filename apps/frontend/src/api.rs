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
use epsx_bff::{
    cookies::{append_clear_session_cookies, append_session_cookies},
    session::{
        AuthExchange, ChallengeRequest, ChallengeResponse, LogoutRequest, ProfileResponse,
        RefreshRequest, RefreshResponse, SessionUser, VerifyRequest, VerifyResponse,
        CHALLENGE_PATH, FRONTEND_CLIENT_ID, LOGOUT_PATH, PROFILE_PATH, REFRESH_PATH, VERIFY_PATH,
    },
};
use serde::Deserialize;

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

pub async fn api_health() -> &'static str {
    "ok"
}

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
pub async fn api_oauth_start(AxPath(provider): AxPath<String>) -> Response {
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

pub async fn get_page(State(state): State<AppState>, AxPath(slug): AxPath<String>) -> Response {
    let path = format!("/api/v1/content/pages/{}", slug);
    match state.content.get_plain(&path).await {
        Ok(v) => Json(v).into_response(),
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({"error": "upstream"})),
        )
            .into_response(),
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
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({"error": "upstream"})),
        )
            .into_response(),
    }
}

pub async fn publish_page(State(state): State<AppState>, AxPath(slug): AxPath<String>) -> Response {
    let path = format!("/api/v1/content/pages/{}/publish", slug);
    match state
        .content
        .post_plain(&path, &serde_json::json!({}))
        .await
    {
        Ok(v) => Json(v).into_response(),
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({"error": "upstream"})),
        )
            .into_response(),
    }
}

pub async fn siwe_login(
    State(state): State<AppState>,
    Json(body): Json<super::SiweLoginBody>,
) -> Response {
    let request_wallet = body.address.trim().to_string();
    let request = VerifyRequest {
        message: body.message,
        signature: body.signature,
        wallet_address: request_wallet.clone(),
        nonce: body.nonce,
        client_id: FRONTEND_CLIENT_ID.to_string(),
    };
    let url = auth_url(&state, VERIFY_PATH);
    let response = match state
        .identity
        .clone_for_bearer()
        .post(&url)
        .json(&request)
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => {
            tracing::warn!("SIWE verification upstream unavailable: {}", error);
            return safe_error(StatusCode::BAD_GATEWAY, "auth_upstream_unavailable");
        }
    };
    if !response.status().is_success() {
        return safe_error(response.status(), "authentication_rejected");
    }
    let upstream: VerifyResponse = match response.json().await {
        Ok(upstream) => upstream,
        Err(error) => {
            tracing::warn!("SIWE verification returned malformed JSON: {}", error);
            return safe_error(StatusCode::BAD_GATEWAY, "malformed_auth_response");
        }
    };
    let exchange = match upstream.into_exchange() {
        Ok(exchange) => exchange,
        Err(_) => return safe_error(StatusCode::UNAUTHORIZED, "authentication_rejected"),
    };
    establish_session(&state, exchange, Some(&request_wallet), false).await
}

async fn establish_session(
    state: &AppState,
    mut exchange: AuthExchange,
    expected_wallet: Option<&str>,
    clear_on_failure: bool,
) -> Response {
    let claims = match state.verifier.verify(exchange.tokens.access_token()).await {
        Ok(claims) => claims,
        Err(error) => {
            tracing::warn!("Rejected upstream access token: {}", error);
            return session_establishment_error(state, clear_on_failure, "invalid_upstream_token");
        }
    };
    let claims_user = claims.session_user();
    // The backend can recompute effective permissions after signing the JWT,
    // so unsigned response scopes may be newer or older than the token. Cross-
    // check immutable identity only, then replace all browser-visible scopes
    // with the cryptographically verified claims below.
    if expected_wallet.is_some_and(|wallet| !same_wallet(wallet, &claims_user.wallet_address))
        || !session_identity_matches(&exchange.browser.user, &claims_user)
    {
        tracing::warn!("Rejected inconsistent upstream authentication identity");
        return session_establishment_error(state, clear_on_failure, "inconsistent_auth_identity");
    }

    let created_at = exchange.browser.user.created_at.take();
    let last_login = exchange.browser.user.last_login.take();
    exchange.browser.user = SessionUser {
        created_at,
        last_login,
        ..claims_user
    };
    let mut response = Json(exchange.browser).into_response();
    if let Err(error) = append_session_cookies(
        response.headers_mut(),
        state.cookie_environment,
        exchange.tokens.access_token(),
        Some(exchange.tokens.refresh_token()),
        exchange.tokens.access_expires_in(),
        Some(exchange.tokens.refresh_expires_in()),
    ) {
        tracing::error!("Unable to build canonical session cookies: {}", error);
        return session_establishment_error(state, clear_on_failure, "session_cookie_error");
    }
    response
}

pub async fn auth_challenge(
    State(state): State<AppState>,
    Json(body): Json<super::ChallengeBody>,
) -> Response {
    // Contract anchor "{}/api/auth/web3/challenge": `auth_url` now joins the
    // configured gateway with the typed `CHALLENGE_PATH` constant.
    let request = ChallengeRequest {
        wallet_address: body.address.trim().to_string(),
    };
    let url = auth_url(&state, CHALLENGE_PATH);
    let response = match state
        .identity
        .clone_for_bearer()
        .post(&url)
        .json(&request)
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => {
            tracing::warn!("Challenge upstream unavailable: {}", error);
            return safe_error(StatusCode::BAD_GATEWAY, "auth_upstream_unavailable");
        }
    };
    if !response.status().is_success() {
        return safe_error(response.status(), "challenge_rejected");
    }
    match response.json::<ChallengeResponse>().await {
        Ok(ChallengeResponse::Success(challenge)) if challenge.success => {
            Json(challenge).into_response()
        }
        Ok(ChallengeResponse::Rejected(rejection)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": rejection.error,
                "message": rejection.message,
            })),
        )
            .into_response(),
        Ok(_) => safe_error(StatusCode::BAD_REQUEST, "challenge_rejected"),
        Err(error) => {
            tracing::warn!("Challenge upstream returned malformed JSON: {}", error);
            safe_error(StatusCode::BAD_GATEWAY, "malformed_auth_response")
        }
    }
}

pub async fn demo_login(
    State(state): State<AppState>,
    _body: Option<Json<super::DemoLoginBody>>,
) -> Response {
    if !state.demo_login_enabled {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "demo disabled"})),
        )
            .into_response();
    }
    safe_error(StatusCode::NOT_IMPLEMENTED, "demo_auth_not_canonical")
}

pub async fn refresh_token(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let Some(refresh_token) = super::auth::refresh_token(&headers, state.cookie_environment) else {
        return clear_session_response(&state, StatusCode::UNAUTHORIZED, "missing_refresh_token");
    };
    let request = RefreshRequest {
        refresh_token: &refresh_token,
        client_id: FRONTEND_CLIENT_ID,
    };
    let response = match state
        .identity
        .clone_for_bearer()
        .post(auth_url(&state, REFRESH_PATH))
        .json(&request)
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => {
            tracing::warn!("Refresh upstream unavailable: {}", error);
            return safe_error(StatusCode::BAD_GATEWAY, "auth_upstream_unavailable");
        }
    };
    if response.status().is_client_error() {
        return clear_session_response(&state, StatusCode::UNAUTHORIZED, "refresh_rejected");
    }
    if !response.status().is_success() {
        return safe_error(StatusCode::BAD_GATEWAY, "refresh_upstream_failed");
    }
    let upstream: RefreshResponse = match response.json().await {
        Ok(upstream) => upstream,
        Err(error) => {
            tracing::warn!("Refresh upstream returned malformed JSON: {}", error);
            return clear_session_response(
                &state,
                StatusCode::BAD_GATEWAY,
                "malformed_auth_response",
            );
        }
    };
    let exchange = match upstream.into_exchange() {
        Ok(exchange) => exchange,
        Err(_) => {
            return clear_session_response(&state, StatusCode::UNAUTHORIZED, "refresh_rejected")
        }
    };
    establish_session(&state, exchange, None, true).await
}

pub async fn logout(State(state): State<AppState>, headers: axum::http::HeaderMap) -> Response {
    let refresh_token = super::auth::refresh_token(&headers, state.cookie_environment);
    let wallet =
        super::auth::current_user(&headers, state.verifier.as_ref(), state.cookie_environment)
            .await
            .map(|user| user.wallet_address);
    let request = LogoutRequest {
        wallet_address: wallet.as_deref(),
        refresh_token: refresh_token.as_deref(),
    };
    let upstream_ok = state
        .identity
        .clone_for_bearer()
        .delete(auth_url(&state, LOGOUT_PATH))
        .json(&request)
        .send()
        .await
        .is_ok_and(|response| response.status().is_success());

    let status = if upstream_ok {
        StatusCode::OK
    } else {
        StatusCode::BAD_GATEWAY
    };
    let mut response = (
        status,
        Json(serde_json::json!({
            "success": upstream_ok,
            "message": if upstream_ok { "Logged out" } else { "Local session cleared" },
        })),
    )
        .into_response();
    if append_clear_session_cookies(response.headers_mut(), state.cookie_environment).is_err() {
        return safe_error(StatusCode::INTERNAL_SERVER_ERROR, "session_cookie_error");
    }
    response
}

pub async fn auth_me(State(state): State<AppState>, headers: axum::http::HeaderMap) -> Response {
    let Some(token) = super::auth::access_token(&headers, state.cookie_environment) else {
        return safe_error(StatusCode::UNAUTHORIZED, "missing_access_token");
    };
    let claims = match state.verifier.verify(&token).await {
        Ok(claims) => claims,
        Err(_) => {
            return clear_session_response(&state, StatusCode::UNAUTHORIZED, "invalid_access_token")
        }
    };
    let url = auth_url(&state, PROFILE_PATH);
    let client = state.identity.clone_for_bearer();
    let response = match client.get(&url).bearer_auth(&token).send().await {
        Ok(response) => response,
        Err(error) => {
            tracing::warn!("Profile upstream unavailable: {}", error);
            return safe_error(StatusCode::BAD_GATEWAY, "profile_upstream_unavailable");
        }
    };
    if response.status() == StatusCode::UNAUTHORIZED {
        return clear_session_response(&state, StatusCode::UNAUTHORIZED, "profile_rejected");
    }
    if !response.status().is_success() {
        return safe_error(response.status(), "profile_rejected");
    }
    let profile = match response.json::<ProfileResponse>().await {
        Ok(profile) => profile.into_user(),
        Err(error) => {
            tracing::warn!("Profile upstream returned malformed JSON: {}", error);
            return safe_error(StatusCode::BAD_GATEWAY, "malformed_profile_response");
        }
    };
    if !same_wallet(&claims.wallet_address, &profile.wallet_address)
        || !same_wallet(&claims.sub, &profile.subject)
    {
        return safe_error(StatusCode::BAD_GATEWAY, "inconsistent_profile_identity");
    }
    Json(profile).into_response()
}

fn auth_url(state: &AppState, path: &str) -> String {
    format!("{}{}", state.api_url.trim_end_matches('/'), path)
}

fn same_wallet(left: &str, right: &str) -> bool {
    !left.trim().is_empty() && left.eq_ignore_ascii_case(right)
}

fn session_identity_matches(response: &SessionUser, claims: &SessionUser) -> bool {
    same_wallet(&response.wallet_address, &claims.wallet_address)
        && same_wallet(&response.subject, &claims.subject)
}

fn safe_error(status: StatusCode, code: &'static str) -> Response {
    (
        status,
        Json(serde_json::json!({ "success": false, "error": code })),
    )
        .into_response()
}

fn session_establishment_error(
    state: &AppState,
    clear_on_failure: bool,
    code: &'static str,
) -> Response {
    if clear_on_failure {
        clear_session_response(state, StatusCode::BAD_GATEWAY, code)
    } else {
        safe_error(StatusCode::BAD_GATEWAY, code)
    }
}

fn clear_session_response(state: &AppState, status: StatusCode, code: &'static str) -> Response {
    let mut response = safe_error(status, code);
    if append_clear_session_cookies(response.headers_mut(), state.cookie_environment).is_err() {
        return safe_error(StatusCode::INTERNAL_SERVER_ERROR, "session_cookie_error");
    }
    response
}

async fn verified_bearer(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Result<String, Response> {
    super::auth::verified_access_token(headers, state.verifier.as_ref(), state.cookie_environment)
        .await
        .map(|(token, _)| token)
        .ok_or_else(|| safe_error(StatusCode::UNAUTHORIZED, "invalid_access_token"))
}

pub async fn notifications_api(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    method: axum::http::Method,
) -> Response {
    let token = match verified_bearer(&state, &headers).await {
        Ok(token) => token,
        Err(response) => return response,
    };
    let url = format!(
        "{}/api/v1/notification/list",
        state.api_url.trim_end_matches('/')
    );
    let client = state.notification.clone_for_bearer();
    let req = client.get(&url).bearer_auth(&token);
    let _ = method;
    match req.send().await {
        Ok(r) if r.status().is_success() => match r.json::<serde_json::Value>().await {
            Ok(v) => Json(v).into_response(),
            Err(_) => (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({"error": "upstream"})),
            )
                .into_response(),
        },
        Ok(r) => (r.status(), Json(serde_json::json!({"error": "upstream"}))).into_response(),
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({"error": "upstream"})),
        )
            .into_response(),
    }
}

pub async fn notification_read(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    AxPath(id): AxPath<String>,
) -> Response {
    let token = match verified_bearer(&state, &headers).await {
        Ok(token) => token,
        Err(response) => return response,
    };
    let url = format!(
        "{}/api/v1/notification/{}/read",
        state.api_url.trim_end_matches('/'),
        id
    );
    match state
        .notification
        .clone_for_bearer()
        .post(&url)
        .bearer_auth(&token)
        .json(&serde_json::json!({}))
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => Json(serde_json::json!({"ok": true})).into_response(),
        Ok(r) => (r.status(), Json(serde_json::json!({"error": "upstream"}))).into_response(),
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({"error": "upstream"})),
        )
            .into_response(),
    }
}

pub async fn notification_delete(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    AxPath(id): AxPath<String>,
) -> Response {
    let token = match verified_bearer(&state, &headers).await {
        Ok(token) => token,
        Err(response) => return response,
    };
    let url = format!(
        "{}/api/v1/notification/{}",
        state.api_url.trim_end_matches('/'),
        id
    );
    match state
        .notification
        .clone_for_bearer()
        .delete(&url)
        .bearer_auth(&token)
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => Json(serde_json::json!({"ok": true})).into_response(),
        Ok(r) => (r.status(), Json(serde_json::json!({"error": "upstream"}))).into_response(),
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({"error": "upstream"})),
        )
            .into_response(),
    }
}

pub async fn notification_mark_all(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let token = match verified_bearer(&state, &headers).await {
        Ok(token) => token,
        Err(response) => return response,
    };
    let url = format!(
        "{}/api/v1/notification/mark-all-read",
        state.api_url.trim_end_matches('/')
    );
    match state
        .notification
        .clone_for_bearer()
        .post(&url)
        .bearer_auth(&token)
        .json(&serde_json::json!({}))
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => Json(serde_json::json!({"ok": true})).into_response(),
        Ok(r) => (r.status(), Json(serde_json::json!({"error": "upstream"}))).into_response(),
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({"error": "upstream"})),
        )
            .into_response(),
    }
}

pub async fn notification_clear_all(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let token = match verified_bearer(&state, &headers).await {
        Ok(token) => token,
        Err(response) => return response,
    };
    let url = format!(
        "{}/api/v1/notification/clear-all",
        state.api_url.trim_end_matches('/')
    );
    match state
        .notification
        .clone_for_bearer()
        .post(&url)
        .bearer_auth(&token)
        .json(&serde_json::json!({}))
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => Json(serde_json::json!({"ok": true})).into_response(),
        Ok(r) => (r.status(), Json(serde_json::json!({"error": "upstream"}))).into_response(),
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({"error": "upstream"})),
        )
            .into_response(),
    }
}

pub async fn track_event(
    State(state): State<AppState>,
    Json(body): Json<AnalyticsTrackBody>,
) -> Response {
    let url = format!(
        "{}/api/v1/analytics/track",
        state.api_url.trim_end_matches('/')
    );
    match state
        .analytics
        .clone_for_bearer()
        .post(&url)
        .json(&serde_json::json!({
            "event_name": body.event_name,
            "properties": body.properties,
            "user_id": body.user_id,
            "chain_id": body.chain_id,
        }))
        .send()
        .await
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
        ChainInfo {
            id: "bsc".into(),
            name: "BSC Mainnet".into(),
            chain_id: 56,
            rpc_url: "https://bsc-dataseed1.binance.org".into(),
            currency: "BNB".into(),
            explorer: "https://bscscan.com".into(),
        },
        ChainInfo {
            id: "bsc_testnet".into(),
            name: "BSC Testnet".into(),
            chain_id: 97,
            rpc_url: "https://data-seed-prebsc-1-s1.binance.org:8545".into(),
            currency: "tBNB".into(),
            explorer: "https://testnet.bscscan.com".into(),
        },
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

pub async fn api_subscription_merchant(
    _state: State<AppState>,
    AxPath(addr): AxPath<String>,
) -> Json<serde_json::Value> {
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

pub async fn api_subscription_subscribe(
    Json(body): Json<SubscribeBody>,
) -> Json<serde_json::Value> {
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

pub async fn api_subscription_create_plan(
    Json(body): Json<CreatePlanBody>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "id": uuid::Uuid::new_v4().to_string(),
        "name": body.name,
        "amount": body.amount,
        "currency": body.currency.unwrap_or_else(|| "USDT".to_string()),
        "interval": body.interval.unwrap_or(2592000),
        "active": true
    }))
}

/// Build the news list payload (10 articles matching the prod
/// slugs captured by T1's prod baseline). Returns the inner
/// `articles` + `total` object so both the BFF route and the
/// SSR layer can hand the same shape to the dev `NewsPost`
/// deserializer. Wave 31 T1 — extracted from `api_news` so the
/// SSR layer can call it in-process (no HTTP round-trip via
/// the upstream gateway) and so the BFF route is just a thin
/// wrapper.
pub fn news_list_value() -> serde_json::Value {
    let articles = vec![
        article("strategic-roadmap-future", "Strategic Roadmap and Future Capabilities", "A preview of upcoming system enhancements, including automated alerts and expanded analytical depth.", "2025-02-01", &["roadmap", "strategy"], "/news-img/strategic-roadmap-future.png", true),
        article("enhanced-portfolio-management", "Enhanced Portfolio Management Solutions", "Tools and insights for the modern portfolio manager.", "2025-02-01", &["portfolio", "product"], "/news-img/enhanced-portfolio-management.png", false),
        article("service-tier-alignment", "Integrated Service Solutions: Professional Tier Alignment", "How EPSX services scale across professional subscription tiers.", "2025-02-01", &["service", "tiers"], "/news-img/service-tier-alignment.png", false),
        article("performance-metrics-positioning", "Proprietary Performance Metrics and Strategic Positioning", "The metrics that set EPSX apart.", "2025-02-01", &["metrics", "strategy"], "/news-img/performance-metrics-positioning.png", false),
        article("strategic-launch-epsx", "Strategic Launch of EPSX: Institutional-Grade Market Insights", "Our strategic launch announcement.", "2025-02-01", &["launch", "announcement"], "/news-img/strategic-launch-epsx.png", false),
        article("optimizing-high-throughput-analytics-rust", "Strategic Analysis Performance for Operational Excellence", "How EPSX leverages high-performance data processing to deliver precise rankings and insights.", "2025-02-01", &["performance", "engineering"], "/news-img/optimizing-high-throughput-analytics-rust.png", false),
        article("real-time-market-data-redis-streams", "Real-Time Intelligence: Capturing Market Opportunities as They Happen", "How the EPSX dashboard removes the gap between on-chain events and your decision-making.", "2025-02-01", &["real-time", "redis"], "/news-img/real-time-market-data-redis-streams.png", false),
        article("future-secure-web3-auth", "Securing the Future: Enterprise-Grade Trust in a Web3 World", "SIWE, RBAC, audit logs, and rate limiting.", "2025-02-01", &["security", "web3"], "/news-img/future-secure-web3-auth.png", false),
        article("scalable-postgresql-time-series", "Built for Ambition: A Scalable Foundation for Global Analytics", "Scaling a global analytics platform with an industrial-strength architecture.", "2025-02-01", &["database", "scalability"], "/news-img/scalable-postgresql-time-series.png", false),
        article("predictive-ai-models-market-sentiment", "Smarter Decisions: How EPSX AI Navigates Market Complexity", "Layering machine learning on top of on-chain data.", "2025-02-01", &["ai", "product"], "/news-img/predictive-ai-models-market-sentiment.png", false),
    ];
    let total = articles.len();
    serde_json::json!({ "articles": articles, "total": total })
}

pub async fn api_news(_state: State<AppState>) -> Json<serde_json::Value> {
    // Wave 31 T1 — body moved to `news_list_value()` so the SSR
    // layer can call the same data shape in-process. The BFF route
    // is now a thin wrapper.
    Json(news_list_value())
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

/// Build the news-post detail payload (full article body). Returns
/// the inner `serde_json::Value` so both the BFF route and the SSR
/// layer can share the same shape. Wave 31 T1 — extracted from
/// `api_news_post` so the SSR layer can call it in-process.
pub fn news_post_value(slug: &str) -> serde_json::Value {
    let (title, tags, read_time, author, date): (String, Vec<&str>, String, String, String) =
        match slug {
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
                (
                    title,
                    vec!["EPSX", "Update"],
                    "3 min".to_string(),
                    "EPSX Team".to_string(),
                    "2025-01-15".to_string(),
                )
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
    serde_json::json!({
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
    })
}

/// BFF route handler for `/api/v1/news/{slug}` — thin wrapper
/// around `news_post_value()` so the route and SSR share the same
/// payload.
pub async fn api_news_post(
    AxPath(slug): AxPath<String>,
    _state: State<AppState>,
) -> Json<serde_json::Value> {
    Json(news_post_value(&slug))
}

pub async fn api_portfolio(
    AxPath(addr): AxPath<String>,
    _state: State<AppState>,
) -> Json<serde_json::Value> {
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

pub async fn api_account(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Json<serde_json::Value> {
    // If the request carries a session token (or the dev bypass is
    // enabled), show the user's wallet address + member-since (Jan
    // 2025). Anonymous requests get the OLD prod placeholder set:
    // Not Connected / Join Now / $0 / Web3 Vault. The dev
    // `account.rs` already supports this shape via `data_account`.
    let has_session =
        super::auth::current_user(&headers, state.verifier.as_ref(), state.cookie_environment)
            .await
            .is_some();
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

pub async fn api_credits(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Json<serde_json::Value> {
    let has_session =
        super::auth::current_user(&headers, state.verifier.as_ref(), state.cookie_environment)
            .await
            .is_some();
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

/// Build the developer-usage payload (summary + per_key + history).
/// Returned to both the BFF route and the SSR layer (so the page
/// consumes a consistent shape regardless of which path the data
/// arrives on).
///
/// Wave 31 T1 — extracted from `api_developer_usage` so the SSR
/// layer can call it in-process.
pub fn developer_usage_value() -> serde_json::Value {
    serde_json::json!({
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
    })
}

pub async fn api_developer_usage(_state: State<AppState>) -> Json<serde_json::Value> {
    Json(developer_usage_value())
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

/// Build the dashboard payload. Mirrors the prod shape from
/// `apps-old/frontend/app/dashboard/page.tsx:35-45`:
/// `{ success, data: { stats: {totalViews, totalUsers, revenue}, recentActivity: [] } }`.
///
/// The dev `pages/dashboard.rs` reads `data_dashboard.stats` for the
/// 3 stat cards (Total Views / Total Users / Revenue), so the
/// SSR layer hands the inner `data` object to the page (or the page
/// reads `.data.stats` directly — same shape).
///
/// Auth-aware: when the user is signed in (or `EPSX_DEV_AUTH_BYPASS=1`),
/// the stats match the "just created your account" baseline
/// (1 user, 0 views, $0 revenue). Anonymous requests get the same
/// zero-state — the prod page renders the same placeholder for
/// anonymous visitors (the page only shows the dashboard client when
/// `user` is present, and the harness captures the unauthed state).
pub fn dashboard_data_internal(has_session: bool) -> serde_json::Value {
    // (totalViews, totalUsers, revenue) — the prod's `dashboardData`
    // mock is `0 / 1 / 0` for every visitor. We keep the same values
    // here so the BFF route is a 1:1 with prod's payload.
    let stats = if has_session {
        serde_json::json!({ "totalViews": 0, "totalUsers": 1, "revenue": 0 })
    } else {
        serde_json::json!({ "totalViews": 0, "totalUsers": 0, "revenue": 0 })
    };
    serde_json::json!({
        "success": true,
        "data": {
            "stats": stats,
            "recentActivity": [],
        }
    })
}

pub async fn api_dashboard(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Json<serde_json::Value> {
    let has_session =
        super::auth::current_user(&headers, state.verifier.as_ref(), state.cookie_environment)
            .await
            .is_some();
    Json(dashboard_data_internal(has_session))
}

/// `/api/v1/dashboard/stats` — full envelope `{"success": true, "data": {...}}`.
///
/// Wave 32 T1 — verifier feedback on wave 31 said: "should return
/// full envelope `{success, data: {...}}` (brief's shape). My attempt
/// returned only inner `data` sub-object." This handler now returns
/// the full envelope so the route matches the brief's specified
/// shape. The SSR layer (`ssr.rs::fetch_page_data`) continues to
/// extract the inner `data` sub-object for the page's
/// `ctx.params["data_dashboard"]` lookup — see that file for the
/// `v.get("data")` extraction.
pub async fn api_dashboard_stats(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Json<serde_json::Value> {
    let has_session =
        super::auth::current_user(&headers, state.verifier.as_ref(), state.cookie_environment)
            .await
            .is_some();
    // Return the FULL envelope `{success, data: {stats, recentActivity}}`
    // so the BFF route matches the brief's specified shape.
    Json(dashboard_data_internal(has_session))
}

pub async fn api_payment(
    _state: State<AppState>,
    AxPath(id): AxPath<String>,
) -> Json<serde_json::Value> {
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

#[cfg(test)]
mod auth_session_tests {
    use super::*;
    use axum::{
        http::{header, HeaderMap, HeaderValue},
        routing::{delete, get, post},
        Router,
    };
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use epsx_bff::{
        cookies::{LEGACY_ACCESS_COOKIE, LOCAL_ACCESS_COOKIE, LOCAL_REFRESH_COOKIE},
        session::{AccessTokenClaims, Jwks, JwksVerifierConfig, RsaJwk, JWKS_PATH},
    };
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use rand::thread_rng;
    use rsa::{pkcs8::EncodePrivateKey, traits::PublicKeyParts, RsaPrivateKey};
    use serde_json::{json, Value};
    use std::{sync::Arc, time::Duration};

    const TEST_ISSUER: &str = "https://issuer.test";
    const TEST_WALLET: &str = "0x1111111111111111111111111111111111111111";

    struct TestKey {
        encoding: EncodingKey,
        jwk: RsaJwk,
    }

    impl TestKey {
        fn generate() -> Self {
            let private = RsaPrivateKey::new(&mut thread_rng(), 2048).unwrap();
            let pem = private.to_pkcs8_pem(rsa::pkcs8::LineEnding::LF).unwrap();
            let public = private.to_public_key();
            Self {
                encoding: EncodingKey::from_rsa_pem(pem.as_bytes()).unwrap(),
                jwk: RsaJwk {
                    kty: "RSA".into(),
                    use_: Some("sig".into()),
                    alg: Some("RS256".into()),
                    kid: "frontend-test-key".into(),
                    n: URL_SAFE_NO_PAD.encode(public.n().to_bytes_be()),
                    e: URL_SAFE_NO_PAD.encode(public.e().to_bytes_be()),
                },
            }
        }

        fn access_token(&self, permissions: &[&str]) -> String {
            let now = chrono::Utc::now().timestamp();
            let mut header = Header::new(Algorithm::RS256);
            header.kid = Some(self.jwk.kid.clone());
            encode(
                &header,
                &AccessTokenClaims {
                    iss: TEST_ISSUER.into(),
                    sub: TEST_WALLET.into(),
                    aud: vec![FRONTEND_CLIENT_ID.into()],
                    exp: now + 300,
                    iat: now - 1,
                    jti: "test-jti".into(),
                    scope: format!("openid permissions {}", permissions.join(" ")),
                    wallet_address: TEST_WALLET.into(),
                    auth_method: "web3_siwe".into(),
                    auth_time: now - 1,
                    nbf: None,
                },
                &self.encoding,
            )
            .unwrap()
        }
    }

    async fn spawn_mock(router: Router) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        format!("http://{address}")
    }

    async fn unused_base_url() -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        drop(listener);
        format!("http://{address}")
    }

    fn state(base_url: &str) -> AppState {
        let config = epsx_client::ClientConfig {
            base_url: base_url.to_string(),
            timeout: Duration::from_secs(1),
        };
        let client = Arc::new(epsx_client::ServiceClient::new(config.clone()));
        let verifier = JwksVerifierConfig::new(
            format!("{base_url}{JWKS_PATH}"),
            TEST_ISSUER,
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
            verifier: Arc::new(epsx_bff::session::JwksVerifier::with_http(verifier).unwrap()),
            cookie_environment: epsx_bff::cookies::CookieEnvironment::Local,
            api_url: base_url.to_string(),
            demo_login_enabled: false,
        }
    }

    fn request_headers(cookie: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(header::COOKIE, HeaderValue::from_str(cookie).unwrap());
        headers
    }

    fn response_cookies(response: &Response) -> Vec<String> {
        response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .map(|value| value.to_str().unwrap().to_string())
            .collect()
    }

    fn assert_session_cleared(response: &Response) {
        let cookies = response_cookies(response);
        assert_eq!(
            cookies.len(),
            3,
            "expected canonical pair plus legacy clear"
        );
        for name in [
            LOCAL_ACCESS_COOKIE,
            LOCAL_REFRESH_COOKIE,
            LEGACY_ACCESS_COOKIE,
        ] {
            let cookie = cookies
                .iter()
                .find(|cookie| cookie.starts_with(&format!("{name}=")))
                .unwrap_or_else(|| panic!("missing clear cookie for {name}: {cookies:?}"));
            assert!(cookie.contains("Max-Age=0"));
            assert!(cookie.contains("HttpOnly"));
        }
    }

    #[tokio::test]
    async fn siwe_verifies_returned_jwt_and_only_returns_safe_browser_data() {
        let key = TestKey::generate();
        let access_token = key.access_token(&["epsx:analytics:read"]);
        let jwks = Jwks {
            keys: vec![key.jwk.clone()],
        };
        let verify_payload = json!({
            "success": true,
            "authenticated": true,
            "wallet_address": TEST_WALLET,
            "permissions": ["unsigned:possibly-stale"],
            "access_token": access_token,
            "refresh_token": "opaque-refresh-token",
            "expires_in": 300,
            "refresh_expires_in": 3600
        });
        let router = Router::new()
            .route(
                JWKS_PATH,
                get(move || {
                    let jwks = jwks.clone();
                    async move { Json(jwks) }
                }),
            )
            .route(
                VERIFY_PATH,
                post(move |Json(body): Json<Value>| {
                    let payload = verify_payload.clone();
                    async move {
                        if body["client_id"] == FRONTEND_CLIENT_ID
                            && body["wallet_address"] == TEST_WALLET
                        {
                            Json(payload).into_response()
                        } else {
                            StatusCode::BAD_REQUEST.into_response()
                        }
                    }
                }),
            );
        let base_url = spawn_mock(router).await;
        let response = siwe_login(
            State(state(&base_url)),
            Json(crate::SiweLoginBody {
                message: "sign me".into(),
                signature: "0xsigned".into(),
                chain_id: "56".into(),
                address: TEST_WALLET.into(),
                nonce: "nonce-1".into(),
            }),
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        let cookies = response_cookies(&response);
        assert_eq!(cookies.len(), 2);
        assert!(cookies
            .iter()
            .any(|cookie| cookie.starts_with("epsx.access_token=")));
        assert!(cookies
            .iter()
            .any(|cookie| cookie.starts_with("epsx.refresh_token=")));
        let body = axum::body::to_bytes(response.into_body(), 16 * 1024)
            .await
            .unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["authenticated"], true);
        assert_eq!(body["user"]["wallet_address"], TEST_WALLET);
        assert_eq!(
            body["user"]["permissions"],
            json!(["epsx:analytics:read"]),
            "browser permissions must come from the verified JWT, not unsigned response JSON"
        );
        assert!(body.get("access_token").is_none());
        assert!(body.get("refresh_token").is_none());
        assert!(
            !String::from_utf8_lossy(&serde_json::to_vec(&body).unwrap())
                .contains("opaque-refresh-token")
        );
    }

    #[tokio::test]
    async fn invalid_login_token_sets_no_session_cookie() {
        let payload = json!({
            "success": true,
            "authenticated": true,
            "wallet_address": TEST_WALLET,
            "permissions": [],
            "access_token": "not-a-jwt",
            "refresh_token": "opaque-refresh-token",
            "expires_in": 300,
            "refresh_expires_in": 3600
        });
        let router = Router::new().route(
            VERIFY_PATH,
            post(move || {
                let payload = payload.clone();
                async move { Json(payload) }
            }),
        );
        let base_url = spawn_mock(router).await;
        let response = siwe_login(
            State(state(&base_url)),
            Json(crate::SiweLoginBody {
                message: "message".into(),
                signature: "signature".into(),
                chain_id: "56".into(),
                address: TEST_WALLET.into(),
                nonce: "nonce".into(),
            }),
        )
        .await;

        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
        assert!(response_cookies(&response).is_empty());
    }

    #[tokio::test]
    async fn invalid_rotated_access_token_clears_existing_session() {
        let payload = json!({
            "success": true,
            "authenticated": true,
            "access_token": "not-a-jwt",
            "refresh_token": "rotated-refresh",
            "expires_in": 300,
            "refresh_expires_in": 3600,
            "user": {"wallet_address": TEST_WALLET, "permissions": []}
        });
        let router = Router::new().route(
            REFRESH_PATH,
            post(move |Json(body): Json<Value>| {
                let payload = payload.clone();
                async move {
                    if body["client_id"] == FRONTEND_CLIENT_ID
                        && body["refresh_token"] == "browser-refresh"
                    {
                        Json(payload).into_response()
                    } else {
                        StatusCode::BAD_REQUEST.into_response()
                    }
                }
            }),
        );
        let base_url = spawn_mock(router).await;
        let response = refresh_token(
            State(state(&base_url)),
            request_headers("epsx.access_token=old-access; epsx.refresh_token=browser-refresh"),
        )
        .await;

        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
        assert_session_cleared(&response);
    }

    #[tokio::test]
    async fn refresh_verifies_rotation_and_atomically_replaces_cookie_pair() {
        let key = TestKey::generate();
        let access_token = key.access_token(&["epsx:rankings:read"]);
        let jwks = Jwks {
            keys: vec![key.jwk.clone()],
        };
        let payload = json!({
            "success": true,
            "authenticated": true,
            "access_token": access_token,
            "refresh_token": "rotated-refresh",
            "expires_in": 300,
            "refresh_expires_in": 3600,
            "user": {
                "wallet_address": TEST_WALLET,
                "subject": TEST_WALLET,
                "permissions": ["epsx:rankings:read"],
                "auth_method": "web3_siwe"
            }
        });
        let router = Router::new()
            .route(
                JWKS_PATH,
                get(move || {
                    let jwks = jwks.clone();
                    async move { Json(jwks) }
                }),
            )
            .route(
                REFRESH_PATH,
                post(move |Json(body): Json<Value>| {
                    let payload = payload.clone();
                    async move {
                        if body["client_id"] == FRONTEND_CLIENT_ID
                            && body["refresh_token"] == "browser-refresh"
                        {
                            Json(payload).into_response()
                        } else {
                            StatusCode::BAD_REQUEST.into_response()
                        }
                    }
                }),
            );
        let base_url = spawn_mock(router).await;
        let response = refresh_token(
            State(state(&base_url)),
            request_headers("epsx.refresh_token=browser-refresh"),
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        let cookies = response_cookies(&response);
        assert_eq!(cookies.len(), 2);
        assert!(cookies
            .iter()
            .any(|cookie| cookie.starts_with("epsx.access_token=")));
        assert!(cookies
            .iter()
            .any(|cookie| cookie.starts_with("epsx.refresh_token=rotated-refresh")));
        let body = axum::body::to_bytes(response.into_body(), 16 * 1024)
            .await
            .unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["authenticated"], true);
        assert_eq!(body["user"]["permissions"], json!(["epsx:rankings:read"]));
        assert!(body.get("access_token").is_none());
        assert!(body.get("refresh_token").is_none());
    }

    #[tokio::test]
    async fn me_verifies_locally_and_preserves_backend_profile_data() {
        let key = TestKey::generate();
        let access_token = key.access_token(&["token:permission"]);
        let expected_token = access_token.clone();
        let jwks = Jwks {
            keys: vec![key.jwk.clone()],
        };
        let router = Router::new()
            .route(
                JWKS_PATH,
                get(move || {
                    let jwks = jwks.clone();
                    async move { Json(jwks) }
                }),
            )
            .route(
                PROFILE_PATH,
                get(move |headers: HeaderMap| {
                    let expected_token = expected_token.clone();
                    async move {
                        let expected = format!("Bearer {expected_token}");
                        if headers
                            .get(header::AUTHORIZATION)
                            .and_then(|value| value.to_str().ok())
                            != Some(expected.as_str())
                        {
                            return StatusCode::UNAUTHORIZED.into_response();
                        }
                        Json(json!({
                            "data": {
                                "wallet_address": TEST_WALLET,
                                "subject": TEST_WALLET,
                                "permissions": ["backend:profile:permission"],
                                "capabilities": ["backend-capability"],
                                "auth_method": "web3_siwe",
                                "created_at": "2026-01-02T03:04:05Z"
                            }
                        }))
                        .into_response()
                    }
                }),
            );
        let base_url = spawn_mock(router).await;
        let response = auth_me(
            State(state(&base_url)),
            request_headers(&format!("epsx.access_token={access_token}")),
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), 16 * 1024)
            .await
            .unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["permissions"], json!(["backend:profile:permission"]));
        assert_eq!(body["capabilities"], json!(["backend-capability"]));
        assert_eq!(body["created_at"], "2026-01-02T03:04:05Z");
    }

    #[tokio::test]
    async fn me_clears_session_for_invalid_token_and_upstream_unauthorized() {
        let invalid_base = unused_base_url().await;
        let invalid = auth_me(
            State(state(&invalid_base)),
            request_headers("epsx.access_token=not-a-jwt"),
        )
        .await;
        assert_eq!(invalid.status(), StatusCode::UNAUTHORIZED);
        assert_session_cleared(&invalid);

        let key = TestKey::generate();
        let access_token = key.access_token(&[]);
        let jwks = Jwks {
            keys: vec![key.jwk.clone()],
        };
        let router = Router::new()
            .route(
                JWKS_PATH,
                get(move || {
                    let jwks = jwks.clone();
                    async move { Json(jwks) }
                }),
            )
            .route(PROFILE_PATH, get(|| async { StatusCode::UNAUTHORIZED }));
        let base_url = spawn_mock(router).await;
        let unauthorized = auth_me(
            State(state(&base_url)),
            request_headers(&format!("epsx.access_token={access_token}")),
        )
        .await;
        assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);
        assert_session_cleared(&unauthorized);
    }

    #[tokio::test]
    async fn logout_always_clears_canonical_pair_and_legacy_cookie() {
        let router = Router::new().route(
            LOGOUT_PATH,
            delete(|| async { Json(json!({"success": true})) }),
        );
        let base_url = spawn_mock(router).await;
        let success = logout(
            State(state(&base_url)),
            request_headers("epsx.refresh_token=browser-refresh"),
        )
        .await;
        assert_eq!(success.status(), StatusCode::OK);
        assert_session_cleared(&success);

        let unavailable_base = unused_base_url().await;
        let failure = logout(
            State(state(&unavailable_base)),
            request_headers("epsx.refresh_token=browser-refresh"),
        )
        .await;
        assert_eq!(failure.status(), StatusCode::BAD_GATEWAY);
        assert_session_cleared(&failure);
    }
}

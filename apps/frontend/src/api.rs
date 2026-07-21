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
    extract::{Path as AxPath, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, NaiveDate};
use epsx_bff::{
    cookies::{append_clear_session_cookies, append_session_cookies},
    session::{
        AuthExchange, ChallengeRequest, ChallengeResponse, LogoutRequest, ProfileResponse,
        RefreshRequest, RefreshResponse, SessionUser, VerifyRequest, VerifyResponse,
        CHALLENGE_PATH, FRONTEND_CLIENT_ID, LOGOUT_PATH, PROFILE_PATH, REFRESH_PATH, VERIFY_PATH,
    },
};
use epsx_client::{ClientError, ServiceClient};
use serde::{Deserialize, Serialize};

use super::AppState;

#[derive(Deserialize)]
pub struct AnalyticsTrackBody {
    pub event_name: String,
    pub properties: Option<serde_json::Value>,
    pub user_id: Option<String>,
    pub chain_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Default, PartialEq, Eq)]
pub struct NewsQuery {
    pub page: Option<u32>,
    pub q: Option<String>,
    pub category: Option<String>,
    // Accepted only so both Axum and the raw SSR parser can reject the old
    // public knob explicitly. News parity uses the pinned fixed page size.
    limit: Option<u32>,
}

const NEWS_PAGE_SIZE: u32 = 12;
const NEWS_UPSTREAM_FETCH_LIMIT: u32 = 100;
const NEWS_LIST_PATH: &str = "/api/v1/content/news?page=1&limit=100";
const NEWS_CATEGORIES: [&str; 4] = ["all", "updates", "engineering", "product"];

#[derive(Clone, Debug, PartialEq, Eq)]
struct NormalizedNewsQuery {
    page: u32,
    limit: u32,
    q: String,
    category: String,
}

impl NewsQuery {
    pub(crate) fn from_raw_query(raw: &str) -> Result<Self, ()> {
        let url =
            reqwest::Url::parse(&format!("https://frontend.invalid/?{raw}")).map_err(|_| ())?;
        let mut query = Self::default();
        let mut seen = std::collections::HashSet::new();
        for (key, value) in url.query_pairs() {
            let key = key.as_ref();
            if !matches!(key, "page" | "limit" | "q" | "category") {
                continue;
            }
            if !seen.insert(key.to_string()) {
                return Err(());
            }
            match key {
                "page" => query.page = Some(value.parse().map_err(|_| ())?),
                "limit" => query.limit = Some(value.parse().map_err(|_| ())?),
                "q" => query.q = Some(value.into_owned()),
                "category" => query.category = Some(value.into_owned()),
                _ => unreachable!(),
            }
        }
        Ok(query)
    }

    fn normalize(&self) -> Result<NormalizedNewsQuery, ()> {
        let q = self.q.clone().unwrap_or_default();
        if self.limit.is_some() {
            return Err(());
        }
        let category = self.category.clone().unwrap_or_else(|| "all".to_string());
        let safe_text = |value: &str, max: usize| {
            value.chars().count() <= max && !value.chars().any(|ch| ch.is_control())
        };
        if !safe_text(&q, 200)
            || !safe_text(&category, 64)
            || !NEWS_CATEGORIES.contains(&category.as_str())
        {
            return Err(());
        }
        Ok(NormalizedNewsQuery {
            page: self.page.unwrap_or(1).max(1),
            limit: NEWS_PAGE_SIZE,
            q,
            category,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UpstreamNewsArticle {
    #[serde(default)]
    id: Option<String>,
    slug: String,
    #[serde(default, rename = "href")]
    _href: Option<String>,
    title: String,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    excerpt: Option<String>,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    cover_image_url: Option<String>,
    #[serde(default)]
    image: Option<String>,
    #[serde(default)]
    author_wallet: Option<String>,
    #[serde(default)]
    author: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    tag1: String,
    #[serde(default)]
    tag2: String,
    #[serde(default)]
    published_at: Option<String>,
    #[serde(default)]
    published: Option<String>,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    read_time: Option<String>,
    #[serde(default)]
    featured: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct NewsListArticle {
    pub id: Option<String>,
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub cover_image_url: Option<String>,
    pub author: Option<String>,
    pub published_at: Option<String>,
    pub read_time: Option<String>,
    pub tags: Vec<String>,
    pub featured: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct NewsDetailArticle {
    pub id: Option<String>,
    pub slug: String,
    pub title: String,
    pub summary: Option<String>,
    pub body: String,
    pub cover_image_url: Option<String>,
    pub author: Option<String>,
    pub published_at: Option<String>,
    pub read_time: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub(crate) enum NewsListLoadOutcome {
    Ready {
        articles: Vec<NewsListArticle>,
        total: u64,
        page: u32,
        limit: u32,
        total_pages: u32,
        query: String,
        category: String,
    },
    Empty {
        total: u64,
        page: u32,
        limit: u32,
        total_pages: u32,
        query: String,
        category: String,
    },
    Error {
        code: String,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub(crate) enum NewsDetailLoadOutcome {
    Ready { article: NewsDetailArticle },
    NotFound,
    Error { code: String },
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

pub(crate) fn valid_news_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= 128
        && !slug.starts_with('-')
        && !slug.ends_with('-')
        && slug
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn valid_cover_image_url(value: &str) -> bool {
    if value.is_empty() || value.chars().any(char::is_control) || value.contains('\\') {
        return false;
    }
    if value.starts_with('/') {
        return !value.starts_with("//");
    }
    reqwest::Url::parse(value).is_ok_and(|url| {
        url.scheme() == "https"
            && url.username().is_empty()
            && url.password().is_none()
            && url.fragment().is_none()
    })
}

struct DecodedNewsPayload {
    data: serde_json::Value,
    legacy: bool,
}

fn object_has_only_keys(
    object: &serde_json::Map<String, serde_json::Value>,
    allowed: &[&str],
) -> bool {
    object.keys().all(|key| allowed.contains(&key.as_str()))
}

fn upstream_error_is_clear(error: Option<&serde_json::Value>) -> bool {
    match error {
        None | Some(serde_json::Value::Null) => true,
        Some(serde_json::Value::String(message)) => message.is_empty(),
        Some(_) => false,
    }
}

fn upstream_success_data(value: serde_json::Value) -> Result<DecodedNewsPayload, ()> {
    let Some(object) = value.as_object() else {
        return Err(());
    };
    if object.contains_key("success") {
        if !object_has_only_keys(object, &["success", "data", "error"])
            || object.get("success").and_then(serde_json::Value::as_bool) != Some(true)
            || !upstream_error_is_clear(object.get("error"))
        {
            return Err(());
        }
        return Ok(DecodedNewsPayload {
            data: object.get("data").cloned().ok_or(())?,
            legacy: false,
        });
    }
    Ok(DecodedNewsPayload {
        data: value,
        legacy: true,
    })
}

fn normalize_tags(mut article: UpstreamNewsArticle) -> UpstreamNewsArticle {
    if article.tags.is_empty() {
        if !article.tag1.is_empty() {
            article.tags.push(std::mem::take(&mut article.tag1));
        }
        if !article.tag2.is_empty() {
            article.tags.push(std::mem::take(&mut article.tag2));
        }
    }
    article
}

fn normalize_news_date(value: Option<String>) -> Result<Option<String>, ()> {
    let Some(value) = value else {
        return Ok(None);
    };
    let date = DateTime::parse_from_rfc3339(&value)
        .map(|date| date.date_naive())
        .or_else(|_| NaiveDate::parse_from_str(&value, "%Y-%m-%d"))
        .or_else(|_| NaiveDate::parse_from_str(&value, "%B %e, %Y"))
        .map_err(|_| ())?;
    const MONTHS: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    use chrono::Datelike;
    Ok(Some(format!(
        "{} {}, {}",
        MONTHS[date.month0() as usize],
        date.day(),
        date.year()
    )))
}

fn validate_common_article(article: &UpstreamNewsArticle) -> Result<(), ()> {
    if !valid_news_slug(&article.slug)
        || article.title.trim().is_empty()
        || article.title.chars().count() > 300
        || article
            .status
            .as_deref()
            .is_some_and(|status| status != "published")
        || article.tags.len() > 32
        || article.tags.iter().any(|tag| {
            tag.trim().is_empty() || tag.chars().count() > 64 || tag.chars().any(char::is_control)
        })
        || article
            .cover_image_url
            .as_deref()
            .or(article.image.as_deref())
            .is_some_and(|url| !valid_cover_image_url(url))
    {
        return Err(());
    }
    Ok(())
}

fn normalize_list_article(article: UpstreamNewsArticle) -> Result<NewsListArticle, ()> {
    let article = normalize_tags(article);
    validate_common_article(&article)?;
    let published_at = normalize_news_date(
        article
            .published_at
            .clone()
            .or(article.published.clone())
            .or(article.date.clone()),
    )?;
    let summary = article.summary.or(article.excerpt).unwrap_or_default();
    if summary.chars().count() > 2_000 || summary.chars().any(char::is_control) {
        return Err(());
    }
    Ok(NewsListArticle {
        id: article.id,
        slug: article.slug,
        title: article.title,
        summary,
        cover_image_url: article.cover_image_url.or(article.image),
        author: article.author_wallet.or(article.author),
        published_at,
        read_time: article.read_time,
        tags: article.tags,
        featured: article.featured,
    })
}

fn normalize_detail_article(
    article: UpstreamNewsArticle,
    expected_slug: &str,
) -> Result<NewsDetailArticle, ()> {
    let article = normalize_tags(article);
    validate_common_article(&article)?;
    if article.slug != expected_slug {
        return Err(());
    }
    if article.summary.as_deref().is_some_and(|summary| {
        summary.chars().count() > 2_000 || summary.chars().any(char::is_control)
    }) {
        return Err(());
    }
    let published_at = normalize_news_date(
        article
            .published_at
            .clone()
            .or(article.published.clone())
            .or(article.date.clone()),
    )?;
    let body = article.content.or(article.body).ok_or(())?;
    if body.trim().is_empty() || body.chars().count() > 500_000 {
        return Err(());
    }
    Ok(NewsDetailArticle {
        id: article.id,
        slug: article.slug,
        title: article.title,
        summary: article.summary,
        body,
        cover_image_url: article.cover_image_url.or(article.image),
        author: article.author_wallet.or(article.author),
        published_at,
        read_time: article.read_time,
        tags: article.tags,
    })
}

fn parse_news_list(value: serde_json::Value) -> Result<Vec<NewsListArticle>, ()> {
    let payload = upstream_success_data(value)?;
    let object = payload.data.as_object().ok_or(())?;
    let allowed = if payload.legacy {
        &["articles", "total"][..]
    } else {
        &["articles", "total", "page", "limit"][..]
    };
    if !object_has_only_keys(object, allowed)
        || !object.contains_key("articles")
        || !object.contains_key("total")
    {
        return Err(());
    }
    if object.get("page").is_some_and(|page| {
        page.as_u64()
            .is_none_or(|page| page == 0 || page > u32::MAX as u64)
    }) || object.get("limit").is_some_and(|limit| {
        limit
            .as_u64()
            .is_none_or(|limit| limit == 0 || limit > NEWS_UPSTREAM_FETCH_LIMIT as u64)
    }) {
        return Err(());
    }
    let raw_articles = object
        .get("articles")
        .and_then(serde_json::Value::as_array)
        .ok_or(())?;
    let upstream_total = object
        .get("total")
        .and_then(serde_json::Value::as_u64)
        .ok_or(())?;
    if upstream_total < raw_articles.len() as u64
        || raw_articles.len() > NEWS_UPSTREAM_FETCH_LIMIT as usize
    {
        return Err(());
    }
    raw_articles
        .iter()
        .cloned()
        .map(|value| serde_json::from_value(value).map_err(|_| ()))
        .map(|result| result.and_then(normalize_list_article))
        .collect()
}

fn parse_news_detail(
    value: serde_json::Value,
    expected_slug: &str,
) -> Result<NewsDetailArticle, ()> {
    let payload = upstream_success_data(value)?;
    let object = payload.data.as_object().ok_or(())?;
    if payload.legacy {
        const LEGACY_DETAIL_KEYS: [&str; 4] = ["slug", "title", "body", "published"];
        if object.len() != LEGACY_DETAIL_KEYS.len()
            || !LEGACY_DETAIL_KEYS
                .iter()
                .all(|key| object.contains_key(*key))
        {
            return Err(());
        }
    }
    let article = serde_json::from_value(payload.data).map_err(|_| ())?;
    normalize_detail_article(article, expected_slug)
}

fn news_dependency_error(error: ClientError) -> String {
    if matches!(&error, ClientError::Timeout)
        || matches!(&error, ClientError::Http(http) if http.is_timeout())
    {
        "content_timeout".to_string()
    } else {
        "content_unavailable".to_string()
    }
}

/// Load, validate, filter, and paginate public news without a canned fallback.
/// The current content service exposes at most 100 records, so category/search
/// remain a BFF display concern until A5 freezes a canonical query contract.
pub(crate) async fn load_news_list(
    client: &ServiceClient,
    query: &NewsQuery,
) -> NewsListLoadOutcome {
    let normalized = match query.normalize() {
        Ok(query) => query,
        Err(()) => {
            return NewsListLoadOutcome::Error {
                code: "invalid_news_query".to_string(),
            }
        }
    };
    let value = match client.get_plain(NEWS_LIST_PATH).await {
        Ok(value) => value,
        Err(error) => {
            return NewsListLoadOutcome::Error {
                code: news_dependency_error(error),
            }
        }
    };
    let articles = match parse_news_list(value) {
        Ok(articles) => articles,
        Err(()) => {
            return NewsListLoadOutcome::Error {
                code: "malformed_content_response".to_string(),
            }
        }
    };
    let needle = normalized.q.to_lowercase();
    let filtered: Vec<NewsListArticle> = articles
        .into_iter()
        .filter(|article| {
            let category_matches = normalized.category == "all"
                || article
                    .tags
                    .iter()
                    .any(|tag| tag.eq_ignore_ascii_case(&normalized.category));
            let query_matches = needle.is_empty()
                || article.title.to_lowercase().contains(&needle)
                || article.summary.to_lowercase().contains(&needle)
                || article
                    .tags
                    .iter()
                    .any(|tag| tag.to_lowercase().contains(&needle));
            category_matches && query_matches
        })
        .collect();
    let total = filtered.len() as u64;
    let total_pages = if total == 0 {
        0
    } else {
        total.div_ceil(normalized.limit as u64) as u32
    };
    let start = (normalized.page - 1) as usize * normalized.limit as usize;
    let page_articles: Vec<NewsListArticle> = filtered
        .into_iter()
        .skip(start)
        .take(normalized.limit as usize)
        .collect();
    if page_articles.is_empty() {
        NewsListLoadOutcome::Empty {
            total,
            page: normalized.page,
            limit: normalized.limit,
            total_pages,
            query: normalized.q,
            category: normalized.category,
        }
    } else {
        NewsListLoadOutcome::Ready {
            articles: page_articles,
            total,
            page: normalized.page,
            limit: normalized.limit,
            total_pages,
            query: normalized.q,
            category: normalized.category,
        }
    }
}

pub(crate) async fn load_news_post(client: &ServiceClient, slug: &str) -> NewsDetailLoadOutcome {
    if !valid_news_slug(slug) {
        return NewsDetailLoadOutcome::NotFound;
    }
    let path = format!("/api/v1/content/news/{slug}");
    let value = match client.get_plain(&path).await {
        Ok(value) => value,
        Err(ClientError::NotFound) => return NewsDetailLoadOutcome::NotFound,
        Err(error) => {
            return NewsDetailLoadOutcome::Error {
                code: news_dependency_error(error),
            }
        }
    };
    match parse_news_detail(value, slug) {
        Ok(article) => NewsDetailLoadOutcome::Ready { article },
        Err(()) => NewsDetailLoadOutcome::Error {
            code: "malformed_content_response".to_string(),
        },
    }
}

fn news_api_error(status: StatusCode, code: &str) -> Response {
    (
        status,
        Json(serde_json::json!({ "success": false, "error": code })),
    )
        .into_response()
}

pub async fn api_news(State(state): State<AppState>, Query(query): Query<NewsQuery>) -> Response {
    match load_news_list(state.content.as_ref(), &query).await {
        NewsListLoadOutcome::Ready {
            articles,
            total,
            page,
            limit,
            ..
        } => Json(serde_json::json!({
            "success": true,
            "data": { "articles": articles, "total": total, "page": page, "limit": limit }
        }))
        .into_response(),
        NewsListLoadOutcome::Empty {
            total, page, limit, ..
        } => Json(serde_json::json!({
            "success": true,
            "data": { "articles": [], "total": total, "page": page, "limit": limit }
        }))
        .into_response(),
        NewsListLoadOutcome::Error { code } => {
            let status = if code == "invalid_news_query" {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::BAD_GATEWAY
            };
            news_api_error(status, &code)
        }
    }
}

pub async fn api_news_post(
    AxPath(slug): AxPath<String>,
    State(state): State<AppState>,
) -> Response {
    match load_news_post(state.content.as_ref(), &slug).await {
        NewsDetailLoadOutcome::Ready { article } => Json(serde_json::json!({
            "success": true,
            "data": article
        }))
        .into_response(),
        NewsDetailLoadOutcome::NotFound => news_api_error(StatusCode::NOT_FOUND, "not_found"),
        NewsDetailLoadOutcome::Error { code } => news_api_error(StatusCode::BAD_GATEWAY, &code),
    }
}

#[cfg(test)]
mod news_adapter_tests {
    use super::*;
    use axum::{routing::get, Router};
    use std::time::Duration;

    async fn spawn_mock(router: Router) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        format!("http://{address}")
    }

    fn client(base_url: String) -> ServiceClient {
        ServiceClient::new(epsx_client::ClientConfig {
            base_url,
            timeout: Duration::from_secs(1),
        })
    }

    fn article(slug: &str, title: &str, tags: &[&str]) -> serde_json::Value {
        serde_json::json!({
            "id": format!("id-{slug}"),
            "slug": slug,
            "title": title,
            "summary": format!("Summary for {title}"),
            "content": format!("Body for {title}"),
            "cover_image_url": null,
            "author_wallet": "0x1111",
            "status": "published",
            "tags": tags,
            "published_at": "2026-07-22T00:00:00Z",
            "read_time": null,
            "featured": false
        })
    }

    #[tokio::test]
    async fn list_adapter_uses_live_payload_and_url_stable_filters_and_pagination() {
        let mut articles: Vec<serde_json::Value> = (1..=13)
            .map(|index| {
                article(
                    &format!("rust-{index}"),
                    &format!("Rust article {index}"),
                    &["engineering"],
                )
            })
            .collect();
        articles.push(article("product-note", "Product note", &["product"]));
        let payload = serde_json::json!({
            "success": true,
            "data": {
                "articles": articles,
                "total": 14,
                "page": 1,
                "limit": 100
            },
            "error": null
        });
        let router = Router::new().route(
            "/api/v1/content/news",
            get(move || {
                let payload = payload.clone();
                async move { Json(payload) }
            }),
        );
        let outcome = load_news_list(
            &client(spawn_mock(router).await),
            &NewsQuery {
                page: Some(2),
                limit: None,
                q: Some("rust".to_string()),
                category: Some("engineering".to_string()),
            },
        )
        .await;
        match outcome {
            NewsListLoadOutcome::Ready {
                articles,
                total,
                page,
                limit,
                total_pages,
                query,
                category,
                ..
            } => {
                assert_eq!(articles.len(), 1);
                assert_eq!(articles[0].slug, "rust-13");
                assert_eq!(articles[0].published_at.as_deref(), Some("July 22, 2026"));
                assert_eq!(total, 13);
                assert_eq!(page, 2);
                assert_eq!(limit, NEWS_PAGE_SIZE);
                assert_eq!(total_pages, 2);
                assert_eq!(query, "rust");
                assert_eq!(category, "engineering");
            }
            other => panic!("expected ready live list, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn empty_malformed_and_unavailable_list_outcomes_never_become_articles() {
        let empty_router = Router::new().route(
            "/api/v1/content/news",
            get(|| async { Json(serde_json::json!({"articles": [], "total": 0})) }),
        );
        assert!(matches!(
            load_news_list(
                &client(spawn_mock(empty_router).await),
                &NewsQuery::default()
            )
            .await,
            NewsListLoadOutcome::Empty { total: 0, .. }
        ));

        let malformed_router = Router::new().route(
            "/api/v1/content/news",
            get(|| async { Json(serde_json::json!({"articles": [{"slug": "fake"}]})) }),
        );
        assert!(matches!(
            load_news_list(
                &client(spawn_mock(malformed_router).await),
                &NewsQuery::default()
            )
            .await,
            NewsListLoadOutcome::Error { code } if code == "malformed_content_response"
        ));

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let unavailable = listener.local_addr().unwrap();
        drop(listener);
        assert!(matches!(
            load_news_list(
                &client(format!("http://{unavailable}")),
                &NewsQuery::default()
            )
            .await,
            NewsListLoadOutcome::Error { code } if code == "content_unavailable"
        ));
    }

    #[tokio::test]
    async fn detail_adapter_preserves_slug_ownership_not_found_and_malformed_states() {
        let valid = article("live-article", "Live article", &["engineering"]);
        let valid_router = Router::new().route(
            "/api/v1/content/news/live-article",
            get(move || {
                let valid = valid.clone();
                async move { Json(serde_json::json!({"success": true, "data": valid})) }
            }),
        );
        assert!(matches!(
            load_news_post(&client(spawn_mock(valid_router).await), "live-article").await,
            NewsDetailLoadOutcome::Ready { article }
                if article.slug == "live-article"
                    && article.published_at.as_deref() == Some("July 22, 2026")
        ));

        let missing_router = Router::new().route(
            "/api/v1/content/news/missing",
            get(|| async { StatusCode::NOT_FOUND }),
        );
        assert!(matches!(
            load_news_post(&client(spawn_mock(missing_router).await), "missing").await,
            NewsDetailLoadOutcome::NotFound
        ));

        let mismatch = article("other-article", "Wrong owner", &["engineering"]);
        let mismatch_router = Router::new().route(
            "/api/v1/content/news/live-article",
            get(move || {
                let mismatch = mismatch.clone();
                async move {
                    Json(serde_json::json!({
                        "success": true,
                        "data": mismatch,
                        "error": null
                    }))
                }
            }),
        );
        assert!(matches!(
            load_news_post(&client(spawn_mock(mismatch_router).await), "live-article").await,
            NewsDetailLoadOutcome::Error { code } if code == "malformed_content_response"
        ));
        assert!(matches!(
            load_news_post(&client("http://127.0.0.1:9".to_string()), "../escape").await,
            NewsDetailLoadOutcome::NotFound
        ));
    }

    #[test]
    fn error_bearing_or_shape_ambiguous_payloads_never_become_success() {
        for payload in [
            serde_json::json!({"articles": [], "total": 0, "error": "boom"}),
            serde_json::json!({
                "success": true,
                "data": {"articles": [], "total": 0},
                "error": "boom"
            }),
            serde_json::json!({
                "success": true,
                "data": {"articles": [], "total": 0, "error": "nested"},
                "error": null
            }),
            serde_json::json!({
                "success": true,
                "data": {"articles": [], "total": 0},
                "error": null,
                "message": "ambiguous"
            }),
        ] {
            assert!(parse_news_list(payload).is_err());
        }
        assert!(parse_news_list(serde_json::json!({
            "success": true,
            "data": {"articles": [], "total": 0},
            "error": ""
        }))
        .is_ok());

        assert!(parse_news_detail(
            serde_json::json!({
                "slug": "live-article",
                "title": "Live article",
                "body": "Body",
                "published": "July 22, 2026",
                "error": "boom"
            }),
            "live-article"
        )
        .is_err());
    }

    #[test]
    fn malformed_dates_fail_closed_and_current_legacy_detail_is_normalized() {
        assert_eq!(
            normalize_news_date(Some("May 9, 2026".to_string())).unwrap(),
            Some("May 9, 2026".to_string())
        );
        let mut malformed = article("live-article", "Live article", &["engineering"]);
        malformed["published_at"] = serde_json::json!("not-a-date");
        assert!(parse_news_detail(
            serde_json::json!({"success": true, "data": malformed.clone(), "error": null}),
            "live-article"
        )
        .is_err());

        assert!(parse_news_list(serde_json::json!({
            "success": true,
            "data": {"articles": [malformed], "total": 1, "page": 1, "limit": 100},
            "error": null
        }))
        .is_err());

        let legacy = parse_news_detail(
            serde_json::json!({
                "slug": "live-article",
                "title": "Live article",
                "body": "Body",
                "published": "2026-07-22T00:00:00Z"
            }),
            "live-article",
        )
        .expect("current exact legacy shape should remain supported");
        assert_eq!(legacy.published_at.as_deref(), Some("July 22, 2026"));
    }

    #[test]
    fn raw_query_parser_rejects_ambiguous_or_malformed_owned_fields() {
        let valid = NewsQuery::from_raw_query("q=rust&category=engineering&page=2")
            .unwrap()
            .normalize()
            .unwrap();
        assert_eq!(valid.limit, NEWS_PAGE_SIZE);
        assert!(NewsQuery::from_raw_query("page=1&page=2").is_err());
        assert!(NewsQuery::from_raw_query("page=not-a-number").is_err());
        assert!(NewsQuery::from_raw_query("category=security")
            .unwrap()
            .normalize()
            .is_err());
        assert!(NewsQuery::from_raw_query("category=Engineering")
            .unwrap()
            .normalize()
            .is_err());
        assert!(NewsQuery::from_raw_query("limit=1")
            .unwrap()
            .normalize()
            .is_err());
        assert!(NewsQuery::from_raw_query(&format!("q={}", "x".repeat(201)))
            .unwrap()
            .normalize()
            .is_err());
    }
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

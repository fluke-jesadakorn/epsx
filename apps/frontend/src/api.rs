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
//! Production-facing handlers fail closed when an authoritative upstream
//! contract is unavailable. News uses strict dependency outcomes; rankings
//! and plans have no compatibility producers until their owning backend
//! contracts are frozen.

use axum::{
    extract::{Path as AxPath, Query, RawQuery, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, NaiveDate};
use epsx_bff::{
    cookies::{append_clear_session_cookies, append_session_cookies, CookieClient},
    refresh_outcome::{
        classify_refresh_outcome, is_rejected_refresh_outcome, mark_session_state,
        RefreshDisposition,
    },
    session::{
        AuthExchange, ChallengeRequest, ChallengeResponse, LogoutRequest, ProfileResponse,
        RefreshRequest, RefreshResponse, SessionUser, VerifyRequest, VerifyResponse,
        CHALLENGE_PATH, FRONTEND_CLIENT_ID, LOGOUT_PATH, PROFILE_PATH, REFRESH_PATH, VERIFY_PATH,
    },
};
use epsx_client::{ClientError, RequestContext, ServiceClient};
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
        .auth_client()
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
    mark_session_no_store(&mut response);
    if let Err(error) = append_session_cookies(
        response.headers_mut(),
        state.cookie_environment,
        CookieClient::Frontend,
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
        .auth_client()
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
        return clear_refresh_session_response(
            &state,
            StatusCode::UNAUTHORIZED,
            "missing_refresh_token",
        );
    };
    let request = RefreshRequest {
        refresh_token: &refresh_token,
        client_id: FRONTEND_CLIENT_ID,
    };
    let response = match state
        .identity
        .auth_client()
        .post(auth_url(&state, REFRESH_PATH))
        .json(&request)
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => {
            tracing::warn!("Refresh upstream unavailable: {}", error);
            return clear_refresh_session_response(
                &state,
                StatusCode::BAD_GATEWAY,
                "refresh_outcome_unknown",
            );
        }
    };
    let status = response.status();
    let rejected = is_rejected_refresh_outcome(status, response.headers());
    match classify_refresh_outcome(status, response.headers()) {
        RefreshDisposition::Preserve => {
            return refresh_response(
                safe_error(status, "refresh_not_rotated"),
                RefreshDisposition::Preserve,
            )
        }
        RefreshDisposition::Clear => {
            let (status, code) = if rejected {
                (StatusCode::UNAUTHORIZED, "refresh_rejected")
            } else {
                (StatusCode::BAD_GATEWAY, "refresh_outcome_unknown")
            };
            return clear_refresh_session_response(&state, status, code);
        }
        RefreshDisposition::Replace => {}
    }
    let upstream: RefreshResponse = match response.json().await {
        Ok(upstream) => upstream,
        Err(error) => {
            tracing::warn!("Refresh upstream returned malformed JSON: {}", error);
            return clear_refresh_session_response(
                &state,
                StatusCode::BAD_GATEWAY,
                "malformed_auth_response",
            );
        }
    };
    let exchange = match upstream.into_exchange() {
        Ok(exchange) => exchange,
        Err(_) => {
            return clear_refresh_session_response(
                &state,
                StatusCode::UNAUTHORIZED,
                "refresh_rejected",
            )
        }
    };
    let response = establish_session(&state, exchange, None, true).await;
    if response.status().is_success() {
        refresh_response(response, RefreshDisposition::Replace)
    } else {
        response
    }
}

fn refresh_response(mut response: Response, disposition: RefreshDisposition) -> Response {
    mark_session_state(&mut response, disposition);
    response
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
        .auth_client()
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
    mark_session_no_store(&mut response);
    if append_clear_session_cookies(
        response.headers_mut(),
        state.cookie_environment,
        CookieClient::Frontend,
    )
    .is_err()
    {
        return safe_error(StatusCode::INTERNAL_SERVER_ERROR, "session_cookie_error");
    }
    mark_session_state(&mut response, RefreshDisposition::Clear);
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
    let client = state.identity.auth_client();
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
    let mut response = Json(profile).into_response();
    mark_session_no_store(&mut response);
    response
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
    let mut response = (
        status,
        Json(serde_json::json!({ "success": false, "error": code })),
    )
        .into_response();
    mark_session_no_store(&mut response);
    response
}

fn mark_session_no_store(response: &mut Response) {
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
}

fn session_establishment_error(
    state: &AppState,
    clear_on_failure: bool,
    code: &'static str,
) -> Response {
    if clear_on_failure {
        clear_refresh_session_response(state, StatusCode::BAD_GATEWAY, code)
    } else {
        safe_error(StatusCode::BAD_GATEWAY, code)
    }
}

fn clear_session_response(state: &AppState, status: StatusCode, code: &'static str) -> Response {
    try_clear_session_response(status, code, |headers| {
        append_clear_session_cookies(
            headers,
            state.cookie_environment,
            CookieClient::Frontend,
        )
        .is_ok()
    })
    .unwrap_or_else(|error| error)
}

fn clear_refresh_session_response(
    state: &AppState,
    status: StatusCode,
    code: &'static str,
) -> Response {
    match try_clear_session_response(status, code, |headers| {
        append_clear_session_cookies(
            headers,
            state.cookie_environment,
            CookieClient::Frontend,
        )
        .is_ok()
    }) {
        Ok(response) => refresh_response(response, RefreshDisposition::Clear),
        Err(error) => error,
    }
}

fn try_clear_session_response(
    status: StatusCode,
    code: &'static str,
    append: impl FnOnce(&mut axum::http::HeaderMap) -> bool,
) -> Result<Response, Response> {
    let mut response = safe_error(status, code);
    if !append(response.headers_mut()) {
        return Err(safe_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "session_cookie_error",
        ));
    }
    Ok(response)
}

async fn verified_bearer(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Result<String, Response> {
    verified_bearer_and_user(state, headers)
        .await
        .map(|(token, _)| token)
}

async fn verified_bearer_and_user(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Result<(String, SessionUser), Response> {
    super::auth::verified_access_token(headers, state.verifier.as_ref(), state.cookie_environment)
        .await
        .ok_or_else(|| safe_error(StatusCode::UNAUTHORIZED, "invalid_access_token"))
}

const NOTIFICATION_LIST_LIMIT_MAX: u16 = 100;
const NOTIFICATION_LIST_OFFSET_MAX: u32 = 1_000_000;
// The list endpoint returns at most 100 rows. A 2 MiB cap leaves roughly
// 20 KiB per row for the body and JSON data while preventing a chunked
// upstream response from forcing unbounded BFF allocation. The unread
// response is the single-field `{ "count": u64 }` DTO constrained to the
// largest integer JavaScript can represent exactly.
const NOTIFICATION_LIST_BODY_MAX: usize = 2 * 1024 * 1024;
const NOTIFICATION_UNREAD_BODY_MAX: usize = 4 * 1024;
const NOTIFICATION_UNREAD_JS_SAFE_MAX: u64 = 9_007_199_254_740_991;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct NotificationListQuery {
    limit: Option<u16>,
    offset: Option<u32>,
    status: Option<String>,
}

impl NotificationListQuery {
    fn from_raw_query(raw: Option<&str>) -> Result<Self, ()> {
        let Some(raw) = raw.filter(|raw| !raw.is_empty()) else {
            return Ok(Self::default());
        };
        let url =
            reqwest::Url::parse(&format!("https://frontend.invalid/?{raw}")).map_err(|_| ())?;
        let mut query = Self::default();
        let mut seen = std::collections::HashSet::new();
        for (key, value) in url.query_pairs() {
            let key = key.as_ref();
            if !seen.insert(key.to_string()) {
                return Err(());
            }
            match key {
                "limit" => {
                    let value: u16 = value.parse().map_err(|_| ())?;
                    if !(1..=NOTIFICATION_LIST_LIMIT_MAX).contains(&value) {
                        return Err(());
                    }
                    query.limit = Some(value);
                }
                "offset" => {
                    let value: u32 = value.parse().map_err(|_| ())?;
                    if value > NOTIFICATION_LIST_OFFSET_MAX {
                        return Err(());
                    }
                    query.offset = Some(value);
                }
                "status" => {
                    if !matches!(value.as_ref(), "pending" | "sent" | "failed") {
                        return Err(());
                    }
                    query.status = Some(value.into_owned());
                }
                _ => return Err(()),
            }
        }
        Ok(query)
    }

    fn upstream_suffix(&self) -> String {
        let mut fields = Vec::new();
        if let Some(limit) = self.limit {
            fields.push(format!("limit={limit}"));
        }
        if let Some(offset) = self.offset {
            fields.push(format!("offset={offset}"));
        }
        if let Some(status) = &self.status {
            fields.push(format!("status={status}"));
        }
        if fields.is_empty() {
            String::new()
        } else {
            format!("?{}", fields.join("&"))
        }
    }
}

#[derive(Debug)]
enum RequiredNullable<T> {
    Missing,
    Present(Option<T>),
}

impl<T> Default for RequiredNullable<T> {
    fn default() -> Self {
        Self::Missing
    }
}

impl<'de, T> Deserialize<'de> for RequiredNullable<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Option::<T>::deserialize(deserializer).map(Self::Present)
    }
}

impl<T> RequiredNullable<T> {
    fn as_ref(&self) -> Result<Option<&T>, ()> {
        match self {
            Self::Missing => Err(()),
            Self::Present(value) => Ok(value.as_ref()),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NotificationListWire {
    items: Vec<NotificationWire>,
    total: i64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct NotificationWire {
    id: String,
    #[serde(default)]
    user_id: RequiredNullable<String>,
    channel: String,
    recipient: String,
    #[serde(default)]
    template_id: RequiredNullable<String>,
    #[serde(default)]
    subject: RequiredNullable<String>,
    body: String,
    #[serde(default)]
    data: RequiredNullable<serde_json::Value>,
    status: String,
    #[serde(default)]
    error: RequiredNullable<String>,
    #[serde(default)]
    sent_at: RequiredNullable<DateTime<chrono::Utc>>,
    created_at: DateTime<chrono::Utc>,
    #[serde(default)]
    read_at: RequiredNullable<DateTime<chrono::Utc>>,
    #[serde(default)]
    title: RequiredNullable<String>,
    #[serde(default)]
    notification_type: RequiredNullable<String>,
    #[serde(default)]
    priority: RequiredNullable<String>,
    #[serde(default)]
    action_url: RequiredNullable<String>,
}

impl NotificationListWire {
    fn validate(&self, owner: &str, query: &NotificationListQuery) -> Result<(), ()> {
        let limit = usize::from(query.limit.unwrap_or(50));
        if self.total < 0 || self.items.len() > limit || self.total < self.items.len() as i64 {
            return Err(());
        }
        // The service's unfiltered count describes the same owner query as the
        // unfiltered page. Require an exact page cardinality so a split count /
        // row read cannot turn contradictory data into an authoritative empty
        // state. The current service count is deliberately not filter-aware,
        // so status-filtered pages retain only the conservative bounds above.
        if query.status.is_none() {
            let offset = u64::from(query.offset.unwrap_or(0));
            let remaining = (self.total as u64).saturating_sub(offset);
            let expected = remaining.min(limit as u64) as usize;
            if self.items.len() != expected {
                return Err(());
            }
        }
        for item in &self.items {
            if !item
                .user_id
                .as_ref()?
                .is_some_and(|user_id| user_id.eq_ignore_ascii_case(owner))
            {
                return Err(());
            }
            item.template_id.as_ref()?;
            item.subject.as_ref()?;
            item.data.as_ref()?;
            item.error.as_ref()?;
            item.sent_at.as_ref()?;
            item.read_at.as_ref()?;
            item.title.as_ref()?;
            item.notification_type.as_ref()?;
            item.priority.as_ref()?;
            item.action_url.as_ref()?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NotificationRequestId(String);

pub(crate) fn notification_request_id(headers: &axum::http::HeaderMap) -> NotificationRequestId {
    NotificationRequestId(RequestContext::from_headers(headers).request_id.to_string())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NotificationListUnavailable {
    Unauthorized,
    UpstreamFailed,
    Dependency,
}

#[derive(Debug, PartialEq)]
pub(crate) enum NotificationListLoadOutcome {
    Ready(serde_json::Value),
    Empty(serde_json::Value),
    Unavailable(NotificationListUnavailable),
    Malformed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NotificationBodyReadError {
    TooLarge,
    Transport,
}

pub(crate) async fn load_owner_notifications(
    client: &ServiceClient,
    bearer: &str,
    owner: &str,
    query: &NotificationListQuery,
    request_id: &NotificationRequestId,
) -> NotificationListLoadOutcome {
    let url = format!(
        "{}/api/v1/notification/list{}",
        client.base_url().trim_end_matches('/'),
        query.upstream_suffix()
    );
    let response = match client
        .auth_client()
        .get(url)
        .bearer_auth(bearer)
        .header("x-request-id", request_id.0.as_str())
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => {
            return NotificationListLoadOutcome::Unavailable(
                NotificationListUnavailable::Dependency,
            )
        }
    };
    if !response.status().is_success() {
        return NotificationListLoadOutcome::Unavailable(
            if response.status() == StatusCode::UNAUTHORIZED {
                NotificationListUnavailable::Unauthorized
            } else {
                NotificationListUnavailable::UpstreamFailed
            },
        );
    }

    let body = match read_notification_body_limited(response, NOTIFICATION_LIST_BODY_MAX).await {
        Ok(body) => body,
        Err(NotificationBodyReadError::TooLarge) => return NotificationListLoadOutcome::Malformed,
        Err(NotificationBodyReadError::Transport) => {
            return NotificationListLoadOutcome::Unavailable(
                NotificationListUnavailable::Dependency,
            )
        }
    };
    let payload = match serde_json::from_slice::<NotificationListWire>(&body) {
        Ok(payload) if payload.validate(owner, query).is_ok() => payload,
        Ok(_) | Err(_) => return NotificationListLoadOutcome::Malformed,
    };
    let value = match serde_json::from_slice::<serde_json::Value>(&body) {
        Ok(value) => value,
        Err(_) => return NotificationListLoadOutcome::Malformed,
    };
    if payload.items.is_empty() {
        NotificationListLoadOutcome::Empty(value)
    } else {
        NotificationListLoadOutcome::Ready(value)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct NotificationUnreadCount {
    count: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NotificationUnreadLoadOutcome {
    Ready(u64),
    DependencyUnavailable,
    UpstreamFailed,
    Malformed,
}

fn valid_notification_json_content_type(headers: &axum::http::HeaderMap) -> bool {
    let mut values = headers.get_all(header::CONTENT_TYPE).iter();
    let Some(value) = values.next() else {
        return false;
    };
    if values.next().is_some() {
        return false;
    }
    let Ok(value) = value.to_str() else {
        return false;
    };
    let mut parts = value.split(';');
    if !parts
        .next()
        .is_some_and(|media_type| media_type.trim().eq_ignore_ascii_case("application/json"))
    {
        return false;
    }

    let Some(parameter) = parts.next() else {
        return true;
    };
    if parts.next().is_some() {
        return false;
    }
    let Some((name, raw_value)) = parameter.trim().split_once('=') else {
        return false;
    };
    if !name.trim().eq_ignore_ascii_case("charset") {
        return false;
    }
    let raw_value = raw_value.trim();
    let charset = match raw_value.strip_prefix('"') {
        Some(value) => match value.strip_suffix('"') {
            Some(value) => value,
            None => return false,
        },
        None if raw_value.ends_with('"') => return false,
        None => raw_value,
    };
    !charset.is_empty()
        && !charset
            .chars()
            .any(|character| matches!(character, '"' | '\\'))
        && charset.eq_ignore_ascii_case("utf-8")
}

fn unread_status_is_dependency_unavailable(status: StatusCode) -> bool {
    matches!(status.as_u16(), 408 | 425 | 429) || status.is_server_error()
}

pub(crate) async fn load_notification_unread_count(
    client: &ServiceClient,
    bearer: &str,
    request_id: &NotificationRequestId,
) -> NotificationUnreadLoadOutcome {
    let url = format!(
        "{}/api/v1/notification/unread-count",
        client.base_url().trim_end_matches('/')
    );
    let response = match client
        .auth_client()
        .get(url)
        .bearer_auth(bearer)
        .header("x-request-id", request_id.0.as_str())
        .header(header::ACCEPT, HeaderValue::from_static("application/json"))
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => return NotificationUnreadLoadOutcome::DependencyUnavailable,
    };
    if response.status() != StatusCode::OK {
        return if unread_status_is_dependency_unavailable(response.status()) {
            NotificationUnreadLoadOutcome::DependencyUnavailable
        } else {
            NotificationUnreadLoadOutcome::UpstreamFailed
        };
    }
    if !valid_notification_json_content_type(response.headers()) {
        return NotificationUnreadLoadOutcome::Malformed;
    }
    let body = match read_notification_body_limited(response, NOTIFICATION_UNREAD_BODY_MAX).await {
        Ok(body) => body,
        Err(NotificationBodyReadError::TooLarge) => {
            return NotificationUnreadLoadOutcome::Malformed
        }
        Err(NotificationBodyReadError::Transport) => {
            return NotificationUnreadLoadOutcome::DependencyUnavailable
        }
    };
    match serde_json::from_slice::<NotificationUnreadCount>(&body) {
        Ok(payload) if payload.count <= NOTIFICATION_UNREAD_JS_SAFE_MAX => {
            NotificationUnreadLoadOutcome::Ready(payload.count)
        }
        Ok(_) | Err(_) => NotificationUnreadLoadOutcome::Malformed,
    }
}

fn notification_upstream_error(status: StatusCode) -> Response {
    if status == StatusCode::UNAUTHORIZED {
        safe_error(
            StatusCode::UNAUTHORIZED,
            "notification_upstream_unauthorized",
        )
    } else {
        safe_error(StatusCode::BAD_GATEWAY, "notification_upstream_failed")
    }
}

fn private_notification_response(mut response: Response) -> Response {
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    response.headers_mut().insert(
        header::VARY,
        HeaderValue::from_static("Cookie, Authorization"),
    );
    response
}

async fn read_notification_body_limited(
    mut response: reqwest::Response,
    limit: usize,
) -> Result<Vec<u8>, NotificationBodyReadError> {
    if response
        .content_length()
        .is_some_and(|length| length > limit as u64)
    {
        return Err(NotificationBodyReadError::TooLarge);
    }

    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| NotificationBodyReadError::Transport)?
    {
        let next_len = body
            .len()
            .checked_add(chunk.len())
            .ok_or(NotificationBodyReadError::TooLarge)?;
        if next_len > limit {
            return Err(NotificationBodyReadError::TooLarge);
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

pub async fn notifications_api(
    state: State<AppState>,
    headers: axum::http::HeaderMap,
    raw_query: RawQuery,
) -> Response {
    private_notification_response(notifications_api_inner(state, headers, raw_query).await)
}

async fn notifications_api_inner(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    RawQuery(raw_query): RawQuery,
) -> Response {
    let query = match NotificationListQuery::from_raw_query(raw_query.as_deref()) {
        Ok(query) => query,
        Err(()) => return safe_error(StatusCode::BAD_REQUEST, "invalid_notification_query"),
    };
    let (token, user) = match verified_bearer_and_user(&state, &headers).await {
        Ok(session) => session,
        Err(response) => return response,
    };
    let request_id = notification_request_id(&headers);
    match load_owner_notifications(
        state.notification.as_ref(),
        &token,
        &user.wallet_address,
        &query,
        &request_id,
    )
    .await
    {
        NotificationListLoadOutcome::Ready(value) | NotificationListLoadOutcome::Empty(value) => {
            Json(value).into_response()
        }
        NotificationListLoadOutcome::Malformed => {
            safe_error(StatusCode::BAD_GATEWAY, "malformed_notification_response")
        }
        NotificationListLoadOutcome::Unavailable(NotificationListUnavailable::Unauthorized) => {
            notification_upstream_error(StatusCode::UNAUTHORIZED)
        }
        NotificationListLoadOutcome::Unavailable(NotificationListUnavailable::UpstreamFailed) => {
            notification_upstream_error(StatusCode::BAD_GATEWAY)
        }
        NotificationListLoadOutcome::Unavailable(NotificationListUnavailable::Dependency) => {
            safe_error(StatusCode::BAD_GATEWAY, "notification_upstream_unavailable")
        }
    }
}

pub async fn notification_unread_count(
    state: State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    private_notification_response(notification_unread_count_inner(state, headers).await)
}

fn notification_unread_load_response(outcome: NotificationUnreadLoadOutcome) -> Response {
    match outcome {
        NotificationUnreadLoadOutcome::Ready(count) => {
            Json(NotificationUnreadCount { count }).into_response()
        }
        NotificationUnreadLoadOutcome::DependencyUnavailable => safe_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "notification_upstream_unavailable",
        ),
        NotificationUnreadLoadOutcome::UpstreamFailed => {
            safe_error(StatusCode::BAD_GATEWAY, "notification_upstream_failed")
        }
        NotificationUnreadLoadOutcome::Malformed => {
            safe_error(StatusCode::BAD_GATEWAY, "malformed_notification_response")
        }
    }
}

async fn notification_unread_count_inner(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let token = match verified_bearer(&state, &headers).await {
        Ok(token) => token,
        Err(response) => return response,
    };
    let request_id = notification_request_id(&headers);
    notification_unread_load_response(
        load_notification_unread_count(state.notification.as_ref(), &token, &request_id).await,
    )
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
    if value.is_empty()
        || value.len() > 2_048
        || value.chars().any(char::is_control)
        || value.contains('\\')
        || value.contains('#')
    {
        return false;
    }
    if value.starts_with('/') {
        return !value.starts_with("//")
            && reqwest::Url::parse("https://epsx.invalid/")
                .and_then(|base| base.join(value))
                .is_ok_and(|url| {
                    url.scheme() == "https"
                        && url.host_str() == Some("epsx.invalid")
                        && url.username().is_empty()
                        && url.password().is_none()
                        && url.fragment().is_none()
                });
    }
    let Some(authority) = canonical_https_authority(value) else {
        return false;
    };
    if authority.contains('@') {
        return false;
    }
    reqwest::Url::parse(value).is_ok_and(|url| {
        url.scheme() == "https"
            && url.host_str().is_some()
            && url.username().is_empty()
            && url.password().is_none()
            && url.fragment().is_none()
    })
}

fn canonical_https_authority(value: &str) -> Option<&str> {
    let Some((scheme, rest)) = value.split_once("://") else {
        return None;
    };
    if !scheme.eq_ignore_ascii_case("https") {
        return None;
    }
    let authority = rest.split(['/', '?', '#']).next()?;
    (!authority.is_empty()).then_some(authority)
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
    let year = date.year();
    if !(1..=9_999).contains(&year) {
        return Err(());
    }
    Ok(Some(format!(
        "{} {}, {year:04}",
        MONTHS[date.month0() as usize],
        date.day(),
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
    let title = article.title.trim().to_string();
    if article.slug != expected_slug
        || article
            .id
            .as_deref()
            .is_some_and(|id| id.chars().count() > 128 || id.chars().any(char::is_control))
        || title.is_empty()
        || title.chars().count() > 200
        || title.chars().any(char::is_control)
    {
        return Err(());
    }
    if article.summary.as_deref().is_some_and(|summary| {
        summary.chars().count() > 500 || summary.chars().any(char::is_control)
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
    if body.trim().is_empty() || body.len() > 256 * 1_024 {
        return Err(());
    }
    let author = article
        .author_wallet
        .or(article.author)
        .map(|author| author.trim().to_string());
    if author.as_deref().is_some_and(|author| {
        author.trim().is_empty()
            || author.chars().count() > 120
            || author.chars().any(char::is_control)
    }) {
        return Err(());
    }
    Ok(NewsDetailArticle {
        id: article.id,
        slug: article.slug,
        title,
        summary: article.summary,
        body,
        cover_image_url: article.cover_image_url.or(article.image),
        author,
        published_at,
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
    fn news_dates_are_canonical_and_bounded_to_four_digit_common_era_years() {
        assert_eq!(
            normalize_news_date(Some("0001-01-01".to_string())).unwrap(),
            Some("January 1, 0001".to_string())
        );
        assert_eq!(
            normalize_news_date(Some("9999-12-31".to_string())).unwrap(),
            Some("December 31, 9999".to_string())
        );
        for invalid in ["0000-01-01", "+10000-01-01"] {
            assert!(
                normalize_news_date(Some(invalid.to_string())).is_err(),
                "out-of-contract year was accepted: {invalid}"
            );
        }
    }

    #[test]
    fn detail_adapter_enforces_production_field_and_body_bounds() {
        let valid = serde_json::json!({
            "success": true,
            "data": article("live-article", "  Live article  ", &["engineering"]),
            "error": null
        });
        let normalized = parse_news_detail(valid, "live-article").unwrap();
        assert_eq!(normalized.title, "Live article");
        assert_eq!(normalized.author.as_deref(), Some("0x1111"));

        for (field, value) in [
            ("id", serde_json::json!("x".repeat(129))),
            ("id", serde_json::json!("bad\u{0}id")),
            ("title", serde_json::json!("x".repeat(201))),
            ("title", serde_json::json!("bad\ntitle")),
            ("summary", serde_json::json!("x".repeat(501))),
            ("summary", serde_json::json!("bad\u{0}summary")),
            ("author_wallet", serde_json::json!(" ")),
            ("author_wallet", serde_json::json!("bad\nauthor")),
        ] {
            let mut malformed = article("live-article", "Live article", &["engineering"]);
            malformed[field] = value;
            assert!(parse_news_detail(
                serde_json::json!({"success": true, "data": malformed, "error": null}),
                "live-article"
            )
            .is_err());
        }

        let mut exact_body = article("live-article", "Live article", &["engineering"]);
        exact_body["content"] = serde_json::json!("x".repeat(256 * 1_024));
        assert!(parse_news_detail(
            serde_json::json!({"success": true, "data": exact_body, "error": null}),
            "live-article"
        )
        .is_ok());

        let mut oversized_body = article("live-article", "Live article", &["engineering"]);
        oversized_body["content"] = serde_json::json!("x".repeat(256 * 1_024 + 1));
        assert!(parse_news_detail(
            serde_json::json!({"success": true, "data": oversized_body, "error": null}),
            "live-article"
        )
        .is_err());
    }

    #[test]
    fn detail_adapter_ignores_upstream_read_time() {
        let mut upstream = article("live-article", "Live article", &["engineering"]);
        upstream["read_time"] = serde_json::json!("999 min");
        let normalized = parse_news_detail(
            serde_json::json!({"success": true, "data": upstream, "error": null}),
            "live-article",
        )
        .unwrap();
        let serialized = serde_json::to_value(normalized).unwrap();
        assert!(serialized.get("read_time").is_none());
    }

    #[test]
    fn detail_adapter_accepts_only_safe_covers_and_real_dates() {
        let oversized_cover = format!("https://example.com/{}", "x".repeat(2_048));
        for cover in [
            "//evil.example/image.png",
            "http://example.com/image.png",
            "https://user@example.com/image.png",
            "https://@example.com/image.png",
            "HTTPS://user@example.com/image.png",
            "HTTPS://@example.com/image.png",
            "https:////@example.com/image.png",
            "https:/@example.com/image.png",
            "https:@example.com/image.png",
            "https://example.com/image.png#fragment",
            "/image\\name.png",
            oversized_cover.as_str(),
        ] {
            let mut malformed = article("live-article", "Live article", &["engineering"]);
            malformed["cover_image_url"] = serde_json::json!(cover);
            assert!(parse_news_detail(
                serde_json::json!({"success": true, "data": malformed, "error": null}),
                "live-article"
            )
            .is_err());
        }

        for cover in [
            "/images/news.png",
            "https://example.com/images/news.png",
            "HTTPS://example.com/images/news.png",
        ] {
            let mut valid = article("live-article", "Live article", &["engineering"]);
            valid["cover_image_url"] = serde_json::json!(cover);
            let normalized = parse_news_detail(
                serde_json::json!({"success": true, "data": valid, "error": null}),
                "live-article",
            )
            .unwrap();
            assert_eq!(normalized.cover_image_url.as_deref(), Some(cover));
        }

        let mut impossible_date = article("live-article", "Live article", &["engineering"]);
        impossible_date["published_at"] = serde_json::json!("2026-02-30");
        assert!(parse_news_detail(
            serde_json::json!({"success": true, "data": impossible_date, "error": null}),
            "live-article"
        )
        .is_err());
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

#[cfg(test)]
mod notification_contract_tests {
    use super::*;

    fn notification(owner: &str) -> serde_json::Value {
        serde_json::json!({
            "id": "notification-1",
            "user_id": owner,
            "channel": "in_app",
            "recipient": owner,
            "template_id": null,
            "subject": "Subject",
            "body": "Body",
            "data": null,
            "status": "sent",
            "error": null,
            "sent_at": "2026-07-22T00:00:00Z",
            "created_at": "2026-07-22T00:00:00Z",
            "read_at": null,
            "title": "Title",
            "notification_type": "system",
            "priority": "normal",
            "action_url": null
        })
    }

    #[test]
    fn list_query_allows_only_bounded_service_fields() {
        let query =
            NotificationListQuery::from_raw_query(Some("status=sent&offset=1000000&limit=100"))
                .unwrap();
        assert_eq!(query.limit, Some(100));
        assert_eq!(query.offset, Some(1_000_000));
        assert_eq!(query.status.as_deref(), Some("sent"));
        assert_eq!(
            query.upstream_suffix(),
            "?limit=100&offset=1000000&status=sent"
        );

        for invalid in [
            "user_id=0xother",
            "caller=0xother",
            "limit=0",
            "limit=101",
            "limit=-1",
            "offset=-1",
            "offset=1000001",
            "status=read",
            "status=sent&status=failed",
            "limit=1&limit=2",
            "unknown=value",
        ] {
            assert!(
                NotificationListQuery::from_raw_query(Some(invalid)).is_err(),
                "query must fail closed: {invalid}"
            );
        }
    }

    #[test]
    fn exact_list_shape_requires_owner_and_every_service_field() {
        let owner = "0x1111111111111111111111111111111111111111";
        let one_row_query = NotificationListQuery {
            limit: Some(1),
            ..NotificationListQuery::default()
        };
        let valid = serde_json::json!({"items": [notification(owner)], "total": 1});
        serde_json::from_value::<NotificationListWire>(valid)
            .unwrap()
            .validate(owner, &one_row_query)
            .unwrap();

        let mut wrong_owner = notification("0x2222222222222222222222222222222222222222");
        wrong_owner.as_object_mut().unwrap().remove("action_url");
        let malformed = serde_json::json!({"items": [wrong_owner], "total": 1});
        let payload = serde_json::from_value::<NotificationListWire>(malformed).unwrap();
        assert!(payload.validate(owner, &one_row_query).is_err());

        let extra = serde_json::json!({
            "items": [notification(owner)],
            "total": 1,
            "sample": true
        });
        assert!(serde_json::from_value::<NotificationListWire>(extra).is_err());

        let mut unknown_row = notification(owner);
        unknown_row
            .as_object_mut()
            .unwrap()
            .insert("sample".to_string(), serde_json::Value::Bool(true));
        assert!(
            serde_json::from_value::<NotificationListWire>(serde_json::json!({
                "items": [unknown_row],
                "total": 1
            }))
            .is_err()
        );

        for total in [-1, 0] {
            let payload = serde_json::from_value::<NotificationListWire>(serde_json::json!({
                "items": [notification(owner)],
                "total": total
            }))
            .unwrap();
            assert!(payload.validate(owner, &one_row_query).is_err());
        }
    }

    #[test]
    fn unfiltered_page_cardinality_must_agree_with_total() {
        let owner = "0x1111111111111111111111111111111111111111";
        let contradictory = serde_json::from_value::<NotificationListWire>(serde_json::json!({
            "items": [],
            "total": 1
        }))
        .unwrap();
        assert!(contradictory
            .validate(owner, &NotificationListQuery::default())
            .is_err());

        let past_end = NotificationListQuery {
            limit: Some(1),
            offset: Some(2),
            status: None,
        };
        assert!(contradictory.validate(owner, &past_end).is_ok());

        // The current service reports an unfiltered total for a filtered row
        // query. Until that service contract is repaired, do not pretend the
        // filtered page can be checked against the global total.
        let filtered = NotificationListQuery {
            limit: Some(1),
            offset: None,
            status: Some("sent".to_string()),
        };
        assert!(contradictory.validate(owner, &filtered).is_ok());
    }

    #[test]
    fn unread_wire_shape_is_exact_and_never_defaults_to_zero() {
        assert_eq!(
            serde_json::from_value::<NotificationUnreadCount>(serde_json::json!({"count": 7}))
                .unwrap()
                .count,
            7
        );
        for malformed in [
            serde_json::json!({}),
            serde_json::json!({"count": 0, "items": []}),
            serde_json::json!({"count": -1}),
            serde_json::json!({"count": 1.5}),
            serde_json::json!({"count": "0"}),
            serde_json::Value::Null,
        ] {
            assert!(serde_json::from_value::<NotificationUnreadCount>(malformed).is_err());
        }
    }
}

#[cfg(test)]
mod auth_session_tests {
    use super::*;
    use axum::{
        body::Body,
        http::{header, HeaderMap, HeaderValue},
        routing::{delete, get, post},
        Router,
    };
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use epsx_bff::{
        cookies::{
            LEGACY_ACCESS_COOKIE, LEGACY_LOCAL_ACCESS_COOKIE, LEGACY_LOCAL_REFRESH_COOKIE,
            LOCAL_ACCESS_COOKIE, LOCAL_REFRESH_COOKIE,
        },
        refresh_outcome::{
            REFRESH_OUTCOME_HEADER, REFRESH_OUTCOME_NOT_ROTATED, REFRESH_OUTCOME_ROTATED,
            SESSION_STATE_CLEARED, SESSION_STATE_HEADER, SESSION_STATE_PRESERVED,
            SESSION_STATE_ROTATED,
        },
        session::{AccessTokenClaims, Jwks, JwksVerifierConfig, RsaJwk, JWKS_PATH},
    };
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use rand::thread_rng;
    use rsa::{pkcs8::EncodePrivateKey, traits::PublicKeyParts, RsaPrivateKey};
    use serde_json::{json, Value};
    use std::{sync::Arc, time::Duration};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    const TEST_ISSUER: &str = "https://issuer.test";
    const TEST_WALLET: &str = "0x1111111111111111111111111111111111111111";

    #[test]
    fn failed_cookie_clear_never_attests_cleared_session_state() {
        let response = match try_clear_session_response(
            StatusCode::UNAUTHORIZED,
            "forced_cookie_failure",
            |_| false,
        ) {
            Ok(_) => panic!("forced cookie failure unexpectedly succeeded"),
            Err(response) => response,
        };
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(response.headers().get(SESSION_STATE_HEADER).is_none());
    }

    fn upstream_refresh_response(mut response: Response, outcome: &'static str) -> Response {
        response
            .headers_mut()
            .insert(REFRESH_OUTCOME_HEADER, HeaderValue::from_static(outcome));
        response
    }

    fn assert_session_state(response: &Response, expected: &'static str) {
        assert_eq!(
            response
                .headers()
                .get(SESSION_STATE_HEADER)
                .and_then(|value| value.to_str().ok()),
            Some(expected)
        );
    }

    fn assert_private_notification_response(response: &Response) {
        assert_eq!(
            response.headers().get(header::CACHE_CONTROL),
            Some(&HeaderValue::from_static("private, no-store"))
        );
        assert_eq!(
            response.headers().get(header::VARY),
            Some(&HeaderValue::from_static("Cookie, Authorization"))
        );
    }

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

    async fn spawn_chunked_body(chunks: Vec<Vec<u8>>) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0_u8; 2048];
            let _ = stream.read(&mut request).await.unwrap();
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n",
                )
                .await
                .unwrap();
            for chunk in chunks {
                stream
                    .write_all(format!("{:X}\r\n", chunk.len()).as_bytes())
                    .await
                    .unwrap();
                stream.write_all(&chunk).await.unwrap();
                stream.write_all(b"\r\n").await.unwrap();
            }
            stream.write_all(b"0\r\n\r\n").await.unwrap();
        });
        format!("http://{address}")
    }

    async fn spawn_stalled_chunked_body(chunk: Vec<u8>, stall: Duration) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0_u8; 2048];
            let _ = stream.read(&mut request).await.unwrap();
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n",
                )
                .await
                .unwrap();
            stream
                .write_all(format!("{:X}\r\n", chunk.len()).as_bytes())
                .await
                .unwrap();
            stream.write_all(&chunk).await.unwrap();
            stream.write_all(b"\r\n").await.unwrap();
            stream.flush().await.unwrap();
            tokio::time::sleep(stall).await;
        });
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
            5,
            "expected scoped pair plus ambiguous local and legacy access clears"
        );
        for name in [
            LOCAL_ACCESS_COOKIE,
            LOCAL_REFRESH_COOKIE,
            LEGACY_LOCAL_ACCESS_COOKIE,
            LEGACY_LOCAL_REFRESH_COOKIE,
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

    fn notification_payload(owner: &str) -> Value {
        json!({
            "items": [{
                "id": "notification-1",
                "user_id": owner,
                "channel": "in_app",
                "recipient": owner,
                "template_id": null,
                "subject": "Subject",
                "body": "Body",
                "data": null,
                "status": "sent",
                "error": null,
                "sent_at": "2026-07-22T00:00:00Z",
                "created_at": "2026-07-22T00:00:00Z",
                "read_at": null,
                "title": "Title",
                "notification_type": "system",
                "priority": "normal",
                "action_url": null
            }],
            "total": 1
        })
    }

    async fn response_json(response: Response) -> Value {
        serde_json::from_slice(
            &axum::body::to_bytes(response.into_body(), 32 * 1024)
                .await
                .unwrap(),
        )
        .unwrap()
    }

    fn notification_client(base_url: &str, timeout: Duration) -> ServiceClient {
        ServiceClient::new(epsx_client::ClientConfig {
            base_url: base_url.to_string(),
            timeout,
        })
    }

    fn test_notification_request_id() -> NotificationRequestId {
        NotificationRequestId("11111111-1111-4111-8111-111111111111".to_string())
    }

    async fn load_unread_response(
        status: StatusCode,
        content_type: Option<&str>,
        body: impl Into<Vec<u8>>,
    ) -> NotificationUnreadLoadOutcome {
        let content_type = content_type.map(str::to_string);
        let body = body.into();
        let router = Router::new().route(
            "/api/v1/notification/unread-count",
            get(move || {
                let content_type = content_type.clone();
                let body = body.clone();
                async move {
                    let mut response = Response::builder().status(status);
                    if let Some(content_type) = content_type {
                        response = response.header(header::CONTENT_TYPE, content_type);
                    }
                    response.body(Body::from(body)).unwrap()
                }
            }),
        );
        let base_url = spawn_mock(router).await;
        load_notification_unread_count(
            &notification_client(&base_url, Duration::from_secs(1)),
            "verified-bearer",
            &test_notification_request_id(),
        )
        .await
    }

    #[tokio::test]
    async fn notification_unread_loader_accepts_only_exact_200_and_json_content_type() {
        for content_type in [
            "application/json",
            "Application/JSON",
            "application/json; charset=utf-8",
            "APPLICATION/JSON; CHARSET=UTF-8",
            "application/json; charset=\"utf-8\"",
        ] {
            assert_eq!(
                load_unread_response(
                    StatusCode::OK,
                    Some(content_type),
                    br#"{"count":7}"#.to_vec(),
                )
                .await,
                NotificationUnreadLoadOutcome::Ready(7),
                "valid content type rejected: {content_type}",
            );
        }

        for content_type in [
            None,
            Some("text/json"),
            Some("application/problem+json"),
            Some("application/json; charset=iso-8859-1"),
            Some("application/json; foo=bar"),
            Some("application/json; charset=utf-8; charset=utf-8"),
            Some("application/json;"),
            Some("application/json; charset"),
            Some("application/json; charset=\"utf-8"),
        ] {
            assert_eq!(
                load_unread_response(StatusCode::OK, content_type, br#"{"count":7}"#.to_vec(),)
                    .await,
                NotificationUnreadLoadOutcome::Malformed,
                "invalid content type accepted: {content_type:?}",
            );
        }

        let mut duplicate_content_type = HeaderMap::new();
        duplicate_content_type.append(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        duplicate_content_type.append(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        assert!(!valid_notification_json_content_type(
            &duplicate_content_type
        ));

        for status in [
            StatusCode::REQUEST_TIMEOUT,
            StatusCode::TOO_EARLY,
            StatusCode::TOO_MANY_REQUESTS,
            StatusCode::INTERNAL_SERVER_ERROR,
            StatusCode::SERVICE_UNAVAILABLE,
        ] {
            assert_eq!(
                load_unread_response(status, Some("application/json"), br#"{"count":7}"#.to_vec(),)
                    .await,
                NotificationUnreadLoadOutcome::DependencyUnavailable,
                "dependency status misclassified: {status}",
            );
        }

        for status in [
            StatusCode::CREATED,
            StatusCode::ACCEPTED,
            StatusCode::NO_CONTENT,
            StatusCode::PARTIAL_CONTENT,
            StatusCode::BAD_REQUEST,
            StatusCode::UNAUTHORIZED,
            StatusCode::FORBIDDEN,
            StatusCode::NOT_FOUND,
            StatusCode::CONFLICT,
        ] {
            assert_eq!(
                load_unread_response(status, Some("application/json"), br#"{"count":7}"#.to_vec(),)
                    .await,
                NotificationUnreadLoadOutcome::UpstreamFailed,
                "upstream status misclassified: {status}",
            );
        }
    }

    #[tokio::test]
    async fn notification_unread_loader_enforces_exact_js_safe_counts() {
        for (body, expected) in [
            (
                br#"{"count":0}"#.as_slice(),
                NotificationUnreadLoadOutcome::Ready(0),
            ),
            (
                br#"{"count":9007199254740991}"#.as_slice(),
                NotificationUnreadLoadOutcome::Ready(NOTIFICATION_UNREAD_JS_SAFE_MAX),
            ),
        ] {
            assert_eq!(
                load_unread_response(StatusCode::OK, Some("application/json"), body.to_vec()).await,
                expected,
            );
        }

        for body in [
            br#"{}"#.as_slice(),
            br#"null"#.as_slice(),
            br#"[]"#.as_slice(),
            br#"{"count":-1}"#.as_slice(),
            br#"{"count":1.5}"#.as_slice(),
            br#"{"count":"1"}"#.as_slice(),
            br#"{"count":0,"extra":true}"#.as_slice(),
            br#"{"count":1,"count":2}"#.as_slice(),
            br#"{"count":9007199254740992}"#.as_slice(),
            br#"{"count":18446744073709551616}"#.as_slice(),
            br#"{"count":1}{"count":2}"#.as_slice(),
        ] {
            assert_eq!(
                load_unread_response(StatusCode::OK, Some("application/json"), body.to_vec()).await,
                NotificationUnreadLoadOutcome::Malformed,
                "malformed unread payload accepted: {}",
                String::from_utf8_lossy(body),
            );
        }
    }

    #[tokio::test]
    async fn notification_unread_loader_fails_closed_on_connect_stall_overflow_and_redirect() {
        assert_eq!(
            load_notification_unread_count(
                &notification_client(&unused_base_url().await, Duration::from_millis(50)),
                "verified-bearer",
                &test_notification_request_id(),
            )
            .await,
            NotificationUnreadLoadOutcome::DependencyUnavailable,
        );

        let stalled_base_url =
            spawn_stalled_chunked_body(br#"{"count":"#.to_vec(), Duration::from_millis(200)).await;
        assert_eq!(
            load_notification_unread_count(
                &notification_client(&stalled_base_url, Duration::from_millis(50)),
                "verified-bearer",
                &test_notification_request_id(),
            )
            .await,
            NotificationUnreadLoadOutcome::DependencyUnavailable,
        );

        let oversized_base_url =
            spawn_chunked_body(vec![vec![b'x'; NOTIFICATION_UNREAD_BODY_MAX], vec![b'x']]).await;
        assert_eq!(
            load_notification_unread_count(
                &notification_client(&oversized_base_url, Duration::from_secs(1)),
                "verified-bearer",
                &test_notification_request_id(),
            )
            .await,
            NotificationUnreadLoadOutcome::Malformed,
        );

        let followed = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let followed_target = followed.clone();
        let redirect_router = Router::new()
            .route(
                "/api/v1/notification/unread-count",
                get(|| async { axum::response::Redirect::temporary("/redirect-target") }),
            )
            .route(
                "/redirect-target",
                get(move || {
                    let followed_target = followed_target.clone();
                    async move {
                        followed_target.store(true, std::sync::atomic::Ordering::SeqCst);
                        Json(json!({"count": 7}))
                    }
                }),
            );
        let redirect_base_url = spawn_mock(redirect_router).await;
        assert_eq!(
            load_notification_unread_count(
                &notification_client(&redirect_base_url, Duration::from_secs(1)),
                "verified-bearer",
                &test_notification_request_id(),
            )
            .await,
            NotificationUnreadLoadOutcome::UpstreamFailed,
        );
        assert!(!followed.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[tokio::test]
    async fn notification_unread_response_mapping_is_private_exact_and_redacted() {
        for (outcome, expected_status, expected_body) in [
            (
                NotificationUnreadLoadOutcome::Ready(7),
                StatusCode::OK,
                json!({"count": 7}),
            ),
            (
                NotificationUnreadLoadOutcome::DependencyUnavailable,
                StatusCode::SERVICE_UNAVAILABLE,
                json!({"success": false, "error": "notification_upstream_unavailable"}),
            ),
            (
                NotificationUnreadLoadOutcome::UpstreamFailed,
                StatusCode::BAD_GATEWAY,
                json!({"success": false, "error": "notification_upstream_failed"}),
            ),
            (
                NotificationUnreadLoadOutcome::Malformed,
                StatusCode::BAD_GATEWAY,
                json!({"success": false, "error": "malformed_notification_response"}),
            ),
        ] {
            let response =
                private_notification_response(notification_unread_load_response(outcome));
            assert_eq!(response.status(), expected_status);
            assert_private_notification_response(&response);
            let body = response_json(response).await;
            assert_eq!(body, expected_body);
            assert!(!body.to_string().contains("http"));
            assert!(!body.to_string().contains("verified-bearer"));
        }
    }

    async fn load_notification_payload(payload: Value) -> NotificationListLoadOutcome {
        let router = Router::new().route(
            "/api/v1/notification/list",
            get(move || {
                let payload = payload.clone();
                async move { Json(payload) }
            }),
        );
        let base_url = spawn_mock(router).await;
        load_owner_notifications(
            &notification_client(&base_url, Duration::from_secs(1)),
            "verified-bearer",
            TEST_WALLET,
            &NotificationListQuery::default(),
            &NotificationRequestId("11111111-1111-4111-8111-111111111111".to_string()),
        )
        .await
    }

    #[tokio::test]
    async fn shared_notification_loader_classifies_empty_and_strict_contract_failures() {
        let ready = notification_payload(TEST_WALLET);
        assert_eq!(
            load_notification_payload(ready.clone()).await,
            NotificationListLoadOutcome::Ready(ready)
        );

        let empty = json!({"items": [], "total": 0});
        assert_eq!(
            load_notification_payload(empty.clone()).await,
            NotificationListLoadOutcome::Empty(empty)
        );

        assert_eq!(
            load_notification_payload(json!({"items": [], "total": 1})).await,
            NotificationListLoadOutcome::Malformed
        );

        assert_eq!(
            load_owner_notifications(
                &notification_client(&unused_base_url().await, Duration::from_millis(50)),
                "verified-bearer",
                TEST_WALLET,
                &NotificationListQuery::default(),
                &NotificationRequestId("11111111-1111-4111-8111-111111111111".to_string(),),
            )
            .await,
            NotificationListLoadOutcome::Unavailable(NotificationListUnavailable::Dependency)
        );

        let malformed_router = Router::new().route(
            "/api/v1/notification/list",
            get(|| async {
                Response::builder()
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from("{"))
                    .unwrap()
            }),
        );
        let malformed_base_url = spawn_mock(malformed_router).await;
        assert_eq!(
            load_owner_notifications(
                &notification_client(&malformed_base_url, Duration::from_secs(1)),
                "verified-bearer",
                TEST_WALLET,
                &NotificationListQuery::default(),
                &NotificationRequestId("11111111-1111-4111-8111-111111111111".to_string(),),
            )
            .await,
            NotificationListLoadOutcome::Malformed
        );

        assert_eq!(
            load_notification_payload(notification_payload(
                "0x2222222222222222222222222222222222222222"
            ))
            .await,
            NotificationListLoadOutcome::Malformed
        );

        let mut unknown = notification_payload(TEST_WALLET);
        unknown["items"][0]["unknown"] = json!(true);
        assert_eq!(
            load_notification_payload(unknown).await,
            NotificationListLoadOutcome::Malformed
        );

        let mut negative_total = notification_payload(TEST_WALLET);
        negative_total["total"] = json!(-1);
        assert_eq!(
            load_notification_payload(negative_total).await,
            NotificationListLoadOutcome::Malformed
        );

        let mut impossible_total = notification_payload(TEST_WALLET);
        impossible_total["total"] = json!(0);
        assert_eq!(
            load_notification_payload(impossible_total).await,
            NotificationListLoadOutcome::Malformed
        );
    }

    #[tokio::test]
    async fn shared_notification_loader_fails_closed_on_timeout_redirect_and_chunked_overflow() {
        let timeout_router = Router::new().route(
            "/api/v1/notification/list",
            get(|| async {
                tokio::time::sleep(Duration::from_millis(200)).await;
                Json(json!({"items": [], "total": 0}))
            }),
        );
        let timeout_base_url = spawn_mock(timeout_router).await;
        let request_id = NotificationRequestId("11111111-1111-4111-8111-111111111111".to_string());
        assert_eq!(
            load_owner_notifications(
                &notification_client(&timeout_base_url, Duration::from_millis(20)),
                "verified-bearer",
                TEST_WALLET,
                &NotificationListQuery::default(),
                &request_id,
            )
            .await,
            NotificationListLoadOutcome::Unavailable(NotificationListUnavailable::Dependency)
        );

        let stalled_base_url =
            spawn_stalled_chunked_body(br#"{"items":[]"#.to_vec(), Duration::from_millis(200))
                .await;
        assert_eq!(
            load_owner_notifications(
                &notification_client(&stalled_base_url, Duration::from_millis(50)),
                "verified-bearer",
                TEST_WALLET,
                &NotificationListQuery::default(),
                &request_id,
            )
            .await,
            NotificationListLoadOutcome::Unavailable(NotificationListUnavailable::Dependency)
        );

        let followed = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let followed_target = followed.clone();
        let redirect_router = Router::new()
            .route(
                "/api/v1/notification/list",
                get(|| async { axum::response::Redirect::temporary("/redirect-target") }),
            )
            .route(
                "/redirect-target",
                get(move || {
                    let followed_target = followed_target.clone();
                    async move {
                        followed_target.store(true, std::sync::atomic::Ordering::SeqCst);
                        Json(notification_payload(TEST_WALLET))
                    }
                }),
            );
        let redirect_base_url = spawn_mock(redirect_router).await;
        assert_eq!(
            load_owner_notifications(
                &notification_client(&redirect_base_url, Duration::from_secs(1)),
                "verified-bearer",
                TEST_WALLET,
                &NotificationListQuery::default(),
                &request_id,
            )
            .await,
            NotificationListLoadOutcome::Unavailable(NotificationListUnavailable::UpstreamFailed)
        );
        assert!(!followed.load(std::sync::atomic::Ordering::SeqCst));

        let oversized_base_url =
            spawn_chunked_body(vec![vec![b'x'; NOTIFICATION_LIST_BODY_MAX], vec![b'x']]).await;
        assert_eq!(
            load_owner_notifications(
                &notification_client(&oversized_base_url, Duration::from_secs(1)),
                "verified-bearer",
                TEST_WALLET,
                &NotificationListQuery::default(),
                &request_id,
            )
            .await,
            NotificationListLoadOutcome::Malformed
        );
    }

    #[tokio::test]
    async fn notification_reads_forward_only_verified_bearer_and_safe_query() {
        let key = TestKey::generate();
        let access_token = key.access_token(&[]);
        let expected_token = access_token.clone();
        let unread_expected_token = access_token.clone();
        let jwks = Jwks {
            keys: vec![key.jwk.clone()],
        };
        let observations = Arc::new(std::sync::Mutex::new(Vec::<Value>::new()));
        let list_observations = observations.clone();
        let unread_observations = observations.clone();
        let router = Router::new()
            .route(
                JWKS_PATH,
                get(move || {
                    let jwks = jwks.clone();
                    async move { Json(jwks) }
                }),
            )
            .route(
                "/api/v1/notification/list",
                get(move |RawQuery(query): RawQuery, headers: HeaderMap| {
                    let observations = list_observations.clone();
                    let expected_token = expected_token.clone();
                    async move {
                        observations.lock().unwrap().push(json!({
                            "endpoint": "list",
                            "query": query,
                            "authorization": headers
                                .get(header::AUTHORIZATION)
                                .and_then(|value| value.to_str().ok()),
                            "accept": headers
                                .get(header::ACCEPT)
                                .and_then(|value| value.to_str().ok()),
                            "x_user_id": headers.get("x-user-id").and_then(|value| value.to_str().ok()),
                            "x_user_address": headers
                                .get("x-user-address")
                                .and_then(|value| value.to_str().ok()),
                            "x_permissions": headers
                                .get("x-permissions")
                                .and_then(|value| value.to_str().ok()),
                            "cookie": headers
                                .get(header::COOKIE)
                                .and_then(|value| value.to_str().ok()),
                            "request_id": headers
                                .get("x-request-id")
                                .and_then(|value| value.to_str().ok()),
                        }));
                        if headers
                            .get(header::AUTHORIZATION)
                            .and_then(|value| value.to_str().ok())
                            != Some(format!("Bearer {expected_token}").as_str())
                        {
                            return StatusCode::UNAUTHORIZED.into_response();
                        }
                        Json(notification_payload(TEST_WALLET)).into_response()
                    }
                }),
            )
            .route(
                "/api/v1/notification/unread-count",
                get(move |headers: HeaderMap| {
                    let observations = unread_observations.clone();
                    let expected_token = unread_expected_token.clone();
                    async move {
                        observations.lock().unwrap().push(json!({
                            "endpoint": "unread",
                            "authorization": headers
                                .get(header::AUTHORIZATION)
                                .and_then(|value| value.to_str().ok()),
                            "accept": headers
                                .get(header::ACCEPT)
                                .and_then(|value| value.to_str().ok()),
                            "x_user_id": headers
                                .get("x-user-id")
                                .and_then(|value| value.to_str().ok()),
                            "x_user_address": headers
                                .get("x-user-address")
                                .and_then(|value| value.to_str().ok()),
                            "x_permissions": headers
                                .get("x-permissions")
                                .and_then(|value| value.to_str().ok()),
                            "cookie": headers
                                .get(header::COOKIE)
                                .and_then(|value| value.to_str().ok()),
                            "request_id": headers
                                .get("x-request-id")
                                .and_then(|value| value.to_str().ok()),
                        }));
                        if headers
                            .get(header::AUTHORIZATION)
                            .and_then(|value| value.to_str().ok())
                            != Some(format!("Bearer {expected_token}").as_str())
                        {
                            return StatusCode::UNAUTHORIZED.into_response();
                        }
                        Json(json!({"count": 7})).into_response()
                    }
                }),
            );
        let base_url = spawn_mock(router).await;
        let app_state = state(&base_url);
        let mut headers = request_headers(&format!("epsx.frontend.access_token={access_token}"));
        headers.insert(
            "x-request-id",
            HeaderValue::from_static("22222222-2222-4222-8222-222222222222"),
        );
        headers.insert("x-user-id", HeaderValue::from_static("browser-user"));
        headers.insert(
            "x-user-address",
            HeaderValue::from_static("0x2222222222222222222222222222222222222222"),
        );
        headers.insert(
            "x-permissions",
            HeaderValue::from_static("browser:unverified"),
        );

        let list = notifications_api(
            State(app_state.clone()),
            headers.clone(),
            RawQuery(Some("status=sent&offset=2&limit=25".to_string())),
        )
        .await;
        assert_eq!(list.status(), StatusCode::OK);
        assert_private_notification_response(&list);
        assert_eq!(response_json(list).await["total"], 1);

        let unread = notification_unread_count(State(app_state.clone()), headers.clone()).await;
        assert_eq!(unread.status(), StatusCode::OK);
        assert_private_notification_response(&unread);
        assert_eq!(response_json(unread).await, json!({"count": 7}));

        headers.insert("x-request-id", HeaderValue::from_static("not-a-request-id"));
        let unread_with_generated_request_id =
            notification_unread_count(State(app_state), headers).await;
        assert_eq!(unread_with_generated_request_id.status(), StatusCode::OK);
        assert_private_notification_response(&unread_with_generated_request_id);
        assert_eq!(
            response_json(unread_with_generated_request_id).await,
            json!({"count": 7})
        );

        let observations = observations.lock().unwrap();
        assert_eq!(observations.len(), 3);
        assert_eq!(observations[0]["endpoint"], "list");
        assert_eq!(observations[0]["query"], "limit=25&offset=2&status=sent");
        assert_eq!(
            observations[0]["authorization"],
            format!("Bearer {access_token}")
        );
        assert!(observations[0]["x_user_id"].is_null());
        assert!(observations[0]["x_user_address"].is_null());
        assert!(observations[0]["x_permissions"].is_null());
        assert!(observations[0]["cookie"].is_null());
        assert_eq!(
            observations[0]["request_id"],
            "22222222-2222-4222-8222-222222222222"
        );
        assert!(!observations[0]["query"]
            .as_str()
            .unwrap()
            .contains("user_id"));

        for unread_observation in &observations[1..] {
            assert_eq!(unread_observation["endpoint"], "unread");
            assert_eq!(
                unread_observation["authorization"],
                format!("Bearer {access_token}")
            );
            assert_eq!(unread_observation["accept"], "application/json");
            assert!(unread_observation["x_user_id"].is_null());
            assert!(unread_observation["x_user_address"].is_null());
            assert!(unread_observation["x_permissions"].is_null());
            assert!(unread_observation["cookie"].is_null());
        }
        assert_eq!(
            observations[1]["request_id"],
            "22222222-2222-4222-8222-222222222222"
        );
        let generated_request_id = observations[2]["request_id"].as_str().unwrap();
        assert_ne!(generated_request_id, "not-a-request-id");
        assert!(uuid::Uuid::parse_str(generated_request_id).is_ok());
    }

    #[tokio::test]
    async fn notification_reads_distinguish_auth_malformed_and_upstream_failures() {
        let invalid_query = notifications_api(
            State(state(&unused_base_url().await)),
            HeaderMap::new(),
            RawQuery(Some("limit=0".to_string())),
        )
        .await;
        assert_eq!(invalid_query.status(), StatusCode::BAD_REQUEST);
        assert_private_notification_response(&invalid_query);
        assert_eq!(
            response_json(invalid_query).await["error"],
            "invalid_notification_query"
        );

        let missing =
            notification_unread_count(State(state(&unused_base_url().await)), HeaderMap::new())
                .await;
        assert_eq!(missing.status(), StatusCode::UNAUTHORIZED);
        assert_private_notification_response(&missing);
        assert_eq!(
            response_json(missing).await["error"],
            "invalid_access_token"
        );

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
            .route(
                "/api/v1/notification/list",
                get(|RawQuery(query): RawQuery| async move {
                    if query
                        .as_deref()
                        .is_some_and(|query| query.contains("status=failed"))
                    {
                        StatusCode::SERVICE_UNAVAILABLE.into_response()
                    } else {
                        Json(notification_payload(
                            "0x2222222222222222222222222222222222222222",
                        ))
                        .into_response()
                    }
                }),
            )
            .route(
                "/api/v1/notification/unread-count",
                get(|| async { Json(json!({"count": 0, "items": []})) }),
            );
        let base_url = spawn_mock(router).await;
        let headers = request_headers(&format!("epsx.frontend.access_token={access_token}"));

        let wrong_owner =
            notifications_api(State(state(&base_url)), headers.clone(), RawQuery(None)).await;
        assert_eq!(wrong_owner.status(), StatusCode::BAD_GATEWAY);
        assert_private_notification_response(&wrong_owner);
        assert_eq!(
            response_json(wrong_owner).await["error"],
            "malformed_notification_response"
        );

        let upstream = notifications_api(
            State(state(&base_url)),
            headers.clone(),
            RawQuery(Some("status=failed".to_string())),
        )
        .await;
        assert_eq!(upstream.status(), StatusCode::BAD_GATEWAY);
        assert_private_notification_response(&upstream);
        assert_eq!(
            response_json(upstream).await["error"],
            "notification_upstream_failed"
        );

        let malformed = notification_unread_count(State(state(&base_url)), headers).await;
        assert_eq!(malformed.status(), StatusCode::BAD_GATEWAY);
        assert_private_notification_response(&malformed);
        assert_eq!(
            response_json(malformed).await["error"],
            "malformed_notification_response"
        );
    }

    #[tokio::test]
    async fn notification_body_reader_caps_chunked_responses_without_content_length() {
        for limit in [NOTIFICATION_LIST_BODY_MAX, NOTIFICATION_UNREAD_BODY_MAX] {
            let exact_url = spawn_chunked_body(vec![vec![b'a'; limit]]).await;
            let exact = reqwest::get(exact_url).await.unwrap();
            assert_eq!(
                read_notification_body_limited(exact, limit)
                    .await
                    .unwrap()
                    .len(),
                limit
            );

            let oversized_url = spawn_chunked_body(vec![vec![b'a'; limit], vec![b'b']]).await;
            let oversized = reqwest::get(oversized_url).await.unwrap();
            assert_eq!(
                read_notification_body_limited(oversized, limit).await,
                Err(NotificationBodyReadError::TooLarge),
                "chunked response must fail above {limit} bytes",
            );
        }
    }

    #[tokio::test]
    async fn notification_handlers_map_oversized_success_bodies_to_safe_502() {
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
            .route(
                "/api/v1/notification/list",
                get(|| async {
                    Response::builder()
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(vec![b'x'; NOTIFICATION_LIST_BODY_MAX + 1]))
                        .unwrap()
                }),
            )
            .route(
                "/api/v1/notification/unread-count",
                get(|| async {
                    Response::builder()
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(vec![b'x'; NOTIFICATION_UNREAD_BODY_MAX + 1]))
                        .unwrap()
                }),
            );
        let base_url = spawn_mock(router).await;
        let headers = request_headers(&format!("epsx.frontend.access_token={access_token}"));

        let list =
            notifications_api(State(state(&base_url)), headers.clone(), RawQuery(None)).await;
        assert_eq!(list.status(), StatusCode::BAD_GATEWAY);
        assert_private_notification_response(&list);
        assert_eq!(
            response_json(list).await["error"],
            "malformed_notification_response"
        );

        let unread = notification_unread_count(State(state(&base_url)), headers).await;
        assert_eq!(unread.status(), StatusCode::BAD_GATEWAY);
        assert_private_notification_response(&unread);
        assert_eq!(
            response_json(unread).await["error"],
            "malformed_notification_response"
        );
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
        assert_eq!(
            response.headers().get(header::CACHE_CONTROL),
            Some(&HeaderValue::from_static("private, no-store"))
        );
        let cookies = response_cookies(&response);
        assert_eq!(cookies.len(), 5);
        assert!(cookies
            .iter()
            .any(|cookie| cookie.starts_with("epsx.frontend.access_token=")));
        assert!(cookies
            .iter()
            .any(|cookie| cookie.starts_with("epsx.frontend.refresh_token=")));
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
                        upstream_refresh_response(
                            Json(payload).into_response(),
                            REFRESH_OUTCOME_ROTATED,
                        )
                    } else {
                        StatusCode::BAD_REQUEST.into_response()
                    }
                }
            }),
        );
        let base_url = spawn_mock(router).await;
        let response = refresh_token(
            State(state(&base_url)),
            request_headers(
                "epsx.frontend.access_token=old-access; epsx.frontend.refresh_token=browser-refresh",
            ),
        )
        .await;

        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
        assert_session_cleared(&response);
        assert_session_state(&response, SESSION_STATE_CLEARED);

        for (upstream_status, outcome, expected_status, expected_state, clears) in [
            (
                StatusCode::BAD_REQUEST,
                Some(REFRESH_OUTCOME_NOT_ROTATED),
                StatusCode::BAD_REQUEST,
                SESSION_STATE_PRESERVED,
                false,
            ),
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(REFRESH_OUTCOME_NOT_ROTATED),
                StatusCode::INTERNAL_SERVER_ERROR,
                SESSION_STATE_PRESERVED,
                false,
            ),
            (
                StatusCode::SERVICE_UNAVAILABLE,
                None,
                StatusCode::BAD_GATEWAY,
                SESSION_STATE_CLEARED,
                true,
            ),
            (
                StatusCode::REQUEST_TIMEOUT,
                Some(REFRESH_OUTCOME_NOT_ROTATED),
                StatusCode::BAD_GATEWAY,
                SESSION_STATE_CLEARED,
                true,
            ),
            (
                StatusCode::TOO_MANY_REQUESTS,
                None,
                StatusCode::BAD_GATEWAY,
                SESSION_STATE_CLEARED,
                true,
            ),
        ] {
            let retryable_base = spawn_mock(Router::new().route(
                REFRESH_PATH,
                post(move || async move {
                    let response = upstream_status.into_response();
                    match outcome {
                        Some(outcome) => upstream_refresh_response(response, outcome),
                        None => response,
                    }
                }),
            ))
            .await;
            let retryable = refresh_token(
                State(state(&retryable_base)),
                request_headers("epsx.frontend.refresh_token=browser-refresh"),
            )
            .await;
            assert_eq!(retryable.status(), expected_status);
            assert_session_state(&retryable, expected_state);
            assert_eq!(!response_cookies(&retryable).is_empty(), clears);
        }
    }

    #[tokio::test]
    async fn refresh_transport_failure_and_redirect_fail_closed_without_replay() {
        let unavailable = refresh_token(
            State(state(&unused_base_url().await)),
            request_headers("epsx.frontend.refresh_token=browser-refresh"),
        )
        .await;
        assert_eq!(unavailable.status(), StatusCode::BAD_GATEWAY);
        assert_session_cleared(&unavailable);
        assert_session_state(&unavailable, SESSION_STATE_CLEARED);

        let redirect_target = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let target_url = format!("http://{}/capture", redirect_target.local_addr().unwrap());
        let redirect_base = spawn_mock(Router::new().route(
            REFRESH_PATH,
            post(move || {
                let target_url = target_url.clone();
                async move {
                    (
                        StatusCode::TEMPORARY_REDIRECT,
                        [(header::LOCATION, target_url)],
                    )
                }
            }),
        ))
        .await;
        let redirected = refresh_token(
            State(state(&redirect_base)),
            request_headers("epsx.frontend.refresh_token=browser-refresh"),
        )
        .await;
        assert_eq!(redirected.status(), StatusCode::BAD_GATEWAY);
        assert_session_cleared(&redirected);
        assert_session_state(&redirected, SESSION_STATE_CLEARED);
        assert!(
            tokio::time::timeout(Duration::from_millis(100), redirect_target.accept())
                .await
                .is_err(),
            "credential-bearing refresh followed an upstream redirect"
        );
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
                            upstream_refresh_response(
                                Json(payload).into_response(),
                                REFRESH_OUTCOME_ROTATED,
                            )
                        } else {
                            StatusCode::BAD_REQUEST.into_response()
                        }
                    }
                }),
            );
        let base_url = spawn_mock(router).await;
        let response = refresh_token(
            State(state(&base_url)),
            request_headers("epsx.frontend.refresh_token=browser-refresh"),
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_session_state(&response, SESSION_STATE_ROTATED);
        assert_eq!(
            response.headers().get(header::CACHE_CONTROL),
            Some(&HeaderValue::from_static("private, no-store"))
        );
        let cookies = response_cookies(&response);
        assert_eq!(cookies.len(), 5);
        assert!(cookies
            .iter()
            .any(|cookie| cookie.starts_with("epsx.frontend.access_token=")));
        assert!(cookies
            .iter()
            .any(|cookie| cookie.starts_with("epsx.frontend.refresh_token=rotated-refresh")));
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
            request_headers(&format!("epsx.frontend.access_token={access_token}")),
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
            request_headers("epsx.frontend.access_token=not-a-jwt"),
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
            request_headers(&format!("epsx.frontend.access_token={access_token}")),
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
            request_headers("epsx.frontend.refresh_token=browser-refresh"),
        )
        .await;
        assert_eq!(success.status(), StatusCode::OK);
        assert_session_cleared(&success);
        assert_session_state(&success, SESSION_STATE_CLEARED);

        let unavailable_base = unused_base_url().await;
        let failure = logout(
            State(state(&unavailable_base)),
            request_headers("epsx.frontend.refresh_token=browser-refresh"),
        )
        .await;
        assert_eq!(failure.status(), StatusCode::BAD_GATEWAY);
        assert_session_cleared(&failure);
        assert_session_state(&failure, SESSION_STATE_CLEARED);
    }
}

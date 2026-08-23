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
    body::Body,
    extract::{Path as AxPath, Query, RawQuery, Request, State},
    http::{header, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Redirect, Response},
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
use futures::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
const NEWS_LIST_PATH: &str = "/api/public/news?page=1&limit=100";
const NEWS_DETAIL_PATH: &str = "/api/public/news";
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
    // The backend public-news DTO includes lifecycle metadata that the public
    // page does not display. Declare it explicitly so the strict decoder can
    // accept the authoritative response without relaxing unknown-field checks.
    #[serde(default, rename = "created_at")]
    _created_at: Option<String>,
    #[serde(default, rename = "updated_at")]
    _updated_at: Option<String>,
    #[serde(default, rename = "is_pinned")]
    _is_pinned: Option<bool>,
    #[serde(default, rename = "pinned_at")]
    _pinned_at: Option<String>,
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

const PUBLIC_PLANS_PATH: &str = "/api/public/plans";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UpstreamPublicPlansEnvelope {
    success: bool,
    data: Option<Vec<epsx_dioxus_ui::pages::plans::PublicPlan>>,
    error: Option<serde_json::Value>,
    meta: Option<serde_json::Value>,
}

fn safe_plan_text(value: &str, max_chars: usize) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= max_chars
        && !value.chars().any(char::is_control)
}

fn decode_public_plans(
    value: serde_json::Value,
) -> Result<Vec<epsx_dioxus_ui::pages::plans::PublicPlan>, ()> {
    let envelope: UpstreamPublicPlansEnvelope = serde_json::from_value(value).map_err(|_| ())?;
    let _ = envelope.meta;
    if !envelope.success || envelope.error.is_some() {
        return Err(());
    }
    validate_public_plan_collection(envelope.data.ok_or(())?)
}

fn validate_public_plan_collection(
    mut plans: Vec<epsx_dioxus_ui::pages::plans::PublicPlan>,
) -> Result<Vec<epsx_dioxus_ui::pages::plans::PublicPlan>, ()> {
    if plans.len() > 100 {
        return Err(());
    }
    let mut ids = std::collections::HashSet::new();
    for plan in &mut plans {
        let current_price = plan.current_price.parse::<f64>().map_err(|_| ())?;
        let checkout_price = plan.checkout_price.parse::<f64>().map_err(|_| ())?;
        if uuid::Uuid::parse_str(&plan.id).is_err()
            || !ids.insert(plan.id.to_ascii_lowercase())
            || !safe_plan_text(&plan.name, 160)
            || !safe_plan_text(&plan.plan_type, 100)
            || !safe_plan_text(&plan.currency, 12)
            || !safe_plan_text(&plan.billing_cycle, 40)
            || !safe_plan_text(&plan.plan_group, 80)
            || !safe_plan_text(&plan.promotion_status, 40)
            || !current_price.is_finite()
            || current_price < 0.0
            || !checkout_price.is_finite()
            || checkout_price <= 0.0
            || plan.checkout_price.len() > 32
            || !plan
                .checkout_price
                .bytes()
                .all(|byte| byte.is_ascii_digit() || byte == b'.')
            || !matches!(plan.settlement_currency.as_str(), "USDT" | "USDC")
            || plan
                .duration_days
                .is_some_and(|days| !(1..=3_650).contains(&days))
            || !plan.effective_price.is_finite()
            || plan.effective_price < 0.0
            || !plan.promotion_discount.is_finite()
            || !(0.0..=100.0).contains(&plan.promotion_discount)
            || !(-10_000..=10_000).contains(&plan.tier_level)
            || !(0..=10_000).contains(&plan.ranking_offset)
            || !(plan.rankings_limit == -1 || (1..=10_000).contains(&plan.rankings_limit))
            || plan.features.len() > 100
            || plan.permissions.len() > 500
            || plan
                .features
                .iter()
                .any(|feature| !safe_plan_text(feature, 500))
            || plan
                .permissions
                .iter()
                .any(|permission| !safe_plan_text(permission, 300))
        {
            return Err(());
        }
        if plan
            .promotion_ends_at
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            plan.promotion_ends_at = None;
        }
        if plan
            .promotion_ends_at
            .as_deref()
            .is_some_and(|value| value.chars().count() > 80 || value.chars().any(char::is_control))
        {
            return Err(());
        }
    }
    Ok(plans)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UpstreamPublicPlanEnvelope {
    success: bool,
    data: Option<epsx_dioxus_ui::pages::plans::PublicPlan>,
    error: Option<serde_json::Value>,
    meta: Option<serde_json::Value>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PublicPlanLoadError {
    NotFound,
    Unavailable,
    Malformed,
}

pub(crate) async fn load_public_plan_by_id(
    client: &ServiceClient,
    plan_id: &str,
) -> Result<epsx_dioxus_ui::pages::plans::PublicPlan, PublicPlanLoadError> {
    if uuid::Uuid::parse_str(plan_id).is_err() {
        return Err(PublicPlanLoadError::Malformed);
    }
    let value = client
        .get_plain(&format!("{PUBLIC_PLANS_PATH}/{plan_id}"))
        .await
        .map_err(|error| match error {
            ClientError::NotFound => PublicPlanLoadError::NotFound,
            _ => PublicPlanLoadError::Unavailable,
        })?;
    let envelope: UpstreamPublicPlanEnvelope =
        serde_json::from_value(value).map_err(|_| PublicPlanLoadError::Malformed)?;
    let _ = envelope.meta;
    if !envelope.success || envelope.error.is_some() {
        return Err(PublicPlanLoadError::Malformed);
    }
    validate_public_plan_collection(vec![envelope.data.ok_or(PublicPlanLoadError::Malformed)?])
        .map_err(|()| PublicPlanLoadError::Malformed)?
        .pop()
        .ok_or(PublicPlanLoadError::Malformed)
}

pub(crate) async fn load_public_plans(
    client: &ServiceClient,
) -> epsx_dioxus_ui::pages::plans::PublicPlansLoadOutcome {
    use epsx_dioxus_ui::pages::plans::PublicPlansLoadOutcome;
    let value = match client.get_plain(PUBLIC_PLANS_PATH).await {
        Ok(value) => value,
        Err(_) => {
            return PublicPlansLoadOutcome::Error {
                code: "plans_unavailable".to_string(),
            };
        }
    };
    match decode_public_plans(value) {
        Ok(plans) if plans.is_empty() => PublicPlansLoadOutcome::Empty,
        Ok(plans) => PublicPlansLoadOutcome::Ready { plans },
        Err(()) => PublicPlansLoadOutcome::Error {
            code: "malformed_plans_response".to_string(),
        },
    }
}

#[cfg(test)]
mod public_plans_adapter_tests {
    use super::*;
    use axum::{routing::get, Router};
    use epsx_dioxus_ui::pages::plans::PublicPlansLoadOutcome;
    use std::time::Duration;

    async fn client(payload: serde_json::Value) -> ServiceClient {
        let router = Router::new().route(
            PUBLIC_PLANS_PATH,
            get(move || {
                let payload = payload.clone();
                async move { Json(payload) }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
        ServiceClient::new(epsx_client::ClientConfig {
            base_url: format!("http://{address}"),
            timeout: Duration::from_secs(1),
        })
    }

    fn plan() -> serde_json::Value {
        serde_json::json!({
            "id": "61a62cbe-3371-41db-bd90-321c53a71e06",
            "name": "Verified Pro",
            "plan_type": "PRO",
            "current_price": "20.00",
            "effective_price": 15.0,
            "promotion_active": true,
            "promotion_status": "active",
            "promotion_discount": 25.0,
            "promotion_ends_at": "",
            "currency": "USD",
            "billing_cycle": "monthly",
            "features": ["Live analytics"],
            "permissions": ["epsx:analytics:read"],
            "is_active": true,
            "tier_level": 2,
            "plan_group": "personal",
            "ranking_offset": 0,
            "rankings_limit": -1,
            "checkout_price": "9.90",
            "settlement_currency": "USDT",
            "duration_days": 30
        })
    }

    #[tokio::test]
    async fn accepts_the_backend_public_plan_envelope() {
        let payload = serde_json::json!({
            "success": true,
            "data": [plan()],
            "error": null,
            "meta": {"timestamp": "2026-08-21T00:00:00Z"}
        });
        match load_public_plans(&client(payload).await).await {
            PublicPlansLoadOutcome::Ready { plans } => {
                assert_eq!(plans.len(), 1);
                assert_eq!(plans[0].name, "Verified Pro");
                assert_eq!(plans[0].promotion_ends_at, None);
            }
            other => panic!("expected ready public plans, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn rejects_untrusted_or_ambiguous_plan_payloads() {
        for payload in [
            serde_json::json!({"success": true, "data": [{"name": "partial"}], "error": null, "meta": null}),
            serde_json::json!({"success": true, "data": [plan()], "error": {"code": "boom"}, "meta": null}),
            serde_json::json!({"success": true, "data": [plan(), plan()], "error": null, "meta": null}),
        ] {
            assert!(matches!(
                load_public_plans(&client(payload).await).await,
                PublicPlansLoadOutcome::Error { code } if code == "malformed_plans_response"
            ));
        }
    }
}

pub async fn api_health() -> &'static str {
    "ok"
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
            );
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
            );
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
            return clear_session_response(
                &state,
                StatusCode::UNAUTHORIZED,
                "invalid_access_token",
            );
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
        append_clear_session_cookies(headers, state.cookie_environment, CookieClient::Frontend)
            .is_ok()
    })
    .unwrap_or_else(|error| *error)
}

fn clear_refresh_session_response(
    state: &AppState,
    status: StatusCode,
    code: &'static str,
) -> Response {
    match try_clear_session_response(status, code, |headers| {
        append_clear_session_cookies(headers, state.cookie_environment, CookieClient::Frontend)
            .is_ok()
    }) {
        Ok(response) => refresh_response(response, RefreshDisposition::Clear),
        Err(error) => *error,
    }
}

fn try_clear_session_response(
    status: StatusCode,
    code: &'static str,
    append: impl FnOnce(&mut axum::http::HeaderMap) -> bool,
) -> Result<Response, Box<Response>> {
    let mut response = safe_error(status, code);
    if !append(response.headers_mut()) {
        return Err(Box::new(safe_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "session_cookie_error",
        )));
    }
    Ok(response)
}

#[allow(
    clippy::result_large_err,
    reason = "BFF authentication helpers preserve complete HTTP error responses"
)]
async fn verified_bearer(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Result<String, Response> {
    verified_bearer_and_user(state, headers)
        .await
        .map(|(token, _)| token)
}

#[allow(
    clippy::result_large_err,
    reason = "BFF authentication helpers preserve complete HTTP error responses"
)]
async fn verified_bearer_and_user(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Result<(String, SessionUser), Response> {
    super::auth::verified_access_token(headers, state.verifier.as_ref(), state.cookie_environment)
        .await
        .ok_or_else(|| safe_error(StatusCode::UNAUTHORIZED, "invalid_access_token"))
}

const WATCHLIST_PATH: &str = "/api/users/watchlist";
const WATCHLIST_LAYOUT_PATH: &str = "/api/users/watchlist/layout";
const WATCHLIST_GROUPS_PATH: &str = "/api/users/watchlist/groups";
const WATCHLIST_BODY_MAX: usize = 4 * 1024;
const WATCHLIST_LAYOUT_BODY_MAX: usize = 256 * 1024;
const WATCHLIST_FORM_MAX: usize = 1024;

#[derive(Debug, Deserialize)]
struct UpstreamWatchlistEnvelope {
    success: bool,
    data: Option<epsx_dioxus_ui::pages::analytics::WatchlistData>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WatchlistMutationRequest {
    symbol: String,
    group_ids: Option<Vec<uuid::Uuid>>,
}

#[derive(Debug, Deserialize)]
struct UpstreamWatchlistLayoutEnvelope {
    success: bool,
    data: Option<epsx_dioxus_ui::pages::portfolio::WatchlistLayoutData>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct WatchlistGroupNameMutation {
    name: String,
}

fn parse_watchlist_form(body: &[u8]) -> Result<(String, Vec<uuid::Uuid>), ()> {
    let body = std::str::from_utf8(body).map_err(|_| ())?;
    let mut symbol = None;
    let mut group_ids = Vec::new();
    for (key, value) in url::form_urlencoded::parse(body.as_bytes()) {
        match key.as_ref() {
            "symbol" if symbol.is_none() => symbol = Some(value.into_owned()),
            "group_ids" if group_ids.len() < 200 => {
                let id = value.parse::<uuid::Uuid>().map_err(|_| ())?;
                if group_ids.contains(&id) {
                    return Err(());
                }
                group_ids.push(id);
            }
            _ => return Err(()),
        }
    }
    let symbol = symbol
        .and_then(|symbol| epsx_dioxus_ui::pages::analytics::normalize_watchlist_symbol(&symbol))
        .ok_or(())?;
    Ok((symbol, group_ids))
}

pub(crate) fn decode_watchlist_response(
    value: serde_json::Value,
) -> Result<epsx_dioxus_ui::pages::analytics::WatchlistData, ()> {
    let envelope = serde_json::from_value::<UpstreamWatchlistEnvelope>(value).map_err(|_| ())?;
    if !envelope.success {
        return Err(());
    }
    envelope.data.ok_or(())?.validated().map_err(|_| ())
}

pub(crate) fn decode_watchlist_layout_response(
    value: serde_json::Value,
) -> Result<epsx_dioxus_ui::pages::portfolio::WatchlistLayoutData, ()> {
    let envelope =
        serde_json::from_value::<UpstreamWatchlistLayoutEnvelope>(value).map_err(|_| ())?;
    if !envelope.success {
        return Err(());
    }
    envelope.data.ok_or(())?.validated().map_err(|_| ())
}

fn private_watchlist_response(mut response: Response) -> Response {
    mark_session_no_store(&mut response);
    response
}

fn watchlist_success_response(
    watchlist: epsx_dioxus_ui::pages::analytics::WatchlistData,
) -> Response {
    private_watchlist_response(
        Json(serde_json::json!({
            "success": true,
            "data": watchlist,
            "error": null
        }))
        .into_response(),
    )
}

fn verified_watchlist_context(token: String) -> RequestContext {
    let mut context = RequestContext::new();
    context.auth_token = Some(token);
    context
}

fn watchlist_upstream_error() -> Response {
    private_watchlist_response(safe_error(
        StatusCode::SERVICE_UNAVAILABLE,
        "watchlist_upstream_unavailable",
    ))
}

fn watchlist_malformed_error() -> Response {
    private_watchlist_response(safe_error(
        StatusCode::BAD_GATEWAY,
        "watchlist_upstream_malformed",
    ))
}

fn watchlist_client_error(error: ClientError) -> Response {
    let (status, code) = match error {
        ClientError::Unauthorized => (StatusCode::UNAUTHORIZED, "invalid_access_token"),
        ClientError::NotFound => (StatusCode::NOT_FOUND, "watchlist_group_not_found"),
        ClientError::UpstreamStatus(400) => (StatusCode::BAD_REQUEST, "invalid_watchlist_layout"),
        ClientError::UpstreamStatus(409) => (StatusCode::CONFLICT, "watchlist_group_name_conflict"),
        ClientError::UpstreamStatus(422) => {
            (StatusCode::UNPROCESSABLE_ENTITY, "invalid_watchlist_layout")
        }
        ClientError::UpstreamStatus(429) => {
            (StatusCode::TOO_MANY_REQUESTS, "watchlist_rate_limited")
        }
        _ => {
            return watchlist_upstream_error();
        }
    };
    private_watchlist_response(safe_error(status, code))
}

fn watchlist_result(value: serde_json::Value) -> Response {
    match decode_watchlist_response(value) {
        Ok(watchlist) => watchlist_success_response(watchlist),
        Err(()) => watchlist_malformed_error(),
    }
}

pub async fn watchlist_get(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let token = match verified_bearer(&state, &headers).await {
        Ok(token) => token,
        Err(response) => return private_watchlist_response(response),
    };
    match state
        .wallet
        .get_with_ctx(WATCHLIST_PATH, &verified_watchlist_context(token))
        .await
    {
        Ok(value) => watchlist_result(value),
        Err(_) => watchlist_upstream_error(),
    }
}

pub async fn watchlist_post(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    if !watchlist_mutation_origin_allowed(&parts.headers) {
        return private_watchlist_response(safe_error(
            StatusCode::FORBIDDEN,
            "watchlist_mutation_origin_rejected",
        ));
    }
    let token = match verified_bearer(&state, &parts.headers).await {
        Ok(token) => token,
        Err(response) => return private_watchlist_response(response),
    };
    let body = match axum::body::to_bytes(body, WATCHLIST_BODY_MAX).await {
        Ok(body) => body,
        Err(_) => {
            return private_watchlist_response(safe_error(
                StatusCode::PAYLOAD_TOO_LARGE,
                "watchlist_body_too_large",
            ));
        }
    };
    let request = match serde_json::from_slice::<WatchlistMutationRequest>(&body) {
        Ok(request) => request,
        Err(_) => {
            return private_watchlist_response(safe_error(
                StatusCode::BAD_REQUEST,
                "invalid_watchlist_symbol",
            ));
        }
    };
    let symbol = match epsx_dioxus_ui::pages::analytics::normalize_watchlist_symbol(&request.symbol)
    {
        Some(symbol) => symbol,
        None => {
            return private_watchlist_response(safe_error(
                StatusCode::BAD_REQUEST,
                "invalid_watchlist_symbol",
            ));
        }
    };
    let context = verified_watchlist_context(token);
    let body = serde_json::json!({
        "symbol": symbol,
        "group_ids": request.group_ids.unwrap_or_default(),
    });
    match state
        .wallet
        .post_with_ctx(WATCHLIST_PATH, &body, &context)
        .await
    {
        Ok(value) => watchlist_result(value),
        Err(_) => watchlist_upstream_error(),
    }
}

fn watchlist_layout_success_response(
    layout: epsx_dioxus_ui::pages::portfolio::WatchlistLayoutData,
) -> Response {
    private_watchlist_response(
        Json(serde_json::json!({
            "success": true,
            "data": layout,
            "error": null
        }))
        .into_response(),
    )
}

fn watchlist_layout_result(value: serde_json::Value) -> Response {
    match decode_watchlist_layout_response(value) {
        Ok(layout) => watchlist_layout_success_response(layout),
        Err(()) => watchlist_malformed_error(),
    }
}

fn valid_layout_update(update: &epsx_dioxus_ui::pages::portfolio::WatchlistLayoutUpdate) -> bool {
    use std::collections::HashSet;

    if update.groups.len() > 200 || update.ungrouped.len() > 1_000 {
        return false;
    }
    let mut group_ids = HashSet::new();
    let mut grouped = HashSet::new();
    for group in &update.groups {
        if !group_ids.insert(group.id) || group.symbols.len() > 1_000 {
            return false;
        }
        let mut local = HashSet::new();
        for raw in &group.symbols {
            let Some(symbol) = epsx_dioxus_ui::pages::analytics::normalize_watchlist_symbol(raw)
            else {
                return false;
            };
            if !local.insert(symbol.clone()) {
                return false;
            }
            grouped.insert(symbol);
        }
    }
    let mut ungrouped = HashSet::new();
    update.ungrouped.iter().all(|raw| {
        epsx_dioxus_ui::pages::analytics::normalize_watchlist_symbol(raw)
            .is_some_and(|symbol| !grouped.contains(&symbol) && ungrouped.insert(symbol))
    })
}

async fn watchlist_json_body<T: serde::de::DeserializeOwned>(body: Body) -> Result<T, Response> {
    let body = axum::body::to_bytes(body, WATCHLIST_LAYOUT_BODY_MAX)
        .await
        .map_err(|_| {
            private_watchlist_response(safe_error(
                StatusCode::PAYLOAD_TOO_LARGE,
                "watchlist_body_too_large",
            ))
        })?;
    serde_json::from_slice(&body).map_err(|_| {
        private_watchlist_response(safe_error(
            StatusCode::BAD_REQUEST,
            "invalid_watchlist_layout",
        ))
    })
}

pub async fn watchlist_layout_get(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let token = match verified_bearer(&state, &headers).await {
        Ok(token) => token,
        Err(response) => return private_watchlist_response(response),
    };
    match state
        .wallet
        .get_with_ctx(WATCHLIST_LAYOUT_PATH, &verified_watchlist_context(token))
        .await
    {
        Ok(value) => watchlist_layout_result(value),
        Err(error) => watchlist_client_error(error),
    }
}

pub async fn watchlist_layout_put(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    if !watchlist_mutation_origin_allowed(&parts.headers) {
        return private_watchlist_response(safe_error(
            StatusCode::FORBIDDEN,
            "watchlist_mutation_origin_rejected",
        ));
    }
    let token = match verified_bearer(&state, &parts.headers).await {
        Ok(token) => token,
        Err(response) => return private_watchlist_response(response),
    };
    let update =
        match watchlist_json_body::<epsx_dioxus_ui::pages::portfolio::WatchlistLayoutUpdate>(body)
            .await
        {
            Ok(update) if valid_layout_update(&update) => update,
            Ok(_) => {
                return private_watchlist_response(safe_error(
                    StatusCode::BAD_REQUEST,
                    "invalid_watchlist_layout",
                ));
            }
            Err(response) => return response,
        };
    let body = serde_json::to_value(update).expect("validated layout update is serializable");
    match state
        .wallet
        .put_with_ctx(
            WATCHLIST_LAYOUT_PATH,
            &body,
            &verified_watchlist_context(token),
        )
        .await
    {
        Ok(value) => watchlist_layout_result(value),
        Err(error) => watchlist_client_error(error),
    }
}

pub async fn watchlist_group_post(State(state): State<AppState>, request: Request) -> Response {
    watchlist_group_mutation(state, request, None).await
}

pub async fn watchlist_group_put(
    State(state): State<AppState>,
    AxPath(group_id): AxPath<uuid::Uuid>,
    request: Request,
) -> Response {
    watchlist_group_mutation(state, request, Some(group_id)).await
}

async fn watchlist_group_mutation(
    state: AppState,
    request: Request,
    group_id: Option<uuid::Uuid>,
) -> Response {
    let (parts, body) = request.into_parts();
    if !watchlist_mutation_origin_allowed(&parts.headers) {
        return private_watchlist_response(safe_error(
            StatusCode::FORBIDDEN,
            "watchlist_mutation_origin_rejected",
        ));
    }
    let token = match verified_bearer(&state, &parts.headers).await {
        Ok(token) => token,
        Err(response) => return private_watchlist_response(response),
    };
    let mut mutation = match watchlist_json_body::<WatchlistGroupNameMutation>(body).await {
        Ok(mutation) => mutation,
        Err(response) => return response,
    };
    mutation.name = mutation.name.trim().to_string();
    if !(1..=50).contains(&mutation.name.chars().count())
        || mutation.name.chars().any(char::is_control)
    {
        return private_watchlist_response(safe_error(
            StatusCode::BAD_REQUEST,
            "invalid_watchlist_group_name",
        ));
    }
    let body = serde_json::to_value(mutation).expect("validated group name is serializable");
    let context = verified_watchlist_context(token);
    let result = if let Some(group_id) = group_id {
        state
            .wallet
            .put_with_ctx(
                &format!("{WATCHLIST_GROUPS_PATH}/{group_id}"),
                &body,
                &context,
            )
            .await
    } else {
        state
            .wallet
            .post_with_ctx(WATCHLIST_GROUPS_PATH, &body, &context)
            .await
    };
    match result {
        Ok(value) => watchlist_layout_result(value),
        Err(error) => watchlist_client_error(error),
    }
}

pub async fn watchlist_group_delete(
    State(state): State<AppState>,
    AxPath(group_id): AxPath<uuid::Uuid>,
    headers: axum::http::HeaderMap,
) -> Response {
    if !watchlist_mutation_origin_allowed(&headers) {
        return private_watchlist_response(safe_error(
            StatusCode::FORBIDDEN,
            "watchlist_mutation_origin_rejected",
        ));
    }
    let token = match verified_bearer(&state, &headers).await {
        Ok(token) => token,
        Err(response) => return private_watchlist_response(response),
    };
    match state
        .wallet
        .delete_with_ctx(
            &format!("{WATCHLIST_GROUPS_PATH}/{group_id}"),
            &verified_watchlist_context(token),
        )
        .await
    {
        Ok(value) => watchlist_layout_result(value),
        Err(error) => watchlist_client_error(error),
    }
}

pub async fn watchlist_delete(
    State(state): State<AppState>,
    AxPath(raw_symbol): AxPath<String>,
    headers: axum::http::HeaderMap,
) -> Response {
    if !watchlist_mutation_origin_allowed(&headers) {
        return private_watchlist_response(safe_error(
            StatusCode::FORBIDDEN,
            "watchlist_mutation_origin_rejected",
        ));
    }
    let token = match verified_bearer(&state, &headers).await {
        Ok(token) => token,
        Err(response) => return private_watchlist_response(response),
    };
    let symbol = match epsx_dioxus_ui::pages::analytics::normalize_watchlist_symbol(&raw_symbol) {
        Some(symbol) => symbol,
        None => {
            return private_watchlist_response(safe_error(
                StatusCode::BAD_REQUEST,
                "invalid_watchlist_symbol",
            ));
        }
    };
    let path = format!("{WATCHLIST_PATH}/{symbol}");
    match state
        .wallet
        .delete_with_ctx(&path, &verified_watchlist_context(token))
        .await
    {
        Ok(value) => watchlist_result(value),
        Err(_) => watchlist_upstream_error(),
    }
}

async fn watchlist_form_mutation(state: AppState, request: Request, remove: bool) -> Response {
    let (parts, body) = request.into_parts();
    if !same_origin_preferences_form(&parts.headers) {
        return private_watchlist_response(safe_error(
            StatusCode::FORBIDDEN,
            "watchlist_mutation_origin_rejected",
        ));
    }
    let token = match verified_bearer(&state, &parts.headers).await {
        Ok(token) => token,
        Err(response) => return private_watchlist_response(response),
    };
    let content_type = parts
        .headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    if !content_type.split(';').next().is_some_and(|value| {
        value
            .trim()
            .eq_ignore_ascii_case("application/x-www-form-urlencoded")
    }) {
        return private_watchlist_response(safe_error(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "watchlist_form_content_type",
        ));
    }
    let body = match axum::body::to_bytes(body, WATCHLIST_FORM_MAX).await {
        Ok(body) => body,
        Err(_) => {
            return private_watchlist_response(safe_error(
                StatusCode::PAYLOAD_TOO_LARGE,
                "watchlist_body_too_large",
            ));
        }
    };
    let (symbol, group_ids) = match parse_watchlist_form(&body) {
        Ok(parsed) => parsed,
        Err(()) => {
            return private_watchlist_response(safe_error(
                StatusCode::BAD_REQUEST,
                "invalid_watchlist_symbol",
            ));
        }
    };
    let context = verified_watchlist_context(token);
    let result = if remove {
        let path = format!("{WATCHLIST_PATH}/{symbol}");
        state.wallet.delete_with_ctx(&path, &context).await
    } else {
        state
            .wallet
            .post_with_ctx(
                WATCHLIST_PATH,
                &serde_json::json!({"symbol": symbol, "group_ids": group_ids}),
                &context,
            )
            .await
    };
    match result {
        Ok(value) => {
            if decode_watchlist_response(value).is_ok() {
                let mut response = Redirect::to("/portfolio").into_response();
                mark_session_no_store(&mut response);
                response
            } else {
                watchlist_malformed_error()
            }
        }
        Err(_) => watchlist_upstream_error(),
    }
}

/// No-JavaScript fallback for the Portfolio Watch form. The JSON endpoint is
/// canonical; this adapter keeps the server-rendered control functional and
/// reloads the owner-scoped list after a successful save.
pub async fn watchlist_add_form(State(state): State<AppState>, request: Request) -> Response {
    watchlist_form_mutation(state, request, false).await
}

pub async fn watchlist_remove_form(State(state): State<AppState>, request: Request) -> Response {
    watchlist_form_mutation(state, request, true).await
}

// ---------------------------------------------------------------------------
// Developer Portal BFF
// ---------------------------------------------------------------------------

const DEVELOPER_REQUEST_MAX: usize = 32 * 1024;
const DEVELOPER_RESPONSE_MAX: usize = 512 * 1024;
const DEVELOPER_TRY_RESPONSE_MAX: usize = 256 * 1024;
const DEVELOPER_OVERVIEW_PATH: &str = "/api/developer-portal/overview";
const DEVELOPER_KEYS_PATH: &str = "/api/developer-portal/my-keys";
const DEVELOPER_OPENAPI_PATH: &str = "/api-docs/openapi.json";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeveloperOverviewQuery {
    days: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeveloperKeysQuery {
    limit: Option<i64>,
    offset: Option<i64>,
    status: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DeveloperCreateRequest {
    name: String,
    description: Option<String>,
    scopes: Vec<String>,
    expires_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeveloperRevokeRequest {
    reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeveloperEnvelope<T> {
    success: bool,
    data: Option<T>,
    error: Option<serde_json::Value>,
    meta: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DeveloperKeyList {
    api_keys: Vec<epsx_dioxus_ui::pages::developer::DeveloperApiKey>,
    total: i64,
    limit: i64,
    offset: i64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DeveloperCreateResult {
    api_key: epsx_dioxus_ui::pages::developer::DeveloperApiKey,
    secret: Option<String>,
    replayed: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DeveloperRevokeResult {
    id: uuid::Uuid,
    status: String,
    replayed: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeveloperTryRequest {
    operation_id: String,
    api_key: String,
    query: Option<String>,
    body: Option<serde_json::Value>,
    confirm_mutation: Option<bool>,
    idempotency_key: Option<String>,
}

fn developer_private(mut response: Response) -> Response {
    mark_session_no_store(&mut response);
    response.headers_mut().insert(
        header::VARY,
        HeaderValue::from_static("Cookie, Authorization"),
    );
    response
}

fn developer_error(status: StatusCode, code: &'static str) -> Response {
    developer_private(safe_error(status, code))
}

fn developer_success<T: Serialize>(status: StatusCode, data: T) -> Response {
    developer_private(
        (
            status,
            Json(serde_json::json!({
                "success": true,
                "data": data,
                "error": null
            })),
        )
            .into_response(),
    )
}

#[allow(
    clippy::result_large_err,
    reason = "developer adapters preserve the complete safe HTTP error response"
)]
fn valid_developer_days(days: Option<i32>) -> Result<i32, Response> {
    let days = days.unwrap_or(30);
    if matches!(days, 7 | 30 | 90) {
        Ok(days)
    } else {
        Err(developer_error(
            StatusCode::BAD_REQUEST,
            "invalid_developer_usage_window",
        ))
    }
}

fn valid_idempotency_key(value: &str) -> bool {
    (8..=128).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':'))
}

fn valid_create_request(request: &DeveloperCreateRequest) -> bool {
    let valid_text = |value: &str, max: usize, empty: bool| {
        (empty || !value.trim().is_empty())
            && value.chars().count() <= max
            && !value.chars().any(char::is_control)
    };
    valid_text(&request.name, 255, false)
        && request
            .description
            .as_deref()
            .is_none_or(|value| valid_text(value, 2_000, true))
        && (1..=100).contains(&request.scopes.len())
        && request
            .scopes
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            == request.scopes.len()
        && request.scopes.iter().all(|scope| {
            !scope.is_empty()
                && scope.len() <= 255
                && !scope.starts_with("admin:")
                && !scope.chars().any(char::is_control)
        })
        && request.expires_at.as_deref().is_none_or(|value| {
            chrono::DateTime::parse_from_rfc3339(value)
                .ok()
                .is_some_and(|expiry| {
                    let now = chrono::Utc::now();
                    expiry > now && expiry <= now + chrono::Duration::days(3_653)
                })
        })
}

fn developer_upstream_error(status: StatusCode) -> Response {
    match status {
        StatusCode::BAD_REQUEST
        | StatusCode::UNAUTHORIZED
        | StatusCode::FORBIDDEN
        | StatusCode::NOT_FOUND
        | StatusCode::CONFLICT
        | StatusCode::UNPROCESSABLE_ENTITY
        | StatusCode::TOO_MANY_REQUESTS => developer_error(status, "developer_request_rejected"),
        StatusCode::SERVICE_UNAVAILABLE | StatusCode::GATEWAY_TIMEOUT => developer_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "developer_upstream_unavailable",
        ),
        _ => developer_error(StatusCode::BAD_GATEWAY, "developer_upstream_rejected"),
    }
}

async fn developer_response_value(
    response: reqwest::Response,
    limit: usize,
) -> Result<(StatusCode, serde_json::Value), Response> {
    let status = response.status();
    if !status.is_success() {
        return Err(developer_upstream_error(status));
    }
    if !valid_notification_json_content_type(response.headers()) {
        return Err(developer_error(
            StatusCode::BAD_GATEWAY,
            "malformed_developer_response",
        ));
    }
    let body = read_notification_body_limited(response, limit)
        .await
        .map_err(|_| developer_error(StatusCode::BAD_GATEWAY, "malformed_developer_response"))?;
    let value = serde_json::from_slice(&body)
        .map_err(|_| developer_error(StatusCode::BAD_GATEWAY, "malformed_developer_response"))?;
    Ok((status, value))
}

#[allow(
    clippy::result_large_err,
    reason = "developer adapters preserve the complete safe HTTP error response"
)]
fn decode_developer_data<T: serde::de::DeserializeOwned>(
    value: serde_json::Value,
) -> Result<T, Response> {
    let envelope = serde_json::from_value::<DeveloperEnvelope<T>>(value)
        .map_err(|_| developer_error(StatusCode::BAD_GATEWAY, "malformed_developer_response"))?;
    let _ = &envelope.meta;
    if !envelope.success || envelope.error.is_some() {
        return Err(developer_error(
            StatusCode::BAD_GATEWAY,
            "malformed_developer_response",
        ));
    }
    envelope
        .data
        .ok_or_else(|| developer_error(StatusCode::BAD_GATEWAY, "malformed_developer_response"))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DeveloperLoadError {
    Forbidden,
    Unavailable,
    Malformed,
}

pub(crate) async fn load_developer_overview_for_ssr(
    client: &ServiceClient,
    bearer: &str,
    days: i32,
) -> Result<epsx_dioxus_ui::pages::developer::DeveloperOverview, DeveloperLoadError> {
    if !matches!(days, 7 | 30 | 90) {
        return Err(DeveloperLoadError::Malformed);
    }
    let url = format!(
        "{}{}?days={days}",
        client.base_url().trim_end_matches('/'),
        DEVELOPER_OVERVIEW_PATH
    );
    let response = client
        .auth_client()
        .get(url)
        .bearer_auth(bearer)
        .send()
        .await
        .map_err(|_| DeveloperLoadError::Unavailable)?;
    if matches!(
        response.status(),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
    ) {
        return Err(DeveloperLoadError::Forbidden);
    }
    if !response.status().is_success() {
        return Err(DeveloperLoadError::Unavailable);
    }
    if !valid_notification_json_content_type(response.headers()) {
        return Err(DeveloperLoadError::Malformed);
    }
    let body = read_notification_body_limited(response, DEVELOPER_RESPONSE_MAX)
        .await
        .map_err(|error| match error {
            NotificationBodyReadError::TooLarge => DeveloperLoadError::Malformed,
            NotificationBodyReadError::Transport => DeveloperLoadError::Unavailable,
        })?;
    let envelope = serde_json::from_slice::<DeveloperEnvelope<serde_json::Value>>(&body)
        .map_err(|_| DeveloperLoadError::Malformed)?;
    if !envelope.success || envelope.error.is_some() {
        return Err(DeveloperLoadError::Malformed);
    }
    let value = envelope.data.ok_or(DeveloperLoadError::Malformed)?;
    epsx_dioxus_ui::pages::developer::decode_developer_overview(value)
        .ok_or(DeveloperLoadError::Malformed)
}

async fn load_developer_overview(
    state: &AppState,
    bearer: &str,
    days: i32,
) -> Result<epsx_dioxus_ui::pages::developer::DeveloperOverview, Response> {
    load_developer_overview_for_ssr(state.wallet.as_ref(), bearer, days)
        .await
        .map_err(|error| match error {
            DeveloperLoadError::Forbidden => {
                developer_error(StatusCode::FORBIDDEN, "developer_access_forbidden")
            }
            DeveloperLoadError::Unavailable => developer_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "developer_upstream_unavailable",
            ),
            DeveloperLoadError::Malformed => {
                developer_error(StatusCode::BAD_GATEWAY, "malformed_developer_response")
            }
        })
}

pub async fn developer_overview(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Query(query): Query<DeveloperOverviewQuery>,
) -> Response {
    let days = match valid_developer_days(query.days) {
        Ok(days) => days,
        Err(response) => return response,
    };
    let token = match verified_bearer(&state, &headers).await {
        Ok(token) => token,
        Err(response) => return developer_private(response),
    };
    match load_developer_overview(&state, &token, days).await {
        Ok(data) => developer_success(StatusCode::OK, data),
        Err(response) => response,
    }
}

pub async fn developer_usage(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Query(query): Query<DeveloperOverviewQuery>,
) -> Response {
    let days = match valid_developer_days(query.days) {
        Ok(days) => days,
        Err(response) => return response,
    };
    let token = match verified_bearer(&state, &headers).await {
        Ok(token) => token,
        Err(response) => return developer_private(response),
    };
    match load_developer_overview(&state, &token, days).await {
        Ok(data) => developer_success(StatusCode::OK, data.usage),
        Err(response) => response,
    }
}

pub async fn developer_keys(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Query(query): Query<DeveloperKeysQuery>,
) -> Response {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);
    if !(1..=100).contains(&limit)
        || !(0..=1_000_000).contains(&offset)
        || query
            .status
            .as_deref()
            .is_some_and(|value| !matches!(value, "active" | "revoked" | "expired"))
    {
        return developer_error(StatusCode::BAD_REQUEST, "invalid_developer_key_query");
    }
    let token = match verified_bearer(&state, &headers).await {
        Ok(token) => token,
        Err(response) => return developer_private(response),
    };
    let mut path = format!("{DEVELOPER_KEYS_PATH}?limit={limit}&offset={offset}");
    if let Some(status) = query.status {
        path.push_str("&status=");
        path.push_str(&status);
    }
    let response = match state
        .wallet
        .auth_client()
        .get(auth_url(&state, &path))
        .bearer_auth(token)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => {
            return developer_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "developer_upstream_unavailable",
            )
        }
    };
    let (_, value) = match developer_response_value(response, DEVELOPER_RESPONSE_MAX).await {
        Ok(result) => result,
        Err(response) => return response,
    };
    let data = match decode_developer_data::<DeveloperKeyList>(value) {
        Ok(data)
            if data.total >= 0
                && data.limit == limit
                && data.offset == offset
                && data.api_keys.len() <= limit as usize
                && data
                    .api_keys
                    .iter()
                    .all(|key| key.clone().validated().is_ok()) =>
        {
            data
        }
        Ok(_) => return developer_error(StatusCode::BAD_GATEWAY, "malformed_developer_response"),
        Err(response) => return response,
    };
    developer_success(StatusCode::OK, data)
}

async fn parse_developer_create_body(body: Body) -> Result<DeveloperCreateRequest, Response> {
    let body = axum::body::to_bytes(body, DEVELOPER_REQUEST_MAX)
        .await
        .map_err(|_| {
            developer_error(StatusCode::PAYLOAD_TOO_LARGE, "developer_request_too_large")
        })?;
    let request = serde_json::from_slice::<DeveloperCreateRequest>(&body).map_err(|_| {
        developer_error(StatusCode::BAD_REQUEST, "invalid_developer_create_request")
    })?;
    if valid_create_request(&request) {
        Ok(request)
    } else {
        Err(developer_error(
            StatusCode::BAD_REQUEST,
            "invalid_developer_create_request",
        ))
    }
}

async fn create_developer_key_upstream(
    state: &AppState,
    token: &str,
    idempotency_key: &str,
    request: &DeveloperCreateRequest,
) -> Result<(StatusCode, DeveloperCreateResult), Response> {
    let url = auth_url(state, DEVELOPER_KEYS_PATH);
    let response = state
        .wallet
        .auth_client()
        .post(url)
        .bearer_auth(token)
        .header("idempotency-key", idempotency_key)
        .json(request)
        .send()
        .await
        .map_err(|_| {
            developer_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "developer_upstream_unavailable",
            )
        })?;
    let (status, value) = developer_response_value(response, DEVELOPER_RESPONSE_MAX).await?;
    let data = decode_developer_data::<DeveloperCreateResult>(value)?;
    let valid_secret = match (data.replayed, data.secret.as_deref()) {
        (false, Some(secret)) => {
            secret.len() == 69
                && secret.starts_with("epsx_")
                && secret[5..].bytes().all(|byte| byte.is_ascii_hexdigit())
        }
        (true, None) => true,
        _ => false,
    };
    if !valid_secret || data.api_key.clone().validated().is_err() {
        return Err(developer_error(
            StatusCode::BAD_GATEWAY,
            "malformed_developer_response",
        ));
    }
    Ok((status, data))
}

pub async fn developer_key_create(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    if !same_origin_preferences_form(&parts.headers) {
        return developer_error(StatusCode::FORBIDDEN, "developer_mutation_origin_rejected");
    }
    if !valid_notification_json_content_type(&parts.headers) {
        return developer_error(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "developer_json_required",
        );
    }
    let token = match verified_bearer(&state, &parts.headers).await {
        Ok(token) => token,
        Err(response) => return developer_private(response),
    };
    let idempotency_key = match parts
        .headers
        .get("idempotency-key")
        .and_then(|value| value.to_str().ok())
        .filter(|value| valid_idempotency_key(value))
    {
        Some(value) => value.to_string(),
        None => return developer_error(StatusCode::BAD_REQUEST, "invalid_idempotency_key"),
    };
    let request = match parse_developer_create_body(body).await {
        Ok(request) => request,
        Err(response) => return response,
    };
    match create_developer_key_upstream(&state, &token, &idempotency_key, &request).await {
        Ok((status, data)) => developer_success(status, data),
        Err(response) => response,
    }
}

async fn revoke_developer_key_upstream(
    state: &AppState,
    token: &str,
    id: uuid::Uuid,
    idempotency_key: &str,
    request: &DeveloperRevokeRequest,
) -> Result<DeveloperRevokeResult, Response> {
    let url = auth_url(state, &format!("{DEVELOPER_KEYS_PATH}/{id}/revoke"));
    let response = state
        .wallet
        .auth_client()
        .post(url)
        .bearer_auth(token)
        .header("idempotency-key", idempotency_key)
        .json(&serde_json::json!({"reason": request.reason}))
        .send()
        .await
        .map_err(|_| {
            developer_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "developer_upstream_unavailable",
            )
        })?;
    let (_, value) = developer_response_value(response, DEVELOPER_RESPONSE_MAX).await?;
    let data = decode_developer_data::<DeveloperRevokeResult>(value)?;
    if data.id != id || data.status != "revoked" {
        return Err(developer_error(
            StatusCode::BAD_GATEWAY,
            "malformed_developer_response",
        ));
    }
    Ok(data)
}

pub async fn developer_key_revoke(
    State(state): State<AppState>,
    AxPath(id): AxPath<uuid::Uuid>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    if !same_origin_preferences_form(&parts.headers) {
        return developer_error(StatusCode::FORBIDDEN, "developer_mutation_origin_rejected");
    }
    if !valid_notification_json_content_type(&parts.headers) {
        return developer_error(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "developer_json_required",
        );
    }
    let token = match verified_bearer(&state, &parts.headers).await {
        Ok(token) => token,
        Err(response) => return developer_private(response),
    };
    let idempotency_key = match parts
        .headers
        .get("idempotency-key")
        .and_then(|value| value.to_str().ok())
        .filter(|value| valid_idempotency_key(value))
    {
        Some(value) => value.to_string(),
        None => return developer_error(StatusCode::BAD_REQUEST, "invalid_idempotency_key"),
    };
    let body = match axum::body::to_bytes(body, DEVELOPER_REQUEST_MAX).await {
        Ok(body) => body,
        Err(_) => {
            return developer_error(StatusCode::PAYLOAD_TOO_LARGE, "developer_request_too_large")
        }
    };
    let request = match serde_json::from_slice::<DeveloperRevokeRequest>(&body) {
        Ok(request)
            if request.reason.as_deref().is_none_or(|reason| {
                !reason.trim().is_empty()
                    && reason.chars().count() <= 500
                    && !reason.chars().any(char::is_control)
            }) =>
        {
            request
        }
        _ => return developer_error(StatusCode::BAD_REQUEST, "invalid_developer_revoke_request"),
    };
    match revoke_developer_key_upstream(&state, &token, id, &idempotency_key, &request).await {
        Ok(data) => developer_success(StatusCode::OK, data),
        Err(response) => response,
    }
}

async fn load_developer_openapi_value(state: &AppState) -> Result<serde_json::Value, Response> {
    let response = state
        .wallet
        .auth_client()
        .get(auth_url(state, DEVELOPER_OPENAPI_PATH))
        .send()
        .await
        .map_err(|_| {
            developer_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "developer_openapi_unavailable",
            )
        })?;
    let (_, value) = developer_response_value(response, DEVELOPER_RESPONSE_MAX).await?;
    epsx_dioxus_ui::pages::developer::decode_openapi(value.clone())
        .ok_or_else(|| developer_error(StatusCode::BAD_GATEWAY, "malformed_developer_openapi"))?;
    Ok(value)
}

pub(crate) async fn load_developer_openapi_for_ssr(
    client: &ServiceClient,
) -> Result<serde_json::Value, DeveloperLoadError> {
    let response = client
        .auth_client()
        .get(format!(
            "{}{}",
            client.base_url().trim_end_matches('/'),
            DEVELOPER_OPENAPI_PATH
        ))
        .send()
        .await
        .map_err(|_| DeveloperLoadError::Unavailable)?;
    if !response.status().is_success() {
        return Err(DeveloperLoadError::Unavailable);
    }
    if !valid_notification_json_content_type(response.headers()) {
        return Err(DeveloperLoadError::Malformed);
    }
    let body = read_notification_body_limited(response, DEVELOPER_RESPONSE_MAX)
        .await
        .map_err(|error| match error {
            NotificationBodyReadError::TooLarge => DeveloperLoadError::Malformed,
            NotificationBodyReadError::Transport => DeveloperLoadError::Unavailable,
        })?;
    let value = serde_json::from_slice::<serde_json::Value>(&body)
        .map_err(|_| DeveloperLoadError::Malformed)?;
    epsx_dioxus_ui::pages::developer::decode_openapi(value.clone())
        .ok_or(DeveloperLoadError::Malformed)?;
    Ok(value)
}

pub async fn developer_openapi(State(state): State<AppState>) -> Response {
    match load_developer_openapi_value(&state).await {
        Ok(value) => developer_private(Json(value).into_response()),
        Err(response) => response,
    }
}

#[allow(
    clippy::result_large_err,
    reason = "Try It validation preserves the complete safe HTTP error response"
)]
fn normalize_try_query(value: Option<&str>) -> Result<String, Response> {
    let value = value.unwrap_or("");
    if value.len() > 2_048
        || value.starts_with('?')
        || value.contains('#')
        || value.chars().any(char::is_control)
    {
        return Err(developer_error(
            StatusCode::BAD_REQUEST,
            "invalid_developer_try_query",
        ));
    }
    let url = reqwest::Url::parse(&format!("https://query.invalid/?{value}"))
        .map_err(|_| developer_error(StatusCode::BAD_REQUEST, "invalid_developer_try_query"))?;
    Ok(url.query().unwrap_or("").to_string())
}

pub async fn developer_try(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    if !same_origin_preferences_form(&parts.headers) {
        return developer_error(StatusCode::FORBIDDEN, "developer_try_origin_rejected");
    }
    if !valid_notification_json_content_type(&parts.headers) {
        return developer_error(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "developer_json_required",
        );
    }
    let body = match axum::body::to_bytes(body, DEVELOPER_REQUEST_MAX).await {
        Ok(body) => body,
        Err(_) => {
            return developer_error(StatusCode::PAYLOAD_TOO_LARGE, "developer_request_too_large")
        }
    };
    let request = match serde_json::from_slice::<DeveloperTryRequest>(&body) {
        Ok(request) => request,
        Err(_) => return developer_error(StatusCode::BAD_REQUEST, "invalid_developer_try_request"),
    };
    if request.api_key.len() != 69
        || !request.api_key.starts_with("epsx_")
        || !request.api_key[5..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return developer_error(StatusCode::BAD_REQUEST, "invalid_developer_api_key");
    }
    let spec = match load_developer_openapi_value(&state).await {
        Ok(value) => value,
        Err(response) => return response,
    };
    let operations = match epsx_dioxus_ui::pages::developer::decode_openapi(spec) {
        Some(operations) => operations,
        None => return developer_error(StatusCode::BAD_GATEWAY, "malformed_developer_openapi"),
    };
    let operation = match operations
        .into_iter()
        .find(|operation| operation.operation_id == request.operation_id)
    {
        Some(operation)
            if operation.api_key_callable
                && !operation.path.contains('{')
                && !operation.path.contains('}') =>
        {
            operation
        }
        _ => return developer_error(StatusCode::FORBIDDEN, "developer_operation_not_callable"),
    };
    if operation.mutation && request.confirm_mutation != Some(true) {
        return developer_error(
            StatusCode::PRECONDITION_REQUIRED,
            "developer_mutation_confirmation_required",
        );
    }
    let idempotency_key = request.idempotency_key.as_deref();
    if operation.mutation
        && (!operation.idempotent || !idempotency_key.is_some_and(valid_idempotency_key))
    {
        return developer_error(StatusCode::BAD_REQUEST, "invalid_idempotency_key");
    }
    if !operation.mutation && request.body.is_some() {
        return developer_error(StatusCode::BAD_REQUEST, "developer_try_body_not_allowed");
    }
    let query = match normalize_try_query(request.query.as_deref()) {
        Ok(query) => query,
        Err(response) => return response,
    };
    let mut url = auth_url(&state, &operation.path);
    if !query.is_empty() {
        url.push('?');
        url.push_str(&query);
    }
    let method = match reqwest::Method::from_bytes(operation.method.as_bytes()) {
        Ok(method) => method,
        Err(_) => return developer_error(StatusCode::BAD_GATEWAY, "malformed_developer_openapi"),
    };
    let mut upstream = state
        .wallet
        .auth_client()
        .request(method, url)
        .bearer_auth(&request.api_key)
        .header(header::ACCEPT, "application/json");
    if let Some(idempotency_key) = idempotency_key {
        upstream = upstream.header("idempotency-key", idempotency_key);
    }
    if let Some(body) = request.body {
        upstream = upstream.json(&body);
    }
    let response = match upstream.send().await {
        Ok(response) => response,
        Err(_) => {
            return developer_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "developer_try_upstream_unavailable",
            )
        }
    };
    let status = response.status().as_u16();
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("application/octet-stream")
        .chars()
        .take(128)
        .collect::<String>();
    let response_body =
        match read_notification_body_limited(response, DEVELOPER_TRY_RESPONSE_MAX).await {
            Ok(body) => body,
            Err(_) => {
                return developer_error(StatusCode::BAD_GATEWAY, "developer_try_response_too_large")
            }
        };
    let response_body = String::from_utf8_lossy(&response_body).into_owned();
    developer_success(
        StatusCode::OK,
        serde_json::json!({
            "status": status,
            "content_type": content_type,
            "body": response_body
        }),
    )
}

fn parse_form_pairs(body: &[u8]) -> BTreeMap<String, Vec<String>> {
    let mut fields = BTreeMap::<String, Vec<String>>::new();
    for (key, value) in url::form_urlencoded::parse(body) {
        fields
            .entry(key.into_owned())
            .or_default()
            .push(value.into_owned());
    }
    fields
}

fn take_single(
    fields: &mut BTreeMap<String, Vec<String>>,
    name: &str,
) -> Result<Option<String>, ()> {
    match fields.remove(name) {
        None => Ok(None),
        Some(mut values) if values.len() == 1 => Ok(values.pop()),
        Some(_) => Err(()),
    }
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub async fn developer_key_create_form(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    if !same_origin_preferences_form(&parts.headers) {
        return developer_error(StatusCode::FORBIDDEN, "developer_mutation_origin_rejected");
    }
    let token = match verified_bearer(&state, &parts.headers).await {
        Ok(token) => token,
        Err(response) => return developer_private(response),
    };
    let body = match axum::body::to_bytes(body, DEVELOPER_REQUEST_MAX).await {
        Ok(body) => body,
        Err(_) => {
            return developer_error(StatusCode::PAYLOAD_TOO_LARGE, "developer_request_too_large")
        }
    };
    let mut fields = parse_form_pairs(&body);
    let (idempotency_key, name, description, expires_at) = match (
        take_single(&mut fields, "idempotency_key"),
        take_single(&mut fields, "name"),
        take_single(&mut fields, "description"),
        take_single(&mut fields, "expires_at"),
    ) {
        (Ok(idempotency_key), Ok(name), Ok(description), Ok(expires_at)) => (
            idempotency_key.filter(|value| valid_idempotency_key(value)),
            name,
            description.filter(|value| !value.is_empty()),
            expires_at.filter(|value| !value.is_empty()),
        ),
        _ => return developer_error(StatusCode::BAD_REQUEST, "invalid_developer_create_request"),
    };
    let scopes = fields.remove("scopes").unwrap_or_default();
    if !fields.is_empty() || idempotency_key.is_none() || name.is_none() {
        return developer_error(StatusCode::BAD_REQUEST, "invalid_developer_create_request");
    }
    let request = DeveloperCreateRequest {
        name: name.unwrap_or_default(),
        description,
        scopes,
        expires_at,
    };
    if !valid_create_request(&request) {
        return developer_error(StatusCode::BAD_REQUEST, "invalid_developer_create_request");
    }
    let result = create_developer_key_upstream(
        &state,
        &token,
        idempotency_key.as_deref().unwrap_or_default(),
        &request,
    )
    .await;
    let (status, body) = match result {
        Ok((_, data)) => {
            let message = data.secret.as_deref().map_or_else(
                || {
                    "This request was already completed. The secret cannot be shown again."
                        .to_string()
                },
                |secret| {
                    format!(
                        "<code id=\"developer-secret\">{}</code>",
                        html_escape(secret)
                    )
                },
            );
            (
                StatusCode::OK,
                format!("<!doctype html><meta name=\"robots\" content=\"noindex\"><title>API key created</title><main><h1>API key created</h1><p>Save this secret now. It will not be shown again.</p>{message}<p><a href=\"/developer\">Back to Developer Portal</a></p></main>"),
            )
        }
        Err(response) => return response,
    };
    let mut response = (
        status,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        body,
    )
        .into_response();
    mark_session_no_store(&mut response);
    response
}

pub async fn developer_key_revoke_form(
    State(state): State<AppState>,
    AxPath(id): AxPath<uuid::Uuid>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    if !same_origin_preferences_form(&parts.headers) {
        return developer_error(StatusCode::FORBIDDEN, "developer_mutation_origin_rejected");
    }
    let token = match verified_bearer(&state, &parts.headers).await {
        Ok(token) => token,
        Err(response) => return developer_private(response),
    };
    let body = match axum::body::to_bytes(body, DEVELOPER_REQUEST_MAX).await {
        Ok(body) => body,
        Err(_) => {
            return developer_error(StatusCode::PAYLOAD_TOO_LARGE, "developer_request_too_large")
        }
    };
    let mut fields = parse_form_pairs(&body);
    let (idempotency_key, reason, confirm_revoke) = match (
        take_single(&mut fields, "idempotency_key"),
        take_single(&mut fields, "reason"),
        take_single(&mut fields, "confirm_revoke"),
    ) {
        (Ok(idempotency_key), Ok(reason), Ok(confirm_revoke)) => (
            idempotency_key.filter(|value| valid_idempotency_key(value)),
            reason,
            confirm_revoke,
        ),
        _ => return developer_error(StatusCode::BAD_REQUEST, "invalid_developer_revoke_request"),
    };
    if !fields.is_empty() || idempotency_key.is_none() || confirm_revoke.as_deref() != Some("yes") {
        return developer_error(StatusCode::BAD_REQUEST, "invalid_developer_revoke_request");
    }
    let request = DeveloperRevokeRequest { reason };
    match revoke_developer_key_upstream(
        &state,
        &token,
        id,
        idempotency_key.as_deref().unwrap_or_default(),
        &request,
    )
    .await
    {
        Ok(_) => {
            let mut response = Redirect::to("/developer").into_response();
            mark_session_no_store(&mut response);
            response
        }
        Err(response) => response,
    }
}

#[cfg(test)]
mod developer_contract_tests {
    use super::*;

    #[test]
    fn create_contract_cannot_select_wallet_plans_or_rate_limits() {
        for authority_field in ["wallet_address", "plan_ids", "rate_limit_per_minute"] {
            let mut value = serde_json::json!({
                "name": "integration",
                "description": null,
                "scopes": ["epsx:analytics:view"],
                "expires_at": null
            });
            value[authority_field] = serde_json::json!("attacker-controlled");
            assert!(serde_json::from_value::<DeveloperCreateRequest>(value).is_err());
        }
    }

    #[test]
    fn try_contract_has_no_arbitrary_url_or_header_surface() {
        let valid = serde_json::json!({
            "operation_id": "getAnalyticsRankings",
            "api_key": format!("epsx_{}", "a".repeat(64)),
            "query": "country=US",
            "body": null,
            "confirm_mutation": false,
            "idempotency_key": null
        });
        assert!(serde_json::from_value::<DeveloperTryRequest>(valid.clone()).is_ok());
        for field in ["url", "headers", "authorization"] {
            let mut unsafe_value = valid.clone();
            unsafe_value[field] = serde_json::json!("https://evil.test");
            assert!(serde_json::from_value::<DeveloperTryRequest>(unsafe_value).is_err());
        }
    }

    #[test]
    fn usage_windows_and_try_queries_fail_closed() {
        for days in [7, 30, 90] {
            assert_eq!(valid_developer_days(Some(days)).unwrap(), days);
        }
        assert!(valid_developer_days(Some(31)).is_err());
        assert!(normalize_try_query(Some("https://evil.test")).is_ok());
        assert!(normalize_try_query(Some("?url=https://evil.test")).is_err());
        assert!(normalize_try_query(Some("country=US#fragment")).is_err());
    }
}

#[cfg(test)]
mod watchlist_contract_tests {
    use super::*;

    #[test]
    fn watchlist_decoder_accepts_only_successful_valid_owner_scoped_symbols() {
        let decoded = decode_watchlist_response(serde_json::json!({
            "success": true,
            "data": {"symbols": ["aapl", "BRK.B", "AAPL"]},
            "error": null,
            "meta": {"timestamp": "2026-07-27T00:00:00Z"}
        }))
        .unwrap();
        assert_eq!(decoded.symbols, vec!["AAPL", "BRK.B"]);

        for malformed in [
            serde_json::json!({"symbols": ["AAPL"]}),
            serde_json::json!({"success": false, "data": {"symbols": ["AAPL"]}}),
            serde_json::json!({"success": true, "data": {"symbols": ["../AAPL"]}}),
            serde_json::json!({"success": true, "data": {"symbols": "AAPL"}}),
            serde_json::json!({"success": true, "data": {"symbols": ["AAPL"], "owner": "other"}}),
        ] {
            assert!(decode_watchlist_response(malformed).is_err());
        }
    }

    #[test]
    fn layout_decoder_counts_distinct_symbols_and_rejects_invalid_virtual_ungrouped() {
        let group_id = uuid::Uuid::new_v4();
        let decoded = decode_watchlist_layout_response(serde_json::json!({
            "success": true,
            "data": {
                "groups": [{
                    "id": group_id,
                    "name": " Growth ",
                    "position": 0,
                    "symbols": ["aapl", "MSFT"]
                }],
                "ungrouped": ["BRK.B"],
                "watched": 3
            }
        }))
        .unwrap();
        assert_eq!(decoded.groups[0].name, "Growth");
        assert_eq!(decoded.groups[0].symbols, ["AAPL", "MSFT"]);
        assert_eq!(decoded.watched, 3);

        for malformed in [
            serde_json::json!({
                "success": true,
                "data": {
                    "groups": [{"id": group_id, "name": "Growth", "position": 0, "symbols": ["AAPL"]}],
                    "ungrouped": ["AAPL"],
                    "watched": 1
                }
            }),
            serde_json::json!({
                "success": true,
                "data": {"groups": [], "ungrouped": ["AAPL"], "watched": 2}
            }),
        ] {
            assert!(decode_watchlist_layout_response(malformed).is_err());
        }
    }

    #[test]
    fn layout_mutation_validation_allows_multi_group_but_not_local_duplicates() {
        use epsx_dioxus_ui::pages::portfolio::{WatchlistGroupLayoutUpdate, WatchlistLayoutUpdate};

        let first = uuid::Uuid::new_v4();
        let second = uuid::Uuid::new_v4();
        assert!(valid_layout_update(&WatchlistLayoutUpdate {
            groups: vec![
                WatchlistGroupLayoutUpdate {
                    id: first,
                    symbols: vec!["AAPL".into()],
                },
                WatchlistGroupLayoutUpdate {
                    id: second,
                    symbols: vec!["aapl".into()],
                },
            ],
            ungrouped: vec!["MSFT".into()],
        }));
        assert!(!valid_layout_update(&WatchlistLayoutUpdate {
            groups: vec![WatchlistGroupLayoutUpdate {
                id: first,
                symbols: vec!["AAPL".into(), " aapl ".into()],
            }],
            ungrouped: vec![],
        }));
    }

    #[test]
    fn watchlist_symbols_are_bounded_and_canonical() {
        for accepted in ["aapl", "BRK.B", "BTC-USD", "2317"] {
            assert!(
                epsx_dioxus_ui::pages::analytics::normalize_watchlist_symbol(accepted).is_some()
            );
        }
        for rejected in [
            "",
            "../AAPL",
            "AAPL/USD",
            "AAPL value",
            "💥",
            "ABCDEFGHIJKLMNOPQRSTU",
        ] {
            assert!(
                epsx_dioxus_ui::pages::analytics::normalize_watchlist_symbol(rejected).is_none()
            );
        }
    }

    #[test]
    fn portfolio_form_accepts_exactly_one_canonical_symbol() {
        assert_eq!(
            parse_watchlist_form(b"symbol=brk.b").unwrap(),
            ("BRK.B".to_string(), vec![])
        );
        let group_id = uuid::Uuid::new_v4();
        assert_eq!(
            parse_watchlist_form(format!("symbol=BTC-USD&group_ids={group_id}").as_bytes())
                .unwrap(),
            ("BTC-USD".to_string(), vec![group_id])
        );
        for invalid in [
            b"".as_slice(),
            b"symbol=..%2FAAPL".as_slice(),
            b"symbol=AAPL&symbol=MSFT".as_slice(),
            b"symbol=AAPL&owner=other".as_slice(),
        ] {
            assert!(parse_watchlist_form(invalid).is_err());
        }
    }
}

const NOTIFICATION_LIST_LIMIT_MAX: u16 = 100;
const NOTIFICATION_LIST_OFFSET_MAX: u32 = 1_000_000;
pub(crate) const NOTIFICATION_SSR_PAGE_SIZE: u16 = 20;
#[cfg(test)]
pub(crate) const NOTIFICATION_SSR_MAX_PAGE: u32 =
    (NOTIFICATION_LIST_OFFSET_MAX / NOTIFICATION_SSR_PAGE_SIZE as u32) + 1;
// The list endpoint returns at most 100 rows. A 2 MiB cap leaves roughly
// 20 KiB per row for the body and JSON data while preventing a chunked
// upstream response from forcing unbounded BFF allocation. The unread
// response is the single-field `{ "count": u64 }` DTO constrained to the
// largest integer JavaScript can represent exactly.
const NOTIFICATION_LIST_BODY_MAX: usize = 2 * 1024 * 1024;
const NOTIFICATION_UNREAD_BODY_MAX: usize = 4 * 1024;
const NOTIFICATION_UNREAD_JS_SAFE_MAX: u64 = 9_007_199_254_740_991;
const NOTIFICATION_PREFERENCES_BODY_MAX: usize = 64 * 1024;
const NOTIFICATION_PREFERENCES_RESPONSE_MAX: usize = 128 * 1024;
const NOTIFICATION_BULK_MUTATION_BODY_MAX: usize = 4 * 1024;
const NOTIFICATION_PREFERENCES_FORM_MAX: usize = 64 * 1024;
pub(crate) const NOTIFICATION_PREFERENCES_FLASH_COOKIE: &str =
    "epsx.notification_preferences_flash";
const NOTIFICATION_ID_MAX: usize = 128;
const NOTIFICATION_RECIPIENT_MAX: usize = 2 * 1024;
const NOTIFICATION_SUBJECT_MAX: usize = 512;
const NOTIFICATION_BODY_MAX: usize = 16 * 1024;
const NOTIFICATION_ERROR_MAX: usize = 1024;
const NOTIFICATION_TITLE_MAX: usize = 512;
const NOTIFICATION_TYPE_MAX: usize = 64;
const NOTIFICATION_PRIORITY_MAX: usize = 32;
const NOTIFICATION_ACTION_URL_MAX: usize = 2 * 1024;
const NOTIFICATION_DATA_MAX: usize = 64 * 1024;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct NotificationListQuery {
    limit: Option<u16>,
    offset: Option<u32>,
    status: Option<String>,
    notification_type: Option<String>,
    priority: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
}

impl NotificationListQuery {
    /// Build the fixed, owner-scoped page used by `/notifications` SSR.
    ///
    /// The browser selects only a canonical page number. Page size, offset,
    /// and owner are not caller-controlled: size is frozen to the development
    /// source's 20 rows and offset is derived here.
    #[cfg(test)]
    pub(crate) fn for_ssr_page(page: u32) -> Option<Self> {
        Self::for_ssr_page_and_status(page, None)
    }

    /// Build a fixed owner-scoped SSR page with the bounded status filter.
    #[cfg(test)]
    pub(crate) fn for_ssr_page_and_status(page: u32, status: Option<&str>) -> Option<Self> {
        Self::for_ssr_page_and_filters(page, status, None, None)
    }

    /// Build a fixed owner-scoped SSR page with the source-compatible status,
    /// type, and priority filters. The caller supplies canonical values only;
    /// this final boundary still rejects whitespace/control/unbounded values
    /// before an upstream request is possible.
    #[cfg(test)]
    pub(crate) fn for_ssr_page_and_filters(
        page: u32,
        status: Option<&str>,
        notification_type: Option<&str>,
        priority: Option<&str>,
    ) -> Option<Self> {
        Self::for_ssr_page_and_filters_and_dates(
            page,
            status,
            notification_type,
            priority,
            None,
            None,
        )
    }

    /// Build a fixed owner-scoped SSR page with every bounded source filter.
    /// Dates remain source-owned RFC3339 instants and are never interpreted by
    /// the UI; they are validated here before the upstream request is built.
    pub(crate) fn for_ssr_page_and_filters_and_dates(
        page: u32,
        status: Option<&str>,
        notification_type: Option<&str>,
        priority: Option<&str>,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Option<Self> {
        if status.is_some_and(|status| !matches!(status, "read" | "unread")) {
            return None;
        }
        let valid_filter = |value: Option<&str>, max: usize| {
            value.is_none_or(|value| {
                !value.is_empty()
                    && value.len() <= max
                    && !value
                        .chars()
                        .any(|character| character.is_control() || character.is_whitespace())
            })
        };
        if !valid_filter(notification_type, NOTIFICATION_TYPE_MAX)
            || !valid_filter(priority, NOTIFICATION_PRIORITY_MAX)
        {
            return None;
        }
        let valid_date = |value: Option<&str>| {
            value.is_none_or(|value| {
                value.len() <= 64 && DateTime::parse_from_rfc3339(value).is_ok()
            })
        };
        if !valid_date(start_date) || !valid_date(end_date) {
            return None;
        }
        if let (Some(start), Some(end)) = (start_date, end_date) {
            if DateTime::parse_from_rfc3339(start).ok()? > DateTime::parse_from_rfc3339(end).ok()? {
                return None;
            }
        }
        let offset = page
            .checked_sub(1)?
            .checked_mul(u32::from(NOTIFICATION_SSR_PAGE_SIZE))?;
        if offset > NOTIFICATION_LIST_OFFSET_MAX {
            return None;
        }
        Some(Self {
            limit: Some(NOTIFICATION_SSR_PAGE_SIZE),
            offset: Some(offset),
            status: status.map(str::to_string),
            notification_type: notification_type.map(str::to_string),
            priority: priority.map(str::to_string),
            start_date: start_date.map(str::to_string),
            end_date: end_date.map(str::to_string),
        })
    }

    fn from_raw_query(raw: Option<&str>) -> Result<Self, ()> {
        let Some(raw) = raw.filter(|raw| !raw.is_empty()) else {
            return Ok(Self::default());
        };
        let url =
            reqwest::Url::parse(&format!("https://frontend.invalid/?{raw}")).map_err(|_| ())?;
        let mut query = Self::default();
        let mut page = None;
        let mut seen = std::collections::HashSet::new();
        for (key, value) in url.query_pairs() {
            let key = key.as_ref();
            if !seen.insert(key.to_string()) {
                return Err(());
            }
            match key {
                "page" => {
                    let value: u32 = value.parse().map_err(|_| ())?;
                    if value == 0 {
                        return Err(());
                    }
                    page = Some(value);
                }
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
                    if !matches!(
                        value.as_ref(),
                        "pending" | "sent" | "failed" | "suppressed" | "read" | "unread" | "all"
                    ) {
                        return Err(());
                    }
                    query.status = Some(value.into_owned());
                }
                "type" | "notification_type" => {
                    if query.notification_type.is_some() {
                        return Err(());
                    }
                    if value.is_empty()
                        || value.len() > NOTIFICATION_TYPE_MAX
                        || value
                            .chars()
                            .any(|character| character.is_control() || character.is_whitespace())
                    {
                        return Err(());
                    }
                    query.notification_type = Some(value.into_owned());
                }
                "priority" => {
                    if value.is_empty()
                        || value.len() > NOTIFICATION_PRIORITY_MAX
                        || value
                            .chars()
                            .any(|character| character.is_control() || character.is_whitespace())
                    {
                        return Err(());
                    }
                    query.priority = Some(value.into_owned());
                }
                "start_date" | "end_date" => {
                    if value.len() > 64 || DateTime::parse_from_rfc3339(value.as_ref()).is_err() {
                        return Err(());
                    }
                    if key == "start_date" {
                        query.start_date = Some(value.into_owned());
                    } else {
                        query.end_date = Some(value.into_owned());
                    }
                }
                _ => return Err(()),
            }
        }
        if page.is_some() && query.offset.is_some() {
            return Err(());
        }
        if let Some(page) = page {
            let page_size = u32::from(query.limit.unwrap_or(NOTIFICATION_SSR_PAGE_SIZE));
            query.offset = Some(
                page.checked_sub(1)
                    .and_then(|value| value.checked_mul(page_size))
                    .filter(|offset| *offset <= NOTIFICATION_LIST_OFFSET_MAX)
                    .ok_or(())?,
            );
        }
        if query
            .start_date
            .as_deref()
            .zip(query.end_date.as_deref())
            .is_some_and(|(start, end)| {
                DateTime::parse_from_rfc3339(start).ok() > DateTime::parse_from_rfc3339(end).ok()
            })
        {
            return Err(());
        }
        Ok(query)
    }

    pub(crate) fn upstream_suffix(&self) -> String {
        let mut url = reqwest::Url::parse("https://frontend.invalid/").expect("static URL");
        let mut fields = url.query_pairs_mut();
        if let Some(limit) = self.limit {
            fields.append_pair("limit", &limit.to_string());
        }
        if let Some(offset) = self.offset {
            fields.append_pair("offset", &offset.to_string());
        }
        if let Some(status) = &self.status {
            fields.append_pair("status", status);
        }
        if let Some(notification_type) = &self.notification_type {
            fields.append_pair("type", notification_type);
        }
        if let Some(priority) = &self.priority {
            fields.append_pair("priority", priority);
        }
        if let Some(start_date) = &self.start_date {
            fields.append_pair("start_date", start_date);
        }
        if let Some(end_date) = &self.end_date {
            fields.append_pair("end_date", end_date);
        }
        drop(fields);
        if let Some(query) = url.query() {
            if query.is_empty() {
                String::new()
            } else {
                format!("?{query}")
            }
        } else {
            String::new()
        }
    }

    pub(crate) fn upstream_unread_suffix(&self) -> String {
        let mut url = reqwest::Url::parse("https://frontend.invalid/").expect("static URL");
        let mut fields = url.query_pairs_mut();
        if let Some(status) = &self.status {
            fields.append_pair("status", status);
        }
        if let Some(notification_type) = &self.notification_type {
            fields.append_pair("type", notification_type);
        }
        if let Some(priority) = &self.priority {
            fields.append_pair("priority", priority);
        }
        if let Some(start_date) = &self.start_date {
            fields.append_pair("start_date", start_date);
        }
        if let Some(end_date) = &self.end_date {
            fields.append_pair("end_date", end_date);
        }
        drop(fields);
        url.query()
            .filter(|query| !query.is_empty())
            .map(|query| format!("?{query}"))
            .unwrap_or_default()
    }
}

#[derive(Debug, Default)]
enum RequiredNullable<T> {
    #[default]
    Missing,
    Present(Option<T>),
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
    clicked_at: RequiredNullable<DateTime<chrono::Utc>>,
    #[serde(default)]
    title: RequiredNullable<String>,
    #[serde(default)]
    notification_type: RequiredNullable<String>,
    #[serde(default)]
    priority: RequiredNullable<String>,
    #[serde(default)]
    action_url: RequiredNullable<String>,
    #[serde(default)]
    expires_at: RequiredNullable<DateTime<chrono::Utc>>,
}

impl NotificationListWire {
    fn validate(&self, owner: &str, query: &NotificationListQuery) -> Result<(), ()> {
        let limit = usize::from(query.limit.unwrap_or(50));
        if self.total < 0 || self.items.len() > limit || self.total < self.items.len() as i64 {
            return Err(());
        }
        // The service count describes the same owner, broadcast, expiry, and
        // optional-status predicate as the row query. Require exact page
        // cardinality so a split count/row read cannot become an authoritative
        // empty or partial page.
        let offset = u64::from(query.offset.unwrap_or(0));
        let remaining = (self.total as u64).saturating_sub(offset);
        let expected = remaining.min(limit as u64) as usize;
        if self.items.len() != expected {
            return Err(());
        }
        for item in &self.items {
            if !bounded_notification_text(&item.id, NOTIFICATION_ID_MAX, false)
                || !bounded_notification_text(&item.channel, 32, false)
                || !bounded_notification_text(&item.recipient, NOTIFICATION_RECIPIENT_MAX, false)
                || !bounded_notification_text(&item.status, 32, false)
                || !bounded_notification_text(&item.body, NOTIFICATION_BODY_MAX, true)
                || item
                    .data
                    .as_ref()?
                    .is_some_and(|data| notification_json_size(data) > NOTIFICATION_DATA_MAX)
            {
                return Err(());
            }
            let owner_matches = item.user_id.as_ref()?.is_some_and(|user_id| {
                bounded_notification_text(user_id, NOTIFICATION_ID_MAX, false)
                    && user_id.eq_ignore_ascii_case(owner)
            });
            let broadcast_matches = item.user_id.as_ref()?.is_none()
                && item.recipient.eq_ignore_ascii_case("all")
                && !item.channel.is_empty();
            if !owner_matches && !broadcast_matches {
                return Err(());
            }
            if item
                .template_id
                .as_ref()?
                .is_some_and(|value| !bounded_notification_text(value, NOTIFICATION_ID_MAX, false))
                || item.subject.as_ref()?.is_some_and(|value| {
                    !bounded_notification_text(value, NOTIFICATION_SUBJECT_MAX, true)
                })
                || item.error.as_ref()?.is_some_and(|value| {
                    !bounded_notification_text(value, NOTIFICATION_ERROR_MAX, true)
                })
                || item.title.as_ref()?.is_some_and(|value| {
                    !bounded_notification_text(value, NOTIFICATION_TITLE_MAX, true)
                })
                || item.notification_type.as_ref()?.is_some_and(|value| {
                    !bounded_notification_text(value, NOTIFICATION_TYPE_MAX, true)
                        || value.chars().any(char::is_whitespace)
                })
                || item.priority.as_ref()?.is_some_and(|value| {
                    !bounded_notification_text(value, NOTIFICATION_PRIORITY_MAX, true)
                        || value.chars().any(char::is_whitespace)
                })
                || item.action_url.as_ref()?.is_some_and(|value| {
                    !bounded_notification_text(value, NOTIFICATION_ACTION_URL_MAX, true)
                        || !valid_legacy_action_url(value)
                })
                || item
                    .expires_at
                    .as_ref()?
                    .is_some_and(|value| *value <= item.created_at)
            {
                return Err(());
            }
            item.template_id.as_ref()?;
            item.subject.as_ref()?;
            item.data.as_ref()?;
            item.error.as_ref()?;
            item.sent_at.as_ref()?;
            item.read_at.as_ref()?;
            item.clicked_at.as_ref()?;
            item.title.as_ref()?;
            item.notification_type.as_ref()?;
            item.priority.as_ref()?;
            item.action_url.as_ref()?;
            item.expires_at.as_ref()?;
        }
        Ok(())
    }
}

fn bounded_notification_text(value: &str, max_bytes: usize, allow_empty: bool) -> bool {
    (allow_empty || !value.is_empty())
        && value.len() <= max_bytes
        && !value.chars().any(char::is_control)
}

fn notification_json_size(value: &serde_json::Value) -> usize {
    serde_json::to_vec(value).map_or(usize::MAX, |encoded| encoded.len())
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
            );
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
            );
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
    load_notification_unread_count_with_query(client, bearer, request_id, None).await
}

pub(crate) async fn load_notification_unread_count_with_query(
    client: &ServiceClient,
    bearer: &str,
    request_id: &NotificationRequestId,
    query: Option<&NotificationListQuery>,
) -> NotificationUnreadLoadOutcome {
    let url = format!(
        "{}/api/v1/notification/unread-count",
        client.base_url().trim_end_matches('/'),
    );
    let url = match query.map(NotificationListQuery::upstream_unread_suffix) {
        Some(suffix) => format!("{url}{suffix}"),
        None => url,
    };
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
            return NotificationUnreadLoadOutcome::Malformed;
        }
        Err(NotificationBodyReadError::Transport) => {
            return NotificationUnreadLoadOutcome::DependencyUnavailable;
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

// ---------------------------------------------------------------------------
// Development-branch notification compatibility adapter
// ---------------------------------------------------------------------------
// Keep the source `/api/notifications` contract at the BFF boundary while
// `/api/v1/notifications` remains the canonical Rust contract.  These
// projections are owner-bound, bounded, and fail closed on malformed target
// data; they never forward a caller-controlled owner selector to the service.

#[derive(Debug, Serialize)]
struct LegacyNotification {
    id: String,
    wallet_address: String,
    notification_type: String,
    title: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
    priority: String,
    timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    read_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    clicked_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delivered_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    action_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_url: Option<String>,
    read: bool,
}

#[derive(Debug, Serialize)]
struct LegacyNotificationsData {
    notifications: Vec<LegacyNotification>,
    total_count: u64,
    unread_count: u64,
    page: u32,
    limit: u16,
    total_pages: u64,
}

#[derive(Debug, Serialize)]
struct LegacyNotificationsResponse {
    success: bool,
    data: LegacyNotificationsData,
    api_version: &'static str,
    access_level: &'static str,
}

fn legacy_notification_type(value: Option<&str>) -> String {
    match value {
        Some(value)
            if matches!(
                value,
                "system"
                    | "security"
                    | "permission"
                    | "wallet_management"
                    | "wallet"
                    | "payment"
                    | "general"
                    | "announcement"
                    | "advertisement"
                    | "chat"
            ) =>
        {
            value.to_string()
        }
        _ => "system".to_string(),
    }
}

fn legacy_notification_priority(value: Option<&str>) -> String {
    match value {
        Some(value) if matches!(value, "low" | "normal" | "high" | "critical" | "urgent") => {
            value.to_string()
        }
        _ => "normal".to_string(),
    }
}

fn target_notification_id_from_legacy(value: &str) -> String {
    uuid::Uuid::parse_str(value)
        .map(|id| format!("0x{}", id.simple()))
        .unwrap_or_else(|_| value.to_string())
}

fn legacy_notification_id(value: &str) -> Option<String> {
    if let Ok(id) = uuid::Uuid::parse_str(value) {
        return Some(id.to_string());
    }
    let raw = value.strip_prefix("0x")?;
    if raw.len() != 32 || !raw.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    uuid::Uuid::parse_str(raw).ok().map(|id| id.to_string())
}

fn required_nullable_ref<T>(value: &RequiredNullable<T>) -> Option<&T> {
    match value {
        RequiredNullable::Present(value) => value.as_ref(),
        RequiredNullable::Missing => None,
    }
}

fn valid_legacy_action_url(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= NOTIFICATION_ACTION_URL_MAX
        && value.starts_with('/')
        && !value.starts_with("//")
        && !value.contains('\\')
        && !value.contains("://")
        && value
            .chars()
            .all(|character| !character.is_control() && !character.is_whitespace())
}

fn valid_legacy_image_url(value: &str) -> bool {
    if value.is_empty()
        || value.len() > NOTIFICATION_ACTION_URL_MAX
        || value
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
        || value.contains('\\')
        || value.starts_with("//")
    {
        return false;
    }
    if value.starts_with('/') {
        return true;
    }
    reqwest::Url::parse(value).is_ok_and(|url| {
        url.scheme() == "https"
            && url.host_str().is_some()
            && url.username().is_empty()
            && url.password().is_none()
    })
}

fn legacy_notification_from_wire(
    item: &NotificationWire,
    owner: &str,
) -> Option<LegacyNotification> {
    let id = legacy_notification_id(&item.id)?;
    let wallet_address = required_nullable_ref(&item.user_id)
        .map(|value| value.to_string())
        .or_else(|| {
            if item.recipient.eq_ignore_ascii_case("all") {
                Some(owner.to_string())
            } else {
                Some(item.recipient.clone())
            }
        })?;
    let title = required_nullable_ref(&item.title)
        .or_else(|| required_nullable_ref(&item.subject))
        .filter(|value| !value.is_empty())
        .cloned()
        .unwrap_or_else(|| "Notification".to_string());
    let message = if item.body.is_empty() {
        title.clone()
    } else {
        item.body.clone()
    };
    if title.is_empty()
        || title.chars().count() > 200
        || title.chars().any(char::is_control)
        || message.is_empty()
        || message.chars().count() > 1_000
        || message.chars().any(char::is_control)
    {
        return None;
    }
    let data = match required_nullable_ref(&item.data) {
        Some(value) if value.is_object() => match serde_json::to_value(value) {
            Ok(value) => Some(value),
            Err(_) => return None,
        },
        Some(_) => return None,
        None => None,
    };
    let image_url = match data.as_ref().and_then(|value| value.get("image_url")) {
        Some(value) => {
            let value = value.as_str()?;
            if !valid_legacy_image_url(value) {
                return None;
            }
            Some(value.to_owned())
        }
        None => None,
    };
    let action_url = required_nullable_ref(&item.action_url).cloned();
    if action_url
        .as_deref()
        .is_some_and(|value| !valid_legacy_action_url(value))
    {
        return None;
    }
    let read_at = required_nullable_ref(&item.read_at).map(DateTime::to_rfc3339);
    Some(LegacyNotification {
        id,
        wallet_address,
        notification_type: legacy_notification_type(
            required_nullable_ref(&item.notification_type).map(String::as_str),
        ),
        title,
        message,
        data,
        priority: legacy_notification_priority(
            required_nullable_ref(&item.priority).map(String::as_str),
        ),
        timestamp: item.created_at.to_rfc3339(),
        expires_at: required_nullable_ref(&item.expires_at).map(DateTime::to_rfc3339),
        read_at: read_at.clone(),
        clicked_at: required_nullable_ref(&item.clicked_at).map(DateTime::to_rfc3339),
        // `sent_at` records target/provider acceptance, not end-user delivery.
        // Keep the source field absent until a provider-delivered event is
        // durably reconciled; acceptance must never be presented as delivery.
        delivered_at: None,
        action_url,
        image_url,
        read: read_at.is_some(),
    })
}

fn legacy_notification_query(
    raw_query: Option<&str>,
    owner: &str,
) -> Result<(NotificationListQuery, u32, u16), ()> {
    let Some(raw_query) = raw_query.filter(|value| !value.is_empty()) else {
        return Ok((NotificationListQuery::default(), 1, 50));
    };
    let url =
        reqwest::Url::parse(&format!("https://frontend.invalid/?{raw_query}")).map_err(|_| ())?;
    let mut normalized = url::form_urlencoded::Serializer::new(String::new());
    let mut saw_wallet = false;
    for (key, value) in url.query_pairs() {
        if key == "wallet_address" {
            if saw_wallet || !value.eq_ignore_ascii_case(owner) {
                return Err(());
            }
            saw_wallet = true;
            continue;
        }
        normalized.append_pair(&key, &value);
    }
    let query_string = normalized.finish();
    let query = NotificationListQuery::from_raw_query(
        (!query_string.is_empty()).then_some(query_string.as_str()),
    )?;
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);
    let page = offset / u32::from(limit) + 1;
    Ok((query, page, limit))
}

fn legacy_notification_error(status: StatusCode, code: &'static str) -> Response {
    private_notification_response(safe_error(status, code))
}

pub async fn legacy_notifications_api(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    raw_query: RawQuery,
) -> Response {
    let (token, user) = match verified_bearer_and_user(&state, &headers).await {
        Ok(session) => session,
        Err(response) => return response,
    };
    let (query, page, limit) =
        match legacy_notification_query(raw_query.0.as_deref(), &user.wallet_address) {
            Ok(query) => query,
            Err(()) => {
                return legacy_notification_error(
                    StatusCode::BAD_REQUEST,
                    "invalid_notification_query",
                );
            }
        };
    let request_id = notification_request_id(&headers);
    let list = load_owner_notifications(
        state.notification.as_ref(),
        &token,
        &user.wallet_address,
        &query,
        &request_id,
    )
    .await;
    let unread = load_notification_unread_count_with_query(
        state.notification.as_ref(),
        &token,
        &request_id,
        Some(&query),
    )
    .await;
    let payload = match list {
        NotificationListLoadOutcome::Ready(value) | NotificationListLoadOutcome::Empty(value) => {
            match serde_json::from_value::<NotificationListWire>(value) {
                Ok(value) => value,
                Err(_) => {
                    return legacy_notification_error(
                        StatusCode::BAD_GATEWAY,
                        "malformed_notification_response",
                    );
                }
            }
        }
        NotificationListLoadOutcome::Unavailable(NotificationListUnavailable::Unauthorized) => {
            return legacy_notification_error(
                StatusCode::UNAUTHORIZED,
                "notification_upstream_unauthorized",
            );
        }
        NotificationListLoadOutcome::Unavailable(NotificationListUnavailable::Dependency) => {
            return legacy_notification_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "notification_upstream_unavailable",
            );
        }
        NotificationListLoadOutcome::Unavailable(NotificationListUnavailable::UpstreamFailed)
        | NotificationListLoadOutcome::Malformed => {
            return legacy_notification_error(
                StatusCode::BAD_GATEWAY,
                "malformed_notification_response",
            );
        }
    };
    let unread_count = match unread {
        NotificationUnreadLoadOutcome::Ready(count) => count,
        NotificationUnreadLoadOutcome::DependencyUnavailable => {
            return legacy_notification_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "notification_upstream_unavailable",
            );
        }
        NotificationUnreadLoadOutcome::UpstreamFailed
        | NotificationUnreadLoadOutcome::Malformed => {
            return legacy_notification_error(
                StatusCode::BAD_GATEWAY,
                "malformed_notification_response",
            );
        }
    };
    let notifications = match payload
        .items
        .iter()
        .map(|item| legacy_notification_from_wire(item, &user.wallet_address))
        .collect::<Option<Vec<_>>>()
    {
        Some(items) => items,
        None => {
            return legacy_notification_error(
                StatusCode::BAD_GATEWAY,
                "legacy_notification_projection_failed",
            );
        }
    };
    let total_count = payload.total as u64;
    let total_pages = if total_count == 0 {
        0
    } else {
        total_count.div_ceil(u64::from(limit))
    };
    private_notification_response(
        Json(LegacyNotificationsResponse {
            success: true,
            data: LegacyNotificationsData {
                notifications,
                total_count,
                unread_count,
                page,
                limit,
                total_pages,
            },
            api_version: "v1",
            access_level: "auth",
        })
        .into_response(),
    )
}

pub async fn legacy_notification_unread_count(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let token = match verified_bearer(&state, &headers).await {
        Ok(token) => token,
        Err(response) => return response,
    };
    let request_id = notification_request_id(&headers);
    match load_notification_unread_count(state.notification.as_ref(), &token, &request_id).await {
        NotificationUnreadLoadOutcome::Ready(count) => private_notification_response(
            Json(serde_json::json!({"unread_count": count})).into_response(),
        ),
        NotificationUnreadLoadOutcome::DependencyUnavailable => legacy_notification_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "notification_upstream_unavailable",
        ),
        NotificationUnreadLoadOutcome::UpstreamFailed
        | NotificationUnreadLoadOutcome::Malformed => {
            legacy_notification_error(StatusCode::BAD_GATEWAY, "malformed_notification_response")
        }
    }
}

async fn legacy_mutation_response(response: Response, message: &'static str) -> Response {
    let status = response.status();
    if status.is_success() {
        return private_notification_response(
            Json(serde_json::json!({"success": true, "message": message})).into_response(),
        );
    }
    legacy_notification_error(status, "notification_mutation_failed")
}

pub async fn legacy_notification_read(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    AxPath(id): AxPath<String>,
) -> Response {
    legacy_mutation_response(
        notification_read(
            State(state),
            headers,
            AxPath(target_notification_id_from_legacy(&id)),
        )
        .await,
        "Notification marked as read",
    )
    .await
}

pub async fn legacy_notification_unread(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    AxPath(id): AxPath<String>,
) -> Response {
    legacy_mutation_response(
        notification_unread(
            State(state),
            headers,
            AxPath(target_notification_id_from_legacy(&id)),
        )
        .await,
        "Notification marked as unread",
    )
    .await
}

pub async fn legacy_notification_acknowledge(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    AxPath(id): AxPath<String>,
) -> Response {
    legacy_mutation_response(
        notification_acknowledge(
            State(state),
            headers,
            AxPath(target_notification_id_from_legacy(&id)),
        )
        .await,
        "Notification acknowledged",
    )
    .await
}

pub async fn legacy_notification_delete(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    AxPath(id): AxPath<String>,
) -> Response {
    legacy_mutation_response(
        notification_delete(
            State(state),
            headers,
            AxPath(target_notification_id_from_legacy(&id)),
        )
        .await,
        "Notification deleted successfully",
    )
    .await
}

pub async fn legacy_notification_mark_all(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    legacy_bulk_mutation(
        state,
        headers,
        "/api/v1/notification/mark-all-read",
        "marked",
        "updated_count",
    )
    .await
}

pub async fn legacy_notification_clear_all(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    legacy_bulk_mutation(
        state,
        headers,
        "/api/v1/notification/clear-all",
        "deleted",
        "deleted_count",
    )
    .await
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyQuietHours {
    enabled: bool,
    start_time: String,
    end_time: String,
    #[serde(default)]
    timezone: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyPreferencesPatch {
    #[serde(default)]
    email_enabled: Option<bool>,
    #[serde(default)]
    push_enabled: Option<bool>,
    #[serde(default)]
    sms_enabled: Option<bool>,
    #[serde(default)]
    types: Option<serde_json::Value>,
    #[serde(default)]
    priority_filter: Option<String>,
    #[serde(default)]
    quiet_hours: Option<LegacyQuietHours>,
    #[serde(default)]
    timezone: Option<String>,
}

fn legacy_preferences_projection(
    preferences: &NotificationPreferencesResponse,
) -> serde_json::Value {
    let channel = |name: &str| {
        preferences
            .channels
            .get(name)
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
    };
    let quiet_hours = preferences.quiet_hours.as_ref().and_then(|value| {
        let object = value.as_object()?;
        let enabled = object.get("enabled")?.as_bool()?;
        let start = object.get("start")?.as_str()?;
        let end = object.get("end")?.as_str()?;
        Some(serde_json::json!({
            "enabled": enabled,
            "start_time": start,
            "end_time": end,
            "timezone": preferences.timezone.as_deref().unwrap_or("UTC"),
        }))
    });
    let type_enabled = |name: &str| {
        preferences
            .channels
            .get("types")
            .and_then(|value| value.get(name))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true)
    };
    let types = serde_json::json!({
        "system": type_enabled("system"),
        "security": type_enabled("security"),
        "permission": type_enabled("permission"),
        "wallet_management": type_enabled("wallet_management"),
        "wallet": type_enabled("wallet"),
        "payment": type_enabled("payment"),
        "general": type_enabled("general"),
        "announcement": type_enabled("announcement"),
        "advertisement": type_enabled("advertisement"),
        "chat": type_enabled("chat"),
    });
    let priority_filter = preferences
        .channels
        .get("priority_filter")
        .and_then(serde_json::Value::as_str)
        .filter(|value| valid_legacy_notification_priority(value))
        .unwrap_or("normal");
    serde_json::json!({
        "email_enabled": channel("email"),
        "push_enabled": channel("push"),
        "sms_enabled": false,
        "types": types,
        "priority_filter": priority_filter,
        "quiet_hours": quiet_hours,
    })
}

pub async fn legacy_notification_preferences_get(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let token = match verified_bearer(&state, &headers).await {
        Ok(token) => token,
        Err(response) => return response,
    };
    let request_id = notification_request_id(&headers);
    match load_notification_preferences(state.notification.as_ref(), &token, &request_id).await {
        NotificationPreferencesLoadOutcome::Ready(value) => {
            match serde_json::from_value::<NotificationPreferencesResponse>(value) {
                Ok(preferences) => private_notification_response(
                    Json(serde_json::json!({
                        "success": true,
                        "data": legacy_preferences_projection(&preferences),
                        "api_version": "v1",
                        "access_level": "auth",
                    }))
                    .into_response(),
                ),
                Err(_) => legacy_notification_error(
                    StatusCode::BAD_GATEWAY,
                    "malformed_notification_preferences_response",
                ),
            }
        }
        NotificationPreferencesLoadOutcome::Error(NotificationPreferencesLoadError::Malformed) => {
            legacy_notification_error(
                StatusCode::BAD_GATEWAY,
                "malformed_notification_preferences_response",
            )
        }
        NotificationPreferencesLoadOutcome::Error(
            NotificationPreferencesLoadError::DependencyUnavailable,
        ) => legacy_notification_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "notification_preferences_upstream_unavailable",
        ),
        NotificationPreferencesLoadOutcome::Error(
            NotificationPreferencesLoadError::UpstreamFailed,
        ) => {
            legacy_notification_error(StatusCode::BAD_GATEWAY, "notification_preferences_rejected")
        }
    }
}

pub async fn legacy_notification_preferences_put(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    if !notification_mutation_origin_allowed(&parts.headers) {
        return legacy_notification_error(
            StatusCode::FORBIDDEN,
            "notification_mutation_origin_rejected",
        );
    }
    let body = match axum::body::to_bytes(body, NOTIFICATION_PREFERENCES_BODY_MAX).await {
        Ok(body) => body,
        Err(_) => {
            return legacy_notification_error(
                StatusCode::PAYLOAD_TOO_LARGE,
                "notification_preferences_body_too_large",
            );
        }
    };
    let patch = match serde_json::from_slice::<LegacyPreferencesPatch>(&body) {
        Ok(patch) => patch,
        Err(_) => {
            return legacy_notification_error(
                StatusCode::BAD_REQUEST,
                "invalid_notification_preferences",
            );
        }
    };
    // SMS is not a target channel.  A false legacy value is harmless and can
    // be accepted, while an attempt to enable it fails closed rather than
    // claiming unsupported delivery semantics.
    if patch.sms_enabled == Some(true) {
        return legacy_notification_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "legacy_notification_preferences_unsupported",
        );
    }
    let (token, _) = match verified_bearer_and_user(&state, &parts.headers).await {
        Ok(session) => session,
        Err(response) => return response,
    };
    let request_id = notification_request_id(&parts.headers);
    let current =
        match load_notification_preferences(state.notification.as_ref(), &token, &request_id).await
        {
            NotificationPreferencesLoadOutcome::Ready(value) => {
                match serde_json::from_value::<NotificationPreferencesResponse>(value) {
                    Ok(value) => value,
                    Err(_) => {
                        return legacy_notification_error(
                            StatusCode::BAD_GATEWAY,
                            "malformed_notification_preferences_response",
                        );
                    }
                }
            }
            NotificationPreferencesLoadOutcome::Error(
                NotificationPreferencesLoadError::DependencyUnavailable,
            ) => {
                return legacy_notification_error(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "notification_preferences_upstream_unavailable",
                );
            }
            NotificationPreferencesLoadOutcome::Error(_) => {
                return legacy_notification_error(
                    StatusCode::BAD_GATEWAY,
                    "notification_preferences_rejected",
                );
            }
        };
    let current_channel = |name: &str| {
        current
            .channels
            .get(name)
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
    };
    let current_type = |name: &str| {
        current
            .channels
            .get("types")
            .and_then(|value| value.get(name))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true)
    };
    let types = match patch.types {
        Some(value) if valid_legacy_type_preferences(&value) => value,
        Some(_) => {
            return legacy_notification_error(
                StatusCode::BAD_REQUEST,
                "invalid_notification_preferences",
            );
        }
        None => serde_json::json!({
            "system": current_type("system"),
            "security": current_type("security"),
            "permission": current_type("permission"),
            "wallet_management": current_type("wallet_management"),
            "wallet": current_type("wallet"),
            "payment": current_type("payment"),
            "general": current_type("general"),
            "announcement": current_type("announcement"),
            "advertisement": current_type("advertisement"),
            "chat": current_type("chat"),
        }),
    };
    let priority_filter = match patch.priority_filter {
        Some(value) if valid_legacy_notification_priority(&value) => value,
        Some(_) => {
            return legacy_notification_error(
                StatusCode::BAD_REQUEST,
                "invalid_notification_preferences",
            );
        }
        None => current
            .channels
            .get("priority_filter")
            .and_then(serde_json::Value::as_str)
            .filter(|value| valid_legacy_notification_priority(value))
            .unwrap_or("normal")
            .to_owned(),
    };
    let quiet_timezone = patch
        .quiet_hours
        .as_ref()
        .and_then(|value| value.timezone.clone());
    let quiet_hours = match patch.quiet_hours {
        Some(value)
            if valid_clock(&value.start_time)
                && valid_clock(&value.end_time)
                && value
                    .timezone
                    .as_deref()
                    .is_none_or(|timezone| !timezone.is_empty() && timezone.len() <= 64) =>
        {
            Some(serde_json::json!({
                "enabled": value.enabled,
                "start": value.start_time,
                "end": value.end_time,
            }))
        }
        Some(_) => {
            return legacy_notification_error(
                StatusCode::BAD_REQUEST,
                "invalid_notification_preferences",
            );
        }
        None => current.quiet_hours.clone(),
    };
    let timezone = patch
        .timezone
        .or(quiet_timezone)
        .or(current.timezone.clone());
    let target = NotificationPreferencesRequest {
        channels: serde_json::json!({
            "email": patch.email_enabled.unwrap_or_else(|| current_channel("email")),
            "in_app": current_channel("in_app"),
            "push": patch.push_enabled.unwrap_or_else(|| current_channel("push")),
            "types": types,
            "priority_filter": priority_filter,
        }),
        quiet_hours,
        timezone,
    };
    let body = match serde_json::to_vec(&target) {
        Ok(body) => Body::from(body),
        Err(_) => {
            return legacy_notification_error(
                StatusCode::BAD_REQUEST,
                "invalid_notification_preferences",
            );
        }
    };
    let response =
        notification_preferences_put(State(state), Request::from_parts(parts, body)).await;
    if response.status().is_success() {
        private_notification_response(
            Json(serde_json::json!({
                "success": true,
                "message": "Notification preferences updated",
            }))
            .into_response(),
        )
    } else {
        legacy_notification_error(response.status(), "notification_preferences_rejected")
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
    if !notification_mutation_origin_allowed(&headers) {
        return private_notification_response(safe_error(
            StatusCode::FORBIDDEN,
            "notification_mutation_origin_rejected",
        ));
    }
    let token = match verified_bearer(&state, &headers).await {
        Ok(token) => token,
        Err(response) => return response,
    };
    let url = format!(
        "{}/api/v1/notification/{}/read",
        state.notification_url.trim_end_matches('/'),
        id
    );
    let request_id = notification_request_id(&headers);
    match state
        .notification
        .clone_for_bearer()
        .post(&url)
        .bearer_auth(&token)
        .header("x-request-id", request_id.0.as_str())
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

fn legacy_bulk_count(body: &[u8], target_key: &'static str) -> Result<u64, ()> {
    let value = serde_json::from_slice::<serde_json::Value>(body).map_err(|_| ())?;
    let object = value.as_object().ok_or(())?;
    if object.len() != 1 {
        return Err(());
    }
    object
        .get(target_key)
        .and_then(serde_json::Value::as_u64)
        .ok_or(())
}

async fn legacy_bulk_mutation(
    state: AppState,
    headers: axum::http::HeaderMap,
    target_path: &'static str,
    target_key: &'static str,
    source_key: &'static str,
) -> Response {
    if !notification_mutation_origin_allowed(&headers) {
        return legacy_notification_error(
            StatusCode::FORBIDDEN,
            "notification_mutation_origin_rejected",
        );
    }
    let token = match verified_bearer(&state, &headers).await {
        Ok(token) => token,
        Err(response) => return response,
    };
    let request_id = notification_request_id(&headers);
    let url = format!(
        "{}{target_path}",
        state.notification_url.trim_end_matches('/')
    );
    let response = match state
        .notification
        .clone_for_bearer()
        .post(url)
        .bearer_auth(token)
        .header("x-request-id", request_id.0)
        .header(header::ACCEPT, HeaderValue::from_static("application/json"))
        .json(&serde_json::json!({}))
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => {
            return legacy_notification_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "notification_mutation_unavailable",
            );
        }
    };
    if response.status() != StatusCode::OK {
        return legacy_notification_error(response.status(), "notification_mutation_failed");
    }
    if !valid_notification_json_content_type(response.headers()) {
        return legacy_notification_error(
            StatusCode::BAD_GATEWAY,
            "malformed_notification_mutation_response",
        );
    }
    let body =
        match read_notification_body_limited(response, NOTIFICATION_BULK_MUTATION_BODY_MAX).await {
            Ok(body) => body,
            Err(_) => {
                return legacy_notification_error(
                    StatusCode::BAD_GATEWAY,
                    "malformed_notification_mutation_response",
                );
            }
        };
    let count = match legacy_bulk_count(&body, target_key) {
        Ok(count) => count,
        Err(()) => {
            return legacy_notification_error(
                StatusCode::BAD_GATEWAY,
                "malformed_notification_mutation_response",
            );
        }
    };
    let mut payload = serde_json::Map::new();
    payload.insert("success".into(), serde_json::Value::Bool(true));
    payload.insert(source_key.into(), serde_json::json!(count));
    private_notification_response(Json(serde_json::Value::Object(payload)).into_response())
}

pub async fn notification_unread(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    AxPath(id): AxPath<String>,
) -> Response {
    if !notification_mutation_origin_allowed(&headers) {
        return private_notification_response(safe_error(
            StatusCode::FORBIDDEN,
            "notification_mutation_origin_rejected",
        ));
    }
    let token = match verified_bearer(&state, &headers).await {
        Ok(token) => token,
        Err(response) => return response,
    };
    let url = format!(
        "{}/api/v1/notification/{}/unread",
        state.notification_url.trim_end_matches('/'),
        id
    );
    let request_id = notification_request_id(&headers);
    match state
        .notification
        .clone_for_bearer()
        .post(&url)
        .bearer_auth(&token)
        .header("x-request-id", request_id.0.as_str())
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

pub async fn notification_acknowledge(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    AxPath(id): AxPath<String>,
) -> Response {
    if !notification_mutation_origin_allowed(&headers) {
        return private_notification_response(safe_error(
            StatusCode::FORBIDDEN,
            "notification_mutation_origin_rejected",
        ));
    }
    let token = match verified_bearer(&state, &headers).await {
        Ok(token) => token,
        Err(response) => return response,
    };
    let url = format!(
        "{}/api/v1/notification/{}/acknowledge",
        state.notification_url.trim_end_matches('/'),
        id
    );
    let request_id = notification_request_id(&headers);
    match state
        .notification
        .clone_for_bearer()
        .put(&url)
        .bearer_auth(&token)
        .header("x-request-id", request_id.0.as_str())
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

async fn notification_engagement_event(
    state: AppState,
    headers: axum::http::HeaderMap,
    id: String,
    event: &'static str,
) -> Response {
    if !notification_mutation_origin_allowed(&headers) {
        return private_notification_response(safe_error(
            StatusCode::FORBIDDEN,
            "notification_mutation_origin_rejected",
        ));
    }
    let token = match verified_bearer(&state, &headers).await {
        Ok(token) => token,
        Err(response) => return response,
    };
    let url = format!(
        "{}/api/v1/notification/{}/{}",
        state.notification_url.trim_end_matches('/'),
        id,
        event
    );
    let request_id = notification_request_id(&headers);
    match state
        .notification
        .clone_for_bearer()
        .post(&url)
        .bearer_auth(&token)
        .header("x-request-id", request_id.0.as_str())
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

pub async fn notification_click(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    AxPath(id): AxPath<String>,
) -> Response {
    notification_engagement_event(state, headers, id, "click").await
}

pub async fn notification_dismiss(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    AxPath(id): AxPath<String>,
) -> Response {
    notification_engagement_event(state, headers, id, "dismiss").await
}

pub async fn notification_delete(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    AxPath(id): AxPath<String>,
) -> Response {
    if !notification_mutation_origin_allowed(&headers) {
        return private_notification_response(safe_error(
            StatusCode::FORBIDDEN,
            "notification_mutation_origin_rejected",
        ));
    }
    let token = match verified_bearer(&state, &headers).await {
        Ok(token) => token,
        Err(response) => return response,
    };
    let url = format!(
        "{}/api/v1/notification/{}",
        state.notification_url.trim_end_matches('/'),
        id
    );
    let request_id = notification_request_id(&headers);
    match state
        .notification
        .clone_for_bearer()
        .delete(&url)
        .bearer_auth(&token)
        .header("x-request-id", request_id.0.as_str())
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
    if !notification_mutation_origin_allowed(&headers) {
        return private_notification_response(safe_error(
            StatusCode::FORBIDDEN,
            "notification_mutation_origin_rejected",
        ));
    }
    let token = match verified_bearer(&state, &headers).await {
        Ok(token) => token,
        Err(response) => return response,
    };
    let url = format!(
        "{}/api/v1/notification/mark-all-read",
        state.notification_url.trim_end_matches('/')
    );
    let request_id = notification_request_id(&headers);
    match state
        .notification
        .clone_for_bearer()
        .post(&url)
        .bearer_auth(&token)
        .header("x-request-id", request_id.0.as_str())
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
    if !notification_mutation_origin_allowed(&headers) {
        return private_notification_response(safe_error(
            StatusCode::FORBIDDEN,
            "notification_mutation_origin_rejected",
        ));
    }
    let token = match verified_bearer(&state, &headers).await {
        Ok(token) => token,
        Err(response) => return response,
    };
    let url = format!(
        "{}/api/v1/notification/clear-all",
        state.notification_url.trim_end_matches('/')
    );
    let request_id = notification_request_id(&headers);
    match state
        .notification
        .clone_for_bearer()
        .post(&url)
        .bearer_auth(&token)
        .header("x-request-id", request_id.0.as_str())
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct NotificationPreferencesRequest {
    channels: serde_json::Value,
    #[serde(default)]
    quiet_hours: Option<serde_json::Value>,
    #[serde(default)]
    timezone: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct NotificationPreferencesResponse {
    channels: serde_json::Value,
    quiet_hours: Option<serde_json::Value>,
    timezone: Option<String>,
    updated_at: Option<DateTime<chrono::Utc>>,
}

fn parse_notification_preferences_form(body: &[u8]) -> Result<NotificationPreferencesRequest, ()> {
    let mut fields = BTreeMap::new();
    for (key, value) in url::form_urlencoded::parse(body) {
        if key.len() > 64
            || value.len() > 256
            || fields
                .insert(key.into_owned(), value.into_owned())
                .is_some()
        {
            return Err(());
        }
    }
    const ALLOWED: &[&str] = &[
        "email",
        "in_app",
        "push",
        "quiet_enabled",
        "quiet_start",
        "quiet_end",
        "timezone",
    ];
    if fields.keys().any(|key| !ALLOWED.contains(&key.as_str())) {
        return Err(());
    }
    let bool_field = |name: &str| match fields.get(name).map(String::as_str) {
        Some("true") => Ok(true),
        Some("false") => Ok(false),
        _ => Err(()),
    };
    let email = bool_field("email")?;
    let in_app = bool_field("in_app")?;
    let push = bool_field("push")?;
    let quiet_enabled = bool_field("quiet_enabled")?;
    let quiet_start = fields.get("quiet_start").ok_or(())?.clone();
    let quiet_end = fields.get("quiet_end").ok_or(())?.clone();
    let timezone = fields
        .get("timezone")
        .cloned()
        .filter(|value| !value.is_empty());
    let request = NotificationPreferencesRequest {
        channels: serde_json::json!({
            "email": email,
            "in_app": in_app,
            "push": push,
        }),
        quiet_hours: Some(serde_json::json!({
            "enabled": quiet_enabled,
            "start": quiet_start,
            "end": quiet_end,
        })),
        timezone,
    };
    validate_notification_preferences(&request)?;
    Ok(request)
}

fn same_origin_preferences_form(headers: &axum::http::HeaderMap) -> bool {
    let Some(origin) = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
    else {
        return false;
    };
    let Some(host) = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
    else {
        return false;
    };
    let origin_host = origin
        .strip_prefix("https://")
        .or_else(|| origin.strip_prefix("http://"))
        .and_then(|value| value.split('/').next())
        .unwrap_or("");
    if origin_host.is_empty() || origin_host != host {
        return false;
    }
    headers
        .get("sec-fetch-site")
        .and_then(|value| value.to_str().ok())
        .is_none_or(|value| matches!(value, "same-origin" | "same-site"))
}

/// Cookie-backed browser mutations must originate from this BFF. Explicit
/// bearer callers remain supported for non-browser integrations; their
/// cryptographic token is still verified before any upstream request.
fn notification_mutation_origin_allowed(headers: &axum::http::HeaderMap) -> bool {
    headers.contains_key(header::AUTHORIZATION) || same_origin_preferences_form(headers)
}

fn watchlist_mutation_origin_allowed(headers: &axum::http::HeaderMap) -> bool {
    headers.contains_key(header::AUTHORIZATION) || same_origin_preferences_form(headers)
}

fn validate_notification_preferences(request: &NotificationPreferencesRequest) -> Result<(), ()> {
    let bounded_object = |value: &serde_json::Value| {
        value.is_object()
            && serde_json::to_vec(value)
                .is_ok_and(|encoded| encoded.len() <= NOTIFICATION_PREFERENCES_BODY_MAX)
    };
    if !bounded_object(&request.channels)
        || !valid_channel_preferences(&request.channels)
        || request
            .quiet_hours
            .as_ref()
            .is_some_and(|value| !bounded_object(value) || !valid_quiet_hours(value))
        || request.timezone.as_deref().is_some_and(|value| {
            value.is_empty() || value.chars().count() > 64 || value.chars().any(char::is_control)
        })
    {
        return Err(());
    }
    Ok(())
}

fn valid_channel_preferences(value: &serde_json::Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    object.iter().all(|(key, value)| match key.as_str() {
        "email" | "in_app" | "push" => value.is_boolean(),
        "types" => valid_legacy_type_preferences(value),
        "priority_filter" => value
            .as_str()
            .is_some_and(valid_legacy_notification_priority),
        _ => false,
    })
}

const LEGACY_NOTIFICATION_TYPES: &[&str] = &[
    "system",
    "security",
    "permission",
    "wallet_management",
    "wallet",
    "payment",
    "general",
    "announcement",
    "advertisement",
    "chat",
];

fn valid_legacy_type_preferences(value: &serde_json::Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    object
        .iter()
        .all(|(key, value)| LEGACY_NOTIFICATION_TYPES.contains(&key.as_str()) && value.is_boolean())
}

fn valid_legacy_notification_priority(value: &str) -> bool {
    matches!(value, "low" | "normal" | "high" | "critical" | "urgent")
}

fn valid_clock(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 5
        || bytes[2] != b':'
        || !bytes[0].is_ascii_digit()
        || !bytes[1].is_ascii_digit()
        || !bytes[3].is_ascii_digit()
        || !bytes[4].is_ascii_digit()
    {
        return false;
    }
    let hour = (bytes[0] - b'0') * 10 + bytes[1] - b'0';
    let minute = (bytes[3] - b'0') * 10 + bytes[4] - b'0';
    hour < 24 && minute < 60
}

fn valid_quiet_hours(value: &serde_json::Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    let Some(start) = object.get("start").and_then(serde_json::Value::as_str) else {
        return false;
    };
    let Some(end) = object.get("end").and_then(serde_json::Value::as_str) else {
        return false;
    };
    valid_clock(start)
        && valid_clock(end)
        && object.iter().all(|(key, value)| {
            matches!(key.as_str(), "start" | "end" | "enabled")
                && (matches!(key.as_str(), "start" | "end") && value.is_string()
                    || key == "enabled" && value.is_boolean())
        })
}

fn notification_preferences_upstream_error(status: StatusCode) -> Response {
    match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
            safe_error(status, "notification_preferences_unauthorized")
        }
        StatusCode::BAD_REQUEST => {
            safe_error(StatusCode::BAD_REQUEST, "invalid_notification_preferences")
        }
        _ if status.is_client_error() => {
            safe_error(StatusCode::BAD_GATEWAY, "notification_preferences_rejected")
        }
        _ => safe_error(
            StatusCode::BAD_GATEWAY,
            "notification_preferences_upstream_failed",
        ),
    }
}

#[allow(
    clippy::result_large_err,
    reason = "BFF upstream validation preserves the complete safe HTTP error response"
)]
async fn read_notification_preferences_response(
    response: reqwest::Response,
) -> Result<NotificationPreferencesResponse, Response> {
    if response.status() != StatusCode::OK {
        return Err(notification_preferences_upstream_error(response.status()));
    }
    if !valid_notification_json_content_type(response.headers()) {
        return Err(safe_error(
            StatusCode::BAD_GATEWAY,
            "malformed_notification_preferences_response",
        ));
    }
    let body = read_notification_body_limited(response, NOTIFICATION_PREFERENCES_RESPONSE_MAX)
        .await
        .map_err(|error| match error {
            NotificationBodyReadError::TooLarge => safe_error(
                StatusCode::BAD_GATEWAY,
                "malformed_notification_preferences_response",
            ),
            NotificationBodyReadError::Transport => safe_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "notification_preferences_upstream_unavailable",
            ),
        })?;
    let preferences =
        serde_json::from_slice::<NotificationPreferencesResponse>(&body).map_err(|_| {
            safe_error(
                StatusCode::BAD_GATEWAY,
                "malformed_notification_preferences_response",
            )
        })?;
    if validate_notification_preferences(&NotificationPreferencesRequest {
        channels: preferences.channels.clone(),
        quiet_hours: preferences.quiet_hours.clone(),
        timezone: preferences.timezone.clone(),
    })
    .is_err()
    {
        return Err(safe_error(
            StatusCode::BAD_GATEWAY,
            "malformed_notification_preferences_response",
        ));
    }
    Ok(preferences)
}

fn notification_preferences_form_redirect(state: &'static str) -> Response {
    let state = match state {
        "saved" => "saved",
        "error" => "error",
        _ => "error",
    };
    let location = match state {
        "saved" => "/account?preferences=saved",
        "error" => "/account?preferences=error",
        _ => unreachable!("flash state is normalized above"),
    };
    let mut response = Redirect::to(location).into_response();
    let cookie = format!(
        "{}={state}; Path=/account; Max-Age=30; HttpOnly; SameSite=Lax",
        NOTIFICATION_PREFERENCES_FLASH_COOKIE
    );
    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).expect("flash cookie value is static and valid"),
    );
    response
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NotificationPreferencesLoadError {
    DependencyUnavailable,
    UpstreamFailed,
    Malformed,
}

#[derive(Debug, PartialEq)]
pub(crate) enum NotificationPreferencesLoadOutcome {
    Ready(serde_json::Value),
    Error(NotificationPreferencesLoadError),
}

/// Load the authenticated owner's preferences for SSR. This deliberately
/// shares the exact response DTO and validation used by the BFF route, so a
/// malformed or owner-service failure can never become an empty/default UI.
pub(crate) async fn load_notification_preferences(
    client: &ServiceClient,
    bearer: &str,
    request_id: &NotificationRequestId,
) -> NotificationPreferencesLoadOutcome {
    let url = format!(
        "{}/api/v1/notification/preferences",
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
        Err(_) => {
            return NotificationPreferencesLoadOutcome::Error(
                NotificationPreferencesLoadError::DependencyUnavailable,
            );
        }
    };
    if !response.status().is_success() {
        return NotificationPreferencesLoadOutcome::Error(
            if response.status().is_server_error()
                || matches!(response.status().as_u16(), 408 | 425 | 429)
            {
                NotificationPreferencesLoadError::DependencyUnavailable
            } else {
                NotificationPreferencesLoadError::UpstreamFailed
            },
        );
    }
    if !valid_notification_json_content_type(response.headers()) {
        return NotificationPreferencesLoadOutcome::Error(
            NotificationPreferencesLoadError::Malformed,
        );
    }
    let body = match read_notification_body_limited(response, NOTIFICATION_PREFERENCES_RESPONSE_MAX)
        .await
    {
        Ok(body) => body,
        Err(NotificationBodyReadError::TooLarge | NotificationBodyReadError::Transport) => {
            return NotificationPreferencesLoadOutcome::Error(
                NotificationPreferencesLoadError::Malformed,
            );
        }
    };
    let preferences = match serde_json::from_slice::<NotificationPreferencesResponse>(&body) {
        Ok(preferences)
            if validate_notification_preferences(&NotificationPreferencesRequest {
                channels: preferences.channels.clone(),
                quiet_hours: preferences.quiet_hours.clone(),
                timezone: preferences.timezone.clone(),
            })
            .is_ok() =>
        {
            preferences
        }
        Ok(_) | Err(_) => {
            return NotificationPreferencesLoadOutcome::Error(
                NotificationPreferencesLoadError::Malformed,
            );
        }
    };
    match serde_json::to_value(preferences) {
        Ok(value) => NotificationPreferencesLoadOutcome::Ready(value),
        Err(_) => {
            NotificationPreferencesLoadOutcome::Error(NotificationPreferencesLoadError::Malformed)
        }
    }
}

pub async fn notification_preferences_get(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let token = match verified_bearer(&state, &headers).await {
        Ok(token) => token,
        Err(response) => return response,
    };
    let request_id = notification_request_id(&headers);
    let url = format!(
        "{}/api/v1/notification/preferences",
        state.notification_url.trim_end_matches('/')
    );
    let response = match state
        .notification
        .clone_for_bearer()
        .get(url)
        .bearer_auth(token)
        .header("x-request-id", request_id.0)
        .header(header::ACCEPT, HeaderValue::from_static("application/json"))
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => {
            return private_notification_response(safe_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "notification_preferences_upstream_unavailable",
            ));
        }
    };
    match read_notification_preferences_response(response).await {
        Ok(preferences) => private_notification_response(Json(preferences).into_response()),
        Err(response) => private_notification_response(response),
    }
}

pub async fn notification_preferences_put(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    if !notification_mutation_origin_allowed(&parts.headers) {
        return private_notification_response(safe_error(
            StatusCode::FORBIDDEN,
            "notification_mutation_origin_rejected",
        ));
    }
    let token = match verified_bearer(&state, &parts.headers).await {
        Ok(token) => token,
        Err(response) => return response,
    };
    let body = match axum::body::to_bytes(body, NOTIFICATION_PREFERENCES_BODY_MAX).await {
        Ok(body) => body,
        Err(_) => {
            return private_notification_response(safe_error(
                StatusCode::PAYLOAD_TOO_LARGE,
                "notification_preferences_body_too_large",
            ));
        }
    };
    let preferences = match serde_json::from_slice::<NotificationPreferencesRequest>(&body) {
        Ok(preferences) if validate_notification_preferences(&preferences).is_ok() => preferences,
        _ => {
            return private_notification_response(safe_error(
                StatusCode::BAD_REQUEST,
                "invalid_notification_preferences",
            ));
        }
    };
    let request_id = notification_request_id(&parts.headers);
    let url = format!(
        "{}/api/v1/notification/preferences",
        state.notification_url.trim_end_matches('/')
    );
    let response = match state
        .notification
        .clone_for_bearer()
        .put(url)
        .bearer_auth(token)
        .header("x-request-id", request_id.0)
        .header(header::ACCEPT, HeaderValue::from_static("application/json"))
        .json(&preferences)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => {
            return private_notification_response(safe_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "notification_preferences_upstream_unavailable",
            ));
        }
    };
    match read_notification_preferences_response(response).await {
        Ok(preferences) => private_notification_response(Json(preferences).into_response()),
        Err(response) => private_notification_response(response),
    }
}

/// Same-origin HTML form adapter for the Rust/Dioxus account settings view.
/// The JSON BFF remains the canonical API; this route only translates a
/// bounded browser form into that exact DTO and redirects back to SSR so the
/// saved value is re-read from the notification service.
pub async fn notification_preferences_form(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    if !same_origin_preferences_form(&parts.headers) {
        return safe_error(
            StatusCode::FORBIDDEN,
            "notification_preferences_origin_rejected",
        );
    }
    let token = match verified_bearer(&state, &parts.headers).await {
        Ok(token) => token,
        Err(response) => return response,
    };
    let content_type = parts
        .headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    if !content_type.split(';').next().is_some_and(|value| {
        value
            .trim()
            .eq_ignore_ascii_case("application/x-www-form-urlencoded")
    }) {
        return safe_error(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "notification_preferences_form_content_type",
        );
    }
    let body = match axum::body::to_bytes(body, NOTIFICATION_PREFERENCES_FORM_MAX).await {
        Ok(body) => body,
        Err(_) => {
            return safe_error(
                StatusCode::PAYLOAD_TOO_LARGE,
                "notification_preferences_form_too_large",
            );
        }
    };
    let preferences = match parse_notification_preferences_form(&body) {
        Ok(preferences) => preferences,
        Err(_) => return notification_preferences_form_redirect("error"),
    };
    let request_id = notification_request_id(&parts.headers);
    let url = format!(
        "{}/api/v1/notification/preferences",
        state.notification_url.trim_end_matches('/')
    );
    let response = match state
        .notification
        .clone_for_bearer()
        .put(url)
        .bearer_auth(token)
        .header("x-request-id", request_id.0)
        .header(header::ACCEPT, HeaderValue::from_static("application/json"))
        .json(&preferences)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => return notification_preferences_form_redirect("error"),
    };
    if read_notification_preferences_response(response)
        .await
        .is_ok()
    {
        notification_preferences_form_redirect("saved")
    } else {
        notification_preferences_form_redirect("error")
    }
}

pub async fn notification_stream(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let token = match verified_bearer(&state, &headers).await {
        Ok(token) => token,
        Err(response) => return response,
    };
    let url = format!(
        "{}/api/v1/notification/stream",
        state.notification_url.trim_end_matches('/')
    );
    let request_id = notification_request_id(&headers);
    let mut request = state
        .notification
        .clone_for_bearer()
        .get(url)
        .bearer_auth(token)
        .header("x-request-id", request_id.0)
        .header(
            header::ACCEPT,
            HeaderValue::from_static("text/event-stream"),
        );
    if let Some(last_event_id) = headers.get("last-event-id") {
        request = request.header("last-event-id", last_event_id);
    }
    let response = match request.send().await {
        Ok(response) => response,
        Err(_) => {
            return private_notification_response(safe_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "notification_stream_unavailable",
            ));
        }
    };
    if response.status() != StatusCode::OK {
        let status = response.status();
        return private_notification_response(match status {
            StatusCode::UNAUTHORIZED
            | StatusCode::FORBIDDEN
            | StatusCode::GONE
            | StatusCode::TOO_MANY_REQUESTS => safe_error(status, "notification_stream_rejected"),
            _ => safe_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "notification_stream_unavailable",
            ),
        });
    }
    let stream = response
        .bytes_stream()
        .map_err(|error| std::io::Error::other(error.to_string()));
    let mut proxied = Response::new(Body::from_stream(stream));
    *proxied.status_mut() = StatusCode::OK;
    proxied.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/event-stream; charset=utf-8"),
    );
    proxied.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    proxied.headers_mut().insert(
        header::VARY,
        HeaderValue::from_static("Cookie, Authorization, Last-Event-ID"),
    );
    proxied
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacySseTargetNotification {
    id: String,
    title: Option<String>,
    body: String,
    notification_type: Option<String>,
    priority: Option<String>,
    data: Option<serde_json::Value>,
    action_url: Option<String>,
    #[serde(rename = "read_at")]
    _read_at: Option<DateTime<chrono::Utc>>,
    created_at: DateTime<chrono::Utc>,
    #[serde(default)]
    expires_at: Option<DateTime<chrono::Utc>>,
}

fn legacy_stream_query(raw_query: Option<&str>, owner: &str) -> Result<(), &'static str> {
    let Some(raw_query) = raw_query.filter(|value| !value.is_empty()) else {
        return Ok(());
    };
    let url = reqwest::Url::parse(&format!("https://frontend.invalid/?{raw_query}"))
        .map_err(|_| "invalid_notification_stream_query")?;
    let mut seen = std::collections::HashSet::new();
    for (key, value) in url.query_pairs() {
        if !seen.insert(key.to_string()) {
            return Err("invalid_notification_stream_query");
        }
        match key.as_ref() {
            "wallet_address" if value.eq_ignore_ascii_case(owner) => {}
            // The target stream currently has no query-level type/priority
            // predicate. Do not silently ignore source filters; callers get
            // an explicit versioned boundary until the service adds one.
            "types" | "priority" => {
                return Err("legacy_notification_stream_filters_unsupported");
            }
            _ => return Err("invalid_notification_stream_query"),
        }
    }
    Ok(())
}

fn legacy_sse_notification_payload(
    target: LegacySseTargetNotification,
    owner: &str,
) -> Option<(String, serde_json::Value)> {
    let id = legacy_notification_id(&target.id)?;
    let title = target
        .title
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "Notification".to_string());
    let message = if target.body.is_empty() {
        title.clone()
    } else {
        target.body
    };
    if title.chars().count() > 200
        || title.chars().any(char::is_control)
        || message.is_empty()
        || message.chars().count() > 1_000
        || message.chars().any(char::is_control)
    {
        return None;
    }
    let data = match target.data {
        Some(value) if value.is_object() => Some(value),
        Some(_) => return None,
        None => None,
    };
    let action_url = target.action_url;
    if action_url
        .as_deref()
        .is_some_and(|value| !valid_legacy_action_url(value))
    {
        return None;
    }
    let payload = serde_json::json!({
        "id": id,
        "wallet_address": owner,
        "notification_type": legacy_notification_type(target.notification_type.as_deref()),
        "title": title,
        "message": message,
        "data": data,
        "priority": legacy_notification_priority(target.priority.as_deref()),
        "timestamp": target.created_at,
        "expires_at": target.expires_at,
    });
    Some((payload["id"].as_str()?.to_string(), payload))
}

fn next_sse_frame(buffer: &[u8]) -> Option<(String, usize)> {
    let lf = buffer.windows(2).position(|window| window == b"\n\n");
    let crlf = buffer.windows(4).position(|window| window == b"\r\n\r\n");
    let (index, delimiter_len) = match (lf, crlf) {
        (Some(lf), Some(crlf)) if crlf < lf => (crlf, 4),
        (Some(lf), _) => (lf, 2),
        (None, Some(crlf)) => (crlf, 4),
        (None, None) => return None,
    };
    let frame = String::from_utf8(buffer[..index].to_vec()).ok()?;
    Some((frame, index + delimiter_len))
}

fn transform_legacy_sse_frame(frame: &str, owner: &str) -> Option<String> {
    let mut event = "message";
    let mut source_id = None;
    let mut data_lines = Vec::new();
    for line in frame.lines() {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.starts_with(':') {
            continue;
        }
        if let Some(value) = line.strip_prefix("event:") {
            event = value.trim_start();
        } else if let Some(value) = line.strip_prefix("id:") {
            source_id = Some(value.trim_start().to_string());
        } else if let Some(value) = line.strip_prefix("data:") {
            data_lines.push(value.strip_prefix(' ').unwrap_or(value).to_string());
        }
    }
    if event == "ping" || (event == "message" && data_lines == ["ping".to_string()]) {
        return Some("event: ping\ndata: ping\n\n".to_string());
    }
    if !matches!(event, "message" | "notification") || data_lines.is_empty() {
        return None;
    }
    let target =
        serde_json::from_str::<LegacySseTargetNotification>(&data_lines.join("\n")).ok()?;
    if source_id.as_deref() != Some(target.id.as_str()) {
        return None;
    }
    let (id, payload) = legacy_sse_notification_payload(target, owner)?;
    // A target cursor is deliberately not trusted as a source UUID. Only
    // UUID-compatible target IDs are emitted, so reconnect can map the source
    // Last-Event-ID back to the exact target cursor without a stateful table.
    Some(format!(
        "id: {id}\nevent: notification\ndata: {}\n\n",
        serde_json::to_string(&payload).ok()?
    ))
}

/// Development-compatible SSE projection. The target stream is never passed
/// through raw: target `body/created_at/0x` fields are translated to the source
/// `message/timestamp/UUID` contract, and malformed or non-UUID events are
/// dropped rather than presented as a plausible source notification.
pub async fn legacy_notification_stream(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    raw_query: RawQuery,
) -> Response {
    let (token, user) = match verified_bearer_and_user(&state, &headers).await {
        Ok(session) => session,
        Err(response) => return response,
    };
    let owner = user.wallet_address.to_ascii_lowercase();
    if let Err(code) = legacy_stream_query(raw_query.0.as_deref(), &owner) {
        return legacy_notification_error(StatusCode::UNPROCESSABLE_ENTITY, code);
    }
    let target_cursor = match headers
        .get("last-event-id")
        .map(|value| value.to_str())
        .transpose()
    {
        Ok(Some(value)) if !value.is_empty() => {
            let Ok(uuid) = uuid::Uuid::parse_str(value) else {
                return legacy_notification_error(StatusCode::BAD_REQUEST, "invalid_last_event_id");
            };
            Some(format!("0x{}", uuid.simple()))
        }
        Ok(Some(_)) => {
            return legacy_notification_error(StatusCode::BAD_REQUEST, "invalid_last_event_id");
        }
        Ok(None) => None,
        Err(_) => {
            return legacy_notification_error(StatusCode::BAD_REQUEST, "invalid_last_event_id");
        }
    };
    let url = format!(
        "{}/api/v1/notification/stream",
        state.notification_url.trim_end_matches('/')
    );
    let request_id = notification_request_id(&headers);
    let mut upstream = state
        .notification
        .clone_for_bearer()
        .get(url)
        .bearer_auth(token)
        .header("x-request-id", request_id.0)
        .header(
            header::ACCEPT,
            HeaderValue::from_static("text/event-stream"),
        );
    if let Some(cursor) = target_cursor {
        upstream = upstream.header("last-event-id", cursor);
    }
    let response = match upstream.send().await {
        Ok(response) => response,
        Err(_) => {
            return legacy_notification_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "notification_stream_unavailable",
            );
        }
    };
    if response.status() != StatusCode::OK {
        let status = response.status();
        return legacy_notification_error(status, "notification_stream_rejected");
    }
    let owner_for_stream = owner.clone();
    let upstream = response.bytes_stream();
    let stream = futures::stream::unfold(
        (upstream, Vec::<u8>::new(), owner_for_stream),
        |(mut upstream, mut buffer, owner)| async move {
            loop {
                if let Some((frame, consumed)) = next_sse_frame(&buffer) {
                    buffer.drain(..consumed);
                    if let Some(mapped) = transform_legacy_sse_frame(&frame, &owner) {
                        return Some((
                            Ok::<String, std::io::Error>(mapped),
                            (upstream, buffer, owner),
                        ));
                    }
                    continue;
                }
                match upstream.next().await {
                    Some(Ok(chunk)) => buffer.extend_from_slice(chunk.as_ref()),
                    Some(Err(error)) => {
                        return Some((
                            Err(std::io::Error::other(error.to_string())),
                            (upstream, Vec::new(), owner),
                        ));
                    }
                    None => return None,
                }
            }
        },
    );
    let mut proxied = Response::new(Body::from_stream(stream));
    *proxied.status_mut() = StatusCode::OK;
    proxied.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/event-stream; charset=utf-8"),
    );
    proxied.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    proxied.headers_mut().insert(
        header::VARY,
        HeaderValue::from_static("Cookie, Authorization, Last-Event-ID"),
    );
    proxied
}

const NOTIFICATION_STREAM_ACK_BODY_MAX: usize = 4 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct NotificationStreamAckRequest {
    event_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct NotificationStreamAckResponse {
    ok: bool,
    event_id: String,
}

fn valid_notification_stream_cursor(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && !value
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
}

fn notification_stream_ack_upstream_error(status: StatusCode) -> Response {
    match status {
        StatusCode::BAD_REQUEST
        | StatusCode::UNAUTHORIZED
        | StatusCode::FORBIDDEN
        | StatusCode::NOT_FOUND
        | StatusCode::GONE
        | StatusCode::TOO_MANY_REQUESTS => safe_error(status, "notification_stream_ack_rejected"),
        _ => safe_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "notification_stream_ack_unavailable",
        ),
    }
}

#[allow(
    clippy::result_large_err,
    reason = "BFF upstream validation preserves the complete safe HTTP error response"
)]
async fn read_notification_stream_ack_response(
    response: reqwest::Response,
    expected_event_id: &str,
) -> Result<NotificationStreamAckResponse, Response> {
    if response.status() != StatusCode::OK {
        return Err(notification_stream_ack_upstream_error(response.status()));
    }
    if !valid_notification_json_content_type(response.headers()) {
        return Err(safe_error(
            StatusCode::BAD_GATEWAY,
            "malformed_notification_stream_ack_response",
        ));
    }
    let body = read_notification_body_limited(response, NOTIFICATION_STREAM_ACK_BODY_MAX)
        .await
        .map_err(|_| {
            safe_error(
                StatusCode::BAD_GATEWAY,
                "malformed_notification_stream_ack_response",
            )
        })?;
    let payload = serde_json::from_slice::<NotificationStreamAckResponse>(&body).map_err(|_| {
        safe_error(
            StatusCode::BAD_GATEWAY,
            "malformed_notification_stream_ack_response",
        )
    })?;
    if !payload.ok || payload.event_id != expected_event_id {
        return Err(safe_error(
            StatusCode::BAD_GATEWAY,
            "malformed_notification_stream_ack_response",
        ));
    }
    Ok(payload)
}

pub async fn notification_stream_ack(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    if !notification_mutation_origin_allowed(&parts.headers) {
        return private_notification_response(safe_error(
            StatusCode::FORBIDDEN,
            "notification_mutation_origin_rejected",
        ));
    }
    let token = match verified_bearer(&state, &parts.headers).await {
        Ok(token) => token,
        Err(response) => return response,
    };
    let body = match axum::body::to_bytes(body, NOTIFICATION_STREAM_ACK_BODY_MAX).await {
        Ok(body) => body,
        Err(_) => {
            return private_notification_response(safe_error(
                StatusCode::PAYLOAD_TOO_LARGE,
                "notification_stream_ack_body_too_large",
            ));
        }
    };
    let payload = match serde_json::from_slice::<NotificationStreamAckRequest>(&body) {
        Ok(payload) if valid_notification_stream_cursor(&payload.event_id) => payload,
        _ => {
            return private_notification_response(safe_error(
                StatusCode::BAD_REQUEST,
                "invalid_notification_stream_ack_request",
            ));
        }
    };
    let expected_event_id = payload.event_id.clone();
    let url = format!(
        "{}/api/v1/notification/stream/ack",
        state.notification_url.trim_end_matches('/')
    );
    let request_id = notification_request_id(&parts.headers);
    let response = match state
        .notification
        .clone_for_bearer()
        .post(url)
        .bearer_auth(token)
        .header("x-request-id", request_id.0)
        .header(header::ACCEPT, HeaderValue::from_static("application/json"))
        .json(&payload)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => {
            return private_notification_response(safe_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "notification_stream_ack_unavailable",
            ));
        }
    };
    match read_notification_stream_ack_response(response, &expected_event_id).await {
        Ok(payload) => private_notification_response(Json(payload).into_response()),
        Err(response) => private_notification_response(response),
    }
}

const NOTIFICATION_PUSH_BODY_MAX: usize = 8 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct NotificationPushRequest {
    endpoint: String,
    p256dh: String,
    auth: String,
    #[serde(default)]
    user_agent: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct NotificationPushUnsubscribeRequest {
    endpoint: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct NotificationPushResponse {
    enabled: bool,
    subscribed: bool,
    public_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    subscription_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    created_at: Option<String>,
}

fn valid_notification_push_response(payload: &NotificationPushResponse) -> bool {
    payload.enabled == payload.public_key.is_some()
        && (payload.enabled || !payload.subscribed)
        && payload
            .subscription_id
            .as_deref()
            .is_none_or(|_| payload.subscribed)
        && payload
            .created_at
            .as_deref()
            .is_none_or(|_| payload.subscribed)
        && payload.subscription_id.as_deref().is_none_or(|value| {
            !value.is_empty() && value.len() <= 128 && !value.chars().any(char::is_control)
        })
        && payload
            .created_at
            .as_deref()
            .is_none_or(|value| value.len() <= 64 && DateTime::parse_from_rfc3339(value).is_ok())
        && payload.public_key.as_deref().is_none_or(|key| {
            !key.is_empty()
                && key.len() <= 256
                && !key
                    .chars()
                    .any(|character| character.is_control() || character.is_whitespace())
        })
}

fn validate_notification_push_request(request: &NotificationPushRequest) -> Result<(), ()> {
    let endpoint = reqwest::Url::parse(&request.endpoint).map_err(|_| ())?;
    if endpoint.scheme() != "https"
        || !endpoint.username().is_empty()
        || endpoint.password().is_some()
        || endpoint.query().is_some()
        || endpoint.fragment().is_some()
        || request.endpoint.len() > 2048
        || request.p256dh.is_empty()
        || request.auth.is_empty()
        || request.p256dh.len() > 256
        || request.auth.len() > 256
        || !request
            .p256dh
            .bytes()
            .chain(request.auth.bytes())
            .all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'+' | b'/' | b'=')
            })
        || request.user_agent.as_deref().is_some_and(|user_agent| {
            user_agent.is_empty()
                || user_agent.len() > 512
                || user_agent.chars().any(char::is_control)
        })
    {
        return Err(());
    }
    Ok(())
}

fn validate_notification_push_endpoint(endpoint: &str) -> Result<(), ()> {
    let parsed = reqwest::Url::parse(endpoint).map_err(|_| ())?;
    if parsed.scheme() != "https"
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
        || endpoint.len() > 2048
    {
        return Err(());
    }
    Ok(())
}

#[allow(
    clippy::result_large_err,
    reason = "BFF upstream validation preserves the complete safe HTTP error response"
)]
async fn read_notification_push_response(
    response: reqwest::Response,
) -> Result<NotificationPushResponse, Response> {
    if response.status() != StatusCode::OK {
        return Err(notification_preferences_upstream_error(response.status()));
    }
    if !valid_notification_json_content_type(response.headers()) {
        return Err(safe_error(
            StatusCode::BAD_GATEWAY,
            "malformed_notification_push_response",
        ));
    }
    let body = read_notification_body_limited(response, NOTIFICATION_PUSH_BODY_MAX)
        .await
        .map_err(|_| {
            safe_error(
                StatusCode::BAD_GATEWAY,
                "malformed_notification_push_response",
            )
        })?;
    let payload = serde_json::from_slice::<NotificationPushResponse>(&body).map_err(|_| {
        safe_error(
            StatusCode::BAD_GATEWAY,
            "malformed_notification_push_response",
        )
    })?;
    if !valid_notification_push_response(&payload) {
        return Err(safe_error(
            StatusCode::BAD_GATEWAY,
            "malformed_notification_push_response",
        ));
    }
    Ok(payload)
}

pub async fn notification_push_status(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let token = match verified_bearer(&state, &headers).await {
        Ok(token) => token,
        Err(response) => return response,
    };
    let url = format!(
        "{}/api/v1/notification/push",
        state.notification_url.trim_end_matches('/')
    );
    let request_id = notification_request_id(&headers);
    let response = match state
        .notification
        .clone_for_bearer()
        .get(url)
        .bearer_auth(token)
        .header("x-request-id", request_id.0)
        .header(header::ACCEPT, HeaderValue::from_static("application/json"))
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => {
            return private_notification_response(safe_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "notification_push_unavailable",
            ));
        }
    };
    match read_notification_push_response(response).await {
        Ok(payload) => private_notification_response(Json(payload).into_response()),
        Err(response) => private_notification_response(response),
    }
}

async fn notification_push_mutation(
    State(state): State<AppState>,
    request: Request,
    method: Method,
) -> Response {
    let (parts, body) = request.into_parts();
    if !notification_mutation_origin_allowed(&parts.headers) {
        return private_notification_response(safe_error(
            StatusCode::FORBIDDEN,
            "notification_mutation_origin_rejected",
        ));
    }
    let token = match verified_bearer(&state, &parts.headers).await {
        Ok(token) => token,
        Err(response) => return response,
    };
    let body = match axum::body::to_bytes(body, NOTIFICATION_PUSH_BODY_MAX).await {
        Ok(body) => body,
        Err(_) => {
            return private_notification_response(safe_error(
                StatusCode::PAYLOAD_TOO_LARGE,
                "notification_push_body_too_large",
            ));
        }
    };
    let payload = if method == Method::DELETE {
        match serde_json::from_slice::<NotificationPushUnsubscribeRequest>(&body) {
            Ok(payload) if validate_notification_push_endpoint(&payload.endpoint).is_ok() => {
                match serde_json::to_value(payload) {
                    Ok(payload) => payload,
                    Err(_) => {
                        return private_notification_response(safe_error(
                            StatusCode::BAD_REQUEST,
                            "invalid_notification_push_request",
                        ));
                    }
                }
            }
            _ => {
                return private_notification_response(safe_error(
                    StatusCode::BAD_REQUEST,
                    "invalid_notification_push_request",
                ));
            }
        }
    } else {
        match serde_json::from_slice::<NotificationPushRequest>(&body) {
            Ok(payload) if validate_notification_push_request(&payload).is_ok() => {
                match serde_json::to_value(payload) {
                    Ok(payload) => payload,
                    Err(_) => {
                        return private_notification_response(safe_error(
                            StatusCode::BAD_REQUEST,
                            "invalid_notification_push_request",
                        ));
                    }
                }
            }
            _ => {
                return private_notification_response(safe_error(
                    StatusCode::BAD_REQUEST,
                    "invalid_notification_push_request",
                ));
            }
        }
    };
    let url = format!(
        "{}/api/v1/notification/push",
        state.notification_url.trim_end_matches('/')
    );
    let request_id = notification_request_id(&parts.headers);
    let mut upstream = state.notification.clone_for_bearer().request(method, url);
    upstream = upstream
        .bearer_auth(token)
        .header("x-request-id", request_id.0)
        .header(header::ACCEPT, HeaderValue::from_static("application/json"))
        .json(&payload);
    let response = match upstream.send().await {
        Ok(response) => response,
        Err(_) => {
            return private_notification_response(safe_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "notification_push_unavailable",
            ));
        }
    };
    match read_notification_push_response(response).await {
        Ok(result) => private_notification_response(Json(result).into_response()),
        Err(response) => private_notification_response(response),
    }
}

pub async fn notification_push_subscribe(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    notification_push_mutation(State(state), request, Method::PUT).await
}

pub async fn notification_push_unsubscribe(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    notification_push_mutation(State(state), request, Method::DELETE).await
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyPushKeys {
    p256dh: String,
    auth: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyPushSubscription {
    endpoint: String,
    keys: LegacyPushKeys,
    #[serde(default)]
    user_agent: Option<String>,
    #[serde(default)]
    device_type: Option<String>,
}

#[derive(Clone, Copy)]
enum LegacyPushResponseKind {
    Status,
    Subscribe,
    Unsubscribe,
}

fn legacy_push_source_data(
    payload: &NotificationPushResponse,
    kind: LegacyPushResponseKind,
) -> Option<serde_json::Value> {
    match kind {
        // The development client reads status from response.data.data.
        LegacyPushResponseKind::Status => Some(serde_json::json!({
            "data": {
                "subscribed": payload.subscribed,
                "subscription_id": payload.subscription_id,
                "created_at": payload.created_at,
            }
        })),
        // The development client treats subscribe as a durable subscription
        // record, so do not claim success when the target omitted its stable
        // identity or creation timestamp.
        LegacyPushResponseKind::Subscribe => Some(serde_json::json!({
            "subscription_id": payload.subscription_id.as_ref()?,
            "active": payload.subscribed,
            "created_at": payload.created_at.as_ref()?,
        })),
        // Unsubscribe is an action result in the source contract, not a
        // status DTO. The target route is owner-wide and already returns the
        // post-revocation state; expose only the source action result.
        LegacyPushResponseKind::Unsubscribe => Some(serde_json::json!({
            "success": true,
            "message": if payload.subscribed {
                "Push subscription remains active"
            } else {
                "Push notifications unsubscribed"
            },
        })),
    }
}

async fn legacy_push_response(
    response: Response,
    operation: &'static str,
    kind: LegacyPushResponseKind,
) -> Response {
    let status = response.status();
    if !status.is_success() {
        return legacy_notification_error(status, operation);
    }
    let body = match axum::body::to_bytes(response.into_body(), NOTIFICATION_PUSH_BODY_MAX).await {
        Ok(body) => body,
        Err(_) => {
            return legacy_notification_error(
                StatusCode::BAD_GATEWAY,
                "malformed_notification_push_response",
            );
        }
    };
    let payload = match serde_json::from_slice::<NotificationPushResponse>(&body) {
        Ok(payload) if valid_notification_push_response(&payload) => payload,
        _ => {
            return legacy_notification_error(
                StatusCode::BAD_GATEWAY,
                "malformed_notification_push_response",
            );
        }
    };
    let Some(data) = legacy_push_source_data(&payload, kind) else {
        return legacy_notification_error(
            StatusCode::BAD_GATEWAY,
            "malformed_notification_push_response",
        );
    };
    private_notification_response(
        Json(serde_json::json!({
            "success": true,
            "data": data,
            "api_version": "v1",
            "access_level": "auth",
        }))
        .into_response(),
    )
}

pub async fn legacy_notification_push_status(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    legacy_push_response(
        notification_push_status(State(state), headers).await,
        "notification_push_rejected",
        LegacyPushResponseKind::Status,
    )
    .await
}

pub async fn legacy_notification_push_subscribe(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    if !notification_mutation_origin_allowed(&parts.headers) {
        return legacy_notification_error(
            StatusCode::FORBIDDEN,
            "notification_mutation_origin_rejected",
        );
    }
    let body = match axum::body::to_bytes(body, NOTIFICATION_PUSH_BODY_MAX).await {
        Ok(body) => body,
        Err(_) => {
            return legacy_notification_error(
                StatusCode::PAYLOAD_TOO_LARGE,
                "notification_push_body_too_large",
            );
        }
    };
    let source = match serde_json::from_slice::<LegacyPushSubscription>(&body) {
        Ok(source)
            if source
                .device_type
                .as_deref()
                .is_none_or(|value| matches!(value, "desktop" | "mobile" | "tablet")) =>
        {
            source
        }
        _ => {
            return legacy_notification_error(
                StatusCode::BAD_REQUEST,
                "invalid_notification_push_request",
            );
        }
    };
    let target = NotificationPushRequest {
        endpoint: source.endpoint,
        p256dh: source.keys.p256dh,
        auth: source.keys.auth,
        user_agent: source.user_agent,
    };
    if validate_notification_push_request(&target).is_err() {
        return legacy_notification_error(
            StatusCode::BAD_REQUEST,
            "invalid_notification_push_request",
        );
    }
    let target_body = match serde_json::to_vec(&target) {
        Ok(body) => Body::from(body),
        Err(_) => {
            return legacy_notification_error(
                StatusCode::BAD_REQUEST,
                "invalid_notification_push_request",
            );
        }
    };
    legacy_push_response(
        notification_push_mutation(
            State(state),
            Request::from_parts(parts, target_body),
            Method::PUT,
        )
        .await,
        "notification_push_rejected",
        LegacyPushResponseKind::Subscribe,
    )
    .await
}

pub async fn legacy_notification_push_unsubscribe(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, _body) = request.into_parts();
    if !notification_mutation_origin_allowed(&parts.headers) {
        return legacy_notification_error(
            StatusCode::FORBIDDEN,
            "notification_mutation_origin_rejected",
        );
    }
    let token = match verified_bearer(&state, &parts.headers).await {
        Ok(token) => token,
        Err(response) => return response,
    };
    let url = format!(
        "{}/api/v1/notification/push/unsubscribe",
        state.notification_url.trim_end_matches('/')
    );
    let request_id = notification_request_id(&parts.headers);
    let response = match state
        .notification
        .clone_for_bearer()
        .delete(url)
        .bearer_auth(token)
        .header("x-request-id", request_id.0)
        .header(header::ACCEPT, HeaderValue::from_static("application/json"))
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => {
            return legacy_notification_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "notification_push_unavailable",
            );
        }
    };
    match read_notification_push_response(response).await {
        Ok(payload) => {
            legacy_push_response(
                private_notification_response(Json(payload).into_response()),
                "notification_push_rejected",
                LegacyPushResponseKind::Unsubscribe,
            )
            .await
        }
        Err(response) => legacy_notification_error(response.status(), "notification_push_rejected"),
    }
}

pub async fn track_event(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<AnalyticsTrackBody>,
) -> Response {
    let token = match verified_bearer(&state, &headers).await {
        Ok(token) => token,
        Err(response) => return response,
    };
    let url = format!(
        "{}/api/v1/analytics/track",
        state.api_url.trim_end_matches('/')
    );
    match state
        .analytics
        .clone_for_bearer()
        .post(&url)
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "event_name": body.event_name,
            "properties": body.properties,
            "user_id": body.user_id,
            "chain_id": body.chain_id,
        }))
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
    let (scheme, rest) = value.split_once("://")?;
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
        if !object_has_only_keys(object, &["success", "data", "error", "meta"])
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
        .map(|value| {
            serde_json::from_value(value).map_err(|error| {
                tracing::warn!(%error, "public news article did not match the backend DTO");
            })
        })
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
            };
        }
    };
    let value = match client.get_plain(NEWS_LIST_PATH).await {
        Ok(value) => value,
        Err(error) => {
            return NewsListLoadOutcome::Error {
                code: news_dependency_error(error),
            };
        }
    };
    let articles = match parse_news_list(value) {
        Ok(articles) => articles,
        Err(()) => {
            return NewsListLoadOutcome::Error {
                code: "malformed_content_response".to_string(),
            };
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
    let path = format!("{NEWS_DETAIL_PATH}/{slug}");
    let value = match client.get_plain(&path).await {
        Ok(value) => value,
        Err(ClientError::NotFound) => return NewsDetailLoadOutcome::NotFound,
        Err(error) => {
            return NewsDetailLoadOutcome::Error {
                code: news_dependency_error(error),
            };
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
            "/api/public/news",
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
            "/api/public/news",
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
            "/api/public/news",
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
            "/api/public/news/live-article",
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
            "/api/public/news/missing",
            get(|| async { StatusCode::NOT_FOUND }),
        );
        assert!(matches!(
            load_news_post(&client(spawn_mock(missing_router).await), "missing").await,
            NewsDetailLoadOutcome::NotFound
        ));

        let mismatch = article("other-article", "Wrong owner", &["engineering"]);
        let mismatch_router = Router::new().route(
            "/api/public/news/live-article",
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
            "clicked_at": null,
            "title": "Title",
            "notification_type": "system",
            "priority": "normal",
            "action_url": null,
            "expires_at": null
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
        let source_query = NotificationListQuery::from_raw_query(Some(
            "page=2&limit=10&type=payment&priority=high&status=unread&start_date=2026-07-01T00:00:00Z&end_date=2026-07-31T23:59:59Z",
        ))
        .unwrap();
        assert_eq!(source_query.offset, Some(10));
        assert_eq!(source_query.notification_type.as_deref(), Some("payment"));
        assert_eq!(source_query.priority.as_deref(), Some("high"));
        assert_eq!(source_query.status.as_deref(), Some("unread"));
        assert!(source_query.upstream_suffix().contains("type=payment"));
        assert!(source_query
            .upstream_suffix()
            .contains("start_date=2026-07-01T00%3A00%3A00Z"));
        assert_eq!(
            source_query.upstream_unread_suffix(),
            "?status=unread&type=payment&priority=high&start_date=2026-07-01T00%3A00%3A00Z&end_date=2026-07-31T23%3A59%3A59Z"
        );

        for invalid in [
            "user_id=0xother",
            "caller=0xother",
            "limit=0",
            "limit=101",
            "limit=-1",
            "offset=-1",
            "offset=1000001",
            "status=sent&status=failed",
            "type=payment&notification_type=security",
            "limit=1&limit=2",
            "page=2&offset=20",
            "start_date=2026-08-01T00:00:00Z&end_date=2026-07-01T00:00:00Z",
            "unknown=value",
        ] {
            assert!(
                NotificationListQuery::from_raw_query(Some(invalid)).is_err(),
                "query must fail closed: {invalid}"
            );
        }
    }

    #[test]
    fn legacy_query_is_owner_bound_and_preserves_source_pagination() {
        let owner = "0x1111111111111111111111111111111111111111";
        let (query, page, limit) = legacy_notification_query(
            Some("page=2&limit=10&type=payment&wallet_address=0x1111111111111111111111111111111111111111"),
            owner,
        )
        .expect("same owner selector should be accepted by the compatibility adapter");
        assert_eq!(page, 2);
        assert_eq!(limit, 10);
        assert_eq!(query.offset, Some(10));
        assert_eq!(query.notification_type.as_deref(), Some("payment"));
        assert!(legacy_notification_query(
            Some("wallet_address=0x2222222222222222222222222222222222222222"),
            owner,
        )
        .is_err());
    }

    #[test]
    fn legacy_projection_round_trips_target_ids_and_has_explicit_defaults() {
        let value = serde_json::json!({
            "id": "0x00000000000000000000000000000001",
            "user_id": "0x1111111111111111111111111111111111111111",
            "channel": "in_app",
            "recipient": "0x1111111111111111111111111111111111111111",
            "template_id": null,
            "subject": null,
            "body": "Body",
            "data": {"kind": "migration", "image_url": "/images/notification.png"},
            "status": "sent",
            "error": null,
            "sent_at": "2026-07-22T00:00:00Z",
            "created_at": "2026-07-22T00:00:00Z",
            "read_at": null,
            "clicked_at": "2026-07-29T00:00:00Z",
            "title": null,
            "notification_type": null,
            "priority": null,
            "action_url": null,
            "expires_at": "2026-07-30T00:00:00Z"
        });
        let wire: NotificationWire = serde_json::from_value(value).unwrap();
        let projected =
            legacy_notification_from_wire(&wire, "0x1111111111111111111111111111111111111111")
                .unwrap();
        assert_eq!(projected.id, "00000000-0000-0000-0000-000000000001");
        assert_eq!(projected.notification_type, "system");
        assert_eq!(projected.priority, "normal");
        assert_eq!(projected.title, "Notification");
        assert_eq!(
            projected.expires_at.as_deref(),
            Some("2026-07-30T00:00:00+00:00")
        );
        assert_eq!(
            projected.clicked_at.as_deref(),
            Some("2026-07-29T00:00:00+00:00")
        );
        assert!(projected.delivered_at.is_none());
        assert_eq!(
            projected.image_url.as_deref(),
            Some("/images/notification.png")
        );
        assert!(!projected.read);
        assert_eq!(target_notification_id_from_legacy(&projected.id), wire.id);
    }

    #[test]
    fn legacy_projection_rejects_unsafe_data_image_urls() {
        let value = serde_json::json!({
            "id": "0x00000000000000000000000000000001",
            "user_id": "0x1111111111111111111111111111111111111111",
            "channel": "in_app",
            "recipient": "0x1111111111111111111111111111111111111111",
            "body": "Body",
            "data": {"image_url": "javascript:alert(1)"},
            "status": "sent",
            "created_at": "2026-07-22T00:00:00Z"
        });
        let wire: NotificationWire = serde_json::from_value(value).unwrap();
        assert!(
            legacy_notification_from_wire(&wire, "0x1111111111111111111111111111111111111111")
                .is_none()
        );
    }

    #[test]
    fn legacy_sse_projection_translates_target_frames_and_drops_unsafe_ids() {
        let target_id = "0x00000000000000000000000000000001";
        let frame = format!(
            "id: {target_id}\nevent: notification\ndata: {{\"id\":\"{target_id}\",\"title\":\"Title\",\"body\":\"Body\",\"notification_type\":\"payment\",\"priority\":\"high\",\"data\":{{\"kind\":\"test\"}},\"action_url\":null,\"read_at\":null,\"created_at\":\"2026-07-22T00:00:00Z\",\"expires_at\":\"2026-07-30T00:00:00Z\"}}"
        );
        let projected =
            transform_legacy_sse_frame(&frame, "0x1111111111111111111111111111111111111111")
                .expect("UUID-compatible target event should project");
        assert!(projected.contains("event: notification"));
        assert!(projected.contains("00000000-0000-0000-0000-000000000001"));
        assert!(
            projected.contains("\"wallet_address\":\"0x1111111111111111111111111111111111111111\"")
        );
        assert!(projected.contains("\"message\":\"Body\""));
        assert!(projected.contains("\"expires_at\":\"2026-07-30T00:00:00Z\""));

        assert_eq!(
            transform_legacy_sse_frame("event: ping\ndata: ping", "0xowner"),
            Some("event: ping\ndata: ping\n\n".to_string())
        );
        assert_eq!(
            transform_legacy_sse_frame("data: ping", "0xowner"),
            Some("event: ping\ndata: ping\n\n".to_string())
        );
        let unsafe_id = frame.replace(
            target_id,
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        assert!(transform_legacy_sse_frame(&unsafe_id, "0xowner").is_none());
        let mismatched = frame.replacen(
            &format!("id: {target_id}"),
            "id: 0x00000000000000000000000000000002",
            1,
        );
        assert!(transform_legacy_sse_frame(&mismatched, "0xowner").is_none());
    }

    #[test]
    fn legacy_stream_query_binds_owner_and_rejects_unimplemented_filters() {
        let owner = "0x1111111111111111111111111111111111111111";
        assert!(legacy_stream_query(None, owner).is_ok());
        assert!(legacy_stream_query(
            Some("wallet_address=0x1111111111111111111111111111111111111111"),
            owner
        )
        .is_ok());
        assert_eq!(
            legacy_stream_query(
                Some("wallet_address=0x2222222222222222222222222222222222222222"),
                owner
            ),
            Err("invalid_notification_stream_query")
        );
        assert_eq!(
            legacy_stream_query(Some("priority=high"), owner),
            Err("legacy_notification_stream_filters_unsupported")
        );
        assert_eq!(
            legacy_stream_query(Some("wallet_address=0x1111111111111111111111111111111111111111&wallet_address=0x1111111111111111111111111111111111111111"), owner),
            Err("invalid_notification_stream_query")
        );
    }

    #[test]
    fn legacy_bulk_mutation_counts_require_the_exact_target_envelope() {
        assert_eq!(legacy_bulk_count(br#"{"marked":7}"#, "marked"), Ok(7));
        assert_eq!(legacy_bulk_count(br#"{"deleted":3}"#, "deleted"), Ok(3));
        for (body, key) in [
            (br#"{}"#.as_slice(), "marked"),
            (br#"{"marked":-1}"#.as_slice(), "marked"),
            (br#"{"marked":7,"extra":true}"#.as_slice(), "marked"),
            (br#"{"deleted":3}"#.as_slice(), "marked"),
            (br#"not-json"#.as_slice(), "deleted"),
        ] {
            assert!(legacy_bulk_count(body, key).is_err());
        }
    }

    #[test]
    fn legacy_preferences_projection_keeps_target_channels_and_marks_unmapped_fields() {
        let preferences = NotificationPreferencesResponse {
            channels: serde_json::json!({"email": true, "in_app": true, "push": false}),
            quiet_hours: Some(serde_json::json!({
                "enabled": true,
                "start": "22:00",
                "end": "07:00"
            })),
            timezone: Some("Asia/Bangkok".to_string()),
            updated_at: None,
        };
        let value = legacy_preferences_projection(&preferences);
        assert_eq!(value["email_enabled"], true);
        assert_eq!(value["push_enabled"], false);
        assert_eq!(value["quiet_hours"]["start_time"], "22:00");
        assert_eq!(value["quiet_hours"]["timezone"], "Asia/Bangkok");
        assert_eq!(value["sms_enabled"], false);
    }

    #[test]
    fn legacy_preferences_projection_round_trips_type_and_priority_policy() {
        let preferences = NotificationPreferencesResponse {
            channels: serde_json::json!({
                "email": true,
                "in_app": true,
                "push": false,
                "types": {"payment": false, "system": true},
                "priority_filter": "high"
            }),
            quiet_hours: None,
            timezone: None,
            updated_at: None,
        };
        let value = legacy_preferences_projection(&preferences);
        assert_eq!(value["types"]["payment"], false);
        assert_eq!(value["types"]["system"], true);
        assert_eq!(value["types"]["chat"], true);
        assert_eq!(value["priority_filter"], "high");
        assert!(valid_channel_preferences(&preferences.channels));
        assert!(!valid_channel_preferences(
            &serde_json::json!({"types": {"unknown": true}})
        ));
    }

    #[test]
    fn fixed_ssr_pages_derive_only_bounded_source_sized_offsets() {
        for (page, offset) in [
            (1, 0),
            (2, 20),
            (NOTIFICATION_SSR_MAX_PAGE, NOTIFICATION_LIST_OFFSET_MAX),
        ] {
            let query = NotificationListQuery::for_ssr_page(page).expect("bounded SSR page");
            assert_eq!(query.limit, Some(NOTIFICATION_SSR_PAGE_SIZE));
            assert_eq!(query.offset, Some(offset));
            assert_eq!(query.status, None);
            assert_eq!(
                query.upstream_suffix(),
                format!("?limit={NOTIFICATION_SSR_PAGE_SIZE}&offset={offset}")
            );
        }
        assert!(NotificationListQuery::for_ssr_page(0).is_none());
        assert!(NotificationListQuery::for_ssr_page(NOTIFICATION_SSR_MAX_PAGE + 1).is_none());
        let filtered = NotificationListQuery::for_ssr_page_and_filters(
            3,
            Some("unread"),
            Some("payment"),
            Some("critical"),
        )
        .expect("bounded owner filters");
        assert_eq!(
            filtered.upstream_suffix(),
            "?limit=20&offset=40&status=unread&type=payment&priority=critical"
        );
        let dated = NotificationListQuery::for_ssr_page_and_filters_and_dates(
            2,
            Some("read"),
            Some("wallet_management"),
            Some("urgent"),
            Some("2026-01-01T00:00:00Z"),
            Some("2026-01-31T23:59:59Z"),
        )
        .expect("bounded source date filters");
        assert_eq!(
            dated.upstream_suffix(),
            "?limit=20&offset=20&status=read&type=wallet_management&priority=urgent&start_date=2026-01-01T00%3A00%3A00Z&end_date=2026-01-31T23%3A59%3A59Z"
        );
        assert!(NotificationListQuery::for_ssr_page_and_filters(
            1,
            None,
            Some("payment type"),
            None,
        )
        .is_none());
        assert!(NotificationListQuery::for_ssr_page_and_filters_and_dates(
            1,
            None,
            None,
            None,
            Some("2026-02-01T00:00:00Z"),
            Some("2026-01-01T00:00:00Z"),
        )
        .is_none());
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
    fn list_rows_reject_unbounded_or_unsafe_field_values() {
        let owner = "0x1111111111111111111111111111111111111111";
        let query = NotificationListQuery {
            limit: Some(1),
            ..NotificationListQuery::default()
        };
        for (field, value) in [
            (
                "id",
                serde_json::Value::String("x".repeat(NOTIFICATION_ID_MAX + 1)),
            ),
            (
                "recipient",
                serde_json::Value::String("x".repeat(NOTIFICATION_RECIPIENT_MAX + 1)),
            ),
            (
                "subject",
                serde_json::Value::String("x".repeat(NOTIFICATION_SUBJECT_MAX + 1)),
            ),
            (
                "body",
                serde_json::Value::String("x".repeat(NOTIFICATION_BODY_MAX + 1)),
            ),
            (
                "title",
                serde_json::Value::String("x".repeat(NOTIFICATION_TITLE_MAX + 1)),
            ),
            (
                "error",
                serde_json::Value::String("x".repeat(NOTIFICATION_ERROR_MAX + 1)),
            ),
            (
                "action_url",
                serde_json::Value::String("/safe\\\\unsafe".to_string()),
            ),
        ] {
            let mut row = notification(owner);
            row.as_object_mut()
                .unwrap()
                .insert(field.to_string(), value);
            let payload = serde_json::from_value::<NotificationListWire>(
                serde_json::json!({"items": [row], "total": 1}),
            )
            .unwrap();
            assert!(
                payload.validate(owner, &query).is_err(),
                "field must remain bounded and safe: {field}"
            );
        }

        for unsafe_url in [
            "https://evil.example/",
            "javascript:alert(1)",
            "//evil.example",
        ] {
            let mut row = notification(owner);
            row["action_url"] = serde_json::json!(unsafe_url);
            let payload = serde_json::from_value::<NotificationListWire>(
                serde_json::json!({"items": [row], "total": 1}),
            )
            .unwrap();
            assert!(
                payload.validate(owner, &query).is_err(),
                "unsafe action URL must fail closed: {unsafe_url}"
            );
        }

        let mut oversized_data = notification(owner);
        oversized_data["data"] = serde_json::json!({
            "payload": "x".repeat(NOTIFICATION_DATA_MAX)
        });
        let payload = serde_json::from_value::<NotificationListWire>(
            serde_json::json!({"items": [oversized_data], "total": 1}),
        )
        .unwrap();
        assert!(payload.validate(owner, &query).is_err());

        let mut invalid_priority = notification(owner);
        invalid_priority["priority"] = serde_json::json!("high priority");
        let payload = serde_json::from_value::<NotificationListWire>(
            serde_json::json!({"items": [invalid_priority], "total": 1}),
        )
        .unwrap();
        assert!(payload.validate(owner, &query).is_err());
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
            ..NotificationListQuery::default()
        };
        assert!(contradictory.validate(owner, &past_end).is_ok());

        let second_page = NotificationListQuery::for_ssr_page(2).unwrap();
        let final_row = serde_json::from_value::<NotificationListWire>(serde_json::json!({
            "items": [notification(owner)],
            "total": 21
        }))
        .unwrap();
        assert!(final_row.validate(owner, &second_page).is_ok());
        let missing_final_row = serde_json::from_value::<NotificationListWire>(serde_json::json!({
            "items": [],
            "total": 21
        }))
        .unwrap();
        assert!(missing_final_row.validate(owner, &second_page).is_err());

        // Status-filtered totals are now scoped to the same service predicate,
        // so the BFF applies the same exact cardinality check.
        let filtered = NotificationListQuery {
            limit: Some(1),
            offset: None,
            status: Some("sent".to_string()),
            ..NotificationListQuery::default()
        };
        assert!(contradictory.validate(owner, &filtered).is_err());

        let broadcast = serde_json::json!({
            "id": "broadcast-1",
            "user_id": null,
            "channel": "in_app",
            "recipient": "all",
            "template_id": null,
            "subject": null,
            "body": "Maintenance notice",
            "data": null,
            "status": "sent",
            "error": null,
            "sent_at": "2026-07-22T00:00:00Z",
            "created_at": "2026-07-22T00:00:00Z",
            "read_at": null,
            "clicked_at": null,
            "title": "Maintenance",
            "notification_type": "announcement",
            "priority": "normal",
            "action_url": null,
            "expires_at": null
        });
        let payload = serde_json::from_value::<NotificationListWire>(serde_json::json!({
            "items": [broadcast],
            "total": 1
        }))
        .unwrap();
        let one_row_query = NotificationListQuery {
            limit: Some(1),
            ..NotificationListQuery::default()
        };
        assert!(payload.validate(owner, &one_row_query).is_ok());

        let foreign_null_owner = serde_json::json!({
            "id": "foreign-1",
            "user_id": null,
            "channel": "in_app",
            "recipient": "0x2222222222222222222222222222222222222222",
            "template_id": null,
            "subject": null,
            "body": "Not a broadcast",
            "data": null,
            "status": "sent",
            "error": null,
            "sent_at": null,
            "created_at": "2026-07-22T00:00:00Z",
            "read_at": null,
            "clicked_at": null,
            "title": "Foreign",
            "notification_type": "system",
            "priority": "normal",
            "action_url": null,
            "expires_at": null
        });
        let payload = serde_json::from_value::<NotificationListWire>(serde_json::json!({
            "items": [foreign_null_owner],
            "total": 1
        }))
        .unwrap();
        assert!(payload.validate(owner, &one_row_query).is_err());
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

    #[test]
    fn preferences_and_push_payloads_are_bounded_and_fail_closed() {
        let valid_preferences = NotificationPreferencesRequest {
            channels: serde_json::json!({"email": true, "in_app": false}),
            quiet_hours: Some(serde_json::json!({"start": "22:00", "end": "07:00"})),
            timezone: Some("Asia/Bangkok".into()),
        };
        assert!(validate_notification_preferences(&valid_preferences).is_ok());
        for invalid in [
            NotificationPreferencesRequest {
                channels: serde_json::json!([]),
                quiet_hours: None,
                timezone: None,
            },
            NotificationPreferencesRequest {
                channels: serde_json::json!({"email": "yes"}),
                quiet_hours: None,
                timezone: None,
            },
            NotificationPreferencesRequest {
                channels: serde_json::json!({"webhook": true}),
                quiet_hours: None,
                timezone: None,
            },
            NotificationPreferencesRequest {
                channels: serde_json::json!({"email": true}),
                quiet_hours: Some(serde_json::json!("22:00")),
                timezone: None,
            },
            NotificationPreferencesRequest {
                channels: serde_json::json!({"email": true}),
                quiet_hours: Some(serde_json::json!({"start": "25:00", "end": "07:00"})),
                timezone: None,
            },
            NotificationPreferencesRequest {
                channels: serde_json::json!({"email": true}),
                quiet_hours: None,
                timezone: Some("UTC\nInjected".into()),
            },
            NotificationPreferencesRequest {
                channels: serde_json::json!({"payload": "x".repeat(NOTIFICATION_PREFERENCES_BODY_MAX)}),
                quiet_hours: None,
                timezone: None,
            },
        ] {
            assert!(validate_notification_preferences(&invalid).is_err());
        }

        let valid_push = NotificationPushRequest {
            endpoint: "https://push.example.test/subscription".into(),
            p256dh: "key_123".into(),
            auth: "auth_123".into(),
            user_agent: Some("browser".into()),
        };
        assert!(validate_notification_push_request(&valid_push).is_ok());
        for invalid in [
            NotificationPushRequest {
                endpoint: "http://push.example.test/subscription".into(),
                ..valid_push.clone()
            },
            NotificationPushRequest {
                endpoint: "https://user:password@push.example.test/subscription".into(),
                ..valid_push.clone()
            },
            NotificationPushRequest {
                endpoint: "https://push.example.test/subscription?token=secret".into(),
                ..valid_push.clone()
            },
            NotificationPushRequest {
                p256dh: "not allowed!*".into(),
                ..valid_push.clone()
            },
            NotificationPushRequest {
                user_agent: Some("browser\nforged".into()),
                ..valid_push.clone()
            },
        ] {
            assert!(validate_notification_push_request(&invalid).is_err());
        }
        assert!(
            validate_notification_push_endpoint("https://push.example.test/subscription").is_ok()
        );
        assert!(validate_notification_push_endpoint(
            "https://push.example.test/subscription?token=secret"
        )
        .is_err());
        assert!(valid_notification_push_response(
            &NotificationPushResponse {
                enabled: true,
                subscribed: true,
                public_key: Some("B".repeat(65)),
                subscription_id: None,
                created_at: None,
            }
        ));
        let unavailable_push = serde_json::to_value(NotificationPushResponse {
            enabled: false,
            subscribed: false,
            public_key: None,
            subscription_id: None,
            created_at: None,
        })
        .expect("push status should serialize");
        let mut unavailable_keys = unavailable_push
            .as_object()
            .unwrap()
            .keys()
            .collect::<Vec<_>>();
        unavailable_keys.sort();
        assert_eq!(
            unavailable_keys,
            vec!["enabled", "public_key", "subscribed"]
        );
        for invalid in [
            NotificationPushResponse {
                enabled: false,
                subscribed: true,
                public_key: None,
                subscription_id: None,
                created_at: None,
            },
            NotificationPushResponse {
                enabled: true,
                subscribed: false,
                public_key: None,
                subscription_id: None,
                created_at: None,
            },
            NotificationPushResponse {
                enabled: true,
                subscribed: false,
                public_key: Some("key with spaces".into()),
                subscription_id: None,
                created_at: None,
            },
        ] {
            assert!(!valid_notification_push_response(&invalid));
        }
    }

    #[test]
    fn legacy_push_projection_preserves_source_operation_shapes() {
        let payload = NotificationPushResponse {
            enabled: true,
            subscribed: true,
            public_key: Some("B".repeat(65)),
            subscription_id: Some("push_abc".into()),
            created_at: Some("2026-07-22T00:00:00Z".into()),
        };
        let status = legacy_push_source_data(&payload, LegacyPushResponseKind::Status).unwrap();
        assert_eq!(status["data"]["subscribed"], true);
        assert_eq!(status["data"]["subscription_id"], "push_abc");

        let subscribe =
            legacy_push_source_data(&payload, LegacyPushResponseKind::Subscribe).unwrap();
        assert_eq!(subscribe["active"], true);
        assert_eq!(subscribe["subscription_id"], "push_abc");
        assert_eq!(subscribe["created_at"], "2026-07-22T00:00:00Z");

        let unsubscribe =
            legacy_push_source_data(&payload, LegacyPushResponseKind::Unsubscribe).unwrap();
        assert_eq!(unsubscribe["success"], true);
        assert!(unsubscribe["message"].as_str().unwrap().contains("active"));

        let incomplete = NotificationPushResponse {
            subscription_id: None,
            created_at: None,
            ..payload
        };
        assert!(legacy_push_source_data(&incomplete, LegacyPushResponseKind::Status).is_some());
        assert!(legacy_push_source_data(&incomplete, LegacyPushResponseKind::Subscribe).is_none());
    }

    #[test]
    fn notification_preferences_form_maps_only_the_exact_bounded_fields() {
        let request = parse_notification_preferences_form(
            b"email=true&in_app=false&push=true&quiet_enabled=true&quiet_start=22%3A00&quiet_end=07%3A00&timezone=Asia%2FBangkok",
        )
        .expect("canonical form should map");
        assert_eq!(
            request.channels,
            serde_json::json!({"email": true, "in_app": false, "push": true})
        );
        assert_eq!(
            request.quiet_hours,
            Some(serde_json::json!({
                "enabled": true,
                "start": "22:00",
                "end": "07:00"
            }))
        );
        assert_eq!(request.timezone.as_deref(), Some("Asia/Bangkok"));

        for invalid in [
            "email=true&in_app=false&push=true&quiet_enabled=true&quiet_start=22%3A00",
            "email=true&email=false&in_app=false&push=true&quiet_enabled=true&quiet_start=22%3A00&quiet_end=07%3A00",
            "email=true&in_app=false&push=true&quiet_enabled=1&quiet_start=22%3A00&quiet_end=07%3A00",
            "email=true&in_app=false&push=true&quiet_enabled=true&quiet_start=25%3A00&quiet_end=07%3A00",
            "email=true&in_app=false&push=true&quiet_enabled=true&quiet_start=22%3A00&quiet_end=07%3A00&unknown=x",
        ] {
            assert!(
                parse_notification_preferences_form(invalid.as_bytes()).is_err(),
                "form must fail closed: {invalid}"
            );
        }
    }

    #[test]
    fn notification_preferences_form_requires_same_origin_headers() {
        let mut headers = axum::http::HeaderMap::new();
        assert!(!same_origin_preferences_form(&headers));
        headers.insert(
            header::ORIGIN,
            HeaderValue::from_static("https://epsx.test"),
        );
        headers.insert(header::HOST, HeaderValue::from_static("epsx.test"));
        assert!(same_origin_preferences_form(&headers));
        headers.insert("sec-fetch-site", HeaderValue::from_static("cross-site"));
        assert!(!same_origin_preferences_form(&headers));
        headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));
        headers.insert(
            header::ORIGIN,
            HeaderValue::from_static("https://evil.test"),
        );
        assert!(!same_origin_preferences_form(&headers));
    }

    #[test]
    fn cookie_backed_notification_mutations_require_same_origin_but_bearer_callers_remain_supported(
    ) {
        let mut headers = axum::http::HeaderMap::new();
        assert!(!notification_mutation_origin_allowed(&headers));
        headers.insert(
            header::ORIGIN,
            HeaderValue::from_static("https://epsx.test"),
        );
        headers.insert(header::HOST, HeaderValue::from_static("epsx.test"));
        assert!(notification_mutation_origin_allowed(&headers));
        headers.insert(
            header::ORIGIN,
            HeaderValue::from_static("https://evil.test"),
        );
        assert!(!notification_mutation_origin_allowed(&headers));
        headers.remove(header::ORIGIN);
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer verified-token"),
        );
        assert!(notification_mutation_origin_allowed(&headers));
    }

    #[test]
    fn notification_preferences_form_redirect_sets_a_short_lived_matching_flash() {
        for (state, location) in [
            ("saved", "/account?preferences=saved"),
            ("error", "/account?preferences=error"),
            ("unexpected", "/account?preferences=error"),
        ] {
            let response = notification_preferences_form_redirect(state);
            assert_eq!(response.status(), StatusCode::SEE_OTHER);
            assert_eq!(
                response
                    .headers()
                    .get(header::LOCATION)
                    .and_then(|value| value.to_str().ok()),
                Some(location)
            );
            let cookie = response
                .headers()
                .get(header::SET_COOKIE)
                .and_then(|value| value.to_str().ok())
                .expect("form redirect must issue a flash cookie");
            let expected_state = if state == "saved" { "saved" } else { "error" };
            assert!(cookie.contains(&format!(
                "{NOTIFICATION_PREFERENCES_FLASH_COOKIE}={expected_state}"
            )));
            assert!(cookie.contains("Path=/account"));
            assert!(cookie.contains("Max-Age=30"));
            assert!(cookie.contains("HttpOnly"));
            assert!(cookie.contains("SameSite=Lax"));
        }
    }
}

#[cfg(test)]
mod auth_session_tests {
    use super::*;
    use axum::{
        body::Body,
        http::{header, HeaderMap, HeaderValue},
        routing::{delete, get, post, put},
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
            Err(response) => *response,
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
            notification_url: base_url.to_string(),
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
                "clicked_at": null,
                "title": "Title",
                "notification_type": "system",
                "priority": "normal",
                "action_url": null,
                "expires_at": null
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
        let read_observations = observations.clone();
        let read_expected_token = access_token.clone();
        let unread_mutation_observations = observations.clone();
        let unread_mutation_expected_token = access_token.clone();
        let ack_observations = observations.clone();
        let ack_expected_token = access_token.clone();
        let click_observations = observations.clone();
        let click_expected_token = access_token.clone();
        let dismiss_observations = observations.clone();
        let dismiss_expected_token = access_token.clone();
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
                        let mut payload = notification_payload(TEST_WALLET);
                        // This request asks for offset=2.  Model a
                        // snapshot with three matching rows so the single
                        // returned row is the final page item rather than
                        // an impossible row at offset two of a one-row
                        // result set.
                        payload["total"] = json!(3);
                        Json(payload).into_response()
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
            )
            .route(
                "/api/v1/notification/notification-id/read",
                post(move |headers: HeaderMap| {
                    let observations = read_observations.clone();
                    let expected_token = read_expected_token.clone();
                    async move {
                        observations.lock().unwrap().push(json!({
                            "endpoint": "read",
                            "authorization": headers
                                .get(header::AUTHORIZATION)
                                .and_then(|value| value.to_str().ok()),
                            "request_id": headers
                                .get("x-request-id")
                                .and_then(|value| value.to_str().ok()),
                            "cookie": headers
                                .get(header::COOKIE)
                                .and_then(|value| value.to_str().ok()),
                        }));
                        if headers
                            .get(header::AUTHORIZATION)
                            .and_then(|value| value.to_str().ok())
                            != Some(format!("Bearer {expected_token}").as_str())
                        {
                            return StatusCode::UNAUTHORIZED.into_response();
                        }
                        Json(json!({
                            "id": "notification-id",
                            "read_at": "2026-07-24T00:00:00Z"
                        }))
                        .into_response()
                    }
                }),
            )
            .route(
                "/api/v1/notification/notification-id/unread",
                post(move |headers: HeaderMap| {
                    let observations = unread_mutation_observations.clone();
                    let expected_token = unread_mutation_expected_token.clone();
                    async move {
                        observations.lock().unwrap().push(json!({
                            "endpoint": "unread-mutation",
                            "authorization": headers
                                .get(header::AUTHORIZATION)
                                .and_then(|value| value.to_str().ok()),
                            "request_id": headers
                                .get("x-request-id")
                                .and_then(|value| value.to_str().ok()),
                            "cookie": headers
                                .get(header::COOKIE)
                                .and_then(|value| value.to_str().ok()),
                        }));
                        if headers
                            .get(header::AUTHORIZATION)
                            .and_then(|value| value.to_str().ok())
                            != Some(format!("Bearer {expected_token}").as_str())
                        {
                            return StatusCode::UNAUTHORIZED.into_response();
                        }
                        StatusCode::NO_CONTENT.into_response()
                    }
                }),
            )
            .route(
                "/api/v1/notification/notification-id/click",
                post(move |headers: HeaderMap| {
                    let observations = click_observations.clone();
                    let expected_token = click_expected_token.clone();
                    async move {
                        observations.lock().unwrap().push(json!({
                            "endpoint": "click",
                            "authorization": headers
                                .get(header::AUTHORIZATION)
                                .and_then(|value| value.to_str().ok()),
                            "request_id": headers
                                .get("x-request-id")
                                .and_then(|value| value.to_str().ok()),
                            "cookie": headers
                                .get(header::COOKIE)
                                .and_then(|value| value.to_str().ok()),
                        }));
                        if headers
                            .get(header::AUTHORIZATION)
                            .and_then(|value| value.to_str().ok())
                            != Some(format!("Bearer {expected_token}").as_str())
                        {
                            return StatusCode::UNAUTHORIZED.into_response();
                        }
                        Json(json!({"id": "notification-id", "event": "clicked"})).into_response()
                    }
                }),
            )
            .route(
                "/api/v1/notification/notification-id/dismiss",
                post(move |headers: HeaderMap| {
                    let observations = dismiss_observations.clone();
                    let expected_token = dismiss_expected_token.clone();
                    async move {
                        observations.lock().unwrap().push(json!({
                            "endpoint": "dismiss",
                            "authorization": headers
                                .get(header::AUTHORIZATION)
                                .and_then(|value| value.to_str().ok()),
                            "request_id": headers
                                .get("x-request-id")
                                .and_then(|value| value.to_str().ok()),
                            "cookie": headers
                                .get(header::COOKIE)
                                .and_then(|value| value.to_str().ok()),
                        }));
                        if headers
                            .get(header::AUTHORIZATION)
                            .and_then(|value| value.to_str().ok())
                            != Some(format!("Bearer {expected_token}").as_str())
                        {
                            return StatusCode::UNAUTHORIZED.into_response();
                        }
                        Json(json!({"id": "notification-id", "event": "dismissed"})).into_response()
                    }
                }),
            )
            .route(
                "/api/v1/notification/notification-id/acknowledge",
                put(move |headers: HeaderMap| {
                    let observations = ack_observations.clone();
                    let expected_token = ack_expected_token.clone();
                    async move {
                        observations.lock().unwrap().push(json!({
                            "endpoint": "acknowledge",
                            "authorization": headers
                                .get(header::AUTHORIZATION)
                                .and_then(|value| value.to_str().ok()),
                            "request_id": headers
                                .get("x-request-id")
                                .and_then(|value| value.to_str().ok()),
                            "cookie": headers
                                .get(header::COOKIE)
                                .and_then(|value| value.to_str().ok()),
                        }));
                        if headers
                            .get(header::AUTHORIZATION)
                            .and_then(|value| value.to_str().ok())
                            != Some(format!("Bearer {expected_token}").as_str())
                        {
                            return StatusCode::UNAUTHORIZED.into_response();
                        }
                        Json(json!({
                            "id": "notification-id",
                            "acknowledged_at": "2026-07-24T00:00:00Z"
                        }))
                        .into_response()
                    }
                }),
            );
        let base_url = spawn_mock(router).await;
        let app_state = state(&base_url);
        let parsed_base_url = url::Url::parse(&base_url).unwrap();
        let form_host = format!(
            "{}:{}",
            parsed_base_url.host_str().unwrap(),
            parsed_base_url.port().unwrap()
        );
        let mut headers = request_headers(&format!("epsx.frontend.access_token={access_token}"));
        headers.insert(header::HOST, HeaderValue::from_str(&form_host).unwrap());
        headers.insert(header::ORIGIN, HeaderValue::from_str(&base_url).unwrap());
        headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));
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
        assert_eq!(response_json(list).await["total"], 3);

        let unread = notification_unread_count(State(app_state.clone()), headers.clone()).await;
        assert_eq!(unread.status(), StatusCode::OK);
        assert_private_notification_response(&unread);
        assert_eq!(response_json(unread).await, json!({"count": 7}));

        let read = notification_read(
            State(app_state.clone()),
            headers.clone(),
            AxPath("notification-id".to_string()),
        )
        .await;
        assert_eq!(read.status(), StatusCode::OK);
        assert_eq!(response_json(read).await, json!({"ok": true}));

        let unread_mutation = notification_unread(
            State(app_state.clone()),
            headers.clone(),
            AxPath("notification-id".to_string()),
        )
        .await;
        assert_eq!(unread_mutation.status(), StatusCode::OK);
        assert_eq!(response_json(unread_mutation).await, json!({"ok": true}));

        let acknowledged = notification_acknowledge(
            State(app_state.clone()),
            headers.clone(),
            AxPath("notification-id".to_string()),
        )
        .await;
        assert_eq!(acknowledged.status(), StatusCode::OK);
        assert_eq!(response_json(acknowledged).await, json!({"ok": true}));

        let clicked = notification_click(
            State(app_state.clone()),
            headers.clone(),
            AxPath("notification-id".to_string()),
        )
        .await;
        assert_eq!(clicked.status(), StatusCode::OK);
        assert_eq!(response_json(clicked).await, json!({"ok": true}));

        let dismissed = notification_dismiss(
            State(app_state.clone()),
            headers.clone(),
            AxPath("notification-id".to_string()),
        )
        .await;
        assert_eq!(dismissed.status(), StatusCode::OK);
        assert_eq!(response_json(dismissed).await, json!({"ok": true}));

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
        assert_eq!(observations.len(), 8);
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

        let unread_observations: Vec<_> = observations
            .iter()
            .filter(|observation| observation["endpoint"] == "unread")
            .collect();
        assert_eq!(unread_observations.len(), 2);
        for unread_observation in unread_observations.iter().copied() {
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
            unread_observations[0]["request_id"],
            "22222222-2222-4222-8222-222222222222"
        );
        let generated_request_id = unread_observations[1]["request_id"].as_str().unwrap();
        assert_ne!(generated_request_id, "not-a-request-id");
        assert!(uuid::Uuid::parse_str(generated_request_id).is_ok());
        for endpoint in ["read", "unread-mutation"] {
            let observation = observations
                .iter()
                .find(|observation| observation["endpoint"] == endpoint)
                .expect("legacy mutation alias must reach the service");
            assert_eq!(
                observation["authorization"],
                format!("Bearer {access_token}")
            );
            assert_eq!(
                observation["request_id"],
                "22222222-2222-4222-8222-222222222222"
            );
            assert!(observation["cookie"].is_null());
        }
        let acknowledge_observation = observations
            .iter()
            .find(|observation| observation["endpoint"] == "acknowledge")
            .expect("acknowledge request must reach the service");
        assert_eq!(
            acknowledge_observation["authorization"],
            format!("Bearer {access_token}")
        );
        assert_eq!(
            acknowledge_observation["request_id"],
            "22222222-2222-4222-8222-222222222222"
        );
        assert!(acknowledge_observation["cookie"].is_null());
        for endpoint in ["click", "dismiss"] {
            let observation = observations
                .iter()
                .find(|observation| observation["endpoint"] == endpoint)
                .expect("engagement request must reach the service");
            assert_eq!(
                observation["authorization"],
                format!("Bearer {access_token}")
            );
            assert_eq!(
                observation["request_id"],
                "22222222-2222-4222-8222-222222222222"
            );
            assert!(observation["cookie"].is_null());
        }
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
    async fn notification_preferences_push_and_stream_forward_only_verified_context() {
        let key = TestKey::generate();
        let access_token = key.access_token(&[]);
        let expected_authorization = format!("Bearer {access_token}");
        let jwks = Jwks {
            keys: vec![key.jwk.clone()],
        };
        let observations = Arc::new(std::sync::Mutex::new(Vec::<Value>::new()));
        let preference_get_observations = observations.clone();
        let preference_put_observations = observations.clone();
        let push_get_observations = observations.clone();
        let push_put_observations = observations.clone();
        let push_delete_observations = observations.clone();
        let stream_observations = observations.clone();
        let stream_ack_observations = observations.clone();
        let expected_for_preferences = expected_authorization.clone();
        let expected_for_preference_put = expected_authorization.clone();
        let expected_for_push_get = expected_authorization.clone();
        let expected_for_push_put = expected_authorization.clone();
        let expected_for_push_delete = expected_authorization.clone();
        let expected_for_stream = expected_authorization.clone();
        let expected_for_stream_ack = expected_authorization.clone();
        let router = Router::new()
            .route(
                JWKS_PATH,
                get(move || {
                    let jwks = jwks.clone();
                    async move { Json(jwks) }
                }),
            )
            .route(
                "/api/v1/notification/preferences",
                get(move |headers: HeaderMap| {
                    let observations = preference_get_observations.clone();
                    let expected = expected_for_preferences.clone();
                    async move {
                        observations.lock().unwrap().push(json!({
                            "endpoint": "preferences-get",
                            "authorization": headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()),
                            "request_id": headers.get("x-request-id").and_then(|v| v.to_str().ok()),
                            "cookie": headers.get(header::COOKIE).and_then(|v| v.to_str().ok()),
                        }));
                        if headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok())
                            != Some(expected.as_str())
                        {
                            return StatusCode::UNAUTHORIZED.into_response();
                        }
                        Json(json!({
                            "channels": {"email": true, "in_app": true, "push": false},
                            "quiet_hours": null,
                            "timezone": "UTC",
                            "updated_at": null
                        }))
                        .into_response()
                    }
                })
                .put(move |headers: HeaderMap, Json(body): Json<Value>| {
                    let observations = preference_put_observations.clone();
                    let expected = expected_for_preference_put.clone();
                    async move {
                        observations.lock().unwrap().push(json!({
                            "endpoint": "preferences-put",
                            "authorization": headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()),
                            "request_id": headers.get("x-request-id").and_then(|v| v.to_str().ok()),
                            "body": body,
                        }));
                        if headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok())
                            != Some(expected.as_str())
                        {
                            return StatusCode::UNAUTHORIZED.into_response();
                        }
                        Json(json!({
                            "channels": {"email": false, "in_app": true, "push": false},
                            "quiet_hours": {"start": "22:00", "end": "07:00"},
                            "timezone": "Asia/Bangkok",
                            "updated_at": null
                        }))
                        .into_response()
                    }
                }),
            )
            .route(
                "/api/v1/notification/push",
                get(move |headers: HeaderMap| {
                    let observations = push_get_observations.clone();
                    let expected = expected_for_push_get.clone();
                    async move {
                        observations.lock().unwrap().push(json!({
                            "endpoint": "push-get",
                            "authorization": headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()),
                            "request_id": headers.get("x-request-id").and_then(|v| v.to_str().ok()),
                        }));
                        if headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok())
                            != Some(expected.as_str())
                        {
                            return StatusCode::UNAUTHORIZED.into_response();
                        }
                        Json(json!({"enabled": true, "subscribed": false, "public_key": "B".repeat(65)}))
                            .into_response()
                    }
                })
                .put(move |headers: HeaderMap, Json(body): Json<Value>| {
                    let observations = push_put_observations.clone();
                    let expected = expected_for_push_put.clone();
                    async move {
                        observations.lock().unwrap().push(json!({
                            "endpoint": "push-put",
                            "authorization": headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()),
                            "request_id": headers.get("x-request-id").and_then(|v| v.to_str().ok()),
                            "body": body,
                        }));
                        if headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok())
                            != Some(expected.as_str())
                        {
                            return StatusCode::UNAUTHORIZED.into_response();
                        }
                        Json(json!({"enabled": true, "subscribed": true, "public_key": "B".repeat(65)}))
                            .into_response()
                    }
                })
                .delete(move |headers: HeaderMap, Json(body): Json<Value>| {
                    let observations = push_delete_observations.clone();
                    let expected = expected_for_push_delete.clone();
                    async move {
                        observations.lock().unwrap().push(json!({
                            "endpoint": "push-delete",
                            "authorization": headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()),
                            "request_id": headers.get("x-request-id").and_then(|v| v.to_str().ok()),
                            "body": body,
                        }));
                        if headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok())
                            != Some(expected.as_str())
                        {
                            return StatusCode::UNAUTHORIZED.into_response();
                        }
                        Json(json!({"enabled": true, "subscribed": false, "public_key": "B".repeat(65)}))
                            .into_response()
                    }
                }),
            )
            .route(
                "/api/v1/notification/stream",
                get(move |headers: HeaderMap| {
                    let observations = stream_observations.clone();
                    let expected = expected_for_stream.clone();
                    async move {
                        observations.lock().unwrap().push(json!({
                            "endpoint": "stream",
                            "authorization": headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()),
                            "request_id": headers.get("x-request-id").and_then(|v| v.to_str().ok()),
                            "last_event_id": headers.get("last-event-id").and_then(|v| v.to_str().ok()),
                        }));
                        if headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok())
                            != Some(expected.as_str())
                        {
                            return StatusCode::UNAUTHORIZED.into_response();
                        }
                        Response::builder()
                            .status(StatusCode::OK)
                            .header(header::CONTENT_TYPE, "text/event-stream")
                            .body(Body::from("event: ready\\nid: 1\\ndata: {}\\n\\n"))
                            .unwrap()
                    }
                }),
            );
        let router = router.route(
            "/api/v1/notification/stream/ack",
            post(move |headers: HeaderMap, Json(body): Json<Value>| {
                let observations = stream_ack_observations.clone();
                let expected = expected_for_stream_ack.clone();
                async move {
                    observations.lock().unwrap().push(json!({
                        "endpoint": "stream-ack",
                        "authorization": headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()),
                        "request_id": headers.get("x-request-id").and_then(|v| v.to_str().ok()),
                        "body": body,
                    }));
                    if headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok())
                        != Some(expected.as_str())
                    {
                        return StatusCode::UNAUTHORIZED.into_response();
                    }
                    Json(json!({"ok": true, "event_id": "cursor-1"})).into_response()
                }
            }),
        );
        let base_url = spawn_mock(router).await;
        let app_state = state(&base_url);
        let cookie = format!("epsx.frontend.access_token={access_token}");
        let parsed_base_url = url::Url::parse(&base_url).unwrap();
        let form_host = format!(
            "{}:{}",
            parsed_base_url.host_str().unwrap(),
            parsed_base_url.port().unwrap()
        );
        let mut headers = request_headers(&cookie);
        headers.insert(
            "x-request-id",
            HeaderValue::from_static("33333333-3333-4333-8333-333333333333"),
        );
        headers.insert("last-event-id", HeaderValue::from_static("cursor-1"));

        let preferences =
            notification_preferences_get(State(app_state.clone()), headers.clone()).await;
        assert_eq!(preferences.status(), StatusCode::OK);
        assert_private_notification_response(&preferences);
        assert_eq!(response_json(preferences).await["timezone"], "UTC");

        let put_request = Request::builder()
            .method("PUT")
            .uri("/api/v1/notifications/preferences")
            .header(header::COOKIE, &cookie)
            .header(header::HOST, &form_host)
            .header(header::ORIGIN, &base_url)
            .header("sec-fetch-site", "same-origin")
            .header("x-request-id", "33333333-3333-4333-8333-333333333333")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                r#"{"channels":{"email":false,"in_app":true},"quiet_hours":null,"timezone":"Asia/Bangkok"}"#,
            ))
            .unwrap();
        let preferences_put =
            notification_preferences_put(State(app_state.clone()), put_request).await;
        assert_eq!(preferences_put.status(), StatusCode::OK);
        assert_private_notification_response(&preferences_put);
        assert_eq!(
            response_json(preferences_put).await["timezone"],
            "Asia/Bangkok"
        );

        let form_request = Request::builder()
            .method("POST")
            .uri("/account/notification-preferences")
            .header(header::COOKIE, &cookie)
            .header(header::HOST, &form_host)
            .header(header::ORIGIN, &base_url)
            .header("sec-fetch-site", "same-origin")
            .header("x-request-id", "33333333-3333-4333-8333-333333333333")
            .header(
                header::CONTENT_TYPE,
                "application/x-www-form-urlencoded;charset=UTF-8",
            )
            .body(Body::from(
                "email=false&in_app=true&push=false&quiet_enabled=true&quiet_start=22%3A00&quiet_end=07%3A00&timezone=Asia%2FBangkok",
            ))
            .unwrap();
        let form_response =
            notification_preferences_form(State(app_state.clone()), form_request).await;
        assert_eq!(form_response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            form_response.headers().get(header::LOCATION),
            Some(&HeaderValue::from_static("/account?preferences=saved"))
        );
        assert_eq!(
            form_response
                .headers()
                .get(header::SET_COOKIE)
                .and_then(|value| value.to_str().ok())
                .map(|value| value.contains("epsx.notification_preferences_flash=saved")),
            Some(true)
        );

        let rejected_form = Request::builder()
            .method("POST")
            .uri("/account/notification-preferences")
            .header(header::COOKIE, &cookie)
            .header(header::HOST, &form_host)
            .header(header::ORIGIN, "https://foreign.example")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(
                "email=true&in_app=true&push=false&quiet_enabled=false&quiet_start=22%3A00&quiet_end=07%3A00",
            ))
            .unwrap();
        let rejected_response =
            notification_preferences_form(State(app_state.clone()), rejected_form).await;
        assert_eq!(rejected_response.status(), StatusCode::FORBIDDEN);

        let push_status = notification_push_status(State(app_state.clone()), headers.clone()).await;
        assert_eq!(push_status.status(), StatusCode::OK);
        assert_private_notification_response(&push_status);
        assert_eq!(response_json(push_status).await["enabled"], true);

        let push_body = r#"{"endpoint":"https://push.example.test/subscription","p256dh":"key_123","auth":"auth_123","user_agent":"browser"}"#;
        let push_put_request = Request::builder()
            .method("PUT")
            .uri("/api/v1/notifications/push")
            .header(header::COOKIE, &cookie)
            .header(header::HOST, &form_host)
            .header(header::ORIGIN, &base_url)
            .header("sec-fetch-site", "same-origin")
            .header("x-request-id", "33333333-3333-4333-8333-333333333333")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(push_body))
            .unwrap();
        let push_put =
            notification_push_subscribe(State(app_state.clone()), push_put_request).await;
        assert_eq!(push_put.status(), StatusCode::OK);
        assert_private_notification_response(&push_put);
        assert_eq!(response_json(push_put).await["subscribed"], true);

        let push_delete_request = Request::builder()
            .method("DELETE")
            .uri("/api/v1/notifications/push")
            .header(header::COOKIE, &cookie)
            .header(header::HOST, &form_host)
            .header(header::ORIGIN, &base_url)
            .header("sec-fetch-site", "same-origin")
            .header("x-request-id", "33333333-3333-4333-8333-333333333333")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                r#"{"endpoint":"https://push.example.test/subscription"}"#,
            ))
            .unwrap();
        let push_delete =
            notification_push_unsubscribe(State(app_state.clone()), push_delete_request).await;
        assert_eq!(push_delete.status(), StatusCode::OK);
        assert_private_notification_response(&push_delete);
        assert_eq!(response_json(push_delete).await["subscribed"], false);

        let stream = notification_stream(State(app_state), headers).await;
        assert_eq!(stream.status(), StatusCode::OK);
        assert_eq!(
            stream.headers().get(header::CONTENT_TYPE),
            Some(&HeaderValue::from_static(
                "text/event-stream; charset=utf-8"
            ))
        );
        assert_eq!(
            stream.headers().get(header::CACHE_CONTROL),
            Some(&HeaderValue::from_static("private, no-store"))
        );
        assert_eq!(
            stream.headers().get(header::VARY),
            Some(&HeaderValue::from_static(
                "Cookie, Authorization, Last-Event-ID"
            ))
        );

        let ack_request = Request::builder()
            .method("POST")
            .uri("/api/v1/notifications/stream/ack")
            .header(header::COOKIE, &cookie)
            .header(header::HOST, &form_host)
            .header(header::ORIGIN, &base_url)
            .header("sec-fetch-site", "same-origin")
            .header("x-request-id", "33333333-3333-4333-8333-333333333333")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"event_id":"cursor-1"}"#))
            .unwrap();
        let acknowledged = notification_stream_ack(State(state(&base_url)), ack_request).await;
        assert_eq!(acknowledged.status(), StatusCode::OK);
        assert_private_notification_response(&acknowledged);
        assert_eq!(response_json(acknowledged).await["event_id"], "cursor-1");

        let observations = observations.lock().unwrap();
        assert_eq!(observations.len(), 8);
        for observation in observations.iter() {
            assert_eq!(observation["authorization"], expected_authorization);
            assert_eq!(
                observation["request_id"],
                "33333333-3333-4333-8333-333333333333"
            );
        }
        assert_eq!(observations[6]["last_event_id"], "cursor-1");
        assert_eq!(observations[7]["endpoint"], "stream-ack");
        assert_eq!(observations[7]["body"]["event_id"], "cursor-1");
        assert_eq!(observations[1]["body"]["timezone"], "Asia/Bangkok");
        assert_eq!(observations[2]["body"]["timezone"], "Asia/Bangkok");
        assert_eq!(
            observations[4]["body"]["endpoint"],
            "https://push.example.test/subscription"
        );
        assert_eq!(
            observations[5]["body"]["endpoint"],
            "https://push.example.test/subscription"
        );
    }

    #[tokio::test]
    async fn ssr_notification_preferences_loader_is_strict_and_failure_typed() {
        let ready_router = Router::new().route(
            "/api/v1/notification/preferences",
            get(|| async {
                Json(json!({
                    "channels": {"email": true, "in_app": false},
                    "quiet_hours": {"start": "22:00", "end": "07:00"},
                    "timezone": "UTC",
                    "updated_at": null
                }))
            }),
        );
        let ready = load_notification_preferences(
            &notification_client(&spawn_mock(ready_router).await, Duration::from_secs(1)),
            "verified-bearer",
            &test_notification_request_id(),
        )
        .await;
        assert!(
            matches!(ready, NotificationPreferencesLoadOutcome::Ready(value) if value["timezone"] == "UTC")
        );

        let malformed_router = Router::new().route(
            "/api/v1/notification/preferences",
            get(|| async {
                Json(json!({
                    "channels": {"webhook": true},
                    "quiet_hours": null,
                    "timezone": "UTC",
                    "updated_at": null
                }))
            }),
        );
        let malformed = load_notification_preferences(
            &notification_client(&spawn_mock(malformed_router).await, Duration::from_secs(1)),
            "verified-bearer",
            &test_notification_request_id(),
        )
        .await;
        assert_eq!(
            malformed,
            NotificationPreferencesLoadOutcome::Error(NotificationPreferencesLoadError::Malformed)
        );

        let unavailable_router = Router::new().route(
            "/api/v1/notification/preferences",
            get(|| async { StatusCode::SERVICE_UNAVAILABLE }),
        );
        let unavailable = load_notification_preferences(
            &notification_client(
                &spawn_mock(unavailable_router).await,
                Duration::from_secs(1),
            ),
            "verified-bearer",
            &test_notification_request_id(),
        )
        .await;
        assert_eq!(
            unavailable,
            NotificationPreferencesLoadOutcome::Error(
                NotificationPreferencesLoadError::DependencyUnavailable
            )
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

//! Dioxus SSR rendering for the frontend BFF.
//!
//! Server-side fetches happen in this layer: the page request comes in,
//! the BFF resolves the verified user, optionally fetches page-specific
//! data from the gateway (cached in the SSR result), and renders the
//! Dioxus VNode with all data baked in. The page renders fully on the
//! server; client-side hydration then takes over for interactivity.
//!
//! Auth gating (Wave 23 T3): when an unauthenticated request lands on
//! a protected path, we 307-redirect to `/auth?return_url=<path>` to
//! match the prod Vercel middleware convention
//! (`apps-old/frontend/middleware.ts::handleUnauthenticated` reads
//! `?return_url=` and reads/writes the `epsx.return_url` cookie).
//! The shared wallet bridge and page-level `AuthGate` connect links use the
//! same `?return_url=` parameter, so the round-trip remains same-origin.

use axum::{
    extract::{Request, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use epsx_bff::session::AccessVerification;
use epsx_dioxus_ui::auth::wallet_button::ConnectedWalletState;
use epsx_dioxus_ui::auth::User;
use epsx_dioxus_ui::components::account::{
    decode_pay_history, ACCOUNT_PAYMENT_HISTORY_DATA_PARAM, ACCOUNT_PAYMENT_HISTORY_EMPTY,
    ACCOUNT_PAYMENT_HISTORY_MALFORMED, ACCOUNT_PAYMENT_HISTORY_READY,
    ACCOUNT_PAYMENT_HISTORY_STATE_PARAM, ACCOUNT_PAYMENT_HISTORY_UNAVAILABLE,
};
use epsx_dioxus_ui::pages::account::{
    ACCOUNT_NOTIFICATION_PREFERENCES_DATA_PARAM, ACCOUNT_NOTIFICATION_PREFERENCES_FORM_STATE_PARAM,
    ACCOUNT_NOTIFICATION_PREFERENCES_STATE_PARAM,
};
use epsx_dioxus_ui::pages::analytics::{
    AnalyticsFilters, AnalyticsQueryState, AnalyticsResponse, WatchlistData, ANALYTICS_DATA_PARAM,
    ANALYTICS_FILTERS_DATA_PARAM, ANALYTICS_FILTERS_STATE_PARAM, ANALYTICS_QUERY_PARAM,
    ANALYTICS_STATE_PARAM, ANALYTICS_WATCHLIST_DATA_PARAM, ANALYTICS_WATCHLIST_STATE_PARAM,
};
use epsx_dioxus_ui::pages::auth_page::{
    AUTH_PAGE_SESSION_STATE_PARAM, AUTH_PAGE_SESSION_STATE_RECOVERING,
    AUTH_PAGE_SESSION_STATE_SIGNED_OUT, AUTH_PAGE_SESSION_STATE_VERIFIER_UNAVAILABLE,
};
use epsx_dioxus_ui::pages::chat::{
    CHAT_DETAIL_DATA_PARAM, CHAT_DETAIL_STATE_PARAM, CHAT_EMPTY, CHAT_FORBIDDEN,
    CHAT_INBOX_DATA_PARAM, CHAT_INBOX_STATE_PARAM, CHAT_MALFORMED, CHAT_READY, CHAT_UNAVAILABLE,
};
use epsx_dioxus_ui::pages::developer::{
    DEVELOPER_DATA_PARAM, DEVELOPER_OPENAPI_DATA_PARAM, DEVELOPER_OPENAPI_STATE_PARAM,
    DEVELOPER_STATE_PARAM, DEVELOPER_USAGE_DATA_PARAM, DEVELOPER_USAGE_STATE_PARAM, LOAD_EMPTY,
    LOAD_FORBIDDEN, LOAD_MALFORMED, LOAD_READY, LOAD_UNAVAILABLE,
};
use epsx_dioxus_ui::pages::home::{HOME_ANALYTICS_DATA_PARAM, HOME_ANALYTICS_STATE_PARAM};
use epsx_dioxus_ui::pages::portfolio::{
    WatchlistLayoutData, PORTFOLIO_WATCHLIST_DATA_PARAM, PORTFOLIO_WATCHLIST_STATE_PARAM,
};
use epsx_dioxus_ui::pages::{
    is_known_frontend_route, render_page, PageContext, PageMeta, PageStatus,
};
use std::collections::HashMap;

use super::auth;
use super::AppState;

/// Paths that 307-redirect to /auth when the user is unauthenticated,
/// matching the prod (https://epsx.io) Vercel middleware behavior. The
/// prod baseline shows the /auth page for these routes; without the
/// redirect the dev bff returns 200 + "Sign in required" gate, which
/// diverges from prod and inflates pixel diff.
///
/// Wave 35b T1 added `/about`, `/contact`, and `/offline` for pixel
/// parity with the pinned middleware. B7 removes `/offline`: a PWA
/// recovery surface must remain reachable without a session, especially
/// when authentication cannot complete because the browser is disconnected.
const UNAUTH_REDIRECT_PATHS: &[&str] = &[
    "/permissions",
    "/notifications",
    "/profile",
    "/about",
    "/contact",
];

const NOTIFICATIONS_DATA_PARAM: &str = "data_notifications";
const NOTIFICATIONS_STATE_PARAM: &str = "data_notifications_state";
const NOTIFICATIONS_PAGE_PARAM: &str = "data_notifications_page";
const NOTIFICATIONS_STATUS_PARAM: &str = "data_notifications_status";
const NOTIFICATIONS_TYPE_PARAM: &str = "data_notifications_type";
const NOTIFICATIONS_PRIORITY_PARAM: &str = "data_notifications_priority";
const NOTIFICATIONS_START_DATE_PARAM: &str = "data_notifications_start_date";
const NOTIFICATIONS_END_DATE_PARAM: &str = "data_notifications_end_date";
const NOTIFICATIONS_INVALID_QUERY: &str = "invalid_query";
const NOTIFICATIONS_TYPE_VALUES: &[&str] = &[
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
const NOTIFICATIONS_PRIORITY_VALUES: &[&str] = &["low", "normal", "high", "critical", "urgent"];
const HOME_NEWS_DATA_PARAM: &str = "data_home_news";
const ACCOUNT_PAYMENT_HISTORY_LIMIT: usize = 10;
const ACCOUNT_NOTIFICATION_PREFERENCES_READY: &str = "ready";
const ACCOUNT_NOTIFICATION_PREFERENCES_UNAVAILABLE: &str = "unavailable";
const ACCOUNT_NOTIFICATION_PREFERENCES_MALFORMED: &str = "malformed";
const ANALYTICS_RANKINGS_PATH: &str = "/api/analytics/rankings";
const ANALYTICS_FILTERS_PATH: &str = "/api/analytics/filters";
const ANALYTICS_WATCHLIST_PATH: &str = "/api/users/watchlist";
const PORTFOLIO_WATCHLIST_LAYOUT_PATH: &str = "/api/users/watchlist/layout";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AnalyticsLoadError {
    Unavailable,
    Malformed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AccountPaymentHistoryLoadError {
    Unavailable,
    Malformed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NotificationPageRequest {
    page: u32,
    status: Option<String>,
    notification_type: Option<String>,
    priority: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
}

impl NotificationPageRequest {
    /// Accept only a canonical positive decimal `page` and the explicit
    /// read/unread/all, type, and priority filters. Every other public query
    /// field fails closed before an owner service request can be made.
    fn parse(raw_query: &str) -> Result<Self, ()> {
        if raw_query.is_empty() {
            return Ok(Self {
                page: 1,
                status: None,
                notification_type: None,
                priority: None,
                start_date: None,
                end_date: None,
            });
        }
        let url = reqwest::Url::parse(&format!("https://frontend.invalid/?{raw_query}"))
            .map_err(|_| ())?;
        let mut page = 1;
        let mut status = None;
        let mut notification_type = None;
        let mut priority = None;
        let mut start_date = None;
        let mut end_date = None;
        let mut seen = std::collections::HashSet::new();
        for (key, value) in url.query_pairs() {
            if !seen.insert(key.to_string()) {
                return Err(());
            }
            match key.as_ref() {
                "page" => {
                    if value.is_empty()
                        || (value.len() > 1 && value.starts_with('0'))
                        || !value.bytes().all(|byte| byte.is_ascii_digit())
                    {
                        return Err(());
                    }
                    page = value.parse::<u32>().map_err(|_| ())?;
                }
                "status" => {
                    if !matches!(value.as_ref(), "all" | "read" | "unread") {
                        return Err(());
                    }
                    if value != "all" {
                        status = Some(value.into_owned());
                    }
                }
                "type" => {
                    if value == "all" {
                        continue;
                    }
                    if !NOTIFICATIONS_TYPE_VALUES.contains(&value.as_ref()) {
                        return Err(());
                    }
                    notification_type = Some(value.into_owned());
                }
                "priority" => {
                    if value == "all" {
                        continue;
                    }
                    if !NOTIFICATIONS_PRIORITY_VALUES.contains(&value.as_ref()) {
                        return Err(());
                    }
                    priority = Some(value.into_owned());
                }
                "start_date" | "end_date" => {
                    if value.len() > 64 {
                        return Err(());
                    }
                    if chrono::DateTime::parse_from_rfc3339(value.as_ref()).is_err() {
                        return Err(());
                    }
                    if key == "start_date" {
                        start_date = Some(value.into_owned());
                    } else {
                        end_date = Some(value.into_owned());
                    }
                }
                _ => return Err(()),
            }
        }
        if start_date
            .as_deref()
            .zip(end_date.as_deref())
            .is_some_and(|(start, end)| {
                chrono::DateTime::parse_from_rfc3339(start).ok()
                    > chrono::DateTime::parse_from_rfc3339(end).ok()
            })
        {
            return Err(());
        }
        crate::api::NotificationListQuery::for_ssr_page_and_filters_and_dates(
            page,
            status.as_deref(),
            notification_type.as_deref(),
            priority.as_deref(),
            start_date.as_deref(),
            end_date.as_deref(),
        )
        .map(|_| Self {
            page,
            status,
            notification_type,
            priority,
            start_date,
            end_date,
        })
        .ok_or(())
    }

    fn service_query(&self) -> crate::api::NotificationListQuery {
        crate::api::NotificationListQuery::for_ssr_page_and_filters_and_dates(
            self.page,
            self.status.as_deref(),
            self.notification_type.as_deref(),
            self.priority.as_deref(),
            self.start_date.as_deref(),
            self.end_date.as_deref(),
        )
        .expect("a parsed notification page remains within the fixed offset window")
    }
}

async fn load_notification_page(
    client: &epsx_client::ServiceClient,
    bearer: &str,
    owner: &str,
    headers: &axum::http::HeaderMap,
    raw_query: &str,
) -> Result<
    (
        u32,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        crate::api::NotificationListLoadOutcome,
    ),
    (),
> {
    let page_request = NotificationPageRequest::parse(raw_query)?;
    let request_id = crate::api::notification_request_id(headers);
    let outcome = crate::api::load_owner_notifications(
        client,
        bearer,
        owner,
        &page_request.service_query(),
        &request_id,
    )
    .await;
    Ok((
        page_request.page,
        page_request.status,
        page_request.notification_type,
        page_request.priority,
        page_request.start_date,
        page_request.end_date,
        outcome,
    ))
}

fn auth_page_session_state(
    path: &str,
    access_verification: &AccessVerification,
    refresh_cookie_present: bool,
) -> Option<&'static str> {
    if path != "/auth" {
        return None;
    }

    Some(match access_verification {
        AccessVerification::MissingOrRejected if refresh_cookie_present => {
            AUTH_PAGE_SESSION_STATE_RECOVERING
        }
        AccessVerification::VerifierUnavailable => AUTH_PAGE_SESSION_STATE_VERIFIER_UNAVAILABLE,
        AccessVerification::MissingOrRejected | AccessVerification::Verified { .. } => {
            AUTH_PAGE_SESSION_STATE_SIGNED_OUT
        }
    })
}

/// Record the notification dependency outcome without turning an upstream
/// failure into an empty or demo list. The Dioxus page treats `ok` as
/// permission to parse the exact service payload and every other state as
/// unavailable.
struct NotificationLoadSelection<'a> {
    page: u32,
    status: Option<&'a str>,
    notification_type: Option<&'a str>,
    priority: Option<&'a str>,
    start_date: Option<&'a str>,
    end_date: Option<&'a str>,
}

fn record_notification_load(
    params: &mut HashMap<String, String>,
    selection: NotificationLoadSelection<'_>,
    outcome: crate::api::NotificationListLoadOutcome,
) {
    let NotificationLoadSelection {
        page,
        status,
        notification_type,
        priority,
        start_date,
        end_date,
    } = selection;
    params.remove(NOTIFICATIONS_DATA_PARAM);
    params.remove(NOTIFICATIONS_STATE_PARAM);
    params.remove(NOTIFICATIONS_STATUS_PARAM);
    params.remove(NOTIFICATIONS_TYPE_PARAM);
    params.remove(NOTIFICATIONS_PRIORITY_PARAM);
    params.remove(NOTIFICATIONS_START_DATE_PARAM);
    params.remove(NOTIFICATIONS_END_DATE_PARAM);
    params.insert(NOTIFICATIONS_PAGE_PARAM.into(), page.to_string());
    params.insert(
        NOTIFICATIONS_STATUS_PARAM.into(),
        status.unwrap_or("all").to_string(),
    );
    params.insert(
        NOTIFICATIONS_TYPE_PARAM.into(),
        notification_type.unwrap_or("all").to_string(),
    );
    params.insert(
        NOTIFICATIONS_PRIORITY_PARAM.into(),
        priority.unwrap_or("all").to_string(),
    );
    params.insert(
        NOTIFICATIONS_START_DATE_PARAM.into(),
        start_date.unwrap_or("all").to_string(),
    );
    params.insert(
        NOTIFICATIONS_END_DATE_PARAM.into(),
        end_date.unwrap_or("all").to_string(),
    );
    match outcome {
        crate::api::NotificationListLoadOutcome::Ready(value)
        | crate::api::NotificationListLoadOutcome::Empty(value) => {
            params.insert(NOTIFICATIONS_DATA_PARAM.into(), value.to_string());
            params.insert(NOTIFICATIONS_STATE_PARAM.into(), "ok".into());
        }
        crate::api::NotificationListLoadOutcome::Unavailable(_) => {
            params.insert(NOTIFICATIONS_STATE_PARAM.into(), "error".into());
        }
        crate::api::NotificationListLoadOutcome::Malformed => {
            params.insert(NOTIFICATIONS_STATE_PARAM.into(), "malformed".into());
        }
    }
}

fn record_invalid_notification_query(params: &mut HashMap<String, String>) {
    params.remove(NOTIFICATIONS_DATA_PARAM);
    params.remove(NOTIFICATIONS_PAGE_PARAM);
    params.remove(NOTIFICATIONS_STATUS_PARAM);
    params.remove(NOTIFICATIONS_TYPE_PARAM);
    params.remove(NOTIFICATIONS_PRIORITY_PARAM);
    params.remove(NOTIFICATIONS_START_DATE_PARAM);
    params.remove(NOTIFICATIONS_END_DATE_PARAM);
    params.insert(
        NOTIFICATIONS_STATE_PARAM.into(),
        NOTIFICATIONS_INVALID_QUERY.into(),
    );
}

fn record_account_notification_preferences_load(
    params: &mut HashMap<String, String>,
    outcome: crate::api::NotificationPreferencesLoadOutcome,
) {
    params.remove(ACCOUNT_NOTIFICATION_PREFERENCES_DATA_PARAM);
    let state = match outcome {
        crate::api::NotificationPreferencesLoadOutcome::Ready(value) => {
            params.insert(
                ACCOUNT_NOTIFICATION_PREFERENCES_DATA_PARAM.to_string(),
                value.to_string(),
            );
            ACCOUNT_NOTIFICATION_PREFERENCES_READY
        }
        crate::api::NotificationPreferencesLoadOutcome::Error(error) => match error {
            crate::api::NotificationPreferencesLoadError::DependencyUnavailable
            | crate::api::NotificationPreferencesLoadError::UpstreamFailed => {
                ACCOUNT_NOTIFICATION_PREFERENCES_UNAVAILABLE
            }
            crate::api::NotificationPreferencesLoadError::Malformed => {
                ACCOUNT_NOTIFICATION_PREFERENCES_MALFORMED
            }
        },
    };
    params.insert(
        ACCOUNT_NOTIFICATION_PREFERENCES_STATE_PARAM.to_string(),
        state.to_string(),
    );
}

fn account_notification_preferences_flash_state(
    headers: &axum::http::HeaderMap,
    query: &str,
) -> Option<&'static str> {
    let requested = match query {
        "preferences=saved" => "saved",
        "preferences=error" => "error",
        _ => return None,
    };
    let cookie_state = headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookie_header| {
            let mut state = None;
            for part in cookie_header.split(';') {
                let Some((name, value)) = part.trim().split_once('=') else {
                    continue;
                };
                if name == crate::api::NOTIFICATION_PREFERENCES_FLASH_COOKIE
                    && state.replace(value).is_some()
                {
                    return None;
                }
            }
            state
        });
    (cookie_state == Some(requested)).then_some(requested)
}

fn record_account_notification_preferences_form_state(
    params: &mut HashMap<String, String>,
    state: Option<&str>,
) {
    params.remove(ACCOUNT_NOTIFICATION_PREFERENCES_FORM_STATE_PARAM);
    if let Some(state) = state {
        params.insert(
            ACCOUNT_NOTIFICATION_PREFERENCES_FORM_STATE_PARAM.to_string(),
            state.to_string(),
        );
    }
}

async fn load_home_news(
    client: &epsx_client::ServiceClient,
    path: &str,
) -> Option<crate::api::NewsListLoadOutcome> {
    if !matches!(path, "/" | "/index") {
        return None;
    }
    Some(crate::api::load_news_list(client, &crate::api::NewsQuery::default()).await)
}

fn record_home_news_load(
    params: &mut HashMap<String, String>,
    outcome: crate::api::NewsListLoadOutcome,
) {
    params.insert(
        HOME_NEWS_DATA_PARAM.to_string(),
        serde_json::to_string(&outcome).expect("home news outcome is serializable"),
    );
}

async fn load_home_analytics(
    client: &epsx_client::ServiceClient,
    path: &str,
) -> Option<Result<AnalyticsResponse, AnalyticsLoadError>> {
    if !matches!(path, "/" | "/index") {
        return None;
    }
    let value = match client
        .get_plain("/api/analytics/rankings?page=1&limit=3")
        .await
    {
        Ok(value) => value,
        Err(error) => {
            tracing::warn!("home rankings dependency unavailable: {error}");
            return Some(Err(AnalyticsLoadError::Unavailable));
        }
    };
    let response = match serde_json::from_value::<AnalyticsResponse>(value)
        .ok()
        .and_then(|response| response.validated().ok())
    {
        Some(response)
            if response.pagination.page == 1
                && response.pagination.limit == 3
                && response.data.len() <= 3 =>
        {
            response
        }
        _ => {
            tracing::warn!("home rankings response malformed");
            return Some(Err(AnalyticsLoadError::Malformed));
        }
    };
    Some(Ok(response))
}

fn record_home_analytics_load(
    params: &mut HashMap<String, String>,
    outcome: Result<AnalyticsResponse, AnalyticsLoadError>,
) {
    params.remove(HOME_ANALYTICS_DATA_PARAM);
    let state = match outcome {
        Ok(response) => {
            let state = if response.data.is_empty() {
                "empty"
            } else {
                "ready"
            };
            params.insert(
                HOME_ANALYTICS_DATA_PARAM.to_string(),
                serde_json::to_string(&response)
                    .expect("validated home analytics response is serializable"),
            );
            state
        }
        Err(_) => "unavailable",
    };
    params.insert(HOME_ANALYTICS_STATE_PARAM.to_string(), state.to_string());
}

fn account_payment_history_path(owner: &str) -> Option<String> {
    let reserved = owner.starts_with("force-")
        || matches!(
            owner,
            "health"
                | "pay"
                | "admin"
                | "intents"
                | "escrows"
                | "links"
                | "history"
                | "webhooks"
                | "on-chain"
                | "sync"
                | "confirm"
                | "cancel"
                | "release"
                | "refund"
                | "dispute"
                | "resolve"
                | "confirm-deposit"
                | "redeem"
                | "force-cancel"
                | "force-release"
                | "force-refund"
        );
    let safe = !reserved
        && !owner.is_empty()
        && owner.len() <= 128
        && owner != "."
        && owner != ".."
        && owner
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'));
    safe.then(|| {
        format!("/api/v1/pay/history/{owner}?limit={ACCOUNT_PAYMENT_HISTORY_LIMIT}&offset=0")
    })
}

/// Persist only a semantically validated owner-history payload. Dependency,
/// contract, and genuine-empty outcomes remain distinct so the account page
/// never turns an upstream failure into "No payment history yet."
fn record_account_payment_history_load(
    params: &mut HashMap<String, String>,
    expected_owner: &str,
    result: Result<serde_json::Value, AccountPaymentHistoryLoadError>,
) {
    params.remove(ACCOUNT_PAYMENT_HISTORY_DATA_PARAM);
    let state = match result {
        Ok(value) => match decode_pay_history(value, expected_owner, ACCOUNT_PAYMENT_HISTORY_LIMIT)
        {
            Some(history) => {
                let empty = history.intents.is_empty()
                    && history.escrows.is_empty()
                    && history.total_intents == 0
                    && history.total_escrows == 0;
                params.insert(
                    ACCOUNT_PAYMENT_HISTORY_DATA_PARAM.to_string(),
                    serde_json::to_string(&history)
                        .expect("the validated owner payment history is serializable"),
                );
                if empty {
                    ACCOUNT_PAYMENT_HISTORY_EMPTY
                } else {
                    ACCOUNT_PAYMENT_HISTORY_READY
                }
            }
            None => ACCOUNT_PAYMENT_HISTORY_MALFORMED,
        },
        Err(AccountPaymentHistoryLoadError::Unavailable) => ACCOUNT_PAYMENT_HISTORY_UNAVAILABLE,
        Err(AccountPaymentHistoryLoadError::Malformed) => ACCOUNT_PAYMENT_HISTORY_MALFORMED,
    };
    params.insert(
        ACCOUNT_PAYMENT_HISTORY_STATE_PARAM.to_string(),
        state.to_string(),
    );
}

fn news_detail_route_slug(path: &str) -> Option<&str> {
    let slug = news_detail_route_segment(path)?;
    crate::api::valid_news_slug(slug).then_some(slug)
}

fn news_detail_route_segment(path: &str) -> Option<&str> {
    let slug = path.strip_prefix("/news/")?;
    (!slug.is_empty() && !slug.contains('/')).then_some(slug)
}

fn page_metadata(meta: &PageMeta) -> (String, String) {
    (meta.title.clone(), meta.description.clone())
}

/// Wave 22 T4 — `/pricing` is an alias for `/plans` in prod. The
/// Vercel middleware `rewrites` `/pricing` → `/plans` while
/// preserving the query string. We mirror the same behavior as a
/// `307 Temporary Redirect` so the browser follows it (and the dev
/// baseline matches prod for both `/pricing` and `/pricing?ref=foo`
/// style URLs). The redirect fires BEFORE page rendering so the
/// downstream page code never has to handle the `/pricing` path.
fn pricing_redirect_response(query: &str) -> Response {
    let location = if query.is_empty() {
        "/plans".to_string()
    } else {
        format!("/plans?{query}")
    };
    (
        StatusCode::TEMPORARY_REDIRECT,
        [("location", location.as_str())],
        "",
    )
        .into_response()
}

/// The source capture harness uses `?__design_bypass=1` to expose the
/// authenticated shell without creating a real session. Keep that local-only
/// affordance available for visual checks while never honoring it in the
/// production cookie environment.
fn design_bypass_requested(query: &str, environment: epsx_bff::cookies::CookieEnvironment) -> bool {
    if environment != epsx_bff::cookies::CookieEnvironment::Local {
        return false;
    }

    url::form_urlencoded::parse(query.as_bytes()).any(|(key, value)| {
        key == "__design_bypass"
            && matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
    })
}

fn design_bypass_identity_enabled(enabled: bool, path: &str) -> bool {
    enabled && !matches!(path, "/" | "/index" | "/dashboard")
}

fn design_bypass_wallet_enabled(enabled: bool, path: &str) -> bool {
    enabled && path != "/dashboard"
}

fn design_bypass_chat_enabled(enabled: bool, path: &str) -> bool {
    enabled && path == "/dashboard"
}

/// All non-API requests land here. We render the page via Dioxus fullstack
/// SSR and return a complete HTML document using the same design-system
/// `<head>` the Next.js frontend emits.
pub async fn ssr_handler(State(state): State<AppState>, request: Request) -> Response {
    let (parts, _body) = request.into_parts();
    let path = parts.uri.path().to_string();
    let query = parts.uri.query().unwrap_or("").to_string();
    let headers = parts.headers.clone();
    let design_bypass = design_bypass_requested(&query, state.cookie_environment);
    // The source capture intentionally keeps `/dashboard` in its signed-out
    // shell even though the matrix appends the bypass query; the query there
    // exists only to expose the floating support affordance.
    // The two supplied homepage references intentionally cover both sides of
    // the auth boundary: `/` shows a connected wallet that still needs SIWE,
    // while private-style captures use the UI-only identity fixture. Keep the
    // homepage wallet-only so its header renders the wallet pill instead of
    // bell/profile/sign-out controls. The pill remains the unobtrusive route
    // back to SIWE when the server session is absent.
    let design_bypass_identity = design_bypass_identity_enabled(design_bypass, &path);
    let design_bypass_wallet = design_bypass_wallet_enabled(design_bypass, &path);
    let preference_flash_state = account_notification_preferences_flash_state(&headers, &query);

    let offline_shell = path == "/offline";
    let mut wallet = if offline_shell {
        ConnectedWalletState::default()
    } else {
        ConnectedWalletState::from_cookies(&headers)
    };
    let access_verification = if offline_shell {
        AccessVerification::MissingOrRejected
    } else {
        auth::access_verification(&headers, state.verifier.as_ref(), state.cookie_environment).await
    };
    // Local visual-test fixture only: it supplies authenticated shell state
    // without a bearer token, so no synthetic identity reaches an upstream
    // data service.
    let dev_bypass_user = (!offline_shell)
        .then(|| auth::dev_bypass_ui_user(Some(56)))
        .flatten();
    let design_bypass_user = (!offline_shell)
        .then(|| auth::design_bypass_ui_user(design_bypass_identity, Some(56)))
        .flatten();
    let refresh_cookie_present = auth::refresh_token(&headers, state.cookie_environment).is_some();
    let recover_session = access_verification.permits_refresh_recovery()
        && refresh_cookie_present
        && path != "/offline";
    let auth_page_session_state =
        auth_page_session_state(&path, &access_verification, refresh_cookie_present);
    let auth_page_verifier_unavailable =
        auth_page_session_state == Some(AUTH_PAGE_SESSION_STATE_VERIFIER_UNAVAILABLE);
    let (verified_access_token, user) = match access_verification {
        AccessVerification::Verified { token, user } => {
            (Some(token), Some(auth::ui_user(user, wallet.chain_id)))
        }
        AccessVerification::MissingOrRejected | AccessVerification::VerifierUnavailable => {
            (None, design_bypass_user.or(dev_bypass_user))
        }
    };

    // Wave 22 T4 — `/pricing` is an alias for `/plans` in prod
    // (Vercel middleware rewrite). We 307-redirect to `/plans`
    // preserving the query string, so both `/pricing` and
    // `/pricing?ref=foo` style URLs land on the plans page.
    if path == "/pricing" {
        return pricing_redirect_response(&query);
    }

    let route_is_known = is_known_frontend_route(&path);

    if path == "/auth" && user.is_some() {
        return private_session_redirect(safe_return_url(&query));
    }

    // Wave 22 T5 — mirror prod Vercel middleware 307 redirect behavior
    // for paths that prod always bounces to /auth when the user has
    // no session. The redirect fires BEFORE page rendering, so the
    // browser follows to /auth and the dev baseline matches the
    // prod baseline PNG (the auth page) for these routes.
    //
    // Two categories of path:
    //  - Exact-match paths in UNAUTH_REDIRECT_PATHS (e.g. /permissions,
    //    /notifications, /profile)
    //  - /chat/* sub-paths (e.g. /chat/<conv-id>, /chat/history) — prod
    //    lists `/chat` as public, but sub-paths are protected and 307
    //    to /auth. /chat itself stays public (browsable).
    let needs_unauth_redirect = route_is_known
        && user.is_none()
        && (UNAUTH_REDIRECT_PATHS.contains(&path.as_str())
            || (path.starts_with("/chat/") && !path.is_empty() && path != "/chat"));
    if needs_unauth_redirect {
        let next = if query.is_empty() {
            path.clone()
        } else {
            format!("{path}?{query}")
        };
        // Wave 23 T3 — prod Vercel middleware uses `?return_url=` (NOT
        // `?next=`). Standardize the dev SSR redirect to match. The
        // auth page's hydration script reads `return_url` from the
        // query string (or the `epsx_return_url` cookie set here)
        // and bounces the user back to that path after sign-in
        // completes. Pre-encoding via `urlencode` matches the shape
        // `apps-old/frontend/middleware.ts::handleUnauthenticated`
        // sets in the `epsx.return_url` cookie.
        //
        let location = format!("/auth?return_url={}", urlencode(&next));
        return private_session_redirect(location);
    }

    // Parse dynamic-route params from path
    let mut params = HashMap::new();
    if let Some(session_state) = auth_page_session_state {
        params.insert(
            AUTH_PAGE_SESSION_STATE_PARAM.to_string(),
            session_state.to_string(),
        );
    }
    if let Some(slug) = news_detail_route_slug(&path) {
        params.insert("slug".into(), slug.to_string());
    }
    if let Some(rest) = path.strip_prefix("/chat/") {
        if !rest.is_empty() && !rest.contains('/') {
            params.insert("id".into(), rest.to_string());
        }
    }
    if let Some(rest) = path.strip_prefix("/payment/") {
        let mut it = rest.splitn(2, '/');
        let ptype = it.next().unwrap_or("").to_string();
        let pid = it.next().unwrap_or("").to_string();
        if !ptype.is_empty() && !pid.is_empty() && !pid.contains('/') {
            params.insert("type".into(), ptype);
            params.insert("id".into(), pid);
        }
    }
    if path == "/account" {
        record_account_notification_preferences_form_state(&mut params, preference_flash_state);
    }

    // Page-specific server-side data fetching. Each block reads from
    // the gateway via `state.*` and adds the result to `params` so the
    // page can consume it.
    if route_is_known {
        fetch_page_data(
            &state,
            &path,
            &query,
            &user,
            &mut params,
            &headers,
            verified_access_token.as_deref(),
        )
        .await;
    }

    // Wave 3a Track B — plumb server-side wallet state into the page
    // context. We delegate the cookie read to
    // `ConnectedWalletState::from_cookies` (the shared parser reads the
    // browser-connected wallet cookie). `is_authenticated`
    // is sourced from the resolved `user` (the SIWE session lifetime),
    // NOT from the cookie (which tracks wallet-connection lifetime).
    //
    // The wallet address is also retained for the SSR navigation shell so a
    // connected browser gets a truthful wallet pill. We deliberately avoid a
    // second full-width sign-in recommendation: the pill itself remains the
    // route to `/auth` whenever the verified server session is absent.
    if wallet.address.is_none() {
        if let Some(design_wallet) = auth::design_bypass_wallet_state(design_bypass_wallet) {
            wallet = design_wallet;
        } else if let Some(dev_wallet) = auth::dev_bypass_wallet_state() {
            wallet = dev_wallet;
        }
    }
    let wallet_address = wallet.address.clone();
    let is_authenticated = user.is_some();
    let navigation_wallet_address =
        authoritative_navigation_wallet(&user, wallet_address.as_deref());
    let user_id = user
        .as_ref()
        .map(|value| value.id.clone())
        .unwrap_or_default();
    wallet.is_authenticated = is_authenticated;

    let ctx = PageContext {
        user,
        path: path.clone(),
        query: query.clone(),
        params,
        api_url: state.api_url.clone(),
        demo_login_enabled: state.demo_login_enabled,
        wallet,
    };

    let (meta, body_element) = render_page(&ctx, false);
    let status = notifications_ssr_status(&path, &ctx.params)
        .or_else(|| news_ssr_status(&path, &ctx.params))
        .unwrap_or(match meta.status {
            PageStatus::Ok => StatusCode::OK,
            PageStatus::NotFound => StatusCode::NOT_FOUND,
        });
    let body_html = dioxus_ssr::render_element(body_element);

    // === Wave 49+ — SSR-safe navbar ===
    //
    // The Dioxus `<NavigationClient>` / `<GroupDropdown>` chain uses
    // Dioxus `onclick:` closures to toggle the desktop dropdowns.
    // Dioxus 0.7 SSR is hydration-less — those closures are stripped
    // from the rendered HTML, so clicking "Market / Developer /
    // Company" does nothing and the user can never reach the
    // Rankings / Portfolio / Developer Portal / About / News /
    // Contact / Support pages from the navbar.
    //
    // The fix: use `epsx_templates::epsx_header()` (the static HTML
    // sticky header that mirrors the production NavMenu). It emits
    // progressive `data-epsx-action` attributes handled by the generated
    // Rust/WASM runtime. Dropdown items remain in the DOM while CSS and the
    // runtime coordinate their open state, so every link stays reachable.
    //
    // The auth page hides the navbar via the `path == "/auth"`
    // short-circuit (the dedicated `<AuthLayout>` is full-bleed).
    let nav_html = frontend_navigation_html(
        &path,
        &query,
        is_authenticated,
        navigation_wallet_address.as_deref(),
    );

    // Source-compatible shell: the development root has no global footer.
    // Page bodies remain responsible for any route-specific footer content.
    let include_footer = false;

    let (metadata_title, metadata_description) = page_metadata(&meta);
    let doc = epsx_templates::page_shell_with_body_class_and_keywords(
        &metadata_title,
        &metadata_description,
        meta.keywords.as_deref(),
        &nav_html,
        &body_html,
        include_footer,
        meta.body_class.as_deref().unwrap_or(""),
    );

    let recovery_runtime = if recover_session {
        r#"<span hidden data-epsx-session-recovery="true" data-epsx-action="session-recover"></span>"#.to_string()
    } else {
        String::new()
    };
    // The development frontend mounts the floating support affordance from
    // the global layout for authenticated pages. Keep the same shell-level
    // placement in the SSR document; `/chat` and its route descendants own
    // their full-page conversation UI and therefore hide the floating trigger.
    let design_bypass_chat = design_bypass_chat_enabled(design_bypass, &path);
    let owns_chat_surface = path == "/chat" || path.starts_with("/chat/");
    let chat_widget_html =
        if (is_authenticated || design_bypass_chat) && path != "/auth" && !owns_chat_surface {
            crate::widgets::chat_widget(true, &user_id)
        } else {
            String::new()
        };
    let doc = doc.replace(
        "</body>",
        &format!("{recovery_runtime}{chat_widget_html}</body>"),
    );

    let mut response =
        (status, [("content-type", "text/html; charset=utf-8")], doc).into_response();
    apply_ssr_cache_policy(
        &mut response,
        is_authenticated,
        recover_session,
        auth_page_verifier_unavailable,
        &path,
    );
    if preference_flash_state.is_some() {
        let cookie = format!(
            "{}=; Path=/account; Max-Age=0; HttpOnly; SameSite=Lax",
            crate::api::NOTIFICATION_PREFERENCES_FLASH_COOKIE
        );
        response.headers_mut().append(
            header::SET_COOKIE,
            HeaderValue::from_str(&cookie).expect("flash cookie clear value is valid"),
        );
    }
    response
}

fn private_session_redirect(location: String) -> Response {
    let mut response =
        (StatusCode::TEMPORARY_REDIRECT, [("location", location)], "").into_response();
    apply_ssr_cache_policy(&mut response, true, false, false, "/auth");
    response
}

/// Keep owner-specific SSR output out of browser and intermediary caches.
/// `/offline` is the one reviewed public exception and never receives the
/// authenticated notification runtime, even when the request carried a valid
/// session.
fn apply_ssr_cache_policy(
    response: &mut Response,
    is_authenticated: bool,
    recover_session: bool,
    auth_page_verifier_unavailable: bool,
    path: &str,
) {
    if path == "/offline" {
        // This marker is the service worker's fail-closed permission to cache
        // the response. The worker fetches it with credentials omitted and
        // refuses any response that loses this exact public-shell contract.
        response.headers_mut().insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=0, must-revalidate"),
        );
        response.headers_mut().insert(
            "x-epsx-public-cache",
            HeaderValue::from_static("offline-shell-v1"),
        );
    } else if is_authenticated
        || recover_session
        || auth_page_verifier_unavailable
        || path == "/developer"
        || path.starts_with("/developer/")
    {
        response.headers_mut().insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("private, no-store"),
        );
        response.headers_mut().insert(
            header::VARY,
            HeaderValue::from_static("Cookie, Authorization"),
        );
    }
}

/// Fetch page-specific data and add it to `params` as JSON-serialized
/// values. The page reads them via `ctx.params.get("data_X")` and
/// deserializes into a typed struct.
async fn fetch_page_data(
    state: &AppState,
    path: &str,
    query: &str,
    user: &Option<User>,
    params: &mut HashMap<String, String>,
    headers: &axum::http::HeaderMap,
    verified_access_token: Option<&str>,
) {
    use epsx_client::RequestContext;
    let mut request_context = RequestContext::from_headers(headers);
    // Browser cookies are not authorization headers. After local RS256/JWKS
    // verification, explicitly forward the canonical token as a bearer value.
    request_context.auth_token = verified_access_token.map(str::to_owned);
    // SSR data is represented as an explicit outcome. A dependency failure is
    // rendered as a failure and never replaced with production-looking sample
    // data. `/dashboard` intentionally has no loader until an owner-scoped
    // producer contract exists.
    // `/` and `/index` load public news and the fixed public ranking preview
    // independently. The ranking call intentionally has no request context or
    // credentials, so its backend-owned public offset remains deterministic.
    let (home_news, home_analytics) = tokio::join!(
        load_home_news(state.content.as_ref(), path),
        load_home_analytics(state.analytics.as_ref(), path)
    );
    if let Some(outcome) = home_news {
        record_home_news_load(params, outcome);
    }
    if let Some(outcome) = home_analytics {
        record_home_analytics_load(params, outcome);
    }
    // /news: load the content dependency through the same strict adapter used
    // by the JSON BFF route. The outcome keeps empty distinct from unavailable
    // or malformed and carries the URL-stable q/category/page selection.
    if path == "/news" {
        let outcome = match crate::api::NewsQuery::from_raw_query(query) {
            Ok(query) => crate::api::load_news_list(state.content.as_ref(), &query).await,
            Err(()) => crate::api::NewsListLoadOutcome::Error {
                code: "invalid_news_query".to_string(),
            },
        };
        params.insert(
            "data_news".into(),
            serde_json::to_string(&outcome).expect("news list outcome is serializable"),
        );
    }
    // /plans: render only the backend-owned public plan projection. The
    // frontend validates the transport shape but does not derive prices,
    // promotions, visibility, permissions, or eligibility.
    if path == "/plans" {
        let outcome = crate::api::load_public_plans(state.content.as_ref()).await;
        params.insert(
            epsx_dioxus_ui::pages::plans::PLANS_DATA_PARAM.to_string(),
            serde_json::to_string(&outcome).expect("public plans outcome is serializable"),
        );
    }
    // /analytics: rankings and public filter options are independent and load
    // concurrently. Authenticated sessions also load the owner-scoped
    // watchlist through the same locally verified bearer.
    if path == "/analytics" {
        let normalized_query = analytics_query(query);
        if let Ok(query) = &normalized_query {
            record_analytics_query(params, query);
        } else {
            params.remove(ANALYTICS_QUERY_PARAM);
        }
        let rankings = async {
            match normalized_query.as_deref() {
                Ok(query) => {
                    load_analytics_rankings(
                        state.analytics.as_ref(),
                        query,
                        headers,
                        verified_access_token,
                    )
                    .await
                }
                Err(()) => Err(AnalyticsLoadError::Malformed),
            }
        };
        let filters = load_analytics_filters(state.analytics.as_ref());
        let watchlist = load_analytics_watchlist(state.wallet.as_ref(), verified_access_token);
        let (rankings, filters, watchlist) = tokio::join!(rankings, filters, watchlist);
        record_analytics_load(params, rankings);
        record_analytics_filters_load(params, filters);
        record_analytics_watchlist_load(params, watchlist);
    }
    // /portfolio consumes the same strict owner-scoped watchlist contract as
    // Analytics. The token is forwarded only after local session verification;
    // a connected-wallet cookie never selects or authorizes portfolio data.
    if path == "/portfolio" {
        let layout =
            load_portfolio_watchlist_layout(state.wallet.as_ref(), verified_access_token).await;
        record_portfolio_watchlist_load(params, layout);
    }
    // /news/[slug]: the content service owns slug resolution. Unknown records
    // remain not-found, while dependency/malformed responses remain errors.
    if let Some(slug) = news_detail_route_slug(path) {
        let outcome = crate::api::load_news_post(state.content.as_ref(), slug).await;
        params.insert(
            "data_news_post".into(),
            serde_json::to_string(&outcome).expect("news detail outcome is serializable"),
        );
    }
    if matches!(path, "/developer" | "/developer/usage") {
        let (data_param, state_param) = if path == "/developer/usage" {
            (DEVELOPER_USAGE_DATA_PARAM, DEVELOPER_USAGE_STATE_PARAM)
        } else {
            (DEVELOPER_DATA_PARAM, DEVELOPER_STATE_PARAM)
        };
        let days = developer_usage_days(query).unwrap_or(-1);
        let outcome = match (verified_access_token, days) {
            (Some(bearer), 7 | 30 | 90) => {
                crate::api::load_developer_overview_for_ssr(state.wallet.as_ref(), bearer, days)
                    .await
            }
            (None, _) => Err(crate::api::DeveloperLoadError::Forbidden),
            _ => Err(crate::api::DeveloperLoadError::Malformed),
        };
        match outcome {
            Ok(data) => {
                let empty = data.api_keys.is_empty() && data.usage.total_requests == 0;
                params.insert(
                    data_param.to_string(),
                    serde_json::to_string(&data)
                        .expect("validated developer overview is serializable"),
                );
                params.insert(
                    state_param.to_string(),
                    if empty { LOAD_EMPTY } else { LOAD_READY }.to_string(),
                );
            }
            Err(crate::api::DeveloperLoadError::Forbidden) => {
                params.insert(state_param.to_string(), LOAD_FORBIDDEN.to_string());
            }
            Err(crate::api::DeveloperLoadError::Unavailable) => {
                params.insert(state_param.to_string(), LOAD_UNAVAILABLE.to_string());
            }
            Err(crate::api::DeveloperLoadError::Malformed) => {
                params.insert(state_param.to_string(), LOAD_MALFORMED.to_string());
            }
        }
    }
    if matches!(path, "/chat" | "/chat/history") || chat_route_id(path).is_some() {
        load_chat_page_data(state, path, query, user, params, verified_access_token).await;
    }
    // /notifications: fetch the authenticated owner's list. Preserve an
    // explicit dependency outcome so an upstream failure never renders as an
    // empty or sample-backed success state.
    if path == "/notifications" {
        if let (Some(owner), Some(bearer)) = (
            user.as_ref().map(|user| user.address.as_str()),
            verified_access_token,
        ) {
            match load_notification_page(state.notification.as_ref(), bearer, owner, headers, query)
                .await
            {
                Ok((page, status, notification_type, priority, start_date, end_date, outcome)) => {
                    record_notification_load(
                        params,
                        NotificationLoadSelection {
                            page,
                            status: status.as_deref(),
                            notification_type: notification_type.as_deref(),
                            priority: priority.as_deref(),
                            start_date: start_date.as_deref(),
                            end_date: end_date.as_deref(),
                        },
                        outcome,
                    )
                }
                Err(()) => record_invalid_notification_query(params),
            }
        }
    }
    // `/plans` intentionally has no loader. Pricing, eligibility, features,
    // sale windows, and subscription decisions remain unavailable until a
    // subscription-owned public catalog contract is frozen end to end.
    // `/account` renders identity details only from the locally verified
    // session. It deliberately performs no ambiguous profile/credit read;
    // owner payment history and notification preferences each use their own
    // strict owner-scoped read outcome below.
    if path == "/account" {
        // Source parity starts with the canonical first owner-history page.
        // The path owner is derived only from the locally verified session;
        // URL query, connected-wallet cookies, and account payloads never
        // select financial records. Pay repeats the owner comparison.
        if let Some(owner) = user.as_ref().map(|user| user.address.as_str()) {
            let result = match (verified_access_token, account_payment_history_path(owner)) {
                (Some(_), Some(path)) => state
                    .payment
                    .get_with_ctx(&path, &request_context)
                    .await
                    .map_err(|_| {
                        tracing::warn!("account owner payment-history dependency unavailable");
                        AccountPaymentHistoryLoadError::Unavailable
                    }),
                _ => Err(AccountPaymentHistoryLoadError::Malformed),
            };
            record_account_payment_history_load(params, owner, result);
        }
        if let Some(bearer) = verified_access_token {
            let request_id = crate::api::notification_request_id(headers);
            let outcome = crate::api::load_notification_preferences(
                state.notification.as_ref(),
                bearer,
                &request_id,
            )
            .await;
            record_account_notification_preferences_load(params, outcome);
        } else {
            params.insert(
                ACCOUNT_NOTIFICATION_PREFERENCES_STATE_PARAM.to_string(),
                ACCOUNT_NOTIFICATION_PREFERENCES_MALFORMED.to_string(),
            );
        }
    }
    // `/account/credits` intentionally has no loader. A6 has not selected a
    // credit-ledger authority, so failure must not become a zero balance.
    if path == "/developer/docs" {
        match crate::api::load_developer_openapi_for_ssr(state.wallet.as_ref()).await {
            Ok(spec) => {
                params.insert(
                    DEVELOPER_OPENAPI_DATA_PARAM.to_string(),
                    serde_json::to_string(&spec)
                        .expect("validated developer OpenAPI is serializable"),
                );
                params.insert(
                    DEVELOPER_OPENAPI_STATE_PARAM.to_string(),
                    LOAD_READY.to_string(),
                );
            }
            Err(crate::api::DeveloperLoadError::Malformed) => {
                params.insert(
                    DEVELOPER_OPENAPI_STATE_PARAM.to_string(),
                    LOAD_MALFORMED.to_string(),
                );
            }
            Err(crate::api::DeveloperLoadError::Forbidden)
            | Err(crate::api::DeveloperLoadError::Unavailable) => {
                params.insert(
                    DEVELOPER_OPENAPI_STATE_PARAM.to_string(),
                    LOAD_UNAVAILABLE.to_string(),
                );
            }
        }
    }
    // Dynamic payment pages intentionally perform no intent lookup until A6
    // provides an owner-safe intent and finality contract.
}

async fn load_chat_page_data(
    state: &AppState,
    path: &str,
    query: &str,
    user: &Option<User>,
    params: &mut HashMap<String, String>,
    verified_access_token: Option<&str>,
) {
    let Some(owner) = user.as_ref().map(|user| user.address.as_str()) else {
        params.insert(CHAT_INBOX_STATE_PARAM.into(), CHAT_FORBIDDEN.into());
        params.insert(CHAT_DETAIL_STATE_PARAM.into(), CHAT_FORBIDDEN.into());
        return;
    };
    let Some(bearer) = verified_access_token else {
        params.insert(CHAT_INBOX_STATE_PARAM.into(), CHAT_FORBIDDEN.into());
        params.insert(CHAT_DETAIL_STATE_PARAM.into(), CHAT_FORBIDDEN.into());
        return;
    };

    if matches!(path, "/chat" | "/chat/history") {
        let inbox =
            crate::chat_adapter::load_chat_inbox_for_ssr(state.wallet.as_ref(), bearer, owner)
                .await;
        let first_id = inbox
            .as_ref()
            .ok()
            .and_then(|inbox| inbox.conversations.first())
            .and_then(|conversation| uuid::Uuid::parse_str(&conversation.id).ok());
        record_chat_inbox(params, inbox);

        if path == "/chat" && !chat_new_requested(query) {
            if let Some(id) = first_id {
                let detail = crate::chat_adapter::load_chat_detail_for_ssr(
                    state.wallet.as_ref(),
                    bearer,
                    owner,
                    id,
                )
                .await;
                record_chat_detail(params, detail);
            }
        }
    } else if let Some(id) = chat_route_id(path) {
        let detail =
            crate::chat_adapter::load_chat_detail_for_ssr(state.wallet.as_ref(), bearer, owner, id)
                .await;
        record_chat_detail(params, detail);
    }
}

fn record_chat_inbox(
    params: &mut HashMap<String, String>,
    result: Result<epsx_dioxus_ui::pages::chat::ChatInboxData, crate::chat_adapter::ChatLoadError>,
) {
    params.remove(CHAT_INBOX_DATA_PARAM);
    let state = match result {
        Ok(inbox) => {
            let state = if inbox.conversations.is_empty() {
                CHAT_EMPTY
            } else {
                CHAT_READY
            };
            params.insert(
                CHAT_INBOX_DATA_PARAM.into(),
                serde_json::to_string(&inbox).expect("validated chat inbox is serializable"),
            );
            state
        }
        Err(crate::chat_adapter::ChatLoadError::Forbidden) => CHAT_FORBIDDEN,
        Err(crate::chat_adapter::ChatLoadError::Unavailable) => CHAT_UNAVAILABLE,
        Err(crate::chat_adapter::ChatLoadError::Malformed) => CHAT_MALFORMED,
    };
    params.insert(CHAT_INBOX_STATE_PARAM.into(), state.into());
}

fn record_chat_detail(
    params: &mut HashMap<String, String>,
    result: Result<epsx_dioxus_ui::pages::chat::ChatDetailData, crate::chat_adapter::ChatLoadError>,
) {
    params.remove(CHAT_DETAIL_DATA_PARAM);
    let state = match result {
        Ok(detail) => {
            params.insert(
                CHAT_DETAIL_DATA_PARAM.into(),
                serde_json::to_string(&detail).expect("validated chat detail is serializable"),
            );
            CHAT_READY
        }
        Err(crate::chat_adapter::ChatLoadError::Forbidden) => CHAT_FORBIDDEN,
        Err(crate::chat_adapter::ChatLoadError::Unavailable) => CHAT_UNAVAILABLE,
        Err(crate::chat_adapter::ChatLoadError::Malformed) => CHAT_MALFORMED,
    };
    params.insert(CHAT_DETAIL_STATE_PARAM.into(), state.into());
}

fn chat_route_id(path: &str) -> Option<uuid::Uuid> {
    let value = path.strip_prefix("/chat/")?;
    if value == "history" || value.is_empty() || value.contains('/') {
        return None;
    }
    uuid::Uuid::parse_str(value).ok()
}

fn chat_new_requested(query: &str) -> bool {
    url::form_urlencoded::parse(query.as_bytes()).any(|(key, value)| key == "new" && value == "1")
}

fn developer_usage_days(raw_query: &str) -> Result<i32, ()> {
    if raw_query.is_empty() {
        return Ok(30);
    }
    let url =
        reqwest::Url::parse(&format!("https://frontend.invalid/?{raw_query}")).map_err(|_| ())?;
    let mut days = None;
    for (key, value) in url.query_pairs() {
        if key != "days" {
            continue;
        }
        if days.is_some() {
            return Err(());
        }
        days = Some(value.parse::<i32>().map_err(|_| ())?);
    }
    let days = days.unwrap_or(30);
    matches!(days, 7 | 30 | 90).then_some(days).ok_or(())
}

fn analytics_query(raw_query: &str) -> Result<String, ()> {
    if raw_query.is_empty() {
        return Ok(String::new());
    }
    let url =
        reqwest::Url::parse(&format!("https://frontend.invalid/?{raw_query}")).map_err(|_| ())?;
    let mut seen = std::collections::HashSet::new();
    let mut normalized = url::form_urlencoded::Serializer::new(String::new());
    for (key, value) in url.query_pairs() {
        let key = key.as_ref();
        if !matches!(
            key,
            "page" | "limit" | "country" | "sector" | "sort_by" | "min_eps" | "min_growth"
        ) {
            // Design/runtime flags and unrelated query fields must not select
            // or constrain the backend rankings request.
            continue;
        }
        if !seen.insert(key.to_string()) {
            return Err(());
        }
        match key {
            "page" => {
                let value = value.parse::<u32>().map_err(|_| ())?;
                if value == 0 || value > 1_000_000 {
                    return Err(());
                }
                normalized.append_pair(key, &value.to_string());
            }
            "limit" => {
                let value = value.parse::<u32>().map_err(|_| ())?;
                if value == 0 || value > 100 {
                    return Err(());
                }
                normalized.append_pair(key, &value.to_string());
            }
            "country" | "sector" => {
                if value.is_empty() {
                    continue;
                }
                if value.len() > 64 || value.chars().any(|character| character.is_control()) {
                    return Err(());
                }
                normalized.append_pair(key, &value);
            }
            "sort_by" => {
                if value.is_empty()
                    || value.len() > 64
                    || value.chars().any(|character| character.is_control())
                {
                    return Err(());
                }
                normalized.append_pair(key, &value);
            }
            "min_eps" | "min_growth" => {
                let number = value.parse::<f64>().map_err(|_| ())?;
                if !number.is_finite() {
                    return Err(());
                }
                normalized.append_pair(key, &value);
            }
            _ => unreachable!(),
        }
    }
    Ok(normalized.finish())
}

async fn load_analytics_rankings(
    client: &epsx_client::ServiceClient,
    normalized_query: &str,
    headers: &axum::http::HeaderMap,
    verified_access_token: Option<&str>,
) -> Result<AnalyticsResponse, AnalyticsLoadError> {
    let path = if normalized_query.is_empty() {
        ANALYTICS_RANKINGS_PATH.to_string()
    } else {
        format!("{ANALYTICS_RANKINGS_PATH}?{normalized_query}")
    };
    let mut request_context = epsx_client::RequestContext::from_headers(headers);
    request_context.auth_token = verified_access_token.map(str::to_owned);
    let value = client
        .get_with_ctx(&path, &request_context)
        .await
        .map_err(|error| {
            tracing::warn!("analytics rankings dependency unavailable: {error}");
            AnalyticsLoadError::Unavailable
        })?;
    serde_json::from_value::<AnalyticsResponse>(value)
        .map_err(|error| {
            tracing::warn!("analytics rankings response malformed: {error}");
            AnalyticsLoadError::Malformed
        })?
        .validated()
        .map_err(|_| {
            tracing::warn!("analytics rankings response failed semantic validation");
            AnalyticsLoadError::Malformed
        })
}

async fn load_analytics_filters(
    client: &epsx_client::ServiceClient,
) -> Result<AnalyticsFilters, AnalyticsLoadError> {
    let value = client
        .get_plain(ANALYTICS_FILTERS_PATH)
        .await
        .map_err(|error| {
            tracing::warn!("analytics filters dependency unavailable: {error}");
            AnalyticsLoadError::Unavailable
        })?;
    serde_json::from_value::<AnalyticsFilters>(value)
        .map_err(|error| {
            tracing::warn!("analytics filters response malformed: {error}");
            AnalyticsLoadError::Malformed
        })?
        .validated()
        .map_err(|_| {
            tracing::warn!("analytics filters response failed semantic validation");
            AnalyticsLoadError::Malformed
        })
}

async fn load_analytics_watchlist(
    client: &epsx_client::ServiceClient,
    verified_access_token: Option<&str>,
) -> Result<Option<WatchlistData>, AnalyticsLoadError> {
    let Some(token) = verified_access_token else {
        return Ok(None);
    };
    let mut context = epsx_client::RequestContext::new();
    context.auth_token = Some(token.to_string());
    let value = client
        .get_with_ctx(ANALYTICS_WATCHLIST_PATH, &context)
        .await
        .map_err(|error| {
            tracing::warn!("analytics watchlist dependency unavailable: {error}");
            AnalyticsLoadError::Unavailable
        })?;
    crate::api::decode_watchlist_response(value)
        .map(Some)
        .map_err(|()| {
            tracing::warn!("analytics watchlist response malformed");
            AnalyticsLoadError::Malformed
        })
}

async fn load_portfolio_watchlist_layout(
    client: &epsx_client::ServiceClient,
    token: Option<&str>,
) -> Result<Option<WatchlistLayoutData>, AnalyticsLoadError> {
    let Some(token) = token else {
        return Ok(None);
    };
    let mut context = epsx_client::RequestContext::new();
    context.auth_token = Some(token.to_string());
    let value = client
        .get_with_ctx(PORTFOLIO_WATCHLIST_LAYOUT_PATH, &context)
        .await
        .map_err(|error| {
            tracing::warn!("portfolio watchlist layout dependency unavailable: {error}");
            AnalyticsLoadError::Unavailable
        })?;
    crate::api::decode_watchlist_layout_response(value)
        .map(Some)
        .map_err(|()| {
            tracing::warn!("portfolio watchlist layout response malformed");
            AnalyticsLoadError::Malformed
        })
}

fn record_analytics_query(params: &mut HashMap<String, String>, normalized_query: &str) {
    let query = AnalyticsQueryState::from_normalized_query(normalized_query)
        .expect("the bounded SSR query is valid analytics query state");
    params.insert(
        ANALYTICS_QUERY_PARAM.to_string(),
        serde_json::to_string(&query).expect("analytics query state is serializable"),
    );
}

fn record_analytics_load(
    params: &mut HashMap<String, String>,
    outcome: Result<AnalyticsResponse, AnalyticsLoadError>,
) {
    params.remove(ANALYTICS_DATA_PARAM);
    let state = match outcome {
        Ok(response) => {
            let state = if response.data.is_empty() {
                "empty"
            } else {
                "ready"
            };
            params.insert(
                ANALYTICS_DATA_PARAM.to_string(),
                serde_json::to_string(&response)
                    .expect("validated analytics response is serializable"),
            );
            state
        }
        Err(AnalyticsLoadError::Malformed) => "malformed",
        Err(AnalyticsLoadError::Unavailable) => "unavailable",
    };
    params.insert(ANALYTICS_STATE_PARAM.to_string(), state.to_string());
}

fn record_analytics_filters_load(
    params: &mut HashMap<String, String>,
    outcome: Result<AnalyticsFilters, AnalyticsLoadError>,
) {
    params.remove(ANALYTICS_FILTERS_DATA_PARAM);
    let state = match outcome {
        Ok(filters) => {
            params.insert(
                ANALYTICS_FILTERS_DATA_PARAM.to_string(),
                serde_json::to_string(&filters)
                    .expect("validated analytics filters are serializable"),
            );
            "ready"
        }
        Err(AnalyticsLoadError::Malformed) => "malformed",
        Err(AnalyticsLoadError::Unavailable) => "unavailable",
    };
    params.insert(ANALYTICS_FILTERS_STATE_PARAM.to_string(), state.to_string());
}

fn record_analytics_watchlist_load(
    params: &mut HashMap<String, String>,
    outcome: Result<Option<WatchlistData>, AnalyticsLoadError>,
) {
    params.remove(ANALYTICS_WATCHLIST_DATA_PARAM);
    let state = match outcome {
        Ok(Some(watchlist)) => {
            params.insert(
                ANALYTICS_WATCHLIST_DATA_PARAM.to_string(),
                serde_json::to_string(&watchlist)
                    .expect("validated analytics watchlist is serializable"),
            );
            "ready"
        }
        Ok(None) => "signed_out",
        Err(AnalyticsLoadError::Malformed) => "malformed",
        Err(AnalyticsLoadError::Unavailable) => "unavailable",
    };
    params.insert(
        ANALYTICS_WATCHLIST_STATE_PARAM.to_string(),
        state.to_string(),
    );
}

fn record_portfolio_watchlist_load(
    params: &mut HashMap<String, String>,
    outcome: Result<Option<WatchlistLayoutData>, AnalyticsLoadError>,
) {
    params.remove(PORTFOLIO_WATCHLIST_DATA_PARAM);
    let state = match outcome {
        Ok(Some(watchlist)) => {
            params.insert(
                PORTFOLIO_WATCHLIST_DATA_PARAM.to_string(),
                serde_json::to_string(&watchlist)
                    .expect("validated portfolio watchlist is serializable"),
            );
            "ready"
        }
        Ok(None) => "signed_out",
        Err(AnalyticsLoadError::Malformed) => "malformed",
        Err(AnalyticsLoadError::Unavailable) => "unavailable",
    };
    params.insert(
        PORTFOLIO_WATCHLIST_STATE_PARAM.to_string(),
        state.to_string(),
    );
}

fn news_ssr_status(path: &str, params: &HashMap<String, String>) -> Option<StatusCode> {
    let (key, is_list) = if path == "/news" {
        ("data_news", true)
    } else {
        let slug = news_detail_route_segment(path)?;
        if !crate::api::valid_news_slug(slug) {
            return Some(StatusCode::NOT_FOUND);
        }
        ("data_news_post", false)
    };
    let Some(raw) = params.get(key) else {
        return Some(StatusCode::SERVICE_UNAVAILABLE);
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(raw) else {
        return Some(StatusCode::SERVICE_UNAVAILABLE);
    };
    let state = value.get("state").and_then(serde_json::Value::as_str);
    let code = value.get("code").and_then(serde_json::Value::as_str);
    match (is_list, state, code) {
        (true, Some("ready" | "empty"), _) => Some(StatusCode::OK),
        (true, Some("error"), Some("invalid_news_query")) => Some(StatusCode::BAD_REQUEST),
        (false, Some("ready"), _) => Some(StatusCode::OK),
        (false, Some("not_found"), _) => Some(StatusCode::NOT_FOUND),
        _ => Some(StatusCode::SERVICE_UNAVAILABLE),
    }
}

fn notifications_ssr_status(path: &str, params: &HashMap<String, String>) -> Option<StatusCode> {
    if path != "/notifications" {
        return None;
    }
    Some(
        match params.get(NOTIFICATIONS_STATE_PARAM).map(String::as_str) {
            Some("ok") if params.contains_key(NOTIFICATIONS_DATA_PARAM) => StatusCode::OK,
            Some(NOTIFICATIONS_INVALID_QUERY) => StatusCode::BAD_REQUEST,
            Some("malformed") => StatusCode::BAD_GATEWAY,
            _ => StatusCode::SERVICE_UNAVAILABLE,
        },
    )
}

fn normalized_request_target(path: &str, query: &str) -> String {
    if query.is_empty() {
        path.to_string()
    } else {
        format!("{path}?{query}")
    }
}

/// Pick the identity displayed in the public navigation. An authenticated
/// session always wins over the provider cookie so a stale/disconnected
/// browser wallet cannot replace the backend-verified owner identity.
fn authoritative_navigation_wallet(
    user: &Option<User>,
    connected_wallet: Option<&str>,
) -> Option<String> {
    user.as_ref()
        .map(|user| user.address.trim())
        .filter(|address| !address.is_empty())
        .or_else(|| {
            connected_wallet
                .map(str::trim)
                .filter(|address| !address.is_empty())
        })
        .map(str::to_owned)
}

fn frontend_navigation_html(
    path: &str,
    query: &str,
    is_authenticated: bool,
    wallet_address: Option<&str>,
) -> String {
    if path == "/auth" {
        return String::new();
    }

    let return_target = normalized_request_target(path, query);
    // Production keeps the chain selector out of the global navigation. The
    // current network remains available inside wallet-owned flows where it is
    // actionable, rather than occupying the public header as a read-only tag.
    epsx_templates::epsx_header_for_session_and_wallet(
        is_authenticated,
        &return_target,
        wallet_address,
    )
}

fn urlencode(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

fn safe_return_url(query: &str) -> String {
    let Ok(request_url) = reqwest::Url::parse(&format!("https://frontend.invalid/?{query}")) else {
        return "/".to_string();
    };
    let Some(raw) = request_url
        .query_pairs()
        .find_map(|(key, value)| (key == "return_url").then(|| value.into_owned()))
    else {
        return "/".to_string();
    };
    if !raw.starts_with('/') || raw.starts_with("//") || raw.contains('\\') {
        return "/".to_string();
    }
    let Ok(target) = request_url.join(&raw) else {
        return "/".to_string();
    };
    if target.origin() != request_url.origin() || target.path() == "/auth" {
        return "/".to_string();
    }

    let mut value = target.path().to_string();
    if let Some(query) = target.query() {
        value.push('?');
        value.push_str(query);
    }
    if let Some(fragment) = target.fragment() {
        value.push('#');
        value.push_str(fragment);
    }
    value
}

#[cfg(test)]
mod tests {
    use super::account_notification_preferences_flash_state;
    use super::account_payment_history_path;
    use super::analytics_query;
    use super::apply_ssr_cache_policy;
    use super::auth_page_session_state;
    use super::authoritative_navigation_wallet;
    use super::design_bypass_chat_enabled;
    use super::design_bypass_identity_enabled;
    use super::design_bypass_requested;
    use super::design_bypass_wallet_enabled;
    use super::frontend_navigation_html;
    use super::load_home_analytics;
    use super::load_home_news;
    use super::news_detail_route_segment;
    use super::news_detail_route_slug;
    use super::news_ssr_status;
    use super::normalized_request_target;
    use super::notifications_ssr_status;
    use super::page_metadata;
    use super::pricing_redirect_response;
    use super::record_account_notification_preferences_form_state;
    use super::record_account_notification_preferences_load;
    use super::record_account_payment_history_load;
    use super::record_analytics_load;
    use super::record_home_analytics_load;
    use super::record_home_news_load;
    use super::record_notification_load;
    use super::record_portfolio_watchlist_load;
    use super::safe_return_url;
    use super::urlencode;
    use super::AnalyticsLoadError;
    use super::NotificationLoadSelection;
    use super::NotificationPageRequest;
    use crate::api::{NotificationPreferencesLoadError, NotificationPreferencesLoadOutcome};
    use axum::http::{header, HeaderMap, HeaderValue};
    use epsx_bff::session::AccessVerification;
    use epsx_dioxus_ui::auth::User;
    use epsx_dioxus_ui::pages::auth_page::{
        AUTH_PAGE_SESSION_STATE_RECOVERING, AUTH_PAGE_SESSION_STATE_SIGNED_OUT,
        AUTH_PAGE_SESSION_STATE_VERIFIER_UNAVAILABLE,
    };
    use epsx_dioxus_ui::pages::{PageContext, PageMeta, PageStatus};
    use std::collections::HashMap;

    #[test]
    fn analytics_query_keeps_only_bounded_backend_filters() {
        assert_eq!(
            analytics_query("page=2&limit=25&country=america&sector=Technology&__design_bypass=1")
                .unwrap(),
            "page=2&limit=25&country=america&sector=Technology"
        );
        assert_eq!(
            analytics_query("page=1&limit=10&country=america&sector=").unwrap(),
            "page=1&limit=10&country=america"
        );
        assert!(analytics_query("page=0").is_err());
        assert!(analytics_query("page=1&page=2").is_err());
        assert!(analytics_query("limit=101").is_err());
        assert!(analytics_query("min_growth=not-a-number").is_err());
    }

    #[test]
    fn analytics_load_records_ready_empty_and_unavailable_outcomes() {
        let response = epsx_dioxus_ui::pages::analytics::AnalyticsResponse {
            success: true,
            data: vec![],
            pagination: epsx_dioxus_ui::pages::analytics::AnalyticsPagination {
                page: 1,
                limit: 10,
                total: 0,
                total_pages: 0,
                has_next: false,
                has_prev: false,
            },
            metadata: epsx_dioxus_ui::pages::analytics::AnalyticsMetadata {
                available_countries: vec!["america".into()],
                available_sectors: vec!["Technology".into()],
                request_timestamp: "2026-07-27T00:00:00Z".into(),
                data_source: "live".into(),
            },
            access_info: Some(epsx_dioxus_ui::pages::analytics::AnalyticsAccessInfo {
                min_accessible_rank: 100,
                locked_ranks_count: 99,
            }),
            message: None,
            processing_time_ms: 1,
        };
        let mut params = HashMap::new();
        record_analytics_load(&mut params, Ok(response));
        assert_eq!(
            params.get(super::ANALYTICS_STATE_PARAM).map(String::as_str),
            Some("empty")
        );
        assert!(params.contains_key(super::ANALYTICS_DATA_PARAM));

        let response = epsx_dioxus_ui::pages::analytics::AnalyticsResponse {
            success: true,
            data: vec![epsx_dioxus_ui::pages::analytics::AnalyticsRow {
                rank: 100,
                symbol: "LIVE".into(),
                company_name: Some("Live row".into()),
                latest_date: "2026-07-27".into(),
                value: 1.0,
                active_status: "TRACK".into(),
                quarterly_performance: vec![],
                next_quarter_estimate: None,
                next_earnings_date: None,
                last_earnings_date: None,
                next_earnings_date_formatted: None,
                days_until_next_earnings: None,
                progress_percentage: None,
                current_eps: Some(1.0),
                growth_factor: Some(1.0),
                price_current: Some(1.0),
            }],
            pagination: epsx_dioxus_ui::pages::analytics::AnalyticsPagination {
                page: 1,
                limit: 10,
                total: 1,
                total_pages: 1,
                has_next: false,
                has_prev: false,
            },
            metadata: epsx_dioxus_ui::pages::analytics::AnalyticsMetadata {
                available_countries: vec!["america".into()],
                available_sectors: vec!["Technology".into()],
                request_timestamp: "2026-07-27T00:00:00Z".into(),
                data_source: "live".into(),
            },
            access_info: Some(epsx_dioxus_ui::pages::analytics::AnalyticsAccessInfo {
                min_accessible_rank: 100,
                locked_ranks_count: 99,
            }),
            message: None,
            processing_time_ms: 1,
        };
        record_analytics_load(&mut params, Ok(response));
        assert_eq!(
            params.get(super::ANALYTICS_STATE_PARAM).map(String::as_str),
            Some("ready")
        );
        record_analytics_load(&mut params, Err(AnalyticsLoadError::Unavailable));
        assert_eq!(
            params.get(super::ANALYTICS_STATE_PARAM).map(String::as_str),
            Some("unavailable")
        );
        assert!(!params.contains_key(super::ANALYTICS_DATA_PARAM));

        record_analytics_load(&mut params, Err(AnalyticsLoadError::Malformed));
        assert_eq!(
            params.get(super::ANALYTICS_STATE_PARAM).map(String::as_str),
            Some("malformed")
        );
    }

    fn owner_history_payload(owner: &str, with_intent: bool) -> serde_json::Value {
        let intents = if with_intent {
            vec![serde_json::json!({
                "id": "intent-1",
                "chain_id": "56",
                "payer": owner,
                "payee": "0x2222222222222222222222222222222222222222",
                "amount": "1000000",
                "token_address": "0x3333333333333333333333333333333333333333",
                "status": "pending",
                "escrow_id": null,
                "tx_hash": null,
                "description": null,
                "expires_at": null,
                "created_at": "2026-07-22T10:00:00Z",
                "updated_at": "2026-07-22T10:00:00Z"
            })]
        } else {
            Vec::new()
        };
        serde_json::json!({
            "address": owner,
            "intents": intents,
            "escrows": [],
            "total_intents": if with_intent { 1 } else { 0 },
            "total_escrows": 0
        })
    }

    #[test]
    fn notification_page_query_accepts_only_bounded_owner_filters_and_page() {
        assert_eq!(
            NotificationPageRequest::parse("").unwrap(),
            NotificationPageRequest {
                page: 1,
                status: None,
                notification_type: None,
                priority: None,
                start_date: None,
                end_date: None,
            }
        );
        assert_eq!(
            NotificationPageRequest::parse("status=unread&page=3")
                .unwrap()
                .service_query()
                .upstream_suffix(),
            "?limit=20&offset=40&status=unread"
        );
        assert_eq!(
            NotificationPageRequest::parse("status=all&page=2")
                .unwrap()
                .service_query()
                .upstream_suffix(),
            "?limit=20&offset=20"
        );
        assert_eq!(
            NotificationPageRequest::parse("status=unread&type=payment&priority=critical&page=3")
                .unwrap()
                .service_query()
                .upstream_suffix(),
            "?limit=20&offset=40&status=unread&type=payment&priority=critical"
        );
        assert_eq!(
            NotificationPageRequest::parse(
                "type=wallet_management&priority=urgent&start_date=2026-01-01T00:00:00Z&end_date=2026-01-31T23:59:59Z&page=2"
            )
            .unwrap()
            .service_query()
            .upstream_suffix(),
            "?limit=20&offset=20&type=wallet_management&priority=urgent&start_date=2026-01-01T00%3A00%3A00Z&end_date=2026-01-31T23%3A59%3A59Z"
        );
        for invalid in [
            "status=pending",
            "status=unread&status=read",
            "type=unknown",
            "priority=urgent&priority=low",
            "priority=unknown",
            "type=wallet_management&start_date=2026-02-01T00:00:00Z&end_date=2026-01-01T00:00:00Z",
            "start_date=not-a-date",
            "start_date=2026-01-01T00:00:00Z&start_date=2026-01-02T00:00:00Z",
            "page=0",
            "page=1&offset=20",
            "unknown=value",
        ] {
            assert!(
                NotificationPageRequest::parse(invalid).is_err(),
                "notification query must fail closed: {invalid}"
            );
        }
    }

    #[test]
    fn auth_page_session_state_is_closed_and_auth_route_only() {
        assert_eq!(
            auth_page_session_state("/auth", &AccessVerification::MissingOrRejected, false),
            Some(AUTH_PAGE_SESSION_STATE_SIGNED_OUT)
        );
        assert_eq!(
            auth_page_session_state("/auth", &AccessVerification::MissingOrRejected, true),
            Some(AUTH_PAGE_SESSION_STATE_RECOVERING)
        );
        assert_eq!(
            auth_page_session_state("/auth", &AccessVerification::VerifierUnavailable, true),
            Some(AUTH_PAGE_SESSION_STATE_VERIFIER_UNAVAILABLE)
        );
        assert_eq!(
            auth_page_session_state("/account", &AccessVerification::MissingOrRejected, true),
            None
        );
    }

    #[test]
    fn account_payment_history_path_uses_only_a_safe_fixed_first_page() {
        assert_eq!(
            account_payment_history_path("0xAbC"),
            Some("/api/v1/pay/history/0xAbC?limit=10&offset=0".to_string())
        );
        for unsafe_owner in [
            "",
            ".",
            "..",
            "0xabc/foreign",
            "0xabc%2Fforeign",
            "0xabc?limit=100",
            "0xabc foreign",
            "0xabc\nforeign",
            "history",
            "force-release",
            "force-anything",
        ] {
            assert_eq!(
                account_payment_history_path(unsafe_owner),
                None,
                "{unsafe_owner:?}"
            );
        }
        assert!(account_payment_history_path(&"a".repeat(129)).is_none());
    }

    #[test]
    fn account_payment_history_records_ready_and_authoritative_empty() {
        let owner = "0x1111111111111111111111111111111111111111";
        let mut params = HashMap::new();
        record_account_payment_history_load(
            &mut params,
            owner,
            Ok(owner_history_payload(owner, true)),
        );
        assert_eq!(
            params
                .get(epsx_dioxus_ui::components::account::ACCOUNT_PAYMENT_HISTORY_STATE_PARAM)
                .map(String::as_str),
            Some(epsx_dioxus_ui::components::account::ACCOUNT_PAYMENT_HISTORY_READY)
        );
        assert!(params
            .contains_key(epsx_dioxus_ui::components::account::ACCOUNT_PAYMENT_HISTORY_DATA_PARAM));

        record_account_payment_history_load(
            &mut params,
            owner,
            Ok(owner_history_payload(owner, false)),
        );
        assert_eq!(
            params
                .get(epsx_dioxus_ui::components::account::ACCOUNT_PAYMENT_HISTORY_STATE_PARAM)
                .map(String::as_str),
            Some(epsx_dioxus_ui::components::account::ACCOUNT_PAYMENT_HISTORY_EMPTY)
        );
    }

    #[test]
    fn account_payment_history_never_turns_wrong_owner_or_failure_into_empty() {
        let owner = "0x1111111111111111111111111111111111111111";
        let mut params = HashMap::from([(
            epsx_dioxus_ui::components::account::ACCOUNT_PAYMENT_HISTORY_DATA_PARAM.to_string(),
            "stale-owner-data".to_string(),
        )]);
        record_account_payment_history_load(
            &mut params,
            owner,
            Ok(owner_history_payload(
                "0x9999999999999999999999999999999999999999",
                true,
            )),
        );
        assert_eq!(
            params
                .get(epsx_dioxus_ui::components::account::ACCOUNT_PAYMENT_HISTORY_STATE_PARAM)
                .map(String::as_str),
            Some(epsx_dioxus_ui::components::account::ACCOUNT_PAYMENT_HISTORY_MALFORMED)
        );
        assert!(!params
            .contains_key(epsx_dioxus_ui::components::account::ACCOUNT_PAYMENT_HISTORY_DATA_PARAM));

        record_account_payment_history_load(
            &mut params,
            owner,
            Err(super::AccountPaymentHistoryLoadError::Unavailable),
        );
        assert_eq!(
            params
                .get(epsx_dioxus_ui::components::account::ACCOUNT_PAYMENT_HISTORY_STATE_PARAM)
                .map(String::as_str),
            Some(epsx_dioxus_ui::components::account::ACCOUNT_PAYMENT_HISTORY_UNAVAILABLE)
        );
        assert!(!params
            .contains_key(epsx_dioxus_ui::components::account::ACCOUNT_PAYMENT_HISTORY_DATA_PARAM));
    }

    #[test]
    fn account_notification_preferences_record_only_validated_outcome_states() {
        let mut params = HashMap::from([(
            epsx_dioxus_ui::pages::account::ACCOUNT_NOTIFICATION_PREFERENCES_DATA_PARAM.to_string(),
            "stale-preferences".to_string(),
        )]);
        record_account_notification_preferences_load(
            &mut params,
            NotificationPreferencesLoadOutcome::Ready(serde_json::json!({
                "channels": {"email": true},
                "quiet_hours": null,
                "timezone": "UTC",
                "updated_at": null
            })),
        );
        assert_eq!(
            params
                .get(epsx_dioxus_ui::pages::account::ACCOUNT_NOTIFICATION_PREFERENCES_STATE_PARAM)
                .map(String::as_str),
            Some("ready")
        );
        assert!(params.contains_key(
            epsx_dioxus_ui::pages::account::ACCOUNT_NOTIFICATION_PREFERENCES_DATA_PARAM
        ));

        record_account_notification_preferences_load(
            &mut params,
            NotificationPreferencesLoadOutcome::Error(NotificationPreferencesLoadError::Malformed),
        );
        assert_eq!(
            params
                .get(epsx_dioxus_ui::pages::account::ACCOUNT_NOTIFICATION_PREFERENCES_STATE_PARAM)
                .map(String::as_str),
            Some("malformed")
        );
        assert!(!params.contains_key(
            epsx_dioxus_ui::pages::account::ACCOUNT_NOTIFICATION_PREFERENCES_DATA_PARAM
        ));

        record_account_notification_preferences_load(
            &mut params,
            NotificationPreferencesLoadOutcome::Error(
                NotificationPreferencesLoadError::DependencyUnavailable,
            ),
        );
        assert_eq!(
            params
                .get(epsx_dioxus_ui::pages::account::ACCOUNT_NOTIFICATION_PREFERENCES_STATE_PARAM)
                .map(String::as_str),
            Some("unavailable")
        );
    }

    #[test]
    fn account_notification_preferences_form_state_accepts_only_canonical_redirects() {
        let mut params = HashMap::from([(
            epsx_dioxus_ui::pages::account::ACCOUNT_NOTIFICATION_PREFERENCES_FORM_STATE_PARAM
                .to_string(),
            "stale".to_string(),
        )]);
        record_account_notification_preferences_form_state(&mut params, Some("saved"));
        assert_eq!(
            params.get(
                epsx_dioxus_ui::pages::account::ACCOUNT_NOTIFICATION_PREFERENCES_FORM_STATE_PARAM
            ),
            Some(&"saved".to_string())
        );
        record_account_notification_preferences_form_state(&mut params, Some("error"));
        assert_eq!(
            params.get(
                epsx_dioxus_ui::pages::account::ACCOUNT_NOTIFICATION_PREFERENCES_FORM_STATE_PARAM
            ),
            Some(&"error".to_string())
        );
        record_account_notification_preferences_form_state(&mut params, None);
        assert!(!params.contains_key(
            epsx_dioxus_ui::pages::account::ACCOUNT_NOTIFICATION_PREFERENCES_FORM_STATE_PARAM
        ));

        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("epsx.notification_preferences_flash=saved; other=ignored"),
        );
        assert_eq!(
            account_notification_preferences_flash_state(&headers, "preferences=saved"),
            Some("saved")
        );
        assert_eq!(
            account_notification_preferences_flash_state(&headers, "preferences=error"),
            None
        );
        assert_eq!(
            account_notification_preferences_flash_state(&HeaderMap::new(), "preferences=saved"),
            None
        );
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static(
                "epsx.notification_preferences_flash=saved; epsx.notification_preferences_flash=error",
            ),
        );
        assert_eq!(
            account_notification_preferences_flash_state(&headers, "preferences=saved"),
            None
        );
    }

    #[test]
    fn notification_load_records_ready_and_authoritative_empty_as_200() {
        let payload = serde_json::json!({
            "items": [{
                "id": "0x1",
                "user_id": "0x1111111111111111111111111111111111111111",
                "channel": "in_app",
                "recipient": "0x1111111111111111111111111111111111111111",
                "template_id": null,
                "subject": null,
                "body": "body",
                "data": null,
                "status": "sent",
                "error": null,
                "sent_at": null,
                "created_at": "2026-07-22T00:00:00Z",
                "read_at": null,
                "title": null,
                "notification_type": null,
                "priority": null,
                "action_url": null
            }],
            "total": 1
        });
        let mut params = HashMap::new();

        record_notification_load(
            &mut params,
            NotificationLoadSelection {
                page: 1,
                status: None,
                notification_type: None,
                priority: None,
                start_date: None,
                end_date: None,
            },
            crate::api::NotificationListLoadOutcome::Ready(payload.clone()),
        );

        assert_eq!(
            params
                .get(super::NOTIFICATIONS_STATE_PARAM)
                .map(String::as_str),
            Some("ok")
        );
        assert_eq!(
            params
                .get(super::NOTIFICATIONS_DATA_PARAM)
                .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok()),
            Some(payload)
        );
        assert_eq!(
            notifications_ssr_status("/notifications", &params),
            Some(axum::http::StatusCode::OK)
        );

        let empty = serde_json::json!({"items": [], "total": 0});
        record_notification_load(
            &mut params,
            NotificationLoadSelection {
                page: 1,
                status: None,
                notification_type: None,
                priority: None,
                start_date: None,
                end_date: None,
            },
            crate::api::NotificationListLoadOutcome::Empty(empty.clone()),
        );
        assert_eq!(
            params
                .get(super::NOTIFICATIONS_DATA_PARAM)
                .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok()),
            Some(empty)
        );
        assert_eq!(
            notifications_ssr_status("/notifications", &params),
            Some(axum::http::StatusCode::OK)
        );
    }

    #[test]
    fn notification_load_records_explicit_failure_statuses_and_removes_stale_payload() {
        for (outcome, state, status) in [
            (
                crate::api::NotificationListLoadOutcome::Unavailable(
                    crate::api::NotificationListUnavailable::Dependency,
                ),
                "error",
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
            ),
            (
                crate::api::NotificationListLoadOutcome::Malformed,
                "malformed",
                axum::http::StatusCode::BAD_GATEWAY,
            ),
        ] {
            let mut params = HashMap::from([(
                super::NOTIFICATIONS_DATA_PARAM.to_string(),
                serde_json::json!({"items": [{"title": "stale"}], "total": 1}).to_string(),
            )]);

            record_notification_load(
                &mut params,
                NotificationLoadSelection {
                    page: 1,
                    status: None,
                    notification_type: None,
                    priority: None,
                    start_date: None,
                    end_date: None,
                },
                outcome,
            );

            assert_eq!(
                params
                    .get(super::NOTIFICATIONS_STATE_PARAM)
                    .map(String::as_str),
                Some(state)
            );
            assert!(!params.contains_key(super::NOTIFICATIONS_DATA_PARAM));
            assert_eq!(
                notifications_ssr_status("/notifications", &params),
                Some(status)
            );
        }
    }

    #[test]
    fn return_url_must_remain_same_origin() {
        assert_eq!(
            safe_return_url("return_url=%2Fprofile%3Ftab%3Dauth"),
            "/profile?tab=auth"
        );
        assert_eq!(
            safe_return_url("return_url=https%3A%2F%2Fevil.example"),
            "/"
        );
        assert_eq!(safe_return_url("return_url=%2F%2Fevil.example%2Fx"), "/");
        assert_eq!(safe_return_url("return_url=%5C%5Cevil.example"), "/");
        assert_eq!(safe_return_url("return_url=%2Fauth"), "/");
    }

    #[tokio::test]
    async fn home_news_loader_uses_one_default_list_call_for_exact_root_paths_only() {
        let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let route_calls = std::sync::Arc::clone(&calls);
        let router = axum::Router::new().route(
            "/api/public/news",
            axum::routing::get(move || {
                let route_calls = std::sync::Arc::clone(&route_calls);
                async move {
                    route_calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    axum::Json(serde_json::json!({"articles": [], "total": 0}))
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("loopback listener");
        let address = listener.local_addr().expect("loopback address");
        tokio::spawn(async move {
            axum::serve(listener, router)
                .await
                .expect("home news mock server");
        });
        let client = epsx_client::ServiceClient::new(epsx_client::ClientConfig {
            base_url: format!("http://{address}"),
            timeout: std::time::Duration::from_secs(1),
        });

        for (expected_calls, path) in [(1, "/"), (2, "/index")] {
            let outcome = load_home_news(&client, path)
                .await
                .expect("exact home path must load news");
            assert!(matches!(
                outcome,
                crate::api::NewsListLoadOutcome::Empty {
                    total: 0,
                    page: 1,
                    limit: 12,
                    total_pages: 0,
                    ref query,
                    ref category,
                } if query.is_empty() && category == "all"
            ));
            assert_eq!(
                calls.load(std::sync::atomic::Ordering::SeqCst),
                expected_calls
            );
        }

        for path in ["/news", "/about", "/?q=foreign", "/index/"] {
            assert!(load_home_news(&client, path).await.is_none(), "{path}");
        }
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn home_analytics_loader_uses_fixed_credential_free_public_first_page() {
        let calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let route_calls = std::sync::Arc::clone(&calls);
        let router = axum::Router::new().route(
            "/api/analytics/rankings",
            axum::routing::get(
                move |headers: axum::http::HeaderMap, raw: axum::extract::RawQuery| {
                    let route_calls = std::sync::Arc::clone(&route_calls);
                    async move {
                        assert!(
                            !headers.contains_key(axum::http::header::AUTHORIZATION),
                            "the home preview must not inherit a user's ranking offset"
                        );
                        route_calls.lock().unwrap().push(raw.0.unwrap_or_default());
                        axum::Json(serde_json::json!({
                            "success": true,
                            "data": [],
                            "pagination": {
                                "page": 1,
                                "limit": 3,
                                "total": 0,
                                "totalPages": 0,
                                "hasNext": false,
                                "hasPrev": false
                            },
                            "metadata": {
                                "available_countries": ["america"],
                                "available_sectors": ["Technology"],
                                "request_timestamp": "2026-07-27T00:00:00Z",
                                "data_source": "live"
                            },
                            "access_info": {
                                "min_accessible_rank": 100,
                                "locked_ranks_count": 99
                            },
                            "message": null,
                            "processing_time_ms": 1
                        }))
                    }
                },
            ),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("loopback listener");
        let address = listener.local_addr().expect("loopback address");
        tokio::spawn(async move {
            axum::serve(listener, router)
                .await
                .expect("home analytics mock server");
        });
        let client = epsx_client::ServiceClient::new(epsx_client::ClientConfig {
            base_url: format!("http://{address}"),
            timeout: std::time::Duration::from_secs(1),
        });

        for path in ["/", "/index"] {
            let response = load_home_analytics(&client, path)
                .await
                .expect("exact home path must load rankings")
                .expect("valid public rankings response");
            assert!(response.data.is_empty());
            assert_eq!(response.pagination.page, 1);
            assert_eq!(response.pagination.limit, 3);
        }
        for path in ["/analytics", "/?page=1", "/index/"] {
            assert!(load_home_analytics(&client, path).await.is_none(), "{path}");
        }
        assert_eq!(
            *calls.lock().unwrap(),
            vec!["page=1&limit=3", "page=1&limit=3"]
        );
    }

    #[test]
    fn home_analytics_and_route_runtime_record_truthful_independent_states() {
        let response = epsx_dioxus_ui::pages::analytics::AnalyticsResponse {
            success: true,
            data: vec![],
            pagination: epsx_dioxus_ui::pages::analytics::AnalyticsPagination {
                page: 1,
                limit: 3,
                total: 0,
                total_pages: 0,
                has_next: false,
                has_prev: false,
            },
            metadata: epsx_dioxus_ui::pages::analytics::AnalyticsMetadata {
                available_countries: vec![],
                available_sectors: vec![],
                request_timestamp: "2026-07-27T00:00:00Z".into(),
                data_source: "live".into(),
            },
            access_info: Some(epsx_dioxus_ui::pages::analytics::AnalyticsAccessInfo {
                min_accessible_rank: 100,
                locked_ranks_count: 99,
            }),
            message: None,
            processing_time_ms: 1,
        };
        let mut params = HashMap::from([(
            super::HOME_NEWS_DATA_PARAM.to_string(),
            "independent-news-outcome".to_string(),
        )]);
        record_home_analytics_load(&mut params, Ok(response));
        assert_eq!(
            params
                .get(super::HOME_ANALYTICS_STATE_PARAM)
                .map(String::as_str),
            Some("empty")
        );
        assert_eq!(
            params.get(super::HOME_NEWS_DATA_PARAM).map(String::as_str),
            Some("independent-news-outcome")
        );
        record_home_analytics_load(&mut params, Err(super::AnalyticsLoadError::Unavailable));
        assert_eq!(
            params
                .get(super::HOME_ANALYTICS_STATE_PARAM)
                .map(String::as_str),
            Some("unavailable")
        );
        assert!(!params.contains_key(super::HOME_ANALYTICS_DATA_PARAM));
    }

    #[test]
    fn home_news_uses_a_distinct_param_and_keeps_root_status_soft_ok() {
        let mut params = HashMap::from([(
            "data_news".to_string(),
            "dedicated-news-route-value".to_string(),
        )]);
        record_home_news_load(
            &mut params,
            crate::api::NewsListLoadOutcome::Error {
                code: "content_unavailable".to_string(),
            },
        );

        assert_eq!(
            params.get("data_news").map(String::as_str),
            Some("dedicated-news-route-value")
        );
        assert_eq!(
            params
                .get(super::HOME_NEWS_DATA_PARAM)
                .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok()),
            Some(serde_json::json!({
                "state": "error",
                "code": "content_unavailable"
            }))
        );
        assert_eq!(news_ssr_status("/", &params), None);
        assert_eq!(news_ssr_status("/index", &params), None);

        let ctx = PageContext {
            path: "/".to_string(),
            params,
            ..Default::default()
        };
        let (meta, _) = epsx_dioxus_ui::pages::home::render(&ctx);
        assert_eq!(meta.status, PageStatus::Ok);
        let status = news_ssr_status(&ctx.path, &ctx.params).unwrap_or(match meta.status {
            PageStatus::Ok => StatusCode::OK,
            PageStatus::NotFound => StatusCode::NOT_FOUND,
        });
        assert_eq!(status, StatusCode::OK);
    }

    #[test]
    fn news_outcomes_drive_truthful_http_statuses() {
        let mut params = HashMap::new();
        params.insert(
            "data_news".to_string(),
            serde_json::json!({"state": "empty"}).to_string(),
        );
        assert_eq!(news_ssr_status("/news", &params), Some(StatusCode::OK));

        params.insert(
            "data_news".to_string(),
            serde_json::json!({"state": "error", "code": "invalid_news_query"}).to_string(),
        );
        assert_eq!(
            news_ssr_status("/news", &params),
            Some(StatusCode::BAD_REQUEST)
        );

        params.insert(
            "data_news_post".to_string(),
            serde_json::json!({"state": "not_found"}).to_string(),
        );
        assert_eq!(
            news_ssr_status("/news/missing", &params),
            Some(StatusCode::NOT_FOUND)
        );

        params.insert(
            "data_news_post".to_string(),
            serde_json::json!({"state": "error", "code": "content_unavailable"}).to_string(),
        );
        assert_eq!(
            news_ssr_status("/news/live-article", &params),
            Some(StatusCode::SERVICE_UNAVAILABLE)
        );
        assert_eq!(
            news_ssr_status("/news/live-article", &HashMap::new()),
            Some(StatusCode::SERVICE_UNAVAILABLE)
        );
        let dependency_error = HashMap::from([(
            "data_news_post".to_string(),
            serde_json::json!({"state": "error", "code": "content_unavailable"}).to_string(),
        )]);
        for malformed in ["/news/live-article/", "/news/not/a-route"] {
            assert_eq!(news_detail_route_slug(malformed), None, "{malformed}");
            assert_eq!(
                news_ssr_status(malformed, &dependency_error),
                None,
                "{malformed}"
            );
        }
        for invalid_slug in [
            "/news/%2Fsecret",
            "/news/%2E%2E",
            "/news/%zz",
            "/news/UPPER",
            "/news/-leading-hyphen",
        ] {
            assert_eq!(news_detail_route_slug(invalid_slug), None, "{invalid_slug}");
            assert_eq!(
                news_ssr_status(invalid_slug, &dependency_error),
                Some(StatusCode::NOT_FOUND),
                "{invalid_slug}"
            );
        }
        assert_eq!(
            news_detail_route_slug("/news/live-article"),
            Some("live-article")
        );
        assert_eq!(news_ssr_status("/about", &HashMap::new()), None);
    }

    #[test]
    fn news_list_ssr_status_accepts_only_list_outcomes() {
        for state in ["ready", "empty"] {
            let params = HashMap::from([(
                "data_news".to_string(),
                serde_json::json!({"state": state}).to_string(),
            )]);
            assert_eq!(
                news_ssr_status("/news", &params),
                Some(StatusCode::OK),
                "{state}"
            );
        }

        let invalid_query = HashMap::from([(
            "data_news".to_string(),
            serde_json::json!({"state": "error", "code": "invalid_news_query"}).to_string(),
        )]);
        assert_eq!(
            news_ssr_status("/news", &invalid_query),
            Some(StatusCode::BAD_REQUEST)
        );

        for outcome in [
            serde_json::json!({"state": "not_found"}),
            serde_json::json!({"state": "error", "code": "content_unavailable"}),
        ] {
            let params = HashMap::from([("data_news".to_string(), outcome.to_string())]);
            assert_eq!(
                news_ssr_status("/news", &params),
                Some(StatusCode::SERVICE_UNAVAILABLE)
            );
        }
    }

    #[test]
    fn news_detail_ssr_status_accepts_only_detail_outcomes() {
        let ready = HashMap::from([(
            "data_news_post".to_string(),
            serde_json::json!({"state": "ready"}).to_string(),
        )]);
        assert_eq!(
            news_ssr_status("/news/live-article", &ready),
            Some(StatusCode::OK)
        );

        let not_found = HashMap::from([(
            "data_news_post".to_string(),
            serde_json::json!({"state": "not_found"}).to_string(),
        )]);
        assert_eq!(
            news_ssr_status("/news/missing", &not_found),
            Some(StatusCode::NOT_FOUND)
        );

        for outcome in [
            serde_json::json!({"state": "empty"}),
            serde_json::json!({"state": "error", "code": "invalid_news_query"}),
            serde_json::json!({"state": "error", "code": "content_unavailable"}),
        ] {
            let params = HashMap::from([("data_news_post".to_string(), outcome.to_string())]);
            assert_eq!(
                news_ssr_status("/news/live-article", &params),
                Some(StatusCode::SERVICE_UNAVAILABLE)
            );
        }
    }

    #[test]
    fn news_ssr_status_fails_closed_on_missing_malformed_or_unknown_state() {
        for (path, key) in [
            ("/news", "data_news"),
            ("/news/live-article", "data_news_post"),
        ] {
            assert_eq!(
                news_ssr_status(path, &HashMap::new()),
                Some(StatusCode::SERVICE_UNAVAILABLE),
                "{path}: missing"
            );
            for raw in [
                "{not-json".to_string(),
                serde_json::json!({}).to_string(),
                serde_json::json!({"state": "future_state"}).to_string(),
            ] {
                let params = HashMap::from([(key.to_string(), raw)]);
                assert_eq!(
                    news_ssr_status(path, &params),
                    Some(StatusCode::SERVICE_UNAVAILABLE),
                    "{path}"
                );
            }
        }

        let dependency_error = HashMap::from([(
            "data_news_post".to_string(),
            serde_json::json!({"state": "error", "code": "content_unavailable"}).to_string(),
        )]);
        for path in [
            "/about",
            "/news/",
            "/news/live-article/",
            "/news/not/a-route",
        ] {
            assert_eq!(news_ssr_status(path, &dependency_error), None, "{path}");
        }
    }

    #[test]
    fn news_ssr_status_invalid_single_segment_slugs_are_not_found() {
        let ready = HashMap::from([(
            "data_news_post".to_string(),
            serde_json::json!({"state": "ready"}).to_string(),
        )]);
        for path in [
            "/news/UPPER",
            "/news/-leading-hyphen",
            "/news/%2Fsecret",
            "/news/%2E%2E",
            "/news/%zz",
        ] {
            assert!(news_detail_route_segment(path).is_some(), "{path}");
            assert_eq!(news_detail_route_slug(path), None, "{path}");
            assert_eq!(
                news_ssr_status(path, &ready),
                Some(StatusCode::NOT_FOUND),
                "{path}"
            );
        }
    }

    #[test]
    fn dynamic_news_metadata_is_context_escaped_in_the_full_shell() {
        let mut meta = PageMeta::marketing("News article");
        meta.title = "Close </title><script>alert(\"metadata-title\")</script> & news".into();
        meta.description =
            "Summary \"quoted\"'><script>alert('metadata-summary')</script> & more".into();
        let (title, description) = page_metadata(&meta);
        let shell = epsx_templates::page_shell(&title, &description, "", "", false);

        assert_eq!(shell.matches("<title>").count(), 1);
        assert_eq!(shell.matches("</title>").count(), 1);
        assert!(!shell.contains("</title><script>alert(\"metadata-title\")"));
        assert!(shell.contains(
            "<title>Close &lt;/title&gt;&lt;script&gt;alert(\"metadata-title\")&lt;/script&gt; &amp; news</title>"
        ));
        assert!(!shell.contains("\"><script>alert('metadata-summary')"));
        assert!(shell.contains(
            "content=\"Summary &quot;quoted&quot;&#39;&gt;&lt;script&gt;alert(&#39;metadata-summary&#39;)&lt;/script&gt; &amp; more\""
        ));
    }
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    #[test]
    fn portfolio_watchlist_loader_records_ready_signed_out_and_fail_closed_states() {
        use epsx_dioxus_ui::pages::portfolio::{
            WatchlistLayoutData, PORTFOLIO_WATCHLIST_DATA_PARAM, PORTFOLIO_WATCHLIST_STATE_PARAM,
        };

        let mut params = HashMap::new();
        record_portfolio_watchlist_load(
            &mut params,
            Ok(Some(WatchlistLayoutData {
                groups: vec![],
                ungrouped: vec!["AAPL".to_string()],
                watched: 1,
            })),
        );
        assert_eq!(
            params
                .get(PORTFOLIO_WATCHLIST_STATE_PARAM)
                .map(String::as_str),
            Some("ready")
        );
        assert_eq!(
            params
                .get(PORTFOLIO_WATCHLIST_DATA_PARAM)
                .map(String::as_str),
            Some(r#"{"groups":[],"ungrouped":["AAPL"],"watched":1}"#)
        );

        record_portfolio_watchlist_load(&mut params, Ok(None));
        assert_eq!(
            params
                .get(PORTFOLIO_WATCHLIST_STATE_PARAM)
                .map(String::as_str),
            Some("signed_out")
        );
        assert!(!params.contains_key(PORTFOLIO_WATCHLIST_DATA_PARAM));

        for (error, expected) in [
            (AnalyticsLoadError::Malformed, "malformed"),
            (AnalyticsLoadError::Unavailable, "unavailable"),
        ] {
            record_portfolio_watchlist_load(&mut params, Err(error));
            assert_eq!(
                params
                    .get(PORTFOLIO_WATCHLIST_STATE_PARAM)
                    .map(String::as_str),
                Some(expected)
            );
            assert!(!params.contains_key(PORTFOLIO_WATCHLIST_DATA_PARAM));
        }
    }

    #[test]
    fn urlencode_passes_alnum() {
        // Matches Vercel's prod middleware `epsx.return_url=%2F<path>` shape.
        assert_eq!(urlencode("/notifications"), "%2Fnotifications");
        // Wave 23 T3 — query parameter is now `?return_url=` (NOT
        // `?next=`). The encoder must still produce the same per-byte
        // shape regardless of the query parameter name.
        assert_eq!(
            urlencode("/auth?return_url=/x"),
            "%2Fauth%3Freturn_url%3D%2Fx"
        );
        assert_eq!(urlencode("plain"), "plain");
    }

    /// Wave 23 T3 — `?return_url=<path>` round-trip: a path that
    /// already contains a query string must be re-encoded so the
    /// `searchParams.get("return_url")` call on the auth page only
    /// sees the encoded value (not the inner `?` / `&`).
    #[test]
    fn urlencode_encodes_inner_query_separators() {
        assert_eq!(urlencode("/pricing?ref=foo"), "%2Fpricing%3Fref%3Dfoo");
        assert_eq!(urlencode("/x?a=1&b=2"), "%2Fx%3Fa%3D1%26b%3D2");
    }

    #[test]
    fn shared_navigation_preserves_normalized_path_and_raw_query_for_auth() {
        let target = normalized_request_target("/news/example", "q=eps&category=markets");
        assert_eq!(target, "/news/example?q=eps&category=markets");

        let header =
            frontend_navigation_html("/news/example", "q=eps&category=markets", false, None);
        let expected = "href=\"/auth?return_url=%2Fnews%2Fexample%3Fq%3Deps%26category%3Dmarkets\"";
        assert_eq!(header.matches(expected).count(), 3);
        assert_eq!(header.matches("data-epsx-auth-link").count(), 3);
        assert!(!header.contains("href=\"/auth\""));

        let encoded_query = "q=a%20b&q=c%2Bd&next=%2Fportfolio&probe=%3Ctag%3E";
        let encoded_target = normalized_request_target("/news/example", encoded_query);
        let encoded_header = frontend_navigation_html("/news/example", encoded_query, false, None);
        let encoded_return_url = "%2Fnews%2Fexample%3Fq%3Da%2520b%26q%3Dc%252Bd%26next%3D%252Fportfolio%26probe%3D%253Ctag%253E";
        assert_eq!(
            encoded_header
                .matches(&format!("href=\"/auth?return_url={encoded_return_url}\""))
                .count(),
            3
        );
        assert_eq!(
            safe_return_url(&format!("return_url={encoded_return_url}")),
            encoded_target
        );
    }

    #[test]
    fn shared_navigation_omits_auth_header_and_fails_hostile_targets_closed() {
        assert!(frontend_navigation_html("/auth", "return_url=%2Fnews", false, None).is_empty());

        for hostile_path in [
            "https://evil.example",
            "//evil.example",
            "/\\evil.example",
            "/news/\u{0007}bad",
        ] {
            let header = frontend_navigation_html(hostile_path, "", false, None);
            assert_eq!(
                header.matches("href=\"/auth?return_url=%2F\"").count(),
                3,
                "{hostile_path:?}"
            );
            assert!(!header.contains("evil.example"), "{hostile_path:?}");
            assert!(!header.contains("href=\"/auth\""), "{hostile_path:?}");
        }
    }

    #[test]
    fn shared_navigation_matches_production_without_a_network_badge() {
        let home = frontend_navigation_html("/", "", false, None);
        assert!(!home.contains("data-epsx-network=\"bsc-testnet\""));

        let plans = frontend_navigation_html("/plans", "", false, None);
        assert!(!plans.contains("data-epsx-network=\"bsc-testnet\""));
        assert!(!plans.contains("Current network: BSC Testnet"));
    }

    #[test]
    fn shared_navigation_connected_wallet_omits_full_width_sign_in_recommendation() {
        let header = frontend_navigation_html(
            "/analytics",
            "page=1&limit=10",
            false,
            Some("0x2ae30000000000000000000000000000000023be"),
        );

        assert_eq!(header.matches("data-epsx-wallet-pill").count(), 3);
        assert!(header.contains("href=\"/auth?return_url=%2Fanalytics%3Fpage%3D1%26limit%3D10\""));
        assert!(!header.contains("epsx-sign-in-banner"));
        assert!(!header.contains("Your wallet is connected"));
    }

    #[test]
    fn authenticated_navigation_prefers_the_verified_session_wallet() {
        let user = Some(User {
            id: "verified-user".to_string(),
            address: "0x2ae30000000000000000000000000000000023be".to_string(),
            ..Default::default()
        });

        assert_eq!(
            authoritative_navigation_wallet(
                &user,
                Some("0x9999000000000000000000000000000000009999")
            )
            .as_deref(),
            Some("0x2ae30000000000000000000000000000000023be")
        );

        let header = frontend_navigation_html(
            "/analytics",
            "",
            true,
            authoritative_navigation_wallet(&user, None).as_deref(),
        );
        assert!(header.contains("Wallet menu for 0x2ae3…23be"));
        assert!(header.contains("data-copy=\"0x2ae30000000000000000000000000000000023be\""));
        assert!(!header.contains("class=\"epsx-connect-btn\" type=\"button\" data-epsx-logout"));
    }

    #[test]
    fn shared_navigation_inlines_icons_and_preserves_progressive_actions() {
        let header = frontend_navigation_html(
            "/analytics",
            "",
            true,
            Some("0x2ae30000000000000000000000000000000023be"),
        );

        assert!(!header.contains("data-lucide"));
        assert!(!header.contains("<i "));
        for icon in [
            "lucide-chart-column",
            "lucide-code",
            "lucide-building",
            "lucide-chevron-down",
            "lucide-bell",
            "lucide-sun",
            "lucide-moon",
            "lucide-wallet",
            "lucide-user",
            "lucide-copy",
            "lucide-log-out",
            "lucide-menu",
        ] {
            assert!(header.contains(icon), "missing inline navbar icon {icon}");
        }

        assert_eq!(header.matches("data-epsx-action=\"toggle-nav\"").count(), 6);
        assert_eq!(
            header
                .matches("data-epsx-action=\"toggle-mobile-menu\"")
                .count(),
            2
        );
        assert_eq!(header.matches("data-epsx-logout").count(), 3);
        assert!(header.contains("data-epsx-action=\"theme-toggle\""));
        assert!(header.contains("href=\"/notifications\""));
        assert!(header.contains("href=\"/account\""));
    }

    #[test]
    fn shared_navigation_uses_lg_desktop_and_mobile_contract() {
        let header = frontend_navigation_html("/analytics", "", false, None);

        assert!(
            header.contains("class=\"epsx-desktop-navigation hidden lg:flex items-center gap-6\"")
        );
        assert!(header
            .contains("class=\"epsx-compact-brand lg:hidden flex items-center gap-2.5 group\""));
        assert!(header.contains("class=\"epsx-theme-btn lg:hidden\""));
        assert!(header.contains("id=\"epsx-mobile-menu-btn\""));
        assert!(header.contains("aria-controls=\"epsx-mobile-sheet\""));
        assert!(header.contains("id=\"epsx-mobile-sheet\""));
        assert!(header.contains("aria-label=\"Primary\""));
        assert!(header.contains("aria-label=\"Mobile\""));
        assert!(header.contains("id=\"epsx-nav-market-trigger\" class=\"epsx-nav-trigger active\""));
        assert!(header.contains(
            "id=\"epsx-mobile-market-trigger\" class=\"epsx-mobile-group-trigger active\""
        ));
        assert!(header.contains(
            "id=\"epsx-mobile-market-trigger\" class=\"epsx-mobile-group-trigger active\" type=\"button\" aria-expanded=\"true\""
        ));
        assert!(header.contains(
            "id=\"epsx-mobile-developer-panel\" class=\"epsx-mobile-group-items\" aria-labelledby=\"epsx-mobile-developer-trigger\" hidden"
        ));

        let article_header = frontend_navigation_html("/news/example", "", false, None);
        assert!(article_header
            .contains("id=\"epsx-nav-company-trigger\" class=\"epsx-nav-trigger active\""));
        assert!(article_header.contains("href=\"/news\" class=\"epsx-mobile-link active\""));
    }

    /// Wave 22 T4 — `/pricing` (no query) → 307 `/plans`.
    #[test]
    fn pricing_redirect_no_query() {
        let r = pricing_redirect_response("");
        assert_eq!(r.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(r.headers().get("location").unwrap(), "/plans");
    }

    /// Wave 22 T4 — `/pricing?ref=foo` → 307 `/plans?ref=foo`
    /// (query string is preserved verbatim).
    #[test]
    fn pricing_redirect_preserves_query() {
        let r = pricing_redirect_response("ref=foo&affiliate=bar");
        assert_eq!(r.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(
            r.headers().get("location").unwrap(),
            "/plans?ref=foo&affiliate=bar"
        );
    }

    #[test]
    fn design_bypass_query_is_local_only_and_truthy() {
        use epsx_bff::cookies::CookieEnvironment;

        assert!(design_bypass_requested(
            "__design_bypass=1",
            CookieEnvironment::Local
        ));
        assert!(design_bypass_requested(
            "theme=dark&__design_bypass=true",
            CookieEnvironment::Local
        ));
        assert!(!design_bypass_requested(
            "__design_bypass=0",
            CookieEnvironment::Local
        ));
        assert!(!design_bypass_requested(
            "__design_bypass=1",
            CookieEnvironment::Production
        ));
    }

    #[test]
    fn design_bypass_home_is_wallet_only_and_dashboard_keeps_support() {
        assert!(!design_bypass_identity_enabled(true, "/"));
        assert!(!design_bypass_identity_enabled(true, "/index"));
        assert!(!design_bypass_identity_enabled(true, "/dashboard"));
        assert!(design_bypass_identity_enabled(true, "/portfolio"));

        assert!(design_bypass_wallet_enabled(true, "/"));
        assert!(design_bypass_wallet_enabled(true, "/portfolio"));
        assert!(!design_bypass_wallet_enabled(true, "/dashboard"));

        assert!(!design_bypass_chat_enabled(true, "/"));
        assert!(design_bypass_chat_enabled(true, "/dashboard"));
        assert!(!design_bypass_chat_enabled(true, "/portfolio"));
    }

    // === Wave 35b T1 — AuthGate 307-redirect for marketing routes ===

    /// `/offline` must remain reachable without an authenticated session.
    /// An offline fallback that redirects through wallet auth cannot recover
    /// a disconnected browser. `/about` and `/contact` retain their pinned
    /// middleware behavior.
    #[test]
    fn unauth_redirect_paths_keeps_marketing_routes_but_exposes_offline() {
        let paths = super::UNAUTH_REDIRECT_PATHS;
        assert!(
            paths.contains(&"/about"),
            "UNAUTH_REDIRECT_PATHS must contain `/about` (Wave 35b T1)"
        );
        assert!(
            paths.contains(&"/contact"),
            "UNAUTH_REDIRECT_PATHS must contain `/contact` (Wave 35b T1)"
        );
        assert!(
            !paths.contains(&"/offline"),
            "offline fallback must be public"
        );
    }

    /// Wave 35b T1 — pre-existing protected paths from Wave 22/23
    /// (`/permissions`, `/notifications`, `/profile`) must remain in
    /// the list. This test guards against accidental removal during
    /// the Wave 35b edit.
    #[test]
    fn unauth_redirect_paths_keeps_wave22_23_entries() {
        let paths = super::UNAUTH_REDIRECT_PATHS;
        for path in &["/permissions", "/notifications", "/profile"] {
            assert!(
                paths.contains(path),
                "UNAUTH_REDIRECT_PATHS must still contain `{path}` (Wave 22/23 entry, do not regress)"
            );
        }
    }

    /// Wave 35b T1 — the redirect target must use the prod-shaped
    /// `?return_url=<urlencoded path>` query string (matches
    /// `apps-old/frontend/middleware.ts::handleUnauthenticated`).
    /// Encoding the slash as `%2F` is critical: the auth page's
    /// hydration script uses `searchParams.get("return_url")` which
    /// only returns the first segment if the slash is left raw.
    #[test]
    fn unauth_redirect_uses_return_url_shape() {
        // The exact encoder logic is in `urlencode`; this test pins
        // the SHAPE so future edits can't accidentally rename it
        // back to `?next=` (Wave 23 T3 explicitly retired `?next=`).
        assert_eq!(urlencode("/about"), "%2Fabout");
        assert_eq!(urlencode("/contact"), "%2Fcontact");
        assert_eq!(urlencode("/offline"), "%2Foffline");
        // The redirect location string shape:
        assert_eq!(
            format!("/auth?return_url={}", urlencode("/about")),
            "/auth?return_url=%2Fabout"
        );
    }

    #[test]
    fn authenticated_badge_shell_is_private_while_offline_stays_public() {
        let mut authenticated = StatusCode::OK.into_response();
        apply_ssr_cache_policy(&mut authenticated, true, false, false, "/rankings");
        assert_eq!(
            authenticated
                .headers()
                .get(axum::http::header::CACHE_CONTROL),
            Some(&axum::http::HeaderValue::from_static("private, no-store"))
        );
        assert!(authenticated.headers().get("x-epsx-public-cache").is_none());
        assert_eq!(
            authenticated.headers().get(axum::http::header::VARY),
            Some(&axum::http::HeaderValue::from_static(
                "Cookie, Authorization"
            ))
        );

        let mut signed_out = StatusCode::OK.into_response();
        apply_ssr_cache_policy(&mut signed_out, false, false, false, "/rankings");
        assert!(signed_out
            .headers()
            .get(axum::http::header::CACHE_CONTROL)
            .is_none());

        for authenticated_request in [false, true] {
            let mut offline = StatusCode::OK.into_response();
            apply_ssr_cache_policy(
                &mut offline,
                authenticated_request,
                authenticated_request,
                authenticated_request,
                "/offline",
            );
            assert_eq!(
                offline.headers().get(axum::http::header::CACHE_CONTROL),
                Some(&axum::http::HeaderValue::from_static(
                    "public, max-age=0, must-revalidate"
                ))
            );
            assert_eq!(
                offline.headers().get("x-epsx-public-cache"),
                Some(&axum::http::HeaderValue::from_static("offline-shell-v1"))
            );
            assert!(offline.headers().get(axum::http::header::VARY).is_none());
        }
    }

    #[test]
    fn recovery_bearing_frontend_html_is_private_and_varies_by_credentials() {
        let mut response = StatusCode::OK.into_response();
        apply_ssr_cache_policy(&mut response, false, true, false, "/auth");
        assert_eq!(
            response.headers().get(axum::http::header::CACHE_CONTROL),
            Some(&axum::http::HeaderValue::from_static("private, no-store"))
        );
        assert_eq!(
            response.headers().get(axum::http::header::VARY),
            Some(&axum::http::HeaderValue::from_static(
                "Cookie, Authorization"
            ))
        );
    }

    #[test]
    fn verifier_unavailable_auth_html_is_private_and_varies_by_credentials() {
        let mut response = StatusCode::SERVICE_UNAVAILABLE.into_response();
        apply_ssr_cache_policy(&mut response, false, false, true, "/auth");
        assert_eq!(
            response.headers().get(axum::http::header::CACHE_CONTROL),
            Some(&axum::http::HeaderValue::from_static("private, no-store"))
        );
        assert_eq!(
            response.headers().get(axum::http::header::VARY),
            Some(&axum::http::HeaderValue::from_static(
                "Cookie, Authorization"
            ))
        );
    }
}

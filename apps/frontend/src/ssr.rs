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
use epsx_dioxus_ui::pages::home::{HOME_ANALYTICS_DATA_PARAM, HOME_ANALYTICS_STATE_PARAM};
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

fn escaped_page_metadata(meta: &PageMeta) -> (String, String) {
    (
        epsx_templates::html_text_escape_pub(&meta.title),
        epsx_templates::html_attr_escape_pub(&meta.description),
    )
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
    // homepage wallet-only so its header renders the wallet pill + sign-in
    // banner instead of bell/profile/sign-out controls.
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
    // connected-but-not-signed-in browser gets the same wallet pill and
    // sign-in banner as the development navigation client.
    if wallet.address.is_none() {
        if let Some(design_wallet) = auth::design_bypass_wallet_state(design_bypass_wallet) {
            wallet = design_wallet;
        } else if let Some(dev_wallet) = auth::dev_bypass_wallet_state() {
            wallet = dev_wallet;
        }
    }
    let wallet_address = wallet.address.clone();
    let is_authenticated = user.is_some();
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
    // sticky header that mirrors prod's `epsx.io` NavMenu) which
    // emits raw `onclick="epsx.toggleNav(this)"` attributes that the
    // `global_js()` controller already understands. The dropdown
    // menu items are rendered unconditionally (visibility is
    // controlled by CSS `.epsx-nav-wrap.open .epsx-nav-menu { display:
    // block; }`) — so every link is in the DOM and clickable.
    //
    // The auth page hides the navbar via the `path == "/auth"`
    // short-circuit (the dedicated `<AuthLayout>` is full-bleed).
    let nav_html =
        frontend_navigation_html(&path, &query, is_authenticated, wallet_address.as_deref());

    // Source-compatible shell: the development root has no global footer.
    // Page bodies remain responsible for any route-specific footer content.
    let include_footer = false;

    let (metadata_title, metadata_description) = escaped_page_metadata(&meta);
    let doc = epsx_templates::page_shell_with_body_class_and_keywords(
        &metadata_title,
        &metadata_description,
        meta.keywords.as_deref(),
        &nav_html,
        &body_html,
        include_footer,
        meta.body_class.as_deref().unwrap_or(""),
    );

    let route_runtime = match path.as_str() {
        "/analytics" => analytics_runtime_script(),
        "/offline" => offline_runtime_script(),
        "/manual" => manual_runtime_script(),
        "/developer/docs" => developer_docs_runtime_script(),
        _ => "",
    };
    let authenticated_header_runtime = notification_badge_runtime(is_authenticated, &path);
    let notification_push_runtime = notification_push_runtime(is_authenticated, &path);
    let notification_mutation_runtime = notification_mutation_runtime(is_authenticated, &path);
    let notification_realtime_runtime = notification_realtime_runtime(is_authenticated, &path);
    let authenticated_header_runtime = format!(
        "{authenticated_header_runtime}{notification_push_runtime}{notification_mutation_runtime}{notification_realtime_runtime}"
    );
    let recovery_runtime = if recover_session {
        format!(
            "<script data-epsx-session-recovery>{}</script>",
            epsx_bff::browser_auth::browser_session_recovery_script()
        )
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
        &format!(
            "<script>{}</script>{recovery_runtime}{}{route_runtime}{authenticated_header_runtime}{chat_widget_html}</body>",
            wallet_shim(),
            offline_worker_registration_script()
        ),
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
    } else if is_authenticated || recover_session || auth_page_verifier_unavailable {
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
    // /news/[slug]: the content service owns slug resolution. Unknown records
    // remain not-found, while dependency/malformed responses remain errors.
    if let Some(slug) = news_detail_route_slug(path) {
        let outcome = crate::api::load_news_post(state.content.as_ref(), slug).await;
        params.insert(
            "data_news_post".into(),
            serde_json::to_string(&outcome).expect("news detail outcome is serializable"),
        );
    }
    // `/developer/usage` intentionally has no loader. No owner-safe metering
    // contract exists, so the page renders an explicit unavailable state.
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
    // `/portfolio` intentionally has no loader. No frozen owner-scoped
    // holdings/watchlist contract exists, so the page fails closed instead of
    // treating an ambiguous wallet-service path as authoritative.
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
    // `/developer` intentionally has no loader. API-key and rate-limit data
    // remain unavailable until A4/A5 provide owner-scoped reads and secret-once
    // mutation contracts.
    // `/developer/docs` intentionally does not fetch the historical
    // `/api/v1/developer/docs` canned fixture. Its version-pinned catalog is
    // rendered directly until A5 provides a generated contract that can prove
    // route/auth/rate-limit drift end to end.
    // Dynamic payment pages intentionally perform no intent lookup until A6
    // provides an owner-safe intent and finality contract.
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

/// Shared SSR-safe wallet challenge/sign/verify bridge.
fn wallet_shim() -> &'static str {
    epsx_bff::browser_auth::browser_auth_script()
}

/// `/analytics` is hydration-less. Bind only the route's native page-size
/// selector and backend-authorized watchlist controls.
fn analytics_runtime_script() -> &'static str {
    r#"<script data-epsx-analytics-runtime>
(function () {
  var symbolPattern = /^[A-Z0-9][A-Z0-9.-]{0,19}$/;
  function normalizedSymbols(payload) {
    if (!payload || payload.success !== true || !payload.data || !Array.isArray(payload.data.symbols)) return null;
    var seen = Object.create(null);
    var symbols = [];
    for (var i = 0; i < payload.data.symbols.length; i += 1) {
      if (typeof payload.data.symbols[i] !== 'string') return null;
      var symbol = payload.data.symbols[i].trim().toUpperCase();
      if (!symbolPattern.test(symbol)) return null;
      if (!seen[symbol]) {
        seen[symbol] = true;
        symbols.push(symbol);
      }
    }
    return symbols;
  }
  function statusFor(button) {
    var sibling = button.nextElementSibling;
    return sibling && sibling.classList.contains('stock-watchlist-status') ? sibling : null;
  }
  function applyMembership(symbols) {
    var set = Object.create(null);
    symbols.forEach(function (symbol) { set[symbol] = true; });
    document.querySelectorAll('[data-watchlist-toggle="true"]').forEach(function (button) {
      var symbol = (button.getAttribute('data-symbol') || '').toUpperCase();
      var watched = set[symbol] === true;
      button.setAttribute('data-watchlisted', watched ? 'true' : 'false');
      button.setAttribute('aria-label', (watched ? 'Remove ' : 'Add ') + symbol + ' ' + (watched ? 'from' : 'to') + ' watchlist');
      button.classList.toggle('text-pink-500', watched);
      button.classList.toggle('text-gray-400', !watched);
      var glyph = button.querySelector('[data-watchlist-glyph="true"]');
      if (glyph) glyph.textContent = watched ? '♥' : '♡';
    });
  }
  document.addEventListener('change', function (event) {
    var select = event.target instanceof Element ? event.target.closest('[data-analytics-limit="true"]') : null;
    if (select && select.form) select.form.requestSubmit();
  });
  document.addEventListener('click', function (event) {
    var button = event.target instanceof Element ? event.target.closest('[data-watchlist-toggle="true"]') : null;
    if (!button || button.disabled || button.getAttribute('aria-busy') === 'true') return;
    event.preventDefault();
    var symbol = (button.getAttribute('data-symbol') || '').trim().toUpperCase();
    if (!symbolPattern.test(symbol)) return;
    var removing = button.getAttribute('data-watchlisted') === 'true';
    var status = statusFor(button);
    button.disabled = true;
    button.setAttribute('aria-busy', 'true');
    if (status) status.textContent = (removing ? 'Removing ' : 'Adding ') + symbol + '…';
    var options = {
      method: removing ? 'DELETE' : 'POST',
      credentials: 'same-origin',
      headers: { 'Accept': 'application/json' }
    };
    var endpoint = '/api/users/watchlist';
    if (removing) {
      endpoint += '/' + encodeURIComponent(symbol);
    } else {
      options.headers['Content-Type'] = 'application/json';
      options.body = JSON.stringify({ symbol: symbol });
    }
    fetch(endpoint, options).then(function (response) {
      if (response.status === 401) {
        window.location.assign('/auth?return_url=%2Fanalytics');
        return null;
      }
      if (!response.ok) throw new Error('watchlist request failed');
      return response.json();
    }).then(function (payload) {
      if (payload === null) return;
      var symbols = normalizedSymbols(payload);
      if (symbols === null) throw new Error('watchlist response malformed');
      applyMembership(symbols);
      if (status) status.textContent = symbol + (removing ? ' removed from' : ' added to') + ' watchlist.';
    }).catch(function () {
      if (status) status.textContent = 'Could not update ' + symbol + '. Please try again.';
    }).then(function () {
      button.disabled = false;
      button.setAttribute('aria-busy', 'false');
    });
  });
})();
</script>"#
}

/// `/offline` is SSR-only, so ordinary Dioxus event closures are not hydrated.
/// Bind the retry button in the page shell without a `javascript:` URL. The
/// script is constant and contains no request/query/user data.
fn offline_runtime_script() -> &'static str {
    r#"<script data-epsx-offline-runtime>
(function () {
  var button = document.querySelector('[data-offline-reload="true"]');
  if (!button) return;
  button.addEventListener('click', function () {
    button.disabled = true;
    button.setAttribute('aria-busy', 'true');
    var status = document.getElementById('offline-retry-status');
    if (status) status.textContent = 'Checking your connection…';
    window.location.reload();
  });
})();
</script>"#
}

/// Register the public recovery worker from every frontend document so a
/// user can install the `/offline` shell before the connection is lost. The
/// worker itself caches no current page, request data, API response, session,
/// or owner resource.
fn offline_worker_registration_script() -> &'static str {
    r#"<script data-epsx-offline-worker-registration>
(function () {
  if (!('serviceWorker' in navigator) || !window.isSecureContext) return;
  navigator.serviceWorker.register('/service-worker.js', {
    scope: '/',
    updateViaCache: 'none'
  }).catch(function () {
    // Offline recovery is progressive enhancement. Registration failure must
    // not prevent the network-rendered application from remaining usable.
  });
})();
</script>"#
}

/// Versioned, public-shell-only service worker. Its entire CacheStorage write
/// surface is one credential-free, query-free `/offline` document bearing a
/// server-owned public-cache marker. All other requests, including API, auth,
/// user, owner, admin, notification, analytics, and payment traffic, bypass
/// the worker without a cache read or write.
pub(crate) fn offline_service_worker_script() -> &'static str {
    r#"'use strict';

const CACHE_PREFIX = 'epsx-public-offline-';
const CACHE_NAME = 'epsx-public-offline-v1';
const OFFLINE_PATH = '/offline';
const PUBLIC_CACHE_MARKER = 'offline-shell-v1';

self.addEventListener('install', function (event) {
  event.waitUntil((async function () {
    const response = await fetch(new Request(OFFLINE_PATH, {
      method: 'GET',
      credentials: 'omit',
      cache: 'reload',
      redirect: 'error'
    }));
    const contentType = response.headers.get('content-type') || '';
    const responseUrl = new URL(response.url);
    if (!response.ok || response.type !== 'basic' ||
        responseUrl.origin !== self.location.origin ||
        responseUrl.pathname !== OFFLINE_PATH || responseUrl.search !== '' ||
        !contentType.startsWith('text/html') ||
        response.headers.get('x-epsx-public-cache') !== PUBLIC_CACHE_MARKER) {
      throw new Error('offline shell did not satisfy the public cache contract');
    }
    const cache = await caches.open(CACHE_NAME);
    await cache.put(OFFLINE_PATH, response.clone());
    await self.skipWaiting();
  })());
});

self.addEventListener('activate', function (event) {
  event.waitUntil((async function () {
    const names = await caches.keys();
    await Promise.all(names.map(function (name) {
      if (name.startsWith(CACHE_PREFIX) && name !== CACHE_NAME) {
        return caches.delete(name);
      }
      return Promise.resolve(false);
    }));
    await self.clients.claim();
  })());
});

const PUSH_TITLE_MAX = 160;
const PUSH_BODY_MAX = 2048;

function safePushActionUrl(value) {
  if (value === null) return null;
  if (typeof value !== 'string' || value.length === 0 || value.length > 2048 ||
      value.indexOf('\\') !== -1 || /[\u0000-\u001f\u007f]/.test(value)) return null;
  try {
    const url = new URL(value, self.location.origin);
    if (url.origin !== self.location.origin || url.username !== '' || url.password !== '' ||
        (url.protocol !== 'http:' && url.protocol !== 'https:')) return null;
    return url.href;
  } catch (_error) {
    return null;
  }
}

function exactPushPayload(event) {
  if (!event.data) return null;
  let payload;
  try {
    payload = event.data.json();
  } catch (_error) {
    return null;
  }
  if (payload === null || typeof payload !== 'object' || Array.isArray(payload) ||
      Object.getPrototypeOf(payload) !== Object.prototype) return null;
  const keys = Object.keys(payload).sort();
  if (keys.length !== 4 || keys.join(',') !== 'action_url,body,data,title') return null;
  if (typeof payload.title !== 'string' || payload.title.length === 0 ||
      payload.title.length > PUSH_TITLE_MAX || /[\u0000-\u001f\u007f]/.test(payload.title)) return null;
  if (typeof payload.body !== 'string' || payload.body.length > PUSH_BODY_MAX ||
      /[\u0000-\u001f\u007f]/.test(payload.body)) return null;
  if (payload.data !== null && (typeof payload.data !== 'object' || Array.isArray(payload.data))) return null;
  const actionUrl = safePushActionUrl(payload.action_url);
  if (payload.action_url !== null && actionUrl === null) return null;
  return { title: payload.title, body: payload.body, actionUrl: actionUrl };
}

self.addEventListener('push', function (event) {
  const payload = exactPushPayload(event);
  if (!payload) return;
  const options = { body: payload.body };
  if (payload.actionUrl !== null) options.data = { action_url: payload.actionUrl };
  event.waitUntil(self.registration.showNotification(payload.title, options));
});

self.addEventListener('notificationclick', function (event) {
  event.notification.close();
  const actionUrl = event.notification.data &&
    safePushActionUrl(event.notification.data.action_url);
  if (!actionUrl) return;
  event.waitUntil(clients.matchAll({ type: 'window', includeUncontrolled: true }).then(function (windows) {
    for (const client of windows) {
      if (client.url && new URL(client.url).origin === self.location.origin) {
        if (typeof client.navigate === 'function') return client.navigate(actionUrl);
        if (typeof client.focus === 'function') return client.focus();
      }
    }
    if (typeof clients.openWindow === 'function') return clients.openWindow(actionUrl);
    return undefined;
  }));
});

function isExactOfflineNavigation(request) {
  if (request.method !== 'GET' || request.mode !== 'navigate') return false;
  const url = new URL(request.url);
  if (url.origin !== self.location.origin || url.search !== '' || url.hash !== '') return false;
  return url.pathname === OFFLINE_PATH;
}

self.addEventListener('fetch', function (event) {
  if (!isExactOfflineNavigation(event.request)) return;
  event.respondWith(fetch(event.request).catch(async function () {
    const cache = await caches.open(CACHE_NAME);
    const cached = await cache.match(OFFLINE_PATH, { ignoreSearch: false });
    return cached || Response.error();
  }));
});
"#
}

/// `/manual` retains the pinned source's screenshot viewer without requiring
/// Dioxus hydration. This route-scoped script reads only static DOM attributes,
/// supplies image-error fallbacks, and implements dialog focus restoration,
/// Escape/backdrop close, and a single-control focus trap.
fn manual_runtime_script() -> &'static str {
    r#"<script data-epsx-manual-runtime>
(function () {
  var dialog = document.querySelector('[data-manual-dialog="true"]');
  if (!dialog) return;
  var panel = dialog.querySelector('[data-manual-dialog-panel="true"]');
  var image = dialog.querySelector('[data-manual-dialog-image="true"]');
  var title = dialog.querySelector('[data-manual-dialog-title="true"]');
  var close = dialog.querySelector('[data-manual-dialog-close="true"]');
  var previousFocus = null;
  var bodyOverflowBeforeDialog = null;

  function lockBodyScroll() {
    if (bodyOverflowBeforeDialog !== null) return;
    bodyOverflowBeforeDialog = document.body.style.overflow;
    document.body.style.overflow = 'hidden';
  }

  function restoreBodyScroll() {
    if (bodyOverflowBeforeDialog === null) return;
    document.body.style.overflow = bodyOverflowBeforeDialog;
    bodyOverflowBeforeDialog = null;
  }

  function hideDialog() {
    dialog.hidden = true;
    restoreBodyScroll();
    if (previousFocus && typeof previousFocus.focus === 'function') previousFocus.focus();
    previousFocus = null;
  }

  document.querySelectorAll('[data-manual-screenshot="true"]').forEach(function (button) {
    var thumb = button.querySelector('img');
    function showFallback() {
      button.setAttribute('data-image-error', 'true');
      button.disabled = true;
      button.removeAttribute('aria-haspopup');
      button.setAttribute('aria-label', button.getAttribute('data-screenshot-alt') + ' screenshot unavailable');
    }
    if (thumb) {
      thumb.addEventListener('error', showFallback);
      if (thumb.complete && thumb.naturalWidth === 0) showFallback();
    }
    button.addEventListener('click', function () {
      previousFocus = button;
      image.src = button.getAttribute('data-screenshot-src') || '';
      image.alt = button.getAttribute('data-screenshot-alt') || '';
      title.textContent = button.getAttribute('data-screenshot-alt') || 'Feature screenshot';
      dialog.hidden = false;
      lockBodyScroll();
      close.focus();
    });
  });

  close.addEventListener('click', hideDialog);
  dialog.addEventListener('click', function (event) {
    if (!panel.contains(event.target)) hideDialog();
  });
  dialog.addEventListener('keydown', function (event) {
    if (event.key === 'Escape') {
      event.preventDefault();
      hideDialog();
    } else if (event.key === 'Tab') {
      event.preventDefault();
      close.focus();
    }
  });

  document.querySelectorAll('[data-route-template="true"]').forEach(function (link) {
    link.addEventListener('click', function (event) { event.preventDefault(); });
  });
})();
</script>"#
}

/// `/developer/docs` is rendered by hydration-less Dioxus SSR. This constant,
/// route-scoped controller restores the pinned source's mobile navigator,
/// accessible endpoint accordions, language tabs, and copy controls. It reads
/// only the static documentation DOM and never sends an API request or handles
/// a credential; the source's live executor remains fail-closed until A4/A5.
fn developer_docs_runtime_script() -> &'static str {
    r#"<script data-epsx-developer-docs-runtime>
(function () {
  var root = document.querySelector('.developer-docs-page');
  if (!root) return;
  var sidebar = root.querySelector('[data-docs-sidebar="true"]');
  var sidebarToggle = root.querySelector('[data-docs-sidebar-toggle="true"]');
  var overlay = root.querySelector('[data-docs-sidebar-overlay="true"]');
  var sidebarWasOpened = false;

  function setSidebar(open) {
    if (!sidebar || !sidebarToggle || !overlay) return;
    sidebar.classList.toggle('open', open);
    sidebarToggle.setAttribute('aria-expanded', open ? 'true' : 'false');
    sidebarToggle.setAttribute('aria-label', open ? 'Close API reference navigation' : 'Open API reference navigation');
    sidebarToggle.querySelector('span').textContent = open ? '×' : '☰';
    overlay.hidden = !open;
    if (open) {
      sidebarWasOpened = true;
      var firstLink = sidebar.querySelector('[data-docs-section-link]');
      if (firstLink) firstLink.focus();
    } else if (sidebarWasOpened) {
      window.requestAnimationFrame(function () { sidebarToggle.focus(); });
    }
  }

  if (sidebarToggle) sidebarToggle.addEventListener('click', function () {
    setSidebar(sidebarToggle.getAttribute('aria-expanded') !== 'true');
  });
  if (overlay) overlay.addEventListener('click', function () { setSidebar(false); });
  root.querySelectorAll('[data-docs-section-link]').forEach(function (link) {
    link.addEventListener('click', function (event) {
      event.preventDefault();
      root.querySelectorAll('[data-docs-section-link]').forEach(function (item) { item.classList.remove('active'); });
      link.classList.add('active');
      var section = document.getElementById('section-' + link.getAttribute('data-docs-section-link'));
      var reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
      if (section) section.scrollIntoView({ behavior: reduceMotion ? 'auto' : 'smooth', block: 'start' });
      if (window.matchMedia('(max-width: 1023px)').matches) setSidebar(false);
    });
  });

  root.querySelectorAll('[data-docs-endpoint-toggle="true"]').forEach(function (button) {
    var body = document.getElementById(button.getAttribute('aria-controls'));
    if (!body) return;
    button.addEventListener('click', function () {
      var open = button.getAttribute('aria-expanded') !== 'true';
      button.setAttribute('aria-expanded', open ? 'true' : 'false');
      body.hidden = !open;
      var chevron = button.querySelector('.docs-endpoint-card-chevron');
      if (chevron) chevron.textContent = open ? '▾' : '▸';
    });
  });

  root.querySelectorAll('.docs-code-example').forEach(function (example) {
    var tabs = Array.prototype.slice.call(example.querySelectorAll('[data-docs-code-tab]'));
    var panels = Array.prototype.slice.call(example.querySelectorAll('[data-docs-code-panel]'));
    function select(index, focus) {
      tabs.forEach(function (tab, tabIndex) {
        var selected = tabIndex === index;
        tab.classList.toggle('active', selected);
        tab.setAttribute('aria-selected', selected ? 'true' : 'false');
        tab.tabIndex = selected ? 0 : -1;
      });
      panels.forEach(function (panel) {
        panel.hidden = panel.getAttribute('data-docs-code-panel') !== tabs[index].getAttribute('data-docs-code-tab');
      });
      if (focus) tabs[index].focus();
    }
    tabs.forEach(function (tab, index) {
      tab.addEventListener('click', function () { select(index, false); });
      tab.addEventListener('keydown', function (event) {
        var next = index;
        if (event.key === 'ArrowRight') next = (index + 1) % tabs.length;
        else if (event.key === 'ArrowLeft') next = (index - 1 + tabs.length) % tabs.length;
        else if (event.key === 'Home') next = 0;
        else if (event.key === 'End') next = tabs.length - 1;
        else return;
        event.preventDefault();
        select(next, true);
      });
    });
    var copy = example.querySelector('[data-docs-copy-code="true"]');
    if (copy) copy.addEventListener('click', function () {
      var current = example.querySelector('[data-docs-code-panel]:not([hidden]) code');
      if (current && window.epsx) window.epsx.copyText(current.textContent || '', copy);
    });
  });

  root.querySelectorAll('[data-docs-copy-response="true"]').forEach(function (button) {
    button.addEventListener('click', function () {
      var current = button.parentElement.querySelector('.docs-response-panel code');
      if (current && window.epsx) window.epsx.copyText(current.textContent || '', button);
    });
  });

  document.addEventListener('keydown', function (event) {
    if (event.key === 'Escape' && sidebarToggle && sidebarToggle.getAttribute('aria-expanded') === 'true') {
      event.preventDefault();
      setSidebar(false);
    }
  });
})();
</script>"#
}

/// The shared header is static, so its badge target always starts unavailable,
/// empty, and hidden. Only a server-verified authenticated response receives
/// this controller; signed-out pages therefore cannot request notification
/// data. Every refresh clears the previous display before it calls the exact
/// unread-count BFF route, and any non-success, malformed body, network error,
/// or superseded response leaves the badge unavailable rather than showing a
/// fabricated zero or stale count. The notifications page owns the complete
/// notification projection, including its unread total and dependency state,
/// so it must not issue a second badge request that can contradict the page
/// response or create a duplicate browser error.
fn notification_badge_runtime(is_authenticated: bool, path: &str) -> &'static str {
    // `/offline` is an explicitly public/cacheable recovery shell even when a
    // request happens to carry a valid session. Never let user-specific
    // notification activity enter that response.
    if !is_authenticated || matches!(path, "/offline" | "/notifications") {
        return "";
    }
    r#"<script data-epsx-notification-badge-runtime>
(function () {
  'use strict';
  var endpoint = '/api/v1/notifications/unread-count';
  var target = document.querySelector('[data-epsx-notification-badge-target="true"]');
  var badge = document.querySelector('[data-epsx-notification-unread-badge="true"]');
  if (!target || !badge) return;
  var pollTimer = null;
  var requestController = null;
  var requestGeneration = 0;

  function clearPoll() {
    if (pollTimer !== null) window.clearTimeout(pollTimer);
    pollTimer = null;
  }

  function setUnavailable() {
    badge.textContent = '';
    badge.hidden = true;
    badge.setAttribute('aria-hidden', 'true');
    badge.setAttribute('data-state', 'unavailable');
    target.setAttribute('aria-label', 'Notifications');
  }

  function exactCount(payload) {
    if (payload === null || typeof payload !== 'object' || Array.isArray(payload)) return null;
    if (Object.getPrototypeOf(payload) !== Object.prototype) return null;
    var keys = Object.keys(payload);
    if (keys.length !== 1 || keys[0] !== 'count') return null;
    if (!Object.prototype.hasOwnProperty.call(payload, 'count')) return null;
    if (!Number.isSafeInteger(payload.count) || payload.count < 0) return null;
    return payload.count;
  }

  function showCount(count) {
    if (count === 0) {
      setUnavailable();
      badge.setAttribute('data-state', 'available');
      return;
    }
    badge.textContent = count > 99 ? '99+' : String(count);
    badge.hidden = false;
    badge.setAttribute('aria-hidden', 'false');
    badge.setAttribute('data-state', 'available');
    target.setAttribute('aria-label', 'Notifications, ' + String(count) + ' unread');
  }

  function schedulePoll() {
    clearPoll();
    if (!document.hidden) pollTimer = window.setTimeout(loadCount, 60000);
  }

  async function loadCount() {
    clearPoll();
    setUnavailable();
    if (document.hidden) return;

    requestGeneration += 1;
    var generation = requestGeneration;
    if (requestController) requestController.abort();
    requestController = typeof AbortController === 'function' ? new AbortController() : null;

    try {
      var response = await fetch(endpoint, {
        method: 'GET',
        cache: 'no-store',
        credentials: 'include',
        headers: { 'accept': 'application/json' },
        signal: requestController ? requestController.signal : undefined
      });
      if (generation !== requestGeneration || document.hidden || !response.ok) return;
      var payload = await response.json();
      if (generation !== requestGeneration || document.hidden) return;
      var count = exactCount(payload);
      if (count === null) return;
      showCount(count);
    } catch (_error) {
      if (generation === requestGeneration && !document.hidden) setUnavailable();
    } finally {
      if (generation === requestGeneration) {
        requestController = null;
        if (!document.hidden) schedulePoll();
      }
    }
  }

  document.addEventListener('visibilitychange', function () {
    if (document.hidden) {
      requestGeneration += 1;
      clearPoll();
      if (requestController) requestController.abort();
      requestController = null;
      setUnavailable();
    } else {
      loadCount();
    }
  });

  if (window.location.pathname === '/notifications' &&
      !document.querySelector('[data-notifications-window="complete"]')) {
    setUnavailable();
    return;
  }
  loadCount();
})();
</script>"#
}

/// Account-only browser push controller. The server remains authoritative for
/// capability and subscription state; this runtime only requests browser
/// permission after an explicit click and forwards the browser subscription
/// through the same authenticated BFF boundary used by the Dioxus page.
/// Provider delivery is intentionally not claimed by any UI state.
///
/// Minimal URL-encoder for the `next=` query parameter. Only handles
/// the characters Vercel's middleware actually encodes; intentionally
/// avoids pulling in a full url-encoding crate for this one call site.
/// Keep the URL helper immediately before this runtime so the notification
/// badge verifier's bounded function slice remains limited to the badge.
fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push_str(&format!("%{:02X}", b));
            }
        }
    }
    out
}

fn notification_push_runtime(is_authenticated: bool, path: &str) -> &'static str {
    if !is_authenticated || path != "/account" {
        return "";
    }
    r#"<script data-epsx-notification-push-runtime>
(function () {
  'use strict';
  var root = document.querySelector('[data-epsx-notification-push="true"]');
  if (!root) return;
  var statusNode = root.querySelector('[data-push-status="true"]');
  var enable = root.querySelector('[data-push-action="enable"]');
  var disable = root.querySelector('[data-push-action="disable"]');
  if (!statusNode || !enable || !disable) return;

  var endpoint = '/api/v1/notifications/push';
  var capability = null;
  var currentSubscription = null;
  var busy = false;

  function setState(state, message) {
    root.setAttribute('data-push-state', state);
    statusNode.textContent = message;
    enable.hidden = state === 'subscribed';
    enable.disabled = state !== 'ready';
    disable.hidden = state !== 'subscribed';
    disable.disabled = state !== 'subscribed';
  }

  function exactStatus(payload) {
    if (payload === null || typeof payload !== 'object' || Array.isArray(payload)) return null;
    if (Object.getPrototypeOf(payload) !== Object.prototype) return null;
    var keys = Object.keys(payload).sort();
    if (keys.length !== 3 || keys.join(',') !== 'enabled,public_key,subscribed') return null;
    if (typeof payload.enabled !== 'boolean' || typeof payload.subscribed !== 'boolean') return null;
    if (payload.public_key !== null && typeof payload.public_key !== 'string') return null;
    if (typeof payload.public_key === 'string' &&
        (payload.public_key.length === 0 || payload.public_key.length > 256 ||
         !/^[A-Za-z0-9_-]+$/.test(payload.public_key))) return null;
    if (payload.enabled !== (payload.public_key !== null)) return null;
    if (!payload.enabled && payload.subscribed) return null;
    return payload;
  }

  function validEndpoint(value) {
    if (typeof value !== 'string' || value.length === 0 || value.length > 2048) return false;
    try {
      var parsed = new URL(value);
      return parsed.protocol === 'https:' && parsed.username === '' &&
        parsed.password === '' && parsed.search === '' && parsed.hash === '';
    } catch (_error) {
      return false;
    }
  }

  function validKey(value) {
    return typeof value === 'string' && value.length > 0 && value.length <= 256 &&
      /^[A-Za-z0-9_-]+$/.test(value);
  }

  function subscriptionBody(subscription) {
    if (!subscription || typeof subscription.toJSON !== 'function') return null;
    var value;
    try {
      value = subscription.toJSON();
    } catch (_error) {
      return null;
    }
    var keys = value && value.keys;
    if (!value || typeof value !== 'object' || !validEndpoint(value.endpoint) ||
        !keys || typeof keys !== 'object' || !validKey(keys.p256dh) || !validKey(keys.auth)) {
      return null;
    }
    return { endpoint: value.endpoint, p256dh: keys.p256dh, auth: keys.auth };
  }

  function base64urlBytes(value) {
    if (!validKey(value) || value.length % 4 === 1) return null;
    var encoded = value.replace(/-/g, '+').replace(/_/g, '/');
    while (encoded.length % 4 !== 0) encoded += '=';
    try {
      var decoded = atob(encoded);
      var bytes = new Uint8Array(decoded.length);
      for (var index = 0; index < decoded.length; index += 1) bytes[index] = decoded.charCodeAt(index);
      return bytes;
    } catch (_error) {
      return null;
    }
  }

  function browserPushSupported() {
    return window.isSecureContext && 'Notification' in window &&
      'serviceWorker' in navigator && 'PushManager' in window;
  }

  async function readStatus() {
    setState('checking', 'Checking whether browser push is available…');
    try {
      var response = await fetch(endpoint, {
        method: 'GET',
        cache: 'no-store',
        credentials: 'include',
        headers: { 'accept': 'application/json' }
      });
      if (!response.ok) throw new Error('status');
      var payload = exactStatus(await response.json());
      if (!payload) throw new Error('malformed');
      capability = payload;
      if (!payload.enabled) {
        setState('unavailable', 'Browser push is unavailable until the notification service is configured.');
        return;
      }
      if (!browserPushSupported()) {
        setState('unsupported', 'This browser cannot create a push subscription.');
        return;
      }
      if (Notification.permission === 'denied') {
        setState('blocked', 'Browser notification permission is blocked; change it in browser settings to retry.');
        return;
      }
      setState(payload.subscribed ? 'subscribed' : 'ready', payload.subscribed
        ? 'A browser push subscription is registered for this wallet.'
        : 'Browser push is ready. Enable it from this browser when you are ready.');
    } catch (_error) {
      capability = null;
      setState('unavailable', 'Browser push availability could not be verified.');
    }
  }

  enable.addEventListener('click', async function () {
    if (busy || !capability || !capability.enabled) return;
    busy = true;
    setState('pending', 'Requesting browser notification permission…');
    try {
      if (!browserPushSupported()) {
        setState('unsupported', 'This browser cannot create a push subscription.');
        return;
      }
      var permission = Notification.permission;
      if (permission === 'default') permission = await Notification.requestPermission();
      if (permission !== 'granted') {
        setState('blocked', 'Browser notification permission was not granted.');
        return;
      }
      var registration = await navigator.serviceWorker.ready;
      currentSubscription = await registration.pushManager.getSubscription();
      if (!currentSubscription) {
        var applicationServerKey = base64urlBytes(capability.public_key);
        if (!applicationServerKey) throw new Error('key');
        currentSubscription = await registration.pushManager.subscribe({
          userVisibleOnly: true,
          applicationServerKey: applicationServerKey
        });
      }
      var body = subscriptionBody(currentSubscription);
      if (!body) throw new Error('subscription');
      var response = await fetch(endpoint, {
        method: 'PUT',
        cache: 'no-store',
        credentials: 'include',
        headers: { 'accept': 'application/json', 'content-type': 'application/json' },
        body: JSON.stringify(body)
      });
      if (!response.ok) throw new Error('subscribe');
      var payload = exactStatus(await response.json());
      if (!payload || !payload.enabled || !payload.subscribed) throw new Error('subscribe_response');
      capability = payload;
      setState('subscribed', 'A browser push subscription is registered for this wallet.');
    } catch (_error) {
      currentSubscription = null;
      setState('ready', 'The browser subscription could not be registered. Try again when the service is available.');
    } finally {
      busy = false;
    }
  });

  disable.addEventListener('click', async function () {
    if (busy || !capability || !capability.subscribed) return;
    busy = true;
    setState('pending', 'Removing the browser push subscription…');
    try {
      if (!currentSubscription) {
        var registration = await navigator.serviceWorker.ready;
        currentSubscription = await registration.pushManager.getSubscription();
      }
      var body = subscriptionBody(currentSubscription);
      if (!body) throw new Error('subscription');
      var response = await fetch(endpoint, {
        method: 'DELETE',
        cache: 'no-store',
        credentials: 'include',
        headers: { 'accept': 'application/json', 'content-type': 'application/json' },
        body: JSON.stringify({ endpoint: body.endpoint })
      });
      if (!response.ok) throw new Error('unsubscribe');
      var payload = exactStatus(await response.json());
      if (!payload || payload.subscribed) throw new Error('unsubscribe_response');
      capability = payload;
      if (currentSubscription && typeof currentSubscription.unsubscribe === 'function') {
        try { await currentSubscription.unsubscribe(); } catch (_error) {}
      }
      currentSubscription = null;
      setState(payload.enabled ? 'ready' : 'unavailable', payload.enabled
        ? 'Browser push is ready to enable again.'
        : 'Browser push is unavailable until the notification service is configured.');
    } catch (_error) {
      setState('subscribed', 'The browser push subscription could not be removed.');
    } finally {
      busy = false;
    }
  });

  readStatus();
})();
</script>"#
}

/// Authenticated `/notifications` owner mutation controller. Every action is
/// selected from a closed map, carries only a bounded notification identity,
/// uses same-origin credentials, and reloads the server-rendered page only
/// after the Rust BFF confirms `{ ok: true }`. A successful mutation never
/// claims provider acceptance, delivery, or acknowledgement beyond the exact
/// operation requested.
fn notification_mutation_runtime(is_authenticated: bool, path: &str) -> &'static str {
    if !is_authenticated || path != "/notifications" {
        return "";
    }
    r#"<script data-epsx-notification-mutation-runtime>
(function () {
  'use strict';
  var statusNode = document.querySelector('[data-notification-mutation-status="true"]');
  if (!statusNode) return;

  var actions = {
    read: { method: 'POST', suffix: '/read' },
    unread: { method: 'POST', suffix: '/unread' },
    acknowledge: { method: 'PUT', suffix: '/acknowledge' },
    dismiss: { method: 'POST', suffix: '/dismiss' },
    delete: { method: 'POST', suffix: '/delete' },
    'mark-all': { method: 'POST', path: '/api/v1/notifications/mark-all-read' },
    'clear-all': { method: 'POST', path: '/api/v1/notifications/clear-all' }
  };
  var busy = false;

  function validId(value) {
    return typeof value === 'string' && value.length > 0 && value.length <= 128 &&
      !/[\u0000-\u0020\u007f]/.test(value);
  }

  function requestFor(action, id) {
    var definition = actions[action];
    if (!definition) return null;
    if (definition.path) return definition;
    if (!validId(id)) return null;
    return { method: definition.method, path: '/api/v1/notifications/' + encodeURIComponent(id) + definition.suffix };
  }

  function exactSuccess(payload) {
    return payload !== null && typeof payload === 'object' && !Array.isArray(payload) &&
      Object.getPrototypeOf(payload) === Object.prototype &&
      Object.keys(payload).length === 1 && Object.keys(payload)[0] === 'ok' && payload.ok === true;
  }

  function setStatus(message, alert) {
    statusNode.textContent = message;
    statusNode.setAttribute('data-notification-mutation-state', alert ? 'error' : 'ready');
  }

  async function run(button) {
    if (busy) return;
    var action = button.getAttribute('data-notification-mutation');
    var id = button.getAttribute('data-notification-id');
    var request = requestFor(action, id);
    if (!request) {
      setStatus('This notification action is unavailable.', true);
      return;
    }
    if (action === 'clear-all' && typeof window.confirm === 'function' &&
        !window.confirm('Remove all notifications from this wallet?')) return;
    busy = true;
    document.querySelectorAll('[data-notification-mutation]').forEach(function (item) {
      item.disabled = true;
    });
    setStatus('Saving notification changes…', false);
    try {
      var response = await fetch(request.path, {
        method: request.method,
        cache: 'no-store',
        credentials: 'include',
        headers: { 'accept': 'application/json' },
        body: request.method === 'POST' || request.method === 'PUT' ? '{}' : undefined
      });
      if (!response.ok || !exactSuccess(await response.json())) throw new Error('mutation');
      setStatus('Notification changes saved. Reloading…', false);
      window.location.reload();
    } catch (_error) {
      setStatus('Notification changes could not be saved. Try again when the service is available.', true);
      document.querySelectorAll('[data-notification-mutation]').forEach(function (item) {
        item.disabled = false;
      });
      busy = false;
    }
  }

  document.querySelectorAll('[data-notification-mutation]').forEach(function (button) {
    button.addEventListener('click', function () { run(button); });
  });
})();
</script>"#
}

/// Authenticated `/notifications` owner stream controller. The stream remains
/// a server-rendered data source: a validated event is acknowledged through
/// the Rust BFF and the page is reloaded only after the durable cursor write
/// succeeds. No event payload is copied into HTML and no provider delivery is
/// inferred from the connection state.
fn notification_realtime_runtime(is_authenticated: bool, path: &str) -> &'static str {
    if !is_authenticated || path != "/notifications" {
        return "";
    }
    r#"<script data-epsx-notification-realtime-runtime>
(function () {
  'use strict';
  var statusNode = document.querySelector('[data-notifications-live-status="true"]');
  if (!statusNode || typeof EventSource !== 'function') return;

  var endpoint = '/api/v1/notifications/stream';
  var ackEndpoint = '/api/v1/notifications/stream/ack';
  var source = null;
  var reloadTimer = null;
  var closed = false;

  function setState(state, message) {
    statusNode.setAttribute('data-notifications-live-state', state);
    statusNode.textContent = message;
  }

  function boundedText(value, max, allowEmpty) {
    return typeof value === 'string' && (allowEmpty || value.length > 0) &&
      value.length <= max && !/[\u0000-\u001f\u007f]/.test(value);
  }

  function safeActionUrl(value) {
    if (value === null) return null;
    if (!boundedText(value, 2048, false) || value.indexOf('\\') !== -1) return null;
    try {
      var url = new URL(value, window.location.origin);
      if (url.origin !== window.location.origin || url.username !== '' || url.password !== '' ||
          (url.protocol !== 'http:' && url.protocol !== 'https:')) return null;
      return url.href;
    } catch (_error) {
      return null;
    }
  }

  function exactNotification(event) {
    if (!event || typeof event.data !== 'string' || event.data.length === 0 ||
        event.data.length > 16 * 1024) return null;
    var payload;
    try { payload = JSON.parse(event.data); } catch (_error) { return null; }
    if (payload === null || typeof payload !== 'object' || Array.isArray(payload) ||
        Object.getPrototypeOf(payload) !== Object.prototype) return null;
    var keys = Object.keys(payload).sort();
    if (keys.length !== 9 || keys.join(',') !==
        'action_url,body,created_at,data,id,notification_type,priority,read_at,title') return null;
    if (!boundedText(payload.id, 128, false) || !boundedText(payload.title, 160, false) ||
        !boundedText(payload.body, 2048, true) || !boundedText(payload.created_at, 128, false)) return null;
    if (payload.data !== null &&
        (typeof payload.data !== 'object' || Array.isArray(payload.data))) return null;
    if (payload.notification_type !== null && !boundedText(payload.notification_type, 64, false)) return null;
    if (payload.priority !== null && !boundedText(payload.priority, 32, false)) return null;
    if (payload.read_at !== null && !boundedText(payload.read_at, 128, false)) return null;
    var actionUrl = safeActionUrl(payload.action_url);
    if (payload.action_url !== null && actionUrl === null) return null;
    return payload;
  }

  function scheduleReload() {
    if (reloadTimer !== null || closed) return;
    reloadTimer = window.setTimeout(function () {
      reloadTimer = null;
      if (!closed) window.location.reload();
    }, 250);
  }

  async function acknowledge(eventId) {
    if (typeof eventId !== 'string' || eventId.length === 0 || eventId.length > 128 ||
        /[\u0000-\u0020\u007f]/.test(eventId)) return false;
    try {
      var response = await fetch(ackEndpoint, {
        method: 'POST',
        cache: 'no-store',
        credentials: 'include',
        headers: { 'accept': 'application/json', 'content-type': 'application/json' },
        body: JSON.stringify({ event_id: eventId })
      });
      if (!response.ok) return false;
      var payload = await response.json();
      if (payload === null || typeof payload !== 'object' || Array.isArray(payload) ||
          Object.getPrototypeOf(payload) !== Object.prototype) return false;
      var keys = Object.keys(payload).sort();
      return keys.length === 2 && keys.join(',') === 'event_id,ok' &&
        payload.ok === true && payload.event_id === eventId;
    } catch (_error) {
      return false;
    }
  }

  function closeSource() {
    if (source) source.close();
    source = null;
  }

  function openSource() {
    if (closed || document.hidden) return;
    closeSource();
    setState('connecting', 'Live notification updates are connecting…');
    try {
      source = new EventSource(endpoint, { withCredentials: true });
    } catch (_error) {
      setState('unavailable', 'Live notification updates are unavailable.');
      return;
    }
    source.onopen = function () {
      setState('connected', 'Live notification updates connected.');
    };
    source.onerror = function () {
      setState('reconnecting', 'Live notification updates are reconnecting.');
    };
    source.addEventListener('notification', function (event) {
      var payload = exactNotification(event);
      if (!payload || !event.lastEventId) return;
      acknowledge(event.lastEventId).then(function (accepted) {
        if (accepted) scheduleReload();
        else setState('reconnecting', 'Live notification updates are reconnecting.');
      });
    });
  }

  document.addEventListener('visibilitychange', function () {
    if (document.hidden) {
      closeSource();
      setState('paused', 'Live notification updates are paused while this page is hidden.');
    } else {
      openSource();
    }
  });
  window.addEventListener('beforeunload', function () {
    closed = true;
    closeSource();
    if (reloadTimer !== null) window.clearTimeout(reloadTimer);
  });

  openSource();
})();
</script>"#
}

fn normalized_request_target(path: &str, query: &str) -> String {
    if query.is_empty() {
        path.to_string()
    } else {
        format!("{path}?{query}")
    }
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
    // Keep the supplied home references unchanged: their compact action
    // cluster intentionally has no network label. Other public routes mirror
    // the development tablet/desktop shell, which shows the read-only
    // BSC Testnet target beside the wallet action.
    let show_network = !matches!(path, "/" | "/index");
    let mut html = epsx_templates::epsx_header_for_session_and_wallet_with_network(
        is_authenticated,
        &return_target,
        wallet_address,
        show_network,
    );
    if !is_authenticated && wallet_address.is_some() {
        html.push_str(&epsx_templates::epsx_wallet_sign_in_banner(&return_target));
    }
    html
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
    use super::analytics_runtime_script;
    use super::apply_ssr_cache_policy;
    use super::auth_page_session_state;
    use super::design_bypass_chat_enabled;
    use super::design_bypass_identity_enabled;
    use super::design_bypass_requested;
    use super::design_bypass_wallet_enabled;
    use super::developer_docs_runtime_script;
    use super::escaped_page_metadata;
    use super::frontend_navigation_html;
    use super::load_home_analytics;
    use super::load_home_news;
    use super::manual_runtime_script;
    use super::news_detail_route_segment;
    use super::news_detail_route_slug;
    use super::news_ssr_status;
    use super::normalized_request_target;
    use super::notification_badge_runtime;
    use super::notification_mutation_runtime;
    use super::notification_push_runtime;
    use super::notification_realtime_runtime;
    use super::notifications_ssr_status;
    use super::offline_runtime_script;
    use super::offline_service_worker_script;
    use super::offline_worker_registration_script;
    use super::pricing_redirect_response;
    use super::record_account_notification_preferences_form_state;
    use super::record_account_notification_preferences_load;
    use super::record_account_payment_history_load;
    use super::record_analytics_load;
    use super::record_home_analytics_load;
    use super::record_home_news_load;
    use super::record_notification_load;
    use super::safe_return_url;
    use super::urlencode;
    use super::AnalyticsLoadError;
    use super::NotificationLoadSelection;
    use super::NotificationPageRequest;
    use crate::api::{NotificationPreferencesLoadError, NotificationPreferencesLoadOutcome};
    use axum::http::{header, HeaderMap, HeaderValue};
    use epsx_bff::session::AccessVerification;
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
            "/api/v1/content/news",
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

        let runtime = analytics_runtime_script();
        assert!(runtime.contains("data-epsx-analytics-runtime"));
        assert!(runtime.contains("credentials: 'same-origin'"));
        assert!(runtime.contains("method: removing ? 'DELETE' : 'POST'"));
        assert!(runtime.contains("/auth?return_url=%2Fanalytics"));
        assert!(!runtime.contains("localStorage"));
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
        let (title, description) = escaped_page_metadata(&meta);
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
        assert_eq!(header.matches(expected).count(), 2);
        assert_eq!(header.matches("data-epsx-auth-link").count(), 2);
        assert!(!header.contains("href=\"/auth\""));

        let encoded_query = "q=a%20b&q=c%2Bd&next=%2Fportfolio&probe=%3Ctag%3E";
        let encoded_target = normalized_request_target("/news/example", encoded_query);
        let encoded_header = frontend_navigation_html("/news/example", encoded_query, false, None);
        let encoded_return_url = "%2Fnews%2Fexample%3Fq%3Da%2520b%26q%3Dc%252Bd%26next%3D%252Fportfolio%26probe%3D%253Ctag%253E";
        assert_eq!(
            encoded_header
                .matches(&format!("href=\"/auth?return_url={encoded_return_url}\""))
                .count(),
            2
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
                2,
                "{hostile_path:?}"
            );
            assert!(!header.contains("evil.example"), "{hostile_path:?}");
            assert!(!header.contains("href=\"/auth\""), "{hostile_path:?}");
        }
    }

    #[test]
    fn shared_navigation_keeps_home_reference_clean_and_marks_non_home_network() {
        let home = frontend_navigation_html("/", "", false, None);
        assert!(!home.contains("data-epsx-network=\"bsc-testnet\""));

        let plans = frontend_navigation_html("/plans", "", false, None);
        assert!(plans.contains("data-epsx-network=\"bsc-testnet\""));
        assert!(plans.contains("Current network: BSC Testnet"));
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
    fn offline_runtime_binds_button_without_user_data_or_javascript_url() {
        let script = offline_runtime_script();
        assert!(script.contains("data-epsx-offline-runtime"));
        assert!(script.contains("[data-offline-reload=\"true\"]"));
        assert!(script.contains("window.location.reload()"));
        assert!(script.contains("aria-busy"));
        assert!(!script.contains("javascript:"));
        assert!(!script.contains("reason"));
        assert!(!script.contains("return_url"));
    }

    #[test]
    fn offline_worker_registration_is_constant_and_body_free() {
        let script = offline_worker_registration_script();
        assert!(script.contains("data-epsx-offline-worker-registration"));
        assert!(script.contains("register('/service-worker.js'"));
        assert!(script.contains("updateViaCache: 'none'"));
        assert!(!script.contains("fetch("));
        assert!(!script.contains("cookie"));
        assert!(!script.contains("Authorization"));
        assert!(!script.contains("location.search"));
    }

    #[test]
    fn offline_worker_cache_write_is_exact_public_shell_only() {
        let script = offline_service_worker_script();
        assert!(script.contains("credentials: 'omit'"));
        assert!(script.contains("url.pathname === OFFLINE_PATH"));
        assert!(script.contains("url.search !== ''"));
        assert!(script.contains("request.mode !== 'navigate'"));
        assert!(script.contains("response.type !== 'basic'"));
        assert!(script.contains("responseUrl.origin !== self.location.origin"));
        assert!(script.contains("responseUrl.pathname !== OFFLINE_PATH"));
        assert!(script.contains("response.headers.get('x-epsx-public-cache')"));
        assert!(script.contains("cache.put(OFFLINE_PATH"));
        assert!(!script.contains("cache.add"));
        assert!(!script.contains("cache.addAll"));
        assert!(!script.contains("cache.put(event.request"));
        assert!(!script.contains("ignoreSearch: true"));
    }

    #[test]
    fn offline_worker_push_delivery_is_bounded_and_same_origin_only() {
        let script = offline_service_worker_script();
        for anchor in [
            "addEventListener('push'",
            "Object.getPrototypeOf(payload) !== Object.prototype",
            "keys.length !== 4 || keys.join(',') !== 'action_url,body,data,title'",
            "PUSH_TITLE_MAX = 160",
            "PUSH_BODY_MAX = 2048",
            "showNotification(payload.title, options)",
            "addEventListener('notificationclick'",
            "clients.openWindow(actionUrl)",
            "url.origin !== self.location.origin",
        ] {
            assert!(
                script.contains(anchor),
                "missing push worker guard: {anchor}"
            );
        }
        assert!(!script.contains("event.data.text()"));
        assert!(!script.contains("console."));
        assert!(!script.contains("cache.put(event.request"));
    }

    #[test]
    fn offline_worker_push_script_is_javascript_syntax_valid() {
        use std::io::Write;
        use std::process::{Command, Stdio};

        let mut child = Command::new("node")
            .args(["--check", "-"])
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("run Node.js syntax check for the service worker");
        child
            .stdin
            .as_mut()
            .expect("Node.js syntax checker stdin")
            .write_all(offline_service_worker_script().as_bytes())
            .expect("write service worker source to Node.js");
        let output = child
            .wait_with_output()
            .expect("wait for Node.js syntax checker");
        assert!(
            output.status.success(),
            "service worker JavaScript syntax failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn manual_runtime_binds_accessible_dialog_without_request_data() {
        let script = manual_runtime_script();
        assert!(script.contains("data-epsx-manual-runtime"));
        assert!(script.contains("[data-manual-screenshot=\"true\"]"));
        assert!(script.contains("event.key === 'Escape'"));
        assert!(script.contains("previousFocus.focus()"));
        assert!(script.contains("data-image-error"));
        assert!(script.contains("thumb.complete && thumb.naturalWidth === 0"));
        assert!(script.contains("event.preventDefault()"));
        assert!(script.contains("var bodyOverflowBeforeDialog = null;"));
        assert!(script.contains("bodyOverflowBeforeDialog = document.body.style.overflow;"));
        assert!(script.contains("document.body.style.overflow = bodyOverflowBeforeDialog;"));
        assert!(!script.contains("document.body.style.overflow = '';"));
        assert!(!script.contains("javascript:"));
        assert!(!script.contains("window.location"));
        assert!(!script.contains("fetch("));
    }

    #[test]
    fn manual_runtime_restores_exact_body_overflow_for_every_dialog_close_path() {
        let source = manual_runtime_script()
            .strip_prefix("<script data-epsx-manual-runtime>\n")
            .and_then(|script| script.strip_suffix("\n</script>"))
            .expect("manual controller script envelope");
        let source_json = serde_json::to_string(source).expect("serialize manual controller");
        let harness = r###"
const assert = require('node:assert/strict');
const vm = require('node:vm');
const source = __SOURCE_JSON__;

function interactive(name) {
  return {
    name,
    listeners: Object.create(null),
    focusCalls: 0,
    addEventListener(type, listener) { this.listeners[type] = listener; },
    focus() { this.focusCalls += 1; document.activeElement = this; },
  };
}

const close = interactive('close');
const panelChild = {};
const panel = { contains(target) { return target === panelChild; } };
const image = { src: '', alt: '' };
const title = { textContent: '' };
const dialog = interactive('dialog');
dialog.hidden = true;
dialog.querySelector = function(selector) {
  const nodes = {
    '[data-manual-dialog-panel="true"]': panel,
    '[data-manual-dialog-image="true"]': image,
    '[data-manual-dialog-title="true"]': title,
    '[data-manual-dialog-close="true"]': close,
  };
  return nodes[selector] || null;
};

const thumb = interactive('thumb');
thumb.complete = false;
thumb.naturalWidth = 1;
const trigger = interactive('trigger');
trigger.disabled = false;
trigger.attributes = {
  'data-screenshot-src': '/public/screenshots/home.webp',
  'data-screenshot-alt': 'Home overview',
  'aria-haspopup': 'dialog',
};
trigger.querySelector = function(selector) {
  assert.equal(selector, 'img');
  return thumb;
};
trigger.getAttribute = function(name) { return this.attributes[name] || null; };
trigger.setAttribute = function(name, value) { this.attributes[name] = String(value); };
trigger.removeAttribute = function(name) { delete this.attributes[name]; };

const routeTemplate = interactive('route-template');
const document = {
  activeElement: null,
  body: { style: { overflow: 'clip' } },
  querySelector(selector) {
    assert.equal(selector, '[data-manual-dialog="true"]');
    return dialog;
  },
  querySelectorAll(selector) {
    if (selector === '[data-manual-screenshot="true"]') return [trigger];
    if (selector === '[data-route-template="true"]') return [routeTemplate];
    throw new Error('unexpected selector: ' + selector);
  },
};
let fetchCalls = 0;
vm.runInNewContext(source, {
  document,
  fetch() { fetchCalls += 1; throw new Error('manual controller must not fetch'); },
});

trigger.listeners.click();
assert.equal(dialog.hidden, false);
assert.equal(document.body.style.overflow, 'hidden');
assert.equal(image.src, '/public/screenshots/home.webp');
assert.equal(image.alt, 'Home overview');
assert.equal(title.textContent, 'Home overview');
assert.equal(document.activeElement, close);

trigger.listeners.click();
assert.equal(document.body.style.overflow, 'hidden');
close.listeners.click();
assert.equal(dialog.hidden, true);
assert.equal(document.body.style.overflow, 'clip');
assert.equal(document.activeElement, trigger);

document.body.style.overflow = 'scroll';
close.listeners.click();
assert.equal(document.body.style.overflow, 'scroll');

document.body.style.overflow = 'auto';
trigger.listeners.click();
dialog.listeners.click({ target: {} });
assert.equal(dialog.hidden, true);
assert.equal(document.body.style.overflow, 'auto');
assert.equal(document.activeElement, trigger);

document.body.style.overflow = 'clip';
trigger.listeners.click();
let prevented = 0;
dialog.listeners.keydown({ key: 'Escape', preventDefault() { prevented += 1; } });
assert.equal(prevented, 1);
assert.equal(dialog.hidden, true);
assert.equal(document.body.style.overflow, 'clip');
assert.equal(document.activeElement, trigger);

document.body.style.overflow = 'visible';
trigger.listeners.click();
prevented = 0;
dialog.listeners.keydown({ key: 'Tab', preventDefault() { prevented += 1; } });
assert.equal(prevented, 1);
assert.equal(dialog.hidden, false);
assert.equal(document.body.style.overflow, 'hidden');
assert.equal(document.activeElement, close);
close.listeners.click();
assert.equal(document.body.style.overflow, 'visible');
assert.equal(document.activeElement, trigger);

let routePrevented = 0;
routeTemplate.listeners.click({ preventDefault() { routePrevented += 1; } });
assert.equal(routePrevented, 1);
assert.equal(fetchCalls, 0);
"###
        .replace("__SOURCE_JSON__", &source_json);
        let output = std::process::Command::new("node")
            .arg("-e")
            .arg(harness)
            .output()
            .expect("run manual dialog hermetic Node.js fake DOM");
        assert!(
            output.status.success(),
            "manual dialog Node.js fake DOM failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    #[test]
    fn developer_docs_runtime_is_static_accessible_and_fail_closed() {
        let script = developer_docs_runtime_script();
        assert!(script.contains("data-epsx-developer-docs-runtime"));
        assert!(script.contains("data-docs-endpoint-toggle"));
        assert!(script.contains("aria-expanded"));
        assert!(script.contains("ArrowRight"));
        assert!(script.contains("window.epsx.copyText"));
        assert!(script.contains("window.matchMedia('(prefers-reduced-motion: reduce)').matches"));
        assert!(script.contains("behavior: reduceMotion ? 'auto' : 'smooth'"));
        assert!(script.contains("block: 'start'"));
        assert!(!script.contains("fetch("));
        assert!(!script.contains("Authorization"));
    }

    #[test]
    fn developer_docs_runtime_rechecks_reduced_motion_for_each_section_click() {
        let source = developer_docs_runtime_script()
            .strip_prefix("<script data-epsx-developer-docs-runtime>\n")
            .and_then(|script| script.strip_suffix("\n</script>"))
            .expect("developer docs controller script envelope");
        let source_json =
            serde_json::to_string(source).expect("serialize developer docs controller");
        let harness = format!(
            r#"
const assert = require('node:assert/strict');
const vm = require('node:vm');
const source = {source_json};
const listeners = Object.create(null);
const scrollCalls = [];
const mediaQueries = [];
let reduceMotion = false;
let prevented = 0;
let fetchCalls = 0;
const classNames = new Set();
let activeAdds = 0;
let activeRemoves = 0;

const link = {{
  classList: {{
    add(value) {{
      classNames.add(value);
      if (value === 'active') activeAdds += 1;
    }},
    remove(value) {{
      classNames.delete(value);
      if (value === 'active') activeRemoves += 1;
    }},
  }},
  addEventListener(type, listener) {{ listeners[type] = listener; }},
  getAttribute(name) {{
    assert.equal(name, 'data-docs-section-link');
    return 'overview';
  }},
}};
const section = {{
  scrollIntoView(options) {{ scrollCalls.push(options); }},
}};
const root = {{
  querySelector() {{ return null; }},
  querySelectorAll(selector) {{
    if (selector === '[data-docs-section-link]') return [link];
    if (selector === '[data-docs-endpoint-toggle="true"]') return [];
    if (selector === '.docs-code-example') return [];
    if (selector === '[data-docs-copy-response="true"]') return [];
    throw new Error('unexpected selector: ' + selector);
  }},
}};
const document = {{
  querySelector(selector) {{
    assert.equal(selector, '.developer-docs-page');
    return root;
  }},
  getElementById(id) {{
    assert.equal(id, 'section-overview');
    return section;
  }},
  addEventListener(type) {{ assert.equal(type, 'keydown'); }},
}};
const window = {{
  matchMedia(query) {{
    mediaQueries.push(query);
    if (query === '(prefers-reduced-motion: reduce)') return {{ matches: reduceMotion }};
    if (query === '(max-width: 1023px)') return {{ matches: false }};
    throw new Error('unexpected media query: ' + query);
  }},
  requestAnimationFrame(callback) {{ callback(); }},
}};
const context = {{
  document,
  window,
  fetch() {{
    fetchCalls += 1;
    throw new Error('controller must not fetch');
  }},
}};
vm.runInNewContext(source, context);
assert.equal(typeof listeners.click, 'function');

listeners.click({{ preventDefault() {{ prevented += 1; }} }});
reduceMotion = true;
listeners.click({{ preventDefault() {{ prevented += 1; }} }});

assert.deepEqual(
  scrollCalls.map((options) => ({{ behavior: options.behavior, block: options.block }})),
  [
    {{ behavior: 'smooth', block: 'start' }},
    {{ behavior: 'auto', block: 'start' }},
  ],
);
assert.deepEqual(mediaQueries, [
  '(prefers-reduced-motion: reduce)',
  '(max-width: 1023px)',
  '(prefers-reduced-motion: reduce)',
  '(max-width: 1023px)',
]);
assert.equal(prevented, 2);
assert.equal(activeAdds, 2);
assert.equal(activeRemoves, 2);
assert.equal(classNames.has('active'), true);
assert.equal(fetchCalls, 0);
"#
        );
        let output = std::process::Command::new("node")
            .arg("-e")
            .arg(harness)
            .output()
            .expect("run developer docs hermetic Node.js fake DOM");
        assert!(
            output.status.success(),
            "developer docs Node.js fake DOM failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    #[test]
    fn developer_docs_runtime_forwards_exact_active_copy_payloads_without_fetching() {
        let source = developer_docs_runtime_script()
            .strip_prefix("<script data-epsx-developer-docs-runtime>\n")
            .and_then(|script| script.strip_suffix("\n</script>"))
            .expect("developer docs controller script envelope");
        let source_json =
            serde_json::to_string(source).expect("serialize developer docs controller");
        let harness = format!(
            r#"
const assert = require('node:assert/strict');
const vm = require('node:vm');
const source = {source_json};
const codeListeners = Object.create(null);
const responseListeners = Object.create(null);
const copyCalls = [];
let fetchCalls = 0;

const codeButton = {{
  addEventListener(type, listener) {{ codeListeners[type] = listener; }},
}};
const responseButton = {{
  addEventListener(type, listener) {{ responseListeners[type] = listener; }},
  parentElement: {{
    querySelector(selector) {{
      assert.equal(selector, '.docs-response-panel code');
      return {{ textContent: '{{"success":true}}' }};
    }},
  }},
}};
function makeTab(language, selected) {{
  const listeners = Object.create(null);
  const attributes = {{
    'data-docs-code-tab': language,
    'aria-selected': selected ? 'true' : 'false',
  }};
  return {{
    language,
    listeners,
    tabIndex: selected ? 0 : -1,
    classList: {{ toggle() {{}} }},
    addEventListener(type, listener) {{ listeners[type] = listener; }},
    setAttribute(name, value) {{ attributes[name] = String(value); }},
    getAttribute(name) {{ return attributes[name] || null; }},
    focus() {{}},
  }};
}}
const tabs = [
  makeTab('curl', true),
  makeTab('javascript', false),
  makeTab('python', false),
];
const panels = [
  {{ language: 'curl', hidden: false, code: {{ textContent: 'curl active-example' }}, getAttribute() {{ return this.language; }} }},
  {{ language: 'javascript', hidden: true, code: {{ textContent: 'const activeLanguage = "javascript";' }}, getAttribute() {{ return this.language; }} }},
  {{ language: 'python', hidden: true, code: {{ textContent: 'active_language = "python"' }}, getAttribute() {{ return this.language; }} }},
];
const example = {{
  querySelectorAll(selector) {{
    if (selector === '[data-docs-code-tab]') return tabs;
    if (selector === '[data-docs-code-panel]') return panels;
    throw new Error('unexpected example querySelectorAll: ' + selector);
  }},
  querySelector(selector) {{
    if (selector === '[data-docs-copy-code="true"]') return codeButton;
    if (selector === '[data-docs-code-panel]:not([hidden]) code') {{
      const activePanel = panels.find((panel) => !panel.hidden);
      return activePanel ? activePanel.code : null;
    }}
    throw new Error('unexpected example querySelector: ' + selector);
  }},
}};
const root = {{
  querySelector(selector) {{
    if (selector === '[data-docs-sidebar="true"]') return null;
    if (selector === '[data-docs-sidebar-toggle="true"]') return null;
    if (selector === '[data-docs-sidebar-overlay="true"]') return null;
    throw new Error('unexpected root querySelector: ' + selector);
  }},
  querySelectorAll(selector) {{
    if (selector === '[data-docs-section-link]') return [];
    if (selector === '[data-docs-endpoint-toggle="true"]') return [];
    if (selector === '.docs-code-example') return [example];
    if (selector === '[data-docs-copy-response="true"]') return [responseButton];
    throw new Error('unexpected root querySelectorAll: ' + selector);
  }},
}};
const document = {{
  querySelector(selector) {{
    assert.equal(selector, '.developer-docs-page');
    return root;
  }},
  addEventListener(type) {{ assert.equal(type, 'keydown'); }},
}};
const window = {{
  epsx: {{
    copyText(text, button) {{ copyCalls.push({{ text, button }}); }},
  }},
}};
const context = {{
  document,
  window,
  fetch() {{
    fetchCalls += 1;
    throw new Error('developer docs copy runtime must not fetch');
  }},
}};
vm.runInNewContext(source, context);
assert.equal(typeof codeListeners.click, 'function');
assert.equal(typeof responseListeners.click, 'function');

codeListeners.click();
assert.equal(typeof tabs[1].listeners.click, 'function');
tabs[1].listeners.click();
codeListeners.click();
responseListeners.click();

assert.equal(copyCalls.length, 3);
assert.equal(copyCalls[0].text, 'curl active-example');
assert.equal(copyCalls[0].button, codeButton);
assert.equal(copyCalls[1].text, 'const activeLanguage = "javascript";');
assert.equal(copyCalls[1].button, codeButton);
assert.equal(copyCalls[2].text, '{{"success":true}}');
assert.equal(copyCalls[2].button, responseButton);
assert.equal(fetchCalls, 0);
"#
        );
        let output = std::process::Command::new("node")
            .arg("-e")
            .arg(harness)
            .output()
            .expect("run developer docs copy hermetic Node.js fake DOM");
        assert!(
            output.status.success(),
            "developer docs copy Node.js fake DOM failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    #[test]
    fn notification_badge_runtime_is_authenticated_only_and_uses_exact_read_route() {
        assert_eq!(notification_badge_runtime(false, "/rankings"), "");
        assert_eq!(notification_badge_runtime(true, "/notifications"), "");

        let script = notification_badge_runtime(true, "/rankings");
        assert!(script.contains("data-epsx-notification-badge-runtime"));
        assert_eq!(
            script.matches("/api/v1/notifications/unread-count").count(),
            1
        );
        assert!(script.contains("credentials: 'include'"));
        assert!(script.contains("method: 'GET'"));
        assert!(script.contains("cache: 'no-store'"));
        assert!(!script.contains("/api/v1/notifications?"));
        assert!(!script.contains("limit=1"));
        assert!(!script.contains("items.filter"));
    }

    #[test]
    fn notification_push_runtime_is_authenticated_account_only() {
        assert_eq!(notification_push_runtime(false, "/account"), "");
        assert_eq!(notification_push_runtime(true, "/rankings"), "");
        assert_eq!(notification_push_runtime(true, "/offline"), "");

        let script = notification_push_runtime(true, "/account");
        assert!(script.contains("data-epsx-notification-push-runtime"));
        assert!(script.contains("/api/v1/notifications/push"));
        assert!(script.contains("credentials: 'include'"));
        assert!(script.contains("cache: 'no-store'"));
        assert!(script.contains("Notification.requestPermission()"));
        assert!(script.contains("navigator.serviceWorker.ready"));
        assert!(script.contains("pushManager.subscribe"));
        assert!(script.contains("method: 'PUT'"));
        assert!(script.contains("method: 'DELETE'"));
        assert!(!script.contains("innerHTML"));
        assert!(!script.contains("console."));
    }

    #[test]
    fn notification_push_runtime_validates_capability_and_never_claims_delivery() {
        let script = notification_push_runtime(true, "/account");
        for anchor in [
            "Object.getPrototypeOf(payload) !== Object.prototype",
            "keys.length !== 3 || keys.join(',') !== 'enabled,public_key,subscribed'",
            "payload.enabled !== (payload.public_key !== null)",
            "!payload.enabled && payload.subscribed",
            "Browser push is unavailable until the notification service is configured.",
            "applicationServerKey: applicationServerKey",
            "if (!payload || !payload.enabled || !payload.subscribed)",
        ] {
            assert!(script.contains(anchor), "missing push guard: {anchor}");
        }
    }

    #[test]
    fn notification_realtime_runtime_is_authenticated_notifications_only() {
        assert_eq!(notification_realtime_runtime(false, "/notifications"), "");
        assert_eq!(notification_realtime_runtime(true, "/rankings"), "");
        assert_eq!(notification_realtime_runtime(true, "/offline"), "");

        let script = notification_realtime_runtime(true, "/notifications");
        for anchor in [
            "data-epsx-notification-realtime-runtime",
            "new EventSource(endpoint, { withCredentials: true })",
            "'/api/v1/notifications/stream/ack'",
            "method: 'POST'",
            "credentials: 'include'",
            "data-notifications-live-status=\"true\"",
            "event.lastEventId",
            "window.location.reload()",
            "Object.getPrototypeOf(payload) !== Object.prototype",
            "keys.length !== 9",
            "payload.ok === true",
        ] {
            assert!(script.contains(anchor), "missing realtime guard: {anchor}");
        }
        assert!(!script.contains("innerHTML"));
        assert!(!script.contains("console."));
    }

    #[test]
    fn notification_mutation_runtime_is_authenticated_notifications_only_and_closed() {
        assert_eq!(notification_mutation_runtime(false, "/notifications"), "");
        assert_eq!(notification_mutation_runtime(true, "/account"), "");
        assert_eq!(notification_mutation_runtime(true, "/offline"), "");

        let script = notification_mutation_runtime(true, "/notifications");
        for anchor in [
            "data-epsx-notification-mutation-runtime",
            "read: { method: 'POST', suffix: '/read' }",
            "acknowledge: { method: 'PUT', suffix: '/acknowledge' }",
            "'mark-all': { method: 'POST', path: '/api/v1/notifications/mark-all-read' }",
            "'clear-all': { method: 'POST', path: '/api/v1/notifications/clear-all' }",
            "encodeURIComponent(id)",
            "credentials: 'include'",
            "cache: 'no-store'",
            "window.location.reload()",
            "Notification changes could not be saved",
            "Object.getPrototypeOf(payload) === Object.prototype",
        ] {
            assert!(script.contains(anchor), "missing mutation guard: {anchor}");
        }
        assert!(!script.contains("innerHTML"));
        assert!(!script.contains("Provider delivered"));
        assert!(!script.contains("console."));
    }

    #[test]
    fn notification_badge_runtime_validates_exact_counts_and_caps_only_display() {
        let script = notification_badge_runtime(true, "/rankings");
        for anchor in [
            "Object.getPrototypeOf(payload) !== Object.prototype",
            "keys.length !== 1 || keys[0] !== 'count'",
            "Object.prototype.hasOwnProperty.call(payload, 'count')",
            "Number.isSafeInteger(payload.count)",
            "payload.count < 0",
            "if (count === 0)",
            "count > 99 ? '99+' : String(count)",
            "'Notifications, ' + String(count) + ' unread'",
        ] {
            assert!(script.contains(anchor), "missing badge guard: {anchor}");
        }
        assert!(!script.contains("Math.min"));
    }

    #[test]
    fn notification_badge_runtime_clears_stale_and_never_injects_payload_html() {
        let script = notification_badge_runtime(true, "/rankings");
        assert!(script.contains("setUnavailable();\n    if (document.hidden) return;"));
        assert!(script.contains("generation !== requestGeneration"));
        assert!(script.contains(
            "if (generation === requestGeneration && !document.hidden) setUnavailable();"
        ));
        assert!(script.contains("if (count === null) return;"));
        assert!(script.contains("if (requestController) requestController.abort();"));
        assert!(script.contains("document.addEventListener('visibilitychange'"));
        assert!(script.contains("badge.textContent = '';"));
        assert!(script.contains("badge.textContent = count > 99 ? '99+' : String(count);"));
        assert!(!script.contains("innerHTML"));
        assert!(!script.contains("insertAdjacentHTML"));
        assert!(!script.contains("document.write"));
    }

    #[test]
    fn offline_public_cache_shell_never_receives_authenticated_badge_runtime() {
        assert_eq!(notification_badge_runtime(false, "/offline"), "");
        assert_eq!(notification_badge_runtime(true, "/offline"), "");
        assert!(!notification_badge_runtime(true, "/offline").contains("fetch("));
        assert_eq!(notification_badge_runtime(true, "/notifications"), "");
        assert!(!notification_badge_runtime(true, "/notifications").contains("fetch("));

        let script = notification_badge_runtime(true, "/rankings");
        assert!(script.contains("target.setAttribute('aria-label', 'Notifications');"));
        assert!(script.contains(
            "target.setAttribute('aria-label', 'Notifications, ' + String(count) + ' unread');"
        ));
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

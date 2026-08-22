//! Dioxus SSR rendering for the admin BFF.
//!
//! The HTTP request is parsed into a `PageContext` (path, query, user) and
//! dispatched to the appropriate `rsx!` page from `epsx_dioxus_ui::pages`.
//! The HTML is wrapped in the EPSX design-system page shell so the visuals
//! match the Next.js admin 1:1.
//!
//! Wave 3a Track C — the rendered page body is wrapped in
//! `AdminLayout::Auth` (from `epsx_dioxus_ui::layout::shell`) so the
//! admin chrome (`Header` + `Sidebar` + `AdminFooter`) is owned by the
//! layout, not by each page. Pages are body-only after this wave.

use axum::{
    extract::{Request, State},
    http::{header, HeaderMap, HeaderValue},
    response::{IntoResponse, Response},
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use epsx_bff::session::AccessVerification;
use epsx_client::RequestContext;
use epsx_dioxus_ui::layout::shell::{AdminLayout, ServerUser};
use epsx_dioxus_ui::pages::admin_pages::analytics::{
    ADMIN_ANALYTICS_DATA_PARAM, ADMIN_ANALYTICS_EMPTY, ADMIN_ANALYTICS_FORBIDDEN,
    ADMIN_ANALYTICS_MALFORMED, ADMIN_ANALYTICS_READY, ADMIN_ANALYTICS_STATE_PARAM,
    ADMIN_ANALYTICS_UNAVAILABLE,
};
use epsx_dioxus_ui::pages::admin_pages::audit_log::{
    ADMIN_AUDIT_CATEGORY_PARAM, ADMIN_AUDIT_CURSOR_PARAM, ADMIN_AUDIT_DATA_PARAM,
    ADMIN_AUDIT_EMPTY, ADMIN_AUDIT_FORBIDDEN, ADMIN_AUDIT_MALFORMED, ADMIN_AUDIT_READY,
    ADMIN_AUDIT_STATE_PARAM, ADMIN_AUDIT_UNAVAILABLE,
};
use epsx_dioxus_ui::pages::admin_pages::chat::{
    ADMIN_CHAT_DETAIL_DATA_PARAM, ADMIN_CHAT_DETAIL_STATE_PARAM, ADMIN_CHAT_EMPTY,
    ADMIN_CHAT_FORBIDDEN, ADMIN_CHAT_LIST_DATA_PARAM, ADMIN_CHAT_LIST_STATE_PARAM,
    ADMIN_CHAT_MALFORMED, ADMIN_CHAT_READY, ADMIN_CHAT_UNAVAILABLE,
};
use epsx_dioxus_ui::pages::admin_pages::dashboard::{
    ADMIN_DASHBOARD_USER_STATUS_FORBIDDEN, ADMIN_DASHBOARD_USER_STATUS_MALFORMED,
    ADMIN_DASHBOARD_USER_STATUS_PARAM, ADMIN_DASHBOARD_USER_STATUS_READY,
    ADMIN_DASHBOARD_USER_STATUS_STATE_PARAM, ADMIN_DASHBOARD_USER_STATUS_UNAVAILABLE,
};
use epsx_dioxus_ui::pages::admin_pages::developer_portal::{
    decode_admin_developer_secret_once, AdminDeveloperSecretOnceProjection,
    ADMIN_DEVELOPER_CREATE_CONFLICT, ADMIN_DEVELOPER_CREATE_CREATED,
    ADMIN_DEVELOPER_CREATE_DATA_PARAM, ADMIN_DEVELOPER_CREATE_FORBIDDEN,
    ADMIN_DEVELOPER_CREATE_FORM, ADMIN_DEVELOPER_CREATE_MALFORMED,
    ADMIN_DEVELOPER_CREATE_STATE_PARAM, ADMIN_DEVELOPER_CREATE_UNAVAILABLE,
    ADMIN_DEVELOPER_DATA_PARAM, ADMIN_DEVELOPER_EMPTY, ADMIN_DEVELOPER_FORBIDDEN,
    ADMIN_DEVELOPER_MALFORMED, ADMIN_DEVELOPER_READY, ADMIN_DEVELOPER_STATE_PARAM,
    ADMIN_DEVELOPER_UNAVAILABLE,
};
use epsx_dioxus_ui::pages::admin_pages::media::{
    ADMIN_MEDIA_BUCKET_PARAM, ADMIN_MEDIA_DATA_PARAM, ADMIN_MEDIA_EMPTY, ADMIN_MEDIA_FORBIDDEN,
    ADMIN_MEDIA_MALFORMED, ADMIN_MEDIA_MUTATION_COMMITTED, ADMIN_MEDIA_MUTATION_DATA_PARAM,
    ADMIN_MEDIA_MUTATION_ERROR_PARAM, ADMIN_MEDIA_MUTATION_STATE_PARAM, ADMIN_MEDIA_READY,
    ADMIN_MEDIA_STATE_PARAM, ADMIN_MEDIA_UNAVAILABLE,
};
use epsx_dioxus_ui::pages::admin_pages::news::{
    ADMIN_NEWS_DATA_PARAM, ADMIN_NEWS_EDITOR_DATA_PARAM, ADMIN_NEWS_EDITOR_FORM,
    ADMIN_NEWS_EDITOR_READY, ADMIN_NEWS_EDITOR_STATE_PARAM, ADMIN_NEWS_EMPTY, ADMIN_NEWS_FORBIDDEN,
    ADMIN_NEWS_IMAGE_COMMITTED, ADMIN_NEWS_IMAGE_STATE_PARAM, ADMIN_NEWS_IMAGE_URL_PARAM,
    ADMIN_NEWS_MALFORMED, ADMIN_NEWS_MUTATION_CONFLICT, ADMIN_NEWS_MUTATION_ERROR_PARAM,
    ADMIN_NEWS_MUTATION_FORBIDDEN, ADMIN_NEWS_MUTATION_MALFORMED, ADMIN_NEWS_MUTATION_STATE_PARAM,
    ADMIN_NEWS_MUTATION_UNAVAILABLE, ADMIN_NEWS_PAGE_PARAM, ADMIN_NEWS_READY,
    ADMIN_NEWS_STATE_PARAM, ADMIN_NEWS_STATUS_PARAM, ADMIN_NEWS_UNAVAILABLE,
};
use epsx_dioxus_ui::pages::admin_pages::notifications::{
    decode_admin_notification_create_result, ADMIN_NOTIFICATIONS_DATA_PARAM,
    ADMIN_NOTIFICATIONS_EMPTY, ADMIN_NOTIFICATIONS_FORBIDDEN, ADMIN_NOTIFICATIONS_MALFORMED,
    ADMIN_NOTIFICATIONS_MUTATION_PARAM, ADMIN_NOTIFICATIONS_PAGE_PARAM, ADMIN_NOTIFICATIONS_READY,
    ADMIN_NOTIFICATIONS_SEND_ACCEPTED, ADMIN_NOTIFICATIONS_SEND_ERROR,
    ADMIN_NOTIFICATIONS_SEND_STATE_PARAM, ADMIN_NOTIFICATIONS_STATE_PARAM,
    ADMIN_NOTIFICATIONS_UNAVAILABLE, ADMIN_NOTIFICATION_CREATE_CONFLICT,
    ADMIN_NOTIFICATION_CREATE_DATA_PARAM, ADMIN_NOTIFICATION_CREATE_FAILED,
    ADMIN_NOTIFICATION_CREATE_FORBIDDEN, ADMIN_NOTIFICATION_CREATE_FORM,
    ADMIN_NOTIFICATION_CREATE_INVALID, ADMIN_NOTIFICATION_CREATE_MALFORMED,
    ADMIN_NOTIFICATION_CREATE_PENDING, ADMIN_NOTIFICATION_CREATE_SENT,
    ADMIN_NOTIFICATION_CREATE_STATE_PARAM, ADMIN_NOTIFICATION_CREATE_UNAVAILABLE,
    ADMIN_NOTIFICATION_METRICS_DATA_PARAM, ADMIN_NOTIFICATION_METRICS_STATE_PARAM,
};
use epsx_dioxus_ui::pages::admin_pages::payments::{
    decode_admin_payment_intent_list, AdminPaymentLinkListProjection,
    AdminPaymentUserAccessProjection, AdminPaymentUserAccessQuery, ADMIN_PAYMENTS_DATA_PARAM,
    ADMIN_PAYMENTS_EMPTY, ADMIN_PAYMENTS_LIMIT_PARAM, ADMIN_PAYMENTS_MALFORMED,
    ADMIN_PAYMENTS_OFFSET_PARAM, ADMIN_PAYMENTS_PAYER_PARAM, ADMIN_PAYMENTS_READY,
    ADMIN_PAYMENTS_STATE_PARAM, ADMIN_PAYMENTS_STATUS_PARAM, ADMIN_PAYMENTS_TAB_PARAM,
    ADMIN_PAYMENTS_UNAVAILABLE, ADMIN_PAYMENT_LINKS_DATA_PARAM, ADMIN_PAYMENT_LINKS_EMPTY,
    ADMIN_PAYMENT_LINKS_FORBIDDEN, ADMIN_PAYMENT_LINKS_MALFORMED, ADMIN_PAYMENT_LINKS_READY,
    ADMIN_PAYMENT_LINKS_STATE_PARAM, ADMIN_PAYMENT_LINKS_UNAVAILABLE, ADMIN_PAYMENT_MUTATION_PARAM,
    ADMIN_PAYMENT_USER_ACCESS_DATA_PARAM, ADMIN_PAYMENT_USER_ACCESS_EMPTY,
    ADMIN_PAYMENT_USER_ACCESS_FORBIDDEN, ADMIN_PAYMENT_USER_ACCESS_MALFORMED,
    ADMIN_PAYMENT_USER_ACCESS_READY, ADMIN_PAYMENT_USER_ACCESS_STATE_PARAM,
    ADMIN_PAYMENT_USER_ACCESS_UNAVAILABLE,
};
use epsx_dioxus_ui::pages::admin_pages::settings::{
    AdminSettingsQuery, ADMIN_SETTINGS_DATA_PARAM, ADMIN_SETTINGS_EMPTY, ADMIN_SETTINGS_FORBIDDEN,
    ADMIN_SETTINGS_MALFORMED, ADMIN_SETTINGS_MUTATION_PARAM, ADMIN_SETTINGS_READY,
    ADMIN_SETTINGS_STATE_PARAM, ADMIN_SETTINGS_TAB_PARAM, ADMIN_SETTINGS_UNAVAILABLE,
};
use epsx_dioxus_ui::pages::admin_pages::wallet_access::{
    AdminAccessProjection, ADMIN_ACCESS_DATA_PARAM, ADMIN_ACCESS_FORBIDDEN, ADMIN_ACCESS_MALFORMED,
    ADMIN_ACCESS_READY, ADMIN_ACCESS_STATE_PARAM, ADMIN_ACCESS_UNAVAILABLE,
};
use epsx_dioxus_ui::pages::admin_pages::wallet_credits::{
    AdminCreditStatsProjection, ADMIN_CREDITS_DATA_PARAM, ADMIN_CREDITS_FORBIDDEN,
    ADMIN_CREDITS_MALFORMED, ADMIN_CREDITS_READY, ADMIN_CREDITS_STATE_PARAM,
    ADMIN_CREDITS_UNAVAILABLE,
};
use epsx_dioxus_ui::pages::admin_pages::wallet_plans::{
    AdminPlanListProjection, AdminPlanProjection, ADMIN_PLANS_DATA_PARAM, ADMIN_PLANS_EMPTY,
    ADMIN_PLANS_FORBIDDEN, ADMIN_PLANS_MALFORMED, ADMIN_PLANS_READY, ADMIN_PLANS_STATE_PARAM,
    ADMIN_PLANS_UNAVAILABLE, ADMIN_PLAN_DETAIL_DATA_PARAM, ADMIN_PLAN_DETAIL_STATE_PARAM,
};
use epsx_dioxus_ui::pages::admin_pages::wallet_wallets::{
    AdminWalletDetailProjection, AdminWalletListQuery, AdminWalletStatsSummary,
    ADMIN_WALLET_DETAIL_DATA_PARAM, ADMIN_WALLET_DETAIL_FORBIDDEN, ADMIN_WALLET_DETAIL_MALFORMED,
    ADMIN_WALLET_DETAIL_READY, ADMIN_WALLET_DETAIL_STATE_PARAM, ADMIN_WALLET_DETAIL_UNAVAILABLE,
    ADMIN_WALLET_DISABLE_CONFLICT, ADMIN_WALLET_DISABLE_FORBIDDEN, ADMIN_WALLET_DISABLE_FORM,
    ADMIN_WALLET_DISABLE_MALFORMED, ADMIN_WALLET_DISABLE_STATE_PARAM, ADMIN_WALLET_DISABLE_SUCCESS,
    ADMIN_WALLET_DISABLE_UNAVAILABLE, ADMIN_WALLET_LIST_DATA_PARAM, ADMIN_WALLET_LIST_EMPTY,
    ADMIN_WALLET_LIST_FORBIDDEN, ADMIN_WALLET_LIST_MALFORMED, ADMIN_WALLET_LIST_READY,
    ADMIN_WALLET_LIST_STATE_PARAM, ADMIN_WALLET_LIST_UNAVAILABLE, ADMIN_WALLET_STATS_DATA_PARAM,
    ADMIN_WALLET_STATS_FORBIDDEN, ADMIN_WALLET_STATS_MALFORMED, ADMIN_WALLET_STATS_READY,
    ADMIN_WALLET_STATS_STATE_PARAM, ADMIN_WALLET_STATS_UNAVAILABLE,
};
use epsx_dioxus_ui::pages::analytics::{
    AnalyticsFilters, AnalyticsQueryState, AnalyticsResponse, ANALYTICS_DATA_PARAM,
    ANALYTICS_FILTERS_DATA_PARAM, ANALYTICS_FILTERS_STATE_PARAM, ANALYTICS_QUERY_PARAM,
    ANALYTICS_STATE_PARAM, ANALYTICS_WATCHLIST_DATA_PARAM, ANALYTICS_WATCHLIST_STATE_PARAM,
};
use epsx_dioxus_ui::pages::{admin_pages, render_page, PageContext, PageStatus};
use std::collections::HashMap;

use super::analytics_admin_adapter::{load_admin_analytics, AdminAnalyticsLoad};
use super::audit_log_adapter::{load_admin_audit, AdminAuditLoad, AdminAuditQuery};
use super::auth;
use super::chat_admin_adapter::{
    load_admin_chat, load_admin_chat_detail, AdminChatDetailLoad, AdminChatListLoad, AdminChatQuery,
};
use super::commerce_adapter::{
    load_access, load_credit_stats, load_payment_links, load_payment_user_access, load_plan_detail,
    load_plans, load_wallet_access, load_wallet_detail, load_wallet_list, load_wallet_stats,
    AdminCommerceLoad as CommerceLoad,
};
use super::dashboard_user_status_adapter::{
    load_admin_dashboard_user_status, AdminDashboardUserStatusLoad, AdminDashboardUserStatusQuery,
};
use super::developer_portal_adapter::{load_admin_developer_portal, AdminDeveloperLoad};
use super::media_adapter::{load_admin_media, AdminMediaLoad, AdminMediaQuery};
use super::news_adapter::{
    load_admin_news, load_admin_news_editor, AdminNewsEditorLoad, AdminNewsLoad, AdminNewsQuery,
};
use super::notification_admin_adapter::{
    load_admin_notification_metrics, load_admin_notifications, AdminNotificationLoad,
    AdminNotificationMetricsLoad, AdminNotificationQuery,
};
use super::settings_admin_adapter::{load_admin_settings, AdminSettingsLoad};
#[cfg(test)]
use super::wallet_stats_adapter::AdminWalletStatsLoad;
use super::AppState;

const ANALYTICS_RANKINGS_PATH: &str = "/api/analytics/rankings";
const ANALYTICS_FILTERS_PATH: &str = "/api/analytics/filters";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RankingAnalyticsLoadError {
    Unavailable,
    Malformed,
}

fn record_admin_dashboard_user_status_load(
    params: &mut HashMap<String, String>,
    load: AdminDashboardUserStatusLoad,
) {
    params.remove(ADMIN_DASHBOARD_USER_STATUS_PARAM);
    let state = match load {
        AdminDashboardUserStatusLoad::Ready(payload) => {
            params.insert(
                ADMIN_DASHBOARD_USER_STATUS_PARAM.to_string(),
                serde_json::to_string(&payload)
                    .expect("the typed admin dashboard projection is serializable"),
            );
            ADMIN_DASHBOARD_USER_STATUS_READY
        }
        AdminDashboardUserStatusLoad::Forbidden => ADMIN_DASHBOARD_USER_STATUS_FORBIDDEN,
        AdminDashboardUserStatusLoad::Unavailable => ADMIN_DASHBOARD_USER_STATUS_UNAVAILABLE,
        AdminDashboardUserStatusLoad::Malformed => ADMIN_DASHBOARD_USER_STATUS_MALFORMED,
    };
    params.insert(
        ADMIN_DASHBOARD_USER_STATUS_STATE_PARAM.to_string(),
        state.to_string(),
    );
}

fn record_admin_chat_list_load(params: &mut HashMap<String, String>, load: AdminChatListLoad) {
    params.remove(ADMIN_CHAT_LIST_DATA_PARAM);
    let state = match load {
        AdminChatListLoad::Ready(payload) => {
            params.insert(
                ADMIN_CHAT_LIST_DATA_PARAM.to_string(),
                serde_json::to_string(&payload)
                    .expect("the typed admin-chat list projection is serializable"),
            );
            ADMIN_CHAT_READY
        }
        AdminChatListLoad::Empty(payload) => {
            params.insert(
                ADMIN_CHAT_LIST_DATA_PARAM.to_string(),
                serde_json::to_string(&payload)
                    .expect("the typed empty admin-chat list projection is serializable"),
            );
            ADMIN_CHAT_EMPTY
        }
        AdminChatListLoad::Forbidden => ADMIN_CHAT_FORBIDDEN,
        AdminChatListLoad::Unavailable => ADMIN_CHAT_UNAVAILABLE,
        AdminChatListLoad::Malformed => ADMIN_CHAT_MALFORMED,
    };
    params.insert(ADMIN_CHAT_LIST_STATE_PARAM.to_string(), state.to_string());
}

fn record_admin_chat_detail_load(params: &mut HashMap<String, String>, load: AdminChatDetailLoad) {
    params.remove(ADMIN_CHAT_DETAIL_DATA_PARAM);
    let state = match load {
        AdminChatDetailLoad::Ready(payload) => {
            params.insert(
                ADMIN_CHAT_DETAIL_DATA_PARAM.to_string(),
                serde_json::to_string(&payload)
                    .expect("the typed admin-chat detail projection is serializable"),
            );
            ADMIN_CHAT_READY
        }
        AdminChatDetailLoad::Forbidden => ADMIN_CHAT_FORBIDDEN,
        AdminChatDetailLoad::Unavailable => ADMIN_CHAT_UNAVAILABLE,
        AdminChatDetailLoad::Malformed => ADMIN_CHAT_MALFORMED,
    };
    params.insert(ADMIN_CHAT_DETAIL_STATE_PARAM.to_string(), state.to_string());
}

fn record_admin_media_load(
    params: &mut HashMap<String, String>,
    query: &AdminMediaQuery,
    load: AdminMediaLoad,
) {
    params.remove(ADMIN_MEDIA_DATA_PARAM);
    params.insert(
        ADMIN_MEDIA_BUCKET_PARAM.to_string(),
        query.bucket.to_string(),
    );

    let state = match load {
        AdminMediaLoad::Ready(payload) => {
            params.insert(
                ADMIN_MEDIA_DATA_PARAM.to_string(),
                serde_json::to_string(&payload)
                    .expect("the typed admin-media projection is serializable"),
            );
            ADMIN_MEDIA_READY
        }
        AdminMediaLoad::Empty(payload) => {
            params.insert(
                ADMIN_MEDIA_DATA_PARAM.to_string(),
                serde_json::to_string(&payload)
                    .expect("the typed empty admin-media projection is serializable"),
            );
            ADMIN_MEDIA_EMPTY
        }
        AdminMediaLoad::Forbidden => ADMIN_MEDIA_FORBIDDEN,
        AdminMediaLoad::Unavailable => ADMIN_MEDIA_UNAVAILABLE,
        AdminMediaLoad::Malformed => ADMIN_MEDIA_MALFORMED,
    };
    params.insert(ADMIN_MEDIA_STATE_PARAM.to_string(), state.to_string());
}

type AdminMediaMutationQuery = (String, Option<String>, Option<i64>, bool);
type ParsedAdminMediaQuery = (AdminMediaQuery, Option<AdminMediaMutationQuery>);

fn parse_admin_media_query(raw_query: &str) -> Result<ParsedAdminMediaQuery, ()> {
    let mut bucket = None;
    let mut mutation = None;
    let mut key = None;
    let mut size = None;
    let mut deleted = None;
    for (name, value) in url::form_urlencoded::parse(raw_query.as_bytes()) {
        match name.as_ref() {
            "bucket" if bucket.is_none() && matches!(value.as_ref(), "news" | "public") => {
                bucket = Some(value.into_owned());
            }
            "mutation"
                if mutation.is_none()
                    && matches!(
                        value.as_ref(),
                        "committed" | "conflict" | "forbidden" | "unavailable" | "malformed"
                    ) =>
            {
                mutation = Some(value.into_owned());
            }
            "key"
                if key.is_none()
                    && !value.is_empty()
                    && value.len() <= 1_024
                    && value.trim() == value
                    && !value.chars().any(char::is_control) =>
            {
                key = Some(value.into_owned());
            }
            "size" if size.is_none() => {
                let parsed = value.parse::<i64>().map_err(|_| ())?;
                if parsed < 0 {
                    return Err(());
                }
                size = Some(parsed);
            }
            "deleted" if deleted.is_none() => {
                deleted = Some(match value.as_ref() {
                    "true" => true,
                    "false" => false,
                    _ => return Err(()),
                });
            }
            _ => return Err(()),
        }
    }
    let bucket_name = bucket.unwrap_or_else(|| "news".to_string());
    let inventory_query = AdminMediaQuery::from_raw(&format!("bucket={bucket_name}"))?;
    match mutation {
        None if key.is_none() && size.is_none() && deleted.is_none() => Ok((inventory_query, None)),
        Some(state) if state == "committed" => {
            let key = key.ok_or(())?;
            let deleted = deleted.ok_or(())?;
            Ok((inventory_query, Some((state, Some(key), size, deleted))))
        }
        Some(state) if key.is_none() && size.is_none() && deleted.is_none() => {
            Ok((inventory_query, Some((state, None, None, false))))
        }
        _ => Err(()),
    }
}

fn record_admin_media_mutation_query(
    params: &mut HashMap<String, String>,
    mutation: Option<(String, Option<String>, Option<i64>, bool)>,
) {
    params.remove(ADMIN_MEDIA_MUTATION_DATA_PARAM);
    params.remove(ADMIN_MEDIA_MUTATION_ERROR_PARAM);
    let Some((state, key, size, deleted)) = mutation else {
        return;
    };
    params.insert(ADMIN_MEDIA_MUTATION_STATE_PARAM.to_string(), state.clone());
    if state == ADMIN_MEDIA_MUTATION_COMMITTED {
        if let Some(key) = key {
            params.insert(
                ADMIN_MEDIA_MUTATION_DATA_PARAM.to_string(),
                serde_json::to_string(&serde_json::json!({
                    "bucket": params.get(ADMIN_MEDIA_BUCKET_PARAM).cloned().unwrap_or_else(|| "news".to_string()),
                    "key": key,
                    "size": size,
                    "deleted": deleted,
                }))
                .expect("the bounded media mutation projection is serializable"),
            );
        }
    }
}

#[cfg(test)]
fn record_admin_wallet_stats_load(
    params: &mut HashMap<String, String>,
    load: AdminWalletStatsLoad,
) {
    params.remove(ADMIN_WALLET_STATS_DATA_PARAM);
    let state = match load {
        AdminWalletStatsLoad::Ready(payload) => {
            params.insert(
                ADMIN_WALLET_STATS_DATA_PARAM.to_string(),
                serde_json::to_string(&payload)
                    .expect("the typed admin wallet-stats projection is serializable"),
            );
            ADMIN_WALLET_STATS_READY
        }
        AdminWalletStatsLoad::Forbidden => ADMIN_WALLET_STATS_FORBIDDEN,
        AdminWalletStatsLoad::Unavailable => ADMIN_WALLET_STATS_UNAVAILABLE,
        AdminWalletStatsLoad::Malformed => ADMIN_WALLET_STATS_MALFORMED,
    };
    params.insert(
        ADMIN_WALLET_STATS_STATE_PARAM.to_string(),
        state.to_string(),
    );
}

fn record_admin_news_load(
    params: &mut HashMap<String, String>,
    query: &AdminNewsQuery,
    load: AdminNewsLoad,
) {
    params.remove(ADMIN_NEWS_DATA_PARAM);
    params.insert(ADMIN_NEWS_PAGE_PARAM.to_string(), query.page.to_string());
    params.insert(
        ADMIN_NEWS_STATUS_PARAM.to_string(),
        query.status.to_string(),
    );

    let state = match load {
        AdminNewsLoad::Ready(payload) => {
            params.insert(
                ADMIN_NEWS_DATA_PARAM.to_string(),
                serde_json::to_string(&payload)
                    .expect("the typed admin-news projection is serializable"),
            );
            ADMIN_NEWS_READY
        }
        AdminNewsLoad::Empty(payload) => {
            params.insert(
                ADMIN_NEWS_DATA_PARAM.to_string(),
                serde_json::to_string(&payload)
                    .expect("the typed empty admin-news projection is serializable"),
            );
            ADMIN_NEWS_EMPTY
        }
        AdminNewsLoad::Forbidden => ADMIN_NEWS_FORBIDDEN,
        AdminNewsLoad::Unavailable => ADMIN_NEWS_UNAVAILABLE,
        AdminNewsLoad::Malformed => ADMIN_NEWS_MALFORMED,
    };
    params.insert(ADMIN_NEWS_STATE_PARAM.to_string(), state.to_string());
}

fn record_admin_news_editor_load(
    params: &mut HashMap<String, String>,
    id: &str,
    load: AdminNewsEditorLoad,
) {
    params.remove(ADMIN_NEWS_EDITOR_DATA_PARAM);
    match load {
        AdminNewsEditorLoad::Ready(projection) => {
            if projection.id == id {
                params.insert(
                    ADMIN_NEWS_EDITOR_DATA_PARAM.to_string(),
                    serde_json::to_string(&projection)
                        .expect("the typed admin-news editor projection is serializable"),
                );
                params.insert(
                    ADMIN_NEWS_EDITOR_STATE_PARAM.to_string(),
                    ADMIN_NEWS_EDITOR_READY.to_string(),
                );
            } else {
                params.insert(
                    ADMIN_NEWS_MUTATION_STATE_PARAM.to_string(),
                    ADMIN_NEWS_MUTATION_MALFORMED.to_string(),
                );
            }
        }
        AdminNewsEditorLoad::Forbidden => {
            params.insert(
                ADMIN_NEWS_MUTATION_STATE_PARAM.to_string(),
                ADMIN_NEWS_MUTATION_FORBIDDEN.to_string(),
            );
        }
        AdminNewsEditorLoad::Unavailable => {
            params.insert(
                ADMIN_NEWS_MUTATION_STATE_PARAM.to_string(),
                ADMIN_NEWS_MUTATION_UNAVAILABLE.to_string(),
            );
        }
        AdminNewsEditorLoad::Malformed => {
            params.insert(
                ADMIN_NEWS_MUTATION_STATE_PARAM.to_string(),
                ADMIN_NEWS_MUTATION_MALFORMED.to_string(),
            );
        }
    }
}

fn news_mutation_query(query: &str) -> Option<&'static str> {
    let mut value = None;
    for (key, candidate) in url::form_urlencoded::parse(query.as_bytes()) {
        if key != "mutation" || value.is_some() {
            return None;
        }
        value = Some(match candidate.as_ref() {
            "conflict" => ADMIN_NEWS_MUTATION_CONFLICT,
            "forbidden" => ADMIN_NEWS_MUTATION_FORBIDDEN,
            "unavailable" => ADMIN_NEWS_MUTATION_UNAVAILABLE,
            "malformed" => ADMIN_NEWS_MUTATION_MALFORMED,
            _ => return None,
        });
    }
    value
}

fn news_inventory_mutation_query(query: &str) -> Option<&'static str> {
    let mut value = None;
    for (key, candidate) in url::form_urlencoded::parse(query.as_bytes()) {
        match key.as_ref() {
            "mutation" if value.is_none() => {
                value = Some(match candidate.as_ref() {
                    "committed" => "committed",
                    "conflict" => "conflict",
                    "forbidden" => "forbidden",
                    "unavailable" => "unavailable",
                    "malformed" => "malformed",
                    _ => return None,
                });
            }
            "page" | "status" => {}
            _ => return None,
        }
    }
    value
}

fn news_image_url_query(query: &str) -> Option<String> {
    let mut image_url = None;
    for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
        if key != ADMIN_NEWS_IMAGE_URL_PARAM || image_url.is_some() {
            return None;
        }
        let value = value.into_owned();
        if value.len() > 2_048
            || value.chars().any(char::is_control)
            || !reqwest::Url::parse(&value).ok().is_some_and(|url| {
                matches!(url.scheme(), "http" | "https") && url.host_str().is_some()
            })
        {
            return None;
        }
        image_url = Some(value);
    }
    image_url
}

fn chat_mutation_query(query: &str) -> Option<&'static str> {
    let mut value = None;
    for (key, candidate) in url::form_urlencoded::parse(query.as_bytes()) {
        if key != "mutation" || value.is_some() {
            return None;
        }
        value = Some(match candidate.as_ref() {
            "success" => "success",
            "conflict" => "conflict",
            "forbidden" => "forbidden",
            "unavailable" => "unavailable",
            "malformed" => "malformed",
            _ => return None,
        });
    }
    value
}

fn wallet_plan_mutation_query(query: &str) -> Option<&'static str> {
    let mut value = None;
    for (key, candidate) in url::form_urlencoded::parse(query.as_bytes()) {
        if key != "mutation" || value.is_some() {
            return None;
        }
        value = Some(match candidate.as_ref() {
            "success" => "success",
            "conflict" => "conflict",
            "forbidden" => "forbidden",
            "unavailable" => "unavailable",
            "malformed" => "malformed",
            _ => return None,
        });
    }
    value
}

fn notification_mutation_query(query: &str) -> Option<&'static str> {
    let mut value = None;
    for (key, candidate) in url::form_urlencoded::parse(query.as_bytes()) {
        if key != ADMIN_NOTIFICATIONS_MUTATION_PARAM || value.is_some() {
            return None;
        }
        value = Some(match candidate.as_ref() {
            "committed" => "committed",
            "conflict" => "conflict",
            "forbidden" => "forbidden",
            "unavailable" => "unavailable",
            "malformed" => "malformed",
            _ => return None,
        });
    }
    value
}

fn record_admin_notification_load(
    params: &mut HashMap<String, String>,
    query: &AdminNotificationQuery,
    load: AdminNotificationLoad,
) {
    params.remove(ADMIN_NOTIFICATIONS_DATA_PARAM);
    params.insert(
        ADMIN_NOTIFICATIONS_PAGE_PARAM.to_string(),
        query.page.to_string(),
    );

    let state = match load {
        AdminNotificationLoad::Ready(payload) => {
            params.insert(
                ADMIN_NOTIFICATIONS_DATA_PARAM.to_string(),
                serde_json::to_string(&payload)
                    .expect("the typed admin-notification projection is serializable"),
            );
            ADMIN_NOTIFICATIONS_READY
        }
        AdminNotificationLoad::Empty(payload) => {
            params.insert(
                ADMIN_NOTIFICATIONS_DATA_PARAM.to_string(),
                serde_json::to_string(&payload)
                    .expect("the typed empty admin-notification projection is serializable"),
            );
            ADMIN_NOTIFICATIONS_EMPTY
        }
        AdminNotificationLoad::Forbidden => ADMIN_NOTIFICATIONS_FORBIDDEN,
        AdminNotificationLoad::Unavailable => ADMIN_NOTIFICATIONS_UNAVAILABLE,
        AdminNotificationLoad::Malformed => ADMIN_NOTIFICATIONS_MALFORMED,
    };
    params.insert(
        ADMIN_NOTIFICATIONS_STATE_PARAM.to_string(),
        state.to_string(),
    );
}

fn record_admin_notification_metrics_load(
    params: &mut HashMap<String, String>,
    load: AdminNotificationMetricsLoad,
) {
    params.remove(ADMIN_NOTIFICATION_METRICS_DATA_PARAM);
    let state = match load {
        AdminNotificationMetricsLoad::Ready(payload) => {
            params.insert(
                ADMIN_NOTIFICATION_METRICS_DATA_PARAM.to_string(),
                serde_json::to_string(&payload)
                    .expect("the typed admin-notification metrics projection is serializable"),
            );
            ADMIN_NOTIFICATIONS_READY
        }
        AdminNotificationMetricsLoad::Forbidden => ADMIN_NOTIFICATIONS_FORBIDDEN,
        AdminNotificationMetricsLoad::Unavailable => ADMIN_NOTIFICATIONS_UNAVAILABLE,
        AdminNotificationMetricsLoad::Malformed => ADMIN_NOTIFICATIONS_MALFORMED,
    };
    params.insert(
        ADMIN_NOTIFICATION_METRICS_STATE_PARAM.to_string(),
        state.to_string(),
    );
}

fn record_admin_audit_load(
    params: &mut HashMap<String, String>,
    query: &AdminAuditQuery,
    load: AdminAuditLoad,
) {
    params.remove(ADMIN_AUDIT_DATA_PARAM);
    if let Some(category) = &query.category {
        params.insert(ADMIN_AUDIT_CATEGORY_PARAM.to_string(), category.clone());
    } else {
        params.remove(ADMIN_AUDIT_CATEGORY_PARAM);
    }
    if let Some(cursor) = &query.cursor {
        params.insert(ADMIN_AUDIT_CURSOR_PARAM.to_string(), cursor.clone());
    } else {
        params.remove(ADMIN_AUDIT_CURSOR_PARAM);
    }

    let state = match load {
        AdminAuditLoad::Ready(payload) => {
            params.insert(
                ADMIN_AUDIT_DATA_PARAM.to_string(),
                serde_json::to_string(&payload)
                    .expect("the typed admin-audit projection is serializable"),
            );
            ADMIN_AUDIT_READY
        }
        AdminAuditLoad::Empty(payload) => {
            params.insert(
                ADMIN_AUDIT_DATA_PARAM.to_string(),
                serde_json::to_string(&payload)
                    .expect("the typed empty admin-audit projection is serializable"),
            );
            ADMIN_AUDIT_EMPTY
        }
        AdminAuditLoad::Forbidden => ADMIN_AUDIT_FORBIDDEN,
        AdminAuditLoad::Unavailable => ADMIN_AUDIT_UNAVAILABLE,
        AdminAuditLoad::Malformed => ADMIN_AUDIT_MALFORMED,
    };
    params.insert(ADMIN_AUDIT_STATE_PARAM.to_string(), state.to_string());
}

fn record_admin_settings_load(params: &mut HashMap<String, String>, load: AdminSettingsLoad) {
    params.remove(ADMIN_SETTINGS_DATA_PARAM);
    let state = match load {
        AdminSettingsLoad::Ready(projection) => {
            params.insert(
                ADMIN_SETTINGS_DATA_PARAM.to_string(),
                serde_json::to_string(&projection)
                    .expect("the typed admin-settings projection is serializable"),
            );
            ADMIN_SETTINGS_READY
        }
        AdminSettingsLoad::Empty => ADMIN_SETTINGS_EMPTY,
        AdminSettingsLoad::Forbidden => ADMIN_SETTINGS_FORBIDDEN,
        AdminSettingsLoad::Unavailable => ADMIN_SETTINGS_UNAVAILABLE,
        AdminSettingsLoad::Malformed => ADMIN_SETTINGS_MALFORMED,
    };
    params.insert(ADMIN_SETTINGS_STATE_PARAM.to_string(), state.to_string());
}

fn record_admin_analytics_load(params: &mut HashMap<String, String>, load: AdminAnalyticsLoad) {
    params.remove(ADMIN_ANALYTICS_DATA_PARAM);
    let state = match load {
        AdminAnalyticsLoad::Ready(snapshot) => {
            params.insert(
                ADMIN_ANALYTICS_DATA_PARAM.to_string(),
                serde_json::to_string(&snapshot)
                    .expect("the typed admin-analytics projection is serializable"),
            );
            ADMIN_ANALYTICS_READY
        }
        AdminAnalyticsLoad::Empty(snapshot) => {
            params.insert(
                ADMIN_ANALYTICS_DATA_PARAM.to_string(),
                serde_json::to_string(&snapshot)
                    .expect("the typed empty admin-analytics projection is serializable"),
            );
            ADMIN_ANALYTICS_EMPTY
        }
        AdminAnalyticsLoad::Forbidden => ADMIN_ANALYTICS_FORBIDDEN,
        AdminAnalyticsLoad::Unavailable => ADMIN_ANALYTICS_UNAVAILABLE,
        AdminAnalyticsLoad::Malformed => ADMIN_ANALYTICS_MALFORMED,
    };
    params.insert(ADMIN_ANALYTICS_STATE_PARAM.to_string(), state.to_string());
}

fn ranking_analytics_query(raw_query: &str) -> Result<String, ()> {
    if raw_query.is_empty() {
        return Ok(String::new());
    }
    let url =
        reqwest::Url::parse(&format!("https://admin.invalid/?{raw_query}")).map_err(|_| ())?;
    let mut seen = std::collections::HashSet::new();
    let mut normalized = url::form_urlencoded::Serializer::new(String::new());
    for (key, value) in url.query_pairs() {
        let key = key.as_ref();
        if !matches!(
            key,
            "page" | "limit" | "country" | "sector" | "sort_by" | "min_eps" | "min_growth"
        ) {
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
                if value.len() > 64 || value.chars().any(char::is_control) {
                    return Err(());
                }
                normalized.append_pair(key, &value);
            }
            "sort_by" => {
                if value.is_empty() || value.len() > 64 || value.chars().any(char::is_control) {
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

async fn load_ranking_analytics(
    client: &epsx_client::ServiceClient,
    normalized_query: &str,
    headers: &HeaderMap,
    verified_access_token: Option<&str>,
) -> Result<AnalyticsResponse, RankingAnalyticsLoadError> {
    let path = if normalized_query.is_empty() {
        ANALYTICS_RANKINGS_PATH.to_string()
    } else {
        format!("{ANALYTICS_RANKINGS_PATH}?{normalized_query}")
    };
    let mut context = RequestContext::from_headers(headers);
    context.auth_token = verified_access_token.map(str::to_owned);
    let value = client
        .get_with_ctx(&path, &context)
        .await
        .map_err(|error| {
            tracing::warn!("admin ranking analytics dependency unavailable: {error}");
            RankingAnalyticsLoadError::Unavailable
        })?;
    serde_json::from_value::<AnalyticsResponse>(value)
        .map_err(|error| {
            tracing::warn!("admin ranking analytics response malformed: {error}");
            RankingAnalyticsLoadError::Malformed
        })?
        .validated()
        .map_err(|_| RankingAnalyticsLoadError::Malformed)
}

async fn load_ranking_analytics_filters(
    client: &epsx_client::ServiceClient,
) -> Result<AnalyticsFilters, RankingAnalyticsLoadError> {
    let value = client
        .get_plain(ANALYTICS_FILTERS_PATH)
        .await
        .map_err(|error| {
            tracing::warn!("admin ranking filters dependency unavailable: {error}");
            RankingAnalyticsLoadError::Unavailable
        })?;
    serde_json::from_value::<AnalyticsFilters>(value)
        .map_err(|error| {
            tracing::warn!("admin ranking filters response malformed: {error}");
            RankingAnalyticsLoadError::Malformed
        })?
        .validated()
        .map_err(|_| RankingAnalyticsLoadError::Malformed)
}

fn record_ranking_analytics_query(params: &mut HashMap<String, String>, normalized_query: &str) {
    params.remove(ANALYTICS_QUERY_PARAM);
    if let Ok(query) = AnalyticsQueryState::from_normalized_query(normalized_query) {
        params.insert(
            ANALYTICS_QUERY_PARAM.to_string(),
            serde_json::to_string(&query).expect("validated ranking query is serializable"),
        );
    }
}

fn record_ranking_analytics_load(
    params: &mut HashMap<String, String>,
    outcome: Result<AnalyticsResponse, RankingAnalyticsLoadError>,
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
                    .expect("validated ranking response is serializable"),
            );
            state
        }
        Err(RankingAnalyticsLoadError::Malformed) => "malformed",
        Err(RankingAnalyticsLoadError::Unavailable) => "unavailable",
    };
    params.insert(ANALYTICS_STATE_PARAM.to_string(), state.to_string());
}

fn record_ranking_analytics_filters(
    params: &mut HashMap<String, String>,
    outcome: Result<AnalyticsFilters, RankingAnalyticsLoadError>,
) {
    params.remove(ANALYTICS_FILTERS_DATA_PARAM);
    let state = match outcome {
        Ok(filters) => {
            params.insert(
                ANALYTICS_FILTERS_DATA_PARAM.to_string(),
                serde_json::to_string(&filters)
                    .expect("validated ranking filters are serializable"),
            );
            "ready"
        }
        Err(RankingAnalyticsLoadError::Malformed) => "malformed",
        Err(RankingAnalyticsLoadError::Unavailable) => "unavailable",
    };
    params.insert(ANALYTICS_FILTERS_STATE_PARAM.to_string(), state.to_string());
}

fn record_admin_developer_load(params: &mut HashMap<String, String>, load: AdminDeveloperLoad) {
    params.remove(ADMIN_DEVELOPER_DATA_PARAM);
    let state = match load {
        AdminDeveloperLoad::Ready(projection) => {
            params.insert(
                ADMIN_DEVELOPER_DATA_PARAM.to_string(),
                serde_json::to_string(&projection)
                    .expect("the typed developer-portal projection is serializable"),
            );
            ADMIN_DEVELOPER_READY
        }
        AdminDeveloperLoad::Empty(projection) => {
            params.insert(
                ADMIN_DEVELOPER_DATA_PARAM.to_string(),
                serde_json::to_string(&projection)
                    .expect("the typed empty developer-portal projection is serializable"),
            );
            ADMIN_DEVELOPER_EMPTY
        }
        AdminDeveloperLoad::Forbidden => ADMIN_DEVELOPER_FORBIDDEN,
        AdminDeveloperLoad::Unavailable => ADMIN_DEVELOPER_UNAVAILABLE,
        AdminDeveloperLoad::Malformed => ADMIN_DEVELOPER_MALFORMED,
    };
    params.insert(ADMIN_DEVELOPER_STATE_PARAM.to_string(), state.to_string());
}

struct CommerceLoadContract<'a> {
    data_param: &'a str,
    state_param: &'a str,
    ready: &'a str,
    empty: Option<&'a str>,
    forbidden: &'a str,
    unavailable: &'a str,
    malformed: &'a str,
}

macro_rules! record_commerce_load {
    (
        $params:expr,
        $load:expr,
        $data_param:expr,
        $state_param:expr,
        $ready:expr,
        $empty:expr,
        $forbidden:expr,
        $unavailable:expr,
        $malformed:expr $(,)?
    ) => {
        record_commerce_load(
            $params,
            $load,
            CommerceLoadContract {
                data_param: $data_param,
                state_param: $state_param,
                ready: $ready,
                empty: $empty,
                forbidden: $forbidden,
                unavailable: $unavailable,
                malformed: $malformed,
            },
        )
    };
}

fn record_commerce_load<T: serde::Serialize>(
    params: &mut HashMap<String, String>,
    load: CommerceLoad<T>,
    contract: CommerceLoadContract<'_>,
) {
    let CommerceLoadContract {
        data_param,
        state_param,
        ready,
        empty,
        forbidden,
        unavailable,
        malformed,
    } = contract;
    params.remove(data_param);
    let state = match load {
        CommerceLoad::Ready(projection) => {
            params.insert(
                data_param.to_string(),
                serde_json::to_string(&projection)
                    .expect("the typed commerce projection is serializable"),
            );
            ready
        }
        CommerceLoad::Empty => empty.unwrap_or(malformed),
        CommerceLoad::Forbidden => forbidden,
        CommerceLoad::Unavailable => unavailable,
        CommerceLoad::Malformed => malformed,
    };
    params.insert(state_param.to_string(), state.to_string());
}

fn record_payment_links_load(
    params: &mut HashMap<String, String>,
    load: CommerceLoad<AdminPaymentLinkListProjection>,
) {
    if matches!(load, CommerceLoad::Empty) {
        let empty = AdminPaymentLinkListProjection {
            items: Vec::new(),
            total: 0,
            limit: 100,
            offset: 0,
        };
        params.insert(
            ADMIN_PAYMENT_LINKS_DATA_PARAM.to_string(),
            serde_json::to_string(&empty)
                .expect("the typed empty payment-link projection is serializable"),
        );
        params.insert(
            ADMIN_PAYMENT_LINKS_STATE_PARAM.to_string(),
            ADMIN_PAYMENT_LINKS_EMPTY.to_string(),
        );
        return;
    }
    record_commerce_load(
        params,
        load,
        CommerceLoadContract {
            data_param: ADMIN_PAYMENT_LINKS_DATA_PARAM,
            state_param: ADMIN_PAYMENT_LINKS_STATE_PARAM,
            ready: ADMIN_PAYMENT_LINKS_READY,
            empty: Some(ADMIN_PAYMENT_LINKS_EMPTY),
            forbidden: ADMIN_PAYMENT_LINKS_FORBIDDEN,
            unavailable: ADMIN_PAYMENT_LINKS_UNAVAILABLE,
            malformed: ADMIN_PAYMENT_LINKS_MALFORMED,
        },
    );
}

fn private_admin_html_response(status: axum::http::StatusCode, doc: String) -> Response {
    (
        status,
        [
            ("content-type", "text/html; charset=utf-8"),
            ("cache-control", "private, no-store"),
            ("vary", "Cookie, Authorization"),
        ],
        doc,
    )
        .into_response()
}

/// Consume the short-lived POST/redirect/GET notification feedback only when
/// the query state is paired with the HttpOnly cookie issued by the form
/// handler. Query input alone can never manufacture a success banner.
fn consume_notification_send_flash(
    headers: &HeaderMap,
    query: &str,
) -> (Option<&'static str>, bool) {
    let cookie_prefix = format!("{}=", super::ADMIN_NOTIFICATION_FLASH_COOKIE);
    let cookie_state = headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| {
            value.split(';').find_map(|part| {
                let part = part.trim();
                part.strip_prefix(&cookie_prefix)
                    .filter(|state| matches!(*state, "accepted" | "error"))
            })
        });
    let cookie_present = headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| {
            value
                .split(';')
                .any(|part| part.trim().strip_prefix(&cookie_prefix).is_some())
        });

    let mut query_state = None;
    let mut valid_query = true;
    for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
        if key == "send" {
            if query_state.is_some() || !matches!(value.as_ref(), "accepted" | "error") {
                valid_query = false;
            } else {
                query_state = Some(if value == "accepted" {
                    ADMIN_NOTIFICATIONS_SEND_ACCEPTED
                } else {
                    ADMIN_NOTIFICATIONS_SEND_ERROR
                });
            }
        }
    }

    let state = valid_query.then_some((query_state, cookie_state)).and_then(
        |(query_state, cookie_state)| match (query_state, cookie_state) {
            (Some(query_state), Some(cookie_state)) if query_state == cookie_state => {
                Some(query_state)
            }
            _ => None,
        },
    );
    (state, cookie_present)
}

fn record_payment_intent_load(
    params: &mut HashMap<String, String>,
    result: Result<serde_json::Value, ()>,
) {
    params.remove(ADMIN_PAYMENTS_DATA_PARAM);
    match result {
        Ok(value) => match decode_admin_payment_intent_list(value) {
            Some(payload) => {
                let state = if payload.items.is_empty() && payload.total == 0 {
                    ADMIN_PAYMENTS_EMPTY
                } else {
                    ADMIN_PAYMENTS_READY
                };
                params.insert(
                    ADMIN_PAYMENTS_DATA_PARAM.to_string(),
                    serde_json::to_string(&payload)
                        .expect("the typed payment-intent response is serializable"),
                );
                params.insert(ADMIN_PAYMENTS_STATE_PARAM.to_string(), state.to_string());
            }
            None => {
                params.insert(
                    ADMIN_PAYMENTS_STATE_PARAM.to_string(),
                    ADMIN_PAYMENTS_MALFORMED.to_string(),
                );
            }
        },
        Err(()) => {
            params.insert(
                ADMIN_PAYMENTS_STATE_PARAM.to_string(),
                ADMIN_PAYMENTS_UNAVAILABLE.to_string(),
            );
        }
    }
}

/// Preserve the closed mutation result on the payment page after a native
/// form redirect. Read query fields remain owned by their strict parsers; the
/// mutation marker is a separate, bounded UI outcome and never reaches a
/// service request.
fn payment_mutation_query(raw_query: &str) -> Option<&'static str> {
    let mut state = None;
    let mut malformed = false;
    for (key, value) in url::form_urlencoded::parse(raw_query.as_bytes()) {
        if key != "mutation" {
            continue;
        }
        let candidate = match value.as_ref() {
            "success" => "success",
            "conflict" => "conflict",
            "forbidden" => "forbidden",
            "unavailable" => "unavailable",
            "malformed" => "malformed",
            _ => "malformed",
        };
        if state.replace(candidate).is_some() {
            malformed = true;
        }
    }
    if malformed {
        Some("malformed")
    } else {
        state
    }
}

/// Strip exactly one canonical admin mount prefix. Repeated prefixes remain
/// in the routed path and therefore cannot alias an allowlisted admin loader.
fn strip_single_admin_prefix(path: &str) -> Option<&str> {
    if path == "/admin" {
        Some("/")
    } else if path.starts_with("/admin/") {
        path.strip_prefix("/admin")
    } else {
        None
    }
}

fn is_dashboard_user_status_route(route_path: &str) -> bool {
    matches!(route_path, "/" | "/index")
}

/// The responsive capture harness uses `?__design_bypass=1` for authenticated
/// admin states. Honor it only for local-cookie requests and only as a
/// UI-only fixture; it never creates a bearer token or changes backend policy.
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

fn developer_secret_once_cookie(headers: &HeaderMap) -> Option<AdminDeveloperSecretOnceProjection> {
    let raw = headers.get(header::COOKIE)?.to_str().ok()?;
    let encoded = raw.split(';').find_map(|cookie| {
        let (name, value) = cookie.trim().split_once('=')?;
        (name == super::ADMIN_DEVELOPER_SECRET_COOKIE).then_some(value)
    })?;
    let bytes = URL_SAFE_NO_PAD.decode(encoded).ok()?;
    let value = serde_json::from_slice::<serde_json::Value>(&bytes).ok()?;
    decode_admin_developer_secret_once(value)
}

fn notification_create_once_cookie(
    headers: &HeaderMap,
) -> Option<epsx_dioxus_ui::pages::admin_pages::notifications::AdminNotificationCreateResult> {
    let raw = headers.get(header::COOKIE)?.to_str().ok()?;
    let encoded = raw.split(';').find_map(|cookie| {
        let (name, value) = cookie.trim().split_once('=')?;
        (name == super::ADMIN_NOTIFICATION_CREATE_COOKIE).then_some(value)
    })?;
    let bytes = URL_SAFE_NO_PAD.decode(encoded).ok()?;
    let value = serde_json::from_slice::<serde_json::Value>(&bytes).ok()?;
    decode_admin_notification_create_result(value)
}

pub async fn ssr_handler(State(state): State<AppState>, request: Request) -> Response {
    let (parts, _body) = request.into_parts();
    let path = parts.uri.path().to_string();
    let raw_query = parts.uri.query().map(str::to_string);
    let query = raw_query.clone().unwrap_or_default();
    let headers = parts.headers.clone();
    let design_bypass = design_bypass_requested(&query, state.cookie_environment);

    // Resolve only a cryptographically verified canonical cookie/bearer user.
    // Permissions are backend-issued and remain verbatim; the admin UI does no
    // role-to-permission expansion.
    let access_verification =
        auth::access_verification(&headers, state.verifier.as_ref(), state.cookie_environment)
            .await;
    // Local visual-test fixture only: it supplies authenticated admin shell
    // state without a bearer token, so no synthetic identity reaches an
    // upstream data service.
    let dev_bypass_user = auth::dev_bypass_ui_user(&headers, Some(56));
    let design_bypass_user = auth::design_bypass_ui_user(design_bypass, Some(56));
    let recover_session = access_verification.permits_refresh_recovery()
        && auth::refresh_token(&headers, state.cookie_environment).is_some();
    let (verified_access_token, user) = match access_verification {
        AccessVerification::Verified { token, user } => {
            (Some(token), Some(auth::ui_user(user, None)))
        }
        AccessVerification::MissingOrRejected | AccessVerification::VerifierUnavailable => {
            (None, design_bypass_user.or(dev_bypass_user))
        }
    };

    // Admin: load only the bounded, read-only payment-intent dependency. Every
    // outcome is explicit; an upstream error or malformed payload is never
    // represented as an authoritative empty list.
    let mut params = HashMap::new();
    let route_path = strip_single_admin_prefix(&path).unwrap_or(path.as_str());
    let mut notification_send_flash_clear = false;
    if route_path == "/notifications/manage" {
        let (state, clear_cookie) = consume_notification_send_flash(&headers, &query);
        notification_send_flash_clear = clear_cookie;
        if let Some(state) = state {
            params.insert(
                ADMIN_NOTIFICATIONS_SEND_STATE_PARAM.to_string(),
                state.to_string(),
            );
        }
    }
    // The root dashboard combines its narrow user-status contract with the
    // backend-owned analytics snapshot. Both requests use the same verified
    // admin bearer and run concurrently; the page keeps their outcomes
    // independent so a partial backend outage cannot be rendered as zeros.
    if is_dashboard_user_status_route(route_path) {
        if let Some(token) = verified_access_token.as_ref() {
            match AdminDashboardUserStatusQuery::from_raw(raw_query.as_deref()) {
                Ok(dashboard_query) => {
                    let mut request_context = RequestContext::from_headers(&headers);
                    request_context.auth_token = Some(token.clone());
                    let (status_load, analytics_load) = tokio::join!(
                        load_admin_dashboard_user_status(
                            &state.identity,
                            dashboard_query,
                            &request_context,
                        ),
                        // This aggregate is owned by the core backend. The
                        // extracted analytics service owns the audit-log API,
                        // so using its client here would turn a healthy local
                        // service topology into a misleading 404.
                        load_admin_analytics(&state.identity, &request_context),
                    );
                    record_admin_dashboard_user_status_load(&mut params, status_load);
                    record_admin_analytics_load(&mut params, analytics_load);
                }
                Err(()) => {
                    record_admin_dashboard_user_status_load(
                        &mut params,
                        AdminDashboardUserStatusLoad::Malformed,
                    );
                    record_admin_analytics_load(&mut params, AdminAnalyticsLoad::Malformed);
                }
            }
        } else {
            record_admin_dashboard_user_status_load(
                &mut params,
                AdminDashboardUserStatusLoad::Unavailable,
            );
            record_admin_analytics_load(&mut params, AdminAnalyticsLoad::Unavailable);
        }
    }
    // Chat reads use the extracted backend projection. The list query is
    // closed and URL-persistent; dynamic conversation identifiers are parsed
    // and canonicalized by the adapter before any upstream request.
    if route_path == "/chat" {
        match AdminChatQuery::from_raw(&query) {
            Ok(chat_query) => match verified_access_token.as_ref() {
                Some(token) => {
                    let mut request_context = RequestContext::from_headers(&headers);
                    request_context.auth_token = Some(token.clone());
                    let load =
                        load_admin_chat(&state.identity, &chat_query, &request_context).await;
                    record_admin_chat_list_load(&mut params, load);
                }
                None => record_admin_chat_list_load(&mut params, AdminChatListLoad::Unavailable),
            },
            Err(()) => record_admin_chat_list_load(&mut params, AdminChatListLoad::Malformed),
        }
    } else if let Some(conversation_id) = route_path
        .strip_prefix("/chat/")
        .filter(|value| !value.is_empty() && !value.contains('/'))
    {
        if query.is_empty() || chat_mutation_query(&query).is_some() {
            match verified_access_token.as_ref() {
                Some(token) => {
                    let mut request_context = RequestContext::from_headers(&headers);
                    request_context.auth_token = Some(token.clone());
                    let load =
                        load_admin_chat_detail(&state.identity, conversation_id, &request_context)
                            .await;
                    record_admin_chat_detail_load(&mut params, load);
                }
                None => {
                    record_admin_chat_detail_load(&mut params, AdminChatDetailLoad::Unavailable)
                }
            }
        } else {
            record_admin_chat_detail_load(&mut params, AdminChatDetailLoad::Malformed);
        }
    }
    // Wallet inventory is a bounded, backend-authoritative list projection.
    // The shared wallet-management hub reads its aggregate status projection
    // on every nested wallet route, mirroring the deployed parent layout.
    if route_path == "/wallet-management/wallets" {
        match AdminWalletListQuery::from_raw(&query) {
            Ok(wallet_query) => match verified_access_token.as_ref() {
                Some(token) => {
                    let mut request_context = RequestContext::from_headers(&headers);
                    request_context.auth_token = Some(token.clone());
                    let list =
                        load_wallet_list(&state.wallet, &wallet_query, &request_context).await;
                    record_commerce_load!(
                        &mut params,
                        list,
                        ADMIN_WALLET_LIST_DATA_PARAM,
                        ADMIN_WALLET_LIST_STATE_PARAM,
                        ADMIN_WALLET_LIST_READY,
                        Some(ADMIN_WALLET_LIST_EMPTY),
                        ADMIN_WALLET_LIST_FORBIDDEN,
                        ADMIN_WALLET_LIST_UNAVAILABLE,
                        ADMIN_WALLET_LIST_MALFORMED,
                    );
                }
                None => {
                    record_commerce_load!(
                        &mut params,
                        CommerceLoad::<epsx_dioxus_ui::pages::admin_pages::wallet_wallets::AdminWalletListProjection>::Unavailable,
                        ADMIN_WALLET_LIST_DATA_PARAM,
                        ADMIN_WALLET_LIST_STATE_PARAM,
                        ADMIN_WALLET_LIST_READY,
                        Some(ADMIN_WALLET_LIST_EMPTY),
                        ADMIN_WALLET_LIST_FORBIDDEN,
                        ADMIN_WALLET_LIST_UNAVAILABLE,
                        ADMIN_WALLET_LIST_MALFORMED,
                    );
                }
            },
            Err(()) => {
                record_commerce_load!(
                    &mut params,
                    CommerceLoad::<epsx_dioxus_ui::pages::admin_pages::wallet_wallets::AdminWalletListProjection>::Malformed,
                    ADMIN_WALLET_LIST_DATA_PARAM,
                    ADMIN_WALLET_LIST_STATE_PARAM,
                    ADMIN_WALLET_LIST_READY,
                    Some(ADMIN_WALLET_LIST_EMPTY),
                    ADMIN_WALLET_LIST_FORBIDDEN,
                    ADMIN_WALLET_LIST_UNAVAILABLE,
                    ADMIN_WALLET_LIST_MALFORMED,
                );
            }
        }
    }
    if route_path.starts_with("/wallet-management/") {
        match verified_access_token.as_ref() {
            Some(token) => {
                let mut request_context = RequestContext::from_headers(&headers);
                request_context.auth_token = Some(token.clone());
                let load = load_wallet_stats(&state.wallet, &request_context).await;
                record_commerce_load!(
                    &mut params,
                    load,
                    ADMIN_WALLET_STATS_DATA_PARAM,
                    ADMIN_WALLET_STATS_STATE_PARAM,
                    ADMIN_WALLET_STATS_READY,
                    None,
                    ADMIN_WALLET_STATS_FORBIDDEN,
                    ADMIN_WALLET_STATS_UNAVAILABLE,
                    ADMIN_WALLET_STATS_MALFORMED,
                );
            }
            None => record_commerce_load!(
                &mut params,
                CommerceLoad::<AdminWalletStatsSummary>::Unavailable,
                ADMIN_WALLET_STATS_DATA_PARAM,
                ADMIN_WALLET_STATS_STATE_PARAM,
                ADMIN_WALLET_STATS_READY,
                None,
                ADMIN_WALLET_STATS_FORBIDDEN,
                ADMIN_WALLET_STATS_UNAVAILABLE,
                ADMIN_WALLET_STATS_MALFORMED,
            ),
        }
    }
    if route_path == "/wallet-management/credits" {
        match verified_access_token.as_ref() {
            Some(token) => {
                let mut request_context = RequestContext::from_headers(&headers);
                request_context.auth_token = Some(token.clone());
                let load = load_credit_stats(&state.wallet, &request_context).await;
                record_commerce_load!(
                    &mut params,
                    load,
                    ADMIN_CREDITS_DATA_PARAM,
                    ADMIN_CREDITS_STATE_PARAM,
                    ADMIN_CREDITS_READY,
                    None,
                    ADMIN_CREDITS_FORBIDDEN,
                    ADMIN_CREDITS_UNAVAILABLE,
                    ADMIN_CREDITS_MALFORMED,
                );
            }
            None => record_commerce_load!(
                &mut params,
                CommerceLoad::<AdminCreditStatsProjection>::Unavailable,
                ADMIN_CREDITS_DATA_PARAM,
                ADMIN_CREDITS_STATE_PARAM,
                ADMIN_CREDITS_READY,
                None,
                ADMIN_CREDITS_FORBIDDEN,
                ADMIN_CREDITS_UNAVAILABLE,
                ADMIN_CREDITS_MALFORMED,
            ),
        }
    }
    if route_path == "/wallet-management/access" {
        match verified_access_token.as_ref() {
            Some(token) => {
                let mut request_context = RequestContext::from_headers(&headers);
                request_context.auth_token = Some(token.clone());
                let load = load_access(&state.subscription, &request_context).await;
                record_commerce_load!(
                    &mut params,
                    load,
                    ADMIN_ACCESS_DATA_PARAM,
                    ADMIN_ACCESS_STATE_PARAM,
                    ADMIN_ACCESS_READY,
                    None,
                    ADMIN_ACCESS_FORBIDDEN,
                    ADMIN_ACCESS_UNAVAILABLE,
                    ADMIN_ACCESS_MALFORMED,
                );
            }
            None => record_commerce_load!(
                &mut params,
                CommerceLoad::<AdminAccessProjection>::Unavailable,
                ADMIN_ACCESS_DATA_PARAM,
                ADMIN_ACCESS_STATE_PARAM,
                ADMIN_ACCESS_READY,
                None,
                ADMIN_ACCESS_FORBIDDEN,
                ADMIN_ACCESS_UNAVAILABLE,
                ADMIN_ACCESS_MALFORMED,
            ),
        }
    }
    if route_path == "/wallet-management/access/plans" {
        match verified_access_token.as_ref() {
            Some(token) => {
                let mut request_context = RequestContext::from_headers(&headers);
                request_context.auth_token = Some(token.clone());
                let load = load_plans(&state.subscription, &request_context).await;
                record_commerce_load!(
                    &mut params,
                    load,
                    ADMIN_PLANS_DATA_PARAM,
                    ADMIN_PLANS_STATE_PARAM,
                    ADMIN_PLANS_READY,
                    Some(ADMIN_PLANS_EMPTY),
                    ADMIN_PLANS_FORBIDDEN,
                    ADMIN_PLANS_UNAVAILABLE,
                    ADMIN_PLANS_MALFORMED,
                );
            }
            None => record_commerce_load!(
                &mut params,
                CommerceLoad::<AdminPlanListProjection>::Unavailable,
                ADMIN_PLANS_DATA_PARAM,
                ADMIN_PLANS_STATE_PARAM,
                ADMIN_PLANS_READY,
                Some(ADMIN_PLANS_EMPTY),
                ADMIN_PLANS_FORBIDDEN,
                ADMIN_PLANS_UNAVAILABLE,
                ADMIN_PLANS_MALFORMED,
            ),
        }
    }
    if let Some(plan_id) = route_path
        .strip_prefix("/wallet-management/access/plans/")
        .filter(|value| !value.is_empty() && !value.contains('/'))
    {
        if query.is_empty() || wallet_plan_mutation_query(&query).is_some() {
            match verified_access_token.as_ref() {
                Some(token) => {
                    let mut request_context = RequestContext::from_headers(&headers);
                    request_context.auth_token = Some(token.clone());
                    let load =
                        load_plan_detail(&state.subscription, plan_id, &request_context).await;
                    record_commerce_load!(
                        &mut params,
                        load,
                        ADMIN_PLAN_DETAIL_DATA_PARAM,
                        ADMIN_PLAN_DETAIL_STATE_PARAM,
                        ADMIN_PLANS_READY,
                        None,
                        ADMIN_PLANS_FORBIDDEN,
                        ADMIN_PLANS_UNAVAILABLE,
                        ADMIN_PLANS_MALFORMED,
                    );
                }
                None => record_commerce_load!(
                    &mut params,
                    CommerceLoad::<AdminPlanProjection>::Unavailable,
                    ADMIN_PLAN_DETAIL_DATA_PARAM,
                    ADMIN_PLAN_DETAIL_STATE_PARAM,
                    ADMIN_PLANS_READY,
                    None,
                    ADMIN_PLANS_FORBIDDEN,
                    ADMIN_PLANS_UNAVAILABLE,
                    ADMIN_PLANS_MALFORMED,
                ),
            }
        } else {
            record_commerce_load!(
                &mut params,
                CommerceLoad::<AdminPlanProjection>::Malformed,
                ADMIN_PLAN_DETAIL_DATA_PARAM,
                ADMIN_PLAN_DETAIL_STATE_PARAM,
                ADMIN_PLANS_READY,
                None,
                ADMIN_PLANS_FORBIDDEN,
                ADMIN_PLANS_UNAVAILABLE,
                ADMIN_PLANS_MALFORMED,
            );
        }
    }
    if let Some(address) = route_path
        .strip_prefix("/wallet-management/")
        .filter(|value| {
            !value.is_empty()
                && !value.contains('/')
                && !matches!(*value, "wallets" | "credits" | "access")
        })
    {
        if query.is_empty() {
            match verified_access_token.as_ref() {
                Some(token) => {
                    let mut request_context = RequestContext::from_headers(&headers);
                    request_context.auth_token = Some(token.clone());
                    let detail = load_wallet_detail(&state.wallet, address, &request_context).await;
                    let access =
                        load_wallet_access(&state.subscription, address, &request_context).await;
                    let plans = load_plans(&state.subscription, &request_context).await;
                    record_commerce_load!(
                        &mut params,
                        detail,
                        ADMIN_WALLET_DETAIL_DATA_PARAM,
                        ADMIN_WALLET_DETAIL_STATE_PARAM,
                        ADMIN_WALLET_DETAIL_READY,
                        None,
                        ADMIN_WALLET_DETAIL_FORBIDDEN,
                        ADMIN_WALLET_DETAIL_UNAVAILABLE,
                        ADMIN_WALLET_DETAIL_MALFORMED,
                    );
                    record_commerce_load!(
                        &mut params,
                        access,
                        ADMIN_ACCESS_DATA_PARAM,
                        ADMIN_ACCESS_STATE_PARAM,
                        ADMIN_ACCESS_READY,
                        None,
                        ADMIN_ACCESS_FORBIDDEN,
                        ADMIN_ACCESS_UNAVAILABLE,
                        ADMIN_ACCESS_MALFORMED,
                    );
                    record_commerce_load!(
                        &mut params,
                        plans,
                        ADMIN_PLANS_DATA_PARAM,
                        ADMIN_PLANS_STATE_PARAM,
                        ADMIN_PLANS_READY,
                        Some(ADMIN_PLANS_EMPTY),
                        ADMIN_PLANS_FORBIDDEN,
                        ADMIN_PLANS_UNAVAILABLE,
                        ADMIN_PLANS_MALFORMED,
                    );
                }
                None => {
                    record_commerce_load!(
                        &mut params,
                        CommerceLoad::<AdminWalletDetailProjection>::Unavailable,
                        ADMIN_WALLET_DETAIL_DATA_PARAM,
                        ADMIN_WALLET_DETAIL_STATE_PARAM,
                        ADMIN_WALLET_DETAIL_READY,
                        None,
                        ADMIN_WALLET_DETAIL_FORBIDDEN,
                        ADMIN_WALLET_DETAIL_UNAVAILABLE,
                        ADMIN_WALLET_DETAIL_MALFORMED,
                    );
                    record_commerce_load!(
                        &mut params,
                        CommerceLoad::<AdminAccessProjection>::Unavailable,
                        ADMIN_ACCESS_DATA_PARAM,
                        ADMIN_ACCESS_STATE_PARAM,
                        ADMIN_ACCESS_READY,
                        None,
                        ADMIN_ACCESS_FORBIDDEN,
                        ADMIN_ACCESS_UNAVAILABLE,
                        ADMIN_ACCESS_MALFORMED,
                    );
                    record_commerce_load!(
                        &mut params,
                        CommerceLoad::<AdminPlanListProjection>::Unavailable,
                        ADMIN_PLANS_DATA_PARAM,
                        ADMIN_PLANS_STATE_PARAM,
                        ADMIN_PLANS_READY,
                        Some(ADMIN_PLANS_EMPTY),
                        ADMIN_PLANS_FORBIDDEN,
                        ADMIN_PLANS_UNAVAILABLE,
                        ADMIN_PLANS_MALFORMED,
                    );
                }
            }
        } else {
            record_commerce_load!(
                &mut params,
                CommerceLoad::<AdminWalletDetailProjection>::Malformed,
                ADMIN_WALLET_DETAIL_DATA_PARAM,
                ADMIN_WALLET_DETAIL_STATE_PARAM,
                ADMIN_WALLET_DETAIL_READY,
                None,
                ADMIN_WALLET_DETAIL_FORBIDDEN,
                ADMIN_WALLET_DETAIL_UNAVAILABLE,
                ADMIN_WALLET_DETAIL_MALFORMED,
            );
            record_commerce_load!(
                &mut params,
                CommerceLoad::<AdminAccessProjection>::Malformed,
                ADMIN_ACCESS_DATA_PARAM,
                ADMIN_ACCESS_STATE_PARAM,
                ADMIN_ACCESS_READY,
                None,
                ADMIN_ACCESS_FORBIDDEN,
                ADMIN_ACCESS_UNAVAILABLE,
                ADMIN_ACCESS_MALFORMED,
            );
            record_commerce_load!(
                &mut params,
                CommerceLoad::<AdminPlanListProjection>::Malformed,
                ADMIN_PLANS_DATA_PARAM,
                ADMIN_PLANS_STATE_PARAM,
                ADMIN_PLANS_READY,
                Some(ADMIN_PLANS_EMPTY),
                ADMIN_PLANS_FORBIDDEN,
                ADMIN_PLANS_UNAVAILABLE,
                ADMIN_PLANS_MALFORMED,
            );
        }
    }
    if let Some(address) = route_path
        .strip_prefix("/wallet-management/wallets/")
        .and_then(|value| value.strip_suffix("/disable"))
        .filter(|value| !value.is_empty() && !value.contains('/'))
    {
        if query.is_empty() {
            match verified_access_token.as_ref() {
                Some(token) => {
                    let mut request_context = RequestContext::from_headers(&headers);
                    request_context.auth_token = Some(token.clone());
                    let load = load_wallet_detail(&state.wallet, address, &request_context).await;
                    record_commerce_load!(
                        &mut params,
                        load,
                        ADMIN_WALLET_DETAIL_DATA_PARAM,
                        ADMIN_WALLET_DETAIL_STATE_PARAM,
                        ADMIN_WALLET_DETAIL_READY,
                        None,
                        ADMIN_WALLET_DETAIL_FORBIDDEN,
                        ADMIN_WALLET_DETAIL_UNAVAILABLE,
                        ADMIN_WALLET_DETAIL_MALFORMED,
                    );
                    params.insert(
                        ADMIN_WALLET_DISABLE_STATE_PARAM.to_string(),
                        ADMIN_WALLET_DISABLE_FORM.to_string(),
                    );
                }
                None => {
                    params.insert(
                        ADMIN_WALLET_DISABLE_STATE_PARAM.to_string(),
                        ADMIN_WALLET_DISABLE_UNAVAILABLE.to_string(),
                    );
                }
            }
        } else {
            let state = match url::form_urlencoded::parse(query.as_bytes())
                .collect::<Vec<_>>()
                .as_slice()
            {
                [(key, value)] if key == "mutation" => match value.as_ref() {
                    "success" => ADMIN_WALLET_DISABLE_SUCCESS,
                    "conflict" => ADMIN_WALLET_DISABLE_CONFLICT,
                    "forbidden" => ADMIN_WALLET_DISABLE_FORBIDDEN,
                    "unavailable" => ADMIN_WALLET_DISABLE_UNAVAILABLE,
                    _ => ADMIN_WALLET_DISABLE_MALFORMED,
                },
                _ => ADMIN_WALLET_DISABLE_MALFORMED,
            };
            params.insert(
                ADMIN_WALLET_DISABLE_STATE_PARAM.to_string(),
                state.to_string(),
            );
        }
    }
    if route_path == "/payments" {
        match (
            super::payment_tab(&query),
            super::PaymentIntentQuery::from_raw(&query),
        ) {
            (Ok(tab), Ok(payment_query)) => {
                params.insert(ADMIN_PAYMENTS_TAB_PARAM.to_string(), tab.to_string());
                params.insert(
                    ADMIN_PAYMENTS_LIMIT_PARAM.to_string(),
                    payment_query.limit.to_string(),
                );
                params.insert(
                    ADMIN_PAYMENTS_OFFSET_PARAM.to_string(),
                    payment_query.offset.to_string(),
                );
                if let Some(payer) = &payment_query.payer {
                    params.insert(ADMIN_PAYMENTS_PAYER_PARAM.to_string(), payer.clone());
                }
                if let Some(status) = &payment_query.status {
                    params.insert(ADMIN_PAYMENTS_STATUS_PARAM.to_string(), status.clone());
                }

                if tab == "payments" {
                    match verified_access_token.as_ref() {
                        Some(token) => {
                            let mut request_context = RequestContext::from_headers(&headers);
                            request_context.auth_token = Some(token.clone());
                            let result = state
                                .payment
                                .get_with_ctx(&payment_query.upstream_path(), &request_context)
                                .await
                                .map_err(|error| {
                                    tracing::warn!(
                                        "admin payment-intent SSR load unavailable: {error}"
                                    );
                                });
                            record_payment_intent_load(&mut params, result);
                        }
                        None => record_payment_intent_load(&mut params, Err(())),
                    }
                } else if tab == "payment-links" {
                    match verified_access_token.as_ref() {
                        Some(token) => {
                            let mut request_context = RequestContext::from_headers(&headers);
                            request_context.auth_token = Some(token.clone());
                            let load = load_payment_links(&state.payment, &request_context).await;
                            record_payment_links_load(&mut params, load);
                        }
                        None => record_payment_links_load(
                            &mut params,
                            CommerceLoad::<AdminPaymentLinkListProjection>::Unavailable,
                        ),
                    }
                } else if tab == "user-access" {
                    match AdminPaymentUserAccessQuery::from_raw(&query) {
                        Ok(user_access_query) => match verified_access_token.as_ref() {
                            Some(token) => {
                                let mut request_context = RequestContext::from_headers(&headers);
                                request_context.auth_token = Some(token.clone());
                                let load = load_payment_user_access(
                                    &state.identity,
                                    &user_access_query,
                                    &request_context,
                                )
                                .await;
                                record_commerce_load!(
                                    &mut params,
                                    load,
                                    ADMIN_PAYMENT_USER_ACCESS_DATA_PARAM,
                                    ADMIN_PAYMENT_USER_ACCESS_STATE_PARAM,
                                    ADMIN_PAYMENT_USER_ACCESS_READY,
                                    Some(ADMIN_PAYMENT_USER_ACCESS_EMPTY),
                                    ADMIN_PAYMENT_USER_ACCESS_FORBIDDEN,
                                    ADMIN_PAYMENT_USER_ACCESS_UNAVAILABLE,
                                    ADMIN_PAYMENT_USER_ACCESS_MALFORMED,
                                );
                            }
                            None => record_commerce_load!(
                                &mut params,
                                CommerceLoad::<AdminPaymentUserAccessProjection>::Unavailable,
                                ADMIN_PAYMENT_USER_ACCESS_DATA_PARAM,
                                ADMIN_PAYMENT_USER_ACCESS_STATE_PARAM,
                                ADMIN_PAYMENT_USER_ACCESS_READY,
                                Some(ADMIN_PAYMENT_USER_ACCESS_EMPTY),
                                ADMIN_PAYMENT_USER_ACCESS_FORBIDDEN,
                                ADMIN_PAYMENT_USER_ACCESS_UNAVAILABLE,
                                ADMIN_PAYMENT_USER_ACCESS_MALFORMED,
                            ),
                        },
                        Err(()) => record_commerce_load!(
                            &mut params,
                            CommerceLoad::<AdminPaymentUserAccessProjection>::Malformed,
                            ADMIN_PAYMENT_USER_ACCESS_DATA_PARAM,
                            ADMIN_PAYMENT_USER_ACCESS_STATE_PARAM,
                            ADMIN_PAYMENT_USER_ACCESS_READY,
                            Some(ADMIN_PAYMENT_USER_ACCESS_EMPTY),
                            ADMIN_PAYMENT_USER_ACCESS_FORBIDDEN,
                            ADMIN_PAYMENT_USER_ACCESS_UNAVAILABLE,
                            ADMIN_PAYMENT_USER_ACCESS_MALFORMED,
                        ),
                    }
                }
            }
            _ => {
                params.insert(ADMIN_PAYMENTS_TAB_PARAM.to_string(), "payments".to_string());
                params.insert(
                    ADMIN_PAYMENTS_STATE_PARAM.to_string(),
                    ADMIN_PAYMENTS_MALFORMED.to_string(),
                );
            }
        }
        if let Some(state) = payment_mutation_query(&query) {
            params.insert(ADMIN_PAYMENT_MUTATION_PARAM.to_string(), state.to_string());
        }
    }
    // Media inventory SSR is a single strict compatibility read. Only the
    // verified session bearer and request ID cross this boundary; object URLs
    // and every mutation-capable storage detail are projected away.
    if route_path == "/media" {
        match parse_admin_media_query(&query) {
            Ok((media_query, mutation)) => match verified_access_token.as_ref() {
                Some(token) => {
                    let mut request_context = RequestContext::from_headers(&headers);
                    request_context.auth_token = Some(token.clone());
                    let load =
                        load_admin_media(&state.content, &media_query, &request_context).await;
                    record_admin_media_load(&mut params, &media_query, load);
                    record_admin_media_mutation_query(&mut params, mutation);
                }
                None => {
                    record_admin_media_load(&mut params, &media_query, AdminMediaLoad::Unavailable);
                    record_admin_media_mutation_query(&mut params, mutation);
                }
            },
            Err(()) => {
                let default_query =
                    AdminMediaQuery::from_raw("").expect("the empty admin-media query is valid");
                record_admin_media_load(&mut params, &default_query, AdminMediaLoad::Malformed);
                record_admin_media_mutation_query(&mut params, None);
            }
        }
    }
    // The pinned admin news list still has one exact, backend-owned Rust read
    // contract. Use that compatibility endpoint only for `/news`; the public
    // file-backed content-service feed is not an admin record authority. The
    // adapter projects away article bodies and rejects every contract drift.
    if route_path == "/news" {
        match AdminNewsQuery::from_raw(&query) {
            Ok(news_query) => match verified_access_token.as_ref() {
                Some(token) => {
                    let mut request_context = RequestContext::from_headers(&headers);
                    request_context.auth_token = Some(token.clone());
                    let load = load_admin_news(&state.content, &news_query, &request_context).await;
                    record_admin_news_load(&mut params, &news_query, load);
                    if let Some(state) = news_inventory_mutation_query(&query) {
                        params.insert(
                            ADMIN_NEWS_MUTATION_STATE_PARAM.to_string(),
                            state.to_string(),
                        );
                    }
                }
                None => {
                    record_admin_news_load(&mut params, &news_query, AdminNewsLoad::Unavailable);
                    if let Some(state) = news_inventory_mutation_query(&query) {
                        params.insert(
                            ADMIN_NEWS_MUTATION_STATE_PARAM.to_string(),
                            state.to_string(),
                        );
                    }
                }
            },
            Err(()) => {
                let default_query =
                    AdminNewsQuery::from_raw("").expect("the empty admin-news query is valid");
                record_admin_news_load(&mut params, &default_query, AdminNewsLoad::Malformed);
            }
        }
    }
    // The create workspace contains no backend record and is still safe to
    // render before a token is available. `AuthGate` owns page visibility,
    // while the POST action independently requires verified admin auth.
    if route_path == "/news/create" {
        match query.is_empty() {
            true => {
                params.insert(
                    ADMIN_NEWS_EDITOR_STATE_PARAM.to_string(),
                    ADMIN_NEWS_EDITOR_FORM.to_string(),
                );
            }
            false => {
                if let Some(state) = news_mutation_query(&query) {
                    params.insert(
                        ADMIN_NEWS_MUTATION_STATE_PARAM.to_string(),
                        state.to_string(),
                    );
                    if state == ADMIN_NEWS_MUTATION_CONFLICT {
                        params.insert(
                            ADMIN_NEWS_MUTATION_ERROR_PARAM.to_string(),
                            "The article mutation conflicted with current backend state."
                                .to_string(),
                        );
                    }
                } else {
                    params.insert(
                        ADMIN_NEWS_MUTATION_STATE_PARAM.to_string(),
                        ADMIN_NEWS_MUTATION_MALFORMED.to_string(),
                    );
                }
            }
        }
    }
    if let Some(article_id) = route_path
        .strip_prefix("/news/")
        .and_then(|value| value.strip_suffix("/edit"))
        .filter(|value| !value.is_empty() && !value.contains('/'))
    {
        if verified_access_token.is_some() {
            if query.is_empty() {
                let mut request_context = RequestContext::from_headers(&headers);
                request_context.auth_token = verified_access_token.clone();
                let load =
                    load_admin_news_editor(&state.content, article_id, &request_context).await;
                record_admin_news_editor_load(&mut params, article_id, load);
            } else if let Some(image_url) = news_image_url_query(&query) {
                let mut request_context = RequestContext::from_headers(&headers);
                request_context.auth_token = verified_access_token.clone();
                let load =
                    load_admin_news_editor(&state.content, article_id, &request_context).await;
                record_admin_news_editor_load(&mut params, article_id, load);
                params.insert(ADMIN_NEWS_IMAGE_URL_PARAM.to_string(), image_url);
                params.insert(
                    ADMIN_NEWS_IMAGE_STATE_PARAM.to_string(),
                    ADMIN_NEWS_IMAGE_COMMITTED.to_string(),
                );
            } else if let Some(state) = news_mutation_query(&query) {
                params.insert(
                    ADMIN_NEWS_MUTATION_STATE_PARAM.to_string(),
                    state.to_string(),
                );
                if state == ADMIN_NEWS_MUTATION_CONFLICT {
                    params.insert(
                        ADMIN_NEWS_MUTATION_ERROR_PARAM.to_string(),
                        "The article mutation conflicted with current backend state.".to_string(),
                    );
                }
            } else {
                params.insert(
                    ADMIN_NEWS_MUTATION_STATE_PARAM.to_string(),
                    ADMIN_NEWS_MUTATION_MALFORMED.to_string(),
                );
            }
        }
    }
    // The management page requires a global admin inventory. Never reuse the
    // owner-scoped `/notification/list` feed: it would show only the admin's
    // wallet records and mislabel them as global state. This exact read keeps
    // authorization in the notification service and projects no identity or
    // message body through SSR.
    if route_path == "/notifications/manage" {
        params.remove(ADMIN_NOTIFICATIONS_MUTATION_PARAM);
        if let Some(mutation) = notification_mutation_query(&query) {
            params.insert(
                ADMIN_NOTIFICATIONS_MUTATION_PARAM.to_string(),
                mutation.to_string(),
            );
        }
        match AdminNotificationQuery::from_raw(&query) {
            Ok(notification_query) => match verified_access_token.as_ref() {
                Some(token) => {
                    let mut request_context = RequestContext::from_headers(&headers);
                    request_context.auth_token = Some(token.clone());
                    let (load, metrics) = tokio::join!(
                        load_admin_notifications(
                            &state.notification,
                            &notification_query,
                            &request_context,
                        ),
                        load_admin_notification_metrics(&state.notification, &request_context),
                    );
                    record_admin_notification_load(&mut params, &notification_query, load);
                    record_admin_notification_metrics_load(&mut params, metrics);
                }
                None => {
                    record_admin_notification_load(
                        &mut params,
                        &notification_query,
                        AdminNotificationLoad::Unavailable,
                    );
                    record_admin_notification_metrics_load(
                        &mut params,
                        AdminNotificationMetricsLoad::Unavailable,
                    );
                }
            },
            Err(()) => {
                let default_query = AdminNotificationQuery::from_raw("")
                    .expect("the empty admin-notification query is valid");
                record_admin_notification_load(
                    &mut params,
                    &default_query,
                    AdminNotificationLoad::Malformed,
                );
                record_admin_notification_metrics_load(
                    &mut params,
                    AdminNotificationMetricsLoad::Unavailable,
                );
            }
        }
        match verified_access_token.as_ref() {
            Some(token) => {
                let mut request_context = RequestContext::from_headers(&headers);
                request_context.auth_token = Some(token.clone());
                let load =
                    load_admin_notification_metrics(&state.notification, &request_context).await;
                record_admin_notification_metrics_load(&mut params, load);
            }
            None => record_admin_notification_metrics_load(
                &mut params,
                AdminNotificationMetricsLoad::Unavailable,
            ),
        }
    }
    if route_path == "/notifications/create" && verified_access_token.is_some() {
        if let Some(result) = notification_create_once_cookie(&headers) {
            params.insert(
                ADMIN_NOTIFICATION_CREATE_DATA_PARAM.to_string(),
                serde_json::to_string(&result)
                    .expect("the notification create projection is serializable"),
            );
            params.insert(
                ADMIN_NOTIFICATION_CREATE_STATE_PARAM.to_string(),
                result.status,
            );
        } else if query.is_empty() {
            params.insert(
                ADMIN_NOTIFICATION_CREATE_STATE_PARAM.to_string(),
                ADMIN_NOTIFICATION_CREATE_FORM.to_string(),
            );
        } else {
            let state = match url::form_urlencoded::parse(query.as_bytes())
                .collect::<Vec<_>>()
                .as_slice()
            {
                [(key, value)] if key == "mutation" => match value.as_ref() {
                    "pending" => ADMIN_NOTIFICATION_CREATE_PENDING,
                    "sent" => ADMIN_NOTIFICATION_CREATE_SENT,
                    "failed" => ADMIN_NOTIFICATION_CREATE_FAILED,
                    "forbidden" => ADMIN_NOTIFICATION_CREATE_FORBIDDEN,
                    "conflict" => ADMIN_NOTIFICATION_CREATE_CONFLICT,
                    "invalid" => ADMIN_NOTIFICATION_CREATE_INVALID,
                    "unavailable" => ADMIN_NOTIFICATION_CREATE_UNAVAILABLE,
                    "malformed" => ADMIN_NOTIFICATION_CREATE_MALFORMED,
                    _ => ADMIN_NOTIFICATION_CREATE_MALFORMED,
                },
                _ => ADMIN_NOTIFICATION_CREATE_MALFORMED,
            };
            params.insert(
                ADMIN_NOTIFICATION_CREATE_STATE_PARAM.to_string(),
                state.to_string(),
            );
        }
    }
    // Audit records are loaded only from the extracted analytics service's
    // exact redacted admin feed. The legacy monolith route exposes sensitive
    // fields and uses the wrong permission, so it is never used as fallback.
    if route_path == "/audit-log" {
        match AdminAuditQuery::from_raw(&query) {
            Ok(audit_query) => match verified_access_token.as_ref() {
                Some(token) => {
                    let mut request_context = RequestContext::from_headers(&headers);
                    request_context.auth_token = Some(token.clone());
                    let load =
                        load_admin_audit(&state.analytics, &audit_query, &request_context).await;
                    record_admin_audit_load(&mut params, &audit_query, load);
                }
                None => {
                    record_admin_audit_load(&mut params, &audit_query, AdminAuditLoad::Unavailable)
                }
            },
            Err(()) => {
                let default_query =
                    AdminAuditQuery::from_raw("").expect("the empty admin-audit query is valid");
                record_admin_audit_load(&mut params, &default_query, AdminAuditLoad::Malformed);
            }
        }
    }
    if route_path == "/settings" {
        match verified_access_token.as_ref() {
            Some(token) => {
                let mut request_context = RequestContext::from_headers(&headers);
                request_context.auth_token = Some(token.clone());
                let load = load_admin_settings(&state.identity, &request_context).await;
                record_admin_settings_load(&mut params, load);
            }
            None => record_admin_settings_load(&mut params, AdminSettingsLoad::Unavailable),
        }
        match AdminSettingsQuery::from_raw(&query) {
            Ok(settings_query) => {
                params.insert(ADMIN_SETTINGS_TAB_PARAM.to_string(), settings_query.tab);
                if let Some(mutation) = settings_query.mutation {
                    params.insert(ADMIN_SETTINGS_MUTATION_PARAM.to_string(), mutation);
                }
            }
            Err(()) => {
                params.insert(ADMIN_SETTINGS_TAB_PARAM.to_string(), "general".to_string());
                params.insert(
                    ADMIN_SETTINGS_MUTATION_PARAM.to_string(),
                    "malformed".to_string(),
                );
            }
        }
    }
    if route_path == "/analytics" {
        let normalized_query = ranking_analytics_query(&query);
        let rankings = async {
            match normalized_query.as_deref() {
                Ok(query) => {
                    load_ranking_analytics(
                        state.analytics.as_ref(),
                        query,
                        &headers,
                        verified_access_token.as_deref(),
                    )
                    .await
                }
                Err(()) => Err(RankingAnalyticsLoadError::Malformed),
            }
        };
        let filters = load_ranking_analytics_filters(state.analytics.as_ref());
        let (rankings, filters) = tokio::join!(rankings, filters);
        if let Ok(query) = normalized_query.as_deref() {
            record_ranking_analytics_query(&mut params, query);
        } else {
            params.remove(ANALYTICS_QUERY_PARAM);
        }
        record_ranking_analytics_load(&mut params, rankings);
        record_ranking_analytics_filters(&mut params, filters);
        params.remove(ANALYTICS_WATCHLIST_DATA_PARAM);
        params.insert(
            ANALYTICS_WATCHLIST_STATE_PARAM.to_string(),
            "unavailable".to_string(),
        );
    }
    if route_path == "/developer-portal" {
        match verified_access_token.as_ref() {
            Some(token) => {
                let mut request_context = RequestContext::from_headers(&headers);
                request_context.auth_token = Some(token.clone());
                let load = load_admin_developer_portal(&state.identity, &request_context).await;
                record_admin_developer_load(&mut params, load);
            }
            None => record_admin_developer_load(&mut params, AdminDeveloperLoad::Unavailable),
        }
    }
    // The create form contains no backend record. AuthGate controls whether it
    // is visible; the POST action still requires verified admin credentials.
    if route_path == "/developer-portal/api-keys/create" {
        if let Some(projection) = developer_secret_once_cookie(&headers) {
            params.insert(
                ADMIN_DEVELOPER_CREATE_DATA_PARAM.to_string(),
                serde_json::to_string(&projection)
                    .expect("the secret-once developer projection is serializable"),
            );
            params.insert(
                ADMIN_DEVELOPER_CREATE_STATE_PARAM.to_string(),
                ADMIN_DEVELOPER_CREATE_CREATED.to_string(),
            );
        } else if query.is_empty() {
            params.insert(
                ADMIN_DEVELOPER_CREATE_STATE_PARAM.to_string(),
                ADMIN_DEVELOPER_CREATE_FORM.to_string(),
            );
        } else {
            let state = match url::form_urlencoded::parse(query.as_bytes())
                .collect::<Vec<_>>()
                .as_slice()
            {
                [(key, value)] if key == "mutation" => match value.as_ref() {
                    "conflict" => ADMIN_DEVELOPER_CREATE_CONFLICT,
                    "forbidden" => ADMIN_DEVELOPER_CREATE_FORBIDDEN,
                    "unavailable" => ADMIN_DEVELOPER_CREATE_UNAVAILABLE,
                    "malformed" => ADMIN_DEVELOPER_CREATE_MALFORMED,
                    _ => ADMIN_DEVELOPER_CREATE_MALFORMED,
                },
                _ => ADMIN_DEVELOPER_CREATE_MALFORMED,
            };
            params.insert(
                ADMIN_DEVELOPER_CREATE_STATE_PARAM.to_string(),
                state.to_string(),
            );
        }
    }
    // Wave 3a Track B — admin doesn't render the wallet dropdown yet,
    // so the BFF just plumbs the default `ConnectedWalletState`. The
    // type is here so Track A's MainLayout can read `ctx.wallet`
    // uniformly; admin pages ignore it for now.
    let ctx = PageContext {
        user: user.clone(),
        path: path.clone(),
        query: query.clone(),
        params,
        api_url: state.api_url.clone(),
        demo_login_enabled: state.demo_login_enabled,
        wallet: epsx_dioxus_ui::auth::wallet_button::ConnectedWalletState::default(),
    };

    // Use the dedicated admin dispatcher regardless of `is_admin` so the
    // admin's own auth middleware (if installed) can decide. The frontend
    // BFF will have the same UX.
    //
    // Wave 38b T2 — also derive the `layout_path` (the path with
    // the `/admin` prefix stripped) and pass it as the
    // `AdminLayout::Auth`'s `current_path`. The `default_no_layout_
    // paths()` registry uses the un-prefixed path (e.g.
    // `/access-denied`) so the layout's `is_no_layout` check
    // (`current_path == *p || current_path.starts_with(p)`) only
    // matches when we pass the stripped path. Previously the BFF
    // passed the raw `/admin/access-denied` path which made the
    // check fail — the AuthGate overlay then masked the
    // red-shield Access Denied panel and ballooned the
    // pixel-diff to ~99%.
    let (meta, body_element, layout_path) = if let Some(stripped) = strip_single_admin_prefix(&path)
    {
        let stripped = stripped.to_string();
        let mut c = ctx.clone();
        c.path = stripped.clone();
        let (m, b) = admin_pages::dispatch(&c);
        (m, b, stripped)
    } else {
        let (m, b) = render_page(&ctx, true);
        (m, b, path.clone())
    };

    // Wave 3a Track C — wrap the page body in `AdminLayout::Auth` so the
    // admin shell chrome is rendered by the layout, not by each page.
    //
    // The admin BFF does not yet plumb a server user into the layout —
    // the cookie-based session check happens higher in the request
    // lifecycle. Until Track B's `wallet` field lands on `PageContext`
    // we pass a default `ConnectedWalletState` (no wallet dropdown for
    // admin yet) and let the layout's `is_authenticated` default to
    // `false` — pages still get the chrome and the AuthGate will
    // overlay when needed.
    //
    // Wave 38b T2 — `no_layout_paths` extension. The 3 outlier
    // routes (`/access-denied`, `/unauthorized`,
    // `/developer-portal/api-keys/create`) render the SAME SSR
    // "Access Denied" panel in prod (verified by owner probe
    // 2026-06-18) — there is NO admin sidebar / header / footer
    // on those pages. The 2 first routes are already in the
    // shared `default_no_layout_paths()`; we add the 3rd here so
    // the dev BFF strips the chrome and the AuthGate overlay
    // (which would otherwise mask the centered Access Denied
    // panel and balloon the pixel-diff to ~99% per Wave 24 T1'
    // report).
    let server_user: Option<ServerUser> = user.as_ref().map(|u| ServerUser {
        id: u.id.clone(),
        address: u.address.clone(),
        chain_id: u.chain_id.clone(),
        email: u.email.clone().unwrap_or_default(),
        name: u.display_name.clone(),
        role: u.roles.first().cloned().unwrap_or_default(),
        tier: u.tier.clone(),
        permissions: u.permissions.clone(),
    });
    let is_authenticated = user.is_some();
    let shell_layout_path = safe_admin_layout_path(&layout_path, is_authenticated);
    let mut no_layout_paths = vec![
        "/auth".to_string(),
        "/login".to_string(),
        "/unauthorized".to_string(),
        "/access-denied".to_string(),
        "/permissions/policies".to_string(),
    ];
    if !is_authenticated {
        no_layout_paths.push("/developer-portal/api-keys/create".to_string());
    }
    let no_layout_paths_override = Some(no_layout_paths);
    // Production has one normal-route composition:
    // `AuthLayout -> MainLayout -> page body`. Page modules must remain
    // body-only so every route receives identical height, overflow, header,
    // sidebar, footer, and authenticated-wallet inputs. The dispatcher owns
    // the route-aware signed-out overlay; disabling the generic layout gate
    // prevents a second auth composition from being stacked over it.
    let body_element = if meta.status == PageStatus::NotFound {
        body_element
    } else {
        AdminLayout::Auth {
            current_path: shell_layout_path,
            server_user,
            is_authenticated,
            is_gated: Some(false),
            no_layout_paths: no_layout_paths_override,
        }
        .render(body_element, None, None, None)
    };

    let body_html = dioxus_ssr::render_element(body_element);
    let status = match meta.status {
        PageStatus::Ok => axum::http::StatusCode::OK,
        PageStatus::NotFound => axum::http::StatusCode::NOT_FOUND,
    };

    let doc = epsx_templates::page_shell_with_body_class_and_keywords(
        &meta.title,
        &meta.description,
        meta.keywords.as_deref(),
        "",
        &body_html,
        meta.include_footer,
        // Wave 38c T1 — body_class is now Option<String>. None
        // means "no body class override beyond the page shell's
        // default `min-h-screen`". The 3 admin outliers
        // (`/access-denied`, `/unauthorized`,
        // `/developer-portal/api-keys/create`) set their own body
        // class via `PageMeta::admin_with_body_class(...)` to
        // mirror prod's `h-screen overflow-hidden font-sans`
        // wrapper.
        meta.body_class.as_deref().unwrap_or(""),
    );

    let recovery_runtime = if recover_session {
        r#"<span hidden data-epsx-session-recovery="true" data-epsx-action="session-recover"></span>"#.to_string()
    } else {
        Default::default()
    };
    let doc = doc.replace("</body>", &format!("{recovery_runtime}</body>"));

    let mut response = private_admin_html_response(status, doc);
    if route_path == "/developer-portal/api-keys/create"
        && headers
            .get(header::COOKIE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| {
                value.split(';').any(|cookie| {
                    cookie
                        .trim_start()
                        .starts_with(super::ADMIN_DEVELOPER_SECRET_COOKIE)
                })
            })
    {
        response.headers_mut().append(
            header::SET_COOKIE,
            HeaderValue::from_static(
                "epsx.admin.developer_secret_once=; Path=/developer-portal; Max-Age=0; HttpOnly; SameSite=Lax",
            ),
        );
    }
    if route_path == "/notifications/create"
        && headers
            .get(header::COOKIE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| {
                value.split(';').any(|cookie| {
                    cookie
                        .trim_start()
                        .starts_with(super::ADMIN_NOTIFICATION_CREATE_COOKIE)
                })
            })
    {
        response.headers_mut().append(
            header::SET_COOKIE,
            HeaderValue::from_static(
                "epsx.admin.notification_create=; Path=/notifications/create; Max-Age=0; HttpOnly; SameSite=Lax",
            ),
        );
    }
    if notification_send_flash_clear {
        response.headers_mut().append(
            header::SET_COOKIE,
            HeaderValue::from_static(
                "epsx.admin.notification_send=; Path=/notifications; Max-Age=0; HttpOnly; SameSite=Lax",
            ),
        );
    }
    response
}

/// Keep security-sensitive dynamic chat, news, wallet, and plan references out
/// of signed-out shell breadcrumbs and return URLs. The dispatcher still
/// receives the real path so authenticated routing can resolve it normally.
fn safe_admin_layout_path(layout_path: &str, is_authenticated: bool) -> String {
    if !is_authenticated {
        if let Some(reference) = layout_path.strip_prefix("/chat/") {
            if !reference.is_empty() && !reference.contains('/') {
                return "/chat".to_string();
            }
        }

        if let Some(rest) = layout_path.strip_prefix("/news/") {
            if let Some((reference, suffix)) = rest.split_once('/') {
                if !reference.is_empty() && suffix == "edit" {
                    return "/news".to_string();
                }
            }
        }

        if let Some(reference) = layout_path.strip_prefix("/wallet-management/access/plans/") {
            if !reference.is_empty() && !reference.contains('/') {
                return "/wallet-management/access/plans".to_string();
            }
        }

        if let Some(rest) = layout_path.strip_prefix("/wallet-management/wallets/") {
            if let Some((reference, suffix)) = rest.split_once('/') {
                if !reference.is_empty() && suffix == "disable" {
                    return "/wallet-management/wallets".to_string();
                }
            }
        }

        if let Some(reference) = layout_path.strip_prefix("/wallet-management/") {
            if !reference.is_empty()
                && !reference.contains('/')
                && !matches!(reference, "wallets" | "credits" | "access")
            {
                return "/wallet-management/wallets".to_string();
            }
        }
    }

    layout_path.to_string()
}

#[cfg(test)]
mod tests {
    //! Smoke tests for Wave 3a Track C — verify that the admin BFF
    //! wraps page bodies in `AdminLayout::Auth` (which renders the
    //! `Header` component with the `admin-header` class).
    //!
    //! The full BFF render path is async/axum-bound; we exercise the
    //! thin render-only path (construct a `PageContext`, dispatch the
    //! page, wrap in `AdminLayout::Auth`, serialize) to confirm the
    //! chrome is present.

    use super::*;
    use epsx_dioxus_ui::{
        auth::{user::AuthMethod, User},
        pages::admin_pages::audit_log::{AdminAuditList, AdminAuditSummary},
        pages::admin_pages::dashboard::AdminDashboardUserStatus,
        pages::admin_pages::media::{AdminMediaList, AdminMediaObject},
        pages::admin_pages::news::{AdminNewsArticleSummary, AdminNewsList},
        pages::admin_pages::notifications::{
            AdminNotificationList, AdminNotificationMetrics, AdminNotificationSummary,
        },
        pages::admin_pages::wallet_wallets::AdminWalletStatsSummary,
        pages::PageContext,
    };

    fn build_ctx(path: &str) -> PageContext {
        PageContext {
            user: None,
            path: path.to_string(),
            query: String::new(),
            params: HashMap::new(),
            api_url: String::new(),
            demo_login_enabled: true,
            // Wave 3a Track B — `PageContext` carries a
            // `ConnectedWalletState` so layouts can read `ctx.wallet`
            // uniformly. Admin pages ignore the wallet field, so the
            // test helper just plugs in a default.
            wallet: epsx_dioxus_ui::auth::wallet_button::ConnectedWalletState::default(),
        }
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

    fn payment_payload(items: Vec<serde_json::Value>, total: i64) -> serde_json::Value {
        serde_json::json!({ "items": items, "total": total })
    }

    fn payment_item(id: &str) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "chain_id": "56",
            "payer": "0x1111111111111111111111111111111111111111",
            "payee": "0x2222222222222222222222222222222222222222",
            "amount": "1000000000000000000",
            "token_address": "0x3333333333333333333333333333333333333333",
            "status": "pending",
            "escrow_id": null,
            "tx_hash": null,
            "description": null,
            "expires_at": null,
            "created_at": "2026-07-22T10:00:00Z",
            "updated_at": "2026-07-22T10:00:00Z"
        })
    }

    fn admin_media_query() -> AdminMediaQuery {
        AdminMediaQuery::from_raw("bucket=public").unwrap()
    }

    fn admin_media_payload(items: Vec<AdminMediaObject>) -> AdminMediaList {
        AdminMediaList { items }
    }

    fn admin_media_item() -> AdminMediaObject {
        AdminMediaObject {
            key: "brand/banner.png".to_string(),
            size: 42,
            last_modified: Some("2026-07-22T10:00:00Z".to_string()),
        }
    }

    fn news_query() -> AdminNewsQuery {
        AdminNewsQuery::from_raw("page=2&status=published").unwrap()
    }

    fn news_payload(articles: Vec<AdminNewsArticleSummary>, total: i64) -> AdminNewsList {
        AdminNewsList {
            articles,
            total,
            page: 2,
            limit: 20,
        }
    }

    fn news_item() -> AdminNewsArticleSummary {
        AdminNewsArticleSummary {
            id: "2f68f1aa-08d7-4b40-a25f-b35e7fd0ed31".to_string(),
            title: "Migration status".to_string(),
            slug: "migration-status".to_string(),
            summary: Some("A backend-authoritative update".to_string()),
            status: "published".to_string(),
            tags: vec!["migration".to_string()],
            published_at: Some("2026-07-22T03:04:05Z".to_string()),
            created_at: "2026-07-21T03:04:05Z".to_string(),
            updated_at: "2026-07-22T03:04:05Z".to_string(),
            is_pinned: true,
        }
    }

    fn admin_notification_query() -> AdminNotificationQuery {
        AdminNotificationQuery::from_raw("page=2").unwrap()
    }

    fn admin_notification_payload(
        items: Vec<AdminNotificationSummary>,
        total: i64,
    ) -> AdminNotificationList {
        AdminNotificationList {
            items,
            total,
            limit: 20,
            offset: 20,
        }
    }

    fn admin_notification_metrics() -> AdminNotificationMetrics {
        AdminNotificationMetrics {
            queue_depth: 2,
            queue_age_seconds: Some(1),
            suppressed: 0,
            retry_wait: 0,
            terminal_failed: 0,
            dead_lettered: 0,
            provider_accepted: 1,
            attempting: 0,
            channel_outcomes: std::collections::BTreeMap::from([(String::from("in_app"), 2)]),
            provider_events: 1,
            delivery_attempts: 1,
            replay_cursors: 1,
            replay_cursor_age_seconds: Some(1),
            active_streams: 1,
            stream_connections_total: 1,
            stream_reconnects_total: 0,
            stream_replayed_events_total: 0,
            stream_lag_seconds: Some(1),
            stream_query_failures_total: 0,
        }
    }

    fn admin_notification_item() -> AdminNotificationSummary {
        AdminNotificationSummary {
            id: "0x0123456789abcdef0123456789abcdef".to_string(),
            title: Some("Migration delivery".to_string()),
            subject: Some("Read-only notification inventory".to_string()),
            channel: "in_app".to_string(),
            status: "sent".to_string(),
            notification_type: Some("system".to_string()),
            priority: Some("normal".to_string()),
            sent_at: Some("2026-07-22T10:00:00Z".to_string()),
            created_at: "2026-07-22T09:59:00Z".to_string(),
        }
    }

    fn admin_audit_query() -> AdminAuditQuery {
        AdminAuditQuery::from_raw("category=system").unwrap()
    }

    fn admin_audit_payload(items: Vec<AdminAuditSummary>) -> AdminAuditList {
        AdminAuditList {
            has_more: !items.is_empty(),
            next_cursor: (!items.is_empty()).then(|| "cursor_token_2".to_string()),
            items,
        }
    }

    fn admin_audit_item() -> AdminAuditSummary {
        AdminAuditSummary {
            id: "00000000-0000-0000-0000-000000000002".to_string(),
            category: "system".to_string(),
            action: "settings.updated".to_string(),
            resource_type: "settings".to_string(),
            effect: "success".to_string(),
            occurred_at: "2026-07-22T12:00:00Z".to_string(),
        }
    }

    /// Render a page through the admin BFF render path (without
    /// `page_shell_with_body_class`) so we can assert on the
    /// layout-wrapped HTML in isolation.
    fn render_admin_html(path: &str) -> String {
        render_admin_html_with_user(path, None)
    }

    fn render_admin_html_with_user(path: &str, user: Option<User>) -> String {
        let mut ctx = build_ctx(path);
        ctx.user = user.clone();
        let admin_path = strip_single_admin_prefix(path).unwrap_or(path).to_string();
        let mut c = ctx.clone();
        c.path = admin_path;
        let (meta, body) = admin_pages::dispatch(&c);
        let server_user = user.as_ref().map(|user| ServerUser {
            id: user.id.clone(),
            address: user.address.clone(),
            chain_id: user.chain_id.clone(),
            email: user.email.clone().unwrap_or_default(),
            name: user.display_name.clone(),
            role: user.roles.first().cloned().unwrap_or_default(),
            tier: user.tier.clone(),
            permissions: user.permissions.clone(),
        });
        let is_authenticated = user.is_some();
        let shell_layout_path = safe_admin_layout_path(&c.path, is_authenticated);
        // Wave 38b T2 — mirror the production `no_layout_paths`
        // override from `ssr_handler` so the test exercises the
        // same render path as the live BFF (the 3 outliers skip
        // the chrome + AuthGate).
        let no_layout_paths_override = Some(vec![
            "/login".to_string(),
            "/unauthorized".to_string(),
            "/access-denied".to_string(),
            "/permissions/policies".to_string(),
            "/developer-portal/api-keys/create".to_string(),
        ]);
        let body = if meta.status == PageStatus::NotFound {
            body
        } else {
            AdminLayout::Auth {
                current_path: shell_layout_path,
                server_user,
                is_authenticated,
                is_gated: Some(false),
                no_layout_paths: no_layout_paths_override,
            }
            .render(body, None, None, None)
        };
        dioxus_ssr::render_element(body)
    }

    #[test]
    fn notification_send_flash_requires_matching_cookie_and_query() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("epsx.admin.notification_send=accepted"),
        );
        assert_eq!(
            consume_notification_send_flash(&headers, "send=accepted&page=1"),
            (Some(ADMIN_NOTIFICATIONS_SEND_ACCEPTED), true)
        );
        assert_eq!(
            consume_notification_send_flash(&headers, "send=error"),
            (None, true)
        );
        assert_eq!(
            consume_notification_send_flash(&headers, "send=accepted&send=accepted"),
            (None, true)
        );

        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("epsx.admin.notification_send=unexpected"),
        );
        assert_eq!(
            consume_notification_send_flash(&headers, "send=accepted"),
            (None, true)
        );
        let empty = HeaderMap::new();
        assert_eq!(
            consume_notification_send_flash(&empty, "send=accepted"),
            (None, false)
        );
    }

    #[test]
    fn wallet_plan_mutation_query_is_single_valued_and_closed() {
        for state in [
            "success",
            "conflict",
            "forbidden",
            "unavailable",
            "malformed",
        ] {
            assert_eq!(
                wallet_plan_mutation_query(&format!("mutation={state}")),
                Some(state)
            );
        }
        for query in [
            "",
            "mutation=unknown",
            "mutation=conflict&mutation=conflict",
            "mutation=conflict&plan=private",
            "plan=private",
        ] {
            assert_eq!(wallet_plan_mutation_query(query), None, "{query}");
        }
    }

    #[test]
    fn admin_dashboard_renders_with_admin_header() {
        let user = User {
            id: "admin-session".to_string(),
            address: "0x1234".to_string(),
            chain_id: "56".to_string(),
            roles: vec![],
            email: None,
            tier: None,
            permissions: vec![],
            last_login_at: None,
            auth_method: AuthMethod::Siwe,
            display_name: None,
        };
        let html = render_admin_html_with_user("/admin", Some(user));
        // Every normal route uses the same BFF-owned production shell.
        assert!(
            html.contains("admin-header"),
            "expected authenticated admin dashboard HTML to include the shared shell header; got: {}",
            html
        );
        assert_eq!(html.matches("class=\"admin-sidebar ").count(), 1);
        assert!(!html.contains("class=\"admin-shell admin-shell-page\""));
    }

    #[test]
    fn wallet_inventory_keeps_bff_owned_shell() {
        let user = User {
            id: "admin-session".to_string(),
            address: "0x1234".to_string(),
            chain_id: "56".to_string(),
            roles: vec![],
            email: None,
            tier: None,
            permissions: vec![],
            last_login_at: None,
            auth_method: AuthMethod::Siwe,
            display_name: None,
        };
        let html = render_admin_html_with_user("/admin/wallet-management/wallets", Some(user));

        assert_eq!(
            html.matches("class=\"admin-sidebar ").count(),
            1,
            "wallet inventory must remain body-only inside one BFF shell: {html}"
        );
    }

    #[test]
    fn authenticated_admin_chrome_variants_expose_shared_logout_hook() {
        let user = User {
            id: "admin-session".to_string(),
            address: "0x1234".to_string(),
            chain_id: "56".to_string(),
            roles: vec![],
            email: None,
            tier: None,
            permissions: vec![],
            last_login_at: None,
            auth_method: AuthMethod::Siwe,
            display_name: None,
        };
        for path in ["/admin", "/admin/wallet-management/wallets"] {
            let html = render_admin_html_with_user(path, Some(user.clone()));
            assert!(
                html.contains("data-epsx-logout=\"true\""),
                "authenticated admin SSR chrome must expose the shared logout hook for {path}: {html}"
            );
        }
    }

    #[test]
    fn admin_mount_prefix_is_boundary_aware_and_removed_only_once() {
        assert_eq!(strip_single_admin_prefix("/admin"), Some("/"));
        assert_eq!(
            strip_single_admin_prefix("/admin/wallet-management/wallets"),
            Some("/wallet-management/wallets")
        );
        assert_eq!(
            strip_single_admin_prefix("/admin/admin/wallet-management/wallets"),
            Some("/admin/wallet-management/wallets")
        );
        assert_eq!(strip_single_admin_prefix("/administrator"), None);

        let mut ctx = build_ctx("/admin/admin/wallet-management/wallets");
        ctx.path = strip_single_admin_prefix(&ctx.path).unwrap().to_string();
        let (meta, body) = admin_pages::dispatch(&ctx);
        assert_eq!(meta.status, PageStatus::NotFound);
        let rendered = dioxus_ssr::render_element(body);
        assert!(!rendered.contains("data-admin-wallets-state"));
        assert!(!rendered.contains("data-admin-wallet-stats"));
    }

    #[test]
    fn audit_log_authenticated_render_has_exactly_one_admin_sidebar() {
        let user = User {
            id: "admin-session".to_string(),
            address: "0x1234".to_string(),
            chain_id: "56".to_string(),
            roles: vec![],
            email: None,
            tier: None,
            permissions: vec![],
            last_login_at: None,
            auth_method: AuthMethod::Siwe,
            display_name: None,
        };
        let html = render_admin_html_with_user("/admin/audit-log", Some(user));

        assert!(html.contains("data-audit-log-state=\"unavailable\""));
        assert_eq!(
            html.matches("class=\"admin-sidebar ").count(),
            1,
            "audit-log must rely on the BFF admin layout instead of nesting a second shell: {html}"
        );
    }

    #[test]
    fn media_authenticated_render_has_exactly_one_bff_admin_sidebar() {
        let user = User {
            id: "admin-session".to_string(),
            address: "0x1234".to_string(),
            chain_id: "56".to_string(),
            roles: vec![],
            email: None,
            tier: None,
            permissions: vec![],
            last_login_at: None,
            auth_method: AuthMethod::Siwe,
            display_name: None,
        };
        let html = render_admin_html_with_user("/admin/media", Some(user));

        assert!(html.contains("data-admin-media-state=\"unavailable\""));
        assert_eq!(
            html.matches("class=\"admin-sidebar ").count(),
            1,
            "media must rely on the BFF AdminLayout::Auth instead of owning another shell: {html}"
        );
    }

    #[test]
    fn media_signed_out_bff_render_has_one_route_aware_gate_and_safe_return_url() {
        let html = render_admin_html("/admin/media");

        assert_eq!(
            html.matches("data-wave25-t3-marker=\"auth-page-overlay\"")
                .count(),
            1,
            "media must render exactly one route-aware auth overlay: {html}"
        );
        assert!(html.contains("href=\"/auth?return_url=%2Fmedia\""));
        assert!(html.contains("data-return-url=\"/media\""));
        assert!(!html.contains("class=\"auth-gate "));
        assert!(!html.contains("the admin dashboard"));
    }

    #[test]
    fn signed_out_dynamic_full_layout_hides_private_route_references() {
        for (path, safe_return_url) in [
            ("/admin/chat/private-case-reference", "/chat"),
            ("/admin/news/private-case-reference/edit", "/news"),
            (
                "/admin/wallet-management/private-case-reference",
                "/wallet-management/wallets",
            ),
            (
                "/admin/wallet-management/wallets/private-case-reference/disable",
                "/wallet-management/wallets",
            ),
            (
                "/admin/wallet-management/access/plans/private-case-reference",
                "/wallet-management/access/plans",
            ),
        ] {
            let html = render_admin_html(path);

            assert!(!html.contains("private-case-reference"), "{path}: {html}");
            assert!(
                html.contains(&format!("data-return-url=\"{safe_return_url}\"")),
                "{path}: {html}"
            );
        }
    }

    #[test]
    fn authenticated_layout_keeps_dynamic_route_references() {
        for path in [
            "/chat/conversation-1",
            "/news/article-1/edit",
            "/wallet-management/0xabc",
            "/wallet-management/wallets/0xabc/disable",
            "/wallet-management/access/plans/pro",
        ] {
            assert_eq!(safe_admin_layout_path(path, true), path);
        }
    }

    #[test]
    fn payment_load_records_ready_with_only_typed_payload() {
        let mut params = HashMap::new();
        record_payment_intent_load(
            &mut params,
            Ok(payment_payload(vec![payment_item("intent-1")], 1)),
        );
        assert_eq!(
            params.get(ADMIN_PAYMENTS_STATE_PARAM).map(String::as_str),
            Some(ADMIN_PAYMENTS_READY)
        );
        let stored: serde_json::Value =
            serde_json::from_str(params.get(ADMIN_PAYMENTS_DATA_PARAM).unwrap()).unwrap();
        assert_eq!(stored["items"][0]["id"], "intent-1");
        assert_eq!(stored["total"], 1);
    }

    #[test]
    fn payment_load_records_authoritative_empty() {
        let mut params = HashMap::new();
        record_payment_intent_load(&mut params, Ok(payment_payload(vec![], 0)));
        assert_eq!(
            params.get(ADMIN_PAYMENTS_STATE_PARAM).map(String::as_str),
            Some(ADMIN_PAYMENTS_EMPTY)
        );
        assert!(params.contains_key(ADMIN_PAYMENTS_DATA_PARAM));
    }

    #[test]
    fn payment_load_keeps_nonzero_total_empty_page_ready_for_recovery() {
        let mut params = HashMap::new();
        record_payment_intent_load(&mut params, Ok(payment_payload(vec![], 41)));
        assert_eq!(
            params.get(ADMIN_PAYMENTS_STATE_PARAM).map(String::as_str),
            Some(ADMIN_PAYMENTS_READY)
        );
        let stored: serde_json::Value =
            serde_json::from_str(params.get(ADMIN_PAYMENTS_DATA_PARAM).unwrap()).unwrap();
        assert_eq!(stored["total"], 41);
    }

    #[test]
    fn payment_load_records_malformed_without_payload() {
        let mut params = HashMap::from([(
            ADMIN_PAYMENTS_DATA_PARAM.to_string(),
            "stale-sensitive-data".to_string(),
        )]);
        record_payment_intent_load(
            &mut params,
            Ok(serde_json::json!({ "items": [{ "id": "incomplete" }], "total": 1 })),
        );
        assert_eq!(
            params.get(ADMIN_PAYMENTS_STATE_PARAM).map(String::as_str),
            Some(ADMIN_PAYMENTS_MALFORMED)
        );
        assert!(!params.contains_key(ADMIN_PAYMENTS_DATA_PARAM));
    }

    #[test]
    fn payment_mutation_query_preserves_only_closed_states() {
        assert_eq!(payment_mutation_query("mutation=success"), Some("success"));
        assert_eq!(
            payment_mutation_query("tab=payment-links&mutation=conflict"),
            Some("conflict")
        );
        assert_eq!(
            payment_mutation_query("mutation=unknown"),
            Some("malformed")
        );
        assert_eq!(
            payment_mutation_query("mutation=success&mutation=conflict"),
            Some("malformed")
        );
        assert_eq!(payment_mutation_query("tab=payments"), None);
    }

    #[test]
    fn payment_load_records_unavailable_without_payload() {
        let mut params = HashMap::from([(
            ADMIN_PAYMENTS_DATA_PARAM.to_string(),
            "stale-sensitive-data".to_string(),
        )]);
        record_payment_intent_load(&mut params, Err(()));
        assert_eq!(
            params.get(ADMIN_PAYMENTS_STATE_PARAM).map(String::as_str),
            Some(ADMIN_PAYMENTS_UNAVAILABLE)
        );
        assert!(!params.contains_key(ADMIN_PAYMENTS_DATA_PARAM));
    }

    #[test]
    fn dashboard_user_status_load_records_only_the_safe_snapshot() {
        let mut params = HashMap::new();
        record_admin_dashboard_user_status_load(
            &mut params,
            AdminDashboardUserStatusLoad::Ready(AdminDashboardUserStatus {
                observed_at: "2026-07-23T03:04:04Z".to_string(),
                total_users: 11,
                active_users: 8,
            }),
        );

        assert_eq!(
            params
                .get(ADMIN_DASHBOARD_USER_STATUS_STATE_PARAM)
                .map(String::as_str),
            Some(ADMIN_DASHBOARD_USER_STATUS_READY)
        );
        let stored: serde_json::Value =
            serde_json::from_str(params.get(ADMIN_DASHBOARD_USER_STATUS_PARAM).unwrap()).unwrap();
        assert_eq!(stored["observed_at"], "2026-07-23T03:04:04Z");
        assert_eq!(stored["total_users"], 11);
        assert_eq!(stored["active_users"], 8);
        assert_eq!(stored.as_object().unwrap().len(), 3);
    }

    #[test]
    fn dashboard_user_status_failures_remove_stale_projection_data() {
        for (load, expected) in [
            (
                AdminDashboardUserStatusLoad::Forbidden,
                ADMIN_DASHBOARD_USER_STATUS_FORBIDDEN,
            ),
            (
                AdminDashboardUserStatusLoad::Unavailable,
                ADMIN_DASHBOARD_USER_STATUS_UNAVAILABLE,
            ),
            (
                AdminDashboardUserStatusLoad::Malformed,
                ADMIN_DASHBOARD_USER_STATUS_MALFORMED,
            ),
        ] {
            let mut params = HashMap::from([(
                ADMIN_DASHBOARD_USER_STATUS_PARAM.to_string(),
                "stale-dashboard-data".to_string(),
            )]);
            record_admin_dashboard_user_status_load(&mut params, load);
            assert_eq!(
                params
                    .get(ADMIN_DASHBOARD_USER_STATUS_STATE_PARAM)
                    .map(String::as_str),
                Some(expected)
            );
            assert!(!params.contains_key(ADMIN_DASHBOARD_USER_STATUS_PARAM));
        }
    }

    #[test]
    fn dashboard_loader_matches_only_exact_root_aliases_after_one_mount_strip() {
        for path in ["/", "/index", "/admin", "/admin/index"] {
            let route_path = strip_single_admin_prefix(path).unwrap_or(path);
            assert!(is_dashboard_user_status_route(route_path), "{path}");
        }
        for path in [
            "/dashboard",
            "/index/extra",
            "/admin/admin",
            "/admin/admin/index",
            "/administrator",
        ] {
            let route_path = strip_single_admin_prefix(path).unwrap_or(path);
            assert!(!is_dashboard_user_status_route(route_path), "{path}");
        }
    }

    #[test]
    fn admin_wallet_stats_load_records_only_the_safe_count_projection() {
        let mut params = HashMap::new();
        record_admin_wallet_stats_load(
            &mut params,
            AdminWalletStatsLoad::Ready(AdminWalletStatsSummary {
                total_users: 11,
                active_users: 8,
                inactive_users: 3,
                new_users_30_days: 2,
            }),
        );

        assert_eq!(
            params
                .get(ADMIN_WALLET_STATS_STATE_PARAM)
                .map(String::as_str),
            Some(ADMIN_WALLET_STATS_READY)
        );
        let stored: serde_json::Value =
            serde_json::from_str(params.get(ADMIN_WALLET_STATS_DATA_PARAM).unwrap()).unwrap();
        assert_eq!(stored["total_users"], 11);
        assert_eq!(stored["active_users"], 8);
        assert_eq!(stored["inactive_users"], 3);
        assert_eq!(stored["new_users_30_days"], 2);
        for forbidden in [
            "users_by_tier",
            "active_users_30_days",
            "growth_rate",
            "timestamp",
            "message",
            "performed_by",
            "metadata",
        ] {
            assert!(stored.get(forbidden).is_none(), "{forbidden}");
        }
    }

    #[test]
    fn admin_wallet_stats_failure_states_remove_stale_projection_data() {
        for (load, expected) in [
            (
                AdminWalletStatsLoad::Forbidden,
                ADMIN_WALLET_STATS_FORBIDDEN,
            ),
            (
                AdminWalletStatsLoad::Unavailable,
                ADMIN_WALLET_STATS_UNAVAILABLE,
            ),
            (
                AdminWalletStatsLoad::Malformed,
                ADMIN_WALLET_STATS_MALFORMED,
            ),
        ] {
            let mut params = HashMap::from([(
                ADMIN_WALLET_STATS_DATA_PARAM.to_string(),
                "stale-sensitive-wallet-stats".to_string(),
            )]);
            record_admin_wallet_stats_load(&mut params, load);
            assert_eq!(
                params
                    .get(ADMIN_WALLET_STATS_STATE_PARAM)
                    .map(String::as_str),
                Some(expected)
            );
            assert!(!params.contains_key(ADMIN_WALLET_STATS_DATA_PARAM));
        }
    }

    #[test]
    fn admin_media_load_records_only_the_projection_and_bucket() {
        let mut params = HashMap::new();
        record_admin_media_load(
            &mut params,
            &admin_media_query(),
            AdminMediaLoad::Ready(admin_media_payload(vec![admin_media_item()])),
        );

        assert_eq!(
            params.get(ADMIN_MEDIA_STATE_PARAM).map(String::as_str),
            Some(ADMIN_MEDIA_READY)
        );
        assert_eq!(
            params.get(ADMIN_MEDIA_BUCKET_PARAM).map(String::as_str),
            Some("public")
        );
        let stored: serde_json::Value =
            serde_json::from_str(params.get(ADMIN_MEDIA_DATA_PARAM).unwrap()).unwrap();
        assert_eq!(stored["items"][0]["key"], "brand/banner.png");
        assert_eq!(stored["items"][0]["size"], 42);
        assert_eq!(stored["items"][0]["last_modified"], "2026-07-22T10:00:00Z");
        assert!(stored["items"][0].get("url").is_none());
    }

    #[test]
    fn admin_media_load_records_authoritative_empty() {
        let mut params = HashMap::new();
        record_admin_media_load(
            &mut params,
            &admin_media_query(),
            AdminMediaLoad::Empty(admin_media_payload(Vec::new())),
        );

        assert_eq!(
            params.get(ADMIN_MEDIA_STATE_PARAM).map(String::as_str),
            Some(ADMIN_MEDIA_EMPTY)
        );
        assert_eq!(
            params.get(ADMIN_MEDIA_BUCKET_PARAM).map(String::as_str),
            Some("public")
        );
        assert!(params.contains_key(ADMIN_MEDIA_DATA_PARAM));
    }

    #[test]
    fn admin_media_failure_states_remove_stale_projection_and_normalize_bucket() {
        for (load, expected) in [
            (AdminMediaLoad::Forbidden, ADMIN_MEDIA_FORBIDDEN),
            (AdminMediaLoad::Unavailable, ADMIN_MEDIA_UNAVAILABLE),
            (AdminMediaLoad::Malformed, ADMIN_MEDIA_MALFORMED),
        ] {
            let mut params = HashMap::from([
                (
                    ADMIN_MEDIA_DATA_PARAM.to_string(),
                    "stale-sensitive-media-data".to_string(),
                ),
                (ADMIN_MEDIA_BUCKET_PARAM.to_string(), "chat".to_string()),
            ]);
            record_admin_media_load(&mut params, &admin_media_query(), load);
            assert_eq!(
                params.get(ADMIN_MEDIA_STATE_PARAM).map(String::as_str),
                Some(expected)
            );
            assert!(!params.contains_key(ADMIN_MEDIA_DATA_PARAM));
            assert_eq!(
                params.get(ADMIN_MEDIA_BUCKET_PARAM).map(String::as_str),
                Some("public")
            );
        }

        let mut malformed_query = HashMap::from([
            (
                ADMIN_MEDIA_DATA_PARAM.to_string(),
                "stale-sensitive-media-data".to_string(),
            ),
            (ADMIN_MEDIA_BUCKET_PARAM.to_string(), "chat".to_string()),
        ]);
        let default_query = AdminMediaQuery::from_raw("").unwrap();
        record_admin_media_load(
            &mut malformed_query,
            &default_query,
            AdminMediaLoad::Malformed,
        );
        assert_eq!(
            malformed_query
                .get(ADMIN_MEDIA_BUCKET_PARAM)
                .map(String::as_str),
            Some("news")
        );
        assert!(!malformed_query.contains_key(ADMIN_MEDIA_DATA_PARAM));
    }

    #[test]
    fn admin_news_load_records_only_the_projection_and_normalized_query() {
        let mut params = HashMap::new();
        record_admin_news_load(
            &mut params,
            &news_query(),
            AdminNewsLoad::Ready(news_payload(vec![news_item()], 41)),
        );

        assert_eq!(
            params.get(ADMIN_NEWS_STATE_PARAM).map(String::as_str),
            Some(ADMIN_NEWS_READY)
        );
        assert_eq!(
            params.get(ADMIN_NEWS_PAGE_PARAM).map(String::as_str),
            Some("2")
        );
        assert_eq!(
            params.get(ADMIN_NEWS_STATUS_PARAM).map(String::as_str),
            Some("published")
        );
        let stored: serde_json::Value =
            serde_json::from_str(params.get(ADMIN_NEWS_DATA_PARAM).unwrap()).unwrap();
        assert_eq!(stored["articles"][0]["slug"], "migration-status");
        assert_eq!(stored["total"], 41);
        for forbidden in ["content", "author_wallet", "cover_image_url", "pinned_at"] {
            assert!(
                stored["articles"][0].get(forbidden).is_none(),
                "{forbidden}"
            );
        }
    }

    #[test]
    fn admin_news_load_distinguishes_true_empty_from_an_out_of_range_page() {
        let query = news_query();
        let mut empty = HashMap::new();
        record_admin_news_load(
            &mut empty,
            &query,
            AdminNewsLoad::Empty(news_payload(Vec::new(), 0)),
        );
        assert_eq!(
            empty.get(ADMIN_NEWS_STATE_PARAM).map(String::as_str),
            Some(ADMIN_NEWS_EMPTY)
        );
        assert!(empty.contains_key(ADMIN_NEWS_DATA_PARAM));

        let mut recoverable = HashMap::new();
        record_admin_news_load(
            &mut recoverable,
            &query,
            AdminNewsLoad::Ready(news_payload(Vec::new(), 41)),
        );
        assert_eq!(
            recoverable.get(ADMIN_NEWS_STATE_PARAM).map(String::as_str),
            Some(ADMIN_NEWS_READY)
        );
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(
                recoverable.get(ADMIN_NEWS_DATA_PARAM).unwrap()
            )
            .unwrap()["total"],
            41
        );
    }

    #[test]
    fn admin_news_failure_states_remove_stale_projection_data() {
        for (load, expected) in [
            (AdminNewsLoad::Forbidden, ADMIN_NEWS_FORBIDDEN),
            (AdminNewsLoad::Unavailable, ADMIN_NEWS_UNAVAILABLE),
            (AdminNewsLoad::Malformed, ADMIN_NEWS_MALFORMED),
        ] {
            let mut params = HashMap::from([(
                ADMIN_NEWS_DATA_PARAM.to_string(),
                "stale-sensitive-news-data".to_string(),
            )]);
            record_admin_news_load(&mut params, &news_query(), load);
            assert_eq!(
                params.get(ADMIN_NEWS_STATE_PARAM).map(String::as_str),
                Some(expected)
            );
            assert!(!params.contains_key(ADMIN_NEWS_DATA_PARAM));
        }
    }

    #[test]
    fn admin_notification_load_records_only_the_safe_projection_and_page() {
        let mut params = HashMap::new();
        record_admin_notification_load(
            &mut params,
            &admin_notification_query(),
            AdminNotificationLoad::Ready(admin_notification_payload(
                vec![admin_notification_item()],
                21,
            )),
        );

        assert_eq!(
            params
                .get(ADMIN_NOTIFICATIONS_STATE_PARAM)
                .map(String::as_str),
            Some(ADMIN_NOTIFICATIONS_READY)
        );
        assert_eq!(
            params
                .get(ADMIN_NOTIFICATIONS_PAGE_PARAM)
                .map(String::as_str),
            Some("2")
        );
        let stored: serde_json::Value =
            serde_json::from_str(params.get(ADMIN_NOTIFICATIONS_DATA_PARAM).unwrap()).unwrap();
        assert_eq!(stored["items"][0]["channel"], "in_app");
        assert_eq!(stored["total"], 21);
        for forbidden in [
            "user_id",
            "recipient",
            "template_id",
            "body",
            "message",
            "data",
            "error",
            "read_at",
            "action_url",
        ] {
            assert!(stored["items"][0].get(forbidden).is_none(), "{forbidden}");
        }
    }

    #[test]
    fn admin_notification_metrics_load_records_bounded_observations_or_explicit_failure() {
        let mut ready = HashMap::from([(
            ADMIN_NOTIFICATION_METRICS_DATA_PARAM.to_string(),
            "stale-metrics".to_string(),
        )]);
        record_admin_notification_metrics_load(
            &mut ready,
            AdminNotificationMetricsLoad::Ready(admin_notification_metrics()),
        );
        assert_eq!(
            ready
                .get(ADMIN_NOTIFICATION_METRICS_STATE_PARAM)
                .map(String::as_str),
            Some(ADMIN_NOTIFICATIONS_READY)
        );
        let stored: serde_json::Value =
            serde_json::from_str(ready.get(ADMIN_NOTIFICATION_METRICS_DATA_PARAM).unwrap())
                .unwrap();
        assert_eq!(stored["queue_depth"], 2);
        assert_eq!(stored["channel_outcomes"]["in_app"], 2);

        let mut failed = HashMap::from([(
            ADMIN_NOTIFICATION_METRICS_DATA_PARAM.to_string(),
            "stale-metrics".to_string(),
        )]);
        record_admin_notification_metrics_load(
            &mut failed,
            AdminNotificationMetricsLoad::Malformed,
        );
        assert_eq!(
            failed
                .get(ADMIN_NOTIFICATION_METRICS_STATE_PARAM)
                .map(String::as_str),
            Some(ADMIN_NOTIFICATIONS_MALFORMED)
        );
        assert!(!failed.contains_key(ADMIN_NOTIFICATION_METRICS_DATA_PARAM));
    }

    #[test]
    fn admin_notification_load_distinguishes_empty_from_out_of_range_recovery() {
        let query = admin_notification_query();
        let mut empty = HashMap::new();
        record_admin_notification_load(
            &mut empty,
            &query,
            AdminNotificationLoad::Empty(admin_notification_payload(Vec::new(), 0)),
        );
        assert_eq!(
            empty
                .get(ADMIN_NOTIFICATIONS_STATE_PARAM)
                .map(String::as_str),
            Some(ADMIN_NOTIFICATIONS_EMPTY)
        );
        assert!(empty.contains_key(ADMIN_NOTIFICATIONS_DATA_PARAM));

        let mut recoverable = HashMap::new();
        record_admin_notification_load(
            &mut recoverable,
            &query,
            AdminNotificationLoad::Ready(admin_notification_payload(Vec::new(), 1)),
        );
        assert_eq!(
            recoverable
                .get(ADMIN_NOTIFICATIONS_STATE_PARAM)
                .map(String::as_str),
            Some(ADMIN_NOTIFICATIONS_READY)
        );
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(
                recoverable.get(ADMIN_NOTIFICATIONS_DATA_PARAM).unwrap()
            )
            .unwrap()["total"],
            1
        );
    }

    #[test]
    fn admin_notification_failure_states_remove_stale_projection_data() {
        for (load, expected) in [
            (
                AdminNotificationLoad::Forbidden,
                ADMIN_NOTIFICATIONS_FORBIDDEN,
            ),
            (
                AdminNotificationLoad::Unavailable,
                ADMIN_NOTIFICATIONS_UNAVAILABLE,
            ),
            (
                AdminNotificationLoad::Malformed,
                ADMIN_NOTIFICATIONS_MALFORMED,
            ),
        ] {
            let mut params = HashMap::from([(
                ADMIN_NOTIFICATIONS_DATA_PARAM.to_string(),
                "stale-sensitive-notification-data".to_string(),
            )]);
            record_admin_notification_load(&mut params, &admin_notification_query(), load);
            assert_eq!(
                params
                    .get(ADMIN_NOTIFICATIONS_STATE_PARAM)
                    .map(String::as_str),
                Some(expected)
            );
            assert!(!params.contains_key(ADMIN_NOTIFICATIONS_DATA_PARAM));
        }
    }

    #[test]
    fn admin_audit_load_records_only_the_redacted_projection_and_query() {
        let mut params = HashMap::new();
        record_admin_audit_load(
            &mut params,
            &admin_audit_query(),
            AdminAuditLoad::Ready(admin_audit_payload(vec![admin_audit_item()])),
        );

        assert_eq!(
            params.get(ADMIN_AUDIT_STATE_PARAM).map(String::as_str),
            Some(ADMIN_AUDIT_READY)
        );
        assert_eq!(
            params.get(ADMIN_AUDIT_CATEGORY_PARAM).map(String::as_str),
            Some("system")
        );
        assert!(!params.contains_key(ADMIN_AUDIT_CURSOR_PARAM));
        let stored: serde_json::Value =
            serde_json::from_str(params.get(ADMIN_AUDIT_DATA_PARAM).unwrap()).unwrap();
        assert_eq!(stored["items"][0]["action"], "settings.updated");
        for forbidden in [
            "actor",
            "actor_type",
            "resource_id",
            "ip_address",
            "user_agent",
            "before_state",
            "after_state",
            "metadata",
            "details",
            "total",
        ] {
            assert!(stored["items"][0].get(forbidden).is_none(), "{forbidden}");
        }
    }

    #[test]
    fn admin_audit_failure_states_remove_stale_projection_and_query_data() {
        for (load, expected) in [
            (AdminAuditLoad::Forbidden, ADMIN_AUDIT_FORBIDDEN),
            (AdminAuditLoad::Unavailable, ADMIN_AUDIT_UNAVAILABLE),
            (AdminAuditLoad::Malformed, ADMIN_AUDIT_MALFORMED),
        ] {
            let mut params = HashMap::from([
                (
                    ADMIN_AUDIT_DATA_PARAM.to_string(),
                    "stale-sensitive-audit-data".to_string(),
                ),
                (ADMIN_AUDIT_CATEGORY_PARAM.to_string(), "wallet".to_string()),
                (
                    ADMIN_AUDIT_CURSOR_PARAM.to_string(),
                    "stale_cursor".to_string(),
                ),
            ]);
            let default_query = AdminAuditQuery::from_raw("").unwrap();
            record_admin_audit_load(&mut params, &default_query, load);
            assert_eq!(
                params.get(ADMIN_AUDIT_STATE_PARAM).map(String::as_str),
                Some(expected)
            );
            assert!(!params.contains_key(ADMIN_AUDIT_DATA_PARAM));
            assert!(!params.contains_key(ADMIN_AUDIT_CATEGORY_PARAM));
            assert!(!params.contains_key(ADMIN_AUDIT_CURSOR_PARAM));
        }
    }

    #[test]
    fn admin_html_is_private_no_store_and_varies_by_session_credentials() {
        let response = private_admin_html_response(
            axum::http::StatusCode::OK,
            "private draft summary".to_string(),
        );
        assert_eq!(
            response
                .headers()
                .get(axum::http::header::CACHE_CONTROL)
                .and_then(|value| value.to_str().ok()),
            Some("private, no-store")
        );
        assert_eq!(
            response
                .headers()
                .get(axum::http::header::VARY)
                .and_then(|value| value.to_str().ok()),
            Some("Cookie, Authorization")
        );
    }
}

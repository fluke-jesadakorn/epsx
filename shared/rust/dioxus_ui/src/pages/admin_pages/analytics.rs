//! Authenticated, backend-projected analytics read state for `/analytics`.
//!
//! This leaf consumes only the strict JSON projection placed in
//! `PageContext.params` by the admin BFF. It never derives entitlement from
//! frontend roles, invents missing metrics, or emits mutation controls.

use chrono::DateTime;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::auth::AuthGate;
use crate::components::admin::data_state_banner::{AdminDataState, AdminDataStateBanner};
use crate::primitives::Icon;

use super::super::{PageContext, PageMeta};

const ANALYTICS_PATH: &str = "/analytics";
const MAX_COUNT: i64 = 9_000_000_000_000_000;
const MAX_OBSERVED_AT_CHARS: usize = 64;

pub const ADMIN_ANALYTICS_DATA_PARAM: &str = "data_admin_analytics";
pub const ADMIN_ANALYTICS_STATE_PARAM: &str = "data_admin_analytics_state";

pub const ADMIN_ANALYTICS_READY: &str = "ready";
pub const ADMIN_ANALYTICS_EMPTY: &str = "empty";
pub const ADMIN_ANALYTICS_FORBIDDEN: &str = "forbidden";
pub const ADMIN_ANALYTICS_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_ANALYTICS_MALFORMED: &str = "malformed";
pub const ADMIN_ANALYTICS_UNAUTHENTICATED: &str = "unauthenticated";
pub const ADMIN_ANALYTICS_UNAUTHORIZED: &str = "unauthorized";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminAnalyticsUserStats {
    pub total: i64,
    pub active: i64,
    pub today_connections: i64,
    pub total_users: i64,
    pub active_users: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminAnalyticsPermissionStats {
    pub total: i64,
    pub total_plans: i64,
    pub total_permissions: i64,
    pub active_permissions: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminAnalyticsPlanStats {
    pub total_plans: i64,
    pub active_plans: i64,
    pub total_memberships: i64,
    pub active_memberships: i64,
    pub recent_assignments: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminAnalyticsDeveloperStats {
    pub total_api_keys: i64,
    pub active_api_keys: i64,
}

/// A deliberately empty typed placeholder. The current backend does not
/// expose operational telemetry in this contract. A non-null telemetry block
/// is rejected until it has a reviewed, typed source rather than being shown
/// as an unverified health claim.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminAnalyticsSystemMetrics {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminAnalyticsSnapshot {
    #[serde(default)]
    pub observed_at: Option<String>,
    #[serde(default)]
    pub user_stats: Option<AdminAnalyticsUserStats>,
    #[serde(default)]
    pub permission_analytics: Option<AdminAnalyticsPermissionStats>,
    #[serde(default)]
    pub plan_stats: Option<AdminAnalyticsPlanStats>,
    #[serde(default)]
    pub system_metrics: Option<AdminAnalyticsSystemMetrics>,
    #[serde(default)]
    pub developer_portal: Option<AdminAnalyticsDeveloperStats>,
}

pub fn decode_admin_analytics_projection(
    value: serde_json::Value,
) -> Option<AdminAnalyticsSnapshot> {
    let projection: AdminAnalyticsSnapshot = serde_json::from_value(value).ok()?;
    if projection
        .observed_at
        .as_deref()
        .is_some_and(|observed_at| !valid_observed_at(observed_at))
        || projection
            .user_stats
            .as_ref()
            .is_some_and(|stats| !valid_user_stats(stats))
        || projection
            .permission_analytics
            .as_ref()
            .is_some_and(|stats| !valid_permission_stats(stats))
        || projection
            .plan_stats
            .as_ref()
            .is_some_and(|stats| !valid_plan_stats(stats))
        || projection
            .developer_portal
            .as_ref()
            .is_some_and(|stats| !valid_developer_stats(stats))
    {
        return None;
    }
    Some(projection)
}

fn valid_observed_at(value: &str) -> bool {
    !value.is_empty()
        && value.chars().count() <= MAX_OBSERVED_AT_CHARS
        && !value.chars().any(char::is_control)
        && DateTime::parse_from_rfc3339(value).is_ok()
}

fn valid_count(value: i64) -> bool {
    (0..=MAX_COUNT).contains(&value)
}

fn valid_user_stats(stats: &AdminAnalyticsUserStats) -> bool {
    [
        stats.total,
        stats.active,
        stats.today_connections,
        stats.total_users,
        stats.active_users,
    ]
    .into_iter()
    .all(valid_count)
        && stats.total == stats.total_users
        && stats.active == stats.active_users
        && stats.active <= stats.total
}

fn valid_permission_stats(stats: &AdminAnalyticsPermissionStats) -> bool {
    [
        stats.total,
        stats.total_plans,
        stats.total_permissions,
        stats.active_permissions,
    ]
    .into_iter()
    .all(valid_count)
        && stats.total == stats.total_permissions
        && stats.active_permissions <= stats.total_permissions
}

fn valid_plan_stats(stats: &AdminAnalyticsPlanStats) -> bool {
    [
        stats.total_plans,
        stats.active_plans,
        stats.total_memberships,
        stats.active_memberships,
        stats.recent_assignments,
    ]
    .into_iter()
    .all(valid_count)
        && stats.active_plans <= stats.total_plans
        && stats.active_memberships <= stats.total_memberships
        && stats.recent_assignments <= stats.total_memberships
}

fn valid_developer_stats(stats: &AdminAnalyticsDeveloperStats) -> bool {
    valid_count(stats.total_api_keys)
        && valid_count(stats.active_api_keys)
        && stats.active_api_keys <= stats.total_api_keys
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
enum AnalyticsLoad {
    Ready(AdminAnalyticsSnapshot),
    Empty(AdminAnalyticsSnapshot),
    Unauthenticated,
    Unauthorized,
    Forbidden,
    Unavailable,
    Malformed,
}

#[allow(dead_code)]
fn analytics_load(ctx: &PageContext) -> AnalyticsLoad {
    let state = ctx
        .params
        .get(ADMIN_ANALYTICS_STATE_PARAM)
        .map(String::as_str);
    match state {
        Some(ADMIN_ANALYTICS_READY) | Some(ADMIN_ANALYTICS_EMPTY) => {
            let Some(raw) = ctx.params.get(ADMIN_ANALYTICS_DATA_PARAM) else {
                return AnalyticsLoad::Malformed;
            };
            let Some(snapshot) = serde_json::from_str(raw)
                .ok()
                .and_then(decode_admin_analytics_projection)
            else {
                return AnalyticsLoad::Malformed;
            };
            let has_data = has_data(&snapshot);
            let has_observed_at = snapshot.observed_at.is_some();
            match (state, has_data, has_observed_at) {
                (Some(ADMIN_ANALYTICS_READY), true, true) => AnalyticsLoad::Ready(snapshot),
                (Some(ADMIN_ANALYTICS_EMPTY), false, true) => AnalyticsLoad::Empty(snapshot),
                _ => AnalyticsLoad::Malformed,
            }
        }
        Some(ADMIN_ANALYTICS_FORBIDDEN) => AnalyticsLoad::Forbidden,
        Some(ADMIN_ANALYTICS_MALFORMED) => AnalyticsLoad::Malformed,
        Some(ADMIN_ANALYTICS_UNAUTHENTICATED) => AnalyticsLoad::Unauthenticated,
        Some(ADMIN_ANALYTICS_UNAUTHORIZED) => AnalyticsLoad::Unauthorized,
        Some(ADMIN_ANALYTICS_UNAVAILABLE) | None => AnalyticsLoad::Unavailable,
        Some(_) => AnalyticsLoad::Malformed,
    }
}

#[allow(dead_code)]
fn has_data(snapshot: &AdminAnalyticsSnapshot) -> bool {
    snapshot.user_stats.is_some()
        || snapshot.permission_analytics.is_some()
        || snapshot.plan_stats.is_some()
        || snapshot.developer_portal.is_some()
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Analytics");
    (meta, rsx! { RenderAnalytics { ctx: ctx.clone() } })
}

#[component]
fn RenderAnalytics(ctx: PageContext) -> Element {
    let load = analytics_load(&ctx);
    let surface = match load {
        AnalyticsLoad::Unauthenticated | AnalyticsLoad::Unauthorized => {
            let state = if matches!(load, AnalyticsLoad::Unauthenticated) {
                AdminDataState::Unauthenticated
            } else {
                AdminDataState::Unauthorized
            };
            rsx! {
                AdminDataStateBanner {
                    state,
                    subject: "Analytics dashboard".to_string(),
                    return_path: ANALYTICS_PATH.to_string(),
                    retry_href: ANALYTICS_PATH.to_string(),
                }
            }
        }
        _ => crate::pages::analytics::render_surface(&ctx),
    };
    rsx! {
        AuthGate {
            user: ctx.user.clone(),
            feature: Some("the EPS growth analytics workspace".to_string()),
            return_url: Some(ANALYTICS_PATH.to_string()),
            {surface}
        }
    }
}

#[component]
#[allow(dead_code)]
fn AnalyticsSurface(
    state: &'static str,
    snapshot: Option<AdminAnalyticsSnapshot>,
    issue_title: Option<String>,
    issue_detail: Option<String>,
) -> Element {
    let observed_at = snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.observed_at.clone());
    let total_users = metric_value(
        snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.user_stats.as_ref())
            .map(|stats| stats.total_users),
    );
    let active_users = metric_value(
        snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.user_stats.as_ref())
            .map(|stats| stats.active_users),
    );
    let connections_today = metric_value(
        snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.user_stats.as_ref())
            .map(|stats| stats.today_connections),
    );
    let total_permissions = metric_value(
        snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.permission_analytics.as_ref())
            .map(|stats| stats.total_permissions),
    );
    let active_permissions = metric_value(
        snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.permission_analytics.as_ref())
            .map(|stats| stats.active_permissions),
    );
    let total_plans = metric_value(
        snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.plan_stats.as_ref())
            .map(|stats| stats.total_plans),
    );
    let active_plans = metric_value(
        snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.plan_stats.as_ref())
            .map(|stats| stats.active_plans),
    );
    let total_memberships = metric_value(
        snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.plan_stats.as_ref())
            .map(|stats| stats.total_memberships),
    );
    let active_memberships = metric_value(
        snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.plan_stats.as_ref())
            .map(|stats| stats.active_memberships),
    );
    let recent_assignments = metric_value(
        snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.plan_stats.as_ref())
            .map(|stats| stats.recent_assignments),
    );
    let total_api_keys = metric_value(
        snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.developer_portal.as_ref())
            .map(|stats| stats.total_api_keys),
    );
    let active_api_keys = metric_value(
        snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.developer_portal.as_ref())
            .map(|stats| stats.active_api_keys),
    );

    rsx! {
        div {
            class: "admin-analytics space-y-6",
            "data-admin-analytics-state": state,
            div { class: "flex flex-wrap items-center justify-between gap-3 rounded-xl border border-border/20 bg-card px-5 py-4 shadow-xl",
                div {
                    p { class: "text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground", "Analytics snapshot" }
                    p { class: "mt-1 text-sm font-semibold text-foreground", "Backend-authoritative platform projection" }
                }
                if let Some(observed_at) = observed_at {
                    time {
                        class: "font-mono text-xs text-muted-foreground",
                        datetime: observed_at.clone(),
                        "data-admin-analytics-freshness": "backend",
                        "Observed {observed_at}"
                    }
                } else {
                    span { class: "font-mono text-xs text-amber-400", "Snapshot unavailable" }
                }
            }

            if let (Some(title), Some(detail)) = (issue_title, issue_detail) {
                section { class: "rounded-xl border border-amber-500/30 bg-amber-500/10 px-5 py-4", role: "alert",
                    div { class: "flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between",
                        div {
                            h2 { class: "font-semibold text-foreground", "{title}" }
                            p { class: "mt-1 text-sm text-muted-foreground", "{detail}" }
                        }
                        nav { class: "flex shrink-0 gap-2", aria_label: "Analytics recovery",
                            a { class: "btn btn-sm btn-outline", href: ANALYTICS_PATH,
                                Icon { name: "refresh-cw".to_string(), size: Some(15) }
                                " Check again"
                            }
                            a { class: "btn btn-sm btn-ghost", href: "/", "Admin home" }
                        }
                    }
                }
            }

            div { class: "grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-4 sm:gap-6", aria_label: "Analytics summary",
                AnalyticsSummaryTile { title: "Total Users".to_string(), value: total_users.clone(), subtitle: "Registered platform accounts".to_string(), accent: "text-[#1fc7d4]".to_string() }
                AnalyticsSummaryTile { title: "API Requests".to_string(), value: "Unavailable".to_string(), subtitle: "No request-volume contract".to_string(), accent: "text-[#7645d9]".to_string() }
                AnalyticsSummaryTile { title: "Active Permissions".to_string(), value: active_permissions.clone(), subtitle: "Effective permission grants".to_string(), accent: "text-[#31d0aa]".to_string() }
                AnalyticsSummaryTile { title: "System Health".to_string(), value: "Unavailable".to_string(), subtitle: "No verified telemetry contract".to_string(), accent: "text-[#ed4b9e]".to_string() }
            }

            div { class: "grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3 sm:gap-6", aria_label: "Realtime analytics",
                AnalyticsStatusTile { title: "Active Users".to_string(), value: active_users.clone(), subtitle: "Accounts marked active".to_string(), icon: "users".to_string(), accent: "text-[#31d0aa] border-[#31d0aa]/20 bg-[#31d0aa]/10".to_string() }
                AnalyticsStatusTile { title: "Expiring Permissions".to_string(), value: "Unavailable".to_string(), subtitle: "Expiry summary is not exposed".to_string(), icon: "shield".to_string(), accent: "text-[#ffb237] border-[#ffb237]/20 bg-[#ffb237]/10".to_string() }
                AnalyticsStatusTile { title: "Response Time".to_string(), value: "Unavailable".to_string(), subtitle: "No verified latency telemetry".to_string(), icon: "activity".to_string(), accent: "text-[#1fc7d4] border-[#1fc7d4]/20 bg-[#1fc7d4]/10".to_string() }
            }

            div { class: "grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-4 sm:gap-6", aria_label: "Platform metrics",
                AnalyticsMetricTile { title: "Connections Today".to_string(), value: connections_today, subtitle: "Authenticated sessions".to_string(), icon: "activity".to_string(), accent: "text-[#1fc7d4] border-[#1fc7d4]/10 bg-[#1fc7d4]/10".to_string() }
                AnalyticsMetricTile { title: "Permissions".to_string(), value: total_permissions, subtitle: "Configured permission records".to_string(), icon: "shield".to_string(), accent: "text-[#31d0aa] border-[#31d0aa]/10 bg-[#31d0aa]/10".to_string() }
                AnalyticsMetricTile { title: "Active Plans".to_string(), value: active_plans.clone(), subtitle: "Plans marked active".to_string(), icon: "layers".to_string(), accent: "text-[#7645d9] border-[#7645d9]/10 bg-[#7645d9]/10".to_string() }
                AnalyticsMetricTile { title: "Active API Keys".to_string(), value: active_api_keys.clone(), subtitle: "Developer credentials".to_string(), icon: "key".to_string(), accent: "text-[#ed4b9e] border-[#ed4b9e]/10 bg-[#ed4b9e]/10".to_string() }
            }

            section { class: "overflow-hidden rounded-2xl border border-border/20 bg-card shadow-xl", aria_label: "API usage analytics",
                div { class: "border-b border-border/20 bg-muted/20 p-6 sm:p-8",
                    h2 { class: "text-lg font-black uppercase tracking-widest text-foreground", "API Usage Analytics" }
                }
                div { class: "grid gap-4 p-6 sm:grid-cols-2 sm:p-8",
                    AnalyticsDefinition { label: "Total API keys".to_string(), value: total_api_keys }
                    AnalyticsDefinition { label: "Active API keys".to_string(), value: active_api_keys }
                }
            }

            section { class: "overflow-hidden rounded-2xl border border-border/20 bg-card shadow-xl", aria_label: "Plan analytics",
                div { class: "border-b border-border/20 bg-muted/20 p-6 sm:p-8",
                    h2 { class: "text-lg font-black uppercase tracking-widest text-foreground", "Plan Analytics" }
                }
                div { class: "grid gap-4 p-6 sm:grid-cols-2 lg:grid-cols-5 sm:p-8",
                    AnalyticsDefinition { label: "Total plans".to_string(), value: total_plans }
                    AnalyticsDefinition { label: "Active plans".to_string(), value: active_plans }
                    AnalyticsDefinition { label: "Memberships".to_string(), value: total_memberships }
                    AnalyticsDefinition { label: "Active memberships".to_string(), value: active_memberships }
                    AnalyticsDefinition { label: "Recent assignments".to_string(), value: recent_assignments }
                }
            }
        }
    }
}

#[component]
#[allow(dead_code)]
fn AnalyticsSummaryTile(title: String, value: String, subtitle: String, accent: String) -> Element {
    rsx! {
        article { class: "rounded-2xl border border-border/20 bg-card p-6 shadow-xl sm:p-8",
            p { class: "text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground", "{title}" }
            p { class: "mt-3 break-words text-3xl font-black tracking-tight {accent}", "{value}" }
            p { class: "mt-2 text-xs text-muted-foreground", "{subtitle}" }
        }
    }
}

#[component]
#[allow(dead_code)]
fn AnalyticsStatusTile(
    title: String,
    value: String,
    subtitle: String,
    icon: String,
    accent: String,
) -> Element {
    rsx! {
        article { class: "overflow-hidden rounded-2xl border border-border/20 bg-card p-6 shadow-xl",
            div { class: "flex items-start justify-between gap-4",
                div {
                    p { class: "text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground", "{title}" }
                    p { class: "mt-3 break-words text-2xl font-black tracking-tight text-foreground", "{value}" }
                    p { class: "mt-1 text-xs text-muted-foreground", "{subtitle}" }
                }
                span { class: "inline-flex rounded-2xl border p-3 {accent}", aria_hidden: "true",
                    Icon { name: icon, size: Some(22) }
                }
            }
        }
    }
}

#[component]
#[allow(dead_code)]
fn AnalyticsMetricTile(
    title: String,
    value: String,
    subtitle: String,
    icon: String,
    accent: String,
) -> Element {
    rsx! {
        article { class: "overflow-hidden rounded-2xl border border-border/20 bg-card p-6 shadow-xl sm:p-8",
            div { class: "flex items-center gap-5",
                span { class: "inline-flex shrink-0 rounded-2xl border p-3 {accent}", aria_hidden: "true",
                    Icon { name: icon, size: Some(22) }
                }
                div { class: "min-w-0",
                    p { class: "text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground", "{title}" }
                    p { class: "mt-1 text-xl font-black tracking-tight text-foreground 2xl:text-2xl", "{value}" }
                    p { class: "mt-1 text-xs text-muted-foreground", "{subtitle}" }
                }
            }
        }
    }
}

#[component]
#[allow(dead_code)]
fn AnalyticsDefinition(label: String, value: String) -> Element {
    rsx! {
        dl { class: "rounded-xl border border-border/20 bg-background/40 p-4",
            dt { class: "text-xs font-semibold text-muted-foreground", "{label}" }
            dd { class: "mt-2 break-words text-xl font-black text-foreground", "{value}" }
        }
    }
}

#[allow(dead_code)]
fn metric_value(value: Option<i64>) -> String {
    value
        .map(format_count)
        .unwrap_or_else(|| "Unavailable".to_string())
}

#[allow(dead_code)]
fn format_count(value: i64) -> String {
    let digits = value.to_string();
    let mut formatted = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, character) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            formatted.push(',');
        }
        formatted.push(character);
    }
    formatted
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::auth::user::{AuthMethod, User};
    use crate::pages::analytics::{ANALYTICS_DATA_PARAM, ANALYTICS_STATE_PARAM};
    use serde_json::json;

    fn signed_in_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "analytics-session".to_string(),
                address: "0x1234abcd5678ef90".to_string(),
                chain_id: "56".to_string(),
                roles: vec![],
                email: None,
                tier: None,
                permissions: vec![],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: None,
            }),
            path: ANALYTICS_PATH.to_string(),
            ..Default::default()
        }
    }

    fn html(ctx: &PageContext) -> String {
        let (_, element) = render(ctx);
        dioxus_ssr::render_element(element)
    }

    fn ready_snapshot() -> AdminAnalyticsSnapshot {
        AdminAnalyticsSnapshot {
            observed_at: Some("2026-07-27T00:00:00Z".to_string()),
            user_stats: Some(AdminAnalyticsUserStats {
                total: 120,
                active: 100,
                today_connections: 12,
                total_users: 120,
                active_users: 100,
            }),
            permission_analytics: None,
            plan_stats: None,
            system_metrics: None,
            developer_portal: None,
        }
    }

    fn ranking_payload() -> serde_json::Value {
        json!({
            "success": true,
            "data": [{
                "rank": 1,
                "symbol": "NVDA",
                "company_name": "NVIDIA Corporation",
                "latest_date": "2026-06-30",
                "value": 215.31,
                "active_status": "TRACK",
                "quarterly_performance": [{
                    "quarter": "Q2",
                    "date": "2026-06-30",
                    "price": 215.31,
                    "eps": 2.5,
                    "eps_growth": 35.98,
                    "price_growth": 4.0,
                    "announcement_date": "Jul 20, 2026",
                    "announcement_timestamp": 1784505600,
                    "is_estimated": false
                }],
                "next_quarter_estimate": null,
                "next_earnings_date": null,
                "last_earnings_date": null,
                "next_earnings_date_formatted": null,
                "days_until_next_earnings": null,
                "progress_percentage": 50.0,
                "current_eps": 2.5,
                "growth_factor": 35.98,
                "price_current": 215.31
            }],
            "pagination": {"page":1,"limit":10,"total":1,"totalPages":1,"hasNext":false,"hasPrev":false},
            "metadata": {"available_countries":[],"available_sectors":[],"request_timestamp":"2026-07-27T00:00:00Z","data_source":"analytics"},
            "access_info": {"min_accessible_rank":1,"locked_ranks_count":0},
            "message": null,
            "processing_time_ms": 12
        })
    }

    fn with_ranking_state(
        mut ctx: PageContext,
        state: &str,
        data: Option<serde_json::Value>,
    ) -> PageContext {
        ctx.params
            .insert(ANALYTICS_STATE_PARAM.to_string(), state.to_string());
        if let Some(data) = data {
            ctx.params.insert(
                ANALYTICS_DATA_PARAM.to_string(),
                serde_json::to_string(&data).unwrap(),
            );
        }
        ctx
    }

    #[test]
    fn unauthenticated_and_unauthorized_states_decode_to_the_new_variants() {
        let mut unauthenticated = signed_in_ctx();
        unauthenticated.params.insert(
            ADMIN_ANALYTICS_STATE_PARAM.to_string(),
            ADMIN_ANALYTICS_UNAUTHENTICATED.to_string(),
        );
        assert_eq!(
            analytics_load(&unauthenticated),
            AnalyticsLoad::Unauthenticated
        );

        let mut unauthorized = signed_in_ctx();
        unauthorized.params.insert(
            ADMIN_ANALYTICS_STATE_PARAM.to_string(),
            ADMIN_ANALYTICS_UNAUTHORIZED.to_string(),
        );
        assert_eq!(analytics_load(&unauthorized), AnalyticsLoad::Unauthorized);
    }

    #[test]
    fn decoder_accepts_only_bounded_backend_projection() {
        assert!(
            decode_admin_analytics_projection(serde_json::to_value(ready_snapshot()).unwrap())
                .is_some()
        );

        let mut unknown = serde_json::to_value(ready_snapshot()).unwrap();
        unknown["sample_series"] = json!([1, 2, 3]);
        assert!(decode_admin_analytics_projection(unknown).is_none());

        let mut negative = serde_json::to_value(ready_snapshot()).unwrap();
        negative["user_stats"]["active"] = json!(-1);
        assert!(decode_admin_analytics_projection(negative).is_none());

        let mut fabricated_telemetry = serde_json::to_value(ready_snapshot()).unwrap();
        fabricated_telemetry["system_metrics"] = json!({"health_percentage": 99.9});
        assert!(decode_admin_analytics_projection(fabricated_telemetry).is_none());
    }

    #[test]
    fn signed_out_route_keeps_analytics_projection_private() {
        let rendered = html(&PageContext {
            path: ANALYTICS_PATH.to_string(),
            ..Default::default()
        });
        assert!(rendered.contains("Sign in required"));
        assert!(!rendered.contains("data-analytics-state"));
        assert!(!rendered.contains("120"));
    }

    #[test]
    fn ready_projection_renders_the_production_ranking_workspace() {
        let rendered = html(&with_ranking_state(
            signed_in_ctx(),
            "ready",
            Some(ranking_payload()),
        ));
        assert!(rendered.contains("data-analytics-state=\"ready\""));
        for section in [
            "Analytics",
            "Top-performing stocks by EPS growth",
            "Rankings access",
            "Country",
            "Sector",
            "NVDA",
            "NVIDIA Corporation",
        ] {
            assert!(
                rendered.contains(section),
                "missing production section {section}"
            );
        }
        for forbidden in [
            "Permission required",
            "admin:analytics:view",
            "Upgrade",
            "Export",
            "Save",
            "Delete",
            "Analytics Dashboard",
            "API Usage Analytics",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "unsupported analytics UI: {forbidden}"
            );
        }
    }

    #[test]
    fn ranking_failure_states_keep_the_production_composition_and_fail_closed() {
        for state in ["unavailable", "malformed"] {
            let rendered = html(&with_ranking_state(signed_in_ctx(), state, None));
            assert!(rendered.contains(&format!("data-analytics-state=\"{state}\"")));
            assert!(rendered.contains("Analytics"));
            assert!(rendered.contains("Country"));
            assert!(rendered.contains("Sector"));
            assert!(rendered.contains(if state == "malformed" {
                "Ranking data could not be validated"
            } else {
                "Rankings are temporarily unavailable"
            }));
            assert!(!rendered.contains("NVDA"));
        }

        let mut hostile = signed_in_ctx();
        hostile.query = "role=admin&permission=admin:analytics:view".to_string();
        hostile.params = HashMap::from([(
            ADMIN_ANALYTICS_DATA_PARAM.to_string(),
            "{\"sample_series\":[\"HOSTILE\"]}".to_string(),
        )]);
        let rendered = html(&hostile);
        assert!(rendered.contains("data-analytics-state=\"unavailable\""));
        assert!(!rendered.contains("HOSTILE"));
        assert!(!rendered.contains("admin:analytics:view"));
    }

    #[test]
    fn authenticated_page_is_body_only_with_production_header_and_safe_links() {
        let rendered = html(&signed_in_ctx());
        assert!(rendered.contains("Analytics"));
        assert!(rendered.contains("Top-performing stocks by EPS growth"));
        assert!(!rendered.contains("class=\"admin-shell admin-shell-page\""));
        assert!(rendered.contains("action=\"/analytics\""));
        assert!(!rendered.contains("javascript:"));
        assert!(!rendered.contains("onclick="));
    }
}

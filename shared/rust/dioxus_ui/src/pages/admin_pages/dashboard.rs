//! `/` (plus the target-only `/index` alias) — authenticated admin
//! command-center snapshot.
//!
//! The page accepts strict server projections for user status and platform
//! analytics. It never invents operational health, uptime, or activity; each
//! visible count comes from a backend-owned read model.

use chrono::DateTime;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::auth::AuthGate;
use crate::components::admin::page_layout::{PageLayout, PageMaxWidth};
use crate::primitives::Icon;

use super::super::{PageContext, PageMeta};
use super::analytics::{
    decode_admin_analytics_projection, AdminAnalyticsSnapshot, ADMIN_ANALYTICS_DATA_PARAM,
    ADMIN_ANALYTICS_EMPTY, ADMIN_ANALYTICS_FORBIDDEN, ADMIN_ANALYTICS_MALFORMED,
    ADMIN_ANALYTICS_READY, ADMIN_ANALYTICS_STATE_PARAM, ADMIN_ANALYTICS_UNAVAILABLE,
};

pub const ADMIN_DASHBOARD_USER_STATUS_PARAM: &str = "data_admin_dashboard_user_status";
pub const ADMIN_DASHBOARD_USER_STATUS_STATE_PARAM: &str = "data_admin_dashboard_user_status_state";

pub const ADMIN_DASHBOARD_USER_STATUS_READY: &str = "ready";
pub const ADMIN_DASHBOARD_USER_STATUS_FORBIDDEN: &str = "forbidden";
pub const ADMIN_DASHBOARD_USER_STATUS_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_DASHBOARD_USER_STATUS_MALFORMED: &str = "malformed";

const MAX_OBSERVED_AT_CHARS: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminDashboardUserStatus {
    pub observed_at: String,
    pub total_users: i64,
    pub active_users: i64,
}

/// Decode the exact root-dashboard projection. Zero is authoritative ready
/// data; negative or internally inconsistent counts are not.
pub fn decode_admin_dashboard_user_status(
    value: serde_json::Value,
) -> Option<AdminDashboardUserStatus> {
    let projection: AdminDashboardUserStatus = serde_json::from_value(value).ok()?;
    if projection.total_users < 0
        || projection.active_users < 0
        || projection.active_users > projection.total_users
        || projection.observed_at.is_empty()
        || projection.observed_at.chars().count() > MAX_OBSERVED_AT_CHARS
        || projection.observed_at.chars().any(char::is_control)
        || DateTime::parse_from_rfc3339(&projection.observed_at).is_err()
    {
        return None;
    }
    Some(projection)
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum DashboardLoad {
    Ready(AdminDashboardUserStatus),
    Forbidden,
    Unavailable,
    Malformed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum DashboardOverviewLoad {
    Ready(AdminAnalyticsSnapshot),
    Empty(AdminAnalyticsSnapshot),
    Forbidden,
    Unavailable,
    Malformed,
}

fn dashboard_overview_load(ctx: &PageContext) -> DashboardOverviewLoad {
    let state = ctx
        .params
        .get(ADMIN_ANALYTICS_STATE_PARAM)
        .map(String::as_str);
    match state {
        Some(ADMIN_ANALYTICS_READY) | Some(ADMIN_ANALYTICS_EMPTY) => {
            let Some(raw) = ctx.params.get(ADMIN_ANALYTICS_DATA_PARAM) else {
                return DashboardOverviewLoad::Malformed;
            };
            let Some(snapshot) = serde_json::from_str(raw)
                .ok()
                .and_then(decode_admin_analytics_projection)
            else {
                return DashboardOverviewLoad::Malformed;
            };
            let has_data = snapshot.user_stats.is_some()
                || snapshot.permission_analytics.is_some()
                || snapshot.plan_stats.is_some()
                || snapshot.developer_portal.is_some();
            if snapshot.observed_at.is_none() {
                return DashboardOverviewLoad::Malformed;
            }
            match (state, has_data) {
                (Some(ADMIN_ANALYTICS_READY), true) => DashboardOverviewLoad::Ready(snapshot),
                (Some(ADMIN_ANALYTICS_EMPTY), false) => DashboardOverviewLoad::Empty(snapshot),
                _ => DashboardOverviewLoad::Malformed,
            }
        }
        Some(ADMIN_ANALYTICS_FORBIDDEN) => DashboardOverviewLoad::Forbidden,
        Some(ADMIN_ANALYTICS_MALFORMED) => DashboardOverviewLoad::Malformed,
        Some(ADMIN_ANALYTICS_UNAVAILABLE) | None => DashboardOverviewLoad::Unavailable,
        Some(_) => DashboardOverviewLoad::Malformed,
    }
}

fn dashboard_load(ctx: &PageContext) -> DashboardLoad {
    match ctx
        .params
        .get(ADMIN_DASHBOARD_USER_STATUS_STATE_PARAM)
        .map(String::as_str)
    {
        Some(ADMIN_DASHBOARD_USER_STATUS_READY) => {
            let Some(raw) = ctx.params.get(ADMIN_DASHBOARD_USER_STATUS_PARAM) else {
                return DashboardLoad::Malformed;
            };
            serde_json::from_str(raw)
                .ok()
                .and_then(decode_admin_dashboard_user_status)
                .map(DashboardLoad::Ready)
                .unwrap_or(DashboardLoad::Malformed)
        }
        Some(ADMIN_DASHBOARD_USER_STATUS_FORBIDDEN) => DashboardLoad::Forbidden,
        Some(ADMIN_DASHBOARD_USER_STATUS_MALFORMED) => DashboardLoad::Malformed,
        Some(ADMIN_DASHBOARD_USER_STATUS_UNAVAILABLE) | None => DashboardLoad::Unavailable,
        Some(_) => DashboardLoad::Malformed,
    }
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Command Center");
    (meta, rsx! { RenderDashboard { ctx: ctx.clone() } })
}

#[component]
fn RenderDashboard(ctx: PageContext) -> Element {
    let load = dashboard_load(&ctx);
    let overview_load = dashboard_overview_load(&ctx);
    let snapshot_observed_at = match &overview_load {
        DashboardOverviewLoad::Ready(snapshot) | DashboardOverviewLoad::Empty(snapshot) => {
            snapshot.observed_at.clone()
        }
        _ => match &load {
            DashboardLoad::Ready(projection) => Some(projection.observed_at.clone()),
            _ => None,
        },
    };
    let overview = match &overview_load {
        DashboardOverviewLoad::Ready(snapshot) | DashboardOverviewLoad::Empty(snapshot) => {
            Some(snapshot.clone())
        }
        _ => None,
    };

    rsx! {
        AuthGate {
            user: ctx.user.clone(),
            feature: Some("the private admin command center".to_string()),
            return_url: Some("/".to_string()),
            PageLayout {
                max_width: Some(PageMaxWidth::Full),
                div { class: "admin-dashboard max-w-[1600px] mx-auto w-full @container pb-12",
                    DashboardPulseHeader {
                        observed_at: snapshot_observed_at,
                        overview: overview.clone(),
                    }
                    DashboardHudMetrics {
                        overview: overview.clone(),
                        user_status: match &load {
                            DashboardLoad::Ready(projection) => Some(projection.clone()),
                            _ => None,
                        },
                    }
                    div { class: "grid grid-cols-1 gap-6 xl:grid-cols-4",
                        div { class: "xl:col-span-3",
                            div { class: "mb-4 flex items-center justify-between",
                                h2 { class: "text-sm font-bold text-muted-foreground uppercase tracking-widest",
                                    "Operational Modules"
                                }
                            }
                            DashboardBentoTools {}
                        }
                        div { class: "h-full xl:col-span-1",
                            div { class: "mb-4 flex items-center justify-between",
                                h2 { class: "text-sm font-bold text-muted-foreground uppercase tracking-widest hidden xl:block opacity-0",
                                    "Global Event Stream"
                                }
                            }
                            DashboardActivityStream { overview: overview.clone() }
                        }
                    }

                    match overview_load {
                        DashboardOverviewLoad::Ready(_) => rsx! {},
                        DashboardOverviewLoad::Empty(snapshot) => rsx! {
                            DashboardOverviewProblem {
                                state: ADMIN_ANALYTICS_EMPTY,
                                title: "No platform analytics are recorded yet".to_string(),
                                detail: format!(
                                    "The backend returned an authoritative empty snapshot observed at {}.",
                                    snapshot.observed_at.unwrap_or_default(),
                                ),
                            }
                        },
                        DashboardOverviewLoad::Forbidden => rsx! {
                            DashboardOverviewProblem {
                                state: ADMIN_ANALYTICS_FORBIDDEN,
                                title: "Dashboard metrics access was denied".to_string(),
                                detail: "This session can open the dashboard but cannot read the platform analytics projection.".to_string(),
                            }
                        },
                        DashboardOverviewLoad::Unavailable => rsx! {
                            DashboardOverviewProblem {
                                state: ADMIN_ANALYTICS_UNAVAILABLE,
                                title: "Dashboard metrics are unavailable".to_string(),
                                detail: "The backend analytics projection could not be reached. No metric has been replaced with a synthetic value.".to_string(),
                            }
                        },
                        DashboardOverviewLoad::Malformed => rsx! {
                            DashboardOverviewProblem {
                                state: ADMIN_ANALYTICS_MALFORMED,
                                title: "Dashboard metrics could not be verified".to_string(),
                                detail: "The backend response did not match the strict analytics contract.".to_string(),
                            }
                        },
                    }

                    match load {
                        DashboardLoad::Ready(projection) => rsx! {
                            DashboardReady { projection }
                        },
                        DashboardLoad::Forbidden => rsx! {
                            DashboardProblem {
                                state: ADMIN_DASHBOARD_USER_STATUS_FORBIDDEN,
                                title: "Dashboard snapshot access was denied".to_string(),
                                detail: "The backend did not authorize this session to read the user-status snapshot. No counts are being shown.".to_string(),
                            }
                        },
                        DashboardLoad::Unavailable => rsx! {
                            DashboardProblem {
                                state: ADMIN_DASHBOARD_USER_STATUS_UNAVAILABLE,
                                title: "Dashboard snapshot is unavailable".to_string(),
                                detail: "The backend could not provide an authoritative user-status snapshot. No counts are being shown.".to_string(),
                            }
                        },
                        DashboardLoad::Malformed => rsx! {
                            DashboardProblem {
                                state: ADMIN_DASHBOARD_USER_STATUS_MALFORMED,
                                title: "Dashboard snapshot could not be verified".to_string(),
                                detail: "The backend response did not match the strict user-status contract. No counts are being shown.".to_string(),
                            }
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn DashboardPulseHeader(
    observed_at: Option<String>,
    overview: Option<AdminAnalyticsSnapshot>,
) -> Element {
    let (state_label, state_class, timestamp) = match observed_at {
        Some(value) => ("BACKEND CONNECTED", "text-emerald-400", value),
        None => (
            "BACKEND DATA UNAVAILABLE",
            "text-amber-300",
            "No verified snapshot timestamp".to_string(),
        ),
    };
    let signal_count = overview
        .as_ref()
        .map(|snapshot| {
            [
                snapshot.user_stats.is_some(),
                snapshot.permission_analytics.is_some(),
                snapshot.plan_stats.is_some(),
                snapshot.developer_portal.is_some(),
            ]
            .into_iter()
            .filter(|present| *present)
            .count()
        })
        .unwrap_or(0);
    let response = if overview.is_some() {
        "Verified"
    } else {
        "Unavailable"
    };
    let availability = if overview.is_some() {
        "Ready"
    } else {
        "Unavailable"
    };

    rsx! {
        section {
            class: "relative mb-6 overflow-hidden rounded-2xl border border-border/20 bg-card/80 shadow-2xl backdrop-blur-xl",
            "data-admin-dashboard-surface": "pulse-header",
            div { class: "absolute inset-0 bg-gradient-to-r from-blue-500/5 via-purple-500/5 to-cyan-500/5", aria_hidden: "true" }
            div { class: "absolute left-0 top-0 h-px w-full bg-gradient-to-r from-transparent via-cyan-500/50 to-transparent opacity-50", aria_hidden: "true" }
            div { class: "relative flex flex-col items-start justify-between gap-6 p-6 sm:p-8 md:flex-row md:items-center",
                div {
                    div { class: "mb-2 flex flex-wrap items-center gap-3",
                        h1 { class: "text-3xl font-black tracking-tight text-foreground sm:text-4xl", "Command Center" }
                        span { class: "rounded-full border border-border/30 bg-background/50 px-3 py-1 text-xs font-bold uppercase tracking-wider {state_class}",
                            "{state_label}"
                        }
                    }
                    p { class: "flex flex-wrap items-center gap-3 font-mono text-sm text-muted-foreground",
                        "{timestamp}"
                        span { class: "text-border/50", "|" }
                        span { class: "flex items-center gap-1.5",
                            Icon { name: "activity".to_string(), size: Some(14), class_name: Some("text-cyan-400".to_string()) }
                            "Pulse status is backend-owned"
                        }
                    }
                }
                div { class: "flex shrink-0 items-center divide-x divide-border/30 rounded-xl border border-border/20 bg-background/50 p-2 backdrop-blur-md",
                    DashboardPulseMetric { label: "Response", value: response.to_string(), class_name: "text-cyan-400".to_string() }
                    DashboardPulseMetric { label: "Availability", value: availability.to_string(), class_name: "text-primary".to_string() }
                    DashboardPulseMetric { label: "Signals", value: format!("{signal_count} sources"), class_name: "text-amber-300".to_string() }
                }
            }
        }
    }
}

#[component]
fn DashboardPulseMetric(label: &'static str, value: String, class_name: String) -> Element {
    rsx! {
        div { class: "px-4 py-1 text-center",
            div { class: "mb-1 text-[10px] font-bold uppercase tracking-widest text-muted-foreground", "{label}" }
            div { class: "font-mono text-xs font-bold {class_name}", "{value}" }
        }
    }
}

#[component]
fn DashboardHudMetrics(
    overview: Option<AdminAnalyticsSnapshot>,
    user_status: Option<AdminDashboardUserStatus>,
) -> Element {
    let user_stats = overview
        .as_ref()
        .and_then(|snapshot| snapshot.user_stats.as_ref());
    let permission_stats = overview
        .as_ref()
        .and_then(|snapshot| snapshot.permission_analytics.as_ref());
    let total_wallets = user_stats
        .map(|stats| stats.total)
        .or_else(|| user_status.as_ref().map(|status| status.total_users));
    let active_wallets = user_stats
        .map(|stats| stats.active)
        .or_else(|| user_status.as_ref().map(|status| status.active_users));
    let daily_connections = user_stats.map(|stats| stats.today_connections);
    let active_permissions = permission_stats.map(|stats| stats.active_permissions);
    rsx! {
        div { class: "mb-6 grid grid-cols-2 gap-4 lg:grid-cols-4", "data-admin-dashboard-surface": "hud-metrics",
            DashboardMetricCard {
                label: "Total Wallets",
                value: metric_value(total_wallets),
                subtext: "Registered wallet records",
                icon: "wallet",
                accent: "text-cyan-400",
                border: "border-cyan-500/30",
            }
            DashboardMetricCard {
                label: "Active Wallets",
                value: metric_value(active_wallets),
                subtext: "Accounts marked active",
                icon: "activity",
                accent: "text-emerald-400",
                border: "border-emerald-500/30",
            }
            DashboardMetricCard {
                label: "Daily Connections",
                value: metric_value(daily_connections),
                subtext: "Authenticated in the last 24h",
                icon: "users",
                accent: "text-pink-400",
                border: "border-pink-500/30",
            }
            DashboardMetricCard {
                label: "Active Permissions",
                value: metric_value(active_permissions),
                subtext: "Effective permission grants",
                icon: "clock",
                accent: "text-amber-300",
                border: "border-amber-500/30",
            }
        }
    }
}

#[component]
fn DashboardMetricCard(
    label: &'static str,
    value: String,
    subtext: &'static str,
    icon: &'static str,
    accent: &'static str,
    border: &'static str,
) -> Element {
    rsx! {
        div { class: "relative overflow-hidden rounded-xl border {border} bg-card/60 p-5 backdrop-blur-md",
            div { class: "mb-4 flex items-center justify-between",
                span { class: "whitespace-nowrap text-[10px] font-bold uppercase tracking-[0.2em] text-muted-foreground", "{label}" }
                div { class: "rounded-lg border border-white/5 bg-background/50 p-2 shadow-inner {accent}",
                    Icon { name: icon.to_string(), size: Some(20) }
                }
            }
            div { class: "mb-1 break-words font-mono text-xl font-black tracking-tight {accent} sm:text-2xl", "{value}" }
            div { class: "flex items-center gap-1.5 text-xs font-medium text-muted-foreground opacity-80",
                span { class: "block h-1 w-1 rounded-full bg-current opacity-50" }
                "{subtext}"
            }
        }
    }
}

#[component]
fn DashboardBentoTools() -> Element {
    rsx! {
        div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 auto-rows-[220px]", "data-admin-dashboard-surface": "bento-tools",
            DashboardToolCard {
                href: "/wallet-management/wallets",
                title: "Wallet Database",
                description: "Deep inspect connected wallets, view connection history, and manage active sessions.",
                icon: "wallet",
                span: "lg:col-span-2",
                accent: "text-cyan-400",
            }
            DashboardToolCard {
                href: "/wallet-management/access",
                title: "Security & Perms",
                description: "Critical access control and permission surfaces remain backend-authorized.",
                icon: "shield",
                span: "row-span-2",
                accent: "text-purple-400",
            }
            DashboardToolCard {
                href: "/audit-log",
                title: "Global Audit Log",
                description: "Review the immutable history of administrative actions when the service is available.",
                icon: "file-text",
                span: "",
                accent: "text-pink-400",
            }
            DashboardToolCard {
                href: "/notifications/manage",
                title: "Broadcast Hub",
                description: "Manage critical system notices and global updates through the notifications workspace.",
                icon: "bell",
                span: "",
                accent: "text-amber-300",
            }
            DashboardToolCard {
                href: "/developer-portal",
                title: "Dev Infrastructure",
                description: "Manage API keys, integrations, and webhooks when backend data is available.",
                icon: "database",
                span: "lg:col-span-2",
                accent: "text-emerald-400",
            }
            DashboardToolCard {
                href: "/settings",
                title: "Settings",
                description: "Core platform configuration and policy controls.",
                icon: "settings",
                span: "",
                accent: "text-slate-400",
            }
        }
    }
}

#[component]
fn DashboardToolCard(
    href: &'static str,
    title: &'static str,
    description: &'static str,
    icon: &'static str,
    span: &'static str,
    accent: &'static str,
) -> Element {
    rsx! {
        a { href: href, class: "group relative flex flex-col overflow-hidden rounded-2xl border border-border/20 bg-card/60 backdrop-blur-md transition-all duration-300 hover:border-white/20 hover:shadow-xl {span}",
            div { class: "absolute inset-0 bg-gradient-to-br from-white/5 to-transparent opacity-60", aria_hidden: "true" }
            div { class: "relative z-10 flex h-full flex-col p-5 sm:p-6",
                div { class: "mb-4 flex items-start justify-between gap-3",
                    div { class: "rounded-xl border border-white/5 bg-background/50 p-3 {accent}",
                        Icon { name: icon.to_string(), size: Some(24) }
                    }
                    span { class: "rounded-full border border-border/30 bg-background/50 px-3 py-1 text-[10px] font-mono uppercase tracking-widest text-muted-foreground",
                        "Open module"
                    }
                }
                div { class: "mt-auto",
                    h3 { class: "mb-2 text-xl font-black tracking-tight text-foreground sm:text-2xl", "{title}" }
                    p { class: "line-clamp-3 text-sm font-medium leading-relaxed text-muted-foreground", "{description}" }
                }
            }
        }
    }
}

#[component]
fn DashboardActivityStream(overview: Option<AdminAnalyticsSnapshot>) -> Element {
    let signals = overview.as_ref().map(dashboard_signals).unwrap_or_default();
    rsx! {
        section { class: "flex h-full min-h-[420px] flex-col overflow-hidden rounded-2xl border border-border/20 bg-card shadow-2xl", "data-admin-dashboard-surface": "activity-stream",
            header { class: "flex items-center justify-between border-b border-border/20 bg-muted/20 p-4",
                div { class: "flex items-center gap-3",
                    span { class: "relative flex h-3 w-3",
                        span { class: "absolute inline-flex h-full w-full rounded-full bg-cyan-400 opacity-50" }
                        span { class: "relative inline-flex h-3 w-3 rounded-full bg-cyan-500" }
                    }
                    h2 { class: "font-mono text-xs font-black uppercase tracking-[0.2em] text-cyan-400", "Platform Signals" }
                }
                Icon { name: "refresh-cw".to_string(), size: Some(16), class_name: Some("text-muted-foreground".to_string()) }
            }
            div { class: "relative flex flex-1 bg-background/50 p-4 font-mono text-sm text-muted-foreground",
                if signals.is_empty() {
                    div { class: "m-auto text-center",
                        Icon { name: "wifi-off".to_string(), size: Some(24), class_name: Some("mx-auto mb-3 text-amber-300".to_string()) }
                        p { class: "font-semibold uppercase tracking-widest", "SIGNALS UNAVAILABLE" }
                        p { class: "mt-2 text-xs leading-5", "No verified platform signal is available." }
                    }
                } else {
                    ul { class: "w-full space-y-2", role: "list",
                        for (label, value, detail) in signals {
                            li { class: "rounded-lg border border-border/20 bg-card/50 p-3",
                                div { class: "flex items-center justify-between gap-3",
                                    span { class: "text-[10px] font-bold uppercase tracking-widest", "{label}" }
                                    strong { class: "text-cyan-400", "{value}" }
                                }
                                p { class: "mt-1 text-[11px] leading-4", "{detail}" }
                            }
                        }
                    }
                }
            }
            footer { class: "border-t border-border/20 bg-background/80 p-2 text-center text-[10px] font-mono uppercase tracking-widest text-muted-foreground",
                "AUTHORITATIVE SNAPSHOT / READ ONLY"
            }
        }
    }
}

fn metric_value(value: Option<i64>) -> String {
    value
        .map(format_count)
        .unwrap_or_else(|| "Unavailable".to_string())
}

fn dashboard_signals(snapshot: &AdminAnalyticsSnapshot) -> Vec<(String, String, String)> {
    let mut signals = Vec::new();
    if let Some(stats) = snapshot.user_stats.as_ref() {
        signals.push((
            "Wallet activity".to_string(),
            format_count(stats.today_connections),
            "Successful authentications recorded during the last 24 hours".to_string(),
        ));
    }
    if let Some(stats) = snapshot.plan_stats.as_ref() {
        signals.push((
            "Active plans".to_string(),
            format_count(stats.active_plans),
            format!(
                "{} active memberships",
                format_count(stats.active_memberships)
            ),
        ));
    }
    if let Some(stats) = snapshot.permission_analytics.as_ref() {
        signals.push((
            "Permissions".to_string(),
            format_count(stats.active_permissions),
            format!(
                "{} definitions in the permission model",
                format_count(stats.total_permissions)
            ),
        ));
    }
    if let Some(stats) = snapshot.developer_portal.as_ref() {
        signals.push((
            "API keys".to_string(),
            format_count(stats.active_api_keys),
            format!("{} keys recorded", format_count(stats.total_api_keys)),
        ));
    }
    signals
}

#[component]
fn DashboardOverviewProblem(state: &'static str, title: String, detail: String) -> Element {
    rsx! {
        section {
            class: "mt-6 rounded-2xl border border-amber-500/30 bg-amber-500/5 p-5",
            "data-admin-dashboard-overview-state": state,
            h2 { class: "font-semibold text-foreground", "{title}" }
            p { class: "mt-1 text-sm leading-6 text-muted-foreground", "{detail}" }
        }
    }
}

#[component]
fn DashboardReady(projection: AdminDashboardUserStatus) -> Element {
    let total_users = format_count(projection.total_users);
    let active_users = format_count(projection.active_users);
    let observed_at = projection.observed_at;

    rsx! {
        section {
            class: "overflow-hidden rounded-2xl border border-border/30 bg-card shadow-xl",
            aria_labelledby: "admin-dashboard-user-status-title",
            "data-admin-dashboard-state": ADMIN_DASHBOARD_USER_STATUS_READY,
            div {
                class: "h-1 bg-gradient-to-r from-[#1fc7d4] via-[#7645d9] to-[#31d0aa]",
                aria_hidden: "true",
            }
            div { class: "flex flex-col gap-3 p-5 sm:flex-row sm:items-start sm:justify-between sm:p-6",
                div {
                    h2 {
                        id: "admin-dashboard-user-status-title",
                        class: "text-lg font-semibold text-foreground",
                        "Registered user status"
                    }
                    p { class: "mt-1 max-w-2xl text-sm leading-6 text-muted-foreground",
                        "Active describes the stored account status, not recent visits or live presence."
                    }
                }
                p { class: "text-xs text-muted-foreground",
                    "Observed at "
                    time { datetime: observed_at.clone(), "{observed_at}" }
                }
            }
            dl {
                class: "grid grid-cols-1 gap-px border-t border-border/30 bg-border/30 sm:grid-cols-2",
                DashboardCount {
                    label: "Total users".to_string(),
                    value: total_users,
                    detail: "All registered user records".to_string(),
                }
                DashboardCount {
                    label: "Users marked active".to_string(),
                    value: active_users,
                    detail: "Records with active account status".to_string(),
                }
            }
        }
    }
}

#[component]
fn DashboardCount(label: String, value: String, detail: String) -> Element {
    rsx! {
        div { class: "min-w-0 bg-card p-5 sm:p-6",
            dt { class: "text-sm font-medium text-muted-foreground", "{label}" }
            dd { class: "mt-2 break-words text-3xl font-black tracking-tight text-foreground", "{value}" }
            dd { class: "mt-2 text-xs leading-5 text-muted-foreground", "{detail}" }
        }
    }
}

#[component]
fn DashboardProblem(state: &'static str, title: String, detail: String) -> Element {
    rsx! {
        section {
            class: "rounded-2xl border border-amber-500/25 bg-amber-500/10 p-6 sm:p-8",
            role: "status",
            aria_labelledby: "admin-dashboard-problem-title",
            "data-admin-dashboard-state": state,
            div { class: "flex flex-col gap-5 sm:flex-row sm:items-start",
                div {
                    class: "flex h-12 w-12 shrink-0 items-center justify-center rounded-xl border border-amber-500/25 bg-background/60 text-amber-700 dark:text-amber-300",
                    aria_hidden: "true",
                    Icon { name: "shield".to_string(), size: Some(24) }
                }
                div { class: "min-w-0",
                    h2 {
                        id: "admin-dashboard-problem-title",
                        class: "text-xl font-bold text-foreground",
                        "{title}"
                    }
                    p { class: "mt-2 max-w-3xl text-sm leading-6 text-muted-foreground", "{detail}" }
                    nav { class: "mt-5 flex flex-wrap gap-3", aria_label: "Dashboard snapshot recovery",
                        a { class: "btn btn-sm btn-outline", href: "/",
                            Icon { name: "refresh-cw".to_string(), size: Some(15) }
                            " Retry snapshot"
                        }
                        a { class: "btn btn-sm btn-ghost", href: "/audit-log",
                            Icon { name: "history".to_string(), size: Some(15) }
                            " Audit workspace"
                        }
                    }
                }
            }
        }
    }
}

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
    use super::*;
    use crate::auth::user::{AuthMethod, User};

    fn authenticated_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "verified-admin-session".to_string(),
                address: "0xsession".to_string(),
                chain_id: "56".to_string(),
                roles: vec![],
                email: None,
                tier: None,
                permissions: vec![],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: None,
            }),
            path: "/".to_string(),
            ..Default::default()
        }
    }

    fn status_json(total_users: i64, active_users: i64) -> String {
        serde_json::json!({
            "observed_at": "2026-07-23T10:20:30Z",
            "total_users": total_users,
            "active_users": active_users,
        })
        .to_string()
    }

    fn ctx_with_snapshot(state: &str, data: Option<String>) -> PageContext {
        let mut ctx = authenticated_ctx();
        ctx.params.insert(
            ADMIN_DASHBOARD_USER_STATUS_STATE_PARAM.to_string(),
            state.to_string(),
        );
        if let Some(data) = data {
            ctx.params
                .insert(ADMIN_DASHBOARD_USER_STATUS_PARAM.to_string(), data);
        }
        ctx
    }

    fn with_ready_overview(mut ctx: PageContext) -> PageContext {
        ctx.params.insert(
            ADMIN_ANALYTICS_STATE_PARAM.to_string(),
            ADMIN_ANALYTICS_READY.to_string(),
        );
        ctx.params.insert(
            ADMIN_ANALYTICS_DATA_PARAM.to_string(),
            serde_json::json!({
                "observed_at": "2026-07-23T10:20:31Z",
                "user_stats": {
                    "total": 1234,
                    "active": 900,
                    "today_connections": 27,
                    "total_users": 1234,
                    "active_users": 900
                },
                "permission_analytics": {
                    "total": 81,
                    "total_plans": 6,
                    "total_permissions": 81,
                    "active_permissions": 72
                },
                "plan_stats": {
                    "total_plans": 6,
                    "active_plans": 5,
                    "total_memberships": 43,
                    "active_memberships": 40,
                    "recent_assignments": 3
                },
                "system_metrics": null,
                "developer_portal": {
                    "total_api_keys": 14,
                    "active_api_keys": 11
                }
            })
            .to_string(),
        );
        ctx
    }

    fn html(ctx: &PageContext) -> String {
        dioxus_ssr::render_element(render(ctx).1)
    }

    fn assert_no_fabricated_dashboard_claims(rendered: &str) {
        let lowered = rendered.to_ascii_lowercase();
        for forbidden in [
            "system health",
            "uptime",
            "latency",
            "avg resp",
            "alert",
            "recent activity",
            "recent transaction",
            "core services running",
            "99.9%",
            "120ms",
        ] {
            assert!(
                !lowered.contains(forbidden),
                "dashboard rendered an unsupported claim `{forbidden}`: {rendered}"
            );
        }
    }

    #[test]
    fn strict_projection_accepts_zero_and_rejects_invalid_snapshots() {
        let zero = serde_json::json!({
            "observed_at": "2026-07-23T10:20:30Z",
            "total_users": 0,
            "active_users": 0,
        });
        assert_eq!(
            decode_admin_dashboard_user_status(zero),
            Some(AdminDashboardUserStatus {
                observed_at: "2026-07-23T10:20:30Z".to_string(),
                total_users: 0,
                active_users: 0,
            })
        );

        for malformed in [
            serde_json::json!({
                "observed_at": "2026-07-23T10:20:30Z",
                "total_users": -1,
                "active_users": 0,
            }),
            serde_json::json!({
                "observed_at": "2026-07-23T10:20:30Z",
                "total_users": 3,
                "active_users": 4,
            }),
            serde_json::json!({
                "observed_at": "not-a-time",
                "total_users": 3,
                "active_users": 2,
            }),
            serde_json::json!({
                "observed_at": "2026-07-23T10:20:30Z",
                "total_users": 3,
                "active_users": 2,
                "system_health": 100,
            }),
        ] {
            assert!(decode_admin_dashboard_user_status(malformed).is_none());
        }
    }

    #[test]
    fn signed_out_route_keeps_snapshot_private() {
        let mut ctx = ctx_with_snapshot(
            ADMIN_DASHBOARD_USER_STATUS_READY,
            Some(status_json(1_234, 900)),
        );
        ctx.user = None;
        let rendered = html(&ctx);

        assert!(rendered.contains("Sign in required"));
        assert!(rendered.contains("href=\"/auth?return_url=%2F\""));
        assert!(!rendered.contains("data-admin-dashboard-state"));
        assert!(!rendered.contains("1,234"));
        assert!(!rendered.contains("2026-07-23T10:20:30Z"));
    }

    #[test]
    fn ready_snapshot_renders_two_semantic_counts_and_observation_time() {
        let ctx = ctx_with_snapshot(
            ADMIN_DASHBOARD_USER_STATUS_READY,
            Some(status_json(1_234, 900)),
        );
        let rendered = html(&ctx);

        assert!(rendered.contains("data-admin-dashboard-state=\"ready\""));
        assert!(rendered.contains("aria-labelledby=\"admin-dashboard-user-status-title\""));
        assert!(rendered.contains("<dl"));
        assert_eq!(rendered.matches("<dt").count(), 2);
        assert_eq!(rendered.matches("<dd").count(), 4);
        assert!(rendered.contains("Total users"));
        assert!(rendered.contains(">1,234<"));
        assert!(rendered.contains("Users marked active"));
        assert!(rendered.contains(">900<"));
        assert!(rendered.contains("Observed at"));
        assert!(rendered.contains("datetime=\"2026-07-23T10:20:30Z\""));
        assert!(rendered.contains("not recent visits or live presence"));
        assert!(rendered.contains("p-3 sm:p-6 lg:p-8"));
        assert!(rendered.contains("Operational Modules"));
        assert!(!rendered.contains("class=\"admin-shell admin-shell-page\""));
        assert_no_fabricated_dashboard_claims(&rendered);
    }

    #[test]
    fn zero_counts_are_authoritative_ready_data() {
        let ctx = ctx_with_snapshot(ADMIN_DASHBOARD_USER_STATUS_READY, Some(status_json(0, 0)));
        let rendered = html(&ctx);

        assert!(rendered.contains("data-admin-dashboard-state=\"ready\""));
        assert!(rendered.matches(">0<").count() >= 2);
        assert!(rendered.contains("data-admin-dashboard-overview-state=\"unavailable\""));
        assert!(!rendered.contains("could not be verified"));
    }

    #[test]
    fn ready_overview_replaces_every_dashboard_placeholder_with_backend_values() {
        let ctx = with_ready_overview(ctx_with_snapshot(
            ADMIN_DASHBOARD_USER_STATUS_READY,
            Some(status_json(1_234, 900)),
        ));
        let rendered = html(&ctx);

        for expected in [
            "BACKEND CONNECTED",
            "Verified",
            "4 sources",
            "Total Wallets",
            ">1,234<",
            "Active Wallets",
            ">900<",
            "Daily Connections",
            ">27<",
            "Active Permissions",
            ">72<",
            "Platform Signals",
            "Active plans",
            "API keys",
            "AUTHORITATIVE SNAPSHOT / READ ONLY",
        ] {
            assert!(
                rendered.contains(expected),
                "missing {expected}: {rendered}"
            );
        }
        assert!(!rendered.contains("Not projected"));
        assert!(!rendered.contains("NOT_PROJECTED"));
        assert!(!rendered.contains("Backend values not projected"));
        assert!(!rendered.contains("data-admin-dashboard-overview-state"));
        assert_no_fabricated_dashboard_claims(&rendered);
    }

    #[test]
    fn forbidden_unavailable_and_malformed_are_truthful_explicit_states() {
        for (state, expected) in [
            (
                ADMIN_DASHBOARD_USER_STATUS_FORBIDDEN,
                "Dashboard snapshot access was denied",
            ),
            (
                ADMIN_DASHBOARD_USER_STATUS_UNAVAILABLE,
                "Dashboard snapshot is unavailable",
            ),
            (
                ADMIN_DASHBOARD_USER_STATUS_MALFORMED,
                "Dashboard snapshot could not be verified",
            ),
        ] {
            let rendered = html(&ctx_with_snapshot(state, None));
            assert!(rendered.contains(&format!("data-admin-dashboard-state=\"{state}\"")));
            assert!(rendered.contains(expected));
            assert!(rendered.contains("No counts are being shown."));
            assert!(!rendered.contains("<dl"));
            assert_no_fabricated_dashboard_claims(&rendered);
        }
    }

    #[test]
    fn missing_data_or_unknown_state_fails_closed_as_malformed_or_unavailable() {
        let ready_without_data = html(&ctx_with_snapshot(ADMIN_DASHBOARD_USER_STATUS_READY, None));
        assert!(ready_without_data.contains("data-admin-dashboard-state=\"malformed\""));

        let unknown = html(&ctx_with_snapshot("unexpected", Some(status_json(5, 4))));
        assert!(unknown.contains("data-admin-dashboard-state=\"malformed\""));

        let missing = html(&authenticated_ctx());
        assert!(missing.contains("data-admin-dashboard-state=\"unavailable\""));
    }
}

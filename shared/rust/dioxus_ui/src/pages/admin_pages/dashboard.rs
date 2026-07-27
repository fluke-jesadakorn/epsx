//! `/` (plus the target-only `/index` alias) — authenticated admin
//! command-center snapshot.
//!
//! The page accepts only a strict, server-projected user-status snapshot. It
//! deliberately does not infer operational health, service state, activity,
//! alerts, permissions, or any other dashboard metric.

use chrono::DateTime;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::auth::AuthGate;
use crate::layout::admin_shell::AdminShell;
use crate::primitives::Icon;

use super::super::{PageContext, PageMeta};

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
    let snapshot_observed_at = match &load {
        DashboardLoad::Ready(projection) => Some(projection.observed_at.clone()),
        _ => None,
    };

    rsx! {
        AuthGate {
            user: ctx.user.clone(),
            feature: Some("the private admin command center".to_string()),
            return_url: Some("/".to_string()),
            AdminShell {
                ctx: ctx.clone(),
                page_title: "Command Center".to_string(),
                breadcrumbs: vec![("Dashboard".to_string(), "/".to_string())],
                div { class: "container page-content admin-dashboard mx-auto w-full max-w-[1600px] pb-12",
                    // Keep the development dashboard composition (pulse header,
                    // HUD metrics, bento tools, and event stream) even when the
                    // backend cannot authorize those data feeds. Unavailable
                    // values are rendered explicitly instead of being invented.
                    DashboardPulseHeader { observed_at: snapshot_observed_at }
                    DashboardHudMetrics {}
                    div { class: "mb-4 flex items-center justify-between",
                        h2 { class: "text-sm font-bold uppercase tracking-widest text-muted-foreground",
                            "Admin Modules"
                        }
                        span { class: "text-[10px] font-mono uppercase tracking-widest text-muted-foreground",
                            "Backend values not projected"
                        }
                    }
                    div { class: "grid grid-cols-1 gap-6 xl:grid-cols-4",
                        div { class: "xl:col-span-3",
                            DashboardBentoTools {}
                        }
                        div { class: "h-full xl:col-span-1",
                            DashboardActivityStream {}
                        }
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
fn DashboardPulseHeader(observed_at: Option<String>) -> Element {
    let (state_label, state_class, timestamp) = match observed_at {
        Some(value) => ("SNAPSHOT READY", "text-emerald-400", value),
        None => (
            "DATA UNAVAILABLE",
            "text-amber-300",
            "Snapshot timestamp not projected".to_string(),
        ),
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
                    DashboardPulseMetric { label: "Response", value: "Not projected".to_string(), class_name: "text-cyan-400".to_string() }
                    DashboardPulseMetric { label: "Availability", value: "Not projected".to_string(), class_name: "text-primary".to_string() }
                    DashboardPulseMetric { label: "Signals", value: "Not projected".to_string(), class_name: "text-amber-300".to_string() }
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
fn DashboardHudMetrics() -> Element {
    rsx! {
        div { class: "mb-6 grid grid-cols-2 gap-4 lg:grid-cols-4", "data-admin-dashboard-surface": "hud-metrics",
            DashboardMetricCard {
                label: "Total Wallets",
                value: "Not projected",
                subtext: "Backend value not projected",
                icon: "wallet",
                accent: "text-cyan-400",
                border: "border-cyan-500/30",
            }
            DashboardMetricCard {
                label: "System Status",
                value: "Not projected",
                subtext: "Backend value not projected",
                icon: "activity",
                accent: "text-emerald-400",
                border: "border-emerald-500/30",
            }
            DashboardMetricCard {
                label: "Daily Connections",
                value: "Not projected",
                subtext: "Backend value not projected",
                icon: "users",
                accent: "text-pink-400",
                border: "border-pink-500/30",
            }
            DashboardMetricCard {
                label: "Response Time",
                value: "Not projected",
                subtext: "Backend value not projected",
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
    value: &'static str,
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
        div { class: "grid auto-rows-[190px] grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3", "data-admin-dashboard-surface": "bento-tools",
            DashboardToolCard {
                href: "/wallet-management/wallets",
                title: "Wallet Database",
                description: "Deep inspect connected wallets, view connection history, and manage active sessions.",
                icon: "wallet",
                span: "md:col-span-2",
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
                span: "md:col-span-2",
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
                        "Not projected"
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
fn DashboardActivityStream() -> Element {
    rsx! {
        section { class: "flex h-full min-h-[420px] flex-col overflow-hidden rounded-2xl border border-border/20 bg-card shadow-2xl", "data-admin-dashboard-surface": "activity-stream",
            header { class: "flex items-center justify-between border-b border-border/20 bg-muted/20 p-4",
                div { class: "flex items-center gap-3",
                    span { class: "relative flex h-3 w-3",
                        span { class: "absolute inline-flex h-full w-full rounded-full bg-cyan-400 opacity-50" }
                        span { class: "relative inline-flex h-3 w-3 rounded-full bg-cyan-500" }
                    }
                    h2 { class: "font-mono text-xs font-black uppercase tracking-[0.2em] text-cyan-400", "Global Event Stream" }
                }
                Icon { name: "refresh-cw".to_string(), size: Some(16), class_name: Some("text-muted-foreground".to_string()) }
            }
            div { class: "relative flex flex-1 items-center justify-center bg-background/50 p-6 font-mono text-sm text-muted-foreground",
                div { class: "text-center",
                    Icon { name: "wifi-off".to_string(), size: Some(24), class_name: Some("mx-auto mb-3 text-amber-300".to_string()) }
                    p { class: "font-semibold uppercase tracking-widest", "STREAM.NOT_PROJECTED" }
                    p { class: "mt-2 text-xs leading-5", "Recent wallet activity is not exposed by the backend projection." }
                }
            }
            footer { class: "border-t border-border/20 bg-background/80 p-2 text-center text-[10px] font-mono uppercase tracking-widest text-muted-foreground",
                "END OF STREAM / DATA NOT PROJECTED"
            }
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
            "operational module",
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
        assert_eq!(
            rendered
                .matches("class=\"admin-shell admin-shell-page\"")
                .count(),
            1
        );
        assert_no_fabricated_dashboard_claims(&rendered);
    }

    #[test]
    fn zero_counts_are_authoritative_ready_data() {
        let ctx = ctx_with_snapshot(ADMIN_DASHBOARD_USER_STATUS_READY, Some(status_json(0, 0)));
        let rendered = html(&ctx);

        assert!(rendered.contains("data-admin-dashboard-state=\"ready\""));
        assert_eq!(rendered.matches(">0<").count(), 2);
        assert!(!rendered.contains("unavailable"));
        assert!(!rendered.contains("could not be verified"));
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

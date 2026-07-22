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

    rsx! {
        AuthGate {
            user: ctx.user.clone(),
            feature: Some("the private admin command center".to_string()),
            return_url: Some("/".to_string()),
            AdminShell {
                ctx: ctx.clone(),
                page_title: "Command Center".to_string(),
                breadcrumbs: vec![("Dashboard".to_string(), "/".to_string())],
                div { class: "container page-content admin-dashboard",
                    div { class: "mb-6",
                        p { class: "text-xs font-semibold uppercase tracking-[0.2em] text-primary",
                            "User status snapshot"
                        }
                        h1 { class: "mt-2 text-3xl font-black tracking-tight text-foreground",
                            "Command Center"
                        }
                        p { class: "mt-2 max-w-3xl text-sm leading-6 text-muted-foreground",
                            "A narrow, backend-authorized view of registered user status."
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

//! `/` (plus the target-only `/index` alias) — authenticated admin
//! command-center shell. The reserved `/dashboard` path remains not found.
//!
//! The Rust admin frontend does not yet consume a backend-owned dashboard
//! aggregate. Rendering wallet counts, activity, transactions, service health,
//! alerts, latency, uptime, or timestamps would therefore manufacture
//! operational success. This page keeps its page-owned `AdminShell` while
//! failing closed with an explicit unavailable state.

use dioxus::prelude::*;

use crate::auth::AuthGate;
use crate::layout::admin_shell::AdminShell;
use crate::primitives::Icon;

use super::super::{PageContext, PageMeta};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Command center unavailable");
    (meta, rsx! { RenderDashboard { ctx: ctx.clone() } })
}

/// Query and route parameters are deliberately ignored. Compatibility values
/// are not a trusted source of dashboard aggregates, health state, freshness,
/// or authorization decisions.
#[component]
fn RenderDashboard(ctx: PageContext) -> Element {
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
                    section {
                        class: "relative overflow-hidden rounded-2xl border border-primary/20 bg-card shadow-xl",
                        role: "status",
                        aria_labelledby: "admin-dashboard-unavailable-title",
                        "data-section": "admin-dashboard-unavailable",
                        "data-admin-dashboard-state": "unavailable",
                        div { class: "h-1 bg-gradient-to-r from-primary via-cyan-400 to-purple-500" }
                        div { class: "p-6 sm:p-10",
                            div { class: "flex flex-col gap-5 sm:flex-row sm:items-start",
                                div {
                                    class: "flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl bg-primary/10 text-primary",
                                    aria_hidden: "true",
                                    Icon { name: "layout-dashboard".to_string(), size: Some(28) }
                                }
                                div { class: "max-w-3xl",
                                    p { class: "text-xs font-semibold uppercase tracking-[0.2em] text-primary",
                                        "Backend aggregate unavailable"
                                    }
                                    h2 {
                                        id: "admin-dashboard-unavailable-title",
                                        class: "mt-2 text-2xl font-semibold text-foreground",
                                        "Operational overview unavailable"
                                    }
                                    p { class: "mt-3 text-sm leading-6 text-muted-foreground",
                                        "No wallet, plan, transaction, activity, service-health, alert, latency, uptime, timestamp, chart, or total is being inferred. The command center will remain unavailable until the backend supplies a typed, authorized, and freshness-aware aggregate."
                                    }
                                }
                            }

                            div { class: "mt-8 grid grid-cols-1 gap-4 md:grid-cols-3",
                                BoundaryItem {
                                    icon: "database",
                                    title: "Typed aggregate",
                                    detail: "Every dashboard value needs a backend-owned response schema rather than compatibility or sample data."
                                }
                                BoundaryItem {
                                    icon: "shield",
                                    title: "Backend authorization",
                                    detail: "The backend must decide which operational fields this verified session may receive."
                                }
                                BoundaryItem {
                                    icon: "clock",
                                    title: "Freshness and failure",
                                    detail: "The contract must distinguish freshness, partial failure, and unavailable dependencies."
                                }
                            }

                            nav {
                                class: "mt-8 flex flex-col gap-3 border-t border-border/30 pt-6 sm:flex-row sm:flex-wrap",
                                aria_label: "Command center recovery",
                                a { class: "btn btn-primary", href: "/",
                                    Icon { name: "refresh-cw".to_string(), size: Some(16) }
                                    " Check again"
                                }
                                a { class: "btn btn-outline", href: "/audit-log",
                                    Icon { name: "history".to_string(), size: Some(16) }
                                    " Audit workspace"
                                }
                                a { class: "btn btn-outline", href: "/notifications/manage",
                                    Icon { name: "bell".to_string(), size: Some(16) }
                                    " Notifications workspace"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn BoundaryItem(icon: &'static str, title: &'static str, detail: &'static str) -> Element {
    rsx! {
        article { class: "rounded-xl border border-border/30 bg-background/40 p-4",
            div { class: "flex items-center gap-2",
                Icon { name: icon.to_string(), size: Some(16) }
                h3 { class: "text-sm font-semibold", "{title}" }
            }
            p { class: "mt-2 text-sm leading-6 text-muted-foreground", "{detail}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::{AuthMethod, User};

    fn authenticated_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "u-admin-session".to_string(),
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
            path: "/".to_string(),
            ..Default::default()
        }
    }

    fn html(ctx: &PageContext) -> String {
        let (_, element) = render(ctx);
        dioxus_ssr::render_element(element)
    }

    #[test]
    fn signed_out_route_keeps_dashboard_state_private() {
        let rendered = html(&PageContext {
            path: "/".to_string(),
            ..Default::default()
        });

        assert!(rendered.contains("Sign in required"));
        assert!(rendered.contains("href=\"/auth?return_url=%2F\""));
        assert!(!rendered.contains("data-admin-dashboard-state"));
        assert!(!rendered.contains("admin-shell admin-shell-page"));
        assert!(!rendered.contains("Operational overview unavailable"));
    }

    #[test]
    fn role_empty_authenticated_session_reaches_explicit_unavailable_state() {
        let rendered = html(&authenticated_ctx());

        assert!(rendered.contains("data-admin-dashboard-state=\"unavailable\""));
        assert!(rendered.contains("aria-labelledby=\"admin-dashboard-unavailable-title\""));
        assert!(rendered.contains("Backend authorization"));
        assert!(!rendered.contains("Permission required"));
        assert!(!rendered.contains("admin:dashboard:view"));
    }

    #[test]
    fn unavailable_state_has_no_sample_operations_or_controls() {
        let rendered = html(&authenticated_ctx());

        for forbidden in [
            "1,247",
            "$2.4M",
            "84.2K",
            "42ms",
            "99.97%",
            "OPERATIONAL",
            "SYS_OK",
            "Registered",
            "Active Nodes",
            "Pending Broadcasts",
            "BSC Mainnet",
            "Recent transactions",
            "Global Event Stream",
            "Indexer sync complete",
            "all clear",
            "<form",
            "<input",
            "<select",
            "<button",
            "onclick=",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "sample dashboard value or control leaked: {forbidden}"
            );
        }
    }

    #[test]
    fn hostile_legacy_params_cannot_hydrate_dashboard_claims() {
        let mut ctx = authenticated_ctx();
        ctx.query = "uptime=LEGACY_UPTIME_100&health=LEGACY_HEALTH_OK&wallets=999999&alerts=LEGACY_ZERO_ALERTS".to_string();
        ctx.params.insert(
            "dashboard_data".to_string(),
            "SYSTEM_HEALTHY_999999_WALLETS".to_string(),
        );
        let rendered = html(&ctx);

        for forbidden in [
            "LEGACY_UPTIME_100",
            "LEGACY_HEALTH_OK",
            "999999",
            "LEGACY_ZERO_ALERTS",
            "SYSTEM_HEALTHY_999999_WALLETS",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "hostile dashboard value leaked: {forbidden}"
            );
        }
        assert!(rendered.contains("data-admin-dashboard-state=\"unavailable\""));
    }

    #[test]
    fn page_owns_one_shell_and_uses_safe_native_recovery() {
        let rendered = html(&authenticated_ctx());

        assert_eq!(
            rendered
                .matches("class=\"admin-shell admin-shell-page\"")
                .count(),
            1,
            "the dashboard route must own exactly one admin shell"
        );
        for href in ["/", "/audit-log", "/notifications/manage"] {
            assert!(rendered.contains(&format!("href=\"{href}\"")));
        }
        assert!(!rendered.contains("javascript:"));
        assert!(!rendered.contains("onclick="));
    }
}

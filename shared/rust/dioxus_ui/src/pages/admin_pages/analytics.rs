//! `/analytics` — authenticated admin analytics availability shell.
//!
//! The legacy page renders system and user analytics from backend-owned
//! responses. The Rust admin frontend does not yet consume a verified
//! analytics response, so it must not replace missing data with sample values,
//! zeroes, generated timestamps, status claims, or inert controls.

use dioxus::prelude::*;

use crate::auth::AuthGate;
use crate::layout::admin_shell::AdminShell;
use crate::primitives::Icon;

use super::super::{PageContext, PageMeta};

const ANALYTICS_PATH: &str = "/analytics";

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Analytics");
    (meta, rsx! { RenderAnalytics { ctx: ctx.clone() } })
}

/// Session presence is the only frontend gate. Analytics authorization and
/// field-level visibility remain backend decisions. Query and route parameters
/// are deliberately ignored because they are not trusted analytics data.
#[component]
fn RenderAnalytics(ctx: PageContext) -> Element {
    rsx! {
        AuthGate {
            user: ctx.user.clone(),
            feature: Some("the private admin analytics workspace".to_string()),
            return_url: Some(ANALYTICS_PATH.to_string()),
            AdminShell {
                ctx: ctx.clone(),
                page_title: "Analytics".to_string(),
                breadcrumbs: vec![
                    ("Dashboard".to_string(), "/".to_string()),
                    ("Analytics".to_string(), ANALYTICS_PATH.to_string()),
                ],
                div {
                    class: "container page-content admin-analytics py-8",
                    "data-admin-analytics-state": "unavailable",
                    div { class: "mb-6 flex flex-col gap-4 sm:mb-8 sm:flex-row sm:items-center sm:justify-between",
                        div { class: "flex items-center gap-3",
                            div { class: "flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-purple-500 to-pink-500",
                                Icon { name: "bar-chart-3".to_string(), size: Some(20), class_name: Some("text-white".to_string()) }
                            }
                            div {
                                h1 { class: "text-2xl font-bold text-foreground", "Analytics" }
                                p { class: "text-sm text-slate-400", "Top-performing stocks by EPS growth" }
                            }
                        }
                        div { class: "flex items-center gap-2",
                            span { class: "flex items-center gap-1.5 rounded-lg border border-amber-500/20 bg-amber-500/10 px-3 py-1.5",
                                Icon { name: "alert-circle".to_string(), size: Some(14), class_name: Some("text-amber-400".to_string()) }
                                span { class: "text-xs font-medium text-amber-400", "Unavailable" }
                            }
                            span { class: "flex items-center gap-1.5 rounded-lg border border-slate-500/20 bg-slate-500/10 px-3 py-1.5",
                                Icon { name: "shield-check".to_string(), size: Some(14), class_name: Some("text-slate-400".to_string()) }
                                span { class: "text-xs font-medium text-slate-400", "Backend-owned" }
                            }
                        }
                    }
                    section {
                        class: "relative overflow-hidden rounded-3xl border border-border/40 bg-card shadow-2xl",
                        role: "status",
                        aria_labelledby: "admin-analytics-unavailable-title",
                        "data-section": "admin-analytics-unavailable",
                        div { class: "absolute inset-x-0 top-0 h-1 bg-gradient-to-r from-[#1fc7d4] via-[#7645d9] to-[#ffb237]" }
                        div { class: "p-8 md:p-12",
                            div { class: "flex flex-col gap-6 sm:flex-row sm:items-start",
                                div {
                                    class: "flex h-16 w-16 shrink-0 items-center justify-center rounded-2xl border border-amber-500/20 bg-amber-500/10 text-[#ffb237]",
                                    aria_hidden: "true",
                                    Icon { name: "bar-chart-3".to_string(), size: Some(30) }
                                }
                                div { class: "min-w-0 max-w-3xl",
                                    p { class: "text-xs font-black uppercase tracking-[0.22em] text-[#ffb237]",
                                        "Backend response unavailable"
                                    }
                                    h2 {
                                        id: "admin-analytics-unavailable-title",
                                        class: "mt-3 text-3xl font-black tracking-tight text-foreground",
                                        "Platform analytics are unavailable"
                                    }
                                    p { class: "mt-4 text-sm leading-6 text-muted-foreground",
                                        "No analytics values, activity records, freshness timestamps, or operational status are shown because a verified backend response is not connected. Unavailable data is not presented as an empty or successful result."
                                    }
                                    p { class: "mt-4 text-sm leading-6 text-muted-foreground",
                                        "The backend must authenticate the request and decide which analytics fields this session may receive before this workspace can display data. Frontend roles and permissions are not used to grant analytics access."
                                    }
                                    nav {
                                        class: "mt-8 flex flex-wrap gap-3 border-t border-border/30 pt-6",
                                        aria_label: "Analytics recovery",
                                        a { class: "btn btn-primary", href: ANALYTICS_PATH,
                                            Icon { name: "refresh-cw".to_string(), size: Some(16) }
                                            " Check again"
                                        }
                                        a { class: "btn btn-outline", href: "/", "Admin home" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::auth::user::{AuthMethod, User};

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

    #[test]
    fn signed_out_route_keeps_analytics_state_private() {
        let rendered = html(&PageContext {
            path: ANALYTICS_PATH.to_string(),
            ..Default::default()
        });

        assert!(rendered.contains("Sign in required"));
        assert!(rendered.contains("href=\"/auth?return_url=%2Fanalytics\""));
        assert!(!rendered.contains("data-admin-analytics-state"));
        assert!(!rendered.contains("admin-shell admin-shell-page"));
        assert!(!rendered.contains("Platform analytics are unavailable"));
    }

    #[test]
    fn empty_role_session_reaches_truthful_unavailable_state() {
        let rendered = html(&signed_in_ctx());

        assert!(rendered.contains("data-admin-analytics-state=\"unavailable\""));
        assert!(rendered.contains("aria-labelledby=\"admin-analytics-unavailable-title\""));
        assert!(rendered.contains("Frontend roles and permissions are not used"));
        assert!(!rendered.contains("Permission required"));
        assert!(!rendered.contains("admin:analytics:view"));
    }

    #[test]
    fn unavailable_state_has_no_samples_claims_or_controls() {
        let rendered = html(&signed_in_ctx());
        let section_start = rendered
            .find("data-section=\"admin-analytics-unavailable\"")
            .expect("analytics unavailable section must render");
        let section_end = rendered[section_start..]
            .find("</section>")
            .map(|offset| section_start + offset)
            .expect("analytics unavailable section must close");
        let unavailable_section = &rendered[section_start..section_end];

        for forbidden in [
            "12,345",
            "1.2M",
            "98%",
            "1,234",
            "142ms",
            "wallet.connect",
            "Active Users",
            "API Requests",
            "System Health",
            "Top events",
            "Volume (7d)",
            "Generated",
            "analytics_prod",
            "Live",
            "AI-Powered",
            "Export",
            "Last 30 days",
            "Search events",
            "<table",
            "<form",
            "<input",
            "<select",
            "<button",
            "onclick=",
        ] {
            assert!(
                !unavailable_section.contains(forbidden),
                "sample analytics claim or unsupported control leaked: {forbidden}"
            );
        }
    }

    #[test]
    fn hostile_params_and_query_are_ignored() {
        let mut ctx = signed_in_ctx();
        ctx.query = "range=HOSTILE_RANGE&health=HOSTILE_HEALTH&export=HOSTILE_EXPORT".to_string();
        ctx.params = HashMap::from([
            (
                "analytics_data".to_string(),
                "HOSTILE_ANALYTICS_DATA".to_string(),
            ),
            ("generated_at".to_string(), "HOSTILE_TIMESTAMP".to_string()),
        ]);
        let rendered = html(&ctx);

        for forbidden in [
            "HOSTILE_RANGE",
            "HOSTILE_HEALTH",
            "HOSTILE_EXPORT",
            "HOSTILE_ANALYTICS_DATA",
            "HOSTILE_TIMESTAMP",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "hostile analytics value leaked: {forbidden}"
            );
        }
        assert!(rendered.contains("data-admin-analytics-state=\"unavailable\""));
    }

    #[test]
    fn page_owns_one_shell_and_exact_native_recovery_links() {
        let rendered = html(&signed_in_ctx());

        assert_eq!(
            rendered
                .matches("class=\"admin-shell admin-shell-page\"")
                .count(),
            1,
            "the analytics page must own exactly one admin shell"
        );
        assert!(rendered.contains("class=\"admin-shell-main\""));
        assert!(rendered.contains("href=\"/analytics\""));
        assert!(rendered.contains("> Check again</a>"));
        assert!(rendered.contains("href=\"/\">Admin home</a>"));
        assert!(!rendered.contains("javascript:"));
        assert!(!rendered.contains("onclick="));
    }
}

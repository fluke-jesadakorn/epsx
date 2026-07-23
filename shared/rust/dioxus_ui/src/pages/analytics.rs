//! `/analytics` — truthful analytics availability shell.
//!
//! The pinned TypeScript page renders public or authenticated EPS rankings
//! returned by the analytics backend. Ranking access, offsets, plan details,
//! filters, pagination, and watchlist state are all server-owned. The Rust BFF
//! has no current verified rankings loader, so this page must not turn absent
//! data or any legacy compatibility parameter into rankings, zero/empty
//! success, plan access, or interactive capabilities.

use dioxus::prelude::*;

use super::{PageContext, PageMeta};
use crate::layout::main_layout::MainLayout;
use crate::primitives::Icon;

const ANALYTICS_SIGN_IN_PATH: &str = "/auth?return_url=%2Fanalytics";

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Analytics unavailable");
    (meta, rsx! { AnalyticsUnavailablePage { ctx: ctx.clone() } })
}

#[component]
fn AnalyticsUnavailablePage(ctx: PageContext) -> Element {
    // The source route is public: its AnalyticsAuthWrapper is a pass-through
    // and its data layer selects public versus authenticated backend reads.
    // Authentication only changes this safe navigation link; it is never used
    // here to invent ranking, permission, plan, or watchlist state.
    let signed_in = ctx.user.is_some();

    rsx! {
        MainLayout { ctx,
            AnalyticsUnavailableContent { signed_in }
        }
    }
}

#[component]
fn AnalyticsUnavailableContent(signed_in: bool) -> Element {
    rsx! {
        div { class: "relative min-h-screen analytics-page",
            div {
                class: "pointer-events-none fixed inset-0 z-0",
                "aria-hidden": "true",
                div { class: "absolute inset-0 bg-gradient-to-b from-white via-gray-50 to-white dark:from-slate-950 dark:via-slate-900 dark:to-slate-950" }
                div { class: "absolute -left-40 -top-40 h-[400px] w-[400px] rounded-full bg-purple-600/15 blur-3xl" }
                div { class: "absolute -right-32 top-1/3 h-[300px] w-[300px] rounded-full bg-blue-600/10 blur-3xl" }
            }

            div { class: "container page-content relative z-10 mx-auto max-w-7xl",
                header {
                    class: "analytics-header mb-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between",
                    "data-section": "analytics-header",
                    div { class: "flex items-center gap-3",
                        div { class: "flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-purple-500 to-pink-500",
                            Icon {
                                name: "bar-chart-3".to_string(),
                                size: Some(20),
                                class_name: Some("text-white".to_string())
                            }
                        }
                        div {
                            h1 { class: "text-2xl font-bold text-foreground", "Analytics" }
                            p { class: "text-sm text-slate-400", "Top-performing stocks by EPS growth" }
                        }
                    }
                    span {
                        class: "inline-flex w-fit items-center gap-2 rounded-lg border border-amber-500/30 bg-amber-500/10 px-3 py-1.5 text-xs font-semibold text-amber-500",
                        Icon { name: "alert-circle".to_string(), size: Some(14) }
                        "Data unavailable"
                    }
                }

                section {
                    class: "analytics-unavailable card card-glass overflow-hidden",
                    "data-section": "analytics-unavailable",
                    "data-analytics-state": "unavailable",
                    aria_labelledby: "analytics-unavailable-title",
                    role: "alert",
                    div { class: "h-1.5 bg-gradient-to-r from-purple-500 via-pink-500 to-orange-500" }
                    div { class: "card-body space-y-6 p-6 sm:p-8",
                        div { class: "flex flex-col gap-4 sm:flex-row sm:items-start",
                            div { class: "flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl bg-amber-500/10 text-amber-500",
                                Icon { name: "database".to_string(), size: Some(28) }
                            }
                            div { class: "max-w-3xl",
                                p { class: "text-xs font-semibold uppercase tracking-widest text-amber-500",
                                    "Analytics unavailable"
                                }
                                h2 {
                                    id: "analytics-unavailable-title",
                                    class: "mt-2 text-2xl font-semibold text-foreground",
                                    "Rankings cannot be verified right now"
                                }
                                p { class: "mt-3 text-sm leading-6 text-muted-foreground",
                                    "This page does not yet have a verified rankings response that matches the production analytics contract. No ranking rows, market metrics, access level, watchlist state, or pagination result is being inferred."
                                }
                            }
                        }

                        div { class: "grid grid-cols-1 gap-3 md:grid-cols-3",
                            AnalyticsBoundaryItem {
                                icon: "bar-chart-3",
                                title: "EPS rankings",
                                body: "Ranking rows remain hidden until the backend response can be validated end to end."
                            }
                            AnalyticsBoundaryItem {
                                icon: "filter",
                                title: "Filters and pagination",
                                body: "Search, sorting, filters, and page controls are not offered without verified results."
                            }
                            AnalyticsBoundaryItem {
                                icon: "shield",
                                title: "Access and watchlist",
                                body: "The frontend does not calculate rank offsets, plan access, permissions, or watchlist membership."
                            }
                        }

                        nav {
                            class: "flex flex-col gap-3 border-t border-border/40 pt-6 sm:flex-row",
                            "aria-label": "Analytics alternatives",
                            a {
                                class: "btn btn-primary",
                                href: "/plans",
                                Icon { name: "layers".to_string(), size: Some(16) }
                                " Browse plans"
                            }
                            if signed_in {
                                a {
                                    class: "btn btn-ghost",
                                    href: "/account",
                                    Icon { name: "user".to_string(), size: Some(16) }
                                    " Return to account"
                                }
                            } else {
                                a {
                                    class: "btn btn-ghost",
                                    href: ANALYTICS_SIGN_IN_PATH,
                                    Icon { name: "wallet".to_string(), size: Some(16) }
                                    " Sign in"
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
fn AnalyticsBoundaryItem(icon: &'static str, title: &'static str, body: &'static str) -> Element {
    rsx! {
        div { class: "rounded-xl border border-border bg-muted/30 p-4",
            div { class: "flex items-center gap-2 font-semibold text-foreground",
                Icon { name: icon.to_string(), size: Some(18) }
                "{title}"
            }
            p { class: "mt-2 text-sm leading-6 text-muted-foreground", "{body}" }
            span {
                class: "mt-3 inline-flex rounded-full border border-amber-500/30 bg-amber-500/10 px-2 py-1 text-xs font-medium text-amber-500",
                "Unavailable"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::{AuthMethod, User};

    fn page_ctx() -> PageContext {
        PageContext {
            path: "/analytics".to_string(),
            ..Default::default()
        }
    }

    fn signed_in_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "analytics-user".to_string(),
                address: "0xanalytics".to_string(),
                chain_id: "1".to_string(),
                roles: vec!["user".to_string()],
                email: None,
                tier: Some("unverified-client-tier".to_string()),
                permissions: vec![],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: None,
            }),
            ..page_ctx()
        }
    }

    fn render_html(ctx: &PageContext) -> String {
        let (_meta, element) = render(ctx);
        dioxus_ssr::render_element(element)
    }

    fn render_content(signed_in: bool) -> String {
        dioxus_ssr::render_element(rsx! { AnalyticsUnavailableContent { signed_in } })
    }

    fn assert_no_ranking_or_mutation_claims(html: &str) {
        for forbidden in [
            "Parker-Hannifin",
            "BHARTIARTL",
            "$952.30",
            "Ranks 100+",
            "Ranks 1-99",
            "Total trades",
            "1,234",
            "Win rate",
            "62%",
            "P&L over time",
            "Volume by day",
            "Recent events",
            "Last 30 days",
            "AI-Powered",
            "Export Analytics Data",
            "analytics-filter-apply",
            "analytics-filter-clear",
            "analytics:read",
            "watchlist-toggle",
            "/api/v1/analytics/summary",
        ] {
            assert!(
                !html.contains(forbidden),
                "analytics page must not render canned data or an unsupported capability `{forbidden}`. Got: {html}"
            );
        }
    }

    #[test]
    fn signed_out_route_is_public_accessible_and_truthfully_unavailable() {
        let html = render_html(&page_ctx());

        assert!(html.contains("data-analytics-state=\"unavailable\""));
        assert!(html.contains("role=\"alert\""));
        assert!(html.contains("Rankings cannot be verified right now"));
        assert!(!html.contains("href=\"/analytics\""));
        assert!(!html.contains("> Retry</a>"));
        assert!(html.contains("href=\"/plans\""));
        assert!(html.contains("href=\"/auth?return_url=%2Fanalytics\""));
        assert!(!html.contains("Sign in required"));
        assert!(!html.contains("Permission required"));
        assert_no_ranking_or_mutation_claims(&html);
    }

    #[test]
    fn signed_in_user_needs_no_frontend_permission_to_see_unavailable_state() {
        let html = render_html(&signed_in_ctx());

        assert!(html.contains("data-analytics-state=\"unavailable\""));
        assert!(html.contains("href=\"/account\""));
        assert!(!html.contains("Permission required"));
        assert!(!html.contains("unverified-client-tier"));
        assert_no_ranking_or_mutation_claims(&html);
    }

    #[test]
    fn canned_partial_and_malformed_payloads_cannot_render_rankings() {
        let payloads = [
            r#"{"stats":{"total_views":12345},"top_movers":[{"asset":"CANARY-CANNED-ASSET","change_24h_pct":99.9}],"rankings":[{"symbol":"CANARY-CANNED-RANK","rank":1}],"watchlist":["CANARY-CANNED-WATCHLIST"]}"#,
            r#"{"rankings":[{"symbol":"CANARY-PARTIAL-RANK"}]}"#,
            r#"{"rankings":[{"symbol":"CANARY-MALFORMED"}"#,
        ];

        for payload in payloads {
            let mut ctx = signed_in_ctx();
            ctx.params
                .insert("data_analytics".to_string(), payload.to_string());
            let html = render_html(&ctx);

            assert!(html.contains("data-analytics-state=\"unavailable\""));
            for canary in [
                "CANARY-CANNED-ASSET",
                "CANARY-CANNED-RANK",
                "CANARY-CANNED-WATCHLIST",
                "CANARY-PARTIAL-RANK",
                "CANARY-MALFORMED",
            ] {
                assert!(
                    !html.contains(canary),
                    "compatibility payload value `{canary}` must never reach analytics output"
                );
            }
            assert_no_ranking_or_mutation_claims(&html);
        }
    }

    #[test]
    fn unavailable_surface_has_no_fake_controls_or_mutations() {
        let html = render_content(false);

        assert!(html.contains("aria-labelledby=\"analytics-unavailable-title\""));
        assert!(html.contains("aria-label=\"Analytics alternatives\""));
        assert!(!html.contains("href=\"/analytics\""));
        assert!(html.contains("href=\"/plans\""));
        assert!(html.contains("href=\"/auth?return_url=%2Fanalytics\""));
        for forbidden in [
            "<form",
            "<input",
            "<select",
            "<button",
            "<script",
            "onclick=",
            "action=",
            "Export",
            "Add to watchlist",
            "Remove from watchlist",
            "Next page",
            "Previous page",
            "> Retry</a>",
        ] {
            assert!(
                !html.contains(forbidden),
                "unavailable analytics surface exposed unsupported control or mutation `{forbidden}`. Got: {html}"
            );
        }
        assert_no_ranking_or_mutation_claims(&html);
    }
}

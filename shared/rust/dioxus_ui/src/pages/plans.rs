//! `/plans` — public, truthful plan-catalog availability shell.
//!
//! The pinned development source (`origin/development` at
//! `373bd231cb7a616c3d4c0ddc1d60e0099a88a5db`) obtains public plans from the
//! subscription backend, then derives prices, promotions, grouping, features,
//! eligibility, credits, and selection behavior from that response. The Rust
//! BFF does not yet expose a frozen subscription-owned public-plan DTO, and
//! payment mutation authority is outside this migration slice. Therefore this
//! page deliberately ignores the legacy `data_plans` compatibility parameter
//! and renders no plan, price, access, eligibility, or purchase claim.

use dioxus::prelude::*;

use super::{PageContext, PageMeta};
use crate::layout::main_layout::MainLayout;
use crate::primitives::Icon;

const CONTACT_PATH: &str = "/contact";

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Plans unavailable");

    // `ctx.params["data_plans"]` is intentionally not read. Until the BFF has
    // a verified backend-owned DTO, compatibility input cannot be used to
    // create catalog, subscription, access, or payment state.
    (meta, rsx! { PlansUnavailablePage { ctx: ctx.clone() } })
}

#[component]
fn PlansUnavailablePage(ctx: PageContext) -> Element {
    rsx! {
        MainLayout { ctx,
            PlansUnavailableContent {}
        }
    }
}

#[component]
fn PlansUnavailableContent() -> Element {
    rsx! {
        div {
                class: "plans-prod-page relative min-h-screen overflow-hidden bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50 dark:from-gray-900 dark:via-gray-900 dark:to-indigo-900",
                "data-plans-state": "unavailable",
                div {
                    class: "pointer-events-none absolute inset-0",
                    "aria-hidden": "true",
                    div { class: "absolute -left-32 -top-32 h-80 w-80 rounded-full bg-emerald-500/15 blur-3xl" }
                    div { class: "absolute -right-32 top-1/3 h-96 w-96 rounded-full bg-purple-600/15 blur-3xl" }
                }

                div { class: "plans-prod-container container relative z-10 mx-auto px-4 py-12 sm:py-20",
                    header { class: "plans-prod-hero mx-auto mb-10 max-w-3xl text-center",
                        div { class: "mx-auto mb-5 flex h-14 w-14 items-center justify-center rounded-2xl bg-gradient-to-br from-emerald-500 to-blue-600 text-white shadow-lg shadow-blue-500/20",
                            Icon { name: "layers".to_string(), size: Some(28) }
                        }
                        h1 { class: "plans-prod-title bg-gradient-to-r from-emerald-600 via-blue-600 to-purple-600 bg-clip-text text-4xl font-bold text-transparent md:text-6xl",
                            "EPSX Plans"
                        }
                        p { class: "plans-prod-subtitle mx-auto mt-5 max-w-2xl text-lg leading-7 text-gray-600 dark:text-gray-300",
                            "Plan details are published from the subscription service. We cannot verify that catalog right now."
                        }
                    }

                    section {
                        class: "plans-unavailable mx-auto max-w-4xl overflow-hidden rounded-3xl border border-white/30 bg-white/70 shadow-2xl shadow-blue-950/10 backdrop-blur-xl dark:border-white/10 dark:bg-slate-900/70",
                        role: "alert",
                        aria_labelledby: "plans-unavailable-title",
                        "data-section": "plans-unavailable",
                        div { class: "h-1.5 bg-gradient-to-r from-emerald-500 via-blue-500 to-purple-500" }
                        div { class: "p-6 sm:p-10",
                            div { class: "flex flex-col gap-5 sm:flex-row sm:items-start",
                                div { class: "flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl bg-amber-500/10 text-amber-600 dark:text-amber-400",
                                    Icon { name: "alert-circle".to_string(), size: Some(28) }
                                }
                                div { class: "max-w-2xl",
                                    p { class: "text-xs font-semibold uppercase tracking-[0.2em] text-amber-600 dark:text-amber-400",
                                        "Catalog unavailable"
                                    }
                                    h2 {
                                        id: "plans-unavailable-title",
                                        class: "mt-2 text-2xl font-semibold text-gray-900 dark:text-white",
                                        "Plan options cannot be verified right now"
                                    }
                                    p { class: "mt-3 text-sm leading-6 text-gray-600 dark:text-gray-300",
                                        "No plan names, prices, promotions, features, availability, eligibility, or subscription actions are shown until a verified public-plan response is available."
                                    }
                                }
                            }

                            div { class: "mt-8 grid grid-cols-1 gap-4 md:grid-cols-3",
                                BoundaryItem {
                                    icon: "database",
                                    title: "Catalog",
                                    body: "Plan records remain hidden without a verified subscription response."
                                }
                                BoundaryItem {
                                    icon: "shield",
                                    title: "Access",
                                    body: "The frontend does not calculate plan access, eligibility, or subscription status."
                                }
                                BoundaryItem {
                                    icon: "credit-card",
                                    title: "Checkout",
                                    body: "Purchase and subscription changes are not offered from an unavailable catalog."
                                }
                            }

                            nav {
                                class: "mt-8 flex flex-col gap-3 border-t border-gray-200/70 pt-6 sm:flex-row dark:border-white/10",
                                "aria-label": "Plan catalog alternatives",
                                a {
                                    class: "btn btn-primary",
                                    href: CONTACT_PATH,
                                    Icon { name: "mail".to_string(), size: Some(16) }
                                    " Contact support"
                                }
                                a {
                                    class: "btn btn-ghost",
                                    href: "/",
                                    Icon { name: "home".to_string(), size: Some(16) }
                                    " Return home"
                                }
                            }
                        }
                    }
                }
        }
    }
}

#[component]
fn BoundaryItem(icon: &'static str, title: &'static str, body: &'static str) -> Element {
    rsx! {
        div { class: "rounded-2xl border border-gray-200/70 bg-white/60 p-5 dark:border-white/10 dark:bg-white/5",
            div { class: "flex items-center gap-2 font-semibold text-gray-900 dark:text-white",
                Icon { name: icon.to_string(), size: Some(18) }
                "{title}"
            }
            p { class: "mt-2 text-sm leading-6 text-gray-600 dark:text-gray-300", "{body}" }
            span { class: "mt-3 inline-flex rounded-full border border-amber-500/30 bg-amber-500/10 px-2 py-1 text-xs font-medium text-amber-600 dark:text-amber-400",
                "Unavailable"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page_ctx() -> PageContext {
        PageContext {
            path: "/plans".to_string(),
            ..Default::default()
        }
    }

    fn render_html(ctx: &PageContext) -> String {
        let (_meta, element) = render(ctx);
        dioxus_ssr::render_element(element)
    }

    fn render_unavailable_content() -> String {
        dioxus_ssr::render_element(rsx! { PlansUnavailableContent {} })
    }

    fn assert_no_catalog_or_purchase_claims(html: &str) {
        for forbidden in [
            "1 Day Package",
            "1 Month Package",
            "Lifetime Package",
            "API Personal",
            "API Company",
            "Revenue Share",
            "$1",
            "$9.9",
            "$4,999",
            "$999",
            "$2,999",
            "80% OFF",
            "90% OFF",
            "Ends in NaNm",
            "7-day free trial",
            "Get Started",
            "Buy Now",
            "Subscribe",
            "Extend Plan",
            "Upgrade Only",
            "Talk to Touch",
            "data-amount",
            "/api/v1/pay/intent",
            "/payment?planId=",
        ] {
            assert!(
                !html.contains(forbidden),
                "plans page must not render unverified catalog or purchase claim `{forbidden}`. Got: {html}"
            );
        }
    }

    #[test]
    fn public_route_renders_accessible_truthful_unavailable_state() {
        let html = render_html(&page_ctx());

        assert!(html.contains("data-plans-state=\"unavailable\""));
        assert!(html.contains("data-section=\"plans-unavailable\""));
        assert!(html.contains("role=\"alert\""));
        assert!(html.contains("aria-labelledby=\"plans-unavailable-title\""));
        assert!(html.contains("Plan options cannot be verified right now"));
        assert!(
            !html.contains("<main"),
            "plans page fragment must defer its sole main landmark to the shared shell"
        );
        assert_no_catalog_or_purchase_claims(&html);
    }

    #[test]
    fn legacy_and_hostile_payloads_never_create_plan_output() {
        let payloads = [
            r#"{"plans":[{"name":"CANARY-PLAN","price":"$0.01","eligible":true,"features":["CANARY-FEATURE"]}]}"#,
            r#"{"personal":[{"name":"CANARY-PERSONAL"}],"api":[{"name":"CANARY-API"}]}"#,
            r#"</p><script>CANARY-SCRIPT</script><button>CANARY-BUY</button>"#,
            r#"{"plans":[{"name":"CANARY-MALFORMED"}"#,
        ];

        for payload in payloads {
            let mut ctx = page_ctx();
            ctx.params
                .insert("data_plans".to_string(), payload.to_string());
            let html = render_html(&ctx);

            assert!(html.contains("data-plans-state=\"unavailable\""));
            for canary in [
                "CANARY-PLAN",
                "CANARY-FEATURE",
                "CANARY-PERSONAL",
                "CANARY-API",
                "CANARY-SCRIPT",
                "CANARY-BUY",
                "CANARY-MALFORMED",
            ] {
                assert!(
                    !html.contains(canary),
                    "legacy compatibility value `{canary}` must never reach plan output"
                );
            }
            assert_no_catalog_or_purchase_claims(&html);
        }
    }

    #[test]
    fn unavailable_surface_has_no_mutation_or_selection_control() {
        let html = render_unavailable_content();

        for forbidden in [
            "<form",
            "<input",
            "<select",
            "<button",
            "<script",
            "onclick=",
            "action=",
            "data-plan-id",
            "data-currency",
            "affiliate_code",
        ] {
            assert!(
                !html.contains(forbidden),
                "unavailable plans surface exposed unsupported control `{forbidden}`. Got: {html}"
            );
        }
        assert_no_catalog_or_purchase_claims(&html);
    }

    #[test]
    fn unavailable_catalog_offers_only_meaningful_safe_navigation() {
        let html = render_html(&page_ctx());

        assert!(html.contains("aria-label=\"Plan catalog alternatives\""));
        assert!(!html.contains("href=\"/plans\""));
        assert!(html.contains("href=\"/contact\""));
        assert!(html.contains("href=\"/\""));
        assert!(!html.contains("Retry catalog"));
        assert!(html.contains("Contact support"));
        assert!(!html.contains("javascript:"));
    }
}

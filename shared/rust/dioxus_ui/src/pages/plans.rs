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
    let meta = PageMeta::marketing("Plans");

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
                // The local SSR stylesheet does not emit Tailwind's standard
                // `dark:from-*` gradient utilities. Keep the source light
                // fallback while making the dark production frame explicit.
                style { "
                    .plans-prod-page {{ background: linear-gradient(to right bottom, #f8fafc 0%, #eff6ff 50%, #eef2ff 100%); }}
                    html.dark .plans-prod-page {{ background: linear-gradient(to right bottom, #111827 0%, #111827 50%, #312e81 100%); }}
                    .plans-catalog-alternatives {{
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        gap: 2.5rem;
                        margin-top: 3rem;
                        padding-top: 2rem;
                        border-top: 1px solid rgba(148, 163, 184, 0.24);
                    }}
                    .plans-catalog-alternatives a {{
                        display: inline-flex;
                        align-items: center;
                        gap: 0.5rem;
                        color: #e2e8f0;
                        font-size: 1rem;
                        font-weight: 600;
                        text-decoration: none;
                        transition: color 0.15s ease, transform 0.15s ease;
                    }}
                    .plans-catalog-alternatives a:hover {{
                        color: #ffffff;
                        transform: translateY(-1px);
                    }}
                    .plans-catalog-alternatives i {{ color: #f8fafc; flex-shrink: 0; }}
                    @media (max-width: 639px) {{
                        .plans-catalog-alternatives {{ flex-direction: column; gap: 1.25rem; }}
                    }}
                " }

                div { class: "plans-prod-container relative z-10 mx-auto max-w-7xl px-4 py-12",
                    header {
                        class: "plans-prod-hero mx-auto mb-16 text-center",
                        style: "margin-bottom: 64px;",
                        h1 { class: "plans-prod-title bg-gradient-to-r from-emerald-600 via-blue-600 to-purple-600 bg-clip-text text-4xl font-bold text-transparent md:text-6xl mb-6",
                            "Choose Your EPSX Plan"
                        }
                        p { class: "plans-prod-subtitle mx-auto max-w-3xl text-xl leading-relaxed text-gray-600 dark:text-gray-300",
                            "Unlock powerful analytics features, API access, and premium tools to supercharge your analytics experience"
                        }
                    }

                    // Keep the unavailable state compact so the source FAQ
                    // remains visible immediately after the failed catalog.
                    // Reserving the full plan-grid height makes the Rust
                    // route diverge from the development capture while still
                    // showing no verified catalog data.
                    div {
                        // The source `PlanSelection` returns its Alert
                        // directly, without an extra vertical wrapper. Keep
                        // only the responsive horizontal inset and the small
                        // two-pixel top breathing room needed after the hero.
                        class: "plans-unavailable-catalog px-4 pt-2",
                        section {
                            class: "plans-unavailable mx-auto max-w-4xl rounded-xl border-2 border-slate-300 bg-white/60 shadow-lg shadow-blue-950/10 backdrop-blur-xl dark:border-slate-300/90 dark:bg-slate-900/10",
                            role: "alert",
                            aria_labelledby: "plans-unavailable-title",
                            "data-section": "plans-unavailable",
                            div { class: "flex items-center gap-3 px-4 py-5 sm:px-4",
                                div { class: "flex h-7 w-7 shrink-0 items-center justify-center rounded-full border border-slate-200 text-slate-700 dark:border-slate-300/70 dark:text-slate-200",
                                    Icon { name: "alert-circle".to_string(), size: Some(16) }
                                }
                                h2 {
                                    id: "plans-unavailable-title",
                                    class: "text-base font-medium text-gray-900 dark:text-white sm:text-lg",
                                    "Failed to load plans. Please try again later."
                                }
                            }
                            div { class: "sr-only",
                                "Plan options cannot be verified right now. No plan names, prices, promotions, features, availability, eligibility, or subscription actions are shown until a verified public-plan response is available."
                            }
                        }
                    }

                    section {
                        class: "plans-faq mx-auto mt-20 max-w-3xl",
                        style: "margin-top: 80px;",
                        aria_labelledby: "plans-faq-title",
                        h2 { id: "plans-faq-title", class: "mb-12 text-center text-3xl font-bold text-gray-900 dark:text-white",
                            "Frequently Asked Questions"
                        }
                        div { class: "space-y-6",
                            FaqItem {
                                title: "Can I change my plan later?",
                                body: "Yes! You can upgrade or downgrade your plan at any time. Changes take effect immediately, and we'll prorate any billing adjustments.",
                            }
                            FaqItem {
                                title: "What happens to my API keys when I change plans?",
                                body: "Your API keys remain valid when upgrading. If downgrading removes API access, we'll notify you 7 days in advance so you can adjust your integrations.",
                            }
                            FaqItem {
                                title: "Do you offer custom enterprise plans?",
                                body: "Absolutely! We can create custom plans with specific features, higher limits, and dedicated support.",
                                link_label: Some("Contact us"),
                                link_href: Some(CONTACT_PATH),
                            }
                            FaqItem {
                                title: "Is there a free trial?",
                                body: "We offer a 7-day free trial for all premium plans. No credit card required - just sign up and start exploring advanced features immediately.",
                            }
                        }
                        nav {
                            // The source plans composition keeps these safe
                            // recovery links visible below the FAQ cards.
                            // They remain useful even while the catalog is
                            // unavailable, and preserve the route's visual
                            // footer at tablet and mobile widths.
                            class: "plans-catalog-alternatives",
                            "aria-label": "Plan catalog alternatives",
                            a { href: CONTACT_PATH,
                                Icon { name: "mail".to_string(), size: Some(16) }
                                "Contact support"
                            }
                            a { href: "/",
                                Icon { name: "home".to_string(), size: Some(16) }
                                "Return home"
                            }
                        }
                    }
                }
        }
    }
}

#[component]
fn FaqItem(
    title: &'static str,
    body: &'static str,
    #[props(default)] link_label: Option<&'static str>,
    #[props(default)] link_href: Option<&'static str>,
) -> Element {
    rsx! {
        article { class: "rounded-2xl bg-white p-6 shadow-lg dark:bg-slate-800/90 sm:p-8",
            h3 { class: "text-lg font-semibold text-gray-900 dark:text-white", "{title}" }
            p { class: "mt-3 text-base leading-relaxed text-gray-600 dark:text-gray-300",
                    "{body}"
                if let (Some(label), Some(href)) = (link_label, link_href) {
                    " "
                    a { class: "text-emerald-500 hover:underline", href, "{label}" }
                    " to discuss your needs."
                                }
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
        assert!(html.contains("plans-unavailable-catalog"));
        assert!(html.contains("role=\"alert\""));
        assert!(html.contains("aria-labelledby=\"plans-unavailable-title\""));
        assert!(html.contains("Plan options cannot be verified right now"));
        assert!(html.contains("Choose Your EPSX Plan"));
        assert!(html.contains("Frequently Asked Questions"));
        assert!(html.contains("Can I change my plan later?"));
        assert!(html.contains("Contact us</a> to discuss your needs."));
        assert!(
            html.find("Failed to load plans") < html.find("Frequently Asked Questions"),
            "FAQ should follow the compact unavailable catalog"
        );
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
        assert!(html.contains("class=\"plans-catalog-alternatives\""));
        assert!(!html.contains("class=\"plans-catalog-alternatives sr-only\""));
        assert!(!html.contains("href=\"/plans\""));
        assert!(html.contains("href=\"/contact\""));
        assert!(html.contains("href=\"/\""));
        assert!(!html.contains("Retry catalog"));
        assert!(html.contains("Contact support"));
        assert!(!html.contains("javascript:"));
    }
}

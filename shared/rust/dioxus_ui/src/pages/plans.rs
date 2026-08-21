//! `/plans` — public, backend-owned plan catalog.
//!
//! The frontend only renders the public projection returned by the Rust
//! backend. Pricing, promotion, visibility, grouping, and lifecycle values are
//! never derived from permissions or local sample data here.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use super::{PageContext, PageMeta};
use crate::layout::main_layout::MainLayout;
use crate::primitives::Icon;

const CONTACT_PATH: &str = "/contact";
pub const PLANS_DATA_PARAM: &str = "data_plans";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PublicPlan {
    pub id: String,
    pub name: String,
    pub plan_type: String,
    pub current_price: String,
    pub effective_price: f64,
    pub promotion_active: bool,
    pub promotion_status: String,
    pub promotion_discount: f64,
    pub promotion_ends_at: Option<String>,
    pub currency: String,
    pub billing_cycle: String,
    pub features: Vec<String>,
    pub permissions: Vec<String>,
    pub is_active: bool,
    pub tier_level: i32,
    pub plan_group: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum PublicPlansLoadOutcome {
    Ready { plans: Vec<PublicPlan> },
    Empty,
    Error { code: String },
}

#[derive(Clone, Debug, PartialEq)]
enum PlansLoad {
    Ready(Vec<PublicPlan>),
    Empty,
    Unavailable,
    Malformed,
}

fn plans_load(ctx: &PageContext) -> PlansLoad {
    let Some(raw) = ctx.params.get(PLANS_DATA_PARAM) else {
        return PlansLoad::Unavailable;
    };
    match serde_json::from_str::<PublicPlansLoadOutcome>(raw) {
        Ok(PublicPlansLoadOutcome::Ready { plans }) if !plans.is_empty() => PlansLoad::Ready(plans),
        Ok(PublicPlansLoadOutcome::Ready { .. } | PublicPlansLoadOutcome::Empty) => {
            PlansLoad::Empty
        }
        Ok(PublicPlansLoadOutcome::Error { code }) if code == "malformed_plans_response" => {
            PlansLoad::Malformed
        }
        Ok(PublicPlansLoadOutcome::Error { .. }) => PlansLoad::Unavailable,
        Err(_) => PlansLoad::Malformed,
    }
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Plans");
    (meta, rsx! { PlansPage { ctx: ctx.clone() } })
}

#[component]
fn PlansPage(ctx: PageContext) -> Element {
    let content = match plans_load(&ctx) {
        PlansLoad::Ready(plans) => rsx! { PlansReadyContent { plans } },
        PlansLoad::Empty => rsx! { PlansEmptyContent {} },
        PlansLoad::Unavailable => rsx! { PlansUnavailableContent {} },
        PlansLoad::Malformed => rsx! { PlansMalformedContent {} },
    };
    rsx! {
        MainLayout { ctx,
            {content}
        }
    }
}

fn display_price(plan: &PublicPlan) -> String {
    let value = if plan.promotion_active {
        plan.effective_price
    } else {
        plan.current_price.parse::<f64>().unwrap_or_default()
    };
    format!("{} {value:.2}", plan.currency)
}

fn billing_label(value: &str) -> String {
    value.replace('_', " ")
}

#[component]
fn PlansReadyContent(plans: Vec<PublicPlan>) -> Element {
    rsx! {
        div {
            class: "plans-prod-page relative min-h-screen overflow-hidden bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50 dark:from-gray-900 dark:via-gray-900 dark:to-indigo-900",
            "data-plans-state": "ready",
            div { class: "relative z-10 mx-auto max-w-7xl px-4 py-12",
                header { class: "mx-auto mb-12 max-w-3xl text-center",
                    h1 { class: "bg-gradient-to-r from-emerald-600 via-blue-600 to-purple-600 bg-clip-text text-4xl font-bold text-transparent md:text-6xl mb-6",
                        "Choose Your EPSX Plan"
                    }
                    p { class: "text-xl leading-relaxed text-gray-600 dark:text-gray-300",
                        "Compare the current public plans and features provided by EPSX."
                    }
                }
                section {
                    class: "grid grid-cols-1 gap-6 md:grid-cols-2 xl:grid-cols-3",
                    aria_label: "Available EPSX plans",
                    for plan in plans {
                        article {
                            class: "relative flex min-h-full flex-col rounded-2xl border border-slate-200 bg-white p-6 shadow-lg dark:border-slate-700 dark:bg-slate-800/90",
                            "data-plan-id": plan.id.clone(),
                            "data-plan-group": plan.plan_group.clone(),
                            div { class: "mb-4 flex items-start justify-between gap-3",
                                div {
                                    p { class: "text-xs font-semibold uppercase tracking-wider text-blue-600 dark:text-blue-300", "{plan.plan_group}" }
                                    h2 { class: "mt-1 text-2xl font-bold text-slate-950 dark:text-white", "{plan.name}" }
                                }
                                if plan.promotion_active {
                                    span { class: "shrink-0 rounded-full bg-emerald-100 px-3 py-1 text-xs font-bold text-emerald-800 dark:bg-emerald-900/50 dark:text-emerald-200",
                                        "{plan.promotion_discount:.0}% off"
                                    }
                                }
                            }
                            div { class: "mb-5",
                                p { class: "text-3xl font-black text-slate-950 dark:text-white", "{display_price(&plan)}" }
                                if plan.promotion_active {
                                    p { class: "mt-1 text-sm text-slate-500 line-through dark:text-slate-400", "{plan.currency} {plan.current_price}" }
                                }
                                p { class: "mt-1 text-sm capitalize text-slate-500 dark:text-slate-400", "{billing_label(&plan.billing_cycle)}" }
                            }
                            ul { class: "mb-6 flex-1 space-y-3",
                                for feature in plan.features.iter() {
                                    li { class: "flex items-start gap-2 text-sm leading-6 text-slate-700 dark:text-slate-200",
                                        Icon { name: "check".to_string(), size: Some(16), class_name: Some("mt-1 shrink-0 text-emerald-600".to_string()) }
                                        span { "{feature}" }
                                    }
                                }
                            }
                            a {
                                class: "inline-flex w-full items-center justify-center rounded-xl bg-blue-600 px-4 py-3 font-semibold text-white transition hover:bg-blue-700",
                                href: CONTACT_PATH,
                                "Ask about this plan"
                            }
                        }
                    }
                }
                PlansFaq {}
            }
        }
    }
}

#[component]
fn PlansEmptyContent() -> Element {
    rsx! { PlansProblemContent {
        state: "empty",
        title: "No public plans are available",
        message: "The backend returned an authoritative empty plan catalog.",
    } }
}

#[component]
fn PlansMalformedContent() -> Element {
    rsx! { PlansProblemContent {
        state: "malformed",
        title: "Plan data could not be verified",
        message: "The plan service returned an unexpected response, so no pricing claims are shown.",
    } }
}

#[component]
fn PlansProblemContent(state: &'static str, title: &'static str, message: &'static str) -> Element {
    rsx! {
        div { class: "plans-prod-page min-h-screen bg-slate-50 px-4 py-12 dark:bg-slate-900", "data-plans-state": state,
            div { class: "mx-auto max-w-4xl",
                h1 { class: "mb-8 text-center text-4xl font-bold text-slate-950 dark:text-white", "Choose Your EPSX Plan" }
                section { class: "rounded-xl border border-slate-300 bg-white p-6 text-center shadow dark:border-slate-700 dark:bg-slate-800", role: "status",
                    h2 { class: "text-xl font-semibold text-slate-950 dark:text-white", "{title}" }
                    p { class: "mt-2 text-slate-600 dark:text-slate-300", "{message}" }
                    a { class: "mt-5 inline-flex rounded-lg bg-blue-600 px-5 py-2 font-semibold text-white", href: "/plans", "Try again" }
                }
                PlansFaq {}
            }
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
                        color: #334155;
                        font-size: 1rem;
                        font-weight: 600;
                        text-decoration: none;
                        transition: color 0.15s ease, transform 0.15s ease;
                    }}
                    .plans-catalog-alternatives a:hover {{
                        color: #0f172a;
                        transform: translateY(-1px);
                    }}
                    .plans-catalog-alternatives i {{ color: #475569; flex-shrink: 0; }}
                    html.dark .plans-catalog-alternatives a {{ color: #e2e8f0; }}
                    html.dark .plans-catalog-alternatives a:hover {{ color: #ffffff; }}
                    html.dark .plans-catalog-alternatives i {{ color: #f8fafc; }}
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
                                body: "Plan-change availability and timing depend on the terms confirmed by the backend for your account.",
                            }
                            FaqItem {
                                title: "What happens to my API keys when I change plans?",
                                body: "API access is enforced by backend permissions. Review the developer portal after any confirmed plan change.",
                            }
                            FaqItem {
                                title: "Do you offer custom enterprise plans?",
                                body: "Absolutely! We can create custom plans with specific features, higher limits, and dedicated support.",
                                link_label: Some("Contact us"),
                                link_href: Some(CONTACT_PATH),
                            }
                            FaqItem {
                                title: "Is there a free trial?",
                                body: "A trial or promotion is available only when it appears in the current backend-provided plan catalog.",
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
fn PlansFaq() -> Element {
    rsx! {
        section {
            class: "plans-faq mx-auto mt-20 max-w-3xl",
            aria_labelledby: "plans-faq-title",
            h2 { id: "plans-faq-title", class: "mb-12 text-center text-3xl font-bold text-gray-900 dark:text-white",
                "Frequently Asked Questions"
            }
            div { class: "space-y-6",
                FaqItem {
                    title: "Can I change my plan later?",
                    body: "Plan-change availability and timing depend on the terms confirmed by the backend for your account.",
                }
                FaqItem {
                    title: "What happens to my API keys when I change plans?",
                    body: "API access is enforced by backend permissions. Review the developer portal after any confirmed plan change.",
                }
                FaqItem {
                    title: "Do you offer custom enterprise plans?",
                    body: "Absolutely! We can create custom plans with specific features, higher limits, and dedicated support.",
                    link_label: Some("Contact us"),
                    link_href: Some(CONTACT_PATH),
                }
                FaqItem {
                    title: "Is there a free trial?",
                    body: "A trial or promotion is available only when it appears in the current backend-provided plan catalog.",
                }
            }
            nav {
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
                    a { class: "text-emerald-700 hover:underline dark:text-emerald-400", href, "{label}" }
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

    fn verified_plan() -> PublicPlan {
        PublicPlan {
            id: "61a62cbe-3371-41db-bd90-321c53a71e06".to_string(),
            name: "Verified Pro".to_string(),
            plan_type: "PRO".to_string(),
            current_price: "20.00".to_string(),
            effective_price: 15.0,
            promotion_active: true,
            promotion_status: "active".to_string(),
            promotion_discount: 25.0,
            promotion_ends_at: None,
            currency: "USD".to_string(),
            billing_cycle: "monthly".to_string(),
            features: vec!["Live analytics".to_string()],
            permissions: vec!["epsx:analytics:read".to_string()],
            is_active: true,
            tier_level: 2,
            plan_group: "personal".to_string(),
        }
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

            assert!(html.contains("data-plans-state=\"malformed\""));
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
    fn verified_backend_projection_renders_catalog_without_permission_claims() {
        let mut ctx = page_ctx();
        ctx.params.insert(
            PLANS_DATA_PARAM.to_string(),
            serde_json::to_string(&PublicPlansLoadOutcome::Ready {
                plans: vec![verified_plan()],
            })
            .unwrap(),
        );
        let html = render_html(&ctx);

        assert!(html.contains("data-plans-state=\"ready\""));
        assert!(html.contains("Verified Pro"));
        assert!(html.contains("USD 15.00"));
        assert!(html.contains("USD 20.00"));
        assert!(html.contains("25% off"));
        assert!(html.contains("Live analytics"));
        assert!(!html.contains("epsx:analytics:read"));
        assert!(!html.contains("Subscribe"));
        assert!(html.contains("Ask about this plan"));
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

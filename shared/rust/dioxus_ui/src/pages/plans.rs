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
    #[serde(default)]
    pub promotion_savings: String,
    pub promotion_ends_at: Option<String>,
    pub currency: String,
    pub billing_cycle: String,
    pub features: Vec<String>,
    pub permissions: Vec<String>,
    pub is_active: bool,
    pub tier_level: i32,
    pub plan_group: String,
    /// Backend-owned number of leading market ranks hidden by this plan.
    /// `0` and `1` both represent top-rank access for legacy catalog rows.
    pub ranking_offset: i32,
    /// Backend-owned maximum ranking inventory for the plan. `-1` is unlimited.
    pub rankings_limit: i32,
    /// Exact backend-owned stablecoin amount used by checkout.
    pub checkout_price: String,
    /// Stablecoin used for on-chain settlement (USDT or USDC).
    pub settlement_currency: String,
    /// Backend-owned access duration. `None` represents lifetime access.
    pub duration_days: Option<i64>,
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

pub fn display_price(plan: &PublicPlan) -> String {
    if plan.currency == plan.settlement_currency {
        format!("{} {}", plan.currency, plan.checkout_price)
    } else {
        format!("{} {:.2}", plan.currency, plan.effective_price)
    }
}

pub fn billing_label(value: &str) -> String {
    value.replace('_', " ")
}

pub fn ranking_access_label(plan: &PublicPlan) -> String {
    let first_rank = plan.ranking_offset.max(0).saturating_add(1);
    match plan.rankings_limit {
        -1 => format!("Stock rankings from rank {first_rank} · unlimited inventory"),
        limit => {
            let last_rank = first_rank.saturating_add(limit.max(1)).saturating_sub(1);
            format!("Stock rankings {first_rank}-{last_rank} · {limit} results")
        }
    }
}

#[component]
pub fn PlanCard(plan: PublicPlan, #[props(default)] frontend: bool) -> Element {
    rsx! {
        article {
            class: "plan-card relative flex min-h-full flex-col rounded-2xl border border-slate-200 bg-white p-6 shadow-lg dark:border-slate-700 dark:bg-slate-800/90 fe-surface",
            "data-plan-id": plan.id.clone(),
            "data-plan-group": plan.plan_group.clone(),
            div { class: "mb-4 flex items-start justify-between gap-3",
                div {
                    p { class: "text-xs font-semibold uppercase tracking-wider text-blue-600 dark:text-blue-300", "{plan.plan_group}" }
                    h2 { class: "mt-1 text-2xl font-bold text-slate-900 dark:text-white fe-tone-text", "{plan.name}" }
                }
                if plan.promotion_active {
                    span { class: if frontend { "fe-plan-discount" } else { "plan-discount-badge shrink-0 rounded-full bg-emerald-100 px-3 py-1 text-xs font-bold text-emerald-800 ring-1 ring-emerald-600/10 dark:bg-emerald-700 dark:text-white dark:ring-emerald-500/30" },
                        "{plan.promotion_discount:.0}% off"
                    }
                }
            }
            div { class: "mb-5",
                {
                    let is_custom = plan.plan_group == "custom";
                    if is_custom {
                        rsx! {
                            p { class: "text-3xl font-black text-purple-700 dark:text-purple-300 fe-tone-accent", "Contact us" }
                            p { class: "mt-1 text-sm capitalize text-slate-500 dark:text-slate-400 fe-tone-muted", "Custom plan" }
                        }
                    } else {
                        rsx! {
                            p { class: "text-3xl font-black text-slate-900 dark:text-white fe-tone-text", "{display_price(&plan)}" }
                            if plan.promotion_active {
                                p { class: "mt-1 text-sm text-slate-500 line-through dark:text-slate-400 fe-tone-muted", "{plan.currency} {plan.current_price}" }
                                if !plan.promotion_savings.is_empty() {
                                    p { class: "fe-sale-saving", "Save {plan.currency} {plan.promotion_savings}" }
                                }
                                p { class: "fe-sale-caption", "Sale price · one-time payment" }
                                if let Some(ends) = plan.promotion_ends_at.as_deref().filter(|s| !s.is_empty()) {
                                    p { class: "fe-sale-caption", "Offer ends ", time { datetime: ends, "{ends}" } }
                                }
                            }
                            p { class: "mt-1 text-sm capitalize text-slate-500 dark:text-slate-400 fe-tone-muted", "{billing_label(&plan.billing_cycle)}" }
                        }
                    }
                }
            }
            ul { class: "mb-6 flex-1 space-y-3",
                for feature in plan.features.iter() {
                    li { class: "flex items-start gap-2 text-sm leading-6 text-slate-700 dark:text-slate-200",
                        Icon { name: "check".to_string(), size: Some(16), class_name: Some("mt-1 shrink-0 text-emerald-600".to_string()) }
                        span { "{feature}" }
                    }
                }
            }
            p {
                class: if frontend { "fe-plan-access" } else { "plan-ranking-pill mb-4 rounded-lg bg-blue-50 px-3 py-2 text-sm font-semibold text-blue-800 ring-1 ring-blue-200/60 dark:bg-blue-700 dark:text-white dark:ring-blue-500/30" },
                {if frontend { ranking_access_label(&plan).replacen("Stock rankings", "Company rankings", 1) } else { ranking_access_label(&plan) }}
            }
            {
                let is_custom = plan.plan_group == "custom";
                if is_custom {
                    rsx! {
                        crate::fullstack::shell::ShellLink {
                            class: "inline-flex w-full items-center justify-center rounded-xl bg-gradient-to-r from-purple-600 to-fuchsia-500 px-4 py-3 font-semibold text-white transition hover:from-purple-600 hover:to-fuchsia-600 fe-fill-neutral fe-tone-text fe-action-primary",
                            href: "/contact",
                            Icon { name: "message-square".to_string(), size: Some(16) }
                            "Get in Touch"
                        }
                    }
                } else {
                    rsx! {
                        crate::fullstack::shell::ShellLink {
                            class: "inline-flex w-full items-center justify-center rounded-xl bg-blue-600 px-4 py-3 font-semibold text-white transition hover:bg-blue-700 fe-tone-text fe-action-primary",
                            href: format!("/payment/plan/{}", plan.id),
                            "Review plan"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PlansReadyContent(plans: Vec<PublicPlan>) -> Element {
    rsx! {
        div {
            class: "plans-prod-page relative min-h-screen overflow-hidden bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50 dark:from-gray-900 dark:via-gray-900 dark:to-indigo-900 fe-base-page fe-fill-neutral",
            "data-plans-state": "ready",
            div { class: "relative z-10 mx-auto max-w-7xl px-4 py-12",
                header { class: "mx-auto mb-12 max-w-3xl text-center",
                    h1 { class: "bg-gradient-to-r from-emerald-600 via-blue-600 to-purple-600 bg-clip-text text-4xl font-bold text-transparent md:text-6xl mb-6 fe-fill-neutral fe-type-title",
                        "Plans"
                    }
                    p { class: "text-xl leading-relaxed text-gray-600 dark:text-gray-300 fe-tone-muted",
                        "Compare the current public plans and features provided by EPSX."
                    }
                }
                section {
                    class: "grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3",
                    aria_label: "Available EPSX plans",
                    for plan in plans {
                        PlanCard { plan: plan.clone(), frontend: true }
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
        message: "No public plans are available right now.",
    } }
}

#[component]
fn PlansMalformedContent() -> Element {
    rsx! { PlansProblemContent {
        state: "malformed",
        title: "Plans are temporarily unavailable",
        message: "We couldn’t load the current plans. Please try again.",
    } }
}

#[component]
fn PlansProblemContent(state: &'static str, title: &'static str, message: &'static str) -> Element {
    rsx! {
        div { class: "plans-prod-page min-h-screen bg-slate-50 px-4 py-12 dark:bg-slate-900 fe-base-page fe-fill-neutral", "data-plans-state": state,
            div { class: "mx-auto max-w-4xl",
                h1 { class: "mb-8 text-center text-4xl font-bold text-slate-900 dark:text-white fe-tone-text fe-type-title", "Plans" }
                section { class: "rounded-xl border border-slate-300 bg-white p-6 text-center shadow dark:border-slate-700 dark:bg-slate-800 fe-surface", role: "status",
                    h2 { class: "text-xl font-semibold text-slate-900 dark:text-white fe-tone-text", "{title}" }
                    p { class: "mt-2 text-slate-600 dark:text-slate-300 fe-tone-muted", "{message}" }
                    crate::fullstack::shell::ShellLink { class: "mt-5 inline-flex rounded-lg bg-blue-600 px-5 py-2 font-semibold text-white fe-tone-text fe-action-primary", href: "/plans", "Try again" }
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
                class: "plans-prod-page relative min-h-screen overflow-hidden bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50 dark:from-gray-900 dark:via-gray-900 dark:to-indigo-900 fe-base-page fe-fill-neutral",
                "data-plans-state": "unavailable",

                div { class: "plans-prod-container relative z-10 mx-auto max-w-7xl px-4 py-12",
                    header {
                        class: "plans-prod-hero mx-auto mb-[64px]! text-center",
                        h1 { class: "plans-prod-title bg-gradient-to-r from-emerald-600 via-blue-600 to-purple-600 bg-clip-text text-4xl font-bold text-transparent md:text-6xl mb-6 fe-fill-neutral fe-type-title",
                            "Plans"
                        }
                        p { class: "plans-prod-subtitle mx-auto max-w-3xl text-xl leading-relaxed text-gray-600 dark:text-gray-300 fe-tone-muted",
                            "Compare available plans and review the price and access included in each."
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
                            class: "plans-unavailable mx-auto max-w-4xl rounded-xl border-2 border-slate-300 bg-white/60 shadow-lg shadow-blue-950/10 backdrop-blur-xl dark:border-slate-300/90 dark:bg-slate-900/10 fe-fill-neutral",
                            role: "alert",
                            aria_labelledby: "plans-unavailable-title",
                            "data-section": "plans-unavailable",
                            div { class: "flex items-center gap-3 px-4 py-5 sm:px-4",
                                div { class: "flex h-7 w-7 shrink-0 items-center justify-center rounded-full border border-slate-200 text-slate-700 dark:border-slate-300/70 dark:text-slate-200",
                                    Icon { name: "alert-circle".to_string(), size: Some(16) }
                                }
                                h2 {
                                    id: "plans-unavailable-title",
                                    class: "text-base font-medium text-gray-900 dark:text-white sm:text-lg fe-tone-text",
                                    "Failed to load plans. Please try again later."
                                }
                            }
                            div { class: "sr-only",
                                "Plan options cannot be verified right now. No plan names, prices, promotions, features, availability, eligibility, or subscription actions are shown until a verified public-plan response is available."
                            }
                        }
                    }

                    PlansFaq {}
                }
        }
    }
}

#[component]
fn PlansFaq() -> Element {
    rsx! {
        section {
            class: "plans-faq",
            aria_labelledby: "plans-faq-title",
            h2 { id: "plans-faq-title", class: "plans-faq-heading fe-tone-text",
                "Frequently Asked Questions"
            }
            div { class: "plans-faq-list",
                FaqItem {
                    title: "Can I change my plan later?",
                    body: "Review your plan terms for availability and timing of changes.",
                }
                FaqItem {
                    title: "What happens to my API keys when I change plans?",
                    body: "Your plan determines API access. Review the developer portal after changing plans.",
                }
                FaqItem {
                    title: "Do you offer custom enterprise plans?",
                    body: "Absolutely! We can create custom plans with specific features, higher limits, and dedicated support.",
                    link_label: Some("Contact us"),
                    link_href: Some(CONTACT_PATH),
                }
                FaqItem {
                    title: "Is there a free trial?",
                    body: "Available trials and promotions are shown alongside each plan.",
                }
            }
            nav {
                class: "plans-catalog-alternatives",
                "aria-label": "Plan catalog alternatives",
                crate::fullstack::shell::ShellLink { href: CONTACT_PATH,
                    Icon { name: "mail".to_string(), size: Some(16) }
                    "Contact support"
                }
                crate::fullstack::shell::ShellLink { href: "/",
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
        article { class: "plans-faq-item",
            h3 { class: "text-lg font-semibold text-gray-900 dark:text-white fe-tone-text", "{title}" }
            p { class: "mt-3 text-base leading-relaxed text-gray-600 dark:text-gray-300 fe-tone-muted",
                    "{body}"
                if let (Some(label), Some(href)) = (link_label, link_href) {
                    " "
                    crate::fullstack::shell::ShellLink { class: "text-emerald-700 hover:underline dark:text-emerald-400 fe-tone-positive", href, "{label}" }
                    " to discuss your needs."
                                }
                            }
        }
    }
}

#[cfg(feature = "server")]
pub type PlansProviderCallback = std::sync::Arc<
    dyn Fn() -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = Result<PublicPlansLoadOutcome, crate::fullstack::LoadError>,
                    > + Send,
            >,
        > + Send
        + Sync,
>;

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
            promotion_savings: "5.00".into(),
            promotion_ends_at: None,
            currency: "USD".to_string(),
            billing_cycle: "monthly".to_string(),
            features: vec!["Live analytics".to_string()],
            permissions: vec!["epsx:analytics:read".to_string()],
            is_active: true,
            tier_level: 2,
            plan_group: "personal".to_string(),
            ranking_offset: 0,
            rankings_limit: -1,
            checkout_price: "9.90".to_string(),
            settlement_currency: "USDT".to_string(),
            duration_days: Some(30),
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
            "Review plan",
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
        assert!(html.contains("Plans"));
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
        assert!(html.contains("Save USD 5.00"));
        assert!(html.contains("Sale price · one-time payment"));
        assert!(html.contains("Live analytics"));
        assert!(!html.contains("epsx:analytics:read"));
        assert!(!html.contains("Subscribe"));
        assert!(html.contains("Review plan"));
        assert!(html.contains("/payment/plan/61a62cbe-3371-41db-bd90-321c53a71e06"));
        assert!(html.contains("Company rankings from rank 1 · unlimited inventory"));
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

#[cfg(feature = "server")]
#[derive(Clone)]
pub struct PlansProvider(pub PlansProviderCallback);

#[server(prefix = "/_server/frontend", endpoint = "plans")]
pub async fn read_public_plans(
) -> Result<Result<PublicPlansLoadOutcome, crate::fullstack::LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<PlansProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("Plans provider unavailable"))?;
    Ok((provider.0)().await)
}

#[component]
pub fn HydratedPlans() -> Element {
    let mut result = use_server_future(move || async move { read_public_plans().await })?;
    let outcome = result.read().clone();
    rsx! {
        document::Title { "Plans — EPSX" }
        document::Meta { name: "description", content: "Compare the current public plans and features provided by EPSX." }
        section { "data-dioxus-plans": "true",
            match outcome {
                Some(Ok(Ok(PublicPlansLoadOutcome::Ready { plans }))) => rsx! { PlansReadyContent { plans } },
                Some(Ok(Ok(PublicPlansLoadOutcome::Empty))) => rsx! { PlansEmptyContent {} },
                None => rsx! { p { role: "status", "Loading plans…" } },
                _ => rsx! { div { role: "status", class: "fe-purchase-note",
                    p { "Plans are temporarily unavailable." }
                    button { r#type: "button", class: "fe-button", onclick: move |_| result.restart(), "Try again" }
                } },
            }
        }
    }
}

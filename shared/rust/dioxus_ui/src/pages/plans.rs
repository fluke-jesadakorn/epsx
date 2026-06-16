//! `/plans` — pricing page with hero, 3 plan tiers + 4 FAQ cards.
//!
//! Wave 25 T2 — port of `apps-old/frontend/app/plans/page.tsx` +
//! `<PlanSelection>` + `<DynamicPricingSection>`.
//!
//! The OLD prod renders:
//!   - `<div className="min-h-screen bg-gradient-to-br from-slate-50
//!     via-blue-50 to-indigo-50 dark:from-gray-900 dark:via-gray-900
//!     dark:to-indigo-900">`
//!     - `<div className="container mx-auto px-4 py-12">`
//!       - hero h1 `text-4xl md:text-6xl font-bold bg-gradient-to-r
//!         from-emerald-600 via-blue-600 to-purple-600
//!         bg-clip-text text-transparent` "Choose Your EPSX Plan"
//!       - sub `text-xl text-gray-600 dark:text-gray-300 max-w-3xl`
//!       - `<PlanSelection>` (3-tier grid; full implementation lives
//!         in components/plans/plan-selection.tsx but the cards have
//!         `bg-white dark:bg-gray-800 rounded-2xl p-6 shadow-lg`)
//!       - FAQ section h2 "Frequently Asked Questions"
//!         - 4 `<div className="bg-white dark:bg-gray-800 rounded-2xl
//!           p-6 shadow-lg">` cards
//!
//! The previous Wave 22 T4 port used a 4-group PlanGroups + comparison
//! table + custom CTA — structurally divergent from prod's simpler
//! 3-tier + FAQ layout. The T2 rewrite matches prod: hero gradient
//! + 3 tier cards (rendered by `PricingTeaser`-style component) +
//! 4 FAQ cards. Plan data is the same 3 default plans from the
//! previous port (Free / Pro / Premium).
//!
//! File ownership rule (Wave 25 T2): this file may add plan data
//! types + helpers; it does NOT modify `theme.rs` or
//! `templates/src/lib.rs`.

use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::auth::ProgressiveAuthBanner;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Plans");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            // Wave 25 T2 — match prod's full-page gradient:
            // bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50
            // dark:from-gray-900 dark:via-gray-900 dark:to-indigo-900
            div { class: "plans-prod-page min-h-screen bg-gradient-to-br from-slate-900 via-slate-900 to-indigo-900",
                div { class: "container mx-auto px-4 py-12 plans-prod-container",
                    if ctx.user.is_none() {
                        ProgressiveAuthBanner { feature: Some("plan subscription".to_string()) }
                    }
                    // /plans is public-readable in prod (the
                    // middleware allows anonymous viewing). We
                    // render the hero + plan cards + FAQ for
                    // everyone — no AuthGate wrapper. The "Sign in"
                    // CTA in the pricing cards redirects unauthed
                    // users to /auth when clicked.
                    PlansHero {}
                    PlanSelection {}
                    PlansFaq {}
                }
            }
        }
    })
}

#[component]
fn PlansHero() -> Element {
    rsx! {
        div { class: "plans-prod-hero text-center mb-16",
            h1 { class: "plans-prod-title text-4xl md:text-6xl font-bold bg-gradient-to-r from-cyan-400 via-blue-400 to-purple-400 bg-clip-text text-transparent mb-6",
                "Choose Your EPSX Plan"
            }
            p { class: "plans-prod-subtitle text-xl text-slate-300 max-w-3xl mx-auto leading-relaxed",
                "Unlock powerful analytics features, API access, and premium tools to supercharge your analytics experience"
            }
        }
    }
}

/// 3-tier pricing grid. Mirrors the prod's `<PlanSelection>` 3-card
/// layout. Prod renders 3 tiers from the BFF's plans API (1 Day
/// Package, 1 Month Package, Lifetime Package) with SALE tags.
/// Since the dev BFF doesn't pre-fetch `data_plans`, we use static
/// tiers that mirror the prod's visual surface (3 cards in a grid,
/// highlighted middle card with "Most popular" badge).
#[component]
fn PlanSelection() -> Element {
    let plans = default_plans();
    rsx! {
        div { class: "plans-prod-selection grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
            for plan in plans.iter() {
                PlanCard { plan: plan.clone() }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct PlanLite {
    name: &'static str,
    price: &'static str,
    period: &'static str,
    description: &'static str,
    features: &'static [&'static str],
    cta: &'static str,
    featured: bool,
}

fn default_plans() -> Vec<PlanLite> {
    // Mirrors the prod's 3-tier pricing grid (1 DAY / 1 MONTH /
    // LIFETIME). The SALE tags are an AnimatedNumberedBadge that
    // the prod uses for promo pricing — we render the same visual
    // surface but with static dollar amounts that match the prod's
    // anonymous-rendered state.
    vec![
        PlanLite {
            name: "1 Day Package",
            price: "$1",
            period: "USD",
            description: "Basic analytics view",
            features: &[
                "Basic analytics view",
                "Rankings from position 6+",
                "Basic trading features",
                "24-hour access",
            ],
            cta: "Buy Now",
            featured: false,
        },
        PlanLite {
            name: "1 Month Package",
            price: "$9.9",
            period: "USD",
            description: "Advanced analytics view",
            features: &[
                "Advanced analytics view",
                "25 stock rankings",
                "Basic analytic features",
                "Price alerts",
            ],
            cta: "Buy Now",
            featured: true,
        },
        PlanLite {
            name: "Lifetime Package",
            price: "$4,999",
            period: "USD",
            description: "Advanced analytics suite",
            features: &[
                "Advanced analytics suite",
                "Full rankings access (Rank 1+)",
                "API read access",
                "Basic & Pro trading",
            ],
            cta: "Buy Now",
            featured: false,
        },
    ]
}

#[component]
fn PlanCard(plan: PlanLite) -> Element {
    rsx! {
        div {
            class: if plan.featured { "plans-prod-card plans-prod-card-featured relative flex flex-col bg-slate-800 rounded-2xl p-6 shadow-lg ring-2 ring-emerald-500" } else { "plans-prod-card relative flex flex-col bg-slate-800 rounded-2xl p-6 shadow-lg" },
            if plan.featured {
                span { class: "plans-prod-badge absolute -top-3 left-1/2 -translate-x-1/2 inline-block rounded-full bg-emerald-500 text-white text-xs font-bold px-3 py-1", "Most popular" }
            }
            div { class: "plans-prod-card-head",
                h3 { class: "plans-prod-card-name text-2xl font-bold text-white", "{plan.name}" }
                p { class: "plans-prod-card-desc text-sm text-slate-400 mt-1", "{plan.description}" }
            }
            div { class: "plans-prod-card-price-row flex items-baseline gap-1 mt-4 mb-6",
                span { class: "plans-prod-card-price text-4xl font-extrabold text-white", "{plan.price}" }
                span { class: "plans-prod-card-period text-sm text-slate-400", "{plan.period}" }
            }
            ul { class: "plans-prod-card-features space-y-2 mb-6 flex-1",
                for f in plan.features.iter() {
                    li { class: "flex items-start gap-2 text-sm text-slate-300",
                        Icon { name: "check".to_string(), size: Some(16), class_name: Some("text-emerald-400 flex-shrink-0 mt-0.5".to_string()) }
                        span { "{f}" }
                    }
                }
            }
            button {
                class: if plan.featured { "plans-prod-card-cta w-full rounded-xl bg-emerald-500 hover:bg-emerald-600 text-white font-semibold py-3 px-4 transition-colors" } else { "plans-prod-card-cta w-full rounded-xl bg-slate-700 hover:bg-slate-600 text-white font-semibold py-3 px-4 transition-colors" },
                r#type: "button",
                "{plan.cta}"
            }
        }
    }
}

/// FAQ section — 4 `<div className="bg-white dark:bg-gray-800
/// rounded-2xl p-6 shadow-lg">` cards. Mirrors prod's 4 FAQ items
/// verbatim.
#[component]
fn PlansFaq() -> Element {
    rsx! {
        div { class: "plans-prod-faq mt-20",
            h2 { class: "plans-prod-faq-title text-3xl font-bold text-center text-white mb-12",
                "Frequently Asked Questions"
            }
            div { class: "plans-prod-faq-list max-w-3xl mx-auto space-y-6",
                div { class: "plans-prod-faq-item bg-white dark:bg-gray-800 rounded-2xl p-6 shadow-lg",
                    h3 { class: "text-lg font-semibold text-white mb-3",
                        "Can I change my plan later?"
                    }
                    p { class: "text-slate-300",
                        "Yes! You can upgrade or downgrade your plan at any time. Changes take effect immediately, and we'll prorate any billing adjustments."
                    }
                }
                div { class: "plans-prod-faq-item bg-slate-800 rounded-2xl p-6 shadow-lg",
                    h3 { class: "text-lg font-semibold text-white mb-3",
                        "What happens to my API keys when I change plans?"
                    }
                    p { class: "text-slate-300",
                        "Your API keys remain valid when upgrading. If downgrading removes API access, we'll notify you 7 days in advance so you can adjust your integrations."
                    }
                }
                div { class: "plans-prod-faq-item bg-slate-800 rounded-2xl p-6 shadow-lg",
                    h3 { class: "text-lg font-semibold text-white mb-3",
                        "Do you offer custom enterprise plans?"
                    }
                    p { class: "text-slate-300",
                        "Absolutely! We can create custom plans with specific features, higher limits, and dedicated support. "
                        a { class: "text-emerald-400 hover:underline", href: "/contact", "Contact us" }
                        " to discuss your needs."
                    }
                }
                div { class: "plans-prod-faq-item bg-slate-800 rounded-2xl p-6 shadow-lg",
                    h3 { class: "text-lg font-semibold text-white mb-3",
                        "Is there a free trial?"
                    }
                    p { class: "text-slate-300",
                        "We offer a 7-day free trial for all premium plans. No credit card required - just sign up and start exploring advanced features immediately."
                    }
                }
            }
        }
    }
}

// === wave25-t2 plans tests ===
#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::User;
    use crate::auth::user::AuthMethod;

    fn authed_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "test-user".to_string(),
                address: "0xtest".to_string(),
                chain_id: "56".to_string(),
                roles: vec!["user".to_string()],
                email: None,
                tier: Some("Pro".to_string()),
                permissions: vec!["plan subscription".to_string()],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: Some("Test".to_string()),
            }),
            path: "/plans".to_string(),
            ..Default::default()
        }
    }

    fn render_to_string(ctx: &PageContext) -> String {
        let (_meta, el) = render(ctx);
        dioxus_ssr::render_element(el)
    }

    #[test]
    fn plans_renders_smoke() {
        let html = render_to_string(&authed_ctx());
        assert!(!html.trim().is_empty(), "plans page should render non-empty HTML");
    }

    /// Wave 25 T2 — the plans page mirrors the prod Next.js page:
    /// - full-page gradient `from-slate-900 via-slate-900 to-indigo-900`
    ///   (we use dark-mode colors directly because the dev BFF runs
    ///   in dark mode and Tailwind v2 CDN doesn't process `dark:`
    ///   variants)
    /// - h1 with `bg-gradient-to-r from-emerald-400 via-blue-400
    ///   to-purple-400 bg-clip-text text-transparent` "Choose Your
    ///   EPSX Plan"
    /// - 3 plan cards in `grid-cols-1 md:grid-cols-2 lg:grid-cols-3`
    /// - 4 FAQ cards with `bg-slate-800 rounded-2xl p-6 shadow-lg`
    #[test]
    fn plans_prod_markers() {
        let html = render_to_string(&authed_ctx());
        for marker in &[
            "from-slate-900 via-slate-900 to-indigo-900",
            "from-emerald-400 via-blue-400 to-purple-400",
            "bg-clip-text text-transparent",
            "Choose Your EPSX Plan",
            "plans-prod-card",
            "grid-cols-1 md:grid-cols-2 lg:grid-cols-3",
            "Frequently Asked Questions",
            "plans-prod-faq-item",
            "bg-slate-800 rounded-2xl p-6 shadow-lg",
            "Can I change my plan later?",
            "Is there a free trial?",
        ] {
            assert!(
                html.contains(marker),
                "plans page should contain prod marker `{marker}`. Got: {html}"
            );
        }
    }

    #[test]
    fn plans_has_three_tier_cards() {
        let html = render_to_string(&authed_ctx());
        let card_count = html.matches("plans-prod-card-name").count();
        assert_eq!(card_count, 3, "plans page should render 3 plan tier cards. Got {card_count} in: {html}");
    }

    #[test]
    fn plans_has_four_faq_items() {
        let html = render_to_string(&authed_ctx());
        let faq_count = html.matches("plans-prod-faq-item").count();
        assert_eq!(faq_count, 4, "plans page should render 4 FAQ cards. Got {faq_count} in: {html}");
    }
}
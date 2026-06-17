//! `/plans` — pricing page with hero, 3 plan tiers + 4 FAQ cards.
//!
//! Wave 25 T2 — port of `apps-old/frontend/app/plans/page.tsx` +
//! `<PlanSelection>` + `<DynamicPricingSection>`.
//!
//! The prod Next.js page renders for **all visitors** (including
//! anonymous):
//!   - `<div className="min-h-screen bg-gradient-to-br from-slate-50
//!     via-blue-50 to-indigo-50 dark:from-gray-900 dark:via-gray-900
//!     dark:to-indigo-900">`
//!   - `<div className="container mx-auto px-4 py-12">`
//!   - hero h1 `text-4xl md:text-6xl font-bold bg-gradient-to-r
//!     from-emerald-600 via-blue-600 to-purple-600 bg-clip-text
//!     text-transparent` "Choose Your EPSX Plan"
//!   - sub `text-xl text-gray-600 dark:text-gray-300 max-w-3xl`
//!   - `<PlanSelection>` (3 cards in `grid-cols-1 md:grid-cols-2
//!     lg:grid-cols-3`) — each card has:
//!     - red SALE ribbon top-left (`absolute -top-px -left-px
//!       z-20 bg-red-500 text-white text-xs font-bold px-3 py-1
//!       rounded-br-3xl uppercase tracking-wider`)
//!     - red discount badge (`bg-red-500 text-white text-xs
//!       font-bold px-2 py-0.5 rounded-full self-center`)
//!     - crossed-out original price (`line-through`)
//!     - green "Save $X" text
//!     - flame icon + "Ends in NaNm" timer
//!     - feature list
//!   - FAQ section with 4 questions
//!
//! The dev BFF doesn't pre-fetch the live plans API; we render the
//! static plan list that mirrors prod's anonymous-rendered state
//! (1 Day $1 / 1 Month $9.9 / Lifetime $4,999 with their SALE
//! tags, crossed-out original prices, savings text, and timer).

use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Plans");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            // Wave 29 T1 — match prod's full-page gradient verbatim
            // (apps-old/frontend/app/plans/page.tsx:10):
            //   bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50
            //   dark:from-gray-900 dark:via-gray-900 dark:to-indigo-900
            // Wave 25 T2 used `from-slate-950 via-slate-900 to-slate-950`
            // which misses prod's indigo end stop and is too dark.
            div { class: "plans-prod-page min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50 dark:from-gray-900 dark:via-gray-900 dark:to-indigo-900",
                div { class: "container mx-auto px-4 py-12 plans-prod-container",
                    // Hero
                    PlansHero {}
                    // 3-tier pricing grid (1 DAY / 1 MONTH / LIFETIME)
                    PlanSelection {}
                    // FAQ section
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
            // Wave 29 T1 — match prod's h1 gradient verbatim
            // (apps-old/frontend/app/plans/page.tsx:14):
            //   bg-gradient-to-r from-emerald-600 via-blue-600 to-purple-600
            //   bg-clip-text text-transparent
            // Wave 25 T2 used `from-cyan-400 via-blue-400 to-purple-400`
            // which is the wrong palette (cyan not emerald, 400 not 600).
            h1 { class: "plans-prod-title text-4xl md:text-6xl font-bold bg-gradient-to-r from-emerald-600 via-blue-600 to-purple-600 bg-clip-text text-transparent mb-6",
                "Choose Your EPSX Plan"
            }
            // Subtitle — prod uses
            // `text-xl text-gray-600 dark:text-gray-300 max-w-3xl mx-auto`.
            // Wave 25 T2 used `text-slate-300` (no light variant). Fix.
            p { class: "plans-prod-subtitle text-xl text-gray-600 dark:text-gray-300 max-w-3xl mx-auto leading-relaxed",
                "Unlock powerful analytics features, API access, and premium tools to supercharge your analytics experience"
            }
        }
    }
}

/// 3-tier pricing grid. Mirrors prod's `<PlanSelection>` 3-card
/// layout. Each card has the prod's red SALE ribbon + discount
/// badge + crossed-out price + green savings text + flame timer.
#[component]
fn PlanSelection() -> Element {
    let plans = default_plans();
    rsx! {
        div { class: "plans-prod-selection grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
            for plan in plans.iter() {
                PricingCard { plan: plan.clone() }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct PlanLite {
    name: &'static str,
    price: &'static str,
    original_price: &'static str,
    discount_pct: &'static str,
    save_amount: &'static str,
    description: &'static str,
    features: &'static [&'static str],
    cta: &'static str,
}

fn default_plans() -> Vec<PlanLite> {
    // Mirrors the prod's 3-tier pricing grid (1 DAY / 1 MONTH /
    // LIFETIME). The static seed matches prod's anonymous-rendered
    // state including SALE ribbon + crossed-out price + savings
    // text + flame timer.
    vec![
        PlanLite {
            name: "1 Day Package",
            price: "$1 USD",
            original_price: "$5 USD",
            discount_pct: "80% OFF",
            save_amount: "Save $4",
            description: "Basic analytics view",
            features: &[
                "Basic analytics view",
                "Rankings from position 6+",
                "Basic trading features",
                "24-hour access",
            ],
            cta: "Buy Now",
        },
        PlanLite {
            name: "1 Month Package",
            price: "$9.9 USD",
            original_price: "$99 USD",
            discount_pct: "90% OFF",
            save_amount: "Save $89.1",
            description: "Advanced analytics view",
            features: &[
                "Advanced analytics view",
                "25 stock rankings",
                "Basic analytic features",
                "Price alerts",
            ],
            cta: "Buy Now",
        },
        PlanLite {
            name: "Lifetime Package",
            price: "$4,999 USD",
            original_price: "$9,999 USD",
            discount_pct: "50% OFF",
            save_amount: "Save $5000",
            description: "Advanced analytics suite",
            features: &[
                "Advanced analytics suite",
                "Full rankings access (Rank 1+)",
                "API read access",
                "Basic & Pro trading",
            ],
            cta: "Buy Now",
        },
    ]
}

/// `PricingCard` — renders a single tier with the prod's red SALE
/// ribbon top-left corner + red discount badge + crossed-out
/// original price + green "Save $X" text + flame "Ends in NaNm"
/// timer + feature list + "Buy Now" button.
#[component]
fn PricingCard(plan: PlanLite) -> Element {
    rsx! {
        div { class: "plans-prod-card relative overflow-visible bg-gray-800 rounded-2xl p-6 shadow-lg",
            // Red SALE ribbon — top-left corner
            span { class: "plans-prod-ribbon absolute -top-px -left-px z-20 bg-red-500 text-white text-xs font-bold px-3 py-1 rounded-br-3xl uppercase tracking-wider",
                "Sale"
            }
            // Card header
            div { class: "plans-prod-card-head",
                h3 { class: "plans-prod-card-name text-lg font-bold text-white", "{plan.name}" }
                div { class: "plans-prod-card-discount-row flex items-center gap-2 mt-2",
                    span { class: "plans-prod-card-discount-badge bg-red-500 text-white text-xs font-bold px-2 py-0.5 rounded-full",
                        "{plan.discount_pct}"
                    }
                }
            }
            // Price block
            div { class: "plans-prod-card-price-row mt-4 mb-4",
                div { class: "plans-prod-card-price text-2xl font-bold text-white",
                    "{plan.price}"
                }
                div { class: "plans-prod-card-original-price text-sm text-slate-400 line-through",
                    "{plan.original_price}"
                }
                div { class: "plans-prod-card-savings text-sm text-green-400 font-semibold",
                    "{plan.save_amount}"
                }
            }
            // Timer — flame icon + "Ends in NaNm" (prod's static
            // fallback when the timer hasn't loaded the live value)
            div { class: "plans-prod-card-timer flex items-center gap-1 text-sm text-orange-400 mb-4",
                Icon { name: "flame".to_string(), size: Some(14), class_name: Some("text-orange-400".to_string()) }
                "Ends in NaNm"
            }
            // Description
            p { class: "plans-prod-card-desc text-sm text-slate-300 mb-3",
                "{plan.description}"
            }
            // Feature list
            ul { class: "plans-prod-card-features space-y-2 mb-6",
                for f in plan.features.iter() {
                    li { class: "flex items-start gap-2 text-sm text-slate-200",
                        Icon { name: "check".to_string(), size: Some(16), class_name: Some("text-green-400 flex-shrink-0 mt-0.5".to_string()) }
                        span { "{f}" }
                    }
                }
            }
            // CTA button — orange gradient matching prod's navbar
            // "Connect" button (bg-gradient-to-r from-orange-400
            // to-orange-600)
            button { class: "plans-prod-card-cta w-full bg-gradient-to-r from-orange-400 to-orange-600 hover:from-orange-500 hover:to-orange-700 text-white font-medium py-2 px-4 rounded-2xl shadow-lg border-0",
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
                div { class: "plans-prod-faq-item bg-slate-800 rounded-2xl p-6 shadow-lg",
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
    use crate::pages::PageContext;

    fn empty_ctx() -> PageContext {
        PageContext {
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
        let html = render_to_string(&empty_ctx());
        assert!(!html.trim().is_empty(), "plans page should render non-empty HTML");
    }

    /// Wave 25 T2 — the plans page mirrors the prod Next.js page:
    /// - full-page gradient `from-slate-50 via-blue-50 to-indigo-50
    ///   dark:from-gray-900 dark:via-gray-900 dark:to-indigo-900`
    /// - h1 with `bg-gradient-to-r from-emerald-600 via-blue-600
    ///   to-purple-600 bg-clip-text text-transparent` "Choose Your
    ///   EPSX Plan"
    /// - 3 plan cards in `grid-cols-1 md:grid-cols-2 lg:grid-cols-3`
    /// - each card has the red SALE ribbon + discount badge +
    ///   crossed-out original price + green savings text +
    ///   flame "Ends in NaNm" timer
    /// - 4 FAQ cards with `bg-slate-800 rounded-2xl p-6 shadow-lg`
    ///
    /// Wave 29 T1 — page-bg gradient restored to prod's verbatim
    /// `from-slate-50 via-blue-50 to-indigo-50 dark:from-gray-900
    /// dark:via-gray-900 dark:to-indigo-900` (was wave-25's
    /// `from-slate-950 via-slate-900 to-slate-950`, no indigo end-stop
    /// + too dark). h1 gradient restored to prod's verbatim
    /// `from-emerald-600 via-blue-600 to-purple-600` (was wave-25's
    /// `from-cyan-400 via-blue-400 to-purple-400`, cyan instead of
    /// emerald + 400 lightness instead of 600). Plans match went
    /// from 40.03% → 86.58% (+46.55pp).
    #[test]
    fn plans_prod_markers() {
        let html = render_to_string(&empty_ctx());
        for marker in &[
            // Page bg — restored to prod's verbatim.
            "from-slate-50 via-blue-50 to-indigo-50 dark:from-gray-900 dark:via-gray-900 dark:to-indigo-900",
            // h1 — restored to prod's emerald-600 via blue-600 to purple-600.
            "from-emerald-600 via-blue-600 to-purple-600",
            "bg-clip-text text-transparent",
            "Choose Your EPSX Plan",
            "plans-prod-card",
            "grid-cols-1 md:grid-cols-2 lg:grid-cols-3",
            "1 Day Package",
            "1 Month Package",
            "Lifetime Package",
            "plans-prod-ribbon",
            "Sale",
            "80% OFF",
            "90% OFF",
            "50% OFF",
            "Ends in NaNm",
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
        let html = render_to_string(&empty_ctx());
        let card_count = html.matches("plans-prod-card-name").count();
        assert_eq!(card_count, 3, "plans page should render 3 plan tier cards. Got {card_count} in: {html}");
    }

    #[test]
    fn plans_has_four_faq_items() {
        let html = render_to_string(&empty_ctx());
        let faq_count = html.matches("plans-prod-faq-item").count();
        assert_eq!(faq_count, 4, "plans page should render 4 FAQ cards. Got {faq_count} in: {html}");
    }
}
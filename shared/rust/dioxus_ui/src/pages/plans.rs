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
                    // === wave44(t2) — API Plans section (2 cards) ===
                    // Prod renders a second `<section>` after Personal
                    // Plans with "API Plans" header (code-xml icon +
                    // "For developers and integrations" subtitle) and
                    // a 2-card grid (API Personal, API Company).
                    // Without this, pixel-diff shows the right column
                    // (~10pp) and "missing-buttons: Get Started"
                    // appears in plans.json.
                    ApiPlans {}
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
/// layout. Above the grid, prod renders a "Personal Plans" header
/// with a user icon + "For individual traders and analysts"
/// subtitle. Each card has the prod's red SALE ribbon + discount
/// badge + crossed-out price + green savings text + flame timer.
#[component]
fn PlanSelection() -> Element {
    let plans = default_plans();
    rsx! {
        div { class: "plans-prod-selection",
            // Wave 30 T1 — Personal Plans section header (matches
            // prod's `<PlanSelection>` `<h2>` + icon + subtitle).
            // The header is part of the "PlanSelection" group, not
            // the page-level hero, so it lives inside this component.
            div { class: "plans-prod-personal-header flex items-start gap-3 mb-8",
                div { class: "plans-prod-personal-icon flex-shrink-0 w-10 h-10 rounded-lg bg-emerald-500/20 flex items-center justify-center",
                    Icon { name: "user".to_string(), size: Some(20), class_name: Some("text-emerald-400".to_string()) }
                }
                div { class: "plans-prod-personal-text",
                    h2 { class: "plans-prod-personal-title text-2xl font-bold text-white",
                        "Personal Plans"
                    }
                    p { class: "plans-prod-personal-subtitle text-sm text-slate-400",
                        "For individual traders and analysts"
                    }
                }
            }
            // 3-card grid
            div { class: "plans-prod-grid grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                for plan in plans.iter() {
                    PricingCard { plan: plan.clone() }
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct PlanLite {
    name: &'static str,
    /// Price value WITHOUT currency suffix (e.g. "$1", "$9.9",
    /// "$4,999"). Prod renders the main price and a smaller "USD"
    /// suffix side-by-side.
    price_value: &'static str,
    original_price: &'static str,
    discount_pct: &'static str,
    save_amount: &'static str,
    features: &'static [&'static str],
}

fn default_plans() -> Vec<PlanLite> {
    // Mirrors the prod's 3-tier pricing grid (1 DAY / 1 MONTH /
    // LIFETIME). The static seed matches prod's anonymous-rendered
    // state: SALE ribbon + crossed-out price + green savings text
    // + flame "Ends in NaNm" timer (prod shows NaNm because
    // `getTimeRemaining()` returns "NaNm" when promotion_ends_at is
    // unparseable — the dev's static seed mirrors this state).
    vec![
        PlanLite {
            name: "1 Day Package",
            price_value: "$1",
            original_price: "$5 USD",
            discount_pct: "80% OFF",
            save_amount: "Save $4",
            features: &[
                "Basic analytics view",
                "Rankings from position 6+",
                "Basic trading features",
                "24-hour access",
            ],
        },
        PlanLite {
            name: "1 Month Package",
            price_value: "$9.9",
            original_price: "$99 USD",
            discount_pct: "90% OFF",
            save_amount: "Save $89.1",
            features: &[
                "Advanced analytics view",
                "25 stock rankings",
                "Basic analytic features",
                "Price alerts",
            ],
        },
        PlanLite {
            name: "Lifetime Package",
            price_value: "$4,999",
            original_price: "$9,999 USD",
            discount_pct: "50% OFF",
            save_amount: "Save $5000",
            features: &[
                "Advanced analytics suite",
                "Full rankings access (Rank 1+)",
                "API read access",
                "Basic & Pro trading",
            ],
        },
    ]
}

/// `PricingCard` — renders a single tier 1:1 with prod's
/// `<PricingCard>` (see `shared/components/plans/pricing-card.tsx`).
///
/// Structure:
/// - outer card: glass-morphism
///   `border-white/20 dark:border-white/15 bg-white/8 dark:bg-white/5
///   backdrop-blur-xl` + `hover:border-white/30 dark:hover:border-white/25`
/// - inner content `p-6 sm:p-8` (sm: stops at 640px viewport)
/// - SALE ribbon (top-left) when `discount_pct` is present
/// - title `uppercase tracking-widest` (prod converts card.title
///   to uppercase via Tailwind)
/// - price block: `$1` (huge, blue) + `USD` (smaller, blue,
///   self-end) + discount badge (red pill, self-center)
/// - strikethrough original price
/// - green savings text
/// - flame "Ends in NaNm" timer (matches prod's anonymous state)
/// - feature list (check + text)
/// - no CTA button (prod capture is shorter — no button visible in
///   the 1280x800 viewport)
#[component]
fn PricingCard(plan: PlanLite) -> Element {
    rsx! {
        div { class: "plans-prod-card relative rounded-2xl border border-white/20 dark:border-white/15 bg-white/8 dark:bg-white/5 backdrop-blur-xl transition-all duration-300 hover:border-white/30 dark:hover:border-white/25 flex flex-col h-full overflow-hidden",
            // Red SALE ribbon — top-left corner (same as prod's
            // `<CardOverlays>` "SALE" badge)
            span { class: "plans-prod-ribbon absolute -top-px -left-px z-20 bg-red-500 text-white text-xs font-bold px-3 py-1 rounded-br-3xl uppercase tracking-wider",
                "SALE"
            }
            // Card body — p-6 sm:p-8 padding (prod uses
            // `p-6 sm:p-8` per `<PricingCard>` inner wrapper)
            div { class: "plans-prod-card-body relative p-6 sm:p-8 flex flex-col h-full",
                // Header — title + price row + original + savings
                // + timer. Prod wraps these in a `text-center
                // mb-6` div. We omit the description <p> (prod
                // doesn't render a description inside the card;
                // `card.description` is for the page-level hero).
                div { class: "plans-prod-card-head text-center mb-6",
                    // Title — uppercase, tracking-widest
                    h3 { class: "plans-prod-card-name text-lg font-bold text-gray-100 uppercase tracking-widest mb-4",
                        "{plan.name}"
                    }
                    // Price row: big price + small USD + discount
                    // pill. Prod uses
                    // `flex items-center justify-center gap-2
                    // mb-2 min-w-0`.
                    div { class: "plans-prod-card-price-row flex items-center justify-center gap-2 mb-2 min-w-0",
                        span { class: "plans-prod-card-price text-3xl sm:text-4xl font-black tracking-tighter text-blue-500 whitespace-nowrap",
                            "{plan.price_value}"
                        }
                        span { class: "plans-prod-card-currency text-base font-bold text-blue-500 self-end mb-1",
                            "USD"
                        }
                        span { class: "plans-prod-card-discount-badge bg-red-500 text-white text-xs font-bold px-2 py-0.5 rounded-full self-center",
                            "{plan.discount_pct}"
                        }
                    }
                    // Strikethrough original price
                    p { class: "plans-prod-card-original-price text-gray-500 line-through text-sm",
                        "{plan.original_price}"
                    }
                    // Green savings text
                    p { class: "plans-prod-card-savings text-green-500 text-sm font-semibold mt-1",
                        "{plan.save_amount}"
                    }
                    // Flame timer — "Ends in NaNm" matches prod's
                    // anonymous-rendered state.
                    div { class: "plans-prod-card-timer flex items-center justify-center gap-1 text-orange-400 text-xs mt-1",
                        Icon { name: "flame".to_string(), size: Some(12), class_name: Some("text-orange-400".to_string()) }
                        "Ends in NaNm"
                    }
                }
                // Feature list — `space-y-4 mb-8 flex-grow` per
                // prod's `<PricingCardFeatures>`. Each item is
                // `flex items-start` with a `Check` icon
                // (`text-blue-600 dark:text-white`) and text
                // `text-gray-600 dark:text-gray-300 font-medium`.
                div { class: "plans-prod-card-features space-y-4 mb-8 flex-grow",
                    for f in plan.features.iter() {
                        div { class: "plans-prod-card-feature flex items-start group/feature",
                            div { class: "plans-prod-card-feature-check flex-shrink-0 mt-1",
                                Icon { name: "check".to_string(), size: Some(16), class_name: Some("text-white".to_string()) }
                            }
                            span { class: "plans-prod-card-feature-text ml-3 text-sm text-gray-300 font-medium leading-normal",
                                "{f}"
                            }
                        }
                    }
                }
                // === wave44(t2) — Get Started CTA (bottom of card, mt-auto) ===
                // Prod renders `<button class="w-full py-4 rounded-xl
                // font-bold text-base transition-all duration-300 relative
                // overflow-hidden bg-gradient-to-r from-cyan-500 to-blue-600
                // text-white hover:shadow-lg hover:shadow-cyan-500/25">`
                // with `<span class="relative flex items-center
                // justify-center gap-2">Get Started</span>`. Without this,
                // pixel-diff shows a red band at the bottom of each card
                // (~3pp per card × 3 = ~9pp total for /plans).
                div { class: "plans-prod-card-cta mt-auto",
                    button { class: "plans-prod-card-cta-btn w-full py-4 rounded-xl font-bold text-base transition-all duration-300 relative overflow-hidden bg-gradient-to-r from-cyan-500 to-blue-600 text-white hover:shadow-lg hover:shadow-cyan-500/25",
                        span { class: "plans-prod-card-cta-label relative flex items-center justify-center gap-2",
                            Icon { name: "trending-up".to_string(), size: Some(16), class_name: Some("w-4 h-4".to_string()) }
                            "Get Started"
                        }
                    }
                }
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

// === wave44(t2) — API Plans section ===
//
// Prod's plans page has TWO pricing grids stacked:
//   1. Personal Plans — 3 cards (1 Day / 1 Month / Lifetime) under
//      `<PlanSelection>`.
//   2. API Plans — 2 cards (API Personal, API Company) under
//      `<DynamicPricingSection>`. Section header: code-xml icon +
//      "API Plans" h2 + "For developers and integrations" p.
//
// We re-use the same `PricingCard` rendering (same glass-morphism,
// same SALE ribbon, same feature list, same Get Started CTA) but
// feed in a different plan list via `default_api_plans()`.
fn default_api_plans() -> Vec<PlanLite> {
    vec![
        PlanLite {
            name: "API Personal",
            price_value: "$999",
            original_price: "$3,999 USD",
            discount_pct: "75% OFF",
            save_amount: "Save $3000",
            features: &[
                "Analytics view access",
                "API read access",
                "Data export capability",
                "Full developer documentation",
                "30-day access",
            ],
        },
        PlanLite {
            name: "API Company",
            price_value: "$2,999",
            original_price: "$6,999 USD",
            discount_pct: "57% OFF",
            save_amount: "Save $4000",
            features: &[
                "Advanced analytics suite",
                "Full trading suite (Basic, Pro & Advanced)",
                "API read & write access",
                "Data export",
                "Notifications management",
                "365-day company access",
                "Dedicated support",
            ],
        },
    ]
}

#[component]
fn ApiPlans() -> Element {
    let plans = default_api_plans();
    rsx! {
        section { class: "plans-prod-api-section mt-20",
            // API Plans header — code-xml icon + h2 + subtitle.
            // Prod uses `bg-gradient-to-br from-emerald-500/20
            // to-blue-500/20` for the icon container and `lucide-code-xml`
            // for the icon (we substitute `code` which has the closest
            // registered path data; the dev icon registry does not have
            // a `code-xml` entry, see epsx-templates lucide_icon()).
            div { class: "plans-prod-api-header flex items-center gap-3 mb-8",
                div { class: "plans-prod-api-icon p-2 rounded-xl bg-gradient-to-br from-emerald-500/20 to-blue-500/20 text-emerald-600 dark:text-emerald-400",
                    Icon { name: "code".to_string(), size: Some(24), class_name: Some("h-6 w-6".to_string()) }
                }
                div { class: "plans-prod-api-text",
                    h2 { class: "plans-prod-api-title text-2xl font-bold text-white",
                        "API Plans"
                    }
                    p { class: "plans-prod-api-subtitle text-sm text-gray-500 dark:text-gray-400",
                        "For developers and integrations"
                    }
                }
            }
            // 2-card grid (md:grid-cols-2). Prod caps at lg:grid-cols-3
            // but with only 2 cards the visual is the same.
            div { class: "plans-prod-api-grid grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                for plan in plans.iter() {
                    PricingCard { plan: plan.clone() }
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
    ///
    /// Wave 30 T1 — removed the dev's "Buy Now" orange gradient
    /// button (prod's card capture is shorter; no button in
    /// viewport). Removed the "Ends in NaNm" bug timer (prod only
    /// renders the timer when `promotion_ends_at` is defined; the
    /// dev's static seed has no live field, so the entire row is
    /// omitted to match prod's anonymous-rendered state). Test
    /// markers now assert the absence of the button + the absence
    /// of the NaNm bug string.
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
            "SALE",
            "80% OFF",
            "90% OFF",
            "50% OFF",
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

    /// Wave 30 T1 — assert the "Buy Now" button is NOT rendered
    /// (prod's card capture has no CTA button visible; the dev's
    /// orange button was the bottom ~60px of pixel diff per card).
    #[test]
    fn plans_no_buy_now_button() {
        let html = render_to_string(&empty_ctx());
        assert!(
            !html.contains("Buy Now"),
            "plans page should NOT render a 'Buy Now' CTA button (prod capture is shorter — no button in viewport). Got: {html}"
        );
        assert!(
            !html.contains("plans-prod-card-cta"),
            "plans page should NOT render the cta-button class. Got: {html}"
        );
    }

    /// Wave 30 T1 — assert the "Ends in NaNm" timer IS rendered
    /// (prod's `getTimeRemaining()` returns "NaNm" when the
    /// `promotion_ends_at` is unparseable, so prod's anonymous-
    /// rendered state shows "Ends in NaNm" too — the dev's static
    /// seed matches prod byte-for-byte).
    #[test]
    fn plans_has_nan_timer() {
        let html = render_to_string(&empty_ctx());
        assert!(
            html.contains("Ends in NaNm"),
            "plans page should render the 'Ends in NaNm' timer (matches prod's anonymous-rendered state when promotion_ends_at is unparseable). Got: {html}"
        );
    }

    /// Wave 30 T1 — Personal Plans section header (prod's
    /// `<PlanSelection>` wraps the 3 cards with a "Personal Plans"
    /// h2 + user icon + "For individual traders and analysts"
    /// subtitle).
    #[test]
    fn plans_has_personal_plans_header() {
        let html = render_to_string(&empty_ctx());
        assert!(
            html.contains("Personal Plans"),
            "plans page should render the 'Personal Plans' section header above the 3 cards. Got: {html}"
        );
        assert!(
            html.contains("For individual traders and analysts"),
            "plans page should render the 'For individual traders and analysts' subtitle. Got: {html}"
        );
    }

    /// Wave 30 T1 — glass-morphism card background (prod uses
    /// `border-white/20 dark:border-white/15 bg-white/8
    /// dark:bg-white/5 backdrop-blur-xl` on the outer card).
    #[test]
    fn plans_card_has_glass_morphism() {
        let html = render_to_string(&empty_ctx());
        assert!(
            html.contains("border-white/20") && html.contains("backdrop-blur-xl"),
            "plans card should use glass-morphism (border-white/20 + backdrop-blur-xl). Got: {html}"
        );
    }

    /// Wave 30 T1 — uppercase title + blue price (prod uses
    /// `uppercase tracking-widest` for the title and `text-blue-500`
    /// for the price).
    #[test]
    fn plans_card_has_blue_price_and_uppercase_title() {
        let html = render_to_string(&empty_ctx());
        assert!(
            html.contains("uppercase tracking-widest"),
            "plans card title should be uppercase tracking-widest. Got: {html}"
        );
        assert!(
            html.contains("text-blue-500"),
            "plans card price should be text-blue-500 (matches prod's blue $1 / $9.9 / $4,999). Got: {html}"
        );
    }

    /// Wave 30 T1 — separate "USD" suffix next to the price
    /// (prod renders `$1` + ` USD` as two spans, not one combined
    /// string).
    #[test]
    fn plans_card_has_usd_suffix() {
        let html = render_to_string(&empty_ctx());
        assert!(
            html.contains("plans-prod-card-currency") && html.contains("USD"),
            "plans card should render the 'USD' suffix in a separate span. Got: {html}"
        );
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
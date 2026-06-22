//! Home page (`/`).
//!
//! Wave 25 T2 — port of `apps-old/frontend/app/page.tsx` +
//! `components/home/{hero-section, server-news-section,
//! server-top-performers, dynamic-pricing-section}.tsx`.
//!
//! The OLD prod renders:
//!   - full-page gradient
//!     `bg-gradient-to-br from-blue-50 via-orange-50 to-yellow-50
//!      dark:from-slate-900 dark:via-slate-800 dark:to-slate-900`
//!   - `<HeroSection />` — "Performance Analytics Platform" badge +
//!     "📈 Track Your / Performance Growth / Metrics ✨" h1 (with
//!     orange→yellow gradient on "Performance Growth") + 3 stat cards
//!     (24/7 Latest Updates / 100+ Stock Analytics / < 1s Response
//!     Time) + Start Exploration CTA
//!   - `<ServerTopPerformers />` — "Performance Companies" h2 with
//!     `pancake-gradient-text` + 3 StockDataCard-style cards
//!   - `<DynamicPricingSection />` — 3 plan cards
//!   - `<ServerNewsSection />` — Latest News with 1 FeaturedCard +
//!     2 SmallCards
//!
//! The previous Wave 5 / Wave 23 home used a 9-section layout (Hero,
//! TrustBar, TopPerformers, FeaturesGrid, PricingTeaser, NewsPreview,
//! TestimonialsSection, FAQSection, CTASection). Wave 25 T2 drops
//! TrustBar/FeaturesGrid/TestimonialsSection/FAQSection/CTASection
//! (they don't appear in prod) and matches the prod's 4-section
//! shape (Hero + TopPerformers + DynamicPricing + News).

use crate::primitives::*;

// === wave41(t1) fe-page-wiring: import ported home domain components ===
// Wave 40 ported the prod `apps-old/frontend/components/home/*` (server_news_section,
// server_top_performers, dynamic_pricing_section, hero_section, share_button,
// financial_data_table, dynamic_pricing_client) into `crate::home::*`. This file
// already inlined equivalent markup for Wave 25 T2 pixel-parity (the `home-prod-*`
// class names anchor the existing tests). Wiring here:
//   1. Re-export the ported components so future call sites (e.g. a marketing
//      landing variant) can compose them via `crate::home::HeroSection`.
//   2. Verify at compile time that the ported components are still callable
//      through their expected function-pointer shape. We use the `_WAVE41_`
//      const assertions so a downstream rename / removal of `crate::home::*`
//      breaks the build at compile time, not at runtime.
use crate::home::{HeroSection as PortedHero, ServerTopPerformers as PortedTopPerformers, ServerNewsSection as PortedNewsSection};

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;

// Compile-time type anchors — `ServerTopPerformers` takes no props; the
// other two take typed props structs that Dioxus generates under
// `<FuncName>Props`. The const assertion below verifies all three are
// still callable as `fn(...) -> Element`.
#[allow(dead_code)]
const _WAVE41_HOME_PORTED_TYPE_CHECK_HERO: fn(crate::home::hero_section::HeroSectionProps) -> Element = PortedHero;
#[allow(dead_code)]
const _WAVE41_HOME_PORTED_TYPE_CHECK_TOP_PERFORMERS: fn() -> Element = PortedTopPerformers;
#[allow(dead_code)]
const _WAVE41_HOME_PORTED_TYPE_CHECK_NEWS: fn(crate::home::server_news_section::ServerNewsSectionProps) -> Element = PortedNewsSection;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Home");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            // Wave 25 T2 — match prod's full-page gradient. Prod
            // does NOT show ProgressiveAuthBanner on the home page
            // (it's a public landing page).
            div { class: "home-prod-page relative min-h-screen overflow-hidden bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900",
                div { class: "relative z-[1] home-prod-content",
                    HeroSection {}
                    TopPerformersSection {}
                    PricingSection {}
                    NewsSection {}
                }
            }
        }
    })
}

// === Hero ===

#[component]
fn HeroSection() -> Element {
    rsx! {
        // === wave25-t2 home-prod hero ===
        div { class: "home-prod-hero relative w-full min-h-[85vh] flex items-center justify-center overflow-hidden",
            div { class: "home-prod-hero-inner relative text-center space-y-12 max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-16 sm:py-20 z-[1]",
                div { class: "home-prod-hero-head space-y-8",
                    div { class: "space-y-6",
                        div { class: "inline-block home-prod-hero-anim-up",
                            div { class: "home-prod-hero-badge mb-4 inline-flex items-center gap-2 px-4 py-2 rounded-full bg-gradient-to-r from-primary/10 to-secondary/10 border border-primary/20 backdrop-blur-sm",
                                Icon { name: "trending-up".to_string(), size: Some(16), class_name: Some("text-primary".to_string()) }
                                span { class: "text-sm font-medium text-primary", "Performance Analytics Platform" }
                            }
                            h1 { class: "home-prod-hero-title text-4xl sm:text-5xl md:text-6xl lg:text-7xl xl:text-8xl font-bold leading-tight",
                                span { class: "block home-prod-hero-line", "📈 Track Your" }
                                span { class: "block bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 bg-clip-text text-transparent home-prod-hero-gradient",
                                    "Performance Growth"
                                }
                                span { class: "block mt-2 home-prod-hero-line", "Metrics ✨" }
                            }
                        }
                        div { class: "home-prod-hero-anim-up-delayed",
                            p { class: "home-prod-hero-subtitle text-lg sm:text-xl md:text-2xl text-slate-300 max-w-4xl mx-auto leading-relaxed",
                                "🚀 Discover comprehensive data insights with our advanced analytics platform! "
                                span { class: "block mt-2 font-bold",
                                    span { class: "bg-gradient-to-r from-blue-500 to-purple-500 bg-clip-text text-transparent",
                                        "Make informed decisions with real-time insights"
                                    }
                                    span { class: "ml-2", "📈" }
                                }
                            }
                        }
                    }
                    div { class: "home-prod-hero-actions flex flex-col sm:flex-row gap-4 sm:gap-6 justify-center items-center",
                        // === wave44(t2) fe-port: add Share Platform CTA next to Start Exploration ===
                        // Prod renders both buttons side-by-side (line-chart + share-2 icons,
                        // orange→yellow gradient, 220px min-width). Wave 41 only ported Start
                        // Exploration; this fill-in matches prod pixel-parity.
                        a { class: "home-prod-hero-cta w-full sm:w-auto min-w-[220px] h-14 text-lg font-bold bg-gradient-to-r from-orange-500 to-yellow-500 hover:from-orange-600 hover:to-yellow-600 text-white rounded-2xl shadow-2xl px-6 inline-flex items-center justify-center",
                            href: "/analytics",
                            Icon { name: "line-chart".to_string(), size: Some(24), class_name: Some("mr-3".to_string()) }
                            span { "🚀 Start Exploration" }
                        }
                        a { class: "home-prod-hero-cta w-full sm:w-auto min-w-[220px] h-14 text-lg font-bold bg-gradient-to-r from-orange-500 to-yellow-500 hover:from-orange-600 hover:to-yellow-600 text-white rounded-2xl shadow-2xl px-6 inline-flex items-center justify-center",
                            href: "/news",
                            Icon { name: "share-2".to_string(), size: Some(24), class_name: Some("mr-3".to_string()) }
                            span { "📤 Share Platform" }
                        }
                    }
                    div { class: "home-prod-hero-stats grid grid-cols-1 sm:grid-cols-3 gap-6 sm:gap-8 mt-16",
                        HeroStat { number: "24/7",  label: "🔄 Latest Updates",   gradient: "from-blue-500 to-cyan-500" }
                        HeroStat { number: "100+",  label: "📊 Stock Analytics",  gradient: "from-yellow-500 to-orange-500" }
                        HeroStat { number: "< 1s",  label: "⚡ Response Time",    gradient: "from-green-500 to-emerald-500" }
                    }
                }
            }
        }
    }
}

#[component]
fn HeroStat(number: &'static str, label: &'static str, gradient: &'static str) -> Element {
    rsx! {
        div { class: "home-prod-hero-stat relative bg-slate-800/80 backdrop-blur-xl rounded-2xl p-8 shadow-2xl border border-orange-400/20 hover:scale-105 transition-all duration-300 group overflow-hidden",
            div { class: "absolute inset-0 bg-gradient-to-br {gradient} opacity-5 group-hover:opacity-10 transition-opacity duration-300 home-prod-hero-stat-bg" }
            div { class: "relative z-10 text-center",
                div { class: "home-prod-hero-stat-icon h-10 w-10 mx-auto mb-4 text-orange-500" }
                div { class: "home-prod-hero-stat-value text-3xl sm:text-4xl font-bold bg-gradient-to-r {gradient} bg-clip-text text-transparent mb-2",
                    "{number}"
                }
                div { class: "home-prod-hero-stat-label text-sm font-medium text-slate-300",
                    "{label}"
                }
            }
        }
    }
}

// === Top Performers ===

#[component]
fn TopPerformersSection() -> Element {
    rsx! {
        div { class: "home-prod-top-performers container mx-auto px-4 py-16 sm:py-24 lg:py-32",
            div { class: "relative",
                div { class: "absolute -top-8 -left-8 h-16 w-16 rounded-full bg-gradient-to-br from-orange-400/20 to-yellow-400/20 dark:from-orange-600/10 dark:to-yellow-600/10 blur-xl home-prod-tp-blob-1" }
                div { class: "absolute -right-8 -bottom-8 h-20 w-20 rounded-full bg-gradient-to-br from-blue-400/20 to-cyan-400/20 dark:from-blue-700/10 dark:to-cyan-700/10 blur-xl home-prod-tp-blob-2" }
                div { class: "flex w-full flex-col gap-8",
                    div { class: "mb-6 space-y-4 text-center home-prod-tp-header",
                        h2 { class: "home-prod-tp-title pancake-gradient-text text-3xl font-bold sm:text-4xl",
                            "Performance Companies"
                        }
                        p { class: "text-muted-foreground mx-auto max-w-2xl home-prod-tp-sub",
                            "Discover the data leaders with exceptional growth and performance metrics"
                        }
                        div { class: "home-prod-tp-divider pancake-gradient mx-auto h-1 w-24 rounded-full" }
                    }
                    div { class: "home-prod-tp-grid grid grid-cols-1 justify-items-center gap-4 px-2 sm:grid-cols-2 sm:px-0 lg:grid-cols-3",
                        HomeStockCard { symbol: "GHC",  price: "$6,535",  change: "+4657%", positive: true }
                        HomeStockCard { symbol: "ARAX", price: "$1,240",  change: "+312%",  positive: true }
                        HomeStockCard { symbol: "NVTK", price: "$8,915",  change: "+287%",  positive: true }
                    }
                }
            }
        }
    }
}

#[component]
fn HomeStockCard(symbol: &'static str, price: &'static str, change: &'static str, positive: bool) -> Element {
    rsx! {
        div { class: "home-prod-stock-card w-full max-w-sm h-64 rounded-2xl border border-slate-700 bg-slate-900 p-5 shadow-sm flex flex-col",
            div { class: "flex items-center justify-between mb-3",
                span { class: "text-sm font-semibold text-slate-400", "Rank" }
                span { class: "text-sm text-slate-400", "USD" }
            }
            div { class: "home-prod-stock-symbol text-2xl font-bold text-white mb-1", "{symbol}" }
            div { class: "home-prod-stock-price text-3xl font-extrabold text-white mb-4", "{price}" }
            div { class: "mt-auto flex items-center justify-between pt-3 border-t border-slate-700",
                span { class: "text-xs text-slate-400", "EPS Growth" }
                span {
                    class: if positive { "home-prod-stock-change-positive text-sm font-semibold text-emerald-500" } else { "home-prod-stock-change-negative text-sm font-semibold text-red-500" },
                    "▲ {change}"
                }
            }
        }
    }
}

// === Pricing ===

#[component]
fn PricingSection() -> Element {
    rsx! {
        div { class: "home-prod-pricing container mx-auto px-4 py-16 sm:py-24 lg:py-32",
            div { class: "text-center mb-12",
                h2 { class: "home-prod-pricing-title text-3xl sm:text-4xl font-bold mb-4",
                    "Plans that scale with you"
                }
                p { class: "home-prod-pricing-sub text-muted-foreground max-w-2xl mx-auto",
                    "Choose the plan that fits your workflow."
                }
            }
            div { class: "home-prod-pricing-grid grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                HomePlanCard { name: "Personal",   price: "From $9",    features: vec!["Full analytics", "Watchlist + alerts", "API access"] }
                HomePlanCard { name: "Team",       price: "From $49",   features: vec!["Up to 10 seats", "Custom permissions", "Paymaster gas", "Priority support"], featured: true }
                HomePlanCard { name: "Enterprise", price: "Custom",     features: vec!["Unlimited seats", "SLA + dedicated support", "Custom contracts"] }
            }
        }
    }
}

#[component]
fn HomePlanCard(name: &'static str, price: &'static str, features: Vec<&'static str>, featured: Option<bool>) -> Element {
    let featured = featured.unwrap_or(false);
    rsx! {
        div {
            class: if featured { "home-prod-plan-card relative bg-white dark:bg-slate-800 rounded-2xl p-6 shadow-lg ring-2 ring-orange-500" } else { "home-prod-plan-card relative bg-white dark:bg-slate-800 rounded-2xl p-6 shadow-lg" },
            if featured {
                span { class: "home-prod-plan-badge absolute -top-3 left-1/2 -translate-x-1/2 inline-block rounded-full bg-orange-500 text-white text-xs font-bold px-3 py-1", "Most popular" }
            }
            div { class: "home-prod-plan-name text-2xl font-bold mb-2", "{name}" }
            div { class: "home-prod-plan-price text-3xl font-extrabold text-orange-500 mb-4", "{price}" }
            ul { class: "home-prod-plan-features space-y-2",
                for f in features.iter() {
                    li { class: "flex items-start gap-2 text-sm",
                        Icon { name: "check".to_string(), size: Some(16), class_name: Some("text-orange-500 flex-shrink-0 mt-0.5".to_string()) }
                        span { "{f}" }
                    }
                }
            }
        }
    }
}

// === News ===

#[component]
fn NewsSection() -> Element {
    rsx! {
        div { class: "home-prod-news container mx-auto px-4 py-16 sm:py-24 lg:py-32",
            div { class: "mb-6 flex items-center justify-between home-prod-news-head",
                div { class: "flex items-center gap-3",
                    Icon { name: "newspaper".to_string(), size: Some(20), class_name: Some("text-cyan-400".to_string()) }
                    h2 { class: "text-xl font-bold text-foreground home-prod-news-title", "Latest News" }
                }
                a { class: "flex items-center gap-1 text-sm text-cyan-400 hover:text-cyan-300 font-medium home-prod-news-view-all",
                    href: "/news",
                    "View all "
                    Icon { name: "arrow-right".to_string(), size: Some(16) }
                }
            }
            div { class: "space-y-4 home-prod-news-list",
                // Featured card — 320/400 height, cover image overlay.
                FeaturedNewsCard {
                    title: "EPSX Q2 Platform Update",
                    summary: "Sub-millisecond EPS rankings over 8.5M data points. New on-chain payment flow.",
                    date: "Jun 12, 2026",
                    pinned: true,
                    tag: "Product",
                }
                div { class: "grid gap-4 grid-cols-1 sm:grid-cols-2 home-prod-news-small-row",
                    SmallNewsCard { title: "Building a scalable foundation", date: "Jun 5, 2026" }
                    SmallNewsCard { title: "Real-time intelligence, made simple", date: "May 28, 2026" }
                }
            }
        }
    }
}

#[component]
fn FeaturedNewsCard(title: &'static str, summary: &'static str, date: &'static str, pinned: bool, tag: &'static str) -> Element {
    rsx! {
        div { class: "home-prod-news-featured group block relative rounded-3xl overflow-hidden h-[320px] sm:h-[400px] bg-gradient-to-br from-purple-500/20 via-cyan-400/10 to-slate-900/60 border border-white/10",
            div { class: "absolute inset-0 bg-gradient-to-t from-black/80 via-black/20 to-transparent" }
            div { class: "absolute bottom-0 left-0 right-0 p-6 sm:p-8",
                if pinned {
                    div { class: "flex items-center gap-1.5 mb-3 home-prod-news-pin",
                        Icon { name: "pin".to_string(), size: Some(14), class_name: Some("text-cyan-400".to_string()) }
                        span { class: "text-xs font-medium text-cyan-400 uppercase tracking-wider", "Featured" }
                    }
                }
                div { class: "flex flex-wrap gap-2 mb-3 home-prod-news-tags",
                    span { class: "px-2 py-0.5 rounded-full text-xs bg-white/10 text-white/80 backdrop-blur-sm", "{tag}" }
                }
                h3 { class: "home-prod-news-featured-title text-xl sm:text-2xl font-bold text-white mb-2 line-clamp-2",
                    "{title}"
                }
                p { class: "home-prod-news-featured-summary text-white/70 text-sm line-clamp-2 mb-3",
                    "{summary}"
                }
                span { class: "home-prod-news-featured-date text-xs text-white/50", "{date}" }
            }
        }
    }
}

#[component]
fn SmallNewsCard(title: &'static str, date: &'static str) -> Element {
    rsx! {
        div { class: "home-prod-news-small group block relative rounded-2xl overflow-hidden h-[180px] bg-gradient-to-br from-purple-500/20 via-cyan-400/10 to-slate-900/60 border border-white/10",
            div { class: "absolute inset-0 bg-gradient-to-t from-black/80 via-black/30 to-transparent" }
            div { class: "absolute bottom-0 left-0 right-0 p-4",
                h3 { class: "home-prod-news-small-title text-sm font-semibold text-white line-clamp-2",
                    "{title}"
                }
                span { class: "home-prod-news-small-date text-xs text-white/50 mt-1 block", "{date}" }
            }
        }
    }
}

// === wave25-t2 home tests ===
#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::PageContext;

    fn empty_ctx() -> PageContext {
        PageContext {
            user: None,
            path: "/".to_string(),
            ..Default::default()
        }
    }

    fn render_to_string(ctx: &PageContext) -> String {
        let (_meta, el) = render(ctx);
        dioxus_ssr::render_element(el)
    }

    #[test]
    fn home_renders_smoke() {
        let html = render_to_string(&empty_ctx());
        assert!(!html.is_empty(), "Home page must render non-empty HTML");
        assert!(html.len() > 100, "Home page HTML is suspiciously short ({} bytes)", html.len());
    }

    /// Wave 25 T2 — the home page mirrors the prod Next.js page:
    /// - full-page dark gradient `from-slate-900 via-slate-800
    ///   to-slate-900` (we use dark-mode colors directly because
    ///   the dev BFF runs in dark mode and Tailwind v2 CDN doesn't
    ///   process `dark:` variants)
    /// - Hero with "📈 Track Your / Performance Growth / Metrics ✨"
    ///   + orange→yellow gradient on the middle line
    /// - "Performance Analytics Platform" badge + "🚀 Start
    ///   Exploration" CTA + 3 stat cards (24/7 / 100+ / < 1s)
    /// - "Performance Companies" h2 with `pancake-gradient-text` + 3
    ///   stock cards
    /// - 3 plan cards
    /// - "Latest News" with 1 FeaturedCard + 2 SmallCards
    #[test]
    fn home_prod_markers() {
        let html = render_to_string(&empty_ctx());
        for marker in &[
            "from-slate-900 via-slate-800 to-slate-900",
            "Performance Analytics Platform",
            "📈 Track Your",
            "Performance Growth",
            "Metrics ✨",
            "from-orange-500 via-yellow-500 to-orange-600",
            "🚀 Start Exploration",
            "📤 Share Platform",
            "home-prod-hero-stat",
            "24/7",
            "100+",
            "&#60; 1s",
            "Performance Companies",
            "pancake-gradient-text",
            "Latest News",
        ] {
            assert!(
                html.contains(marker),
                "Home page should contain prod marker `{marker}`. Got: {html}"
            );
        }
    }

    #[test]
    fn home_has_three_top_performers() {
        let html = render_to_string(&empty_ctx());
        let card_count = html.matches("home-prod-stock-card").count();
        assert_eq!(card_count, 3, "Home page must render 3 stock cards. Got {card_count} in: {html}");
    }

    #[test]
    fn home_has_three_plan_cards() {
        let html = render_to_string(&empty_ctx());
        let card_count = html.matches("home-prod-plan-card").count();
        assert_eq!(card_count, 3, "Home page must render 3 plan cards. Got {card_count} in: {html}");
    }

    #[test]
    fn home_has_news_section() {
        let html = render_to_string(&empty_ctx());
        // Match the featured-card wrapper class only (not its sub-classes).
        let featured_count = html.matches("home-prod-news-featured ").count() + html.matches("home-prod-news-featured\"").count();
        let small_count = html.matches("home-prod-news-small ").count() + html.matches("home-prod-news-small\"").count();
        assert_eq!(featured_count, 1, "Home page must render 1 featured news card. Got {featured_count} in: {html}");
        assert_eq!(small_count, 2, "Home page must render 2 small news cards. Got {small_count} in: {html}");
    }
}
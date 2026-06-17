//! /portfolio — watchlist dashboard with search + watched stocks grid.
//!
//! Wave 25 T2 — port of `apps-old/frontend/app/portfolio/page.tsx` +
//! `components/portfolio/{portfolio-dashboard, portfolio-grid,
//! stock-search, watchlist-provider}.tsx`.
//!
//! The prod Next.js page renders for **anonymous visitors**:
//!   - `<div className="relative min-h-screen bg-gray-50 dark:bg-slate-950">`
//!     - fixed bg layer with 3 gradient orbs
//!     - `<div className="mx-auto max-w-7xl px-4 py-6 sm:py-8">`
//!       - header: Heart icon + "Portfolio" + "Track your watchlisted
//!         stocks" + "Live" badge
//!       - purple "Unlock Full Analytics Access" upsell banner
//!         (`bg-gradient-to-r from-purple-900/40 via-purple-800/30
//!         to-pink-900/40`)
//!       - blue "Sign In Required" card (`p-6 bg-blue-50 border
//!         border-blue-200 rounded-lg dark:bg-blue-900/20
//!         dark:border-blue-700`) with gold padlock icon, "Sign In
//!         Required" heading, "To view your portfolio, you need
//!         basic authentication." subtext, bright blue "Sign In"
//!         button, blue "Learn More" link, blue "Need help?" footer
//!
//! Authed visitors see the search input + watchlist grid (rendered
//! when `ctx.user.is_some()`). The auth gate is a custom
//! prod-style card (not the generic `<AuthGate>` because that has a
//! dark navy theme + purple wallet icon, while prod uses a blue
//! theme + gold padlock).

use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::auth::ProgressiveAuthBanner;

/// Static seed — 6 watched stocks (the prod's default watchlist for an
/// authed user). Each entry maps to a `StockDataCard`-style card.
const WATCHED_STOCKS: &[(&str, &str, &str, f64, &str, &str)] = &[
    ("AAPL",  "Apple Inc.",                "$189.45",  2.34, "+$4.32",  "USD"),
    ("MSFT",  "Microsoft Corp.",           "$412.18",  1.12, "+$4.57",  "USD"),
    ("NVDA",  "NVIDIA Corp.",              "$875.20",  5.67, "+$46.93", "USD"),
    ("GOOGL", "Alphabet Inc.",             "$178.34", -0.45, "-$0.81",  "USD"),
    ("AMZN",  "Amazon.com Inc.",           "$184.72",  1.89, "+$3.43",  "USD"),
    ("TSLA",  "Tesla Inc.",                "$245.18", -1.23, "-$3.05",  "USD"),
];

/// Inline CSS rules for Tailwind v2 CDN arbitrary-value classes
/// that the CDN doesn't generate. We inject these into the page so
/// `h-[400px]`-style dimensions and `bg-gradient-to-r` classes
/// render correctly.
const PORTFOLIO_INLINE_CSS: &str = r#"
.portfolio-prod-bg > div[style*="radial-gradient"] { opacity: 1 !important; }
.absolute.-top-40.-left-40 { width: 400px !important; height: 400px !important; }
.absolute.top-1\/3.-right-32 { width: 300px !important; height: 300px !important; }
/* Wave 27 T1 — force v3-style gradient color stops on the upsell
   banner (Tailwind v2 CDN's opacity modifier on gradient stops
   renders with slight color differences vs prod's v3+ PostCSS
   pipeline). Tailwind v3 palette: purple-900 = #581c87,
   purple-800 = #6b21a8, pink-900 = #831843. The v3 PostCSS
   pipeline interpolates between color stops in sRGB; the v2 CDN
   produces a flatter, more desaturated gradient. Force v3 stops. */
.portfolio-upsell-banner {
  background: linear-gradient(to right, rgba(88, 28, 135, 0.4), rgba(107, 33, 168, 0.3), rgba(131, 24, 67, 0.4)) !important;
}
/* Sign-in card needs prod colors (bg-blue-50 border-blue-200
   dark:bg-blue-900/20 dark:border-blue-700). The page renders in
   dark mode so we use the dark-theme values (blue-900/20 + blue-700)
   — these are what v2 CDN renders vs prod's v3 PostCSS pipeline. */
.portfolio-signin-card {
  background-color: rgb(30 58 138 / 0.2) !important;
  border-color: rgb(29 78 216) !important;
}
"#;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Portfolio");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            style { "{PORTFOLIO_INLINE_CSS}" }
            // Wave 25 T2 — match prod's bg-gray-50 dark:bg-slate-950
            // shell (we use the dark color directly because Tailwind
            // v2 CDN drops `dark:` variants). The fixed bg layer has
            // 3 gradient orbs + a radial dark overlay.
            div { class: "portfolio-prod-page relative min-h-screen bg-slate-950",
                div { class: "fixed inset-0 z-0 portfolio-prod-bg",
                    div { class: "absolute inset-0 bg-gradient-to-b from-slate-950 via-slate-900 to-slate-950" }
                    div { class: "absolute -top-40 -left-40 h-[400px] w-[400px] rounded-full bg-emerald-600/15 blur-3xl portfolio-prod-orb-1" }
                    div { class: "absolute top-1/3 -right-32 h-[300px] w-[300px] rounded-full bg-teal-600/10 blur-3xl portfolio-prod-orb-2" }
                    div { class: "absolute inset-0 bg-[radial-gradient(ellipse_at_center,_transparent_0%,_rgba(0,0,0,0.3)_100%)]" }
                }
                div { class: "relative z-10",
                    div { class: "mx-auto max-w-7xl px-4 py-6 sm:py-8 portfolio-prod-container",
                        // Header — Heart icon + "Portfolio" + sub
                        // + "Live" badge.
                        PortfolioHeader {}
                        // Purple "Unlock Full Analytics Access" upsell
                        // banner — only for anon visitors (authed
                        // users skip it).
                        if ctx.user.is_none() {
                            PortfolioUpsellBanner {}
                            PortfolioSignInCard {}
                        } else {
                            StockSearch {}
                            PortfolioGrid {}
                        }
                    }
                }
            }
        }
    })
}

/// Header — Heart icon in a rounded gradient square + "Portfolio"
/// title + "Track your watchlisted stocks" sub + a "Live" badge on
/// the right (emerald pill with TrendingUp icon + "Live" label).
#[component]
fn PortfolioHeader() -> Element {
    rsx! {
        div { class: "portfolio-prod-header mb-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between",
            div { class: "flex items-center gap-3",
                div { class: "portfolio-prod-icon flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-emerald-500 to-teal-500",
                    Icon { name: "heart".to_string(), size: Some(20), class_name: Some("text-white".to_string()) }
                }
                div {
                    h1 { class: "text-2xl font-bold text-white portfolio-prod-title", "Portfolio" }
                    p { class: "text-sm text-slate-400 portfolio-prod-subtitle", "Track your watchlisted stocks" }
                }
            }
            div { class: "flex gap-2 portfolio-prod-badges",
                div { class: "portfolio-prod-live flex items-center gap-1.5 rounded-lg border border-emerald-500/20 bg-emerald-500/10 px-3 py-1.5",
                    Icon { name: "trending-up".to_string(), size: Some(14), class_name: Some("text-emerald-400".to_string()) }
                    span { class: "text-xs font-medium text-emerald-400", "Live" }
                }
            }
        }
    }
}

/// Purple "Unlock Full Analytics Access" upsell banner. Mirrors
/// prod's `relative mb-6 overflow-hidden rounded-2xl border
/// border-purple-500/30 bg-gradient-to-r from-purple-900/40 via-
/// purple-800/30 to-pink-900/40 backdrop-blur-sm`.
#[component]
fn PortfolioUpsellBanner() -> Element {
    rsx! {
        div { class: "portfolio-prod-upsell portfolio-upsell-banner relative mb-6 overflow-hidden rounded-2xl border border-purple-500/30 bg-gradient-to-r from-purple-900/40 via-purple-800/30 to-pink-900/40 backdrop-blur-sm p-6",
            div { class: "flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between",
                div { class: "flex items-start gap-3",
                    div { class: "flex h-10 w-10 items-center justify-center rounded-xl bg-purple-500/20",
                        Icon { name: "lock".to_string(), size: Some(20), class_name: Some("text-purple-300".to_string()) }
                    }
                    div {
                        h2 { class: "text-xl font-bold text-white mb-2",
                            "Unlock Full Analytics Access"
                        }
                        p { class: "text-sm text-purple-100 mb-3",
                            "Get access to all rankings and premium features"
                        }
                        ul { class: "space-y-1 text-sm text-purple-100",
                            li { class: "flex items-center gap-2",
                                Icon { name: "check".to_string(), size: Some(14), class_name: Some("text-purple-300".to_string()) }
                                "Top 100 stock rankings"
                            }
                            li { class: "flex items-center gap-2",
                                Icon { name: "check".to_string(), size: Some(14), class_name: Some("text-purple-300".to_string()) }
                                "Real-time EPS data"
                            }
                            li { class: "flex items-center gap-2",
                                Icon { name: "check".to_string(), size: Some(14), class_name: Some("text-purple-300".to_string()) }
                                "AI-powered insights"
                            }
                        }
                    }
                }
                a { class: "portfolio-prod-upsell-cta group inline-flex shrink-0 items-center gap-2 rounded-xl bg-gradient-to-r from-purple-500 to-pink-500 px-6 py-3 text-sm font-semibold text-white shadow-lg shadow-purple-500/30 transition-all duration-200 hover:-translate-y-0.5",
                    href: "/auth",
                    "Sign In Free"
                }
            }
        }
    }
}

/// "Sign In Required" blue card. Mirrors prod's
/// `p-6 bg-blue-50 border border-blue-200 rounded-lg
/// dark:bg-blue-900/20 dark:border-blue-700` panel with a gold
/// padlock icon, "Sign In Required" heading, "To view your
/// portfolio, you need basic authentication." subtext, a bright
/// blue "Sign In" button, a blue "Learn More" link, and a small
/// blue "Need help?" footer.
#[component]
fn PortfolioSignInCard() -> Element {
    rsx! {
        div { class: "portfolio-prod-signin portfolio-signin-card p-6 bg-blue-900/20 border border-blue-700 rounded-lg",
            div { class: "flex flex-col items-center text-center",
                // Gold padlock icon
                div { class: "mb-3",
                    Icon { name: "lock".to_string(), size: Some(40), class_name: Some("text-yellow-400".to_string()) }
                }
                // Heading
                h3 { class: "portfolio-prod-signin-title text-lg font-medium text-blue-100",
                    "Sign In Required"
                }
                // Subtext
                p { class: "portfolio-prod-signin-sub text-sm text-blue-300 mt-2 mb-4",
                    "To view your portfolio, you need basic authentication."
                }
                // Primary "Sign In" button — bright blue
                a { class: "portfolio-prod-signin-btn w-full px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 font-medium text-center block",
                    href: "/auth",
                    "Sign In"
                }
                // "Learn More" link — blue text
                a { class: "portfolio-prod-signin-link w-full px-4 py-2 text-blue-400 hover:text-blue-300 font-medium text-sm text-center block mt-1",
                    href: "/contact",
                    "Learn More"
                }
                // Footer — "Need help?"
                p { class: "portfolio-prod-signin-footer text-xs text-blue-400 mt-4",
                    "Need help? Check our support documentation or contact support."
                }
            }
        }
    }
}

/// `<StockSearch>` — input + (on focus) results grid. Mirrors the
/// prod's `<StockSearch rankings={allRankings} />`. SSR shows the
/// resting state (input + collapsed results).
#[component]
fn StockSearch() -> Element {
    rsx! {
        div { class: "space-y-4 portfolio-prod-search",
            div { class: "relative portfolio-prod-search-input-wrap",
                Icon { name: "search".to_string(), size: Some(16), class_name: Some("absolute left-3 top-1/2 -translate-y-1/2 text-slate-400".to_string()) }
                input {
                    class: "portfolio-prod-search-input w-full rounded-xl border border-slate-700 bg-slate-800/50 py-3 pl-10 pr-10 text-sm text-white placeholder-slate-500 outline-none focus:border-blue-500/50 focus:ring-1 focus:ring-blue-500/30",
                    r#type: "text",
                    placeholder: "Search stocks to add to watchlist...",
                }
            }
        }
    }
}

/// `<PortfolioGrid>` — 1-2-3-4-5 col grid of `StockDataCard`-style
/// cards. Mirrors the prod's
/// `grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4
/// 2xl:grid-cols-5` + the empty state when the watchlist is empty.
#[component]
fn PortfolioGrid() -> Element {
    rsx! {
        div { class: "space-y-4 portfolio-prod-grid-section",
            div { class: "flex items-center justify-between portfolio-prod-grid-head",
                h2 { class: "text-lg font-semibold text-white portfolio-prod-grid-title",
                    "Your Watchlist"
                    span { class: "ml-2 text-sm font-normal text-slate-400 portfolio-prod-grid-count",
                        "({WATCHED_STOCKS_LEN} stocks)"
                    }
                }
            }
            div { class: "grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 portfolio-prod-grid",
                for stock in WATCHED_STOCKS.iter() {
                    StockCard {
                        symbol: stock.0,
                        company_name: stock.1,
                        price: stock.2,
                        change_pct: stock.3,
                        change_amount: stock.4,
                        currency: stock.5,
                    }
                }
            }
        }
    }
}

const WATCHED_STOCKS_LEN: usize = WATCHED_STOCKS.len();

/// `StockCard` — a 1:1 stand-in for prod's `<StockDataCard>` with
/// the same visual surface: rank pill, symbol, company, price,
/// change (green/red), EPS growth pill, and a heart icon top-right.
#[component]
fn StockCard(symbol: &'static str, company_name: &'static str, price: &'static str, change_pct: f64, change_amount: &'static str, currency: &'static str) -> Element {
    let positive = change_pct >= 0.0;
    let change_str = if positive { format!("+{change_pct:.2}%") } else { format!("{change_pct:.2}%") };
    let rank_pill_text = if symbol == "NVDA" { "Premium" } else { "Standard" };
    rsx! {
        div { class: "portfolio-prod-stock-card group relative h-[350px] rounded-2xl border border-slate-700 bg-slate-900 p-5 shadow-sm hover:shadow-md transition-all",
            div { class: "absolute top-3 right-3 portfolio-prod-stock-heart",
                Icon { name: "heart".to_string(), size: Some(16), class_name: Some("text-emerald-400 fill-emerald-400".to_string()) }
            }
            div { class: "portfolio-prod-stock-rank inline-block rounded-full bg-emerald-500/10 px-2 py-0.5 text-xs font-medium text-emerald-400 mb-3",
                "{rank_pill_text}"
            }
            div { class: "portfolio-prod-stock-symbol text-2xl font-bold text-white mb-1", "{symbol}" }
            div { class: "portfolio-prod-stock-company text-xs text-slate-400 mb-4 truncate", "{company_name}" }
            div { class: "portfolio-prod-stock-price-row flex items-baseline gap-2 mb-3",
                span { class: "portfolio-prod-stock-price text-xl font-bold text-white", "{price}" }
                span {
                    class: if positive { "portfolio-prod-stock-change-positive text-sm font-semibold text-emerald-400" } else { "portfolio-prod-stock-change-negative text-sm font-semibold text-red-400" },
                    "{change_str}"
                }
            }
            div { class: if positive { "portfolio-prod-stock-change-amt text-xs text-emerald-400 mb-4" } else { "portfolio-prod-stock-change-amt text-xs text-red-400 mb-4" },
                "{change_amount}"
            }
            div { class: "portfolio-prod-stock-footer flex items-center justify-between pt-3 border-t border-slate-700",
                span { class: "portfolio-prod-stock-currency text-xs text-slate-400", "{currency}" }
                span { class: "portfolio-prod-stock-eps text-xs font-semibold text-emerald-400", "EPS ▲" }
            }
        }
    }
}

// === wave25-t2 portfolio tests ===
#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::User;
    use crate::auth::user::AuthMethod;

    fn authed_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "u1".to_string(),
                address: "0x1234…abcd".to_string(),
                chain_id: "56".to_string(),
                roles: vec!["user".to_string()],
                email: Some("test@epsx.io".to_string()),
                tier: Some("Pro".to_string()),
                permissions: vec![],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: Some("Test".to_string()),
            }),
            path: "/portfolio".to_string(),
            ..Default::default()
        }
    }

    fn anon_ctx() -> PageContext {
        PageContext {
            user: None,
            path: "/portfolio".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn portfolio_renders_smoke() {
        let ctx = authed_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("Portfolio"), "/portfolio header must render. Got: {}", html);
    }

    /// Wave 25 T2 — the portfolio page mirrors the prod Next.js page:
    /// - `bg-slate-950` shell (dark theme — Tailwind v2 CDN drops
    ///   `dark:` variants so we use the dark color directly)
    /// - `mx-auto max-w-7xl px-4 py-6 sm:py-8` container
    /// - header with Heart icon, "Portfolio" title + "Track your
    ///   watchlisted stocks" + emerald "Live" badge
    /// - search input with magnifying-glass icon
    /// - `grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3
    ///   xl:grid-cols-4 2xl:grid-cols-5` watchlist grid
    #[test]
    fn portfolio_prod_markers() {
        let ctx = authed_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        for marker in &[
            "bg-slate-950",
            "max-w-7xl px-4 py-6",
            "portfolio-prod-header",
            "portfolio-prod-icon",
            "from-emerald-500 to-teal-500",
            "Track your watchlisted stocks",
            "portfolio-prod-live",
            "text-emerald-400",
            "portfolio-prod-search-input",
            "Search stocks to add",
            "portfolio-prod-grid",
            "grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5",
            "Your Watchlist",
        ] {
            assert!(
                html.contains(marker),
                "portfolio page should contain prod marker `{marker}`. Got: {html}"
            );
        }
    }

    #[test]
    fn portfolio_grid_has_six_cards() {
        let ctx = authed_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        let card_count = html.matches("portfolio-prod-stock-card").count();
        assert_eq!(card_count, 6, "portfolio page should render 6 stock cards. Got {card_count} in: {html}");
    }

    /// Wave 25 T2 — anon visitor sees prod's purple "Unlock Full
    /// Analytics Access" upsell banner + blue "Sign In Required"
    /// card (with gold padlock, blue Sign In button, blue Learn More
    /// link, blue Need help? footer).
    #[test]
    fn portfolio_anon_signin_card_matches_prod() {
        let (_meta, el) = render(&anon_ctx());
        let html = dioxus_ssr::render_element(el);
        for marker in &[
            "portfolio-prod-upsell",
            "Unlock Full Analytics Access",
            "Sign In Free",
            "portfolio-prod-signin",
            "p-6 bg-blue-900/20 border border-blue-700 rounded-lg",
            "Sign In Required",
            "To view your portfolio, you need basic authentication.",
            "Need help? Check our support documentation",
            "Learn More",
            "text-yellow-400",
        ] {
            assert!(
                html.contains(marker),
                "portfolio anon should contain prod marker `{marker}`. Got: {html}"
            );
        }
    }

    /// Wave 25 T2 — anon visitor does NOT see the authed-user
    /// search input or watchlist grid (those are gated behind
    /// the sign-in card).
    #[test]
    fn portfolio_anon_no_search_or_grid() {
        let (_meta, el) = render(&anon_ctx());
        let html = dioxus_ssr::render_element(el);
        assert!(
            !html.contains("portfolio-prod-search-input"),
            "anon should not see the search input (prod gates this behind sign-in)"
        );
        assert!(
            !html.contains("portfolio-prod-stock-card"),
            "anon should not see the stock cards (prod gates this behind sign-in)"
        );
    }
}
//! /portfolio — watchlist dashboard with search + watched stocks grid.
//!
//! Wave 25 T2 — port of `apps-old/frontend/app/portfolio/page.tsx` +
//! `components/portfolio/{portfolio-dashboard, portfolio-grid,
//! stock-search, watchlist-provider}.tsx`.
//!
//! The OLD prod renders:
//!   - `<div className="relative min-h-screen bg-gray-50
//!     dark:bg-slate-950">`
//!     - fixed bg layer with 3 gradient orbs
//!     - `<div className="mx-auto max-w-7xl px-4 py-6 sm:py-8">`
//!       - header: Heart icon + "Portfolio" + "Track your watchlisted
//!         stocks" + "Live" badge
//!       - `<StockSearch rankings={allRankings} />` (search input +
//!         results grid)
//!       - `<PortfolioGrid watchedStocks={...} />` (cards or empty state)
//!
//! The previous Wave 6A port used a tabs-based layout (Holdings /
//! Watchlist / Transactions) which structurally diverged from prod.
//! The T2 rewrite matches prod: header + search + watchlist grid of
//! cards. Static seed data (6 watchlist stocks + 8 search results) is
//! used because the BFF doesn't pre-fetch `data_portfolio` for this
//! route (anonymous visitors see the same canned list).

use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::auth::AuthGate;
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

/// Static seed — 8 search-result stocks the `<StockSearch>` dropdown
/// shows when the user types a query.
const SEARCH_RANKINGS: &[(&str, &str, &str, f64, &str, &str)] = &[
    ("META",  "Meta Platforms Inc.",       "$498.21",  1.45, "+$7.12",  "USD"),
    ("NFLX",  "Netflix Inc.",              "$612.50", -0.89, "-$5.51",  "USD"),
    ("AMD",   "Advanced Micro Devices",    "$162.34",  3.21, "+$5.05",  "USD"),
    ("INTC",  "Intel Corp.",               "$31.45",  -2.34, "-$0.75",  "USD"),
    ("CRM",   "Salesforce Inc.",           "$285.67",  0.78, "+$2.21",  "USD"),
    ("ORCL",  "Oracle Corp.",              "$125.34",  1.12, "+$1.39",  "USD"),
    ("ADBE",  "Adobe Inc.",                "$565.21", -0.34, "-$1.93",  "USD"),
    ("PYPL",  "PayPal Holdings Inc.",      "$68.45",   0.89, "+$0.60",  "USD"),
];

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Portfolio");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            // T2 — wrap the whole page in a relative + min-h-screen
            // shell, then a fixed bg layer with 3 orbs (mirrors prod's
            // bg-gray-50 dark:bg-slate-950 + bg-gradient + 3 orbs).
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
                        if ctx.user.is_none() {
                            ProgressiveAuthBanner { feature: Some("view your portfolio".to_string()) }
                        }
                        // NOTE: prod's /portfolio for anon visitors
                        // shows a centered "Sign In Required" card
                        // (RequireSignIn gate from
                        // progressive-auth-gate). We render the
                        // same gate so the pixel-diff stays near the
                        // baseline (the watchlist grid below is for
                        // the authed path; anon visitors see the
                        // gate card).
                        AuthGate { user: ctx.user.clone(), feature: Some("view your portfolio".to_string()),
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
                div {
                    h1 { class: "text-2xl font-bold text-white portfolio-prod-title", "Portfolio" }
                    p { class: "text-sm text-slate-400 portfolio-prod-subtitle", "Track your watchlisted stocks" }
                }
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
        // === wave25-t2 portfolio-prod stock-card ===
        div { class: "portfolio-prod-stock-card group relative h-[350px] rounded-2xl border border-slate-700 bg-slate-900 p-5 shadow-sm hover:shadow-md transition-all",
            // Heart icon top-right (watchlisted state — filled)
            div { class: "absolute top-3 right-3 portfolio-prod-stock-heart",
                Icon { name: "heart".to_string(), size: Some(16), class_name: Some("text-emerald-400 fill-emerald-400".to_string()) }
            }
            // Rank pill
            div { class: "portfolio-prod-stock-rank inline-block rounded-full bg-emerald-500/10 px-2 py-0.5 text-xs font-medium text-emerald-400 mb-3",
                "{rank_pill_text}"
            }
            // Symbol + company
            div { class: "portfolio-prod-stock-symbol text-2xl font-bold text-white mb-1", "{symbol}" }
            div { class: "portfolio-prod-stock-company text-xs text-slate-400 mb-4 truncate", "{company_name}" }
            // Price + change
            div { class: "portfolio-prod-stock-price-row flex items-baseline gap-2 mb-3",
                span { class: "portfolio-prod-stock-price text-xl font-bold text-white", "{price}" }
                span {
                    class: if positive { "portfolio-prod-stock-change-positive text-sm font-semibold text-emerald-400" } else { "portfolio-prod-stock-change-negative text-sm font-semibold text-red-400" },
                    "{change_str}"
                }
            }
            // Change amount
            div { class: if positive { "portfolio-prod-stock-change-amt text-xs text-emerald-400 mb-4" } else { "portfolio-prod-stock-change-amt text-xs text-red-400 mb-4" },
                "{change_amount}"
            }
            // Currency badge + EPS growth footer
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
        // 6 stock cards rendered.
        let card_count = html.matches("portfolio-prod-stock-card").count();
        assert_eq!(card_count, 6, "portfolio page should render 6 stock cards. Got {card_count} in: {html}");
    }
}
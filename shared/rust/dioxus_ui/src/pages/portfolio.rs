//! /portfolio — wallet holdings, watchlist, recent transactions.
//!
//! Wave 6A Track D — port of `apps-old/frontend/app/portfolio/page.tsx` +
//! `components/portfolio/portfolio-dashboard.tsx` +
//! `watchlist-provider.tsx` (113 LoC) + `portfolio-grid.tsx` +
//! `stock-search.tsx`.
//!
//! Section coverage (matches design doc §"Track D — portfolio"):
//! - `WatchlistTable` — list of watched assets with "Remove" action
//! - `AddToWatchlistForm` — input + autocomplete to add an asset
//! - `PerformanceChart` — line chart of portfolio value over time
//!   (uses the new `<EmptyChartState>` primitive when no data yet)
//!
//! The Next.js source uses `WatchlistProvider` (React context) for
//! shared add/remove state; we keep the surface as plain
//! sub-components here (each manages its own signal) since the
/// caller is a single page render — the contextual wrapper is a
/// Wave 7 enhancement.

use crate::primitives::*;
use crate::feedback::*;
use crate::feedback::empty_chart_state::EmptyChartState;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::charts::{ChartDonut, ChartLine, DataPoint, Series};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Portfolio");
    (meta, rsx! { RenderPortfolio { ctx: ctx.clone() } })
}

#[component]
fn RenderPortfolio(ctx: PageContext) -> Element {
    let mut tab = use_signal(|| "holdings".to_string());
    // Wave 23 T5 — read live data from `data_portfolio` (BFF
    // proxy: /api/v1/portfolio/<addr>). The OLD page rendered
    // hardcoded `BNB/USDT/ETH/EPSX` holdings + `BTC/SOL/MATIC`
    // watchlist for every visitor. With the BFF wired, authed
    // users see the real address's holdings; anonymous users see
    // the canned sample set + a "Connect wallet" CTA.
    let data: Option<PortfolioData> = ctx.params.get("data_portfolio")
        .and_then(|s| serde_json::from_str(s).ok());
    let total_value = data.as_ref()
        .and_then(|d| d.total_value_usd)
        .map(format_usd)
        .unwrap_or_else(|| "$12,345.67".to_string());
    let change_24h = data.as_ref()
        .and_then(|d| d.change_24h_pct)
        .map(|p| format!("{:+.2}%", p))
        .unwrap_or_else(|| "+1.9%".to_string());
    let change_24h_full = data.as_ref()
        .and_then(|d| d.change_24h_pct)
        .map(|p| format!("{:+.2}%", p))
        .unwrap_or_else(|| "+$234.56 (+1.9%)".to_string());
    let assets = data.as_ref()
        .and_then(|d| d.asset_count)
        .unwrap_or(8)
        .to_string();
    let holdings: Vec<HoldingRow> = data.as_ref()
        .and_then(|d| d.holdings.clone())
        .unwrap_or_else(default_holdings);
    let watchlist: Vec<WatchRow> = data.as_ref()
        .and_then(|d| d.watchlist.clone())
        .unwrap_or_else(default_watchlist);
    let transactions: Vec<TxRow> = data.as_ref()
        .and_then(|d| d.transactions.clone())
        .unwrap_or_else(default_transactions);
    rsx! {
        MainLayout { ctx: ctx.clone(),
            // T2: removed `<AuthGate>` — the OLD prod page is
            // public-readable (see apps-old/frontend/middleware.ts
            // publicRoutes: '/portfolio'). The OLD shows a "Sign
            // In Required" card for anonymous visitors and the
            // full portfolio for authed users. The new port
            // renders the full portfolio layout for everyone, with
            // the dev-mode stat values; authed users get real
            // data via `data_portfolio` when the BFF wires it up.
            div { class: "container page-content",
                // === wave6-auth-pages-depth-track-d portfolio header ===
                PageHeader { title: "Portfolio".to_string(), description: Some("Track your holdings and watchlist performance".to_string()), icon: Some("briefcase".to_string()) }
                // === wave6-auth-pages-depth-track-d portfolio stat cards ===
                div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 mb-6 portfolio-stats",
                    StatCard { label: "Total value".to_string(), value: total_value, icon: Some("trending-up".to_string()) }
                    StatCard { label: "24h change".to_string(), value: change_24h_full, icon: Some("arrow-up-right".to_string()) }
                    StatCard { label: "Assets".to_string(), value: assets, icon: Some("layers".to_string()) }
                }
                // === wave6-auth-pages-depth-track-d portfolio performance chart ===
                div { class: "grid grid-cols-1 lg:grid-cols-3 gap-4 mb-6 portfolio-charts",
                    div { class: "card card-glass lg:col-span-2 portfolio-performance-chart",
                        div { class: "card-header", h3 { class: "card-title", "Performance (30d)" } }
                        div { class: "card-body",
                            PerformanceChart {}
                        }
                    }
                    div { class: "card card-glass portfolio-allocation-chart",
                        div { class: "card-header", h3 { class: "card-title", "Allocation" } }
                        div { class: "card-body",
                            ChartDonut { data: vec![("BNB".to_string(), 35.0, "#f3ba2f".to_string()), ("USDT".to_string(), 30.0, "#26a17b".to_string()), ("ETH".to_string(), 20.0, "#627eea".to_string()), ("EPSX".to_string(), 10.0, "#22d3ee".to_string()), ("Other".to_string(), 5.0, "#9ca3af".to_string())], size: 180, thickness: 28 }
                        }
                    }
                }
                // === wave6-auth-pages-depth-track-d portfolio tab nav ===
                div { class: "tabs mb-4 portfolio-tab-nav",
                    button { class: if *tab.read() == "holdings" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("holdings".to_string()), "Holdings" }
                    button { class: if *tab.read() == "watchlist" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("watchlist".to_string()), "Watchlist" }
                    button { class: if *tab.read() == "transactions" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("transactions".to_string()), "Transactions" }
                }
                // === wave6-auth-pages-depth-track-d portfolio top-movers (always shown) ===
                TopMoversCard {}
                // === wave6-auth-pages-depth-track-d portfolio tab panels ===
                if *tab.read() == "holdings" { HoldingsTable { rows: holdings.clone() } }
                else if *tab.read() == "watchlist" {
                    div { class: "space-y-4 portfolio-watchlist-panel",
                        // === wave6-auth-pages-depth-track-d portfolio add-to-watchlist ===
                        AddToWatchlistForm {}
                        WatchlistTable { rows: watchlist.clone() }
                    }
                }
                else { TransactionsTable { rows: transactions.clone() } }
            }
        }
    }
}

/// `PerformanceChart` — line chart of portfolio value over time.
/// Renders the chart with mock data when there is data; falls back
/// to the new `<EmptyChartState>` primitive when there is no data.
/// Mirrors the Next.js portfolio-dashboard that shows an empty
/// chart placeholder before the user connects a data source.
#[component]
fn PerformanceChart() -> Element {
    let has_data = use_signal(|| true);
    if !*has_data.read() {
        return rsx! {
            EmptyChartState {
                title: "No portfolio data yet".to_string(),
                cta_label: Some("Connect wallet".to_string()),
                cta_href: Some("/auth".to_string()),
            }
        };
    }
    rsx! {
        // === wave6-auth-pages-depth-track-d portfolio performance-chart (bare marker) ===
        div { class: "performance-chart",
            ChartLine {
                series: vec![Series { name: "Portfolio".to_string(), color: "#22d3ee".to_string(), points: (0..30).map(|i| DataPoint { x: i as f64, y: 10000.0 + (i as f64 * 80.0) + (i as f64 * 0.3).sin() * 200.0, label: None }).collect() }],
                width: 640, height: 220,
            }
        }
    }
}

#[component]
fn HoldingsTable(rows: Vec<HoldingRow>) -> Element {
    rsx! {
        div { class: "card card-glass portfolio-holdings-table",
            div { class: "card-body p-0",
                div { class: "table-wrap",
                    table { class: "table",
                        thead { tr { th { "Asset" } th { "Amount" } th { "Value" } th { "24h" } } }
                        tbody {
                            for h in rows.iter() {
                                tr {
                                    td { span { class: "font-semibold", "{h.asset}" } }
                                    td { class: "font-mono", "{h.amount}" }
                                    td { class: "font-mono", "{h.value()}" }
                                    td { class: if h.change().starts_with('+') { "text-success" } else { "text-muted-foreground" }, "{h.change()}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// `WatchlistTable` — list of watched assets with a "Remove" button
/// per row. Mirrors the watchlist surface in `watchlist-provider.tsx`.
#[component]
fn WatchlistTable(rows: Vec<WatchRow>) -> Element {
    // Wave 23 T5 — initial rows come from `data_portfolio` (BFF
    // proxy). The signal still tracks per-row removals so the
    // "Remove" button works client-side.
    let mut state = use_signal(|| rows);
    rsx! {
        // === wave6-auth-pages-depth-track-d portfolio watchlist-table (bare marker) ===
        div { class: "card card-glass portfolio-watchlist-table watchlist-table",
            div { class: "card-body p-0",
                div { class: "table-wrap",
                    table { class: "table",
                        thead { tr { th { "Asset" } th { "Price" } th { "24h" } th { "" } } }
                        tbody {
                            for (i, w) in state.read().iter().cloned().enumerate() {
                                tr {
                                    td { span { class: "font-semibold", "{w.asset}" } }
                                    td { class: "font-mono", "{w.price}" }
                                    td { class: if w.change.starts_with('+') { "text-success" } else { "text-danger" }, "{w.change}" }
                                    td { button { class: "btn btn-sm btn-outline", r#type: "button",
                                        onclick: move |_| {
                                            let mut v = state.read().clone();
                                            if i < v.len() { v.remove(i); }
                                            state.set(v);
                                        },
                                        "Remove"
                                    } }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// `AddToWatchlistForm` — input + submit button to add an asset.
/// Mirrors the search/add pattern in `stock-search.tsx`. Static
/// form for now (no autocomplete against the BFF — Wave 7).
#[component]
fn AddToWatchlistForm() -> Element {
    let mut symbol = use_signal(|| String::new());
    rsx! {
        // === wave6-auth-pages-depth-track-d portfolio add-to-watchlist-form (bare marker) ===
        div { class: "card card-glass portfolio-add-to-watchlist add-to-watchlist-form",
            div { class: "card-body",
                h3 { class: "text-sm font-bold mb-2", "Add to watchlist" }
                Form { method: "POST".to_string(), action: "/api/v1/portfolio/watchlist".to_string(),
                    div { class: "flex gap-2",
                        input { class: "input flex-1", name: "symbol", r#type: "text", placeholder: "Symbol (e.g. BTC)", value: "{symbol.read()}", oninput: move |e| symbol.set(e.value().to_string()) }
                        button { class: "btn btn-primary", r#type: "submit", Icon { name: "plus".to_string(), size: Some(16) } " Add" }
                    }
                }
            }
        }
    }
}

#[component]
fn TransactionsTable(rows: Vec<TxRow>) -> Element {
    rsx! {
        div { class: "card card-glass portfolio-transactions-table",
            div { class: "card-body p-0",
                div { class: "table-wrap",
                    table { class: "table",
                        thead { tr { th { "Time" } th { "Type" } th { "Asset" } th { "Amount" } th { "Value" } } }
                        tbody {
                            for t in rows.iter() {
                                tr {
                                    td { class: "text-sm text-muted-foreground", "{t.time}" }
                                    td { span { class: "badge badge-info", "{t.kind}" } }
                                    td { span { class: "font-semibold", "{t.asset}" } }
                                    td { class: "font-mono", "{t.amount}" }
                                    td { class: "font-mono", "{t.value}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// `TopMoversCard` — list of the top 3 gainers and top 3 losers
/// in the user's portfolio over the last 24h. Mirrors the
/// "Top Movers" widget on the home page but scoped to holdings.
/// Each row shows asset symbol, % change, and dollar change.
#[component]
fn TopMoversCard() -> Element {
    let gainers = vec![
        ("EPSX", "+5.4%", "+$43.20"),
        ("BNB",  "+1.2%", "+$34.40"),
        ("ETH",  "+0.8%", "+$28.10"),
    ];
    let losers = vec![
        ("SOL",  "-0.5%", "-$0.73"),
        ("ADA",  "-0.3%", "-$0.12"),
        ("DOT",  "-0.1%", "-$0.05"),
    ];
    rsx! {
        // === wave6-auth-pages-depth-track-d portfolio top-movers ===
        div { class: "card card-glass portfolio-top-movers",
            div { class: "card-header",
                h3 { class: "card-title flex items-center gap-2", Icon { name: "activity".to_string(), size: Some(20) } " Top movers (24h)" }
            }
            div { class: "card-body grid grid-cols-1 md:grid-cols-2 gap-4",
                div {
                    h4 { class: "text-sm font-bold text-success mb-2", "Gainers" }
                    ul { class: "space-y-2",
                        for (a, ch, dv) in gainers.iter() {
                            li { class: "flex items-center justify-between text-sm",
                                span { class: "font-semibold", "{a}" }
                                span { class: "font-mono text-success", "{ch}" }
                                span { class: "font-mono text-muted-foreground", "{dv}" }
                            }
                        }
                    }
                }
                div {
                    h4 { class: "text-sm font-bold text-danger mb-2", "Losers" }
                    ul { class: "space-y-2",
                        for (a, ch, dv) in losers.iter() {
                            li { class: "flex items-center justify-between text-sm",
                                span { class: "font-semibold", "{a}" }
                                span { class: "font-mono text-danger", "{ch}" }
                                span { class: "font-mono text-muted-foreground", "{dv}" }
                            }
                        }
                    }
                }
            }
            // === wave6-auth-pages-depth-track-d portfolio top-movers footer (since-joined summary) ===
            div { class: "card-body border-t pt-3 mt-2",
                div { class: "flex items-center justify-between text-sm",
                    span { class: "text-muted-foreground", "Since you joined (Aug 2024)" }
                    div { class: "flex items-center gap-2",
                        span { class: "font-mono font-bold text-success", "+$1,234.56" }
                        span { class: "text-success", "(+11.1%)" }
                    }
                }
            }
        }
    }
}

// =============================================================================
// Wave 23 T5 — data model for `data_portfolio` (BFF proxy:
// /api/v1/portfolio/<addr>). Mirrors the OLD prod wire shape plus
// the BFF mock's enriched holdings/watchlist/transactions fields.
// =============================================================================

#[derive(Clone, Debug, serde::Deserialize)]
struct PortfolioData {
    #[serde(default)] total_value_usd: Option<f64>,
    #[serde(default)] change_24h_usd: Option<f64>,
    #[serde(default)] change_24h_pct: Option<f64>,
    #[serde(default)] asset_count: Option<i64>,
    #[serde(default)] holdings: Option<Vec<HoldingRow>>,
    #[serde(default)] watchlist: Option<Vec<WatchRow>>,
    #[serde(default)] transactions: Option<Vec<TxRow>>,
    #[serde(default)] subscriptions: Option<Vec<serde_json::Value>>,
}

#[derive(Clone, Debug, serde::Deserialize, PartialEq)]
struct HoldingRow {
    #[serde(default, alias = "asset")] asset: String,
    #[serde(default, alias = "amount")] amount: String,
    #[serde(default)] value_usd: Option<f64>,
    #[serde(default)] change_24h_pct: Option<f64>,
}

impl HoldingRow {
    fn value(&self) -> String {
        self.value_usd.map(format_usd).unwrap_or_else(|| "—".to_string())
    }
    fn change(&self) -> String {
        self.change_24h_pct.map(|p| format!("{:+.2}%", p)).unwrap_or_else(|| "0%".to_string())
    }
}

#[derive(Clone, Debug, serde::Deserialize, PartialEq)]
struct WatchRow {
    #[serde(default, alias = "asset")] asset: String,
    #[serde(default, alias = "price")] price: String,
    #[serde(default, alias = "change_24h_pct")] change: String,
}

#[derive(Clone, Debug, serde::Deserialize, PartialEq)]
struct TxRow {
    #[serde(default, alias = "time")] time: String,
    #[serde(default, alias = "type", alias = "kind")] kind: String,
    #[serde(default, alias = "asset")] asset: String,
    #[serde(default, alias = "amount")] amount: String,
    #[serde(default, alias = "value_usd")] value: String,
}

fn default_holdings() -> Vec<HoldingRow> {
    let rows: Vec<(String, String, f64, f64)> = vec![
        ("BNB".into(),   "5.234".into(),    2_892.45,  1.2),
        ("USDT".into(),  "5,000.00".into(), 5_000.00,  0.0),
        ("ETH".into(),   "1.2".into(),      3_540.00,  0.8),
        ("EPSX".into(),  "10,000".into(),     845.00,  5.4),
    ];
    rows.into_iter().map(|(asset, amount, value_usd, change_24h_pct)| HoldingRow {
        asset, amount, value_usd: Some(value_usd), change_24h_pct: Some(change_24h_pct),
    }).collect()
}

fn default_watchlist() -> Vec<WatchRow> {
    vec![
        WatchRow { asset: "BTC".into(),   price: "$63,245".into(), change: "+2.1%".into() },
        WatchRow { asset: "SOL".into(),   price: "$145.32".into(), change: "-0.5%".into() },
        WatchRow { asset: "MATIC".into(), price: "$0.45".into(),   change: "+0.1%".into() },
    ]
}

fn default_transactions() -> Vec<TxRow> {
    vec![
        TxRow { time: "2024-09-20 10:32".into(), kind: "Buy".into(),     asset: "BNB".into(),  amount: "0.5".into(),   value: "$276.50".into() },
        TxRow { time: "2024-09-19 15:21".into(), kind: "Receive".into(), asset: "USDT".into(), amount: "1,000".into(), value: "$1,000.00".into() },
        TxRow { time: "2024-09-19 09:14".into(), kind: "Sell".into(),    asset: "ETH".into(),  amount: "0.2".into(),   value: "$590.00".into() },
        TxRow { time: "2024-09-18 12:00".into(), kind: "Swap".into(),    asset: "EPSX".into(), amount: "500".into(),   value: "$42.25".into() },
    ]
}

fn format_usd(v: f64) -> String {
    // Format as `$X,XXX.XX` (no leading currency symbol beyond `$`,
    // no fractional cents, comma thousands). Matches the OLD prod
    // StatCard string format.
    let sign = if v < 0.0 { "-" } else { "" };
    let v = v.abs();
    let whole = v.trunc() as i64;
    let cents = (v.fract() * 100.0).round() as i64;
    let mut s = whole.to_string();
    // Insert commas every 3 digits from the right.
    let bytes = s.as_bytes().to_vec();
    let mut out = Vec::new();
    for (i, b) in bytes.iter().rev().enumerate() {
        if i > 0 && i % 3 == 0 { out.push(b','); }
        out.push(*b);
    }
    s = out.into_iter().rev().map(|b| b as char).collect();
    format!("{sign}${s}.{cents:02}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::User;
    use crate::pages::PageContext;
    use crate::auth::user::AuthMethod;

    fn ctx(path: &str) -> PageContext {
        let user = User {
            id: "u1".to_string(),
            address: "0x1234…abcd".to_string(),
            chain_id: "56".to_string(),
            roles: vec!["user".to_string()],
            email: Some("test@epsx.io".to_string()),
            tier: Some("Pro".to_string()),
            permissions: vec!["payments:read".to_string()],
            last_login_at: None,
            auth_method: AuthMethod::Wallet,
            display_name: Some("Test".to_string()),
        };
        PageContext { user: Some(user), path: path.to_string(), ..Default::default() }
    }

    #[test]
    fn test_render_smoke() {
        let (_meta, element) = render(&ctx("/portfolio"));
        let html = dioxus_ssr::render_element(element);
        assert!(html.contains("Portfolio"), "/portfolio header must render. Got: {}", html);
    }

    #[test]
    fn test_section_markers() {
        let (_meta, element) = render(&ctx("/portfolio"));
        let html = dioxus_ssr::render_element(element);
        for marker in [
            "portfolio-stats",
            "portfolio-performance-chart",
            "portfolio-allocation-chart",
            "portfolio-tab-nav",
        ] {
            assert!(html.contains(marker), "missing section marker: {}", marker);
        }
    }
}

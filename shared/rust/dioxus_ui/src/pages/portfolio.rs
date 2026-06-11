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
use crate::auth::AuthGate;
use crate::charts::{ChartDonut, ChartLine, DataPoint, Series};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Portfolio");
    (meta, rsx! { RenderPortfolio { ctx: ctx.clone() } })
}

#[component]
fn RenderPortfolio(ctx: PageContext) -> Element {
    let mut tab = use_signal(|| "holdings".to_string());
    rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("your portfolio".to_string()),
                required_permissions: Some(vec!["payments:read".to_string()]),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content",
                    // === wave6-auth-pages-depth-track-d portfolio header ===
                    PageHeader { title: "Portfolio".to_string(), description: Some("Track your holdings and watchlist performance".to_string()), icon: Some("briefcase".to_string()) }
                    // === wave6-auth-pages-depth-track-d portfolio stat cards ===
                    div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 mb-6 portfolio-stats",
                        StatCard { label: "Total value".to_string(), value: "$12,345.67".to_string(), icon: Some("trending-up".to_string()) }
                        StatCard { label: "24h change".to_string(), value: "+$234.56 (+1.9%)".to_string(), icon: Some("arrow-up-right".to_string()) }
                        StatCard { label: "Assets".to_string(), value: "8".to_string(), icon: Some("layers".to_string()) }
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
                    // === wave6-auth-pages-depth-track-d portfolio tab panels ===
                    if *tab.read() == "holdings" { HoldingsTable {} }
                    else if *tab.read() == "watchlist" {
                        div { class: "space-y-4 portfolio-watchlist-panel",
                            // === wave6-auth-pages-depth-track-d portfolio add-to-watchlist ===
                            AddToWatchlistForm {}
                            WatchlistTable {}
                        }
                    }
                    else { TransactionsTable {} }
                }
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
        ChartLine {
            series: vec![Series { name: "Portfolio".to_string(), color: "#22d3ee".to_string(), points: (0..30).map(|i| DataPoint { x: i as f64, y: 10000.0 + (i as f64 * 80.0) + (i as f64 * 0.3).sin() * 200.0, label: None }).collect() }],
            width: 640, height: 220,
        }
    }
}

#[component]
fn HoldingsTable() -> Element {
    let rows = vec![
        ("BNB", "5.234", "$2,892.45", "+1.2%"),
        ("USDT", "5,000.00", "$5,000.00", "0%"),
        ("ETH", "1.2", "$3,540.00", "+0.8%"),
        ("EPSX", "10,000", "$845.00", "+5.4%"),
    ];
    rsx! {
        div { class: "card card-glass portfolio-holdings-table",
            div { class: "card-body p-0",
                div { class: "table-wrap",
                    table { class: "table",
                        thead { tr { th { "Asset" } th { "Amount" } th { "Value" } th { "24h" } } }
                        tbody {
                            for (a, amt, val, ch) in rows {
                                tr {
                                    td { span { class: "font-semibold", "{a}" } }
                                    td { class: "font-mono", "{amt}" }
                                    td { class: "font-mono", "{val}" }
                                    td { class: if ch.starts_with('+') { "text-success" } else { "text-muted-foreground" }, "{ch}" }
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
fn WatchlistTable() -> Element {
    let mut rows = use_signal(|| vec![
        ("BTC".to_string(), "$63,245".to_string(), "+2.1%".to_string()),
        ("SOL".to_string(), "$145.32".to_string(), "-0.5%".to_string()),
        ("MATIC".to_string(), "$0.45".to_string(), "+0.1%".to_string()),
    ]);
    rsx! {
        div { class: "card card-glass portfolio-watchlist-table",
            div { class: "card-body p-0",
                div { class: "table-wrap",
                    table { class: "table",
                        thead { tr { th { "Asset" } th { "Price" } th { "24h" } th { "" } } }
                        tbody {
                            for (i, (a, p, ch)) in rows.read().iter().cloned().enumerate() {
                                tr {
                                    td { span { class: "font-semibold", "{a}" } }
                                    td { class: "font-mono", "{p}" }
                                    td { class: if ch.starts_with('+') { "text-success" } else { "text-danger" }, "{ch}" }
                                    td { button { class: "btn btn-sm btn-outline", r#type: "button",
                                        onclick: move |_| {
                                            let mut v = rows.read().clone();
                                            if i < v.len() { v.remove(i); }
                                            rows.set(v);
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
        div { class: "card card-glass portfolio-add-to-watchlist",
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
fn TransactionsTable() -> Element {
    let rows = vec![
        ("2024-09-20 10:32", "Buy", "BNB", "0.5", "$276.50"),
        ("2024-09-19 15:21", "Receive", "USDT", "1,000", "$1,000.00"),
        ("2024-09-19 09:14", "Sell", "ETH", "0.2", "$590.00"),
        ("2024-09-18 12:00", "Swap", "EPSX", "500", "$42.25"),
    ];
    rsx! {
        div { class: "card card-glass portfolio-transactions-table",
            div { class: "card-body p-0",
                div { class: "table-wrap",
                    table { class: "table",
                        thead { tr { th { "Time" } th { "Type" } th { "Asset" } th { "Amount" } th { "Value" } } }
                        tbody {
                            for (t, ty, a, amt, v) in rows {
                                tr {
                                    td { class: "text-sm text-muted-foreground", "{t}" }
                                    td { span { class: "badge badge-info", "{ty}" } }
                                    td { span { class: "font-semibold", "{a}" } }
                                    td { class: "font-mono", "{amt}" }
                                    td { class: "font-mono", "{v}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
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

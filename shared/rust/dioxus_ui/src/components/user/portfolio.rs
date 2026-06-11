//! Sub-components extracted from `pages/portfolio.rs` during
//! Wave 6C Track E (1:1 user-side component parity).
//!
//! Seven named sub-components to lift: `PerformanceChart`,
//! `HoldingsTable`, `WatchlistTable`, `AddToWatchlistForm`,
//! `TransactionsTable`, `TopMoversCard`. The page file's
//! `RenderPortfolio` wrapper orchestrates them.
//!
//! Source: `apps-old/frontend/app/portfolio/page.tsx` (48 LoC) +
//! `components/portfolio/portfolio-dashboard.tsx` +
//! `watchlist-provider.tsx` (113 LoC) +
//! `portfolio-grid.tsx` + `stock-search.tsx`.
//!
//! Reuses the existing `TopMoversCard` from the
//! Wave 6A-extracted layout (mentioned in the design doc's
//! "existing extracted sub-components" list).

use crate::primitives::*;
use crate::feedback::empty_chart_state::EmptyChartState;

use dioxus::prelude::*;
use crate::charts::{ChartLine, DataPoint, Series};

/// `PerformanceChart` — line chart of portfolio value over time.
/// Renders the chart with mock data when there is data; falls
/// back to the new `<EmptyChartState>` primitive when there is
/// no data. Mirrors the Next.js portfolio-dashboard that shows
/// an empty chart placeholder before the user connects a data
/// source.
#[component]
pub fn PerformanceChart() -> Element {
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
        div { class: "performance-chart",
            ChartLine {
                series: vec![Series { name: "Portfolio".to_string(), color: "#22d3ee".to_string(), points: (0..30).map(|i| DataPoint { x: i as f64, y: 10000.0 + (i as f64 * 80.0) + (i as f64 * 0.3).sin() * 200.0, label: None }).collect() }],
                width: 640, height: 220,
            }
        }
    }
}

/// `HoldingsTable` — list of holdings rows (asset, amount,
/// value, 24h change).
#[component]
pub fn HoldingsTable() -> Element {
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

/// `WatchlistTable` — list of watched assets with a "Remove"
/// button per row. Mirrors the watchlist surface in
/// `watchlist-provider.tsx`.
#[component]
pub fn WatchlistTable() -> Element {
    let mut rows = use_signal(|| vec![
        ("BTC".to_string(), "$63,245".to_string(), "+2.1%".to_string()),
        ("SOL".to_string(), "$145.32".to_string(), "-0.5%".to_string()),
        ("MATIC".to_string(), "$0.45".to_string(), "+0.1%".to_string()),
    ]);
    rsx! {
        div { class: "card card-glass portfolio-watchlist-table watchlist-table",
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

/// `AddToWatchlistForm` — input + submit button to add an
/// asset. Mirrors the search/add pattern in `stock-search.tsx`.
/// Static form for now (no autocomplete against the BFF — Wave
/// 7).
#[component]
pub fn AddToWatchlistForm() -> Element {
    let mut symbol = use_signal(|| String::new());
    rsx! {
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

/// `TransactionsTable` — recent transactions table (time,
/// type, asset, amount, value).
#[component]
pub fn TransactionsTable() -> Element {
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

/// `TopMoversCard` — list of the top 3 gainers and top 3
/// losers in the user's portfolio over the last 24h. Mirrors
/// the "Top Movers" widget on the home page but scoped to
/// holdings. Each row shows asset symbol, % change, and
/// dollar change. Includes a "since you joined" footer.
#[component]
pub fn TopMoversCard() -> Element {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Wave 6C Track E — `test_render_smoke` for the extracted
    /// portfolio sub-components.
    #[test]
    fn portfolio_subcomponents_render_smoke() {
        // PerformanceChart
        let el = rsx! { PerformanceChart {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("performance-chart"), "PerformanceChart missing section-marker");

        // HoldingsTable
        let el = rsx! { HoldingsTable {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("portfolio-holdings-table"), "HoldingsTable missing section-marker");
        assert!(html.contains("BNB"));

        // WatchlistTable
        let el = rsx! { WatchlistTable {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("portfolio-watchlist-table"), "WatchlistTable missing section-marker");
        assert!(html.contains("BTC"));
        assert!(html.contains("Remove"));

        // AddToWatchlistForm
        let el = rsx! { AddToWatchlistForm {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("portfolio-add-to-watchlist"), "AddToWatchlistForm missing section-marker");
        assert!(html.contains("Add to watchlist"));
        assert!(html.contains("action=\"/api/v1/portfolio/watchlist\""));

        // TransactionsTable
        let el = rsx! { TransactionsTable {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("portfolio-transactions-table"), "TransactionsTable missing section-marker");
        assert!(html.contains("Buy"));

        // TopMoversCard
        let el = rsx! { TopMoversCard {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("portfolio-top-movers"), "TopMoversCard missing section-marker");
        assert!(html.contains("Gainers"));
        assert!(html.contains("Losers"));
        assert!(html.contains("Top movers (24h)"));
    }
}

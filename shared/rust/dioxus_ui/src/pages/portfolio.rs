//! /portfolio — wallet holdings, watchlist, recent transactions.

use crate::primitives::*;
use crate::feedback::*;

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
                div { class: "container page-content",
                    PageHeader { title: "Portfolio".to_string(), description: Some("Track your holdings and watchlist performance".to_string()), icon: Some("briefcase".to_string()) }
                    div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 mb-6",
                        StatCard { label: "Total value".to_string(), value: "$12,345.67".to_string(), icon: Some("trending-up".to_string()) }
                        StatCard { label: "24h change".to_string(), value: "+$234.56 (+1.9%)".to_string(), icon: Some("arrow-up-right".to_string()) }
                        StatCard { label: "Assets".to_string(), value: "8".to_string(), icon: Some("layers".to_string()) }
                    }
                    div { class: "grid grid-cols-1 lg:grid-cols-3 gap-4 mb-6",
                        div { class: "card card-glass lg:col-span-2",
                            div { class: "card-header", h3 { class: "card-title", "Performance (30d)" } }
                            div { class: "card-body",
                                ChartLine {
                                    series: vec![Series { name: "Portfolio".to_string(), color: "#22d3ee".to_string(), points: (0..30).map(|i| DataPoint { x: i as f64, y: 10000.0 + (i as f64 * 80.0) + (i as f64 * 0.3).sin() * 200.0, label: None }).collect() }],
                                    width: 640, height: 220,
                                }
                            }
                        }
                        div { class: "card card-glass",
                            div { class: "card-header", h3 { class: "card-title", "Allocation" } }
                            div { class: "card-body",
                                ChartDonut { data: vec![("BNB".to_string(), 35.0, "#f3ba2f".to_string()), ("USDT".to_string(), 30.0, "#26a17b".to_string()), ("ETH".to_string(), 20.0, "#627eea".to_string()), ("EPSX".to_string(), 10.0, "#22d3ee".to_string()), ("Other".to_string(), 5.0, "#9ca3af".to_string())], size: 180, thickness: 28 }
                            }
                        }
                    }
                    div { class: "tabs mb-4",
                        button { class: if *tab.read() == "holdings" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("holdings".to_string()), "Holdings" }
                        button { class: if *tab.read() == "watchlist" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("watchlist".to_string()), "Watchlist" }
                        button { class: if *tab.read() == "transactions" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("transactions".to_string()), "Transactions" }
                    }
                    if *tab.read() == "holdings" { HoldingsTable {} } else if *tab.read() == "watchlist" { WatchlistTable {} } else { TransactionsTable {} }
                }
            }
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
        div { class: "card card-glass",
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

#[component]
fn WatchlistTable() -> Element {
    let rows = vec![
        ("BTC", "$63,245", "+2.1%", "Watching"),
        ("SOL", "$145.32", "-0.5%", "Watching"),
        ("MATIC", "$0.45", "+0.1%", "Watching"),
    ];
    rsx! {
        div { class: "card card-glass",
            div { class: "card-body p-0",
                div { class: "table-wrap",
                    table { class: "table",
                        thead { tr { th { "Asset" } th { "Price" } th { "24h" } th { "" } } }
                        tbody {
                            for (a, p, ch, _) in rows {
                                tr {
                                    td { span { class: "font-semibold", "{a}" } }
                                    td { class: "font-mono", "{p}" }
                                    td { class: if ch.starts_with('+') { "text-success" } else { "text-danger" }, "{ch}" }
                                    td { button { class: "btn btn-sm btn-outline", r#type: "button", "Remove" } }
                                }
                            }
                        }
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
        div { class: "card card-glass",
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

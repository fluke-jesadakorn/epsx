//! /portfolio — wallet holdings, watchlist, recent transactions.
//!
//! Wave 6C Track E — the 6 portfolio sub-components
//! (PerformanceChart, HoldingsTable, WatchlistTable,
//! AddToWatchlistForm, TransactionsTable, TopMoversCard) were
//! extracted to `crate::components::user::portfolio`. The page
//! file's `RenderPortfolio` wrapper orchestrates them.

use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::auth::AuthGate;
use crate::charts::ChartDonut;
use crate::components::user::portfolio::{
    AddToWatchlistForm, HoldingsTable, PerformanceChart, TopMoversCard, TransactionsTable,
    WatchlistTable,
};

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
                    PageHeader { title: "Portfolio".to_string(), description: Some("Track your holdings and watchlist performance".to_string()), icon: Some("briefcase".to_string()) }
                    div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 mb-6 portfolio-stats",
                        StatCard { label: "Total value".to_string(), value: "$12,345.67".to_string(), icon: Some("trending-up".to_string()) }
                        StatCard { label: "24h change".to_string(), value: "+$234.56 (+1.9%)".to_string(), icon: Some("arrow-up-right".to_string()) }
                        StatCard { label: "Assets".to_string(), value: "8".to_string(), icon: Some("layers".to_string()) }
                    }
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
                    div { class: "tabs mb-4 portfolio-tab-nav",
                        button { class: if *tab.read() == "holdings" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("holdings".to_string()), "Holdings" }
                        button { class: if *tab.read() == "watchlist" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("watchlist".to_string()), "Watchlist" }
                        button { class: if *tab.read() == "transactions" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("transactions".to_string()), "Transactions" }
                    }
                    TopMoversCard {}
                    if *tab.read() == "holdings" { HoldingsTable {} }
                    else if *tab.read() == "watchlist" {
                        div { class: "space-y-4 portfolio-watchlist-panel",
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

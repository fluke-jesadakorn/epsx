//! /analytics — analytics dashboard with charts.

use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::auth::AuthGate;
use crate::charts::{ChartBar, ChartLine, DataPoint, Series};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Analytics");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("analytics".to_string()),
                div { class: "container page-content",
                    PageHeader {
                        title: "Analytics".to_string(),
                        description: Some("Your trading performance and platform usage".to_string()),
                        icon: Some("chart-line".to_string()),
                    }
                    div { class: "grid grid-cols-1 md:grid-cols-4 gap-4 mb-6",
                        StatCard { label: "Total trades".to_string(), value: "1,234".to_string(), icon: Some("repeat".to_string()) }
                        StatCard { label: "Win rate".to_string(), value: "62%".to_string(), icon: Some("target".to_string()) }
                        StatCard { label: "Avg return".to_string(), value: "+3.4%".to_string(), icon: Some("trending-up".to_string()) }
                        StatCard { label: "Best trade".to_string(), value: "+24.5%".to_string(), icon: Some("award".to_string()) }
                    }
                    div { class: "grid grid-cols-1 lg:grid-cols-2 gap-4 mb-6",
                        div { class: "card card-glass",
                            div { class: "card-header", h3 { class: "card-title", "P&L over time" } }
                            div { class: "card-body",
                                ChartLine {
                                    series: vec![
                                        Series {
                                            name: "P&L".to_string(),
                                            color: "#22d3ee".to_string(),
                                            points: (0..30).map(|i| DataPoint { x: i as f64, y: (i as f64 * 0.5) + (i as f64 * 0.7).sin() * 3.0, label: None }).collect(),
                                        }
                                    ],
                                    width: 480, height: 220,
                                }
                            }
                        }
                        div { class: "card card-glass",
                            div { class: "card-header", h3 { class: "card-title", "Volume by day" } }
                            div { class: "card-body",
                                ChartBar {
                                    data: (0..7).map(|i| (format!("D{}", i + 1), 100.0 + (i as f64 * 30.0) + (i as f64 * 0.9).sin() * 40.0)).collect(),
                                    width: 480, height: 220,
                                }
                            }
                        }
                    }
                    div { class: "card card-glass",
                        div { class: "card-header", h3 { class: "card-title", "Top assets" } }
                        div { class: "card-body p-0",
                            div { class: "table-wrap",
                                table { class: "table",
                                    thead { tr { th { "Asset" } th { "Trades" } th { "Volume" } th { "P&L" } th { "Win %" } } }
                                    tbody {
                                        tr { td { span { class: "font-semibold", "BNB" } } td { "234" } td { "$45,231" } td { class: "text-success font-mono", "+$1,234" } td { "65%" } }
                                        tr { td { span { class: "font-semibold", "ETH" } } td { "189" } td { "$32,109" } td { class: "text-success font-mono", "+$890" } td { "58%" } }
                                        tr { td { span { class: "font-semibold", "EPSX" } } td { "412" } td { "$8,432" } td { class: "text-danger font-mono", "-$123" } td { "48%" } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}

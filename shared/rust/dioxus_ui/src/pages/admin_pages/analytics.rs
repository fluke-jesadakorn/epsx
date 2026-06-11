//! /admin/analytics — analytics dashboard.

use crate::primitives::*;
use crate::charts::{ChartLine, DataPoint, Series, ChartBar};

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::auth::AuthGate;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Analytics");
    (meta, rsx! {
        AuthGate { user: ctx.user.clone(), feature: Some("platform analytics".to_string()),
            div { class: "container page-content",
                div { class: "mb-6",
                    h1 { class: "text-2xl font-bold", "Platform analytics" }
                    p { class: "text-muted-foreground", "Real-time platform metrics" }
                }
                div { class: "grid grid-cols-1 md:grid-cols-4 gap-4 mb-6",
                    StatCard { label: "MAU".to_string(), value: "12,345".to_string(), icon: Some("users".to_string()) }
                    StatCard { label: "DAU".to_string(), value: "1,234".to_string(), icon: Some("user".to_string()) }
                    StatCard { label: "Conversion".to_string(), value: "3.4%".to_string(), icon: Some("target".to_string()) }
                    StatCard { label: "Churn".to_string(), value: "1.2%".to_string(), icon: Some("log-out".to_string()) }
                }
                div { class: "grid grid-cols-1 lg:grid-cols-2 gap-4 mb-6",
                    div { class: "card card-glass",
                        div { class: "card-header", h3 { class: "card-title", "Active users (30d)" } }
                        div { class: "card-body",
                            ChartLine {
                                series: vec![
                                    Series { name: "DAU".to_string(), color: "#22d3ee".to_string(),
                                        points: (0..30).map(|i| DataPoint { x: i as f64, y: 1000.0 + (i as f64 * 25.0) + (i as f64 * 0.4).sin() * 100.0, label: None }).collect() }
                                ],
                                width: 480, height: 220,
                            }
                        }
                    }
                    div { class: "card card-glass",
                        div { class: "card-header", h3 { class: "card-title", "Volume (7d)" } }
                        div { class: "card-body",
                            ChartBar {
                                data: (0..7).map(|i| (format!("D{}", i + 1), 1000.0 + (i as f64 * 100.0))).collect(),
                                width: 480, height: 220,
                            }
                        }
                    }
                }
                div { class: "card card-glass",
                    div { class: "card-header", h3 { class: "card-title", "Top events" } }
                    div { class: "card-body p-0",
                        div { class: "table-wrap",
                            table { class: "table",
                                thead { tr { th { "Event" } th { "Count" } th { "Users" } th { "Conversion" } } }
                                tbody {
                                    tr { td { code { "wallet.connect" } } td { "1,234" } td { "890" } td { "72%" } }
                                    tr { td { code { "plan.subscribe" } } td { "432" } td { "412" } td { "95%" } }
                                    tr { td { code { "payment.confirm" } } td { "345" } td { "320" } td { "92%" } }
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}

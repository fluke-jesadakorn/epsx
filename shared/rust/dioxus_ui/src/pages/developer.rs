//! /developer + /developer/usage + /developer/docs — developer portal.

use crate::primitives::*;
use crate::feedback::*;
use crate::data_table::{Column, DataTable, Row, SortDir};
use crate::charts::ChartLine;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::{PageHeader, DeveloperShell};
use crate::auth::AuthGate;

pub fn render_overview(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Developer");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("the developer portal".to_string()),
                DeveloperShell { current_path: ctx.path.clone(),
                    div { class: "container page-content",
                        PageHeader { title: "Developer portal".to_string(), description: Some("API keys, usage, and documentation".to_string()), icon: Some("code".to_string()),
                            a { class: "btn btn-primary", href: "/developer/api-keys/create", Icon { name: "plus".to_string(), size: Some(16) } " Create key" }
                        }
                        div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                            StatCard { label: "Active keys".to_string(), value: "3".to_string(), icon: Some("key".to_string()) }
                            StatCard { label: "Calls today".to_string(), value: "1,234".to_string(), icon: Some("zap".to_string()) }
                            StatCard { label: "Quota used".to_string(), value: "12%".to_string(), icon: Some("chart-line".to_string()) }
                        }
                        div { class: "card card-glass mt-6",
                            div { class: "card-header", h3 { class: "card-title", "API keys" } }
                            div { class: "card-body p-0",
                                div { class: "table-wrap",
                                    table { class: "table",
                                        thead { tr { th { "Name" } th { "Key" } th { "Created" } th { "Last used" } th { "" } } }
                                        tbody {
                                            tr { td { "Production" } td { code { class: "text-xs", "epsx_live_xxx" } } td { "2024-08-01" } td { "5 min ago" } td { button { class: "btn btn-sm btn-outline", r#type: "button", "Revoke" } } }
                                            tr { td { "Staging" } td { code { class: "text-xs", "epsx_test_xxx" } } td { "2024-08-15" } td { "1 hour ago" } td { button { class: "btn btn-sm btn-outline", r#type: "button", "Revoke" } } }
                                        }
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

pub fn render_usage(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("API usage");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("usage stats".to_string()),
                DeveloperShell { current_path: ctx.path.clone(),
                    div { class: "container page-content",
                        PageHeader { title: "API usage".to_string(), description: Some("Per-day call volume and rate-limit status".to_string()), icon: Some("chart-line".to_string()) }
                        div { class: "card card-glass",
                            div { class: "card-body",
                                ChartLine {
                                    series: vec![
                                        crate::charts::Series { name: "Calls".to_string(), color: "#22d3ee".to_string(),
                                            points: (0..7).map(|i| crate::charts::DataPoint { x: i as f64, y: 800.0 + (i as f64 * 50.0), label: None }).collect() }
                                    ],
                                    width: 720, height: 220,
                                }
                            }
                        }
                        div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 mt-4",
                            StatCard { label: "Rate limit".to_string(), value: "1000/min".to_string(), icon: Some("gauge".to_string()) }
                            StatCard { label: "Current usage".to_string(), value: "234/min".to_string(), icon: Some("activity".to_string()) }
                            StatCard { label: "Errors (24h)".to_string(), value: "3".to_string(), icon: Some("alert-triangle".to_string()) }
                        }
                    }
                }
            }
        }
    })
}

pub fn render_docs(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("API documentation");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            DeveloperShell { current_path: ctx.path.clone(),
                div { class: "container page-content",
                    PageHeader { title: "API documentation".to_string(), description: Some("REST endpoints, request/response schemas, and examples".to_string()), icon: Some("book".to_string()) }
                    div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                        div { class: "card card-glass md:col-span-1",
                            div { class: "card-header", h3 { class: "card-title", "Endpoints" } }
                            div { class: "card-body",
                                ul { class: "docs-nav space-y-1",
                                    li { a { class: "block p-2 rounded hover:bg-muted", href: "#auth", "Auth" } }
                                    li { a { class: "block p-2 rounded hover:bg-muted", href: "#payments", "Payments" } }
                                    li { a { class: "block p-2 rounded hover:bg-muted", href: "#subscriptions", "Subscriptions" } }
                                    li { a { class: "block p-2 rounded hover:bg-muted", href: "#analytics", "Analytics" } }
                                    li { a { class: "block p-2 rounded hover:bg-muted", href: "#notifications", "Notifications" } }
                                }
                            }
                        }
                        div { class: "md:col-span-2 space-y-4",
                            div { class: "card card-glass",
                                div { class: "card-body",
                                    h2 { id: "auth", class: "text-xl font-bold", "Auth" }
                                    p { class: "text-muted-foreground mt-2", "All API calls require a Bearer token. Get one via the SIWE flow." }
                                    pre { class: "code-block mt-3", "POST /api/v1/auth/challenge\nPOST /api/v1/auth/siwe" }
                                }
                            }
                            div { class: "card card-glass",
                                div { class: "card-body",
                                    h2 { id: "payments", class: "text-xl font-bold", "Payments" }
                                    p { class: "text-muted-foreground mt-2", "Create and confirm payment intents." }
                                    pre { class: "code-block mt-3", "POST /api/v1/payment/intents\nPOST /api/v1/payment/intents/[id]/confirm" }
                                }
                            }
                            div { class: "card card-glass",
                                div { class: "card-body",
                                    h2 { id: "subscriptions", class: "text-xl font-bold", "Subscriptions" }
                                    p { class: "text-muted-foreground mt-2", "Create plans and subscribe." }
                                    pre { class: "code-block mt-3", "POST /api/v1/subscription/plans\nPOST /api/v1/subscription/subscribe" }
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}

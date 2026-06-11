//! /admin — Command Center (admin dashboard).
//!
//! Real-time visibility: stat cards (active wallets, total volume,
//! active plans, open tickets, API calls), recent activity, quick
//! actions. Mirrors `apps/admin-frontend/components/admin/...` admin
//! dashboard.

use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::auth::AdminAuthGate;
use crate::charts::{ChartLine, Series, DataPoint};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Command Center");
    (meta, rsx! {
        AdminAuthGate { user: ctx.user.clone(), feature: Some("the admin dashboard".to_string()), required_permissions: Some(vec!["admin:*".to_string()]), return_url: Some(ctx.path.clone()),
            div { class: "container page-content",
                // Pulse header
                div { class: "admin-pulse-header card card-primary-solid",
                    div { class: "card-body",
                        div { class: "flex items-center justify-between",
                            div {
                                h1 { class: "text-2xl font-bold", "Welcome to the Command Center" }
                                p { class: "text-muted-foreground", "Real-time visibility into your platform" }
                            }
                            div { class: "pulse-indicator",
                                span { class: "pulse-dot" }
                                span { "Live" }
                            }
                        }
                    }
                }
                // 5-up stat row
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4 mt-6",
                    StatCard { label: "Active wallets".to_string(), value: "0".to_string(), icon: Some("wallet".to_string()) }
                    StatCard { label: "Total volume".to_string(), value: "$0".to_string(), icon: Some("trending-up".to_string()) }
                    StatCard { label: "Active plans".to_string(), value: "0".to_string(), icon: Some("layout-dashboard".to_string()) }
                    StatCard { label: "Open tickets".to_string(), value: "0".to_string(), icon: Some("message-circle".to_string()) }
                    StatCard { label: "API calls today".to_string(), value: "0".to_string(), icon: Some("zap".to_string()) }
                }
                // Volume chart
                div { class: "card card-glass mt-6",
                    div { class: "card-header flex items-center justify-between",
                        h3 { class: "card-title", "Volume (7d)" }
                        div { class: "flex gap-1",
                            button { class: "btn btn-sm btn-outline", "1D" }
                            button { class: "btn btn-sm btn-primary", "7D" }
                            button { class: "btn btn-sm btn-outline", "30D" }
                            button { class: "btn btn-sm btn-outline", "90D" }
                        }
                    }
                    div { class: "card-body",
                        ChartLine {
                            series: vec![
                                Series {
                                    name: "Volume".to_string(),
                                    color: "#22d3ee".to_string(),
                                    points: (0..7).map(|i| DataPoint { x: i as f64, y: 100.0 + (i as f64 * 17.0).sin() * 30.0 + i as f64 * 5.0, label: None }).collect(),
                                }
                            ],
                            width: 720,
                            height: 220,
                        }
                    }
                }
                // Two-col: activity + quick actions
                div { class: "grid grid-cols-1 lg:grid-cols-3 gap-4 mt-6",
                    div { class: "lg:col-span-2",
                        div { class: "card card-glass",
                            div { class: "card-header", h3 { class: "card-title", "Recent platform activity" } }
                            div { class: "card-body",
                                EmptyState { title: "No recent activity".to_string(), description: Some("Wallet activity, plan subscriptions, and API calls will appear here in real time.".to_string()), icon: Some("history".to_string()) }
                            }
                        }
                    }
                    div {
                        div { class: "card card-glass",
                            div { class: "card-header", h3 { class: "card-title", "Quick actions" } }
                            div { class: "card-body flex flex-col gap-2",
                                a { class: "btn btn-outline btn-block", href: "/wallet-management/wallets", Icon { name: "wallet".to_string(), size: Some(16) } " Manage wallets" }
                                a { class: "btn btn-outline btn-block", href: "/wallet-management/access/plans", Icon { name: "list".to_string(), size: Some(16) } " Manage plans" }
                                a { class: "btn btn-outline btn-block", href: "/notifications/create", Icon { name: "send".to_string(), size: Some(16) } " Send notification" }
                                a { class: "btn btn-outline btn-block", href: "/news/create", Icon { name: "file-plus".to_string(), size: Some(16) } " Publish news" }
                                a { class: "btn btn-outline btn-block", href: "/audit-log", Icon { name: "list-checks".to_string(), size: Some(16) } " Audit log" }
                            }
                        }
                        div { class: "card card-glass mt-4",
                            div { class: "card-header", h3 { class: "card-title", "System" } }
                            div { class: "card-body text-sm",
                                div { class: "flex justify-between py-1", span { "Environment" } span { class: "text-muted-foreground", "production" } }
                                div { class: "flex justify-between py-1", span { "Database" } span { class: "text-muted-foreground", "PostgreSQL" } }
                                div { class: "flex justify-between py-1", span { "Cache" } span { class: "text-muted-foreground", "Redis" } }
                            }
                        }
                    }
                }
            }
        }
    })
}

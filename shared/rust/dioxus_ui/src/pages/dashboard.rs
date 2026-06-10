//! /dashboard — the EPSX public dashboard with real stat cards, news
//! strip, and quick actions.
//!
//! Mirrors `apps/frontend/components/dashboard/dashboard-client.tsx` and
//! the `/api/v1/analytics/dashboard` aggregator.

use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::{Navbar, Footer, PageHeader};
use crate::auth::AuthGate;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Dashboard");
    (meta, rsx! { RenderDashboard { ctx: ctx.clone() } })
}

#[component]
fn RenderDashboard(ctx: PageContext) -> Element {
    let data_dashboard: Option<DashboardData> = ctx.params.get("data_dashboard")
        .and_then(|s| serde_json::from_str(s).ok());
    let earnings = data_dashboard.as_ref().map(|d| d.total_earnings.as_str()).unwrap_or("$0.00").to_string();
    let watchlist = data_dashboard.as_ref().map(|d| d.watchlist_count).unwrap_or(0);
    let active_plans = data_dashboard.as_ref().map(|d| d.active_plans).unwrap_or(0);
    let api_calls = data_dashboard.as_ref().map(|d| d.api_calls_today).unwrap_or(0);
    let recent: Vec<Activity> = data_dashboard.as_ref().map(|d| d.recent.clone()).unwrap_or_default();
    let mut watchlist_signal = use_signal(|| watchlist);
    let mut plans_signal = use_signal(|| active_plans);

    rsx! {
        Navbar { user: ctx.user.clone(), current_path: Some(ctx.path.clone()) }
        AuthGate { user: ctx.user.clone(), feature: Some("your dashboard".to_string()),
            div { class: "container page-content",
                PageHeader {
                    title: "Dashboard".to_string(),
                    description: Some("Overview of your EPSX account".to_string()),
                    icon: Some("layout-dashboard".to_string()),
                }
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4",
                    StatCard { label: "Total earnings".to_string(), value: earnings, icon: Some("trending-up".to_string()) }
                    StatCard { label: "Watchlist".to_string(), value: watchlist_signal.read().to_string(), icon: Some("briefcase".to_string()) }
                    StatCard { label: "Active plans".to_string(), value: plans_signal.read().to_string(), icon: Some("layout-dashboard".to_string()) }
                    StatCard { label: "API calls today".to_string(), value: api_calls.to_string(), icon: Some("code".to_string()) }
                }
                div { class: "grid grid-cols-1 lg:grid-cols-3 gap-4 mt-6",
                    div { class: "lg:col-span-2",
                        div { class: "card card-glass",
                            div { class: "card-header flex justify-between items-center",
                                h3 { class: "card-title", "Recent activity" }
                                button { class: "btn btn-sm btn-outline", r#type: "button", "Refresh" }
                            }
                            div { class: "card-body",
                                if recent.is_empty() {
                                    EmptyState { title: "No recent activity".to_string(), description: Some("Your account activity will appear here once you start using EPSX.".to_string()), icon: Some("history".to_string()) }
                                } else {
                                    ActivityList { items: recent }
                                }
                            }
                        }
                    }
                    div {
                        div { class: "card card-glass",
                            div { class: "card-header", h3 { class: "card-title", "Quick actions" } }
                            div { class: "card-body flex flex-col gap-2",
                                a { class: "btn btn-outline btn-block", href: "/analytics", Icon { name: "chart-line".to_string(), size: Some(16) } " Open analytics" }
                                a { class: "btn btn-outline btn-block", href: "/portfolio", Icon { name: "briefcase".to_string(), size: Some(16) } " Manage portfolio" }
                                a { class: "btn btn-outline btn-block", href: "/plans", Icon { name: "zap".to_string(), size: Some(16) } " Upgrade plan" }
                                a { class: "btn btn-outline btn-block", href: "/developer", Icon { name: "code".to_string(), size: Some(16) } " Developer portal" }
                                a { class: "btn btn-outline btn-block", href: "/notifications", Icon { name: "bell".to_string(), size: Some(16) } " Notifications" }
                            }
                        }
                        div { class: "card card-glass mt-4",
                            div { class: "card-header", h3 { class: "card-title", "Your account" } }
                            div { class: "card-body text-sm",
                                if let Some(u) = &ctx.user {
                                    p { "Address: " span { class: "font-mono text-xs", "{u.address}" } }
                                    p { "Chain: " span { class: "font-semibold", "{u.chain_id}" } }
                                    p { "Roles: " span { class: "font-semibold", "{u.roles.join(\", \")}" } }
                                } else {
                                    p { "Not signed in" }
                                }
                            }
                        }
                    }
                }
            }
        }
        Footer {}
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
struct DashboardData {
    #[serde(default)]
    total_earnings: String,
    #[serde(default)]
    watchlist_count: i64,
    #[serde(default)]
    active_plans: i64,
    #[serde(default)]
    api_calls_today: i64,
    #[serde(default)]
    recent: Vec<Activity>,
}

#[derive(Clone, Debug, serde::Deserialize, PartialEq)]
struct Activity {
    #[serde(default)]
    kind: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    timestamp: String,
}

#[component]
fn ActivityList(items: Vec<Activity>) -> Element {
    rsx! {
        div { class: "activity-list",
            for item in items.iter() {
                div { class: "activity-item flex items-start gap-3 py-3 border-b border-border last:border-0",
                    div { class: "activity-icon w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center shrink-0",
                        Icon { name: activity_icon(&item.kind), size: Some(16) }
                    }
                    div { class: "flex-1",
                        p { class: "font-semibold", "{item.title}" }
                        if !item.description.is_empty() {
                            p { class: "text-sm text-muted-foreground", "{item.description}" }
                        }
                    }
                    span { class: "text-xs text-muted-foreground shrink-0", "{item.timestamp}" }
                }
            }
        }
    }
}

fn activity_icon(kind: &str) -> String {
    match kind {
        "payment" => "credit-card",
        "subscription" => "zap",
        "notification" => "bell",
        "api" => "code",
        "wallet" => "wallet",
        _ => "activity",
    }.to_string()
}

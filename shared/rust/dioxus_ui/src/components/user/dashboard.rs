//! Sub-components extracted from `pages/dashboard.rs` during
//! Wave 6C Track E (1:1 user-side component parity).
//!
//! Nine named sub-components: `RenderDashboard`, `StatCardsRow`,
//! `EarningsChart`, `WatchlistSnapshot`, `PlanSummaryCard`,
//! `ActivityCard`, `QuickActionsCard`, `YourAccountCard`,
//! `ActivityList`. Also the `DashboardData` + `Activity` data types
//! (made `pub` for module surface).

use crate::auth::User;
use crate::pages::PageContext;
use crate::primitives::*;
use crate::feedback::*;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::auth::AuthGate;
use crate::charts::{ChartLine, DataPoint, Series};

use dioxus::prelude::*;

/// `DashboardData` — the BFF-supplied payload (parsed from
/// `ctx.params["data_dashboard"]`).
#[derive(Clone, Debug, serde::Deserialize)]
pub struct DashboardData {
    #[serde(default)]
    pub total_earnings: String,
    #[serde(default)]
    pub watchlist_count: i64,
    #[serde(default)]
    pub active_plans: i64,
    #[serde(default)]
    pub api_calls_today: i64,
    #[serde(default)]
    pub recent: Vec<Activity>,
}

#[derive(Clone, Debug, serde::Deserialize, PartialEq)]
pub struct Activity {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub timestamp: String,
}

/// Page-level orchestrator for the `/dashboard` route.
#[component]
pub fn RenderDashboard(ctx: PageContext) -> Element {
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
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("your dashboard".to_string()),
                required_permissions: Some(vec!["dashboard:read".to_string()]),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content",
                    PageHeader {
                        title: "Dashboard".to_string(),
                        description: Some("Overview of your EPSX account".to_string()),
                        icon: Some("layout-dashboard".to_string()),
                    }
                    StatCardsRow {
                        earnings: earnings.clone(),
                        watchlist: watchlist_signal.read().to_string(),
                        active_plans: plans_signal.read().to_string(),
                        api_calls: api_calls.to_string(),
                    }
                    EarningsChart { earnings: earnings.clone() }
                    div { class: "grid grid-cols-1 lg:grid-cols-3 gap-4 mt-6",
                        WatchlistSnapshot {}
                        PlanSummaryCard {}
                    }
                    div { class: "grid grid-cols-1 lg:grid-cols-3 gap-4 mt-6",
                        div { class: "lg:col-span-2",
                            ActivityCard { recent: recent.clone() }
                        }
                        div {
                            QuickActionsCard {}
                            div { class: "mt-4",
                                YourAccountCard { user: ctx.user.clone() }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 4-card top row: earnings, watchlist, active plans, API calls.
#[component]
pub fn StatCardsRow(
    earnings: String,
    watchlist: String,
    active_plans: String,
    api_calls: String,
) -> Element {
    rsx! {
        div { class: "stat-cards-row",
            div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4",
                StatCard { label: "Total earnings".to_string(), value: earnings, icon: Some("trending-up".to_string()) }
                StatCard { label: "Watchlist".to_string(), value: watchlist, icon: Some("briefcase".to_string()) }
                StatCard { label: "Active plans".to_string(), value: active_plans, icon: Some("layout-dashboard".to_string()) }
                StatCard { label: "API calls today".to_string(), value: api_calls, icon: Some("code".to_string()) }
            }
        }
    }
}

/// 7-day line chart of earnings.
#[component]
pub fn EarningsChart(earnings: String) -> Element {
    let series = vec![Series {
        name: "Earnings (7d)".to_string(),
        color: "#22d3ee".to_string(),
        points: vec![
            DataPoint { x: 1.0, y: 12.0, label: Some("Mon".to_string()) },
            DataPoint { x: 2.0, y: 18.0, label: Some("Tue".to_string()) },
            DataPoint { x: 3.0, y: 15.0, label: Some("Wed".to_string()) },
            DataPoint { x: 4.0, y: 22.0, label: Some("Thu".to_string()) },
            DataPoint { x: 5.0, y: 28.0, label: Some("Fri".to_string()) },
            DataPoint { x: 6.0, y: 24.0, label: Some("Sat".to_string()) },
            DataPoint { x: 7.0, y: 32.0, label: Some("Sun".to_string()) },
        ],
    }];
    rsx! {
        div { class: "dashboard-earnings-chart",
            div { class: "card card-glass mt-6",
                div { class: "card-header flex justify-between items-center",
                    h3 { class: "card-title", "Earnings (last 7 days)" }
                    span { class: "badge badge-info", "Total: {earnings}" }
                }
                div { class: "card-body",
                    ChartLine { series: series, width: 720, height: 220 }
                }
            }
        }
    }
}

/// Top 3 watchlist items with a tiny sparkline each.
#[component]
pub fn WatchlistSnapshot() -> Element {
    let items = vec![
        ("ETH",  "+3.4%",  true),
        ("BTC",  "+1.2%",  true),
        ("SOL",  "-0.8%",  false),
    ];
    rsx! {
        div { class: "watchlist-snapshot",
            div { class: "card card-glass",
                div { class: "card-header flex justify-between items-center",
                    h3 { class: "card-title", "Watchlist" }
                    a { class: "btn btn-sm btn-outline", href: "/portfolio", "View all" }
                }
                div { class: "card-body p-0",
                    div { class: "table-wrap",
                        table { class: "table",
                            thead { tr { th { "Asset" } th { "24h" } th { "Trend" } } }
                            tbody {
                                for (asset, change, up) in items {
                                    tr { class: "watchlist-snapshot-row",
                                        td { class: "font-semibold", "{asset}" }
                                        td {
                                            span { class: if up { "badge badge-success" } else { "badge badge-danger" }, "{change}" }
                                        }
                                        td {
                                            Icon {
                                                name: (if up { "trending-up" } else { "trending-down" }).to_string(),
                                                size: Some(16),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Current plan + usage % + upgrade CTA.
#[component]
pub fn PlanSummaryCard() -> Element {
    rsx! {
        div { class: "plan-summary-card",
            div { class: "card card-glass",
                div { class: "card-header",
                    h3 { class: "card-title", "Your plan" }
                }
                div { class: "card-body",
                    p { class: "text-2xl font-bold", "Pro" }
                    p { class: "text-sm text-muted-foreground mt-1", "Renews Sep 15, 2026" }
                    div { class: "mt-3",
                        div { class: "text-xs flex justify-between mb-1",
                            span { "API usage this month" }
                            span { "62%" }
                        }
                        div { class: "progress",
                            div { class: "progress-bar", style: "width: 62%;" }
                        }
                    }
                    a { class: "btn btn-primary btn-block mt-4", href: "/plans",
                        Icon { name: "zap".to_string(), size: Some(16) }
                        " Upgrade plan"
                    }
                }
            }
        }
    }
}

/// Recent activity feed with a Refresh button and an empty-state CTA link.
#[component]
pub fn ActivityCard(recent: Vec<Activity>) -> Element {
    rsx! {
        div { class: "activity-card",
            div { class: "card card-glass",
                div { class: "card-header flex justify-between items-center",
                    h3 { class: "card-title", "Recent activity" }
                    button { class: "btn btn-sm btn-outline", r#type: "button", "Refresh" }
                }
                div { class: "card-body",
                    if recent.is_empty() {
                        EmptyState {
                            title: "No recent activity".to_string(),
                            description: Some("Your account activity will appear here once you start using EPSX.".to_string()),
                            icon: Some("history".to_string()),
                            action: Some("Explore plans".to_string()),
                            action_href: Some("/plans".to_string()),
                        }
                    } else {
                        ActivityList { items: recent }
                    }
                }
            }
        }
    }
}

/// 5 quick action buttons.
#[component]
pub fn QuickActionsCard() -> Element {
    rsx! {
        div { class: "quick-actions-card",
            div { class: "card card-glass",
                div { class: "card-header", h3 { class: "card-title", "Quick actions" } }
                div { class: "card-body flex flex-col gap-2",
                    a { class: "btn btn-outline btn-block", href: "/analytics",
                        Icon { name: "chart-line".to_string(), size: Some(16) }
                        " Open analytics"
                    }
                    a { class: "btn btn-outline btn-block", href: "/portfolio",
                        Icon { name: "briefcase".to_string(), size: Some(16) }
                        " Manage portfolio"
                    }
                    a { class: "btn btn-outline btn-block", href: "/plans",
                        Icon { name: "zap".to_string(), size: Some(16) }
                        " Upgrade plan"
                    }
                    a { class: "btn btn-outline btn-block", href: "/developer",
                        Icon { name: "code".to_string(), size: Some(16) }
                        " Developer portal"
                    }
                    a { class: "btn btn-outline btn-block", href: "/notifications",
                        Icon { name: "bell".to_string(), size: Some(16) }
                        " Notifications"
                    }
                }
            }
        }
    }
}

/// Wallet + chain + roles summary.
#[component]
pub fn YourAccountCard(user: Option<User>) -> Element {
    rsx! {
        div { class: "your-account-card",
            div { class: "card card-glass",
                div { class: "card-header flex justify-between items-center",
                    h3 { class: "card-title", "Your account" }
                    a { class: "btn btn-sm btn-outline", href: "/account", "Manage" }
                }
                div { class: "card-body text-sm",
                    if let Some(u) = &user {
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

#[component]
pub fn ActivityList(items: Vec<Activity>) -> Element {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Wave 6C Track E — `test_render_smoke` for the extracted
    /// dashboard sub-components.
    #[test]
    fn dashboard_subcomponents_render_smoke() {
        // StatCardsRow
        let el = rsx! {
            StatCardsRow {
                earnings: "$0.00".to_string(),
                watchlist: "0".to_string(),
                active_plans: "0".to_string(),
                api_calls: "0".to_string(),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("stat-cards-row"), "StatCardsRow missing section-marker");
        assert!(html.contains("Total earnings"));

        // EarningsChart
        let el = rsx! { EarningsChart { earnings: "$0".to_string() } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("dashboard-earnings-chart"));
        assert!(html.contains("Earnings (last 7 days)"));

        // WatchlistSnapshot
        let el = rsx! { WatchlistSnapshot {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("watchlist-snapshot"));
        assert!(html.contains("ETH"));

        // PlanSummaryCard
        let el = rsx! { PlanSummaryCard {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("plan-summary-card"));
        assert!(html.contains("Upgrade plan"));

        // ActivityCard (empty)
        let el = rsx! { ActivityCard { recent: vec![] } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("activity-card"));
        assert!(html.contains("No recent activity"));

        // QuickActionsCard
        let el = rsx! { QuickActionsCard {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("quick-actions-card"));
        assert!(html.contains("Open analytics"));

        // YourAccountCard (no user)
        let el = rsx! { YourAccountCard { user: None } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("your-account-card"));
        assert!(html.contains("Not signed in"));
    }
}

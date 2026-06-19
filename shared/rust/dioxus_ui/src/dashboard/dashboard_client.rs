//! `DashboardClient` — full dashboard view with stats cards +
//! activity feed.
//!
//! Port of
//! `apps-old/frontend/components/dashboard/dashboard-client.tsx`
//! (295 LoC). The TS source is a client component that fetches
//! dashboard stats + activity via the dashboard service. The
//! Dioxus port renders the same visual structure with the data
//! provided by the caller (the BFF fetches via
//! `/api/v1/dashboard/stats` and passes the result through the
//! page context).

use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct DashboardStats {
    pub total_credits: String,
    pub active_subscriptions: u64,
    pub api_calls_24h: u64,
    pub rank: String,
    pub recent_activity: Vec<DashboardActivity>,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct DashboardActivity {
    pub id: String,
    pub title: String,
    pub timestamp: String,
    pub kind: String,
}

#[component]
pub fn DashboardClient(stats: DashboardStats) -> Element {
    rsx! {
        section { class: "dashboard-client",
            div { class: "container mx-auto px-4 py-8",
                h1 { class: "dashboard-client-title text-3xl font-bold text-foreground mb-6",
                    "Dashboard"
                }
                div { class: "dashboard-client-stats grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-8",
                    StatCard { label: "Total Credits", value: "{stats.total_credits}", icon: "dollar-sign" }
                    StatCard { label: "Subscriptions", value: "{stats.active_subscriptions}", icon: "repeat" }
                    StatCard { label: "API Calls (24h)", value: "{stats.api_calls_24h}", icon: "activity" }
                    StatCard { label: "Rank", value: "{stats.rank}", icon: "trending-up" }
                }
                div { class: "dashboard-client-activity card card-glass",
                    div { class: "card-header",
                        div { class: "card-title", "Recent activity" }
                    }
                    div { class: "card-body",
                        if stats.recent_activity.is_empty() {
                            div { class: "dashboard-client-empty text-center py-8 text-slate-500",
                                "No recent activity"
                            }
                        } else {
                            ul { class: "dashboard-client-activity-list space-y-3",
                                for a in stats.recent_activity.iter() {
                                    li { class: "dashboard-client-activity-item flex items-center justify-between p-3 rounded-lg border border-slate-200 dark:border-slate-700",
                                        div { class: "flex items-center gap-3",
                                            Icon { name: a.kind.clone(), size: Some(16), class_name: Some("text-orange-500".to_string()) }
                                            div {
                                                div { class: "font-medium text-foreground", "{a.title}" }
                                                div { class: "text-xs text-slate-500", "{a.timestamp}" }
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

#[component]
fn StatCard(label: &'static str, value: String, icon: &'static str) -> Element {
    rsx! {
        div { class: "dashboard-client-stat-card card card-glass p-5",
            div { class: "flex items-center justify-between mb-3",
                Icon { name: icon.to_string(), size: Some(20), class_name: Some("text-orange-500".to_string()) }
            }
            div { class: "dashboard-client-stat-value text-2xl font-bold text-foreground", "{value}" }
            div { class: "dashboard-client-stat-label text-xs text-slate-500", "{label}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dashboard_stats_default() {
        let s = DashboardStats::default();
        assert!(s.total_credits.is_empty());
        assert_eq!(s.active_subscriptions, 0);
    }

    #[test]
    fn dashboard_client_smoke() {
        
    }
}

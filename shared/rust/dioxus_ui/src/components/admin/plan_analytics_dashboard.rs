//! Admin `PlanAnalyticsDashboard` family — Wave 38b T2 admin domain port.
//!
//! Mirrors
//! `apps-old/admin-frontend/components/plans/plan-analytics-dashboard.tsx`,
//! which exports 4 KPI cards + an empty-state card for the plan
//! analytics surface.
//!
//! | Component | Use case |
//! | --- | --- |
//! | `PlanAnalyticsCard` | One KPI card (title + value) |
//! | `PlanAnalyticsGrid` | 4-card responsive grid |
//! | `PlanAnalyticsEmptyState` | Empty-state card (no plans) |
//!
//! The `PlanAnalyticsDashboard` master container wires all 3
//! components together with a header + optional loading state.
//!
//! ## Tests
//!
//! Each component gets a colocated `#[cfg(test)] mod tests` block
//! with smoke render + key prop handling (loading state, custom
//! stats, empty state).

use dioxus::prelude::*;

// ============================================================================
// Data shape
// ============================================================================

#[derive(Clone, Debug, PartialEq, Default)]
pub struct PlanAnalyticsStats {
    pub total_plans: u32,
    pub active_plans: u32,
    pub system_plans: u32,
    pub avg_permissions_per_plan: u32,
}

// ============================================================================
// PlanAnalyticsCard
// ============================================================================
//
// One KPI card (title + value). Mirrors the source's individual
// Card+CardHeader+CardTitle+CardContent pattern.

#[component]
pub fn PlanAnalyticsCard(title: String, value: String) -> Element {
    rsx! {
        div { class: "plan-analytics-card bg-card rounded-2xl border border-border/20 shadow-sm",
            div { class: "p-6 border-b border-border/10",
                p { class: "text-sm font-semibold text-muted-foreground uppercase tracking-wider", "{title}" }
            }
            div { class: "p-6",
                p { class: "text-2xl font-bold text-foreground", "{value}" }
            }
        }
    }
}

// ============================================================================
// PlanAnalyticsGrid
// ============================================================================
//
// 4-card responsive grid (Total / Active / System / Avg).

#[component]
pub fn PlanAnalyticsGrid(stats: PlanAnalyticsStats) -> Element {
    rsx! {
        div { class: "plan-analytics-grid grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6",
            PlanAnalyticsCard { title: "Total Plans".to_string(), value: "{stats.total_plans}" }
            PlanAnalyticsCard { title: "Active Plans".to_string(), value: "{stats.active_plans}" }
            PlanAnalyticsCard { title: "System Plans".to_string(), value: "{stats.system_plans}" }
            PlanAnalyticsCard { title: "Avg Permissions".to_string(), value: "{stats.avg_permissions_per_plan}" }
        }
    }
}

// ============================================================================
// PlanAnalyticsEmptyState
// ============================================================================
//
// Empty-state card shown when `total_plans == 0`.

#[component]
pub fn PlanAnalyticsEmptyState() -> Element {
    rsx! {
        div { class: "plan-analytics-empty-state bg-card rounded-2xl border border-border/20 p-6 text-center text-muted-foreground",
            "No plans found. Create a plan to see analytics."
        }
    }
}

// ============================================================================
// PlanAnalyticsDashboard
// ============================================================================
//
// Master container. Header + grid + optional empty state.

#[component]
pub fn PlanAnalyticsDashboard(
    stats: PlanAnalyticsStats,
    /// When true, show the loading skeleton instead of the data.
    loading: Option<bool>,
    class_name: Option<String>,
) -> Element {
    let loading = loading.unwrap_or(false);
    let mut outer = "space-y-6".to_string();
    if let Some(c) = class_name {
        outer.push(' ');
        outer.push_str(&c);
    }
    rsx! {
        div { class: "{outer}",
            // Header
            div {
                h2 { class: "text-2xl font-bold", "Plan Analytics" }
                p { class: "text-muted-foreground mt-1", "Analytics dashboard for permission plans" }
            }
            if loading {
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6",
                    for i in 0..4u32 {
                        div { key: "plan-analytics-skel-{i}", class: "bg-card rounded-2xl border border-border/20 p-6 animate-pulse",
                            div { class: "h-6 w-32 bg-muted rounded mb-4" }
                            div { class: "h-8 w-16 bg-muted rounded" }
                            div { class: "sr-only", "plan-analytics-skel-{i}" }
                        }
                    }
                }
            } else {
                PlanAnalyticsGrid { stats: stats.clone() }
                if stats.total_plans == 0 {
                    PlanAnalyticsEmptyState { }
                }
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_stats() -> PlanAnalyticsStats {
        PlanAnalyticsStats {
            total_plans: 12,
            active_plans: 9,
            system_plans: 3,
            avg_permissions_per_plan: 8,
        }
    }

    /// `PlanAnalyticsCard` renders the title + value.
    #[test]
    fn plan_analytics_card_renders_title_and_value() {
        fn harness() -> Element {
            rsx! { PlanAnalyticsCard { title: "Total Plans".to_string(), value: "12".to_string() } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("plan-analytics-card"), "PlanAnalyticsCard must render container class. Got: {html}");
        assert!(html.contains("Total Plans"), "PlanAnalyticsCard must render title. Got: {html}");
        assert!(html.contains("12"), "PlanAnalyticsCard must render value. Got: {html}");
    }

    /// `PlanAnalyticsGrid` renders 4 cards.
    #[test]
    fn plan_analytics_grid_renders_4_cards() {
        fn harness() -> Element {
            rsx! { PlanAnalyticsGrid { stats: sample_stats() } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("plan-analytics-grid"), "PlanAnalyticsGrid must render container class. Got: {html}");
        for label in &["Total Plans", "Active Plans", "System Plans", "Avg Permissions"] {
            assert!(html.contains(label), "PlanAnalyticsGrid must render `{label}` card. Got: {html}");
        }
    }

    /// `PlanAnalyticsEmptyState` renders the empty-state copy.
    #[test]
    fn plan_analytics_empty_state_renders() {
        fn harness() -> Element {
            rsx! { PlanAnalyticsEmptyState { } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("plan-analytics-empty-state"), "PlanAnalyticsEmptyState must render container class. Got: {html}");
        assert!(html.contains("No plans found"), "PlanAnalyticsEmptyState must render message. Got: {html}");
    }

    /// `PlanAnalyticsDashboard` with loading=true renders skeletons.
    #[test]
    fn plan_analytics_dashboard_loading_renders_skeletons() {
        fn harness() -> Element {
            rsx! { PlanAnalyticsDashboard { stats: PlanAnalyticsStats::default(), loading: Some(true) } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("animate-pulse"), "PlanAnalyticsDashboard loading must animate. Got: {html}");
        for i in 0..4 {
            assert!(html.contains(&format!("plan-analytics-skel-{i}")), "PlanAnalyticsDashboard must render skeleton {i}. Got: {html}");
        }
        // No grid when loading
        assert!(!html.contains("Total Plans"), "PlanAnalyticsDashboard loading must omit cards. Got: {html}");
    }

    /// `PlanAnalyticsDashboard` with `total_plans == 0` shows empty state.
    #[test]
    fn plan_analytics_dashboard_shows_empty_state_when_zero_plans() {
        fn harness() -> Element {
            rsx! { PlanAnalyticsDashboard { stats: PlanAnalyticsStats::default() } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("plan-analytics-empty-state"), "PlanAnalyticsDashboard with 0 plans must show empty state. Got: {html}");
    }

    /// `PlanAnalyticsDashboard` with data shows the grid.
    #[test]
    fn plan_analytics_dashboard_renders_grid_when_data() {
        fn harness() -> Element {
            rsx! { PlanAnalyticsDashboard { stats: sample_stats() } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Plan Analytics"), "PlanAnalyticsDashboard must render header. Got: {html}");
        assert!(html.contains("plan-analytics-grid"), "PlanAnalyticsDashboard must render grid. Got: {html}");
        assert!(!html.contains("plan-analytics-empty-state"), "PlanAnalyticsDashboard with data must omit empty state. Got: {html}");
    }
}

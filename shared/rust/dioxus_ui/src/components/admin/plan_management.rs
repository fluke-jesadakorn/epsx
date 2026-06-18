//! Admin `PlanManagement` — Wave 38b T2 admin domain port.
//!
//! Mirrors `apps-old/admin-frontend/components/plans/plan-management.tsx`,
//! which renders the plan list page for admin. We port the
//! stats grid + plan card + loading skeleton — the BFF hydrates
//! real data on click.
//!
//! | Component | Use case |
//! | --- | --- |
//! | `PlanManagementStatsGrid` | 4-card stats header (Total / Active / Enterprise / Avg Price) |
//! | `PlanManagementLoadingState` | Loading skeleton for the page |
//! | `PlanManagementPlanCard` | One plan card (name / price / features) |
//!
//! ## Tests
//!
//! Each component gets a colocated `#[cfg(test)] mod tests` block
//! with smoke render + key prop handling.

use dioxus::prelude::*;

// ============================================================================
// Data shape
// ============================================================================

#[derive(Clone, Debug, PartialEq)]
pub struct PlanSummary {
    pub id: String,
    pub name: String,
    pub tier_level: u32,
    pub price_usd: f64,
    pub subscriber_count: u32,
    pub is_active: bool,
    pub plan_category: Option<String>,
    pub revenue_last_30_days: f64,
}

// ============================================================================
// PlanManagementStatsGrid
// ============================================================================
//
// 4-card stats header. Mirrors the source's `StatsGrid`.

#[component]
pub fn PlanManagementStatsGrid(
    total_plans: u32,
    active_plans: u32,
    enterprise_plans: u32,
    avg_price_usd: f64,
) -> Element {
    rsx! {
        div { class: "plan-management-stats grid grid-cols-2 sm:grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-6 mb-6 sm:mb-8",
            // Total
            div { class: "bg-card rounded-2xl p-4 sm:p-6 shadow-xl border border-primary/20",
                div { class: "flex items-center justify-between mb-3 sm:mb-4",
                    div { class: "text-xl sm:text-2xl", "\u{1F4B3}" }
                    span { class: "text-xs sm:text-sm font-medium text-muted-foreground", "Total" }
                }
                div { class: "space-y-1",
                    div { class: "text-2xl sm:text-3xl font-bold text-primary", "{total_plans}" }
                    div { class: "text-xs sm:text-sm text-foreground/80", "Plans" }
                    div { class: "text-xs text-muted-foreground", "All time" }
                }
            }
            // Active
            div { class: "bg-card/80 rounded-2xl p-4 sm:p-6 shadow-xl border border-secondary/20",
                div { class: "flex items-center justify-between mb-3 sm:mb-4",
                    div { class: "text-xl sm:text-2xl", "\u{2705}" }
                    span { class: "text-xs sm:text-sm font-medium text-muted-foreground", "Active" }
                }
                div { class: "space-y-1",
                    div { class: "text-2xl sm:text-3xl font-bold text-secondary", "{active_plans}" }
                    div { class: "text-xs sm:text-sm text-foreground/80", "Active" }
                    div { class: "text-xs text-muted-foreground", "Available" }
                }
            }
            // Enterprise
            div { class: "bg-card/80 rounded-2xl p-4 sm:p-6 shadow-xl border border-secondary/20",
                div { class: "flex items-center justify-between mb-3 sm:mb-4",
                    div { class: "text-xl sm:text-2xl", "\u{1F3E2}" }
                    span { class: "text-xs sm:text-sm font-medium text-muted-foreground", "Enterprise" }
                }
                div { class: "space-y-1",
                    div { class: "text-2xl sm:text-3xl font-bold text-secondary", "{enterprise_plans}" }
                    div { class: "text-xs sm:text-sm text-foreground/80", "Enterprise" }
                    div { class: "text-xs text-muted-foreground", "Premium" }
                }
            }
            // Avg Price
            div { class: "bg-card/80 rounded-2xl p-4 sm:p-6 shadow-xl border border-success/20",
                div { class: "flex items-center justify-between mb-3 sm:mb-4",
                    div { class: "text-xl sm:text-2xl", "\u{1F4B5}" }
                    span { class: "text-xs sm:text-sm font-medium text-muted-foreground", "Price" }
                }
                div { class: "space-y-1",
                    div { class: "text-xl sm:text-3xl font-bold text-success truncate",
                        {
                            let p = format!("${:.2}", avg_price_usd);
                            rsx! { "{p}" }
                        }
                    }
                    div { class: "text-xs sm:text-sm text-foreground/80", "Average" }
                    div { class: "text-xs text-muted-foreground", "USD" }
                }
            }
        }
    }
}

// ============================================================================
// PlanManagementLoadingState
// ============================================================================
//
// Loading skeleton for the plan management page.

#[component]
pub fn PlanManagementLoadingState() -> Element {
    rsx! {
        div { class: "plan-management-loading max-w-7xl mx-auto space-y-8 animate-pulse",
            div { class: "text-center mb-12",
                div { class: "h-16 bg-primary/20 rounded-2xl w-96 mx-auto mb-6" }
                div { class: "h-6 bg-muted rounded-full w-64 mx-auto" }
            }
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-6 mb-12",
                for i in 0..3 {
                    div { key: "plan-tile-{i}", class: "bg-card border border-border rounded-3xl h-64",
                        div { class: "sr-only", "plan-tile-{i}" }
                    }
                }
            }
            div { class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8",
                for i in 0..4 {
                    div { key: "plan-stat-{i}", class: "bg-card border border-border rounded-3xl h-32",
                        div { class: "sr-only", "plan-stat-{i}" }
                    }
                }
            }
        }
    }
}

// ============================================================================
// PlanManagementPlanCard
// ============================================================================
//
// One plan card (name / price / features / actions).

#[component]
pub fn PlanManagementPlanCard(plan: PlanSummary) -> Element {
    let category_cls = match plan.plan_category.as_deref() {
        Some("enterprise") => "border-[#7645d9]/30",
        _ => "border-border/20",
    };
    rsx! {
        div { class: "plan-management-plan-card bg-card rounded-2xl p-6 border {category_cls} shadow-sm hover:shadow-lg transition-shadow",
            div { class: "flex items-start justify-between mb-3",
                h3 { class: "text-lg font-bold text-foreground", "{plan.name}" }
                if plan.is_active {
                    span { class: "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-semibold bg-success/10 text-success border border-success/20", "Active" }
                } else {
                    span { class: "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-semibold bg-muted text-muted-foreground border border-border/50", "Inactive" }
                }
            }
            div { class: "space-y-2",
                div { class: "text-2xl font-bold text-primary",
                    {
                        let p = format!("${:.2}", plan.price_usd);
                        rsx! { "{p}" }
                    }
                    span { class: "text-sm font-medium text-muted-foreground", " / mo" }
                }
                div { class: "flex items-center gap-2 text-xs text-muted-foreground",
                    span { "{plan.subscriber_count} subscribers" }
                    span { "\u{2022}" }
                    span { "Tier {plan.tier_level}" }
                }
                div { class: "text-xs text-muted-foreground",
                    "Revenue 30d: "
                    {
                        let r = format!("${:.2}", plan.revenue_last_30_days);
                        rsx! { "{r}" }
                    }
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

    fn sample_plan() -> PlanSummary {
        PlanSummary {
            id: "plan_pro".to_string(),
            name: "Pro".to_string(),
            tier_level: 2,
            price_usd: 29.0,
            subscriber_count: 412,
            is_active: true,
            plan_category: Some("standard".to_string()),
            revenue_last_30_days: 11948.0,
        }
    }

    /// `PlanManagementStatsGrid` renders 4 cards.
    #[test]
    fn plan_management_stats_grid_renders_4_cards() {
        fn harness() -> Element {
            rsx! {
                PlanManagementStatsGrid {
                    total_plans: 12u32,
                    active_plans: 9u32,
                    enterprise_plans: 3u32,
                    avg_price_usd: 49.99,
                }
            }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("plan-management-stats"), "PlanManagementStatsGrid must render container class. Got: {html}");
        for label in &["Total", "Active", "Enterprise", "Price"] {
            assert!(html.contains(label), "PlanManagementStatsGrid must render `{label}` label. Got: {html}");
        }
        assert!(html.contains("12"), "PlanManagementStatsGrid must render total count. Got: {html}");
        assert!(html.contains("9"), "PlanManagementStatsGrid must render active count. Got: {html}");
        assert!(html.contains("$49.99"), "PlanManagementStatsGrid must render avg price. Got: {html}");
    }

    /// `PlanManagementLoadingState` renders animated skeleton.
    #[test]
    fn plan_management_loading_state_renders_skeleton() {
        fn harness() -> Element {
            rsx! { PlanManagementLoadingState { } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("plan-management-loading"), "PlanManagementLoadingState must render container class. Got: {html}");
        assert!(html.contains("animate-pulse"), "PlanManagementLoadingState must animate. Got: {html}");
        // 3 tile skeletons
        for i in 0..3 {
            assert!(html.contains(&format!("plan-tile-{i}")), "PlanManagementLoadingState must render plan-tile-{i}. Got: {html}");
        }
        // 4 stat skeletons
        for i in 0..4 {
            assert!(html.contains(&format!("plan-stat-{i}")), "PlanManagementLoadingState must render plan-stat-{i}. Got: {html}");
        }
    }

    /// `PlanManagementPlanCard` renders the plan name + price + meta.
    #[test]
    fn plan_management_plan_card_renders_plan_data() {
        fn harness() -> Element {
            rsx! { PlanManagementPlanCard { plan: sample_plan() } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("plan-management-plan-card"), "PlanManagementPlanCard must render container class. Got: {html}");
        assert!(html.contains("Pro"), "PlanManagementPlanCard must render plan name. Got: {html}");
        assert!(html.contains("$29.00"), "PlanManagementPlanCard must render price. Got: {html}");
        assert!(html.contains("412 subscribers"), "PlanManagementPlanCard must render subscriber count. Got: {html}");
        assert!(html.contains("Active"), "PlanManagementPlanCard active plan must show Active badge. Got: {html}");
    }

    /// `PlanManagementPlanCard` for inactive plan shows Inactive badge.
    #[test]
    fn plan_management_plan_card_inactive_renders_inactive_badge() {
        fn harness() -> Element {
            let mut p = sample_plan();
            p.is_active = false;
            rsx! { PlanManagementPlanCard { plan: p } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Inactive"), "PlanManagementPlanCard inactive plan must show Inactive badge. Got: {html}");
    }

    /// `PlanManagementPlanCard` for enterprise plan uses purple border.
    #[test]
    fn plan_management_plan_card_enterprise_uses_purple_border() {
        fn harness() -> Element {
            let mut p = sample_plan();
            p.plan_category = Some("enterprise".to_string());
            rsx! { PlanManagementPlanCard { plan: p } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("border-[#7645d9]/30"), "PlanManagementPlanCard enterprise must use purple border. Got: {html}");
    }
}

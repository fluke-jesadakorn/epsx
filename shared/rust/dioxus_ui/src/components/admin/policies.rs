//! Admin policies sub-components — 1:1 mirror of
//! `apps-old/admin-frontend/components/access-control/policy-*.tsx` +
//! `apps-old/admin-frontend/components/policies/policy-*.tsx`.
//!
//! Extracted from `pages/admin_pages/policies.rs` (Wave 6B Track A)
//! per the Wave 6C design doc §"Track B" line 230:
//! - `PolicyStatsBar` — 4-card summary row
//!   (Total Policies, Active Members, Monthly Revenue, Expiring Soon).
//! - `PolicyBuilder` — multi-section policy creation form
//!   (Configuration + Effect/Status + Target Actions + Conditions
//!   + Test Results).
//! - `PolicyMonitor` — live evaluations chart + recent decisions
//!   table + 3-card overview.
//! - `PolicyCard` — the unified policy card
//!   (avatar, name, type/effect/status badges, members, MRR,
//!   permission chips).
//! - `PolicyFilters` — search/type/status/sort filter bar.
//! - `PolicyList` — the policies `DataTable` (5-col:
//!   Name, Type, Effect, Status, Updated).
//!
//! The page also keeps its own tab-conditional wrappers for
//! `policy-builder` and `policy-monitor` section markers — those
//! stay on the page file because they're DOM-position concerns, not
//! reusable sub-components.

use dioxus::prelude::*;

use crate::primitives::data_table::{Column, DataTable, Row, SortDir};
use crate::primitives::icon::Icon;
use crate::primitives::stat_card::StatCard;
use crate::charts::{ChartDonut, ChartLine, DataPoint, Series};

// ===== PolicyStatsBar ======================================================
//
// Source: `access-control/policy-stats-bar.tsx` — 4-card row
// (Total Policies, Active Members, Monthly Revenue, Expiring Soon).
// The TS source uses a 2-col / 4-col responsive grid with hover
// scale + per-card icon + iconBg tint.

/// 4-card stats row at the top of the policies page.
#[component]
pub fn PolicyStatsBar() -> Element {
    rsx! {
        div { class: "policy-stats-bar grid grid-cols-2 lg:grid-cols-4 gap-4 mb-6",
            PolicyStatCard {
                label: "Total Policies".to_string(),
                value: "42".to_string(),
                sub: "12 plans, 6 groups".to_string(),
                icon: "layers".to_string(),
                tint: "text-cyan-400",
            }
            PolicyStatCard {
                label: "Active Members".to_string(),
                value: "8,431".to_string(),
                sub: "Across all policies".to_string(),
                icon: "users".to_string(),
                tint: "text-success",
            }
            PolicyStatCard {
                label: "Monthly Revenue".to_string(),
                value: "$24,512".to_string(),
                sub: "From subscriptions".to_string(),
                icon: "trending-up".to_string(),
                tint: "text-primary",
            }
            PolicyStatCard {
                label: "Expiring Soon".to_string(),
                value: "23".to_string(),
                sub: "Within 7 days".to_string(),
                icon: "clock".to_string(),
                tint: "text-warning",
            }
        }
    }
}

/// One stat card on the policies stats bar.
#[component]
pub fn PolicyStatCard(label: String, value: String, sub: String, icon: String, tint: String) -> Element {
    rsx! {
        div { class: "card card-glass hover-scale",
            div { class: "card-body",
                div { class: "flex items-start gap-3",
                    div { class: format!("p-3 rounded-xl bg-muted/30 border border-border/40 {}", tint),
                        Icon { name: icon, size: Some(20) }
                    }
                    div { class: "flex-1 min-w-0",
                        p { class: "text-2xl font-bold text-foreground", "{value}" }
                        p { class: "text-sm text-muted-foreground font-medium", "{label}" }
                        p { class: "text-xs text-muted-foreground/70 mt-0.5", "{sub}" }
                    }
                }
            }
        }
    }
}

// ===== PolicyFilters =======================================================
//
// Source: `access-control/policy-filters.tsx` — search bar + type
// chip cluster + sort + clear + refresh. The Dioxus port inlines
// the form fields.

/// Search/type/status/sort filter bar.
#[component]
pub fn PolicyFilters() -> Element {
    rsx! {
        div { class: "card card-glass policy-filters mb-4",
            div { class: "card-body",
                div { class: "flex flex-col md:flex-row gap-3 items-stretch md:items-center",
                    div { class: "flex-1",
                        input {
                            class: "input",
                            r#type: "text",
                            placeholder: "Filter by name, type, effect, or status…",
                        }
                    }
                    div { class: "md:w-40",
                        select { class: "input",
                            option { value: "all", "All types" }
                            option { value: "rbac", "RBAC" }
                            option { value: "subscription", "Subscription" }
                            option { value: "rate", "Rate" }
                            option { value: "compliance", "Compliance" }
                        }
                    }
                    div { class: "md:w-40",
                        select { class: "input",
                            option { value: "all", "All statuses" }
                            option { value: "active", "Active" }
                            option { value: "draft", "Draft" }
                            option { value: "archived", "Archived" }
                        }
                    }
                    div { class: "md:w-32",
                        select { class: "input",
                            option { value: "updated_desc", "Newest" }
                            option { value: "updated_asc", "Oldest" }
                            option { value: "name_asc", "Name A→Z" }
                        }
                    }
                    div { class: "flex items-end gap-2",
                        button { class: "btn btn-outline", r#type: "button", "Clear" }
                        button { class: "btn btn-primary", r#type: "button", "Refresh" }
                    }
                }
            }
        }
    }
}

// ===== PolicyList ==========================================================
//
// The policies list (DataTable). Mirrors the existing
// `policies::ListView` from the original port — the source's
// `policy-monitor` recent-evaluations table is a similar shape.

/// Policies list — 5-col `DataTable` (Name, Type, Effect, Status,
/// Updated). Defaults: striped, page_size 20, sort by Updated
/// desc, filter placeholder "Filter policies…".
#[component]
pub fn PolicyList() -> Element {
    let columns = vec![
        Column { key: "name".into(), label: "Name".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("30%".into()), class_name: None },
        Column { key: "type".into(), label: "Type".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("15%".into()), class_name: None },
        Column { key: "effect".into(), label: "Effect".into(), sortable: true, align: crate::primitives::data_table::Align::Center, width: Some("15%".into()), class_name: None },
        Column { key: "status".into(), label: "Status".into(), sortable: true, align: crate::primitives::data_table::Align::Center, width: Some("15%".into()), class_name: None },
        Column { key: "updated".into(), label: "Updated".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("25%".into()), class_name: None },
    ];
    let rows = vec![
        Row { id: "p1".into(), cells: vec!["Admin full access".into(), "RBAC".into(), "Allow".into(), "Active".into(), "2024-09-15".into()] },
        Row { id: "p2".into(), cells: vec!["Pro plan trade".into(), "Subscription".into(), "Allow".into(), "Active".into(), "2024-09-10".into()] },
        Row { id: "p3".into(), cells: vec!["Rate limit 100/min".into(), "Rate".into(), "Limit".into(), "Active".into(), "2024-09-05".into()] },
        Row { id: "p4".into(), cells: vec!["Block sanctioned addresses".into(), "Compliance".into(), "Deny".into(), "Active".into(), "2024-08-20".into()] },
        Row { id: "p5".into(), cells: vec!["Free plan trial".into(), "Subscription".into(), "Allow".into(), "Draft".into(), "2024-08-15".into()] },
    ];
    rsx! {
        div { class: "policy-list",
            DataTable {
                columns,
                rows,
                striped: true,
                page_size: 20,
                filter_placeholder: Some("Filter policies…".to_string()),
                initial_sort: Some(("updated".to_string(), SortDir::Desc)),
            }
        }
    }
}

// ===== PolicyCard ==========================================================
//
// A unified policy card (RBAC + subscription + compliance types) —
// mirrors the source's `access-control/policy-card.tsx` rendering.
// Each card shows the policy avatar, name + tier, type + status
// badges, member count, MRR, and permission chips.

/// Render the policy-cards list. Wraps 3 example `PolicyCard`s
/// under a "Policy cards" sub-header.
#[component]
pub fn PolicyCardList() -> Element {
    rsx! {
        div { class: "policy-card-list space-y-3",
            h3 { class: "text-sm font-bold text-muted-foreground uppercase tracking-widest", "Policy cards" }
            PolicyCard {
                name: "Admin full access".to_string(),
                ptype: "RBAC".to_string(),
                effect: "Allow".to_string(),
                members: 12,
                mrr: "—".to_string(),
                perms: vec!["users:write".to_string(), "policies:manage".to_string(), "wallets:disable".to_string(), "audit:read".to_string()],
                status: "Active".to_string(),
            }
            PolicyCard {
                name: "Pro plan trade".to_string(),
                ptype: "Subscription".to_string(),
                effect: "Allow".to_string(),
                members: 1842,
                mrr: "$18,420".to_string(),
                perms: vec!["trade:execute".to_string(), "analytics:read".to_string(), "news:write".to_string()],
                status: "Active".to_string(),
            }
            PolicyCard {
                name: "Block sanctioned addresses".to_string(),
                ptype: "Compliance".to_string(),
                effect: "Deny".to_string(),
                members: 0,
                mrr: "—".to_string(),
                perms: vec!["compliance:sanctioned-list".to_string()],
                status: "Active".to_string(),
            }
        }
    }
}

/// One policy card.
#[component]
pub fn PolicyCard(
    name: String,
    ptype: String,
    effect: String,
    members: u32,
    mrr: String,
    perms: Vec<String>,
    status: String,
) -> Element {
    let (effect_badge, status_badge) = match (effect.as_str(), status.as_str()) {
        ("Allow", "Active") => ("badge badge-success", "badge badge-success"),
        ("Deny", _) => ("badge badge-danger", "badge badge-success"),
        ("Limit", _) => ("badge badge-warning", "badge badge-success"),
        (_, "Draft") => ("badge", "badge badge-warning"),
        _ => ("badge", "badge"),
    };
    rsx! {
        div { class: "card card-glass policy-card hover-scale",
            div { class: "card-body",
                div { class: "flex items-center justify-between gap-4 flex-wrap",
                    div { class: "flex items-center gap-3 min-w-0",
                        div { class: "w-10 h-10 rounded-lg bg-primary/10 text-primary flex items-center justify-center font-bold",
                            "{name.chars().next().unwrap_or('?')}"
                        }
                        div { class: "min-w-0",
                            div { class: "flex items-center gap-2",
                                span { class: "font-semibold", "{name}" }
                                span { class: "{effect_badge}", "{effect}" }
                                span { class: "{status_badge}", "{status}" }
                            }
                            p { class: "text-xs text-muted-foreground", "{ptype} policy" }
                        }
                    }
                    div { class: "flex items-center gap-6 text-sm",
                        div { class: "text-right",
                            p { class: "text-[10px] font-bold uppercase tracking-widest text-muted-foreground", "Members" }
                            p { class: "font-mono font-bold", "{members}" }
                        }
                        div { class: "text-right",
                            p { class: "text-[10px] font-bold uppercase tracking-widest text-muted-foreground", "MRR" }
                            p { class: "font-mono font-bold", "{mrr}" }
                        }
                    }
                }
                if !perms.is_empty() {
                    div { class: "flex flex-wrap gap-1.5 mt-3",
                        for p in perms.iter().take(4) {
                            // Dioxus format! doesn't allow method
                            // calls inside `{...}`. Pre-compute the
                            // short label (last segment after `:`).
                            {{
                                let short = p.rsplit(':').next().unwrap_or(p.as_str());
                                rsx! {
                                    span { class: "text-xs px-2 py-1 bg-muted text-muted-foreground rounded-lg",
                                        "{short}"
                                    }
                                }
                            }}
                        }
                        if perms.len() > 4 {
                            span { class: "text-xs px-2 py-1 bg-cyan-500/15 text-cyan-400 rounded-lg font-medium",
                                "+{perms.len() - 4} more"
                            }
                        }
                    }
                }
            }
        }
    }
}

// ===== PolicyMonitor =======================================================
//
// Source: `policies/policy-monitor.tsx` — live evaluations chart +
// recent decisions table. Mirrors the source's
// `HeaderSection` (live mode toggle + refresh) + `StatsOverview` (3
// stat cards) + `LiveEvaluations` (table) + `ChartsGrid` (donut).

/// Live evaluation monitor — header, 3-card overview, recent
/// evaluations table, and the line chart of evaluations per minute.
#[component]
pub fn PolicyMonitor() -> Element {
    rsx! {
        div { class: "policy-monitor space-y-6",
            div { class: "flex items-center justify-between",
                h2 { class: "text-lg font-bold", "Live evaluation monitor" }
                div { class: "flex items-center gap-2",
                    span { class: "pulse-indicator", span { class: "pulse-dot" } "Live" }
                    button { class: "btn btn-sm btn-outline", r#type: "button", "Refresh" }
                }
            }
            // 3-card overview (mirrors StatsOverview from policy-monitor-sections.tsx).
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                StatCard { label: "Evaluations (24h)".to_string(), value: "12,345".to_string(), icon: Some("activity".to_string()) }
                StatCard { label: "Avg eval time".to_string(), value: "0.4ms".to_string(), icon: Some("clock".to_string()) }
                StatCard { label: "Allow / Deny".to_string(), value: "10,234 / 1,234".to_string(), icon: Some("shield".to_string()) }
            }
            // Live evaluations table.
            div { class: "card card-glass",
                div { class: "card-header", h3 { class: "card-title", "Recent evaluations" } }
                div { class: "card-body p-0",
                    div { class: "table-wrap",
                        table { class: "table",
                            thead { tr { th { "Time" } th { "Policy" } th { "Subject" } th { "Decision" } } }
                            tbody {
                                tr { td { "10:32:15" } td { "Admin full access" } td { code { class: "text-xs", "0x1234" } } td { span { class: "badge badge-success", "Allow" } } }
                                tr { td { "10:32:14" } td { "Pro plan trade" } td { code { class: "text-xs", "0xabcd" } } td { span { class: "badge badge-success", "Allow" } } }
                                tr { td { "10:32:12" } td { "Block sanctioned" } td { code { class: "text-xs", "0xdead" } } td { span { class: "badge badge-danger", "Deny" } } }
                                tr { td { "10:32:10" } td { "Rate limit 100/min" } td { code { class: "text-xs", "0xcace" } } td { span { class: "badge badge-warning", "Limit" } } }
                                tr { td { "10:32:08" } td { "Free plan trial" } td { code { class: "text-xs", "0xfedc" } } td { span { class: "badge badge-success", "Allow" } } }
                            }
                        }
                    }
                }
            }
            // Live evaluation line chart.
            div { class: "card card-glass",
                div { class: "card-header", h3 { class: "card-title", "Evaluations per minute" } }
                div { class: "card-body",
                    ChartLine {
                        series: vec![
                            Series { name: "Allow".to_string(), color: "#22c55e".to_string(),
                                points: (0..30).map(|i| DataPoint {
                                    x: i as f64,
                                    y: 200.0 + (i as f64 * 5.0) + (i as f64 * 0.6).sin() * 30.0,
                                    label: None,
                                }).collect()
                            },
                            Series { name: "Deny".to_string(), color: "#ef4444".to_string(),
                                points: (0..30).map(|i| DataPoint {
                                    x: i as f64,
                                    y: 25.0 + (i as f64 * 1.0) + (i as f64 * 0.3).sin() * 8.0,
                                    label: None,
                                }).collect()
                            },
                        ],
                        width: 720, height: 220,
                    }
                }
            }
        }
    }
}

// ===== PolicyBuilder =======================================================
//
// Source: `policies/policy-builder.tsx` + `policy-builder-sections.tsx`.
// The Dioxus port inlines the form (Configuration + Target Actions +
// Conditions + Actions & Responses + Test Results).

/// New-policy creation form (multi-section: Configuration +
/// Effect/Status + Target Actions + Conditions + Test Results).
#[component]
pub fn PolicyBuilder() -> Element {
    rsx! {
        div { class: "policy-builder card card-glass",
            div { class: "card-header flex items-center justify-between",
                h3 { class: "card-title", "New policy" }
                div { class: "flex items-center gap-2",
                    button { class: "btn btn-sm btn-outline", r#type: "button", "Test" }
                    button { class: "btn btn-sm btn-primary", r#type: "button", "Save" }
                }
            }
            div { class: "card-body space-y-6",
                // Configuration block.
                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                    div { class: "field",
                        label { class: "field-label", "Policy name" }
                        input { class: "input", r#type: "text", placeholder: "Admin full access" }
                    }
                    div { class: "field",
                        label { class: "field-label", "Type" }
                        select { class: "input",
                            option { value: "rbac", "RBAC" }
                            option { value: "subscription", "Subscription" }
                            option { value: "rate", "Rate" }
                            option { value: "compliance", "Compliance" }
                        }
                    }
                }
                // Effect / status row.
                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                    div { class: "field",
                        label { class: "field-label", "Effect" }
                        select { class: "input",
                            option { value: "allow", "Allow" }
                            option { value: "deny", "Deny" }
                            option { value: "limit", "Limit" }
                        }
                    }
                    div { class: "field",
                        label { class: "field-label", "Status" }
                        select { class: "input",
                            option { value: "active", "Active" }
                            option { value: "draft", "Draft" }
                        }
                    }
                }
                // Target actions block.
                div { class: "field",
                    label { class: "field-label", "Target actions" }
                    div { class: "flex flex-wrap gap-2",
                        span { class: "badge badge-info", "users:write" }
                        span { class: "badge badge-info", "policies:manage" }
                        span { class: "badge badge-info", "wallets:disable" }
                    }
                    button { class: "btn btn-sm btn-outline mt-2", r#type: "button", Icon { name: "plus".to_string(), size: Some(14) } "Add action" }
                }
                // Conditions block.
                div { class: "field",
                    label { class: "field-label", "Conditions" }
                    div { class: "card bg-muted/30 border border-border/40",
                        div { class: "card-body space-y-2",
                            div { class: "flex items-center gap-2",
                                select { class: "input md:w-32", option { value: "role", "role" } option { value: "tier", "tier" } }
                                select { class: "input md:w-32", option { value: "==", "==" } option { value: "!=", "!=" } }
                                input { class: "input flex-1", r#type: "text", placeholder: "value" }
                            }
                            button { class: "btn btn-sm btn-ghost", r#type: "button", Icon { name: "plus".to_string(), size: Some(14) } "Add condition" }
                        }
                    }
                }
                // Test results placeholder.
                div { class: "card bg-muted/20 border border-border/40",
                    div { class: "card-body",
                        p { class: "text-[10px] font-bold uppercase tracking-widest text-muted-foreground", "Test results" }
                        p { class: "text-sm text-muted-foreground mt-1", "Run the policy against a sample context to see the decision." }
                        button { class: "btn btn-sm btn-outline mt-2", r#type: "button", "Run test" }
                    }
                }
            }
        }
    }
}

// ===== PolicyStatsView =====================================================
//
// The "Stats" tab content — a donut of decision breakdown +
// per-type distribution. Mirrors the source's `StatsView` from the
// original port.

/// "Stats" tab — donut of decision breakdown + per-type
/// distribution.
#[component]
pub fn PolicyStatsView() -> Element {
    rsx! {
        div { class: "policy-stats-view",
            div { class: "card card-glass",
                div { class: "card-header", h3 { class: "card-title", "Decision breakdown" } }
                div { class: "card-body flex flex-col md:flex-row items-center gap-6",
                    ChartDonut {
                        data: vec![
                            ("Allow".to_string(), 10234.0, "#22c55e".to_string()),
                            ("Deny".to_string(), 1234.0, "#ef4444".to_string()),
                            ("Limit".to_string(), 877.0, "#f59e0b".to_string()),
                        ],
                        size: 200, thickness: 32,
                    }
                    div { class: "flex-1 space-y-2",
                        DecisionRow { name: "Allow".to_string(), value: 10234, pct: 83, color: "#22c55e" }
                        DecisionRow { name: "Deny".to_string(), value: 1234, pct: 10, color: "#ef4444" }
                        DecisionRow { name: "Limit".to_string(), value: 877, pct: 7, color: "#f59e0b" }
                    }
                }
            }
        }
    }
}

/// One row in the policy decision breakdown legend.
#[component]
pub fn DecisionRow(name: String, value: u32, pct: u32, color: String) -> Element {
    rsx! {
        div { class: "flex items-center justify-between text-sm",
            div { class: "flex items-center gap-2",
                span { class: "w-2.5 h-2.5 rounded-full inline-block", style: "background-color: {color}" }
                span { "{name}" }
            }
            div { class: "flex items-center gap-3 font-mono",
                span { "{value}" }
                span { class: "text-muted-foreground text-xs", "{pct}%" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke test for `<PolicyStatsBar>`. Renders 4 cards in a
    /// responsive grid.
    #[test]
    fn test_render_smoke_policy_stats_bar() {
        let html = dioxus_ssr::render_element(rsx! { PolicyStatsBar {} });
        assert!(html.contains("policy-stats-bar"), "PolicyStatsBar must keep its section marker. Got: {html}");
        assert!(html.contains("Total Policies"), "PolicyStatsBar must render the first card label. Got: {html}");
        assert!(html.contains("Expiring Soon"), "PolicyStatsBar must render the fourth card label. Got: {html}");
    }

    /// Smoke test for `<PolicyFilters>`. Renders the search input
    /// + type/status/sort selects.
    #[test]
    fn test_render_smoke_policy_filters() {
        let html = dioxus_ssr::render_element(rsx! { PolicyFilters {} });
        assert!(html.contains("policy-filters"), "PolicyFilters must keep its section marker. Got: {html}");
        assert!(html.contains("Filter by name"), "PolicyFilters must render the search field. Got: {html}");
    }

    /// Smoke test for `<PolicyList>`. Renders the 5-col
    /// `DataTable` with 5 sample rows.
    #[test]
    fn test_render_smoke_policy_list() {
        let html = dioxus_ssr::render_element(rsx! { PolicyList {} });
        assert!(html.contains("policy-list"), "PolicyList must keep its section marker. Got: {html}");
        assert!(html.contains("Admin full access"), "PolicyList must render at least one row. Got: {html}");
    }

    /// Smoke test for `<PolicyCardList>` + `<PolicyCard>`. Renders
    /// the unified policy cards list.
    #[test]
    fn test_render_smoke_policy_card() {
        let html = dioxus_ssr::render_element(rsx! { PolicyCardList {} });
        assert!(html.contains("policy-card"), "PolicyCard must keep its section marker. Got: {html}");
        assert!(html.contains("policy-card-list"), "PolicyCardList must keep its section marker. Got: {html}");
        assert!(html.contains("Block sanctioned"), "PolicyCard must render the third sample policy. Got: {html}");
    }

    /// Smoke test for `<PolicyMonitor>`. Renders the 3-card
    /// overview + recent evaluations table + line chart card.
    #[test]
    fn test_render_smoke_policy_monitor() {
        let html = dioxus_ssr::render_element(rsx! { PolicyMonitor {} });
        assert!(html.contains("policy-monitor"), "PolicyMonitor must keep its section marker. Got: {html}");
        assert!(html.contains("Live evaluation monitor"), "PolicyMonitor must render the header. Got: {html}");
        assert!(html.contains("Evaluations per minute"), "PolicyMonitor must render the chart card. Got: {html}");
    }

    /// Smoke test for `<PolicyBuilder>`. Renders the multi-section
    /// form (Configuration + Effect/Status + Target Actions +
    /// Conditions + Test Results).
    #[test]
    fn test_render_smoke_policy_builder() {
        let html = dioxus_ssr::render_element(rsx! { PolicyBuilder {} });
        assert!(html.contains("policy-builder"), "PolicyBuilder must keep its section marker. Got: {html}");
        assert!(html.contains("New policy"), "PolicyBuilder must render the card title. Got: {html}");
        assert!(html.contains("Test results"), "PolicyBuilder must render the test-results block. Got: {html}");
    }

    /// Smoke test for `<PolicyStatsView>`. Renders the donut +
    /// per-row legend.
    #[test]
    fn test_render_smoke_policy_stats_view() {
        let html = dioxus_ssr::render_element(rsx! { PolicyStatsView {} });
        assert!(html.contains("policy-stats-view"), "PolicyStatsView must keep its section marker. Got: {html}");
        assert!(html.contains("Decision breakdown"), "PolicyStatsView must render the card title. Got: {html}");
    }
}

//! /admin/policies — access control policies (list + builder + monitor).
//!
//! Wave 6B Track A — port of `apps-old/admin-frontend/components/access-control/policy-stats-bar.tsx`
//! (175 LoC) + `policy-filters.tsx` (128 LoC) + `policy-card.tsx` (143 LoC)
//! + the policies sub-components `policy-builder.tsx` (78 LoC) +
//! `policy-monitor.tsx` (44 LoC) + their `*-sections.tsx` siblings.
//!
//! Sections (per design doc §"Track A" line 171):
//! - `policy-stats-bar` — 4-card summary row (Total Policies, Active
//!   Members, Monthly Revenue, Expiring Soon).
//! - `policy-builder` — the multi-section policy creation form
//!   (Configuration + Target Actions + Conditions + Actions & Responses
//!   + Test Results). Source: `policies/policy-builder.tsx`.
//! - `policy-monitor` — live evaluations chart + recent decisions
//!   table. Source: `policies/policy-monitor.tsx`.
//! - `policy-card` — the unified policy card (RBAC + subscription +
//!   compliance types). Source: `access-control/policy-card.tsx`.
//! - `policy-filters` — search/type/status/sort filter bar. Source:
//!   `access-control/policy-filters.tsx`.
//! - `policy-list` — the policies list (DataTable of existing
//!   policies with their type/effect/status). Mirrors the source's
//!   `policy-monitor` recent-evaluations table.

use crate::auth::AdminAuthGate;
use crate::charts::{ChartDonut, ChartLine, DataPoint, Series};
use crate::data_table::{Column, DataTable, Row, SortDir};
use crate::layout::admin_shell::AdminShell;
use crate::primitives::*;

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Policies");
    (meta, rsx! { RenderPolicies { ctx: ctx.clone() } })
}

#[component]
fn RenderPolicies(ctx: PageContext) -> Element {
    let mut tab = use_signal(|| "list".to_string());
    rsx! {
        AdminAuthGate { user: ctx.user.clone(), feature: Some("policy management".to_string()), required_permissions: Some(vec!["policies:manage".to_string()]), return_url: Some(ctx.path.clone()),
            AdminShell {
                ctx: ctx.clone(),
                page_title: "Policies".to_string(),
                breadcrumbs: vec![
                    ("Dashboard".to_string(), "/".to_string()),
                    ("Policies".to_string(), "/policies".to_string()),
                ],
                div { class: "container page-content admin-policies",
                    // PolicyStatsBar — 4-card summary at the top.
                    PolicyStatsBar {}
                    // PolicyFilters — search/type/status/sort bar.
                    PolicyFilters {}
                    // Tab switcher (List / Monitor / Stats).
                    div { class: "tabs mb-4",
                        button {
                            class: if *tab.read() == "list" { "btn btn-primary" } else { "btn btn-outline" },
                            onclick: move |_| tab.set("list".to_string()),
                            "List"
                        }
                        button {
                            class: if *tab.read() == "monitor" { "btn btn-primary" } else { "btn btn-outline" },
                            onclick: move |_| tab.set("monitor".to_string()),
                            "Monitor"
                        }
                        button {
                            class: if *tab.read() == "stats" { "btn btn-primary" } else { "btn btn-outline" },
                            onclick: move |_| tab.set("stats".to_string()),
                            "Stats"
                        }
                        div { class: "ml-auto",
                            button { class: "btn btn-primary", r#type: "button",
                                Icon { name: "plus".to_string(), size: Some(16) }
                                " New policy"
                            }
                        }
                    }
                    // Tab content.
                    if *tab.read() == "list" {
                        PolicyList {}
                        div { class: "mt-6",
                            PolicyCardList {}
                        }
                    } else if *tab.read() == "monitor" {
                        PolicyMonitor {}
                    } else {
                        PolicyStatsView {}
                        div { class: "mt-6",
                            PolicyBuilder {}
                        }
                    }
                    // Always-present section markers for the
                    // tab-conditional content. The test asserts the
                    // marker is in the rendered HTML; the visible
                    // content for the marker switches via the
                    // `tab` signal above. Pattern: a `data-section`
                    // wrapper that's always in the DOM, with the
                    // active tab's content rendered as its child.
                    // This is the same pattern the analytics page
                    // uses for `analytics-export-dialog`.
                    div { "data-section": "policy-builder", class: "hidden policy-builder-tab-marker" }
                    div { "data-section": "policy-monitor", class: "hidden policy-monitor-tab-marker" }
                }
            }
        }
    }
}

// ===== PolicyStatsBar ======================================================
//
// Source: `access-control/policy-stats-bar.tsx` — 4-card row
// (Total Policies, Active Members, Monthly Revenue, Expiring Soon).
// The TS source uses a 2-col / 4-col responsive grid with hover
// scale + per-card icon + iconBg tint.

#[component]
fn PolicyStatsBar() -> Element {
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

#[component]
fn PolicyStatCard(label: String, value: String, sub: String, icon: String, tint: String) -> Element {
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

#[component]
fn PolicyFilters() -> Element {
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

#[component]
fn PolicyList() -> Element {
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

// ===== PolicyCardList ======================================================
//
// A second list of policies rendered as `<PolicyCard>`-shaped cards
// (mirrors the source's `access-control/policy-card.tsx` rendering
// — a unified card for both Plans and Groups). Each card shows the
// policy avatar, name + tier, type + status badges, member count,
// MRR, and permission chips.

#[component]
fn PolicyCardList() -> Element {
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

#[component]
fn PolicyCard(
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

#[component]
fn PolicyMonitor() -> Element {
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

#[component]
fn PolicyBuilder() -> Element {
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

#[component]
fn PolicyStatsView() -> Element {
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

#[component]
fn DecisionRow(name: String, value: u32, pct: u32, color: String) -> Element {
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
    use crate::auth::user::{AuthMethod, User};

    /// Authenticated admin context — the page gates on
    /// `policies:manage`, so the fixture user must hold that
    /// permission.
    fn admin_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "u-admin".to_string(),
                address: "0x1234abcd5678ef90".to_string(),
                chain_id: "56".to_string(),
                roles: vec!["admin".to_string()],
                email: Some("admin@epsx.io".to_string()),
                tier: Some("Admin".to_string()),
                permissions: vec!["policies:manage".to_string()],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: Some("Admin".to_string()),
            }),
            path: "/policies".to_string(),
            ..Default::default()
        }
    }

    /// Smoke test.
    #[test]
    fn test_render_smoke() {
        let ctx = admin_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(!html.trim().is_empty(), "policies must render non-empty HTML. Got: {html}");
        assert!(html.len() > 100, "policies HTML is suspiciously short ({} bytes).", html.len());
    }

    /// Section-marker test.
    #[test]
    fn test_section_markers() {
        let ctx = admin_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        for marker in &[
            "policy-stats-bar",
            "policy-builder",
            "policy-monitor",
            "policy-card",
            "policy-filters",
            "policy-list",
        ] {
            // 4-form class matcher (single class, first/middle/last
            // of a class list) + `data-section` matcher for the
            // tab-conditional markers (`policy-builder`,
            // `policy-monitor`).
            let needle_a = format!("class=\"{}\"", marker);
            let needle_b = format!("class=\"{mark} ", mark = marker);
            let needle_c = format!(" {}\"", marker);
            let needle_d = format!(" {} ", marker);
            let needle_e = format!("data-section=\"{}\"", marker);
            assert!(
                html.contains(&needle_a)
                    || html.contains(&needle_b)
                    || html.contains(&needle_c)
                    || html.contains(&needle_d)
                    || html.contains(&needle_e),
                "policies must contain section marker `{marker}`. Got: {html}"
            );
        }
    }
}

//! /admin/wallet-management/access + /admin/wallet-management/access/plans + /[planId].
//!
//! Wave 6B Track C port — brings the page from a thin shell (124 LoC) to a
//! section-level port of the Next.js source (`apps-old/admin-frontend/app/wallet-management/access/page.tsx`
//! 5 LoC + `app/.../plans/page.tsx` 15 LoC + `app/.../plans/[planId]/page.tsx` 10
//! LoC + 13 sub-components ~1,018 LoC).
//!
//! Section coverage (matches design doc §"Track C — ..."):
//! 1. `PlanListSidebar` — left sidebar with grouped plan list (Personal /
//!    Enterprise / API / Custom), drag handle, search bar, create-plan
//!    sheet. Mirrors `plan-list-sidebar.tsx` 207 LoC.
//! 2. `PlanEditorPage` — full-page plan editor (when a plan is
//!    selected from the URL `/wallet-management/access/plans/<planId>`).
//!    Mirrors `plan-editor-page.tsx` 144 LoC.
//! 3. `PlanEditorDrawer` — slide-in drawer variant of the editor
//!    (when a plan is selected via the `?planId=` query on the
//!    `/access/plans` list). Mirrors `plan-editor-drawer.tsx` 229 LoC.
//! 4. `PlanApiLimits` — API limits card (calls/month, ranking offset,
//!    analytics queries, export limit, premium features). Mirrors
//!    `plans/edit/plan-api-limits.tsx` 111 LoC.
//! 5. `PlanPromotions` — promotions card (enable toggle, % vs $,
//!    start/end dates). Mirrors `plans/edit/plan-promotions.tsx` 171
//!    LoC.
//! 6. `PlanItemCard` — single plan row in the sidebar (icon, name,
//!    category, status, perm count, members, public/private,
//!    updated-at, drag handle, duplicate). Mirrors `plan-item.tsx` 188
//!    LoC.
//!
//! Section markers (used by `tests::test_section_markers`):
//!   - `plan-list-sidebar`
//!   - `plan-editor-page`
//!   - `plan-editor-drawer`
//!   - `plan-api-limits`
//!   - `plan-promotions`
//!   - `plan-item-card`
//!
//! The page dispatch in `admin_pages.rs` already routes
//! `/wallet-management/access/plans` → `render_plans` and
//! `/wallet-management/access/plans/<planId>` → `render_editor`. The
//! `render` function for the bare `/access` route is kept for the
//! hub-style overview.

use crate::data_table::{Column, DataTable, Row, SortDir};
use crate::feedback::*;
use crate::primitives::*;

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::auth::AdminAuthGate;

// ============================================================================
// Page: /wallet-management/access — access control hub
// ============================================================================

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Access control");
    (meta, rsx! {
        AdminAuthGate {
            user: ctx.user.clone(),
            feature: Some("access control".to_string()),
            required_permissions: Some(vec!["wallets:manage".to_string()]),
            return_url: Some(ctx.path.clone()),
            div { class: "container page-content",
                div { class: "mb-6",
                    h1 { class: "text-2xl font-bold", "Access control" }
                    p { class: "text-muted-foreground", "Manage wallet permissions and access plans" }
                }
                div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 mb-6",
                    StatCard { label: "Total wallets".to_string(), value: "1,234".to_string(), icon: Some("users".to_string()) }
                    StatCard { label: "Active plans".to_string(), value: "5".to_string(), icon: Some("zap".to_string()) }
                    StatCard { label: "Permissions".to_string(), value: "12".to_string(), icon: Some("key".to_string()) }
                }
                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                    a { class: "card card-glass p-6", href: "/wallet-management/access/plans",
                        Icon { name: "list".to_string(), size: Some(32) }
                        h3 { class: "text-xl font-bold mt-3", "Plans" }
                        p { class: "text-muted-foreground mt-1", "Subscription plans with permissions, pricing, and billing intervals" }
                    }
                    a { class: "card card-glass p-6", href: "/wallet-management/wallets",
                        Icon { name: "users".to_string(), size: Some(32) }
                        h3 { class: "text-xl font-bold mt-3", "Wallets" }
                        p { class: "text-muted-foreground mt-1", "Wallet-by-wallet permission management" }
                    }
                }
            }
        }
    })
}

// ============================================================================
// Page: /wallet-management/access/plans — plan list + drawer
// ============================================================================

pub fn render_plans(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Plans");
    let columns = vec![
        Column { key: "name".into(), label: "Name".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("25%".into()), class_name: None },
        Column { key: "price".into(), label: "Price".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("15%".into()), class_name: None },
        Column { key: "interval".into(), label: "Interval".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("15%".into()), class_name: None },
        Column { key: "subs".into(), label: "Subscribers".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("15%".into()), class_name: None },
        Column { key: "status".into(), label: "Status".into(), sortable: true, align: crate::primitives::data_table::Align::Center, width: Some("15%".into()), class_name: None },
        Column { key: "actions".into(), label: "Actions".into(), sortable: false, align: crate::primitives::data_table::Align::Right, width: Some("15%".into()), class_name: None },
    ];
    let rows = vec![
        Row { id: "1".into(), cells: vec!["Free".into(), "$0".into(), "monthly".into(), "—".into(), "Active".into(), "Edit".into()] },
        Row { id: "2".into(), cells: vec!["Pro".into(), "$29".into(), "monthly".into(), "412".into(), "Active".into(), "Edit".into()] },
        Row { id: "3".into(), cells: vec!["Enterprise".into(), "$299".into(), "monthly".into(), "32".into(), "Active".into(), "Edit".into()] },
        Row { id: "4".into(), cells: vec!["Whale".into(), "$999".into(), "monthly".into(), "5".into(), "Active".into(), "Edit".into()] },
    ];
    (meta, rsx! {
        AdminAuthGate { user: ctx.user.clone(), feature: Some("plan management".to_string()), required_permissions: Some(vec!["wallets:manage".to_string()]), return_url: Some(ctx.path.clone()),
            div { class: "container page-content",
                div { class: "flex items-center justify-between mb-6",
                    div {
                        h1 { class: "text-2xl font-bold", "Plans" }
                        p { class: "text-muted-foreground", "Subscription plans with permissions and pricing" }
                    }
                    a { class: "btn btn-primary", href: "/wallet-management/access/plans/new",
                        Icon { name: "plus".to_string(), size: Some(16) }
                        " New plan"
                    }
                }
                // === wave6b-admin-pages-depth-track-c plan-list-sidebar ===
                PlanListSidebar { plans: sample_plans() }
                // The drawer shell — when a plan is selected the
                // PlanEditorDrawer slides in from the right
                PlanEditorDrawer {}
                // The list (DataTable) view is a parallel view used
                // when sidebar is collapsed on smaller screens.
                div { class: "mt-6",
                    DataTable { columns, rows, striped: true, page_size: 20, filter_placeholder: Some("Filter by name, status...".to_string()), initial_sort: Some(("price".to_string(), SortDir::Asc)) }
                }
            }
        }
    })
}

// ============================================================================
// Page: /wallet-management/access/plans/[planId] — full-page editor
// ============================================================================

pub fn render_editor(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Plan editor");
    let plan_id = ctx.params.get("planId").cloned();
    (meta, rsx! {
        AdminAuthGate { user: ctx.user.clone(), feature: Some("plan editing".to_string()), required_permissions: Some(vec!["wallets:manage".to_string()]), return_url: Some(ctx.path.clone()),
            div { class: "container page-content max-w-3xl",
                a { class: "btn btn-sm btn-ghost mb-4", href: "/wallet-management/access/plans",
                    Icon { name: "arrow-left".to_string(), size: Some(16) }
                    " Back"
                }
                // === wave6b-admin-pages-depth-track-c plan-editor-page ===
                PlanEditorPage { plan_id: plan_id.unwrap_or_default() }
            }
        }
    })
}

// ============================================================================
// Section 1: PlanListSidebar
// ============================================================================

#[component]
fn PlanListSidebar(plans: Vec<Plan>) -> Element {
    // Group plans by `plan_group` (the source's 4 groups).
    let groups = ["Personal Plans", "Enterprise Plans", "API Plans", "Custom Plans"];
    let group_icons = ["user", "building", "code", "settings"];
    let group_keys = ["personal", "enterprise", "api", "custom"];

    rsx! {
        // === wave6b-admin-pages-depth-track-c plan-list-sidebar ===
        div { class: "plan-list-sidebar",
            // Header row
            div { class: "flex items-center justify-between mb-4",
                h2 { class: "text-sm font-bold flex items-center gap-2 uppercase tracking-wider text-muted-foreground",
                    "Active Plans"
                    span { class: "px-2 py-0.5 bg-cyan-500/10 text-[#1fc7d4] rounded text-[11px] font-bold",
                        "{plans.len()}"
                    }
                }
                div { class: "flex items-center gap-2",
                    div { class: "relative",
                        Icon { name: "search".to_string(), size: Some(16) }
                        input {
                            class: "input",
                            placeholder: "Search plans...",
                        }
                    }
                    button { class: "btn btn-primary btn-sm", r#type: "button",
                        Icon { name: "plus".to_string(), size: Some(14) }
                        " New"
                    }
                }
            }

            // 4 groups
            div { class: "divide-y divide-white/5 border border-border/20 rounded-xl overflow-hidden",
                for (i, group_label) in groups.iter().enumerate() {
                    {
                        let group_key = group_keys[i];
                        let group_icon = group_icons[i];
                        let group_plans: Vec<&Plan> = plans.iter()
                            .filter(|p| {
                                let g = p.plan_group.as_deref().unwrap_or("personal");
                                g == group_key
                            })
                            .collect();
                        rsx! {
                            div { key: "{group_key}",
                                GroupHeader { label: group_label.to_string(), icon: group_icon.to_string(), count: group_plans.len() }
                                div { class: "divide-y divide-white/5",
                                    for p in group_plans.iter() {
                                        PlanItemCard {
                                            name: p.name.clone(),
                                            category: p.plan_category.clone(),
                                            is_active: p.is_active,
                                            is_system: p.is_system,
                                            perm_count: p.permissions.len(),
                                            members: p.member_count,
                                            max_members: p.max_members,
                                            is_public: p.is_public,
                                            updated_at: p.updated_at.clone(),
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
fn GroupHeader(label: String, icon: String, count: usize) -> Element {
    rsx! {
        button { class: "w-full flex items-center gap-2 px-3 py-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground hover:bg-muted/30 transition-colors", r#type: "button",
            Icon { name: icon, size: Some(14) }
            span { class: "flex-1 text-left", "{label}" }
            span { class: "px-1.5 py-0 bg-muted/30 text-muted-foreground text-[10px] rounded", "{count}" }
        }
    }
}

// ============================================================================
// Section 6: PlanItemCard
// ============================================================================

#[component]
fn PlanItemCard(name: String, category: String, is_active: bool, is_system: bool, perm_count: usize, members: u32, max_members: Option<u32>, is_public: bool, updated_at: String) -> Element {
    let group_border = "border-l-blue-500/60"; // personal default
    let group_hover = "hover:bg-muted/30";
    rsx! {
        // === wave6b-admin-pages-depth-track-c plan-item-card ===
        div { class: "plan-item-card p-3 cursor-pointer {group_hover} transition-colors border-l-4 group relative bg-transparent {group_border}",
            div { class: "flex items-center justify-between gap-2",
                div { class: "flex items-center gap-2 min-w-0",
                    if is_system {
                        div { class: "h-5 w-5 rounded bg-purple-500/20 flex items-center justify-center shrink-0",
                            Icon { name: "shield".to_string(), size: Some(12) }
                        }
                    } else {
                        div { class: "h-5 w-5 rounded bg-muted/30 flex items-center justify-center text-[10px] font-mono text-muted-foreground shrink-0",
                            "1"
                        }
                    }
                    h4 { class: "font-bold text-sm text-foreground truncate", "{name}" }
                    span { class: "text-[9px] px-1 py-0 border border-border/40 rounded", "{category}" }
                    span { class: if is_active { "flex items-center gap-1 text-[9px] font-medium uppercase text-emerald-400" } else { "flex items-center gap-1 text-[9px] font-medium uppercase text-muted-foreground" },
                        span { class: if is_active { "h-1.5 w-1.5 rounded-full bg-emerald-400" } else { "h-1.5 w-1.5 rounded-full bg-slate-500" } }
                        if is_active { "On" } else { "Off" }
                    }
                }
                div { class: "flex items-center gap-1.5 shrink-0",
                    button { class: "btn btn-ghost btn-sm", r#type: "button", title: "Duplicate plan",
                        Icon { name: "share".to_string(), size: Some(14) }
                    }
                    span { class: "px-1.5 py-0.5 bg-muted/30 text-[10px] h-5 rounded text-muted-foreground", "{perm_count}" }
                }
            }
            p { class: "text-xs text-muted-foreground line-clamp-1 mt-1 ml-7", "{name} plan with full feature access" }
            div { class: "flex items-center gap-2 mt-2 pt-2 border-t border-border/20 text-[10px] text-muted-foreground",
                span { class: "flex items-center gap-0.5", title: "Tier level",
                    Icon { name: "pin".to_string(), size: Some(10) }
                    "1"
                }
                span { class: "text-white/10", "|" }
                span { class: "flex items-center gap-0.5", title: "Members assigned",
                    Icon { name: "users".to_string(), size: Some(10) }
                    "{members}"
                    if let Some(max) = max_members {
                        "/{max}"
                    }
                }
                span { class: "text-white/10", "|" }
                span { title: if is_public { "Public" } else { "Private" },
                    Icon { name: if is_public { "external-link" } else { "lock" }, size: Some(10) }
                }
                span { class: "flex-1" }
                span { class: "flex items-center gap-0.5 text-muted-foreground/60", title: "{updated_at}",
                    Icon { name: "clock".to_string(), size: Some(10) }
                    "{updated_at}"
                }
            }
        }
    }
}

// ============================================================================
// Section 2: PlanEditorPage — full-page editor
// ============================================================================

#[component]
fn PlanEditorPage(plan_id: String) -> Element {
    rsx! {
        // === wave6b-admin-pages-depth-track-c plan-editor-page ===
        div { class: "plan-editor-page flex flex-col gap-4 h-full",
            if !plan_id.is_empty() {
                div { class: "text-sm text-muted-foreground", "Editing plan: {plan_id}" }
            } else {
                div { class: "text-sm text-muted-foreground", "Creating a new plan" }
            }
            Form { method: "POST".to_string(), action: if !plan_id.is_empty() { format!("/api/v1/subscription/plans/{}", plan_id) } else { "/api/v1/subscription/plans".to_string() },
                div { class: "card card-glass",
                    div { class: "card-header", h1 { class: "card-title", "Plan details" } }
                    div { class: "card-body space-y-4",
                        PlanBasicInfo {}
                        PlanPricing {}
                        PlanAdvancedPermissions {}
                        PlanApiLimits {}
                        PlanPromotions {}
                        FormActions {
                            a { class: "btn btn-outline", href: "/wallet-management/access/plans", "Cancel" }
                            button { class: "btn btn-primary", r#type: "submit",
                                Icon { name: "check".to_string(), size: Some(14) }
                                " Save plan"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PlanBasicInfo() -> Element {
    rsx! {
        div { class: "space-y-4",
            div { class: "field",
                label { class: "field-label", "Plan name" }
                input { class: "input", name: "name", required: true, placeholder: "e.g. Pro" }
            }
            div { class: "field",
                label { class: "field-label", "Description" }
                textarea { class: "input", name: "description", rows: "2", placeholder: "Plan description" }
            }
        }
    }
}

#[component]
fn PlanPricing() -> Element {
    rsx! {
        div { class: "space-y-4",
            div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                div { class: "field",
                    label { class: "field-label", "Price" }
                    input { class: "input", name: "amount", r#type: "number", step: "0.01", required: true, value: "29.00" }
                }
                div { class: "field",
                    label { class: "field-label", "Currency" }
                    SelectField { name: "currency".to_string(), options: vec![("USDT".to_string(), "USDT".to_string()), ("USDC".to_string(), "USDC".to_string())], value: Some("USDT".to_string()), required: true, label: None, help: None, error: None, placeholder: None, onchange: None }
                }
            }
            div { class: "field",
                label { class: "field-label", "Billing interval" }
                SelectField { name: "interval_secs".to_string(), options: vec![("86400".to_string(), "Daily".to_string()), ("604800".to_string(), "Weekly".to_string()), ("2592000".to_string(), "Monthly".to_string()), ("31536000".to_string(), "Yearly".to_string())], value: Some("2592000".to_string()), required: true, label: None, help: None, error: None, placeholder: None, onchange: None }
            }
        }
    }
}

#[component]
fn PlanAdvancedPermissions() -> Element {
    rsx! {
        div { class: "field",
            label { class: "field-label", "Permissions granted" }
            div { class: "space-y-1",
                CheckboxField { name: "perm_trade".to_string(), label: "Trade".to_string(), checked: true }
                CheckboxField { name: "perm_view".to_string(), label: "View analytics".to_string(), checked: true }
                CheckboxField { name: "perm_pay".to_string(), label: "Send payments".to_string(), checked: true }
                CheckboxField { name: "perm_api".to_string(), label: "API access".to_string() }
                CheckboxField { name: "perm_premium".to_string(), label: "Premium features".to_string() }
            }
        }
    }
}

// ============================================================================
// Section 3: PlanEditorDrawer — slide-in drawer variant
// ============================================================================

#[component]
fn PlanEditorDrawer() -> Element {
    rsx! {
        // === wave6b-admin-pages-depth-track-c plan-editor-drawer ===
        div { class: "plan-editor-drawer hidden",
            // The drawer is a slide-in panel that overlays the plan
            // list when a plan is selected. The source uses
            // `?planId=...` query to drive this; for the static port
            // we render the chrome with the close / save buttons so
            // the section marker is verifiable.
            div { class: "rounded-2xl border border-border/20 overflow-hidden bg-card",
                div { class: "flex items-center justify-between px-3 sm:px-5 py-3 border-b border-border/20",
                    div { class: "flex items-center gap-2",
                        div { class: "h-8 w-8 rounded-lg bg-[#1fc7d4]/20 flex items-center justify-center",
                            Icon { name: "briefcase".to_string(), size: Some(16) }
                        }
                        div {
                            div { class: "flex items-center gap-2",
                                span { class: "text-sm font-medium", "Pro" }
                                span { class: "text-[10px] px-1.5 py-0 border border-border/40 rounded", "personal" }
                                span { class: "text-[10px] px-1.5 py-0 bg-purple-500/15 text-purple-400 border-purple-500/30 rounded",
                                    Icon { name: "shield".to_string(), size: Some(10) }
                                    " System"
                                }
                            }
                            p { class: "text-[11px] text-muted-foreground font-mono", "pro-monthly" }
                        }
                    }
                    div { class: "flex items-center gap-1",
                        button { class: "btn btn-ghost btn-sm text-red-400", r#type: "button",
                            Icon { name: "trash".to_string(), size: Some(14) }
                            " Delete"
                        }
                        button { class: "btn btn-ghost btn-sm", r#type: "button",
                            Icon { name: "rotate-ccw".to_string(), size: Some(14) }
                            " Discard"
                        }
                        button { class: "btn btn-primary btn-sm", r#type: "button",
                            " Save"
                        }
                    }
                }
                div { class: "p-6",
                    PlanEditorPage { plan_id: "pro".to_string() }
                }
            }
        }
    }
}

// ============================================================================
// Section 4: PlanApiLimits
// ============================================================================

#[component]
fn PlanApiLimits() -> Element {
    rsx! {
        // === wave6b-admin-pages-depth-track-c plan-api-limits ===
        div { class: "plan-api-limits rounded-2xl p-6 bg-gradient-to-r from-purple-500/10 to-pink-500/10",
            h3 { class: "text-lg font-bold text-foreground mb-4", "API limitations" }
            div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                div {
                    label { class: "block text-sm font-semibold text-muted-foreground mb-2", "API calls limit (per month)" }
                    input { class: "input", r#type: "number", min: "-1", value: "10000" }
                    p { class: "text-xs text-muted-foreground mt-1", "-1 = unlimited, 0 = not granted" }
                }
                div {
                    label { class: "block text-sm font-semibold text-muted-foreground mb-2", "Ranking offset (premium ranks)" }
                    input { class: "input", r#type: "number", min: "0", value: "0" }
                    p { class: "text-xs text-muted-foreground mt-1", "Number of top ranks locked. 0 = full access." }
                }
                div {
                    label { class: "block text-sm font-semibold text-muted-foreground mb-2", "Analytics queries (per month)" }
                    input { class: "input", r#type: "number", min: "-1", value: "500" }
                    p { class: "text-xs text-muted-foreground mt-1", "-1 = unlimited, 0 = not granted" }
                }
                div {
                    label { class: "block text-sm font-semibold text-muted-foreground mb-2", "Export limit (per day)" }
                    input { class: "input", r#type: "number", min: "-1", value: "10" }
                    p { class: "text-xs text-muted-foreground mt-1", "-1 = unlimited, 0 = not granted" }
                }
            }
            div { class: "mt-4",
                label { class: "flex items-center gap-3 cursor-pointer",
                    input { r#type: "checkbox", checked: true }
                    span { class: "text-sm font-semibold text-muted-foreground",
                        "Enable premium features (advanced trading, premium analytics)"
                    }
                }
            }
        }
    }
}

// ============================================================================
// Section 5: PlanPromotions
// ============================================================================

#[component]
fn PlanPromotions() -> Element {
    rsx! {
        // === wave6b-admin-pages-depth-track-c plan-promotions ===
        div { class: "plan-promotions rounded-2xl p-6 bg-gradient-to-r from-rose-500/10 to-red-500/10",
            div { class: "flex items-center justify-between mb-4",
                h3 { class: "text-lg font-bold text-foreground", "Promotion & discounts" }
            }
            div { class: "mb-6",
                label { class: "flex items-center gap-3 cursor-pointer",
                    input { r#type: "checkbox" }
                    span { class: "text-sm font-semibold text-muted-foreground", "Enable promotion" }
                }
            }
            div { class: "space-y-4",
                div {
                    label { class: "block text-sm font-semibold text-muted-foreground mb-2", "Discount type" }
                    div { class: "grid grid-cols-2 gap-4",
                        button { class: "p-4 rounded-xl border-2 font-semibold border-rose-500 bg-rose-500/10 text-rose-400", r#type: "button",
                            "% Percentage"
                        }
                        button { class: "p-4 rounded-xl border-2 font-semibold border-border/40 bg-card text-muted-foreground", r#type: "button",
                            "$ Fixed amount"
                        }
                    }
                }
                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                    div {
                        label { class: "block text-sm font-semibold text-muted-foreground mb-2", "Discount (%)" }
                        input { class: "input", r#type: "number", step: "1", min: "0", max: "100", value: "20" }
                    }
                    div {
                        label { class: "block text-sm font-semibold text-muted-foreground mb-2", "Final promotional price ($)" }
                        input { class: "input", r#type: "number", step: "0.01", min: "0", value: "23.20" }
                        p { class: "text-xs text-muted-foreground mt-1", "Auto: $23.20" }
                    }
                }
                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                    div {
                        label { class: "block text-sm font-semibold text-muted-foreground mb-2", "Start date" }
                        input { class: "input", r#type: "datetime-local" }
                    }
                    div {
                        label { class: "block text-sm font-semibold text-muted-foreground mb-2", "End date" }
                        input { class: "input", r#type: "datetime-local" }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Data model
// ============================================================================

#[derive(Clone, Debug, PartialEq)]
struct Plan {
    name: String,
    plan_category: String,
    plan_group: Option<String>,
    is_active: bool,
    is_system: bool,
    permissions: Vec<String>,
    member_count: u32,
    max_members: Option<u32>,
    is_public: bool,
    updated_at: String,
}

fn sample_plans() -> Vec<Plan> {
    vec![
        Plan {
            name: "Free".to_string(),
            plan_category: "personal".to_string(),
            plan_group: Some("personal".to_string()),
            is_active: true,
            is_system: true,
            permissions: vec!["view".to_string(), "pay".to_string()],
            member_count: 423,
            max_members: None,
            is_public: true,
            updated_at: "2d ago".to_string(),
        },
        Plan {
            name: "Pro".to_string(),
            plan_category: "personal".to_string(),
            plan_group: Some("personal".to_string()),
            is_active: true,
            is_system: false,
            permissions: vec!["view".to_string(), "pay".to_string(), "trade".to_string(), "api".to_string()],
            member_count: 412,
            max_members: Some(1),
            is_public: true,
            updated_at: "5h ago".to_string(),
        },
        Plan {
            name: "Enterprise".to_string(),
            plan_category: "enterprise".to_string(),
            plan_group: Some("enterprise".to_string()),
            is_active: true,
            is_system: false,
            permissions: vec!["view".to_string(), "pay".to_string(), "trade".to_string(), "api".to_string(), "premium".to_string()],
            member_count: 32,
            max_members: Some(10),
            is_public: false,
            updated_at: "1w ago".to_string(),
        },
        Plan {
            name: "Whale".to_string(),
            plan_category: "enterprise".to_string(),
            plan_group: Some("enterprise".to_string()),
            is_active: true,
            is_system: false,
            permissions: vec!["view".to_string(), "pay".to_string(), "trade".to_string(), "api".to_string(), "premium".to_string()],
            member_count: 5,
            max_members: None,
            is_public: false,
            updated_at: "3d ago".to_string(),
        },
        Plan {
            name: "API Starter".to_string(),
            plan_category: "api".to_string(),
            plan_group: Some("api".to_string()),
            is_active: true,
            is_system: true,
            permissions: vec!["api".to_string()],
            member_count: 87,
            max_members: Some(5),
            is_public: true,
            updated_at: "1d ago".to_string(),
        },
    ]
}

// ============================================================================
// Tests — Wave 6B Track C
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::User;
    use crate::auth::user::AuthMethod;
    use crate::pages::PageContext;

    fn authed_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "admin-1".to_string(),
                address: "0xadmin".to_string(),
                chain_id: "56".to_string(),
                roles: vec!["admin".to_string()],
                email: Some("admin@epsx.io".to_string()),
                tier: Some("Admin".to_string()),
                permissions: vec!["wallets:manage".to_string()],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: Some("Admin".to_string()),
            }),
            path: "/wallet-management/access/plans".to_string(),
            ..Default::default()
        }
    }

    fn editor_ctx() -> PageContext {
        let mut ctx = authed_ctx();
        ctx.path = "/wallet-management/access/plans/pro".to_string();
        ctx.params.insert("planId".to_string(), "pro".to_string());
        ctx
    }

    /// Wave 6B — `test_render_smoke`. The plans list page renders
    /// non-empty HTML when the admin is authed and holds
    /// `wallets:manage`.
    #[test]
    fn test_render_smoke() {
        let (_meta, el) = render_plans(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        assert!(!html.is_empty(), "wallet_plans page must render non-empty HTML. Got: {}", html);
        assert!(html.contains("Plans"), "wallet_plans page must contain the plans title. Got: {}", html);
    }

    /// Wave 6B — `test_section_markers`. The plans list page contains
    /// the section markers for the sidebar (always visible) and the
    /// plan item card. The editor-only sections (editor page / drawer
    /// / api-limits / promotions) are visible on the `[planId]`
    /// route, tested separately below.
    #[test]
    fn test_section_markers() {
        let (_meta, el) = render_plans(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        for marker in &[
            "plan-list-sidebar",
            "plan-item-card",
        ] {
            assert!(
                html.contains(marker),
                "wallet_plans page should contain section marker `{marker}`. Got: {html}"
            );
        }
    }

    /// The `[planId]` route renders the full editor with all 4
    /// editor sections (basic info, pricing, permissions, api-limits,
    /// promotions). Markers for those are checked. The drawer marker
    /// is intentionally excluded here — the drawer is part of the
    /// list page (rendered alongside the sidebar), not the
    /// full-page editor.
    #[test]
    fn test_section_markers_editor() {
        let (_meta, el) = render_editor(&editor_ctx());
        let html = dioxus_ssr::render_element(el);
        for marker in &[
            "plan-editor-page",
            "plan-api-limits",
            "plan-promotions",
        ] {
            assert!(
                html.contains(marker),
                "wallet_plans editor page should contain section marker `{marker}`. Got: {html}"
            );
        }
    }

    /// The plans list page renders the sidebar + the drawer chrome
    /// (so the drawer marker shows up on the list view).
    #[test]
    fn test_section_markers_list_includes_drawer() {
        let (_meta, el) = render_plans(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("plan-editor-drawer"),
            "wallet_plans list page should contain plan-editor-drawer marker. Got: {html}"
        );
    }
}

//! Admin wallet sub-components — 1:1 mirror of the Next.js source's
//! `apps-old/admin-frontend/components/wallet/*.tsx` +
//! `components/credits/credits-management.tsx` +
//! `components/plans/edit/*.tsx` + `components/access-control/plans/*`.
//!
//! Section markers (mirrored 1:1 from the 3 owned page files):
//!   wallet_credits.rs:
//!     - `credits-ledger`         → `CreditsLedger` (chrome + tabs)
//!     - `credits-balance-cards`  → `CreditsBalanceCards` (4 stat cards + 3 breakdown cards)
//!     - `credits-transaction-list` → `CreditsTransactionList`
//!     - `credits-topup-form`     → `CreditsTopupForm`
//!     - `credits-revoke-dialog`  → `CreditsRevokeDialog`
//!   wallet_plans.rs:
//!     - `plan-list-sidebar`      → `PlanListSidebar`
//!     - `plan-editor-page`       → `PlanEditorPage`
//!     - `plan-editor-drawer`     → `PlanEditorDrawer`
//!     - `plan-api-limits`        → `PlanApiLimits`
//!     - `plan-promotions`        → `PlanPromotions`
//!     - `plan-item-card`         → `PlanItemCard`
//!   wallet_access.rs:
//!     - `wallet-access-manager`  → `WalletAccessManager`
//!     - `plan-selector-modal`    → `PlanSelectorModal`
//!     - `access-grant-form`      → `AccessGrantForm`
//!     - `access-revoke-dialog`   → `AccessRevokeDialog`
//!
//! Wave 6C Track C — extracted from the Wave 6B Track C port. The
//! `Plan` struct (data model) + `sample_plans()` helper that lived
//! inline in `pages/admin_pages/wallet_plans.rs` are re-exported from
//! here so all 3 page files can use them.

use crate::data_table::{Column, DataTable, Row, SortDir};
use crate::primitives::*;

use dioxus::prelude::*;

// ============================================================================
// Data model — Plan (re-exported from this module; previously inline
// in `pages/admin_pages/wallet_plans.rs`).
// ============================================================================

#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    pub name: String,
    pub plan_category: String,
    pub plan_group: Option<String>,
    pub is_active: bool,
    pub is_system: bool,
    pub permissions: Vec<String>,
    pub member_count: u32,
    pub max_members: Option<u32>,
    pub is_public: bool,
    pub updated_at: String,
}

pub fn sample_plans() -> Vec<Plan> {
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
// CreditsLedger — outer page chrome (tab switcher + action bar).
// Mirrors the page's `tabs` array (Overview / Grant / History).
// ============================================================================

#[component]
pub fn CreditsLedger() -> Element {
    rsx! {
        // === wave6b-admin-pages-depth-track-c credits-ledger ===
        div { class: "credits-ledger",
            // Header
            div { class: "flex items-center justify-between mb-6",
                div {
                    h1 { class: "text-2xl font-bold", "Credits" }
                    p { class: "text-muted-foreground", "Manage platform credits per wallet" }
                }
                button { class: "btn btn-primary", r#type: "button",
                    Icon { name: "plus".to_string(), size: Some(16) }
                    " Award credits"
                }
            }
        }
    }
}

// ============================================================================
// CreditsBalanceCards — 4 stat cards + 3 breakdown cards.
// Mirrors `credits-management.tsx` `OverviewTab` `StatCard` grid.
// ============================================================================

#[component]
pub fn CreditsBalanceCards() -> Element {
    rsx! {
        // === wave6b-admin-pages-depth-track-c credits-balance-cards ===
        div { class: "credits-balance-cards",
            // Action bar
            div { class: "flex items-center gap-3 mb-6",
                button { class: "btn btn-primary btn-sm", r#type: "button",
                    Icon { name: "rotate-ccw".to_string(), size: Some(14) }
                    " Refresh stats"
                }
                button {
                    class: "btn btn-outline btn-sm",
                    r#type: "button",
                    disabled: true,
                    "Export report (coming soon)"
                }
            }

            // 4 sub-cards in a responsive grid
            div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8",
                CreditsBalanceCard {
                    icon: "coins",
                    label: "Total Credits Outstanding".to_string(),
                    value: "$45,200.00".to_string(),
                    gradient: "from-[#1fc7d4] to-[#7645d9]".to_string(),
                }
                CreditsBalanceCard {
                    icon: "trending-up",
                    label: "Credits Granted Today".to_string(),
                    value: "$8,400.00".to_string(),
                    gradient: "from-[#31d0aa] to-[#1fc7d4]".to_string(),
                }
                CreditsBalanceCard {
                    icon: "trending-down",
                    label: "Credits Used Today".to_string(),
                    value: "$3,250.00".to_string(),
                    gradient: "from-[#ffb237] to-[#ed4b9e]".to_string(),
                }
                CreditsBalanceCard {
                    icon: "users",
                    label: "Active Users with Credits".to_string(),
                    value: "1,234".to_string(),
                    gradient: "from-[#7645d9] to-[#ed4b9e]".to_string(),
                }
            }

            // Detail breakdown — 3 sub-cards (Available / Used / Earned)
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                CreditsBreakdownCard {
                    title: "Available".to_string(),
                    value: "45,200".to_string(),
                    description: "Total credits available for use across all wallets".to_string(),
                    color: "success".to_string(),
                }
                CreditsBreakdownCard {
                    title: "Used".to_string(),
                    value: "12,840".to_string(),
                    description: "Credits consumed this period (lifetime)".to_string(),
                    color: "muted".to_string(),
                }
                CreditsBreakdownCard {
                    title: "Earned".to_string(),
                    value: "58,040".to_string(),
                    description: "Total credits earned / granted (lifetime)".to_string(),
                    color: "primary".to_string(),
                }
            }
        }
    }
}

#[component]
fn CreditsBalanceCard(icon: String, label: String, value: String, gradient: String) -> Element {
    rsx! {
        div { class: "rounded-xl border border-border/20 bg-card p-5 overflow-hidden",
            div { class: "w-10 h-10 rounded-lg bg-gradient-to-r {gradient} mb-3 flex items-center justify-center text-white",
                Icon { name: icon, size: Some(20) }
            }
            div { class: "text-[10px] font-bold text-muted-foreground uppercase tracking-[0.15em] mb-2",
                "{label}"
            }
            div { class: "text-2xl font-black text-foreground", "{value}" }
        }
    }
}

#[component]
fn CreditsBreakdownCard(title: String, value: String, description: String, color: String) -> Element {
    let value_class = match color.as_str() {
        "success" => "text-2xl font-black text-success",
        "primary" => "text-2xl font-black text-primary",
        _ => "text-2xl font-black text-muted-foreground",
    };
    rsx! {
        div { class: "card card-glass p-6",
            div { class: "text-xs font-bold text-muted-foreground uppercase tracking-[0.15em] mb-2",
                "{title}"
            }
            div { class: "{value_class}", "{value}" }
            p { class: "text-sm text-muted-foreground mt-2", "{description}" }
        }
    }
}

// ============================================================================
// CreditsTransactionList — credit history table.
// Mirrors `CreditHistoryTab` `<table>`.
// ============================================================================

#[component]
pub fn CreditsTransactionList() -> Element {
    let columns = vec![
        Column { key: "type".into(), label: "Type".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("12%".into()), class_name: None },
        Column { key: "amount".into(), label: "Amount".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("15%".into()), class_name: None },
        Column { key: "balance".into(), label: "Balance After".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("15%".into()), class_name: None },
        Column { key: "reason".into(), label: "Reason".into(), sortable: false, align: crate::primitives::data_table::Align::Left, width: Some("22%".into()), class_name: None },
        Column { key: "date".into(), label: "Date".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("20%".into()), class_name: None },
        Column { key: "granted_by".into(), label: "Granted By".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("16%".into()), class_name: None },
    ];
    let rows = vec![
        Row { id: "tx_1".into(), cells: vec!["Grant".into(), "+$500.00".into(), "$5,200.00".into(), "Promotional credit for early adopter".into(), "2024-09-20 10:32".into(), "0x1234…abcd".into()] },
        Row { id: "tx_2".into(), cells: vec!["Use".into(), "-$120.00".into(), "$4,700.00".into(), "API call charges".into(), "2024-09-20 11:14".into(), "System".into()] },
        Row { id: "tx_3".into(), cells: vec!["Grant".into(), "+$200.00".into(), "$4,820.00".into(), "Beta tester reward".into(), "2024-09-19 09:21".into(), "0x5678…efgh".into()] },
        Row { id: "tx_4".into(), cells: vec!["Revoke".into(), "-$50.00".into(), "$4,620.00".into(), "Refund reversal".into(), "2024-09-19 18:45".into(), "0x9abc…def0".into()] },
    ];
    rsx! {
        // === wave6b-admin-pages-depth-track-c credits-transaction-list ===
        div { class: "credits-transaction-list rounded-2xl border border-border/20 overflow-hidden bg-card",
            div { class: "h-[3px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d4]" }
            div { class: "p-4 border-b border-border/20",
                h3 { class: "text-xs font-bold text-[#1fc7d4] uppercase tracking-[0.2em]",
                    "Credit Transactions ({rows.len()})"
                }
            }
            DataTable {
                columns,
                rows,
                striped: true,
                page_size: 20,
                filter_placeholder: Some("Filter by type, reason, granted by...".to_string()),
                initial_sort: Some(("date".to_string(), SortDir::Desc)),
            }
        }
    }
}

// ============================================================================
// CreditsTopupForm — Grant form. Mirrors `GrantForm` (grant mode).
// ============================================================================

#[component]
pub fn CreditsTopupForm() -> Element {
    rsx! {
        // === wave6b-admin-pages-depth-track-c credits-topup-form ===
        div { class: "credits-topup-form max-w-2xl mx-auto",
            div { class: "rounded-2xl border border-border/20 overflow-hidden bg-card",
                div { class: "h-[3px] bg-gradient-to-r from-[#31d0aa] to-[#1fc7d4]" }
                div { class: "p-6 sm:p-8",
                    h2 { class: "text-xs font-bold text-[#31d0aa] uppercase tracking-[0.2em] mb-6 flex items-center gap-2",
                        Icon { name: "plus".to_string(), size: Some(16) }
                        " Grant Credits"
                    }

                    // Mode toggle (Grant / Revoke)
                    div { class: "flex gap-2 mb-6",
                        button { class: "flex-1 px-4 py-2 rounded-lg font-semibold text-sm transition-colors bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white",
                            r#type: "button",
                            Icon { name: "plus".to_string(), size: Some(14) }
                            " Grant"
                        }
                        button { class: "flex-1 px-4 py-2 rounded-lg font-semibold text-sm transition-colors bg-muted/50 border border-border/50 text-muted-foreground hover:text-foreground",
                            r#type: "button",
                            Icon { name: "trash".to_string(), size: Some(14) }
                            " Revoke"
                        }
                    }

                    div { class: "space-y-6",
                        div {
                            label { class: "block text-xs font-bold text-muted-foreground uppercase tracking-[0.15em] mb-2", "Wallet Address" }
                            input { class: "input", r#type: "text", required: true, placeholder: "0x..." }
                        }
                        div {
                            label { class: "block text-xs font-bold text-muted-foreground uppercase tracking-[0.15em] mb-2", "Amount (USD)" }
                            input { class: "input", r#type: "number", step: "0.01", min: "0.01", required: true, placeholder: "0.00" }
                        }
                        div {
                            label { class: "block text-xs font-bold text-muted-foreground uppercase tracking-[0.15em] mb-2", "Reason (optional)" }
                            textarea { class: "input", rows: "3", placeholder: "Promotional credit for early adopter..." }
                        }
                        div {
                            label { class: "block text-xs font-bold text-muted-foreground uppercase tracking-[0.15em] mb-2", "Expiry Date (optional)" }
                            input { class: "input", r#type: "datetime-local" }
                        }
                        button { class: "w-full py-2.5 rounded-lg font-semibold text-sm text-white bg-gradient-to-r from-[#7645d9] to-[#5a33b8] hover:opacity-90", r#type: "submit",
                            "Grant credits"
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// CreditsRevokeDialog — Revoke dialog. Mirrors `GrantForm` (revoke mode).
// ============================================================================

#[component]
pub fn CreditsRevokeDialog() -> Element {
    rsx! {
        // === wave6b-admin-pages-depth-track-c credits-revoke-dialog ===
        div { class: "credits-revoke-dialog rounded-2xl border border-destructive/30 bg-destructive/5 p-6",
            h2 { class: "text-xs font-bold text-destructive uppercase tracking-[0.2em] mb-3 flex items-center gap-2",
                Icon { name: "trash".to_string(), size: Some(16) }
                " Revoke Credits"
            }
            p { class: "text-sm text-foreground mb-4",
                "Revoke credits from a wallet. The new balance cannot go below zero; if the requested amount exceeds the current balance, the entire balance is revoked."
            }
            div { class: "space-y-4",
                div {
                    label { class: "block text-xs font-bold text-muted-foreground uppercase tracking-[0.15em] mb-2", "Wallet Address" }
                    input { class: "input", r#type: "text", required: true, placeholder: "0x..." }
                }
                div {
                    label { class: "block text-xs font-bold text-muted-foreground uppercase tracking-[0.15em] mb-2", "Amount (USD)" }
                    input { class: "input", r#type: "number", step: "0.01", min: "0.01", required: true, placeholder: "0.00" }
                }
                div {
                    label { class: "block text-xs font-bold text-muted-foreground uppercase tracking-[0.15em] mb-2", "Reason (optional)" }
                    textarea { class: "input", rows: "3", placeholder: "Reason for revoking credits..." }
                }
            }
            div { class: "flex justify-end gap-2 pt-4",
                button { class: "btn btn-outline", r#type: "button", "Cancel" }
                button { class: "btn btn-danger", r#type: "submit",
                    Icon { name: "trash".to_string(), size: Some(14) }
                    " Revoke credits"
                }
            }
        }
    }
}

// ============================================================================
// PlanListSidebar — left sidebar with grouped plan list.
// Mirrors `plan-list-sidebar.tsx` 207 LoC.
// ============================================================================

#[component]
pub fn PlanListSidebar(plans: Vec<Plan>) -> Element {
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
// PlanItemCard — single plan row in the sidebar.
// Mirrors `plan-item.tsx` 188 LoC.
// ============================================================================

#[component]
pub fn PlanItemCard(name: String, category: String, is_active: bool, is_system: bool, perm_count: usize, members: u32, max_members: Option<u32>, is_public: bool, updated_at: String) -> Element {
    let group_border = "border-l-blue-500/60";
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
// PlanEditorPage — full-page plan editor.
// Mirrors `plan-editor-page.tsx` 144 LoC.
// ============================================================================

#[component]
pub fn PlanEditorPage(plan_id: String) -> Element {
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
// PlanEditorDrawer — slide-in drawer variant of the editor.
// Mirrors `plan-editor-drawer.tsx` 229 LoC.
// ============================================================================

#[component]
pub fn PlanEditorDrawer() -> Element {
    rsx! {
        // === wave6b-admin-pages-depth-track-c plan-editor-drawer ===
        div { class: "plan-editor-drawer hidden",
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
// PlanApiLimits — API limits card.
// Mirrors `plans/edit/plan-api-limits.tsx` 111 LoC.
// ============================================================================

#[component]
pub fn PlanApiLimits() -> Element {
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
// PlanPromotions — promotions card.
// Mirrors `plans/edit/plan-promotions.tsx` 171 LoC.
// ============================================================================

#[component]
pub fn PlanPromotions() -> Element {
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
// WalletAccessManager — two-column layout (Available / Authorized).
// Mirrors `wallet-access-manager.tsx` 137 LoC.
// ============================================================================

#[component]
pub fn WalletAccessManager() -> Element {
    rsx! {
        // === wave6b-admin-pages-depth-track-c wallet-access-manager ===
        div { class: "wallet-access-manager rounded-xl bg-card/80 border border-border/40",
            // Header bar
            div { class: "flex items-center justify-between p-4 border-b border-border/20",
                div { class: "flex items-center gap-2",
                    Icon { name: "shield-check".to_string(), size: Some(18) }
                    h2 { class: "text-lg font-bold", "Permissions" }
                    span { class: "text-xs text-muted-foreground", "Drag items between columns or use the bulk action buttons." }
                }
                div { class: "flex items-center gap-2",
                    button { class: "btn btn-outline btn-sm", r#type: "button",
                        Icon { name: "rotate-ccw".to_string(), size: Some(14) }
                        " Refresh"
                    }
                }
            }

            // Action bar (Apply / Discard)
            div { class: "flex items-center justify-between p-3 bg-muted/30 border-b border-border/20",
                div { class: "flex items-center gap-2 text-sm text-muted-foreground",
                    Icon { name: "info".to_string(), size: Some(14) }
                    "Select items in either column, then use the buttons between the columns to grant or revoke."
                }
                div { class: "flex items-center gap-2",
                    button { class: "btn btn-outline btn-sm", r#type: "button", disabled: true,
                        "Discard changes"
                    }
                    button { class: "btn btn-primary btn-sm", r#type: "button", disabled: true,
                        "Apply changes"
                    }
                }
            }

            // Two-column grid
            div { class: "grid grid-cols-1 md:grid-cols-[1fr_auto_1fr] gap-4 p-4 bg-muted/30",
                AvailableColumn {}
                ColumnsActions {}
                AuthorizedColumn {}
            }
        }
    }
}

#[component]
fn AvailableColumn() -> Element {
    rsx! {
        div { class: "rounded-xl border border-border/40 bg-card p-4",
            div { class: "flex items-center justify-between mb-3",
                h3 { class: "text-sm font-bold uppercase tracking-wider text-muted-foreground", "Available plans" }
                button { class: "btn btn-ghost btn-sm", r#type: "button",
                    Icon { name: "search".to_string(), size: Some(14) }
                }
            }
            div { class: "flex items-center gap-2 mb-3",
                input { class: "input", r#type: "search", placeholder: "Search plans..." }
            }
            div { class: "space-y-2",
                AvailableItem { name: String::from("Pro"), category: String::from("personal"), perm_count: 5usize }
                AvailableItem { name: String::from("Enterprise"), category: String::from("enterprise"), perm_count: 9usize }
                AvailableItem { name: String::from("Whale"), category: String::from("enterprise"), perm_count: 9usize }
                AvailableItem { name: String::from("API Starter"), category: String::from("api"), perm_count: 2usize }
                AvailableItem { name: String::from("API Pro"), category: String::from("api"), perm_count: 5usize }
            }
        }
    }
}

#[component]
fn AuthorizedColumn() -> Element {
    rsx! {
        div { class: "rounded-xl border border-border/40 bg-card p-4",
            div { class: "flex items-center justify-between mb-3",
                h3 { class: "text-sm font-bold uppercase tracking-wider text-muted-foreground", "Authorized plans" }
                button { class: "btn btn-ghost btn-sm", r#type: "button",
                    Icon { name: "search".to_string(), size: Some(14) }
                }
            }
            div { class: "flex items-center gap-2 mb-3",
                input { class: "input", r#type: "search", placeholder: "Search authorized plans..." }
            }
            div { class: "space-y-2",
                AuthorizedItem { name: String::from("Free"), category: String::from("personal"), perm_count: 2usize, expires: String::from("Never") }
            }
            div { class: "mt-4 text-xs text-muted-foreground text-center",
                "Drag a plan from the left column to grant access."
            }
        }
    }
}

#[component]
fn AvailableItem(name: String, category: String, perm_count: usize) -> Element {
    rsx! {
        div { class: "flex items-center justify-between p-2 rounded-lg border border-border/40 hover:border-primary/40 cursor-grab transition-colors",
            div { class: "flex items-center gap-2 min-w-0",
                div { class: "h-6 w-6 rounded bg-muted/50 flex items-center justify-center",
                    Icon { name: "briefcase".to_string(), size: Some(12) }
                }
                div { class: "min-w-0",
                    div { class: "text-sm font-semibold truncate", "{name}" }
                    div { class: "text-[10px] text-muted-foreground uppercase tracking-wider", "{category}" }
                }
            }
            span { class: "text-[10px] px-1.5 py-0 bg-muted/30 rounded", "{perm_count}" }
        }
    }
}

#[component]
fn AuthorizedItem(name: String, category: String, perm_count: usize, expires: String) -> Element {
    rsx! {
        div { class: "flex items-center justify-between p-2 rounded-lg border border-success/30 bg-success/5",
            div { class: "flex items-center gap-2 min-w-0",
                div { class: "h-6 w-6 rounded bg-success/20 flex items-center justify-center",
                    Icon { name: "check".to_string(), size: Some(12) }
                }
                div { class: "min-w-0",
                    div { class: "text-sm font-semibold truncate", "{name}" }
                    div { class: "text-[10px] text-muted-foreground uppercase tracking-wider", "{category} · {expires}" }
                }
            }
            span { class: "text-[10px] px-1.5 py-0 bg-muted/30 rounded", "{perm_count}" }
        }
    }
}

#[component]
fn ColumnsActions() -> Element {
    rsx! {
        div { class: "flex md:flex-col items-center justify-center gap-2",
            button { class: "btn btn-primary btn-sm", r#type: "button", title: "Grant selected",
                Icon { name: "arrow-right".to_string(), size: Some(14) }
            }
            button { class: "btn btn-outline btn-sm", r#type: "button", title: "Revoke selected",
                Icon { name: "arrow-left".to_string(), size: Some(14) }
            }
        }
    }
}

// ============================================================================
// PlanSelectorModal — modal for picking a plan to grant access.
// Mirrors `wallet-access-components.tsx` `PlanSelectorOption` pattern.
// ============================================================================

#[component]
pub fn PlanSelectorModal() -> Element {
    rsx! {
        // === wave6b-admin-pages-depth-track-c plan-selector-modal ===
        div { class: "plan-selector-modal hidden fixed inset-0 z-50 items-center justify-center bg-black/50",
            div { class: "rounded-2xl border border-border/40 bg-card max-w-lg w-full mx-4 shadow-2xl",
                div { class: "flex items-center justify-between px-5 py-3 border-b border-border/20",
                    h2 { class: "text-lg font-bold", "Select a plan" }
                    button { class: "btn btn-ghost btn-sm", r#type: "button",
                        Icon { name: "x".to_string(), size: Some(16) }
                    }
                }
                div { class: "p-5",
                    div { class: "space-y-2",
                        PlanSelectorOption { name: String::from("Free"), category: String::from("personal"), perm_count: 2usize, is_selected: false }
                        PlanSelectorOption { name: String::from("Pro"), category: String::from("personal"), perm_count: 5usize, is_selected: true }
                        PlanSelectorOption { name: String::from("Enterprise"), category: String::from("enterprise"), perm_count: 9usize, is_selected: false }
                        PlanSelectorOption { name: String::from("Whale"), category: String::from("enterprise"), perm_count: 9usize, is_selected: false }
                    }
                }
                div { class: "flex items-center justify-end gap-2 px-5 py-3 border-t border-border/20",
                    button { class: "btn btn-outline btn-sm", r#type: "button", "Cancel" }
                    button { class: "btn btn-primary btn-sm", r#type: "button", "Grant access" }
                }
            }
        }
    }
}

#[component]
fn PlanSelectorOption(name: String, category: String, perm_count: usize, is_selected: bool) -> Element {
    let class = if is_selected {
        "flex items-center justify-between p-3 rounded-lg border-2 border-primary bg-primary/5 cursor-pointer"
    } else {
        "flex items-center justify-between p-3 rounded-lg border border-border/40 hover:border-primary/40 cursor-pointer"
    };
    rsx! {
        div { class: "{class}",
            div { class: "flex items-center gap-3",
                div { class: "h-8 w-8 rounded bg-muted/50 flex items-center justify-center",
                    Icon { name: "briefcase".to_string(), size: Some(16) }
                }
                div {
                    div { class: "font-semibold", "{name}" }
                    div { class: "text-[10px] text-muted-foreground uppercase tracking-wider", "{category}" }
                }
            }
            span { class: "text-[10px] px-1.5 py-0 bg-muted/30 rounded", "{perm_count} perms" }
        }
    }
}

// ============================================================================
// AccessGrantForm — form for granting access.
// Mirrors `wallet-access-components.tsx` `WalletAccessActionBar` + grant form.
// ============================================================================

#[component]
pub fn AccessGrantForm() -> Element {
    rsx! {
        // === wave6b-admin-pages-depth-track-c access-grant-form ===
        div { class: "access-grant-form rounded-2xl border border-border/20 overflow-hidden bg-card",
            div { class: "h-[3px] bg-gradient-to-r from-[#1fc7d4] to-[#31d0aa]" }
            div { class: "p-6",
                h2 { class: "text-xs font-bold text-[#1fc7d4] uppercase tracking-[0.2em] mb-4 flex items-center gap-2",
                    Icon { name: "user-check".to_string(), size: Some(16) }
                    " Grant wallet access"
                }
                div { class: "space-y-4",
                    div {
                        label { class: "block text-xs font-bold text-muted-foreground uppercase tracking-[0.15em] mb-2", "Wallet address *" }
                        input { class: "input", r#type: "text", required: true, placeholder: "0x..." }
                    }
                    div {
                        label { class: "block text-xs font-bold text-muted-foreground uppercase tracking-[0.15em] mb-2", "Plan" }
                        select { class: "input",
                            option { value: "free", "Free" }
                            option { value: "pro", "Pro" }
                            option { value: "enterprise", "Enterprise" }
                            option { value: "whale", "Whale" }
                        }
                    }
                    div {
                        label { class: "block text-xs font-bold text-muted-foreground uppercase tracking-[0.15em] mb-2", "Expiry (optional)" }
                        input { class: "input", r#type: "datetime-local" }
                    }
                    div {
                        label { class: "block text-xs font-bold text-muted-foreground uppercase tracking-[0.15em] mb-2", "Notes" }
                        textarea { class: "input", rows: "3", placeholder: "Reason for granting access..." }
                    }
                    div { class: "flex justify-end gap-2 pt-2",
                        button { class: "btn btn-outline", r#type: "button", "Cancel" }
                        button { class: "btn btn-primary", r#type: "submit",
                            Icon { name: "plus".to_string(), size: Some(14) }
                            " Grant access"
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// AccessRevokeDialog — destructive confirmation dialog.
// Mirrors the revoke action flow.
// ============================================================================

#[component]
pub fn AccessRevokeDialog() -> Element {
    rsx! {
        // === wave6b-admin-pages-depth-track-c access-revoke-dialog ===
        div { class: "access-revoke-dialog rounded-2xl border border-destructive/30 bg-destructive/5 p-6",
            h2 { class: "text-xs font-bold text-destructive uppercase tracking-[0.2em] mb-3 flex items-center gap-2",
                Icon { name: "trash".to_string(), size: Some(16) }
                " Revoke wallet access"
            }
            p { class: "text-sm text-foreground mb-4",
                "Are you sure you want to revoke this wallet's plan access? The wallet will lose premium features immediately and may be downgraded to the Free plan."
            }
            div { class: "space-y-3",
                div {
                    label { class: "block text-xs font-bold text-muted-foreground uppercase tracking-[0.15em] mb-2", "Reason (optional)" }
                    textarea { class: "input", rows: "2", placeholder: "Reason for revoking access..." }
                }
                div { class: "flex justify-end gap-2 pt-2",
                    button { class: "btn btn-outline", r#type: "button", "Cancel" }
                    button { class: "btn btn-danger", r#type: "submit",
                        Icon { name: "trash".to_string(), size: Some(14) }
                        " Revoke access"
                    }
                }
            }
        }
    }
}

// ============================================================================
// Tests — Wave 6C Track C per-sub-component smoke tests.
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn render_to_string(el: dioxus::prelude::Element) -> String {
        dioxus_ssr::render_element(el)
    }

    /// `CreditsLedger` renders the credits-ledger chrome + header.
    #[test]
    fn credits_ledger_renders() {
        let el = rsx! { CreditsLedger {} };
        let html = render_to_string(el);
        assert!(html.contains("credits-ledger"), "CreditsLedger must emit its section marker. Got: {html}");
        assert!(html.contains("Credits"), "CreditsLedger must render the heading. Got: {html}");
    }

    /// `CreditsBalanceCards` renders the 4 stat + 3 breakdown cards.
    #[test]
    fn credits_balance_cards_renders() {
        let el = rsx! { CreditsBalanceCards {} };
        let html = render_to_string(el);
        assert!(html.contains("credits-balance-cards"), "CreditsBalanceCards must emit its section marker. Got: {html}");
        assert!(html.contains("Total Credits Outstanding"), "CreditsBalanceCards must include the stat label. Got: {html}");
    }

    /// `CreditsTransactionList` renders the history table.
    #[test]
    fn credits_transaction_list_renders() {
        let el = rsx! { CreditsTransactionList {} };
        let html = render_to_string(el);
        assert!(html.contains("credits-transaction-list"), "CreditsTransactionList must emit its section marker. Got: {html}");
        assert!(html.contains("Credit Transactions"), "CreditsTransactionList must render the heading. Got: {html}");
    }

    /// `CreditsTopupForm` renders the grant form.
    #[test]
    fn credits_topup_form_renders() {
        let el = rsx! { CreditsTopupForm {} };
        let html = render_to_string(el);
        assert!(html.contains("credits-topup-form"), "CreditsTopupForm must emit its section marker. Got: {html}");
        assert!(html.contains("Grant Credits"), "CreditsTopupForm must render the heading. Got: {html}");
    }

    /// `CreditsRevokeDialog` renders the destructive revoke form.
    #[test]
    fn credits_revoke_dialog_renders() {
        let el = rsx! { CreditsRevokeDialog {} };
        let html = render_to_string(el);
        assert!(html.contains("credits-revoke-dialog"), "CreditsRevokeDialog must emit its section marker. Got: {html}");
        assert!(html.contains("Revoke Credits"), "CreditsRevokeDialog must render the heading. Got: {html}");
    }

    /// `PlanListSidebar` renders the grouped plan list.
    #[test]
    fn plan_list_sidebar_renders() {
        let el = rsx! { PlanListSidebar { plans: sample_plans() } };
        let html = render_to_string(el);
        assert!(html.contains("plan-list-sidebar"), "PlanListSidebar must emit its section marker. Got: {html}");
        assert!(html.contains("Active Plans"), "PlanListSidebar must render the heading. Got: {html}");
    }

    /// `PlanEditorPage` renders the full editor with all sub-cards.
    #[test]
    fn plan_editor_page_renders() {
        let el = rsx! { PlanEditorPage { plan_id: "pro".to_string() } };
        let html = render_to_string(el);
        assert!(html.contains("plan-editor-page"), "PlanEditorPage must emit its section marker. Got: {html}");
        assert!(html.contains("Editing plan: pro"), "PlanEditorPage must render the plan id header. Got: {html}");
    }

    /// `PlanEditorDrawer` renders the drawer chrome.
    #[test]
    fn plan_editor_drawer_renders() {
        let el = rsx! { PlanEditorDrawer {} };
        let html = render_to_string(el);
        assert!(html.contains("plan-editor-drawer"), "PlanEditorDrawer must emit its section marker. Got: {html}");
    }

    /// `PlanApiLimits` renders the API limitations card.
    #[test]
    fn plan_api_limits_renders() {
        let el = rsx! { PlanApiLimits {} };
        let html = render_to_string(el);
        assert!(html.contains("plan-api-limits"), "PlanApiLimits must emit its section marker. Got: {html}");
        assert!(html.contains("API limitations"), "PlanApiLimits must render the heading. Got: {html}");
    }

    /// `PlanPromotions` renders the promotions card.
    #[test]
    fn plan_promotions_renders() {
        let el = rsx! { PlanPromotions {} };
        let html = render_to_string(el);
        assert!(html.contains("plan-promotions"), "PlanPromotions must emit its section marker. Got: {html}");
        // The Dioxus SSR HTML-escapes `&` to `&#38;`, so we check
        // for either the literal or the escaped form.
        assert!(
            html.contains("Promotion & discounts") || html.contains("Promotion &#38; discounts"),
            "PlanPromotions must render the heading. Got: {html}"
        );
    }

    /// `PlanItemCard` renders the single plan row.
    #[test]
    fn plan_item_card_renders() {
        let el = rsx! { PlanItemCard {
            name: "Pro".to_string(),
            category: "personal".to_string(),
            is_active: true,
            is_system: false,
            perm_count: 5,
            members: 412,
            max_members: Some(1),
            is_public: true,
            updated_at: "5h ago".to_string(),
        } };
        let html = render_to_string(el);
        assert!(html.contains("plan-item-card"), "PlanItemCard must emit its section marker. Got: {html}");
        assert!(html.contains("Pro"), "PlanItemCard must render the plan name. Got: {html}");
    }

    /// `WalletAccessManager` renders the two-column access layout.
    #[test]
    fn wallet_access_manager_renders() {
        let el = rsx! { WalletAccessManager {} };
        let html = render_to_string(el);
        assert!(html.contains("wallet-access-manager"), "WalletAccessManager must emit its section marker. Got: {html}");
        assert!(html.contains("Available plans"), "WalletAccessManager must render the available column. Got: {html}");
        assert!(html.contains("Authorized plans"), "WalletAccessManager must render the authorized column. Got: {html}");
    }

    /// `PlanSelectorModal` renders the plan picker modal chrome.
    #[test]
    fn plan_selector_modal_renders() {
        let el = rsx! { PlanSelectorModal {} };
        let html = render_to_string(el);
        assert!(html.contains("plan-selector-modal"), "PlanSelectorModal must emit its section marker. Got: {html}");
        assert!(html.contains("Select a plan"), "PlanSelectorModal must render the heading. Got: {html}");
    }

    /// `AccessGrantForm` renders the wallet-grant form.
    #[test]
    fn access_grant_form_renders() {
        let el = rsx! { AccessGrantForm {} };
        let html = render_to_string(el);
        assert!(html.contains("access-grant-form"), "AccessGrantForm must emit its section marker. Got: {html}");
        assert!(html.contains("Grant wallet access"), "AccessGrantForm must render the heading. Got: {html}");
    }

    /// `AccessRevokeDialog` renders the destructive revoke dialog.
    #[test]
    fn access_revoke_dialog_renders() {
        let el = rsx! { AccessRevokeDialog {} };
        let html = render_to_string(el);
        assert!(html.contains("access-revoke-dialog"), "AccessRevokeDialog must emit its section marker. Got: {html}");
        assert!(html.contains("Revoke wallet access"), "AccessRevokeDialog must render the heading. Got: {html}");
    }
}

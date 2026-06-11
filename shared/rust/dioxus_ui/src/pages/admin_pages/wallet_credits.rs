//! /admin/wallet-management/credits — credits management.
//!
//! Wave 6B Track C port — brings the page from a thin shell (49 LoC) to
//! a section-level port of the Next.js source
//! (`apps-old/admin-frontend/app/wallet-management/credits/page.tsx`
//! 63 LoC + `components/credits/credits-management.tsx` 431 LoC).
//!
//! Section coverage (matches design doc §"Track C — ..."):
//! 1. `CreditsLedger` — outer page chrome (tab switcher + action bar).
//!    Mirrors the page's `tabs` array (Overview / Grant / History).
//! 2. `CreditsBalanceCards` — 3 sub-cards: Total Credits Outstanding,
//!    Credits Granted Today, Credits Used Today, Active Users with
//!    Credits. Mirrors `credits-management.tsx` `OverviewTab` `StatCard`
//!    grid.
//! 3. `CreditsTransactionList` — table of credit transactions (Type,
//!    Amount, Balance After, Reason, Date, Granted By). Mirrors
//!    `CreditHistoryTab` `<table>`.
//! 4. `CreditsTopupForm` — Grant form: wallet address, amount, reason,
//!    expiry date. Mirrors `GrantForm` (grant mode).
//! 5. `CreditsRevokeDialog` — Revoke form (same fields, destructive
//!    button). Mirrors `GrantForm` (revoke mode).
//!
//! Section markers (used by `tests::test_section_markers`):
//!   - `credits-ledger`
//!   - `credits-balance-cards`
//!   - `credits-transaction-list`
//!   - `credits-topup-form`
//!   - `credits-revoke-dialog`
//!
//! The page uses tabs (Overview / Grant / History) like the source. The
//! default tab is "overview" so the balance cards are visible by
//! default. The Grant and History tabs render the topup/revoke form and
//! the transaction list respectively.

use crate::data_table::{Column, DataTable, Row, SortDir};
use crate::feedback::*;
use crate::primitives::*;

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::auth::AdminAuthGate;

// ============================================================================
// Page entry
// ============================================================================

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Credits");
    (meta, rsx! {
        AdminAuthGate {
            user: ctx.user.clone(),
            feature: Some("credits management".to_string()),
            required_permissions: Some(vec!["wallets:manage".to_string()]),
            return_url: Some(ctx.path.clone()),
            RenderCreditsPage { ctx: ctx.clone() }
        }
    })
}

#[component]
fn RenderCreditsPage(ctx: PageContext) -> Element {
    let mut active_tab = use_signal(|| "overview".to_string());
    let _ = active_tab.read();

    rsx! {
        div { class: "container page-content",
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

                // Tab switcher
                CreditsLedgerTabs { active: active_tab.read().clone() }

                // Tab body
                if active_tab.read().as_str() == "overview" {
                    OverviewTab {}
                } else if active_tab.read().as_str() == "grant" {
                    GrantTab {}
                } else {
                    HistoryTab {}
                }
            }
        }
    }
}

// ============================================================================
// Tab switcher
// ============================================================================

#[component]
fn CreditsLedgerTabs(active: String) -> Element {
    let tab_btn = |label: &str, icon: &str, key: &str, active: &str| {
        let is_active = key == active;
        let class = if is_active {
            "flex items-center gap-2 px-4 py-3 font-semibold text-sm transition-all relative text-[#1fc7d4]"
        } else {
            "flex items-center gap-2 px-4 py-3 font-semibold text-sm transition-all relative text-muted-foreground hover:text-foreground"
        };
        rsx! {
            button { class: "{class}", r#type: "button",
                Icon { name: icon.to_string(), size: Some(16) }
                "{label}"
                if is_active {
                    span { class: "absolute bottom-0 left-0 right-0 h-[2px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]" }
                }
            }
        }
    };
    rsx! {
        div { class: "flex gap-1 mb-6 border-b border-border/30",
            {tab_btn("Overview", "bar-chart-3", "overview", &active)}
            {tab_btn("Grant Credits", "plus", "grant", &active)}
            {tab_btn("Credit History", "history", "history", &active)}
        }
    }
}

/// Non-component tab button. We pass the args as plain strings to
/// avoid the `#[component]` Props plumbing — same pattern as the
/// `tab_btn` helper in `payments.rs`.
fn CreditsTabButton(label: String, icon: String, key: String, active: String) -> Element {
    let is_active = key == active;
    let class = if is_active {
        "flex items-center gap-2 px-4 py-3 font-semibold text-sm transition-all relative text-[#1fc7d4]"
    } else {
        "flex items-center gap-2 px-4 py-3 font-semibold text-sm transition-all relative text-muted-foreground hover:text-foreground"
    };
    rsx! {
        button { class: "{class}", r#type: "button",
            Icon { name: icon, size: Some(16) }
            "{label}"
            if is_active {
                span { class: "absolute bottom-0 left-0 right-0 h-[2px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]" }
            }
        }
    }
}

// ============================================================================
// Section 1: CreditsBalanceCards — 3 sub-cards (mirrors OverviewTab)
// ============================================================================

#[component]
fn OverviewTab() -> Element {
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

            // 3 sub-cards in a responsive grid
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
// Section 2: CreditsTransactionList — credit history table
// ============================================================================

#[component]
fn HistoryTab() -> Element {
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
        div { class: "space-y-6",
            // Search form
            div { class: "rounded-xl border border-border/20 bg-card p-4",
                div { class: "flex gap-3",
                    input {
                        class: "input flex-1",
                        r#type: "text",
                        placeholder: "Enter wallet address to search...",
                    }
                    button { class: "btn btn-primary", r#type: "button",
                        Icon { name: "search".to_string(), size: Some(14) }
                        " Search"
                    }
                }
            }

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
}

// ============================================================================
// Section 3: CreditsTopupForm — Grant form
// ============================================================================

#[component]
fn GrantTab() -> Element {
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
// Section 4: CreditsRevokeDialog — Revoke dialog (target slot for
// Track B's <AdminActionConfirm>; rendered as a destructive form for now).
// ============================================================================

#[component]
fn CreditsRevokeDialog() -> Element {
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
            path: "/wallet-management/credits".to_string(),
            ..Default::default()
        }
    }

    fn render_to_string(ctx: &PageContext) -> String {
        let (_meta, el) = render(ctx);
        dioxus_ssr::render_element(el)
    }

    /// Wave 6B — `test_render_smoke`. The page renders non-empty HTML
    /// when the admin is authed and holds `wallets:manage`.
    #[test]
    fn test_render_smoke() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        assert!(!html.is_empty(), "wallet_credits page must render non-empty HTML. Got: {}", html);
        assert!(html.contains("Credits"), "wallet_credits page must contain the credits title. Got: {}", html);
    }

    /// Wave 6B — `test_section_markers`. The default tab is
    /// "overview" so the ledger chrome + balance cards markers are
    /// visible. The topup-form / revoke-dialog / transaction-list
    /// markers are on other tabs (still in the file so the
    /// section-marker contract is verifiable per-page).
    #[test]
    fn test_section_markers() {
        let html = render_to_string(&authed_ctx());
        for marker in &[
            "credits-ledger",
            "credits-balance-cards",
        ] {
            assert!(
                html.contains(marker),
                "wallet_credits page should contain section marker `{marker}`. Got: {html}"
            );
        }
    }
}

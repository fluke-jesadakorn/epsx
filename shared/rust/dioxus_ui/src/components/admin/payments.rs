//! Admin payments sub-components (1:1 mirror of the Next.js source's
//! `apps-old/admin-frontend/components/payments/*.tsx` + the inline
//! shapes from `app/payments/page.tsx`).
//!
//! Section markers (mirrored 1:1 from
//! `pages/admin_pages/payments.rs`):
//!   - `payments-stats` → `PaymentLinkStats`
//!   - `payments-filter-panel` → `PaymentsFilterPanel`
//!   - `payment-links-list` → `PaymentLinksList`
//!   - `access-management-list` → `AccessManagementList`
//!   - `create-link-form` → `CreateLinkForm`
//!   - `link-revoke-confirm` → `LinkRevokeConfirm`
//!
//! Wave 6C Track C — extracted from the Wave 6B Track C port of the
//! payments hub. Section markers are emitted with the same
//! `// === wave6b-admin-pages-depth-track-c <marker> ===` comment
//! convention the Wave 6B port used, so the parent page's
//! `test_section_markers` test passes without modification.

use crate::data_table::{Column, DataTable, Row, SortDir};
use crate::primitives::*;

use dioxus::prelude::*;

// ============================================================================
// PaymentLinkStats — 4 stat cards at the top of the payments hub.
// Mirrors `payments-management.tsx` `StatsGrid`.
// ============================================================================

#[component]
pub fn PaymentLinkStats() -> Element {
    rsx! {
        // === wave6b-admin-pages-depth-track-c payments-stats ===
        div { class: "payments-stats grid grid-cols-2 lg:grid-cols-4 gap-4 mb-8",
            StatCard { label: "Total Revenue".to_string(), value: "$45,231.00".to_string(), icon: Some("trending-up".to_string()) }
            StatCard { label: "Successful".to_string(), value: "1,234".to_string(), icon: Some("check".to_string()) }
            StatCard { label: "Pending".to_string(), value: "12".to_string(), icon: Some("clock".to_string()) }
            StatCard { label: "Today".to_string(), value: "$2,310.00".to_string(), icon: Some("credit-card".to_string()) }
        }
    }
}

// ============================================================================
// PaymentsFilterPanel — search + status + method + plan filter bar.
// Mirrors `payments-management.tsx` `FilterSection`.
// ============================================================================

#[component]
pub fn PaymentsFilterPanel() -> Element {
    rsx! {
        // === wave6b-admin-pages-depth-track-c payments-filter-panel ===
        div { class: "payments-filter-panel rounded-xl border border-border/20 bg-card p-4 mb-6",
            div { class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-4 items-end",
                div { class: "space-y-2",
                    label { class: "text-[10px] font-black text-muted-foreground uppercase tracking-[0.2em] ml-2", "Search" }
                    input {
                        class: "input",
                        r#type: "text",
                        placeholder: "Reference, wallet...",
                    }
                }
                div { class: "space-y-2",
                    label { class: "text-[10px] font-black text-muted-foreground uppercase tracking-[0.2em] ml-2", "Status" }
                    select {
                        class: "input",
                        option { value: "", "All Status" }
                        option { value: "succeeded", "Succeeded" }
                        option { value: "pending", "Pending" }
                        option { value: "failed", "Failed" }
                    }
                }
                div { class: "space-y-2",
                    label { class: "text-[10px] font-black text-muted-foreground uppercase tracking-[0.2em] ml-2", "Method" }
                    select {
                        class: "input",
                        option { value: "", "All Methods" }
                        option { value: "on_chain", "On Chain" }
                        option { value: "on_line", "Online" }
                    }
                }
                div { class: "space-y-2",
                    label { class: "text-[10px] font-black text-muted-foreground uppercase tracking-[0.2em] ml-2", "Plan" }
                    select {
                        class: "input",
                        option { value: "BASIC", "Basic" }
                        option { value: "PRO", "Pro" }
                        option { value: "ENTERPRISE", "Enterprise" }
                        option { value: "WHALE", "Whale" }
                    }
                }
                button {
                    class: "btn btn-outline",
                    r#type: "button",
                    "Reset"
                }
            }
        }
    }
}

// ============================================================================
// PaymentLinksList — main payments table.
// Mirrors `payments-management.tsx` `PaymentTableRow` + table shell.
// ============================================================================

#[component]
pub fn PaymentLinksList() -> Element {
    let columns = vec![
        Column { key: "reference".into(), label: "Reference".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("18%".into()), class_name: None },
        Column { key: "wallet".into(), label: "Wallet".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("18%".into()), class_name: None },
        Column { key: "plan".into(), label: "Plan".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("12%".into()), class_name: None },
        Column { key: "amount".into(), label: "Amount".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("12%".into()), class_name: None },
        Column { key: "status".into(), label: "Status".into(), sortable: true, align: crate::primitives::data_table::Align::Center, width: Some("12%".into()), class_name: None },
        Column { key: "created".into(), label: "Created".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("16%".into()), class_name: None },
        Column { key: "actions".into(), label: "Actions".into(), sortable: false, align: crate::primitives::data_table::Align::Right, width: Some("12%".into()), class_name: None },
    ];
    let rows = vec![
        Row { id: "pi_1".into(), cells: vec!["pi_abc123".into(), "0x1234…5678".into(), "Pro".into(), "$29.00".into(), "Succeeded".into(), "2024-09-20 10:32".into(), "Explorer".into()] },
        Row { id: "pi_2".into(), cells: vec!["pi_def456".into(), "0xabcd…ef12".into(), "Pro".into(), "$29.00".into(), "Pending".into(), "2024-09-20 11:14".into(), "Explorer".into()] },
        Row { id: "pi_3".into(), cells: vec!["pi_ghi789".into(), "0x9876…5432".into(), "Enterprise".into(), "$299.00".into(), "Failed".into(), "2024-09-19 09:21".into(), "Explorer".into()] },
        Row { id: "pi_4".into(), cells: vec!["pi_jkl012".into(), "0x4bcd…9af0".into(), "Whale".into(), "$999.00".into(), "Succeeded".into(), "2024-09-19 18:45".into(), "Explorer".into()] },
        Row { id: "pi_5".into(), cells: vec!["pi_mno345".into(), "0x8f12…4e9a".into(), "Pro".into(), "$29.00".into(), "Succeeded".into(), "2024-09-18 22:11".into(), "Explorer".into()] },
    ];
    rsx! {
        // === wave6b-admin-pages-depth-track-c payment-links-list ===
        div { class: "payment-links-list rounded-2xl border border-border/20 overflow-hidden bg-card",
            div { class: "h-[3px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]" }
            div { class: "p-6",
                div { class: "flex items-center justify-between mb-4",
                    h2 { class: "text-xs font-bold text-[#1fc7d4] uppercase tracking-[0.2em]",
                        "Recent Transactions"
                    }
                    span { class: "px-3 py-1 bg-muted/50 rounded-full border border-border/40 text-xs font-bold text-muted-foreground uppercase tracking-widest",
                        "{rows.len()} Payments"
                    }
                }
                DataTable {
                    columns,
                    rows,
                    striped: true,
                    page_size: 25,
                    filter_placeholder: Some("Filter by reference, wallet, plan...".to_string()),
                    initial_sort: Some(("created".to_string(), SortDir::Desc)),
                }
            }
        }
    }
}

// ============================================================================
// AccessManagementList — users with active plan access.
// Mirrors `user-access-management.tsx` `UserAccessDesktopTable`.
// ============================================================================

#[component]
pub fn AccessManagementList() -> Element {
    let columns = vec![
        Column { key: "wallet".into(), label: "Wallet".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("22%".into()), class_name: None },
        Column { key: "plan".into(), label: "Plan".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("16%".into()), class_name: None },
        Column { key: "status".into(), label: "Status".into(), sortable: true, align: crate::primitives::data_table::Align::Center, width: Some("14%".into()), class_name: None },
        Column { key: "days_left".into(), label: "Days Left".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("12%".into()), class_name: None },
        Column { key: "expires".into(), label: "Expires".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("22%".into()), class_name: None },
        Column { key: "actions".into(), label: "Actions".into(), sortable: false, align: crate::primitives::data_table::Align::Right, width: Some("14%".into()), class_name: None },
    ];
    let rows = vec![
        Row { id: "0x1234".into(), cells: vec!["0x1234…5678".into(), "Pro".into(), "Active".into(), "23 days".into(), "2024-10-15 00:00".into(), "View".into()] },
        Row { id: "0xabcd".into(), cells: vec!["0xabcd…ef12".into(), "Enterprise".into(), "Active".into(), "312 days".into(), "2025-08-22 00:00".into(), "View".into()] },
        Row { id: "0x9876".into(), cells: vec!["0x9876…5432".into(), "Whale".into(), "Expiring soon".into(), "3 days".into(), "2024-09-25 00:00".into(), "View".into()] },
        Row { id: "0x4bcd".into(), cells: vec!["0x4bcd…9af0".into(), "Pro".into(), "Active".into(), "18 days".into(), "2024-10-08 00:00".into(), "View".into()] },
    ];
    rsx! {
        // === wave6b-admin-pages-depth-track-c access-management-list ===
        div { class: "access-management-list rounded-2xl border border-border/20 overflow-hidden bg-card",
            div { class: "h-[3px] bg-gradient-to-r from-[#31d0aa] to-[#1fc7d4]" }
            div { class: "p-6",
                div { class: "flex items-center justify-between mb-4",
                    h2 { class: "text-xs font-bold text-[#31d0aa] uppercase tracking-[0.2em]",
                        "All Users with Plan Access"
                    }
                    span { class: "px-3 py-1 bg-muted/50 rounded-full border border-border/40 text-xs font-bold text-muted-foreground",
                        "{rows.len()} users"
                    }
                }
                DataTable {
                    columns,
                    rows,
                    striped: true,
                    page_size: 20,
                    filter_placeholder: Some("Filter by wallet, plan...".to_string()),
                    initial_sort: Some(("days_left".to_string(), SortDir::Asc)),
                }
            }
        }
    }
}

// ============================================================================
// CreateLinkForm — modal form for new payment links.
// Mirrors `payment-links-ui.tsx` `ModalFormFields`.
// ============================================================================

#[component]
pub fn CreateLinkForm() -> Element {
    rsx! {
        // === wave6b-admin-pages-depth-track-c create-link-form ===
        div { class: "create-link-form rounded-2xl border border-border/20 overflow-hidden bg-card",
            div { class: "h-[3px] bg-gradient-to-r from-[#7645d9] to-[#ed4b9e]" }
            div { class: "p-6",
                h2 { class: "text-xs font-bold text-[#7645d9] uppercase tracking-[0.2em] mb-4",
                    "Create Payment Link"
                }
                div { class: "space-y-4",
                    div {
                        label { class: "block text-sm font-medium text-muted-foreground mb-2", "Context Type *" }
                        select {
                            class: "input",
                            required: true,
                            option { value: "plan", "Plan — Plan payment" }
                            option { value: "group", "Group — Permission group access" }
                            option { value: "product", "Product — One-time product purchase" }
                            option { value: "campaign", "Campaign — Promotional campaign" }
                            option { value: "custom", "Custom — Custom payment link" }
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-muted-foreground mb-2", "Name *" }
                        input { class: "input", r#type: "text", required: true, placeholder: "e.g., Pro Plan Monthly" }
                    }
                    div {
                        label { class: "block text-sm font-medium text-muted-foreground mb-2", "Description" }
                        textarea { class: "input", rows: "2", placeholder: "Optional description" }
                    }
                    div { class: "grid grid-cols-2 gap-4",
                        div {
                            label { class: "block text-sm font-medium text-muted-foreground mb-2", "Amount *" }
                            input { class: "input", r#type: "number", step: "0.01", min: "0.01", required: true, placeholder: "0.00" }
                        }
                        div {
                            label { class: "block text-sm font-medium text-muted-foreground mb-2", "Currency" }
                            select { class: "input",
                                option { value: "USDT", "USDT" }
                                option { value: "USDC", "USDC" }
                                option { value: "BNB", "BNB" }
                            }
                        }
                    }
                    div { class: "grid grid-cols-2 gap-4",
                        div {
                            label { class: "block text-sm font-medium text-muted-foreground mb-2", "Expires In (hours)" }
                            input { class: "input", r#type: "number", min: "1", placeholder: "24" }
                            p { class: "text-xs text-muted-foreground mt-1", "Leave empty for no expiration" }
                        }
                        div {
                            label { class: "block text-sm font-medium text-muted-foreground mb-2", "Max Uses" }
                            input { class: "input", r#type: "number", min: "1", placeholder: "Unlimited" }
                            p { class: "text-xs text-muted-foreground mt-1", "Leave empty for unlimited" }
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-muted-foreground mb-2", "Custom Slug (optional)" }
                        input { class: "input", r#type: "text", placeholder: "Auto-generated if empty" }
                    }
                    div { class: "flex justify-end gap-2 pt-2",
                        button { class: "btn btn-outline", r#type: "button", "Cancel" }
                        button { class: "btn btn-primary", r#type: "submit",
                            Icon { name: "plus".to_string(), size: Some(14) }
                            " Create Link"
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// LinkRevokeConfirm — confirm-revoke dialog. Target slot for
// Track B's `<AdminActionConfirm>`; rendered as a static card for now.
// ============================================================================

#[component]
pub fn LinkRevokeConfirm() -> Element {
    rsx! {
        // === wave6b-admin-pages-depth-track-c link-revoke-confirm ===
        div { class: "link-revoke-confirm rounded-2xl border border-destructive/30 bg-destructive/5 p-6",
            h2 { class: "text-xs font-bold text-destructive uppercase tracking-[0.2em] mb-3",
                "Revoke Payment Link"
            }
            p { class: "text-sm text-foreground mb-4",
                "Are you sure you want to revoke this payment link? This action cannot be undone."
            }
            p { class: "text-xs text-muted-foreground mb-4",
                "Once revoked, the link will be deactivated, marked unusable, and any pending payments will be cancelled."
            }
            div { class: "flex justify-end gap-2",
                button { class: "btn btn-outline", r#type: "button", "Cancel" }
                button { class: "btn btn-danger", r#type: "button",
                    Icon { name: "trash-2".to_string(), size: Some(14) }
                    " Revoke link"
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

    /// `PaymentLinkStats` renders the 4 stat cards with the
    /// `payments-stats` section marker.
    #[test]
    fn test_render_smoke() {
        let el = rsx! { PaymentLinkStats {} };
        let html = render_to_string(el);
        assert!(html.contains("payments-stats"), "PaymentLinkStats must emit its section marker. Got: {html}");
        assert!(html.contains("Total Revenue"), "PaymentLinkStats must include its labels. Got: {html}");
    }

    /// `PaymentsFilterPanel` renders the search/status/method/plan
    /// filter row.
    #[test]
    fn payments_filter_panel_renders() {
        let el = rsx! { PaymentsFilterPanel {} };
        let html = render_to_string(el);
        assert!(html.contains("payments-filter-panel"), "PaymentsFilterPanel must emit its section marker. Got: {html}");
        assert!(html.contains("All Status"), "PaymentsFilterPanel must include the status select options. Got: {html}");
    }

    /// `PaymentLinksList` renders the recent transactions table.
    #[test]
    fn payment_links_list_renders() {
        let el = rsx! { PaymentLinksList {} };
        let html = render_to_string(el);
        assert!(html.contains("payment-links-list"), "PaymentLinksList must emit its section marker. Got: {html}");
        assert!(html.contains("Recent Transactions"), "PaymentLinksList must render the heading. Got: {html}");
    }

    /// `AccessManagementList` renders the user-access table.
    #[test]
    fn access_management_list_renders() {
        let el = rsx! { AccessManagementList {} };
        let html = render_to_string(el);
        assert!(html.contains("access-management-list"), "AccessManagementList must emit its section marker. Got: {html}");
        assert!(html.contains("All Users with Plan Access"), "AccessManagementList must render the heading. Got: {html}");
    }

    /// `CreateLinkForm` renders the new-link form fields.
    #[test]
    fn create_link_form_renders() {
        let el = rsx! { CreateLinkForm {} };
        let html = render_to_string(el);
        assert!(html.contains("create-link-form"), "CreateLinkForm must emit its section marker. Got: {html}");
        assert!(html.contains("Create Payment Link"), "CreateLinkForm must render the heading. Got: {html}");
    }

    /// `LinkRevokeConfirm` renders the destructive confirm panel.
    #[test]
    fn link_revoke_confirm_renders() {
        let el = rsx! { LinkRevokeConfirm {} };
        let html = render_to_string(el);
        assert!(html.contains("link-revoke-confirm"), "LinkRevokeConfirm must emit its section marker. Got: {html}");
        assert!(html.contains("Revoke Payment Link"), "LinkRevokeConfirm must render the heading. Got: {html}");
    }
}

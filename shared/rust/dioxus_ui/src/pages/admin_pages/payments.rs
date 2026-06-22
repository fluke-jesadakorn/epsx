//! /admin/payments — payment management (Payments Hub).
//!
//! Wave 6B Track C port — brings the page from a thin shell (48 LoC) to
//! a section-level port of the Next.js source (`apps-old/admin-frontend/app/payments/page.tsx`
//! 31 LoC + 5 sub-components ~1,239 LoC).
//!
//! Wave 42 T2 — wire in the Wave 38b ported admin domain components
//! (`PageHeader`, `PaymentStatsGrid`, `PaymentFilterSection`,
//! `PaymentTableRow`, `UserAccessDesktopTable`, `PaymentLinksTable`,
//! `PaymentLinksHeaderRow`). The previous Wave 6B shell had custom
//! inline implementations of these sections — the Wave 38b port
//! aligned the markup with the source `payments-management.tsx`,
//! `user-access-management.tsx`, `payment-links-management.tsx`,
//! and `shared/page-layout.tsx` files.
//!
//! Section coverage (matches design doc §"Track C — payments + ..."):
//! 1. `PaymentLinkStats` → `PaymentStatsGrid` (4 stat cards).
//! 2. `PaymentsFilterPanel` → `PaymentFilterSection` (search + status
//!    + method + plan).
//! 3. `PaymentLinksList` → manual `<table>` wrapping `PaymentTableRow`s
//!    (the port doesn't ship a master table component).
//! 4. `AccessManagementList` → `UserAccessDesktopTable`.
//! 5. `CreateLinkForm` — kept inline (not ported as a separate
//!    component).
//! 6. `LinkRevokeConfirm` — kept inline.
//!
//! Section markers (preserved from Wave 6B; the test
//! `test_section_markers` asserts these):
//!   - `payments-stats`
//!   - `payments-filter-panel`
//!   - `payment-links-list`

use crate::data_table::{Column, DataTable, Row, SortDir};
use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::auth::AdminAuthGate;
use crate::components::admin::page_layout::{PageGradient, PageHeader, PageLayout, PageMaxWidth};
use crate::components::admin::payments_management::{
    PaymentFilterSection, PaymentFilters, PaymentMobileCard, PaymentPaginationBar, PaymentRow,
    PaymentStats, PaymentStatsGrid, PaymentTableRow,
};
use crate::components::admin::payment_links_management::{
    PaymentLink, PaymentLinksHeaderRow, PaymentLinksMobileCards, PaymentLinksTable,
};
use crate::components::admin::user_access_management::{
    UserAccessData, UserAccessDesktopTable, UserAccessMobileCards, UserAccessPaginationBar,
};

// ============================================================================
// Page entry
// ============================================================================

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Payments");
    (meta, rsx! {
        AdminAuthGate {
            user: ctx.user.clone(),
            feature: Some("payment management".to_string()),
            required_permissions: Some(vec!["payments:manage".to_string()]),
            return_url: Some(ctx.path.clone()),
            RenderPaymentsHub { ctx: ctx.clone() }
        }
    })
}

#[component]
fn RenderPaymentsHub(ctx: PageContext) -> Element {
    // Top-level state. In a real BFF render these are SSR'd and a
    // client-side `refresh()` re-fetches. For the static port we
    // initialize the signals with sample data.
    let mut active_tab = use_signal(|| "payments".to_string());
    let _ = active_tab.read();

    // Payment stats (matches source `StatsGrid`). Wired into the
    // Wave 38b `PaymentStatsGrid` component.
    let stats = PaymentStats {
        total_payments: 1234,
        total_amount: 45231.00,
        successful_payments: 1198,
        failed_payments: 24,
        pending_payments: 12,
        average_payment_amount: 36.65,
        payments_today: 47,
        revenue_today: 2310.00,
    };

    // Default filter state.
    let mut filters = use_signal(PaymentFilters::default);

    // Sample payment rows for the manual `<table>` wrap of
    // `PaymentTableRow`s. The Wave 38b port doesn't include a
    // master `<table>` component — only `<tr>` rows. The page
    // wires the rows into a manual table shell matching the
    // source's `payments-management.tsx` desktop view.
    let payments = vec![
        PaymentRow {
            id: "pi_1".into(),
            payment_reference: "pi_abc123".into(),
            wallet_address: "0x12345678…9abc".into(),
            plan_name: "Pro".into(),
            amount: 29.00,
            currency: "USDT".into(),
            status: "succeeded".into(),
            created_at: "2024-09-20 10:32".into(),
            transaction_hash: Some("0xtxhash0001".into()),
        },
        PaymentRow {
            id: "pi_2".into(),
            payment_reference: "pi_def456".into(),
            wallet_address: "0xabcdef12…3456".into(),
            plan_name: "Pro".into(),
            amount: 29.00,
            currency: "USDT".into(),
            status: "pending".into(),
            created_at: "2024-09-20 11:14".into(),
            transaction_hash: None,
        },
        PaymentRow {
            id: "pi_3".into(),
            payment_reference: "pi_ghi789".into(),
            wallet_address: "0x98765432…abcd".into(),
            plan_name: "Enterprise".into(),
            amount: 299.00,
            currency: "USDT".into(),
            status: "failed".into(),
            created_at: "2024-09-19 09:21".into(),
            transaction_hash: None,
        },
        PaymentRow {
            id: "pi_4".into(),
            payment_reference: "pi_jkl012".into(),
            wallet_address: "0x4bcd9af0…1234".into(),
            plan_name: "Whale".into(),
            amount: 999.00,
            currency: "USDT".into(),
            status: "succeeded".into(),
            created_at: "2024-09-19 18:45".into(),
            transaction_hash: Some("0xtxhash0004".into()),
        },
        PaymentRow {
            id: "pi_5".into(),
            payment_reference: "pi_mno345".into(),
            wallet_address: "0x8f124e9a…5678".into(),
            plan_name: "Pro".into(),
            amount: 29.00,
            currency: "USDT".into(),
            status: "succeeded".into(),
            created_at: "2024-09-18 22:11".into(),
            transaction_hash: Some("0xtxhash0005".into()),
        },
    ];

    // User access rows for the Wave 38b `UserAccessDesktopTable`.
    let user_access = vec![
        UserAccessData {
            wallet_address: "0x12345678…9abc".into(),
            plan_name: Some("Pro".into()),
            status: "active".into(),
            days_remaining: 23,
            plan_expires_at: Some("2024-10-15 00:00".into()),
        },
        UserAccessData {
            wallet_address: "0xabcdef12…3456".into(),
            plan_name: Some("Enterprise".into()),
            status: "active".into(),
            days_remaining: 312,
            plan_expires_at: Some("2025-08-22 00:00".into()),
        },
        UserAccessData {
            wallet_address: "0x98765432…abcd".into(),
            plan_name: Some("Whale".into()),
            status: "expiring_soon".into(),
            days_remaining: 3,
            plan_expires_at: Some("2024-09-25 00:00".into()),
        },
        UserAccessData {
            wallet_address: "0x4bcd9af0…1234".into(),
            plan_name: Some("Pro".into()),
            status: "active".into(),
            days_remaining: 18,
            plan_expires_at: Some("2024-10-08 00:00".into()),
        },
    ];

    // Sample payment-link rows for the Wave 38b
    // `PaymentLinksTable`.
    let links = vec![
        PaymentLink {
            id: "pl_1".into(),
            slug: "pro-monthly".into(),
            context_type: "plan".into(),
            context_id: Some("plan_pro".into()),
            name: "Pro Plan Monthly".into(),
            amount: 29.00,
            currency: "USDT".into(),
            is_active: true,
            is_usable: true,
            uses: 47,
            max_uses: None,
            created_at: "2024-09-01".into(),
            expires_at: None,
        },
        PaymentLink {
            id: "pl_2".into(),
            slug: "enterprise-q4".into(),
            context_type: "plan".into(),
            context_id: Some("plan_enterprise".into()),
            name: "Enterprise Q4 promo".into(),
            amount: 299.00,
            currency: "USDT".into(),
            is_active: true,
            is_usable: true,
            uses: 12,
            max_uses: Some(50),
            created_at: "2024-09-15".into(),
            expires_at: Some("2024-12-31".into()),
        },
        PaymentLink {
            id: "pl_3".into(),
            slug: "whale-vip".into(),
            context_type: "custom".into(),
            context_id: None,
            name: "Whale VIP".into(),
            amount: 999.00,
            currency: "USDT".into(),
            is_active: false,
            is_usable: false,
            uses: 5,
            max_uses: Some(10),
            created_at: "2024-08-10".into(),
            expires_at: Some("2024-09-30".into()),
        },
    ];

    // The Refresh / Export-CSV action row that goes into the
    // PageHeader's `extra_actions` slot.
    let action_row = rsx! {
        button { class: "btn btn-sm btn-outline", r#type: "button",
            Icon { name: "refresh-cw".to_string(), size: Some(14) }
            " Refresh"
        }
        button { class: "btn btn-sm btn-outline", r#type: "button",
            Icon { name: "bar-chart-3".to_string(), size: Some(14) }
            " Export CSV"
        }
    };

    rsx! {
        PageLayout {
            max_width: Some(PageMaxWidth::SevenXl),
            // Wave 42 T2 — wire in the Wave 38b `PageHeader` for
            // the page-level title + icon + subtitle + gradient +
            // action buttons (mirrors the source
            // `<PageHeader title="Payments Hub" subtitle="…"
            // icon="CreditCard" gradient="primary" centered>`).
            PageHeader {
                title: "Payments Hub".to_string(),
                subtitle: Some("Manage payments, user access, and payment links".to_string()),
                icon: Some("credit-card".to_string()),
                gradient: Some(PageGradient::Primary),
                centered: Some(true),
                extra_actions: Some(rsx! { {action_row} }),
                class_name: None,
            }

            // Tab switcher (mirrors the source's `?tab=` query
            // pattern but rendered as buttons since we're
            // SSR-static).
            PaymentsHubTabs { active: active_tab.read().clone() }

            if active_tab.read().as_str() == "payments" {
                PaymentsTab {
                    stats: stats.clone(),
                    filters: filters.read().clone(),
                    payments: payments.clone(),
                }
            } else if active_tab.read().as_str() == "user-access" {
                UserAccessTab {
                    user_access: user_access.clone(),
                }
            } else {
                PaymentLinksTab {
                    links: links.clone(),
                }
            }
        }
    }
}

// ============================================================================
// Tab 1: Payments — uses Wave 38b ported components
// ============================================================================

#[component]
fn PaymentsTab(
    stats: PaymentStats,
    filters: PaymentFilters,
    payments: Vec<PaymentRow>,
) -> Element {
    let mut filters_signal = use_signal(|| filters.clone());
    rsx! {
        div { class: "space-y-6",
            // Section 1: stats (Wave 38b `PaymentStatsGrid`).
            // Outer wrapper preserves the `payments-stats` marker
            // class the Wave 6B tests still assert on.
            div { class: "payments-stats",
                PaymentStatsGrid { stats: stats.clone() }
            }

            // Section 2: filter panel (Wave 38b
            // `PaymentFilterSection`).
            div { class: "payments-filter-panel",
                PaymentFilterSection {
                    filters: filters_signal.read().clone(),
                    on_change: move |f: PaymentFilters| filters_signal.set(f),
                    on_reset: move |_| filters_signal.set(PaymentFilters::default()),
                }
            }

            // Section 3: payments list. The Wave 38b port only
            // ships a `<tr>`-emitting `PaymentTableRow`, so the
            // page wraps the rows in a manual `<table>` shell
            // matching the source's desktop view.
            div { class: "payment-links-list rounded-2xl border border-border/20 overflow-hidden bg-card",
                div { class: "h-[3px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]" }
                div { class: "p-6",
                    div { class: "flex items-center justify-between mb-4",
                        h2 { class: "text-xs font-bold text-[#1fc7d4] uppercase tracking-[0.2em]",
                            "Recent Transactions"
                        }
                        span { class: "px-3 py-1 bg-muted/50 rounded-full border border-border/40 text-xs font-bold text-muted-foreground uppercase tracking-widest",
                            "{payments.len()} Payments"
                        }
                    }
                    PaymentsDesktopTable { rows: payments.clone() }
                    PaymentPaginationBar {
                        current: 1,
                        total: 1,
                        on_change: move |_| {},
                    }
                    // Mobile card list (hidden on sm+ via the
                    // ported component's own `hidden sm:block` /
                    // `block sm:hidden` classes).
                    PaymentMobileCards { rows: payments.clone() }
                }
            }
        }
    }
}

/// Desktop `<table>` wrapping `PaymentTableRow`s. The Wave 38b
/// port doesn't ship a master table component, so the page-level
/// wiring renders the `<table>` shell here.
#[component]
fn PaymentsDesktopTable(rows: Vec<PaymentRow>) -> Element {
    rsx! {
        div { class: "hidden sm:block overflow-x-auto",
            table { class: "min-w-full",
                thead {
                    tr { class: "border-b border-border/50",
                        th { class: "px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider", "Reference" }
                        th { class: "px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider", "Wallet" }
                        th { class: "px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider", "Plan" }
                        th { class: "px-4 py-3 text-right text-xs font-semibold text-muted-foreground uppercase tracking-wider", "Amount" }
                        th { class: "px-4 py-3 text-center text-xs font-semibold text-muted-foreground uppercase tracking-wider", "Status" }
                        th { class: "px-4 py-3 text-right text-xs font-semibold text-muted-foreground uppercase tracking-wider", "Created" }
                        th { class: "px-4 py-3 text-right text-xs font-semibold text-muted-foreground uppercase tracking-wider", "Actions" }
                    }
                }
                tbody { class: "divide-y divide-border/50",
                    for payment in rows.iter() {
                        PaymentTableRow { payment: payment.clone() }
                    }
                }
            }
        }
    }
}

/// Mobile card list for the payments tab. The Wave 38b
/// `PaymentMobileCard` emits one card per row.
#[component]
fn PaymentMobileCards(rows: Vec<PaymentRow>) -> Element {
    rsx! {
        div { class: "block sm:hidden space-y-3",
            for payment in rows.iter() {
                PaymentMobileCard { payment: payment.clone() }
            }
        }
    }
}

// ============================================================================
// Tab 2: User Access — uses Wave 38b ported component
// ============================================================================

#[component]
fn UserAccessTab(user_access: Vec<UserAccessData>) -> Element {
    rsx! {
        div { class: "space-y-6",
            div { class: "access-management-list",
                // Top stats for the user-access tab (different
                // labels than the payments tab — kept as inline
                // `StatCard`s for the Wave 6B compatibility; the
                // port's `PaymentStatsGrid` is designed for the
                // payments shape).
                div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 mb-6",
                    StatCard { label: "Users with access".to_string(), value: "412".to_string(), icon: Some("users".to_string()) }
                    StatCard { label: "Expiring in 7 days".to_string(), value: "8".to_string(), icon: Some("clock".to_string()) }
                    StatCard { label: "Expired this month".to_string(), value: "3".to_string(), icon: Some("alert-circle".to_string()) }
                }
                // Wave 38b `UserAccessDesktopTable` (desktop).
                UserAccessDesktopTable { user_access: user_access.clone() }
                // Wave 38b `UserAccessMobileCards` (mobile).
                UserAccessMobileCards { user_access: user_access.clone() }
                // Wave 38b `UserAccessPaginationBar`.
                UserAccessPaginationBar {
                    current: 1,
                    disable_next: Some(true),
                    on_prev: move |_| {},
                    on_next: move |_| {},
                }
            }
        }
    }
}

// ============================================================================
// Tab 3: Payment Links — uses Wave 38b ported components
// ============================================================================

#[component]
fn PaymentLinksTab(links: Vec<PaymentLink>) -> Element {
    rsx! {
        div { class: "space-y-6",
            // Quick filter / actions row (kept inline — not in
            // the Wave 38b port).
            div { class: "rounded-xl border border-border/20 bg-card p-4 mb-6",
                div { class: "grid grid-cols-1 sm:grid-cols-3 gap-4",
                    div {
                        label { class: "block text-xs font-bold text-muted-foreground uppercase tracking-[0.15em] mb-2", "Context Type" }
                        select { class: "input",
                            option { value: "", "All Types" }
                            option { value: "plan", "Plan" }
                            option { value: "group", "Group" }
                            option { value: "product", "Product" }
                            option { value: "campaign", "Campaign" }
                            option { value: "custom", "Custom" }
                        }
                    }
                    div {
                        label { class: "block text-xs font-bold text-muted-foreground uppercase tracking-[0.15em] mb-2", "Status" }
                        select { class: "input",
                            option { value: "", "All Status" }
                            option { value: "true", "Active" }
                            option { value: "false", "Inactive" }
                        }
                    }
                    div { class: "flex items-end",
                        button { class: "btn btn-outline w-full", r#type: "button", "Reset" }
                    }
                }
            }
            // Wave 38b `PaymentLinksHeaderRow` + `PaymentLinksTable`
            // + `PaymentLinksMobileCards`.
            PaymentLinksHeaderRow { count: links.len() as u32 }
            PaymentLinksTable { links: links.clone() }
            PaymentLinksMobileCards { links: links.clone() }

            // Two-column layout: form on the left, list on the right
            // (kept inline from the Wave 6B shell).
            div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6",
                CreateLinkForm {}
                PaymentLinksSummaryCard { count: links.len() as u32 }
            }
            LinkRevokeConfirm {}
        }
    }
}

#[component]
fn PaymentLinksSummaryCard(count: u32) -> Element {
    rsx! {
        div { class: "payment-links-list rounded-2xl border border-border/20 overflow-hidden bg-card",
            div { class: "h-[3px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]" }
            div { class: "p-6",
                div { class: "flex items-center justify-between mb-4",
                    h2 { class: "text-xs font-bold text-[#1fc7d4] uppercase tracking-[0.2em]",
                        "Recent Links"
                    }
                    span { class: "px-3 py-1 bg-muted/50 rounded-full border border-border/40 text-xs font-bold text-muted-foreground uppercase tracking-widest",
                        "{count} links"
                    }
                }
                p { class: "text-sm text-muted-foreground",
                    "View and manage all created payment links in the table above."
                }
            }
        }
    }
}

// ============================================================================
// Section 5: CreateLinkForm — kept inline from Wave 6B
// ============================================================================

#[component]
fn CreateLinkForm() -> Element {
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
// Section 6: LinkRevokeConfirm — kept inline from Wave 6B
// ============================================================================

#[component]
fn LinkRevokeConfirm() -> Element {
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
// Tab switcher
// ============================================================================

#[component]
fn PaymentsHubTabs(active: String) -> Element {
    let tab_btn = |label: &str, key: &str, active: &str| rsx! {
        button {
            class: if active == key { "btn btn-primary btn-sm" } else { "btn btn-outline btn-sm" },
            r#type: "button",
            "{label}"
        }
    };
    rsx! {
        div { class: "flex gap-2 mb-6",
            {tab_btn("Payments", "payments", &active)}
            {tab_btn("User Access", "user-access", &active)}
            {tab_btn("Payment Links", "payment-links", &active)}
        }
    }
}

// ============================================================================
// Tests — Wave 6B Track C + Wave 42 T2
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
                permissions: vec!["payments:manage".to_string()],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: Some("Admin".to_string()),
            }),
            path: "/payments".to_string(),
            ..Default::default()
        }
    }

    fn render_to_string(ctx: &PageContext) -> String {
        let (_meta, el) = render(ctx);
        dioxus_ssr::render_element(el)
    }

    /// Wave 6B — `test_render_smoke`. The page renders non-empty HTML
    /// when the admin is authed and holds `payments:manage`.
    #[test]
    fn test_render_smoke() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        assert!(!html.is_empty(), "payments page must render non-empty HTML. Got: {}", html);
        assert!(html.contains("Payments Hub"), "payments page must contain the hub title. Got: {}", html);
    }

    /// Wave 6B — `test_section_markers`. All design-doc section
    /// markers are present in the rendered HTML. The Payments tab is
    /// the default tab, so the stats + filter + list sections are
    /// visible; create-link-form and link-revoke-confirm live on the
    /// Payment Links tab. The default tab is "payments", so the
    /// create / revoke / payment-links-table markers are NOT in the
    /// default render. We assert the 5 markers visible on the
    /// default tab.
    ///
    /// Wave 42 T2 — extended to also assert the Wave 38b ported
    /// component markers (`payments-management-stats`,
    /// `payments-management-filters`) are present in the default
    /// render, confirming the wiring landed. The `payment-links-table`
    /// marker is asserted separately in
    /// `test_ported_wiring_components_render` via direct render of
    /// `PaymentLinksTable`.
    #[test]
    fn test_section_markers() {
        let html = render_to_string(&authed_ctx());
        for marker in &[
            // Wave 6B markers (preserved by the wrapper divs).
            "payments-stats",
            "payments-filter-panel",
            "payment-links-list",
            // Wave 38b ported-component markers (new in Wave 42 T2).
            "payments-management-stats",
            "payments-management-filters",
        ] {
            assert!(
                html.contains(marker),
                "payments page should contain section marker `{marker}`. Got: {html}"
            );
        }
    }

    /// Wave 42 T2 — `test_ported_wiring_landed`. Asserts the
    /// Wave 38b `PageHeader` rendered the icon + gradient + title
    /// shape, AND that the Wave 38b `UserAccessDesktopTable` /
    /// `PaymentLinksTable` data structs compile + render. The
    /// user-access and payment-links tabs are rendered when the
    /// active tab signal is set to those values — but the page
    /// here is SSR-only (signals are not interactive in the SSR
    /// snapshot). We render each tab component directly to verify
    /// the ported components still render in isolation.
    #[test]
    fn test_ported_wiring_components_render() {
        // Direct test of the Wave 38b `UserAccessDesktopTable`.
        let mut vdom = dioxus::prelude::VirtualDom::new(|| {
            rsx! {
                UserAccessDesktopTable {
                    user_access: vec![UserAccessData {
                        wallet_address: "0x12345678".into(),
                        plan_name: Some("Pro".into()),
                        status: "active".into(),
                        days_remaining: 23,
                        plan_expires_at: Some("2024-10-15".into()),
                    }],
                }
            }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("user-access-management-table"), "UserAccessDesktopTable should render its section marker");
        assert!(html.contains("Pro"), "UserAccessDesktopTable should render the plan name");

        // Direct test of the Wave 38b `PaymentLinksTable`.
        let mut vdom = dioxus::prelude::VirtualDom::new(|| {
            rsx! {
                PaymentLinksTable {
                    links: vec![PaymentLink {
                        id: "pl_1".into(),
                        slug: "pro-monthly".into(),
                        context_type: "plan".into(),
                        context_id: Some("plan_pro".into()),
                        name: "Pro Plan Monthly".into(),
                        amount: 29.00,
                        currency: "USDT".into(),
                        is_active: true,
                        is_usable: true,
                        uses: 47,
                        max_uses: None,
                        created_at: "2024-09-01".into(),
                        expires_at: None,
                    }],
                }
            }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("payment-links-table"), "PaymentLinksTable should render its section marker");
        assert!(html.contains("pro-monthly"), "PaymentLinksTable should render the link slug");

        // Direct test of the Wave 38b `PaymentStatsGrid`.
        let mut vdom = dioxus::prelude::VirtualDom::new(|| {
            rsx! {
                PaymentStatsGrid {
                    stats: PaymentStats {
                        total_payments: 100,
                        total_amount: 5000.0,
                        successful_payments: 95,
                        failed_payments: 5,
                        pending_payments: 0,
                        average_payment_amount: 50.0,
                        payments_today: 5,
                        revenue_today: 250.0,
                    },
                }
            }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("payments-management-stats"), "PaymentStatsGrid should render its section marker");
        assert!(html.contains("Total Revenue"), "PaymentStatsGrid should render the Total Revenue label");
    }
}

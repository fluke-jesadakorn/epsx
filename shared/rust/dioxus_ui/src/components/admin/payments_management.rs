//! Admin `PaymentsManagement` family — Wave 38b T2 admin domain port.
//!
//! Mirrors `apps-old/admin-frontend/components/payments/payments-management.tsx`,
//! which exports 5 payment-management components:
//!
//! | Component | Use case |
//! | --- | --- |
//! | `PaymentStatsGrid` | 4-card stats header (Total Revenue / Successful / Pending / Today) |
//! | `PaymentFilterSection` | Search + status + method + plan filter row |
//! | `PaymentTableRow` | One row in the desktop payments table |
//! | `PaymentMobileCard` | One card in the mobile list |
//! | `PaymentPaginationBar` | Prev / Next page controls |
//!
//! The `PaymentsManagement` page wraps all of these plus an
//! action row (Refresh / Export CSV). It's the master container
//! used by `/admin/payments`.
//!
//! ## Status color helper
//!
//! `payment_status_class(status)` — maps a payment status string
//! to the design-system pill class (success / destructive / warning / muted).
//!
//! ## Tests
//!
//! Each component gets a colocated `#[cfg(test)] mod tests` block
//! with smoke render + key prop handling.

use dioxus::prelude::*;
use crate::primitives::icon::Icon;

// ============================================================================
// Status helper
// ============================================================================
//
// Returns the Tailwind pill class for a payment status. Mirrors
// the source's `getStatusColor`. Exposed publicly so admin pages
// can reuse the same color treatment.

/// Maps a payment status string to the Tailwind pill class.
/// Status values are normalized to lowercase before matching.
pub fn payment_status_class(status: &str) -> &'static str {
    match status.to_lowercase().as_str() {
        "succeeded" | "completed" => "bg-success/10 text-success border border-success/20",
        "failed" | "cancelled" | "expired" => "bg-destructive/10 text-destructive border border-destructive/20",
        "pending" | "processing" => "bg-warning/10 text-warning border border-warning/20",
        _ => "bg-muted text-muted-foreground border border-border/50",
    }
}

/// Currency formatter — adds the currency suffix and 2-decimals.
pub fn format_payment_currency(amount: f64, currency: &str) -> String {
    if currency.eq_ignore_ascii_case("USDT") {
        return format!("{amount:.2} USDT");
    }
    format!("{amount:.2} {currency}")
}

// ============================================================================
// PaymentStatsGrid
// ============================================================================
//
// 4-card stats header. Mirrors the source's `StatsGrid`.

#[derive(Clone, Debug, PartialEq)]
pub struct PaymentStats {
    pub total_payments: u32,
    pub total_amount: f64,
    pub successful_payments: u32,
    pub failed_payments: u32,
    pub pending_payments: u32,
    pub average_payment_amount: f64,
    pub payments_today: u32,
    pub revenue_today: f64,
}

#[component]
pub fn PaymentStatsGrid(stats: PaymentStats) -> Element {
    rsx! {
        div { class: "payments-management-stats grid grid-cols-2 lg:grid-cols-4 gap-4 mb-8",
            // Total Revenue
            div { class: "rounded-xl border border-border/20 bg-card p-5 overflow-hidden",
                div { class: "text-[10px] font-bold text-muted-foreground uppercase tracking-[0.2em] mb-3", "Total Revenue" }
                div { class: "text-2xl sm:text-3xl font-black text-[#1fc7d4] tracking-tight mb-1",
                    "{format_payment_currency(stats.total_amount, \"USD\")}"
                }
                div { class: "text-xs text-muted-foreground/60", "Platform Total" }
            }
            // Successful
            div { class: "rounded-xl border border-border/20 bg-card p-5 overflow-hidden",
                div { class: "text-[10px] font-bold text-muted-foreground uppercase tracking-[0.2em] mb-3", "Successful" }
                div { class: "text-2xl sm:text-3xl font-black text-[#31d0aa] tracking-tight mb-1",
                    "{stats.successful_payments}"
                }
                div { class: "text-xs text-muted-foreground/60", "Completed" }
            }
            // Pending
            div { class: "rounded-xl border border-border/20 bg-card p-5 overflow-hidden",
                div { class: "text-[10px] font-bold text-muted-foreground uppercase tracking-[0.2em] mb-3", "Pending" }
                div { class: "text-2xl sm:text-3xl font-black text-[#ffb237] tracking-tight mb-1",
                    "{stats.pending_payments}"
                }
                div { class: "text-xs text-muted-foreground/60", "In Progress" }
            }
            // Today
            div { class: "rounded-xl border border-border/20 bg-card p-5 overflow-hidden",
                div { class: "text-[10px] font-bold text-muted-foreground uppercase tracking-[0.2em] mb-3", "Today" }
                div { class: "text-2xl sm:text-3xl font-black text-[#ed4b9e] tracking-tight mb-1 truncate",
                    "{format_payment_currency(stats.revenue_today, \"USD\")}"
                }
                div { class: "text-xs text-muted-foreground/60", "Current Revenue" }
            }
        }
    }
}

// ============================================================================
// PaymentFilterSection
// ============================================================================
//
// Search + status + method + plan filter row. Mirrors the
// source's `FilterSection`.

#[derive(Clone, Debug, PartialEq, Default)]
pub struct PaymentFilters {
    pub status: String,
    pub payment_method: String,
    pub date_range: String,
    pub plan_template: String,
    pub search: String,
}

#[component]
pub fn PaymentFilterSection(
    filters: PaymentFilters,
    /// Called when any filter input changes.
    on_change: EventHandler<PaymentFilters>,
    on_reset: EventHandler<()>,
) -> Element {
    let emit = |f: PaymentFilters| {
        let f_clone = f.clone();
        move |_: String| on_change.call(f_clone.clone())
    };
    rsx! {
        div { class: "payments-management-filters rounded-xl border border-border/20 bg-card p-4 mb-6",
            div { class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-6 items-end",
                // Search
                div { class: "space-y-2",
                    label { class: "text-[10px] font-black text-muted-foreground uppercase tracking-[0.2em] ml-2", "Search" }
                    div { class: "relative group",
                        Icon { name: "search".to_string(), size: Some(16), class_name: Some("absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground group-focus-within:text-[#1fc7d4] transition-colors".to_string()) }
                        input {
                            class: "w-full pl-11 pr-4 py-3 bg-muted/30 border border-border/40 rounded-2xl text-foreground placeholder-muted-foreground/50 focus:outline-none focus:border-[#1fc7d4]/50 focus:bg-muted/50 transition-all font-bold text-sm",
                            r#type: "text",
                            placeholder: "Reference, wallet...",
                            value: "{filters.search}",
                        }
                    }
                }
                // Status
                div { class: "space-y-2",
                    label { class: "text-[10px] font-black text-muted-foreground uppercase tracking-[0.2em] ml-2", "Status" }
                    select {
                        class: "w-full px-4 py-3 bg-muted/30 border border-border/40 rounded-2xl text-foreground focus:outline-none focus:border-[#1fc7d4]/50 focus:bg-muted/50 transition-all font-bold text-sm",
                        value: "{filters.status}",
                        option { value: "", "All Status" }
                        option { value: "succeeded", "Succeeded" }
                        option { value: "pending", "Pending" }
                        option { value: "failed", "Failed" }
                    }
                }
                // Method
                div { class: "space-y-2",
                    label { class: "text-[10px] font-black text-muted-foreground uppercase tracking-[0.2em] ml-2", "Method" }
                    select {
                        class: "w-full px-4 py-3 bg-muted/30 border border-border/40 rounded-2xl text-foreground focus:outline-none focus:border-[#1fc7d4]/50 focus:bg-muted/50 transition-all font-bold text-sm",
                        value: "{filters.payment_method}",
                        option { value: "", "All Methods" }
                        option { value: "on_chain", "On Chain" }
                        option { value: "on_line", "Online" }
                    }
                }
                // Plan
                div { class: "space-y-2",
                    label { class: "text-[10px] font-black text-muted-foreground uppercase tracking-[0.2em] ml-2", "Plan" }
                    select {
                        class: "w-full px-4 py-3 bg-muted/30 border border-border/40 rounded-2xl text-foreground focus:outline-none focus:border-[#1fc7d4]/50 focus:bg-muted/50 transition-all font-bold text-sm",
                        value: "{filters.plan_template}",
                        option { value: "BASIC", "Basic" }
                        option { value: "PRO", "Pro" }
                        option { value: "ENTERPRISE", "Enterprise" }
                        option { value: "WHALE", "Whale" }
                    }
                }
                button {
                    class: "w-full px-4 py-3 bg-muted/30 border border-border/40 rounded-2xl text-muted-foreground hover:text-foreground hover:bg-muted/50 font-black text-xs uppercase tracking-widest transition-all",
                    r#type: "button",
                    onclick: move |_| on_reset.call(()),
                    "Reset"
                }
            }
        }
    }
}

// ============================================================================
// PaymentRow
// ============================================================================
//
// One row in the desktop payments table. Mirrors the source's
// `PaymentTableRow`.

#[derive(Clone, Debug, PartialEq)]
pub struct PaymentRow {
    pub id: String,
    pub payment_reference: String,
    pub wallet_address: String,
    pub plan_name: String,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub created_at: String,
    pub transaction_hash: Option<String>,
}

#[component]
pub fn PaymentTableRow(payment: PaymentRow) -> Element {
    let status_cls = payment_status_class(&payment.status);
    rsx! {
        tr { class: "payments-management-table-row hover:bg-muted/30 transition-colors",
            td { class: "px-4 py-4 whitespace-nowrap text-sm font-mono text-foreground", "{payment.payment_reference}" }
            td { class: "px-4 py-4 whitespace-nowrap",
                div { class: "text-xs font-mono text-muted-foreground", "{payment.wallet_address}" }
            }
            td { class: "px-4 py-4 whitespace-nowrap text-sm font-medium text-foreground", "{payment.plan_name}" }
            td { class: "px-6 py-5 whitespace-nowrap text-sm font-bold text-[#1fc7d4]", "{format_payment_currency(payment.amount, &payment.currency)}" }
            td { class: "px-4 py-4 whitespace-nowrap",
                span { class: "inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold {status_cls}",
                    "{payment.status}"
                }
            }
            td { class: "px-4 py-4 whitespace-nowrap text-sm text-muted-foreground", "{payment.created_at}" }
            td { class: "px-4 py-4 whitespace-nowrap",
                if let Some(hash) = payment.transaction_hash.clone() {
                    if !hash.is_empty() {
                        a {
                            class: "inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs font-semibold text-[#1fc7d4] border border-[#1fc7d4]/30 hover:bg-[#1fc7d4]/10 transition-colors",
                            href: "#explorer-{hash}",
                            Icon { name: "external-link".to_string(), size: Some(14), class_name: Some("w-3.5 h-3.5".to_string()) }
                            "Explorer"
                        }
                    } else {
                        span { class: "text-xs text-muted-foreground/40", "—" }
                    }
                } else {
                    span { class: "text-xs text-muted-foreground/40", "—" }
                }
            }
        }
    }
}

// ============================================================================
// PaymentMobileCard
// ============================================================================
//
// One card in the mobile list. Mirrors the source's
// `PaymentMobileCard`.

#[component]
pub fn PaymentMobileCard(payment: PaymentRow) -> Element {
    let status_cls = payment_status_class(&payment.status);
    rsx! {
        div { class: "payments-management-mobile-card p-4 bg-muted/30 border border-border/50 rounded-2xl",
            div { class: "flex items-center justify-between mb-3",
                span { class: "font-mono text-xs text-muted-foreground", "{payment.payment_reference}" }
                span { class: "inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold {status_cls}",
                    "{payment.status}"
                }
            }
            div { class: "grid grid-cols-2 gap-3",
                div { class: "bg-card rounded-xl p-3 border border-border/50",
                    div { class: "text-sm font-medium text-muted-foreground", "Amount" }
                    div { class: "text-lg font-bold text-primary", "{format_payment_currency(payment.amount, &payment.currency)}" }
                }
                div { class: "bg-card rounded-xl p-3 border border-border/50",
                    div { class: "text-sm font-medium text-muted-foreground", "Plan" }
                    div { class: "text-lg font-bold text-secondary", "{payment.plan_name}" }
                }
            }
            div { class: "mt-3 flex items-center justify-between",
                span { class: "text-xs text-muted-foreground", "{payment.created_at}" }
                if let Some(hash) = payment.transaction_hash.clone() {
                    if !hash.is_empty() {
                        a {
                            class: "inline-flex items-center gap-1 px-2.5 py-1 rounded-lg text-xs font-semibold text-[#1fc7d4] border border-[#1fc7d4]/30 hover:bg-[#1fc7d4]/10 transition-colors",
                            href: "#explorer-{hash}",
                            Icon { name: "external-link".to_string(), size: Some(12), class_name: Some("w-3 h-3".to_string()) }
                            "Explorer"
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// PaymentPaginationBar
// ============================================================================
//
// Prev / Next page controls. Renders nothing when `total <= 1`.

#[component]
pub fn PaymentPaginationBar(
    current: u32,
    total: u32,
    on_change: EventHandler<u32>,
) -> Element {
    if total <= 1 {
        return rsx! { Fragment {} };
    }
    rsx! {
        div { class: "payments-management-pagination mt-6 flex items-center justify-between border-t border-border/50 pt-6",
            p { class: "text-sm text-muted-foreground",
                "Page "
                span { class: "font-semibold text-foreground", "{current}" }
                " of "
                span { class: "font-semibold text-foreground", "{total}" }
            }
            div { class: "flex gap-2",
                button {
                    class: "px-4 py-2 text-sm font-medium rounded-xl bg-muted text-muted-foreground hover:bg-muted/80 disabled:opacity-50 disabled:cursor-not-allowed transition-colors border border-border/50",
                    r#type: "button",
                    disabled: current == 1,
                    onclick: move |_| {
                        if current > 1 { on_change.call(current - 1); }
                    },
                    "Previous"
                }
                button {
                    class: "px-4 py-2 text-sm font-medium rounded-xl bg-primary text-primary-foreground hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors",
                    r#type: "button",
                    disabled: current == total,
                    onclick: move |_| {
                        if current < total { on_change.call(current + 1); }
                    },
                    "Next"
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

    fn sample_stats() -> PaymentStats {
        PaymentStats {
            total_payments: 1234,
            total_amount: 45231.0,
            successful_payments: 1180,
            failed_payments: 42,
            pending_payments: 12,
            average_payment_amount: 36.66,
            payments_today: 18,
            revenue_today: 2310.0,
        }
    }

    fn sample_row() -> PaymentRow {
        PaymentRow {
            id: "pi_1".to_string(),
            payment_reference: "pi_abc123".to_string(),
            wallet_address: "0x1234\u{2026}5678".to_string(),
            plan_name: "Pro".to_string(),
            amount: 29.0,
            currency: "USDT".to_string(),
            status: "succeeded".to_string(),
            created_at: "2024-09-20 10:32".to_string(),
            transaction_hash: Some("0xdeadbeef".to_string()),
        }
    }
    /// `PaymentStatsGrid` renders 4 stat cards with the design-system
    /// gradient text colors.
    #[test]
    fn payment_stats_grid_renders_4_cards() {
        fn harness() -> Element {
            rsx! { PaymentStatsGrid { stats: sample_stats() } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Total Revenue"), "PaymentStatsGrid must render Total Revenue. Got: {html}");
        assert!(html.contains("Successful"), "PaymentStatsGrid must render Successful. Got: {html}");
        assert!(html.contains("Pending"), "PaymentStatsGrid must render Pending. Got: {html}");
        assert!(html.contains("Today"), "PaymentStatsGrid must render Today. Got: {html}");
        assert!(html.contains("45231.00"), "PaymentStatsGrid must render total amount. Got: {html}");
        assert!(html.contains("text-[#1fc7d4]"), "PaymentStatsGrid must use cyan accent. Got: {html}");
    }

    /// `PaymentFilterSection` renders the 4 filter labels + Reset.
    #[test]
    fn payment_filter_section_renders_all_filters() {
        fn harness() -> Element {
            rsx! { PaymentFilterSection { filters: PaymentFilters::default(), on_change: move |_: PaymentFilters| {}, on_reset: move |_: ()| {} } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        for label in &["Search", "Status", "Method", "Plan", "Reset"] {
            assert!(html.contains(label), "PaymentFilterSection must render `{label}`. Got: {html}");
        }
        assert!(html.contains("All Status"), "PaymentFilterSection must render status options. Got: {html}");
        assert!(html.contains("All Methods"), "PaymentFilterSection must render method options. Got: {html}");
    }

    /// `PaymentTableRow` renders all 7 column cells.
    #[test]
    fn payment_table_row_renders_all_cells() {
        fn harness() -> Element {
            rsx! { PaymentTableRow { payment: sample_row() } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("pi_abc123"), "PaymentTableRow must render payment_reference. Got: {html}");
        assert!(html.contains("Pro"), "PaymentTableRow must render plan_name. Got: {html}");
        assert!(html.contains("29.00 USDT"), "PaymentTableRow must render formatted amount. Got: {html}");
        assert!(html.contains("succeeded"), "PaymentTableRow must render status. Got: {html}");
        assert!(html.contains("bg-success/10"), "PaymentTableRow must apply success status class. Got: {html}");
        assert!(html.contains("Explorer"), "PaymentTableRow must render Explorer link. Got: {html}");
    }

    /// `PaymentTableRow` with failed status applies destructive class.
    #[test]
    fn payment_table_row_uses_destructive_class_for_failed() {
        fn harness() -> Element {
            let mut payment = sample_row();
            payment.status = "failed".to_string();
            rsx! { PaymentTableRow { payment } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("bg-destructive/10"), "PaymentTableRow failed must use destructive class. Got: {html}");
    }

    /// `PaymentTableRow` with no transaction_hash shows "—".
    #[test]
    fn payment_table_row_renders_dash_when_no_hash() {
        fn harness() -> Element {
            let mut payment = sample_row();
            payment.transaction_hash = None;
            rsx! { PaymentTableRow { payment } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("\u{2014}"), "PaymentTableRow must render dash when no hash. Got: {html}");
        assert!(!html.contains("Explorer"), "PaymentTableRow must not render Explorer link. Got: {html}");
    }

    /// `PaymentMobileCard` renders the mobile-friendly card layout.
    #[test]
    fn payment_mobile_card_renders_layout() {
        fn harness() -> Element {
            rsx! { PaymentMobileCard { payment: sample_row() } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("payments-management-mobile-card"), "PaymentMobileCard must render section class. Got: {html}");
        assert!(html.contains("Amount"), "PaymentMobileCard must render Amount label. Got: {html}");
        assert!(html.contains("Plan"), "PaymentMobileCard must render Plan label. Got: {html}");
    }

    /// `PaymentPaginationBar` renders nothing when total <= 1.
    #[test]
    fn payment_pagination_bar_hidden_when_single_page() {
        fn harness() -> Element {
            rsx! { PaymentPaginationBar { current: 1u32, total: 1u32, on_change: move |_: u32| {} } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(!html.contains("payments-management-pagination"), "PaymentPaginationBar must hide when total <= 1. Got: {html}");
    }

    /// `PaymentPaginationBar` renders Prev / Next when total > 1.
    #[test]
    fn payment_pagination_bar_renders_prev_next() {
        fn harness() -> Element {
            rsx! { PaymentPaginationBar { current: 2u32, total: 5u32, on_change: move |_: u32| {} } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Previous"), "PaymentPaginationBar must render Previous. Got: {html}");
        assert!(html.contains("Next"), "PaymentPaginationBar must render Next. Got: {html}");
        assert!(html.contains("Page "), "PaymentPaginationBar must render page indicator. Got: {html}");
    }

    /// `payment_status_class` returns the expected class per status.
    #[test]
    fn payment_status_class_matches_source() {
        assert_eq!(payment_status_class("succeeded"), "bg-success/10 text-success border border-success/20");
        assert_eq!(payment_status_class("completed"), "bg-success/10 text-success border border-success/20");
        assert_eq!(payment_status_class("failed"), "bg-destructive/10 text-destructive border border-destructive/20");
        assert_eq!(payment_status_class("expired"), "bg-destructive/10 text-destructive border border-destructive/20");
        assert_eq!(payment_status_class("pending"), "bg-warning/10 text-warning border border-warning/20");
        assert_eq!(payment_status_class("processing"), "bg-warning/10 text-warning border border-warning/20");
        assert_eq!(payment_status_class("unknown"), "bg-muted text-muted-foreground border border-border/50");
        assert_eq!(payment_status_class("SUCCEEDED"), "bg-success/10 text-success border border-success/20");
    }

    /// `format_payment_currency` formats USDT with suffix, others
    /// with 2-decimals + suffix.
    #[test]
    fn format_payment_currency_formats_usdt() {
        assert_eq!(format_payment_currency(29.0, "USDT"), "29.00 USDT");
        assert_eq!(format_payment_currency(100.5, "USD"), "100.50 USD");
    }
}

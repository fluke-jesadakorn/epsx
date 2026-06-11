//! Sub-components for `/admin/wallet-management/wallets` — Wave 6C Track D.
//!
//! 1:1 mirror of `apps-old/admin-frontend/components/wallet/*.tsx`:
//!   1. `WalletStatsBar`         — 4 stat cards + platform-distribution sub-card
//!   2. `WalletList`             — DataTable of all wallets
//!   3. `WalletDetailView`       — per-wallet detail page header + tab nav
//!   4. `WalletTableRow`         — row inside the list table
//!   5. `WalletCardSections`     — mobile card variant
//!   6. `WalletDetailPanel`      — right-hand side of detail view
//!   7. `WalletDisableDialog`    — disable modal
//!   8. `WalletReenableDialog`   — re-enable modal
//!
//! The `WalletStatsData` struct is `pub` so the parent page can build
//! it from BFF-provided numbers. The helper components
//! (`PlatformDistributionRow`, `WalletCardStat`, `QuickAccessRow`,
//! `WalletTransactionTable`, `DetailField`) stay private to this
//! module — they're implementation details of the 8 public sub-components.

use crate::primitives::*;
use crate::primitives::admin_metric_card::{AdminMetricCard, MetricTrend};
use crate::data_table::{Column, DataTable, Row, SortDir};

use dioxus::prelude::*;

// ============================================================================
// Section 1: WalletStatsBar
// ============================================================================

/// Stats payload for `WalletStatsBar`. Mirrors a subset of
/// `WalletStats` from `components/wallet/types.ts`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WalletStatsData {
    pub total: u32,
    pub active: u32,
    pub disabled: u32,
    pub subscribed: u32,
    pub total_change: f32,
    pub active_change: f32,
    pub disabled_change: f32,
    pub subscribed_change: f32,
    pub platform_analytics: u32,
    pub platform_pay: u32,
    pub platform_token: u32,
    pub platform_markets: u32,
}

#[component]
pub fn WalletStatsBar(stats: WalletStatsData) -> Element {
    let total = stats.platform_analytics + stats.platform_pay + stats.platform_token + stats.platform_markets;
    let pct = |n: u32| -> u32 { if total == 0 { 0 } else { (n * 100) / total } };
    rsx! {
        div { class: "space-y-4 wallet-stats-bar",
            div { class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4",
                AdminMetricCard {
                    label: "Total wallets".to_string(),
                    value: format!("{}", stats.total),
                    trend: Some(MetricTrend::Up(stats.total_change)),
                    sparkline_data: Some(vec![12.0, 14.0, 13.0, 15.0, 17.0, 16.0, 18.0]),
                    icon: Some("wallet".to_string()),
                }
                AdminMetricCard {
                    label: "Active".to_string(),
                    value: format!("{}", stats.active),
                    trend: Some(MetricTrend::Up(stats.active_change)),
                    sparkline_data: Some(vec![10.0, 11.0, 11.5, 12.0, 12.5, 13.0, 13.5]),
                    icon: Some("users".to_string()),
                }
                AdminMetricCard {
                    label: "Disabled".to_string(),
                    value: format!("{}", stats.disabled),
                    trend: Some(if stats.disabled_change > 0.0 { MetricTrend::Down(stats.disabled_change.abs()) } else { MetricTrend::Up(stats.disabled_change) }),
                    sparkline_data: Some(vec![3.0, 3.0, 2.5, 2.0, 2.0, 1.5, 1.5]),
                    icon: Some("alert-triangle".to_string()),
                }
                AdminMetricCard {
                    label: "Subscribed".to_string(),
                    value: format!("{}", stats.subscribed),
                    trend: Some(MetricTrend::Up(stats.subscribed_change)),
                    sparkline_data: Some(vec![5.0, 6.0, 6.0, 7.0, 7.5, 8.0, 8.5]),
                    icon: Some("package".to_string()),
                }
            }
            div { class: "card card-glass platform-distribution-card",
                div { class: "card-header",
                    h4 { class: "text-sm font-semibold text-foreground/80", "Platform distribution" }
                }
                div { class: "card-body space-y-3",
                    PlatformDistributionRow { label: "Analytics".to_string(), emoji: "\u{1f4ca}".to_string(), count: stats.platform_analytics, percent: pct(stats.platform_analytics), color: "bg-[#1fc7d4]".to_string() }
                    PlatformDistributionRow { label: "Pay".to_string(), emoji: "\u{1f4b3}".to_string(), count: stats.platform_pay, percent: pct(stats.platform_pay), color: "bg-[#7645d9]".to_string() }
                    PlatformDistributionRow { label: "Token".to_string(), emoji: "\u{1fa99}".to_string(), count: stats.platform_token, percent: pct(stats.platform_token), color: "bg-[#ffb237]".to_string() }
                    PlatformDistributionRow { label: "Markets".to_string(), emoji: "\u{1f4c8}".to_string(), count: stats.platform_markets, percent: pct(stats.platform_markets), color: "bg-[#31d0aa]".to_string() }
                }
            }
        }
    }
}

#[component]
fn PlatformDistributionRow(label: String, emoji: String, count: u32, percent: u32, color: String) -> Element {
    rsx! {
        div {
            div { class: "flex items-center justify-between text-sm mb-1",
                span { class: "flex items-center gap-2 text-muted-foreground",
                    span { "{emoji}" }
                    "{label}"
                }
                span { class: "font-medium text-foreground", "{count} ({percent}%)" }
            }
            div { class: "h-2 bg-muted rounded-full overflow-hidden",
                div { class: "h-full rounded-full transition-all duration-500 {color}", style: "width: {percent}%" }
            }
        }
    }
}

// ============================================================================
// Section 2: WalletList
// ============================================================================

#[component]
pub fn WalletList() -> Element {
    let columns = vec![
        Column { key: "address".into(), label: "Address".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("30%".into()), class_name: None },
        Column { key: "chain".into(), label: "Chain".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("10%".into()), class_name: None },
        Column { key: "balance".into(), label: "Balance".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("15%".into()), class_name: None },
        Column { key: "status".into(), label: "Status".into(), sortable: true, align: crate::primitives::data_table::Align::Center, width: Some("10%".into()), class_name: None },
        Column { key: "permissions".into(), label: "Permissions".into(), sortable: false, align: crate::primitives::data_table::Align::Left, width: Some("20%".into()), class_name: None },
        Column { key: "last_active".into(), label: "Last active".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("15%".into()), class_name: None },
    ];
    let rows = vec![
        Row { id: "0x1234567890abcdef1234567890abcdef12345678".into(), cells: vec!["0x1234\u{2026}5678".into(), "BSC".into(), "1.234 BNB".into(), "Active".into(), "trade, view, pay".into(), "2 min ago".into()] },
        Row { id: "0xabcdef1234567890abcdef1234567890abcdef12".into(), cells: vec!["0xabcd\u{2026}ef12".into(), "BSC".into(), "0.5 BNB".into(), "Active".into(), "trade".into(), "1 hour ago".into()] },
        Row { id: "0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef".into(), cells: vec!["0xdead\u{2026}beef".into(), "BSC".into(), "0.0 BNB".into(), "Disabled".into(), "\u{2014}".into(), "1 day ago".into()] },
    ];
    rsx! {
        DataTable {
            columns,
            rows,
            striped: true,
            page_size: 20,
            filter_placeholder: Some("Filter by address, status, or permission...".to_string()),
            initial_sort: Some(("last_active".to_string(), SortDir::Desc)),
        }
    }
}

// ============================================================================
// Section 3: WalletDetailView
// ============================================================================

#[component]
pub fn WalletDetailView(address: String) -> Element {
    let mut tab = use_signal(|| "overview".to_string());
    rsx! {
        div { class: "wallet-detail-view space-y-4",
            div { class: "card card-glass",
                div { class: "card-header",
                    h1 { class: "card-title", "Wallet" }
                    code { class: "text-sm text-muted-foreground", "{address}" }
                }
                div { class: "card-body grid grid-cols-1 md:grid-cols-3 gap-4",
                    DetailField { label: "Address".to_string(), value: address.clone() }
                    DetailField { label: "Chain".to_string(), value: "BSC (56)".to_string() }
                    DetailField { label: "Status".to_string(), value: "Active".to_string() }
                    DetailField { label: "Balance".to_string(), value: "1.234 BNB".to_string() }
                    DetailField { label: "Created".to_string(), value: "2024-01-15".to_string() }
                    DetailField { label: "Last active".to_string(), value: "2 min ago".to_string() }
                }
            }
            div { class: "tabs mt-4 mb-4",
                button { class: if *tab.read() == "overview" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("overview".to_string()), "Overview" }
                button { class: if *tab.read() == "tx" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("tx".to_string()), "Transactions" }
                button { class: if *tab.read() == "subs" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("subs".to_string()), "Subscriptions" }
                button { class: if *tab.read() == "perms" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("perms".to_string()), "Permissions" }
            }
            if *tab.read() == "overview" {
                WalletDetailPanel { address: address.clone() }
            } else if *tab.read() == "tx" {
                WalletTransactionTable {}
            } else if *tab.read() == "subs" {
                div { class: "card card-glass", div { class: "card-body",
                    p { "Active: Pro plan ($29/mo)" }
                    p { class: "text-muted-foreground text-sm", "Next billing: 2024-10-15" }
                } }
            } else {
                div { class: "card card-glass", div { class: "card-body flex gap-2",
                    Badge { kind: BadgeKind::Success, "trade" }
                    Badge { kind: BadgeKind::Info, "view" }
                    Badge { kind: BadgeKind::Warning, "pay" }
                } }
            }
            div { class: "mt-4",
                a { class: "btn btn-danger", href: format!("/wallet-management/wallets/{}/disable", address), "Disable wallet" }
            }
        }
    }
}

// ============================================================================
// Section 4: WalletTableRow
// ============================================================================

#[component]
pub fn WalletTableRow(
    address: String,
    plan: String,
    status: String,
    last_active: String,
) -> Element {
    let status_cls = if status == "active" { "text-success" } else { "text-warning" };
    let dot_cls = if status == "active" { "bg-success" } else { "bg-warning" };
    rsx! {
        div { class: "wallet-table-row",
            div { class: "wallet-table-row-address font-mono text-xs", "{address}" }
            div { class: "wallet-table-row-plan",
                Badge { kind: BadgeKind::Primary, "{plan}" }
            }
            div { class: "wallet-table-row-status flex items-center gap-2",
                div { class: "h-1.5 w-1.5 rounded-full {dot_cls}" }
                span { class: "capitalize text-xs font-medium {status_cls}", "{status}" }
            }
            div { class: "wallet-table-row-last-login text-right text-xs text-muted-foreground", "{last_active}" }
        }
    }
}

// ============================================================================
// Section 5: WalletCardSections
// ============================================================================

#[component]
pub fn WalletCardSections(
    address: String,
    label: String,
    plan: String,
    joined: String,
    last_login: String,
) -> Element {
    let initials: String = address.chars().skip(2).take(2).collect::<String>().to_uppercase();
    rsx! {
        div { class: "card card-glass wallet-card-sections p-4 space-y-4",
            div { class: "wallet-card-identity flex items-center gap-3 min-w-0",
                div { class: "wallet-card-avatar",
                    div { class: "wallet-card-avatar-bg" }
                    div { class: "wallet-card-avatar-text", "{initials}" }
                }
                div { class: "min-w-0 flex-1",
                    div { class: "font-mono text-sm font-bold truncate", "{address}" }
                    if !label.is_empty() {
                        Badge { kind: BadgeKind::Primary, "{label}" }
                    } else {
                        span { class: "text-xs text-muted-foreground", "Add label" }
                    }
                }
            }
            div { class: "grid grid-cols-2 sm:grid-cols-4 gap-3",
                WalletCardStat { label: "Plan".to_string(), value: plan.clone() }
                WalletCardStat { label: "Joined".to_string(), value: joined.clone() }
                WalletCardStat { label: "Last login".to_string(), value: last_login.clone() }
                WalletCardStat { label: "Platforms".to_string(), value: "Analytics, Pay".to_string() }
            }
            div { class: "grid grid-cols-2 gap-3",
                button { class: "btn btn-outline", r#type: "button", Icon { name: "edit".to_string(), size: Some(14) } " Edit" }
                button { class: "btn btn-outline", r#type: "button", Icon { name: "more-horizontal".to_string(), size: Some(14) } " More actions" }
            }
        }
    }
}

#[component]
fn WalletCardStat(label: String, value: String) -> Element {
    rsx! {
        div { class: "flex flex-col gap-1",
            span { class: "text-[10px] font-bold uppercase tracking-wider text-muted-foreground", "{label}" }
            span { class: "text-sm font-semibold truncate", "{value}" }
        }
    }
}

// ============================================================================
// Section 6: WalletDetailPanel
// ============================================================================

#[component]
pub fn WalletDetailPanel(address: String) -> Element {
    rsx! {
        div { class: "wallet-detail-panel space-y-4",
            div { class: "card card-glass",
                div { class: "card-header",
                    h3 { class: "card-title", "Subscription" }
                    Badge { kind: BadgeKind::Success, "Active" }
                }
                div { class: "card-body",
                    p { class: "text-sm text-muted-foreground", "Wallet" }
                    code { class: "text-xs", "{address}" }
                    div { class: "mt-3 grid grid-cols-2 gap-3",
                        DetailField { label: "Plan".to_string(), value: "Pro".to_string() }
                        DetailField { label: "Billed".to_string(), value: "$29 / mo".to_string() }
                        DetailField { label: "Renews".to_string(), value: "2024-10-15".to_string() }
                        DetailField { label: "Trial ends".to_string(), value: "\u{2014}".to_string() }
                    }
                }
            }
            div { class: "card card-glass",
                div { class: "card-header",
                    h3 { class: "card-title", "Recent transactions" }
                }
                div { class: "card-body p-0",
                    WalletTransactionTable {}
                }
            }
            div { class: "card card-glass",
                div { class: "card-header",
                    h3 { class: "card-title", "Quick access" }
                }
                div { class: "card-body space-y-2",
                    QuickAccessRow { label: "View on explorer".to_string(), href: format!("https://bscscan.com/address/{}", address) }
                    QuickAccessRow { label: "Download transactions CSV".to_string(), href: format!("/api/v1/wallet/wallets/{}/transactions.csv", address) }
                    QuickAccessRow { label: "Open support ticket".to_string(), href: "/chat".to_string() }
                }
            }
        }
    }
}

#[component]
fn QuickAccessRow(label: String, href: String) -> Element {
    rsx! {
        a { class: "flex items-center justify-between p-2 rounded hover:bg-muted/30 text-sm", href: "{href}",
            span { "{label}" }
            Icon { name: "arrow-up-right".to_string(), size: Some(14) }
        }
    }
}

#[component]
fn WalletTransactionTable() -> Element {
    rsx! {
        div { class: "table-wrap",
            table { class: "table",
                thead { tr { th { "Time" } th { "Type" } th { "Amount" } th { "Token" } th { "Hash" } } }
                tbody {
                    tr { td { "2024-09-20 10:32" } td { "in" } td { class: "font-mono", "+0.5" } td { "BNB" } td { code { class: "text-xs", "0xabc...123" } } }
                    tr { td { "2024-09-19 15:21" } td { "out" } td { class: "font-mono", "-0.2" } td { "BNB" } td { code { class: "text-xs", "0xdef...456" } } }
                    tr { td { "2024-09-18 09:14" } td { "in" } td { class: "font-mono", "+0.1" } td { "BNB" } td { code { class: "text-xs", "0x789...abc" } } }
                }
            }
        }
    }
}

// ============================================================================
// Section 7: WalletDisableDialog
// ============================================================================

#[component]
pub fn WalletDisableDialog(address: String) -> Element {
    rsx! {
        div { class: "alert-dialog wallet-disable-dialog",
            div { class: "alert-dialog-content",
                div { class: "alert-dialog-header",
                    h2 { class: "alert-dialog-title", "Disable wallet" }
                    p { class: "alert-dialog-description",
                        "You are about to disable the wallet "
                        span { class: "font-mono", "{address}" }
                        ". This will revoke all permissions and freeze any active subscriptions."
                    }
                }
                Form { method: "POST".to_string(), action: format!("/api/v1/wallet/wallets/{}/disable", address),
                    div { class: "field",
                        label { class: "field-label", "Reason" }
                        input { class: "input", name: "reason", required: true, placeholder: "e.g. suspicious activity" }
                    }
                    FormActions {
                        button { class: "btn btn-danger", r#type: "submit", "Disable wallet" }
                        a { class: "btn btn-outline", href: format!("/wallet-management/wallets/{}", address), "Cancel" }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Section 8: WalletReenableDialog
// ============================================================================

#[component]
pub fn WalletReenableDialog(address: String) -> Element {
    rsx! {
        div { class: "alert-dialog wallet-reenable-dialog",
            div { class: "alert-dialog-content",
                div { class: "alert-dialog-header",
                    h2 { class: "alert-dialog-title", "Re-enable wallet" }
                    p { class: "alert-dialog-description",
                        "Re-enable the wallet "
                        span { class: "font-mono", "{address}" }
                        ". The previous disable reason will be cleared from the audit log."
                    }
                }
                Form { method: "POST".to_string(), action: format!("/api/v1/wallet/wallets/{}/reenable", address),
                    div { class: "field",
                        label { class: "field-label", "Note (optional)" }
                        input { class: "input", name: "note", placeholder: "e.g. verified identity" }
                    }
                    FormActions {
                        button { class: "btn btn-success", r#type: "submit", "Re-enable wallet" }
                        a { class: "btn btn-outline", href: format!("/wallet-management/wallets/{}", address), "Cancel" }
                    }
                }
            }
        }
    }
}

// ============================================================================
// DetailField helper (shared)
// ============================================================================

#[component]
pub fn DetailField(label: String, value: String) -> Element {
    rsx! {
        div { class: "detail-field",
            div { class: "text-sm text-muted-foreground", "{label}" }
            div { class: "font-semibold", "{value}" }
        }
    }
}

// ============================================================================
// Tests — one per sub-component
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::PageContext;
    use crate::auth::User;

    fn test_admin() -> User {
        User {
            id: "u1".to_string(),
            address: "0xADMIN0000000000000000000000000000000001".to_string(),
            chain_id: "56".to_string(),
            roles: vec!["admin".to_string()],
            email: Some("admin@epsx.io".to_string()),
            tier: Some("admin".to_string()),
            permissions: vec!["wallets:manage".to_string()],
            ..Default::default()
        }
    }

    fn render_to_string(el: Element) -> String {
        dioxus_ssr::render_element(el)
    }

    fn sample_stats() -> WalletStatsData {
        WalletStatsData {
            total: 100, active: 80, disabled: 20, subscribed: 60,
            total_change: 1.0, active_change: 1.0, disabled_change: -1.0, subscribed_change: 1.0,
            platform_analytics: 50, platform_pay: 30, platform_token: 15, platform_markets: 5,
        }
    }

    #[test]
    fn test_render_smoke_wallet_stats_bar() {
        let el = rsx! { WalletStatsBar { stats: sample_stats() } };
        let html = render_to_string(el);
        assert!(html.contains("Platform distribution"), "WalletStatsBar must render the platform distribution sub-card. Got: {}", html);
        assert!(html.contains("Total wallets"), "WalletStatsBar must render the 'Total wallets' card. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_wallet_list() {
        let el = rsx! { WalletList {} };
        let html = render_to_string(el);
        assert!(html.contains("Filter by address, status, or permission..."), "WalletList must render the filter placeholder. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_wallet_detail_view() {
        let el = rsx! { WalletDetailView { address: "0xABCD".to_string() } };
        let html = render_to_string(el);
        assert!(html.contains("Overview"), "WalletDetailView must render the Overview tab. Got: {}", html);
        assert!(html.contains("0xABCD"), "WalletDetailView must render the address. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_wallet_table_row() {
        let el = rsx! { WalletTableRow { address: "0x1234".to_string(), plan: "Pro".to_string(), status: "active".to_string(), last_active: "2m ago".to_string() } };
        let html = render_to_string(el);
        assert!(html.contains("wallet-table-row"), "WalletTableRow must render its class. Got: {}", html);
        assert!(html.contains("0x1234"), "WalletTableRow must render the address. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_wallet_card_sections() {
        let el = rsx! { WalletCardSections { address: "0xABCD1234".to_string(), label: "Treasury".to_string(), plan: "Pro".to_string(), joined: "30 days ago".to_string(), last_login: "5 min ago".to_string() } };
        let html = render_to_string(el);
        assert!(html.contains("0xABCD1234"), "WalletCardSections must render the address. Got: {}", html);
        assert!(html.contains("wallet-card-sections"), "WalletCardSections must render its class. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_wallet_detail_panel() {
        let el = rsx! { WalletDetailPanel { address: "0x1234".to_string() } };
        let html = render_to_string(el);
        assert!(html.contains("Subscription"), "WalletDetailPanel must render the subscription card. Got: {}", html);
        assert!(html.contains("Recent transactions"), "WalletDetailPanel must render the recent transactions card. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_wallet_disable_dialog() {
        let el = rsx! { WalletDisableDialog { address: "0xABCD".to_string() } };
        let html = render_to_string(el);
        assert!(html.contains("Disable wallet"), "WalletDisableDialog must render the title. Got: {}", html);
        assert!(html.contains("wallet-disable-dialog"), "WalletDisableDialog must render its class. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_wallet_reenable_dialog() {
        let el = rsx! { WalletReenableDialog { address: "0xABCD".to_string() } };
        let html = render_to_string(el);
        assert!(html.contains("Re-enable wallet"), "WalletReenableDialog must render the title. Got: {}", html);
        assert!(html.contains("wallet-reenable-dialog"), "WalletReenableDialog must render its class. Got: {}", html);
    }

    /// `test_section_markers` for the parent page — keep it here as a
    /// re-export shim so the existing parent-page test still passes.
    #[test]
    fn test_section_markers_parent_page() {
        let ctx = PageContext {
            user: Some(test_admin()),
            path: "/wallet-management/wallets".to_string(),
            ..Default::default()
        };
        // Use the re-exported render fn via the parent module path.
        let (_, el) = crate::pages::admin_pages::wallet_wallets::render(&ctx);
        let html = render_to_string(el);
        assert!(html.contains("Platform distribution"), "section 1 (WalletStatsBar) marker missing");
        assert!(html.contains("Total wallets"), "section 1 (WalletStatsBar) stat card missing");
        assert!(html.contains("Filter by address, status, or permission..."), "section 2 (WalletList) marker missing");
    }
}

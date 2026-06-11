//! Admin dashboard sub-components — 1:1 mirror of
//! `apps-old/admin-frontend/components/admin/dashboard-*.tsx`.
//!
//! Extracted from `pages/admin_pages/dashboard.rs` (Wave 6B Track A)
//! per the Wave 6C design doc §"Track B" line 230:
//! - `AdminStatsCards` — 5-up stat row
//!   (Active wallets, Total volume, Active plans, Open tickets,
//!   API calls today). Source: `dashboard-hud-metrics.tsx` extended
//!   to 5 cards per the design-doc Track A spec.
//! - `WalletsByChain` — donut + per-chain legend
//!   (BSC / ETH / Polygon / Arbitrum / Base).
//! - `RecentTransactions` — 6-col transactions table
//!   (Time / Type / From / To / Amount / Status).
//! - `SystemAlerts` — top alerts list
//!   (Info / Warning / Danger levels).
//! - `ActivityStream` — "Global Event Stream" sidebar feed
//!   (recent wallet auth events).
//! - `AdminPulseHeader` — Wave 6B dashboard's pulse header card
//!   (title + status pill + Latency/Uptime/Alerts stat cluster).
//!   Re-exported here so the dashboard page's only "non-1:1" bit
//!   also lives under the cluster.

use dioxus::prelude::*;

use crate::primitives::icon::Icon;
use crate::primitives::stat_card::StatCard;
use crate::charts::ChartDonut;

// ===== AdminPulseHeader ====================================================
//
// Source: `dashboard-pulse-header.tsx` (70 LoC) — header card with
// title, "OPERATIONAL" / "DEGRADED" pill (animated dot), and a
// right-side stats cluster (Latency, Uptime, Alerts).

/// Command Center pulse header — the page's "title card" with the
/// status pill + Latency/Uptime/Alerts stat cluster.
///
/// In the source, the pulse is fed by `DashboardStats` (live data).
/// The Dioxus port renders a static OPERATIONAL pill (the live
/// version would plumb the same `stats.systemHealth >= 90` check).
#[component]
pub fn AdminPulseHeader() -> Element {
    rsx! {
        div { class: "card card-glass admin-pulse-header mb-6",
            div { class: "card-body",
                div { class: "flex items-center justify-between flex-wrap gap-4",
                    div {
                        div { class: "flex items-center gap-3 mb-2",
                            h1 { class: "text-2xl font-bold", "Command Center" }
                            span { class: "pulse-indicator",
                                span { class: "pulse-dot" }
                                span { "OPERATIONAL" }
                            }
                        }
                        p { class: "text-muted-foreground text-sm font-mono", "Real-time visibility into your platform" }
                    }
                    div { class: "flex items-center divide-x divide-border rounded-xl border border-border bg-background/50 p-2",
                        div { class: "px-4 text-center",
                            div { class: "text-[10px] font-bold uppercase tracking-widest text-muted-foreground mb-1", "Latency" }
                            div { class: "font-mono font-bold text-primary", "42ms" }
                        }
                        div { class: "px-4 text-center",
                            div { class: "text-[10px] font-bold uppercase tracking-widest text-muted-foreground mb-1", "Uptime" }
                            div { class: "font-mono font-bold text-success", "99.97%" }
                        }
                        div { class: "px-4 text-center",
                            div { class: "text-[10px] font-bold uppercase tracking-widest text-muted-foreground mb-1", "Alerts" }
                            div { class: "font-mono font-bold text-warning", "0" }
                        }
                    }
                }
            }
        }
    }
}

// ===== AdminStatsCards =====================================================
//
// Source: `dashboard-hud-metrics.tsx` (80 LoC, 4 cards) extended to 5
// per the design doc §"Track A" line 165 ("Active wallets, Total
// volume, Active plans, Open tickets, API calls today"). The TS
// source's 4-card row is `Total Wallets, Sys Health, Daily Conns,
// Avg Resp`; the design doc's 5-card row is the platform-overview
// variant. Both render in the same `.admin-stats-grid` shell.

/// 5-up stat row. Wraps `StatCard` for each card; the source's
/// `DashboardHudMetrics` does the same with metric structs.
#[component]
pub fn AdminStatsCards() -> Element {
    rsx! {
        div { class: "admin-stats-cards admin-stats-grid grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4",
            StatCard { label: "Active wallets".to_string(), value: "1,247".to_string(), icon: Some("wallet".to_string()) }
            StatCard { label: "Total volume".to_string(), value: "$2.4M".to_string(), icon: Some("trending-up".to_string()) }
            StatCard { label: "Active plans".to_string(), value: "38".to_string(), icon: Some("layout-dashboard".to_string()) }
            StatCard { label: "Open tickets".to_string(), value: "7".to_string(), icon: Some("message-circle".to_string()) }
            StatCard { label: "API calls today".to_string(), value: "84.2K".to_string(), icon: Some("zap".to_string()) }
        }
    }
}

// ===== WalletsByChain ======================================================
//
// Source: assembled from `useDashboardData`'s `walletsByChain` data +
// visual parity with the TS `ChartDonut` rendering. The chain list
// (BSC, ETH, Polygon, Arbitrum, Base) mirrors the
// `use-dashboard-data` hook's stable values.

/// Per-chain wallet distribution — donut + 5-row legend.
#[component]
pub fn WalletsByChain() -> Element {
    rsx! {
        div { class: "card card-glass wallets-by-chain mt-6",
            div { class: "card-header flex items-center justify-between",
                h3 { class: "card-title", "Wallets by chain" }
                span { class: "text-xs text-muted-foreground font-mono", "live snapshot" }
            }
            div { class: "card-body",
                div { class: "grid grid-cols-1 md:grid-cols-2 gap-6 items-center",
                    div { class: "flex justify-center",
                        ChartDonut {
                            data: vec![
                                ("BSC".to_string(), 712.0, "#f0b90b".to_string()),
                                ("ETH".to_string(), 312.0, "#627eea".to_string()),
                                ("Polygon".to_string(), 134.0, "#8247e5".to_string()),
                                ("Arbitrum".to_string(), 67.0, "#28a0f0".to_string()),
                                ("Base".to_string(), 22.0, "#0052ff".to_string()),
                            ],
                            size: 200, thickness: 28,
                        }
                    }
                    div { class: "space-y-2",
                        ChainLegendRow { name: "BSC Mainnet", count: 712, pct: 57, color: "#f0b90b" }
                        ChainLegendRow { name: "Ethereum", count: 312, pct: 25, color: "#627eea" }
                        ChainLegendRow { name: "Polygon", count: 134, pct: 11, color: "#8247e5" }
                        ChainLegendRow { name: "Arbitrum", count: 67, pct: 5, color: "#28a0f0" }
                        ChainLegendRow { name: "Base", count: 22, pct: 2, color: "#0052ff" }
                    }
                }
            }
        }
    }
}

/// One row in the per-chain legend.
#[component]
pub fn ChainLegendRow(name: String, count: u32, pct: u32, color: String) -> Element {
    rsx! {
        div { class: "flex items-center justify-between text-sm",
            div { class: "flex items-center gap-2",
                span { class: "w-2.5 h-2.5 rounded-full inline-block", style: "background-color: {color}" }
                span { "{name}" }
            }
            div { class: "flex items-center gap-3 font-mono",
                span { "{count}" }
                span { class: "text-muted-foreground text-xs", "{pct}%" }
            }
        }
    }
}

// ===== RecentTransactions ===================================================
//
// Recent platform transactions table — adapted from the TS source's
// `DashboardActivityStream` row shape (wallet address + time + status
// pill) but rendered as a tabular view for the admin's "Command
// Center" overview. The TS source uses 6 columns (Time, Type, From,
// To, Amount, Status); we match that to keep section parity.

/// Recent platform transactions — 6-col table.
#[component]
pub fn RecentTransactions() -> Element {
    rsx! {
        div { class: "card card-glass recent-transactions",
            div { class: "card-header flex items-center justify-between",
                h3 { class: "card-title", "Recent transactions" }
                a { class: "btn btn-sm btn-ghost", href: "/audit-log", "View all" }
            }
            div { class: "card-body p-0",
                div { class: "table-wrap",
                    table { class: "table",
                        thead { tr {
                            th { "Time" }
                            th { "Type" }
                            th { "From" }
                            th { "To" }
                            th { "Amount" }
                            th { "Status" }
                        } }
                        tbody {
                            tr {
                                td { class: "text-xs text-muted-foreground", "10:32:15" }
                                td { span { class: "badge badge-info", "transfer" } }
                                td { code { class: "text-xs", "0x1234…5678" } }
                                td { code { class: "text-xs", "0xabcd…ef90" } }
                                td { class: "font-mono", "0.5 BNB" }
                                td { span { class: "badge badge-success", "Confirmed" } }
                            }
                            tr {
                                td { class: "text-xs text-muted-foreground", "10:31:48" }
                                td { span { class: "badge badge-warning", "subscription" } }
                                td { code { class: "text-xs", "0xbeef…dead" } }
                                td { code { class: "text-xs", "0xplan…treas" } }
                                td { class: "font-mono", "$29.00" }
                                td { span { class: "badge badge-success", "Confirmed" } }
                            }
                            tr {
                                td { class: "text-xs text-muted-foreground", "10:30:21" }
                                td { span { class: "badge badge-danger", "revoke" } }
                                td { code { class: "text-xs", "0xcace…face" } }
                                td { code { class: "text-xs", "0xperm…node" } }
                                td { class: "font-mono", "—" }
                                td { span { class: "badge badge-warning", "Pending" } }
                            }
                            tr {
                                td { class: "text-xs text-muted-foreground", "10:28:54" }
                                td { span { class: "badge badge-info", "transfer" } }
                                td { code { class: "text-xs", "0x9876…5432" } }
                                td { code { class: "text-xs", "0xfedc…ba98" } }
                                td { class: "font-mono", "1.2 BNB" }
                                td { span { class: "badge badge-success", "Confirmed" } }
                            }
                            tr {
                                td { class: "text-xs text-muted-foreground", "10:27:11" }
                                td { span { class: "badge badge-info", "transfer" } }
                                td { code { class: "text-xs", "0xaaaa…bbbb" } }
                                td { code { class: "text-xs", "0xcccc…dddd" } }
                                td { class: "font-mono", "0.1 BNB" }
                                td { span { class: "badge badge-danger", "Failed" } }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ===== SystemAlerts ========================================================
//
// System alerts list — mirrors the "Alerts" stat in
// `dashboard-pulse-header.tsx` (the `pendingNotifications` counter)
// and surfaces the top-3 active alerts inline. The TS source's alert
// data lives in the `useDashboardData` hook's `systemAlerts` array.

/// Severity level for a system alert.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlertLevel {
    Info,
    Warning,
    Danger,
}

/// Top alerts list (3-row summary) — header row + body with one
/// `AlertRow` per alert.
#[component]
pub fn SystemAlerts() -> Element {
    rsx! {
        div { class: "card card-glass system-alerts",
            div { class: "card-header flex items-center justify-between",
                h3 { class: "card-title", "System alerts" }
                span { class: "badge badge-success", "all clear" }
            }
            div { class: "card-body space-y-2",
                AlertRow { level: AlertLevel::Info, title: "Indexer sync complete".to_string(), detail: "BSC Mainnet — 1,247,891 blocks".to_string() }
                AlertRow { level: AlertLevel::Warning, title: "API latency elevated".to_string(), detail: "p95 = 312ms (threshold 200ms)".to_string() }
                AlertRow { level: AlertLevel::Info, title: "Backup completed".to_string(), detail: "Snapshot 2024-09-20 03:00 UTC".to_string() }
            }
        }
    }
}

/// One alert row. The TS source uses `AlertTriangle` and
/// `AlertOctagon` from lucide-react. EPSX's icon registry ships
/// `triangle-alert` (added in Wave 5) but NOT `alert-octagon` —
/// fall back to `circle-x` for the danger visual. Both are
/// semantically "something is wrong" markers.
#[component]
pub fn AlertRow(level: AlertLevel, title: String, detail: String) -> Element {
    let (badge_cls, icon) = match level {
        AlertLevel::Info => ("badge badge-info", "info"),
        AlertLevel::Warning => ("badge badge-warning", "triangle-alert"),
        AlertLevel::Danger => ("badge badge-danger", "circle-x"),
    };
    rsx! {
        div { class: "flex items-start gap-3 p-3 rounded-lg border border-border/40 hover:bg-muted/30",
            span { class: "card-icon mt-0.5", Icon { name: icon.to_string(), size: Some(16) } }
            div { class: "flex-1 min-w-0",
                div { class: "flex items-center gap-2",
                    span { "{title}" }
                    span { class: "{badge_cls}", "{level:?}" }
                }
                p { class: "text-xs text-muted-foreground mt-0.5", "{detail}" }
            }
        }
    }
}

// ===== ActivityStream ======================================================
//
// Source: `dashboard-activity-stream.tsx` (140 LoC) — "Global Event
// Stream" sidebar feed showing recent wallet auth events. The TS
// source uses React Query to fetch `recent-wallets-stream`; the port
// is a static sample (the BFF would hydrate via `ctx.params`).

/// "Global Event Stream" sidebar feed — recent wallet auth events
/// rendered as a vertical list with per-row status pill.
#[component]
pub fn ActivityStream() -> Element {
    rsx! {
        div { class: "card card-glass activity-stream h-full flex flex-col",
            div { class: "card-header flex items-center justify-between",
                div { class: "flex items-center gap-2",
                    span { class: "pulse-indicator", span { class: "pulse-dot" } }
                    h3 { class: "card-title", "Global Event Stream" }
                }
                a { class: "btn btn-sm btn-ghost", href: "/audit-log", Icon { name: "external-link".to_string(), size: Some(14) } }
            }
            div { class: "card-body p-2 space-y-1 flex-1 overflow-y-auto",
                ActivityRow { addr: "0x1234…5678", status: "Active", time: "2 min ago", is_new: false }
                ActivityRow { addr: "0xabcd…ef90", status: "Active", time: "5 min ago", is_new: true }
                ActivityRow { addr: "0xbeef…dead", status: "Active", time: "12 min ago", is_new: false }
                ActivityRow { addr: "0x9876…5432", status: "Offline", time: "1 hour ago", is_new: false }
                ActivityRow { addr: "0xfedc…ba98", status: "Active", time: "2 hours ago", is_new: true }
                ActivityRow { addr: "0xcace…face", status: "Offline", time: "3 hours ago", is_new: false }
            }
            div { class: "card-footer text-[10px] text-muted-foreground font-mono uppercase tracking-widest text-center",
                "END OF STREAM / 6 NODES LOGGED"
            }
        }
    }
}

/// One row in the activity stream.
///
/// The TS source uses `Wifi`/`WifiOff` from lucide-react. EPSX's
/// icon registry only ships `wifi-off` (per the Wave 5
/// lucide-icon additions list). Fall back to `check`/`x` for
/// the online / offline visual — semantically the same
/// (online = check, offline = x).
#[component]
pub fn ActivityRow(addr: String, status: String, time: String, is_new: bool) -> Element {
    let active_cls = if status == "Active" {
        "text-success".to_string()
    } else {
        "text-muted-foreground/50".to_string()
    };
    let icon_name = if status == "Active" { "check".to_string() } else { "wifi-off".to_string() };
    rsx! {
        div { class: "group flex items-start gap-3 p-2 rounded-lg border border-transparent hover:border-border/30 hover:bg-muted/30",
            Icon { name: icon_name, size: Some(14), class_name: Some(active_cls) }
            div { class: "flex-1 min-w-0",
                div { class: "flex items-center gap-2",
                    span { class: "font-mono text-sm", "{addr}" }
                    if is_new {
                        span { class: "px-1.5 py-0.5 rounded text-[9px] bg-cyan-500/20 text-cyan-400 border border-cyan-500/30 uppercase tracking-widest leading-none", "New Auth" }
                    }
                    span { class: "px-1.5 py-0.5 rounded text-[9px] bg-muted text-muted-foreground uppercase tracking-widest leading-none", "{status}" }
                }
                p { class: "text-[11px] text-muted-foreground mt-0.5", "{time}" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke test for `<AdminPulseHeader>`. The component should
    /// render the title + status pill without panicking.
    #[test]
    fn test_render_smoke_admin_pulse_header() {
        let html = dioxus_ssr::render_element(rsx! { AdminPulseHeader {} });
        assert!(html.contains("Command Center"), "AdminPulseHeader must render the title. Got: {html}");
        assert!(html.contains("OPERATIONAL"), "AdminPulseHeader must render the status pill. Got: {html}");
    }

    /// Smoke test for `<AdminStatsCards>` — 5 cards in the
    /// `.admin-stats-grid` shell.
    #[test]
    fn test_render_smoke_admin_stats_cards() {
        let html = dioxus_ssr::render_element(rsx! { AdminStatsCards {} });
        assert!(html.contains("admin-stats-cards"), "AdminStatsCards must keep its section marker. Got: {html}");
        assert!(html.contains("Active wallets"), "AdminStatsCards must render the first card label. Got: {html}");
        assert!(html.contains("API calls today"), "AdminStatsCards must render the fifth card label. Got: {html}");
    }

    /// Smoke test for `<WalletsByChain>` — donut + legend.
    #[test]
    fn test_render_smoke_wallets_by_chain() {
        let html = dioxus_ssr::render_element(rsx! { WalletsByChain {} });
        assert!(html.contains("wallets-by-chain"), "WalletsByChain must keep its section marker. Got: {html}");
        assert!(html.contains("BSC Mainnet"), "WalletsByChain must render the chain legend. Got: {html}");
    }

    /// Smoke test for `<RecentTransactions>` — 6-col table.
    #[test]
    fn test_render_smoke_recent_transactions() {
        let html = dioxus_ssr::render_element(rsx! { RecentTransactions {} });
        assert!(html.contains("recent-transactions"), "RecentTransactions must keep its section marker. Got: {html}");
        assert!(html.contains("Confirmed"), "RecentTransactions must render at least one status row. Got: {html}");
    }

    /// Smoke test for `<SystemAlerts>` — top alerts list.
    #[test]
    fn test_render_smoke_system_alerts() {
        let html = dioxus_ssr::render_element(rsx! { SystemAlerts {} });
        assert!(html.contains("system-alerts"), "SystemAlerts must keep its section marker. Got: {html}");
        assert!(html.contains("all clear"), "SystemAlerts must render the all-clear badge. Got: {html}");
    }

    /// Smoke test for `<ActivityStream>` — Global Event Stream.
    #[test]
    fn test_render_smoke_activity_stream() {
        let html = dioxus_ssr::render_element(rsx! { ActivityStream {} });
        assert!(html.contains("activity-stream"), "ActivityStream must keep its section marker. Got: {html}");
        assert!(html.contains("Global Event Stream"), "ActivityStream must render its title. Got: {html}");
        assert!(html.contains("END OF STREAM"), "ActivityStream must render the footer. Got: {html}");
    }
}

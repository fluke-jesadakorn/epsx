//! /admin — Command Center (admin dashboard).
//!
//! Wave 6B Track A — port of `apps-old/admin-frontend/app/page.tsx` (24 LoC)
//! + `dashboard-client.tsx` (52 LoC) + the 4 admin sub-components:
//! - `dashboard-pulse-header.tsx` (70 LoC) — Command Center pulse header
//!   with system-health pill, latency/uptime/alerts stat row.
//! - `dashboard-hud-metrics.tsx` (80 LoC) — 4-card HUD metrics row
//!   (Total Wallets, Sys Health, Daily Conns, Avg Resp).
//! - `dashboard-bento-tools.tsx` (121 LoC) — 6-card bento grid linking
//!   to the main operational modules.
//! - `dashboard-activity-stream.tsx` (140 LoC) — Global Event Stream
//!   showing recent wallet auth events.
//!
//! Sections (per design doc §"Track A" line 165):
//! - `AdminStatsCards` — 5-up stat row (Active wallets, Total volume,
//!   Active plans, Open tickets, API calls today).
//! - `WalletsByChain` — wallet distribution per chain (BSC, ETH, etc.).
//! - `RecentTransactions` — recent platform transactions table.
//! - `SystemAlerts` — system health alerts list.
//! - `ActivityStream` — global event stream (mirrors the TS
//!   `DashboardActivityStream` source — recent wallet connections
//!   with the "Global Event Stream" header).
//!
//! All section markers are asserted in the `tests` module below.

use crate::auth::AdminAuthGate;
use crate::charts::{ChartBar, ChartDonut, ChartLine, DataPoint, Series};
use crate::feedback::EmptyState;
use crate::layout::admin_shell::AdminShell;
use crate::primitives::*;

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Command Center");
    (meta, rsx! { RenderDashboard { ctx: ctx.clone() } })
}

#[component]
fn RenderDashboard(ctx: PageContext) -> Element {
    rsx! {
        AdminAuthGate { user: ctx.user.clone(), feature: Some("the admin dashboard".to_string()), required_permissions: Some(vec!["admin:*".to_string()]), return_url: Some(ctx.path.clone()),
            AdminShell {
                ctx: ctx.clone(),
                page_title: "Command Center".to_string(),
                breadcrumbs: vec![
                    ("Dashboard".to_string(), "/".to_string()),
                    ("Command Center".to_string(), "/".to_string()),
                ],
                div { class: "container page-content admin-dashboard",
                    // Command Center pulse header (mirrors
                    // `dashboard-pulse-header.tsx`).
                    AdminPulseHeader {}
                    // 5-up stat row (AdminStatsCards).
                    AdminStatsCards {}
                    // Operational Modules — 6-card bento grid (port of
                    // `dashboard-bento-tools.tsx`).
                    OperationalModulesBento {}
                    // WalletsByChain — donut + per-chain legend.
                    WalletsByChain {}
                    // ActivityStream + RecentTransactions + SystemAlerts in a 3-col grid.
                    div { class: "grid grid-cols-1 lg:grid-cols-3 gap-4 mt-6",
                        div { class: "lg:col-span-1",
                            ActivityStream {}
                        }
                        div { class: "lg:col-span-2",
                            RecentTransactions {}
                            div { class: "mt-4",
                                SystemAlerts {}
                            }
                        }
                    }
                }
            }
        }
    }
}

// ===== AdminPulseHeader =====================================================
//
// Source: `dashboard-pulse-header.tsx` — header card with title,
// "OPERATIONAL" / "DEGRADED" pill (animated dot), current time, and a
// right-side stats cluster (Latency, Uptime, Alerts).

#[component]
fn AdminPulseHeader() -> Element {
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

// ===== OperationalModulesBento ==============================================
//
// Port of `dashboard-bento-tools.tsx` (121 LoC) — 6-card bento grid
// linking to the main operational modules (Wallet Database, Security
// & Perms, Global Audit Log, Broadcast Hub, Dev Infrastructure,
// Settings). Each card shows: icon, status pill, title, description.
//
// Layout: 3-col responsive grid; 2 cards span 2 cols; Security card
// spans 2 rows.

#[component]
fn OperationalModulesBento() -> Element {
    rsx! {
        div { class: "admin-bento-tools mt-6",
            div { class: "mb-4 flex items-center justify-between",
                h2 { class: "text-sm font-bold text-muted-foreground uppercase tracking-widest",
                    "Operational Modules"
                }
            }
            div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 auto-rows-[220px]",
                // Wallet Database — 2 cols, 1 row
                a { class: "group relative overflow-hidden rounded-2xl border border-border/20 bg-card/60 backdrop-blur-md transition-all duration-300 hover:scale-[1.02] hover:shadow-[0_0_30px_-5px_rgba(255,255,255,0.1)] hover:border-white/20 col-span-1 lg:col-span-2 row-span-1 flex flex-col",
                    href: "/wallet-management",
                    div { class: "absolute inset-0 bg-gradient-to-br from-cyan-500/20 to-blue-500/20 opacity-50 transition-opacity group-hover:opacity-100" }
                    div { class: "relative p-6 flex flex-col h-full z-10",
                        div { class: "flex justify-between items-start mb-4",
                            div { class: "p-3 rounded-xl bg-background/50 border border-white/5 backdrop-blur-md shadow-inner text-cyan-400",
                                Icon { name: "wallet".to_string(), size: Some(24) }
                            }
                            div { class: "text-[10px] font-mono uppercase tracking-widest px-3 py-1 bg-background/50 border border-white/5 rounded-full text-cyan-400",
                                "1,247 Registered"
                            }
                        }
                        div { class: "mt-auto",
                            h3 { class: "text-xl sm:text-2xl font-black tracking-tight text-foreground mb-2 group-hover:text-white transition-colors",
                                "Wallet Database"
                            }
                            p { class: "text-sm font-medium text-muted-foreground line-clamp-3 leading-relaxed",
                                "Deep inspect connected wallets, view connection history, and force disconnect active sessions."
                            }
                        }
                    }
                }
                // Security & Perms — 1 col, 2 rows
                a { class: "group relative overflow-hidden rounded-2xl border border-border/20 bg-card/60 backdrop-blur-md transition-all duration-300 hover:scale-[1.02] hover:shadow-[0_0_30px_-5px_rgba(255,255,255,0.1)] hover:border-white/20 col-span-1 row-span-2 flex flex-col",
                    href: "/wallet-management/access",
                    div { class: "absolute inset-0 bg-gradient-to-br from-purple-500/20 to-fuchsia-500/20 opacity-50 transition-opacity group-hover:opacity-100" }
                    div { class: "relative p-6 flex flex-col h-full z-10",
                        div { class: "flex justify-between items-start mb-4",
                            div { class: "p-3 rounded-xl bg-background/50 border border-white/5 backdrop-blur-md shadow-inner text-purple-400",
                                Icon { name: "shield".to_string(), size: Some(24) }
                            }
                            div { class: "text-[10px] font-mono uppercase tracking-widest px-3 py-1 bg-background/50 border border-white/5 rounded-full text-purple-400",
                                "24 Active Nodes"
                            }
                        }
                        div { class: "mt-auto",
                            h3 { class: "text-xl sm:text-2xl font-black tracking-tight text-foreground mb-2 group-hover:text-white transition-colors",
                                "Security & Perms"
                            }
                            p { class: "text-sm font-medium text-muted-foreground line-clamp-3 leading-relaxed",
                                "Critical access control. Manage robust permissions across all modules."
                            }
                        }
                    }
                }
                // Global Audit Log — 1 col, 1 row
                a { class: "group relative overflow-hidden rounded-2xl border border-border/20 bg-card/60 backdrop-blur-md transition-all duration-300 hover:scale-[1.02] hover:shadow-[0_0_30px_-5px_rgba(255,255,255,0.1)] hover:border-white/20 col-span-1 row-span-1 flex flex-col",
                    href: "/audit-log",
                    div { class: "absolute inset-0 bg-gradient-to-br from-pink-500/20 to-rose-500/20 opacity-50 transition-opacity group-hover:opacity-100" }
                    div { class: "relative p-6 flex flex-col h-full z-10",
                        div { class: "flex justify-between items-start mb-4",
                            div { class: "p-3 rounded-xl bg-background/50 border border-white/5 backdrop-blur-md shadow-inner text-pink-400",
                                Icon { name: "file-text".to_string(), size: Some(24) }
                            }
                            div { class: "text-[10px] font-mono uppercase tracking-widest px-3 py-1 bg-background/50 border border-white/5 rounded-full text-pink-400",
                                "Monitoring Active"
                            }
                        }
                        div { class: "mt-auto",
                            h3 { class: "text-xl sm:text-2xl font-black tracking-tight text-foreground mb-2 group-hover:text-white transition-colors",
                                "Global Audit Log"
                            }
                            p { class: "text-sm font-medium text-muted-foreground line-clamp-3 leading-relaxed",
                                "Immutable history of all administrative cross-system actions."
                            }
                        }
                    }
                }
                // Broadcast Hub — 1 col, 1 row
                a { class: "group relative overflow-hidden rounded-2xl border border-border/20 bg-card/60 backdrop-blur-md transition-all duration-300 hover:scale-[1.02] hover:shadow-[0_0_30px_-5px_rgba(255,255,255,0.1)] hover:border-white/20 col-span-1 row-span-1 flex flex-col",
                    href: "/notifications",
                    div { class: "absolute inset-0 bg-gradient-to-br from-amber-500/20 to-orange-500/20 opacity-50 transition-opacity group-hover:opacity-100" }
                    div { class: "relative p-6 flex flex-col h-full z-10",
                        div { class: "flex justify-between items-start mb-4",
                            div { class: "p-3 rounded-xl bg-background/50 border border-white/5 backdrop-blur-md shadow-inner text-amber-400",
                                Icon { name: "bell".to_string(), size: Some(24) }
                            }
                            div { class: "text-[10px] font-mono uppercase tracking-widest px-3 py-1 bg-background/50 border border-white/5 rounded-full text-amber-400",
                                "3 Pending Broadcasts"
                            }
                        }
                        div { class: "mt-auto",
                            h3 { class: "text-xl sm:text-2xl font-black tracking-tight text-foreground mb-2 group-hover:text-white transition-colors",
                                "Broadcast Hub"
                            }
                            p { class: "text-sm font-medium text-muted-foreground line-clamp-3 leading-relaxed",
                                "Push critical system alerts and global updates."
                            }
                        }
                    }
                }
                // Dev Infrastructure — 2 cols, 1 row
                a { class: "group relative overflow-hidden rounded-2xl border border-border/20 bg-card/60 backdrop-blur-md transition-all duration-300 hover:scale-[1.02] hover:shadow-[0_0_30px_-5px_rgba(255,255,255,0.1)] hover:border-white/20 col-span-1 lg:col-span-2 row-span-1 flex flex-col",
                    href: "/developer-portal",
                    div { class: "absolute inset-0 bg-gradient-to-br from-emerald-500/20 to-teal-500/20 opacity-50 transition-opacity group-hover:opacity-100" }
                    div { class: "relative p-6 flex flex-col h-full z-10",
                        div { class: "flex justify-between items-start mb-4",
                            div { class: "p-3 rounded-xl bg-background/50 border border-white/5 backdrop-blur-md shadow-inner text-emerald-400",
                                Icon { name: "database".to_string(), size: Some(24) }
                            }
                            div { class: "text-[10px] font-mono uppercase tracking-widest px-3 py-1 bg-background/50 border border-white/5 rounded-full text-emerald-400",
                                "SYS_OK"
                            }
                        }
                        div { class: "mt-auto",
                            h3 { class: "text-xl sm:text-2xl font-black tracking-tight text-foreground mb-2 group-hover:text-white transition-colors",
                                "Dev Infrastructure"
                            }
                            p { class: "text-sm font-medium text-muted-foreground line-clamp-3 leading-relaxed",
                                "Manage system API keys, global integrations and webhooks."
                            }
                        }
                    }
                }
                // Settings — 1 col, 1 row
                a { class: "group relative overflow-hidden rounded-2xl border border-border/20 bg-card/60 backdrop-blur-md transition-all duration-300 hover:scale-[1.02] hover:shadow-[0_0_30px_-5px_rgba(255,255,255,0.1)] hover:border-white/20 col-span-1 row-span-1 flex flex-col",
                    href: "/settings",
                    div { class: "absolute inset-0 bg-gradient-to-br from-slate-500/20 to-gray-500/20 opacity-50 transition-opacity group-hover:opacity-100" }
                    div { class: "relative p-6 flex flex-col h-full z-10",
                        div { class: "flex justify-between items-start mb-4",
                            div { class: "p-3 rounded-xl bg-background/50 border border-white/5 backdrop-blur-md shadow-inner text-slate-400",
                                Icon { name: "settings".to_string(), size: Some(24) }
                            }
                            div { class: "text-[10px] font-mono uppercase tracking-widest px-3 py-1 bg-background/50 border border-white/5 rounded-full text-slate-400",
                                "V2.4.0"
                            }
                        }
                        div { class: "mt-auto",
                            h3 { class: "text-xl sm:text-2xl font-black tracking-tight text-foreground mb-2 group-hover:text-white transition-colors",
                                "Settings"
                            }
                            p { class: "text-sm font-medium text-muted-foreground line-clamp-3 leading-relaxed",
                                "Core Platform config."
                            }
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

#[component]
fn AdminStatsCards() -> Element {
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
// Source: not a single component in the TS source — assembled from
// the source's `useDashboardData` hook's `walletsByChain` data +
// visual parity with the TS `ChartDonut` rendering. The chain list
// (BSC, ETH, Polygon, Arbitrum, Base) mirrors the
// `use-dashboard-data` hook's stable values.

#[component]
fn WalletsByChain() -> Element {
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

#[component]
fn ChainLegendRow(name: String, count: u32, pct: u32, color: String) -> Element {
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

#[component]
fn RecentTransactions() -> Element {
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

#[component]
fn SystemAlerts() -> Element {
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AlertLevel { Info, Warning, Danger }

#[component]
fn AlertRow(level: AlertLevel, title: String, detail: String) -> Element {
    // The TS source uses `AlertTriangle` and `AlertOctagon` from
    // lucide-react. EPSX's icon registry ships `triangle-alert`
    // (added in Wave 5) but NOT `alert-octagon` — fall back to
    // `circle-x` for the danger visual. Both are semantically
    // "something is wrong" markers.
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

#[component]
fn ActivityStream() -> Element {
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

#[component]
fn ActivityRow(addr: String, status: String, time: String, is_new: bool) -> Element {
    let active_cls = if status == "Active" {
        "text-success".to_string()
    } else {
        "text-muted-foreground/50".to_string()
    };
    // The TS source uses `Wifi`/`WifiOff` from lucide-react. EPSX's
    // icon registry only ships `wifi-off` (per the Wave 5
    // lucide-icon additions list). Fall back to `check`/`x` for
    // the online / offline visual — semantically the same
    // (online = check, offline = x).
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
    use crate::auth::user::{AuthMethod, User};

    /// Build an authenticated admin `PageContext` with the
    /// `admin:*` permission the gate checks.
    fn admin_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "u-admin".to_string(),
                address: "0x1234abcd5678ef90".to_string(),
                chain_id: "56".to_string(),
                roles: vec!["admin".to_string()],
                email: Some("admin@epsx.io".to_string()),
                tier: Some("Admin".to_string()),
                permissions: vec!["admin:*".to_string()],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: Some("Admin".to_string()),
            }),
            path: "/".to_string(),
            ..Default::default()
        }
    }

    /// Smoke test: `render()` returns a non-empty Element.
    #[test]
    fn test_render_smoke() {
        let ctx = admin_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(!html.trim().is_empty(), "dashboard must render non-empty HTML. Got: {html}");
        assert!(html.len() > 100, "dashboard HTML is suspiciously short ({} bytes).", html.len());
    }

    /// Section-marker test: the rendered HTML must contain every
    /// design-doc-named section.
    #[test]
    fn test_section_markers() {
        let ctx = admin_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        for marker in &[
            "admin-stats-cards",
            "admin-bento-tools",
            "wallets-by-chain",
            "recent-transactions",
            "system-alerts",
            "activity-stream",
        ] {
            // 4-form matcher (single class, first/middle/last of a
            // class list).
            let needle_a = format!("class=\"{}\"", marker);
            let needle_b = format!("class=\"{mark} ", mark = marker);
            let needle_c = format!(" {}\"", marker);
            let needle_d = format!(" {} ", marker);
            assert!(
                html.contains(&needle_a)
                    || html.contains(&needle_b)
                    || html.contains(&needle_c)
                    || html.contains(&needle_d),
                "dashboard must contain section marker `{marker}`. Got: {html}"
            );
        }
    }
}

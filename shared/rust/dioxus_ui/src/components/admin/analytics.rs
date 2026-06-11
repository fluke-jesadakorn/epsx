//! Admin analytics sub-components — 1:1 mirror of
//! `apps-old/admin-frontend/app/analytics/analytics-dashboard.tsx`
//! + the `components/analytics/*` and `components/admin/*` siblings
//! referenced by the page.
//!
//! Extracted from `pages/admin_pages/analytics.rs` (Wave 6B Track A)
//! per the Wave 6C design doc §"Track B" line 230:
//! - `AnalyticsHeader` — page title + Live/AI-Powered status pills
//!   + Export button.
//! - `AnalyticsCardGrid` — 4-card summary row (Total Users, API
//!   Requests, Active Permissions, System Health) + 4-card
//!   secondary grid (Active Users, Expiring Permissions, Response
//!   Time, Memory).
//! - `AnalyticsChart` — 2-up chart row (Active users 30d line +
//!   Volume 7d bar).
//! - `AnalyticsTable` — "Top events" 6-col table.
//! - `AnalyticsFilterPanel` — search + time-range + source selects
//!   + Apply button.
//! - `AnalyticsExportDialog` — wrapper around Wave 6A's
//!   `<ExportDialog>` primitive with the `analytics-export-dialog`
//!   section marker (always-present `data-section` wrapper).
//! - `AnalyticsMetadata` — generated-on / data-source / schema
//!   metadata row.

use dioxus::prelude::*;

use crate::primitives::icon::Icon;
use crate::charts::{ChartBar, ChartLine, DataPoint, Series};
use crate::data::export_dialog::{ExportDialog, ExportFormat, ExportScope};

// ===== AnalyticsHeader =====================================================
//
// Source: the page header rows of `analytics-dashboard.tsx` lines
// 89-101 — title + "Refresh Data" button. We keep the status pills
// from `app/analytics/page.tsx` lines 56-64 (Live / AI-Powered) as
// visual chips on the right.

/// Page header with title, Live/AI-Powered pills, and an Export
/// button. The `on_export` callback fires when the user clicks
/// Export; the parent owns the open/close signal for the dialog.
#[component]
pub fn AnalyticsHeader(on_export: EventHandler<MouseEvent>) -> Element {
    rsx! {
        div { class: "analytics-header flex items-center justify-between flex-wrap gap-4 mb-6",
            div {
                h1 { class: "text-2xl font-bold", "Platform analytics" }
                p { class: "text-muted-foreground", "Real-time system performance and user activity" }
            }
            div { class: "flex items-center gap-2",
                span { class: "status-pill status-pill-live",
                    span { class: "pulse-dot" }
                    "Live"
                }
                span { class: "status-pill status-pill-ai",
                    Icon { name: "sparkles".to_string(), size: Some(14) }
                    "AI-Powered"
                }
                button {
                    class: "btn btn-outline",
                    r#type: "button",
                    onclick: move |e| on_export.call(e),
                    Icon { name: "download".to_string(), size: Some(14) }
                    "Export"
                }
            }
        }
    }
}

// ===== AnalyticsFilterPanel ================================================
//
// Source: the `ServerFilters` sub-component in the analytics
// sub-component cluster (search, type chip, sort). The Dioxus port
// inlines the form fields (no sub-components — section-level parity).

/// Search + time-range + source filter form with an Apply button.
#[component]
pub fn AnalyticsFilterPanel() -> Element {
    rsx! {
        div { class: "card card-glass analytics-filter-panel mb-6",
            div { class: "card-body",
                div { class: "flex flex-col md:flex-row gap-3 items-stretch md:items-center",
                    div { class: "flex-1",
                        label { class: "field-label", "Search events" }
                        input {
                            class: "input",
                            r#type: "text",
                            placeholder: "Filter by event name, user, or session…",
                        }
                    }
                    div { class: "md:w-48",
                        label { class: "field-label", "Time range" }
                        select { class: "input",
                            option { value: "24h", "Last 24 hours" }
                            option { value: "7d", "Last 7 days" }
                            option { value: "30d", value: "30d", "Last 30 days" }
                            option { value: "90d", value: "90d", "Last 90 days" }
                        }
                    }
                    div { class: "md:w-48",
                        label { class: "field-label", "Source" }
                        select { class: "input",
                            option { value: "all", "All sources" }
                            option { value: "web", "Web" }
                            option { value: "api", "API" }
                            option { value: "mobile", "Mobile" }
                        }
                    }
                    div { class: "md:w-32 flex items-end",
                        button { class: "btn btn-primary w-full", r#type: "button", "Apply" }
                    }
                }
            }
        }
    }
}

// ===== AnalyticsCardGrid ===================================================
//
// Source: `SummarySection` + `StatsSection` + `MetricsGrid` from
// `analytics-dashboard.tsx` (lines 156-336). The Dioxus port merges
// the 3 sub-sections into a single 4-card grid at the top of the
// page (Total Users, API Requests, Active Permissions, System
// Health), then a 4-up secondary grid (Active Users, Expiring
// Permissions, Response Time, Memory) below.

/// 4 + 4 card grid — the platform overview primary row + the
/// secondary "MetricsGrid" row.
#[component]
pub fn AnalyticsCardGrid() -> Element {
    rsx! {
        div { class: "analytics-card-grid space-y-4 mb-6",
            // Primary 4-card row.
            div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4",
                SummaryCard { label: "Total Users".to_string(), value: "12,345".to_string(), sub: "Active users in system" }
                SummaryCard { label: "API Requests".to_string(), value: "1.2M".to_string(), sub: "Total requests processed" }
                SummaryCard { label: "Active Permissions".to_string(), value: "847".to_string(), sub: "Current permission sets" }
                SummaryCard { label: "System Health".to_string(), value: "98%".to_string(), sub: "Overall health score" }
            }
            // Secondary 4-card row (the TS source's "MetricsGrid").
            div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4",
                MetricCard { label: "Active Users".to_string(), value: "1,234".to_string(), trend: "+12.4%".to_string(), color: "text-cyan-400" }
                MetricCard { label: "Expiring Permissions".to_string(), value: "23".to_string(), trend: "next 7 days".to_string(), color: "text-warning" }
                MetricCard { label: "Response Time".to_string(), value: "142ms".to_string(), trend: "real-time".to_string(), color: "text-success" }
                MetricCard { label: "Memory".to_string(), value: "62%".to_string(), trend: "system usage".to_string(), color: "text-primary" }
            }
        }
    }
}

/// One summary card (primary row). The TS source's
/// `AnalyticsSummaryCard` accepts `title`/`value`/`subtitle` —
/// mirrored as `label`/`value`/`sub` for consistency with the
/// rest of the EPSX admin surface.
#[component]
pub fn SummaryCard(label: String, value: String, sub: String) -> Element {
    rsx! {
        div { class: "card card-glass",
            div { class: "card-body",
                p { class: "text-[10px] font-bold uppercase tracking-widest text-muted-foreground", "{label}" }
                p { class: "text-3xl font-black tracking-tight mt-1", "{value}" }
                p { class: "text-xs text-muted-foreground mt-1", "{sub}" }
            }
        }
    }
}

/// One metric card (secondary row) with a colored trend line.
#[component]
pub fn MetricCard(label: String, value: String, trend: String, color: String) -> Element {
    rsx! {
        div { class: "card card-glass",
            div { class: "card-body",
                p { class: "text-[10px] font-bold uppercase tracking-widest text-muted-foreground", "{label}" }
                p { class: "text-3xl font-black tracking-tight mt-1", "{value}" }
                p { class: format!("text-xs mt-1 {}", color), "{trend}" }
            }
        }
    }
}

// ===== AnalyticsChart ======================================================
//
// Source: the two chart cards in the analytics-dashboard's body
// (Active users 30d line + Volume 7d bar). The Dioxus port renders
// both as a 2-up grid using the existing `<ChartLine>` + `<ChartBar>`
// primitives.

/// 2-up chart row — Active users 30d line + Volume 7d bar.
#[component]
pub fn AnalyticsChart() -> Element {
    rsx! {
        div { class: "analytics-chart grid grid-cols-1 lg:grid-cols-2 gap-4 mb-6",
            div { class: "card card-glass",
                div { class: "card-header flex items-center justify-between",
                    h3 { class: "card-title", "Active users (30d)" }
                    div { class: "flex gap-1",
                        button { class: "btn btn-sm btn-outline", "1D" }
                        button { class: "btn btn-sm btn-primary", "30D" }
                        button { class: "btn btn-sm btn-outline", "90D" }
                    }
                }
                div { class: "card-body",
                    ChartLine {
                        series: vec![
                            Series { name: "DAU".to_string(), color: "#22d3ee".to_string(),
                                points: (0..30).map(|i| DataPoint {
                                    x: i as f64,
                                    y: 1000.0 + (i as f64 * 25.0) + (i as f64 * 0.4).sin() * 100.0,
                                    label: None,
                                }).collect()
                            }
                        ],
                        width: 480, height: 220,
                    }
                }
            }
            div { class: "card card-glass",
                div { class: "card-header flex items-center justify-between",
                    h3 { class: "card-title", "Volume (7d)" }
                    span { class: "text-xs text-muted-foreground font-mono", "BNB" }
                }
                div { class: "card-body",
                    ChartBar {
                        data: (0..7).map(|i| (format!("D{}", i + 1), 1000.0 + (i as f64 * 100.0))).collect(),
                        width: 480, height: 220,
                    }
                }
            }
        }
    }
}

// ===== AnalyticsTable ======================================================
//
// Source: the "Top events" table in the original analytics page +
// the API usage analytics table from analytics-dashboard. The
// Dioxus port renders a single 6-col table (Event, Count, Users,
// Conversion, Source, Last seen).

/// "Top events" 6-col table.
#[component]
pub fn AnalyticsTable() -> Element {
    rsx! {
        div { class: "card card-glass analytics-table mb-6",
            div { class: "card-header flex items-center justify-between",
                h3 { class: "card-title", "Top events" }
                a { class: "btn btn-sm btn-ghost", href: "/audit-log", "Full audit log" }
            }
            div { class: "card-body p-0",
                div { class: "table-wrap",
                    table { class: "table",
                        thead { tr {
                            th { "Event" }
                            th { "Count" }
                            th { "Users" }
                            th { "Conversion" }
                            th { "Source" }
                            th { "Last seen" }
                        } }
                        tbody {
                            tr { td { code { "wallet.connect" } } td { "1,234" } td { "890" } td { "72%" } td { span { class: "badge badge-info", "web" } } td { class: "text-xs text-muted-foreground", "2 min ago" } }
                            tr { td { code { "plan.subscribe" } } td { "432" } td { "412" } td { "95%" } td { span { class: "badge badge-info", "web" } } td { class: "text-xs text-muted-foreground", "5 min ago" } }
                            tr { td { code { "payment.confirm" } } td { "345" } td { "320" } td { "92%" } td { span { class: "badge badge-info", "api" } } td { class: "text-xs text-muted-foreground", "8 min ago" } }
                            tr { td { code { "auth.siwe" } } td { "278" } td { "265" } td { "95%" } td { span { class: "badge badge-info", "web" } } td { class: "text-xs text-muted-foreground", "11 min ago" } }
                            tr { td { code { "policy.evaluate" } } td { "12,345" } td { "1,180" } td { "—" } td { span { class: "badge badge-info", "api" } } td { class: "text-xs text-muted-foreground", "12 min ago" } }
                        }
                    }
                }
            }
        }
    }
}

// ===== AnalyticsMetadata ===================================================
//
// Source: not a single component in the TS source — the TS
// analytics-dashboard surfaces generated-at / data-source /
// schema-version as small text in the page footer. The Dioxus port
// inlines that footer as the `analytics-metadata` section so the
// section-marker assertion in the test has a stable hook.

/// Generated-on / data-source / schema-version metadata row.
#[component]
pub fn AnalyticsMetadata() -> Element {
    rsx! {
        div { class: "card card-glass analytics-metadata",
            div { class: "card-body",
                dl { class: "grid grid-cols-1 md:grid-cols-3 gap-4 text-sm",
                    div { class: "flex flex-col",
                        dt { class: "text-[10px] font-bold uppercase tracking-widest text-muted-foreground", "Generated" }
                        dd { class: "font-mono", "2024-09-20 10:32:18 UTC" }
                    }
                    div { class: "flex flex-col",
                        dt { class: "text-[10px] font-bold uppercase tracking-widest text-muted-foreground", "Data source" }
                        dd { "PostgreSQL analytics_prod" }
                    }
                    div { class: "flex flex-col",
                        dt { class: "text-[10px] font-bold uppercase tracking-widest text-muted-foreground", "Schema" }
                        dd { class: "font-mono", "v2.4.0" }
                    }
                }
            }
        }
    }
}

// ===== AnalyticsExportDialog ===============================================
//
// Wraps Wave 6A's `<ExportDialog>` primitive with an always-present
// `data-section="analytics-export-dialog"` wrapper so the
// section-marker test passes even when the dialog is closed. The
// pattern is the same one Wave 6B uses for the policies
// `policy-builder` and `policy-monitor` tab-conditional markers.

/// Wrapper around `<ExportDialog>` with a stable
/// `data-section="analytics-export-dialog"` marker.
///
/// `open` controls dialog visibility; `on_close` fires when the
/// user dismisses the dialog (via X, Escape, or backdrop click).
/// `on_export` fires with the chosen `ExportFormat` when the user
/// clicks the export button.
#[component]
pub fn AnalyticsExportDialog(
    open: bool,
    on_close: EventHandler,
    on_export: EventHandler<ExportFormat>,
) -> Element {
    rsx! {
        div { "data-section": "analytics-export-dialog", class: "analytics-export-dialog",
            ExportDialog {
                open,
                on_close,
                on_export,
                default_format: ExportFormat::Json,
                default_scope: ExportScope::Filtered,
                title: Some("Export analytics data".to_string()),
                description: Some("Choose the format and scope for the analytics snapshot.".to_string()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test harness — a `#[component]` so Dioxus sets up a runtime
    /// for the `EventHandler<MouseEvent>` callback the
    /// `<AnalyticsHeader>` component takes. We render
    /// `<AnalyticsHeader>` here so the smoke test doesn't have to
    /// spawn a `VirtualDom` itself. Mirrors the pattern from
    /// `data/export_dialog.rs::tests::TestHarness`.
    #[component]
    fn HeaderHarness() -> Element {
        rsx! { AnalyticsHeader { on_export: move |_| {} } }
    }

    /// Test harness for `<AnalyticsExportDialog>`. Renders the
    /// dialog wrapper with `open=false` (so the body is hidden) but
    /// the `data-section` marker is still in the DOM.
    #[component]
    fn ExportDialogHarness() -> Element {
        rsx! {
            AnalyticsExportDialog {
                open: false,
                on_close: move |_| {},
                on_export: move |_| {},
            }
        }
    }

    /// Smoke test for `<AnalyticsHeader>`. Should render the title
    /// + Live / AI-Powered pills + Export button.
    #[test]
    fn test_render_smoke_analytics_header() {
        let mut vdom = dioxus::prelude::VirtualDom::new(HeaderHarness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("analytics-header"), "AnalyticsHeader must keep its section marker. Got: {html}");
        assert!(html.contains("Platform analytics"), "AnalyticsHeader must render the title. Got: {html}");
        assert!(html.contains("AI-Powered"), "AnalyticsHeader must render the AI-Powered pill. Got: {html}");
    }

    /// Smoke test for `<AnalyticsFilterPanel>`. Renders the search
    /// input + time range + source selects + Apply.
    #[test]
    fn test_render_smoke_analytics_filter_panel() {
        let html = dioxus_ssr::render_element(rsx! { AnalyticsFilterPanel {} });
        assert!(html.contains("analytics-filter-panel"), "AnalyticsFilterPanel must keep its section marker. Got: {html}");
        assert!(html.contains("Search events"), "AnalyticsFilterPanel must render the search field. Got: {html}");
    }

    /// Smoke test for `<AnalyticsCardGrid>`. Renders both 4-card
    /// rows.
    #[test]
    fn test_render_smoke_analytics_card_grid() {
        let html = dioxus_ssr::render_element(rsx! { AnalyticsCardGrid {} });
        assert!(html.contains("analytics-card-grid"), "AnalyticsCardGrid must keep its section marker. Got: {html}");
        assert!(html.contains("Total Users"), "AnalyticsCardGrid must render the first card. Got: {html}");
        assert!(html.contains("Memory"), "AnalyticsCardGrid must render the secondary row's last card. Got: {html}");
    }

    /// Smoke test for `<AnalyticsChart>`. Renders both chart
    /// cards.
    #[test]
    fn test_render_smoke_analytics_chart() {
        let html = dioxus_ssr::render_element(rsx! { AnalyticsChart {} });
        assert!(html.contains("analytics-chart"), "AnalyticsChart must keep its section marker. Got: {html}");
        assert!(html.contains("Active users (30d)"), "AnalyticsChart must render the line chart card. Got: {html}");
        assert!(html.contains("Volume (7d)"), "AnalyticsChart must render the bar chart card. Got: {html}");
    }

    /// Smoke test for `<AnalyticsTable>`. Renders the 6-col table
    /// with the top events.
    #[test]
    fn test_render_smoke_analytics_table() {
        let html = dioxus_ssr::render_element(rsx! { AnalyticsTable {} });
        assert!(html.contains("analytics-table"), "AnalyticsTable must keep its section marker. Got: {html}");
        assert!(html.contains("wallet.connect"), "AnalyticsTable must render at least one event row. Got: {html}");
    }

    /// Smoke test for `<AnalyticsMetadata>`. Renders the 3-col
    /// metadata row.
    #[test]
    fn test_render_smoke_analytics_metadata() {
        let html = dioxus_ssr::render_element(rsx! { AnalyticsMetadata {} });
        assert!(html.contains("analytics-metadata"), "AnalyticsMetadata must keep its section marker. Got: {html}");
        assert!(html.contains("PostgreSQL analytics_prod"), "AnalyticsMetadata must render the data source. Got: {html}");
    }

    /// Smoke test for `<AnalyticsExportDialog>`. The dialog is
    /// closed in the smoke test (so the body is hidden), but the
    /// `data-section` marker must always be in the DOM.
    #[test]
    fn test_render_smoke_analytics_export_dialog() {
        let mut vdom = dioxus::prelude::VirtualDom::new(ExportDialogHarness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("data-section=\"analytics-export-dialog\""), "AnalyticsExportDialog must render its data-section marker. Got: {html}");
    }
}

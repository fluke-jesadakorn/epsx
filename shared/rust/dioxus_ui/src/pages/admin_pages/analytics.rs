//! /admin/analytics — system-wide analytics dashboard.
//!
//! Wave 6B Track A — port of `apps-old/admin-frontend/app/analytics/page.tsx`
//! (78 LoC) + `analytics-dashboard.tsx` (337 LoC) + the
//! `analytics-card-grid.tsx` subdir (SummarySection + StatsSection +
//! MetricsGrid).
//!
//! Sections (per design doc §"Track A" line 167):
//! - `analytics-header` — page title + live/AI-Powered status pills.
//! - `analytics-card-grid` — 4-card summary grid (Total Users, API
//!   Requests, Active Permissions, System Health).
//! - `analytics-chart` — line/bar chart of platform activity.
//! - `analytics-table` — top events table.
//! - `analytics-filter-panel` — search/sort/scope filter chips.
//! - `analytics-export-dialog` — ExportDialog primitive (Wave 6A's
//!   `<ExportDialog>` from `data/export_dialog.rs`).
//! - `analytics-metadata` — generated-on / data-source / schema-version
//!   metadata row.
//!
//! Reuses Wave 6A's `<ExportDialog>` primitive — the design doc says
//! "reuse Wave 6A's `<ExportDialog>` primitive from
//! `shared/rust/dioxus_ui/src/data/export_dialog.rs`".

use crate::auth::AdminAuthGate;
use crate::charts::{ChartBar, ChartLine, DataPoint, Series};
use crate::data::export_dialog::{ExportDialog, ExportFormat, ExportScope};
use crate::layout::admin_shell::AdminShell;
use crate::primitives::*;

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Analytics");
    (meta, rsx! { RenderAnalytics { ctx: ctx.clone() } })
}

#[component]
fn RenderAnalytics(ctx: PageContext) -> Element {
    let mut export_open = use_signal(|| false);
    rsx! {
        AdminAuthGate { user: ctx.user.clone(), feature: Some("platform analytics".to_string()), required_permissions: Some(vec!["analytics:read".to_string()]), return_url: Some(ctx.path.clone()),
            AdminShell {
                ctx: ctx.clone(),
                page_title: "Platform analytics".to_string(),
                breadcrumbs: vec![
                    ("Dashboard".to_string(), "/".to_string()),
                    ("Analytics".to_string(), "/analytics".to_string()),
                ],
                div { class: "container page-content admin-analytics",
                    // AnalyticsHeader — title + status pills.
                    AnalyticsHeader { on_export: move |_| export_open.set(true) }
                    // AnalyticsFilterPanel — search + sort + scope.
                    AnalyticsFilterPanel {}
                    // AnalyticsCardGrid — 4-card summary row.
                    AnalyticsCardGrid {}
                    // AnalyticsChart — line + bar chart row.
                    AnalyticsChart {}
                    // AnalyticsTable — top events table.
                    AnalyticsTable {}
                    // AnalyticsMetadata — generated-on / data-source / schema-version row.
                    AnalyticsMetadata {}
                    // AnalyticsExportDialog — wraps the Wave 6A primitive
                    // with a `data-section` marker so the test assertion
                    // works even when the dialog is closed.
                    div { "data-section": "analytics-export-dialog", class: "analytics-export-dialog",
                        ExportDialog {
                            open: *export_open.read(),
                            on_close: move |_| export_open.set(false),
                            on_export: move |fmt| {
                                let label = match fmt {
                                    ExportFormat::Csv => "csv",
                                    ExportFormat::Json => "json",
                                    ExportFormat::Parquet => "parquet",
                                };
                                // The actual download is wired by the
                                // BFF (the page just bubbles the
                                // format choice up). We log via
                                // `eprintln!` since `tracing` is not
                                // available in this crate.
                                eprintln!("[analytics] export triggered format={label}");
                                export_open.set(false);
                            },
                            default_format: ExportFormat::Json,
                            default_scope: ExportScope::Filtered,
                            title: Some("Export analytics data".to_string()),
                            description: Some("Choose the format and scope for the analytics snapshot.".to_string()),
                        }
                    }
                }
            }
        }
    }
}

// ===== AnalyticsHeader =====================================================
//
// Source: the page header rows of `analytics-dashboard.tsx` lines
// 89-101 — title + "Refresh Data" button. We keep the status pills
// from `app/analytics/page.tsx` lines 56-64 (Live / AI-Powered) as
// visual chips on the right.

#[component]
fn AnalyticsHeader(on_export: EventHandler<MouseEvent>) -> Element {
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

#[component]
fn AnalyticsFilterPanel() -> Element {
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

#[component]
fn AnalyticsCardGrid() -> Element {
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

#[component]
fn SummaryCard(label: String, value: String, sub: String) -> Element {
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

#[component]
fn MetricCard(label: String, value: String, trend: String, color: String) -> Element {
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

#[component]
fn AnalyticsChart() -> Element {
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

#[component]
fn AnalyticsTable() -> Element {
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

#[component]
fn AnalyticsMetadata() -> Element {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::{AuthMethod, User};

    /// Authenticated admin context — the page gates on
    /// `analytics:read`, so the fixture user must hold that
    /// permission to pass the gate.
    fn admin_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "u-admin".to_string(),
                address: "0x1234abcd5678ef90".to_string(),
                chain_id: "56".to_string(),
                roles: vec!["admin".to_string()],
                email: Some("admin@epsx.io".to_string()),
                tier: Some("Admin".to_string()),
                permissions: vec!["analytics:read".to_string()],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: Some("Admin".to_string()),
            }),
            path: "/analytics".to_string(),
            ..Default::default()
        }
    }

    /// Smoke test: `render()` returns a non-empty Element.
    #[test]
    fn test_render_smoke() {
        let ctx = admin_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(!html.trim().is_empty(), "analytics must render non-empty HTML. Got: {html}");
        assert!(html.len() > 100, "analytics HTML is suspiciously short ({} bytes).", html.len());
    }

    /// Section-marker test: the rendered HTML must contain every
    /// design-doc-named section.
    #[test]
    fn test_section_markers() {
        let ctx = admin_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        for marker in &[
            "analytics-header",
            "analytics-card-grid",
            "analytics-chart",
            "analytics-table",
            "analytics-filter-panel",
            "analytics-export-dialog",
            "analytics-metadata",
        ] {
            // 4-form matcher (single class, first/middle/last of a
            // class list). The export-dialog marker is via
            // `data-section=` so we use the 2nd form for that one.
            let needle_a = format!("class=\"{}\"", marker);
            let needle_b = format!("class=\"{mark} ", mark = marker);
            let needle_c = format!(" {}\"", marker);
            let needle_d = format!(" {} ", marker);
            let needle_e = format!("data-section=\"{}\"", marker);
            assert!(
                html.contains(&needle_a)
                    || html.contains(&needle_b)
                    || html.contains(&needle_c)
                    || html.contains(&needle_d)
                    || html.contains(&needle_e),
                "analytics must contain section marker `{marker}`. Got: {html}"
            );
        }
    }
}

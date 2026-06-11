//! /admin/analytics — system-wide analytics dashboard.
//!
//! Wave 6C Track B — 1:1 component parity with the Next.js source
//! `apps-old/admin-frontend/app/analytics/page.tsx` +
//! `analytics-dashboard.tsx` + the analytics sub-components. The
//! page composes the extracted sub-components from
//! `crate::components::admin::analytics::*`.
//!
//! Sections (per design doc §"Track A" line 167) — markers
//! asserted by `test_section_markers`:
//! - `analytics-header` — page title + live/AI-Powered status pills.
//! - `analytics-card-grid` — 4-card summary grid (Total Users, API
//!   Requests, Active Permissions, System Health) + secondary row.
//! - `analytics-chart` — line/bar chart of platform activity.
//! - `analytics-table` — top events table.
//! - `analytics-filter-panel` — search/sort/scope filter chips.
//! - `analytics-export-dialog` — `<ExportDialog>` primitive
//!   wrapped in an always-present `data-section` div.
//! - `analytics-metadata` — generated-on / data-source /
//!   schema-version metadata row.

use crate::auth::AdminAuthGate;
use crate::components::admin::analytics::{
    AnalyticsCardGrid, AnalyticsChart, AnalyticsExportDialog, AnalyticsFilterPanel,
    AnalyticsHeader, AnalyticsMetadata, AnalyticsTable,
};
use crate::data::export_dialog::{ExportFormat, ExportScope};
use crate::layout::admin_shell::AdminShell;

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
                    // `AnalyticsHeader` — title + status pills + Export.
                    AnalyticsHeader { on_export: move |_| export_open.set(true) }
                    // `AnalyticsFilterPanel` — search + sort + scope.
                    AnalyticsFilterPanel {}
                    // `AnalyticsCardGrid` — 4-card summary row.
                    AnalyticsCardGrid {}
                    // `AnalyticsChart` — line + bar chart row.
                    AnalyticsChart {}
                    // `AnalyticsTable` — top events table.
                    AnalyticsTable {}
                    // `AnalyticsMetadata` — generated-on / data-source / schema-version row.
                    AnalyticsMetadata {}
                    // `AnalyticsExportDialog` — wraps the Wave 6A
                    // `<ExportDialog>` primitive with a stable
                    // `data-section` marker.
                    AnalyticsExportDialog {
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

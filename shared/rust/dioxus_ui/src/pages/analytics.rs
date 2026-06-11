//! /analytics — analytics dashboard.
//!
//! Wave 6C Track E — the 9 analytics sub-components
//! (`AnalyticsHeader`, `AnalyticsPlanStatusBar`, `FilterPanel`,
//! `AnalyticsCardGrid`, `AnalyticsChart`, `AnalyticsTable`,
//! `AnalyticsMetadata`, `AnalyticsMetadataCard`, `AnalyticsPageBody`)
//! were extracted to `crate::components::user::analytics`. The
//! `AnalyticsRankingCard` data type and the sample data helpers
//! (`sample_rankings`, `sample_events`, `sample_pnl_series`,
//! `sample_volume_series`) are also lifted and `pub`.

use crate::components::user::analytics::AnalyticsPageBody;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;

/// Top-level `render` — composes the 8 sections. The dialog state is
/// local to `AnalyticsPageBody` (the page is a self-contained unit;
/// in a real hydration pass it would lift into a global store).
pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Analytics");
    let body = rsx! { AnalyticsPageBody { ctx: ctx.clone() } };
    (meta, body)
}

// === wave6-auth-pages-depth-track-b ===
// Unit tests for the analytics page.
#[cfg(test)]
mod tests {
    use super::*;

    fn authed_ctx() -> PageContext {
        PageContext {
            user: Some(crate::auth::User {
                id: "test-user".to_string(),
                address: "0xtest".to_string(),
                chain_id: "56".to_string(),
                roles: vec!["user".to_string()],
                email: None,
                tier: Some("Pro".to_string()),
                permissions: vec!["analytics:read".to_string()],
                last_login_at: None,
                auth_method: crate::auth::AuthMethod::Wallet,
                display_name: Some("Test".to_string()),
            }),
            path: "/analytics".to_string(),
            ..Default::default()
        }
    }

    fn render_to_string(ctx: &PageContext) -> String {
        let (_meta, el) = render(ctx);
        dioxus_ssr::render_element(el)
    }

    /// Wave 6A — `test_render_smoke`. The `render` function returns
    /// a non-empty HTML string.
    #[test]
    fn test_render_smoke() {
        let ctx = authed_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(
            !html.trim().is_empty(),
            "analytics page should render non-empty HTML"
        );
    }

    /// Wave 6A — `test_section_markers`. The rendered HTML must
    /// contain each of the 8 design-doc section markers.
    #[test]
    fn test_section_markers() {
        let html = render_to_string(&authed_ctx());
        for marker in &[
            "analytics-header",
            "analytics-plan-status",
            "analytics-filter-panel",
            "analytics-card-grid",
            "analytics-chart",
            "analytics-table",
            "analytics-export-dialog",
            "analytics-metadata",
        ] {
            assert!(
                html.contains(marker),
                "analytics page should contain section marker `{marker}`. Got: {html}"
            );
        }
    }
}

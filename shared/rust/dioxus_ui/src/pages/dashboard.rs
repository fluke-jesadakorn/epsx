//! /dashboard — the EPSX public dashboard.
//!
//! Wave 6C Track E — the 9 dashboard sub-components were extracted
//! to `crate::components::user::dashboard`. The `DashboardData` +
//! `Activity` data types are also lifted and `pub`.

use crate::components::user::dashboard::RenderDashboard;
use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Dashboard");
    (meta, rsx! { RenderDashboard { ctx: ctx.clone() } })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::User;
    use crate::auth::user::AuthMethod;

    fn authed_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "u-1".to_string(),
                address: "0x1234abcd".to_string(),
                chain_id: "1".to_string(),
                roles: vec!["user".to_string()],
                email: Some("test@epsx.io".to_string()),
                tier: Some("pro".to_string()),
                permissions: vec!["dashboard:read".to_string()],
                last_login_at: None,
                auth_method: AuthMethod::default(),
                display_name: Some("EPSX tester".to_string()),
            }),
            path: "/dashboard".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_render_smoke() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        assert!(!html.is_empty(), "dashboard must render non-empty HTML. Got: {}", html);
        assert!(html.len() > 100, "dashboard HTML is suspiciously short ({} bytes).", html.len());
    }

    #[test]
    fn test_section_markers() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        for marker in &[
            "stat-cards-row",
            "dashboard-earnings-chart",
            "watchlist-snapshot",
            "plan-summary-card",
            "activity-card",
            "quick-actions-card",
            "your-account-card",
        ] {
            let needle_a = format!("class=\"{}\"", marker);
            let needle_b = format!("class=\"{mark} ", mark = marker);
            let needle_c = format!(" {}\"", marker);
            let needle_d = format!(" {} ", marker);
            assert!(
                html.contains(&needle_a)
                    || html.contains(&needle_b)
                    || html.contains(&needle_c)
                    || html.contains(&needle_d),
                "dashboard must contain section marker '{}'. Got: {}",
                marker, html
            );
        }
    }

    #[test]
    fn test_recent_activity_emptystate() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("href=\"/plans\""),
            "dashboard activity empty-state must link to /plans. Got: {}",
            html
        );
    }
}

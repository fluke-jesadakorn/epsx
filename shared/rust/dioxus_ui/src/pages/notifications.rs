//! /notifications — notification center.
//!
//! Wave 6C Track E — the 8 notifications sub-components were
//! extracted to `crate::components::user::notifications`. The
//! `Notification` data type is also lifted and `pub`.

use crate::components::user::notifications::RenderNotifications;
use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Notifications");
    (meta, rsx! { RenderNotifications { ctx: ctx.clone() } })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::{AuthMethod, User};

    fn notif_ctx(user_perms: &[&str]) -> PageContext {
        let user = User {
            id: "u1".to_string(),
            address: "0x1234…abcd".to_string(),
            chain_id: "56".to_string(),
            roles: vec!["user".to_string()],
            email: None,
            tier: Some("Pro".to_string()),
            permissions: user_perms.iter().map(|s| s.to_string()).collect(),
            last_login_at: None,
            auth_method: AuthMethod::Wallet,
            display_name: None,
        };
        PageContext {
            user: Some(user),
            path: "/notifications".to_string(),
            ..Default::default()
        }
    }

    /// Wave 6A — `test_render_smoke`. Notifications page must
    /// render non-empty HTML.
    #[test]
    fn test_render_smoke() {
        let ctx = notif_ctx(&["notifications:read"]);
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(!html.is_empty(), "Notifications page must render non-empty HTML.");
        assert!(html.len() > 200, "Notifications HTML is suspiciously short ({} bytes).", html.len());
    }

    /// Wave 6A — `test_section_markers`. The notifications page
    /// must render every section the design doc claims.
    #[test]
    fn test_section_markers() {
        let ctx = notif_ctx(&["notifications:read"]);
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        for marker in &[
            "notifications-page",
            "notifications-list",
            "notifications-filterbar",
            "notifications-list-card",
            "notification-row",
            "notification-icon",
            "browser-notifications",
            "browser-notifications-header",
            "browser-notifications-prompt",
            "permission-badge",
            "notification-settings",
            "notification-settings-heading",
            "notification-settings-types",
        ] {
            let needle = format!("class=\"{}\"", marker);
            let found = html.contains(&needle)
                || html.contains(&format!("\"{} ", marker))
                || html.contains(&format!(" {} ", marker))
                || html.contains(&format!(" {}", marker));
            assert!(
                found,
                "Notifications page must contain section marker '{}'. Got: {}",
                needle, html
            );
        }
    }
}

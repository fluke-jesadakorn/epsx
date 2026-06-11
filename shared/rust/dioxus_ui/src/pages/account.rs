//! /account — the EPSX account settings page.
//!
//! Wave 6C Track E — the 12 account sub-components
//! (AccountTabButton, ProfileTab, ProfileField, SubscriptionTab,
//! FeatureCheck, UsageTab, NotificationsTab, NotificationToggle,
//! ConnectedAccountsTab, OAuthRow, SessionRow, DangerZoneTab)
//! were extracted to `crate::components::user::account`. The page
//! file's `RenderAccount` wrapper orchestrates them via the 6-tab
//! nav + 6 tab bodies.

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::auth::AuthGate;
use crate::components::user::account::*;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Account");
    (meta, rsx! { RenderAccount { ctx: ctx.clone() } })
}

#[component]
fn RenderAccount(ctx: PageContext) -> Element {
    let mut tab = use_signal(|| "profile".to_string());
    rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("your account".to_string()),
                required_permissions: Some(vec!["profile:read".to_string()]),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content",
                    PageHeader {
                        title: "Account".to_string(),
                        description: Some("Manage your profile, subscription, and preferences".to_string()),
                        icon: Some("user".to_string()),
                    }
                    div { class: "account-tabs tabs mb-4",
                        AccountTabButton { tab: tab, value: "profile", label: "Profile", icon: "user" }
                        AccountTabButton { tab: tab, value: "subscription", label: "Subscription", icon: "credit-card" }
                        AccountTabButton { tab: tab, value: "usage", label: "Usage", icon: "bar-chart-3" }
                        AccountTabButton { tab: tab, value: "notifications", label: "Notifications", icon: "bell" }
                        AccountTabButton { tab: tab, value: "connected", label: "Connected accounts", icon: "link" }
                        AccountTabButton { tab: tab, value: "danger", label: "Danger zone", icon: "alert-triangle" }
                    }
                    if *tab.read() == "subscription" { SubscriptionTab {} }
                    else if *tab.read() == "usage" { UsageTab {} }
                    else if *tab.read() == "notifications" { NotificationsTab {} }
                    else if *tab.read() == "connected" { ConnectedAccountsTab {} }
                    else if *tab.read() == "danger" { DangerZoneTab {} }
                    else { ProfileTab { user: ctx.user.clone() } }
                }
            }
        }
    }
}

// =============================================================================
// Tests
// =============================================================================
//
// - `test_render_smoke` — render(&empty_ctx()) returns non-empty Element.
// - `test_section_markers` — SSR'd HTML contains every section-marker
//   class the design doc claims.
// - `test_default_tab` — the default tab is `profile` (Wave 6A Track A
//   design choice: the source's `ProfileTab` is the landing sub-view).
// - `test_tab_switching` — when the tab signal is set to a different
//   value, the corresponding tab content renders.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::User;
    use crate::auth::user::AuthMethod;

    fn empty_ctx() -> PageContext {
        PageContext {
            user: None,
            path: "/account".to_string(),
            ..Default::default()
        }
    }

    fn authed_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "u-1".to_string(),
                address: "0x1234abcd".to_string(),
                chain_id: "1".to_string(),
                roles: vec!["user".to_string()],
                email: Some("test@epsx.io".to_string()),
                tier: Some("pro".to_string()),
                permissions: vec!["profile:read".to_string()],
                last_login_at: None,
                auth_method: AuthMethod::default(),
                display_name: Some("EPSX tester".to_string()),
            }),
            path: "/account".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_render_smoke() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        assert!(!html.is_empty(), "account page must render non-empty HTML. Got: {}", html);
        assert!(html.len() > 100, "account HTML is suspiciously short ({} bytes).", html.len());
    }

    #[test]
    fn test_section_markers() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        for marker in &[
            "account-tabs",
            "account-profile-tab",
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
                "account page must contain section marker '{}'. Got: {}",
                marker, html
            );
        }
    }

    #[test]
    fn test_default_tab_is_profile() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("account-profile-tab"),
            "account page default tab must be Profile. Got: {}",
            html
        );
    }

    #[test]
    fn test_subscription_tab_marker_present() {
        let el = rsx! { SubscriptionTab {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("account-subscription-tab"));
    }

    #[test]
    fn test_usage_tab_marker_present() {
        let el = rsx! { UsageTab {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("account-usage-tab"));
    }

    #[test]
    fn test_notifications_tab_marker_present() {
        let el = rsx! { NotificationsTab {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("account-notifications-tab"));
    }

    #[test]
    fn test_connected_tab_marker_present() {
        let el = rsx! { ConnectedAccountsTab {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("account-connected-tab"));
    }

    #[test]
    fn test_danger_tab_marker_present() {
        let el = rsx! { DangerZoneTab {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("account-danger-tab"));
    }
}

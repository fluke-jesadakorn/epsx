//! Admin `WalletManagementTabs` component — Wave 38a T1 admin
//! wallet domain port.
//!
//! Mirrors `apps-old/admin-frontend/components/wallet/wallet-management-tabs.tsx`.
//! Renders the wallet-management sub-page tabs (Wallets / Credits /
//! Access / Plans). Each tab is a link with the right href +
//! active-state styling.
//!
//! ## Visual layout
//!
//! Horizontal flex row of 4 tabs. Active tab gets the cyan
//! border-bottom + cyan text; inactive tabs use the muted text
//! color + transparent border.
//!
//! ## Tests
//!
//! `test_wallet_management_tabs_renders_4_tabs` — all 4 tabs
//! render with the right labels.
//! `test_wallet_management_tabs_active_state` — the active tab
//! gets the cyan border + text.
//! `test_wallet_management_tabs_hrefs` — each tab's href points
//! to the right sub-route.
//! `test_wallet_management_tabs_propagates_class_name` — caller
//! `class_name` is appended.

use dioxus::prelude::*;

/// Wallet-management sub-page tabs. Caller passes the active tab
/// key + an optional class_name.
#[component]
pub fn WalletManagementTabs(
    /// Active tab key: `"wallets"`, `"credits"`, `"access"`, or
    /// `"plans"`.
    active: String,
    /// Optional extra classes appended to the outer wrapper.
    class_name: Option<String>,
) -> Element {
    let mut cls = "flex items-center gap-1 border-b border-border/20".to_string();
    if let Some(extra) = class_name {
        if !extra.is_empty() {
            cls.push(' ');
            cls.push_str(&extra);
        }
    }
    let tabs: [(&str, &str, &str); 4] = [
        ("wallets", "Wallets", "/wallet-management/wallets"),
        ("credits", "Credits", "/wallet-management/credits"),
        ("access", "Access", "/wallet-management/access"),
        ("plans", "Plans", "/wallet-management/plans"),
    ];
    rsx! {
        nav { class: "{cls}", role: "tablist",
            for t in tabs.iter() {
                {
                    let is_active = t.0 == active.as_str();
                    let mut tab_cls = "px-4 py-2.5 text-sm font-medium border-b-2 transition-colors".to_string();
                    if is_active {
                        tab_cls.push_str(" border-[#1fc7d4] text-[#1fc7d4]");
                    } else {
                        tab_cls.push_str(" border-transparent text-muted-foreground hover:text-foreground hover:border-border/40");
                    }
                    let href = t.2.to_string();
                    rsx! {
                        a {
                            key: "{t.0}",
                            href: "{href}",
                            class: "{tab_cls}",
                            role: "tab",
                            "aria-selected": if is_active { "true" } else { "false" },
                            "{t.1}"
                        }
                    }
                }
            }
        }
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// All 4 tabs render with the right labels.
    #[test]
    fn test_wallet_management_tabs_renders_4_tabs() {
        let el = rsx! {
            WalletManagementTabs {
                active: "wallets".to_string(),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("Wallets"), "Wallets tab. Got: {html}");
        assert!(html.contains("Credits"), "Credits tab. Got: {html}");
        assert!(html.contains("Access"), "Access tab. Got: {html}");
        assert!(html.contains("Plans"), "Plans tab. Got: {html}");
    }

    /// Active tab gets the cyan border + text.
    #[test]
    fn test_wallet_management_tabs_active_state() {
        let el = rsx! {
            WalletManagementTabs {
                active: "credits".to_string(),
            }
        };
        let html = dioxus_ssr::render_element(el);
        // Credits tab is active.
        assert!(
            html.contains("border-[#1fc7d4]"),
            "Active tab should have cyan border. Got: {html}"
        );
        assert!(
            html.contains("text-[#1fc7d4]"),
            "Active tab should have cyan text. Got: {html}"
        );
        assert!(
            html.contains("aria-selected=\"true\""),
            "Active tab should have aria-selected=true. Got: {html}"
        );
    }

    /// Each tab's href points to the right sub-route.
    #[test]
    fn test_wallet_management_tabs_hrefs() {
        let el = rsx! {
            WalletManagementTabs {
                active: "wallets".to_string(),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("href=\"/wallet-management/wallets\""),
            "Wallets href. Got: {html}"
        );
        assert!(
            html.contains("href=\"/wallet-management/credits\""),
            "Credits href. Got: {html}"
        );
        assert!(
            html.contains("href=\"/wallet-management/access\""),
            "Access href. Got: {html}"
        );
        assert!(
            html.contains("href=\"/wallet-management/plans\""),
            "Plans href. Got: {html}"
        );
    }

    /// Inactive tab gets `border-transparent` (no cyan border).
    #[test]
    fn test_wallet_management_tabs_inactive_state() {
        let el = rsx! {
            WalletManagementTabs {
                active: "wallets".to_string(),
            }
        };
        let html = dioxus_ssr::render_element(el);
        // Plans tab is inactive.
        assert!(
            html.contains("border-transparent"),
            "Inactive tab should have transparent border. Got: {html}"
        );
        assert!(
            html.contains("text-muted-foreground"),
            "Inactive tab should use muted text. Got: {html}"
        );
    }

    /// `class_name` is propagated.
    #[test]
    fn test_wallet_management_tabs_propagates_class_name() {
        let el = rsx! {
            WalletManagementTabs {
                active: "wallets".to_string(),
                class_name: Some("mb-6".to_string()),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("mb-6"),
            "class_name should propagate. Got: {html}"
        );
    }
}

//! Admin header — full 1:1 port of
//! `apps-old/admin-frontend/components/layout/header.tsx`.
//!
//! Composition:
//! - Sticky top-0, z-40, border-b border-border/40, bg-card
//! - Left: `Breadcrumb` slot
//! - Right: notification bell slot, vertical separator, theme toggle slot,
//!   chain selector slot (dev-only), `WalletConnectButton`
//!
//! Slots (slots are normal `Option<Element>` props in Dioxus 0.7 — there
//! is no `<Slot>` concept; consumers pass `Some(rsx!{ ... })`).
//!
//! Backward-compat: the legacy `header` scaffold is not present in the
//! existing public API; this module is purely additive.

use crate::auth::User;
use crate::primitives::icon::Icon;

use dioxus::prelude::*;

/// Notification payload (subset of `ApiNotification`). Kept local so the
/// header doesn't depend on a particular SSE/REST schema.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct HeaderNotification {
    pub id: String,
    pub title: String,
    pub body: Option<String>,
    pub read: bool,
}

/// Admin header — sticky top bar with breadcrumb + actions.
///
/// Mirrors `Header` from `header.tsx`. All slots are optional: when a
/// slot is `None`, the corresponding area collapses (no placeholder
/// rendered). The `chain_selector` slot is only rendered when
/// `is_production` is `false` (matches the TS source's
/// `{!isProduction && <ChainSelector />}` guard).
#[component]
pub fn Header(
    /// Optional authenticated user used by the production-shaped wallet
    /// trigger and account dropdown.
    user: Option<crate::auth::User>,
    /// Pre-fetched notifications to seed the bell. Optional.
    initial_notifications: Option<Vec<HeaderNotification>>,
    /// Pre-fetched unread count. Defaults to the count of
    /// `initial_notifications` where `read == false` when `None`.
    initial_unread_count: Option<u32>,
    /// Current pathname. Used by the breadcrumb slot default render.
    current_path: Option<String>,
    /// Whether the runtime is production. When `true`, the chain
    /// selector slot is hidden (matches the TS source).
    is_production: Option<bool>,
    /// Optional custom breadcrumb slot. When `None`, a default
    /// `Breadcrumb` is rendered with the `current_path` prop.
    breadcrumb: Option<Element>,
    /// Optional custom notification-bell slot. When `None`, a
    /// default bell button is rendered with the unread count.
    notification_bell: Option<Element>,
    /// Optional custom theme-toggle slot. When `None`, a default
    /// `IconButton` is rendered.
    theme_toggle: Option<Element>,
    /// Optional chain-selector slot. Hidden entirely when
    /// `is_production` is `Some(true)`. Ignored when production.
    chain_selector: Option<Element>,
    /// Optional click handler fired when the user clicks the bell.
    on_bell_click: Option<EventHandler<MouseEvent>>,
    /// Optional click handler fired when the user clicks the theme
    /// toggle (only used for the default `IconButton`).
    on_theme_toggle: Option<EventHandler<MouseEvent>>,
    /// `class_name` override for the outer `<header>` element.
    class_name: Option<String>,
    /// Optional id for the outer `<header>` element.
    id: Option<String>,
    /// SSR-verified session truth. When `Fixture`, an amber badge marks the
    /// UI-only bypass identity next to the wallet control.
    #[props(default = None)]
    session_state: Option<crate::layout::session_state::SessionState>,
) -> Element {
    // Compute effective unread count.
    let effective_unread = initial_unread_count.unwrap_or_else(|| {
        initial_notifications
            .as_ref()
            .map(|v| v.iter().filter(|n| !n.read).count() as u32)
            .unwrap_or(0)
    });
    let show_chain = !is_production.unwrap_or(false) && chain_selector.is_some();
    let current_path = current_path.unwrap_or_else(|| "/".to_string());
    let default_notifications = initial_notifications.unwrap_or_default();
    let header_class = {
        let mut c = String::from(
            "sticky top-0 z-40 border-b border-border/40 bg-card admin-header admin-header-chrome",
        );
        if let Some(extra) = class_name {
            c.push(' ');
            c.push_str(&extra);
        }
        c
    };

    rsx! {
        header { class: "{header_class}", id: id.clone(),
            div { class: "flex h-16 w-full items-center justify-between px-6 gap-3",
                // Left: breadcrumb
                div { class: "flex items-center gap-2 min-w-0 flex-shrink",
                    if let Some(b) = breadcrumb {
                        {b}
                    } else {
                        // Default breadcrumb — uses the auto-generated
                        // `<Breadcrumb current_path />` component.
                        crate::layout::Breadcrumb { current_path: current_path.clone() }
                    }
                }

                // Right: actions
                div { class: "flex items-center gap-3 flex-shrink-0",
                    if session_state == Some(crate::layout::session_state::SessionState::Fixture) {
                        span {
                            class: "hidden sm:inline-flex items-center rounded-full border border-amber-400/40 bg-amber-400/10 px-2.5 py-0.5 text-[10px] font-bold uppercase tracking-wide text-amber-400",
                            "data-session-state": "fixture",
                            title: "UI-only fixture identity — no backend session",
                            "Fixture"
                        }
                    }
                    // Notification bell
                    div { class: "hidden sm:block",
                        if let Some(bell) = notification_bell {
                            {bell}
                        } else {
                            DefaultBell {
                                unread: effective_unread,
                                notifications: default_notifications.clone(),
                                on_bell_click: on_bell_click,
                            }
                        }
                    }

                    // Vertical separator
                    div { class: "w-[1px] h-6 bg-border hidden sm:block" }

                    // Theme toggle
                    if let Some(toggle) = theme_toggle {
                        {toggle}
                    } else {
                        button {
                            class: "btn btn-ghost btn-icon admin-header-theme-toggle",
                            r#type: "button",
                            title: "Toggle theme",
                            "aria-label": "Toggle theme",
                            "data-epsx-action": "theme-toggle",
                            onclick: move |e| if let Some(h) = &on_theme_toggle { h.call(e); },
                            span { class: "admin-theme-icon admin-theme-icon-sun",
                                Icon { name: "sun".to_string(), size: Some(16) }
                            }
                            span { class: "admin-theme-icon admin-theme-icon-moon",
                                Icon { name: "moon".to_string(), size: Some(16) }
                            }
                        }
                    }

                    // Chain selector (dev only)
                    if show_chain {
                        div { class: "hidden lg:block",
                            {chain_selector.unwrap()}
                        }
                    }

                    // Production admin wallet trigger + dropdown.
                    AdminWalletControl {
                        user: user.clone(),
                        return_url: current_path.clone(),
                    }
                }
            }
        }
    }
}

/// Default notification-bell button. Renders an icon + unread badge.
#[component]
fn DefaultBell(
    unread: u32,
    notifications: Vec<HeaderNotification>,
    on_bell_click: Option<EventHandler<MouseEvent>>,
) -> Element {
    let aria = if unread > 0 {
        format!("unread notifications: {unread}")
    } else {
        "Notifications".to_string()
    };
    let badge_text = if unread > 99 {
        "99+".to_string()
    } else {
        unread.to_string()
    };
    rsx! {
        div { class: "admin-header-menu-wrap",
            button {
                class: "btn btn-ghost btn-icon relative admin-header-bell",
                r#type: "button",
                title: "Notifications",
                "aria-label": "{aria}",
                "aria-haspopup": "menu",
                "aria-expanded": "false",
                "aria-controls": "admin-notifications-menu",
                "data-epsx-action": "toggle-dropdown",
                onclick: move |e| if let Some(h) = &on_bell_click { h.call(e); },
                Icon {
                    name: "bell".to_string(),
                    size: Some(16),
                    class_name: Some("admin-header-bell-icon".to_string()),
                }
                if unread > 0 {
                    span { class: "absolute -top-0.5 -right-0.5 min-w-[16px] h-4 px-1 rounded-full bg-gradient-to-r from-violet-500 to-purple-500 text-white text-[10px] font-bold flex items-center justify-center shadow-sm shadow-violet-500/30",
                        "{badge_text}"
                    }
                }
            }
            div {
                id: "admin-notifications-menu",
                class: "admin-header-popover admin-notifications-menu",
                role: "menu",
                hidden: true,
                "aria-hidden": "true",
                "data-epsx-dropdown": "true",
                div { class: "admin-notifications-heading",
                    h2 { "Notifications" }
                    if unread > 0 {
                        span { "{unread} unread" }
                    }
                }
                if notifications.is_empty() {
                    div { class: "admin-notifications-empty",
                        Icon { name: "bell".to_string(), size: Some(24) }
                        p { "No notifications" }
                        span { "You're all caught up." }
                    }
                } else {
                    div { class: "admin-notifications-list",
                        for notification in notifications.iter().take(6) {
                            a {
                                class: if notification.read { "admin-notification-item" } else { "admin-notification-item admin-notification-item-unread" },
                                href: "/notifications",
                                role: "menuitem",
                                strong { "{notification.title}" }
                                if let Some(body) = &notification.body {
                                    span { "{body}" }
                                }
                            }
                        }
                    }
                }
                a {
                    class: "admin-notifications-view-all",
                    href: "/notifications",
                    role: "menuitem",
                    Icon { name: "external-link".to_string(), size: Some(14) }
                    span { "View all notifications" }
                }
            }
        }
    }
}

/// Production-shaped connected-wallet control for the admin header. The
/// node-free browser runtime owns only the disclosure state; navigation,
/// copy, and logout continue to use the shared progressive-action contract.
#[component]
fn AdminWalletControl(user: Option<User>, return_url: String) -> Element {
    let Some(user) = user else {
        let href = format!("/auth?return_url={}", encode_query_value(&return_url));
        return rsx! {
            a {
                class: "admin-wallet-connect",
                href,
                Icon { name: "wallet".to_string(), size: Some(16) }
                span { class: "admin-wallet-connect-label", "Connect Wallet" }
            }
        };
    };

    let address = user.address.clone();
    let short_address = user.short_address();
    let role = user
        .roles
        .first()
        .cloned()
        .unwrap_or_else(|| "admin".to_string())
        .replace('_', " ");
    let tier = user.tier.clone();
    let permission_count = user.permissions.len();
    let (network_class, network_label) = match user.chain_id.as_str() {
        "56" => ("wallet-network-live", "BSC Mainnet".to_string()),
        "97" => ("wallet-network-testnet", "BSC Testnet".to_string()),
        value if !value.is_empty() => ("wallet-network-other", format!("Chain {value}")),
        _ => ("wallet-network-other", "Unknown network".to_string()),
    };
    let explorer_href = is_evm_address(&address).then(|| {
        let host = if user.chain_id == "97" {
            "https://testnet.bscscan.com/address/"
        } else {
            "https://bscscan.com/address/"
        };
        format!("{host}{address}")
    });
    // UI-only development identities deliberately use a non-canonical
    // address. Do not attach the injected-wallet watcher to those sessions:
    // a real MetaMask account can never match the preview identity, which
    // would otherwise cause logout -> reload -> bypass login loops.
    let provider_watch = is_evm_address(&address);
    let wallet_aria = format!("Wallet menu for {short_address}");

    rsx! {
        div {
            class: "admin-header-menu-wrap admin-wallet-control",
            "data-epsx-session-wallet": provider_watch.then_some(address.as_str()),
            "data-wallet-provider-watch": provider_watch.then_some("metamask"),
            button {
                class: "admin-wallet-trigger",
                r#type: "button",
                "aria-label": "{wallet_aria}",
                "aria-haspopup": "menu",
                "aria-expanded": "false",
                "aria-controls": "admin-wallet-menu",
                "data-epsx-action": "toggle-dropdown",
                Icon { name: "wallet".to_string(), size: Some(16) }
                span { class: "admin-wallet-short-address", "{short_address}" }
                Icon {
                    name: "chevron-down".to_string(),
                    size: Some(12),
                    class_name: Some("admin-wallet-chevron".to_string()),
                }
            }
            div {
                id: "admin-wallet-menu",
                class: "admin-header-popover admin-wallet-menu",
                role: "menu",
                hidden: true,
                "aria-hidden": "true",
                "data-epsx-dropdown": "true",
                div { class: "admin-wallet-accent" }
                div { class: "admin-wallet-address-block",
                    div { class: "admin-wallet-label",
                        Icon { name: "wallet".to_string(), size: Some(12) }
                        span { "Wallet" }
                    }
                    p { "{address}" }
                }
                div { class: "wallet-meta-grid",
                    div { class: "wallet-meta-cell",
                        div { class: "wallet-meta-label", "Role" }
                        div { class: "wallet-meta-value wallet-meta-value-role", "{role}" }
                    }
                    if let Some(tier) = tier {
                        div { class: "wallet-meta-cell",
                            div { class: "wallet-meta-label", "Tier" }
                            div { class: "wallet-meta-value wallet-meta-value-tier", "{tier}" }
                        }
                    }
                    if permission_count > 0 {
                        div { class: "wallet-meta-cell",
                            div { class: "wallet-meta-label", "Permissions" }
                            div { class: "wallet-meta-value", "{permission_count}" }
                        }
                    }
                }
                div { class: "wallet-network-badge {network_class}",
                    span { class: "wallet-network-dot" }
                    span { "{network_label}" }
                }
                div { class: "admin-wallet-actions",
                    button {
                        class: "admin-wallet-menu-item",
                        r#type: "button",
                        role: "menuitem",
                        "data-epsx-action": "copy",
                        "data-copy": "{address}",
                        Icon { name: "copy".to_string(), size: Some(16) }
                        span { "Copy address" }
                    }
                    if let Some(explorer_href) = explorer_href {
                        a {
                            class: "admin-wallet-menu-item",
                            href: explorer_href,
                            target: "_blank",
                            rel: "noopener noreferrer",
                            role: "menuitem",
                            Icon { name: "external-link".to_string(), size: Some(16) }
                            span { "View on explorer" }
                        }
                    }
                    div { class: "admin-wallet-menu-separator" }
                    button {
                        class: "admin-wallet-menu-item admin-wallet-disconnect",
                        r#type: "button",
                        role: "menuitem",
                        "data-epsx-logout": "true",
                        Icon { name: "log-out".to_string(), size: Some(16) }
                        span { "Disconnect" }
                    }
                }
            }
        }
    }
}

fn is_evm_address(value: &str) -> bool {
    value.len() == 42
        && value.starts_with("0x")
        && value.as_bytes()[2..].iter().all(u8::is_ascii_hexdigit)
}

fn encode_query_value(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::AuthMethod;

    fn admin_user() -> User {
        User {
            id: "admin-1".to_string(),
            address: "0x1111111111111111111111111111111111111111".to_string(),
            chain_id: "56".to_string(),
            roles: vec!["super_admin".to_string()],
            email: Some("admin@epsx.io".to_string()),
            tier: Some("Admin".to_string()),
            permissions: vec!["admin:*".to_string()],
            last_login_at: None,
            auth_method: AuthMethod::Wallet,
            display_name: None,
        }
    }

    fn rendered_header(user: Option<User>) -> String {
        dioxus_ssr::render_element(rsx! {
            Header {
                user,
                initial_notifications: None,
                initial_unread_count: Some(0),
                current_path: Some("/audit-log".to_string()),
                is_production: Some(true),
                breadcrumb: None,
                notification_bell: None,
                theme_toggle: None,
                chain_selector: None,
                on_bell_click: None,
                on_theme_toggle: None,
                class_name: None,
                id: None,
            }
        })
    }

    #[test]
    fn authenticated_header_matches_progressive_dropdown_contract() {
        let html = rendered_header(Some(admin_user()));
        assert!(html.contains("admin-header-chrome"));
        assert!(html.contains("h-16 w-full"));
        assert!(html.contains("data-epsx-action=\"theme-toggle\""));
        assert!(html.contains("aria-controls=\"admin-notifications-menu\""));
        assert!(html.contains("id=\"admin-notifications-menu\""));
        assert!(html.contains("aria-controls=\"admin-wallet-menu\""));
        assert!(html.contains("id=\"admin-wallet-menu\""));
        assert!(html.matches("data-epsx-action=\"toggle-dropdown\"").count() >= 2);
        assert!(html.contains("data-copy=\"0x1111111111111111111111111111111111111111\""));
        assert!(html
            .contains("data-epsx-session-wallet=\"0x1111111111111111111111111111111111111111\""));
        assert!(html.contains("data-wallet-provider-watch=\"metamask\""));
        assert!(html.contains("class=\"admin-wallet-short-address\">0x1111…1111</span>"));
        assert!(html.contains("View on explorer"));
        assert!(html.contains("Disconnect"));
        assert!(!html.contains("wallet-connect-legacy"));
    }

    #[test]
    fn signed_out_header_links_to_the_route_aware_auth_page() {
        let html = rendered_header(None);
        assert!(html.contains("class=\"admin-wallet-connect\""));
        assert!(html.contains("href=\"/auth?return_url=%2Faudit-log\""));
    }

    #[test]
    fn non_canonical_preview_identity_does_not_bind_wallet_synchronization() {
        let mut user = admin_user();
        user.id = "dev-bypass".to_string();
        user.address = "0x000000000000000000000000000000000000d3v1".to_string();
        user.auth_method = AuthMethod::Demo;

        let html = rendered_header(Some(user));
        assert!(html.contains("admin-wallet-control"));
        assert!(!html.contains("data-epsx-session-wallet"));
        assert!(!html.contains("data-wallet-provider-watch"));
    }

    #[test]
    fn explorer_link_is_limited_to_canonical_evm_addresses() {
        assert!(is_evm_address("0x1111111111111111111111111111111111111111"));
        assert!(!is_evm_address("dev-bypass"));
        assert!(!is_evm_address(
            "0xZZ11111111111111111111111111111111111111"
        ));
    }
}

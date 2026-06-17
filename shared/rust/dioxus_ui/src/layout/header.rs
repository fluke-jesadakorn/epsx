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

use crate::auth::WalletConnectButton;
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
    /// Optional authenticated user (unused at the moment — kept on the
    /// signature to match the TS source and to make future
    /// user-aware copy trivial to add).
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
) -> Element {
    // Compute effective unread count.
    let effective_unread = initial_unread_count.unwrap_or_else(|| {
        initial_notifications
            .as_ref()
            .map(|v| v.iter().filter(|n| !n.read).count() as u32)
            .unwrap_or(0)
    });
    let show_chain = !is_production.unwrap_or(false) && chain_selector.is_some();
    let header_class = {
        let mut c = String::from("sticky top-0 z-40 border-b border-border/40 bg-card admin-header");
        if let Some(extra) = class_name { c.push(' '); c.push_str(&extra); }
        c
    };

    rsx! {
        header { class: "{header_class}", id: id.clone(),
            div { class: "flex h-16 items-center justify-between px-6 gap-3",
                // Left: breadcrumb
                div { class: "flex items-center gap-2 min-w-0 flex-shrink",
                    if let Some(b) = breadcrumb {
                        {b}
                    } else {
                        // Default breadcrumb — uses the auto-generated
                        // `<Breadcrumb current_path />` component.
                        crate::layout::Breadcrumb { current_path: current_path.unwrap_or_else(|| "/".to_string()) }
                    }
                }

                // Right: actions
                div { class: "flex items-center gap-3 flex-shrink-0",
                    // Notification bell
                    div { class: "hidden sm:block",
                        if let Some(bell) = notification_bell {
                            {bell}
                        } else {
                            DefaultBell {
                                unread: effective_unread,
                                on_bell_click: on_bell_click.clone(),
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
                            onclick: move |e| if let Some(h) = &on_theme_toggle { h.call(e); },
                            Icon { name: "sun".to_string(), size: Some(16) }
                        }
                    }

                    // Chain selector (dev only)
                    if show_chain {
                        div { class: "hidden lg:block",
                            {chain_selector.unwrap()}
                        }
                    }

                    // Wallet connect
                    WalletConnectButton {
                        user: user.clone(),
                        on_connect: None,
                    }
                }
            }
        }
    }
}

/// Default notification-bell button. Renders an icon + unread badge.
#[component]
fn DefaultBell(unread: u32, on_bell_click: Option<EventHandler<MouseEvent>>) -> Element {
    let aria = if unread > 0 {
        format!("unread notifications: {unread}")
    } else {
        "Notifications".to_string()
    };
    let badge_text = if unread > 99 { "99+".to_string() } else { unread.to_string() };
    rsx! {
        button {
            class: "btn btn-ghost btn-icon relative admin-header-bell",
            r#type: "button",
            title: "Notifications",
            "aria-label": "{aria}",
            onclick: move |e| if let Some(h) = &on_bell_click { h.call(e); },
            Icon { name: "bell".to_string(), size: Some(16) }
            if unread > 0 {
                span { class: "absolute -top-0.5 -right-0.5 min-w-[16px] h-4 px-1 rounded-full bg-gradient-to-r from-violet-500 to-purple-500 text-white text-[10px] font-bold flex items-center justify-center shadow-sm shadow-violet-500/30",
                    "{badge_text}"
                }
            }
        }
    }
}

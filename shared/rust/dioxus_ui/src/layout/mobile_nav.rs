//! MobileNav — port of `apps-old/frontend/components/nav/mobile-nav.tsx`.
//!
//! Slide-in sheet (right side) that contains:
//! 1. Brand strip
//! 2. Wallet card (if `is_connected` + `wallet_address` set)
//! 3. Nav groups (each a collapsible accordion)
//! 4. Notifications link (when authenticated)
//! 5. Wallet button slot (bottom)
//!
//! Uses the Wave 1 `Sheet` primitive for the side-panel + `aria-modal`
//! + `aria-labelledby` plumbing. The hamburger trigger has
//! `aria-label="Open menu"` and is rendered as a sibling of the Sheet
//! (the Sheet primitive's `!open` short-circuit hides the children, so
//! the trigger must live outside).
//!
//! Wave 2 a11y: `role="dialog"` and `aria-modal="true"` are provided
//! by `<Sheet>`; this component sets the visible header inside the
//! sheet and the per-group accordion buttons.

use dioxus::prelude::*;

use crate::primitives::icon::Icon;
use crate::primitives::sheet::Sheet;

use super::nav_config::{is_group_active, is_item_active, NavGroup, NAV_GROUPS};

/// Mobile slide-in nav sheet. Mirrors the TS `MobileNav` component
/// 1:1, but takes wallet / auth state as props (the TS source reads
/// these from React context providers; in Dioxus the BFF supplies them).
///
/// - `is_authenticated` — required.
/// - `current_path` — optional; defaults to `/`. Used to highlight
///   the active nav group and to open the matching accordion by default.
/// - `is_connected` — optional; when `true` + `wallet_address` set,
///   renders the wallet card inside the sheet.
/// - `wallet_address` — optional formatted short address (`0x1234…abcd`).
/// - `auth_status` — optional; when `true` the wallet card shows
///   "Authenticated" + green dot, otherwise "Connected".
/// - `wallet_button` — slot for the bottom wallet button (Track C's
///   `WalletProviderIcon` goes here). The component does NOT import
///   the wallet provider dropdown logic — that's Track C's scope.
#[component]
pub fn MobileNav(
    is_authenticated: bool,
    current_path: Option<String>,
    is_connected: Option<bool>,
    wallet_address: Option<String>,
    auth_status: Option<bool>,
    wallet_button: Option<Element>,
    #[props(default)] on_sign_in: Option<EventHandler<MouseEvent>>,
) -> Element {
    let path = current_path.unwrap_or_else(|| "/".to_string());
    let connected = is_connected.unwrap_or(false);
    let authd = auth_status.unwrap_or(false);
    let mut open = use_signal(|| false);
    // Per-group open state. Single signal containing a Vec<bool>
    // because `use_signal` is hook-like and cannot be called in a
    // dynamic loop (Dioxus 0.7 rule).
    let initial_group_open: Vec<bool> = NAV_GROUPS
        .iter()
        .map(|g| is_group_active(g, &path))
        .collect();
    let mut group_open = use_signal(move || initial_group_open);
    let close_sheet = move |_: MouseEvent| {
        open.set(false);
    };

    rsx! {
        // Hamburger trigger — sibling of the Sheet so it remains
        // visible when the sheet is closed.
        button {
            class: "lg:hidden flex items-center justify-center w-9 h-9 rounded-md hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors",
            r#type: "button",
            "aria-label": "Open menu",
            "aria-expanded": "{open}",
            onclick: move |_| open.toggle(),
            Icon { name: "menu", size: Some(20), class_name: Some("text-orange-500".to_string()) }
        }
        Sheet {
            open: open(),
            on_close: move |_| open.set(false),
            side: Some("right".to_string()),
            class_name: Some("w-[85vw] max-w-sm p-0".to_string()),
            div {
                // Header strip
                div { class: "flex items-center gap-2.5 p-4 border-b border-slate-200 dark:border-slate-700",
                    span {
                        class: "epsx-icon",
                        dangerous_inner_html: "{epsx_templates::epsx_icon_svg()}"
                    }
                    span { class: "text-xl font-black tracking-widest text-transparent bg-clip-text bg-gradient-to-r from-[#488BFA] to-[#A43FF3] leading-none mt-0.5", "EPSX" }
                }
                // Wallet card
                if connected {
                    if let Some(addr) = wallet_address.clone() {
                        div { class: "p-4 border-b border-slate-100 dark:border-slate-800",
                            div { class: "flex items-center gap-3",
                                div { class: "flex items-center justify-center w-9 h-9 rounded-full bg-slate-100 dark:bg-slate-800",
                                    Icon { name: "wallet", size: Some(16), class_name: Some("text-orange-500".to_string()) }
                                }
                                div { class: "min-w-0 flex-1",
                                    div { class: "text-sm font-medium text-slate-900 dark:text-white truncate", "{addr}" }
                                    div {
                                        class: if authd { "text-xs text-emerald-500 flex items-center gap-1" } else { "text-xs text-slate-400 flex items-center gap-1" },
                                        if authd {
                                            span { class: "w-1.5 h-1.5 rounded-full bg-emerald-500" }
                                            "Authenticated"
                                        } else {
                                            "Connected"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // Nav groups
                div { class: "flex-1 overflow-y-auto p-4 space-y-1",
                    for (idx, group) in NAV_GROUPS.iter().enumerate() {
                        MobileGroupAccordion {
                            group: (*group).clone(),
                            current_path: path.clone(),
                            group_index: idx,
                            group_open: group_open,
                            on_navigate: close_sheet,
                        }
                    }
                    div { class: "my-3 border-t border-slate-100 dark:border-slate-800" }
                    // Notifications link (only when authed)
                    if is_authenticated {
                        a {
                            class: "flex items-center gap-2.5 px-3 py-2.5 rounded-md text-sm font-medium text-slate-600 hover:text-slate-900 hover:bg-slate-50 dark:text-slate-400 dark:hover:text-white dark:hover:bg-slate-800 transition-colors",
                            href: "/notifications",
                            onclick: move |_| open.set(false),
                            Icon { name: "bell", size: Some(16), class_name: Some("text-orange-500".to_string()) }
                            "Notifications"
                        }
                    }
                }
                // Bottom wallet button
                if let Some(wb) = wallet_button.clone() {
                    div { class: "p-4 border-t border-slate-200 dark:border-slate-700",
                        {wb}
                    }
                }
                // Sign-in banner (only when wallet connected but
                // not authenticated — matches TS SignInBanner)
                if let Some(handler) = on_sign_in.clone() {
                    if connected && !is_authenticated {
                        div { class: "signin-banner",
                            button {
                                r#type: "button",
                                class: "signin-banner-cta",
                                onclick: move |e| handler.call(e),
                                "Sign In with Wallet"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Single accordion group inside the mobile sheet. Lifted out of
/// `MobileNav` so the `use_signal` rules-of-hooks invariant is
/// trivially satisfied (one `use_signal` per component, always at
/// the top level).
#[component]
pub fn MobileGroupAccordion(
    group: NavGroup,
    current_path: String,
    group_index: usize,
    group_open: Signal<Vec<bool>>,
    on_navigate: EventHandler<MouseEvent>,
) -> Element {
    let active = is_group_active(&group, &current_path);
    let is_open = group_open().get(group_index).copied().unwrap_or(false);
    let g_icon = group.icon.clone();
    let g_key = group.key.clone();
    let g_label = group.label.clone();
    let idx = group_index;
    let on_toggle = move |_: MouseEvent| {
        let mut current = group_open();
        if idx < current.len() {
            current[idx] = !current[idx];
            group_open.set(current);
        }
    };
    let on_item_click = move |e: MouseEvent| {
        on_navigate.call(e);
    };
    let p = current_path.clone();
    rsx! {
        div { class: "mobile-nav-group",
            button {
                class: if active { "mobile-nav-group-trigger active" } else { "mobile-nav-group-trigger" },
                r#type: "button",
                "aria-expanded": "{is_open}",
                "aria-controls": "mobile-nav-group-panel-{g_key}",
                onclick: on_toggle,
                span { class: "flex items-center gap-2",
                    if let Some(i) = g_icon {
                        span { class: "text-orange-500", Icon { name: i, size: Some(16) } }
                    }
                    span { "{g_label}" }
                }
                span {
                    class: if is_open { "chev rotate-90 text-orange-500" } else { "chev text-orange-500" },
                    Icon { name: "chevron-right", size: Some(16) }
                }
            }
            if is_open {
                div {
                    id: "mobile-nav-group-panel-{g_key}",
                    class: "ml-3 border-l border-slate-200 dark:border-slate-700 pl-3 space-y-0.5",
                    for item in group.items.iter() {
                        {
                            let item_href = item.href.clone();
                            let item_label = item.label.clone();
                            let item_icon = item.icon.clone();
                            let item_key = item.key.clone();
                            let item_active = is_item_active(item, &p);
                            let cls = if item_active {
                                "flex items-center gap-2.5 px-3 py-2 rounded-md text-sm bg-slate-100 text-slate-900 dark:bg-slate-800 dark:text-white"
                            } else {
                                "flex items-center gap-2.5 px-3 py-2 rounded-md text-sm text-slate-500 hover:text-slate-900 hover:bg-slate-50 dark:text-slate-400 dark:hover:text-white dark:hover:bg-slate-800"
                            };
                            rsx! {
                                a {
                                    key: "{item_key}",
                                    class: "{cls}",
                                    href: "{item_href}",
                                    "aria-current": if item_active { "page" } else { "" },
                                    onclick: on_item_click,
                                    if let Some(i) = item_icon {
                                        span { class: "text-orange-500 shrink-0", Icon { name: i, size: Some(16) } }
                                    }
                                    span { "{item_label}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

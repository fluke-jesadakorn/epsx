//! Layout chrome — navbar cluster.
//!
//! This module historically held the site-wide `Navbar` component (used by
//! every page in `pages/*.rs`). Wave 2 adds the frontend navigation
//! cluster port from `apps-old/frontend/components/nav/*`:
//!
//! - [`NavigationClient`] — top-level entry point; renders the
//!   sticky header, hydration skeleton, desktop nav, nav actions, and
//!   sign-in banner. The BFF passes wallet / auth state as props.
//! - [`DesktopNav`] — logo + group dropdowns (>=lg).
//! - [`GroupDropdown`] — single group's `<Dropdown>` trigger + menu.
//! - [`SignInBanner`] — purple→teal banner that prompts unauthenticated
//!   wallet-connected users to sign in.
//!
//! These new components are **dead code** in Wave 2 — pages still use
//! the existing `Navbar`. Wave 3 will wire the new components into
//! pages.
//!
//! Re-exports for the rest of the cluster live in:
//! - `super::nav_config` — `NavItem`, `NavGroup`, `NAV_GROUPS`, `FOOTER_LINKS`,
//!   `is_group_active`, `is_item_active`.
//! - `super::navbar_skeleton` — `NavbarSkeleton`.
//! - `super::mobile_nav` — `MobileNav`.
//! - `super::nav_actions` — `NavActions`.

use crate::auth::User;
use crate::i18n::t;
use crate::primitives::dropdown::Dropdown;
use crate::primitives::icon::Icon;

use dioxus::prelude::*;

use super::nav_actions::NavActions;
use super::nav_config::{is_group_active, is_item_active, NavGroup, NavItem, NAV_GROUPS};

/// Existing site navbar. Used by every page in `pages/*.rs`. **Wave 2
/// keeps this API stable** — see Wave 1's Public API Stability rule.
///
/// - `user` — optional authenticated user; shows a wallet pill in the
///   actions area when present.
/// - `current_path` — optional path; defaults to `/`. Used to highlight
///   the active nav group.
/// - `on_theme_toggle` — optional event handler for the theme toggle
///   button.
#[component]
pub fn Navbar(
    user: Option<User>,
    current_path: Option<String>,
    on_theme_toggle: Option<EventHandler<MouseEvent>>,
) -> Element {
    let is_authed = user.is_some();
    let path = current_path.unwrap_or_else(|| "/".to_string());
    let groups = main_nav_groups(is_authed);
    let logo_svg = epsx_templates::epsx_icon_svg().to_string();
    let sun_svg = epsx_templates::lucide("sun", "18", "").to_string();
    let moon_svg = epsx_templates::lucide("moon", "18", "").to_string();
    let chev_svg = epsx_templates::lucide("chevron-down", "14", "").to_string();
    rsx! {
        nav { class: "navbar",
            div { class: "navbar-inner",
                a { class: "navbar-brand", href: "/",
                    span { class: "navbar-logo", dangerous_inner_html: "{logo_svg}" }
                    span { class: "navbar-title gradient-text", "EPSX" }
                }
                div { class: "navbar-menu",
                    for group in groups {
                        div { class: "nav-dropdown",
                            button { class: "nav-dropdown-trigger",
                                span { "{group.label}" }
                                span { class: "chev", dangerous_inner_html: "{chev_svg}" }
                            }
                            div { class: "nav-dropdown-menu",
                                for item in &group.items {
                                    a {
                                        class: if path == item.href { "nav-dropdown-item active" } else { "nav-dropdown-item" },
                                        href: "{item.href}",
                                        if let Some(i) = &item.icon {
                                            span { dangerous_inner_html: "{epsx_templates::lucide(i, \"16\", \"\")}" }
                                        }
                                        span { "{item.label}" }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "navbar-actions",
                    button { class: "theme-toggle", onclick: move |e| if let Some(h) = &on_theme_toggle { h.call(e); },
                        span { class: "theme-icon-sun", dangerous_inner_html: "{sun_svg}" }
                        span { class: "theme-icon-moon", dangerous_inner_html: "{moon_svg}" }
                    }
                    if let Some(u) = &user {
                        a { class: "btn btn-primary", href: "/dashboard",
                            span { dangerous_inner_html: "{epsx_templates::lucide(\"wallet\", \"16\", \"\")}" }
                            span { "{u.short_address()}" }
                        }
                    } else {
                        a { class: "btn btn-gradient", href: "/auth",
                            span { "{t(\"nav.connect\")}" }
                        }
                    }
                }
            }
        }
    }
}

/// Build the legacy `Navbar`'s nav groups. Pages use this indirectly
/// via the `<Navbar>` component.
pub fn main_nav_groups(is_authed: bool) -> Vec<NavGroup> {
    let mut groups = vec![
        NavGroup {
            label: "Platform".to_string(),
            key: "platform".to_string(),
            icon: None,
            items: vec![
                NavItem {
                    label: t("nav.home"),
                    href: "/".to_string(),
                    key: "home".to_string(),
                    icon: Some("home".to_string()),
                    desc: None,
                },
                NavItem {
                    label: t("nav.pricing"),
                    href: "/pricing".to_string(),
                    key: "pricing".to_string(),
                    icon: Some("credit-card".to_string()),
                    desc: None,
                },
                NavItem {
                    label: t("nav.plans"),
                    href: "/plans".to_string(),
                    key: "plans".to_string(),
                    icon: Some("layout-dashboard".to_string()),
                    desc: None,
                },
                NavItem {
                    label: t("nav.analytics"),
                    href: "/analytics".to_string(),
                    key: "analytics".to_string(),
                    icon: Some("chart-line".to_string()),
                    desc: None,
                },
                NavItem {
                    label: t("nav.portfolio"),
                    href: "/portfolio".to_string(),
                    key: "portfolio".to_string(),
                    icon: Some("briefcase".to_string()),
                    desc: None,
                },
            ],
        },
        NavGroup {
            label: "Resources".to_string(),
            key: "resources".to_string(),
            icon: None,
            items: vec![
                NavItem {
                    label: t("nav.news"),
                    href: "/news".to_string(),
                    key: "news".to_string(),
                    icon: Some("newspaper".to_string()),
                    desc: None,
                },
                NavItem {
                    label: t("nav.manual"),
                    href: "/manual".to_string(),
                    key: "manual".to_string(),
                    icon: Some("book".to_string()),
                    desc: None,
                },
                NavItem {
                    label: t("nav.about"),
                    href: "/about".to_string(),
                    key: "about".to_string(),
                    icon: Some("info".to_string()),
                    desc: None,
                },
                NavItem {
                    label: t("nav.contact"),
                    href: "/contact".to_string(),
                    key: "contact".to_string(),
                    icon: Some("mail".to_string()),
                    desc: None,
                },
            ],
        },
    ];
    if is_authed {
        groups.push(NavGroup {
            label: "Account".to_string(),
            key: "account".to_string(),
            icon: None,
            items: vec![
                NavItem {
                    label: t("nav.dashboard"),
                    href: "/dashboard".to_string(),
                    key: "dashboard".to_string(),
                    icon: Some("layout-dashboard".to_string()),
                    desc: None,
                },
                NavItem {
                    label: t("nav.profile"),
                    href: "/profile".to_string(),
                    key: "profile".to_string(),
                    icon: Some("user".to_string()),
                    desc: None,
                },
                NavItem {
                    label: t("nav.notifications"),
                    href: "/notifications".to_string(),
                    key: "notifications".to_string(),
                    icon: Some("bell".to_string()),
                    desc: None,
                },
                NavItem {
                    label: t("nav.chat"),
                    href: "/chat".to_string(),
                    key: "chat".to_string(),
                    icon: Some("message-circle".to_string()),
                    desc: None,
                },
                NavItem {
                    label: t("nav.developer"),
                    href: "/developer".to_string(),
                    key: "developer".to_string(),
                    icon: Some("code".to_string()),
                    desc: None,
                },
            ],
        });
    }
    groups
}

// === Wave 2: frontend navigation cluster port ===
// (kept in the same file as the existing `Navbar` because the plan
// designates `navbar.rs` as the home for `NavigationClient` +
// `DesktopNav` + `GroupDropdown`; the `NavbarSkeleton` itself lives in
// the dedicated `navbar_skeleton.rs` file per the design doc.)

/// Group dropdown inside [`DesktopNav`]. Renders a `<Dropdown>` whose
/// trigger shows the group's icon + label + chevron, and whose menu
/// contains one `<DropdownItem>` per child item.
///
/// - `group` — the group to render.
/// - `current_path` — the current pathname; the trigger gets an
///   "active" class when the group contains the active route.
#[component]
pub fn GroupDropdown(group: NavGroup, current_path: String) -> Element {
    let active = is_group_active(&group, &current_path);
    let trigger_cls = if active {
        "flex items-center gap-1 px-3 py-1.5 text-sm font-medium rounded-md transition-colors text-slate-900 dark:text-white"
    } else {
        "flex items-center gap-1 px-3 py-1.5 text-sm font-medium rounded-md transition-colors text-slate-500 hover:text-slate-900 dark:text-slate-400 dark:hover:text-white"
    };
    let mut open = use_signal(|| false);
    let g_icon = group.icon.clone();
    let g_label = group.label.clone();
    rsx! {
        Dropdown {
            open: open(),
            on_toggle: move |_| open.toggle(),
            on_open_change: Some(EventHandler::new(move |v: bool| open.set(v))),
            align: Some("start".to_string()),
            side: None,
            menu_class: Some("w-52 p-1.5 bg-white border border-slate-200 shadow-xl rounded-lg dark:bg-slate-900 dark:border-slate-700".to_string()),
            trigger: rsx! {
                button { class: "{trigger_cls}", r#type: "button",
                    if let Some(i) = g_icon {
                        span { class: "text-orange-500", Icon { name: i, size: Some(16) } }
                    }
                    span { "{g_label}" }
                    span { class: "text-orange-500", Icon { name: "chevron-down", size: Some(12) } }
                }
            },
            for item in group.items.iter() {
                {
                    let item_href = item.href.clone();
                    let item_label = item.label.clone();
                    let item_icon = item.icon.clone();
                    let item_desc = item.desc.clone();
                    let item_key = item.key.clone();
                    let item_active = is_item_active(item, &current_path);
                    let item_cls = if item_active {
                        "flex items-center gap-2.5 px-2.5 py-2 rounded-md text-sm cursor-pointer transition-colors bg-slate-100 text-slate-900 dark:bg-slate-800 dark:text-white"
                    } else {
                        "flex items-center gap-2.5 px-2.5 py-2 rounded-md text-sm cursor-pointer transition-colors text-slate-600 hover:bg-slate-50 hover:text-slate-900 dark:text-slate-300 dark:hover:bg-slate-800 dark:hover:text-white"
                    };
                    let mut on_click_open = open;
                    rsx! {
                        div {
                            key: "{item_key}",
                            class: "{item_cls}",
                            role: "menuitem",
                            "aria-current": if item_active { "page" } else { "" },
                            onclick: move |_| on_click_open.set(false),
                            a { href: "{item_href}",
                                div { class: "flex items-center gap-2.5 min-w-0",
                                    if let Some(i) = item_icon {
                                        span { class: "text-orange-500 shrink-0", Icon { name: i, size: Some(16) } }
                                    }
                                    div { class: "min-w-0",
                                        div { class: "font-medium", "{item_label}" }
                                        if let Some(d) = item_desc {
                                            div { class: "text-xs text-slate-400 dark:text-slate-500", "{d}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Desktop navigation: logo + group dropdowns. Hidden below `lg` via
/// the `hidden lg:flex` classes. Mirrors the TS `DesktopNav` 1:1.
#[component]
pub fn DesktopNav(current_path: String) -> Element {
    rsx! {
        div { class: "hidden lg:flex items-center gap-6",
            // Logo (desktop)
            a { class: "navbar-brand", href: "/",
                span { class: "navbar-logo",
                    dangerous_inner_html: "{epsx_templates::epsx_icon_svg()}"
                }
                span { class: "text-xl font-black tracking-widest text-transparent bg-clip-text bg-gradient-to-r from-[#488BFA] to-[#A43FF3] leading-none mt-0.5", "EPSX" }
            }
            nav { class: "flex items-center gap-0.5",
                for group in NAV_GROUPS.iter() {
                    {
                        let g = (*group).clone();
                        let p = current_path.clone();
                        rsx! {
                            GroupDropdown { group: g, current_path: p }
                        }
                    }
                }
            }
        }
    }
}

/// Purple→teal gradient banner shown above the navbar when the user
/// has a wallet connected but is not authenticated. Mirrors the TS
/// `SignInBanner`.
///
/// - `is_connected` — whether the wallet is connected.
/// - `is_authenticated` — whether the user has a valid session.
/// - `is_loading` — whether auth state is still being fetched.
/// - `on_sign_in` — fired when the CTA button is clicked.
#[component]
pub fn SignInBanner(
    is_connected: bool,
    is_authenticated: bool,
    is_loading: bool,
    on_sign_in: Option<EventHandler<MouseEvent>>,
) -> Element {
    if !is_connected || is_authenticated || is_loading {
        return rsx! { Fragment {} };
    }
    rsx! {
        div { class: "sticky top-0 z-40 flex items-center justify-center gap-3 bg-gradient-to-r from-[#5a33b8] via-[#7645d9] to-[#1a9bab] px-6 py-3 text-base text-white shadow-lg dark:from-[#7645d9]/90 dark:via-[#5a33b8] dark:to-[#1fc7d4]/80",
            role: "region",
            "aria-label": "Sign-in prompt",
            span { class: "font-medium opacity-90", "Your wallet is connected —" }
            if let Some(handler) = on_sign_in {
                button {
                    r#type: "button",
                    class: "rounded-md bg-white/20 px-4 py-1 font-bold transition-colors hover:bg-white/30",
                    onclick: move |e| handler.call(e),
                    "Sign In with Wallet"
                }
            }
            span { class: "opacity-70", "to access all features" }
        }
    }
}

/// Top-level frontend nav cluster. Mirrors the TS `NavigationClient`
/// 1:1 — renders a hydration skeleton when not yet hydrated, renders
/// nothing on `/auth`, otherwise renders the sticky header +
/// `<SignInBanner>`.
///
/// All wallet / auth / wallet-button state is passed in as props
/// (the BFF supplies them; this component does NOT import
/// `wagmi` / React context equivalents).
///
/// - `is_hydrated` — defaults to `true` (Dioxus SSR is hydration-less
///   in the Next.js sense; the BFF gets the final HTML on first
///   render). Set to `false` to render the skeleton (used in tests
///   and any future client-side router).
/// - `current_path` — the BFF's request path. Hides the navbar on
///   `/auth` (matches the TS source).
/// - `is_connected` — whether the wallet is connected.
/// - `is_authenticated` — whether the user has a valid session.
/// - `is_loading` — whether auth state is still being fetched.
/// - `wallet_address` — optional short address (`0x1234…abcd`)
///   shown in the mobile sheet's wallet card.
/// - `auth_status` — passed to `<NavActions>` (flips the wallet
///   card between "Connected" and "Authenticated").
/// - `on_sign_in` — fired by `<SignInBanner>` and the mobile CTA.
/// - `theme_toggle` / `chain_selector` / `notification_bell` —
///   slots consumed by `<NavActions>` (desktop + tablet tiers).
/// - `wallet_button_desktop` / `wallet_button_tablet` —
///   slots consumed by `<NavActions>` for the wallet provider
///   dropdown (Track C supplies these).
#[component]
pub fn NavigationClient(
    is_hydrated: Option<bool>,
    current_path: Option<String>,
    is_connected: Option<bool>,
    is_authenticated: Option<bool>,
    is_loading: Option<bool>,
    wallet_address: Option<String>,
    auth_status: Option<bool>,
    theme_toggle: Option<Element>,
    chain_selector: Option<Element>,
    notification_bell: Option<Element>,
    wallet_button_desktop: Option<Element>,
    wallet_button_tablet: Option<Element>,
    #[props(default)] on_sign_in: Option<EventHandler<MouseEvent>>,
) -> Element {
    let hydrated = is_hydrated.unwrap_or(true);
    let path = current_path.unwrap_or_else(|| "/".to_string());
    let connected = is_connected.unwrap_or(false);
    let authed = is_authenticated.unwrap_or(false);
    let loading = is_loading.unwrap_or(false);
    let fully_authed = connected && authed && !loading;

    // Hydration skeleton (mirrors TS `if (!isHydrated) { ... }`).
    if !hydrated {
        return rsx! {
            header { class: "epsx-header sticky top-0 z-50",
                div { class: "mx-auto flex h-14 max-w-7xl items-center justify-between px-4 md:px-6",
                    a { class: "navbar-brand", href: "/",
                        span { class: "navbar-logo",
                            dangerous_inner_html: "{epsx_templates::epsx_icon_svg()}"
                        }
                        span { class: "text-xl font-black tracking-widest text-transparent bg-clip-text bg-gradient-to-r from-[#488BFA] to-[#A43FF3] leading-none mt-0.5", "EPSX" }
                    }
                    div { class: "flex items-center gap-2",
                        div { class: "hidden md:flex items-center gap-2",
                            div { class: "h-7 w-16 bg-slate-100 dark:bg-slate-800 rounded-md animate-pulse" }
                            div { class: "h-7 w-20 bg-slate-100 dark:bg-slate-800 rounded-md animate-pulse" }
                        }
                        div { class: "h-9 w-9 bg-slate-100 dark:bg-slate-800 rounded-md animate-pulse lg:hidden" }
                    }
                }
            }
        };
    }

    // Hide on auth page.
    if path == "/auth" {
        return rsx! { Fragment {} };
    }

    rsx! {
        header { class: "epsx-header sticky top-0 z-50",
            div { class: "mx-auto flex h-14 max-w-7xl items-center justify-between px-4 md:px-6",
                // Mobile logo (visible <lg; DesktopNav has its own logo >=lg)
                a { class: "lg:hidden navbar-brand",
                    href: "/",
                    span { class: "navbar-logo",
                        dangerous_inner_html: "{epsx_templates::epsx_icon_svg()}"
                    }
                    span { class: "text-xl font-black tracking-widest text-transparent bg-clip-text bg-gradient-to-r from-[#488BFA] to-[#A43FF3] leading-none mt-0.5", "EPSX" }
                }
                DesktopNav { current_path: path.clone() }
                NavActions {
                    is_authenticated: fully_authed,
                    current_path: Some(path.clone()),
                    is_connected: Some(connected),
                    wallet_address: wallet_address.clone(),
                    auth_status: auth_status,
                    theme_toggle: theme_toggle.clone(),
                    chain_selector: chain_selector.clone(),
                    notification_bell: notification_bell.clone(),
                    wallet_button_desktop: wallet_button_desktop.clone(),
                    wallet_button_tablet: wallet_button_tablet.clone(),
                }
            }
        }
        SignInBanner {
            is_connected: connected,
            is_authenticated: authed,
            is_loading: loading,
            on_sign_in: on_sign_in.clone(),
        }
    }
}

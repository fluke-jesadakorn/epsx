//! Layout shells — `DashboardShell` / `DeveloperShell` (existing
//! public API used by `pages/admin_pages/*`) plus the new TS-parity
//! `MainLayout` / `AuthLayout` / `AdminLayout` enum dispatcher.

use crate::auth::{AuthGate, User};
use crate::layout::footer::AdminFooter;
use crate::layout::header::Header;
use crate::layout::sidebar::{AdminSidebar, SidebarItem};

use dioxus::prelude::*;

// ─────────────────────────────────────────────────────────────────────────
// Existing public API — `DashboardShell` and `DeveloperShell`.
// These are the components `pages/admin_pages/*.rs` import via
// `crate::layout::DashboardShell`. Their signatures MUST stay stable.
// ─────────────────────────────────────────────────────────────────────────

#[component]
pub fn DashboardShell(current_path: String, page_title: String, user: Option<User>, children: Element) -> Element {
    let items = vec![
        SidebarItem { id: "dashboard".into(), label: "Dashboard".into(), href: "/".into(), icon: "layout-dashboard".into(), badge: None, active_paths: vec!["/".into()], ..Default::default() },
        SidebarItem { id: "analytics".into(), label: "Analytics".into(), href: "/analytics".into(), icon: "chart-line".into(), badge: None, active_paths: vec!["/analytics".into()], ..Default::default() },
        SidebarItem { id: "audit-log".into(), label: "Audit Log".into(), href: "/audit-log".into(), icon: "history".into(), badge: None, active_paths: vec!["/audit-log".into()], ..Default::default() },
        SidebarItem { id: "wallets".into(), label: "Wallets".into(), href: "/wallet-management/wallets".into(), icon: "wallet".into(), badge: None, active_paths: vec!["/wallet-management".into()], ..Default::default() },
        SidebarItem { id: "credits".into(), label: "Credits".into(), href: "/wallet-management/credits".into(), icon: "credit-card".into(), badge: None, active_paths: vec!["/wallet-management/credits".into()], ..Default::default() },
        SidebarItem { id: "access".into(), label: "Access".into(), href: "/wallet-management/access".into(), icon: "key".into(), badge: None, active_paths: vec!["/wallet-management/access".into()], ..Default::default() },
        SidebarItem { id: "payments".into(), label: "Payments".into(), href: "/payments".into(), icon: "credit-card".into(), badge: None, active_paths: vec!["/payments".into()], ..Default::default() },
        SidebarItem { id: "chat".into(), label: "Chat".into(), href: "/chat".into(), icon: "message-circle".into(), badge: None, active_paths: vec!["/chat".into()], ..Default::default() },
        SidebarItem { id: "notifications".into(), label: "Notifications".into(), href: "/notifications/manage".into(), icon: "bell".into(), badge: None, active_paths: vec!["/notifications".into()], ..Default::default() },
        SidebarItem { id: "news".into(), label: "News".into(), href: "/news".into(), icon: "newspaper".into(), badge: None, active_paths: vec!["/news".into()], ..Default::default() },
        SidebarItem { id: "media".into(), label: "Media".into(), href: "/media".into(), icon: "file-text".into(), badge: None, active_paths: vec!["/media".into()], ..Default::default() },
        SidebarItem { id: "developer".into(), label: "Developer".into(), href: "/developer-portal".into(), icon: "code".into(), badge: None, active_paths: vec!["/developer-portal".into()], ..Default::default() },
        SidebarItem { id: "settings".into(), label: "Settings".into(), href: "/settings".into(), icon: "settings".into(), badge: None, active_paths: vec!["/settings".into()], ..Default::default() },
    ];
    rsx! {
        div { class: "admin-shell",
            AdminSidebar {
                current_path: current_path.clone(),
                is_authenticated: user.as_ref().map(|u| u.is_authed()).unwrap_or(false),
                items: Some(items),
            }
            div { class: "admin-main",
                header { class: "admin-header",
                    div { class: "admin-header-left", h1 { class: "admin-page-title", "{page_title}" } }
                    div { class: "admin-header-right",
                        if let Some(u) = &user {
                            span { class: "admin-user-badge", "{u.short_address()}" }
                        }
                    }
                }
                main { class: "admin-content", {children} }
            }
        }
    }
}

#[component]
pub fn DeveloperShell(current_path: String, children: Element) -> Element {
    let items = vec![
        SidebarItem { id: "overview".into(), label: "Overview".into(), href: "/developer".into(), icon: "home".into(), badge: None, active_paths: vec!["/developer".into()], ..Default::default() },
        SidebarItem { id: "api-keys".into(), label: "API Keys".into(), href: "/developer".into(), icon: "key".into(), badge: None, active_paths: vec!["/developer".into()], ..Default::default() },
        SidebarItem { id: "usage".into(), label: "Usage".into(), href: "/developer/usage".into(), icon: "chart-line".into(), badge: None, active_paths: vec!["/developer/usage".into()], ..Default::default() },
        SidebarItem { id: "docs".into(), label: "Docs".into(), href: "/developer/docs".into(), icon: "book".into(), badge: None, active_paths: vec!["/developer/docs".into()], ..Default::default() },
    ];
    rsx! {
        div { class: "developer-shell",
            AdminSidebar { current_path, is_authenticated: false, items: Some(items) }
            main { class: "developer-main", {children} }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────
// New TS-parity shells — `MainLayout` / `AuthLayout` / `AdminLayout`.
// ─────────────────────────────────────────────────────────────────────────

/// TS-parity [`MainLayout`](apps-old/.../main-layout.tsx) — Sidebar +
/// Header + scrollable main + thin admin footer.
///
/// Renders the App Shell fixed-layout structure:
/// - Outer: `flex h-screen w-full overflow-hidden bg-background`
/// - Sidebar: full height, hidden on mobile
/// - Right side: header (sticky), main (scrollable), footer (glass)
#[component]
pub fn MainLayout(
    current_path: String,
    is_authenticated: bool,
    user: Option<User>,
    initial_notifications: Option<Vec<crate::layout::header::HeaderNotification>>,
    initial_unread_count: Option<u32>,
    is_production: Option<bool>,
    /// Optional custom sidebar items. `None` uses the TS-parity
    /// `default_nav_items()`.
    sidebar_items: Option<Vec<SidebarItem>>,
    /// Slot for the breadcrumb inside the header. `None` falls back to
    /// the default auto-generated breadcrumb.
    breadcrumb: Option<Element>,
    /// Slot for the notification bell.
    notification_bell: Option<Element>,
    /// Slot for the theme toggle.
    theme_toggle: Option<Element>,
    /// Slot for the chain selector (dev only).
    chain_selector: Option<Element>,
    children: Element,
) -> Element {
    rsx! {
        div { class: "flex h-screen w-full overflow-hidden bg-background",
            // Sidebar — hidden on mobile, full height.
            div { class: "hidden md:block",
                AdminSidebar {
                    current_path: current_path.clone(),
                    is_authenticated,
                    items: sidebar_items,
                }
            }

            // Right side — content area.
            div { class: "flex flex-1 flex-col h-full overflow-hidden",
                Header {
                    user: user.clone(),
                    initial_notifications: initial_notifications.clone(),
                    initial_unread_count: initial_unread_count,
                    current_path: current_path.clone(),
                    is_production: is_production,
                    breadcrumb: breadcrumb.clone(),
                    notification_bell: notification_bell.clone(),
                    theme_toggle: theme_toggle.clone(),
                    chain_selector: chain_selector.clone(),
                }

                // Content frame: scrollable main + thin footer.
                div { class: "flex-1 flex flex-col min-h-0 overflow-hidden",
                    main { class: "flex-1 overflow-y-auto overflow-x-hidden p-0",
                        {children}
                    }
                    AdminFooter {}
                }
            }
        }
    }
}

/// TS-parity [`AuthLayout`](apps-old/.../auth-layout.tsx) — wraps
/// `MainLayout` and overlays `AuthGate` when needed.
///
/// Mirrors the TS source's behaviour:
/// - `no_layout_paths` (e.g. `/login`, `/unauthorized`, ...) bypass the
///   chrome entirely and render `{children}` directly.
/// - When `is_gated` is true (no server user AND no client auth), an
///   `AuthGate` overlay is rendered on top of the main layout — the
///   page tree stays mounted underneath.
#[component]
pub fn AuthLayout(
    current_path: String,
    server_user: Option<ServerUser>,
    is_authenticated: bool,
    is_gated: Option<bool>,
    no_layout_paths: Option<Vec<String>>,
    initial_notifications: Option<Vec<crate::layout::header::HeaderNotification>>,
    initial_unread_count: Option<u32>,
    is_production: Option<bool>,
    sidebar_items: Option<Vec<SidebarItem>>,
    breadcrumb: Option<Element>,
    notification_bell: Option<Element>,
    theme_toggle: Option<Element>,
    chain_selector: Option<Element>,
    children: Element,
) -> Element {
    // Resolve no-layout path overrides.
    let no_layout_paths = no_layout_paths.unwrap_or_else(default_no_layout_paths);
    let is_no_layout = no_layout_paths.iter().any(|p| current_path == *p || current_path.starts_with(p));
    if is_no_layout {
        return rsx! { Fragment { {children} } };
    }

    // Compute effective gated state. Caller can override via `is_gated`
    // for testing; default mirrors the TS source's
    // `serverUser == null && !isAuthenticated` check.
    let effective_gated = is_gated.unwrap_or_else(|| server_user.is_none() && !is_authenticated);

    // Map the (optional) server-side user to a domain `User` for the
    // header. When the user is authenticated, the header gets the authed
    // user; otherwise `None` (the header's wallet button will render its
    // "Connect" CTA).
    let header_user: Option<User> = if is_authenticated {
        server_user.as_ref().and_then(|s| s.to_user())
    } else {
        None
    };

    rsx! {
        Fragment {
            MainLayout {
                current_path: current_path.clone(),
                is_authenticated,
                user: header_user,
                initial_notifications: initial_notifications.clone(),
                initial_unread_count: initial_unread_count,
                is_production: is_production,
                sidebar_items: sidebar_items.clone(),
                breadcrumb: breadcrumb.clone(),
                notification_bell: notification_bell.clone(),
                theme_toggle: theme_toggle.clone(),
                chain_selector: chain_selector.clone(),
                {children}
            }
            if effective_gated {
                AuthGate { user: None, feature: Some("the admin dashboard".to_string()) }
            }
        }
    }
}

/// The default set of paths that bypass the admin chrome entirely
/// (matches `NO_LAYOUT_PATHS` in `auth-layout.tsx`).
pub fn default_no_layout_paths() -> Vec<String> {
    vec![
        "/login".to_string(),
        "/unauthorized".to_string(),
        "/access-denied".to_string(),
        "/permissions/policies".to_string(),
    ]
}

/// Server-side user shape — the layout doesn't need the full domain
/// `User`, just the bits that the admin chrome displays (address, role).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ServerUser {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub role: String,
}

impl ServerUser {
    /// Convert into a domain `User` for the header. Uses `id` as the
    /// address (admin chrome only ever shows the short form).
    pub fn to_user(&self) -> Option<User> {
        if self.id.is_empty() { return None; }
        Some(User {
            id: self.id.clone(),
            address: self.id.clone(),
            chain_id: "56".to_string(),
            roles: vec![self.role.clone()],
            email: if self.email.is_empty() { None } else { Some(self.email.clone()) },
            ..Default::default()
        })
    }
}

/// TS-parity dispatcher enum. Renders the correct shell for the chosen
/// variant. Pages that want to opt into the new chrome can switch from
/// `DashboardShell` to `AdminLayout::Dashboard.render(...)` without
/// touching the rest of their tree.
///
/// The enum is a plain data type; the `render` method returns an
/// `Element` you can drop into a `rsx!` block. (We don't use `match` in
/// the body because `Element` is the return type of `#[component]`
/// functions, not a closure.)
#[derive(Clone, Debug, PartialEq)]
pub enum AdminLayout {
    /// Renders a side-bar + header + main layout with the default
    /// admin chrome. `is_authenticated` toggles disabled-row styling,
    /// the user pill, and the connect-wallet CTA.
    Main {
        current_path: String,
        is_authenticated: bool,
        user: Option<User>,
    },
    /// Same as `Main` but with the `AuthGate` overlay shown when no
    /// user is present. Use this for the global admin layout
    /// (matches `AuthLayout` from the TS source).
    Auth {
        current_path: String,
        server_user: Option<ServerUser>,
        is_authenticated: bool,
        is_gated: Option<bool>,
        no_layout_paths: Option<Vec<String>>,
    },
    /// No layout — render `{children}` directly. Useful for the
    /// `no_layout_paths` cases.
    None,
}

impl AdminLayout {
    /// Render the selected layout around the given children. Returns
    /// an `Element` you can splice into `rsx!`.
    pub fn render(
        self,
        children: Element,
        initial_notifications: Option<Vec<crate::layout::header::HeaderNotification>>,
        initial_unread_count: Option<u32>,
        is_production: Option<bool>,
    ) -> Element {
        match self {
            AdminLayout::Main { current_path, is_authenticated, user } => rsx! {
                MainLayout {
                    current_path,
                    is_authenticated,
                    user,
                    initial_notifications,
                    initial_unread_count,
                    is_production,
                    {children}
                }
            },
            AdminLayout::Auth { current_path, server_user, is_authenticated, is_gated, no_layout_paths } => rsx! {
                AuthLayout {
                    current_path,
                    server_user,
                    is_authenticated,
                    is_gated,
                    no_layout_paths,
                    initial_notifications,
                    initial_unread_count,
                    is_production,
                    {children}
                }
            },
            AdminLayout::None => rsx! { Fragment { {children} } },
        }
    }
}

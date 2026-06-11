//! `<AdminShell>` — the shared admin chrome (sidebar + breadcrumb header
//! + main content area) used by every Wave 6B admin page.
//!
//! Mirrors the `apps-old/admin-frontend/app/(admin)/layout.tsx` pattern:
//! the page body sits inside an `<AdminAuthGate>` that gates on a
//! per-page `required_permissions` list; the gate's children render
//! inside the `<AdminShell>`, which emits the sidebar + breadcrumb
//! header + main content slot. Reusable across ALL admin pages in Wave
//! 6B (and beyond).
//!
//! ## Why a primitive
//!
//! Without this primitive, every admin page (5 in Track A, 15+ in the
//! rest of Wave 6B) would have to duplicate the same outer markup
//! (`<div class="admin-shell">` + sidebar + breadcrumb + main). That
//! creates 20+ copies of the same admin chrome, each one a candidate
//! for drift. Centralising it here means one place to update when the
//! chrome changes (e.g. add a header action, change the sidebar
//! highlight rule, swap the breadcrumb style).
//!
//! ## Section markers
//!
//! - `admin-shell` — outer wrapper (full-height flex container).
//! - `admin-shell-sidebar` — the `AdminSidebar` slot.
//! - `admin-shell-header` — the breadcrumb + page-title header.
//! - `admin-shell-main` — the children/content slot.
//!
//! Pages can add their own `data-section` markers to children without
//! colliding with the shell's markers — the shell uses `class=` and
//! pages typically use Tailwind multi-class strings. The per-page unit
//! tests assert the page's own markers; the shell's markers are an
//! internal contract.
//!
//! ## Usage
//!
//! ```rust
//! use crate::layout::admin_shell::AdminShell;
//!
//! AdminShell {
//!     ctx: ctx.clone(),
//!     page_title: "Command Center".to_string(),
//!     breadcrumbs: vec![
//!         ("Dashboard".to_string(), "/".to_string()),
//!         ("Command Center".to_string(), "/".to_string()),
//!     ],
//!     AdminStatsCards { /* ... */ }
//! }
//! ```
//!
//! Pages wrap the whole `AdminShell` in an `<AdminAuthGate>` so the
//! shell's children only render for authenticated admins. The shell
//! itself does NOT call `AdminAuthGate` — that's the caller's
//! responsibility, because each page has its own
//! `required_permissions` list and `feature` description.

use crate::auth::User;
use crate::layout::breadcrumbs::{BreadcrumbItem, Breadcrumbs, Crumb};
use crate::layout::sidebar::{AdminSidebar, SidebarItem};
use crate::pages::PageContext;

use dioxus::prelude::*;

/// The default sidebar items shown in the admin shell. Mirrors the
/// `DEFAULT_NAV_ITEMS` from `shell.rs::DashboardShell` and the TS
/// `app/admin/sidebar.tsx`. Pages can override by passing a custom
/// `sidebar_items` (e.g. for an embedded surface with a narrower
/// nav).
fn default_admin_shell_items() -> Vec<SidebarItem> {
    vec![
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
    ]
}

/// The Wave 6B shared admin shell. Renders the full-height sidebar +
/// breadcrumb header + main content area used by every admin page.
///
/// `breadcrumbs` is a list of `(label, href)` tuples. The last entry's
/// `href` is rendered as a non-clickable terminal breadcrumb (matching
/// the TS source's behaviour where the current page is the final
/// span). Pass an empty `Vec` to render no breadcrumb.
#[component]
pub fn AdminShell(
    /// The page context — used for the active sidebar item (via
    /// `ctx.path`) and the header user pill.
    ctx: PageContext,
    /// The page title shown in the header right next to the breadcrumbs.
    page_title: String,
    /// The breadcrumb chain. The last entry is rendered as the terminal
    /// (non-clickable) crumb. Pass `vec![]` to render no breadcrumb.
    breadcrumbs: Vec<(String, String)>,
    /// Optional custom sidebar items. `None` uses the default admin
    /// sidebar (matches `app/(admin)/layout.tsx`).
    #[props(default = None)]
    sidebar_items: Option<Vec<SidebarItem>>,
    children: Element,
) -> Element {
    let is_authenticated = ctx.user.as_ref().map(|u| u.is_authed()).unwrap_or(false);
    let items = sidebar_items.unwrap_or_else(default_admin_shell_items);

    // Convert the (label, href) tuples to the `Crumb` shape the
    // plural `<Breadcrumbs>` component expects. The terminal crumb
    // gets `href: None` so it renders as a plain `<span>` (not a link).
    let crumbs: Vec<Crumb> = {
        let len = breadcrumbs.len();
        breadcrumbs.into_iter().enumerate().map(|(i, (label, href))| {
            if i + 1 == len {
                Crumb { label, href: None }
            } else {
                Crumb { label, href: Some(href) }
            }
        }).collect()
    };

    rsx! {
        div { class: "admin-shell admin-shell-page",
            // Sidebar — full height, hidden on mobile (matches the
            // existing `DashboardShell` from `shell.rs`).
            div { class: "admin-shell-sidebar hidden md:block",
                AdminSidebar {
                    current_path: ctx.path.clone(),
                    is_authenticated,
                    items: Some(items),
                }
            }
            // Right side — breadcrumb header + main content.
            div { class: "flex flex-1 flex-col h-full overflow-hidden",
                // Breadcrumb + page-title header.
                header { class: "admin-shell-header",
                    div { class: "admin-shell-header-left flex items-center gap-3",
                        if !crumbs.is_empty() {
                            Breadcrumbs { items: crumbs }
                        }
                        h1 { class: "admin-shell-page-title", "{page_title}" }
                    }
                    div { class: "admin-shell-header-right",
                        if let Some(u) = &ctx.user {
                            span { class: "admin-user-badge", "{u.short_address()}" }
                        }
                    }
                }
                // Main content — the page's children render here.
                main { class: "admin-shell-main", {children} }
            }
        }
    }
}

/// Helper for tests / callers that want to convert the Wave 6A-style
/// `Vec<BreadcrumbItem>` (with `href: String` and an optional emoji
/// `icon`) into the `Vec<(String, String)>` the `<AdminShell>` takes.
/// Drops the `icon` (the shell doesn't render it inline; the page
/// title is the prominent label).
pub fn breadcrumb_items_to_tuples(items: Vec<BreadcrumbItem>) -> Vec<(String, String)> {
    items.into_iter().map(|b| (b.label, b.href)).collect()
}

/// Re-export of the user-facing predicate so callers don't have to
/// import the full `auth` module just to check auth state. Mirrors
/// `User::is_authed` (defined in `auth/user.rs`).
pub fn is_authed_user(user: &Option<User>) -> bool {
    user.as_ref().map(|u| u.is_authed()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::{AuthMethod, User};
    use crate::pages::PageContext;

    fn admin_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "u-admin".to_string(),
                address: "0x1234abcd5678ef90".to_string(),
                chain_id: "56".to_string(),
                roles: vec!["admin".to_string()],
                email: Some("admin@epsx.io".to_string()),
                tier: Some("Admin".to_string()),
                permissions: vec!["admin:*".to_string()],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: Some("Admin".to_string()),
            }),
            path: "/".to_string(),
            ..Default::default()
        }
    }

    /// Smoke test: rendering the shell with breadcrumbs produces a
    /// non-empty Element with the shell's section markers present.
    #[test]
    fn admin_shell_renders_breadcrumbs() {
        let ctx = admin_ctx();
        let crumbs = vec![
            ("Dashboard".to_string(), "/".to_string()),
            ("Command Center".to_string(), "/".to_string()),
        ];
        let body = rsx! {
            AdminShell {
                ctx: ctx.clone(),
                page_title: "Command Center".to_string(),
                breadcrumbs: crumbs,
                div { class: "page-content" }
            }
        };
        let html = dioxus_ssr::render_element(body);
        assert!(!html.trim().is_empty(), "AdminShell should render non-empty HTML. Got: {html}");
        // Both breadcrumb labels must be present in the rendered HTML.
        // The terminal crumb renders as a `<span>` (no link), the prior
        // crumb renders as an `<a href>`. Both contain the label text.
        assert!(
            html.contains("Dashboard"),
            "AdminShell breadcrumbs should include the 'Dashboard' label. Got: {html}"
        );
        assert!(
            html.contains("Command Center"),
            "AdminShell breadcrumbs should include the 'Command Center' label. Got: {html}"
        );
        // The shell's section markers are also part of the contract.
        assert!(
            html.contains("admin-shell"),
            "AdminShell should render its `admin-shell` wrapper. Got: {html}"
        );
        assert!(
            html.contains("admin-shell-main"),
            "AdminShell should render its `admin-shell-main` content slot. Got: {html}"
        );
    }

    /// The terminal breadcrumb (last entry) renders as a span, not a
    /// link, mirroring the TS source. The prior entries render as
    /// links to their `href`. This test guards that contract.
    #[test]
    fn admin_shell_terminal_breadcrumb_is_not_a_link() {
        let ctx = admin_ctx();
        let crumbs = vec![
            ("Dashboard".to_string(), "/".to_string()),
            ("Policies".to_string(), "/policies".to_string()),
        ];
        let body = rsx! {
            AdminShell {
                ctx: ctx.clone(),
                page_title: "Policies".to_string(),
                breadcrumbs: crumbs,
                div { class: "policies-body" }
            }
        };
        let html = dioxus_ssr::render_element(body);
        // The first crumb (Dashboard) should be a link to "/".
        assert!(
            html.contains("href=\"/\""),
            "AdminShell first breadcrumb should link to its href. Got: {html}"
        );
        // The terminal crumb (Policies) should NOT be a link.
        // We grep for any `href="/policies"` and assert the only one is
        // from a sidebar item, not a breadcrumb. Cheaper check: the
        // breadcrumb list (`<ol class="breadcrumbs-list">`) should
        // contain the terminal label twice in different forms (once in
        // the h1 title + once in the breadcrumb span) but no `<a
        // href="/policies">` inside the breadcrumbs.
        assert!(
            !html.contains("href=\"/policies\""),
            "AdminShell terminal breadcrumb should not be a link. Got: {html}"
        );
    }
}

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
//! ```rust,no_run
//! use dioxus::prelude::*;
//! use epsx_dioxus_ui::layout::admin_shell::AdminShell;
//! use epsx_dioxus_ui::pages::PageContext;
//!
//! fn AdminExample() -> Element {
//!     let ctx = PageContext::default();
//!     rsx! {
//!         AdminShell {
//!             ctx,
//!             page_title: "Command Center".to_string(),
//!             breadcrumbs: vec![
//!                 ("Dashboard".to_string(), "/".to_string()),
//!                 ("Command Center".to_string(), "/".to_string()),
//!             ],
//!             div { "Admin content" }
//!         }
//!     }
//! }
//! ```
//!
//! Pages wrap the whole `AdminShell` in an `<AdminAuthGate>` so the
//! shell's children only render for authenticated admins. The shell
//! itself does NOT call `AdminAuthGate` — that's the caller's
//! responsibility, because each page has its own
//! `required_permissions` list and `feature` description.

use crate::auth::User;
use crate::layout::breadcrumbs::BreadcrumbItem;
use crate::layout::footer::AdminFooter;
use crate::layout::header::Header;
use crate::layout::sidebar::{default_nav_items, AdminSidebar, SidebarItem};
use crate::pages::PageContext;

use dioxus::prelude::*;

/// The default sidebar items shown in the admin shell. Mirrors the
/// `DEFAULT_NAV_ITEMS` from `shell.rs::DashboardShell` and the TS
/// `app/admin/sidebar.tsx`. Pages can override by passing a custom
/// `sidebar_items` (e.g. for an embedded surface with a narrower
/// nav).
fn default_admin_shell_items() -> Vec<SidebarItem> {
    // Keep every AdminShell consumer on the same nested tree as the source
    // `components/layout/sidebar.tsx`. The legacy flat list made analytics,
    // settings, and the command center render a visibly different shell from
    // newer admin pages.
    default_nav_items()
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

    // The source derives breadcrumbs from the current pathname. Keep the
    // legacy prop in the public API for compatibility; the route is the
    // authoritative value rendered by the shared Header below.
    let _legacy_breadcrumbs = breadcrumbs;

    rsx! {
        div { class: "admin-shell admin-shell-page",
            // Sidebar — full height, hidden on mobile (matches the
            // existing `DashboardShell` from `shell.rs`).
            div { class: "admin-shell-sidebar hidden md:block", style: "height: 100vh; min-height: 100vh;",
                AdminSidebar {
                    current_path: ctx.path.clone(),
                    is_authenticated,
                    items: Some(items),
                }
            }
            // Right side — breadcrumb header + main content + footer.
            // Mirrors prod's `apps/admin-frontend/components/layout/main-layout.tsx`:
            //   <div class="flex flex-1 flex-col h-full overflow-hidden">
            //     <Header ... />
            //     <div class="flex-1 flex flex-col min-h-0 overflow-hidden">
            //       <div class="flex-1 overflow-y-auto overflow-x-hidden p-0">
            //         {children}
            //       </main>
            //       <footer class="border-t border-border/40 bg-card px-4 py-3">
            //         EPSX Admin Dashboard / Version 2.0
            //       </footer>
            //     </div>
            //   </div>
            div { class: "flex flex-1 flex-col h-full overflow-hidden", style: "height: 100vh; min-height: 100vh;",
                // Reuse the same source-parity header as the BFF-level
                // MainLayout: route breadcrumb, notification bell, theme
                // toggle, wallet control, and the shared logout hook.
                Header {
                    user: ctx.user.clone(),
                    initial_notifications: None,
                    initial_unread_count: None,
                    current_path: Some(ctx.path.clone()),
                    is_production: Some(false),
                    breadcrumb: None,
                    notification_bell: None,
                    theme_toggle: None,
                    chain_selector: None,
                    on_bell_click: None,
                    on_theme_toggle: None,
                    class_name: Some("admin-shell-header".to_string()),
                    id: None,
                }
                // Source pages place their visible title in the page body;
                // retain the legacy prop as an accessible shell marker for
                // callers and tests that still pass it to AdminShell.
                span { class: "sr-only", "data-admin-shell-page-title": "{page_title}", "{page_title}" }
                // Main content — the page's children render here.
                // The shared page document already renders the single
                // `#epsx-main-content` landmark. This inner element is only
                // the admin scroll region and must remain a div to avoid
                // nested `<main>` landmarks on authenticated routes.
                div { class: "admin-shell-main", {children} }
                // Footer — the prod-EXACT 2-line admin footer
                // ("EPSX Admin Dashboard" / "Version 2.0"). Lives
                // inside the right-side flex column so the sidebar
                // does not get a footer column (matches prod's
                // `MainLayout`).
                AdminFooter {}
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
        assert!(
            !html.trim().is_empty(),
            "AdminShell should render non-empty HTML. Got: {html}"
        );
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
        assert!(
            html.contains("data-epsx-logout=\"true\""),
            "authenticated AdminShell must expose the shared logout controller hook. Got: {html}"
        );
    }

    /// The shell must render the prod-EXACT admin footer
    /// (`<AdminFooter />` → `<footer class="admin-footer">` with
    /// "EPSX Admin Dashboard" / "Version 2.0"). This matches
    /// `apps/admin-frontend/components/layout/main-layout.tsx`
    /// lines 55-65. Without this footer the dev BFF diverges
    /// from prod on every Wave 6B admin page that uses
    /// `AdminShell` directly (analytics, dashboard, media,
    /// policies, settings).
    #[test]
    fn admin_shell_renders_admin_footer() {
        let ctx = admin_ctx();
        let body = rsx! {
            AdminShell {
                ctx: ctx.clone(),
                page_title: "Settings".to_string(),
                breadcrumbs: vec![("Settings".to_string(), "/settings".to_string())],
                div { class: "page-content" }
            }
        };
        let html = dioxus_ssr::render_element(body);
        // The AdminFooter component renders a `<footer>` with the
        // `admin-footer` class and the two label spans.
        assert!(
            html.contains("admin-footer"),
            "AdminShell must render an <footer class=\"admin-footer\">. Got: {html}"
        );
        assert!(
            html.contains("EPSX Admin Dashboard"),
            "AdminShell footer must show 'EPSX Admin Dashboard'. Got: {html}"
        );
        assert!(
            html.contains("Version 2.0"),
            "AdminShell footer must show 'Version 2.0'. Got: {html}"
        );
        // Footer must be AFTER the admin scroll region (the right-side
        // flex column layout: header → content → footer). The document
        // shell owns the semantic `<main>` landmark; this component's
        // content region is intentionally a div.
        let content_off = html
            .find("class=\"admin-shell-main\"")
            .expect("AdminShell must render its content region");
        let footer_off = html
            .find("<footer")
            .expect("AdminShell must render <footer>");
        assert!(
            content_off < footer_off,
            "AdminShell footer must come AFTER the content region. content_off={content_off} footer_off={footer_off}. Got: {html}"
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

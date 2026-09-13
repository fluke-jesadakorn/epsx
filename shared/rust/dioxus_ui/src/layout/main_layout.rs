//! Frontend layout wrappers — `MainLayout` and `AuthLayout`.
//!
//! Wave 3a Track A: the per-page `<Navbar>` / `<Footer>` call sites are
//! moved up into a layout-level wrapper. Pages now return body content
//! only; the chrome (sticky header + footer) is rendered once per app
//! by the layout.
//!
//! Conventions (see `docs/wave3a-wiring/design.md`):
//!
//! - **Layout ownership (§1)**: pages MUST NOT call `<Navbar>` or
//!   `<Footer>` directly after this wave. They wrap their body in
//!   `MainLayout` (or `AuthLayout` for `/auth`).
//! - **Public API stability (§2)**: this module adds two new
//!   components; existing `Navbar` / `Footer` / `NavigationClient`
//!   props are unchanged.
//! - **BFF render path invariants (§5)**: the BFF continues to
//!   construct `PageContext`, call `render_page`, get back a
//!   `PageMeta` + `Element`. The layout swap happens INSIDE the page
//!   render functions, not in the BFF.
//!
//! Wave 3a Track B (completed): `PageContext.wallet: ConnectedWalletState`
//! is plumbed (was added in wave-3a). This layout now forwards it
//! into `NavigationClient` (see the body below for the priority
//! order — SIWE session wins over the wallet cookie for
//! `is_authenticated`). The previous `TODO: pass ctx.wallet here in
//! Track B` comment is resolved.

use dioxus::prelude::*;

use crate::pages::PageContext;
use crate::theme::ThemeRoot;

/// Standard frontend layout — renders the page body wrapped in the
/// shared theme bootstrap.
///
/// **Wave 49+ — SSR-safe navbar/footer moved to the page shell.**
///
/// This layout previously rendered `<NavigationClient />` (the
/// Dioxus sticky header) and `<Footer />` inline around the body.
/// Both were broken under Dioxus 0.7 SSR:
///
/// - `<NavigationClient />` uses Dioxus `onclick:` closures to
///   toggle the Market / Developer / Company dropdowns. SSR is
///   hydration-less, so the closures were stripped from the HTML
///   and clicking the navbar items did nothing.
///
/// - `<Footer />` was removed in Wave 38c to fix a structural
///   double-footer with the templates `footer()`, but was never
///   re-added. Most pages rendered no footer at all, so the
///   Terms / Privacy / About / Contact / Rankings / Portfolio /
///   Pricing / API Keys / Documentation / Support / News links
///   were unreachable from the footer on every page except
///   `/terms` (which has its own page-local `TermsFooter`).
///
/// The fix lives at the BFF layer: `apps/frontend/src/ssr.rs`
/// now passes `epsx_templates::epsx_header()` (which emits raw
/// `onclick="epsx.toggleNav(this)"` attributes that survive SSR)
/// as the `nav` arg to `page_shell_with_body_class`, and forces
/// `include_footer = true` so the templates 4-column footer
/// renders after `</main>`. The navbar / footer are now in the
/// page-shell `<body>`, OUTSIDE the Dioxus subtree, so they work
/// without hydration.
///
/// This layout's only remaining responsibility is:
/// 1. Wrap the body in `<ThemeRoot>` so the page picks up the
///    persisted dark/light preference on first paint (sets the
///    `--bg` / `--text` CSS vars + pre-paint theme bootstrap).
/// 2. Provide a stable call signature (`<MainLayout ctx={ctx}>`)
///    so existing pages keep compiling.
///
/// On `path == "/auth"` the page shell passes an empty nav and
/// the dedicated `<AuthLayout>` is full-bleed.
///
/// - `ctx` — the BFF-supplied page context. Currently unused
///   inside the body (theme + footer + navbar are now handled at
///   the page-shell level); kept on the signature for symmetry
///   with `<AuthLayout>` and to make future per-route BFF plumbing
///   (e.g. injecting the `return_url` query param into a
///   sub-component) trivial.
/// - `children` — the page body. Rendered verbatim inside
///   `<ThemeRoot>`.
///
/// Wave 23 T4: the layout wraps the body in `ThemeRoot` (CSS vars
/// + pre-paint theme bootstrap). Before this, the theme toggle
/// in the navbar was rendered but no handler was wired, so the
/// click was a no-op and the `.dark` class was never applied.
#[allow(non_snake_case)] // PascalCase is intentional — see design doc §1.
#[component]
pub fn MainLayout(ctx: PageContext, children: Element) -> Element {
    // Suppress the unused-variable warning for `ctx` without renaming
    // the prop (renaming would break the `<MainLayout ctx={ctx}>` call
    // sites in the pages). We deliberately accept the prop because
    // future plumbing may forward it.
    let _ = ctx;
    rsx! {
        ThemeRoot {
            { children }
        }
    }
}

/// Full-bleed layout for the `/auth` route — no chrome, no footer.
///
/// Mirrors the TS source which hides the entire navbar on `/auth`
/// (the auth page is its own design). Use this wrapper in
/// `pages/auth_page.rs` so the page never enters `MainLayout`.
///
/// - `ctx` — the BFF-supplied page context. Currently unused inside
///   the body (the auth route never shows the chrome regardless of
///   `ctx.path`); kept on the signature for symmetry with
///   `MainLayout` and to make future per-route BFF plumbing (e.g.
///   passing the `return_url` query param to a sub-component) trivial.
/// - `children` — the auth content.
///
/// Wave 23 T4: wraps the body in `ThemeRoot` so the auth page
/// picks up the persisted dark/light preference on first paint.
#[allow(non_snake_case)] // PascalCase is intentional — see design doc §1.
#[component]
pub fn AuthLayout(ctx: PageContext, children: Element) -> Element {
    // Suppress the unused-variable warning for `ctx` without renaming
    // the prop (renaming would break the `AuthLayout { ctx, ... }`
    // call sites in the pages). We deliberately accept the prop
    // because future plumbing may forward it.
    let _ = ctx;
    // === wave3a-wiring-track-a ===
    // Full-bleed: no header, no footer. Just the page body.
    rsx! {
        ThemeRoot {
            Fragment { { children } }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::PageContext;

    /// Build a minimal `PageContext` for tests.
    fn ctx_for(path: &str) -> PageContext {
        PageContext {
            path: path.to_string(),
            ..Default::default()
        }
    }

    /// Render a Dioxus `Element` to an HTML string for assertion.
    /// `dioxus_ssr` is a dev-dependency of `epsx-dioxus-ui`; its
    /// `render_element` function formats an `Element` to HTML.
    fn render_to_string(el: Element) -> String {
        // `dioxus_ssr::render_element(el) -> String` serializes a
        // Dioxus `Element` (Result<VNode, _>) to an HTML string.
        dioxus_ssr::render_element(el)
    }

    #[test]
    fn main_layout_preserves_body_and_emits_no_chrome() {
        let ctx = ctx_for("/");
        let html = render_to_string(rsx! {
            MainLayout { ctx,
                div { class: "page-body-marker", "hello body" }
            }
        });
        // Wave 49+ — `<NavigationClient />` was removed from
        // `MainLayout` (its Dioxus onclick handlers were stripped
        // by SSR, leaving the dropdowns un-clickable). The navbar
        // and footer are now rendered at the BFF page-shell level
        // (`apps/frontend/src/ssr.rs` passes `epsx_templates::
        // epsx_header()` as the nav + forces `include_footer = true`).
        // This layout wraps the body in `<ThemeRoot>` and emits no
        // `<header>` / `<footer>` of its own.
        assert!(
            !html.contains("<header"),
            "MainLayout must NOT render <header> (navbar lives at the page-shell level now). Got: {}",
            html
        );
        assert!(
            !html.contains("<footer"),
            "MainLayout must NOT render <footer> (footer lives at the page-shell level now). Got: {}",
            html
        );
        // Body content must still be present.
        assert!(
            html.contains("page-body-marker"),
            "MainLayout must preserve body content. Got: {}",
            html
        );
        assert!(
            html.contains("hello body"),
            "MainLayout must preserve body text. Got: {}",
            html
        );
    }

    #[test]
    fn auth_layout_is_full_bleed() {
        let ctx = ctx_for("/auth");
        let html = render_to_string(rsx! {
            AuthLayout { ctx,
                div { class: "auth-body-marker", "sign in content" }
            }
        });
        // AuthLayout has always been full-bleed (no header, no
        // footer); the BFF also passes an empty nav for path
        // "/auth" so the page-shell level doesn't add a navbar
        // either.
        assert!(
            !html.contains("<header"),
            "AuthLayout must NOT render <header>. Got: {}",
            html
        );
        assert!(
            !html.contains("<footer"),
            "AuthLayout must NOT render <footer>. Got: {}",
            html
        );
        // Auth body content must still be present.
        assert!(
            html.contains("auth-body-marker"),
            "AuthLayout must preserve body content. Got: {}",
            html
        );
    }

    #[test]
    fn main_layout_preserves_body_content() {
        let ctx = ctx_for("/dashboard");
        let html = render_to_string(rsx! {
            MainLayout { ctx,
                div { class: "dashboard-marker", "dashboard body" }
            }
        });
        // The body's marker class + text must be in the output
        // unchanged — the wrapper must not eat or rewrite the body.
        assert!(
            html.contains("dashboard-marker"),
            "MainLayout must preserve body content. Got: {}",
            html
        );
        assert!(
            html.contains("dashboard body"),
            "MainLayout must preserve body text. Got: {}",
            html
        );
    }

    /// `PageMeta::include_footer` is the default policy exposed to
    /// consuming BFFs. All current variants opt out so admin pages do
    /// not duplicate their in-layout `<AdminFooter />`. The frontend
    /// BFF deliberately overrides the value and emits one SSR-safe
    /// templates footer outside the Dioxus subtree. This test guards
    /// the metadata default, not that consumer-specific override.
    #[test]
    fn all_page_meta_variants_default_to_no_legacy_footer() {
        use crate::pages::PageMeta;
        // Marketing — was the source of the double-footer bug.
        let m = PageMeta::marketing("Home");
        assert!(
            !m.include_footer,
            "PageMeta::marketing must not include footer"
        );
        // App — was the source of the spurious single-footer.
        let a = PageMeta::app("Dashboard");
        assert!(!a.include_footer, "PageMeta::app must not include footer");
        // Admin — admin chrome is rendered by `shell::MainLayout` /
        // `AdminShell`, NOT by the templates `footer()`.
        let d = PageMeta::admin("Command Center");
        assert!(!d.include_footer, "PageMeta::admin must not include footer");
        let d_bc = PageMeta::admin_with_body_class("Access Denied", "h-screen");
        assert!(
            !d_bc.include_footer,
            "PageMeta::admin_with_body_class must not include footer"
        );
    }
}

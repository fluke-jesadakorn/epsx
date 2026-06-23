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

use super::footer::Footer;
use super::navbar::NavigationClient;
use crate::pages::PageContext;
use crate::theme::ThemeRoot;
use crate::theme::UnifiedThemeToggle;

/// Standard frontend layout — renders the sticky navigation chrome +
/// the page body + the site footer.
///
/// On `path == "/auth"` the chrome short-circuits (no header); the
/// footer is still rendered to keep the page visually anchored. (The
/// `AuthLayout` wrapper is the recommended way to do an
/// actually-full-bleed auth page; this fallback keeps historical
/// `path == "/auth"` behavior safe even if a page forgets the
/// dedicated wrapper.)
///
/// - `ctx` — the BFF-supplied page context. `ctx.path` decides
///   chrome visibility; `ctx.user` is forwarded to the chrome for
///   the wallet pill in the actions slot. (Track B will additionally
///   forward `ctx.wallet` for the new `ConnectButton` /
///   `ConnectedWalletDropdown` slot — see the TODO inside the body.)
/// - `children` — the page body. Rendered verbatim between the
///   chrome and the footer.
///
/// Wave 23 T4: the layout now wraps the body in `ThemeRoot` (CSS
/// vars + pre-paint theme bootstrap) and passes a
/// `UnifiedThemeToggle` to the navbar so the click target actually
/// flips light/dark mode. Before this, the navbar rendered the
/// theme button but `MainLayout` never supplied a handler, so the
/// click was a no-op and the `.dark` class was never applied.
#[allow(non_snake_case)] // PascalCase is intentional — see design doc §1.
#[component]
pub fn MainLayout(ctx: PageContext, children: Element) -> Element {
    let path = ctx.path.clone();
    let user = ctx.user.clone();
    // Wave 3a Track B (TODO cleanup): wire the BFF-supplied
    // `ConnectedWalletState` from `ctx.wallet` into the
    // NavigationClient chrome. Without this the wallet pill always
    // renders the "disconnected" placeholder even when the user
    // has connected via SIWE, because the layout was passing
    // hardcoded `is_connected: Some(false)` regardless of the
    // actual cookie state.
    //
    // Priority order: if the SIWE session has a user, the user is
    // considered authenticated even if the wallet cookie expired
    // (the session lifetime is independent). Wallet connection
    // status is sourced from the wallet cookie only.
    let wallet = ctx.wallet.clone();
    // `ConnectedWalletState` derives connection from the presence
    // of `address` — there's no separate `is_connected` boolean.
    let is_connected = Some(wallet.address.is_some());
    let is_authenticated = Some(user.is_some() || wallet.is_authenticated);
    let wallet_address = wallet.address.clone();
    rsx! {
        ThemeRoot {
            // === wave3a-wiring-track-a ===
            // Chrome cluster. NavigationClient already short-circuits on
            // `path == "/auth"` (renders Fragment {}) so we always render
            // the component and let it decide chrome visibility.
            NavigationClient {
                is_hydrated: Some(true),
                current_path: Some(path),
                is_connected,
                is_authenticated,
                is_loading: Some(false),
                wallet_address,
                theme_toggle: Some(rsx! { UnifiedThemeToggle {} }),
            }
            { children }
            Footer {}
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
    fn main_layout_renders_header_and_footer() {
        let ctx = ctx_for("/");
        let html = render_to_string(rsx! {
            MainLayout { ctx,
                div { class: "page-body-marker", "hello body" }
            }
        });
        // <header> from NavigationClient
        assert!(
            html.contains("<header"),
            "MainLayout(/) should render a <header> element. Got: {}",
            html
        );
        // <footer> from the Footer component
        assert!(
            html.contains("<footer"),
            "MainLayout(/) should render a <footer> element. Got: {}",
            html
        );
    }

    #[test]
    fn main_layout_hides_chrome_on_auth() {
        let ctx = ctx_for("/auth");
        let html = render_to_string(rsx! {
            MainLayout { ctx,
                div { class: "auth-body-marker", "sign in content" }
            }
        });
        // NavigationClient short-circuits on path == "/auth" and
        // returns Fragment {}. The Footer component's <footer> may
        // still appear, but the chrome <header> must NOT.
        assert!(
            !html.contains("<header"),
            "MainLayout(/auth) must NOT render <header>. Got: {}",
            html
        );
        // Auth body content must still be present.
        assert!(
            html.contains("auth-body-marker"),
            "MainLayout(/auth) must preserve body content. Got: {}",
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
}

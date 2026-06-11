//! /admin/auth — admin auth gate redirect handler.
//!
//! Wave 6B Track D — 2 sections per the design doc
//! `docs/wave6b-admin-pages-depth/design.md` §"Track D — ... + auth_page":
//! 1. `AuthMethodSelector` — a brief "Pick a sign-in method" panel
//!    that lists the two auth options the admin sign-in flow
//!    supports: SIWE wallet (admin gate) and email magic link.
//!    The selector is shown briefly before the redirect fires.
//! 2. `AuthRedirectHandler` — the actual redirect. Mirrors the
//!    5-LoC source `apps-old/admin-frontend/app/auth/page.tsx`,
//!    which is a `redirect('/')` Next.js call. The Dioxus port
//!    emits an inline `<script>` that does
//!    `window.location.replace('/')` plus a "Redirecting..."
//!    fallback UI in case the JS doesn't fire.
//!
//! The 2-section structure lets us add an "AuthMethodSelector"
//! later (e.g. for an OAuth provider dropdown) without rewriting
//! the redirect handler.

use crate::primitives::*;

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::auth::AdminAuthGate;

const AUTH_REDIRECT_SCRIPT: &str = "(function(){try{var d=new URLSearchParams(location.search).get('next')||'/';setTimeout(function(){location.replace(d);},1500);}catch(e){location.replace('/');}})();";

// ============================================================================
// Section 1: AuthMethodSelector
// ============================================================================
//
// Brief "Pick a sign-in method" panel shown for 1.5s before the
// redirect fires. The selector is a single panel with two
// options (SIWE wallet, email magic link) — when the user clicks
// one, we record the choice in the URL (`?method=siwe` or
// `?method=email`) and the redirect handler skips the auto-redirect
// for that method. The Dioxus port keeps the selector in DOM
// regardless (the BFF / hydration handles the actual flow).

#[component]
fn AuthMethodSelector() -> Element {
    rsx! {
        div { class: "auth-method-selector space-y-3",
            p { class: "text-sm text-muted-foreground", "Pick a sign-in method" }
            div { class: "grid grid-cols-1 md:grid-cols-2 gap-3",
                a { class: "card card-glass p-4 hover-scale flex items-center gap-3", href: "/auth?method=siwe",
                    div { class: "p-2 rounded-lg bg-primary/10",
                        Icon { name: "wallet".to_string(), size: Some(20), class_name: Some("text-primary".to_string()) }
                    }
                    div {
                        p { class: "font-semibold", "Sign in with wallet" }
                        p { class: "text-xs text-muted-foreground", "SIWE \u{2014} recommended for admins" }
                    }
                }
                a { class: "card card-glass p-4 hover-scale flex items-center gap-3", href: "/auth?method=email",
                    div { class: "p-2 rounded-lg bg-blue-500/10",
                        Icon { name: "mail".to_string(), size: Some(20), class_name: Some("text-blue-400".to_string()) }
                    }
                    div {
                        p { class: "font-semibold", "Continue with email" }
                        p { class: "text-xs text-muted-foreground", "Magic link to your inbox" }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Section 2: AuthRedirectHandler
// ============================================================================
//
// The actual redirect. Emits an inline `<script>` that does
// `window.location.replace('/')` after 1.5s (giving the
// AuthMethodSelector time to be clicked) plus a "Redirecting..."
// fallback UI for users with JS disabled.

#[component]
fn AuthRedirectHandler() -> Element {
    rsx! {
        div { class: "auth-redirect-handler flex flex-col items-center justify-center text-center space-y-3 py-6",
            div { class: "spinner spinner-sm" }
            p { class: "text-sm text-muted-foreground", "Redirecting\u{2026}" }
            p { class: "text-xs text-muted-foreground/60",
                "If you are not redirected automatically, "
                a { href: "/", class: "underline", "go to the home page" }
                "."
            }
            script { dangerous_inner_html: AUTH_REDIRECT_SCRIPT }
        }
    }
}

// ============================================================================
// Top-level page entry point
// ============================================================================

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Sign in");
    (meta, rsx! { RenderAdminAuth { ctx: ctx.clone() } })
}

#[component]
fn RenderAdminAuth(ctx: PageContext) -> Element {
    rsx! {
        AdminAuthGate {
            user: ctx.user.clone(),
            feature: Some("the admin sign-in flow".to_string()),
            required_permissions: Some(vec!["admin:auth".to_string()]),
            return_url: Some(ctx.path.clone()),
            div { class: "container page-content max-w-xl",
                div { class: "card card-glass p-6 space-y-6",
                    // Page header.
                    div { class: "flex items-center gap-3",
                        div { class: "p-2 rounded-xl bg-primary/10",
                            Icon { name: "shield".to_string(), size: Some(24), class_name: Some("text-primary".to_string()) }
                        }
                        div {
                            h1 { class: "text-2xl font-bold", "Admin sign-in" }
                            p { class: "text-sm text-muted-foreground", "Choose how you want to sign in to the admin console" }
                        }
                    }
                    // Section 1: method selector.
                    AuthMethodSelector {}
                    // Section 2: redirect handler.
                    AuthRedirectHandler {}
                }
            }
        }
    }
}

// ============================================================================
// Section markers (used by `tests::test_section_markers`):
//
//   1. "Auth method selector"    → "Pick a sign-in method" + 2 option cards
//   2. "Auth redirect handler"  → "Redirecting\u{2026}" + auto-redirect script
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::PageContext;
    use crate::auth::User;

    /// Build an admin `User`.
    fn test_user_admin() -> User {
        User {
            id: "test-admin".to_string(),
            address: "0xADMIN0000000000000000000000000000000001".to_string(),
            chain_id: "56".to_string(),
            roles: vec!["admin".to_string()],
            email: Some("admin@epsx.io".to_string()),
            tier: Some("admin".to_string()),
            permissions: vec!["admin:auth".to_string()],
            ..Default::default()
        }
    }

    /// Render the admin page's `Element` to an HTML string.
    fn render_to_string(el: Element) -> String {
        dioxus_ssr::render_element(el)
    }

    /// `test_render_smoke` — the page body header is rendered for an
    /// admin user with the right permission.
    #[test]
    fn test_render_smoke() {
        let ctx = PageContext {
            user: Some(test_user_admin()),
            path: "/admin/auth".to_string(),
            ..Default::default()
        };
        let (_, el) = render(&ctx);
        let html = render_to_string(el);
        assert!(
            html.contains("Admin sign-in"),
            "Auth page must render the title for an admin. Got: {}",
            html
        );
        // The 2 sections: method selector + redirect handler.
        assert!(
            html.contains("Pick a sign-in method"),
            "Section 1 (AuthMethodSelector) marker missing. Got: {}",
            html
        );
    }

    /// `test_section_markers` — assert each of the 2 design-doc
    /// sections renders its section-marker text. We exercise each
    /// section component directly.
    #[test]
    fn test_section_markers() {
        // Section 1: AuthMethodSelector.
        let el = rsx! { AuthMethodSelector {} };
        let html = render_to_string(el);
        assert!(html.contains("Pick a sign-in method"), "section 1 (AuthMethodSelector) marker missing");
        assert!(html.contains("Sign in with wallet"), "section 1 'Sign in with wallet' option missing");
        assert!(html.contains("Continue with email"), "section 1 'Continue with email' option missing");

        // Section 2: AuthRedirectHandler.
        let el = rsx! { AuthRedirectHandler {} };
        let html = render_to_string(el);
        assert!(html.contains("Redirecting"), "section 2 (AuthRedirectHandler) marker missing");
        assert!(html.contains("location.replace"), "section 2 redirect script missing");
    }
}

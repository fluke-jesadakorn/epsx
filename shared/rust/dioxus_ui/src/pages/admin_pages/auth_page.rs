//! /admin/auth — admin auth gate redirect handler.
//!
//! Wave 6C Track D — 2 sections per the design doc
//! `docs/wave6c-live-render-and-1to1/design.md` §"Track D":
//!   1. `AuthMethodSelector`    — pick sign-in method
//!   2. `AuthRedirectHandler`   — auto-redirect after 1.5s
//!
//! The 2 sub-components live in `components::admin::auth`. This
//! page just composes them inside the `AdminAuthGate` wrapper.

use crate::primitives::*;
use crate::components::admin::auth::{AuthMethodSelector, AuthRedirectHandler};

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::auth::AdminAuthGate;

/// Top-level page entry point.
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
                    // Section 1.
                    AuthMethodSelector {}
                    // Section 2.
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
//   2. "Auth redirect handler"  → "Redirecting…" + auto-redirect script
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
        // Section 1: AuthMethodSelector (from the new components module).
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

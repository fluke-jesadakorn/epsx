//! Sub-components for `/admin/auth` — Wave 6C Track D.
//!
//! 2 sections per the design doc:
//!   1. `AuthMethodSelector`    — pick sign-in method (SIWE / email)
//!   2. `AuthRedirectHandler`   — auto-redirect after 1.5s

use crate::primitives::*;

use dioxus::prelude::*;

pub const AUTH_REDIRECT_SCRIPT: &str = "(function(){try{var d=new URLSearchParams(location.search).get('next')||'/';setTimeout(function(){location.replace(d);},1500);}catch(e){location.replace('/');}})();";

// ============================================================================
// Section 1: AuthMethodSelector
// ============================================================================

#[component]
pub fn AuthMethodSelector() -> Element {
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

#[component]
pub fn AuthRedirectHandler() -> Element {
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
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn render_to_string(el: Element) -> String {
        dioxus_ssr::render_element(el)
    }

    #[test]
    fn test_render_smoke_auth_method_selector() {
        let el = rsx! { AuthMethodSelector {} };
        let html = render_to_string(el);
        assert!(html.contains("auth-method-selector"), "AuthMethodSelector must render its class. Got: {}", html);
        assert!(html.contains("Pick a sign-in method"), "AuthMethodSelector must render the prompt. Got: {}", html);
        assert!(html.contains("Sign in with wallet"), "AuthMethodSelector must render the wallet option. Got: {}", html);
        assert!(html.contains("Continue with email"), "AuthMethodSelector must render the email option. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_auth_redirect_handler() {
        let el = rsx! { AuthRedirectHandler {} };
        let html = render_to_string(el);
        assert!(html.contains("auth-redirect-handler"), "AuthRedirectHandler must render its class. Got: {}", html);
        assert!(html.contains("Redirecting"), "AuthRedirectHandler must render the redirecting text. Got: {}", html);
    }
}

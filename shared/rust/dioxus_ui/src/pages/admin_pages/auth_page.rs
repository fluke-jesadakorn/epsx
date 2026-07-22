//! `/auth` — the admin app's fixed redirect back to `/`.
//!
//! The development source is a server-side `redirect("/")`. The shared
//! Dioxus dispatcher cannot issue that framework redirect itself, so direct
//! callers receive the smallest equivalent document: an immediate constant
//! client redirect and a constant same-origin fallback link. Request state is
//! deliberately ignored.

use super::super::{PageContext, PageMeta};
use dioxus::prelude::*;

const AUTH_REDIRECT_SCRIPT: &str = "window.location.replace('/');";

pub fn render(_ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Redirecting");
    (meta, rsx! { AdminAuthRedirect {} })
}

#[component]
fn AdminAuthRedirect() -> Element {
    rsx! {
        section {
            "data-admin-auth-state": "redirect",
            class: "auth-redirect-handler",
            aria_label: "Redirecting to the admin home page",
            p { "Redirecting…" }
            p {
                "If you are not redirected automatically, "
                a { href: "/", "continue to the admin home page" }
                "."
            }
            script { dangerous_inner_html: AUTH_REDIRECT_SCRIPT }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::User;
    use std::collections::HashMap;

    fn render_to_string(ctx: &PageContext) -> String {
        let (_, element) = render(ctx);
        dioxus_ssr::render_element(element)
    }

    fn signed_in_user_without_legacy_permission() -> User {
        User {
            id: "admin".to_string(),
            address: "0xadmin".to_string(),
            chain_id: "56".to_string(),
            roles: vec!["admin".to_string()],
            permissions: Vec::new(),
            ..Default::default()
        }
    }

    #[test]
    fn signed_out_and_signed_in_contexts_render_the_same_fixed_redirect() {
        let mut params = HashMap::new();
        params.insert(
            "next".to_string(),
            "https://attacker.example/collect".to_string(),
        );
        let signed_out = PageContext {
            path: "/auth".to_string(),
            query: "next=https%3A%2F%2Fattacker.example%2Fcollect&method=siwe".to_string(),
            params: params.clone(),
            ..Default::default()
        };
        let signed_in = PageContext {
            user: Some(signed_in_user_without_legacy_permission()),
            path: "/auth/ignored".to_string(),
            query: "clear=1&next=//attacker.example".to_string(),
            params,
            ..Default::default()
        };

        let signed_out_html = render_to_string(&signed_out);
        let signed_in_html = render_to_string(&signed_in);

        assert_eq!(signed_out_html, signed_in_html);
        assert!(signed_out_html.contains("data-admin-auth-state=\"redirect\""));
        assert!(signed_out_html.contains("window.location.replace('/');"));
        assert!(signed_out_html.contains("href=\"/\""));
    }

    #[test]
    fn request_values_cannot_enter_the_redirect_document() {
        let ctx = PageContext {
            path: "https://attacker.example/path-token".to_string(),
            query: "next=javascript%3Aalert%281%29&method=email&query-token=reflected".to_string(),
            params: HashMap::from([
                (
                    "next".to_string(),
                    "//attacker.example/param-token".to_string(),
                ),
                ("method".to_string(), "siwe".to_string()),
            ]),
            ..Default::default()
        };

        let html = render_to_string(&ctx);

        for reflected in [
            "attacker.example",
            "javascript",
            "alert",
            "query-token",
            "param-token",
        ] {
            assert!(
                !html.contains(reflected),
                "reflected request value: {reflected}"
            );
        }
        assert!(!html.contains("URLSearchParams"));
        assert!(!html.contains("location.search"));
        assert!(!html.contains("setTimeout"));
    }

    #[test]
    fn legacy_auth_gate_and_invented_method_selector_are_absent() {
        let html = render_to_string(&PageContext::default());

        for invented in [
            "admin:auth",
            "Pick a sign-in method",
            "Sign in with wallet",
            "Continue with email",
            "Magic link",
            "method=siwe",
            "method=email",
        ] {
            assert!(
                !html.contains(invented),
                "invented auth UI remains: {invented}"
            );
        }
    }

    #[test]
    fn leaf_does_not_render_an_application_shell_or_main_landmark() {
        let html = render_to_string(&PageContext::default());

        assert!(!html.contains("<main"));
        assert!(!html.contains("admin-shell"));
        assert!(!html.contains("data-admin-shell"));
    }
}

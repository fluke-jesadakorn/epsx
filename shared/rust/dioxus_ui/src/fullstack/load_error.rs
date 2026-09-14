//! Shared recovery UI for load failures. Authentication remains owned by the BFF.
use super::LoadError;
use dioxus::prelude::*;
use dioxus_router::RouterContext;

fn sign_in_href(path: &str) -> String {
    let path = if path.starts_with('/')
        && !path.starts_with("//")
        && !path.contains(['\\', '\r', '\n'])
        && path.split(['?', '#']).next() != Some("/auth")
    {
        path
    } else {
        "/"
    };
    format!(
        "/auth?{}",
        url::form_urlencoded::Serializer::new(String::new())
            .append_pair("return_url", path)
            .finish()
    )
}

#[component]
pub fn SessionNotice(
    #[props(default)] return_path: Option<String>,
    #[props(default)] expired: bool,
    #[props(default)] actions: Option<Element>,
) -> Element {
    let path = return_path.unwrap_or_else(|| {
        try_use_context::<RouterContext>()
            .map(|router| router.full_route_string())
            .unwrap_or_else(|| "/".into())
    });
    let href = sign_in_href(&path);
    rsx! {
        if try_consume_context::<crate::app::FrontendTailwindStyles>().is_none() {
            style { dangerous_inner_html: include_str!("load_error.css") }
        }
        section { class: "epsx-session-card", role: "status", aria_live: "polite", aria_label: if expired { "Session expired" } else { "Sign in required" }, "data-session-state": "sign-in-required",
            div { class: "epsx-session-symbol", aria_hidden: "true",
                svg { width: "28", height: "28", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.6", stroke_linecap: "round", stroke_linejoin: "round",
                    rect { x: "3", y: "6", width: "18", height: "15", rx: "3" }
                    path { d: "M3 9V6a2 2 0 0 1 2-2h12M21 12h-5v5h5" }
                    path { d: "M17 14.5h.01" }
                }
            }
            div { class: "epsx-session-copy",
                span { class: "epsx-session-eyebrow", "YOUR EPSX ACCOUNT" }
                h2 { if expired { "Let’s reconnect." } else { "Your workspace is one sign-in away." } }
                p { if expired { "Your session has ended. Connect your wallet to continue where you left off." } else { "Connect your wallet to access this page. You’ll return here after signing in." } }
                div { class: "epsx-session-actions",
                    if let Some(actions) = actions { {actions} }
                    else { crate::navigation::AppLink { class: "epsx-session-primary", href, "Connect wallet", span { aria_hidden: "true", "→" } } }
                    crate::navigation::AppLink { class: "epsx-session-secondary", href: "/", "Back to home" }
                }
                p { class: "epsx-session-footnote", "Your wallet stays in your control." }
            }
        }
    }
}

#[component]
pub fn LoadErrorNotice(
    error: LoadError,
    #[props(default)] return_path: Option<String>,
    #[props(default)] children: Option<Element>,
) -> Element {
    if error == LoadError::Unauthenticated {
        return rsx! { SessionNotice { return_path } };
    }
    rsx! { div { class: "epsx-load-error", p { role: "status", "{error.message()}" } {children} } }
}

/// Compact recovery action for errors beside a form or a table control.
#[component]
pub fn SessionMessage(message: String) -> Element {
    if message != LoadError::Unauthenticated.message() {
        return rsx! { "{message}" };
    }
    let path = try_use_context::<RouterContext>()
        .map(|router| router.full_route_string())
        .unwrap_or_else(|| "/".into());
    let href = sign_in_href(&path);
    rsx! {
        if try_consume_context::<crate::app::FrontendTailwindStyles>().is_none() {
            style { dangerous_inner_html: include_str!("load_error.css") }
        }
        span { class: "epsx-session-inline", role: "status",
            crate::navigation::AppLink { href, "Connect wallet to continue", span { aria_hidden: "true", " ↗" } }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontend_recovery_and_theme_use_external_styles() {
        #[allow(non_snake_case)]
        fn Frontend() -> Element {
            use_context_provider(|| crate::app::FrontendTailwindStyles);
            rsx! { crate::theme::ThemeRoot { SessionNotice {} } }
        }
        let html = dioxus_ssr::render_element(rsx! { Frontend {} });
        assert!(html.contains("epsx-session-card"));
        assert!(!html.contains("<style"));
        let shared = dioxus_ssr::render_element(rsx! { SessionNotice {} });
        assert!(shared.contains("<style"));
    }

    #[test]
    fn inline_session_failure_has_an_action_without_replacing_other_feedback() {
        let html = dioxus_ssr::render_element(
            rsx! { SessionMessage { message: LoadError::Unauthenticated.message() } },
        );
        assert!(html.contains("href=\"/auth?return_url=%2F\""));
        assert!(html.contains("Connect wallet to continue"));
        let feedback = dioxus_ssr::render_element(
            rsx! { SessionMessage { message: "Attachment could not be uploaded." } },
        );
        assert!(feedback.contains("Attachment could not be uploaded."));
        assert!(!feedback.contains("/auth"));
    }

    #[test]
    fn sign_in_preserves_local_route_and_filters() {
        let href = sign_in_href("/developer/usage?period=30d&key=abc#requests");
        let query = url::form_urlencoded::parse(href.split_once('?').unwrap().1.as_bytes())
            .collect::<Vec<_>>();
        assert_eq!(query[0].1, "/developer/usage?period=30d&key=abc#requests");
        for unsafe_path in [
            "https://evil.example",
            "//evil.example",
            "/\\evil.example",
            "/auth?return_url=/auth",
            "/\ninvalid",
        ] {
            assert_eq!(sign_in_href(unsafe_path), "/auth?return_url=%2F");
        }
    }

    #[test]
    fn authentication_has_one_action_and_never_retries_a_read() {
        let html = dioxus_ssr::render_element(rsx! { LoadErrorNotice {
            error: LoadError::Unauthenticated, return_path: "/account/payments?offset=10",
            button { "Try again" }
        } });
        assert!(html.contains("data-session-state=\"sign-in-required\""));
        assert!(html.contains("Connect wallet"));
        assert!(html.contains("return_url=%2Faccount%2Fpayments%3Foffset%3D10"));
        assert!(!html.contains("Try again"));
        assert!(!html.contains("Please sign in again"));
        assert!(!html.contains("<!--node-id"));
        let unavailable = dioxus_ssr::render_element(rsx! { LoadErrorNotice {
            error: LoadError::Unavailable, button { "Try again" }
        } });
        assert!(unavailable.contains("Try again"));
        assert!(!unavailable.contains("Connect wallet"));
        let forbidden =
            dioxus_ssr::render_element(rsx! { LoadErrorNotice { error: LoadError::Forbidden } });
        assert!(!forbidden.contains("Connect wallet"));
    }
}

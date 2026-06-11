//! `/error` — generic runtime-error page.
//!
//! Source of truth: `apps-old/frontend/app/error.tsx`. The source
//! branches on three error classes:
//!
//! 1. `isBackendConnectivityError` (status 404/502/503/504, network
//!    errors, `/api/...` 404s) → "Backend Unavailable" panel.
//! 2. `isPermissionError` (403, "forbidden", "permission denied") →
//!    "Access Denied" panel with a "Sign In" button.
//! 3. Default → "Something went wrong" panel with the error
//!    message + a "Try Again" button.
//!
//! The Dioxus port branches on the same heuristics by reading the
//! error message from the page's `?error=…` query string. The BFF
//! can populate that query string when serving `/error` from an
//! upstream failure; the design doc calls for the message to come
//! from `PageContext.error`, but `PageContext` doesn't currently
//! carry an `error` field, so the port uses the query string as
//! the transport (a Wave 6 enhancement could plumb the field
//! directly through `PageContext`).
//!
//! All three branches share the same chrome — a centered
//! illustration, a one-liner, and 1–2 action buttons.

use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Error");
    let error_message = ctx.query_param("error").unwrap_or_default();
    let (kind, headline, sub, primary_href, primary_label, secondary_href, secondary_label) = classify(&error_message);
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            div { class: "container page-content",
                div { class: "error-page",
                    ErrorIllustration { kind: kind }
                    h1 { class: "error-page-title", "{headline}" }
                    p { class: "error-page-subtitle text-muted-foreground", "{sub}" }
                    if !error_message.is_empty() && kind == ErrorKind::Generic {
                        div { class: "error-page-message", "{error_message}" }
                    }
                    div { class: "error-page-actions",
                        a { class: "btn btn-primary btn-lg", href: "{primary_href}", "{primary_label}" }
                        if let (Some(h), Some(l)) = (secondary_href, secondary_label) {
                            a { class: "btn btn-outline btn-lg", href: "{h}", "{l}" }
                        }
                    }
                    if kind == ErrorKind::Backend {
                        ErrorBackendHints {}
                    }
                }
            }
        }
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ErrorKind { Generic, Backend, Permission }

/// Classify an error message into one of the three branches the
/// source uses. Mirrors the source's `isBackendConnectivityError`
/// and `isPermissionError` heuristics.
fn classify(msg: &str) -> (ErrorKind, String, String, String, String, Option<String>, Option<String>) {
    let lower = msg.to_lowercase();
    let is_backend = lower.contains("failed to fetch")
        || lower.contains("network error")
        || lower.contains("connection refused")
        || lower.contains("502")
        || lower.contains("503")
        || lower.contains("504")
        || (lower.contains("/api/") && (lower.contains("404") || lower.contains("not found")));
    let is_permission = lower.contains("403")
        || lower.contains("forbidden")
        || lower.contains("permission denied");
    if is_backend {
        (
            ErrorKind::Backend,
            "Backend Unavailable".to_string(),
            "Service Temporarily Down".to_string(),
            "Reload Page".to_string(),
            "javascript:location.reload()".to_string(),
            Some("Go to Homepage".to_string()),
            Some("/".to_string()),
        )
    } else if is_permission {
        (
            ErrorKind::Permission,
            "Access Denied".to_string(),
            "Permission Required".to_string(),
            "Sign In".to_string(),
            "/auth".to_string(),
            Some("Back to Homepage".to_string()),
            Some("/".to_string()),
        )
    } else {
        (
            ErrorKind::Generic,
            "Oops!".to_string(),
            "Something went wrong".to_string(),
            "Try Again".to_string(),
            "javascript:location.reload()".to_string(),
            Some("Go to Homepage".to_string()),
            Some("/".to_string()),
        )
    }
}

#[component]
fn ErrorIllustration(kind: ErrorKind) -> Element {
    let (icon_name, icon_class) = match kind {
        ErrorKind::Generic => ("info", "text-muted-foreground"),
        ErrorKind::Backend => ("info", "text-orange-500"),
        ErrorKind::Permission => ("shield", "text-red-500"),
    };
    rsx! {
        div { class: "error-page-illustration", "aria-hidden": "true",
            div { class: "error-page-icon",
                Icon { name: icon_name.to_string(), size: Some(48), class_name: Some(icon_class.to_string()) }
            }
        }
    }
}

/// Bullet list of "what could be wrong" hints, only shown on the
/// backend-error branch. Mirrors the source's `<ul>` of
/// maintenance / network / server problems.
#[component]
fn ErrorBackendHints() -> Element {
    rsx! {
        div { class: "error-page-hints",
            p { class: "error-page-hints-label", "We can't connect to our servers right now. This could be due to:" }
            ul {
                li { "Backend service maintenance" }
                li { "Network connectivity issues" }
                li { "Temporary server problems" }
            }
        }
    }
}

// === wave5-page-depth-track-b ===
// Unit test for the error page. Smoke test plus a classification
// check — the 3-branch logic is the page's defining feature.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::PageContext;

    fn empty_ctx() -> PageContext {
        PageContext {
            path: "/error".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn error_page_renders_smoke() {
        let ctx = empty_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(!html.trim().is_empty(), "error page should render non-empty HTML");
    }

    #[test]
    fn classify_backend_error() {
        let (kind, _, _, _, _, _, _) = classify("Failed to fetch: /api/v1/foo");
        assert_eq!(kind, ErrorKind::Backend);
    }

    #[test]
    fn classify_permission_error() {
        let (kind, _, _, _, _, _, _) = classify("403 Forbidden: missing permission");
        assert_eq!(kind, ErrorKind::Permission);
    }

    #[test]
    fn classify_generic_error() {
        let (kind, _, _, _, _, _, _) = classify("something else went wrong");
        assert_eq!(kind, ErrorKind::Generic);
    }
}

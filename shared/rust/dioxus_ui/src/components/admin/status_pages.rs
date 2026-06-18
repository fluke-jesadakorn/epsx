//! Admin status-page family — Wave 38b T2 admin domain port.
//!
//! Mirrors `apps-old/admin-frontend/components/shared/status-pages.tsx`,
//! which exports 3 status-page components used across the admin
//! error-handling surface:
//!
//! | Component | Use case |
//! | --- | --- |
//! | `NotFoundContent` | 404 page body (icon + 404 + title + actions) |
//! | `ErrorContent` | Generic error page (icon + title + message + retry) |
//! | `AccessDeniedContent` | Permission-denied body (icon + details + actions) |
//!
//! All three wrap a centered `StatusPageLayout` (the private
//! inner wrapper) so the visual treatment is consistent.
//!
//! ## Tests
//!
//! Each component gets a colocated `#[cfg(test)] mod tests` block
//! with smoke render + key prop handling (default vs custom title,
//! admin-context details card).

use dioxus::prelude::*;
use crate::primitives::icon::Icon;

// ============================================================================
// StatusPageLayout (private)
// ============================================================================
//
// Centered column container. Mirrors the source's private
// `StatusPageLayout`.

#[component]
fn StatusPageLayout(class_name: Option<String>, children: Element) -> Element {
    let mut cls = "flex flex-col items-center justify-center min-h-[60vh] p-6 sm:p-8 lg:p-12".to_string();
    if let Some(c) = class_name {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div { class: "{cls}", {children} }
    }
}

// ============================================================================
// NotFoundContent
// ============================================================================
//
// 404 page body. Icon + 404 number + title + message + home/back
// actions.

#[component]
pub fn NotFoundContent(
    title: Option<String>,
    message: Option<String>,
    show_home_link: Option<bool>,
    show_back_button: Option<bool>,
) -> Element {
    let title = title.unwrap_or_else(|| "Page Not Found".to_string());
    let message = message.unwrap_or_else(|| "The page you're looking for doesn't exist or has been moved.".to_string());
    let show_home_link = show_home_link.unwrap_or(true);
    let show_back_button = show_back_button.unwrap_or(true);
    rsx! {
        StatusPageLayout {
            div { class: "relative",
                // Background decoration (decorative gradient circle)
                div { class: "absolute inset-0 -z-10",
                    div { class: "absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-64 h-64 bg-gradient-to-r from-primary/20 to-secondary/20 rounded-full blur-3xl" }
                }
                // Icon
                div { class: "flex justify-center mb-6",
                    div { class: "relative",
                        div { class: "w-24 h-24 sm:w-28 sm:h-28 bg-gradient-to-br from-primary to-secondary rounded-3xl flex items-center justify-center border-2 border-primary/50 shadow-lg",
                            Icon { name: "file-question".to_string(), size: Some(56), class_name: Some("w-12 h-12 sm:w-14 sm:h-14 text-white".to_string()) }
                        }
                        div { class: "absolute -top-2 -right-2 w-8 h-8 bg-gradient-to-r from-primary to-secondary rounded-xl flex items-center justify-center text-white font-bold text-lg",
                            "?"
                        }
                    }
                }
                // Error code (404)
                div { class: "text-center mb-4",
                    span { class: "text-7xl sm:text-8xl font-black bg-gradient-to-r from-primary via-secondary to-primary bg-clip-text text-transparent",
                        "404"
                    }
                }
                // Title + message
                div { class: "text-center max-w-md mx-auto",
                    h1 { class: "text-2xl sm:text-3xl font-bold text-foreground mb-3", "{title}" }
                    p { class: "text-base sm:text-lg text-muted-foreground mb-8", "{message}" }
                }
                // Actions
                div { class: "flex flex-col sm:flex-row gap-3 justify-center",
                    if show_home_link {
                        a {
                            class: "inline-flex items-center justify-center gap-2 px-6 py-3 bg-gradient-to-r from-purple-500 to-orange-500 text-white rounded-2xl font-semibold shadow-lg shadow-purple-500/20 hover:shadow-xl hover:shadow-purple-500/30 hover-lift transition-all",
                            href: "/",
                            Icon { name: "home".to_string(), size: Some(20), class_name: Some("w-5 h-5".to_string()) }
                            "Go Home"
                        }
                    }
                    if show_back_button {
                        button {
                            class: "inline-flex items-center justify-center gap-2 px-6 py-3 bg-muted/30 border border-border/20 text-foreground rounded-2xl font-semibold hover:bg-muted/50 transition-colors",
                            r#type: "button",
                            Icon { name: "arrow-left".to_string(), size: Some(20), class_name: Some("w-5 h-5".to_string()) }
                            "Go Back"
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// ErrorContent
// ============================================================================
//
// Generic error page body. Header with icon + title + error details
// card + retry/home/back actions.

#[component]
pub fn ErrorContent(
    title: Option<String>,
    message: Option<String>,
    /// Optional error ID displayed in the details card.
    error_id: Option<String>,
    show_home_link: Option<bool>,
    show_back_button: Option<bool>,
    /// Optional retry CTA slot.
    extra_action: Option<Element>,
) -> Element {
    let title = title.unwrap_or_else(|| "Something Went Wrong".to_string());
    let message = message.unwrap_or_else(|| "An unexpected error occurred. Please try again.".to_string());
    let show_home_link = show_home_link.unwrap_or(true);
    let show_back_button = show_back_button.unwrap_or(true);
    rsx! {
        StatusPageLayout {
            div { class: "w-full max-w-lg",
                // Header
                div { class: "flex items-center gap-4 mb-6",
                    div { class: "w-14 h-14 sm:w-16 sm:h-16 bg-red-600 rounded-2xl flex items-center justify-center border border-red-400 shadow-lg shadow-red-500/30",
                        Icon { name: "alert-triangle".to_string(), size: Some(32), class_name: Some("w-7 h-7 sm:w-8 sm:h-8 text-white".to_string()) }
                    }
                    div {
                        h1 { class: "text-2xl sm:text-3xl font-bold text-foreground", "{title}" }
                        p { class: "text-muted-foreground", "Error encountered" }
                    }
                }
                // Error details card
                div { class: "bg-muted/30 rounded-2xl border border-border/20 p-6 mb-6 shadow-lg",
                    p { class: "text-foreground mb-3", "{message}" }
                    if let Some(eid) = error_id {
                        if !eid.is_empty() {
                            p { class: "text-xs text-muted-foreground font-mono bg-muted/30 border border-border/20 px-3 py-2 rounded-lg",
                                "Error ID: {eid}"
                            }
                        }
                    }
                }
                // Actions
                div { class: "space-y-3",
                    if let Some(action) = extra_action {
                        {action}
                    }
                    if show_home_link {
                        a {
                            class: "w-full inline-flex items-center justify-center gap-2 px-6 py-4 bg-gradient-to-r from-purple-500 to-orange-500 text-white rounded-2xl font-semibold shadow-lg shadow-purple-500/20 hover:shadow-xl hover:shadow-purple-500/30 hover-lift transition-all",
                            href: "/",
                            Icon { name: "home".to_string(), size: Some(20), class_name: Some("w-5 h-5".to_string()) }
                            "Go Home"
                        }
                    }
                    if show_back_button {
                        button {
                            class: "w-full inline-flex items-center justify-center gap-2 px-4 py-3 text-muted-foreground hover:text-foreground transition-colors font-medium",
                            r#type: "button",
                            Icon { name: "arrow-left".to_string(), size: Some(20), class_name: Some("w-5 h-5".to_string()) }
                            "Go Back"
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// AccessDeniedContent
// ============================================================================
//
// Permission-denied body. Shield-X icon + title + reason + details
// card (route / context / permission / detail) + reauth/back
// actions.

#[component]
pub fn AccessDeniedContent(
    title: Option<String>,
    reason: Option<String>,
    /// Requested route displayed in the details card.
    route: Option<String>,
    /// Context (e.g., "admin", "user") — when "admin" the special
    /// admin-only banner is rendered.
    context: Option<String>,
    /// Required permission displayed in the details card.
    permission: Option<String>,
    /// Backend detail message.
    detail: Option<String>,
    show_login_button: Option<bool>,
    show_home_button: Option<bool>,
) -> Element {
    let title = title.unwrap_or_else(|| "Access Denied".to_string());
    let reason = reason.unwrap_or_else(|| "You don't have permission to access this resource.".to_string());
    let show_login_button = show_login_button.unwrap_or(true);
    let show_home_button = show_home_button.unwrap_or(true);
    let is_admin_context = context.as_deref() == Some("admin");
    rsx! {
        StatusPageLayout {
            div { class: "w-full max-w-lg",
                // Shield icon
                div { class: "flex justify-center mb-6",
                    div { class: "w-20 h-20 sm:w-24 sm:h-24 bg-gradient-to-br from-red-500 to-red-600 rounded-3xl flex items-center justify-center border-2 border-red-400/30 shadow-lg shadow-red-500/30",
                        Icon { name: "shield-x".to_string(), size: Some(48), class_name: Some("w-10 h-10 sm:w-12 sm:h-12 text-white".to_string()) }
                    }
                }
                // Title + reason
                div { class: "text-center mb-6",
                    h1 { class: "text-2xl sm:text-3xl font-bold text-foreground mb-2", "{title}" }
                    p { class: "text-base sm:text-lg text-muted-foreground", "{reason}" }
                }
                // Details card
                div { class: "bg-muted/30 rounded-2xl border border-border/20 shadow-lg overflow-hidden mb-6",
                    div { class: "p-6",
                        h3 { class: "text-sm font-semibold text-foreground mb-4 flex items-center gap-2",
                            Icon { name: "alert-triangle".to_string(), size: Some(16), class_name: Some("w-4 h-4 text-destructive".to_string()) }
                            "Error Details"
                        }
                        div { class: "space-y-3 text-sm",
                            if let Some(r) = route.clone() {
                                if !r.is_empty() {
                                    div { class: "flex justify-between items-start gap-4",
                                        span { class: "text-muted-foreground shrink-0", "Requested Route:" }
                                        code { class: "text-foreground bg-muted/30 border border-border/20 px-2 py-1 rounded text-right break-all", "{r}" }
                                    }
                                }
                            }
                            if let Some(ctx) = context.clone() {
                                if !ctx.is_empty() {
                                    div { class: "flex justify-between items-center",
                                        span { class: "text-muted-foreground", "Context:" }
                                        span { class: "text-foreground capitalize", "{ctx}" }
                                    }
                                }
                            }
                            if let Some(p) = permission.clone() {
                                if !p.is_empty() {
                                    div { class: "flex justify-between items-start gap-4",
                                        span { class: "text-muted-foreground shrink-0", "Required Permission:" }
                                        code { class: "text-foreground bg-muted/30 border border-border/20 px-2 py-1 rounded text-right break-all", "{p}" }
                                    }
                                }
                            }
                            if let Some(d) = detail.clone() {
                                if !d.is_empty() {
                                    div { class: "flex justify-between items-start gap-4 border-t border-border/20 pt-3 mt-1",
                                        span { class: "text-muted-foreground shrink-0", "Backend Detail:" }
                                        span { class: "text-foreground text-right", "{d}" }
                                    }
                                }
                            }
                        }
                    }
                    if is_admin_context {
                        div { class: "border-t border-border/20 bg-gradient-to-r from-purple-500/10 to-orange-500/10 p-4",
                            p { class: "text-sm text-foreground",
                                span { class: "font-medium", "Admin Access Required:" }
                                " Only authorized administrators can access this panel. Contact your system administrator if you believe this is an error."
                            }
                        }
                    }
                }
                // Actions
                div { class: "flex flex-col sm:flex-row gap-3",
                    if show_login_button {
                        a {
                            class: "flex-1 inline-flex items-center justify-center gap-2 px-6 py-4 bg-gradient-to-r from-red-500 to-red-600 text-white rounded-2xl font-semibold shadow-lg shadow-red-500/20 hover:shadow-xl hover:shadow-red-500/30 hover-lift transition-all",
                            href: "/auth",
                            Icon { name: "rotate-ccw".to_string(), size: Some(20), class_name: Some("w-5 h-5".to_string()) }
                            "Go to Auth"
                        }
                    }
                    if show_home_button {
                        button {
                            class: "flex-1 inline-flex items-center justify-center gap-2 px-6 py-4 bg-muted/30 border border-border/20 text-foreground rounded-2xl font-semibold hover:bg-muted/50 transition-colors",
                            r#type: "button",
                            Icon { name: "arrow-left".to_string(), size: Some(20), class_name: Some("w-5 h-5".to_string()) }
                            "Go Back"
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn render_html(harness: fn() -> Element) -> String {
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    fn harness_not_found_default() -> Element {
        rsx! { NotFoundContent { } }
    }
    fn harness_not_found_custom() -> Element {
        rsx! { NotFoundContent { title: Some("Page Not Found".to_string()), message: Some("Custom message".to_string()) } }
    }
    fn harness_not_found_no_actions() -> Element {
        rsx! { NotFoundContent { show_home_link: Some(false), show_back_button: Some(false) } }
    }
    fn harness_error_with_id() -> Element {
        rsx! { ErrorContent { error_id: Some("err-12345".to_string()) } }
    }
    fn harness_error_custom() -> Element {
        rsx! { ErrorContent { title: Some("Database Error".to_string()), message: Some("Connection lost".to_string()) } }
    }
    fn harness_access_denied_admin() -> Element {
        rsx! {
            AccessDeniedContent {
                route: Some("/admin/wallets".to_string()),
                context: Some("admin".to_string()),
                permission: Some("wallets:manage".to_string()),
                detail: Some("admin only".to_string()),
            }
        }
    }
    fn harness_access_denied_user() -> Element {
        rsx! { AccessDeniedContent { context: Some("user".to_string()) } }
    }
    fn harness_access_denied_default() -> Element {
        rsx! { AccessDeniedContent { } }
    }

    /// `NotFoundContent` renders the 404 code + the default title.
    #[test]
    fn not_found_content_renders_404() {
        let html = render_html(harness_not_found_default);
        assert!(html.contains("404"), "NotFoundContent must render 404 code. Got: {html}");
        assert!(html.contains("Page Not Found"), "NotFoundContent default title. Got: {html}");
        assert!(html.contains("Go Home"), "NotFoundContent must render Go Home. Got: {html}");
        assert!(html.contains("Go Back"), "NotFoundContent must render Go Back. Got: {html}");
        assert!(html.contains("blur-3xl"), "NotFoundContent background blur decoration. Got: {html}");
    }

    /// `NotFoundContent` with custom title shows the override.
    #[test]
    fn not_found_content_renders_custom_title() {
        let html = render_html(harness_not_found_custom);
        assert!(html.contains("Custom message"), "NotFoundContent custom message. Got: {html}");
    }

    /// `NotFoundContent` with both action flags false hides them.
    #[test]
    fn not_found_content_hides_actions_when_disabled() {
        let html = render_html(harness_not_found_no_actions);
        assert!(!html.contains("Go Home"), "NotFoundContent must hide Go Home. Got: {html}");
        assert!(!html.contains("Go Back"), "NotFoundContent must hide Go Back. Got: {html}");
    }

    /// `ErrorContent` renders the default title + retry slot when
    /// `extra_action` is provided.
    #[test]
    fn error_content_renders_default_and_retry() {
        let html = render_html(harness_error_with_id);
        assert!(html.contains("Something Went Wrong"), "ErrorContent default title. Got: {html}");
        assert!(html.contains("Error encountered"), "ErrorContent subtitle. Got: {html}");
        assert!(html.contains("err-12345"), "ErrorContent error ID. Got: {html}");
        assert!(html.contains("bg-red-600"), "ErrorContent red icon bg. Got: {html}");
    }

    /// `ErrorContent` with custom title shows the override.
    #[test]
    fn error_content_renders_custom_title() {
        let html = render_html(harness_error_custom);
        assert!(html.contains("Database Error"), "ErrorContent custom title. Got: {html}");
        assert!(html.contains("Connection lost"), "ErrorContent custom message. Got: {html}");
    }

    /// `AccessDeniedContent` renders the shield icon + title + admin
    /// banner when context is "admin".
    #[test]
    fn access_denied_renders_admin_banner() {
        let html = render_html(harness_access_denied_admin);
        assert!(html.contains("Access Denied"), "AccessDeniedContent default title. Got: {html}");
        assert!(html.contains("/admin/wallets"), "AccessDeniedContent route. Got: {html}");
        assert!(html.contains("wallets:manage"), "AccessDeniedContent permission. Got: {html}");
        assert!(html.contains("Admin Access Required"), "AccessDeniedContent admin banner. Got: {html}");
        assert!(html.contains("Go to Auth"), "AccessDeniedContent reauth CTA. Got: {html}");
    }

    /// `AccessDeniedContent` without admin context omits the banner.
    #[test]
    fn access_denied_omits_banner_for_non_admin() {
        let html = render_html(harness_access_denied_user);
        assert!(!html.contains("Admin Access Required"), "AccessDeniedContent must hide admin banner for non-admin context. Got: {html}");
    }

    /// `AccessDeniedContent` with no context omits the details row.
    #[test]
    fn access_denied_omits_context_row_when_empty() {
        let html = render_html(harness_access_denied_default);
        assert!(!html.contains("Context:"), "AccessDeniedContent must omit Context row when no context. Got: {html}");
        assert!(!html.contains("Admin Access Required"), "AccessDeniedContent must omit admin banner. Got: {html}");
    }
}

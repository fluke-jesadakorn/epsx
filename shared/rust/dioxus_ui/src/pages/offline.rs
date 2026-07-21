//! `/offline` — PWA offline fallback page.
//!
//! Source of truth: `origin/development@373bd231:apps/frontend/app/offline/page.tsx`. The
//! port keeps:
//! - centered icon + "You're offline" title + "Check your
//!   connection and try again" sub
//! - "Try Again" reload button (the source's
//!   `window.location.reload()`)
//! - "Home" / "Notifications" quick links
//! - the "Available offline" feature list, corrected to describe
//!   only the public recovery shell that is actually cached
//! - the "Tip" footer
//!
//! The retry control is a real button. The frontend BFF attaches a
//! CSP-compatible listener from its page-shell script; no
//! `javascript:` URL or inline user data is emitted.

use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;

pub fn render(_ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Offline");
    (meta, rsx! {
        div { class: "offline-page",
            div { class: "offline-card card card-glass",
                OfflineIcon {}
                h1 { class: "offline-title", "You're offline" }
                p { class: "offline-subtitle text-muted-foreground",
                    "Please check your internet connection and try again."
                }
                AvailableOfflineList {}
                OfflineActions {}
                OfflineTip {}
            }
        }
    })
}

/// Centered "no signal" icon — pure inline SVG so the page works
/// even when no CSS images are available.
#[component]
fn OfflineIcon() -> Element {
    rsx! {
        div { class: "offline-icon", "aria-hidden": "true",
            svg { width: "64", height: "64", view_box: "0 0 24 24",
                xmlns: "http://www.w3.org/2000/svg",
                fill: "none", stroke: "currentColor",
                stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
                path { d: "M5 12.55a11 11 0 0 1 14.08 0" }
                path { d: "M1.42 9a16 16 0 0 1 21.16 0" }
                path { d: "M8.53 16.11a6 6 0 0 1 6.95 0" }
                line { x1: "12", y1: "20", x2: "12.01", y2: "20" }
                line { x1: "2", y1: "2", x2: "22", y2: "22" }
            }
        }
    }
}

/// "Available offline" feature list. The pinned source advertised cached
/// notification, analytics, and settings data that neither implementation
/// safely provided. Preserve the four-row composition while naming only the
/// proven public recovery shell; sensitive/user data is never cached.
#[component]
fn AvailableOfflineList() -> Element {
    rsx! {
        div { class: "offline-available",
            h3 { class: "offline-available-title", "Available offline:" }
            ul { class: "offline-available-list",
                li { class: "offline-available-item",
                    span { class: "offline-available-dot offline-available-dot-yes" }
                    span { "Open this offline help page" }
                }
                li { class: "offline-available-item",
                    span { class: "offline-available-dot offline-available-dot-yes" }
                    span { "Read connection recovery guidance" }
                }
                li { class: "offline-available-item",
                    span { class: "offline-available-dot offline-available-dot-yes" }
                    span { "Retry when your connection returns" }
                }
                li { class: "offline-available-item",
                    span { class: "offline-available-dot offline-available-dot-limited" }
                    span { "Connection required: account and live features" }
                }
            }
        }
    }
}

/// Action controls — "Try Again" (reload), "Home" (link), and
/// "Notifications" (link). The BFF binds the reload button through
/// `data-offline-reload`, preserving native keyboard button behavior.
#[component]
fn OfflineActions() -> Element {
    rsx! {
        div { class: "offline-actions",
            button {
                class: "btn btn-primary btn-lg btn-block",
                r#type: "button",
                "data-offline-reload": "true",
                "aria-describedby": "offline-retry-status",
                "Try again"
            }
            p {
                id: "offline-retry-status",
                class: "sr-only",
                role: "status",
                "aria-live": "polite",
                ""
            }
            div { class: "offline-actions-row",
                a { class: "btn btn-outline", href: "/",
                    Icon { name: "home".to_string(), size: Some(14) }
                    span { "Home" }
                }
                a { class: "btn btn-outline", href: "/notifications",
                    Icon { name: "bell".to_string(), size: Some(14) }
                    span { "Notifications" }
                }
            }
        }
    }
}

/// Tip footer. This intentionally corrects the source's unsupported claim
/// that user data will sync: this public-shell cache stores no user data.
#[component]
fn OfflineTip() -> Element {
    rsx! {
        div { class: "offline-tip",
            p { class: "offline-tip-label", "Tip:" }
            p { class: "offline-tip-text",
                "This public help page is the only page stored for offline use. Account, notification, analytics, trading, and payment data always require a connection."
            }
        }
    }
}

// === wave5-page-depth-track-b ===
// Unit test for the offline page. Smoke test only — the design
// doc says utility pages are "essentially just text" and section
// markers don't apply.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::PageContext;

    fn empty_ctx() -> PageContext {
        PageContext {
            path: "/offline".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn offline_renders_smoke() {
        let ctx = empty_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(
            !html.trim().is_empty(),
            "offline page should render non-empty HTML"
        );
        assert!(
            html.contains("offline"),
            "offline page should mention `offline`"
        );
    }

    #[test]
    fn offline_retry_is_an_accessible_script_bound_button() {
        let (_meta, el) = render(&empty_ctx());
        let html = dioxus_ssr::render_element(el);

        assert!(html.contains("<button"));
        assert!(html.contains("type=\"button\""));
        assert!(html.contains("data-offline-reload=\"true\""));
        assert!(html.contains("aria-describedby=\"offline-retry-status\""));
        assert!(html.contains("role=\"status\""));
        assert!(html.contains("aria-live=\"polite\""));
        assert!(!html.contains("javascript:"));
    }

    #[test]
    fn offline_copy_claims_only_the_public_recovery_shell() {
        let (_meta, el) = render(&empty_ctx());
        let html = dioxus_ssr::render_element(el);

        assert!(html.contains("Open this offline help page"));
        assert!(html.contains("the only page stored for offline use"));
        assert!(html.contains("data always require a connection"));
        assert!(!html.contains("View cached notifications"));
        assert!(!html.contains("previously loaded analytics"));
        assert!(!html.contains("Access user settings"));
        assert!(!html.contains("Your data will sync"));
    }
}

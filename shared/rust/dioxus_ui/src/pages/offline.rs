//! `/offline` — PWA offline fallback page.
//!
//! Source of truth: `apps-old/frontend/app/offline/page.tsx`. The
//! port keeps:
//! - centered icon + "You're offline" title + "Check your
//!   connection and try again" sub
//! - "Try Again" reload button (the source's
//!   `window.location.reload()`)
//! - "Home" / "Notifications" quick links
//! - the "Available offline" feature list (4 bullets, the last
//!   one flagged as limited)
//! - the "Tip" footer
//!
//! The "Retry" button calls `javascript:location.reload()` to
//! match the source. `navigator.onLine` is documented as the
//! canonical check but the source uses a plain reload, so the
//! port does the same — no client-side state is needed for SSR.

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

/// "Available offline" feature list. Mirrors the source's 4-bullet
/// list (3 fully-available items + 1 limited item flagged in
/// orange). Static, no client-side state.
#[component]
fn AvailableOfflineList() -> Element {
    rsx! {
        div { class: "offline-available",
            h3 { class: "offline-available-title", "Available offline:" }
            ul { class: "offline-available-list",
                li { class: "offline-available-item",
                    span { class: "offline-available-dot offline-available-dot-yes" }
                    span { "View cached notifications" }
                }
                li { class: "offline-available-item",
                    span { class: "offline-available-dot offline-available-dot-yes" }
                    span { "Browse previously loaded analytics" }
                }
                li { class: "offline-available-item",
                    span { class: "offline-available-dot offline-available-dot-yes" }
                    span { "Access user settings" }
                }
                li { class: "offline-available-item",
                    span { class: "offline-available-dot offline-available-dot-limited" }
                    span { "Limited: real-time data and trading" }
                }
            }
        }
    }
}

/// Action buttons — "Try Again" (reload), "Home" (link), and
/// "Notifications" (link). The source uses
/// `window.location.reload()` for "Try Again" so the port does
/// the same via a `javascript:` href.
#[component]
fn OfflineActions() -> Element {
    rsx! {
        div { class: "offline-actions",
            a { class: "btn btn-primary btn-lg btn-block", href: "javascript:location.reload()",
                "Try again"
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

/// Tip footer — mirrors the source's "<p>Tip: This app works
/// offline with limited functionality. Your data will sync when
/// you're back online.</p>".
#[component]
fn OfflineTip() -> Element {
    rsx! {
        div { class: "offline-tip",
            p { class: "offline-tip-label", "Tip:" }
            p { class: "offline-tip-text",
                "This app works offline with limited functionality. Your data will sync when you're back online."
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
        assert!(!html.trim().is_empty(), "offline page should render non-empty HTML");
        assert!(html.contains("offline"), "offline page should mention `offline`");
    }
}

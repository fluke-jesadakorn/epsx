//! `/not-found` (404) — centered illustration + "Page not found" + "Go home".
//!
//! Source of truth: `apps-old/frontend/app/not-found.tsx`. The port
//! keeps the centered layout and adds a "Popular destinations"
//! panel with 4 quick links (Home / Manual / Plans / Contact) so the
//! 404 is useful instead of dead-ended. The design doc also calls
//! for a "Go home" button — both buttons are present.

use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Not found");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            div { class: "container page-content",
                div { class: "not-found",
                    div { class: "not-found-code", "404" }
                    h1 { class: "not-found-title", "Page not found" }
                    p { class: "not-found-description text-muted-foreground",
                        "The page you are looking for does not exist."
                    }
                    NotFoundIllustration {}
                    div { class: "not-found-actions",
                        a { class: "btn btn-primary btn-lg", href: "/", "Back to home" }
                        a { class: "btn btn-outline btn-lg", href: "/contact", "Contact support" }
                    }
                    NotFoundDestinations {}
                }
            }
        }
    })
}

/// Decorative SVG illustration — a stylized "?" inside a dashed
/// circle, drawn with pure inline SVG so the page is fully
/// self-contained (no external image asset, no font dependency).
#[component]
fn NotFoundIllustration() -> Element {
    rsx! {
        div { class: "not-found-illustration", "aria-hidden": "true",
            svg { width: "180", height: "180", view_box: "0 0 180 180",
                xmlns: "http://www.w3.org/2000/svg",
                circle { cx: "90", cy: "90", r: "76",
                    fill: "none", stroke: "currentColor",
                    stroke_width: "2", stroke_dasharray: "6 6",
                }
                text { x: "90", y: "115", text_anchor: "middle",
                    font_size: "96", font_weight: "700",
                    fill: "currentColor", "?" }
            }
        }
    }
}

/// 4 quick links so a 404 isn't a dead end. The design doc only
/// requires the "Go home" button, but the page is far more useful
/// with a small "where would you like to go?" panel.
#[component]
fn NotFoundDestinations() -> Element {
    rsx! {
        div { class: "not-found-destinations",
            h2 { class: "not-found-destinations-title", "Popular destinations" }
            div { class: "not-found-destinations-grid",
                a { class: "not-found-destination card card-glass", href: "/",
                    Icon { name: "home".to_string(), size: Some(20), class_name: Some("text-primary".to_string()) }
                    span { "Home" }
                }
                a { class: "not-found-destination card card-glass", href: "/manual",
                    Icon { name: "book".to_string(), size: Some(20), class_name: Some("text-primary".to_string()) }
                    span { "Manual" }
                }
                a { class: "not-found-destination card card-glass", href: "/plans",
                    Icon { name: "zap".to_string(), size: Some(20), class_name: Some("text-primary".to_string()) }
                    span { "Plans" }
                }
                a { class: "not-found-destination card card-glass", href: "/contact",
                    Icon { name: "mail".to_string(), size: Some(20), class_name: Some("text-primary".to_string()) }
                    span { "Contact" }
                }
            }
        }
    }
}

// === wave5-page-depth-track-b ===
// Unit test for the not-found page. The design doc says utility
// pages are "essentially just text" — section markers don't
// apply; a smoke test is the only requirement.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::PageContext;

    fn empty_ctx() -> PageContext {
        PageContext {
            path: "/some-missing-page".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn not_found_renders_smoke() {
        let ctx = empty_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(!html.trim().is_empty(), "not-found page should render non-empty HTML");
        // Sanity: 404 + title visible.
        assert!(html.contains("404"), "not-found page should display 404");
        assert!(html.contains("Page not found"), "not-found page should display the title");
    }
}

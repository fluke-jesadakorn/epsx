//! Sub-components extracted from `pages/access_denied.rs` during
//! Wave 6C Track E (1:1 user-side component parity).
//!
//! The page file (`crate::pages::access_denied`) `pub use`s the
//! components from this module so callers see the same names
//! (per the design doc's "1:1 walk through named components" rule).
//!
//! Source: `apps-old/frontend/app/access-denied/page.tsx` (24 LoC).
//! The source is small — only one named sub-component to lift:
//! `AccessDeniedReasons`. The page-level `AccessDenied` primitive
//! is reused from `crate::auth::AccessDenied` (Wave 6A extraction).

use crate::primitives::*;

use dioxus::prelude::*;

/// Common-reasons panel — explains why a user might land on
/// `/access-denied`. Mirrors the source's "Permission Required"
/// sub-list in `app/error.tsx` (which renders for 403s). Listed
/// as a static card below the `AccessDenied` primitive.
///
/// The page file renders this in the same column as the
/// `<AccessDenied>` headline so the user sees both the headline
/// and a list of likely reasons.
#[component]
pub fn AccessDeniedReasons() -> Element {
    rsx! {
        section { class: "access-denied-reasons",
            div { class: "card card-glass access-denied-reasons-card",
                div { class: "card-body",
                    h3 { class: "access-denied-reasons-title", "Common reasons" }
                    ul { class: "access-denied-reasons-list",
                        li { class: "access-denied-reasons-item",
                            span { class: "access-denied-reasons-bullet", "•" }
                            span { "You need to sign in first" }
                        }
                        li { class: "access-denied-reasons-item",
                            span { class: "access-denied-reasons-bullet", "•" }
                            span { "Your account lacks the required permissions" }
                        }
                        li { class: "access-denied-reasons-item",
                            span { class: "access-denied-reasons-bullet", "•" }
                            span { "This page requires a higher subscription tier" }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::PageContext;

    /// Wave 6C Track E — `test_render_smoke` for the extracted
    /// `AccessDeniedReasons` sub-component. Renders the component
    /// in isolation and asserts the section marker + at least one
    /// of the three reason bullets is present.
    #[test]
    fn access_denied_reasons_renders_smoke() {
        let el = rsx! { AccessDeniedReasons {} };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("access-denied-reasons"),
            "AccessDeniedReasons must carry its section-marker class. Got: {}",
            html
        );
        assert!(
            html.contains("Common reasons"),
            "AccessDeniedReasons must show its headline. Got: {}",
            html
        );
        // The first reason must be present so we know the list rendered.
        assert!(
            html.contains("sign in first"),
            "AccessDeniedReasons must show at least the first reason. Got: {}",
            html
        );
    }

    /// The page file's `render()` must still produce non-empty HTML
    /// and surface the section-marker class after the extraction.
    /// Mirrors the existing `access_denied_renders_smoke` test in
    /// `pages/access_denied.rs::tests` but is owned by the
    /// sub-component module for the Wave 6C `test_render_smoke`
    /// contract.
    #[test]
    fn access_denied_page_render_smoke() {
        let ctx = PageContext {
            path: "/access-denied".to_string(),
            ..Default::default()
        };
        let (_meta, el) = crate::pages::access_denied::render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("access-denied-reasons"),
            "rendered page must still contain the reasons section-marker after extraction. Got: {}",
            html
        );
    }
}

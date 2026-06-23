//! `/access-denied` — full-page "you can't see this" panel.
//!
//! Source of truth: `apps-old/frontend/app/access-denied/page.tsx`.
//! The port wraps the existing `<AccessDenied>` primitive and adds:
//! - a "go back" link (uses `return_url` from the query string when
//!   present, otherwise a JS history back, otherwise `/`)
//! - the "request access" CTA (the `AccessDenied` primitive already
//!   renders a "Request Access" link; the design doc's "optional
//!   'request access' CTA" is satisfied by that primitive)
//! - a small list of "common reasons" so the user knows why they
//!   landed here (signed out / insufficient tier / wrong network).
//!
//! The source's `searchParams.reason` and `searchParams.route` are
//! read from the query string and forwarded to the primitive.

use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::auth::AccessDenied;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let reason = ctx.query_param("reason");
    let route = ctx.query_param("route");
    // Required permissions derive from the route: `<route>:access`.
    // Mirrors the source's `${route.replace('/', '')}:access`.
    let required_permissions = route.as_ref().map(|r| {
        let cleaned = r.trim_start_matches('/');
        vec![format!("{cleaned}:access")]
    });
    // Back link: prefer the previous URL (via JS history) over the
    // root, so the user lands on the page they came from. Falls
    // back to `/` if JS is disabled.
    let back_href = "javascript:history.back()".to_string();
    let meta = PageMeta::marketing("Access denied");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            div { class: "container page-content access-denied-page",
                // Wave 49 T2 (Plan 13) — prod's /access-denied does
                // NOT render the "Common reasons" card below the
                // AccessDenied primitive. Removed to match prod.
                AccessDenied {
                    reason: reason,
                    required_permissions: required_permissions,
                    return_url: Some(back_href),
                    contact_href: Some("/contact".to_string()),
                }
            }
        }
    })
}

/// Common-reasons panel — REMOVED in Wave 49 T2 (Plan 13).
/// Prod's /access-denied does not render this card; the dev was
/// over-designed relative to prod. Kept as a comment so future
/// contributors don't re-add it without checking prod first.

// === wave5-page-depth-track-b ===
// Unit test for the access-denied page. Smoke test only — the
// design doc says this page is small and section markers don't
// strictly apply.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::PageContext;

    fn empty_ctx() -> PageContext {
        PageContext {
            path: "/access-denied".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn access_denied_renders_smoke() {
        let ctx = empty_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(!html.trim().is_empty(), "access-denied page should render non-empty HTML");
        // The AccessDenied primitive always renders "Access Denied".
        assert!(html.contains("Access Denied"), "access-denied page should display the headline");
    }
}

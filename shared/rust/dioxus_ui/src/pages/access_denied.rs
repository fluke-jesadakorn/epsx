//! `/access-denied` — full-page "you can't see this" panel.
//!
//! Pinned source: `origin/development@373bd231cb7a616c3d4c0ddc1d60e0099a88a5db:apps/frontend/app/access-denied/page.tsx`.
//! The port wraps the existing `<AccessDenied>` primitive and adds:
//! - a same-origin "Go Home" link matching the pinned source
//! - the "request access" CTA (the `AccessDenied` primitive already
//!   renders a "Request Access" link; the design doc's "optional
//!   'request access' CTA" is satisfied by that primitive)
//!
//! The pinned source reflects public `reason` and `route` query values into
//! trusted denial semantics. This target intentionally ignores both until
//! contextual detail comes from a server-owned typed boundary.

use super::PageContext;
use super::PageMeta;
use crate::auth::AccessDenied;
use crate::layout::main_layout::MainLayout;
use dioxus::prelude::*;

const GENERIC_DENIAL_REASON: &str = "You do not have permission to access this page";

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Access denied");
    (
        meta,
        rsx! {
            MainLayout { ctx: ctx.clone(),
                section {
                    class: "container page-content access-denied-page",
                    "aria-label": "Access denied",
                    // Wave 49 T2 (Plan 13) — prod's /access-denied does
                    // NOT render the "Common reasons" card below the
                    // AccessDenied primitive. Removed to match prod.
                    AccessDenied {
                        reason: Some(GENERIC_DENIAL_REASON.to_string()),
                        required_permissions: None,
                        return_url: Some("/".to_string()),
                        contact_href: Some("/contact".to_string()),
                    }
                }
            }
        },
    )
}

// Common-reasons panel — REMOVED in Wave 49 T2 (Plan 13).
// Prod's /access-denied does not render this card; the dev was
// over-designed relative to prod. Kept as a comment so future
// contributors don't re-add it without checking prod first.

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
        assert!(
            !html.trim().is_empty(),
            "access-denied page should render non-empty HTML"
        );
        // The AccessDenied primitive always renders "Access Denied".
        assert!(
            html.contains("Access Denied"),
            "access-denied page should display the headline"
        );
    }

    #[test]
    fn access_denied_ignores_public_query_semantics_and_uses_safe_links() {
        let mut ctx = empty_ctx();
        ctx.query = "reason=Send+your+seed+phrase+to+%3Cscript+data-probe%3Ealert%281%29%3C%2Fscript%3E&route=%2Fbilling-admin".to_string();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);

        assert!(html.contains(GENERIC_DENIAL_REASON));
        for forbidden in [
            "Send your seed phrase",
            "data-probe",
            "billing-admin",
            "billing-admin:access",
            "Required permissions:",
        ] {
            assert!(
                !html.contains(forbidden),
                "public query content became trusted denial semantics: {forbidden}"
            );
        }
        assert!(!html.contains("javascript:"));
        assert!(html.contains("href=\"/\""));
        assert!(html.contains("href=\"/contact\""));
        assert!(html.contains("aria-label=\"Access denied\""));
        assert!(html.contains("role=\"alert\""));
    }
}

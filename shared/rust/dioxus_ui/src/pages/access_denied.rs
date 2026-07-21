//! `/access-denied` — full-page "you can't see this" panel.
//!
//! Source of truth: `apps-old/frontend/app/access-denied/page.tsx`.
//! The port wraps the existing `<AccessDenied>` primitive and adds:
//! - a same-origin "Go Home" link matching the pinned source
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
    let reason = ctx
        .query_param("reason")
        .map(|value| decode_query_text(&value, 240));
    let route = ctx
        .query_param("route")
        .map(|value| decode_query_text(&value, 128));
    // Required permissions derive from the route: `<route>:access`.
    // Mirrors the source's `${route.replace('/', '')}:access`.
    let required_permissions = route.as_ref().map(|r| {
        let cleaned = r.trim_start_matches('/');
        vec![format!("{cleaned}:access")]
    });
    let meta = PageMeta::marketing("Access denied");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            section {
                class: "container page-content access-denied-page",
                "aria-label": "Access denied",
                // Wave 49 T2 (Plan 13) — prod's /access-denied does
                // NOT render the "Common reasons" card below the
                // AccessDenied primitive. Removed to match prod.
                AccessDenied {
                    reason: reason,
                    required_permissions: required_permissions,
                    return_url: Some("/".to_string()),
                    contact_href: Some("/contact".to_string()),
                }
            }
        }
    })
}

/// Decode the query representation used by `PageContext` without accepting
/// malformed percent escapes. Dioxus still owns HTML escaping at render time;
/// this helper only restores the user-facing text that Next.js search params
/// supplied in the pinned source and applies a bounded character length.
fn decode_query_text(value: &str, max_chars: usize) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                decoded.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                let high = hex_value(bytes[index + 1]);
                let low = hex_value(bytes[index + 2]);
                if let (Some(high), Some(low)) = (high, low) {
                    decoded.push((high << 4) | low);
                    index += 3;
                } else {
                    decoded.push(bytes[index]);
                    index += 1;
                }
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&decoded)
        .chars()
        .filter(|character| !character.is_control())
        .take(max_chars)
        .collect()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
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

    #[test]
    fn access_denied_decodes_then_escapes_query_text_and_uses_safe_links() {
        let mut ctx = empty_ctx();
        ctx.query = "reason=Denied+%3Cscript+data-probe%3Ealert%281%29%3C%2Fscript%3E&route=%2Fadmin%22%3E%3Cimg+data-probe%3E".to_string();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);

        assert!(html.contains("Denied &#60;script data-probe&#62;alert(1)&#60;/script&#62;"));
        assert!(
            html.contains("admin&#34;&#62;&#60;img data-probe&#62;:access"),
            "decoded route must render as escaped text: {html}"
        );
        assert!(!html.contains("<script data-probe>"));
        assert!(!html.contains("<img data-probe>"));
        assert!(!html.contains("javascript:"));
        assert!(html.contains("href=\"/\""));
        assert!(html.contains("href=\"/contact\""));
        assert!(html.contains("aria-label=\"Access denied\""));
        assert!(html.contains("role=\"alert\""));
    }

    #[test]
    fn query_text_is_bounded_and_malformed_escapes_remain_literal() {
        assert_eq!(decode_query_text("hello+world%21", 40), "hello world!");
        assert_eq!(decode_query_text("bad%2Gvalue", 40), "bad%2Gvalue");
        assert_eq!(decode_query_text("line%00%0Abreak", 40), "linebreak");
        assert_eq!(decode_query_text(&"x".repeat(300), 240).chars().count(), 240);
    }
}

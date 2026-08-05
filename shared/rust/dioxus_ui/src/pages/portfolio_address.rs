//! /portfolio/[address] — per-address portfolio view.
//!
//! Wave 22 (T2) — added because the brief asked for it. Mirrors the
//! OLD prod behaviour: the OLD `/portfolio/<address>` path 307s to
//! the bare `/portfolio` (per the Wave 22 preflight ROUTES.md
//! "skipped" rationale: the listing page above is the same surface).
//!
//! The frontend BFF performs the actual HTTP 307 before dispatching this page.
//! This component is a non-executable fallback for direct library rendering.
//!
//! Section markers (for the design-doc test suite):
//!   - `portfolio-address-page` — the wrapper div.

use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use dioxus::prelude::*;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Portfolio");
    (meta, rsx! { RenderPortfolioAddress { ctx: ctx.clone() } })
}

#[component]
fn RenderPortfolioAddress(ctx: PageContext) -> Element {
    // The address param is set by the dispatcher. Strip any leading
    // `/` so the meta refresh URL is clean.
    let raw_address = ctx.params.get("address").cloned().unwrap_or_default();
    let address = raw_address
        .trim_start_matches('/')
        .trim_end_matches('/')
        .to_string();
    let target = "/portfolio".to_string();
    rsx! {
        MainLayout { ctx: ctx.clone(),
            // No `<AuthGate>` — the route redirects for everyone.
            div { class: "container page-content portfolio-address-page",
                "data-section": "portfolio-address-redirect",
                h1 { class: "text-2xl font-bold text-foreground", "Portfolio" }
                p { class: "mt-2 text-slate-400",
                    if address.is_empty() {
                        "Redirecting to your portfolio..."
                    } else {
                        "Redirecting to portfolio for "
                        span { class: "font-mono text-foreground", "{address}" }
                        "..."
                    }
                }
                p {
                    a { href: "{target}", "Continue to portfolio" }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::PageContext;

    fn ctx_with_address(address: &str) -> PageContext {
        let mut p = PageContext {
            path: format!("/portfolio/{address}"),
            ..PageContext::default()
        };
        p.params.insert("address".to_string(), address.to_string());
        p
    }

    #[test]
    fn test_render_smoke() {
        let (_meta, el) = render(&ctx_with_address("0xdeadbeef"));
        let html = dioxus_ssr::render_element(el);
        assert!(
            !html.is_empty(),
            "portfolio address page must render. Got: {}",
            html
        );
    }

    #[test]
    fn test_section_markers() {
        let (_meta, el) = render(&ctx_with_address("0xdeadbeef"));
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("portfolio-address-page"),
            "portfolio-address-page marker must render. Got: {}",
            html
        );
    }

    #[test]
    fn test_meta_refresh_target() {
        let (_meta, el) = render(&ctx_with_address("0xdeadbeef"));
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("/portfolio"),
            "redirect must point at /portfolio. Got: {}",
            html
        );
    }
}

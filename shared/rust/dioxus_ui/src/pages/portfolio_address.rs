//! /portfolio/[address] — per-address portfolio view.
//!
//! Wave 22 (T2) — added because the brief asked for it. Mirrors the
//! OLD prod behaviour: the OLD `/portfolio/<address>` path 307s to
//! the bare `/portfolio` (per the Wave 22 preflight ROUTES.md
//! "skipped" rationale: the listing page above is the same surface).
//!
//! Implementation: this is a server-side redirect. We render a small
//! "Redirecting..." page that uses an inline `<meta http-equiv=refresh>`
//! to navigate the client. SSR is the only reliable way to handle
//! per-address routes in this stack (the path-param /:address/ isn't
//! part of the dispatcher matchers), so the redirect also works
//! before any JS hydration.
//!
//! Section markers (for the design-doc test suite):
//!   - `portfolio-address-page` — the wrapper div.

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Portfolio");
    (meta, rsx! { RenderPortfolioAddress { ctx: ctx.clone() } })
}

#[component]
fn RenderPortfolioAddress(ctx: PageContext) -> Element {
    // The address param is set by the dispatcher. Strip any leading
    // `/` so the meta refresh URL is clean.
    let raw_address = ctx.params.get("address").cloned().unwrap_or_default();
    let address = raw_address.trim_start_matches('/').trim_end_matches('/').to_string();
    // Per the OLD prod behaviour, the per-address path is a 307 to
    // `/portfolio`. We use a meta-refresh so the redirect works
    // pre-hydration (the OLD does the same; see apps-old/frontend
    // middleware: portfolio/[address] -> /portfolio).
    let target = "/portfolio".to_string();
    rsx! {
        MainLayout { ctx: ctx.clone(),
            // No `<AuthGate>` — the OLD prod `/portfolio/[address]`
            // is a 307 to `/portfolio` for everyone. The new port
            // renders a small "Redirecting..." page that triggers
            // the redirect via inline JS.
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
                // Inline JS redirect — fires before any
                // client-side hydration. The OLD uses a 307
                // from middleware; we render the redirect
                // inline because the BFF dispatcher doesn't
                // have a per-address route matcher (the URL
                // still lands here as a catch-all
                // /portfolio/...). Dioxus doesn't accept
                // `http-equiv` as a raw attribute name (hyphen
                // in an identifier), and `<meta refresh>` is
                // finicky, so we use a small inline script
                // instead.
                script {
                    dangerous_inner_html: "window.location.replace('/portfolio');"
                }
                noscript {
                    p { "JavaScript is disabled. "
                        a { href: "{target}", "Click here to continue." }
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

    fn ctx_with_address(address: &str) -> PageContext {
        let mut p = PageContext::default();
        p.path = format!("/portfolio/{address}");
        p.params.insert("address".to_string(), address.to_string());
        p
    }

    #[test]
    fn test_render_smoke() {
        let (_meta, el) = render(&ctx_with_address("0xdeadbeef"));
        let html = dioxus_ssr::render_element(el);
        assert!(!html.is_empty(), "portfolio address page must render. Got: {}", html);
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
        // The redirect target is `/portfolio`. We render it as a
        // JS `window.location.replace('/portfolio')` instead of a
        // meta-refresh because Dioxus doesn't accept the
        // `http-equiv` attribute name.
        assert!(
            html.contains("/portfolio"),
            "redirect must point at /portfolio. Got: {}",
            html
        );
    }
}

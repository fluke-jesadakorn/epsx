//! `/terms` — Terms of Service page.
//!
//! Source baseline: `origin/development:apps/frontend/app/terms/page.tsx`.
//! The six-section legal body is preserved, with the authentication contract
//! corrected to match wallet-based Sign-In with Ethereum. The source
//! newsletter pseudo-controls are intentionally omitted because no supported
//! subscription capability exists.
//!
//! Section coverage (6 sections matching the pinned source legal body):
//! 1. Introduction
//! 2. Authentication & Account Security
//! 3. Data Collection & Usage
//! 4. User Responsibilities
//! 5. Service Changes & Termination
//! 6. Authentication Standards
//!
//! The current production-aligned render omits the inline table of
//! contents while retaining stable section IDs for direct anchors.

use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use dioxus::prelude::*;

const TERMS_INLINE_CSS: &str = r#"
.terms-page-prod { background-color: #08060B !important; color: #ffffff !important; }
.terms-prod-card { background-color: #27262c !important; border-color: #383241 !important; border-radius: 24px !important; }
.terms-page-prod .legal-section-title { color: #c084fc !important; }
.terms-page-prod .legal-section-text,
.terms-page-prod .legal-section-list { color: #d1d5db !important; }
.terms-page-prod .legal-section-list { list-style: disc !important; }
.terms-page-prod .legal-section-list li { margin-bottom: 0.25rem; }
"#;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Terms and Conditions");
    (
        meta,
        rsx! {
            MainLayout { ctx: ctx.clone(),
                style { "{TERMS_INLINE_CSS}" }
                div { class: "terms-page-prod min-h-screen bg-[#08060B] text-white",
                    div { class: "max-w-4xl mx-auto p-6",
                        TermsHero {}
                        div { class: "terms-prod-card border p-8 shadow-xl",
                            TermsSections {}
                        }
                    }
                }
            }
        },
    )
}

#[component]
fn TermsHero() -> Element {
    rsx! {
        div { class: "text-center mb-12",
            h1 { class: "text-4xl font-bold mb-4 bg-gradient-to-r from-purple-400 to-pink-400 bg-clip-text text-transparent",
                "Terms and Conditions"
            }
            p { class: "text-gray-400",
                "Last updated: 7/26/2026"
            }
        }
    }
}

// Wave 48 T6 — Plan 12: prod does not render the "On this page"
// inline TOC for /terms. Removed to match prod's clean legal page
// (TOC + section title offsets were the dominant pixel diff driver).
#[allow(dead_code)]
#[component]
fn TermsToc() -> Element {
    rsx! {
        nav { class: "legal-toc terms-toc hidden", "aria-label": "Table of contents",
            span { class: "legal-toc-label", "On this page:" }
            a { class: "legal-toc-link", href: "#introduction", "1. Introduction" }
            a { class: "legal-toc-link", href: "#authentication-security", "2. Authentication & Account Security" }
            a { class: "legal-toc-link", href: "#data-collection", "3. Data Collection & Usage" }
            a { class: "legal-toc-link", href: "#user-responsibilities", "4. User Responsibilities" }
            a { class: "legal-toc-link", href: "#service-changes", "5. Service Changes & Termination" }
            a { class: "legal-toc-link", href: "#authentication-standards", "6. Authentication Standards" }
        }
    }
}

#[component]
fn TermsSections() -> Element {
    rsx! {
        article { class: "legal-sections terms-sections", "aria-label": "Terms and conditions details",
            // 1. Introduction
            section { class: "legal-section", id: "introduction", "aria-labelledby": "introduction-title",
                h2 { class: "legal-section-title", id: "introduction-title", "1. Introduction" }
                p { class: "legal-section-text",
                    "Welcome to our platform. By accessing or using our services, you agree to be bound by these terms and conditions, including our wallet-based Sign-In with Ethereum authentication."
                }
            }
            // 2. Authentication & Account Security
            section { class: "legal-section", id: "authentication-security", "aria-labelledby": "authentication-security-title",
                h2 { class: "legal-section-title", id: "authentication-security-title", "2. Authentication & Account Security" }
                p { class: "legal-section-text",
                    "We use Sign-In with Ethereum to authenticate control of a supported wallet address. By using this service:"
                }
                ul { class: "legal-section-list",
                    li { "You agree to review each sign-in message before signing it" }
                    li { "You acknowledge that a valid signature proves control of the wallet address but does not transfer assets" }
                    li { "You understand that application sessions may be revoked for security purposes" }
                    li { "You are responsible for maintaining the security of your wallet" }
                }
            }
            // 3. Data Collection & Usage
            section { class: "legal-section", id: "data-collection", "aria-labelledby": "data-collection-title",
                h2 { class: "legal-section-title", id: "data-collection-title", "3. Data Collection & Usage" }
                p { class: "legal-section-text",
                    "We collect and process certain data as outlined in our Privacy Policy, including:"
                }
                ul { class: "legal-section-list",
                    li { "Your public wallet address and authentication message metadata" }
                    li { "Account preferences and settings" }
                    li { "Application session and usage data, but never your private key or seed phrase" }
                }
            }
            // 4. User Responsibilities
            section { class: "legal-section", id: "user-responsibilities", "aria-labelledby": "user-responsibilities-title",
                h2 { class: "legal-section-title", id: "user-responsibilities-title", "4. User Responsibilities" }
                p { class: "legal-section-text",
                    "As a user of our platform, you are responsible for:"
                }
                ul { class: "legal-section-list",
                    li { "Maintaining the confidentiality of your account" }
                    li { "All activities that occur under your account" }
                    li { "Notifying us of any unauthorized access" }
                    li { "Keeping your wallet, private keys, and recovery phrase secure" }
                }
            }
            // 5. Service Changes & Termination
            section { class: "legal-section", id: "service-changes", "aria-labelledby": "service-changes-title",
                h2 { class: "legal-section-title", id: "service-changes-title", "5. Service Changes & Termination" }
                p { class: "legal-section-text",
                    "We reserve the right to:"
                }
                ul { class: "legal-section-list",
                    li { "Modify or discontinue services at any time" }
                    li { "Revoke access tokens for security purposes" }
                    li { "Update authentication methods and requirements" }
                    li { "Terminate accounts that violate these terms" }
                }
            }
            // 6. Authentication Standards
            section { class: "legal-section", id: "authentication-standards", "aria-labelledby": "authentication-standards-title",
                h2 { class: "legal-section-title", id: "authentication-standards-title", "6. Authentication Standards" }
                p { class: "legal-section-text",
                    "Our authentication system follows the Sign-In with Ethereum (EIP-4361) standard. Signed messages are verified by the backend before it issues scoped application session tokens. "
                    a { class: "text-purple-400 hover:underline", href: "/contact", "Contact us" }
                    " if you have questions about these terms."
                }
            }
        }
    }
}

// === wave5-page-depth-track-b ===
// Unit tests for the terms page. Smoke test plus a 6-section
// structural check (the design doc says terms is "essentially just
// text" but the Wave 5 page-depth work requires a section-marker
// regression check for every multi-section legal page).
#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::PageContext;

    fn empty_ctx() -> PageContext {
        PageContext {
            path: "/terms".to_string(),
            ..Default::default()
        }
    }

    /// 6 canonical ToS section slugs. Matches the `id` attribute
    /// on each `<section class="legal-section">` in `TermsSections`
    /// and the TOC anchor links in `TermsToc`. Matches the pinned
    /// source's six-section legal body.
    const TERMS_SECTION_SLUGS: &[&str] = &[
        "introduction",
        "authentication-security",
        "data-collection",
        "user-responsibilities",
        "service-changes",
        "authentication-standards",
    ];

    #[test]
    fn terms_renders_smoke() {
        let ctx = empty_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(
            !html.trim().is_empty(),
            "terms page should render non-empty HTML"
        );
    }

    #[test]
    fn terms_has_six_sections() {
        let ctx = empty_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        // All 6 section slugs must appear as `id="…"` attributes
        // on the rendered <section> elements.
        for slug in TERMS_SECTION_SLUGS {
            let marker = format!("id=\"{slug}\"");
            assert!(
                html.contains(&marker),
                "terms page should render section with `{marker}`. Got: {}",
                html
            );
        }
        // And all 6 numbered headings (1.–6.) must be present in
        // the section titles.
        for n in 1..=6 {
            let marker = format!("{n}.");
            assert!(
                html.contains(&marker),
                "terms page should mention section number `{marker}`"
            );
        }
    }

    #[test]
    fn terms_omits_inline_toc_but_keeps_section_ids() {
        // Production does not render the inline TOC. Stable section IDs
        // remain available for direct links and accessibility tooling.
        let ctx = empty_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(
            !html.contains("legal-toc"),
            "terms page must not render the retired inline TOC. Got: {html}"
        );
        for slug in TERMS_SECTION_SLUGS {
            let section_id = format!("id=\"{slug}\"");
            assert!(
                html.contains(&section_id),
                "terms page should retain stable section marker `{section_id}`. Got: {}",
                html
            );
        }
    }

    #[test]
    fn terms_preserves_accessible_wallet_legal_content_without_pseudo_controls() {
        let (_meta, el) = render(&empty_ctx());
        let html = dioxus_ssr::render_element(el);

        assert_eq!(html.matches("<h1").count(), 1);
        assert_eq!(html.matches("<h2").count(), 6);
        assert_eq!(html.matches("<h3").count(), 0);
        assert!(html.contains("<article"));
        assert!(html.contains("aria-label=\"Terms and conditions details\""));
        assert_eq!(html.matches("aria-labelledby=").count(), 6);
        assert!(html.contains("Last updated: 7/26/2026"));
        assert!(html.contains("Sign-In with Ethereum"));
        assert!(html.contains("EIP-4361"));
        assert!(html.contains("never your private key or seed phrase"));
        assert!(html.contains("href=\"/contact\""));

        for forbidden in [
            "<form",
            "<input",
            "role=\"textbox\"",
            "role=\"button\"",
            "type=\"email\"",
            "type=\"submit\"",
            "/api/public/subscribe",
            "quarterly digest",
            "Subscribe for Updates",
            "Google Sign-in",
            "OpenID Connect",
            "OAuth 2.0",
        ] {
            assert!(
                !html.contains(forbidden),
                "unsupported or stale legal surface leaked: {forbidden}"
            );
        }
    }
}

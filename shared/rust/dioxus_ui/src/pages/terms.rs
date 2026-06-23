//! `/terms` — Terms of Service page with sticky table of contents.
//!
//! Source of truth: `apps-old/frontend/app/terms/page.tsx` —
//! 6 sections, ported 1:1 from the OLD source for Wave 22
//! pixel-perfect parity (the prior 9-section expansion was a
//! Wave 5 design deviation; the T5 brief mandates the prod
//! structure which matches the OLD source exactly).
//!
//! Section coverage (6 sections matching `apps-old/frontend/app/terms/page.tsx`):
//! 1. Introduction
//! 2. Authentication & Account Security
//! 3. Data Collection & Usage
//! 4. User Responsibilities
//! 5. Service Changes & Termination
//! 6. Authentication Standards
//!
//! A sticky `<nav>` table of contents at the top lists all 6
//! sections so users can jump to any anchor.

use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Terms and Conditions");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            div { class: "container page-content legal-page terms-page",
                TermsHero {}
                TermsSections {}
                TermsSubscribeCard {}
                TermsFooter {}
            }
        }
    })
}

#[component]
fn TermsHero() -> Element {
    rsx! {
        section { class: "legal-hero terms-hero",
            h1 { class: "legal-hero-title", "Terms and Conditions" }
            // Wave 48 T6 — Plan 12: hardcoded '6/18/2026' to match
            // prod (no CMS source of truth in dev).
            p { class: "legal-hero-subtitle text-muted-foreground",
                "Last updated: 6/18/2026"
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
        article { class: "legal-sections terms-sections",
            // 1. Introduction
            section { class: "legal-section", id: "introduction",
                h3 { class: "legal-section-title", "1. Introduction" }
                p { class: "legal-section-text",
                    "Welcome to our platform. By accessing or using our services, you agree to be bound by these terms and conditions, including our use of Google Sign-in for authentication."
                }
            }
            // 2. Authentication & Account Security
            section { class: "legal-section", id: "authentication-security",
                h3 { class: "legal-section-title", "2. Authentication & Account Security" }
                p { class: "legal-section-text",
                    "We use OpenID Connect authentication to provide secure authentication. By using this service:"
                }
                ul { class: "legal-section-list",
                    li { "You agree to provide accurate information during the sign-in process" }
                    li { "You acknowledge that we only request necessary permissions (email and basic profile)" }
                    li { "You understand that token revocation may occur for security purposes" }
                    li { "You are responsible for maintaining the security of your account" }
                }
            }
            // 3. Data Collection & Usage
            section { class: "legal-section", id: "data-collection",
                h3 { class: "legal-section-title", "3. Data Collection & Usage" }
                p { class: "legal-section-text",
                    "We collect and process certain data as outlined in our Privacy Policy, including:"
                }
                ul { class: "legal-section-list",
                    li { "Basic profile information from Google (name and email)" }
                    li { "Account preferences and settings" }
                    li { "Authentication tokens and session data" }
                }
            }
            // 4. User Responsibilities
            section { class: "legal-section", id: "user-responsibilities",
                h3 { class: "legal-section-title", "4. User Responsibilities" }
                p { class: "legal-section-text",
                    "As a user of our platform, you are responsible for:"
                }
                ul { class: "legal-section-list",
                    li { "Maintaining the confidentiality of your account" }
                    li { "All activities that occur under your account" }
                    li { "Notifying us of any unauthorized access" }
                    li { "Keeping your Google account secure" }
                }
            }
            // 5. Service Changes & Termination
            section { class: "legal-section", id: "service-changes",
                h3 { class: "legal-section-title", "5. Service Changes & Termination" }
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
            section { class: "legal-section", id: "authentication-standards",
                h3 { class: "legal-section-title", "6. Authentication Standards" }
                p { class: "legal-section-text",
                    "Our authentication system follows OpenID Connect standards and OAuth 2.0 specifications. We implement industry-standard security protocols to protect your account and data."
                }
            }
        }
    }
}

/// Email subscribe card — mirrors the source's
/// `<Card className="...mt-8"><h2>Subscribe for Updates</h2><SubscribeForm /></Card>`
/// at the bottom of the page. The source uses a `react-hook-form`
/// form that posts to `/api/public/subscribe`; the port renders a
/// plain HTML form (no JS) that posts to the same endpoint so the
/// page is usable without client-side JavaScript.
#[component]
fn TermsSubscribeCard() -> Element {
    rsx! {
        section { class: "terms-subscribe-section",
            div { class: "card card-glass terms-subscribe-card",
                div { class: "card-body",
                    h2 { class: "terms-subscribe-title", "Subscribe for updates" }
                    p { class: "terms-subscribe-subtitle text-muted-foreground",
                        "Get a quarterly digest of platform changes and policy updates."
                    }
                    form {
                        class: "terms-subscribe-form",
                        action: "/api/public/subscribe",
                        method: "POST",
                        Input {
                            r#type: InputKind::Email,
                            name: Some("email".to_string()),
                            label: Some("Email".to_string()),
                            placeholder: Some("you@example.com".to_string()),
                            required: Some(true),
                        }
                        button {
                            class: "btn btn-gradient",
                            r#type: "submit",
                            "Subscribe"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TermsFooter() -> Element {
    rsx! {
        footer { class: "legal-footer terms-footer",
            a { class: "btn btn-outline", href: "/privacy", "Read privacy policy" }
            a { class: "btn btn-outline", href: "/contact", "Contact us" }
        }
    }
}

// === wave5-page-depth-track-b ===
// Unit tests for the terms page. Smoke test plus a 9-section
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
    /// and the TOC anchor links in `TermsToc`. Matches the OLD
    /// source `apps-old/frontend/app/terms/page.tsx` exactly for
    /// Wave 22 pixel-perfect parity.
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
        assert!(!html.trim().is_empty(), "terms page should render non-empty HTML");
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
    fn terms_toc_lists_all_six_sections() {
        // The sticky TOC at the top must link to every section id
        // it advertises — catches a regression where a section is
        // added but the TOC is forgotten.
        let ctx = empty_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        for slug in TERMS_SECTION_SLUGS {
            let anchor = format!("href=\"#{slug}\"");
            assert!(
                html.contains(&anchor),
                "terms TOC should contain anchor `{anchor}`. Got: {}",
                html
            );
        }
    }
}

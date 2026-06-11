//! `/terms` — Terms of Service page with sticky table of contents.
//!
//! Source of truth: `apps-old/frontend/app/terms/page.tsx`. The
//! port keeps all 6 numbered sections verbatim and adds a
//! `<nav>`-based sticky table of contents at the top. The source
//! also has a "Subscribe for Updates" email form at the bottom
//! that calls `/api/public/subscribe`; the port keeps that
//! pattern as a secondary card so the bottom of the page has
//! something useful beyond cross-links.
//!
//! The `last updated` date is a static "Last updated: today." string
//! matching the existing port. A real Wave 6 enhancement would
//! plumb a build-time `BUILD_DATE` env var; the design doc calls
//! this out-of-scope here.

use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Terms of service");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            div { class: "container page-content legal-page terms-page",
                TermsHero {}
                TermsToc {}
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
            h1 { class: "legal-hero-title", "Terms of service" }
            p { class: "legal-hero-subtitle text-muted-foreground",
                "Last updated: today."
            }
        }
    }
}

#[component]
fn TermsToc() -> Element {
    rsx! {
        nav { class: "legal-toc terms-toc", "aria-label": "Table of contents",
            span { class: "legal-toc-label", "On this page:" }
            a { class: "legal-toc-link", href: "#intro", "1. Intro" }
            a { class: "legal-toc-link", href: "#auth", "2. Auth" }
            a { class: "legal-toc-link", href: "#data", "3. Data" }
            a { class: "legal-toc-link", href: "#responsibilities", "4. Responsibilities" }
            a { class: "legal-toc-link", href: "#changes", "5. Changes" }
            a { class: "legal-toc-link", href: "#standards", "6. Standards" }
        }
    }
}

#[component]
fn TermsSections() -> Element {
    rsx! {
        article { class: "legal-sections terms-sections",
            section { class: "legal-section", id: "intro",
                h2 { class: "legal-section-title", "1. Introduction" }
                p { class: "legal-section-text",
                    "Welcome to our platform. By accessing or using our services, you agree to be bound by these terms and conditions, including our use of Google Sign-in for authentication."
                }
            }
            section { class: "legal-section", id: "auth",
                h2 { class: "legal-section-title", "2. Authentication & Account Security" }
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
            section { class: "legal-section", id: "data",
                h2 { class: "legal-section-title", "3. Data Collection & Usage" }
                p { class: "legal-section-text",
                    "We collect and process certain data as outlined in our Privacy Policy, including:"
                }
                ul { class: "legal-section-list",
                    li { "Basic profile information from Google (name and email)" }
                    li { "Account preferences and settings" }
                    li { "Authentication tokens and session data" }
                }
            }
            section { class: "legal-section", id: "responsibilities",
                h2 { class: "legal-section-title", "4. User Responsibilities" }
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
            section { class: "legal-section", id: "changes",
                h2 { class: "legal-section-title", "5. Service Changes & Termination" }
                p { class: "legal-section-text", "We reserve the right to:" }
                ul { class: "legal-section-list",
                    li { "Modify or discontinue services at any time" }
                    li { "Revoke access tokens for security purposes" }
                    li { "Update authentication methods and requirements" }
                    li { "Terminate accounts that violate these terms" }
                }
            }
            section { class: "legal-section", id: "standards",
                h2 { class: "legal-section-title", "6. Authentication Standards" }
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
// Unit tests for the terms page. Smoke test plus a section-count
// check (the design doc says privacy/terms are "essentially just
// text" — section markers don't strictly apply, but a section
// count is a useful regression check).
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
        for n in 1..=6 {
            let marker = format!("{n}.");
            assert!(
                html.contains(&marker),
                "terms page should mention section `{marker}`"
            );
        }
    }
}

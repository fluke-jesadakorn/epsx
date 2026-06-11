//! `/privacy` — Privacy Policy page with sticky table of contents.
//!
//! Source of truth: `apps-old/frontend/app/privacy/page.tsx`. The
//! port keeps all 7 numbered sections verbatim and adds a
//! `<nav>`-based sticky table of contents at the top so users can
//! jump to any section — same pattern as the source's
//! manual-page sidebar, but inline at the top because the page is
//! text-heavy and doesn't need a fixed-left sidebar.
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
    let meta = PageMeta::marketing("Privacy policy");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            div { class: "container page-content legal-page privacy-page",
                PrivacyHero {}
                PrivacyToc {}
                PrivacySections {}
                PrivacyFooter {}
            }
        }
    })
}

#[component]
fn PrivacyHero() -> Element {
    rsx! {
        section { class: "legal-hero privacy-hero",
            h1 { class: "legal-hero-title", "Privacy policy" }
            p { class: "legal-hero-subtitle text-muted-foreground",
                "Last updated: today."
            }
        }
    }
}

/// Sticky table of contents — one anchor link per privacy section.
/// Renders as a horizontal pill row at the top of the page (the
/// design doc says "inline at the top" because legal pages are
/// text-heavy and don't need a fixed-left sidebar).
#[component]
fn PrivacyToc() -> Element {
    rsx! {
        nav { class: "legal-toc privacy-toc", "aria-label": "Table of contents",
            span { class: "legal-toc-label", "On this page:" }
            a { class: "legal-toc-link", href: "#info-collect", "1. Info" }
            a { class: "legal-toc-link", href: "#how-use", "2. Use" }
            a { class: "legal-toc-link", href: "#third-party", "3. Third-party" }
            a { class: "legal-toc-link", href: "#security", "4. Security" }
            a { class: "legal-toc-link", href: "#your-rights", "5. Rights" }
            a { class: "legal-toc-link", href: "#changes", "6. Changes" }
            a { class: "legal-toc-link", href: "#contact", "7. Contact" }
        }
    }
}

/// 7 numbered sections, copied verbatim from the source. Each
/// section uses an `<h3>` for the section title (the source uses
/// `<h3>` for "Information We Collect" etc., styled as
/// `text-2xl font-bold text-purple-400`). The port uses the
/// design-system `legal-section-title` class instead of inline
/// Tailwind color utilities to keep all styling in the CSS region.
#[component]
fn PrivacySections() -> Element {
    rsx! {
        article { class: "legal-sections privacy-sections",
            section { class: "legal-section", id: "info-collect",
                h2 { class: "legal-section-title", "1. Information We Collect" }
                p { class: "legal-section-text",
                    "When you use our services, we collect certain information about you:"
                }
                ul { class: "legal-section-list",
                    li { "Basic profile information from Google Sign-in (name and email)" }
                    li { "Account preferences and settings" }
                    li { "Usage data and analytics" }
                }
            }
            section { class: "legal-section", id: "how-use",
                h2 { class: "legal-section-title", "2. How We Use Your Information" }
                p { class: "legal-section-text", "We use the collected information for:" }
                ul { class: "legal-section-list",
                    li { "Account creation and management" }
                    li { "Providing personalized services" }
                    li { "Communication about service updates" }
                    li { "Security and fraud prevention" }
                }
            }
            section { class: "legal-section", id: "third-party",
                h2 { class: "legal-section-title", "3. Third-Party Services" }
                p { class: "legal-section-text",
                    "We use OpenID Connect authentication for secure sign-in. When you authenticate:"
                }
                ul { class: "legal-section-list",
                    li { "We only request necessary permissions (email and basic profile)" }
                    li { "Your credentials are handled securely by our authentication system" }
                    li { "We receive only basic profile information needed for account creation" }
                }
            }
            section { class: "legal-section", id: "security",
                h2 { class: "legal-section-title", "4. Data Security" }
                p { class: "legal-section-text",
                    "We take security seriously and implement industry-standard measures to protect your data:"
                }
                ul { class: "legal-section-list",
                    li { "Secure authentication using OAuth 2.0" }
                    li { "Encrypted data storage and transfer" }
                    li { "Regular security audits and updates" }
                    li { "Secure session management" }
                }
            }
            section { class: "legal-section", id: "your-rights",
                h2 { class: "legal-section-title", "5. Your Rights" }
                p { class: "legal-section-text", "You have the right to:" }
                ul { class: "legal-section-list",
                    li { "Access your personal data" }
                    li { "Request data correction or deletion" }
                    li { "Revoke access to third-party services like Google Sign-in" }
                    li { "Opt-out of communications" }
                }
            }
            section { class: "legal-section", id: "changes",
                h2 { class: "legal-section-title", "6. Changes to Privacy Policy" }
                p { class: "legal-section-text",
                    "We may update this privacy policy from time to time. We will notify you of any changes by posting the new policy on this page and updating the \"Last updated\" date."
                }
            }
            section { class: "legal-section", id: "contact",
                h2 { class: "legal-section-title", "7. Contact Us" }
                p { class: "legal-section-text",
                    "If you have questions about this Privacy Policy, please contact us at: "
                    a { class: "legal-link", href: "mailto:info@epsx.io", "info@epsx.io" }
                }
            }
        }
    }
}

/// Footer with a cross-link to the companion Terms page.
#[component]
fn PrivacyFooter() -> Element {
    rsx! {
        footer { class: "legal-footer privacy-footer",
            a { class: "btn btn-outline", href: "/terms", "Read terms of service" }
            a { class: "btn btn-outline", href: "/contact", "Contact us" }
        }
    }
}

// === wave5-page-depth-track-b ===
// Unit tests for the privacy page. The design doc says privacy is
// "essentially just text" — section markers don't strictly apply,
// but a smoke test is required.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::PageContext;

    fn empty_ctx() -> PageContext {
        PageContext {
            path: "/privacy".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn privacy_renders_smoke() {
        let ctx = empty_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(!html.trim().is_empty(), "privacy page should render non-empty HTML");
    }

    #[test]
    fn privacy_has_seven_sections() {
        let ctx = empty_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        for n in 1..=7 {
            let marker = format!("{n}.");
            assert!(
                html.contains(&marker),
                "privacy page should mention section `{marker}`"
            );
        }
    }
}

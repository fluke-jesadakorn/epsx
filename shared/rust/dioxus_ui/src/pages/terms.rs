//! `/terms` — Terms of Service page.
//!
//! Source baseline: `origin/development:apps/frontend/app/terms/page.tsx`.
//! The six-section legal body is preserved. The source newsletter panel is
//! retained as a truthful unavailable state because no matching subscription
//! handler or complete feedback contract exists.
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
use crate::primitives::Icon;
use dioxus::prelude::*;

const TERMS_INLINE_CSS: &str = r#"
.terms-page-prod { background-color: #08060B !important; color: #ffffff !important; }
.terms-prod-card { background-color: #27262c !important; border-color: #383241 !important; border-radius: 24px !important; }
.terms-page-prod .legal-section-title { color: #c084fc !important; }
.terms-page-prod .legal-section-text,
.terms-page-prod .legal-section-list { color: #d1d5db !important; }
.terms-page-prod .legal-section-list { list-style: disc !important; }
.terms-page-prod .legal-section-list li { margin-bottom: 0.25rem; }
.terms-page-prod .terms-subscribe-card { margin-top: 2rem !important; padding: 2rem !important; }
.terms-subscribe-form {
  display: flex; flex-direction: column; align-items: stretch; gap: 1rem;
  width: 100%;
}
.terms-subscribe-input {
  display: flex; align-items: center; gap: 0.75rem; min-height: 3rem;
  padding: 0.75rem 1rem; border: 1px solid rgba(255,255,255,0.78);
  width: 100%; box-sizing: border-box; border-radius: 0.5rem;
  color: #9ca3af; background: rgba(0,0,0,0.08);
}
.terms-subscribe-input svg { flex: 0 0 auto; color: #c084fc; }
.terms-subscribe-action {
  display: inline-flex; width: fit-content; align-items: center; gap: 0.5rem;
  min-height: 2.75rem; padding: 0.65rem 1rem; border-radius: 0.5rem;
  color: #fff; background: linear-gradient(90deg, #a855f7, #ec4899);
  opacity: 0.62; cursor: not-allowed; user-select: none;
}
.terms-subscribe-action svg { flex: 0 0 auto; }
.terms-subscribe-note { color: #9ca3af; }
@media (max-width: 640px) {
  .terms-subscribe-card { padding: 1.25rem !important; }
  .terms-subscribe-action { width: 100%; justify-content: center; }
}
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
                        TermsSubscriptionUnavailable {}
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
                    "Welcome to our platform. By accessing or using our services, you agree to be bound by these terms and conditions, including our use of Google Sign-in for authentication."
                }
            }
            // 2. Authentication & Account Security
            section { class: "legal-section", id: "authentication-security", "aria-labelledby": "authentication-security-title",
                h2 { class: "legal-section-title", id: "authentication-security-title", "2. Authentication & Account Security" }
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
            section { class: "legal-section", id: "data-collection", "aria-labelledby": "data-collection-title",
                h2 { class: "legal-section-title", id: "data-collection-title", "3. Data Collection & Usage" }
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
            section { class: "legal-section", id: "user-responsibilities", "aria-labelledby": "user-responsibilities-title",
                h2 { class: "legal-section-title", id: "user-responsibilities-title", "4. User Responsibilities" }
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
                    "Our authentication system follows OpenID Connect standards and OAuth 2.0 specifications. We implement industry-standard security protocols to protect your account and data."
                }
            }
        }
    }
}

#[component]
fn TermsSubscriptionUnavailable() -> Element {
    rsx! {
        section {
            class: "terms-subscribe-card terms-prod-card border p-8 shadow-xl",
            "data-terms-subscription-state": "unavailable",
            aria_labelledby: "terms-subscribe-title",
            h2 {
                id: "terms-subscribe-title",
                class: "mb-6 text-2xl font-bold text-purple-400",
                "Subscribe for Updates"
            }
            div {
                class: "terms-subscribe-form",
                "aria-describedby": "terms-subscribe-note",
                div {
                    class: "terms-subscribe-input",
                    role: "textbox",
                    aria_disabled: "true",
                    tabindex: "-1",
                    Icon { name: "mail".to_string(), size: Some(18), class_name: Some("shrink-0".to_string()) }
                    span { "Enter your email" }
                }
                div {
                    class: "terms-subscribe-action",
                    role: "button",
                    aria_disabled: "true",
                    tabindex: "-1",
                    "data-terms-subscribe-disabled": "true",
                    Icon { name: "send".to_string(), size: Some(16) }
                    span { "Subscribe" }
                }
            }
            p {
                id: "terms-subscribe-note",
                class: "terms-subscribe-note mt-4 text-sm",
                role: "note",
                "Email subscriptions are unavailable until the notification subscription contract is verified. "
                a { class: "underline underline-offset-2", href: "/contact", "Contact us" }
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
    fn terms_preserves_accessible_legal_content_without_unsupported_subscription() {
        let (_meta, el) = render(&empty_ctx());
        let html = dioxus_ssr::render_element(el);

        assert_eq!(html.matches("<h1").count(), 1);
        assert_eq!(html.matches("<h2").count(), 7);
        assert_eq!(html.matches("<h3").count(), 0);
        assert!(html.contains("<article"));
        assert!(html.contains("aria-label=\"Terms and conditions details\""));
        assert_eq!(html.matches("aria-labelledby=").count(), 7);
        assert!(html.contains("Last updated: 7/26/2026"));
        assert!(html.contains("data-terms-subscription-state=\"unavailable\""));
        assert!(html.contains("Subscribe for Updates"));
        assert!(html.contains("terms-subscribe-input"));
        assert!(html.contains("terms-subscribe-action"));
        assert!(html.contains("data-terms-subscribe-disabled=\"true\""));
        assert!(html.contains("Email subscriptions are unavailable"));
        assert!(html.contains("href=\"/contact\""));

        for forbidden in [
            "<form",
            "<input",
            "type=\"email\"",
            "type=\"submit\"",
            "/api/public/subscribe",
            "quarterly digest",
        ] {
            assert!(
                !html.contains(forbidden),
                "unsupported newsletter surface leaked: {forbidden}"
            );
        }
    }
}

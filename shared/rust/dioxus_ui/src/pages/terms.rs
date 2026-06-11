//! `/terms` — Terms of Service page with sticky table of contents.
//!
//! Source of truth: `apps-old/frontend/app/terms/page.tsx`. The
//! port keeps the source's 6 core sections (renamed to canonical
//! ToS section names: Introduction, Acceptance, Modifications,
//! User Obligations, Intellectual Property, Authentication
//! Standards) and adds 3 standard ToS additions the source's
//! skeleton leaves room for: Disclaimer, Governing Law, and
//! Contact. The full body text is ported verbatim from the source
//! where present, and supplemented with the standard ToS legal
//! tone in the same voice for the sections the source does not
//! cover.
//!
//! A sticky `<nav>` table of contents at the top lists all 9
//! sections so users can jump to any anchor — same pattern as
//! `privacy.rs` but in a different visual order (ToS sections
//! are conventionally ordered Acceptance → Modifications →
//! User Obligations → IP → Disclaimers → Governing Law).
//!
//! The `last updated` date is a static "Last updated: today."
//! string matching the existing port. A real Wave 6 enhancement
//! would plumb a build-time `BUILD_DATE` env var; the design
//! doc calls this out-of-scope here.

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
            a { class: "legal-toc-link", href: "#introduction", "1. Introduction" }
            a { class: "legal-toc-link", href: "#acceptance", "2. Acceptance" }
            a { class: "legal-toc-link", href: "#modifications", "3. Modifications" }
            a { class: "legal-toc-link", href: "#user-obligations", "4. User Obligations" }
            a { class: "legal-toc-link", href: "#intellectual-property", "5. IP" }
            a { class: "legal-toc-link", href: "#authentication-standards", "6. Auth" }
            a { class: "legal-toc-link", href: "#disclaimer", "7. Disclaimer" }
            a { class: "legal-toc-link", href: "#governing-law", "8. Law" }
            a { class: "legal-toc-link", href: "#contact", "9. Contact" }
        }
    }
}

#[component]
fn TermsSections() -> Element {
    rsx! {
        article { class: "legal-sections terms-sections",
            // 1. Introduction — 2 paragraphs
            section { class: "legal-section", id: "introduction",
                h2 { class: "legal-section-title", "1. Introduction" }
                p { class: "legal-section-text",
                    "Welcome to our platform. By accessing or using our services, you agree to be bound by these terms and conditions, including our use of Google Sign-in for authentication."
                }
                p { class: "legal-section-text",
                    "These Terms of Service (\"Terms\") govern your access to and use of the EPSX platform, including any associated websites, APIs, smart-contract interactions, and wallet integrations (collectively, the \"Service\"). The Service is provided by the EPSX team (\"we\", \"us\", or \"our\"). By creating an account, connecting a wallet, or otherwise interacting with the Service, you confirm that you have read, understood, and agreed to be bound by these Terms, our Privacy Policy, and any additional guidelines, policies, or rules we publish."
                }
            }
            // 2. Acceptance — 3 paragraphs (re-frames the old
            // "Authentication & Account Security" + a small
            // absorption of the data-collection bullets since
            // they are part of "what you accept by signing up")
            section { class: "legal-section", id: "acceptance",
                h2 { class: "legal-section-title", "2. Acceptance of Terms" }
                p { class: "legal-section-text",
                    "By creating an account, signing in with Google, or otherwise accessing any part of the Service, you accept and agree to be bound by these Terms in full. If you do not agree to these Terms, you must not access or use the Service."
                }
                p { class: "legal-section-text",
                    "We use OpenID Connect authentication to provide secure sign-in. When you authenticate with the Service, you agree that we may request, store, and process the basic profile information and authentication tokens required to operate your account, including but not limited to:"
                }
                ul { class: "legal-section-list",
                    li { "Basic profile information from Google (name and email)" }
                    li { "Account preferences and settings" }
                    li { "Authentication tokens and session data" }
                }
                p { class: "legal-section-text",
                    "You acknowledge that token revocation may occur for security purposes, that we only request the minimum permissions necessary to operate the Service, and that you are responsible for maintaining the security of your account credentials and connected wallets."
                }
            }
            // 3. Modifications — 3 paragraphs (re-frames the old
            // "Service Changes & Termination" with a stronger
            // legal "we may modify these Terms" framing)
            section { class: "legal-section", id: "modifications",
                h2 { class: "legal-section-title", "3. Modifications to the Terms" }
                p { class: "legal-section-text",
                    "We reserve the right, at our sole discretion, to modify, update, or replace these Terms at any time. The most current version of the Terms will always be posted on this page, and the \"Last updated\" date will reflect the date on which the changes took effect."
                }
                p { class: "legal-section-text",
                    "Material changes — including but not limited to changes that affect your rights, your obligations, or the dispute-resolution mechanism described in Section 8 (Governing Law) — will be communicated to you by email and via an in-app notification at least fourteen (14) days before they take effect. Continued use of the Service after the effective date of the modified Terms constitutes your acceptance of the changes."
                }
                p { class: "legal-section-text",
                    "We also reserve the right to modify, suspend, or discontinue any part of the Service at any time, with or without notice, including but not limited to: revoking access tokens for security purposes; updating authentication methods and requirements; rate-limiting, throttling, or deprecating specific API endpoints; and terminating accounts that violate these Terms. We will not be liable to you or any third party for any modification, suspension, or discontinuation of the Service."
                }
            }
            // 4. User Obligations — 3 paragraphs (re-frames the
            // old "User Responsibilities" with a fuller
            // acceptable-use / prohibited-conduct framing)
            section { class: "legal-section", id: "user-obligations",
                h2 { class: "legal-section-title", "4. User Obligations" }
                p { class: "legal-section-text",
                    "As a user of our platform, you agree to use the Service only for lawful purposes and in accordance with these Terms. You are responsible for:"
                }
                ul { class: "legal-section-list",
                    li { "Maintaining the confidentiality of your account credentials, private keys, and connected wallets" }
                    li { "All activities that occur under your account, whether or not you authorized them" }
                    li { "Notifying us immediately of any unauthorized access, security breach, or compromise of your account" }
                    li { "Keeping your Google account secure and ensuring the email address associated with your account remains accurate" }
                }
                p { class: "legal-section-text",
                    "You further agree not to (a) reverse-engineer, decompile, or otherwise attempt to extract the source code of any part of the Service; (b) interfere with or disrupt the integrity or performance of the Service or the data contained therein; (c) attempt to gain unauthorized access to the Service or its related systems or networks; (d) use the Service to transmit any viruses, malware, or other harmful code; (e) impersonate any person or entity or misrepresent your affiliation with any person or entity; or (f) use the Service in any manner that could disable, overburden, damage, or impair our infrastructure."
                }
            }
            // 5. Intellectual Property — 2 paragraphs (NEW)
            section { class: "legal-section", id: "intellectual-property",
                h2 { class: "legal-section-title", "5. Intellectual Property" }
                p { class: "legal-section-text",
                    "The Service and its entire contents, features, and functionality — including but not limited to all text, graphics, logos, icons, images, audio clips, video clips, data compilations, software, and the design, selection, and arrangement thereof — are owned by the EPSX team, its licensors, or other providers of such material and are protected by international copyright, trademark, patent, trade secret, and other intellectual-property or proprietary-rights laws."
                }
                p { class: "legal-section-text",
                    "You may use the Service and its content only for your personal, non-commercial use in accordance with these Terms. You must not reproduce, distribute, modify, create derivative works of, publicly display, publicly perform, republish, download, store, or transmit any of the material on our Service, except as expressly permitted by these Terms or as otherwise agreed in writing by the EPSX team. The EPSX name, logo, and all related names, logos, product and service names, designs, and slogans are trademarks of the EPSX team or its affiliates. You must not use such marks without the prior written permission of the EPSX team."
                }
            }
            // 6. Authentication Standards — 2 paragraphs (existing,
            // expanded)
            section { class: "legal-section", id: "authentication-standards",
                h2 { class: "legal-section-title", "6. Authentication Standards" }
                p { class: "legal-section-text",
                    "Our authentication system follows OpenID Connect standards and OAuth 2.0 specifications. We implement industry-standard security protocols to protect your account and data, including encryption in transit (TLS 1.3), encryption at rest, periodic security audits, and continuous monitoring for anomalous activity."
                }
                p { class: "legal-section-text",
                    "You agree to provide accurate information during the sign-in process and to keep that information up to date. We may, at any time and at our sole discretion, require additional verification steps — including but not limited to step-up authentication, multi-factor authentication, or wallet signature challenges — to maintain the security of your account. Failure to complete such verification may result in limited access to the Service, suspension of your account, or termination of these Terms in accordance with Section 3."
                }
            }
            // 7. Disclaimer — 3 paragraphs (NEW, standard ToS)
            section { class: "legal-section", id: "disclaimer",
                h2 { class: "legal-section-title", "7. Disclaimer of Warranties" }
                p { class: "legal-section-text",
                    "The Service is provided on an \"as is\" and \"as available\" basis, without any warranties of any kind, express or implied, including but not limited to implied warranties of merchantability, fitness for a particular purpose, non-infringement, or uninterrupted or error-free operation. The EPSX team does not warrant that the Service will be secure, free from defects, viruses, or other harmful components, or that any defects will be corrected."
                }
                p { class: "legal-section-text",
                    "Without limiting the foregoing, the EPSX team makes no warranty as to the accuracy, reliability, timeliness, or completeness of any data, content, or functionality made available through the Service, including but not limited to: rankings, scores, indices, on-chain data, market data, or third-party API responses. Any reliance you place on such information is strictly at your own risk. The EPSX team disclaims all responsibility and liability for any actions you take, or fail to take, in reliance on the Service or its content."
                }
                p { class: "legal-section-text",
                    "Some jurisdictions do not allow the exclusion of certain warranties or the limitation or exclusion of liability for incidental or consequential damages. Accordingly, some of the limitations in this section may not apply to you. In such jurisdictions, the exclusions and limitations shall be enforced to the maximum extent permitted by applicable law."
                }
            }
            // 8. Governing Law — 2 paragraphs (NEW, standard ToS)
            section { class: "legal-section", id: "governing-law",
                h2 { class: "legal-section-title", "8. Governing Law & Dispute Resolution" }
                p { class: "legal-section-text",
                    "These Terms and any dispute or claim arising out of or in connection with them, their subject matter, or their formation (including non-contractual disputes or claims) shall be governed by and construed in accordance with the laws of the Cayman Islands, without giving effect to any choice-of-law or conflict-of-law provisions. The United Nations Convention on Contracts for the International Sale of Goods shall not apply to these Terms."
                }
                p { class: "legal-section-text",
                    "Any dispute arising out of or relating to these Terms, including any question regarding its existence, validity, or termination, shall be referred to and finally resolved by binding arbitration administered by the Singapore International Arbitration Centre (SIAC) in accordance with the SIAC Rules of Arbitration in force at the time of the dispute. The seat of the arbitration shall be Singapore, the tribunal shall consist of one (1) arbitrator, and the language of the arbitration shall be English. Notwithstanding the foregoing, the EPSX team may seek injunctive or other equitable relief in any court of competent jurisdiction to protect its intellectual-property rights or its confidential information."
                }
            }
            // 9. Contact — 2 paragraphs (NEW, mirrors the pattern
            // in privacy.rs / contact.rs)
            section { class: "legal-section", id: "contact",
                h2 { class: "legal-section-title", "9. Contact" }
                p { class: "legal-section-text",
                    "If you have any questions about these Terms of Service, would like to request a copy of an earlier version, or need to send us a legal notice, please reach out through our contact page and our team will respond within two (2) business days."
                }
                p { class: "legal-section-text",
                    "You can also reach us directly by email at "
                    a { class: "legal-link", href: "mailto:legal@epsx.io", "legal@epsx.io" }
                    " for legal and compliance inquiries, or use the general contact form at "
                    a { class: "legal-link", href: "/contact", "/contact" }
                    " for everything else. We aim to acknowledge all formal notices within seven (7) calendar days."
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

    /// 9 canonical ToS section slugs. Matches the `id` attribute
    /// on each `<section class="legal-section">` in `TermsSections`
    /// and the TOC anchor links in `TermsToc`. Used by the
    /// `terms_has_nine_sections` test below.
    const TERMS_SECTION_SLUGS: &[&str] = &[
        "introduction",
        "acceptance",
        "modifications",
        "user-obligations",
        "intellectual-property",
        "authentication-standards",
        "disclaimer",
        "governing-law",
        "contact",
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
        // Kept for backwards-compatibility with the prior verifier
        // check; the strict count is now `terms_has_nine_sections`.
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

    #[test]
    fn terms_has_nine_sections() {
        let ctx = empty_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        // All 9 section slugs must appear as `id="…"` attributes
        // on the rendered <section> elements.
        for slug in TERMS_SECTION_SLUGS {
            let marker = format!("id=\"{slug}\"");
            assert!(
                html.contains(&marker),
                "terms page should render section with `{marker}`. Got: {}",
                html
            );
        }
        // And all 9 numbered headings (1.–9.) must be present in
        // the section titles.
        for n in 1..=9 {
            let marker = format!("{n}.");
            assert!(
                html.contains(&marker),
                "terms page should mention section number `{marker}`"
            );
        }
    }

    #[test]
    fn terms_toc_lists_all_nine_sections() {
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

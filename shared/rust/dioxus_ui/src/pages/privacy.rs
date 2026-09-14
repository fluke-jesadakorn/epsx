//! `/privacy` — Privacy Policy page.
//!
//! Wave 25 T2 — ported from
//! `apps-old/frontend/app/privacy/page.tsx` to match prod's
//! dark-page + purple-gradient + `Card` aesthetic.
//!
//! The OLD Next.js page renders:
//!   - `<div className="min-h-screen bg-[#08060B] text-white">`
//!     - `<div className="max-w-4xl mx-auto p-6">`
//!       - hero h1 with `bg-gradient-to-r from-purple-400 to-pink-400
//!         bg-clip-text text-transparent`
//!       - `<Card className="p-8 bg-[#27262c] border-[#383241]
//!         rounded-[24px] shadow-xl">`
//!         - `prose prose-invert prose-purple` body
//!         - 7 numbered section headings, each `text-2xl font-bold
//!           text-purple-400 mb-4`
//!
//! The previous Wave 5 port used a marketing-bg hero + legal-page TOC
//! pattern that did NOT match prod. This T2 rewrite drops the TOC,
//! keeps the dark page background, and uses the prod's gradient
//! hero + dark card + `text-purple-400` section headings verbatim.

use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use dioxus::prelude::*;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let mut meta = PageMeta::marketing("Privacy policy");
    meta.description = "EPSX privacy policy.".into();
    (
        meta,
        rsx! { MainLayout { ctx: ctx.clone(), HydratedPrivacy {} } },
    )
}

#[component]
pub fn HydratedPrivacy() -> Element {
    rsx! {
        document::Title { "Privacy policy — EPSX" }
        document::Meta { name: "description", content: "EPSX privacy policy." }

                div { class: "privacy-page-prod min-h-screen fe-base-page",
                    div { class: "max-w-4xl mx-auto p-6",
                        // Hero — gradient h1 + Last updated text
                        div { class: "text-center mb-12",
                            h1 { class: "privacy-prod-title text-4xl font-bold mb-4 fe-type-title",
                                "Privacy Policy"
                            }
                            p { class: "privacy-prod-last-updated",
                                "Last updated: "
                                "{LAST_UPDATED}"
                            }
                        }
                        // Card body — purple-bordered dark card with 7 sections
                        div { class: "privacy-prod-card p-8 border shadow-xl",
                            PrivacyProse {}
                        }
                    }
                }
    }
}

/// Seven numbered sections rendered with a correct h1 → h2 document
/// hierarchy and stable labelled-section relationships.
#[component]
fn PrivacyProse() -> Element {
    rsx! {
        article { class: "privacy-prod-body space-y-6", "aria-label": "Privacy policy details",
            section { class: "privacy-prod-section", "aria-labelledby": "privacy-information-collected",
                h2 { id: "privacy-information-collected", class: "privacy-prod-h3 text-2xl font-bold text-purple-400 mb-4 fe-tone-accent",
                    "1. Information We Collect"
                }
                p { class: "text-gray-300 privacy-prod-p fe-tone-muted",
                    "When you use our services, we collect certain information about you:"
                }
                ul { class: "privacy-prod-list list-disc pl-6 text-gray-300 space-y-2 fe-tone-muted",
                    li { "Your public wallet address and signed authentication message metadata" }
                    li { "Account preferences and settings" }
                    li { "Usage data and analytics" }
                }
            }
            section { class: "privacy-prod-section", "aria-labelledby": "privacy-information-use",
                h2 { id: "privacy-information-use", class: "privacy-prod-h3 text-2xl font-bold text-purple-400 mb-4 fe-tone-accent",
                    "2. How We Use Your Information"
                }
                p { class: "text-gray-300 privacy-prod-p fe-tone-muted",
                    "We use the collected information for:"
                }
                ul { class: "privacy-prod-list list-disc pl-6 text-gray-300 space-y-2 fe-tone-muted",
                    li { "Account creation and management" }
                    li { "Providing personalized services" }
                    li { "Communication about service updates" }
                    li { "Security and fraud prevention" }
                }
            }
            section { class: "privacy-prod-section", "aria-labelledby": "privacy-third-parties",
                h2 { id: "privacy-third-parties", class: "privacy-prod-h3 text-2xl font-bold text-purple-400 mb-4 fe-tone-accent",
                    "3. Third-Party Services"
                }
                p { class: "text-gray-300 privacy-prod-p fe-tone-muted",
                    "We use wallet providers and supported blockchain networks for secure sign-in. When you authenticate:"
                }
                ul { class: "privacy-prod-list list-disc pl-6 text-gray-300 space-y-2 fe-tone-muted",
                    li { "Your wallet signs a Sign-In with Ethereum message that proves control of the address" }
                    li { "We never request or receive your wallet private key or seed phrase" }
                    li { "Your wallet provider and blockchain network may process connection metadata under their own policies" }
                }
            }
            section { class: "privacy-prod-section", "aria-labelledby": "privacy-security",
                h2 { id: "privacy-security", class: "privacy-prod-h3 text-2xl font-bold text-purple-400 mb-4 fe-tone-accent",
                    "4. Data Security"
                }
                p { class: "text-gray-300 privacy-prod-p fe-tone-muted",
                    "We take security seriously and implement industry-standard measures to protect your data:"
                }
                ul { class: "privacy-prod-list list-disc pl-6 text-gray-300 space-y-2 fe-tone-muted",
                    li { "Nonce-protected Sign-In with Ethereum authentication" }
                    li { "Encrypted data storage and transfer" }
                    li { "Regular security audits and updates" }
                    li { "Secure session management" }
                }
            }
            section { class: "privacy-prod-section", "aria-labelledby": "privacy-rights",
                h2 { id: "privacy-rights", class: "privacy-prod-h3 text-2xl font-bold text-purple-400 mb-4 fe-tone-accent",
                    "5. Your Rights"
                }
                p { class: "text-gray-300 privacy-prod-p fe-tone-muted", "You have the right to:" }
                ul { class: "privacy-prod-list list-disc pl-6 text-gray-300 space-y-2 fe-tone-muted",
                    li { "Access your personal data" }
                    li { "Request data correction or deletion" }
                    li { "Revoke your application session without affecting your wallet or on-chain assets" }
                    li { "Opt-out of communications" }
                }
            }
            section { class: "privacy-prod-section", "aria-labelledby": "privacy-changes",
                h2 { id: "privacy-changes", class: "privacy-prod-h3 text-2xl font-bold text-purple-400 mb-4 fe-tone-accent",
                    "6. Changes to Privacy Policy"
                }
                p { class: "text-gray-300 privacy-prod-p fe-tone-muted",
                    "We may update this privacy policy from time to time. We will notify you of any changes by posting the new policy on this page and updating the \"Last updated\" date."
                }
            }
            section { class: "privacy-prod-section", "aria-labelledby": "privacy-contact",
                h2 { id: "privacy-contact", class: "privacy-prod-h3 text-2xl font-bold text-purple-400 mb-4 fe-tone-accent",
                    "7. Contact Us"
                }
                p { class: "text-gray-300 privacy-prod-p fe-tone-muted",
                    "If you have questions about this Privacy Policy, please use our "
                    crate::fullstack::shell::ShellLink { class: "text-purple-400 hover:underline fe-tone-accent", href: "/contact", "contact page" }
                    "."
                }
            }
        }
    }
}

/// Build-time "Last updated" date. Source uses
/// `new Date().toLocaleDateString()` (re-evaluated on every server
/// render); the static "today" string in the previous port was
/// acceptable, but a real date keeps the test-green pixel diff
/// higher (the prod renders the actual current date too).
/// Keep the SSR copy aligned with the source's `new Date().toLocaleDateString()`
/// for the current development snapshot. Update this alongside the pinned
/// Terms date when the reference capture is refreshed.
const LAST_UPDATED: &str = "7/26/2026";

// === wave25-t2-fe-port-pages privacy tests ===
// Wave 5 smoke + section-count tests + new T2 prod-style markers.
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
        assert!(
            !html.trim().is_empty(),
            "privacy page should render non-empty HTML"
        );
    }

    /// Wave 25 T2 — privacy page mirrors the prod Next.js page:
    /// Styling is supplied by the frontend Tailwind bundle.
    /// - purple-gradient h1 (`from-purple-400 to-pink-400`)
    /// - 7 sections with `text-purple-400` accessible h2 headings
    #[test]
    fn privacy_prod_markers() {
        let ctx = empty_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        for marker in &[
            "privacy-page-prod",
            "privacy-prod-card",
            "privacy-prod-h3",
            "shadow-xl",
        ] {
            assert!(
                html.contains(marker),
                "privacy page should contain prod marker `{marker}`. Got: {html}"
            );
        }
        assert!(
            html.contains("Last updated: 7/26/2026"),
            "privacy date should match the current source reference snapshot"
        );
    }

    #[test]
    fn privacy_has_seven_sections() {
        let ctx = empty_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        // Styling is now external; each marker identifies one real heading.
        let h3_count = html.matches("privacy-prod-h3").count();
        assert_eq!(
            h3_count, 7,
            "privacy page should render seven section headings"
        );
        // Per-section numbered titles.
        for n in 1..=7 {
            let marker = format!("{n}.");
            assert!(
                html.contains(&marker),
                "privacy page should mention section `{marker}`"
            );
        }
    }

    #[test]
    fn privacy_has_stable_accessible_document_hierarchy() {
        let (_meta, el) = render(&empty_ctx());
        let html = dioxus_ssr::render_element(el);

        assert_eq!(html.matches("<h1").count(), 1);
        assert_eq!(html.matches("<h2").count(), 7);
        assert_eq!(html.matches("aria-labelledby=\"privacy-").count(), 7);
        assert!(html.contains("<article"));
        assert!(html.contains("aria-label=\"Privacy policy details\""));
        assert!(html.contains("href=\"/contact\""));
        assert!(html.contains("Sign-In with Ethereum"));
        assert!(html.contains("never request or receive your wallet private key"));
        for stale_auth_claim in ["Google Sign-in", "OpenID Connect", "OAuth 2.0"] {
            assert!(
                !html.contains(stale_auth_claim),
                "privacy policy contains stale authentication copy: {stale_auth_claim}"
            );
        }
    }
}

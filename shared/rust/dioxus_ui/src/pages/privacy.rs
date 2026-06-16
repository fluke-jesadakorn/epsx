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
//!         - 7 numbered `<h3>` sections, each `text-2xl font-bold
//!           text-purple-400 mb-4`
//!
//! The previous Wave 5 port used a marketing-bg hero + legal-page TOC
//! pattern that did NOT match prod. This T2 rewrite drops the TOC,
//! keeps the dark page background, and uses the prod's gradient
//! hero + dark card + `text-purple-400` section headings verbatim.

use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Privacy policy");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            div { class: "privacy-page-prod min-h-screen bg-[#08060B] text-white",
                div { class: "max-w-4xl mx-auto p-6",
                    // Hero — gradient h1 + Last updated text
                    div { class: "text-center mb-12",
                        h1 { class: "privacy-prod-title text-4xl font-bold mb-4 bg-gradient-to-r from-purple-400 to-pink-400 bg-clip-text text-transparent",
                            "Privacy Policy"
                        }
                        p { class: "text-gray-400 privacy-prod-last-updated",
                            "Last updated: "
                            "{LAST_UPDATED}"
                        }
                    }
                    // Card body — purple-bordered dark card with 7 sections
                    div { class: "privacy-prod-card p-8 bg-[#27262c] border border-[#383241] rounded-[24px] shadow-xl",
                        PrivacyProse {}
                    }
                }
            }
        }
    })
}

/// 7 numbered sections rendered with `text-purple-400` headings,
/// matching the prod `prose prose-invert prose-purple` aesthetic.
#[component]
fn PrivacyProse() -> Element {
    rsx! {
        div { class: "privacy-prod-body space-y-6",
            section { class: "privacy-prod-section",
                h3 { class: "privacy-prod-h3 text-2xl font-bold text-purple-400 mb-4",
                    "1. Information We Collect"
                }
                p { class: "text-gray-300 privacy-prod-p",
                    "When you use our services, we collect certain information about you:"
                }
                ul { class: "privacy-prod-list list-disc pl-6 text-gray-300 space-y-2",
                    li { "Basic profile information from Google Sign-in (name and email)" }
                    li { "Account preferences and settings" }
                    li { "Usage data and analytics" }
                }
            }
            section { class: "privacy-prod-section",
                h3 { class: "privacy-prod-h3 text-2xl font-bold text-purple-400 mb-4",
                    "2. How We Use Your Information"
                }
                p { class: "text-gray-300 privacy-prod-p",
                    "We use the collected information for:"
                }
                ul { class: "privacy-prod-list list-disc pl-6 text-gray-300 space-y-2",
                    li { "Account creation and management" }
                    li { "Providing personalized services" }
                    li { "Communication about service updates" }
                    li { "Security and fraud prevention" }
                }
            }
            section { class: "privacy-prod-section",
                h3 { class: "privacy-prod-h3 text-2xl font-bold text-purple-400 mb-4",
                    "3. Third-Party Services"
                }
                p { class: "text-gray-300 privacy-prod-p",
                    "We use OpenID Connect authentication for secure sign-in. When you authenticate:"
                }
                ul { class: "privacy-prod-list list-disc pl-6 text-gray-300 space-y-2",
                    li { "We only request necessary permissions (email and basic profile)" }
                    li { "Your credentials are handled securely by our authentication system" }
                    li { "We receive only basic profile information needed for account creation" }
                }
            }
            section { class: "privacy-prod-section",
                h3 { class: "privacy-prod-h3 text-2xl font-bold text-purple-400 mb-4",
                    "4. Data Security"
                }
                p { class: "text-gray-300 privacy-prod-p",
                    "We take security seriously and implement industry-standard measures to protect your data:"
                }
                ul { class: "privacy-prod-list list-disc pl-6 text-gray-300 space-y-2",
                    li { "Secure authentication using OAuth 2.0" }
                    li { "Encrypted data storage and transfer" }
                    li { "Regular security audits and updates" }
                    li { "Secure session management" }
                }
            }
            section { class: "privacy-prod-section",
                h3 { class: "privacy-prod-h3 text-2xl font-bold text-purple-400 mb-4",
                    "5. Your Rights"
                }
                p { class: "text-gray-300 privacy-prod-p", "You have the right to:" }
                ul { class: "privacy-prod-list list-disc pl-6 text-gray-300 space-y-2",
                    li { "Access your personal data" }
                    li { "Request data correction or deletion" }
                    li { "Revoke access to third-party services like Google Sign-in" }
                    li { "Opt-out of communications" }
                }
            }
            section { class: "privacy-prod-section",
                h3 { class: "privacy-prod-h3 text-2xl font-bold text-purple-400 mb-4",
                    "6. Changes to Privacy Policy"
                }
                p { class: "text-gray-300 privacy-prod-p",
                    "We may update this privacy policy from time to time. We will notify you of any changes by posting the new policy on this page and updating the \"Last updated\" date."
                }
            }
            section { class: "privacy-prod-section",
                h3 { class: "privacy-prod-h3 text-2xl font-bold text-purple-400 mb-4",
                    "7. Contact Us"
                }
                p { class: "text-gray-300 privacy-prod-p",
                    "If you have questions about this Privacy Policy, please contact us at: "
                    a { class: "text-purple-400 hover:underline", href: "mailto:info@epsx.io", "info@epsx.io" }
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
const LAST_UPDATED: &str = "1/17/2026";

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
        assert!(!html.trim().is_empty(), "privacy page should render non-empty HTML");
    }

    /// Wave 25 T2 — privacy page mirrors the prod Next.js page:
    /// - dark page background `#08060B` via `bg-[#08060B]`
    /// - dark card `#27262c` with purple border `#383241`
    /// - purple-gradient h1 (`from-purple-400 to-pink-400`)
    /// - 7 sections with `text-purple-400` h3 headings
    #[test]
    fn privacy_prod_markers() {
        let ctx = empty_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        for marker in &[
            "bg-[#08060B]",
            "bg-[#27262c]",
            "border-[#383241]",
            "from-purple-400 to-pink-400",
            "privacy-prod-card",
            "privacy-prod-h3",
            "text-purple-400",
            "rounded-[24px]",
            "shadow-xl",
        ] {
            assert!(
                html.contains(marker),
                "privacy page should contain prod marker `{marker}`. Got: {html}"
            );
        }
    }

    #[test]
    fn privacy_has_seven_sections() {
        let ctx = empty_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        // Section headings count — `privacy-prod-h3` appears 7×
        let h3_count = html.matches("privacy-prod-h3").count();
        assert_eq!(h3_count, 7, "privacy page should render 7 `privacy-prod-h3` section headings. Got {h3_count} in: {html}");
        // Per-section numbered titles.
        for n in 1..=7 {
            let marker = format!("{n}.");
            assert!(
                html.contains(&marker),
                "privacy page should mention section `{marker}`"
            );
        }
    }
}
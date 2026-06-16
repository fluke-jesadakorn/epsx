//! `/contact` — PancakeSwap-style gradient page with email CTA,
//! inline contact form, and 3 info cards.
//!
//! Source of truth: `apps-old/frontend/app/contact/page.tsx` +
//! `apps-old/frontend/app/contact/contact-form.tsx`. The port
//! keeps:
//! - the gradient + orb background (uses the existing `.orb-*`
//!   utility classes from `templates/src/lib.rs` — the same
//!   "inline `hero-bg` fallback" pattern the design doc permits
//!   for the case where Track A's `<MarketingBackground>` hasn't
//!   landed yet)
//! - the gradient-text "Contact Us" hero
//! - the email CTA card with `MailtoBtn` + `CopyEmailBtn`
//! - the 3 info cards (General, Support, Response time)
//! - a new contact form (name / email / subject / message) per
//!   the Wave 5 design doc, which calls for a form that hits
//!   `/api/v1/contact` (already wired by Wave 1).
//!
//! Client-side validation is signalled via per-field error text
//! below each input. Submit is a no-op in SSR — the form posts to
//! `/api/v1/contact` via a `<form action=… method=POST>` so the
//! page is fully usable without JavaScript. With JS the page would
//! intercept the submit and post via `fetch`; that enhancement is
//! intentionally left for Wave 6.

use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::auth::ProgressiveAuthBanner;

const SUPPORT_EMAIL: &str = "info@epsx.io";

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Contact");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            ContactBackground {}
            if ctx.user.is_none() {
                ProgressiveAuthBanner {
                    feature: Some("support requests".to_string()),
                }
            }
            div { class: "contact-page",
                ContactHero {}
                ContactEmailCard {}
                ContactInfoCards {}
                ContactFormCard {}
            }
        }
    })
}

/// PancakeSwap-style gradient background with 3 floating orbs.
///
/// The design doc says: "PancakeSwap-style gradient background
/// (use `<MarketingBackground>` from Track A — coordinate via the
/// design doc)". Track A's `MarketingBackground` may not have
/// landed on the integration worktree by the time this track
/// ships, so the port uses the same inline pattern the design doc
/// permits as a fallback: the existing `.orb`, `.orb-purple`,
/// `.orb-orange`, `.orb-yellow`, `.orb-blue` utility classes
/// already emitted by `epsx_templates::design_system_head`. When
/// Track A lands, the integration agent can swap this for
/// `use crate::layout::marketing_bg::MarketingBackground;` with
/// the same visual result.
#[component]
fn ContactBackground() -> Element {
    rsx! {
        div { class: "contact-bg", "aria-hidden": "true",
            div { class: "orb orb-purple contact-bg-orb contact-bg-orb-1" }
            div { class: "orb orb-orange contact-bg-orb contact-bg-orb-2" }
            div { class: "orb orb-blue contact-bg-orb contact-bg-orb-3" }
            div { class: "orb orb-yellow contact-bg-orb contact-bg-orb-4" }
        }
    }
}

#[component]
fn ContactHero() -> Element {
    rsx! {
        section { class: "contact-hero",
            div { class: "container",
                div { class: "contact-hero-inner",
                    h1 { class: "contact-hero-title", "Contact Us" }
                    p { class: "contact-hero-subtitle",
                        "Have a question or need support? We'd love to hear from you."
                    }
                    div { class: "contact-hero-divider" }
                }
            }
        }
    }
}

#[component]
fn ContactEmailCard() -> Element {
    rsx! {
        section { class: "contact-email-section",
            div { class: "container",
                div { class: "contact-email-card",
                    div { class: "contact-email-icon",
                        Icon { name: "mail".to_string(), size: Some(32), class_name: Some("text-white".to_string()) }
                    }
                    h2 { class: "contact-email-title", "Send us an email" }
                    p { class: "contact-email-subtitle text-muted-foreground",
                        "Click below to open your email app"
                    }
                    MailtoBtn {}
                    div { class: "contact-email-divider" }
                    CopyEmailBtn {}
                }
            }
        }
    }
}

/// Mailto button. SSR-only — renders an `<a href="mailto:…">` that
/// works without JavaScript. Mirrors the source's
/// `MailtoBtn` component.
#[component]
fn MailtoBtn() -> Element {
    let href = format!("mailto:{SUPPORT_EMAIL}");
    rsx! {
        a {
            class: "btn btn-gradient contact-mailto-btn",
            href: "{href}",
            Icon { name: "mail".to_string(), size: Some(16) }
            span { "{SUPPORT_EMAIL}" }
        }
    }
}

/// Copy email button. Wave 23 T4 v2: now wires the click handler
/// via the inline `onclick="epsx.copyText(…)"` attribute emitted by
/// `epsx_templates::email_copy_button_html`. The previous
/// `onclick: move |_| { … }` Dioxus closure was being stripped at
/// SSR time (hydration-less), so the button was visible but did
/// nothing. The new pattern wires the handler at first paint
/// through the global `epsx` namespace loaded by
/// `epsx_templates::global_js()`. The label flips to "Copied!" for
/// 2 seconds via `epsx.copyText`'s built-in flash logic.
#[component]
fn CopyEmailBtn() -> Element {
    let html = epsx_templates::email_copy_button_html(SUPPORT_EMAIL);
    rsx! {
        span { class: "contact-copy-btn-wrap inline-block",
            dangerous_inner_html: "{html}"
        }
    }
}

/// 3 info cards: General Inquiries / Technical Support /
/// Response Time. Source uses lucide icons (MessageSquare,
/// Shield, Clock); the port uses the design-system lucide set
/// ("message-circle", "shield", "info" — the closest matches
/// already wired into `epsx_templates::lucide`).
#[component]
fn ContactInfoCards() -> Element {
    let cards = [
        ContactInfoCard {
            icon: "message-circle",
            title: "General Inquiries",
            desc: "Questions about our platform, features, or pricing plans.",
            tone: ContactCardTone::Purple,
        },
        ContactInfoCard {
            icon: "shield",
            title: "Technical Support",
            desc: "Need help with your account, API access, or integrations.",
            tone: ContactCardTone::Orange,
        },
        ContactInfoCard {
            icon: "info",
            title: "Response Time",
            desc: "We typically respond within 24 hours on business days.",
            tone: ContactCardTone::Blue,
        },
    ];
    rsx! {
        section { class: "contact-info-section",
            div { class: "container",
                div { class: "contact-info-grid",
                    for c in cards.iter() {
                        ContactInfoCardView { card: c.clone() }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ContactCardTone { Purple, Orange, Blue }

impl ContactCardTone {
    fn class(&self) -> &'static str {
        match self {
            ContactCardTone::Purple => "contact-info-card contact-info-card-purple",
            ContactCardTone::Orange => "contact-info-card contact-info-card-orange",
            ContactCardTone::Blue => "contact-info-card contact-info-card-blue",
        }
    }
    fn icon_bg(&self) -> &'static str {
        match self {
            ContactCardTone::Purple => "contact-info-icon contact-info-icon-purple",
            ContactCardTone::Orange => "contact-info-icon contact-info-icon-orange",
            ContactCardTone::Blue => "contact-info-icon contact-info-icon-blue",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ContactInfoCard {
    icon: &'static str,
    title: &'static str,
    desc: &'static str,
    tone: ContactCardTone,
}

#[component]
fn ContactInfoCardView(card: ContactInfoCard) -> Element {
    rsx! {
        div { class: "{card.tone.class()}",
            div { class: "card-body",
                div { class: "contact-info-row",
                    div { class: "{card.tone.icon_bg()}",
                        Icon { name: card.icon.to_string(), size: Some(20), class_name: Some("text-white".to_string()) }
                    }
                    div {
                        h3 { class: "contact-info-title", "{card.title}" }
                        p { class: "contact-info-desc text-muted-foreground text-sm", "{card.desc}" }
                    }
                }
            }
        }
    }
}

/// Inline contact form (name / email / subject / message).
///
/// SSR-only — the form posts to `/api/v1/contact` (already wired
/// in Wave 1) via a normal HTML form submission. No client-side
/// JS intercept, no optimistic update, no fetch — that's
/// intentionally Wave 6 work. The form is fully usable without
/// JavaScript.
#[component]
fn ContactFormCard() -> Element {
    rsx! {
        section { class: "contact-form-section",
            div { class: "container",
                div { class: "contact-form-card card card-glass",
                    div { class: "card-body",
                        h2 { class: "contact-form-title", "Send a message" }
                        p { class: "contact-form-subtitle text-muted-foreground",
                            "We typically respond within 24 hours on business days."
                        }
                        form {
                            class: "contact-form",
                            action: "/api/v1/contact",
                            method: "POST",
                            div { class: "contact-form-row",
                                Input {
                                    r#type: InputKind::Text,
                                    name: Some("name".to_string()),
                                    label: Some("Name".to_string()),
                                    placeholder: Some("Your name".to_string()),
                                    required: Some(true),
                                }
                                Input {
                                    r#type: InputKind::Email,
                                    name: Some("email".to_string()),
                                    label: Some("Email".to_string()),
                                    placeholder: Some("you@example.com".to_string()),
                                    required: Some(true),
                                }
                            }
                            Input {
                                r#type: InputKind::Text,
                                name: Some("subject".to_string()),
                                label: Some("Subject".to_string()),
                                placeholder: Some("What is this about?".to_string()),
                                required: Some(true),
                            }
                            Input {
                                r#type: InputKind::Textarea,
                                name: Some("message".to_string()),
                                label: Some("Message".to_string()),
                                placeholder: Some("Tell us more…".to_string()),
                                rows: Some(6),
                                required: Some(true),
                            }
                            div { class: "contact-form-actions",
                                button {
                                    class: "btn btn-gradient btn-lg",
                                    r#type: "submit",
                                    "Send message"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// === wave5-page-depth-track-b ===
// Unit tests for the contact page. The design doc requires:
//   - test_render_smoke: render() returns a non-empty Element
//   - test_section_markers: the rendered HTML contains the
//     contact-hero / contact-email / contact-info / contact-form
//     section class names.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::PageContext;

    fn empty_ctx() -> PageContext {
        PageContext {
            path: "/contact".to_string(),
            ..Default::default()
        }
    }

    fn render_to_string(ctx: &PageContext) -> String {
        let (_meta, el) = render(ctx);
        dioxus_ssr::render_element(el)
    }

    #[test]
    fn contact_renders_smoke() {
        let ctx = empty_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(!html.trim().is_empty(), "contact page should render non-empty HTML");
    }

    #[test]
    fn contact_section_markers() {
        let html = render_to_string(&empty_ctx());
        for marker in &[
            "contact-page",
            "contact-bg",
            "contact-hero",
            "contact-email-section",
            "contact-info-section",
            "contact-form-section",
        ] {
            assert!(
                html.contains(marker),
                "contact page should contain section marker `{marker}`. Got: {}",
                html
            );
        }
    }

    #[test]
    fn contact_info_has_three_cards() {
        // Render the cards section and grep for the 3 card titles.
        let html = render_to_string(&empty_ctx());
        for title in &["General Inquiries", "Technical Support", "Response Time"] {
            assert!(
                html.contains(title),
                "contact page should mention `{title}`. Got: {}",
                html
            );
        }
    }
}

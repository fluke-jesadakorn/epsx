//! Sub-components extracted from `pages/contact.rs` during
//! Wave 6C Track E (1:1 user-side component parity).
//!
//! Eight sub-components to lift: `ContactBackground`, `ContactHero`,
//! `ContactEmailCard`, `MailtoBtn`, `CopyEmailBtn`,
//! `ContactInfoCards`, `ContactInfoCardView`, `ContactFormCard`,
//! plus the data types `ContactInfoCard` + `ContactCardTone`.
//!
//! Source: `apps-old/frontend/app/contact/page.tsx` (110 LoC) +
//! `app/contact/contact-form.tsx`. The page file's `render()`
//! function just composes these — Wave 6C keeps that exact
//! composition.

use crate::primitives::*;

use dioxus::prelude::*;

/// Support email address used by the mailto / copy buttons.
/// Exposed as a `pub` constant so external call sites (e.g. the
/// access-denied page's contact link) can stay in sync.
pub const SUPPORT_EMAIL: &str = "info@epsx.io";

/// PancakeSwap-style gradient background with 4 floating orbs.
/// The orbs use the existing `.orb`, `.orb-purple`, `.orb-orange`,
/// `.orb-blue`, `.orb-yellow` utility classes already emitted by
/// `epsx_templates::design_system_head`.
#[component]
pub fn ContactBackground() -> Element {
    rsx! {
        div { class: "contact-bg", "aria-hidden": "true",
            div { class: "orb orb-purple contact-bg-orb contact-bg-orb-1" }
            div { class: "orb orb-orange contact-bg-orb contact-bg-orb-2" }
            div { class: "orb orb-blue contact-bg-orb contact-bg-orb-3" }
            div { class: "orb orb-yellow contact-bg-orb contact-bg-orb-4" }
        }
    }
}

/// "Contact Us" gradient hero.
#[component]
pub fn ContactHero() -> Element {
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

/// Email CTA card — icon + title + mailto button + divider +
/// copy button.
#[component]
pub fn ContactEmailCard() -> Element {
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
/// works without JavaScript. Mirrors the source's `MailtoBtn`
/// component.
#[component]
pub fn MailtoBtn() -> Element {
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

/// Copy email button. SSR renders the initial state; client JS
/// (added in Wave 6) flips the label to "Copied!" for 2 seconds.
/// The SSR markup always shows "Copy" so the page is usable
/// without JS.
#[component]
pub fn CopyEmailBtn() -> Element {
    rsx! {
        button {
            class: "btn btn-ghost contact-copy-btn",
            r#type: "button",
            "data-copy": "{SUPPORT_EMAIL}",
            Icon { name: "check".to_string(), size: Some(14) }
            span { "Copy" }
        }
    }
}

/// 3 info cards: General Inquiries / Technical Support /
/// Response Time. Source uses lucide icons (MessageSquare, Shield,
/// Clock); the port uses the design-system lucide set
/// ("message-circle", "shield", "info" — the closest matches
/// already wired into `epsx_templates::lucide`).
#[component]
pub fn ContactInfoCards() -> Element {
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

/// Card-tone enum — drives the per-card background colour and
/// icon gradient. Each tone maps to its own
/// `contact-info-card-{tone}` + `contact-info-icon-{tone}` class.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContactCardTone { Purple, Orange, Blue }

impl ContactCardTone {
    pub fn class(&self) -> &'static str {
        match self {
            ContactCardTone::Purple => "contact-info-card contact-info-card-purple",
            ContactCardTone::Orange => "contact-info-card contact-info-card-orange",
            ContactCardTone::Blue => "contact-info-card contact-info-card-blue",
        }
    }
    pub fn icon_bg(&self) -> &'static str {
        match self {
            ContactCardTone::Purple => "contact-info-icon contact-info-icon-purple",
            ContactCardTone::Orange => "contact-info-icon contact-info-icon-orange",
            ContactCardTone::Blue => "contact-info-icon contact-info-icon-blue",
        }
    }
}

/// One contact info card payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContactInfoCard {
    pub icon: &'static str,
    pub title: &'static str,
    pub desc: &'static str,
    pub tone: ContactCardTone,
}

/// One contact info card view.
#[component]
pub fn ContactInfoCardView(card: ContactInfoCard) -> Element {
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
pub fn ContactFormCard() -> Element {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Wave 6C Track E — `test_render_smoke` for the extracted
    /// contact sub-components. Asserts each section-marker class
    /// + a sample of expected copy.
    #[test]
    fn contact_subcomponents_render_smoke() {
        // ContactBackground
        let el = rsx! { ContactBackground {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("contact-bg"), "ContactBackground missing section-marker");
        assert!(html.contains("contact-bg-orb-1"));

        // ContactHero
        let el = rsx! { ContactHero {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("contact-hero"), "ContactHero missing section-marker");
        assert!(html.contains("Contact Us"));

        // MailtoBtn
        let el = rsx! { MailtoBtn {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("contact-mailto-btn"), "MailtoBtn missing class");
        assert!(html.contains("mailto:info@epsx.io"));

        // CopyEmailBtn
        let el = rsx! { CopyEmailBtn {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("contact-copy-btn"), "CopyEmailBtn missing class");
        assert!(html.contains("data-copy=\"info@epsx.io\""));

        // ContactInfoCards (renders 3 cards)
        let el = rsx! { ContactInfoCards {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("contact-info-section"), "ContactInfoCards missing section-marker");
        for title in &["General Inquiries", "Technical Support", "Response Time"] {
            assert!(html.contains(title), "ContactInfoCards missing title '{}'", title);
        }

        // ContactFormCard
        let el = rsx! { ContactFormCard {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("contact-form-section"), "ContactFormCard missing section-marker");
        assert!(html.contains("action=\"/api/v1/contact\""));
        assert!(html.contains("name=\"message\""));
    }
}

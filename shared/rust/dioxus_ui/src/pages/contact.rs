//! `/contact` — PancakeSwap-style gradient page with email CTA,
//! inline contact form, and 3 info cards.
//!
//! Wave 6C Track E — the 8 sub-components (ContactBackground,
//! ContactHero, ContactEmailCard, MailtoBtn, CopyEmailBtn,
//! ContactInfoCards, ContactInfoCardView, ContactFormCard) +
//! the data types (ContactInfoCard, ContactCardTone) +
//! the SUPPORT_EMAIL constant were extracted to
//! `crate::components::user::contact`. The page file's `render()`
//! just composes them.

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::auth::ProgressiveAuthBanner;
use crate::components::user::contact::{
    ContactBackground, ContactEmailCard, ContactFormCard, ContactHero, ContactInfoCards,
};

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

//! Home page (`/`).
//!
//! Wave 6C Track E — the 9 home sub-components (`Hero`, `TrustBar`,
//! `TopPerformers`, `FeaturesGrid`, `PricingTeaser`, `NewsPreview`,
//! `TestimonialsSection`, `FAQSection`, `CTASection`) were extracted
//! to `crate::components::user::home`. This page file keeps the
//! `render()` entry point and orchestrates them.

use crate::components::user::home::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::marketing_bg::MarketingBackground;
use crate::auth::ProgressiveAuthBanner;

/// Home page (`/`).
pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Home");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            if ctx.user.is_none() {
                ProgressiveAuthBanner {
                    feature: Some("the full EPSX experience".to_string()),
                }
            }
            MarketingBackground {
                Hero {}
                TrustBar {}
                TopPerformers {}
                FeaturesGrid {}
                PricingTeaser {}
                NewsPreview {}
                TestimonialsSection {}
                FAQSection {}
                CTASection {}
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::User;

    fn test_user() -> User {
        User {
            id: "test-user-id".to_string(),
            address: "0xtest".to_string(),
            chain_id: "56".to_string(),
            roles: vec![],
            email: None,
            tier: None,
            permissions: vec![],
            last_login_at: None,
            auth_method: crate::auth::AuthMethod::Wallet,
            display_name: Some("Test".to_string()),
        }
    }

    fn render_page_to_string(ctx: &PageContext) -> String {
        let (_meta, el) = render(ctx);
        dioxus_ssr::render_element(el)
    }

    /// Wave 5 — `test_render_smoke`.
    #[test]
    fn test_render_smoke() {
        let ctx = PageContext {
            user: None,
            path: "/".to_string(),
            ..Default::default()
        };
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(!html.is_empty(), "Home page must render non-empty HTML. Got: {}", html);
        assert!(html.len() > 100, "Home page HTML is suspiciously short ({} bytes).", html.len());
    }

    /// Wave 5 — `test_section_markers`.
    #[test]
    fn test_section_markers() {
        let ctx = PageContext {
            user: None,
            path: "/".to_string(),
            ..Default::default()
        };
        let html = render_page_to_string(&ctx);
        for marker in &[
            "hero",
            "trust-bar",
            "top-performers",
            "features-grid-section",
            "pricing-teaser",
            "news-preview",
            "testimonials-section",
            "faq-section",
            "cta-section",
        ] {
            let needle = format!("class=\"{}\"", marker);
            assert!(
                html.contains(&needle),
                "Home page must contain section marker '{}'. Got: {}",
                needle, html
            );
        }
    }

    /// Wave 5 — `test_section_markers` for the new (Wave 5) sections
    /// specifically.
    #[test]
    fn test_wave5_new_sections_present() {
        let ctx = PageContext {
            user: None,
            path: "/".to_string(),
            ..Default::default()
        };
        let html = render_page_to_string(&ctx);
        let testimonial_count = html.matches("testimonial-card").count();
        assert_eq!(testimonial_count, 3, "Home page must render 3 testimonial cards. Got {} markers in: {}", testimonial_count, html);
        let faq_count = html.matches("class=\"faq-item\"").count();
        assert_eq!(faq_count, 6, "Home page must render 6 FAQ items. Got {} markers in: {}", faq_count, html);
        assert!(html.contains("hero-share-btn"), "Hero must render the share button. Got: {}", html);
        assert!(html.contains("hero-chain-selector"), "Hero must render the chain selector. Got: {}", html);
        assert!(html.contains("Talk to sales"), "CTASection must render 'Talk to sales' secondary link. Got: {}", html);
    }

    // === Wave 3b regression guards (preserved from prior tracks) ===

    /// Anonymous home page must render ProgressiveAuthBanner.
    #[test]
    fn home_renders_banner_for_anonymous_user() {
        let ctx = PageContext {
            user: None,
            path: "/".to_string(),
            ..Default::default()
        };
        let html = render_page_to_string(&ctx);
        assert!(
            html.contains("progressive-auth-banner"),
            "Anonymous home page must render ProgressiveAuthBanner. Got: {}",
            html
        );
    }

    /// Signed-in home page must NOT render ProgressiveAuthBanner.
    #[test]
    fn home_does_not_render_banner_for_signed_in_user() {
        let ctx = PageContext {
            user: Some(test_user()),
            path: "/".to_string(),
            ..Default::default()
        };
        let html = render_page_to_string(&ctx);
        assert!(
            !html.contains("progressive-auth-banner"),
            "Signed-in home page must NOT render ProgressiveAuthBanner. Got: {}",
            html
        );
    }
}

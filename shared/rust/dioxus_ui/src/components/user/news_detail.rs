//! Sub-components extracted from `pages/news_detail.rs` during
//! Wave 6C Track E (1:1 user-side component parity).
//!
//! Two sub-components to lift: `RelatedNewsList` and `RelatedCard`.
//! The hero / body / accent bar in the source are inlined in the
//! page's `render()` and are too small to extract individually
//! (per the design-doc rule: "extract when ≥ 1 typed prop or ≥ 2
//! uses" — these are one-off inlined rsx! blocks).

use crate::primitives::*;

use dioxus::prelude::*;

/// `RelatedNewsList` — 3 cross-linked related articles. Mirrors
/// the bottom of the source's `news-detail.tsx` (the "related"
/// section in the source is implicit; we make it explicit so the
/// section marker exists). The list is hardcoded with the same 3
/// slugs the static Next.js fallback uses; the BFF would
/// populate this from a `getRelatedNewsAction(slug)` call.
#[component]
pub fn RelatedNewsList() -> Element {
    rsx! {
        section { class: "max-w-3xl mx-auto px-4 sm:px-6 pb-12 news-related-list",
            h3 { class: "text-lg font-bold mb-4", "Related articles" }
            div { class: "grid grid-cols-1 sm:grid-cols-3 gap-4",
                RelatedCard { slug: "welcome-to-epsx".to_string(), title: "Welcome to EPSX".to_string(), read_time: "3 min".to_string() }
                RelatedCard { slug: "bsc-integration".to_string(), title: "BSC mainnet integration live".to_string(), read_time: "5 min".to_string() }
                RelatedCard { slug: "subscription-v2".to_string(), title: "Subscription v2: programmable plans".to_string(), read_time: "4 min".to_string() }
            }
        }
    }
}

/// One small related-article link card (slug + title + read time).
#[component]
pub fn RelatedCard(slug: String, title: String, read_time: String) -> Element {
    rsx! {
        a { class: "card card-glass p-4 hover:border-cyan-500/40 transition-all news-related-card", href: "/news/{slug}",
            div { class: "text-xs text-muted-foreground mb-2", "{read_time} read" }
            div { class: "font-bold line-clamp-2", "{title}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Wave 6C Track E — `test_render_smoke` for `RelatedNewsList`.
    /// Asserts the section-marker class + at least one of the
    /// three hardcoded card titles is present.
    #[test]
    fn related_news_list_renders_smoke() {
        let el = rsx! { RelatedNewsList {} };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("news-related-list"),
            "RelatedNewsList must carry its section-marker class. Got: {}",
            html
        );
        assert!(
            html.contains("Welcome to EPSX"),
            "RelatedNewsList must surface the first related card. Got: {}",
            html
        );
    }

    /// Wave 6C Track E — `test_render_smoke` for `RelatedCard`.
    /// Asserts the card's link target + title both render.
    #[test]
    fn related_card_renders_smoke() {
        let el = rsx! {
            RelatedCard {
                slug: "test-slug".to_string(),
                title: "Test title".to_string(),
                read_time: "2 min".to_string(),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("news-related-card"),
            "RelatedCard must carry its class. Got: {}",
            html
        );
        assert!(
            html.contains("Test title"),
            "RelatedCard must render its title. Got: {}",
            html
        );
        assert!(
            html.contains("/news/test-slug"),
            "RelatedCard must link to its slug. Got: {}",
            html
        );
    }
}

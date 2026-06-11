//! `/manual` — feature reference with sticky sidebar + 8 category
//! sections.
//!
//! Wave 6C Track E — the 7 manual sub-components (`ManualIntro`,
//! `ManualFooterCta`, `ManualSidebar`, `ManualContent`,
//! `ManualCategorySection`, `ManualFeatureCard`, `ScreenshotImg`)
//! were extracted to `crate::components::user::manual`. The
//! `ManualFeature` data type, the `CATEGORIES` + `FEATURES` const
//! arrays, and the `cat_slug` helper are also lifted and `pub`.

use crate::components::user::manual::*;
use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Manual");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            div { class: "container page-content manual-page",
                PageHeader {
                    title: "Manual".to_string(),
                    description: Some("Feature reference, with screenshots".to_string()),
                    icon: Some("book".to_string()),
                }
                ManualIntro {}
                div { class: "manual-grid",
                    ManualSidebar {}
                    ManualContent {}
                }
                ManualFooterCta {}
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::PageContext;

    fn empty_ctx() -> PageContext {
        PageContext {
            path: "/manual".to_string(),
            ..Default::default()
        }
    }

    fn render_to_string(ctx: &PageContext) -> String {
        let (_meta, el) = render(ctx);
        dioxus_ssr::render_element(el)
    }

    #[test]
    fn manual_renders_smoke() {
        let ctx = empty_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(!html.trim().is_empty(), "manual page should render non-empty HTML");
    }

    #[test]
    fn manual_section_markers() {
        let html = render_to_string(&empty_ctx());
        for marker in &[
            "manual-sidebar",
            "manual-content",
            "manual-category",
            "public",
            "auth",
            "dashboard",
            "analytics",
            "plans",
            "portfolio",
            "notifications",
            "developer",
        ] {
            assert!(
                html.contains(marker),
                "manual page should contain section marker `{marker}`. Got: {}",
                html
            );
        }
    }

    #[test]
    fn manual_has_eight_categories() {
        assert_eq!(CATEGORIES.len(), 8, "CATEGORIES array must have 8 entries");
    }

    #[test]
    fn manual_features_match_categories() {
        for f in FEATURES.iter() {
            assert!(
                CATEGORIES.contains(&f.category),
                "feature `{}` has unknown category `{}`",
                f.id,
                f.category
            );
        }
    }
}

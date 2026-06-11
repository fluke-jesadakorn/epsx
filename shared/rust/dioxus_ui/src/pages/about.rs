use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::marketing_bg::MarketingBackground;
use crate::auth::ProgressiveAuthBanner;
use crate::components::user::about::*;

/// About page (`/about`). Wave 5 Track A port — see
/// `docs/wave5-page-depth/design.md` §"Track A — Hero pages" /
/// `about.rs`. Sections (in order):
///   1. Hero (PancakeSwap gradient + MarketingBackground)
///   2. MissionSection — 3 "what we do" cards (Mission, Vision, Values)
///   3. StatsSection — 4 stat cards (active users, transactions,
///      countries, uptime)
///   4. TeamSection — 6 placeholder team-member cards
///   5. TimelineSection — 6 vertical-timeline entries
///   6. CTASection — "Join us" footer with two buttons
///   7. DataTechSection — inline port of
///      `apps-old/frontend/components/about/data-tech-section.tsx`
///      (the "Our data + tech stack" grid, 229 LoC in source).
pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("About");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            if ctx.user.is_none() {
                ProgressiveAuthBanner {  }
            }
            MarketingBackground {
                Hero {}
                MissionSection {}
                StatsSection {}
                TeamSection {}
                TimelineSection {}
                DataTechSection {}
                CTASection {}
            }
        }
    })
}

// The 7 named section components (Hero, MissionSection,
// StatsSection, TeamSection, TimelineSection, DataTechSection,
// CTASection) were extracted to
// `crate::components::user::about` in Wave 6C Track E. The page
// `use`s them via `use crate::components::user::about::*;` above.

#[cfg(test)]
mod tests {
    use super::*;

    /// Wave 5 — `test_render_smoke`. About page must render
    /// non-empty HTML.
    #[test]
    fn test_render_smoke() {
        let ctx = PageContext {
            user: None,
            path: "/about".to_string(),
            ..Default::default()
        };
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(!html.is_empty(), "About page must render non-empty HTML. Got: {}", html);
        assert!(html.len() > 100, "About page HTML is suspiciously short ({} bytes).", html.len());
    }

    /// Wave 5 — `test_section_markers`. The about page must render
    /// every section the design doc claims.
    #[test]
    fn test_section_markers() {
        let ctx = PageContext {
            user: None,
            path: "/about".to_string(),
            ..Default::default()
        };
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        for marker in &[
            "about-hero-section",
            "mission-section",
            "about-stats-section",
            "team-section",
            "timeline-section",
            "datatech-section",
            "about-cta-section",
        ] {
            let needle = format!("class=\"{}\"", marker);
            assert!(
                html.contains(&needle),
                "About page must contain section marker '{}'. Got: {}",
                needle, html
            );
        }
    }

    /// Wave 5 — `test_mission_cards`. The mission section has 3
    /// cards: Mission, Vision, Values.
    #[test]
    fn test_mission_cards() {
        let ctx = PageContext {
            user: None,
            path: "/about".to_string(),
            ..Default::default()
        };
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        for title in &["Our Mission", "Our Vision", "Our Values"] {
            assert!(
                html.contains(title),
                "About mission section must include '{}'. Got: {}",
                title, html
            );
        }
    }

    /// Wave 5 — `test_datatech_features`. The DataTech section has
    /// 6 feature cards (Collection, Storage, Management, Processing,
    /// Analytics, Visualization) and a Benefits card.
    #[test]
    fn test_datatech_features() {
        let ctx = PageContext {
            user: None,
            path: "/about".to_string(),
            ..Default::default()
        };
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        for title in &[
            "Data Collection",
            "Data Storage",
            "Data Management",
            "Data Processing",
            "Data Analytics",
            "Data Visualization",
        ] {
            assert!(
                html.contains(title),
                "DataTech section must include feature '{}'. Got: {}",
                title, html
            );
        }
        assert!(html.contains("Benefits"), "DataTech section must include Benefits card. Got: {}", html);
        // "What is a DataTech Platform?" overview heading
        assert!(html.contains("What is a DataTech Platform"), "DataTech section must include the 'What is a DataTech Platform?' overview. Got: {}", html);
    }

    /// Wave 5 — `test_timeline_entries`. The timeline has 6 entries
    /// (3 in 2022-2023, 2 in 2024, 1 in 2025). The number of dots
    /// in the rendered HTML must match.
    #[test]
    fn test_timeline_entries() {
        let ctx = PageContext {
            user: None,
            path: "/about".to_string(),
            ..Default::default()
        };
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        let dot_count = html.matches("about-timeline-dot").count();
        // 6 entries, each gets a dot. The last dot has an extra
        // "about-timeline-dot-current" class but is still a match.
        assert!(dot_count >= 6, "Timeline must have at least 6 dots. Got {} matches in: {}", dot_count, html);
    }

    /// Wave 5 — `test_team_cards`. The team section has 6 cards.
    /// Source has 6; the port must have 6.
    #[test]
    fn test_team_cards() {
        let ctx = PageContext {
            user: None,
            path: "/about".to_string(),
            ..Default::default()
        };
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        let team_card_count = html.matches("about-team-card").count();
        assert_eq!(team_card_count, 6, "About team section must have 6 cards. Got {} markers in: {}", team_card_count, html);
    }
}

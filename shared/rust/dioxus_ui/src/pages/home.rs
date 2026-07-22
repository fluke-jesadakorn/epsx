//! Public home page (`/`).
//!
//! The development implementation populated this page from rankings, plans, and
//! news producers. The Rust frontend does not own equivalent verified home-page
//! loaders yet, so this route keeps the visual marketing shell and links to the
//! dedicated routes without presenting fixtures as production data.

use crate::layout::main_layout::MainLayout;
use crate::primitives::*;
use dioxus::prelude::*;

use super::{PageContext, PageMeta};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Home");
    (
        meta,
        rsx! {
            MainLayout { ctx: ctx.clone(),
                div {
                    class: "home-prod-page relative min-h-screen overflow-hidden bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900",
                    div { class: "relative z-[1] home-prod-content",
                        HeroSection {}
                        AnalyticsPreview {}
                        PlansPreview {}
                        NewsPreview {}
                    }
                }
            }
        },
    )
}

#[component]
fn HeroSection() -> Element {
    rsx! {
        section {
            class: "home-prod-hero relative w-full min-h-[85vh] flex items-center justify-center overflow-hidden",
            "aria-labelledby": "home-title",
            div { class: "home-prod-hero-inner relative text-center space-y-12 max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-16 sm:py-20 z-[1]",
                div { class: "home-prod-hero-head space-y-8",
                    div { class: "space-y-6",
                        div { class: "inline-block home-prod-hero-anim-up",
                            div { class: "home-prod-hero-badge mb-4 inline-flex items-center gap-2 px-4 py-2 rounded-full bg-gradient-to-r from-primary/10 to-secondary/10 border border-primary/20 backdrop-blur-sm",
                                Icon {
                                    name: "trending-up".to_string(),
                                    size: Some(16),
                                    class_name: Some("text-primary".to_string()),
                                }
                                span { class: "text-sm font-medium text-primary", "EPSX" }
                            }
                            h1 {
                                id: "home-title",
                                class: "home-prod-hero-title text-4xl sm:text-5xl md:text-6xl lg:text-7xl xl:text-8xl font-bold leading-tight",
                                span { class: "block home-prod-hero-line", "Explore" }
                                span { class: "block bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 bg-clip-text text-transparent home-prod-hero-gradient",
                                    "Market Analytics"
                                }
                                span { class: "block mt-2 home-prod-hero-line", "With Verified Data" }
                            }
                        }
                        p { class: "home-prod-hero-subtitle text-lg sm:text-xl md:text-2xl text-slate-300 max-w-4xl mx-auto leading-relaxed",
                            "Open the dedicated routes below to see their current availability. This page does not load market, plan, or news records."
                        }
                    }
                    div { class: "home-prod-hero-actions flex flex-col sm:flex-row gap-4 sm:gap-6 justify-center items-center",
                        a {
                            class: "home-prod-hero-cta w-full sm:w-auto min-w-[220px] h-14 text-lg font-bold bg-gradient-to-r from-orange-500 to-yellow-500 hover:from-orange-600 hover:to-yellow-600 text-white rounded-2xl shadow-2xl px-6 inline-flex items-center justify-center",
                            href: "/analytics",
                            Icon {
                                name: "line-chart".to_string(),
                                size: Some(24),
                                class_name: Some("mr-3".to_string()),
                            }
                            span { "Open analytics" }
                        }
                        a {
                            class: "home-prod-hero-cta w-full sm:w-auto min-w-[220px] h-14 text-lg font-bold border border-orange-400/40 text-orange-100 rounded-2xl px-6 inline-flex items-center justify-center hover:bg-orange-400/10",
                            href: "/plans",
                            Icon {
                                name: "layers".to_string(),
                                size: Some(24),
                                class_name: Some("mr-3".to_string()),
                            }
                            span { "Review plans" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn AnalyticsPreview() -> Element {
    rsx! {
        section {
            class: "home-prod-top-performers container mx-auto px-4 py-16 sm:py-24 lg:py-32",
            "aria-labelledby": "home-analytics-title",
            "data-home-market-state": "unavailable",
            div { class: "relative",
                div { class: "absolute -top-8 -left-8 h-16 w-16 rounded-full bg-gradient-to-br from-orange-400/20 to-yellow-400/20 blur-xl home-prod-tp-blob-1" }
                div { class: "absolute -right-8 -bottom-8 h-20 w-20 rounded-full bg-gradient-to-br from-blue-400/20 to-cyan-400/20 blur-xl home-prod-tp-blob-2" }
                div { class: "flex w-full flex-col gap-8 text-center",
                    div { class: "mb-6 space-y-4 home-prod-tp-header",
                        h2 {
                            id: "home-analytics-title",
                            class: "home-prod-tp-title pancake-gradient-text text-3xl font-bold sm:text-4xl",
                            "Market analytics"
                        }
                        p { class: "text-slate-300 mx-auto max-w-2xl home-prod-tp-sub",
                            "No ranking or market records are loaded on the home page. Open analytics to check the route's current data state."
                        }
                        div { class: "home-prod-tp-divider pancake-gradient mx-auto h-1 w-24 rounded-full" }
                    }
                    a {
                        class: "mx-auto inline-flex items-center gap-2 rounded-xl border border-cyan-400/30 px-5 py-3 font-semibold text-cyan-300 hover:bg-cyan-400/10",
                        href: "/analytics",
                        "Open analytics"
                        Icon { name: "arrow-right".to_string(), size: Some(16) }
                    }
                }
            }
        }
    }
}

#[component]
fn PlansPreview() -> Element {
    rsx! {
        section {
            class: "home-prod-pricing container mx-auto px-4 py-16 sm:py-24 lg:py-32",
            "aria-labelledby": "home-plans-title",
            "data-home-plans-state": "unavailable",
            div { class: "rounded-3xl border border-orange-400/20 bg-slate-800/70 p-8 sm:p-12 text-center shadow-2xl",
                div { class: "mx-auto mb-5 flex h-12 w-12 items-center justify-center rounded-xl bg-emerald-500/20",
                    Icon {
                        name: "layers".to_string(),
                        size: Some(24),
                        class_name: Some("text-emerald-400".to_string()),
                    }
                }
                h2 {
                    id: "home-plans-title",
                    class: "home-prod-pricing-personal-title text-3xl font-bold text-white",
                    "Plans"
                }
                p { class: "mx-auto mt-4 max-w-2xl text-slate-300",
                    "The home page does not publish a price or feature catalog. Open plans for the route's current availability and verified terms."
                }
                a {
                    class: "mt-7 inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-orange-500 to-yellow-500 px-5 py-3 font-semibold text-white hover:from-orange-600 hover:to-yellow-600",
                    href: "/plans",
                    "Open plans"
                    Icon { name: "arrow-right".to_string(), size: Some(16) }
                }
            }
        }
    }
}

#[component]
fn NewsPreview() -> Element {
    rsx! {
        section {
            class: "home-prod-news container mx-auto px-4 py-16 sm:py-24 lg:py-32",
            "aria-labelledby": "home-news-title",
            "data-home-news-state": "unavailable",
            div { class: "rounded-3xl border border-white/10 bg-gradient-to-br from-purple-500/20 via-cyan-400/10 to-slate-900/60 p-8 sm:p-12 text-center",
                Icon {
                    name: "newspaper".to_string(),
                    size: Some(28),
                    class_name: Some("mx-auto text-cyan-400".to_string()),
                }
                h2 {
                    id: "home-news-title",
                    class: "home-prod-news-title mt-4 text-3xl font-bold text-white",
                    "News"
                }
                p { class: "mx-auto mt-4 max-w-2xl text-slate-300",
                    "No article previews are loaded on the home page. Open news to see content provided by that route."
                }
                a {
                    class: "home-prod-news-view-all mt-7 inline-flex items-center gap-2 rounded-xl border border-cyan-400/30 px-5 py-3 font-semibold text-cyan-300 hover:bg-cyan-400/10",
                    href: "/news",
                    "Open news"
                    Icon { name: "arrow-right".to_string(), size: Some(16) }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_ctx() -> PageContext {
        PageContext {
            user: None,
            path: "/".to_string(),
            ..Default::default()
        }
    }

    fn render_to_string(ctx: &PageContext) -> String {
        let (_meta, el) = render(ctx);
        dioxus_ssr::render_element(el)
    }

    #[test]
    fn home_preserves_visual_landmarks_and_native_links() {
        let html = render_to_string(&empty_ctx());

        for marker in [
            "home-prod-page",
            "home-prod-hero",
            "home-prod-top-performers",
            "home-prod-pricing",
            "home-prod-news",
            "Market Analytics",
            "href=\"/analytics\"",
            "href=\"/plans\"",
            "href=\"/news\"",
        ] {
            assert!(
                html.contains(marker),
                "missing safe home marker `{marker}`: {html}"
            );
        }
    }

    #[test]
    fn home_fails_closed_without_verified_section_loaders() {
        let html = render_to_string(&empty_ctx());

        for marker in [
            "data-home-market-state=\"unavailable\"",
            "data-home-plans-state=\"unavailable\"",
            "data-home-news-state=\"unavailable\"",
            "No ranking or market records are loaded",
            "does not publish a price or feature catalog",
            "No article previews are loaded",
        ] {
            assert!(
                html.contains(marker),
                "missing unavailable-state marker `{marker}`: {html}"
            );
        }
    }

    #[test]
    fn home_does_not_render_legacy_fixtures_or_numeric_claims() {
        let html = render_to_string(&empty_ctx());

        for fixture in [
            "GHC",
            "ARAX",
            "NVTK",
            "$6,535",
            "+4657%",
            "EPSX Q2 Platform Update",
            "Jun 12, 2026",
            "24/7",
            "100+",
            "&#60; 1s",
            "real-time",
            "Revenue Share",
            "API Personal",
        ] {
            assert!(
                !html.contains(fixture),
                "legacy home claim `{fixture}` must not render: {html}"
            );
        }
    }

    #[test]
    fn home_has_no_inert_share_or_data_controls() {
        let html = render_to_string(&empty_ctx());

        for control in [
            "Share Platform",
            "share-2",
            "Refresh",
            "Export",
            "Load more",
        ] {
            assert!(
                !html.contains(control),
                "inert home control `{control}` must not render: {html}"
            );
        }
        assert!(
            !html.contains("<button"),
            "home previews must use working native links: {html}"
        );
    }
}

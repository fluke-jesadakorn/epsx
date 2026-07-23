//! Public home page (`/`).
//!
//! The development implementation populated this page from rankings, plans, and
//! news producers. The Rust frontend now consumes the same strict normalized
//! public-news list outcome as `/news`, while ranking and plan previews remain
//! unavailable until their backend-owned contracts are verified.

use crate::layout::main_layout::MainLayout;
use crate::primitives::*;
use dioxus::prelude::*;

use super::news::{parse_news_list_outcome, NewsListOutcome, NewsPost};
use super::{PageContext, PageMeta};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Home");
    let news_outcome =
        parse_news_list_outcome(ctx.params.get("data_home_news").map(String::as_str));
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
                        NewsPreview { outcome: news_outcome }
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
                            "Explore verified public news below. Market and plan previews remain on their dedicated routes until their data contracts are available."
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

fn news_metadata(post: &NewsPost) -> String {
    [
        post.published_at.as_deref(),
        post.author.as_deref(),
        post.read_time.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" · ")
}

#[component]
fn NewsSectionHeader() -> Element {
    rsx! {
        div { class: "mb-6 flex items-center justify-between",
            div { class: "flex items-center gap-3",
                Icon {
                    name: "newspaper".to_string(),
                    size: Some(20),
                    class_name: Some("text-cyan-400".to_string()),
                }
                h2 {
                    id: "home-news-title",
                    class: "home-prod-news-title text-xl font-bold text-white",
                    "Latest News"
                }
            }
            a {
                class: "home-prod-news-view-all flex items-center gap-1 text-sm text-cyan-400 hover:text-cyan-300 font-medium",
                href: "/news",
                "View all "
                Icon { name: "arrow-right".to_string(), size: Some(16) }
            }
        }
    }
}

#[component]
fn LeadNewsCard(post: NewsPost) -> Element {
    let metadata = news_metadata(&post);
    rsx! {
        a {
            class: "group block home-news-lead",
            href: "/news/{post.slug}",
            article { class: "news-featured",
                if let Some(cover) = &post.cover_image_url {
                    img {
                        src: cover.clone(),
                        alt: post.title.clone(),
                        loading: "eager",
                    }
                } else {
                    div { class: "absolute inset-0 flex items-center justify-center opacity-10",
                        Icon { name: "newspaper".to_string(), size: Some(96) }
                    }
                }
                div { class: "news-overlay" }
                div { class: "news-caption",
                    if post.featured {
                        div { class: "news-featured-tag mb-3",
                            Icon { name: "pin".to_string(), size: Some(14) }
                            span { "Featured" }
                        }
                    }
                    if !post.tags.is_empty() {
                        div { class: "flex flex-wrap gap-2 mb-3",
                            for tag in post.tags.iter().take(2) {
                                span { class: "news-tag", "{tag}" }
                            }
                        }
                    }
                    h3 { class: "news-title line-clamp-2", "{post.title}" }
                    if !post.summary.is_empty() {
                        p { class: "news-excerpt line-clamp-2", "{post.summary}" }
                    }
                    if !metadata.is_empty() {
                        span { class: "news-date", "{metadata}" }
                    }
                }
            }
        }
    }
}

#[component]
fn SmallNewsCard(post: NewsPost) -> Element {
    let metadata = news_metadata(&post);
    rsx! {
        a {
            class: "group block home-news-small",
            href: "/news/{post.slug}",
            article { class: "news-small",
                if let Some(cover) = &post.cover_image_url {
                    img {
                        src: cover.clone(),
                        alt: post.title.clone(),
                        loading: "lazy",
                    }
                } else {
                    div { class: "absolute inset-0 flex items-center justify-center opacity-10",
                        Icon { name: "newspaper".to_string(), size: Some(48) }
                    }
                }
                div { class: "news-overlay" }
                div { class: "news-caption",
                    if post.featured {
                        div { class: "news-featured-tag mb-1.5",
                            Icon { name: "pin".to_string(), size: Some(12) }
                            span { "Featured" }
                        }
                    }
                    h3 { class: "news-title line-clamp-2", "{post.title}" }
                    if !metadata.is_empty() {
                        span { class: "news-date", "{metadata}" }
                    }
                }
            }
        }
    }
}

#[component]
fn NewsPreview(outcome: NewsListOutcome) -> Element {
    match outcome {
        NewsListOutcome::Ready { articles, .. } => {
            let mut preview = articles.into_iter().take(3);
            let lead = preview
                .next()
                .expect("strict ready news outcomes always contain an article");
            let smaller = preview.collect::<Vec<_>>();
            let small_grid_class = if smaller.len() == 1 {
                "grid gap-4 grid-cols-1"
            } else {
                "grid gap-4 grid-cols-1 sm:grid-cols-2"
            };
            rsx! {
                section {
                    class: "home-prod-news container mx-auto px-4 py-16 sm:py-24 lg:py-32",
                    "aria-labelledby": "home-news-title",
                    "data-home-news-state": "ready",
                    NewsSectionHeader {}
                    div { class: "space-y-4 home-news-preview-list",
                        LeadNewsCard { post: lead }
                        if !smaller.is_empty() {
                            div { class: small_grid_class,
                                for post in smaller {
                                    SmallNewsCard { post }
                                }
                            }
                        }
                    }
                }
            }
        }
        NewsListOutcome::Empty { .. } => rsx! {
            section {
                class: "home-prod-news container mx-auto px-4 py-16 sm:py-24 lg:py-32",
                "aria-labelledby": "home-news-title",
                "data-home-news-state": "empty",
                NewsSectionHeader {}
                div {
                    class: "rounded-3xl border border-white/10 bg-gradient-to-br from-purple-500/20 via-cyan-400/10 to-slate-900/60 p-8 sm:p-12 text-center",
                    p { class: "text-slate-300", "No published articles yet." }
                }
            }
        },
        NewsListOutcome::Error { .. } => rsx! {
            section {
                class: "home-prod-news container mx-auto px-4 py-16 sm:py-24 lg:py-32",
                "aria-labelledby": "home-news-title",
                "data-home-news-state": "unavailable",
                NewsSectionHeader {}
                div {
                    class: "rounded-3xl border border-white/10 bg-gradient-to-br from-purple-500/20 via-cyan-400/10 to-slate-900/60 p-8 sm:p-12 text-center",
                    role: "alert",
                    p { class: "mx-auto max-w-2xl text-slate-300",
                        "Latest news is temporarily unavailable. No cached or sample articles are being shown."
                    }
                    div { class: "mt-7 flex flex-wrap justify-center gap-3",
                        a {
                            class: "inline-flex items-center gap-2 rounded-xl border border-cyan-400/30 px-5 py-3 font-semibold text-cyan-300 hover:bg-cyan-400/10",
                            href: "/news",
                            "Open news"
                        }
                        a {
                            class: "inline-flex items-center gap-2 rounded-xl border border-white/20 px-5 py-3 font-semibold text-white hover:bg-white/5",
                            href: "/",
                            "Retry home"
                        }
                    }
                }
            }
        },
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

    fn news_article(index: usize, featured: bool) -> serde_json::Value {
        serde_json::json!({
            "id": format!("article-{index}"),
            "slug": format!("article-{index}"),
            "title": format!("Article {index}"),
            "summary": format!("Summary {index}"),
            "cover_image_url": format!("/images/article-{index}.png"),
            "author": format!("Author {index}"),
            "published_at": "July 22, 2026",
            "read_time": format!("{index} min"),
            "tags": ["engineering"],
            "featured": featured
        })
    }

    fn news_context(outcome: serde_json::Value, home_query: &str) -> PageContext {
        let mut ctx = empty_ctx();
        ctx.query = home_query.to_string();
        ctx.params
            .insert("data_home_news".to_string(), outcome.to_string());
        ctx
    }

    fn ready_context(articles: Vec<serde_json::Value>, home_query: &str) -> PageContext {
        let total = articles.len();
        news_context(
            serde_json::json!({
                "state": "ready",
                "articles": articles,
                "total": total,
                "page": 1,
                "limit": 12,
                "total_pages": 1,
                "query": "",
                "category": "all"
            }),
            home_query,
        )
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
            "Latest News",
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
    fn home_keeps_market_and_plans_unavailable_and_fails_closed_without_news() {
        let html = render_to_string(&empty_ctx());

        for marker in [
            "data-home-market-state=\"unavailable\"",
            "data-home-plans-state=\"unavailable\"",
            "data-home-news-state=\"unavailable\"",
            "No ranking or market records are loaded",
            "does not publish a price or feature catalog",
            "Latest news is temporarily unavailable",
            "No cached or sample articles",
            "href=\"/\"",
        ] {
            assert!(
                html.contains(marker),
                "missing unavailable-state marker `{marker}`: {html}"
            );
        }
    }

    #[test]
    fn home_news_renders_one_to_three_rows_in_source_order_and_truncates_four() {
        for supplied in 1..=4 {
            let articles = (1..=supplied)
                .map(|index| news_article(index, false))
                .collect();
            let html = render_to_string(&ready_context(articles, ""));
            let rendered = supplied.min(3);

            assert!(html.contains("data-home-news-state=\"ready\""));
            assert_eq!(html.matches("home-news-lead").count(), 1, "{supplied}");
            assert_eq!(
                html.matches("home-news-small").count(),
                rendered - 1,
                "{supplied}"
            );
            for index in 1..=rendered {
                assert!(
                    html.contains(&format!("href=\"/news/article-{index}\"")),
                    "{supplied}: missing article {index}"
                );
            }
            if supplied == 4 {
                assert!(!html.contains("href=\"/news/article-4\""));
            }
            let first = html.find("Article 1").expect("first article must render");
            if rendered >= 2 {
                let second = html.find("Article 2").expect("second article must render");
                assert!(first < second);
            }
            if rendered == 3 {
                let second = html.find("Article 2").expect("second article must render");
                let third = html.find("Article 3").expect("third article must render");
                assert!(second < third);
            }
        }
    }

    #[test]
    fn home_news_featured_badges_follow_rows_without_reordering() {
        let html = render_to_string(&ready_context(
            vec![
                news_article(1, false),
                news_article(2, true),
                news_article(3, false),
            ],
            "",
        ));

        assert_eq!(html.matches("Featured").count(), 1);
        assert!(
            html.find("Article 1").expect("lead must render")
                < html.find("Article 2").expect("featured row must render")
        );
        assert!(html.contains("href=\"/news/article-2\""));
    }

    #[test]
    fn home_news_empty_is_distinct_from_unavailable_and_missing_or_malformed() {
        let empty = render_to_string(&news_context(
            serde_json::json!({
                "state": "empty",
                "total": 0,
                "page": 1,
                "limit": 12,
                "total_pages": 0,
                "query": "",
                "category": "all"
            }),
            "",
        ));
        assert!(empty.contains("data-home-news-state=\"empty\""));
        assert!(empty.contains("No published articles yet."));
        assert!(!empty.contains("temporarily unavailable"));

        let unavailable = render_to_string(&news_context(
            serde_json::json!({"state": "error", "code": "content_unavailable"}),
            "",
        ));
        assert!(unavailable.contains("data-home-news-state=\"unavailable\""));
        assert!(unavailable.contains("Latest news is temporarily unavailable"));
        assert!(unavailable.contains("href=\"/news\""));
        assert!(unavailable.contains("href=\"/\""));
        assert!(!unavailable.contains("No published articles yet."));

        let missing = render_to_string(&empty_ctx());
        assert!(missing.contains("data-home-news-state=\"unavailable\""));

        let mut malformed_ctx = empty_ctx();
        malformed_ctx
            .params
            .insert("data_home_news".to_string(), "{not-json".to_string());
        let malformed = render_to_string(&malformed_ctx);
        assert!(malformed.contains("data-home-news-state=\"unavailable\""));
    }

    #[test]
    fn home_news_escapes_hostile_content_and_rejects_malformed_articles() {
        let mut hostile = news_article(1, true);
        hostile["title"] = serde_json::json!("<script>alert('title')</script>");
        hostile["summary"] = serde_json::json!("<img src=x onerror=alert('summary')>");
        hostile["author"] = serde_json::json!("<b>author</b>");
        hostile["read_time"] = serde_json::json!("<i>7 min</i>");
        let escaped = render_to_string(&ready_context(vec![hostile], ""));
        for raw in [
            "<script>alert('title')</script>",
            "<img src=x onerror=alert('summary')>",
            "<b>author</b>",
            "<i>7 min</i>",
        ] {
            assert!(!escaped.contains(raw), "{raw}");
        }
        for visible in ["alert", "title", "summary", "author", "7 min"] {
            assert!(escaped.contains(visible), "{visible}");
        }
        assert!(escaped.contains("&#60;script&#62;"));
        assert!(escaped.contains("&#60;img src=x onerror=alert("));

        let mut malformed = news_article(1, false);
        malformed["slug"] = serde_json::json!("../foreign");
        let rejected = render_to_string(&ready_context(vec![malformed], ""));
        assert!(rejected.contains("data-home-news-state=\"unavailable\""));
        assert!(!rejected.contains("href=\"/news/../foreign\""));
        assert!(!rejected.contains("Article 1"));
    }

    #[test]
    fn home_query_cannot_filter_page_or_limit_news_preview() {
        let html = render_to_string(&ready_context(
            vec![
                news_article(1, false),
                news_article(2, false),
                news_article(3, false),
            ],
            "q=missing&category=product&page=99&limit=1&ref=affiliate",
        ));

        assert!(html.contains("data-home-news-state=\"ready\""));
        for index in 1..=3 {
            assert!(html.contains(&format!("href=\"/news/article-{index}\"")));
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

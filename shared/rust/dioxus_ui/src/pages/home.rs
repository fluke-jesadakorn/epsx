//! Public home page (`/`).
//!
//! Rankings and news are independent live-data outcomes. A failure in either
//! dependency never suppresses a valid response from the other.
//!
//! Home is fully public — single hero variance (`HeroSection`) for all
//! users. Do not branch on `ctx.wallet` or `ctx.user` for hero selection.
//! Image 1 (`SignedOutHero` / Explore Market Analytics) is deprecated for `/`.

use crate::components::stock_data_card::StockDataCard;
use crate::home::HeroSection;
use crate::layout::main_layout::MainLayout;
use crate::primitives::*;
use dioxus::prelude::*;

use super::analytics::{AnalyticsResponse, AnalyticsRow};
use super::news::{parse_news_list_outcome, NewsListOutcome, NewsPost};
use super::{PageContext, PageMeta};

pub const HOME_ANALYTICS_DATA_PARAM: &str = "data_home_analytics";
pub const HOME_ANALYTICS_STATE_PARAM: &str = "data_home_analytics_state";

#[server]
pub async fn get_home_rankings() -> Result<AnalyticsResponse, ServerFnError> {
    if tokio::runtime::Handle::try_current().is_err() {
        return Err(ServerFnError::new(
            "no runtime, fallback to PageContext".to_string(),
        ));
    }
    let api_url = std::env::var("API_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
    let url = format!(
        "{}/api/analytics/rankings?page=1&limit=3",
        api_url.trim_end_matches('/')
    );
    let resp = reqwest::get(&url)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let value: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let response: AnalyticsResponse =
        serde_json::from_value(value).map_err(|e| ServerFnError::new(e.to_string()))?;
    response
        .validated()
        .map_err(|_| ServerFnError::new("validation failed".to_string()))?;
    Ok(response)
}

#[derive(Clone, Debug, PartialEq)]
enum HomeAnalyticsOutcome {
    Ready(AnalyticsResponse),
    Empty,
    Unavailable,
}

fn parse_home_analytics(ctx: &PageContext) -> HomeAnalyticsOutcome {
    let state = ctx
        .param(HOME_ANALYTICS_STATE_PARAM)
        .map(String::as_str)
        .unwrap_or("unavailable");
    let response = ctx
        .param(HOME_ANALYTICS_DATA_PARAM)
        .and_then(|raw| serde_json::from_str::<AnalyticsResponse>(raw).ok())
        .and_then(|response| response.validated().ok());
    match (state, response) {
        ("ready", Some(response)) if !response.data.is_empty() => {
            HomeAnalyticsOutcome::Ready(response)
        }
        ("empty", Some(response)) if response.data.is_empty() => HomeAnalyticsOutcome::Empty,
        _ => HomeAnalyticsOutcome::Unavailable,
    }
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Home");
    let news_outcome =
        parse_news_list_outcome(ctx.params.get("data_home_news").map(String::as_str));
    let analytics_outcome = parse_home_analytics(ctx);
    (
        meta,
        rsx! {
            MainLayout { ctx: ctx.clone(),
                div {
                    class: "home-prod-page relative min-h-screen overflow-hidden bg-gradient-to-br from-blue-50 via-orange-50 to-yellow-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900",
                    div { class: "relative z-[1] home-prod-content",
                        HeroSection {}
                        AnalyticsPreview { outcome: analytics_outcome }
                        PlansPreview {}
                        NewsPreview { outcome: news_outcome }
                    }
                }
            }
        },
    )
}

fn home_card_values(row: &AnalyticsRow) -> (f64, f64, Option<i32>, Option<f64>) {
    let latest = row.quarterly_performance.first();
    let growth = latest
        .map(|quarter| quarter.eps_growth)
        .or(row.growth_factor)
        .unwrap_or(0.0);
    let price = latest
        .map(|quarter| quarter.price)
        .or(row.price_current)
        .unwrap_or(row.value);
    let days = row
        .next_quarter_estimate
        .as_ref()
        .map(|estimate| estimate.days_until_announcement)
        .or(row.days_until_next_earnings)
        .filter(|days| *days >= 0);
    (growth, price, days, row.progress_percentage)
}

#[component]
fn AnalyticsPreview(outcome: HomeAnalyticsOutcome) -> Element {
    let state = match &outcome {
        HomeAnalyticsOutcome::Ready(_) => "ready",
        HomeAnalyticsOutcome::Empty => "empty",
        HomeAnalyticsOutcome::Unavailable => "unavailable",
    };
    rsx! {
        section {
            class: "home-prod-top-performers container mx-auto px-4 py-16 sm:py-24 lg:py-32",
            "aria-labelledby": "home-analytics-title",
            "data-home-market-state": state,
            div { class: "relative",
                div { class: "absolute -top-8 -left-8 h-16 w-16 rounded-full bg-gradient-to-br from-orange-400/20 to-yellow-400/20 blur-xl home-prod-tp-blob-1" }
                div { class: "absolute -right-8 -bottom-8 h-20 w-20 rounded-full bg-gradient-to-br from-blue-400/20 to-cyan-400/20 blur-xl home-prod-tp-blob-2" }
                div { class: "flex w-full flex-col gap-8 text-center",
                    div { class: "mb-6 space-y-4 home-prod-tp-header",
                        h2 {
                            id: "home-analytics-title",
                            class: "home-prod-tp-title pancake-gradient-text text-3xl font-bold sm:text-4xl",
                            "Performance Companies"
                        }
                        p { class: "text-gray-600 dark:text-gray-300 mx-auto max-w-2xl home-prod-tp-sub",
                            "Discover the data leaders with exceptional growth and performance metrics"
                        }
                        div { class: "home-prod-tp-divider pancake-gradient mx-auto h-1 w-24 rounded-full" }
                    }
                    match outcome {
                        HomeAnalyticsOutcome::Ready(response) => rsx! {
                            div {
                                class: "home-ranking-grid grid grid-cols-1 gap-6 px-2 sm:grid-cols-2 sm:px-0 lg:grid-cols-3",
                                "aria-label": "Public EPS ranking preview",
                                for row in response.data.into_iter().take(3) {
                                    {
                                        let (growth, price, days, progress) = home_card_values(&row);
                                        rsx! {
                                            StockDataCard {
                                                symbol: row.symbol,
                                                rank: row.rank,
                                                eps_growth: growth,
                                                price,
                                                company_name: row.company_name,
                                                days_until_next_action: days,
                                                progress_percentage: progress,
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        HomeAnalyticsOutcome::Empty => rsx! {
                            p { class: "text-gray-600 dark:text-gray-400",
                                "No public rankings are available at this time."
                            }
                        },
                        HomeAnalyticsOutcome::Unavailable => rsx! {
                            div { role: "alert",
                                p { class: "text-gray-600 dark:text-gray-400",
                                    "Unable to load ranking data at this time. Please try again later."
                                }
                                p { class: "sr-only",
                                    "No sample ranking or market records are shown on the home page."
                                }
                            }
                        },
                    }
                    a {
                        class: "mx-auto inline-flex items-center gap-2 rounded-xl border border-cyan-500/40 px-5 py-3 font-semibold text-cyan-700 hover:bg-cyan-400/10 dark:text-cyan-300",
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
            div { class: "home-prod-plan-shell mx-auto max-w-3xl text-center",
                h2 {
                    id: "home-plans-title",
                    class: "home-prod-plan-title pancake-gradient-text text-3xl font-bold sm:text-4xl",
                    "Custom Plans"
                }
                p { class: "mx-auto mt-4 max-w-2xl text-gray-600 dark:text-slate-300",
                    "Tailored solutions for partners, corporate, and enterprise needs"
                }
                div { class: "pancake-gradient mx-auto mt-5 h-1 w-24 rounded-full" }
                article { class: "home-prod-plan-card mx-auto mt-8 max-w-md rounded-2xl border border-purple-500/30 bg-slate-950/70 p-6 text-left shadow-2xl shadow-purple-950/20 sm:p-8",
                    div { class: "text-center",
                        p { class: "text-xs font-semibold tracking-[0.25em] text-slate-300", "CUSTOM" }
                        h3 { class: "mt-4 text-2xl font-bold text-purple-300", "Revenue Share" }
                    }
                    ul { class: "mt-6 space-y-3 text-sm text-slate-300",
                        for item in [
                            "Custom feature set & permissions",
                            "Dedicated support & SLA",
                            "Volume-based pricing",
                            "Custom API rate limits",
                            "White-label options",
                            "Priority onboarding",
                        ] {
                            li { class: "flex items-center gap-2",
                                span { class: "text-purple-700 dark:text-purple-400", "✓" }
                                "{item}"
                            }
                        }
                    }
                    a {
                        class: "mt-7 flex items-center justify-center gap-2 rounded-xl bg-gradient-to-r from-purple-700 to-fuchsia-700 px-5 py-3 font-semibold text-white hover:from-purple-800 hover:to-fuchsia-800",
                        href: "/contact",
                        Icon { name: "message-square".to_string(), size: Some(16) }
                        "Get in Touch"
                    }
                    p { class: "mt-3 text-center text-xs text-slate-500",
                        "We'll create a plan that fits your needs"
                    }
                }
                p { class: "sr-only",
                    "The home page does not publish a price or feature catalog. Open plans for the route's current availability and verified terms."
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
                    class: "home-prod-news-title text-xl font-bold text-gray-900 dark:text-white",
                    "Latest News"
                }
            }
            a {
                class: "home-prod-news-view-all flex items-center gap-1 text-sm text-cyan-700 hover:text-cyan-800 font-medium dark:text-cyan-400 dark:hover:text-cyan-300",
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
                    p { class: "text-slate-600 dark:text-slate-300", "No published articles yet." }
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
                    p { class: "mx-auto max-w-2xl text-slate-600 dark:text-slate-300",
                        "Latest news is temporarily unavailable. No cached or sample articles are being shown."
                    }
                    div { class: "mt-7 flex flex-wrap justify-center gap-3",
                        a {
                            class: "inline-flex items-center gap-2 rounded-xl border border-cyan-500/40 px-5 py-3 font-semibold text-cyan-700 hover:bg-cyan-400/10 dark:text-cyan-300",
                            href: "/news",
                            "Open news"
                        }
                        a {
                            class: "inline-flex items-center gap-2 rounded-xl border border-slate-300 px-5 py-3 font-semibold text-slate-800 hover:bg-slate-100 dark:border-white/20 dark:text-white dark:hover:bg-white/5",
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

    fn home_ranking(rank: i32, symbol: &str) -> serde_json::Value {
        serde_json::json!({
            "rank": rank,
            "symbol": symbol,
            "company_name": format!("{symbol} Live Company"),
            "latest_date": "2026-07-27",
            "value": 99.0,
            "active_status": "TRACK",
            "quarterly_performance": [{
                "quarter": "Q2",
                "date": "2026-06-30",
                "price": 250.25,
                "eps": 2.0,
                "eps_growth": 18.5,
                "price_growth": 2.0,
                "announcement_date": null,
                "announcement_timestamp": null,
                "is_estimated": false
            }],
            "next_quarter_estimate": null,
            "next_earnings_date": null,
            "last_earnings_date": null,
            "next_earnings_date_formatted": null,
            "days_until_next_earnings": null,
            "progress_percentage": null,
            "current_eps": 2.0,
            "growth_factor": 18.5,
            "price_current": 250.25
        })
    }

    fn with_home_rankings(mut ctx: PageContext, rows: Vec<serde_json::Value>) -> PageContext {
        let total = rows.len();
        ctx.params.insert(
            HOME_ANALYTICS_STATE_PARAM.to_string(),
            if rows.is_empty() { "empty" } else { "ready" }.to_string(),
        );
        ctx.params.insert(
            HOME_ANALYTICS_DATA_PARAM.to_string(),
            serde_json::json!({
                "success": true,
                "data": rows,
                "pagination": {
                    "page": 1,
                    "limit": 3,
                    "total": total,
                    "totalPages": if total == 0 { 0 } else { 1 },
                    "hasNext": false,
                    "hasPrev": false
                },
                "metadata": {
                    "available_countries": ["america"],
                    "available_sectors": ["Technology"],
                    "request_timestamp": "2026-07-27T00:00:00Z",
                    "data_source": "live"
                },
                "access_info": {
                    "min_accessible_rank": 100,
                    "locked_ranks_count": 99
                },
                "message": "public preview",
                "processing_time_ms": 1
            })
            .to_string(),
        );
        ctx
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
            "home-prod-hero-stats",
            "home-prod-top-performers",
            "home-prod-pricing",
            "home-prod-news",
            "Performance Analytics Platform",
            "Track Your",
            "Performance Growth",
            "Metrics",
            "Start Exploration",
            "Share Platform",
            "Latest News",
            "href=\"/analytics\"",
            "href=\"/news\"",
        ] {
            assert!(
                html.contains(marker),
                "missing safe home marker `{marker}`: {html}"
            );
        }
    }

    #[test]
    fn home_keeps_independent_market_plans_and_news_unavailable_states() {
        let html = render_to_string(&empty_ctx());

        for marker in [
            "data-home-market-state=\"unavailable\"",
            "data-home-plans-state=\"unavailable\"",
            "data-home-news-state=\"unavailable\"",
            "No sample ranking or market records are shown",
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
    fn home_renders_exactly_three_live_public_cards_in_backend_order() {
        let ctx = with_home_rankings(
            empty_ctx(),
            vec![
                home_ranking(100, "LIVE100"),
                home_ranking(101, "LIVE101"),
                home_ranking(102, "LIVE102"),
            ],
        );
        let html = render_to_string(&ctx);

        assert!(html.contains("data-home-market-state=\"ready\""));
        assert_eq!(html.matches("data-stock-card=\"true\"").count(), 3);
        assert!(html.contains("RANK #100"));
        assert!(html.contains("RANK #101"));
        assert!(html.contains("RANK #102"));
        assert!(html.contains("$250.25"));
        assert!(html.contains("Next Action"));
        assert!(!html.contains("+18.50%"));
        assert!(!html.contains("data-watchlist-toggle"));
        assert!(!html.contains("data-watchlist-signed-out"));
        let first = html.find("LIVE100").unwrap();
        let second = html.find("LIVE101").unwrap();
        let third = html.find("LIVE102").unwrap();
        assert!(first < second && second < third);
    }

    #[test]
    fn home_market_empty_and_malformed_do_not_affect_ready_news() {
        let news = ready_context(vec![news_article(1, false)], "");
        let empty = render_to_string(&with_home_rankings(news.clone(), vec![]));
        assert!(empty.contains("data-home-market-state=\"empty\""));
        assert!(empty.contains("No public rankings"));
        assert!(empty.contains("data-home-news-state=\"ready\""));
        assert!(empty.contains("Article 1"));

        let mut malformed = news;
        malformed
            .params
            .insert(HOME_ANALYTICS_STATE_PARAM.to_string(), "ready".to_string());
        malformed.params.insert(
            HOME_ANALYTICS_DATA_PARAM.to_string(),
            r#"{"success":true,"data":[{"rank":100,"symbol":"CANNED"}]}"#.to_string(),
        );
        let malformed = render_to_string(&malformed);
        assert!(malformed.contains("data-home-market-state=\"unavailable\""));
        assert!(!malformed.contains("CANNED"));
        assert!(malformed.contains("data-home-news-state=\"ready\""));
        assert!(malformed.contains("Article 1"));
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
    fn home_does_not_render_legacy_fixtures_or_unverified_market_data() {
        let html = render_to_string(&empty_ctx());

        for fixture in [
            "GHC",
            "ARAX",
            "NVTK",
            "$6,535",
            "+4657%",
            "EPSX Q2 Platform Update",
            "Jun 12, 2026",
            "real-time ranking fixture",
            "API Personal",
        ] {
            assert!(
                !html.contains(fixture),
                "legacy home claim `{fixture}` must not render: {html}"
            );
        }
    }

    #[test]
    fn home_share_cta_is_wired_and_data_controls_remain_absent() {
        let html = render_to_string(&empty_ctx());

        for control in ["Refresh", "Export", "Load more"] {
            assert!(
                !html.contains(control),
                "inert home control `{control}` must not render: {html}"
            );
        }
        assert!(html.contains("Share Platform"));
        assert!(html.contains("data-share-text=\"\""));
        assert!(html.contains("data-epsx-action=\"share\""));
        assert!(!html.contains("onclick=\""));
        assert!(
            html.contains("type=\"button\""),
            "home share CTA must render a native button: {html}"
        );
    }

    #[test]
    fn home_hero_is_public_single_variance_for_wallet_and_user() {
        use crate::auth::wallet_button::ConnectedWalletState;
        use crate::auth::User;

        let anon_html = render_to_string(&empty_ctx());

        let mut wallet_ctx = empty_ctx();
        wallet_ctx.wallet = ConnectedWalletState {
            address: Some("0x1234567890abcdef1234567890abcdef12345678".to_string()),
            connector_id: Some("metaMask".to_string()),
            chain_id: Some(56),
            is_authenticated: false,
            ..Default::default()
        };
        let wallet_html = render_to_string(&wallet_ctx);

        let mut user_ctx = empty_ctx();
        user_ctx.user = Some(User {
            id: "0xabc".to_string(),
            address: "0xabc".to_string(),
            chain_id: "56".to_string(),
            roles: vec![],
            email: None,
            tier: None,
            permissions: vec![],
            last_login_at: None,
            auth_method: crate::auth::user::AuthMethod::Siwe,
            display_name: None,
        });
        user_ctx.wallet = ConnectedWalletState {
            address: Some("0xabc".to_string()),
            connector_id: Some("metaMask".to_string()),
            chain_id: Some(56),
            is_authenticated: true,
            ..Default::default()
        };
        let user_html = render_to_string(&user_ctx);

        for (label, html) in [
            ("anon", anon_html.as_str()),
            ("wallet", wallet_html.as_str()),
            ("user", user_html.as_str()),
        ] {
            assert!(
                html.contains("Performance Analytics Platform"),
                "{label} must render HeroSection badge: {html}"
            );
            assert!(
                html.contains("Track Your"),
                "{label} missing Track Your: {html}"
            );
            assert!(
                html.contains("Performance Growth"),
                "{label} missing Performance Growth: {html}"
            );
            assert!(html.contains("Metrics"), "{label} missing Metrics: {html}");
            assert!(
                html.contains("Start Exploration"),
                "{label} missing Start Exploration CTA: {html}"
            );
            assert!(
                html.contains("Share Platform"),
                "{label} missing Share Platform CTA: {html}"
            );
            assert!(
                html.contains("home-prod-hero-stats"),
                "{label} must contain HeroSection stats grid: {html}"
            );
            assert!(html.contains("24/7"), "{label} missing 24/7 stat: {html}");
            assert!(html.contains("100+"), "{label} missing 100+ stat: {html}");
            // SignedOutHero must not render on `/`
            assert!(
                !html.contains("data-home-hero-state=\"signed-out\""),
                "{label} must not contain signed-out hero state: {html}"
            );
            assert!(
                !html.contains("Explore Market Analytics") && !html.contains("With Verified Data"),
                "{label} must not contain SignedOutHero headline: {html}"
            );
        }

        // All three variances must be structurally identical for hero (public Image 2)
        assert!(anon_html.contains("Performance Analytics Platform"));
        assert!(wallet_html.contains("Performance Analytics Platform"));
        assert!(user_html.contains("Performance Analytics Platform"));
        assert!(anon_html.contains("home-prod-hero-stats"));
        assert!(wallet_html.contains("home-prod-hero-stats"));
        assert!(user_html.contains("home-prod-hero-stats"));
    }
}

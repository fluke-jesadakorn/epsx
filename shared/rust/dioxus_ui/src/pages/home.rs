//! Public home page (`/`).
//!
//! The platform introduction is public and independent of data outcomes.
//! Plans and news retain independent loading and recovery behavior.
//! Do not branch on wallet or user state for hero selection.

use crate::enterprise::{DataState, HomeHero, HOME_DESCRIPTION, HOME_TITLE};

use crate::layout::main_layout::MainLayout;
use crate::primitives::*;
use dioxus::prelude::*;

use super::analytics::AnalyticsResponse;
use super::news::{parse_news_list_outcome, NewsListOutcome, NewsPost};
use super::plans::{PlanCard, PublicPlan, PublicPlansLoadOutcome};
use super::{PageContext, PageMeta};

pub const HOME_ANALYTICS_DATA_PARAM: &str = "data_home_analytics";
pub const HOME_ANALYTICS_STATE_PARAM: &str = "data_home_analytics_state";
pub const HOME_PLANS_DATA_PARAM: &str = "data_home_plans";

#[derive(Clone, Debug, PartialEq)]
enum HomePlansOutcome {
    Ready(Vec<PublicPlan>),
    Empty,
    Unavailable,
    Malformed,
}

fn parse_home_plans(ctx: &PageContext) -> HomePlansOutcome {
    let Some(raw) = ctx.params.get(HOME_PLANS_DATA_PARAM) else {
        return HomePlansOutcome::Unavailable;
    };
    match serde_json::from_str::<PublicPlansLoadOutcome>(raw) {
        Ok(PublicPlansLoadOutcome::Ready { plans }) if !plans.is_empty() => {
            HomePlansOutcome::Ready(plans)
        }
        Ok(PublicPlansLoadOutcome::Ready { .. } | PublicPlansLoadOutcome::Empty) => {
            HomePlansOutcome::Empty
        }
        Ok(PublicPlansLoadOutcome::Error { code }) if code == "malformed_plans_response" => {
            HomePlansOutcome::Malformed
        }
        Ok(PublicPlansLoadOutcome::Error { .. }) => HomePlansOutcome::Unavailable,
        Err(_) => HomePlansOutcome::Malformed,
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HomeData {
    pub rankings: Result<AnalyticsResponse, crate::fullstack::LoadError>,
    pub plans: PublicPlansLoadOutcome,
    pub news: NewsListOutcome,
}

#[cfg(feature = "server")]
#[derive(Clone)]
pub struct HomeProvider(
    pub  std::sync::Arc<
        dyn Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = HomeData> + Send>>
            + Send
            + Sync,
    >,
);

#[server(prefix = "/_server/frontend", endpoint = "home")]
pub async fn read_home() -> Result<HomeData, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<HomeProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("Home provider unavailable"))?;
    Ok((provider.0)().await)
}

#[derive(Clone, Copy)]
struct HomeRefresh(EventHandler<()>);

#[component]
pub fn HydratedHome() -> Element {
    let mut result = use_server_future(|| async { read_home().await })?;
    let refresh = use_callback(move |_| result.restart());
    use_context_provider(|| HomeRefresh(refresh));
    let navigator = use_navigator();
    let navigate = use_callback(move |url: String| {
        navigator.push(url);
    });
    use_context_provider(|| crate::fullstack::analytics::AnalyticsNavigation(navigate));
    let data = result.read().clone();
    rsx! {
        document::Title { "{HOME_TITLE}" }
        document::Meta { name: "description", content: HOME_DESCRIPTION }
        div { class: "fe-home", div { class: "relative z-[1] home-prod-content",
            HomeHero {}
            match data {
                Some(Ok(data)) => {
                    let plans_outcome = match data.plans {
                        PublicPlansLoadOutcome::Ready { plans } if !plans.is_empty() => HomePlansOutcome::Ready(plans),
                        PublicPlansLoadOutcome::Ready { .. } | PublicPlansLoadOutcome::Empty => HomePlansOutcome::Empty,
                        PublicPlansLoadOutcome::Error { code } if code == "malformed_plans_response" => HomePlansOutcome::Malformed,
                        _ => HomePlansOutcome::Unavailable,
                    };
                    rsx! {
                        PlansPreview { outcome: plans_outcome }
                        NewsPreview { outcome: data.news }
                    }
                },
                _ => rsx! { section { class: "fe-page", role: "status",
                    p { "Could not load the home page. Please try again." }
                    button { r#type: "button", class: "fe-button", onclick: move |_| result.restart(), "Try again" }
                } },
            }
        } }
    }
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let mut meta = PageMeta::marketing("Home");
    meta.title = HOME_TITLE.into();
    meta.description = HOME_DESCRIPTION.into();
    let news_outcome =
        parse_news_list_outcome(ctx.params.get("data_home_news").map(String::as_str));
    let plans_outcome = parse_home_plans(ctx);
    (
        meta,
        rsx! {
            MainLayout { ctx: ctx.clone(),
                div {
                    class: "fe-home",
                    div { class: "relative z-[1] home-prod-content",
                        HomeHero {}
                        PlansPreview { outcome: plans_outcome }
                        NewsPreview { outcome: news_outcome }
                    }
                }
            }
        },
    )
}

#[component]
fn PlansPreview(outcome: HomePlansOutcome) -> Element {
    let effective = outcome;
    let state = match &effective {
        HomePlansOutcome::Ready(_) => "ready",
        HomePlansOutcome::Empty => "empty",
        HomePlansOutcome::Unavailable => "unavailable",
        HomePlansOutcome::Malformed => "malformed",
    };
    rsx! {
        section { class: "home-prod-pricing container fe-page-layout", aria_labelledby: "home-plans-title", "data-home-plans-state": state,
            h2 { id: "home-plans-title", class: "text-2xl font-semibold", "Plans for your workflow" }
            p { class: "fe-help", "Compare available features and access before choosing a plan." }
            match effective {
                HomePlansOutcome::Ready(plans) => rsx! {
                    div { class: "grid grid-cols-1 gap-6 md:grid-cols-2 xl:grid-cols-3",
                        for plan in plans { PlanCard { plan, frontend: true } }
                    }
                },
                HomePlansOutcome::Empty => rsx! { DataState { title: "No plans available", message: "There are no public plans to display right now.", href: "/plans", action: "View plans" } },
                _ => rsx! { DataState { title: "Plans are temporarily unavailable", message: "Please try again to see current plans and features.", href: "/plans", action: "View plans" } },
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
    let navigation = try_consume_context::<crate::fullstack::analytics::AnalyticsNavigation>();
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
                    class: "home-prod-news-title text-xl font-bold text-gray-900 dark:text-white fe-tone-text",
                    "Latest News"
                }
            }
            a {
                class: "home-prod-news-view-all flex items-center gap-1 text-sm text-cyan-700 hover:text-cyan-800 font-medium dark:text-cyan-400 dark:hover:text-cyan-300 fe-tone-accent",
                href: "/news",
                onclick: move |event| crate::fullstack::analytics::follow_link(event, navigation, "/news"),
                "View all "
                Icon { name: "arrow-right".to_string(), size: Some(16) }
            }
        }
    }
}

#[component]
fn LeadNewsCard(post: NewsPost) -> Element {
    let navigation = try_consume_context::<crate::fullstack::analytics::AnalyticsNavigation>();
    let metadata = news_metadata(&post);
    rsx! {
        a {
            class: "group block home-news-lead",
            href: "/news/{post.slug}",
            onclick: move |event| crate::fullstack::analytics::follow_link(event, navigation, &format!("/news/{}", post.slug)),
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
    let navigation = try_consume_context::<crate::fullstack::analytics::AnalyticsNavigation>();
    let metadata = news_metadata(&post);
    rsx! {
        a {
            class: "group block home-news-small",
            href: "/news/{post.slug}",
            onclick: move |event| crate::fullstack::analytics::follow_link(event, navigation, &format!("/news/{}", post.slug)),
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
    let refresh = try_consume_context::<HomeRefresh>();
    let navigation = try_consume_context::<crate::fullstack::analytics::AnalyticsNavigation>();
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
                    class: "home-prod-news container mx-auto px-4 py-16 sm:py-24 lg:py-32 fe-page-layout",
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
                class: "home-prod-news container mx-auto px-4 py-16 sm:py-24 lg:py-32 fe-page-layout",
                "aria-labelledby": "home-news-title",
                "data-home-news-state": "empty",
                NewsSectionHeader {}
                div {
                    class: "rounded-3xl border border-white/10 bg-gradient-to-br from-purple-500/20 via-cyan-400/10 to-slate-900/60 p-8 sm:p-12 text-center fe-fill-neutral",
                    p { class: "text-slate-600 dark:text-slate-300 fe-tone-muted", "No published articles yet." }
                }
            }
        },
        NewsListOutcome::Error { .. } => rsx! {
            section {
                class: "home-prod-news container mx-auto px-4 py-16 sm:py-24 lg:py-32 fe-page-layout",
                "aria-labelledby": "home-news-title",
                "data-home-news-state": "unavailable",
                NewsSectionHeader {}
                div {
                    class: "rounded-3xl border border-white/10 bg-gradient-to-br from-purple-500/20 via-cyan-400/10 to-slate-900/60 p-8 sm:p-12 text-center fe-fill-neutral",
                    role: "alert",
                    p { class: "mx-auto max-w-2xl text-slate-600 dark:text-slate-300 fe-tone-muted",
                        "We couldn’t load the latest news. Please try again."
                    }
                    div { class: "mt-7 flex flex-wrap justify-center gap-3",
                        a {
                            class: "inline-flex items-center gap-2 rounded-xl border border-cyan-500/40 px-5 py-3 font-semibold text-cyan-700 hover:bg-cyan-400/10 dark:text-cyan-300 fe-tone-accent",
                            href: "/news",
                onclick: move |event| crate::fullstack::analytics::follow_link(event, navigation, "/news"),
                            "Open news"
                        }
                        a {
                            class: "inline-flex items-center gap-2 rounded-xl border border-slate-300 px-5 py-3 font-semibold text-slate-800 hover:bg-slate-100 dark:border-white/20 dark:text-white dark:hover:bg-white/5",
                            href: "/",
                            onclick: move |event| { if let Some(refresh) = refresh { event.prevent_default(); refresh.0.call(()); } },
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

    fn hero_markup(html: &str) -> &str {
        let start = html.find("<section class=\"fe-hero\"").expect("home hero");
        let end = html[start..].find("</section>").expect("hero closing tag") + start;
        &html[start..end]
    }

    #[test]
    fn home_preserves_visual_landmarks_and_native_links() {
        let (meta, element) = render(&empty_ctx());
        let html = dioxus_ssr::render_element(element);
        assert_eq!(meta.title, HOME_TITLE);
        assert_eq!(meta.description, HOME_DESCRIPTION);
        for marker in [
            "fe-home",
            "fe-hero",
            "fe-hero-art",
            "home-prod-pricing",
            "home-prod-news",
            "Financial Technology Platform",
            "Financial technology.",
            "Connected by design.",
            HOME_DESCRIPTION,
            "Explore platform",
            "About EPSX",
            "Latest News",
            "href=\"/analytics\"",
            "href=\"/about\"",
            "href=\"/news\"",
        ] {
            assert!(html.contains(marker), "missing home marker `{marker}`");
        }
        for retired in [
            "fe-product-preview",
            "fe-workflow",
            "home-prod-top-performers",
            "EPSX’s proprietary methodology.",
        ] {
            assert!(!html.contains(retired), "retired hero content `{retired}`");
        }
        assert_eq!(html.matches("id=\"home-title\"").count(), 1);
        assert!(html.find("class=\"fe-hero\"").unwrap() < html.find("home-prod-pricing").unwrap());
        assert!(html.find("home-prod-pricing").unwrap() < html.find("home-prod-news").unwrap());
    }

    #[test]
    fn home_keeps_introduction_and_independent_plans_and_news_unavailable_states() {
        let html = render_to_string(&empty_ctx());
        for marker in [
            HOME_DESCRIPTION,
            "data-home-plans-state=\"unavailable\"",
            "data-home-news-state=\"unavailable\"",
            "Plans are temporarily unavailable",
            "We couldn’t load the latest news",
            "Please try again",
            "href=\"/\"",
        ] {
            assert!(
                html.contains(marker),
                "missing unavailable-state marker `{marker}`"
            );
        }
        assert!(!html.contains("data-home-market-state"));
    }

    #[test]
    fn home_introduction_is_identical_for_every_ranking_outcome() {
        let baseline = render_to_string(&empty_ctx());
        let mut malformed = empty_ctx();
        malformed
            .params
            .insert(HOME_ANALYTICS_STATE_PARAM.into(), "ready".into());
        malformed
            .params
            .insert(HOME_ANALYTICS_DATA_PARAM.into(), "{not-json".into());
        for ctx in [
            empty_ctx(),
            with_home_rankings(empty_ctx(), vec![]),
            with_home_rankings(
                empty_ctx(),
                vec![home_ranking(100, "LIVE100"), home_ranking(101, "LIVE101")],
            ),
            malformed,
        ] {
            let html = render_to_string(&ctx);
            assert_eq!(hero_markup(&html), hero_markup(&baseline));
            for forbidden in [
                "data-stock-card",
                "data-home-market-state",
                "LIVE100",
                "LIVE101",
                "data-watchlist-toggle",
            ] {
                assert!(
                    !html.contains(forbidden),
                    "home leaked ranking content `{forbidden}`"
                );
            }
        }
    }

    #[test]
    fn home_market_empty_and_malformed_do_not_affect_ready_news() {
        let news = ready_context(vec![news_article(1, false)], "");
        let empty = render_to_string(&with_home_rankings(news.clone(), vec![]));
        let mut malformed = news;
        malformed
            .params
            .insert(HOME_ANALYTICS_STATE_PARAM.into(), "ready".into());
        malformed.params.insert(
            HOME_ANALYTICS_DATA_PARAM.into(),
            r#"{"success":true,"data":[{"rank":100,"symbol":"CANNED"}]}"#.into(),
        );
        let malformed = render_to_string(&malformed);
        for html in [&empty, &malformed] {
            assert!(html.contains(HOME_DESCRIPTION));
            assert!(html.contains("data-home-news-state=\"ready\""));
            assert!(html.contains("Article 1"));
            assert!(!html.contains("data-stock-card"));
            assert!(!html.contains("CANNED"));
            assert!(html.contains("href=\"/analytics\""));
        }
        assert_eq!(hero_markup(&empty), hero_markup(&malformed));
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
        assert!(!empty.contains("We couldn’t load the latest news"));

        let unavailable = render_to_string(&news_context(
            serde_json::json!({"state": "error", "code": "content_unavailable"}),
            "",
        ));
        assert!(unavailable.contains("data-home-news-state=\"unavailable\""));
        assert!(unavailable.contains("We couldn’t load the latest news"));
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
    fn home_ctas_are_native_links_and_data_controls_remain_absent() {
        let html = render_to_string(&empty_ctx());

        for control in ["Refresh", "Export", "Load more"] {
            assert!(
                !html.contains(control),
                "inert home control `{control}` must not render: {html}"
            );
        }
        assert!(!html.contains("How it works"));
        assert!(!html.contains("href=\"/manual\""));
        assert!(!html.contains("data-epsx-action=\"share\""));
        assert!(!html.contains("onclick=\""));
        assert!(
            html.contains("href=\"/analytics\""),
            "home primary CTA must be a native link: {html}"
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

        for html in [&wallet_html, &user_html] {
            assert_eq!(hero_markup(html), hero_markup(&anon_html));
        }
        let hero = hero_markup(&anon_html);
        for marker in [
            HOME_DESCRIPTION,
            "Financial technology.",
            "Connected by design.",
            "Explore platform",
            "About EPSX",
        ] {
            assert!(
                hero.contains(marker),
                "missing public introduction `{marker}`"
            );
        }
        assert!(!hero.contains("data-home-hero-state=\"signed-out\""));
        assert!(!hero.contains("fe-product-preview"));
    }
}

//! Public news listing backed only by an explicit SSR content outcome.
//!
//! The frontend BFF owns transport and envelope adaptation. This page owns
//! presentation-only search/category/page controls and never substitutes
//! sample articles when the content dependency is empty, unavailable, or
//! malformed.

use super::{PageContext, PageMeta};
use crate::layout::main_layout::MainLayout;
use crate::primitives::*;
use dioxus::prelude::*;

const NEWS_PAGE_SIZE: u32 = 12;
const NEWS_CATEGORIES: [&str; 4] = ["all", "updates", "engineering", "product"];
const NEWS_AUTHOR_MAX_CHARS: usize = 160;
const NEWS_READ_TIME_MAX_CHARS: usize = 32;

#[derive(Clone, Debug, serde::Deserialize, PartialEq, Eq)]
pub(super) struct NewsPost {
    id: Option<String>,
    pub(super) slug: String,
    pub(super) title: String,
    pub(super) summary: String,
    pub(super) cover_image_url: Option<String>,
    pub(super) author: Option<String>,
    pub(super) published_at: Option<String>,
    pub(super) read_time: Option<String>,
    pub(super) tags: Vec<String>,
    pub(super) featured: bool,
}

#[derive(Clone, Debug, serde::Deserialize, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub(super) enum NewsListOutcome {
    Ready {
        articles: Vec<NewsPost>,
        total: u64,
        page: u32,
        limit: u32,
        total_pages: u32,
        query: String,
        category: String,
    },
    Empty {
        total: u64,
        page: u32,
        limit: u32,
        total_pages: u32,
        query: String,
        category: String,
    },
    Error {
        code: String,
    },
}

fn safe_text(value: &str, max: usize) -> bool {
    value.chars().count() <= max && !value.chars().any(char::is_control)
}

fn valid_optional_display_text(value: Option<&str>, max: usize) -> bool {
    value.is_none_or(|value| !value.trim().is_empty() && safe_text(value, max))
}

fn safe_slug(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && !value.starts_with('-')
        && !value.ends_with('-')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn safe_cover(value: &str) -> bool {
    !value.is_empty()
        && !value.chars().any(char::is_control)
        && !value.contains('\\')
        && ((value.starts_with('/') && !value.starts_with("//")) || value.starts_with("https://"))
}

fn valid_display_date(value: &str) -> bool {
    const MONTHS: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let Some((month, rest)) = value.split_once(' ') else {
        return false;
    };
    let Some((day, year)) = rest.split_once(", ") else {
        return false;
    };
    MONTHS.contains(&month)
        && day.parse::<u8>().is_ok_and(|day| (1..=31).contains(&day))
        && year.len() == 4
        && year.bytes().all(|byte| byte.is_ascii_digit())
}

fn valid_post(post: &NewsPost) -> bool {
    safe_slug(&post.slug)
        && !post.title.trim().is_empty()
        && safe_text(&post.title, 300)
        && safe_text(&post.summary, 2_000)
        && post.tags.len() <= 32
        && post
            .tags
            .iter()
            .all(|tag| !tag.trim().is_empty() && safe_text(tag, 64))
        && valid_optional_display_text(post.author.as_deref(), NEWS_AUTHOR_MAX_CHARS)
        && valid_optional_display_text(post.read_time.as_deref(), NEWS_READ_TIME_MAX_CHARS)
        && post.cover_image_url.as_deref().is_none_or(safe_cover)
        && post.published_at.as_deref().is_none_or(valid_display_date)
}

pub(super) fn parse_news_list_outcome(raw: Option<&str>) -> NewsListOutcome {
    let Some(raw) = raw else {
        return NewsListOutcome::Error {
            code: "missing_content_outcome".to_string(),
        };
    };
    let Ok(outcome) = serde_json::from_str::<NewsListOutcome>(raw) else {
        return NewsListOutcome::Error {
            code: "malformed_content_response".to_string(),
        };
    };
    let valid = match &outcome {
        NewsListOutcome::Ready {
            articles,
            total,
            page,
            limit,
            total_pages,
            query,
            category,
        } => {
            let expected_pages = (*limit > 0).then(|| total.div_ceil(*limit as u64) as u32);
            !articles.is_empty()
                && *total >= articles.len() as u64
                && *page > 0
                && *limit == NEWS_PAGE_SIZE
                && articles.len() <= *limit as usize
                && Some(*total_pages) == expected_pages
                && *page <= *total_pages
                && safe_text(query, 200)
                && NEWS_CATEGORIES.contains(&category.as_str())
                && articles.iter().all(valid_post)
        }
        NewsListOutcome::Empty {
            total,
            page,
            limit,
            total_pages,
            query,
            category,
        } => {
            let expected_pages = (*limit > 0).then(|| {
                if *total == 0 {
                    0
                } else {
                    total.div_ceil(*limit as u64) as u32
                }
            });
            *page > 0
                && *limit == NEWS_PAGE_SIZE
                && Some(*total_pages) == expected_pages
                && ((*total == 0 && *page == 1) || (*total > 0 && *page > *total_pages))
                && safe_text(query, 200)
                && NEWS_CATEGORIES.contains(&category.as_str())
        }
        NewsListOutcome::Error { code } => !code.is_empty() && safe_text(code, 64),
    };
    if valid {
        outcome
    } else {
        NewsListOutcome::Error {
            code: "malformed_content_response".to_string(),
        }
    }
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let mut meta = PageMeta::marketing("News");
    meta.title = "News — EPSX".to_string();
    meta.description = "Latest news and updates from EPSX analytics platform".to_string();
    let outcome = parse_news_list_outcome(ctx.params.get("data_news").map(String::as_str));
    let retry_href = if ctx.query.is_empty() {
        "/news".to_string()
    } else {
        format!("/news?{}", ctx.query)
    };

    (
        meta,
        rsx! {
            MainLayout { ctx: ctx.clone(),
                NewsPageBody { outcome, retry_href }
            }
        },
    )
}

#[component]
fn NewsPageBody(outcome: NewsListOutcome, retry_href: String) -> Element {
    let (query, category, total) = match &outcome {
        NewsListOutcome::Ready {
            query,
            category,
            total,
            ..
        }
        | NewsListOutcome::Empty {
            query,
            category,
            total,
            ..
        } => (query.clone(), category.clone(), Some(*total)),
        NewsListOutcome::Error { .. } => (String::new(), "all".to_string(), None),
    };

    rsx! {
        // Keep the public news route on the same full-width dark frame as
        // the production page. The responsive inner padding is intentionally
        // owned by this wrapper so the header and featured card share the
        // exact 16px desktop edge used by the source composition.
        div { class: "relative min-h-screen bg-slate-950",
            div { class: "relative z-10 mx-auto max-w-7xl px-4 py-12 sm:py-16",
                div { class: "page-content news-page w-full",
                    header { class: "mb-12 text-center news-header",
                        div { class: "inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-cyan-500/20 bg-cyan-500/5 text-cyan-500 text-xs font-semibold mb-5",
                            Icon { name: "newspaper".to_string(), size: Some(14) }
                            " EPSX Platform"
                        }
                        h1 { class: "text-4xl sm:text-5xl font-extrabold mb-4",
                            "News & " span { class: "gradient-text-cool", "Updates" }
                        }
                        p { class: "text-muted-foreground max-w-xl mx-auto leading-relaxed",
                            "Stay informed with the latest platform updates, feature releases, and market insights from the EPSX team."
                        }
                        if let Some(total) = total {
                            p { class: "mt-3 text-sm text-muted-foreground/60",
                                {
                                    let noun = if total == 1 { "article" } else { "articles" };
                                    format!("{total} {noun}")
                                }
                            }
                        }
                    }
                    // The current production composition exposes filtering via
                    // query-string links rather than a persistent toolbar. Keep
                    // the validated form in the SSR tree for keyboard and
                    // assistive-technology users without adding a visible row
                    // that shifts the featured article below the fold.
                    div { class: "sr-only", NewsFilters { initial_query: query.clone(), initial_category: category.clone() } }
                    match outcome {
                        NewsListOutcome::Ready {
                            articles,
                            total,
                            page,
                            total_pages,
                            query,
                            category,
                            ..
                        } => rsx! {
                            NewsList {
                                posts: articles,
                                total,
                                page,
                                total_pages,
                                query,
                                category,
                            }
                        },
                        NewsListOutcome::Empty { total, page, total_pages, query, category, .. } => rsx! {
                            NewsEmptyState {
                                filtered: !query.is_empty() || category != "all",
                                page,
                                total,
                                total_pages,
                                query,
                                category,
                            }
                        },
                        NewsListOutcome::Error { code } => rsx! {
                            NewsErrorState { code, retry_href }
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn NewsFilters(initial_query: String, initial_category: String) -> Element {
    let category = initial_category;
    rsx! {
        form {
            id: "news-filters-form",
            class: "card card-glass news-filters",
            method: "get",
            action: "/news",
            role: "search",
            div { class: "card-body flex flex-col md:flex-row gap-4 items-stretch md:items-end",
                div { class: "field flex-1",
                    label { class: "field-label", r#for: "news-q", "Search" }
                    input {
                        class: "input",
                        id: "news-q",
                        name: "q",
                        r#type: "search",
                        maxlength: "200",
                        placeholder: "Search articles…",
                        value: initial_query,
                    }
                }
                div { class: "field md:w-48",
                    label { class: "field-label", r#for: "news-category", "Category" }
                    select {
                        class: "input",
                        id: "news-category",
                        name: "category",
                        option { value: "all", selected: category == "all", "All" }
                        option { value: "updates", selected: category == "updates", "Updates" }
                        option { value: "engineering", selected: category == "engineering", "Engineering" }
                        option { value: "product", selected: category == "product", "Product" }
                    }
                }
                button { class: "btn btn-outline", r#type: "submit",
                    Icon { name: "search".to_string(), size: Some(16) }
                    " Search"
                }
            }
        }
    }
}

#[component]
fn NewsList(
    posts: Vec<NewsPost>,
    total: u64,
    page: u32,
    total_pages: u32,
    query: String,
    category: String,
) -> Element {
    let featured_index = posts.iter().position(|post| post.featured);
    let featured_post = featured_index.map(|index| posts[index].clone());
    let normal_posts: Vec<NewsPost> = posts
        .into_iter()
        .enumerate()
        .filter_map(|(index, post)| (Some(index) != featured_index).then_some(post))
        .collect();
    let normal_grid_class = if featured_post.is_some() {
        "news-list-grid grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6 mt-8"
    } else {
        "news-list-grid grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6"
    };

    rsx! {
        section { class: "news-list-section mt-8", aria_label: "News articles",
            if let Some(post) = featured_post {
                NewsFeaturedCard { post }
            }
            if !normal_posts.is_empty() {
                div { class: normal_grid_class,
                    for post in normal_posts {
                        ArticleCard { post }
                    }
                }
            }
            p { class: "mt-6 text-xs text-muted-foreground text-center news-list-count", aria_live: "polite",
                {
                    let noun = if total == 1 { "article" } else { "articles" };
                    format!("{total} {noun}")
                }
            }
            NewsPagination { page, total_pages, query, category }
        }
    }
}

#[component]
fn NewsFeaturedCard(post: NewsPost) -> Element {
    rsx! {
        a { class: "group block news-featured-card", href: "/news/{post.slug}",
            article { class: "relative rounded-3xl overflow-hidden h-[360px] sm:h-[480px] bg-gradient-to-br from-purple-500/20 via-cyan-500/10 to-slate-900/50",
                if let Some(cover) = &post.cover_image_url {
                    img { class: "absolute inset-0 w-full h-full object-cover", src: cover, alt: "", loading: "eager" }
                } else {
                    div { class: "absolute top-8 right-8 opacity-10", Icon { name: "newspaper".to_string(), size: Some(96) } }
                }
                div { class: "absolute inset-0 bg-gradient-to-t from-black/85 via-black/30 to-transparent" }
                div { class: "absolute bottom-0 left-0 right-0 p-6 sm:p-10",
                    div { class: "flex flex-wrap gap-2 mb-4",
                        span { class: "px-3 py-1 rounded-full text-xs font-semibold bg-cyan-500/20 text-cyan-500 border border-cyan-500/30", "Featured" }
                        for tag in post.tags.iter().take(2) {
                            span { class: "px-3 py-1 rounded-full text-xs font-medium bg-white/10 text-white/80", "{tag}" }
                        }
                    }
                    h2 { class: "text-2xl sm:text-3xl font-extrabold text-white mb-3 group-hover:text-cyan-500 transition-colors line-clamp-2", "{post.title}" }
                    if !post.summary.is_empty() {
                        p { class: "text-white/70 text-sm sm:text-base line-clamp-2 max-w-3xl", "{post.summary}" }
                    }
                    div { class: "mt-5 flex items-center gap-4",
                        if let Some(date) = &post.published_at { span { class: "text-xs text-white/60", "{date}" } }
                        span { class: "flex items-center gap-1.5 text-xs font-semibold text-cyan-500", "Read article " Icon { name: "arrow-right".to_string(), size: Some(14) } }
                    }
                }
            }
        }
    }
}

#[component]
fn ArticleCard(post: NewsPost) -> Element {
    rsx! {
        a { class: "group block h-full news-article-card", href: "/news/{post.slug}",
            article { class: "rounded-2xl bg-card border border-border/20 overflow-hidden hover:border-cyan-500/40 transition-all h-full flex flex-col",
                div { class: "relative w-full h-48 overflow-hidden bg-gradient-to-br from-purple-500/15 via-cyan-500/5 to-transparent flex items-center justify-center",
                    if let Some(cover) = &post.cover_image_url {
                        img { class: "w-full h-full object-cover", src: cover, alt: "", loading: "lazy" }
                    } else {
                        Icon { name: "newspaper".to_string(), size: Some(40) }
                    }
                }
                div { class: "p-5 flex flex-col flex-1",
                    if !post.tags.is_empty() {
                        div { class: "flex flex-wrap gap-1.5 mb-3",
                            for tag in post.tags.iter().take(2) {
                                span { class: "px-2 py-0.5 rounded-full text-xs font-medium bg-cyan-500/10 text-cyan-500", "{tag}" }
                            }
                        }
                    }
                    h2 { class: "font-bold group-hover:text-cyan-500 transition-colors line-clamp-2 mb-2 leading-snug", "{post.title}" }
                    if !post.summary.is_empty() {
                        p { class: "text-sm text-muted-foreground line-clamp-3 flex-1 leading-relaxed", "{post.summary}" }
                    }
                    div { class: "mt-4 pt-4 border-t border-border/10 flex items-center justify-between",
                        if let Some(date) = &post.published_at { span { class: "text-xs text-muted-foreground", "{date}" } }
                        span { class: "text-xs text-cyan-500 font-semibold flex items-center gap-1", "Read " Icon { name: "arrow-right".to_string(), size: Some(12) } }
                    }
                }
            }
        }
    }
}

#[component]
fn NewsEmptyState(
    filtered: bool,
    page: u32,
    total: u64,
    total_pages: u32,
    query: String,
    category: String,
) -> Element {
    let title = if filtered || total > 0 {
        "No matching articles"
    } else {
        "No published articles yet"
    };
    let message = if total > 0 && page > total_pages {
        "This page has no articles. Use Previous to return to an available page."
    } else if filtered {
        "Try a different search or category."
    } else {
        "Published updates will appear here when they are available."
    };
    let recovery_href =
        (total > 0 && page > total_pages).then(|| page_href(total_pages, &query, &category));
    rsx! {
        section { class: "flex flex-col items-center justify-center py-24 gap-5 news-empty-state", aria_live: "polite",
            div { class: "p-6 rounded-full bg-gradient-to-br from-purple-500/10 via-cyan-500/5 to-transparent border border-border/20",
                Icon { name: "newspaper".to_string(), size: Some(40) }
            }
            div { class: "text-center",
                h2 { class: "font-semibold text-lg", "{title}" }
                p { class: "text-sm text-muted-foreground mt-1.5 max-w-xs leading-relaxed", "{message}" }
            }
            if let Some(href) = recovery_href {
                a { class: "btn btn-outline", href, "Previous page" }
            }
        }
    }
}

#[component]
fn NewsErrorState(code: String, retry_href: String) -> Element {
    let invalid_query = code == "invalid_news_query";
    rsx! {
        section { class: "news-error-state card card-glass mt-8 p-8 sm:p-12 text-center", role: "alert",
            div { class: "mx-auto mb-4 text-cyan-500", Icon { name: "triangle-alert".to_string(), size: Some(36) } }
            h2 { class: "text-xl font-bold",
                if invalid_query { "These news filters are invalid" } else { "News is temporarily unavailable" }
            }
            p { class: "mt-2 text-sm text-muted-foreground max-w-md mx-auto",
                if invalid_query {
                    "Reset the filters and try again."
                } else {
                    "We could not load published articles. No cached or sample content is being shown."
                }
            }
            div { class: "mt-6 flex flex-wrap justify-center gap-3",
                a { class: "btn btn-primary", href: retry_href, "Try again" }
                if invalid_query {
                    a { class: "btn btn-outline", href: "/news", "Reset filters" }
                }
            }
        }
    }
}

fn encode_query_component(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

fn page_href(page: u32, query: &str, category: &str) -> String {
    let mut pairs = Vec::new();
    if !query.is_empty() {
        pairs.push(format!("q={}", encode_query_component(query)));
    }
    if category != "all" {
        pairs.push(format!("category={}", encode_query_component(category)));
    }
    if page > 1 {
        pairs.push(format!("page={page}"));
    }
    if pairs.is_empty() {
        "/news".to_string()
    } else {
        format!("/news?{}", pairs.join("&"))
    }
}

#[component]
fn NewsPagination(page: u32, total_pages: u32, query: String, category: String) -> Element {
    if total_pages <= 1 {
        return rsx! { Fragment {} };
    }
    rsx! {
        nav { class: "flex items-center justify-center gap-3 mt-12 news-pagination", aria_label: "News pages",
            if page > 1 {
                a { class: "flex items-center gap-1.5 px-4 py-2 rounded-xl text-sm font-medium border border-border/20 bg-card hover:bg-muted/50 transition-colors news-pagination-prev",
                    href: page_href(page - 1, &query, &category),
                    Icon { name: "arrow-left".to_string(), size: Some(14) }
                    " Previous"
                }
            } else {
                span { class: "flex items-center gap-1.5 px-4 py-2 rounded-xl text-sm font-medium border border-border/20 opacity-40", aria_disabled: "true",
                    Icon { name: "arrow-left".to_string(), size: Some(14) }
                    " Previous"
                }
            }
            span { class: "px-4 py-2 rounded-xl text-sm text-muted-foreground bg-muted/20 border border-border/10", aria_current: "page",
                "{page} of {total_pages}"
            }
            if page < total_pages {
                a { class: "flex items-center gap-1.5 px-4 py-2 rounded-xl text-sm font-medium border border-border/20 bg-card hover:bg-muted/50 transition-colors news-pagination-next",
                    href: page_href(page + 1, &query, &category),
                    "Next "
                    Icon { name: "arrow-right".to_string(), size: Some(14) }
                }
            } else {
                span { class: "flex items-center gap-1.5 px-4 py-2 rounded-xl text-sm font-medium border border-border/20 opacity-40", aria_disabled: "true",
                    "Next "
                    Icon { name: "arrow-right".to_string(), size: Some(14) }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(outcome: serde_json::Value, query: &str) -> PageContext {
        let mut ctx = PageContext {
            path: "/news".to_string(),
            query: query.to_string(),
            ..Default::default()
        };
        ctx.params
            .insert("data_news".to_string(), outcome.to_string());
        ctx
    }

    fn article(title: &str) -> serde_json::Value {
        serde_json::json!({
            "id": "article-1",
            "slug": "live-article",
            "title": title,
            "summary": "Only upstream content",
            "cover_image_url": null,
            "author": null,
            "published_at": "July 22, 2026",
            "read_time": null,
            "tags": ["engineering"],
            "featured": true
        })
    }

    fn article_with_feature(title: &str, slug: &str, featured: bool) -> serde_json::Value {
        let mut value = article(title);
        value["id"] = serde_json::json!(format!("id-{slug}"));
        value["slug"] = serde_json::json!(slug);
        value["featured"] = serde_json::json!(featured);
        value
    }

    fn render_articles(articles: Vec<serde_json::Value>) -> String {
        let total = articles.len();
        let (_, element) = render(&context(
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
            "",
        ));
        dioxus_ssr::render_element(element)
    }

    #[test]
    fn no_featured_posts_render_only_normal_cards_without_badge() {
        let html = render_articles(vec![
            article_with_feature("Alpha update", "alpha-update", false),
            article_with_feature("Beta update", "beta-update", false),
        ]);
        assert!(!html.contains("news-featured-card"));
        assert!(!html.contains("Featured"));
        assert_eq!(html.matches("news-article-card").count(), 2);
        assert!(
            html.find("Alpha update").expect("alpha must render")
                < html.find("Beta update").expect("beta must render")
        );

        let single = render_articles(vec![article_with_feature(
            "Only normal",
            "only-normal",
            false,
        )]);
        assert!(!single.contains("news-featured-card"));
        assert!(!single.contains("Featured"));
        assert_eq!(single.matches("news-article-card").count(), 1);
        assert_eq!(single.matches("Only normal").count(), 1);
    }

    #[test]
    fn featured_not_first_preserves_other_order_without_duplication() {
        let html = render_articles(vec![
            article_with_feature("Alpha normal", "alpha-normal", false),
            article_with_feature("Primary feature", "primary-feature", true),
            article_with_feature("Beta normal", "beta-normal", false),
            article_with_feature("Later feature", "later-feature", true),
        ]);
        assert_eq!(html.matches("news-featured-card").count(), 1);
        assert_eq!(html.matches("news-article-card").count(), 3);
        assert_eq!(html.matches("Featured").count(), 1);
        for title in [
            "Alpha normal",
            "Primary feature",
            "Beta normal",
            "Later feature",
        ] {
            assert_eq!(html.matches(title).count(), 1, "{title}");
        }
        let primary = html.find("Primary feature").expect("feature must render");
        let alpha = html.find("Alpha normal").expect("alpha must render");
        let beta = html.find("Beta normal").expect("beta must render");
        let later = html.find("Later feature").expect("later must render");
        assert!(primary < alpha);
        assert!(alpha < beta && beta < later);
    }

    #[test]
    fn single_featured_post_uses_featured_card() {
        let mut article = article_with_feature("Only feature", "only-feature", true);
        article["author"] = serde_json::json!("EPSX Editorial");
        article["read_time"] = serde_json::json!("7 min");
        let html = render_articles(vec![article]);
        assert_eq!(html.matches("news-featured-card").count(), 1);
        assert_eq!(html.matches("news-article-card").count(), 0);
        assert_eq!(html.matches("Featured").count(), 1);
        assert_eq!(html.matches("Only feature").count(), 1);
    }

    #[test]
    fn malformed_author_or_read_time_fails_closed() {
        let cases = [
            ("author", String::new()),
            ("author", "bad\nauthor".to_string()),
            ("author", "a".repeat(NEWS_AUTHOR_MAX_CHARS + 1)),
            ("read_time", " ".to_string()),
            ("read_time", "bad\tvalue".to_string()),
            ("read_time", "r".repeat(NEWS_READ_TIME_MAX_CHARS + 1)),
        ];
        for (field, value) in cases {
            let mut article =
                article_with_feature("Malformed metadata", "malformed-metadata", false);
            article[field] = serde_json::Value::String(value);
            let html = render_articles(vec![article]);
            assert!(html.contains("News is temporarily unavailable"), "{field}");
            assert!(!html.contains("news-article-card"), "{field}");
            assert!(!html.contains("Malformed metadata"), "{field}");
        }
    }

    #[test]
    fn ready_state_renders_only_supplied_articles_and_escapes_text() {
        let ctx = context(
            serde_json::json!({
                "state": "ready",
                "articles": [article("Live <script>alert(1)</script>")],
                "total": 1,
                "page": 1,
                "limit": 12,
                "total_pages": 1,
                "query": "",
                "category": "all"
            }),
            "",
        );
        let (_, element) = render(&ctx);
        let html = dioxus_ssr::render_element(element);
        assert!(html.contains("Live "));
        assert!(html.contains("alert(1)"));
        assert!(html.contains("July 22, 2026"));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(!html.contains("Strategic Roadmap and Future Capabilities"));
    }

    #[test]
    fn missing_or_malformed_data_is_an_error_without_a_sample_fallback() {
        let (_, missing) = render(&PageContext {
            path: "/news".to_string(),
            ..Default::default()
        });
        let missing = dioxus_ssr::render_element(missing);
        assert!(missing.contains("News is temporarily unavailable"));
        assert!(missing.contains("No cached or sample content"));

        let (_, malformed) = render(&context(
            serde_json::json!({
                "state": "ready",
                "articles": [{"slug": "missing-title"}],
                "total": 1,
                "page": 1,
                "limit": 12,
                "total_pages": 1,
                "query": "",
                "category": "all"
            }),
            "",
        ));
        assert!(dioxus_ssr::render_element(malformed).contains("News is temporarily unavailable"));

        let (_, zero_limit) = render(&context(
            serde_json::json!({
                "state": "empty",
                "total": 0,
                "page": 1,
                "limit": 0,
                "total_pages": 0,
                "query": "",
                "category": "all"
            }),
            "",
        ));
        assert!(dioxus_ssr::render_element(zero_limit).contains("News is temporarily unavailable"));

        let mut bad_date = article("Malformed date");
        bad_date["published_at"] = serde_json::json!("yesterday");
        let (_, bad_date) = render(&context(
            serde_json::json!({
                "state": "ready",
                "articles": [bad_date],
                "total": 1,
                "page": 1,
                "limit": 12,
                "total_pages": 1,
                "query": "",
                "category": "all"
            }),
            "",
        ));
        assert!(dioxus_ssr::render_element(bad_date).contains("News is temporarily unavailable"));

        let (_, unknown_category) = render(&context(
            serde_json::json!({
                "state": "empty",
                "total": 0,
                "page": 1,
                "limit": 12,
                "total_pages": 0,
                "query": "",
                "category": "security"
            }),
            "",
        ));
        assert!(dioxus_ssr::render_element(unknown_category)
            .contains("News is temporarily unavailable"));
    }

    #[test]
    fn empty_state_is_distinct_from_dependency_failure() {
        let (_, element) = render(&context(
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
        let html = dioxus_ssr::render_element(element);
        assert!(html.contains("No published articles yet"));
        assert!(!html.contains("temporarily unavailable"));
    }

    #[test]
    fn out_of_range_empty_page_has_filter_preserving_keyboard_recovery() {
        let (_, element) = render(&context(
            serde_json::json!({
                "state": "empty",
                "total": 13,
                "page": 100,
                "limit": 12,
                "total_pages": 2,
                "query": "rust & web",
                "category": "engineering"
            }),
            "q=rust%20%26%20web&category=engineering&page=100",
        ));
        let html = dioxus_ssr::render_element(element);
        assert!(html.contains("Previous page"));
        let expected = "/news?q=rust%20%26%20web&category=engineering&page=2";
        assert_eq!(page_href(2, "rust & web", "engineering"), expected);
        assert!(
            html.contains(expected)
                || html.contains(&expected.replace('&', "&amp;"))
                || html.contains(&expected.replace('&', "&#38;"))
        );
    }

    #[test]
    fn pagination_preserves_search_and_category_without_javascript_urls() {
        assert_eq!(
            page_href(2, "rust & web", "engineering"),
            "/news?q=rust%20%26%20web&category=engineering&page=2"
        );
        let pagination = dioxus_ssr::render_element(rsx! {
            NewsPagination {
                page: 2,
                total_pages: 3,
                query: "rust & web".to_string(),
                category: "engineering".to_string(),
            }
        });
        assert!(pagination.contains("page=3"));
        assert!(!pagination.contains("javascript:"));
        assert!(pagination.contains("aria-label=\"News pages\""));
    }
}

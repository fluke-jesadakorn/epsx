//! Public news detail backed by the content service's slug-scoped outcome.
//!
//! Unknown content is a real not-found state. Dependency and envelope failures
//! are explicit retryable errors; neither path synthesizes an article.

use super::{PageContext, PageMeta, PageStatus};
use crate::layout::main_layout::MainLayout;
use crate::primitives::*;
use dioxus::prelude::*;

#[derive(Clone, Debug, serde::Deserialize, PartialEq, Eq)]
struct NewsArticle {
    id: Option<String>,
    slug: String,
    title: String,
    #[serde(default)]
    summary: Option<String>,
    body: String,
    cover_image_url: Option<String>,
    author: Option<String>,
    published_at: Option<String>,
    read_time: Option<String>,
    tags: Vec<String>,
}

#[derive(Clone, Debug, serde::Deserialize, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
enum NewsDetailOutcome {
    Ready { article: NewsArticle },
    NotFound,
    Error { code: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BodyChunk {
    heading: Option<String>,
    paragraphs: Vec<String>,
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

fn safe_text(value: &str, max: usize) -> bool {
    value.chars().count() <= max && !value.chars().any(char::is_control)
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

fn valid_article(article: &NewsArticle, expected_slug: &str) -> bool {
    safe_slug(&article.slug)
        && article.slug == expected_slug
        && !article.title.trim().is_empty()
        && safe_text(&article.title, 300)
        && article
            .summary
            .as_deref()
            .is_none_or(|summary| safe_text(summary, 2_000))
        && !article.body.trim().is_empty()
        && article.body.chars().count() <= 500_000
        && article.tags.len() <= 32
        && article
            .tags
            .iter()
            .all(|tag| !tag.trim().is_empty() && safe_text(tag, 64))
        && article.cover_image_url.as_deref().is_none_or(safe_cover)
        && article
            .published_at
            .as_deref()
            .is_none_or(valid_display_date)
}

fn parse_outcome(ctx: &PageContext, slug: &str) -> NewsDetailOutcome {
    if !safe_slug(slug) {
        return NewsDetailOutcome::NotFound;
    }
    let Some(raw) = ctx.params.get("data_news_post") else {
        return NewsDetailOutcome::Error {
            code: "missing_content_outcome".to_string(),
        };
    };
    let Ok(outcome) = serde_json::from_str::<NewsDetailOutcome>(raw) else {
        return NewsDetailOutcome::Error {
            code: "malformed_content_response".to_string(),
        };
    };
    match outcome {
        NewsDetailOutcome::Ready { article } if valid_article(&article, slug) => {
            NewsDetailOutcome::Ready { article }
        }
        NewsDetailOutcome::NotFound => NewsDetailOutcome::NotFound,
        NewsDetailOutcome::Error { code } if !code.is_empty() && safe_text(&code, 64) => {
            NewsDetailOutcome::Error { code }
        }
        _ => NewsDetailOutcome::Error {
            code: "malformed_content_response".to_string(),
        },
    }
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let slug = ctx.params.get("slug").cloned().unwrap_or_default();
    let outcome = parse_outcome(ctx, &slug);
    let mut meta = PageMeta::marketing("News article");
    match &outcome {
        NewsDetailOutcome::Ready { article } => {
            meta.title = format!("{} — EPSX News", article.title);
            meta.description = article
                .summary
                .clone()
                .filter(|summary| !summary.trim().is_empty())
                .unwrap_or_else(|| article.title.clone());
        }
        NewsDetailOutcome::NotFound => {
            meta.title = "Article Not Found — EPSX".to_string();
            meta.description = "The requested published news article was not found.".to_string();
            meta.status = PageStatus::NotFound;
        }
        NewsDetailOutcome::Error { .. } => {
            meta.title = "News unavailable — EPSX".to_string();
            meta.description = "The requested news article could not be loaded.".to_string();
        }
    }
    let retry_href = if ctx.query.is_empty() {
        ctx.path.clone()
    } else {
        format!("{}?{}", ctx.path, ctx.query)
    };

    (
        meta,
        rsx! {
            MainLayout { ctx: ctx.clone(),
                match outcome {
                    NewsDetailOutcome::Ready { article } => rsx! { NewsArticleView { article } },
                    NewsDetailOutcome::NotFound => rsx! { NewsNotFound {} },
                    NewsDetailOutcome::Error { .. } => rsx! { NewsDetailError { retry_href } },
                }
            }
        },
    )
}

#[component]
fn NewsArticleView(article: NewsArticle) -> Element {
    let chunks = body_to_chunks(&article.body);
    let read_time = article
        .read_time
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| format!("{} min", read_minutes(&article.body)));
    rsx! {
        article { class: "news-detail-body",
            section { class: "relative w-full overflow-hidden isolate news-detail-hero",
                if let Some(cover) = &article.cover_image_url {
                    img { class: "absolute inset-0 w-full h-full object-cover", src: cover, alt: "" }
                    div { class: "absolute inset-0 bg-gradient-to-t from-black/80 via-black/40 to-black/20" }
                } else {
                    div { class: "absolute inset-0 bg-gradient-to-br from-cyan-500/8 via-background to-purple-500/8" }
                }
                div { class: "relative z-10 max-w-4xl mx-auto px-4 sm:px-6 pt-8 pb-12 flex flex-col min-h-[240px] sm:min-h-[300px]",
                    a { class: "inline-flex items-center gap-2 text-sm mb-auto transition-colors news-detail-back", href: "/news",
                        Icon { name: "arrow-left".to_string(), size: Some(16) }
                        " Back to News"
                    }
                    div {
                        if !article.tags.is_empty() {
                            div { class: "flex flex-wrap gap-2 mb-5",
                                for tag in article.tags.iter() {
                                    span { class: "px-3 py-1 rounded-full text-[11px] font-bold tracking-[0.15em] uppercase bg-cyan-500/15 text-cyan-500 border border-cyan-500/25", "{tag}" }
                                }
                            }
                        }
                        h1 { class: "text-3xl sm:text-4xl lg:text-[2.75rem] font-extrabold leading-[1.1] tracking-tight mb-5", "{article.title}" }
                        div { class: "flex flex-wrap items-center gap-5 text-sm text-muted-foreground",
                            if let Some(date) = &article.published_at {
                                span { class: "flex items-center gap-1.5", Icon { name: "calendar".to_string(), size: Some(14) } " {date}" }
                            }
                            span { class: "flex items-center gap-1.5", Icon { name: "clock".to_string(), size: Some(14) } " {read_time} read" }
                            if let Some(author) = &article.author {
                                span { class: "flex items-center gap-1.5", Icon { name: "user".to_string(), size: Some(14) } " {author}" }
                            }
                        }
                    }
                }
            }
            div { class: "h-[3px] news-detail-accent bg-gradient-to-r from-cyan-500 via-purple-500 to-cyan-500" }
            div { class: "max-w-3xl mx-auto px-4 sm:px-6 pt-12 pb-20 news-detail-content",
                div { class: "prose prose-lg prose-neutral max-w-none",
                    for chunk in chunks.iter() {
                        if let Some(heading) = &chunk.heading {
                            h2 { class: "text-2xl font-bold mt-12 mb-4 pb-3 border-b border-cyan-500/20 news-detail-h2", "{heading}" }
                        }
                        for paragraph in chunk.paragraphs.iter() {
                            p { class: "leading-[1.8] text-muted-foreground news-detail-p", "{paragraph}" }
                        }
                    }
                }
                div { class: "mt-16 pt-8 border-t border-border/20 news-detail-footer",
                    a { class: "inline-flex items-center gap-3 px-5 py-3 rounded-xl text-sm font-medium text-muted-foreground hover:text-foreground bg-card/50 hover:bg-card border border-border/20 hover:border-border/40 transition-all group news-detail-back-link", href: "/news",
                        Icon { name: "arrow-left".to_string(), size: Some(16) }
                        " Back to all articles"
                    }
                }
            }
        }
    }
}

#[component]
fn NewsNotFound() -> Element {
    rsx! {
        main { class: "news-detail-not-found container page-content flex min-h-[60vh] items-center justify-center",
            section { class: "card card-glass max-w-xl p-8 sm:p-12 text-center", aria_labelledby: "news-not-found-title",
                div { class: "mx-auto mb-4 text-cyan-500", Icon { name: "newspaper".to_string(), size: Some(40) } }
                h1 { id: "news-not-found-title", class: "text-2xl font-bold", "Article not found" }
                p { class: "mt-3 text-sm text-muted-foreground", "This article is not available as published content." }
                a { class: "btn btn-primary mt-6", href: "/news", "Browse all news" }
            }
        }
    }
}

#[component]
fn NewsDetailError(retry_href: String) -> Element {
    rsx! {
        main { class: "news-detail-error container page-content flex min-h-[60vh] items-center justify-center",
            section { class: "card card-glass max-w-xl p-8 sm:p-12 text-center", role: "alert",
                div { class: "mx-auto mb-4 text-cyan-500", Icon { name: "triangle-alert".to_string(), size: Some(40) } }
                h1 { class: "text-2xl font-bold", "Article temporarily unavailable" }
                p { class: "mt-3 text-sm text-muted-foreground", "We could not load this published article. No default article is being shown." }
                div { class: "mt-6 flex flex-wrap justify-center gap-3",
                    a { class: "btn btn-primary", href: retry_href, "Try again" }
                    a { class: "btn btn-outline", href: "/news", "Back to news" }
                }
            }
        }
    }
}

fn read_minutes(body: &str) -> usize {
    body.split_whitespace().count().div_ceil(200).max(1)
}

fn strip_markup(value: &str) -> String {
    let mut plain = String::new();
    let mut in_tag = false;
    for character in value.chars() {
        match character {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => plain.push(character),
            _ => {}
        }
    }
    plain.trim().to_string()
}

fn body_to_chunks(body: &str) -> Vec<BodyChunk> {
    let mut chunks = Vec::new();
    let mut paragraphs = Vec::new();
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() {
            if !paragraphs.is_empty() {
                chunks.push(BodyChunk {
                    heading: None,
                    paragraphs: std::mem::take(&mut paragraphs),
                });
            }
            continue;
        }
        let heading = line
            .strip_prefix("## ")
            .or_else(|| line.strip_prefix("# "))
            .map(str::trim)
            .map(str::to_string)
            .or_else(|| {
                (line.starts_with("<h1") || line.starts_with("<h2")).then(|| strip_markup(line))
            });
        if let Some(heading) = heading.filter(|value| !value.is_empty()) {
            if !paragraphs.is_empty() {
                chunks.push(BodyChunk {
                    heading: None,
                    paragraphs: std::mem::take(&mut paragraphs),
                });
            }
            chunks.push(BodyChunk {
                heading: Some(heading),
                paragraphs: Vec::new(),
            });
        } else {
            let paragraph = strip_markup(line);
            if !paragraph.is_empty() {
                paragraphs.push(paragraph);
            }
        }
    }
    if !paragraphs.is_empty() {
        chunks.push(BodyChunk {
            heading: None,
            paragraphs,
        });
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(slug: &str, outcome: serde_json::Value) -> PageContext {
        let mut ctx = PageContext {
            path: format!("/news/{slug}"),
            ..Default::default()
        };
        ctx.params.insert("slug".to_string(), slug.to_string());
        ctx.params
            .insert("data_news_post".to_string(), outcome.to_string());
        ctx
    }

    fn article(title: &str, body: &str) -> serde_json::Value {
        serde_json::json!({
            "id": "article-1",
            "slug": "live-article",
            "title": title,
            "summary": "Live article summary",
            "body": body,
            "cover_image_url": null,
            "author": null,
            "published_at": "July 22, 2026",
            "read_time": null,
            "tags": ["engineering"]
        })
    }

    #[test]
    fn ready_article_uses_live_title_body_metadata_and_safe_text_rendering() {
        let ctx = context(
            "live-article",
            serde_json::json!({
                "state": "ready",
                "article": article("Live <script>alert(1)</script>", "## Update\n\n<p>Safe <img src=x onerror=alert(1)> body</p>")
            }),
        );
        let (meta, element) = render(&ctx);
        let html = dioxus_ssr::render_element(element);
        assert_eq!(meta.title, "Live <script>alert(1)</script> — EPSX News");
        assert_eq!(meta.description, "Live article summary");
        assert!(html.contains("Live "));
        assert!(html.contains("alert(1)"));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("Safe  body"));
        assert!(html.contains("July 22, 2026"));
        assert!(!html.contains("onerror="));
        assert!(!html.contains("Related articles"));
        assert!(!html.contains("Welcome to EPSX"));
    }

    #[test]
    fn not_found_is_explicit_and_never_synthesizes_an_article() {
        let (meta, element) = render(&context(
            "missing-article",
            serde_json::json!({"state": "not_found"}),
        ));
        let html = dioxus_ssr::render_element(element);
        assert_eq!(meta.status, PageStatus::NotFound);
        assert!(html.contains("Article not found"));
        assert!(!html.contains("coming soon"));
        assert!(!html.contains("Welcome to EPSX"));
    }

    #[test]
    fn missing_malformed_or_slug_mismatched_outcome_renders_retryable_error() {
        let mut missing = PageContext {
            path: "/news/live-article".to_string(),
            ..Default::default()
        };
        missing
            .params
            .insert("slug".to_string(), "live-article".to_string());
        let (_, element) = render(&missing);
        assert!(dioxus_ssr::render_element(element).contains("temporarily unavailable"));

        let (_, mismatched) = render(&context(
            "live-article",
            serde_json::json!({
                "state": "ready",
                "article": {
                    "id": null,
                    "slug": "another-article",
                    "title": "Wrong owner",
                    "summary": null,
                    "body": "Body",
                    "cover_image_url": null,
                    "author": null,
                    "published_at": null,
                    "read_time": null,
                    "tags": []
                }
            }),
        ));
        let html = dioxus_ssr::render_element(mismatched);
        assert!(html.contains("temporarily unavailable"));
        assert!(!html.contains("Wrong owner"));

        let mut malformed_date = article("Malformed date", "Body");
        malformed_date["published_at"] = serde_json::json!("not-a-date");
        let (_, malformed_date) = render(&context(
            "live-article",
            serde_json::json!({"state": "ready", "article": malformed_date}),
        ));
        assert!(dioxus_ssr::render_element(malformed_date).contains("temporarily unavailable"));
    }
}

//! Sub-components extracted from `pages/news.rs` during
//! Wave 6C Track E (1:1 user-side component parity).
//!
//! Five sub-components to lift: `NewsFilters`, `NewsFeaturedCard`,
//! `ArticleCard`, `NewsEmptyState`, `NewsPagination`. The page-level
//! `NewsPost` struct is also lifted here so the components can be
//! re-imported by the future admin `news` page (Track D admin side
//! has the same struct shape and will mirror this).

use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;

/// `NewsPost` — the data shape the news list + detail pages
/// consume. Mirrors the source's `NewsPost` type from
/// `apps-old/frontend/shared/api/news.ts` (slug, title, excerpt,
/// author, published_at, read_time, tags, cover_image_url).
/// `pub` so the page file (and any future admin surface) can
/// reuse it.
#[derive(Clone, Debug, serde::Deserialize, PartialEq)]
pub struct NewsPost {
    #[serde(default)] pub slug: String,
    #[serde(default)] pub title: String,
    #[serde(default)] pub excerpt: String,
    #[serde(default)] pub author: String,
    #[serde(default)] pub published_at: String,
    #[serde(default)] pub read_time: String,
    #[serde(default)] pub tags: Vec<String>,
    #[serde(default)] pub cover_image_url: Option<String>,
}

/// `NewsFilters` — category select, date range, search input.
/// Static form (no client-side filtering yet — that's a Wave 7
/// enhancement); the section marker is here so tests can assert
/// the surface area.
#[component]
pub fn NewsFilters() -> Element {
    rsx! {
        div { class: "card card-glass news-filters",
            div { class: "card-body flex flex-col md:flex-row gap-4 items-stretch md:items-end",
                div { class: "field flex-1",
                    label { class: "field-label", "Search" }
                    input { class: "input", r#type: "search", placeholder: "Search articles…" }
                }
                div { class: "field md:w-48",
                    label { class: "field-label", "Category" }
                    SelectField { name: "category".to_string(), options: vec![("all".to_string(), "All".to_string()), ("updates".to_string(), "Updates".to_string()), ("engineering".to_string(), "Engineering".to_string()), ("product".to_string(), "Product".to_string())], value: Some("all".to_string()), required: false, label: None, help: None, error: None, placeholder: None, onchange: None }
                }
                div { class: "field md:w-48",
                    label { class: "field-label", "Date range" }
                    SelectField { name: "range".to_string(), options: vec![("all".to_string(), "All time".to_string()), ("7d".to_string(), "Last 7 days".to_string()), ("30d".to_string(), "Last 30 days".to_string()), ("90d".to_string(), "Last 90 days".to_string())], value: Some("all".to_string()), required: false, label: None, help: None, error: None, placeholder: None, onchange: None }
                }
                button { class: "btn btn-outline", r#type: "button", Icon { name: "search".to_string(), size: Some(16) } " Search" }
            }
        }
    }
}

/// Featured card — large hero card with optional cover image,
/// "Featured" badge, tags, title, summary, and "Read article" CTA.
/// Mirrors `FeaturedCard` in `news-list.tsx`.
#[component]
pub fn NewsFeaturedCard(post: NewsPost) -> Element {
    rsx! {
        a { class: "group block news-featured-card", href: "/news/{post.slug}",
            div { class: "relative rounded-3xl overflow-hidden h-[360px] sm:h-[480px] bg-gradient-to-br from-purple-500/20 via-cyan-500/10 to-slate-900/50",
                div { class: "absolute top-8 right-8 opacity-10", Icon { name: "newspaper".to_string(), size: Some(96) } }
                div { class: "absolute inset-0 bg-gradient-to-t from-black/85 via-black/30 to-transparent" }
                div { class: "absolute bottom-0 left-0 right-0 p-6 sm:p-10",
                    div { class: "flex flex-wrap gap-2 mb-4",
                        span { class: "px-3 py-1 rounded-full text-xs font-semibold bg-cyan-500/20 text-cyan-500 backdrop-blur-sm border border-cyan-500/30", "Featured" }
                    }
                    h2 { class: "text-2xl sm:text-3xl font-extrabold text-white mb-3 group-hover:text-cyan-500 transition-colors line-clamp-2",
                        "{post.title}"
                    }
                    p { class: "text-white/70 text-sm sm:text-base line-clamp-2 max-w-3xl", "{post.excerpt}" }
                    div { class: "mt-5 flex items-center gap-4",
                        span { class: "text-xs text-white/40", "{post.published_at}" }
                        span { class: "flex items-center gap-1.5 text-xs font-semibold text-cyan-500 group-hover:gap-2.5 transition-all", "Read article " Icon { name: "arrow-right".to_string(), size: Some(14) } }
                    }
                }
            }
        }
    }
}

/// Article card — smaller card in the grid layout. Mirrors
/// `ArticleCard` in `news-list.tsx`.
#[component]
pub fn ArticleCard(post: NewsPost) -> Element {
    rsx! {
        a { class: "group block h-full news-article-card", href: "/news/{post.slug}",
            article { class: "rounded-2xl bg-card border border-border/20 overflow-hidden hover:border-cyan-500/40 transition-all h-full flex flex-col",
                div { class: "relative w-full h-48 overflow-hidden bg-gradient-to-br from-purple-500/15 via-cyan-500/5 to-transparent flex items-center justify-center",
                    Icon { name: "newspaper".to_string(), size: Some(40) }
                }
                div { class: "p-5 flex flex-col flex-1",
                    h2 { class: "font-bold group-hover:text-cyan-500 transition-colors line-clamp-2 mb-2 leading-snug", "{post.title}" }
                    p { class: "text-sm text-muted-foreground line-clamp-3 flex-1 leading-relaxed", "{post.excerpt}" }
                    div { class: "mt-4 pt-4 border-t border-border/10 flex items-center justify-between",
                        span { class: "text-xs text-muted-foreground", "{post.published_at}" }
                        span { class: "text-xs text-cyan-500 font-semibold opacity-0 group-hover:opacity-100 transition-opacity flex items-center gap-1", "Read " Icon { name: "arrow-right".to_string(), size: Some(12) } }
                    }
                }
            }
        }
    }
}

/// Empty state — when there are no articles, show a centered
/// newspaper icon + "No articles yet" message. Mirrors
/// `EmptyState` in `news-list.tsx`.
#[component]
pub fn NewsEmptyState() -> Element {
    rsx! {
        div { class: "flex flex-col items-center justify-center py-24 gap-5 news-empty-state",
            div { class: "p-6 rounded-full bg-gradient-to-br from-purple-500/10 via-cyan-500/5 to-transparent border border-border/20",
                Icon { name: "newspaper".to_string(), size: Some(40) }
            }
            div { class: "text-center",
                p { class: "font-semibold text-lg", "No articles yet" }
                p { class: "text-sm text-muted-foreground mt-1.5 max-w-xs leading-relaxed",
                    "Check back soon for updates, announcements, and insights from the EPSX team."
                }
            }
        }
    }
}

/// Pagination — Previous / N of M / Next. Mirrors `Pagination` in
/// `news-list.tsx`. Hidden when there is only one page (renders
/// an empty `Fragment`).
#[component]
pub fn NewsPagination(page: usize, total_pages: usize) -> Element {
    if total_pages <= 1 { return rsx! { Fragment {} }; }
    rsx! {
        div { class: "flex items-center justify-center gap-3 mt-12 news-pagination",
            a { class: "flex items-center gap-1.5 px-4 py-2 rounded-xl text-sm font-medium border border-border/20 bg-card hover:bg-muted/50 transition-colors news-pagination-prev",
                href: if page == 1 { "javascript:void(0)".to_string() } else { format!("/news?page={}", page - 1) },
                Icon { name: "arrow-left".to_string(), size: Some(14) }
                " Previous"
            }
            span { class: "px-4 py-2 rounded-xl text-sm text-muted-foreground bg-muted/20 border border-border/10",
                "{page} of {total_pages}"
            }
            a { class: "flex items-center gap-1.5 px-4 py-2 rounded-xl text-sm font-medium border border-border/20 bg-card hover:bg-muted/50 transition-colors news-pagination-next",
                href: format!("/news?page={}", page + 1),
                " Next"
                Icon { name: "arrow-right".to_string(), size: Some(14) }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_post(slug: &str, title: &str) -> NewsPost {
        NewsPost {
            slug: slug.to_string(),
            title: title.to_string(),
            excerpt: "excerpt".to_string(),
            author: "EPSX Team".to_string(),
            published_at: "2024-09-15".to_string(),
            read_time: "3 min".to_string(),
            tags: vec!["Update".to_string()],
            cover_image_url: None,
        }
    }

    /// Wave 6C Track E — `test_render_smoke` for the extracted
    /// news sub-components. Asserts each one carries its
    /// section-marker class + sample data.
    #[test]
    fn news_subcomponents_render_smoke() {
        // NewsFilters
        let el = rsx! { NewsFilters {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("news-filters"), "NewsFilters missing section-marker");
        assert!(html.contains("Search"));

        // NewsFeaturedCard
        let post = make_post("welcome-to-epsx", "Welcome to EPSX");
        let el = rsx! { NewsFeaturedCard { post } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("news-featured-card"), "NewsFeaturedCard missing section-marker");
        assert!(html.contains("Featured"));
        assert!(html.contains("Welcome to EPSX"));

        // ArticleCard
        let post = make_post("test-slug", "Test article");
        let el = rsx! { ArticleCard { post } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("news-article-card"), "ArticleCard missing section-marker");
        assert!(html.contains("Test article"));

        // NewsEmptyState
        let el = rsx! { NewsEmptyState {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("news-empty-state"), "NewsEmptyState missing section-marker");
        assert!(html.contains("No articles yet"));

        // NewsPagination
        let el = rsx! { NewsPagination { page: 2, total_pages: 5 } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("news-pagination"), "NewsPagination missing section-marker");
        assert!(html.contains("2 of 5"));

        // NewsPagination with total_pages <= 1 renders nothing
        let el = rsx! { NewsPagination { page: 1, total_pages: 1 } };
        let html = dioxus_ssr::render_element(el);
        assert!(!html.contains("news-pagination"), "NewsPagination should be hidden when total_pages <= 1");
    }
}

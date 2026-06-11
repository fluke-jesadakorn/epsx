//! /news — news article listing.
//!
//! Wave 6A Track D — port of `apps-old/frontend/app/news/page.tsx` +
//! `components/news/news-list.tsx`.
//!
//! Section coverage (matches design doc §"Track D — news"):
//! - `NewsHeader` — "EPSX Platform" badge + gradient "News & Updates"
//!   h1 + total articles count
//! - `NewsFilters` — category, date range, search input
//! - `NewsList` — featured card (1) + grid of `ArticleCard`s (rest)
//!   + pagination + empty state
//!
//! The Next.js source uses `getPublicNews` server action; we accept
//! the same payload shape via `ctx.params["data_news"]` and fall
//! back to the static 3-post default when none is provided — same
//! pattern Wave 5 uses for marketing pages.

use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::auth::AuthGate;
use crate::auth::ProgressiveAuthBanner;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("News");
    let data: Option<serde_json::Value> = ctx.params.get("data_news")
        .and_then(|s| serde_json::from_str(s).ok());
    let posts: Vec<NewsPost> = data.as_ref()
        .and_then(|d| serde_json::from_value(d.get("posts").cloned().unwrap_or(serde_json::json!([]))).ok())
        .unwrap_or_else(default_posts);
    let total = posts.len();

    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            if ctx.user.is_none() {
                ProgressiveAuthBanner {
                    feature: Some("personalized news".to_string()),
                }
            }
            AuthGate { user: ctx.user.clone(), feature: Some("news articles".to_string()),
                div { class: "container page-content max-w-7xl",
                    // === wave6-auth-pages-depth-track-d news header ===
                    div { class: "mb-12 text-center news-header",
                        div { class: "inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-cyan-500/20 bg-cyan-500/5 text-cyan-500 text-xs font-semibold mb-5",
                            Icon { name: "newspaper".to_string(), size: Some(14) }
                            " EPSX Platform"
                        }
                        h1 { class: "text-4xl sm:text-5xl font-extrabold mb-4",
                            "News & "
                            span { class: "bg-gradient-to-r from-purple-500 to-cyan-500 bg-clip-text text-transparent", "Updates" }
                        }
                        p { class: "text-muted-foreground max-w-xl mx-auto leading-relaxed",
                            "Stay informed with the latest platform updates, feature releases, and market insights from the EPSX team."
                        }
                        if total > 0 {
                            p { class: "mt-3 text-sm text-muted-foreground/60",
                                {
                                    let noun = if total == 1 { "article" } else { "articles" };
                                    format!("{total} {noun}")
                                }
                            }
                        }
                    }
                    // === wave6-auth-pages-depth-track-d news filters ===
                    NewsFilters {}
                    // === wave6-auth-pages-depth-track-d news list ===
                    div { class: "news-list-section mt-8",
                        if posts.is_empty() {
                            NewsEmptyState {}
                        } else {
                            // Featured card is the first post
                            NewsFeaturedCard { post: posts[0].clone() }
                            // Rest of the posts
                            if posts.len() > 1 {
                                div { class: "news-list-grid grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6 mt-8",
                                    for p in posts.iter().skip(1) {
                                        ArticleCard { post: p.clone() }
                                    }
                                }
                            }
                            // Pagination
                            NewsPagination { page: 1, total_pages: ((total + 11) / 12).max(1) }
                        }
                    }
                }
            }
        }
    })
}

fn default_posts() -> Vec<NewsPost> {
    vec![
        NewsPost { slug: "welcome-to-epsx".into(), title: "Welcome to EPSX".into(), excerpt: "We're excited to launch the new EPSX platform — a Web3 commerce and analytics platform built for modern teams.".into(), author: "EPSX Team".into(), published_at: "2024-09-15".into(), read_time: "3 min".into(), cover_image_url: None, tags: vec!["Update".into()] },
        NewsPost { slug: "bsc-integration".into(), title: "BSC mainnet integration live".into(), excerpt: "Full BSC mainnet support is now live with low fees and fast finality for all EPSX features.".into(), author: "EPSX Engineering".into(), published_at: "2024-09-10".into(), read_time: "5 min".into(), cover_image_url: None, tags: vec!["Engineering".into()] },
        NewsPost { slug: "subscription-v2".into(), title: "Subscription v2: programmable plans".into(), excerpt: "Create, edit, and manage on-chain subscription plans with full merchant controls and refunds.".into(), author: "EPSX Product".into(), published_at: "2024-09-01".into(), read_time: "4 min".into(), cover_image_url: None, tags: vec!["Product".into()] },
    ]
}

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

/// `NewsFilters` — category select, date range, search input. Static
/// form (no client-side filtering yet — that's a Wave 7 enhancement);
/// the section marker is here so tests can assert the surface area.
#[component]
fn NewsFilters() -> Element {
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

/// Featured card — large hero card with optional cover image, "Featured"
/// badge, tags, title, summary, and "Read article" CTA. Mirrors
/// `FeaturedCard` in `news-list.tsx`.
#[component]
fn NewsFeaturedCard(post: NewsPost) -> Element {
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
fn ArticleCard(post: NewsPost) -> Element {
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
fn NewsEmptyState() -> Element {
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
/// `news-list.tsx`. Hidden when there is only one page.
#[component]
fn NewsPagination(page: usize, total_pages: usize) -> Element {
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
    use crate::auth::User;
    use crate::pages::PageContext;
    use crate::auth::user::AuthMethod;

    fn ctx(path: &str) -> PageContext {
        let user = User {
            id: "u1".to_string(),
            address: "0x1234…abcd".to_string(),
            chain_id: "56".to_string(),
            roles: vec!["user".to_string()],
            email: Some("test@epsx.io".to_string()),
            tier: Some("Pro".to_string()),
            permissions: vec!["news:read".to_string()],
            last_login_at: None,
            auth_method: AuthMethod::Wallet,
            display_name: Some("Test".to_string()),
        };
        PageContext { user: Some(user), path: path.to_string(), ..Default::default() }
    }

    #[test]
    fn test_render_smoke() {
        let (_meta, element) = render(&ctx("/news"));
        let html = dioxus_ssr::render_element(element);
        // Dioxus HTML-escapes the `&` to `&#38;`; match the escaped form.
        assert!(html.contains("News &#38;"), "/news header must render. Got: {}", html);
    }

    #[test]
    fn test_section_markers() {
        let (_meta, element) = render(&ctx("/news"));
        let html = dioxus_ssr::render_element(element);
        for marker in ["news-header", "news-filters", "news-list-section"] {
            assert!(html.contains(marker), "missing section marker: {}", marker);
        }
    }
}

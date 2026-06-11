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
use crate::components::user::news::{ArticleCard, NewsEmptyState, NewsFeaturedCard, NewsFilters, NewsPagination, NewsPost};

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

// The 5 sub-components (`NewsFilters`, `NewsFeaturedCard`,
// `ArticleCard`, `NewsEmptyState`, `NewsPagination`) + the
// `NewsPost` data struct were extracted to
// `crate::components::user::news` in Wave 6C Track E. The page
// file `use`s them via the `use` line above.

fn default_posts() -> Vec<NewsPost> {
    vec![
        NewsPost { slug: "welcome-to-epsx".into(), title: "Welcome to EPSX".into(), excerpt: "We're excited to launch the new EPSX platform — a Web3 commerce and analytics platform built for modern teams.".into(), author: "EPSX Team".into(), published_at: "2024-09-15".into(), read_time: "3 min".into(), cover_image_url: None, tags: vec!["Update".into()] },
        NewsPost { slug: "bsc-integration".into(), title: "BSC mainnet integration live".into(), excerpt: "Full BSC mainnet support is now live with low fees and fast finality for all EPSX features.".into(), author: "EPSX Engineering".into(), published_at: "2024-09-10".into(), read_time: "5 min".into(), cover_image_url: None, tags: vec!["Engineering".into()] },
        NewsPost { slug: "subscription-v2".into(), title: "Subscription v2: programmable plans".into(), excerpt: "Create, edit, and manage on-chain subscription plans with full merchant controls and refunds.".into(), author: "EPSX Product".into(), published_at: "2024-09-01".into(), read_time: "4 min".into(), cover_image_url: None, tags: vec!["Product".into()] },
    ]
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

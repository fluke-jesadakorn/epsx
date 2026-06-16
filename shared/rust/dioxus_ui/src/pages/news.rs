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
use crate::auth::ProgressiveAuthBanner;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("News");
    let data: Option<serde_json::Value> = ctx.params.get("data_news")
        .and_then(|s| serde_json::from_str(s).ok());
    let posts: Vec<NewsPost> = data.as_ref()
        .and_then(|d| serde_json::from_value(d.get("posts").cloned().unwrap_or(serde_json::json!([]))).ok())
        .unwrap_or_else(default_posts);
    // Wave 23 T4: read the search query from the page context (set by
    // the BFF from `?q=...`) so SSR-rendered HTML already reflects the
    // filtered list. The client-side filter below re-applies the same
    // match so live typing narrows the list without a server round-trip.
    let initial_query: String = ctx
        .params
        .get("q")
        .cloned()
        .or_else(|| ctx.query_param("q"))
        .unwrap_or_default();
    let initial_category: String = ctx
        .params
        .get("category")
        .cloned()
        .or_else(|| ctx.query_param("category"))
        .unwrap_or_else(|| "all".to_string());
    let total = posts.len();

    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            if ctx.user.is_none() {
                ProgressiveAuthBanner {
                    feature: Some("personalized news".to_string()),
                }
            }
            // === wave22-t3-news-blog news header + list (no AuthGate; public page) ===
            // === wave23-t4-components filter provider + body ===
            NewsPageBody {
                posts: posts.clone(),
                initial_query: initial_query.clone(),
                initial_category: initial_category.clone(),
                total: total,
            }
        }
    })
}

/// Inner body component. Wave 23 T4: this is a Dioxus `#[component]`
/// (not the page's outer `render` function) so it can call hooks
/// (`use_context_provider`, `use_signal`, etc.). The outer `render`
/// is a plain function that returns `(PageMeta, Element)` and is
/// called from the BFF, which has no Dioxus runtime.
#[component]
fn NewsPageBody(
    posts: Vec<NewsPost>,
    initial_query: String,
    initial_category: String,
    total: usize,
) -> Element {
    // Provide a shared (q, cat) signal pair to the filter + list.
    let _ = filter_state::provide_news_filter();
    rsx! {
        div { class: "container page-content news-page",
            // === wave6-auth-pages-depth-track-d news header ===
            div { class: "mb-12 text-center news-header",
                div { class: "inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-cyan-500/20 bg-cyan-500/5 text-cyan-500 text-xs font-semibold mb-5",
                    Icon { name: "newspaper".to_string(), size: Some(14) }
                    " EPSX Platform"
                }
                h1 { class: "text-4xl sm:text-5xl font-extrabold mb-4",
                    "News & "
                    span { class: "gradient-text-purple", "Updates" }
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
            NewsFilters { initial_query: initial_query.clone(), initial_category: initial_category.clone() }
            // === wave6-auth-pages-depth-track-d news list ===
            // Wave 23 T4: pass the same query/category into the
            // list so the live filter and the SSR filter agree.
            NewsList { posts: posts.clone(), initial_query: initial_query.clone(), initial_category: initial_category.clone() }
        }
    }
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

/// `NewsFilters` — category select, date range, search input. Wave 23
/// T4: now reactive. Typing in the search box updates a shared signal
/// (provided by `NewsList`) which re-renders the visible posts in
/// place. The category select also updates the signal; the date range
/// is informational only (no client-side date filter is implemented —
/// the prod source has the same constraint).
///
/// - `initial_query` — SSR-side seed from the page context's
///   `?q=…` query param, so the first paint already reflects the
///   filtered list.
/// - `initial_category` — same idea for `?category=…`.
#[component]
fn NewsFilters(
    initial_query: String,
    initial_category: String,
) -> Element {
    // Lift the filter state up into NewsList so the two stay in sync
    // even if the filter component is re-mounted. We expose it as
    // context-style props here for simplicity (one consumer).
    let (mut q, mut cat) = use_news_filter();
    // Seed from SSR on first paint.
    use_effect(move || {
        if !initial_query.is_empty() {
            q.set(initial_query.clone());
        }
    });
    use_effect(move || {
        if initial_category != "all" {
            cat.set(initial_category.clone());
        }
    });
    rsx! {
        div { class: "card card-glass news-filters",
            div { class: "card-body flex flex-col md:flex-row gap-4 items-stretch md:items-end",
                div { class: "field flex-1",
                    label { class: "field-label", "Search" }
                    input {
                        class: "input",
                        r#type: "search",
                        placeholder: "Search articles…",
                        value: "{q.read()}",
                        oninput: move |e| q.set(e.value().to_string()),
                    }
                }
                div { class: "field md:w-48",
                    label { class: "field-label", "Category" }
                    SelectField {
                        name: "category".to_string(),
                        options: vec![
                            ("all".to_string(), "All".to_string()),
                            ("updates".to_string(), "Updates".to_string()),
                            ("engineering".to_string(), "Engineering".to_string()),
                            ("product".to_string(), "Product".to_string()),
                        ],
                        value: Some(cat.read().clone()),
                        required: false,
                        label: None,
                        help: None,
                        error: None,
                        placeholder: None,
                        onchange: Some(EventHandler::new(move |e: FormEvent| cat.set(e.value()))),
                    }
                }
                div { class: "field md:w-48",
                    label { class: "field-label", "Date range" }
                    SelectField {
                        name: "range".to_string(),
                        options: vec![
                            ("all".to_string(), "All time".to_string()),
                            ("7d".to_string(), "Last 7 days".to_string()),
                            ("30d".to_string(), "Last 30 days".to_string()),
                            ("90d".to_string(), "Last 90 days".to_string()),
                        ],
                        value: Some("all".to_string()),
                        required: false,
                        label: None,
                        help: None,
                        error: None,
                        placeholder: None,
                        onchange: None,
                    }
                }
                button {
                    class: "btn btn-outline",
                    r#type: "button",
                    onclick: move |_| {
                        // The "Search" button forces a hard navigation
                        // to /news?q=…&category=… so the BFF can re-render
                        // with the filter applied server-side. Useful
                        // for users who want a permalink / share a
                        // filtered list.
                        spawn(async move {
                            let script = format!(
                                "window.location.href = '/news?q={q}&category={c}';",
                                q = url_encode(&q.read()),
                                c = url_encode(&cat.read()),
                            );
                            let _ = document::eval(script.as_str()).await;
                        });
                    },
                    Icon { name: "search".to_string(), size: Some(16) }
                    " Search"
                }
            }
        }
    }
}

/// `NewsList` — renders the filtered list. Wave 23 T4: subscribes to
/// the same `q` / `cat` signals as `NewsFilters` and re-renders the
/// visible posts in place. The featured card is always the first
/// surviving post (or hidden if the filter yields 0/1 results).
#[component]
fn NewsList(
    posts: Vec<NewsPost>,
    initial_query: String,
    initial_category: String,
) -> Element {
    let (mut q, mut cat) = use_news_filter();
    // Seed from SSR.
    use_effect(move || {
        if !initial_query.is_empty() {
            q.set(initial_query.clone());
        }
    });
    use_effect(move || {
        if initial_category != "all" {
            cat.set(initial_category.clone());
        }
    });

    let query = q.read().to_lowercase();
    let category = cat.read().clone();
    let filtered: Vec<NewsPost> = posts
        .iter()
        .filter(|p| {
            // Category filter: a post matches if its tags include the
            // selected category, or if the filter is "all".
            let cat_ok = category == "all"
                || p.tags.iter().any(|t| t.to_lowercase() == category.to_lowercase());
            // Query filter: substring match on title + excerpt + tags.
            let q_ok = query.is_empty()
                || p.title.to_lowercase().contains(&query)
                || p.excerpt.to_lowercase().contains(&query)
                || p.tags.iter().any(|t| t.to_lowercase().contains(&query));
            cat_ok && q_ok
        })
        .cloned()
        .collect();
    let total = filtered.len();

    rsx! {
        div { class: "news-list-section mt-8",
            if filtered.is_empty() {
                NewsEmptyState {}
            } else {
                if filtered.len() == 1 {
                    ArticleCard { post: filtered[0].clone() }
                } else {
                    NewsFeaturedCard { post: filtered[0].clone() }
                    div { class: "news-list-grid grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6 mt-8",
                        for p in filtered.iter().skip(1) {
                            ArticleCard { post: p.clone() }
                        }
                    }
                }
                if total > 0 {
                    p { class: "mt-6 text-xs text-muted-foreground text-center news-list-count",
                        {
                            let noun = if total == 1 { "article" } else { "articles" };
                            format!("{total} {noun}")
                        }
                    }
                }
                NewsPagination { page: 1, total_pages: ((total + 11) / 12).max(1) }
            }
        }
    }
}

/// Shared filter signals used by `NewsFilters` + `NewsList`. The two
/// components live in the same render tree, so we use a small
/// Dioxus context to share the `(q, cat)` `Signal` pair. The signals
/// reset on full page navigation (which is the correct SSR
/// behaviour: the BFF re-renders with `?q=…&category=…` from the URL).
mod filter_state {
    use dioxus::prelude::*;

    #[derive(Clone, Copy)]
    pub struct NewsFilter {
        pub q: Signal<String>,
        pub cat: Signal<String>,
    }

    pub fn use_news_filter() -> NewsFilter {
        use_context::<NewsFilter>()
    }

    pub fn provide_news_filter() -> NewsFilter {
        let q = use_signal(String::new);
        let cat = use_signal(|| "all".to_string());
        let f = NewsFilter { q, cat };
        use_context_provider(|| f);
        f
    }
}

fn use_news_filter() -> (Signal<String>, Signal<String>) {
    let f = filter_state::use_news_filter();
    (f.q, f.cat)
}

fn url_encode(s: &str) -> String {
    // Minimal URL-encoding for query-param values. Anything outside
    // `[A-Za-z0-9._~-]` is escaped as `%XX`. Good enough for the
    // search button's `window.location.href` setter.
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'.' | b'_' | b'~' | b'-' => {
                out.push(b as char);
            }
            _ => {
                out.push_str(&format!("%{:02X}", b));
            }
        }
    }
    out
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

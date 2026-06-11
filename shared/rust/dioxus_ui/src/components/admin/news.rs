//! Sub-components for `/admin/news` — Wave 6C Track D.
//!
//! 1:1 mirror of `apps-old/admin-frontend/components/news/*.tsx`:
//!   1. `NewsManagementList`    — outer list + status filter pills
//!   2. `NewsEditor`            — create/edit form with markdown editor
//!   3. `NewsFeaturedCard`      — pinned / featured article highlight
//!   4. `ArticleCard`           — single article row
//!   5. `NewsEmptyState`        — empty state when 0 articles
//!   6. `NewsPagination`        — previous/next page controls
//!
//! The `NewsArticle` data struct is `pub` so the parent page can
//! build articles from BFF data.

use crate::primitives::*;
use crate::rich_text::RichTextEditor;
use crate::feedback::admin_action_confirm::{AdminActionConfirm, ConfirmVariant};

use dioxus::prelude::*;

// ============================================================================
// Data
// ============================================================================

/// News article data. Mirrors the `NewsArticle` interface from
/// `apps-old/admin-frontend/shared/api/news.ts` (subset).
#[derive(Clone, Debug, PartialEq)]
pub struct NewsArticle {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub status: String,
    pub author: String,
    pub published_at: Option<String>,
    pub summary: Option<String>,
    pub cover_image_url: Option<String>,
    pub tags: Vec<String>,
    pub is_pinned: bool,
}

// ============================================================================
// Section 1: NewsManagementList
// ============================================================================

#[component]
pub fn NewsManagementList(articles: Vec<NewsArticle>, status: String) -> Element {
    rsx! {
        div { class: "news-management-list space-y-6",
            {
                let featured = articles.iter().find(|a| a.is_pinned).cloned();
                if let Some(feat) = featured {
                    rsx! { NewsFeaturedCard { article: feat.clone() } }
                } else {
                    rsx! { Fragment {} }
                }
            }
            div { class: "news-management-filters flex items-center gap-2 flex-wrap",
                for s in &["all", "draft", "published"] {
                    a {
                        class: if *s == status { "px-3 py-1.5 rounded-lg text-sm font-medium transition-colors capitalize bg-[#7645d9] text-white shadow-lg shadow-[#7645d9]/20" } else { "px-3 py-1.5 rounded-lg text-sm font-medium transition-colors capitalize bg-card border border-border/20 text-muted-foreground hover:text-foreground hover:border-border/40" },
                        href: format!("/news?status={s}&page=1"),
                        "{s}"
                    }
                }
                span { class: "ml-auto text-sm text-muted-foreground",
                    "{articles.len()} "
                    if articles.len() == 1 { "article" } else { "articles" }
                }
            }
            if articles.is_empty() {
                NewsEmptyState {}
            } else {
                div { class: "news-management-articles space-y-3",
                    for article in articles.iter() {
                        if !article.is_pinned {
                            ArticleCard { article: article.clone() }
                        }
                    }
                }
                NewsPagination { page: 1, total_pages: 1, status: status.clone() }
            }
        }
    }
}

// ============================================================================
// Section 2: NewsEditor
// ============================================================================

#[component]
pub fn NewsEditor(id: Option<String>, title_str: String) -> Element {
    let mut title = use_signal(String::new);
    let mut body = use_signal(|| "## Introduction\n\nWrite your news article here in markdown.\n\n- Point 1\n- Point 2\n\n[Read more](https://epsx.io)".to_string());
    let mut status = use_signal(|| "draft".to_string());
    rsx! {
        div { class: "news-editor max-w-4xl",
            div { class: "news-editor-header sticky top-0 z-10 backdrop-blur-md bg-background/80 border-b border-border/10 py-3 flex items-center justify-between mb-4",
                h1 { class: "text-xl font-bold", "{title_str}" }
                div { class: "flex items-center gap-3",
                    div { class: "flex items-center rounded-lg border border-border/20 bg-card overflow-hidden",
                        button {
                            class: if status.read().as_str() == "draft" { "px-3 py-1.5 text-xs font-medium transition-colors bg-[#7645d9]/20 text-[#7645d9]" } else { "px-3 py-1.5 text-xs font-medium transition-colors text-muted-foreground hover:text-foreground" },
                            r#type: "button",
                            onclick: move |_| status.set("draft".to_string()),
                            "Draft"
                        }
                        button {
                            class: if status.read().as_str() == "published" { "px-3 py-1.5 text-xs font-medium transition-colors bg-[#7645d9]/20 text-[#7645d9]" } else { "px-3 py-1.5 text-xs font-medium transition-colors text-muted-foreground hover:text-foreground" },
                            r#type: "button",
                            onclick: move |_| status.set("published".to_string()),
                            "Published"
                        }
                    }
                    button {
                        class: "news-editor-save flex items-center gap-2 px-4 py-2 rounded-xl bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white text-sm font-semibold",
                        r#type: "submit",
                        "💾 Save"
                    }
                }
            }
            Form { method: "POST".to_string(),
                action: if let Some(ref i) = id { format!("/api/v1/news/{}", i) } else { "/api/v1/news".to_string() },
                div { class: "field",
                    label { class: "field-label", "Title" }
                    input {
                        class: "input",
                        name: "title",
                        required: true,
                        value: "{title.read()}",
                        oninput: move |e| title.set(e.value().to_string()),
                        placeholder: "A clear, descriptive title",
                    }
                }
                div { class: "field",
                    label { class: "field-label", "Slug" }
                    input { class: "input", name: "slug", required: true, placeholder: "auto-generated-from-title" }
                }
                div { class: "field",
                    label { class: "field-label", "Excerpt" }
                    textarea {
                        class: "input",
                        name: "excerpt",
                        rows: "2",
                        placeholder: "Short summary for the listing page",
                    }
                }
                div { class: "field",
                    label { class: "field-label", "Body" }
                    RichTextEditor { name: "body".to_string(), label: None, value: Some(body.read().clone()), rows: 16 }
                }
                div { class: "field",
                    label { class: "field-label", "Status" }
                    input { r#type: "hidden", name: "status", value: "{status.read()}" }
                    span { class: "text-sm text-muted-foreground", "Toggle via the header switch above. Currently: " span { class: "font-bold", "{status.read()}" } }
                }
                FormActions {
                    a { class: "btn btn-outline", href: "/news", "Cancel" }
                    button { class: "btn btn-secondary", r#type: "submit", "Save as draft" }
                    button { class: "btn btn-primary", r#type: "submit", "Publish" }
                }
            }
        }
    }
}

// ============================================================================
// Section 3: NewsFeaturedCard
// ============================================================================

#[component]
pub fn NewsFeaturedCard(article: NewsArticle) -> Element {
    let has_cover = article.cover_image_url.is_some();
    rsx! {
        div { class: "news-featured-card relative overflow-hidden rounded-2xl bg-card border border-[#1fc7d4]/30 shadow-xl",
            div { class: "h-[3px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]" }
            div { class: "p-6 flex flex-col sm:flex-row gap-6",
                div { class: "news-featured-card-cover shrink-0 w-full sm:w-64 h-40 rounded-xl overflow-hidden bg-gradient-to-br from-[#7645d9]/10 via-[#1fc7d4]/5 to-transparent border border-border/20 flex items-center justify-center",
                    if has_cover {
                        img { class: "w-full h-full object-cover", src: "{article.cover_image_url.as_deref().unwrap_or(\"\")}", alt: "" }
                    } else {
                        Icon { name: "file-text".to_string(), size: Some(48), class_name: Some("text-muted-foreground/40".to_string()) }
                    }
                }
                div { class: "flex-1 min-w-0",
                    div { class: "flex items-center gap-2 mb-2",
                        span { class: "news-featured-card-pinned inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-[#1fc7d4]/10 text-[#1fc7d4]",
                            "📌 Pinned"
                        }
                        span { class: "inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-green-500/10 text-green-400 border border-green-500/20",
                            "{article.status}"
                        }
                    }
                    h2 { class: "news-featured-card-title text-2xl font-bold text-foreground mb-2", "{article.title}" }
                    if let Some(s) = &article.summary {
                        p { class: "text-sm text-muted-foreground mb-3", "{s}" }
                    }
                    div { class: "news-featured-card-meta flex items-center gap-3 text-xs text-muted-foreground",
                        span { "By {article.author}" }
                        if let Some(d) = &article.published_at {
                            span { "• {d}" }
                        }
                    }
                }
                div { class: "news-featured-card-actions flex flex-col gap-2",
                    a { class: "btn btn-sm btn-primary", href: format!("/news/{}/edit", article.id), "Edit" }
                    a { class: "btn btn-sm btn-outline", href: format!("/news/{}", article.slug), "View" }
                }
            }
        }
    }
}

// ============================================================================
// Section 4: ArticleCard
// ============================================================================

#[component]
pub fn ArticleCard(article: NewsArticle) -> Element {
    let has_cover = article.cover_image_url.is_some();
    let status_cls = if article.status == "published" {
        "bg-green-500/10 text-green-400 border border-green-500/20"
    } else {
        "bg-yellow-500/10 text-yellow-400 border border-yellow-500/20"
    };
    let date_str = article.published_at.clone().unwrap_or_else(|| "—".to_string());
    let mut confirm_delete_open = use_signal(|| false);
    rsx! {
        div { class: "article-card flex items-start gap-4 p-4 rounded-2xl bg-card border border-border/20 hover:border-border/40 transition-colors",
            div { class: "article-card-cover shrink-0 w-20 h-14 rounded-xl overflow-hidden bg-gradient-to-br from-[#7645d9]/10 via-[#1fc7d4]/5 to-transparent border border-border/20 flex items-center justify-center",
                if has_cover {
                    img { class: "w-full h-full object-cover", src: "{article.cover_image_url.as_deref().unwrap_or(\"\")}", alt: "" }
                } else {
                    Icon { name: "file-text".to_string(), size: Some(20), class_name: Some("text-muted-foreground/40".to_string()) }
                }
            }
            div { class: "flex-1 min-w-0",
                div { class: "flex items-start justify-between gap-3",
                    div { class: "min-w-0 flex-1",
                        div { class: "flex items-center gap-2",
                            p { class: "article-card-title font-semibold text-foreground truncate", "{article.title}" }
                            if article.is_pinned {
                                Icon { name: "pin".to_string(), size: Some(14), class_name: Some("text-[#1fc7d4] shrink-0".to_string()) }
                            }
                        }
                        p { class: "text-xs text-muted-foreground/60 font-mono truncate", "{article.slug}" }
                    }
                    div { class: "flex items-center gap-2 shrink-0",
                        span { class: "article-card-status inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium {status_cls}",
                            "{article.status}"
                        }
                        span { class: "text-xs text-muted-foreground whitespace-nowrap hidden sm:block", "{date_str}" }
                        div { class: "article-card-actions flex items-center gap-1",
                            button {
                                class: "p-1.5 rounded-lg hover:bg-muted/50 text-muted-foreground hover:text-foreground transition-colors",
                                r#type: "button",
                                title: "Toggle pin",
                                if article.is_pinned {
                                    Icon { name: "pin-off".to_string(), size: Some(16), class_name: Some("text-[#1fc7d4]".to_string()) }
                                } else {
                                    Icon { name: "pin".to_string(), size: Some(16) }
                                }
                            }
                            button {
                                class: "p-1.5 rounded-lg hover:bg-muted/50 text-muted-foreground hover:text-foreground transition-colors",
                                r#type: "button",
                                title: "Toggle publish",
                                if article.status == "published" {
                                    Icon { name: "eye-off".to_string(), size: Some(16) }
                                } else {
                                    Icon { name: "eye".to_string(), size: Some(16) }
                                }
                            }
                            a {
                                class: "p-1.5 rounded-lg hover:bg-muted/50 text-muted-foreground hover:text-foreground transition-colors",
                                href: format!("/news/{}/edit", article.id),
                                title: "Edit",
                                Icon { name: "edit".to_string(), size: Some(16) }
                            }
                            button {
                                class: "p-1.5 rounded-lg hover:bg-red-500/10 text-muted-foreground hover:text-red-400 transition-colors",
                                r#type: "button",
                                title: "Delete",
                                onclick: move |_| confirm_delete_open.set(true),
                                Icon { name: "trash-2".to_string(), size: Some(16) }
                            }
                        }
                    }
                }
                if let Some(s) = &article.summary {
                    p { class: "text-sm text-muted-foreground mt-1 line-clamp-1", "{s}" }
                }
                if !article.tags.is_empty() {
                    div { class: "flex flex-wrap gap-1 mt-2",
                        for tag in article.tags.iter().take(4) {
                            span { class: "px-2 py-0.5 rounded-full text-xs bg-[#7645d9]/10 text-[#7645d9]/70 font-medium", "{tag}" }
                        }
                        if article.tags.len() > 4 {
                            span { class: "text-xs text-muted-foreground/50", "+{article.tags.len() - 4}" }
                        }
                    }
                }
            }
            AdminActionConfirm {
                open: *confirm_delete_open.read(),
                title: "Delete Article?".to_string(),
                message: format!("\"{}\" will be permanently deleted.", article.title),
                confirm_label: "Delete".to_string(),
                confirm_variant: ConfirmVariant::Destructive,
                on_confirm: move |_| confirm_delete_open.set(false),
                on_cancel: move |_| confirm_delete_open.set(false),
            }
        }
    }
}

// ============================================================================
// Section 5: NewsEmptyState
// ============================================================================

#[component]
pub fn NewsEmptyState() -> Element {
    rsx! {
        div { class: "news-empty-state rounded-2xl bg-card border border-border/20 shadow-xl flex flex-col items-center justify-center py-20 gap-4",
            div { class: "p-5 rounded-full bg-gradient-to-br from-[#7645d9]/10 via-[#1fc7d4]/5 to-transparent border border-border/20",
                Icon { name: "newspaper".to_string(), size: Some(32), class_name: Some("text-muted-foreground/40".to_string()) }
            }
            div { class: "text-center",
                p { class: "font-semibold text-foreground", "No articles yet" }
                p { class: "text-sm text-muted-foreground mt-1", "Create your first article to get started." }
            }
            a { class: "btn btn-primary", href: "/news/create",
                Icon { name: "plus".to_string(), size: Some(16) }
                " Create Article"
            }
        }
    }
}

// ============================================================================
// Section 6: NewsPagination
// ============================================================================

#[component]
pub fn NewsPagination(page: u32, total_pages: u32, status: String) -> Element {
    if total_pages <= 1 {
        return rsx! { Fragment {} };
    }
    let base = format!("/news?status={status}");
    rsx! {
        div { class: "news-pagination flex items-center justify-center gap-2",
            a {
                class: if page == 1 { "px-3 py-1.5 rounded-lg text-sm border border-border/20 opacity-40 pointer-events-none" } else { "px-3 py-1.5 rounded-lg text-sm border border-border/20 hover:bg-muted/50 transition-colors" },
                href: format!("{base}&page={}", page.saturating_sub(1)),
                "Previous"
            }
            span { class: "text-sm text-muted-foreground", "{page} / {total_pages}" }
            a {
                class: if page == total_pages { "px-3 py-1.5 rounded-lg text-sm border border-border/20 opacity-40 pointer-events-none" } else { "px-3 py-1.5 rounded-lg text-sm border border-border/20 hover:bg-muted/50 transition-colors" },
                href: format!("{base}&page={}", page + 1),
                "Next"
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn render_to_string(el: Element) -> String {
        dioxus_ssr::render_element(el)
    }

    fn sample_article_pinned() -> NewsArticle {
        NewsArticle {
            id: "1".into(), title: "Welcome to EPSX".into(), slug: "welcome-to-epsx".into(),
            status: "published".into(), author: "EPSX Team".into(),
            published_at: Some("2024-09-15".into()),
            summary: Some("Introducing the EPSX platform.".into()),
            cover_image_url: None, tags: vec!["announcement".into()], is_pinned: true,
        }
    }

    fn sample_article_unpinned() -> NewsArticle {
        NewsArticle {
            id: "2".into(), title: "BSC mainnet".into(), slug: "bsc-mainnet".into(),
            status: "published".into(), author: "EPSX Eng".into(),
            published_at: Some("2024-09-10".into()),
            summary: Some("BSC mainnet is now the default chain.".into()),
            cover_image_url: None, tags: vec!["engineering".into()], is_pinned: false,
        }
    }

    #[test]
    fn test_render_smoke_news_management_list() {
        let el = rsx! { NewsManagementList { articles: vec![sample_article_pinned(), sample_article_unpinned()], status: "all".to_string() } };
        let html = render_to_string(el);
        assert!(html.contains("news-management-list"), "NewsManagementList must render its class. Got: {}", html);
        assert!(html.contains("BSC mainnet"), "NewsManagementList must render the article title. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_news_editor() {
        let el = rsx! { NewsEditor { id: None, title_str: "New news post".to_string() } };
        let html = render_to_string(el);
        assert!(html.contains("news-editor"), "NewsEditor must render its class. Got: {}", html);
        assert!(html.contains("New news post"), "NewsEditor must render the title. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_news_featured_card() {
        let el = rsx! { NewsFeaturedCard { article: sample_article_pinned() } };
        let html = render_to_string(el);
        assert!(html.contains("news-featured-card"), "NewsFeaturedCard must render its class. Got: {}", html);
        assert!(html.contains("Pinned"), "NewsFeaturedCard must render the Pinned badge. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_article_card() {
        let el = rsx! { ArticleCard { article: sample_article_unpinned() } };
        let html = render_to_string(el);
        assert!(html.contains("article-card"), "ArticleCard must render its class. Got: {}", html);
        assert!(html.contains("BSC mainnet"), "ArticleCard must render the article title. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_news_empty_state() {
        let el = rsx! { NewsEmptyState {} };
        let html = render_to_string(el);
        assert!(html.contains("news-empty-state"), "NewsEmptyState must render its class. Got: {}", html);
        assert!(html.contains("No articles yet"), "NewsEmptyState must render the headline. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_news_pagination() {
        let el = rsx! { NewsPagination { page: 2, total_pages: 5, status: "all".to_string() } };
        let html = render_to_string(el);
        assert!(html.contains("news-pagination"), "NewsPagination must render its class. Got: {}", html);
        assert!(html.contains("Previous"), "NewsPagination must render the Previous link. Got: {}", html);
        assert!(html.contains("Next"), "NewsPagination must render the Next link. Got: {}", html);
    }

    #[test]
    fn news_pagination_hidden_when_single_page() {
        let mut vdom = dioxus::prelude::VirtualDom::new(|| {
            rsx! { NewsPagination { page: 1, total_pages: 1, status: "all".to_string() } }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(!html.contains("news-pagination"), "Pagination must render nothing when total_pages <= 1. Got: {}", html);
    }
}

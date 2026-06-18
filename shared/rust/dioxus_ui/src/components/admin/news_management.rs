//! Admin `NewsManagement` family — Wave 38b T2 admin domain port.
//!
//! Mirrors `apps-old/admin-frontend/components/news/news-management.tsx`,
//! which exports 4 components for the news list management surface:
//!
//! | Component | Use case |
//! | --- | --- |
//! | `NewsStatusBadge` | Status pill (draft/published) |
//! | `NewsArticleCard` | One article row (cover + title + summary + tags + actions) |
//! | `NewsEmptyState` | Empty-state card (icon + headline + CTA) |
//! | `NewsPagination` | Prev / Next page controls |
//!
//! The `NewsManagement` master container wires the 4 components
//! together (header + status filters + article list + pagination).
//!
//! ## Tests
//!
//! Each component gets a colocated `#[cfg(test)] mod tests` block
//! with smoke render + key prop handling (status variant, empty
//! list, multi-page pagination).

use dioxus::prelude::*;
use crate::primitives::icon::Icon;

// ============================================================================
// Data shape
// ============================================================================

#[derive(Clone, Debug, PartialEq)]
pub struct NewsArticle {
    pub id: String,
    pub title: String,
    pub slug: String,
    /// `draft` | `published`
    pub status: String,
    pub author: String,
    pub published_at: Option<String>,
    pub summary: Option<String>,
    pub cover_image_url: Option<String>,
    pub tags: Vec<String>,
    /// `true` for the article that's pinned to the homepage featured slot.
    pub is_pinned: bool,
}

// ============================================================================
// Status badge
// ============================================================================
//
// Status pill (draft / published). Mirrors the source's `StatusBadge`.

/// Maps a news status string to the Tailwind pill class.
pub fn news_status_class(status: &str) -> &'static str {
    if status == "published" {
        "bg-green-500/10 text-green-400 border border-green-500/20"
    } else {
        "bg-yellow-500/10 text-yellow-400 border border-yellow-500/20"
    }
}

#[component]
pub fn NewsStatusBadge(status: String) -> Element {
    let cls = news_status_class(&status);
    rsx! {
        span { class: "news-management-status-badge inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium {cls}",
            "{status}"
        }
    }
}

// ============================================================================
// Date helper
// ============================================================================

fn format_news_date(date_str: Option<String>) -> String {
    match date_str {
        Some(d) => d,
        None => "\u{2014}".to_string(),
    }
}

// ============================================================================
// NewsArticleCard
// ============================================================================
//
// One article row. Cover thumbnail + title + slug + summary +
// tags + status + date + actions (pin / publish / edit / delete).
// Mirrors the source's `ArticleCard`.

#[component]
pub fn NewsArticleCard(
    article: NewsArticle,
    /// Optional edit URL (e.g., `/news/{id}/edit`).
    edit_href: Option<String>,
    on_toggle_pin: Option<EventHandler<String>>,
    on_toggle_publish: Option<EventHandler<String>>,
    on_delete: Option<EventHandler<String>>,
) -> Element {
    let has_cover = article.cover_image_url.is_some();
    let cover_url = article.cover_image_url.clone().unwrap_or_default();
    let date_str = format_news_date(article.published_at.clone());
    let edit_href = edit_href.unwrap_or_else(|| format!("/news/{}/edit", article.id));
    let id = article.id.clone();
    rsx! {
        div { class: "news-management-article-card flex items-start gap-4 p-4 rounded-2xl bg-card border border-border/20 hover:border-border/40 transition-colors",
            // Cover thumbnail
            div { class: "news-management-article-card-cover shrink-0 w-20 h-14 rounded-xl overflow-hidden bg-gradient-to-br from-[#7645d9]/10 via-[#1fc7d4]/5 to-transparent border border-border/20 flex items-center justify-center",
                if has_cover {
                    img { class: "w-full h-full object-cover", src: "{cover_url}", alt: "" }
                } else {
                    Icon { name: "file-text".to_string(), size: Some(20), class_name: Some("text-muted-foreground/40".to_string()) }
                }
            }
            // Title + slug + summary + tags + status + actions
            div { class: "flex-1 min-w-0",
                div { class: "flex items-start justify-between gap-3",
                    div { class: "min-w-0 flex-1",
                        div { class: "flex items-center gap-2",
                            p { class: "news-management-article-card-title font-semibold text-foreground truncate", "{article.title}" }
                            if article.is_pinned {
                                Icon { name: "pin".to_string(), size: Some(14), class_name: Some("text-[#1fc7d4] shrink-0".to_string()) }
                            }
                        }
                        p { class: "text-xs text-muted-foreground/60 font-mono truncate", "{article.slug}" }
                    }
                    div { class: "flex items-center gap-2 shrink-0",
                        NewsStatusBadge { status: article.status.clone() }
                        span { class: "text-xs text-muted-foreground whitespace-nowrap hidden sm:block", "{date_str}" }
                        // Action buttons
                        div { class: "news-management-article-card-actions flex items-center gap-1",
                            button {
                                class: "p-1.5 rounded-lg hover:bg-muted/50 text-muted-foreground hover:text-foreground transition-colors",
                                r#type: "button",
                                title: "Toggle pin",
                                onclick: {
                                    let id_for_pin = id.clone();
                                    let h = on_toggle_pin.clone();
                                    move |_| {
                                        if let Some(h) = h.clone() { h.call(id_for_pin.clone()); }
                                    }
                                },
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
                                onclick: {
                                    let id_for_pub = id.clone();
                                    let h = on_toggle_publish.clone();
                                    move |_| {
                                        if let Some(h) = h.clone() { h.call(id_for_pub.clone()); }
                                    }
                                },
                                if article.status == "published" {
                                    Icon { name: "eye-off".to_string(), size: Some(16) }
                                } else {
                                    Icon { name: "eye".to_string(), size: Some(16) }
                                }
                            }
                            a {
                                class: "p-1.5 rounded-lg hover:bg-muted/50 text-muted-foreground hover:text-foreground transition-colors",
                                href: "{edit_href}",
                                title: "Edit",
                                Icon { name: "edit".to_string(), size: Some(16) }
                            }
                            button {
                                class: "p-1.5 rounded-lg hover:bg-red-500/10 text-muted-foreground hover:text-red-400 transition-colors",
                                r#type: "button",
                                title: "Delete",
                                onclick: {
                                    let id_for_del = id.clone();
                                    let h = on_delete.clone();
                                    move |_| {
                                        if let Some(h) = h.clone() { h.call(id_for_del.clone()); }
                                    }
                                },
                                Icon { name: "trash-2".to_string(), size: Some(16) }
                            }
                        }
                    }
                }
                if let Some(s) = article.summary.clone() {
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
        }
    }
}

// ============================================================================
// NewsEmptyState
// ============================================================================
//
// Empty-state card (icon + headline + CTA). Mirrors the source's
// `EmptyState`.

#[component]
pub fn NewsEmptyState() -> Element {
    rsx! {
        div { class: "news-management-empty-state rounded-2xl bg-card border border-border/20 shadow-xl flex flex-col items-center justify-center py-20 gap-4",
            div { class: "p-5 rounded-full bg-gradient-to-br from-[#7645d9]/10 via-[#1fc7d4]/5 to-transparent border border-border/20",
                Icon { name: "newspaper".to_string(), size: Some(32), class_name: Some("text-muted-foreground/40".to_string()) }
            }
            div { class: "text-center",
                p { class: "font-semibold text-foreground", "No articles yet" }
                p { class: "text-sm text-muted-foreground mt-1", "Create your first article to get started." }
            }
            a { class: "flex items-center gap-2 px-4 py-2 rounded-xl bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white text-sm font-semibold hover:opacity-90 transition-opacity",
                href: "/news/create",
                Icon { name: "plus".to_string(), size: Some(16) }
                "Create Article"
            }
        }
    }
}

// ============================================================================
// NewsPagination
// ============================================================================
//
// Prev / Next page controls. Mirrors the source's `Pagination`.

#[component]
pub fn NewsPagination(page: u32, total_pages: u32, status: String) -> Element {
    if total_pages <= 1 {
        return rsx! { Fragment {} };
    }
    let base = format!("/news?status={status}");
    rsx! {
        div { class: "news-management-pagination flex items-center justify-center gap-2",
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

    fn sample_article() -> NewsArticle {
        NewsArticle {
            id: "1".to_string(),
            title: "Welcome to EPSX".to_string(),
            slug: "welcome-to-epsx".to_string(),
            status: "published".to_string(),
            author: "EPSX Team".to_string(),
            published_at: Some("2024-09-15".to_string()),
            summary: Some("Introducing the EPSX platform.".to_string()),
            cover_image_url: None,
            tags: vec!["announcement".into(), "platform".into()],
            is_pinned: true,
        }
    }

    /// `NewsStatusBadge` uses success class for "published".
    #[test]
    fn news_status_badge_published_uses_success_class() {
        fn harness() -> Element {
            rsx! { NewsStatusBadge { status: "published".to_string() } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("bg-green-500/10"), "NewsStatusBadge published must use success class. Got: {html}");
        assert!(html.contains("published"), "NewsStatusBadge must render status text. Got: {html}");
    }

    /// `NewsStatusBadge` uses warning class for "draft".
    #[test]
    fn news_status_badge_draft_uses_warning_class() {
        fn harness() -> Element {
            rsx! { NewsStatusBadge { status: "draft".to_string() } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("bg-yellow-500/10"), "NewsStatusBadge draft must use warning class. Got: {html}");
        assert!(html.contains("draft"), "NewsStatusBadge must render status text. Got: {html}");
    }

    /// `NewsArticleCard` renders title + slug + status + summary + tags.
    #[test]
    fn news_article_card_renders_all_fields() {
        fn harness() -> Element {
            rsx! { NewsArticleCard { article: sample_article() } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Welcome to EPSX"), "NewsArticleCard must render title. Got: {html}");
        assert!(html.contains("welcome-to-epsx"), "NewsArticleCard must render slug. Got: {html}");
        assert!(html.contains("published"), "NewsArticleCard must render status. Got: {html}");
        assert!(html.contains("Introducing the EPSX platform."), "NewsArticleCard must render summary. Got: {html}");
        assert!(html.contains("announcement"), "NewsArticleCard must render tag. Got: {html}");
        assert!(html.contains("news-management-article-card"), "NewsArticleCard must render container class. Got: {html}");
    }

    /// `NewsArticleCard` for a pinned article shows the pin icon (data-icon=pin).
    #[test]
    fn news_article_card_renders_pin_icon_when_pinned() {
        fn harness() -> Element {
            rsx! { NewsArticleCard { article: sample_article() } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        // Pinned article uses pin-off (the unpin action) for its icon button
        assert!(html.contains("lucide-pin-off"), "NewsArticleCard pinned must show unpin icon. Got: {html}");
    }

    /// `NewsArticleCard` for a draft article shows the eye (publish action) icon.
    #[test]
    fn news_article_card_renders_publish_icon_when_draft() {
        fn harness() -> Element {
            let mut a = sample_article();
            a.status = "draft".to_string();
            rsx! { NewsArticleCard { article: a } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("lucide-eye "), "NewsArticleCard draft must show publish icon. Got: {html}");
    }

    /// `NewsEmptyState` renders the headline + CTA.
    #[test]
    fn news_empty_state_renders_headline_and_cta() {
        fn harness() -> Element {
            rsx! { NewsEmptyState { } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("news-management-empty-state"), "NewsEmptyState must render container class. Got: {html}");
        assert!(html.contains("No articles yet"), "NewsEmptyState must render headline. Got: {html}");
        assert!(html.contains("Create your first article"), "NewsEmptyState must render subhead. Got: {html}");
        assert!(html.contains("Create Article"), "NewsEmptyState must render CTA. Got: {html}");
    }

    /// `NewsPagination` renders nothing when total_pages <= 1.
    #[test]
    fn news_pagination_hidden_when_single_page() {
        fn harness() -> Element {
            rsx! { NewsPagination { page: 1u32, total_pages: 1u32, status: "all".to_string() } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(!html.contains("news-management-pagination"), "NewsPagination must hide when total_pages <= 1. Got: {html}");
    }

    /// `NewsPagination` renders Prev / Next when total_pages > 1.
    #[test]
    fn news_pagination_renders_prev_next() {
        fn harness() -> Element {
            rsx! { NewsPagination { page: 2u32, total_pages: 5u32, status: "all".to_string() } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Previous"), "NewsPagination must render Previous. Got: {html}");
        assert!(html.contains("Next"), "NewsPagination must render Next. Got: {html}");
        assert!(html.contains("2 / 5"), "NewsPagination must render page indicator. Got: {html}");
    }

    /// `news_status_class` returns the expected class per status.
    #[test]
    fn news_status_class_matches_source() {
        assert_eq!(news_status_class("published"), "bg-green-500/10 text-green-400 border border-green-500/20");
        assert_eq!(news_status_class("draft"), "bg-yellow-500/10 text-yellow-400 border border-yellow-500/20");
    }
}

//! /admin/news (list) + /admin/news/create + /admin/news/[id]/edit.
//!
//! Source of truth: `apps-old/admin-frontend/{app/news/page.tsx,
//! components/news/news-management.tsx, components/news/news-editor.tsx}`.
//!
//! Wave 6B port brought the 6 design-doc sections inline (list
//! + featured card + article card + empty state + pagination +
//! editor). Wave 42 T2 wires the Wave 38b ported components
//! (`PageHeader`, `NewsArticleCard`, `NewsEmptyState`,
//! `NewsPagination`, `NewsEditorHeader`, `NewsEditorFormFields`,
//! `NewsEditorFooter`, `NewsEditorStatusToggle`) into the page.
//!
//! Section coverage:
//!   1. NewsManagementList — outer list + status filter pills
//!   2. NewsEditor — create/edit form (rendered as a skeleton
//!      behind the auth overlay per Wave 25 T3 — see
//!      `render_editor`).
//!   3. NewsFeaturedCard — pinned/featured article highlight
//!   4. ArticleCard → `NewsArticleCard` (Wave 38b ported)
//!   5. NewsEmptyState → `NewsEmptyState` (Wave 38b ported)
//!   6. NewsPagination → `NewsPagination` (Wave 38b ported)
//!
//! The page entry-points (`render`, `render_create`, `render_edit`)
//! dispatch into the right section for each route.

use crate::primitives::*;
use crate::data_table::{Column, DataTable, Row, SortDir};
use crate::rich_text::RichTextEditor;
use crate::feedback::admin_action_confirm::{AdminActionConfirm, ConfirmVariant};
use crate::components::admin::auth_page_overlay::{AuthPageOverlay, SkeletonPage};
use crate::components::admin::page_layout::{PageGradient, PageHeader, PageLayout, PageMaxWidth};
use crate::components::admin::news_editor::{
    NewsEditorFooter, NewsEditorFormFields, NewsEditorHeader, NewsEditorStatusToggle,
};
use crate::components::admin::news_management::{
    NewsArticle, NewsArticleCard, NewsEmptyState, NewsPagination, NewsStatusBadge,
};

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::auth::AdminAuthGate;

// =====================================================================
// Public entry points
// =====================================================================

/// `/admin/news` — the list view. Renders the `NewsManagementList`
/// section + 3 sample `NewsArticleCard`s + the Wave 38b
/// `PageHeader` (the BFF hydrates with the real list on the
/// client). The Wave 38b `DataTable` is kept as a secondary
/// table view for parity with the prior shallow port.
pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("News");

    // SSR'd `DataTable` columns + rows — same shape as the shallow
    // port. The BFF re-renders this with the real list on hydration.
    let columns = vec![
        Column { key: "title".into(), label: "Title".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("40%".into()), class_name: None },
        Column { key: "status".into(), label: "Status".into(), sortable: true, align: crate::primitives::data_table::Align::Center, width: Some("15%".into()), class_name: None },
        Column { key: "author".into(), label: "Author".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("20%".into()), class_name: None },
        Column { key: "created".into(), label: "Created".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("15%".into()), class_name: None },
        Column { key: "actions".into(), label: "Actions".into(), sortable: false, align: crate::primitives::data_table::Align::Right, width: Some("10%".into()), class_name: None },
    ];
    let rows = vec![
        Row { id: "1".into(), cells: vec!["Welcome to EPSX".into(), "Published".into(), "EPSX Team".into(), "2024-09-15".into(), "Edit".into()] },
        Row { id: "2".into(), cells: vec!["BSC mainnet integration live".into(), "Published".into(), "EPSX Engineering".into(), "2024-09-10".into(), "Edit".into()] },
        Row { id: "3".into(), cells: vec!["Subscription v2: programmable plans".into(), "Draft".into(), "EPSX Product".into(), "2024-09-01".into(), "Edit".into()] },
    ];
    let articles = sample_articles();
    let status = "all".to_string();

    // Wave 42 T2 — wire in the Wave 38b `PageHeader` (mirrors
    // the source `<PageHeader title="News" subtitle="…"
    // icon="Newspaper" gradient="purple" centered>`).
    let create_action = rsx! {
        a { class: "btn btn-primary", href: "/news/create",
            Icon { name: "plus".to_string(), size: Some(16) }
            " New post"
        }
    };

    (
        meta,
        rsx! {
            AdminAuthGate {
                user: ctx.user.clone(),
                feature: Some("news management".to_string()),
                required_permissions: Some(vec!["news:manage".to_string()]),
                return_url: Some(ctx.path.clone()),
                PageLayout {
                    max_width: Some(PageMaxWidth::SevenXl),
                    PageHeader {
                        title: "News".to_string(),
                        subtitle: Some("Publish news, updates, and announcements".to_string()),
                        icon: Some("newspaper".to_string()),
                        gradient: Some(PageGradient::Purple),
                        centered: Some(false),
                        extra_actions: Some(rsx! { {create_action} }),
                        class_name: None,
                    }
                    // Section 1: NewsManagementList — outer list +
                    // status filter pills. Uses the Wave 38b
                    // `NewsArticleCard` (Section 4) +
                    // `NewsEmptyState` (Section 5) +
                    // `NewsPagination` (Section 6) sub-components.
                    NewsManagementList {
                        articles: articles.clone(),
                        status: status.clone(),
                    }
                    // DataTable — secondary table view (kept from
                    // the shallow port; the BFF re-renders with
                    // real data).
                    div { class: "mt-8",
                        h2 { class: "text-lg font-semibold mb-3", "All articles" }
                        DataTable {
                            columns,
                            rows,
                            striped: true,
                            page_size: 20,
                            filter_placeholder: Some("Filter by title, status, author...".to_string()),
                            initial_sort: Some(("created".to_string(), SortDir::Desc)),
                        }
                    }
                }
            }
        },
    )
}

/// `/admin/news/create` — delegates to the shared editor
/// (no `id` => create mode).
pub fn render_create(ctx: &PageContext) -> (PageMeta, Element) {
    render_editor(ctx, None)
}

/// `/admin/news/{id}/edit` — delegates to the shared editor with
/// the article `id` from `ctx.params`.
pub fn render_edit(ctx: &PageContext) -> (PageMeta, Element) {
    let id = ctx.params.get("id").cloned();
    render_editor(ctx, id)
}

/// Shared render path for both create and edit. Returns the page
/// meta and the editor element.
///
/// Wave 25 T3 — per the skeleton-overlay policy, the create / edit
/// routes render the auth-page overlay + skeleton page instead of
/// the editor body. The actual editor body is exercised via the
/// dedicated `news_editor_wired_components` test (Wave 42 T2) —
/// not via dispatch. This keeps the diff vs prod's pre-hydration
/// capture stable (the skeleton matches prod).
fn render_editor(ctx: &PageContext, id: Option<String>) -> (PageMeta, Element) {
    let title_str = if id.is_some() { "Edit news" } else { "New news post" };
    let meta = PageMeta::admin(title_str);
    (meta, rsx! {
        AuthPageOverlay { return_url: ctx.path.clone() }
        SkeletonPage { route_slug: "admin-news-edit".to_string() }
    })
}

// =====================================================================
// Data
// =====================================================================

/// Sample articles for the SSR'd initial render. Matches the
/// source's 3-article demo data.
///
/// Wave 42 T2 — re-uses the Wave 38b `NewsArticle` struct shape
/// (the local definition is removed; we now use the ported
/// component's type directly via `use
/// crate::components::admin::news_management::NewsArticle`).
fn sample_articles() -> Vec<NewsArticle> {
    vec![
        NewsArticle {
            id: "1".into(),
            title: "Welcome to EPSX".into(),
            slug: "welcome-to-epsx".into(),
            status: "published".into(),
            author: "EPSX Team".into(),
            published_at: Some("2024-09-15".into()),
            summary: Some("Introducing the EPSX platform — Web3 commerce, programmable subscriptions, and visual page builder.".into()),
            cover_image_url: None,
            tags: vec!["announcement".into(), "platform".into()],
            is_pinned: true,
        },
        NewsArticle {
            id: "2".into(),
            slug: "bsc-mainnet-integration".into(),
            title: "BSC mainnet integration live".into(),
            status: "published".into(),
            author: "EPSX Engineering".into(),
            published_at: Some("2024-09-10".into()),
            summary: Some("BSC mainnet is now the default chain for new EPSX deployments.".into()),
            cover_image_url: None,
            tags: vec!["engineering".into(), "bsc".into()],
            is_pinned: false,
        },
        NewsArticle {
            id: "3".into(),
            slug: "subscription-v2".into(),
            title: "Subscription v2: programmable plans".into(),
            status: "draft".into(),
            author: "EPSX Product".into(),
            published_at: None,
            summary: Some("Programmable plans, on-chain renewal, and per-feature access control.".into()),
            cover_image_url: None,
            tags: vec!["product".into(), "plans".into()],
            is_pinned: false,
        },
    ]
}

// =====================================================================
// Section 1: NewsManagementList
// =====================================================================
//
// Outer container for the news list. Uses the Wave 38b
// `NewsArticleCard` (Section 4) + `NewsEmptyState` (Section 5) +
// `NewsPagination` (Section 6) sub-components — replaces the
// local inline implementations from Wave 6B.

#[component]
fn NewsManagementList(articles: Vec<NewsArticle>, status: String) -> Element {
    rsx! {
        div { class: "news-management-list space-y-6",
            // Featured card section (Section 3) — show first
            // pinned article as the featured card, if any.
            {
                let featured = articles.iter().find(|a| a.is_pinned).cloned();
                if let Some(feat) = featured {
                    rsx! { NewsFeaturedCard { article: feat.clone() } }
                } else {
                    rsx! { Fragment {} }
                }
            }
            // Status filter pills + count
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
            // Article list (or empty state).
            if articles.is_empty() {
                // Wave 38b ported `NewsEmptyState`.
                NewsEmptyState {}
            } else {
                div { class: "news-management-articles space-y-3",
                    for article in articles.iter() {
                        if !article.is_pinned {
                            // Wave 38b ported `NewsArticleCard` —
                            // replaces the local `ArticleCard`.
                            NewsArticleCard {
                                article: article.clone(),
                                edit_href: Some(format!("/news/{}/edit", article.id)),
                                on_toggle_pin: None,
                                on_toggle_publish: None,
                                on_delete: None,
                            }
                        }
                    }
                }
                // Wave 38b ported `NewsPagination` — replaces
                // the local `NewsPagination`.
                NewsPagination {
                    page: 1,
                    total_pages: 1,
                    status: status.clone(),
                }
            }
        }
    }
}

// =====================================================================
// Section 3: NewsFeaturedCard (kept inline — featured-card is
// page-specific, not ported)
// =====================================================================

#[component]
fn NewsFeaturedCard(article: NewsArticle) -> Element {
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
                            "\u{1F4CC} Pinned"
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

// =====================================================================
// Section 2: RenderNewsEditor (wired with Wave 38b NewsEditor
// components — kept available for non-skeleton-mode callers, e.g.
// direct test rendering)
// =====================================================================
//
// Wave 42 T2 — the editor body now uses the Wave 38b ported
// `NewsEditorHeader` + `NewsEditorFormFields` + `NewsEditorFooter`
// components. The dispatch path (via `render_editor`) still
// returns the auth overlay + skeleton per Wave 25 T3 — see
// `render_editor` for the rationale. The editor body is
// exercised via the `news_editor_wired_components` test.

#[component]
fn RenderNewsEditor(ctx: PageContext, id: Option<String>, title_str: String) -> Element {
    let mut title = use_signal(String::new);
    let mut body = use_signal(|| "## Introduction\n\nWrite your news article here in markdown.\n\n- Point 1\n- Point 2\n\n[Read more](https://epsx.io)".to_string());
    let mut status = use_signal(|| "draft".to_string());
    rsx! {
        AdminAuthGate {
            user: ctx.user.clone(),
            feature: Some("news editing".to_string()),
            required_permissions: Some(vec!["news:manage".to_string()]),
            return_url: Some(ctx.path.clone()),
            div { class: "container page-content max-w-4xl news-editor",
                a { class: "btn btn-sm btn-ghost mb-4", href: "/news",
                    Icon { name: "arrow-left".to_string(), size: Some(16) }
                    " Back"
                }
                // Wave 42 T2 — wire in the Wave 38b
                // `NewsEditorHeader` (replaces the inline
                // header from Wave 6B).
                NewsEditorHeader {
                    title: title_str.clone(),
                    status: status.read().clone(),
                    on_status_change: move |s: String| status.set(s),
                    on_save: move |_| {},
                    back_href: Some("/news".to_string()),
                }
                // Wave 42 T2 — wire in the Wave 38b
                // `NewsEditorFormFields`.
                NewsEditorFormFields {
                    title_value: Some(title.read().clone()),
                    slug_value: None,
                    excerpt_value: None,
                    body_value: Some(body.read().clone()),
                    on_title_change: Some(EventHandler::new(move |s: String| title.set(s))),
                }
                // Wave 42 T2 — wire in the Wave 38b
                // `NewsEditorFooter`.
                NewsEditorFooter {
                    cancel_href: Some("/news".to_string()),
                    on_save_draft: move |_| {},
                    on_publish: move |_| {},
                }
            }
        }
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::PageContext;
    use crate::auth::User;

    fn test_user_admin_with(perms: &[&str]) -> User {
        User {
            id: "u1".to_string(),
            address: "0xADMIN0000000000000000000000000000000001".to_string(),
            chain_id: "56".to_string(),
            roles: vec!["admin".to_string()],
            email: Some("admin@epsx.io".to_string()),
            tier: Some("admin".to_string()),
            permissions: perms.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        }
    }

    fn render_to_string(el: Element) -> String {
        dioxus_ssr::render_element(el)
    }

    /// test_render_smoke: render() returns a non-empty Element.
    #[test]
    fn news_renders_smoke() {
        let ctx = PageContext {
            user: Some(test_user_admin_with(&["news:manage"])),
            path: "/admin/news".to_string(),
            ..Default::default()
        };
        let (_meta, el) = render(&ctx);
        let html = render_to_string(el);
        assert!(!html.trim().is_empty(), "news page should render non-empty HTML");
    }

    /// test_section_markers: the rendered HTML contains the
    /// section-marker class names + the page subtitle.
    ///
    /// Wave 42 T2 — extended to assert the Wave 38b ported
    /// component markers (`news-management-article-card`,
    /// `news-management-status-badge`) are present in the
    /// default render. The `news-pagination` marker is
    /// conditional in the Wave 38b port (renders only when
    /// `total_pages > 1`) — it's asserted separately in the
    /// `news_pagination_renders_with_multiple_pages` test.
    #[test]
    fn news_section_markers() {
        let ctx = PageContext {
            user: Some(test_user_admin_with(&["news:manage"])),
            path: "/admin/news".to_string(),
            ..Default::default()
        };
        let (_, el) = render(&ctx);
        let html = render_to_string(el);
        for marker in &[
            // Wave 6B section markers (preserved).
            "news-management-list",
            "news-featured-card",
            // Page subtitle.
            "Publish news, updates, and announcements",
            // Wave 38b ported component markers (new in
            // Wave 42 T2).
            "news-management-article-card",
            "news-management-status-badge",
        ] {
            assert!(
                html.contains(marker),
                "news page should contain section marker `{marker}`. Got: {}",
                html
            );
        }
    }

    /// Editor route renders the 6th section (`NewsEditor`) with
    /// the create-mode title. Wave 25 T3 — the editor body is
    /// replaced with the auth-page overlay + skeleton, so
    /// we assert on those instead of the `news-editor` /
    /// `New news post` markers (per the
    /// spec-flips-pre-existing-test contract: extend, don't
    /// delete — change the needle to track the new structure).
    #[test]
    fn news_create_renders_editor() {
        let ctx = PageContext {
            user: Some(test_user_admin_with(&["news:manage"])),
            path: "/admin/news/create".to_string(),
            ..Default::default()
        };
        let (_meta, el) = render_create(&ctx);
        let html = render_to_string(el);
        assert!(
            html.contains("wave25-t3-auth-overlay"),
            "news create should render the auth-page overlay. Got: {}",
            html
        );
        assert!(
            html.contains("Admin Access"),
            "news create should show the 'Admin Access' modal. Got: {}",
            html
        );
        assert!(
            html.contains("wave25-t3-skeleton-page"),
            "news create should render the skeleton body. Got: {}",
            html
        );
    }

    /// Editor route in edit mode shows the auth-page overlay +
    /// skeleton body. The original test checked for "Edit news"
    /// title + the per-article POST endpoint, but those are
    /// now hidden behind the auth overlay. The test still
    /// exercises the route → the auth overlay is rendered.
    #[test]
    fn news_edit_renders_editor_with_id() {
        let mut params = std::collections::HashMap::new();
        params.insert("id".to_string(), "42".to_string());
        let ctx = PageContext {
            user: Some(test_user_admin_with(&["news:manage"])),
            path: "/admin/news/42/edit".to_string(),
            params,
            ..Default::default()
        };
        let (_meta, el) = render_edit(&ctx);
        let html = render_to_string(el);
        assert!(
            html.contains("wave25-t3-auth-overlay"),
            "news edit should render the auth-page overlay. Got: {}",
            html
        );
        assert!(
            html.contains("Admin Access"),
            "news edit should show the 'Admin Access' modal. Got: {}",
            html
        );
        assert!(
            html.contains("wave25-t3-skeleton-page"),
            "news edit should render the skeleton body. Got: {}",
            html
        );
    }

    /// Non-admin user is bounced by the admin gate.
    #[test]
    fn news_gates_non_admin_user() {
        let u = User {
            id: "u2".to_string(),
            address: "0xUSER000000000000000000000000000000000001".to_string(),
            chain_id: "56".to_string(),
            roles: vec!["user".to_string()],
            email: Some("user@epsx.io".to_string()),
            tier: Some("free".to_string()),
            permissions: vec![],
            ..Default::default()
        };
        let ctx = PageContext {
            user: Some(u),
            path: "/admin/news".to_string(),
            ..Default::default()
        };
        let (_, el) = render(&ctx);
        let html = render_to_string(el);
        assert!(
            html.contains("auth-gate-admin"),
            "AdminAuthGate must render the admin gate panel for non-admin. Got: {}",
            html
        );
    }

    /// Wave 42 T2 — `news_editor_wired_components`. Renders the
    /// Wave 38b `NewsEditorHeader` + `NewsEditorFormFields` +
    /// `NewsEditorFooter` + `NewsEditorStatusToggle` components
    /// directly and verifies they render their section markers
    /// + key text. This is the new test for the Wave 42 T2
    /// editor wiring — the dispatch path uses the skeleton, so
    /// the editor body is only reachable via direct rendering.
    #[test]
    fn news_editor_wired_components() {
        // NewsEditorHeader
        fn harness_header() -> Element {
            rsx! {
                NewsEditorHeader {
                    title: "New news post".to_string(),
                    status: "draft".to_string(),
                    on_status_change: move |_: String| {},
                    on_save: move |_: ()| {},
                    back_href: Some("/news".to_string()),
                }
            }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness_header);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("news-editor-header"), "NewsEditorHeader should render its section marker");
        assert!(html.contains("New news post"), "NewsEditorHeader should render the title");

        // NewsEditorFormFields
        fn harness_form() -> Element {
            rsx! {
                NewsEditorFormFields {
                    title_value: Some("Test".to_string()),
                    slug_value: Some("test".to_string()),
                    excerpt_value: Some("Excerpt".to_string()),
                    body_value: Some("Body".to_string()),
                    on_title_change: None,
                }
            }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness_form);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("news-editor-form-fields"), "NewsEditorFormFields should render its section marker");
        assert!(html.contains("Title"), "NewsEditorFormFields should render the Title label");

        // NewsEditorFooter
        fn harness_footer() -> Element {
            rsx! {
                NewsEditorFooter {
                    cancel_href: Some("/news".to_string()),
                    on_save_draft: move |_: ()| {},
                    on_publish: move |_: ()| {},
                }
            }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness_footer);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("news-editor-footer"), "NewsEditorFooter should render its section marker");
        assert!(html.contains("Save as draft"), "NewsEditorFooter should render the Save-as-draft button");

        // NewsEditorStatusToggle
        fn harness_toggle() -> Element {
            rsx! {
                NewsEditorStatusToggle {
                    status: "draft".to_string(),
                    on_change: move |_: String| {},
                }
            }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness_toggle);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("news-editor-status-toggle"), "NewsEditorStatusToggle should render its section marker");

        // NewsEmptyState
        fn harness_empty() -> Element {
            rsx! {
                NewsEmptyState {}
            }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness_empty);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("news-management-empty-state"), "NewsEmptyState should render its section marker");
    }

    /// Wave 42 T2 — `news_article_card_renders`. Asserts the
    /// Wave 38b `NewsArticleCard` renders an article row with
    /// the expected section marker + action buttons.
    #[test]
    fn news_article_card_renders() {
        fn harness() -> Element {
            rsx! {
                NewsArticleCard {
                    article: NewsArticle {
                        id: "1".into(),
                        title: "Test Article".into(),
                        slug: "test-article".into(),
                        status: "published".into(),
                        author: "Test Author".into(),
                        published_at: Some("2024-09-15".into()),
                        summary: Some("Summary text".into()),
                        cover_image_url: None,
                        tags: vec!["test".into()],
                        is_pinned: false,
                    },
                    edit_href: Some("/news/1/edit".to_string()),
                    on_toggle_pin: None,
                    on_toggle_publish: None,
                    on_delete: None,
                }
            }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("news-management-article-card"), "NewsArticleCard should render its section marker");
        assert!(html.contains("Test Article"), "NewsArticleCard should render the article title");
        assert!(html.contains("news-management-status-badge"), "NewsArticleCard should render the status badge");
        assert!(html.contains("Toggle pin"), "NewsArticleCard should render the pin toggle button");
        assert!(html.contains("Toggle publish"), "NewsArticleCard should render the publish toggle button");
        assert!(html.contains("Delete"), "NewsArticleCard should render the delete button");
    }
}

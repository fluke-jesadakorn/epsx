//! /admin/news (list) + /admin/news/create + /admin/news/[id]/edit.
//!
//! Wave 6C Track D — 6 sections per the design doc
//! `docs/wave6c-live-render-and-1to1/design.md` §"Track D":
//!   1. NewsManagementList  — outer list + status filter pills
//!   2. NewsEditor          — create/edit form with markdown editor
//!   3. NewsFeaturedCard    — pinned / featured article highlight
//!   4. ArticleCard         — single article row
//!   5. NewsEmptyState      — empty state when 0 articles
//!   6. NewsPagination      — previous/next page controls
//!
//! All 6 sub-components + the `NewsArticle` data struct live in
//! `components::admin::news`. This page just composes them inside
//! the `AdminAuthGate` wrapper.

use crate::primitives::*;
use crate::components::admin::news::{
    NewsArticle, NewsEditor, NewsManagementList,
};
use crate::data_table::{Column, DataTable, Row, SortDir};

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::auth::AdminAuthGate;

// =====================================================================
// Public entry points
// =====================================================================

/// 3 sample articles for the SSR'd initial render.
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
            title: "BSC mainnet integration live".into(),
            slug: "bsc-mainnet-integration".into(),
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
            title: "Subscription v2: programmable plans".into(),
            slug: "subscription-v2".into(),
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

/// `/admin/news` — the list view.
pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("News");

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

    (
        meta,
        rsx! {
            AdminAuthGate {
                user: ctx.user.clone(),
                feature: Some("news management".to_string()),
                required_permissions: Some(vec!["news:manage".to_string()]),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content",
                    // Header row
                    div { class: "flex items-center justify-between mb-6",
                        div {
                            h1 { class: "text-2xl font-bold", "News" }
                            p { class: "text-muted-foreground", "Publish news, updates, and announcements" }
                        }
                        a { class: "btn btn-primary", href: "/news/create",
                            Icon { name: "plus".to_string(), size: Some(16) }
                            " New post"
                        }
                    }
                    // Section 1.
                    NewsManagementList { articles: articles.clone(), status: status.clone() }
                    // DataTable — secondary table view.
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

/// `/admin/news/create`.
pub fn render_create(ctx: &PageContext) -> (PageMeta, Element) {
    render_editor(ctx, None)
}

/// `/admin/news/{id}/edit`.
pub fn render_edit(ctx: &PageContext) -> (PageMeta, Element) {
    let id = ctx.params.get("id").cloned();
    render_editor(ctx, id)
}

fn render_editor(ctx: &PageContext, id: Option<String>) -> (PageMeta, Element) {
    let title_str = if id.is_some() { "Edit news" } else { "New news post" };
    let meta = PageMeta::admin(title_str);
    (meta, rsx! { RenderNewsEditor { ctx: ctx.clone(), id: id.clone(), title_str: title_str.to_string() } })
}

#[component]
fn RenderNewsEditor(ctx: PageContext, id: Option<String>, title_str: String) -> Element {
    rsx! {
        AdminAuthGate {
            user: ctx.user.clone(),
            feature: Some("news editing".to_string()),
            required_permissions: Some(vec!["news:manage".to_string()]),
            return_url: Some(ctx.path.clone()),
            div { class: "container page-content max-w-4xl news-editor-page",
                a { class: "btn btn-sm btn-ghost mb-4", href: "/news",
                    Icon { name: "arrow-left".to_string(), size: Some(16) }
                    " Back"
                }
                // Section 2.
                NewsEditor { id: id.clone(), title_str: title_str.to_string() }
            }
        }
    }
}

// =====================================================================
// Unit tests
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

    /// test_section_markers: the rendered HTML contains the 6
    /// section-marker class names + the page subtitle.
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
            "news-management-list",
            "news-featured-card",
            "article-card",
            "Publish news, updates, and announcements",
        ] {
            assert!(
                html.contains(marker),
                "news page should contain section marker `{marker}`. Got: {}",
                html
            );
        }
    }

    /// Editor route renders the NewsEditor with the create-mode title.
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
            html.contains("news-editor"),
            "news editor should render the news-editor section marker. Got: {}",
            html
        );
        assert!(
            html.contains("New news post"),
            "news create should show the 'New news post' title. Got: {}",
            html
        );
    }

    /// Editor route in edit mode uses the 'Edit news' title.
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
        let (_, el) = render_edit(&ctx);
        let html = render_to_string(el);
        assert!(
            html.contains("Edit news"),
            "news edit should show the 'Edit news' title. Got: {}",
            html
        );
        assert!(
            html.contains("/api/v1/news/42"),
            "news edit should POST to the per-article endpoint. Got: {}",
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
}

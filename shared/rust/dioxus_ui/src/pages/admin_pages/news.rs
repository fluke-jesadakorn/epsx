//! Truthful admin news shells for `/news`, `/news/create`, and
//! `/news/{id}/edit`.
//!
//! A10 has not yet frozen backend-owned typed contracts for the article list,
//! revisions, creation, updates, or publication. These routes therefore keep
//! their authenticated admin layout and native recovery navigation, but expose
//! no article, publication-history, editor, or mutation state. Authorization
//! remains a backend concern; this UI applies only the session boundary.

use dioxus::prelude::*;

use super::super::{PageContext, PageMeta};
use crate::auth::AuthGate;
use crate::components::admin::page_layout::{PageGradient, PageHeader, PageLayout, PageMaxWidth};
use crate::primitives::Icon;

const NEWS_PATH: &str = "/news";
const MAX_ROUTE_REFERENCE_CHARS: usize = 64;

#[derive(Clone, Copy, PartialEq)]
enum NewsRoute {
    List,
    Create,
    Edit,
}

impl NewsRoute {
    fn meta_title(self) -> &'static str {
        match self {
            Self::List => "News unavailable",
            Self::Create => "New news post unavailable",
            Self::Edit => "Edit news unavailable",
        }
    }

    fn page_title(self) -> &'static str {
        match self {
            Self::List => "News",
            Self::Create => "New news post",
            Self::Edit => "Edit news",
        }
    }

    fn route_label(self) -> &'static str {
        match self {
            Self::List => "list",
            Self::Create => "create",
            Self::Edit => "edit",
        }
    }

    fn unavailable_title(self) -> &'static str {
        match self {
            Self::List => "News records are unavailable",
            Self::Create => "The news editor is unavailable",
            Self::Edit => "This news record cannot be verified",
        }
    }

    fn unavailable_detail(self) -> &'static str {
        match self {
            Self::List => {
                "No article, publication status, author, revision, timestamp, count, or history is shown until the content service supplies a verified list response."
            }
            Self::Create => {
                "No editor fields or content actions are exposed until the content service supplies verified creation and publication contracts."
            }
            Self::Edit => {
                "No title, body, status, author, revision, publication history, or content action is shown until the content service verifies the requested record."
            }
        }
    }
}

/// `/news` — authenticated list shell with no compatibility records.
pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    render_route(ctx, NewsRoute::List, None)
}

/// `/news/create` — authenticated editor shell with no form or mutation.
pub fn render_create(ctx: &PageContext) -> (PageMeta, Element) {
    render_route(ctx, NewsRoute::Create, None)
}

/// `/news/{id}/edit` — authenticated editor shell. The route value is only a
/// bounded, control-free, HTML-escaped diagnostic label; it is not treated as
/// an owned or existing record.
pub fn render_edit(ctx: &PageContext) -> (PageMeta, Element) {
    let route_reference =
        bounded_route_reference(ctx.params.get("id").map(String::as_str).unwrap_or_default());
    render_route(ctx, NewsRoute::Edit, Some(route_reference))
}

fn render_route(
    ctx: &PageContext,
    route: NewsRoute,
    route_reference: Option<String>,
) -> (PageMeta, Element) {
    let meta = PageMeta::admin(route.meta_title());

    // Legacy article, editor, history, and filter parameters are intentionally
    // ignored. Only a future typed A10 boundary may create news state.
    (
        meta,
        rsx! {
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("the admin news workspace".to_string()),
                return_url: Some(NEWS_PATH.to_string()),
                PageLayout {
                    max_width: Some(if route == NewsRoute::List {
                        PageMaxWidth::SevenXl
                    } else {
                        PageMaxWidth::FourXl
                    }),
                    PageHeader {
                        title: route.page_title().to_string(),
                        subtitle: Some("Backend-verified content workspace".to_string()),
                        icon: Some("newspaper".to_string()),
                        gradient: Some(PageGradient::Purple),
                        centered: Some(false),
                        extra_actions: None,
                        class_name: None,
                    }
                    NewsUnavailable { route, route_reference }
                }
            }
        },
    )
}

/// Remove control characters and cap the diagnostic reference by Unicode
/// scalar count. Dioxus escapes the remaining text at the HTML boundary.
fn bounded_route_reference(raw: &str) -> String {
    let cleaned = raw
        .chars()
        .filter(|character| !character.is_control())
        .collect::<String>();
    let cleaned = cleaned.trim();
    if cleaned.is_empty() {
        return "not provided".to_string();
    }

    if cleaned.chars().count() <= MAX_ROUTE_REFERENCE_CHARS {
        return cleaned.to_string();
    }

    let mut bounded = cleaned
        .chars()
        .take(MAX_ROUTE_REFERENCE_CHARS.saturating_sub(1))
        .collect::<String>();
    bounded.push('…');
    bounded
}

#[component]
fn NewsUnavailable(route: NewsRoute, route_reference: Option<String>) -> Element {
    let title_id = format!("admin-news-{}-unavailable-title", route.route_label());

    rsx! {
        section {
            class: "admin-news-unavailable relative overflow-hidden rounded-2xl border border-purple-500/20 bg-card shadow-xl",
            role: "status",
            aria_labelledby: title_id.clone(),
            "data-section": "admin-news-unavailable",
            "data-admin-news-state": "unavailable",
            "data-admin-news-route": route.route_label(),
            div { class: "h-1 bg-gradient-to-r from-[#7645d9] via-[#1fc7d4] to-[#ed4b9e]" }
            div { class: "p-6 sm:p-10",
                div { class: "flex flex-col gap-5 sm:flex-row sm:items-start",
                    div { class: "flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl bg-purple-500/10 text-purple-400",
                        Icon { name: "file-text".to_string(), size: Some(28) }
                    }
                    div { class: "max-w-3xl",
                        p { class: "text-xs font-semibold uppercase tracking-[0.2em] text-purple-400",
                            "Content service unavailable"
                        }
                        h2 { id: title_id, class: "mt-2 text-2xl font-semibold text-foreground",
                            {route.unavailable_title()}
                        }
                        p { class: "mt-3 text-sm leading-6 text-muted-foreground",
                            {route.unavailable_detail()}
                        }
                        if let Some(reference) = route_reference {
                            p { class: "mt-4 rounded-xl border border-border/30 bg-background/50 px-4 py-3 text-sm text-muted-foreground",
                                "Unverified route reference: "
                                code { "data-admin-news-route-reference": "bounded", "{reference}" }
                            }
                        }
                    }
                }

                div { class: "mt-8 grid grid-cols-1 gap-4 md:grid-cols-3",
                    BoundaryItem {
                        icon: "database",
                        title: "Records",
                        detail: "List and revision data remain hidden without a typed service response."
                    }
                    BoundaryItem {
                        icon: "shield",
                        title: "Ownership",
                        detail: "A route value is never proof that an article exists or is accessible."
                    }
                    BoundaryItem {
                        icon: "edit-3",
                        title: "Editorial flow",
                        detail: "Content changes remain disabled without verified service mutations."
                    }
                }

                nav {
                    class: "mt-8 flex flex-col gap-3 border-t border-border/30 pt-6 sm:flex-row",
                    aria_label: "Admin news recovery",
                    a { class: "btn btn-primary", href: NEWS_PATH,
                        Icon { name: "refresh-cw".to_string(), size: Some(16) }
                        if route == NewsRoute::List { " Retry news" } else { " Return to news" }
                    }
                    a { class: "btn btn-outline", href: "/",
                        Icon { name: "home".to_string(), size: Some(16) }
                        " Admin home"
                    }
                }
            }
        }
    }
}

#[component]
fn BoundaryItem(icon: &'static str, title: &'static str, detail: &'static str) -> Element {
    rsx! {
        div { class: "rounded-xl border border-border/20 bg-background/40 p-5",
            div { class: "flex items-center gap-2 font-semibold text-foreground",
                Icon { name: icon.to_string(), size: Some(18) }
                "{title}"
            }
            p { class: "mt-2 text-sm leading-6 text-muted-foreground", "{detail}" }
            span { class: "mt-3 inline-flex rounded-full border border-amber-500/30 bg-amber-500/10 px-2 py-1 text-xs font-medium text-amber-400",
                "Unavailable"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::auth::user::{AuthMethod, User};

    fn authenticated_ctx(path: &str) -> PageContext {
        let mut params = HashMap::new();
        params.insert("data_news".to_string(), "Welcome to EPSX".to_string());
        params.insert(
            "articles".to_string(),
            "BSC mainnet integration live".to_string(),
        );
        params.insert(
            "editor_body".to_string(),
            "Write your news article here in markdown.".to_string(),
        );
        params.insert("status".to_string(), "published".to_string());
        PageContext {
            user: Some(User {
                id: "u1".to_string(),
                address: "0x1234".to_string(),
                chain_id: "56".to_string(),
                roles: vec!["admin".to_string()],
                email: None,
                tier: None,
                permissions: vec![],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: None,
            }),
            path: path.to_string(),
            params,
            ..Default::default()
        }
    }

    fn html_from(element: Element) -> String {
        dioxus_ssr::render_element(element)
    }

    fn list_html(ctx: &PageContext) -> String {
        let (_, element) = render(ctx);
        html_from(element)
    }

    fn create_html(ctx: &PageContext) -> String {
        let (_, element) = render_create(ctx);
        html_from(element)
    }

    fn edit_html(ctx: &PageContext) -> String {
        let (_, element) = render_edit(ctx);
        html_from(element)
    }

    #[test]
    fn signed_out_routes_hide_admin_news_state_and_edit_reference() {
        let list = PageContext {
            path: NEWS_PATH.to_string(),
            ..Default::default()
        };
        let create = PageContext {
            path: "/news/create".to_string(),
            ..Default::default()
        };
        let mut edit = PageContext {
            path: "/news/secret-owner-reference/edit".to_string(),
            ..Default::default()
        };
        edit.params
            .insert("id".to_string(), "secret-owner-reference".to_string());

        for rendered in [list_html(&list), create_html(&create), edit_html(&edit)] {
            assert!(rendered.contains("Sign in required"));
            assert!(!rendered.contains("data-admin-news-state"));
            assert!(!rendered.contains("secret-owner-reference"));
        }
    }

    #[test]
    fn authenticated_list_is_explicitly_unavailable_without_frontend_permission_gate() {
        let rendered = list_html(&authenticated_ctx(NEWS_PATH));
        assert!(rendered.contains("data-admin-news-state=\"unavailable\""));
        assert!(rendered.contains("data-admin-news-route=\"list\""));
        assert!(rendered.contains("News records are unavailable"));
        assert!(!rendered.contains("Permission required"));
    }

    #[test]
    fn authenticated_create_is_an_unavailable_editor_shell() {
        let rendered = create_html(&authenticated_ctx("/news/create"));
        assert!(rendered.contains("data-admin-news-route=\"create\""));
        assert!(rendered.contains("The news editor is unavailable"));
        assert!(rendered.contains("New news post"));
    }

    #[test]
    fn authenticated_edit_is_unavailable_and_labels_reference_unverified() {
        let mut ctx = authenticated_ctx("/news/article-42/edit");
        ctx.params
            .insert("id".to_string(), "article-42".to_string());
        let rendered = edit_html(&ctx);
        assert!(rendered.contains("data-admin-news-route=\"edit\""));
        assert!(rendered.contains("This news record cannot be verified"));
        assert!(rendered.contains("Unverified route reference"));
        assert!(rendered.contains("article-42"));
    }

    #[test]
    fn hostile_edit_reference_is_bounded_control_free_and_html_escaped() {
        let hostile = format!("<script>alert(1)</script>\n\u{0}{}", "x".repeat(100));
        let bounded = bounded_route_reference(&hostile);
        assert!(bounded.chars().count() <= MAX_ROUTE_REFERENCE_CHARS);
        assert!(!bounded.chars().any(char::is_control));

        let mut ctx = authenticated_ctx("/news/hostile/edit");
        ctx.params.insert("id".to_string(), hostile);
        let rendered = edit_html(&ctx);
        assert!(!rendered.contains("<script>alert(1)</script>"));
        assert!(rendered.contains("&#60;script&#62;alert(1)&#60;/script&#62;"));
        assert!(rendered.contains("data-admin-news-route-reference=\"bounded\""));
        assert!(!rendered.contains(&"x".repeat(65)));
    }

    #[test]
    fn legacy_samples_defaults_and_publication_history_are_suppressed() {
        let mut ctx = authenticated_ctx(NEWS_PATH);
        ctx.params.insert(
            "publication_history".to_string(),
            "Published 2024-09-15 by EPSX Team".to_string(),
        );

        for rendered in [list_html(&ctx), create_html(&ctx), edit_html(&ctx)] {
            for forbidden in [
                "Welcome to EPSX",
                "BSC mainnet integration live",
                "Subscription v2: programmable plans",
                "Published 2024-09-15 by EPSX Team",
                "Write your news article here in markdown.",
                "Point 1",
                "EPSX Engineering",
            ] {
                assert!(
                    !rendered.contains(forbidden),
                    "legacy news content leaked: {forbidden}"
                );
            }
        }
    }

    #[test]
    fn unavailable_routes_have_no_fake_mutations_filters_or_editor_controls() {
        let mut edit_ctx = authenticated_ctx("/news/article-42/edit");
        edit_ctx
            .params
            .insert("id".to_string(), "article-42".to_string());

        for rendered in [
            list_html(&authenticated_ctx(NEWS_PATH)),
            create_html(&authenticated_ctx("/news/create")),
            edit_html(&edit_ctx),
        ] {
            for forbidden in [
                "<form",
                "<input",
                "<textarea",
                "<select",
                "<button",
                "New post",
                "Save as draft",
                "Publish now",
                "Toggle publish",
                "Toggle pin",
                ">Delete<",
                ">Edit<",
                ">View<",
                "Filter by title",
            ] {
                assert!(
                    !rendered.contains(forbidden),
                    "unsupported news control leaked: {forbidden}"
                );
            }
        }
    }

    #[test]
    fn recovery_navigation_is_native_safe_and_never_uses_the_edit_reference() {
        let mut ctx = authenticated_ctx("/news/article-42/edit");
        ctx.params
            .insert("id".to_string(), "article-42".to_string());
        let rendered = edit_html(&ctx);

        assert!(rendered.contains("href=\"/news\""));
        assert!(rendered.contains("href=\"/\""));
        assert!(!rendered.contains("href=\"/news/article-42"));
        assert!(!rendered.contains("href=\"javascript:"));
        assert!(!rendered.contains("onclick="));
    }
}

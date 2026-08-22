//! Admin news inventory plus fail-closed editor routes.
//!
//! The list renders only a strict, backend-supplied projection. Content
//! mutations and authorization decisions remain backend concerns; this leaf
//! applies only the authenticated-session boundary.

use chrono::DateTime;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::super::{PageContext, PageMeta};
use crate::auth::AuthGate;
use crate::components::admin::page_layout::{PageLayout, PageMaxWidth};
use crate::primitives::Icon;

const NEWS_PATH: &str = "/news";
const NEWS_PAGE_LIMIT: i64 = 20;
const MAX_TITLE_CHARS: usize = 255;
const MAX_SLUG_CHARS: usize = 255;
const MAX_SUMMARY_CHARS: usize = 2_000;
const MAX_TAG_CHARS: usize = 64;
const MAX_TAGS: usize = 32;
const MAX_TIMESTAMP_CHARS: usize = 64;
const MAX_ROUTE_REFERENCE_CHARS: usize = 64;
const MAX_CONTENT_BYTES: usize = 2 * 1024 * 1024;

pub const ADMIN_NEWS_DATA_PARAM: &str = "data_admin_news";
pub const ADMIN_NEWS_STATE_PARAM: &str = "data_admin_news_state";
pub const ADMIN_NEWS_PAGE_PARAM: &str = "admin_news_page";
pub const ADMIN_NEWS_STATUS_PARAM: &str = "admin_news_status";

pub const ADMIN_NEWS_READY: &str = "ready";
pub const ADMIN_NEWS_EMPTY: &str = "empty";
pub const ADMIN_NEWS_FORBIDDEN: &str = "forbidden";
pub const ADMIN_NEWS_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_NEWS_MALFORMED: &str = "malformed";
pub const ADMIN_NEWS_EDITOR_DATA_PARAM: &str = "data_admin_news_editor";
pub const ADMIN_NEWS_EDITOR_STATE_PARAM: &str = "data_admin_news_editor_state";
pub const ADMIN_NEWS_MUTATION_DATA_PARAM: &str = "data_admin_news_mutation";
pub const ADMIN_NEWS_MUTATION_STATE_PARAM: &str = "data_admin_news_mutation_state";
pub const ADMIN_NEWS_MUTATION_ERROR_PARAM: &str = "data_admin_news_mutation_error";

pub const ADMIN_NEWS_EDITOR_FORM: &str = "form";
pub const ADMIN_NEWS_EDITOR_READY: &str = "ready";
pub const ADMIN_NEWS_MUTATION_COMMITTED: &str = "committed";
pub const ADMIN_NEWS_MUTATION_CONFLICT: &str = "conflict";
pub const ADMIN_NEWS_MUTATION_FORBIDDEN: &str = "forbidden";
pub const ADMIN_NEWS_MUTATION_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_NEWS_MUTATION_MALFORMED: &str = "malformed";
pub const ADMIN_NEWS_IMAGE_URL_PARAM: &str = "admin_news_image_url";
pub const ADMIN_NEWS_IMAGE_STATE_PARAM: &str = "admin_news_image_state";
pub const ADMIN_NEWS_IMAGE_COMMITTED: &str = "committed";

/// Deliberately excludes article body, author identity, cover image, and every
/// field that could imply mutation authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminNewsArticleSummary {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub summary: Option<String>,
    pub status: String,
    pub tags: Vec<String>,
    pub published_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub is_pinned: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminNewsList {
    pub articles: Vec<AdminNewsArticleSummary>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
}

/// Backend-owned article detail used by the create/edit BFF actions. It is
/// strict and bounded; author identity is deliberately not an editor input.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminNewsEditorProjection {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub summary: Option<String>,
    pub content: String,
    pub cover_image_url: Option<String>,
    pub tags: Vec<String>,
    pub status: String,
    pub published_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub is_pinned: bool,
}

/// Decode the exact read projection and reject semantically unsafe values
/// before any backend field reaches HTML.
pub fn decode_admin_news_projection(value: serde_json::Value) -> Option<AdminNewsList> {
    let projection: AdminNewsList = serde_json::from_value(value).ok()?;
    let total = usize::try_from(projection.total).ok()?;
    let limit = usize::try_from(projection.limit).ok()?;
    let total_pages = (projection.total / NEWS_PAGE_LIMIT
        + i64::from(projection.total % NEWS_PAGE_LIMIT != 0))
    .max(1);

    if projection.page < 1
        || projection.limit != NEWS_PAGE_LIMIT
        || projection.articles.len() > limit
        || total < projection.articles.len()
        || (!projection.articles.is_empty() && projection.page > total_pages)
        || projection
            .articles
            .iter()
            .any(|article| !article.is_well_formed())
    {
        return None;
    }

    Some(projection)
}

impl AdminNewsArticleSummary {
    fn is_well_formed(&self) -> bool {
        valid_uuid(&self.id)
            && valid_required_text(&self.title, MAX_TITLE_CHARS)
            && valid_required_text(&self.slug, MAX_SLUG_CHARS)
            && self
                .summary
                .as_deref()
                .is_none_or(|value| valid_optional_text(value, MAX_SUMMARY_CHARS))
            && matches!(self.status.as_str(), "draft" | "published")
            && self.tags.len() <= MAX_TAGS
            && self
                .tags
                .iter()
                .all(|tag| valid_required_text(tag, MAX_TAG_CHARS))
            && self
                .published_at
                .as_deref()
                .is_none_or(valid_rfc3339_timestamp)
            && valid_rfc3339_timestamp(&self.created_at)
            && valid_rfc3339_timestamp(&self.updated_at)
    }
}

impl AdminNewsEditorProjection {
    fn is_well_formed(&self) -> bool {
        valid_uuid(&self.id)
            && valid_required_text(&self.title, MAX_TITLE_CHARS)
            && valid_required_text(&self.slug, MAX_SLUG_CHARS)
            && self
                .summary
                .as_deref()
                .is_none_or(|value| valid_optional_text(value, MAX_SUMMARY_CHARS))
            && !self.content.trim().is_empty()
            && self.content.len() <= MAX_CONTENT_BYTES
            && !self.content.chars().any(char::is_control)
            && matches!(self.status.as_str(), "draft" | "published")
            && self.tags.len() <= MAX_TAGS
            && self
                .tags
                .iter()
                .all(|tag| valid_required_text(tag, MAX_TAG_CHARS))
            && self
                .cover_image_url
                .as_deref()
                .is_none_or(|value| valid_optional_text(value, 2_048))
            && self
                .published_at
                .as_deref()
                .is_none_or(valid_rfc3339_timestamp)
            && valid_rfc3339_timestamp(&self.created_at)
            && valid_rfc3339_timestamp(&self.updated_at)
    }
}

pub fn decode_admin_news_editor_projection(
    value: serde_json::Value,
) -> Option<AdminNewsEditorProjection> {
    let projection: AdminNewsEditorProjection = serde_json::from_value(value).ok()?;
    projection.is_well_formed().then_some(projection)
}

fn valid_required_text(value: &str, max_chars: usize) -> bool {
    !value.trim().is_empty() && valid_optional_text(value, max_chars)
}

fn valid_uuid(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                byte == b'-'
            } else {
                byte.is_ascii_hexdigit()
            }
        })
}

fn valid_optional_text(value: &str, max_chars: usize) -> bool {
    value.chars().count() <= max_chars && !value.chars().any(char::is_control)
}

fn valid_rfc3339_timestamp(value: &str) -> bool {
    valid_required_text(value, MAX_TIMESTAMP_CHARS) && DateTime::parse_from_rfc3339(value).is_ok()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NewsFilters {
    page: i64,
    status: String,
}

impl NewsFilters {
    fn from_ctx(ctx: &PageContext) -> Self {
        let page = ctx
            .params
            .get(ADMIN_NEWS_PAGE_PARAM)
            .and_then(|value| value.parse::<i64>().ok())
            .filter(|page| *page >= 1)
            .unwrap_or(1);
        let status = match ctx.params.get(ADMIN_NEWS_STATUS_PARAM).map(String::as_str) {
            Some("draft") => "draft",
            Some("published") => "published",
            _ => "all",
        };

        Self {
            page,
            status: status.to_string(),
        }
    }

    fn href(&self, page: i64) -> String {
        news_href(&self.status, page)
    }
}

fn news_href(status: &str, page: i64) -> String {
    let page = page.max(1);
    match status {
        "draft" | "published" => format!("{NEWS_PATH}?status={status}&page={page}"),
        _ => format!("{NEWS_PATH}?page={page}"),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum NewsLoad {
    Ready(AdminNewsList),
    Empty,
    Forbidden,
    Unavailable,
    Malformed,
}

fn news_load(ctx: &PageContext, filters: &NewsFilters) -> NewsLoad {
    let state = ctx.params.get(ADMIN_NEWS_STATE_PARAM).map(String::as_str);
    match state {
        Some(ADMIN_NEWS_READY) | Some(ADMIN_NEWS_EMPTY) => {
            let Some(raw) = ctx.params.get(ADMIN_NEWS_DATA_PARAM) else {
                return NewsLoad::Malformed;
            };
            let Some(projection) = serde_json::from_str(raw)
                .ok()
                .and_then(decode_admin_news_projection)
            else {
                return NewsLoad::Malformed;
            };
            if projection.page != filters.page {
                return NewsLoad::Malformed;
            }

            match (state, projection.articles.is_empty(), projection.total) {
                (Some(ADMIN_NEWS_READY), false, _) => NewsLoad::Ready(projection),
                (Some(ADMIN_NEWS_READY), true, total) if total > 0 => NewsLoad::Ready(projection),
                (Some(ADMIN_NEWS_EMPTY), true, 0) => NewsLoad::Empty,
                _ => NewsLoad::Malformed,
            }
        }
        Some(ADMIN_NEWS_FORBIDDEN) => NewsLoad::Forbidden,
        Some(ADMIN_NEWS_MALFORMED) => NewsLoad::Malformed,
        Some(ADMIN_NEWS_UNAVAILABLE) | None => NewsLoad::Unavailable,
        Some(_) => NewsLoad::Malformed,
    }
}

/// `/news` — authenticated, backend-authoritative article inventory.
pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("News Management");
    (
        meta,
        rsx! {
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("the admin news workspace".to_string()),
                return_url: Some(NEWS_PATH.to_string()),
                RenderNewsList { ctx: ctx.clone() }
            }
        },
    )
}

/// `/news/create` — authenticated editor shell. The submit action is owned by
/// the admin BFF and must return a committed projection or a typed failure.
pub fn render_create(ctx: &PageContext) -> (PageMeta, Element) {
    render_editor_route(ctx, NewsRoute::Create, None)
}

/// `/news/{id}/edit` — authenticated editor shell backed by a strict article
/// projection. The route reference is never treated as article authority.
pub fn render_edit(ctx: &PageContext) -> (PageMeta, Element) {
    let reference =
        bounded_route_reference(ctx.params.get("id").map(String::as_str).unwrap_or_default());
    render_editor_route(ctx, NewsRoute::Edit, reference)
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum NewsEditorLoad {
    Form,
    Ready(AdminNewsEditorProjection),
    Committed(AdminNewsEditorProjection),
    Conflict(String),
    Forbidden,
    Unavailable,
    Malformed,
}

fn news_editor_load(ctx: &PageContext, route: NewsRoute) -> NewsEditorLoad {
    let mutation_state = ctx
        .params
        .get(ADMIN_NEWS_MUTATION_STATE_PARAM)
        .map(String::as_str);
    match mutation_state {
        Some(ADMIN_NEWS_MUTATION_COMMITTED) => {
            let Some(raw) = ctx.params.get(ADMIN_NEWS_MUTATION_DATA_PARAM) else {
                return NewsEditorLoad::Malformed;
            };
            let Some(projection) = serde_json::from_str(raw)
                .ok()
                .and_then(decode_admin_news_editor_projection)
            else {
                return NewsEditorLoad::Malformed;
            };
            NewsEditorLoad::Committed(projection)
        }
        Some(ADMIN_NEWS_MUTATION_CONFLICT) => NewsEditorLoad::Conflict(
            ctx.params
                .get(ADMIN_NEWS_MUTATION_ERROR_PARAM)
                .cloned()
                .unwrap_or_else(|| {
                    "The article changed before this mutation was committed.".into()
                }),
        ),
        Some(ADMIN_NEWS_MUTATION_FORBIDDEN) => NewsEditorLoad::Forbidden,
        Some(ADMIN_NEWS_MUTATION_UNAVAILABLE) => NewsEditorLoad::Unavailable,
        Some(ADMIN_NEWS_MUTATION_MALFORMED) => NewsEditorLoad::Malformed,
        Some(_) => NewsEditorLoad::Malformed,
        None => match ctx
            .params
            .get(ADMIN_NEWS_EDITOR_STATE_PARAM)
            .map(String::as_str)
        {
            Some(ADMIN_NEWS_EDITOR_FORM) if route == NewsRoute::Create => NewsEditorLoad::Form,
            Some(ADMIN_NEWS_EDITOR_READY) if route == NewsRoute::Edit => {
                let Some(raw) = ctx.params.get(ADMIN_NEWS_EDITOR_DATA_PARAM) else {
                    return NewsEditorLoad::Malformed;
                };
                let Some(projection) = serde_json::from_str(raw)
                    .ok()
                    .and_then(decode_admin_news_editor_projection)
                else {
                    return NewsEditorLoad::Malformed;
                };
                if ctx.params.get("id") != Some(&projection.id) {
                    return NewsEditorLoad::Malformed;
                }
                NewsEditorLoad::Ready(projection)
            }
            Some(_) => NewsEditorLoad::Malformed,
            None => NewsEditorLoad::Unavailable,
        },
    }
}

fn render_editor_route(
    ctx: &PageContext,
    route: NewsRoute,
    route_reference: Option<String>,
) -> (PageMeta, Element) {
    let load = news_editor_load(ctx, route);
    let image_url = ctx.params.get(ADMIN_NEWS_IMAGE_URL_PARAM).cloned();
    let image_uploaded = matches!(
        ctx.params
            .get(ADMIN_NEWS_IMAGE_STATE_PARAM)
            .map(String::as_str),
        Some(ADMIN_NEWS_IMAGE_COMMITTED)
    );
    (
        PageMeta::admin(route.page_title()),
        rsx! {
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("the admin news editor".to_string()),
                return_url: Some(NEWS_PATH.to_string()),
                PageLayout {
                    max_width: Some(PageMaxWidth::SevenXl),
                    if image_uploaded {
                        NewsMutationNotice { state: ADMIN_NEWS_MUTATION_COMMITTED, detail: "The backend uploaded the image. The returned URL is ready in the cover image field; save the article to persist it.".to_string() }
                    }
                    match load {
                        NewsEditorLoad::Form => rsx! { NewsEditor { route, projection: None, route_reference, image_url: None } },
                        NewsEditorLoad::Ready(projection) => rsx! { NewsEditor { route, projection: Some(projection), route_reference, image_url } },
                        NewsEditorLoad::Committed(projection) => rsx! {
                            NewsMutationNotice { state: ADMIN_NEWS_MUTATION_COMMITTED, detail: "The backend committed the article and returned it after the write.".to_string() }
                            NewsEditor { route, projection: Some(projection), route_reference, image_url }
                        },
                        NewsEditorLoad::Conflict(detail) => rsx! { NewsMutationNotice { state: ADMIN_NEWS_MUTATION_CONFLICT, detail } },
                        NewsEditorLoad::Forbidden => rsx! { NewsMutationNotice { state: ADMIN_NEWS_MUTATION_FORBIDDEN, detail: "The backend denied this content mutation. No editor data is being shown.".to_string() } },
                        NewsEditorLoad::Unavailable => rsx! { NewsMutationNotice { state: ADMIN_NEWS_MUTATION_UNAVAILABLE, detail: "The content mutation backend did not provide an authoritative editor response.".to_string() } },
                        NewsEditorLoad::Malformed => rsx! { NewsMutationNotice { state: ADMIN_NEWS_MUTATION_MALFORMED, detail: "The editor response did not match the strict content contract.".to_string() } },
                    }
                }
            }
        },
    )
}

#[component]
fn RenderNewsList(ctx: PageContext) -> Element {
    let filters = NewsFilters::from_ctx(&ctx);
    let load = news_load(&ctx, &filters);
    let total_count = match &load {
        NewsLoad::Ready(projection) => Some(projection.total),
        NewsLoad::Empty => Some(0),
        NewsLoad::Forbidden | NewsLoad::Unavailable | NewsLoad::Malformed => None,
    };
    let mutation = ctx
        .params
        .get(ADMIN_NEWS_MUTATION_STATE_PARAM)
        .and_then(|value| match value.as_str() {
            ADMIN_NEWS_MUTATION_COMMITTED => Some(ADMIN_NEWS_MUTATION_COMMITTED),
            ADMIN_NEWS_MUTATION_CONFLICT => Some(ADMIN_NEWS_MUTATION_CONFLICT),
            ADMIN_NEWS_MUTATION_FORBIDDEN => Some(ADMIN_NEWS_MUTATION_FORBIDDEN),
            ADMIN_NEWS_MUTATION_UNAVAILABLE => Some(ADMIN_NEWS_MUTATION_UNAVAILABLE),
            ADMIN_NEWS_MUTATION_MALFORMED => Some(ADMIN_NEWS_MUTATION_MALFORMED),
            _ => None,
        });

    rsx! {
        PageLayout {
            max_width: Some(PageMaxWidth::SevenXl),
            NewsListHeader {}
            if let Some(state) = mutation {
                NewsMutationNotice {
                    state,
                    detail: if state == ADMIN_NEWS_MUTATION_COMMITTED {
                        "The backend committed the article deletion and the inventory was reloaded.".to_string()
                    } else {
                        "The backend did not commit the requested article mutation.".to_string()
                    },
                }
            }
            NewsStatusNavigation { active: filters.status.clone(), total_count }
            match load {
                NewsLoad::Ready(projection) => rsx! {
                    NewsReady { projection, filters: filters.clone() }
                },
                NewsLoad::Empty => rsx! {
                    NewsEmpty { filters: filters.clone() }
                },
                NewsLoad::Forbidden => rsx! {
                    NewsProblem {
                        state: ADMIN_NEWS_FORBIDDEN,
                        title: "News access was denied".to_string(),
                        detail: "The backend did not authorize this session to read the article inventory.".to_string(),
                        retry_href: filters.href(filters.page),
                    }
                    NewsUnavailableInventory {}
                },
                NewsLoad::Unavailable => rsx! {
                    NewsProblem {
                        state: ADMIN_NEWS_UNAVAILABLE,
                        title: "News records are unavailable".to_string(),
                        detail: "The news backend could not provide an authoritative article response. No records are being shown.".to_string(),
                        retry_href: filters.href(filters.page),
                    }
                    NewsUnavailableInventory {}
                },
                NewsLoad::Malformed => rsx! {
                    NewsProblem {
                        state: ADMIN_NEWS_MALFORMED,
                        title: "News data could not be verified".to_string(),
                        detail: "The news backend response did not match the read-only article contract. No records are being shown.".to_string(),
                        retry_href: filters.href(filters.page),
                    }
                    NewsUnavailableInventory {}
                },
            }
        }
    }
}

#[component]
fn NewsListHeader() -> Element {
    rsx! {
        header { class: "flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between",
            div { class: "flex items-center gap-3",
                div { class: "h-[3px] w-8 rounded-full bg-[#1fc7d4]" }
                Icon { name: "newspaper".to_string(), size: Some(20), class_name: Some("text-[#1fc7d4]".to_string()) }
                h1 { class: "text-xl font-bold text-foreground", "News Management" }
            }
            a {
                class: "flex w-full items-center justify-center gap-2 rounded-xl bg-gradient-to-r from-[#7645d9] to-[#5a33b8] px-4 py-2 text-sm font-semibold text-white transition-opacity hover:opacity-90 sm:w-auto",
                href: "/news/create",
                Icon { name: "plus".to_string(), size: Some(16) }
                "Create Article"
            }
        }
    }
}

#[component]
fn NewsStatusNavigation(active: String, total_count: Option<i64>) -> Element {
    rsx! {
        div { class: "flex flex-wrap items-center gap-2",
            nav { class: "flex flex-wrap gap-2", aria_label: "Filter news by publication status",
                for (status, label) in [("all", "All"), ("draft", "Draft"), ("published", "Published")] {
                    a {
                        class: if active == status { "rounded-lg bg-[#7645d9] px-3 py-1.5 text-sm font-medium capitalize text-white shadow-lg shadow-[#7645d9]/20" } else { "rounded-lg border border-border/20 bg-card px-3 py-1.5 text-sm font-medium capitalize text-muted-foreground transition-colors hover:border-border/40 hover:text-foreground" },
                        href: news_href(status, 1),
                        aria_current: if active == status { Some("page") } else { None },
                        "{label}"
                    }
                }
            }
            span { class: "ml-auto whitespace-nowrap text-sm text-muted-foreground",
                if let Some(total) = total_count {
                    if total == 1 { "1 article" } else { "{total} articles" }
                } else {
                    "Articles unavailable"
                }
            }
        }
    }
}

#[component]
fn NewsReady(projection: AdminNewsList, filters: NewsFilters) -> Element {
    let total_pages = (projection.total / projection.limit
        + i64::from(projection.total % projection.limit != 0))
    .max(1);
    let has_previous = projection.page > 1;
    let has_next = projection.page < total_pages;

    rsx! {
        section {
            class: "admin-news-list space-y-3",
            aria_label: "Backend-authoritative news articles",
            "data-admin-news-state": ADMIN_NEWS_READY,
            if projection.articles.is_empty() {
                div { class: "rounded-2xl border border-border/20 bg-card p-10 text-center shadow-xl", role: "status",
                    h3 { class: "font-semibold text-foreground", "No articles on this page" }
                    p { class: "mt-2 text-sm text-muted-foreground", "The filtered inventory still contains records. Return to the first page or use Previous." }
                    a { class: "btn btn-sm btn-outline mt-5", href: filters.href(1), "Return to first page" }
                }
            } else {
                div { class: "space-y-3",
                    for article in projection.articles {
                        NewsArticleCard { article }
                    }
                }
            }
            if total_pages > 1 {
                nav { class: "flex items-center justify-center gap-2 pt-3", aria_label: "News pagination",
                    if has_previous {
                        a { class: "rounded-lg border border-border/20 px-3 py-1.5 text-sm transition-colors hover:bg-muted/50", href: filters.href(projection.page - 1), rel: "prev", "Previous" }
                    } else {
                        span { class: "pointer-events-none rounded-lg border border-border/20 px-3 py-1.5 text-sm opacity-40", aria_disabled: "true", "Previous" }
                    }
                    span { class: "text-sm text-muted-foreground", "{projection.page} / {total_pages}" }
                    if has_next {
                        a { class: "rounded-lg border border-border/20 px-3 py-1.5 text-sm transition-colors hover:bg-muted/50", href: filters.href(projection.page + 1), rel: "next", "Next" }
                    } else {
                        span { class: "pointer-events-none rounded-lg border border-border/20 px-3 py-1.5 text-sm opacity-40", aria_disabled: "true", "Next" }
                    }
                }
            }
        }
    }
}

#[component]
fn NewsArticleCard(article: AdminNewsArticleSummary) -> Element {
    let status_class = if article.status == "published" {
        "border-green-500/20 bg-green-500/10 text-green-400"
    } else {
        "border-amber-500/20 bg-amber-500/10 text-amber-400"
    };
    let border_class = if article.is_pinned {
        "border-[#1fc7d4]/30 hover:border-[#1fc7d4]/50"
    } else {
        "border-border/20 hover:border-border/40"
    };
    let published_date = article
        .published_at
        .as_deref()
        .and_then(|value| value.get(..10))
        .unwrap_or("—")
        .to_string();
    let hidden_tag_count = article.tags.len().saturating_sub(4);

    rsx! {
        article { class: "flex items-start gap-4 rounded-2xl border bg-card p-4 transition-colors {border_class}",
            div { class: "flex h-14 w-20 shrink-0 items-center justify-center overflow-hidden rounded-xl border border-border/20 bg-gradient-to-br from-[#7645d9]/10 via-[#1fc7d4]/5 to-transparent",
                Icon { name: "file-text".to_string(), size: Some(20), class_name: Some("text-muted-foreground/40".to_string()) }
            }
            div { class: "min-w-0 flex-1",
                div { class: "flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between sm:gap-3",
                    div { class: "min-w-0 flex-1",
                        div { class: "flex items-center gap-2",
                            h3 { class: "truncate font-semibold text-foreground", "{article.title}" }
                            if article.is_pinned {
                                Icon { name: "pin".to_string(), size: Some(14), class_name: Some("shrink-0 text-[#1fc7d4]".to_string()) }
                                span { class: "sr-only", "Pinned" }
                            }
                        }
                        p { class: "truncate font-mono text-xs text-muted-foreground/60", "{article.slug}" }
                    }
                    div { class: "flex shrink-0 items-center gap-2",
                        span { class: "inline-flex items-center rounded-full border px-2 py-0.5 text-xs font-medium {status_class}", "{article.status}" }
                        time { class: "hidden whitespace-nowrap text-xs text-muted-foreground sm:block", datetime: article.published_at.clone(), "{published_date}" }
                        NewsSummaryActions { article: article.clone() }
                    }
                }
                if let Some(summary) = &article.summary {
                    p { class: "mt-1 line-clamp-1 text-sm text-muted-foreground", "{summary}" }
                }
                if !article.tags.is_empty() {
                    ul { class: "mt-2 flex flex-wrap gap-1", aria_label: "Article tags",
                        for tag in article.tags.iter().take(4) {
                            li { class: "rounded-full bg-[#7645d9]/10 px-2 py-0.5 text-xs font-medium text-[#7645d9]/70", "{tag}" }
                        }
                        if hidden_tag_count > 0 {
                            li { class: "text-xs text-muted-foreground/50", "+{hidden_tag_count}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn NewsSummaryActions(article: AdminNewsArticleSummary) -> Element {
    rsx! {
        div { class: "flex items-center gap-1",
            button {
                class: "cursor-not-allowed rounded-lg p-1.5 text-muted-foreground opacity-50",
                r#type: "button",
                disabled: true,
                aria_label: if article.is_pinned { "Unpin unavailable from article summary" } else { "Pin unavailable from article summary" },
                title: "Open the editor to change pin state with the complete versioned article contract",
                Icon { name: if article.is_pinned { "pin-off".to_string() } else { "pin".to_string() }, size: Some(16) }
            }
            button {
                class: "cursor-not-allowed rounded-lg p-1.5 text-muted-foreground opacity-50",
                r#type: "button",
                disabled: true,
                aria_label: if article.status == "published" { "Unpublish unavailable from article summary" } else { "Publish unavailable from article summary" },
                title: "Open the editor to change publication state with the complete versioned article contract",
                Icon { name: if article.status == "published" { "eye-off".to_string() } else { "eye".to_string() }, size: Some(16) }
            }
            a {
                class: "rounded-lg p-1.5 text-muted-foreground transition-colors hover:bg-muted/50 hover:text-foreground",
                href: format!("/news/{}/edit", article.id),
                aria_label: "Edit article",
                title: "Edit",
                Icon { name: "edit".to_string(), size: Some(16) }
            }
            details { class: "relative",
                summary {
                    class: "cursor-pointer list-none rounded-lg p-1.5 text-muted-foreground transition-colors hover:bg-red-500/10 hover:text-red-400",
                    aria_label: "Delete article",
                    title: "Delete",
                    Icon { name: "trash-2".to_string(), size: Some(16) }
                }
                form { method: "post", action: NEWS_PATH, class: "absolute right-0 z-20 mt-2 w-64 space-y-3 rounded-xl border border-red-500/20 bg-card p-4 shadow-2xl",
                    p { class: "text-sm font-semibold text-foreground", "Delete article?" }
                    p { class: "text-xs leading-5 text-muted-foreground", "This permanently deletes the backend record." }
                    input { r#type: "hidden", name: "id", value: article.id }
                    input { r#type: "hidden", name: "if_match", value: article.updated_at }
                    input { r#type: "hidden", name: "idempotency_key", value: format!("admin.news.delete.{}", Uuid::new_v4()) }
                    button { r#type: "submit", class: "btn btn-sm btn-outline w-full border-red-500/30 text-red-400", "data-admin-news-delete": "bff", "Delete article" }
                }
            }
        }
    }
}

#[component]
fn NewsEmpty(filters: NewsFilters) -> Element {
    rsx! {
        section { class: "flex flex-col items-center justify-center gap-4 rounded-2xl border border-border/20 bg-card py-20 text-center shadow-xl", role: "status", "data-admin-news-state": ADMIN_NEWS_EMPTY,
            div { class: "rounded-full border border-border/20 bg-gradient-to-br from-[#7645d9]/10 via-[#1fc7d4]/5 to-transparent p-5",
                Icon { name: "newspaper".to_string(), size: Some(32), class_name: Some("text-muted-foreground/40".to_string()) }
            }
            div {
                h2 { class: "font-semibold text-foreground", "No articles yet" }
                p { class: "mt-1 text-sm text-muted-foreground", "Create your first article to get started." }
            }
            a { class: "flex items-center gap-2 rounded-xl bg-gradient-to-r from-[#7645d9] to-[#5a33b8] px-4 py-2 text-sm font-semibold text-white transition-opacity hover:opacity-90", href: "/news/create",
                Icon { name: "plus".to_string(), size: Some(16) }
                "Create Article"
            }
            a { class: "text-xs text-muted-foreground hover:text-foreground", href: filters.href(1), "Refresh articles" }
        }
    }
}

#[component]
fn NewsUnavailableInventory() -> Element {
    rsx! {
        section { class: "flex flex-col items-center justify-center gap-4 rounded-2xl border border-border/20 bg-card py-20 text-center shadow-xl", aria_hidden: "true",
            div { class: "rounded-full border border-border/20 bg-gradient-to-br from-[#7645d9]/10 via-[#1fc7d4]/5 to-transparent p-5",
                Icon { name: "newspaper".to_string(), size: Some(32), class_name: Some("text-muted-foreground/40".to_string()) }
            }
            div {
                h2 { class: "font-semibold text-foreground", "No verified articles" }
                p { class: "mt-1 text-sm text-muted-foreground", "The article list will appear here after an authoritative response." }
            }
            a { class: "flex items-center gap-2 rounded-xl bg-gradient-to-r from-[#7645d9] to-[#5a33b8] px-4 py-2 text-sm font-semibold text-white opacity-60", href: "/news/create", tabindex: "-1",
                Icon { name: "plus".to_string(), size: Some(16) }
                "Create Article"
            }
        }
    }
}

#[component]
fn NewsProblem(state: &'static str, title: String, detail: String, retry_href: String) -> Element {
    rsx! {
        section { class: "rounded-2xl border border-amber-500/30 bg-amber-500/5 p-5 sm:p-6", role: "alert", "data-admin-news-state": state,
            div { class: "flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between",
                div { class: "flex min-w-0 items-start gap-3",
                    Icon { name: "shield".to_string(), size: Some(20), class_name: Some("mt-0.5 shrink-0 text-amber-400".to_string()) }
                    div {
                        h2 { class: "font-semibold text-foreground", "{title}" }
                        p { class: "mt-1 max-w-3xl text-sm leading-6 text-muted-foreground", "{detail}" }
                    }
                }
                a { class: "btn btn-sm btn-outline shrink-0", href: retry_href, "Try again" }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum NewsRoute {
    Create,
    Edit,
}

impl NewsRoute {
    fn marker(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Edit => "edit",
        }
    }

    fn page_title(self) -> &'static str {
        match self {
            Self::Create => "New news post",
            Self::Edit => "Edit news",
        }
    }
}

#[component]
fn NewsEditor(
    route: NewsRoute,
    projection: Option<AdminNewsEditorProjection>,
    route_reference: Option<String>,
    image_url: Option<String>,
) -> Element {
    let is_create = route == NewsRoute::Create;
    let action = if is_create {
        "/news/create".to_string()
    } else {
        format!(
            "/news/{}/edit",
            projection
                .as_ref()
                .map(|item| item.id.as_str())
                .unwrap_or("invalid")
        )
    };
    let title = projection
        .as_ref()
        .map(|item| item.title.clone())
        .unwrap_or_default();
    let slug = projection
        .as_ref()
        .map(|item| item.slug.clone())
        .unwrap_or_default();
    let summary = projection
        .as_ref()
        .and_then(|item| item.summary.clone())
        .unwrap_or_default();
    let content = projection
        .as_ref()
        .map(|item| item.content.clone())
        .unwrap_or_default();
    let status = projection
        .as_ref()
        .map(|item| item.status.clone())
        .unwrap_or_else(|| "draft".to_string());
    let tags = projection
        .as_ref()
        .map(|item| item.tags.join(", "))
        .unwrap_or_default();
    let version = projection.as_ref().map(|item| item.updated_at.clone());
    let idempotency_key = Uuid::new_v4().to_string();
    let cover_image_url = image_url
        .or_else(|| {
            projection
                .as_ref()
                .and_then(|item| item.cover_image_url.clone())
        })
        .unwrap_or_default();
    let save_label = if is_create { "Create" } else { "Update" };

    rsx! {
        section {
            class: "space-y-6",
            role: "region",
            "data-admin-news-editor-state": if is_create { ADMIN_NEWS_EDITOR_FORM } else { ADMIN_NEWS_EDITOR_READY },
            "data-admin-news-route": route.marker(),
            if let Some(reference) = route_reference {
                p { class: "sr-only", "Verified article: {reference}" }
            }
            form { method: "post", action: action.clone(), class: "space-y-6",
                NewsEditorToolbar { status: status.clone(), save_label }
                div { class: "space-y-6 rounded-2xl border border-border/20 bg-card p-4 shadow-xl sm:p-8",
                    NewsCoverField { cover_image_url, is_create }
                    div { class: "h-px bg-border/20" }
                    label { class: "block",
                        span { class: "sr-only", "Title" }
                        input { class: "w-full border-b border-border/20 bg-transparent pb-2 text-2xl font-bold text-foreground placeholder:text-muted-foreground/40 focus:outline-none sm:text-3xl", name: "title", value: title.clone(), maxlength: MAX_TITLE_CHARS, required: true, placeholder: "Article title..." }
                    }
                    if !is_create {
                        label { class: "flex items-center gap-1 font-mono text-sm text-muted-foreground",
                            span { class: "opacity-50", "epsx.io/news/" }
                            span { class: "sr-only", "Slug" }
                            input { class: "min-w-0 flex-1 border-b border-dashed border-[#1fc7d4]/40 bg-transparent text-[#1fc7d4] focus:outline-none", name: "slug", value: slug.clone(), maxlength: MAX_SLUG_CHARS, required: true }
                        }
                    } else {
                        p { class: "font-mono text-sm text-muted-foreground",
                            span { class: "opacity-50", "epsx.io/news/" }
                            span { class: "text-[#1fc7d4]/70", "generated-from-title" }
                        }
                    }
                    div { class: "h-px bg-border/20" }
                    label { class: "block",
                        span { class: "sr-only", "Summary" }
                        textarea { class: "w-full resize-none border-b border-border/10 bg-transparent pb-2 text-base text-muted-foreground placeholder:text-muted-foreground/30 focus:outline-none", name: "summary", rows: 2, maxlength: MAX_SUMMARY_CHARS, placeholder: "Short description…", "{summary}" }
                    }
                    label { class: "block text-sm font-medium text-foreground", "Tags",
                        input { class: "input input-bordered mt-2 w-full", name: "tags", value: tags, maxlength: 2048, placeholder: "announcement, platform" }
                    }
                    div { class: "h-px bg-border/20" }
                    NewsMarkdownField { content: content.clone() }
                    if let Some(version) = version.clone() {
                        input { r#type: "hidden", name: "if_match", value: version, "data-admin-news-version": "updated_at" }
                    }
                    input { r#type: "hidden", name: "idempotency_key", value: idempotency_key }
                    div { class: "flex flex-wrap items-center justify-between gap-3 border-t border-border/20 pt-5",
                        p { class: "text-xs text-muted-foreground", "Saved through the versioned content BFF contract." }
                        div { class: "flex gap-3",
                            a { class: "btn btn-outline", href: NEWS_PATH, "Cancel" }
                            button { r#type: "submit", class: "btn btn-primary", "data-admin-news-submit": "bff", "{save_label}" }
                        }
                    }
                }
            }
            if !is_create {
                if let Some(version) = version {
                    div { class: "mt-5 flex flex-wrap gap-3 border-t border-border/30 pt-5",
                        for operation in ["publish", "unpublish", "pin", "unpin"] {
                            form { method: "post", action: action.clone(), class: "inline-flex",
                                input { r#type: "hidden", name: "operation", value: operation }
                                input { r#type: "hidden", name: "title", value: title.clone() }
                                input { r#type: "hidden", name: "slug", value: slug.clone() }
                                input { r#type: "hidden", name: "content", value: content.clone() }
                                input { r#type: "hidden", name: "if_match", value: version.clone() }
                                input { r#type: "hidden", name: "idempotency_key", value: format!("admin.news.{operation}.{}", Uuid::new_v4()) }
                                button { r#type: "submit", class: "btn btn-sm btn-outline", "data-admin-news-transition": operation, "{operation}" }
                            }
                        }
                    }
                }
                form { method: "post", action: "/news/upload-image", enctype: "multipart/form-data", class: "mt-5 space-y-3 border-t border-border/30 pt-5",
                    input { r#type: "hidden", name: "article_id", value: projection.as_ref().map(|item| item.id.clone()).unwrap_or_default() }
                    input { r#type: "hidden", name: "idempotency_key", value: format!("admin.news.image.{}", Uuid::new_v4()) }
                    label { class: "block text-sm font-medium text-foreground", "Upload cover image",
                        input { class: "file-input file-input-bordered mt-2 w-full", r#type: "file", name: "file", required: true, accept: "image/*" }
                    }
                    button { r#type: "submit", class: "btn btn-sm btn-outline", "Upload image" }
                }
            }
        }
    }
}

#[component]
fn NewsEditorToolbar(status: String, save_label: &'static str) -> Element {
    rsx! {
        header { class: "sticky top-0 z-10 flex flex-col gap-3 border-b border-border/10 bg-background/80 py-3 backdrop-blur-md sm:flex-row sm:items-center sm:justify-between",
            a { class: "flex items-center gap-2 text-sm text-muted-foreground transition-colors hover:text-foreground", href: NEWS_PATH,
                Icon { name: "arrow-left".to_string(), size: Some(16) }
                "Back to News"
            }
            div { class: "flex items-center gap-3",
                label { class: "sr-only", r#for: "news-editor-status", "Article status" }
                select { id: "news-editor-status", class: "rounded-lg border border-border/20 bg-card px-3 py-1.5 text-xs font-medium text-foreground", name: "status",
                    option { value: "draft", selected: status == "draft", "Draft" }
                    option { value: "published", selected: status == "published", "Published" }
                }
                button { r#type: "submit", class: "flex items-center gap-2 rounded-xl bg-gradient-to-r from-[#7645d9] to-[#5a33b8] px-4 py-2 text-sm font-semibold text-white transition-opacity hover:opacity-90", "data-admin-news-submit": "bff",
                    Icon { name: "save".to_string(), size: Some(16) }
                    "{save_label}"
                }
            }
        }
    }
}

#[component]
fn NewsCoverField(cover_image_url: String, is_create: bool) -> Element {
    rsx! {
        div { class: "grid gap-3 sm:grid-cols-3 sm:items-end",
            label { class: "block text-sm font-medium text-foreground sm:col-span-2", "Cover image URL",
                input { class: "input input-bordered mt-2 w-full", name: "cover_image_url", maxlength: 2048, value: cover_image_url, placeholder: "https://…" }
            }
            button {
                class: "btn btn-sm btn-outline cursor-not-allowed opacity-50",
                r#type: "button",
                disabled: true,
                title: if is_create { "Create the article before uploading a cover image" } else { "Use the versioned upload control below" },
                Icon { name: "upload".to_string(), size: Some(14) }
                "Upload"
            }
        }
    }
}

#[component]
fn NewsMarkdownField(content: String) -> Element {
    rsx! {
        div { class: "overflow-hidden rounded-xl border border-border/30",
            div { class: "flex flex-wrap items-center gap-1 border-b border-border/30 bg-muted/20 px-3 py-2",
                for label in ["B", "I", "H1", "H2", "Code", "Quote"] {
                    button {
                        class: "cursor-not-allowed rounded-lg px-2 py-1.5 text-xs font-medium text-muted-foreground opacity-50",
                        r#type: "button",
                        disabled: true,
                        title: "Formatting shortcuts require the hydrated rich-text editor",
                        "{label}"
                    }
                }
                div { class: "mx-1 h-4 w-px bg-border/40" }
                span { class: "rounded-lg bg-[#7645d9]/20 px-3 py-1.5 text-xs font-medium text-[#7645d9]", "Markdown" }
            }
            label { class: "block",
                span { class: "sr-only", "Content" }
                textarea { class: "w-full resize-none bg-background px-4 py-3 font-mono text-sm text-foreground focus:outline-none", style: "min-height: 500px;", name: "content", maxlength: MAX_CONTENT_BYTES, required: true, placeholder: "Write markdown content…", "{content}" }
            }
        }
    }
}

#[component]
fn NewsMutationNotice(state: &'static str, detail: String) -> Element {
    rsx! {
        section {
            class: "mb-5 rounded-2xl border border-amber-500/30 bg-amber-500/5 p-6",
            role: if state == ADMIN_NEWS_MUTATION_FORBIDDEN { "alert" } else { "status" },
            "data-admin-news-mutation-state": state,
            h2 { class: "text-lg font-semibold text-foreground", "Content mutation: {state}" }
            p { class: "mt-2 text-sm leading-6 text-muted-foreground", "{detail}" }
            if matches!(state, ADMIN_NEWS_MUTATION_CONFLICT | ADMIN_NEWS_MUTATION_UNAVAILABLE | ADMIN_NEWS_MUTATION_MALFORMED) {
                a { class: "btn btn-sm btn-outline mt-4", href: NEWS_PATH, "Return to news inventory" }
            }
        }
    }
}

fn bounded_route_reference(raw: &str) -> Option<String> {
    let cleaned = raw
        .chars()
        .filter(|character| !character.is_control())
        .collect::<String>();
    let cleaned = cleaned.trim();
    if cleaned.is_empty() {
        return None;
    }
    if cleaned.chars().count() <= MAX_ROUTE_REFERENCE_CHARS {
        return Some(cleaned.to_string());
    }

    let mut bounded = cleaned
        .chars()
        .take(MAX_ROUTE_REFERENCE_CHARS.saturating_sub(1))
        .collect::<String>();
    bounded.push('…');
    Some(bounded)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::auth::user::{AuthMethod, User};

    fn session() -> User {
        User {
            id: "news-session".to_string(),
            address: "0x1234".to_string(),
            chain_id: "56".to_string(),
            roles: vec![],
            email: None,
            tier: None,
            permissions: vec![],
            last_login_at: None,
            auth_method: AuthMethod::Wallet,
            display_name: None,
        }
    }

    fn article() -> AdminNewsArticleSummary {
        AdminNewsArticleSummary {
            id: "01234567-89ab-4cde-8fab-0123456789ab".to_string(),
            title: "Production migration update".to_string(),
            slug: "production-migration-update".to_string(),
            summary: Some("A verified read-only summary.".to_string()),
            status: "published".to_string(),
            tags: vec!["migration".to_string(), "rust".to_string()],
            published_at: Some("2026-07-22T10:00:00Z".to_string()),
            created_at: "2026-07-22T09:00:00Z".to_string(),
            updated_at: "2026-07-22T10:00:00Z".to_string(),
            is_pinned: true,
        }
    }

    fn editor() -> AdminNewsEditorProjection {
        AdminNewsEditorProjection {
            id: article().id,
            title: "Production migration update".to_string(),
            slug: "production-migration-update".to_string(),
            summary: Some("A verified editor projection.".to_string()),
            content: "Authoritative article body".to_string(),
            cover_image_url: None,
            tags: vec!["migration".to_string()],
            status: "draft".to_string(),
            published_at: None,
            created_at: "2026-07-22T09:00:00Z".to_string(),
            updated_at: "2026-07-22T10:00:00Z".to_string(),
            is_pinned: false,
        }
    }

    fn ctx(state: &str, projection: Option<AdminNewsList>) -> PageContext {
        let mut params = HashMap::from([
            (ADMIN_NEWS_STATE_PARAM.to_string(), state.to_string()),
            (ADMIN_NEWS_PAGE_PARAM.to_string(), "1".to_string()),
            (ADMIN_NEWS_STATUS_PARAM.to_string(), "all".to_string()),
        ]);
        if let Some(projection) = projection {
            params.insert(
                ADMIN_NEWS_DATA_PARAM.to_string(),
                serde_json::to_string(&projection).unwrap(),
            );
        }
        PageContext {
            user: Some(session()),
            path: NEWS_PATH.to_string(),
            params,
            ..Default::default()
        }
    }

    fn projection(articles: Vec<AdminNewsArticleSummary>, total: i64, page: i64) -> AdminNewsList {
        AdminNewsList {
            articles,
            total,
            page,
            limit: NEWS_PAGE_LIMIT,
        }
    }

    fn html(ctx: &PageContext) -> String {
        dioxus_ssr::render_element(render(ctx).1)
    }

    #[test]
    fn signed_out_routes_keep_projection_and_edit_reference_private() {
        let mut list = PageContext {
            path: NEWS_PATH.to_string(),
            ..Default::default()
        };
        list.params.insert(
            ADMIN_NEWS_DATA_PARAM.to_string(),
            "PRIVATE_ARTICLE_PAYLOAD".to_string(),
        );
        let mut edit = PageContext {
            path: "/news/private-reference/edit".to_string(),
            ..Default::default()
        };
        edit.params
            .insert("id".to_string(), "private-reference".to_string());

        for rendered in [
            html(&list),
            dioxus_ssr::render_element(render_edit(&edit).1),
        ] {
            assert!(rendered.contains("Sign in required"));
            assert!(!rendered.contains("data-admin-news-state"));
            assert!(!rendered.contains("PRIVATE_ARTICLE_PAYLOAD"));
            assert!(!rendered.contains("private-reference"));
        }
    }

    #[test]
    fn ready_projection_renders_escaped_cards_and_lifecycle_actions() {
        let mut hostile = article();
        hostile.title = "<script>alert(1)</script>".to_string();
        hostile.summary = Some("<b>trusted?</b>".to_string());
        hostile.tags = vec!["<img>".to_string()];
        let rendered = html(&ctx(
            ADMIN_NEWS_READY,
            Some(projection(vec![hostile], 1, 1)),
        ));

        assert!(rendered.contains("data-admin-news-state=\"ready\""));
        assert!(rendered.contains("&#60;script&#62;alert(1)&#60;/script&#62;"));
        assert!(rendered.contains("&#60;b&#62;trusted?&#60;/b&#62;"));
        assert!(!rendered.contains("<script>alert(1)</script>"));
        assert!(rendered.contains("Pinned"));
        assert!(rendered.contains("1 article"));
        assert!(rendered.contains("data-admin-news-delete=\"bff\""));
        assert!(rendered.contains("Edit article"));
        assert!(rendered.contains("Unpin unavailable from article summary"));
        assert!(rendered.contains("Unpublish unavailable from article summary"));
        assert!(rendered.contains("disabled"));
    }

    #[test]
    fn empty_forbidden_unavailable_and_malformed_are_distinct() {
        let empty = html(&ctx(ADMIN_NEWS_EMPTY, Some(projection(vec![], 0, 1))));
        let forbidden = html(&ctx(ADMIN_NEWS_FORBIDDEN, None));
        let unavailable = html(&ctx(ADMIN_NEWS_UNAVAILABLE, None));
        let malformed = html(&ctx(ADMIN_NEWS_MALFORMED, None));

        assert!(empty.contains("data-admin-news-state=\"empty\""));
        assert!(empty.contains("No articles yet"));
        assert!(empty.contains("Create Article"));
        assert!(forbidden.contains("data-admin-news-state=\"forbidden\""));
        assert!(forbidden.contains("News access was denied"));
        assert!(forbidden.contains("No verified articles"));
        assert!(unavailable.contains("data-admin-news-state=\"unavailable\""));
        assert!(unavailable.contains("News records are unavailable"));
        assert!(unavailable.contains("Articles unavailable"));
        assert!(malformed.contains("data-admin-news-state=\"malformed\""));
        assert!(malformed.contains("News data could not be verified"));
    }

    #[test]
    fn decoder_rejects_unknown_fields_bounds_controls_status_dates_and_counts() {
        let valid = serde_json::to_value(projection(vec![article()], 1, 1)).unwrap();
        assert!(decode_admin_news_projection(valid.clone()).is_some());

        let mut cases = vec![];
        let mut unknown = valid.clone();
        unknown["unexpected"] = serde_json::json!(true);
        cases.push(unknown);
        let mut invalid_id = valid.clone();
        invalid_id["articles"][0]["id"] = serde_json::json!("article-1");
        cases.push(invalid_id);
        let mut overlong = valid.clone();
        overlong["articles"][0]["title"] = serde_json::json!("x".repeat(MAX_TITLE_CHARS + 1));
        cases.push(overlong);
        let mut control = valid.clone();
        control["articles"][0]["summary"] = serde_json::json!("line one\nline two");
        cases.push(control);
        let mut status = valid.clone();
        status["articles"][0]["status"] = serde_json::json!("archived");
        cases.push(status);
        let mut timestamp = valid.clone();
        timestamp["articles"][0]["created_at"] = serde_json::json!("yesterday");
        cases.push(timestamp);
        let mut limit = valid.clone();
        limit["limit"] = serde_json::json!(100);
        cases.push(limit);
        let mut total = valid;
        total["total"] = serde_json::json!(0);
        cases.push(total);
        cases.push(serde_json::to_value(projection(vec![article()], 1, 2)).unwrap());

        for malformed in cases {
            assert!(decode_admin_news_projection(malformed).is_none());
        }
    }

    #[test]
    fn inconsistent_state_page_or_payload_is_malformed() {
        let empty_as_ready = html(&ctx(ADMIN_NEWS_READY, Some(projection(vec![], 0, 1))));
        let records_as_empty = html(&ctx(
            ADMIN_NEWS_EMPTY,
            Some(projection(vec![article()], 1, 1)),
        ));
        let mut wrong_page = ctx(ADMIN_NEWS_READY, Some(projection(vec![article()], 21, 2)));
        wrong_page
            .params
            .insert(ADMIN_NEWS_PAGE_PARAM.to_string(), "1".to_string());

        for rendered in [empty_as_ready, records_as_empty, html(&wrong_page)] {
            assert!(rendered.contains("News data could not be verified"));
        }
    }

    #[test]
    fn pagination_and_retry_preserve_normalized_status_filter() {
        let mut page_two = ctx(ADMIN_NEWS_READY, Some(projection(vec![article()], 41, 2)));
        page_two
            .params
            .insert(ADMIN_NEWS_PAGE_PARAM.to_string(), "2".to_string());
        page_two
            .params
            .insert(ADMIN_NEWS_STATUS_PARAM.to_string(), "published".to_string());
        let rendered = html(&page_two);

        assert!(rendered.contains("href=\"/news?status=published&#38;page=1\""));
        assert!(rendered.contains("href=\"/news?status=published&#38;page=3\""));
        assert!(rendered.contains("aria-label=\"Filter news by publication status\""));
        assert!(rendered.contains("aria-current=\"page\""));

        let mut unavailable = ctx(ADMIN_NEWS_UNAVAILABLE, None);
        unavailable
            .params
            .insert(ADMIN_NEWS_STATUS_PARAM.to_string(), "draft".to_string());
        assert!(html(&unavailable).contains("href=\"/news?status=draft&#38;page=1\""));
    }

    #[test]
    fn nonzero_total_out_of_range_empty_page_is_ready_with_recovery() {
        let mut page_four = ctx(ADMIN_NEWS_READY, Some(projection(vec![], 41, 4)));
        page_four
            .params
            .insert(ADMIN_NEWS_PAGE_PARAM.to_string(), "4".to_string());
        page_four
            .params
            .insert(ADMIN_NEWS_STATUS_PARAM.to_string(), "draft".to_string());
        let rendered = html(&page_four);

        assert!(rendered.contains("data-admin-news-state=\"ready\""));
        assert!(rendered.contains("41 articles"));
        assert!(rendered.contains("No articles on this page"));
        assert!(rendered.contains("Return to first page"));
        assert!(rendered.contains("status=draft&#38;page=1"));
        assert!(!rendered.contains("No articles yet"));
    }

    #[test]
    fn editor_routes_render_only_backend_verified_mutation_states() {
        let list = html(&ctx(
            ADMIN_NEWS_READY,
            Some(projection(vec![article()], 1, 1)),
        ));
        let mut create_ctx = ctx(ADMIN_NEWS_UNAVAILABLE, None);
        create_ctx.params.insert(
            ADMIN_NEWS_EDITOR_STATE_PARAM.to_string(),
            ADMIN_NEWS_EDITOR_FORM.to_string(),
        );
        let create = dioxus_ssr::render_element(render_create(&create_ctx).1);

        let editor = editor();
        let mut edit_ctx = PageContext {
            user: Some(session()),
            path: format!("/news/{}/edit", editor.id),
            ..Default::default()
        };
        edit_ctx.params.insert("id".to_string(), editor.id.clone());
        edit_ctx.params.insert(
            ADMIN_NEWS_EDITOR_STATE_PARAM.to_string(),
            ADMIN_NEWS_EDITOR_READY.to_string(),
        );
        edit_ctx.params.insert(
            ADMIN_NEWS_EDITOR_DATA_PARAM.to_string(),
            serde_json::to_string(&editor).unwrap(),
        );
        let edit = dioxus_ssr::render_element(render_edit(&edit_ctx).1);

        assert!(create.contains("data-admin-news-route=\"create\""));
        assert!(create.contains("data-admin-news-editor-state=\"form\""));
        assert!(create.contains("data-admin-news-submit=\"bff\""));
        assert!(edit.contains("data-admin-news-route=\"edit\""));
        assert!(edit.contains("data-admin-news-editor-state=\"ready\""));
        assert!(edit.contains("data-admin-news-version=\"updated_at\""));
        assert!(list.contains("data-admin-news-state=\"ready\""));
    }

    #[test]
    fn hostile_edit_reference_is_bounded_control_free_and_escaped() {
        let hostile = format!("<script>alert(1)</script>\n\u{0}{}", "x".repeat(100));
        let bounded = bounded_route_reference(&hostile).unwrap();
        assert!(bounded.chars().count() <= MAX_ROUTE_REFERENCE_CHARS);
        assert!(!bounded.chars().any(char::is_control));

        let mut edit_ctx = ctx(ADMIN_NEWS_UNAVAILABLE, None);
        edit_ctx.params.insert("id".to_string(), hostile);
        let rendered = dioxus_ssr::render_element(render_edit(&edit_ctx).1);
        assert!(!rendered.contains("<script>alert(1)</script>"));
        assert!(!rendered.contains("script"));
        assert!(rendered.contains("data-admin-news-mutation-state=\"unavailable\""));
    }
}

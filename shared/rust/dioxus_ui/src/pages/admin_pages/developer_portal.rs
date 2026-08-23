//! /developer-portal — authenticated developer inventory and lifecycle actions.
//!
//! The page accepts only a strict redacted projection from PageContext. Secrets
//! never enter the inventory; lifecycle mutations are bounded native forms
//! forwarded to backend-owned authorization and audit handlers.

use chrono::DateTime;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::AuthGate;
use crate::components::admin::data_state_banner::{AdminDataState, AdminDataStateBanner};
use crate::components::admin::page_layout::{PageGradient, PageHeader, PageLayout, PageMaxWidth};
use crate::primitives::Icon;

use super::super::{PageContext, PageMeta};

const DEVELOPER_PORTAL_PATH: &str = "/developer-portal";
const DEVELOPER_CREATE_PATH: &str = "/developer-portal/api-keys/create";
const MAX_API_KEYS: usize = 100;
const MAX_MODULES: usize = 100;
const MAX_CLIENT_NAME_CHARS: usize = 255;
const MAX_KEY_PREFIX_CHARS: usize = 16;
const MAX_MODULE_NAME_CHARS: usize = 255;
const MAX_TIMESTAMP_CHARS: usize = 64;

pub const ADMIN_DEVELOPER_DATA_PARAM: &str = "data_admin_developer_portal";
pub const ADMIN_DEVELOPER_STATE_PARAM: &str = "data_admin_developer_portal_state";

pub const ADMIN_DEVELOPER_READY: &str = "ready";
pub const ADMIN_DEVELOPER_EMPTY: &str = "empty";
pub const ADMIN_DEVELOPER_FORBIDDEN: &str = "forbidden";
pub const ADMIN_DEVELOPER_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_DEVELOPER_MALFORMED: &str = "malformed";
pub const ADMIN_DEVELOPER_UNAUTHENTICATED: &str = "unauthenticated";
pub const ADMIN_DEVELOPER_UNAUTHORIZED: &str = "unauthorized";
pub const ADMIN_DEVELOPER_CREATE_DATA_PARAM: &str = "data_admin_developer_create";
pub const ADMIN_DEVELOPER_CREATE_STATE_PARAM: &str = "data_admin_developer_create_state";
pub const ADMIN_DEVELOPER_CREATE_FORM: &str = "form";
pub const ADMIN_DEVELOPER_CREATE_CREATED: &str = "created";
pub const ADMIN_DEVELOPER_CREATE_CONFLICT: &str = "conflict";
pub const ADMIN_DEVELOPER_CREATE_FORBIDDEN: &str = "forbidden";
pub const ADMIN_DEVELOPER_CREATE_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_DEVELOPER_CREATE_MALFORMED: &str = "malformed";
pub const ADMIN_DEVELOPER_CREATE_UNAUTHENTICATED: &str = "unauthenticated";
pub const ADMIN_DEVELOPER_CREATE_UNAUTHORIZED: &str = "unauthorized";

/// Redacted API-key inventory row. full_key, wallet ownership, permissions,
/// module grants, contact data, and rate limits are intentionally absent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminDeveloperApiKeySummary {
    pub id: String,
    pub key_prefix: String,
    pub client_name: String,
    pub status: String,
    pub total_requests: i64,
    pub expires_at: Option<String>,
    pub last_used_at: Option<String>,
    pub created_at: String,
}

/// Persisted module usage projection. The module label is resolved from the
/// authoritative module table by the backend; it is never synthesized here.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminDeveloperModuleUsage {
    pub module_id: String,
    pub module_name: String,
    pub request_count: i64,
    pub unique_api_keys: i64,
}

/// Exact backend-owned contract placed in PageContext::params by the BFF.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminDeveloperPortalProjection {
    pub api_keys: Vec<AdminDeveloperApiKeySummary>,
    pub total_api_keys: i64,
    pub active_api_keys: i64,
    pub revoked_api_keys: i64,
    pub expired_api_keys: i64,
    pub total_modules: i64,
    pub active_modules: i64,
    pub total_requests_today: i64,
    pub total_requests_this_month: i64,
    pub top_modules_by_usage: Vec<AdminDeveloperModuleUsage>,
}

/// The only projection permitted to carry a plaintext secret. The BFF must
/// place it in PageContext for the creation response only; list/read DTOs do
/// not contain this field.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminDeveloperSecretOnceProjection {
    pub api_key: AdminDeveloperApiKeySummary,
    pub secret: String,
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

fn valid_text(value: &str, max_chars: usize, required: bool) -> bool {
    (!required || !value.trim().is_empty())
        && value.chars().count() <= max_chars
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

fn valid_timestamp(value: &str) -> bool {
    valid_text(value, MAX_TIMESTAMP_CHARS, true) && DateTime::parse_from_rfc3339(value).is_ok()
}

fn valid_optional_timestamp(value: Option<&str>) -> bool {
    value.is_none_or(valid_timestamp)
}

impl AdminDeveloperApiKeySummary {
    fn is_well_formed(&self) -> bool {
        valid_uuid(&self.id)
            && valid_text(&self.key_prefix, MAX_KEY_PREFIX_CHARS, true)
            && valid_text(&self.client_name, MAX_CLIENT_NAME_CHARS, true)
            && matches!(self.status.as_str(), "active" | "revoked" | "expired")
            && self.total_requests >= 0
            && valid_optional_timestamp(self.expires_at.as_deref())
            && valid_optional_timestamp(self.last_used_at.as_deref())
            && valid_timestamp(&self.created_at)
    }
}

impl AdminDeveloperModuleUsage {
    fn is_well_formed(&self) -> bool {
        valid_uuid(&self.module_id)
            && valid_text(&self.module_name, MAX_MODULE_NAME_CHARS, true)
            && self.request_count >= 0
            && self.unique_api_keys >= 0
            && self.unique_api_keys <= self.request_count
    }
}

/// Decode and validate the exact redacted projection before any backend value
/// reaches HTML. Unknown fields reject the payload, including full_key.
pub fn decode_admin_developer_key_summary(
    value: serde_json::Value,
) -> Option<AdminDeveloperApiKeySummary> {
    let summary: AdminDeveloperApiKeySummary = serde_json::from_value(value).ok()?;
    summary.is_well_formed().then_some(summary)
}

pub fn decode_admin_developer_secret_once(
    value: serde_json::Value,
) -> Option<AdminDeveloperSecretOnceProjection> {
    let projection: AdminDeveloperSecretOnceProjection = serde_json::from_value(value).ok()?;
    let key = decode_admin_developer_key_summary(serde_json::to_value(&projection.api_key).ok()?)?;
    if projection.secret.is_empty()
        || projection.secret.chars().count() > 512
        || projection.secret.chars().any(char::is_control)
    {
        return None;
    }
    Some(AdminDeveloperSecretOnceProjection {
        api_key: key,
        secret: projection.secret,
    })
}

pub fn decode_admin_developer_projection(
    value: serde_json::Value,
) -> Option<AdminDeveloperPortalProjection> {
    let projection: AdminDeveloperPortalProjection = serde_json::from_value(value).ok()?;
    if projection.api_keys.len() > MAX_API_KEYS
        || projection.top_modules_by_usage.len() > MAX_MODULES
        || projection.total_api_keys < 0
        || projection.total_api_keys < projection.api_keys.len() as i64
        || projection.active_api_keys < 0
        || projection.revoked_api_keys < 0
        || projection.expired_api_keys < 0
        || projection
            .active_api_keys
            .checked_add(projection.revoked_api_keys)
            .and_then(|value| value.checked_add(projection.expired_api_keys))
            != Some(projection.total_api_keys)
        || projection.total_modules < 0
        || projection.active_modules < 0
        || projection.active_modules > projection.total_modules
        || projection.total_requests_today < 0
        || projection.total_requests_this_month < 0
        || projection.api_keys.iter().any(|key| !key.is_well_formed())
        || projection
            .top_modules_by_usage
            .iter()
            .any(|module| !module.is_well_formed())
    {
        return None;
    }
    Some(projection)
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum DeveloperPortalLoad {
    Ready(AdminDeveloperPortalProjection),
    Empty(AdminDeveloperPortalProjection),
    Unauthenticated,
    Unauthorized,
    Forbidden,
    Unavailable,
    Malformed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DeveloperTab {
    Overview,
    Keys,
    Docs,
    Usage,
}

impl DeveloperTab {
    fn from_context(ctx: &PageContext) -> Self {
        match ctx.query_param("tab").as_deref() {
            Some("keys") => Self::Keys,
            Some("docs") => Self::Docs,
            Some("usage") => Self::Usage,
            Some("overview") | None => Self::Overview,
            Some(_) => Self::Overview,
        }
    }
}

fn developer_portal_load(ctx: &PageContext) -> DeveloperPortalLoad {
    let state = ctx
        .params
        .get(ADMIN_DEVELOPER_STATE_PARAM)
        .map(String::as_str);
    match state {
        Some(ADMIN_DEVELOPER_READY) | Some(ADMIN_DEVELOPER_EMPTY) => {
            let Some(raw) = ctx.params.get(ADMIN_DEVELOPER_DATA_PARAM) else {
                return DeveloperPortalLoad::Malformed;
            };
            let Some(projection) = serde_json::from_str(raw)
                .ok()
                .and_then(decode_admin_developer_projection)
            else {
                return DeveloperPortalLoad::Malformed;
            };
            match (
                state,
                projection.api_keys.is_empty(),
                projection.total_api_keys,
            ) {
                (Some(ADMIN_DEVELOPER_READY), false, _) => DeveloperPortalLoad::Ready(projection),
                (Some(ADMIN_DEVELOPER_READY), true, total) if total > 0 => {
                    DeveloperPortalLoad::Ready(projection)
                }
                (Some(ADMIN_DEVELOPER_EMPTY), true, 0) => DeveloperPortalLoad::Empty(projection),
                _ => DeveloperPortalLoad::Malformed,
            }
        }
        Some(ADMIN_DEVELOPER_FORBIDDEN) => DeveloperPortalLoad::Forbidden,
        Some(ADMIN_DEVELOPER_MALFORMED) => DeveloperPortalLoad::Malformed,
        Some(ADMIN_DEVELOPER_UNAUTHENTICATED) => DeveloperPortalLoad::Unauthenticated,
        Some(ADMIN_DEVELOPER_UNAUTHORIZED) => DeveloperPortalLoad::Unauthorized,
        Some(ADMIN_DEVELOPER_UNAVAILABLE) | None => DeveloperPortalLoad::Unavailable,
        Some(_) => DeveloperPortalLoad::Malformed,
    }
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    (
        PageMeta::admin("Developer Portal"),
        rsx! {
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("the private developer portal inventory".to_string()),
                return_url: Some(DEVELOPER_PORTAL_PATH.to_string()),
                RenderDeveloperPortal { ctx: ctx.clone() }
            }
        },
    )
}

#[component]
fn RenderDeveloperPortal(ctx: PageContext) -> Element {
    let load = developer_portal_load(&ctx);
    let mutation = developer_mutation_state(&ctx);
    let tab = DeveloperTab::from_context(&ctx);
    rsx! {
        PageLayout {
            max_width: Some(PageMaxWidth::SevenXl),
            PageHeader {
                title: "Developer Portal".to_string(),
                subtitle: Some("Manage API keys, documentation, and third-party integrations".to_string()),
                icon: Some("code".to_string()),
                gradient: Some(PageGradient::Warning),
                centered: Some(true),
                extra_actions: None,
                class_name: None,
            }
            if let Some(state) = mutation {
                DeveloperMutationNotice { state }
            }
            match load {
                DeveloperPortalLoad::Ready(projection) => rsx! { DeveloperPortalReady { projection, tab, state: ADMIN_DEVELOPER_READY } },
                DeveloperPortalLoad::Empty(projection) => rsx! { DeveloperPortalReady { projection, tab, state: ADMIN_DEVELOPER_EMPTY } },
                DeveloperPortalLoad::Forbidden => rsx! {
                    DeveloperPortalProblem {
                        tab,
                        state: ADMIN_DEVELOPER_FORBIDDEN,
                        title: "Developer portal access was denied".to_string(),
                        detail: "The backend did not authorize this session to read the redacted developer inventory.".to_string(),
                    }
                },
                DeveloperPortalLoad::Unauthenticated | DeveloperPortalLoad::Unauthorized => {
                    let state = if matches!(load, DeveloperPortalLoad::Unauthenticated) {
                        AdminDataState::Unauthenticated
                    } else {
                        AdminDataState::Unauthorized
                    };
                    rsx! {
                        AdminDataStateBanner {
                            state,
                            subject: "Developer portal".to_string(),
                            return_path: DEVELOPER_PORTAL_PATH.to_string(),
                            retry_href: DEVELOPER_PORTAL_PATH.to_string(),
                        }
                    }
                }
                DeveloperPortalLoad::Unavailable => rsx! {
                    DeveloperPortalProblem {
                        tab,
                        state: ADMIN_DEVELOPER_UNAVAILABLE,
                        title: "Developer portal data is unavailable".to_string(),
                        detail: "The backend did not provide an authoritative developer inventory. No credential or usage data is being shown.".to_string(),
                    }
                },
                DeveloperPortalLoad::Malformed => rsx! {
                    DeveloperPortalProblem {
                        tab,
                        state: ADMIN_DEVELOPER_MALFORMED,
                        title: "Developer portal data could not be verified".to_string(),
                        detail: "The backend response did not match the strict redacted read contract. No credential or usage data is being shown.".to_string(),
                    }
                },
            }
        }
    }
}

#[component]
fn DeveloperPortalReady(
    projection: AdminDeveloperPortalProjection,
    tab: DeveloperTab,
    state: &'static str,
) -> Element {
    rsx! {
        div { class: "space-y-6", "data-admin-developer-portal-state": state,
            match tab {
                DeveloperTab::Overview => rsx! { DeveloperOverview { projection } },
                DeveloperTab::Keys => rsx! { DeveloperKeys { projection } },
                DeveloperTab::Docs => rsx! { DeveloperDocumentation { modules_available: Some(projection.active_modules) } },
                DeveloperTab::Usage => rsx! { DeveloperUsage { projection } },
            }
        }
    }
}

#[component]
fn DeveloperOverview(projection: AdminDeveloperPortalProjection) -> Element {
    rsx! {
        section { class: "grid gap-4 sm:grid-cols-2 lg:grid-cols-4", aria_label: "Developer portal summary",
            SummaryCard { label: "Total API Keys", value: projection.total_api_keys.to_string(), icon: "key" }
            SummaryCard { label: "Active Keys", value: projection.active_api_keys.to_string(), icon: "activity" }
            SummaryCard { label: "Total Requests", value: projection.total_requests_this_month.to_string(), icon: "bar-chart-3" }
            SummaryCard { label: "Available Modules", value: projection.active_modules.to_string(), icon: "settings" }
        }
        div { class: "grid gap-6 lg:grid-cols-2",
            section { class: "overflow-hidden rounded-2xl border border-border/20 bg-card shadow-xl", aria_labelledby: "developer-recent-keys-title",
                div { class: "h-[3px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]" }
                div { class: "flex flex-wrap items-center justify-between gap-3 border-b border-border/20 p-6",
                    div {
                        h2 { id: "developer-recent-keys-title", class: "text-lg font-medium text-foreground", "Recent API Keys" }
                        p { class: "mt-1 text-sm text-muted-foreground", "Redacted backend-authoritative inventory" }
                    }
                    a { class: "btn btn-sm btn-primary", href: DEVELOPER_CREATE_PATH,
                        Icon { name: "plus".to_string(), size: Some(14) }
                        " Create API Key"
                    }
                }
                if projection.api_keys.is_empty() {
                    p { class: "p-10 text-center text-sm text-muted-foreground", role: "status", "No API keys have been created yet." }
                } else {
                    div { class: "divide-y divide-border/20",
                        for api_key in projection.api_keys.iter().take(5) {
                            DeveloperRecentKeyRow { api_key: api_key.clone() }
                        }
                    }
                }
                a { class: "block border-t border-border/20 p-4 text-center text-sm font-medium text-[#1fc7d4]", href: "/developer-portal?tab=keys", "View all API keys" }
            }
            section { class: "overflow-hidden rounded-2xl border border-border/20 bg-card shadow-xl", aria_labelledby: "developer-module-usage-title",
                div { class: "h-[3px] bg-gradient-to-r from-[#7645d9] to-[#ed4b9e]" }
                div { class: "border-b border-border/20 p-6",
                    h2 { id: "developer-module-usage-title", class: "text-lg font-medium text-foreground", "Module Activity" }
                    p { class: "mt-1 text-sm text-muted-foreground", "Persisted usage totals from the backend" }
                }
                if projection.top_modules_by_usage.is_empty() {
                    p { class: "p-10 text-center text-sm text-muted-foreground", role: "status", "No module usage was reported." }
                } else {
                    ul { class: "divide-y divide-border/20", aria_label: "Top module usage",
                        for module in projection.top_modules_by_usage.iter().take(5) {
                            DeveloperModuleUsageRow { module: module.clone() }
                        }
                    }
                }
                a { class: "block border-t border-border/20 p-4 text-center text-sm font-medium text-[#7645d9]", href: "/developer-portal?tab=usage", "View usage analytics" }
            }
        }
    }
}

#[component]
fn DeveloperRecentKeyRow(api_key: AdminDeveloperApiKeySummary) -> Element {
    let status_class = developer_status_class(&api_key.status);
    rsx! {
        article { class: "flex flex-wrap items-center justify-between gap-4 p-5",
            div { class: "flex min-w-0 items-center gap-3",
                span { class: "rounded-xl bg-muted/30 p-2", Icon { name: "key".to_string(), size: Some(18) } }
                div { class: "min-w-0",
                    h3 { class: "truncate text-sm font-medium text-foreground", "{api_key.client_name}" }
                    p { class: "mt-1 font-mono text-xs text-muted-foreground", "{api_key.key_prefix}..." }
                }
            }
            div { class: "text-right",
                span { class: "inline-flex rounded-full border px-2 py-1 text-xs font-medium {status_class}", "{api_key.status}" }
                p { class: "mt-2 text-xs text-muted-foreground", "{api_key.total_requests} requests" }
            }
        }
    }
}

#[component]
fn DeveloperKeys(projection: AdminDeveloperPortalProjection) -> Element {
    let key_count = projection.api_keys.len();
    rsx! {
        section { class: "space-y-6",
            div { class: "flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between",
                div {
                    h2 { class: "text-2xl font-black tracking-tight text-foreground", "API Keys" }
                    p { class: "mt-1 text-sm text-muted-foreground", "Manage credentials and lifecycle controls" }
                }
                a { class: "btn btn-primary", href: DEVELOPER_CREATE_PATH,
                    Icon { name: "plus".to_string(), size: Some(16) }
                    " Create API Key"
                }
            }
            div { class: "flex flex-wrap gap-3 rounded-2xl border border-border/20 bg-card p-4",
                input { class: "input min-w-56 flex-1", r#type: "search", placeholder: "Search API keys", disabled: true, title: "Server-side API-key search is not exposed by the current endpoint" }
                select { class: "input min-w-40", disabled: true, title: "Server-side status filtering is not exposed by the current endpoint",
                    option { "All Statuses ({projection.total_api_keys})" }
                }
                a { class: "btn btn-outline", href: DEVELOPER_PORTAL_PATH, "Refresh" }
            }
            section { class: "overflow-hidden rounded-2xl border border-border/20 bg-card shadow-xl", aria_labelledby: "developer-api-key-inventory-title",
                div { class: "h-[3px] bg-gradient-to-r from-[#1fc7d4] via-[#7645d9] to-[#ed4b9e]" }
                div { class: "border-b border-border/20 p-5",
                    h3 { id: "developer-api-key-inventory-title", class: "text-lg font-semibold text-foreground", "API-key inventory" }
                    p { class: "mt-1 text-sm text-muted-foreground", "Showing {key_count} of {projection.total_api_keys} redacted API keys" }
                }
                if projection.api_keys.is_empty() {
                    p { class: "p-12 text-center text-sm text-muted-foreground", role: "status", "No API keys have been created yet." }
                } else {
                    div { class: "divide-y divide-border/20",
                        for api_key in projection.api_keys {
                            DeveloperApiKeyRow { api_key }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn DeveloperDocumentation(modules_available: Option<i64>) -> Element {
    rsx! {
        section { class: "overflow-hidden rounded-2xl border border-border/20 bg-card shadow-xl",
            div { class: "border-b border-border/20 p-6",
                h2 { class: "text-lg font-semibold text-foreground", "API Documentation" }
                p { class: "mt-1 text-sm text-muted-foreground", "Complete guide to using the module-based API" }
            }
            div { class: "space-y-6 p-6",
                DeveloperDocumentationSection { icon: "shield", title: "Authentication",
                    p { class: "text-sm text-muted-foreground", "Include your API key in the Authorization header:" }
                    code { class: "mt-3 block overflow-x-auto rounded-lg bg-gray-950 p-4 font-mono text-sm text-green-400", "Authorization: Bearer YOUR_API_KEY" }
                }
                DeveloperDocumentationSection { icon: "globe", title: "Base URL",
                    code { class: "block rounded-lg bg-muted/30 p-4 font-mono text-sm text-foreground", "https://api.epsx.io" }
                }
                DeveloperDocumentationSection { icon: "code", title: "Available Endpoints",
                    div { class: "rounded-lg border border-border/20 bg-muted/20 p-4 text-sm text-muted-foreground",
                        if let Some(count) = modules_available { "{count} active modules are available. Endpoint paths remain backend-owned and are not inferred in this redacted projection." } else { "Module endpoint definitions are unavailable." }
                    }
                }
                DeveloperDocumentationSection { icon: "alert-triangle", title: "Rate Limits",
                    p { class: "rounded-lg border border-amber-500/20 bg-amber-500/5 p-4 text-sm text-muted-foreground", "Limits are assigned per API key by the backend. This inventory intentionally omits credential quotas." }
                }
            }
        }
    }
}

#[component]
fn DeveloperDocumentationSection(
    icon: &'static str,
    title: &'static str,
    children: Element,
) -> Element {
    rsx! {
        section {
            h3 { class: "mb-3 flex items-center gap-2 text-base font-semibold text-foreground",
                Icon { name: icon.to_string(), size: Some(18) }
                "{title}"
            }
            {children}
        }
    }
}

#[component]
fn DeveloperUsage(projection: AdminDeveloperPortalProjection) -> Element {
    rsx! {
        section { class: "space-y-6",
            div { class: "flex flex-wrap items-center justify-between gap-3",
                div {
                    h2 { class: "text-2xl font-black tracking-tight text-foreground", "Usage Analytics" }
                    p { class: "mt-1 text-sm text-muted-foreground", "Backend-authoritative API activity" }
                }
                button { class: "btn btn-outline", r#type: "button", disabled: true, title: "Usage export requires a backend-owned export contract", "Export Data" }
            }
            div { class: "grid gap-4 sm:grid-cols-3",
                SummaryCard { label: "Requests Today", value: projection.total_requests_today.to_string(), icon: "activity" }
                SummaryCard { label: "Requests This Month", value: projection.total_requests_this_month.to_string(), icon: "bar-chart-3" }
                SummaryCard { label: "Active API Keys", value: projection.active_api_keys.to_string(), icon: "key" }
            }
            div { class: "grid gap-6 lg:grid-cols-2",
                DeveloperUsagePanel { title: "Requests Over Time", icon: "bar-chart-3", detail: "Timeline telemetry is unavailable in the current typed projection." }
                section { class: "rounded-2xl border border-border/20 bg-card p-6 shadow-xl",
                    h3 { class: "mb-4 text-base font-medium text-foreground", "Module Usage Distribution" }
                    if projection.top_modules_by_usage.is_empty() {
                        p { class: "flex h-48 items-center justify-center rounded-lg bg-muted/20 text-sm text-muted-foreground", "No module usage was reported." }
                    } else {
                        ul { class: "divide-y divide-border/20 rounded-lg border border-border/20",
                            for module in projection.top_modules_by_usage {
                                DeveloperModuleUsageRow { module }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn DeveloperUsagePanel(title: &'static str, icon: &'static str, detail: &'static str) -> Element {
    rsx! {
        section { class: "rounded-2xl border border-border/20 bg-card p-6 shadow-xl",
            h3 { class: "mb-4 text-base font-medium text-foreground", "{title}" }
            div { class: "flex h-48 flex-col items-center justify-center rounded-lg bg-muted/20 px-6 text-center text-muted-foreground",
                Icon { name: icon.to_string(), size: Some(28) }
                p { class: "mt-3 text-sm", "Unavailable" }
                p { class: "mt-1 text-xs", "{detail}" }
            }
        }
    }
}

#[component]
fn SummaryCard(label: &'static str, value: String, icon: &'static str) -> Element {
    rsx! {
        div { class: "rounded-2xl border border-border/30 bg-card p-5 shadow-lg",
            div { class: "flex items-center justify-between gap-3",
                p { class: "text-xs font-semibold uppercase tracking-wide text-muted-foreground", "{label}" }
                Icon { name: icon.to_string(), size: Some(18) }
            }
            p { class: "mt-3 text-2xl font-black tracking-tight text-foreground", "{value}" }
        }
    }
}

#[component]
fn DeveloperApiKeyRow(api_key: AdminDeveloperApiKeySummary) -> Element {
    let status_class = developer_status_class(&api_key.status);
    let expires_at = api_key.expires_at.as_deref().unwrap_or("No expiry");
    let last_used_at = api_key.last_used_at.as_deref().unwrap_or("Not used");
    rsx! {
        article { class: "grid gap-4 p-5 md:grid-cols-12 md:items-center",
            div { class: "min-w-0 md:col-span-4",
                h3 { class: "truncate text-sm font-semibold text-foreground", "{api_key.client_name}" }
                p { class: "mt-1 break-all font-mono text-xs text-muted-foreground", "Prefix: {api_key.key_prefix}" }
            }
            div { class: "md:col-span-2",
                span { class: "inline-flex rounded-full border px-2.5 py-1 text-xs font-semibold {status_class}", "{api_key.status}" }
            }
            div { class: "md:col-span-2",
                p { class: "text-xs uppercase tracking-wide text-muted-foreground", "Requests" }
                p { class: "mt-1 text-sm text-foreground", "{api_key.total_requests}" }
            }
            div { class: "md:col-span-2",
                p { class: "text-xs uppercase tracking-wide text-muted-foreground", "Expires" }
                p { class: "mt-1 break-words text-sm text-foreground", "{expires_at}" }
            }
            div { class: "md:col-span-2",
                p { class: "text-xs uppercase tracking-wide text-muted-foreground", "Last used" }
                p { class: "mt-1 break-words text-sm text-foreground", "{last_used_at}" }
            }
            if api_key.status == "active" {
                div { class: "flex flex-wrap gap-3 md:col-span-12",
                    form { method: "post", action: DEVELOPER_PORTAL_PATH, class: "flex flex-wrap items-end gap-2",
                        input { r#type: "hidden", name: "operation", value: "revoke" }
                        input { r#type: "hidden", name: "api_key_id", value: api_key.id.clone() }
                        input { r#type: "hidden", name: "idempotency_key", value: format!("admin.developer.revoke.{}", Uuid::new_v4()) }
                        label { class: "text-xs text-muted-foreground", "Revocation reason",
                            input { class: "input input-bordered input-sm ml-2", name: "reason", maxlength: 500, required: true }
                        }
                        button { r#type: "submit", class: "btn btn-sm btn-outline", "Revoke key" }
                    }
                    form { method: "post", action: DEVELOPER_PORTAL_PATH, class: "flex flex-wrap items-end gap-2",
                        input { r#type: "hidden", name: "operation", value: "expiration" }
                        input { r#type: "hidden", name: "api_key_id", value: api_key.id }
                        input { r#type: "hidden", name: "idempotency_key", value: format!("admin.developer.expiration.{}", Uuid::new_v4()) }
                        label { class: "text-xs text-muted-foreground", "Expiration (RFC3339, blank clears)",
                            input { class: "input input-bordered input-sm ml-2", name: "expires_at", maxlength: 64, value: expires_at }
                        }
                        button { r#type: "submit", class: "btn btn-sm btn-outline", "Update expiration" }
                    }
                }
            }
        }
    }
}

fn developer_status_class(status: &str) -> &'static str {
    match status {
        "active" => "border-green-500/20 bg-green-500/10 text-green-400",
        "expired" => "border-red-500/20 bg-red-500/10 text-red-400",
        _ => "border-amber-500/20 bg-amber-500/10 text-amber-400",
    }
}

fn developer_mutation_state(ctx: &PageContext) -> Option<&'static str> {
    match ctx.query_param("mutation").as_deref() {
        Some("success") => Some("success"),
        Some("conflict") => Some("conflict"),
        Some("forbidden") => Some("forbidden"),
        Some("unavailable") => Some("unavailable"),
        Some("malformed") => Some("malformed"),
        _ => None,
    }
}

#[component]
fn DeveloperMutationNotice(state: &'static str) -> Element {
    rsx! {
        section {
            class: "mb-5 rounded-2xl border border-amber-500/30 bg-amber-500/5 p-5",
            role: if state == "forbidden" { "alert" } else { "status" },
            "data-admin-developer-mutation-state": state,
            h2 { class: "text-lg font-semibold text-foreground", "Developer-key mutation: {state}" }
            p { class: "mt-2 text-sm leading-6 text-muted-foreground",
                if state == "success" { "The backend committed the lifecycle change and the inventory will be reloaded." } else { "The backend did not confirm this lifecycle change. No local state is inferred." }
            }
        }
    }
}

#[component]
fn DeveloperModuleUsageRow(module: AdminDeveloperModuleUsage) -> Element {
    rsx! {
        li { class: "flex flex-wrap items-center justify-between gap-4 p-5",
            div { class: "min-w-0",
                h3 { class: "truncate text-sm font-semibold text-foreground", "{module.module_name}" }
                p { class: "mt-1 break-all font-mono text-xs text-muted-foreground", "{module.module_id}" }
            }
            dl { class: "flex gap-6 text-right text-sm",
                div {
                    dt { class: "text-xs uppercase tracking-wide text-muted-foreground", "Requests" }
                    dd { class: "mt-1 font-semibold text-foreground", "{module.request_count}" }
                }
                div {
                    dt { class: "text-xs uppercase tracking-wide text-muted-foreground", "API keys" }
                    dd { class: "mt-1 font-semibold text-foreground", "{module.unique_api_keys}" }
                }
            }
        }
    }
}

#[component]
fn DeveloperPortalProblem(
    tab: DeveloperTab,
    state: &'static str,
    title: String,
    detail: String,
) -> Element {
    rsx! {
        div { class: "space-y-6",
            "data-admin-developer-portal-state": state,
            section { class: "flex flex-col gap-4 rounded-2xl border border-amber-500/25 bg-amber-500/5 p-5 sm:flex-row sm:items-center sm:justify-between",
                role: if state == ADMIN_DEVELOPER_FORBIDDEN { "alert" } else { "status" },
                div { class: "flex items-start gap-3",
                    span { class: "rounded-xl bg-amber-500/10 p-2 text-amber-400", Icon { name: "shield".to_string(), size: Some(20) } }
                    div {
                        h2 { class: "font-semibold text-foreground", "{title}" }
                        p { class: "mt-1 max-w-3xl text-sm leading-6 text-muted-foreground", "{detail}" }
                    }
                }
                div { class: "flex flex-shrink-0 gap-2",
                    a { class: "btn btn-sm btn-primary", href: DEVELOPER_PORTAL_PATH, "Try again" }
                    a { class: "btn btn-sm btn-ghost", href: "/", "Admin home" }
                }
            }
            match tab {
                DeveloperTab::Overview => rsx! { DeveloperOverviewUnavailable {} },
                DeveloperTab::Keys => rsx! { DeveloperKeysUnavailable {} },
                DeveloperTab::Docs => rsx! { DeveloperDocumentation { modules_available: None } },
                DeveloperTab::Usage => rsx! { DeveloperUsageUnavailable {} },
            }
        }
    }
}

#[component]
fn DeveloperOverviewUnavailable() -> Element {
    rsx! {
        section { class: "grid gap-4 sm:grid-cols-2 lg:grid-cols-4", aria_label: "Developer portal summary unavailable",
            SummaryCard { label: "Total API Keys", value: "Unavailable".to_string(), icon: "key" }
            SummaryCard { label: "Active Keys", value: "Unavailable".to_string(), icon: "activity" }
            SummaryCard { label: "Total Requests", value: "Unavailable".to_string(), icon: "bar-chart-3" }
            SummaryCard { label: "Available Modules", value: "Unavailable".to_string(), icon: "settings" }
        }
        div { class: "grid gap-6 lg:grid-cols-2",
            DeveloperUnavailablePanel { title: "Recent API Keys", detail: "No verified redacted key inventory is available.", icon: "key" }
            DeveloperUnavailablePanel { title: "Module Activity", detail: "No verified module-usage projection is available.", icon: "activity" }
        }
    }
}

#[component]
fn DeveloperKeysUnavailable() -> Element {
    rsx! {
        section { class: "space-y-6",
            div { class: "flex flex-wrap items-end justify-between gap-4",
                div {
                    h2 { class: "text-2xl font-black tracking-tight text-foreground", "API Keys" }
                    p { class: "mt-1 text-sm text-muted-foreground", "Manage credentials and lifecycle controls" }
                }
                button { class: "btn btn-primary", r#type: "button", disabled: true, "Create API Key" }
            }
            div { class: "flex flex-wrap gap-3 rounded-2xl border border-border/20 bg-card p-4",
                input { class: "input min-w-56 flex-1", placeholder: "Search API keys", disabled: true }
                select { class: "input min-w-40", disabled: true, option { "All Statuses" } }
                a { class: "btn btn-outline", href: DEVELOPER_PORTAL_PATH, "Refresh" }
            }
            DeveloperUnavailablePanel { title: "API-key inventory", detail: "No verified credential rows are being shown.", icon: "key" }
        }
    }
}

#[component]
fn DeveloperUsageUnavailable() -> Element {
    rsx! {
        section { class: "space-y-6",
            div { class: "flex flex-wrap items-center justify-between gap-3",
                div {
                    h2 { class: "text-2xl font-black tracking-tight text-foreground", "Usage Analytics" }
                    p { class: "mt-1 text-sm text-muted-foreground", "Backend-authoritative API activity" }
                }
                button { class: "btn btn-outline", r#type: "button", disabled: true, "Export Data" }
            }
            div { class: "grid gap-4 sm:grid-cols-3",
                SummaryCard { label: "Requests Today", value: "Unavailable".to_string(), icon: "activity" }
                SummaryCard { label: "Requests This Month", value: "Unavailable".to_string(), icon: "bar-chart-3" }
                SummaryCard { label: "Active API Keys", value: "Unavailable".to_string(), icon: "key" }
            }
            div { class: "grid gap-6 lg:grid-cols-2",
                DeveloperUsagePanel { title: "Requests Over Time", icon: "bar-chart-3", detail: "Timeline telemetry is unavailable." }
                DeveloperUsagePanel { title: "Module Usage Distribution", icon: "activity", detail: "Module usage telemetry is unavailable." }
            }
        }
    }
}

#[component]
fn DeveloperUnavailablePanel(
    title: &'static str,
    detail: &'static str,
    icon: &'static str,
) -> Element {
    rsx! {
        section { class: "overflow-hidden rounded-2xl border border-border/20 bg-card shadow-xl",
            div { class: "h-[3px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]" }
            div { class: "border-b border-border/20 p-6", h3 { class: "text-lg font-medium text-foreground", "{title}" } }
            div { class: "flex min-h-48 flex-col items-center justify-center p-8 text-center text-muted-foreground",
                Icon { name: icon.to_string(), size: Some(28) }
                p { class: "mt-3 text-sm font-medium", "Unavailable" }
                p { class: "mt-1 text-xs", "{detail}" }
            }
        }
    }
}

pub fn render_create_key(ctx: &PageContext) -> (PageMeta, Element) {
    (
        PageMeta::admin("Create API Key"),
        rsx! {
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("API-key creation".to_string()),
                return_url: Some(DEVELOPER_CREATE_PATH.to_string()),
                PageLayout {
                    max_width: Some(PageMaxWidth::FourXl),
                    RenderDeveloperCreateKey { ctx: ctx.clone() }
                }
            }
        },
    )
}

#[derive(Clone, PartialEq, Eq)]
enum DeveloperCreateLoad {
    Form,
    Created(AdminDeveloperSecretOnceProjection),
    Conflict,
    Unauthenticated,
    Unauthorized,
    Forbidden,
    Unavailable,
    Malformed,
}

fn developer_create_load(ctx: &PageContext) -> DeveloperCreateLoad {
    match ctx
        .params
        .get(ADMIN_DEVELOPER_CREATE_STATE_PARAM)
        .map(String::as_str)
    {
        Some(ADMIN_DEVELOPER_CREATE_FORM) => DeveloperCreateLoad::Form,
        Some(ADMIN_DEVELOPER_CREATE_CREATED) => {
            let Some(raw) = ctx.params.get(ADMIN_DEVELOPER_CREATE_DATA_PARAM) else {
                return DeveloperCreateLoad::Malformed;
            };
            serde_json::from_str(raw)
                .ok()
                .and_then(decode_admin_developer_secret_once)
                .map(DeveloperCreateLoad::Created)
                .unwrap_or(DeveloperCreateLoad::Malformed)
        }
        Some(ADMIN_DEVELOPER_CREATE_CONFLICT) => DeveloperCreateLoad::Conflict,
        Some(ADMIN_DEVELOPER_CREATE_FORBIDDEN) => DeveloperCreateLoad::Forbidden,
        Some(ADMIN_DEVELOPER_CREATE_UNAVAILABLE) => DeveloperCreateLoad::Unavailable,
        Some(ADMIN_DEVELOPER_CREATE_UNAUTHENTICATED) => DeveloperCreateLoad::Unauthenticated,
        Some(ADMIN_DEVELOPER_CREATE_UNAUTHORIZED) => DeveloperCreateLoad::Unauthorized,
        Some(ADMIN_DEVELOPER_CREATE_MALFORMED) | Some(_) => DeveloperCreateLoad::Malformed,
        None => DeveloperCreateLoad::Unavailable,
    }
}

#[component]
fn RenderDeveloperCreateKey(ctx: PageContext) -> Element {
    let load = developer_create_load(&ctx);
    match load {
        DeveloperCreateLoad::Form => rsx! { DeveloperCreateForm {} },
        DeveloperCreateLoad::Created(projection) => {
            rsx! { DeveloperCreateSecretOnce { projection } }
        }
        DeveloperCreateLoad::Conflict => {
            rsx! { DeveloperCreateProblem { state: ADMIN_DEVELOPER_CREATE_CONFLICT, detail: "The API-key request conflicted with an existing mutation. Retry with a new idempotency key after verifying the backend state.".to_string() } }
        }
        DeveloperCreateLoad::Unauthenticated | DeveloperCreateLoad::Unauthorized => {
            let state = if matches!(load, DeveloperCreateLoad::Unauthenticated) {
                AdminDataState::Unauthenticated
            } else {
                AdminDataState::Unauthorized
            };
            rsx! {
                AdminDataStateBanner {
                    state,
                    subject: "Developer portal".to_string(),
                    return_path: DEVELOPER_CREATE_PATH.to_string(),
                    retry_href: DEVELOPER_CREATE_PATH.to_string(),
                }
            }
        }
        DeveloperCreateLoad::Forbidden => {
            rsx! { DeveloperCreateProblem { state: ADMIN_DEVELOPER_CREATE_FORBIDDEN, detail: "The backend denied API-key creation for this admin session.".to_string() } }
        }
        DeveloperCreateLoad::Unavailable => {
            rsx! { DeveloperCreateProblem { state: ADMIN_DEVELOPER_CREATE_UNAVAILABLE, detail: "The backend did not provide an authoritative API-key creation response.".to_string() } }
        }
        DeveloperCreateLoad::Malformed => {
            rsx! { DeveloperCreateProblem { state: ADMIN_DEVELOPER_CREATE_MALFORMED, detail: "The creation response did not match the strict secret-once contract.".to_string() } }
        }
    }
}

#[component]
fn DeveloperCreateForm() -> Element {
    let idempotency_key = Uuid::new_v4().to_string();
    rsx! {
        section {
            class: "rounded-2xl border border-border/20 bg-card p-5 shadow-xl sm:p-8",
            role: "region",
            "data-admin-developer-create-state": ADMIN_DEVELOPER_CREATE_FORM,
            header { class: "mb-8 flex items-center gap-3",
                div { class: "rounded-lg bg-blue-500/10 p-2",
                    Icon { name: "shield".to_string(), size: Some(24), class_name: Some("text-blue-400".to_string()) }
                }
                div {
                    h1 { class: "text-2xl font-bold text-foreground", "Create API Key" }
                    p { class: "text-sm text-muted-foreground", "Generate a new API key for a third-party integration" }
                }
            }
            form { method: "post", action: DEVELOPER_CREATE_PATH,
                class: "space-y-6",
                div { class: "grid grid-cols-1 gap-6 md:grid-cols-2",
                    label { class: "block text-sm font-medium text-foreground", "Client Name ", span { class: "text-red-500", "*" }
                        input { class: "input input-bordered mt-2 w-full", id: "client_name", name: "client_name", maxlength: 255, required: true, placeholder: "My Application" }
                    }
                    label { class: "block text-sm font-medium text-foreground", "Contact Email",
                        input { class: "input input-bordered mt-2 w-full", id: "client_contact_email", name: "client_contact_email", r#type: "email", maxlength: 320, placeholder: "contact@example.com" }
                    }
                }
                label { class: "block text-sm font-medium text-foreground", "Description",
                    textarea { class: "textarea textarea-bordered mt-2 w-full", id: "client_description", name: "client_description", rows: 3, maxlength: 2_000, placeholder: "Brief description of your application and use case" }
                }
                label { class: "block text-sm font-medium text-foreground", "Expiration Date (Optional)",
                    input { class: "input input-bordered mt-2 w-full", id: "expires_at", name: "expires_at", r#type: "datetime-local", maxlength: 64 }
                    span { class: "mt-1 block text-xs font-normal text-muted-foreground", "Entered time is normalized to UTC before the backend mutation." }
                }
                label { class: "block text-sm font-medium text-foreground", "IP Restrictions (Optional)",
                    textarea { class: "textarea textarea-bordered mt-2 w-full font-mono", id: "ip_restrictions", name: "ip_restrictions", rows: 4, maxlength: 1_300, placeholder: "192.168.1.0/24\n203.0.113.0/24\nOne IP address or CIDR block per line" }
                }
                aside { class: "rounded-lg border border-yellow-500/20 bg-yellow-500/5 p-4",
                    div { class: "flex items-start gap-3",
                        Icon { name: "triangle-alert".to_string(), size: Some(20), class_name: Some("mt-0.5 shrink-0 text-yellow-400".to_string()) }
                        div {
                            h2 { class: "mb-1 font-medium text-yellow-300", "Module Configuration Required" }
                            p { class: "text-sm leading-6 text-yellow-200/70", "After creating the API key, configure module permissions and access levels in the main developer portal." }
                        }
                    }
                }
                input { r#type: "hidden", name: "idempotency_key", value: idempotency_key }
                div { class: "flex flex-wrap gap-3 border-t border-border/20 pt-6",
                    a { class: "btn btn-outline", href: DEVELOPER_PORTAL_PATH,
                        Icon { name: "arrow-left".to_string(), size: Some(16) }
                        "Cancel"
                    }
                    button { r#type: "submit", class: "btn btn-primary", "data-admin-developer-create-submit": "bff",
                        Icon { name: "plus".to_string(), size: Some(16) }
                        "Create API Key"
                    }
                }
            }
        }
    }
}

#[component]
fn DeveloperCreateSecretOnce(projection: AdminDeveloperSecretOnceProjection) -> Element {
    rsx! {
        section {
            class: "rounded-2xl border border-emerald-500/30 bg-emerald-500/5 p-6 shadow-xl",
            role: "alert",
            "data-admin-developer-create-state": ADMIN_DEVELOPER_CREATE_CREATED,
            h1 { class: "text-xl font-semibold text-foreground", "API key created" }
            p { class: "mt-2 text-sm leading-6 text-muted-foreground", "This secret is shown once. It is not included in list, read, or audit projections." }
            dl { class: "mt-6 space-y-4 text-sm",
                div {
                    dt { class: "text-xs uppercase tracking-wide text-muted-foreground", "Client" }
                    dd { class: "mt-1 font-semibold text-foreground", "{projection.api_key.client_name}" }
                }
                div {
                    dt { class: "text-xs uppercase tracking-wide text-muted-foreground", "Key prefix" }
                    dd { class: "mt-1 font-mono text-foreground", "{projection.api_key.key_prefix}" }
                }
                div {
                    dt { class: "text-xs uppercase tracking-wide text-muted-foreground", "Secret — copy now" }
                    dd { class: "mt-1 break-all rounded-lg border border-amber-500/30 bg-amber-500/10 p-3 font-mono text-amber-200", "data-secret-once": "true", "{projection.secret}" }
                }
            }
            a { class: "btn btn-outline mt-6", href: DEVELOPER_PORTAL_PATH, "Return to developer portal" }
        }
    }
}

#[component]
fn DeveloperCreateProblem(state: &'static str, detail: String) -> Element {
    rsx! {
        section {
            class: "rounded-2xl border border-amber-500/30 bg-amber-500/5 p-6",
            role: "alert",
            "data-admin-developer-create-state": state,
            h1 { class: "text-xl font-semibold text-foreground", "API-key creation: {state}" }
            p { class: "mt-2 text-sm leading-6 text-muted-foreground", "{detail}" }
            a { class: "btn btn-outline mt-5", href: DEVELOPER_CREATE_PATH, "Try again" }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::auth::user::{AuthMethod, User};

    fn session() -> User {
        User {
            id: "developer-session".to_string(),
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

    fn key() -> AdminDeveloperApiKeySummary {
        AdminDeveloperApiKeySummary {
            id: "01234567-89ab-4cde-8fab-0123456789ab".to_string(),
            key_prefix: "epsx_abc123".to_string(),
            client_name: "Production integration".to_string(),
            status: "active".to_string(),
            total_requests: 124,
            expires_at: Some("2026-12-31T00:00:00Z".to_string()),
            last_used_at: Some("2026-07-27T10:00:00Z".to_string()),
            created_at: "2026-07-01T10:00:00Z".to_string(),
        }
    }

    fn module() -> AdminDeveloperModuleUsage {
        AdminDeveloperModuleUsage {
            module_id: "11234567-89ab-4cde-8fab-0123456789ab".to_string(),
            module_name: "Market data".to_string(),
            request_count: 80,
            unique_api_keys: 2,
        }
    }

    fn projection(
        api_keys: Vec<AdminDeveloperApiKeySummary>,
        total_api_keys: i64,
    ) -> AdminDeveloperPortalProjection {
        AdminDeveloperPortalProjection {
            api_keys,
            total_api_keys,
            active_api_keys: total_api_keys,
            revoked_api_keys: 0,
            expired_api_keys: 0,
            total_modules: 1,
            active_modules: 1,
            total_requests_today: 20,
            total_requests_this_month: 800,
            top_modules_by_usage: vec![module()],
        }
    }

    fn ctx(state: &str, projection: Option<AdminDeveloperPortalProjection>) -> PageContext {
        let mut params =
            HashMap::from([(ADMIN_DEVELOPER_STATE_PARAM.to_string(), state.to_string())]);
        if let Some(projection) = projection {
            params.insert(
                ADMIN_DEVELOPER_DATA_PARAM.to_string(),
                serde_json::to_string(&projection).unwrap(),
            );
        }
        PageContext {
            user: Some(session()),
            path: DEVELOPER_PORTAL_PATH.to_string(),
            params,
            ..Default::default()
        }
    }

    fn html(ctx: &PageContext) -> String {
        dioxus_ssr::render_element(render(ctx).1)
    }

    #[test]
    fn signed_out_session_keeps_projection_private() {
        let mut ctx = ctx(ADMIN_DEVELOPER_READY, Some(projection(vec![key()], 1)));
        ctx.user = None;
        ctx.params.insert(
            ADMIN_DEVELOPER_DATA_PARAM.to_string(),
            "PRIVATE_DEVELOPER_PAYLOAD".to_string(),
        );

        let rendered = html(&ctx);

        assert!(rendered.contains("Sign in required"));
        assert!(!rendered.contains("data-admin-developer-portal-state"));
        assert!(!rendered.contains("PRIVATE_DEVELOPER_PAYLOAD"));
        assert!(!rendered.contains("Production integration"));
    }

    #[test]
    fn ready_projection_renders_redacted_fields_and_backend_lifecycle_forms() {
        let mut ready = ctx(ADMIN_DEVELOPER_READY, Some(projection(vec![key()], 1)));
        ready.query = "tab=keys".to_string();
        let rendered = html(&ready);

        assert!(rendered.contains("data-admin-developer-portal-state=\"ready\""));
        assert!(rendered.contains("Production integration"));
        assert!(rendered.contains("Prefix: epsx_abc123"));
        for forbidden in ["full_key", "epsx_live_", "Authorization: Bearer"] {
            assert!(
                !rendered.contains(forbidden),
                "secret or mutation leaked: {forbidden}"
            );
        }
        assert!(rendered.contains("Create API Key"));
        assert!(rendered.contains("Revoke key"));
        assert!(rendered.contains("Update expiration"));
        assert!(rendered.contains("admin.developer.revoke."));
        assert!(rendered.contains("admin.developer.expiration."));

        let overview = html(&ctx(
            ADMIN_DEVELOPER_READY,
            Some(projection(vec![key()], 1)),
        ));
        assert!(overview.contains("Market data"));
        assert!(overview.contains("Available Modules"));
    }

    #[test]
    fn empty_forbidden_unavailable_and_malformed_are_distinct() {
        let empty = html(&ctx(ADMIN_DEVELOPER_EMPTY, Some(projection(vec![], 0))));
        let forbidden = html(&ctx(ADMIN_DEVELOPER_FORBIDDEN, None));
        let unavailable = html(&ctx(ADMIN_DEVELOPER_UNAVAILABLE, None));
        let malformed = html(&ctx(ADMIN_DEVELOPER_MALFORMED, None));

        assert!(empty.contains("data-admin-developer-portal-state=\"empty\""));
        assert!(empty.contains("No API keys have been created yet"));
        assert!(forbidden.contains("data-admin-developer-portal-state=\"forbidden\""));
        assert!(forbidden.contains("access was denied"));
        assert!(unavailable.contains("data-admin-developer-portal-state=\"unavailable\""));
        assert!(unavailable.contains("data is unavailable"));
        assert!(malformed.contains("data-admin-developer-portal-state=\"malformed\""));
        assert!(malformed.contains("could not be verified"));
    }

    #[test]
    fn decoder_rejects_unknown_secret_fields_and_invalid_bounds() {
        let valid = serde_json::to_value(projection(vec![key()], 1)).unwrap();
        assert!(decode_admin_developer_projection(valid.clone()).is_some());

        let mut unknown = valid.clone();
        unknown["full_key"] = serde_json::json!("epsx_live_secret");
        assert!(decode_admin_developer_projection(unknown).is_none());

        let mut invalid_status = valid.clone();
        invalid_status["api_keys"][0]["status"] = serde_json::json!("pending");
        assert!(decode_admin_developer_projection(invalid_status).is_none());

        let mut invalid_counts = valid.clone();
        invalid_counts["api_keys"][0]["total_requests"] = serde_json::json!(-1);
        assert!(decode_admin_developer_projection(invalid_counts).is_none());

        let mut invalid_module = valid;
        invalid_module["top_modules_by_usage"][0]["unique_api_keys"] = serde_json::json!(81);
        assert!(decode_admin_developer_projection(invalid_module).is_none());
    }

    #[test]
    fn inconsistent_state_payload_becomes_malformed() {
        let ready_empty = html(&ctx(ADMIN_DEVELOPER_READY, Some(projection(vec![], 0))));
        let empty_with_record = html(&ctx(
            ADMIN_DEVELOPER_EMPTY,
            Some(projection(vec![key()], 1)),
        ));
        let missing_payload = html(&ctx(ADMIN_DEVELOPER_READY, None));

        for rendered in [ready_empty, empty_with_record, missing_payload] {
            assert!(rendered.contains("data-admin-developer-portal-state=\"malformed\""));
            assert!(rendered.contains("could not be verified"));
        }
    }

    #[test]
    fn unauthenticated_and_unauthorized_decode_and_render_the_shared_banner() {
        let portal_unauthenticated = ctx(ADMIN_DEVELOPER_UNAUTHENTICATED, None);
        assert_eq!(
            developer_portal_load(&portal_unauthenticated),
            DeveloperPortalLoad::Unauthenticated
        );
        let rendered = html(&portal_unauthenticated);
        assert!(rendered.contains("data-admin-data-state=\"unauthenticated\""));
        assert!(rendered.contains("Sign in required"));
        assert!(!rendered.contains("data-admin-developer-portal-state"));

        let portal_unauthorized = ctx(ADMIN_DEVELOPER_UNAUTHORIZED, None);
        assert_eq!(
            developer_portal_load(&portal_unauthorized),
            DeveloperPortalLoad::Unauthorized
        );
        assert!(html(&portal_unauthorized).contains("data-admin-data-state=\"unauthorized\""));

        let mut create_unauthenticated = ctx(ADMIN_DEVELOPER_UNAVAILABLE, None);
        create_unauthenticated.path = "/developer-portal/api-keys/create".to_string();
        create_unauthenticated.params.insert(
            ADMIN_DEVELOPER_CREATE_STATE_PARAM.to_string(),
            ADMIN_DEVELOPER_CREATE_UNAUTHENTICATED.to_string(),
        );
        assert!(matches!(
            developer_create_load(&create_unauthenticated),
            DeveloperCreateLoad::Unauthenticated
        ));
        let rendered = dioxus_ssr::render_element(render_create_key(&create_unauthenticated).1);
        assert!(rendered.contains("data-admin-data-state=\"unauthenticated\""));

        let mut create_unauthorized = ctx(ADMIN_DEVELOPER_UNAVAILABLE, None);
        create_unauthorized.path = "/developer-portal/api-keys/create".to_string();
        create_unauthorized.params.insert(
            ADMIN_DEVELOPER_CREATE_STATE_PARAM.to_string(),
            ADMIN_DEVELOPER_CREATE_UNAUTHORIZED.to_string(),
        );
        assert!(matches!(
            developer_create_load(&create_unauthorized),
            DeveloperCreateLoad::Unauthorized
        ));
        let rendered = dioxus_ssr::render_element(render_create_key(&create_unauthorized).1);
        assert!(rendered.contains("Session expired"));
    }

    #[test]
    fn create_key_route_is_form_or_secret_once_and_never_a_read_projection() {
        let mut create = ctx(ADMIN_DEVELOPER_UNAVAILABLE, None);
        create.path = "/developer-portal/api-keys/create".to_string();
        create.params.insert(
            ADMIN_DEVELOPER_CREATE_STATE_PARAM.to_string(),
            ADMIN_DEVELOPER_CREATE_FORM.to_string(),
        );

        let rendered = dioxus_ssr::render_element(render_create_key(&create).1);

        assert!(rendered.contains("data-admin-developer-create-state=\"form\""));
        assert!(rendered.contains("Create API Key"));
        assert!(rendered.contains("data-admin-developer-create-submit=\"bff\""));
        for field in [
            "client_name",
            "client_contact_email",
            "client_description",
            "expires_at",
            "ip_restrictions",
        ] {
            assert!(rendered.contains(&format!("name=\"{field}\"")));
        }
        assert!(!rendered.contains("name=\"wallet_address\""));
        assert!(!rendered.contains("name=\"permissions\""));

        let projection = AdminDeveloperSecretOnceProjection {
            api_key: key(),
            secret: "epsx_live_secret".to_string(),
        };
        create.params.insert(
            ADMIN_DEVELOPER_CREATE_STATE_PARAM.to_string(),
            ADMIN_DEVELOPER_CREATE_CREATED.to_string(),
        );
        create.params.insert(
            ADMIN_DEVELOPER_CREATE_DATA_PARAM.to_string(),
            serde_json::to_string(&projection).unwrap(),
        );
        let rendered = dioxus_ssr::render_element(render_create_key(&create).1);

        assert!(rendered.contains("data-secret-once=\"true\""));
        assert!(rendered.contains("epsx_live_secret"));
        assert!(rendered.contains("not included in list, read, or audit projections"));
        assert!(!rendered.contains("full_key"));

        let mut value = serde_json::to_value(&projection).unwrap();
        value["api_key"]["full_key"] = serde_json::json!("epsx_leaked");
        create.params.insert(
            ADMIN_DEVELOPER_CREATE_DATA_PARAM.to_string(),
            value.to_string(),
        );
        assert!(dioxus_ssr::render_element(render_create_key(&create).1)
            .contains("data-admin-developer-create-state=\"malformed\""));
    }
}

//! Backend-owned Developer Portal views.

use crate::auth::AuthGate;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use uuid::Uuid;

use super::{PageContext, PageMeta};

pub const DEVELOPER_DATA_PARAM: &str = "data_developer";
pub const DEVELOPER_STATE_PARAM: &str = "data_developer_state";
pub const DEVELOPER_USAGE_DATA_PARAM: &str = "data_developer_usage";
pub const DEVELOPER_USAGE_STATE_PARAM: &str = "data_developer_usage_state";
pub const DEVELOPER_OPENAPI_DATA_PARAM: &str = "data_developer_openapi";
pub const DEVELOPER_OPENAPI_STATE_PARAM: &str = "data_developer_openapi_state";

pub const LOAD_READY: &str = "ready";
pub const LOAD_EMPTY: &str = "empty";
pub const LOAD_UNAVAILABLE: &str = "unavailable";
pub const LOAD_MALFORMED: &str = "malformed";
pub const LOAD_FORBIDDEN: &str = "forbidden";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeveloperContractError;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DeveloperPlan {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub expires_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DeveloperRateLimits {
    pub per_minute: u32,
    pub per_hour: u32,
    pub per_day: u32,
    pub burst: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DeveloperEntitlement {
    pub plans: Vec<DeveloperPlan>,
    pub assignable_scopes: Vec<String>,
    pub rate_limits: DeveloperRateLimits,
    pub can_read: bool,
    pub can_write: bool,
    pub has_active_api_entitlement: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DeveloperApiKey {
    pub id: Uuid,
    pub key_prefix: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub scopes: Vec<String>,
    pub total_requests: i64,
    pub expires_at: Option<String>,
    pub last_used_at: Option<String>,
    pub created_at: String,
}

impl DeveloperApiKey {
    pub fn validated(self) -> Result<Self, DeveloperContractError> {
        if self.name.is_empty()
            || self.key_prefix.is_empty()
            || !self.key_prefix.ends_with('…')
            || !matches!(self.status.as_str(), "active" | "revoked" | "expired")
            || self.total_requests < 0
            || self.scopes.len() > 100
            || self.scopes.iter().any(|scope| !valid_scope(scope))
            || !valid_timestamp(&self.created_at)
            || self
                .expires_at
                .as_deref()
                .is_some_and(|value| !valid_timestamp(value))
            || self
                .last_used_at
                .as_deref()
                .is_some_and(|value| !valid_timestamp(value))
        {
            return Err(DeveloperContractError);
        }
        Ok(self)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DeveloperDailyUsage {
    pub date: String,
    pub total_requests: i64,
    pub error_requests: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DeveloperEndpointUsage {
    pub endpoint: String,
    pub method: String,
    pub request_count: i64,
    pub error_count: i64,
    pub average_response_time_ms: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DeveloperUsage {
    pub days: i32,
    pub total_requests: i64,
    pub successful_requests: i64,
    pub error_requests: i64,
    pub success_rate: f64,
    pub error_rate: f64,
    pub average_response_time_ms: f64,
    pub daily: Vec<DeveloperDailyUsage>,
    pub top_endpoints: Vec<DeveloperEndpointUsage>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DeveloperOverview {
    pub entitlement: DeveloperEntitlement,
    pub api_keys: Vec<DeveloperApiKey>,
    pub total_api_keys: i64,
    pub usage: DeveloperUsage,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeveloperOperation {
    pub operation_id: String,
    pub method: String,
    pub path: String,
    pub summary: String,
    pub required_scopes: Vec<String>,
    pub api_key_callable: bool,
    pub mutation: bool,
    pub idempotent: bool,
}

fn valid_timestamp(value: &str) -> bool {
    chrono::DateTime::parse_from_rfc3339(value).is_ok()
}

fn valid_scope(scope: &str) -> bool {
    !scope.is_empty()
        && scope.len() <= 255
        && !scope.starts_with("admin:")
        && !scope.chars().any(char::is_control)
}

impl DeveloperOverview {
    pub fn validated(self) -> Result<Self, DeveloperContractError> {
        let scopes = self
            .entitlement
            .assignable_scopes
            .iter()
            .collect::<BTreeSet<_>>();
        if self.total_api_keys < 0
            || self.total_api_keys < self.api_keys.len() as i64
            || self.entitlement.assignable_scopes.len() > 100
            || scopes.len() != self.entitlement.assignable_scopes.len()
            || self
                .entitlement
                .assignable_scopes
                .iter()
                .any(|scope| !valid_scope(scope))
            || self.entitlement.plans.iter().any(|plan| {
                plan.name.is_empty()
                    || plan.slug.is_empty()
                    || plan
                        .expires_at
                        .as_deref()
                        .is_some_and(|value| !valid_timestamp(value))
            })
            || !matches!(self.usage.days, 7 | 30 | 90)
            || self.usage.daily.len() != self.usage.days as usize
            || self.usage.total_requests < 0
            || self.usage.successful_requests < 0
            || self.usage.error_requests < 0
            || self.usage.successful_requests + self.usage.error_requests
                != self.usage.total_requests
            || !self.usage.success_rate.is_finite()
            || !self.usage.error_rate.is_finite()
            || !self.usage.average_response_time_ms.is_finite()
            || self.api_keys.iter().any(|key| {
                key.clone().validated().is_err()
                    || key.scopes.iter().any(|scope| !scopes.contains(scope))
            })
            || self.usage.daily.iter().any(|point| {
                chrono::NaiveDate::parse_from_str(&point.date, "%Y-%m-%d").is_err()
                    || point.total_requests < 0
                    || point.error_requests < 0
                    || point.error_requests > point.total_requests
            })
            || self.usage.top_endpoints.iter().any(|endpoint| {
                endpoint.endpoint.is_empty()
                    || !endpoint.endpoint.starts_with('/')
                    || endpoint.method.is_empty()
                    || endpoint.request_count < 0
                    || endpoint.error_count < 0
                    || endpoint.error_count > endpoint.request_count
                    || !endpoint.average_response_time_ms.is_finite()
            })
        {
            return Err(DeveloperContractError);
        }
        Ok(self)
    }
}

pub fn decode_developer_overview(value: serde_json::Value) -> Option<DeveloperOverview> {
    serde_json::from_value::<DeveloperOverview>(value)
        .ok()?
        .validated()
        .ok()
}

pub fn decode_openapi(value: serde_json::Value) -> Option<Vec<DeveloperOperation>> {
    if value.get("openapi")?.as_str()?.is_empty() {
        return None;
    }
    let paths = value.get("paths")?.as_object()?;
    let mut operations = Vec::new();
    let mut ids = BTreeSet::new();
    for (path, item) in paths {
        if !path.starts_with("/api/") || path.contains("..") {
            return None;
        }
        let item = item.as_object()?;
        for method in ["get", "post", "put", "patch", "delete"] {
            let Some(operation) = item.get(method) else {
                continue;
            };
            let operation_id = operation.get("operationId")?.as_str()?.to_string();
            let summary = operation
                .get("summary")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("API operation")
                .to_string();
            let required_scopes = operation
                .get("x-epsx-required-scopes")?
                .as_array()?
                .iter()
                .map(|scope| scope.as_str().map(str::to_string))
                .collect::<Option<Vec<_>>>()?;
            if !ids.insert(operation_id.clone())
                || required_scopes.iter().any(|scope| !valid_scope(scope))
            {
                return None;
            }
            operations.push(DeveloperOperation {
                operation_id,
                method: method.to_ascii_uppercase(),
                path: path.clone(),
                summary,
                required_scopes,
                api_key_callable: operation.get("x-epsx-api-key-callable")?.as_bool()?,
                mutation: operation.get("x-epsx-mutation")?.as_bool()?,
                idempotent: operation.get("x-epsx-idempotent")?.as_bool()?,
            });
        }
    }
    (!operations.is_empty()).then_some(operations)
}

#[derive(Clone, Debug)]
enum Load<T> {
    Ready(T),
    Empty(T),
    Forbidden,
    Unavailable,
    Malformed,
}

fn overview_load(
    ctx: &PageContext,
    data_param: &str,
    state_param: &str,
) -> Load<DeveloperOverview> {
    let state = ctx.params.get(state_param).map(String::as_str);
    let decoded = ctx
        .params
        .get(data_param)
        .and_then(|raw| serde_json::from_str(raw).ok())
        .and_then(decode_developer_overview);
    match (state, decoded) {
        (Some(LOAD_READY), Some(data)) => Load::Ready(data),
        (Some(LOAD_EMPTY), Some(data)) => Load::Empty(data),
        (Some(LOAD_FORBIDDEN), _) => Load::Forbidden,
        (Some(LOAD_UNAVAILABLE), _) | (None, _) => Load::Unavailable,
        _ => Load::Malformed,
    }
}

#[component]
fn ProblemState(kind: &'static str, title: &'static str, detail: &'static str) -> Element {
    rsx! {
        section {
            class: "rounded-2xl border border-border/30 bg-card p-8 text-center shadow-xl",
            "data-developer-state": kind,
            role: "status",
            h2 { class: "text-xl font-semibold text-foreground", "{title}" }
            p { class: "mt-3 text-sm text-muted-foreground", "{detail}" }
        }
    }
}

fn format_number(value: i64) -> String {
    let chars = value.max(0).to_string().chars().rev().collect::<Vec<_>>();
    chars
        .chunks(3)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(",")
        .chars()
        .rev()
        .collect()
}

#[component]
fn Metric(label: &'static str, value: String, detail: String) -> Element {
    rsx! {
        article { class: "rounded-2xl border border-border/20 bg-card p-5 shadow-lg",
            p { class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground", "{label}" }
            p { class: "mt-2 text-2xl font-bold text-foreground", "{value}" }
            p { class: "mt-1 text-xs text-muted-foreground", "{detail}" }
        }
    }
}

#[component]
fn OverviewReady(data: DeveloperOverview) -> Element {
    let entitlement = data.entitlement.clone();
    let plan_names = if entitlement.plans.is_empty() {
        "No active API plan".to_string()
    } else {
        entitlement
            .plans
            .iter()
            .map(|plan| plan.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    };
    rsx! {
        div { class: "space-y-6", "data-developer-state": "ready",
            div { class: "grid gap-4 sm:grid-cols-2 xl:grid-cols-4",
                Metric { label: "API access", value: if entitlement.has_active_api_entitlement { "Active".to_string() } else { "Inactive".to_string() }, detail: plan_names }
                Metric { label: "Rate limit", value: format!("{}/min", entitlement.rate_limits.per_minute), detail: format!("{} requests/day", format_number(i64::from(entitlement.rate_limits.per_day))) }
                Metric { label: "30-day usage", value: format_number(data.usage.total_requests), detail: format!("{:.1}% success", data.usage.success_rate) }
                Metric { label: "API keys", value: data.total_api_keys.to_string(), detail: "Owner-scoped, secrets redacted".to_string() }
            }

            if entitlement.can_write && entitlement.has_active_api_entitlement {
                section { class: "rounded-2xl border border-border/20 bg-card p-6 shadow-xl",
                    h2 { class: "text-lg font-semibold text-foreground", "Create API key" }
                    p { class: "mt-1 text-sm text-muted-foreground", "Choose only the scopes this integration needs. The secret appears once." }
                    form { class: "mt-5 grid gap-4", "data-developer-create-form": "true", action: "/developer/keys/create", method: "post",
                        input { r#type: "hidden", name: "idempotency_key", value: format!("developer.create.{}", Uuid::new_v4()) }
                        label { class: "grid gap-1 text-sm text-foreground", "Name",
                            input { class: "input", name: "name", maxlength: "255", required: true, autocomplete: "off" }
                        }
                        label { class: "grid gap-1 text-sm text-foreground", "Description",
                            textarea { class: "input min-h-24", name: "description", maxlength: "2000" }
                        }
                        fieldset { class: "grid gap-2", legend { class: "text-sm font-medium text-foreground", "Scopes" }
                            for scope in entitlement.assignable_scopes.iter() {
                                label { class: "flex items-center gap-2 rounded-xl border border-border/20 p-3 text-sm",
                                    input { r#type: "checkbox", name: "scopes", value: "{scope}" }
                                    code { "{scope}" }
                                }
                            }
                        }
                        label { class: "grid gap-1 text-sm text-foreground", "Expires at (optional)",
                            input { class: "input font-mono", r#type: "text", name: "expires_at", placeholder: "2030-01-01T00:00:00Z", inputmode: "text" }
                        }
                        button { class: "btn btn-primary justify-self-start", r#type: "submit", "data-developer-create": "true", "Create key" }
                    }
                    div { id: "developer-secret-once", class: "mt-5 rounded-xl border border-amber-400/30 bg-amber-400/10 p-4", hidden: true, "data-developer-secret-panel": "true", role: "status",
                        p { class: "text-sm font-semibold text-foreground", "Save this secret now. It will not be shown again." }
                        code { id: "developer-secret-value", class: "mt-2 block break-all font-mono text-sm" }
                        button { id: "developer-secret-copy", class: "btn btn-outline mt-3", r#type: "button", "data-epsx-action": "copy", "Copy secret" }
                    }
                }
            }

            section { class: "rounded-2xl border border-border/20 bg-card p-6 shadow-xl",
                div { class: "flex flex-wrap items-center justify-between gap-3",
                    h2 { class: "text-lg font-semibold text-foreground", "Your API keys" }
                    a { class: "btn btn-outline", href: "/developer/usage", "View usage" }
                }
                if data.api_keys.is_empty() {
                    p { class: "py-12 text-center text-muted-foreground", "No API keys yet." }
                } else {
                    div { class: "mt-5 grid gap-3",
                        for key in data.api_keys.iter() {
                            article { class: "rounded-xl border border-border/20 p-4", "data-api-key-id": "{key.id}",
                                div { class: "flex flex-wrap items-start justify-between gap-3",
                                    div {
                                        h3 { class: "font-semibold text-foreground", "{key.name}" }
                                        code { class: "mt-1 block text-sm text-muted-foreground", "{key.key_prefix}" }
                                    }
                                    span { class: "rounded-full border px-2 py-1 text-xs", "{key.status}" }
                                }
                                if let Some(description) = &key.description { p { class: "mt-3 text-sm text-muted-foreground", "{description}" } }
                                div { class: "mt-3 flex flex-wrap gap-2",
                                    for scope in key.scopes.iter() { code { class: "rounded bg-background px-2 py-1 text-xs", "{scope}" } }
                                }
                                div { class: "mt-4 flex flex-wrap items-center justify-between gap-3 text-xs text-muted-foreground",
                                    span { "{format_number(key.total_requests)} requests" }
                                    if key.status == "active" && entitlement.can_write {
                                        form { "data-developer-revoke-form": "true", action: format!("/developer/keys/{}/revoke", key.id), method: "post",
                                            input { r#type: "hidden", name: "idempotency_key", value: format!("developer.revoke.{}", Uuid::new_v4()) }
                                            input { r#type: "hidden", name: "reason", value: "Revoked from Developer Portal" }
                                            label { class: "mr-2 inline-flex items-center gap-1", input { r#type: "checkbox", name: "confirm_revoke", value: "yes", required: true } "Confirm" }
                                            button { class: "btn btn-outline text-red-400", r#type: "submit", "data-developer-revoke": "true", "data-key-id": "{key.id}", "Revoke" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn OverviewBody(ctx: PageContext) -> Element {
    rsx! { MainLayout { ctx: ctx.clone(), AuthGate { user: ctx.user.clone(), feature: Some("the developer portal".to_string()), return_url: Some(ctx.path.clone()), wallet_connected: ctx.wallet.address.is_some(),
        div { class: "container page-content space-y-6",
            PageHeader { title: "Developer portal".to_string(), description: Some("Manage scoped API credentials backed by your live plan entitlement.".to_string()), icon: Some("code".to_string()) }
            match overview_load(&ctx, DEVELOPER_DATA_PARAM, DEVELOPER_STATE_PARAM) {
                Load::Ready(data) | Load::Empty(data) => rsx! { OverviewReady { data } },
                Load::Forbidden => rsx! { ProblemState { kind: LOAD_FORBIDDEN, title: "API access required", detail: "Your current plan does not include epsx:api:read." } },
                Load::Unavailable => rsx! { ProblemState { kind: LOAD_UNAVAILABLE, title: "Developer portal unavailable", detail: "The authoritative backend contract could not be reached." } },
                Load::Malformed => rsx! { ProblemState { kind: LOAD_MALFORMED, title: "Developer data rejected", detail: "The backend response did not match the developer contract." } },
            }
        }
    } } }
}

#[component]
fn UsageReady(data: DeveloperOverview) -> Element {
    let maximum = data
        .usage
        .daily
        .iter()
        .map(|point| point.total_requests)
        .max()
        .unwrap_or(0)
        .max(1);
    rsx! {
        div { class: "space-y-6", "data-developer-usage-state": "ready",
            nav { class: "flex gap-2", "aria-label": "Usage range",
                for days in [7, 30, 90] {
                    a { class: if data.usage.days == days { "btn btn-primary" } else { "btn btn-outline" }, href: format!("/developer/usage?days={days}"), "{days} days" }
                }
            }
            div { class: "grid gap-4 sm:grid-cols-2 xl:grid-cols-4",
                Metric { label: "Requests", value: format_number(data.usage.total_requests), detail: format!("Last {} days", data.usage.days) }
                Metric { label: "Success rate", value: format!("{:.1}%", data.usage.success_rate), detail: format!("{} successful", format_number(data.usage.successful_requests)) }
                Metric { label: "Errors", value: format_number(data.usage.error_requests), detail: format!("{:.1}% error rate", data.usage.error_rate) }
                Metric { label: "Average latency", value: format!("{:.0} ms", data.usage.average_response_time_ms), detail: "Completed API-key requests".to_string() }
            }
            section { class: "rounded-2xl border border-border/20 bg-card p-6 shadow-xl",
                h2 { class: "text-lg font-semibold text-foreground", "Daily requests" }
                if data.usage.total_requests == 0 {
                    p { class: "py-12 text-center text-muted-foreground", "No API-key requests in this period." }
                } else {
                    div { class: "mt-5 flex h-56 items-end gap-1 overflow-x-auto", role: "img", "aria-label": "Daily API request chart",
                        for point in data.usage.daily.iter() {
                            div { class: "group flex min-w-2 flex-1 flex-col items-center justify-end", title: format!("{}: {} requests", point.date, point.total_requests),
                                div { class: "w-full rounded-t bg-purple-500", style: format!("height: {}%", (point.total_requests * 100 / maximum).max(2)) }
                            }
                        }
                    }
                }
            }
            section { class: "overflow-hidden rounded-2xl border border-border/20 bg-card shadow-xl",
                h2 { class: "p-6 text-lg font-semibold text-foreground", "Top endpoints" }
                if data.usage.top_endpoints.is_empty() { p { class: "px-6 pb-8 text-muted-foreground", "No endpoint activity in this period." } }
                else { div { class: "overflow-x-auto", table { class: "w-full text-sm",
                    thead { tr { class: "border-t border-b border-border/20 text-left text-muted-foreground", th { class: "p-3", "Method" } th { class: "p-3", "Endpoint" } th { class: "p-3", "Requests" } th { class: "p-3", "Errors" } th { class: "p-3", "Avg latency" } } }
                    tbody { for endpoint in data.usage.top_endpoints.iter() { tr { class: "border-b border-border/10", td { class: "p-3 font-mono", "{endpoint.method}" } td { class: "p-3 font-mono", "{endpoint.endpoint}" } td { class: "p-3", "{format_number(endpoint.request_count)}" } td { class: "p-3", "{format_number(endpoint.error_count)}" } td { class: "p-3", "{endpoint.average_response_time_ms:.0} ms" } } } }
                } } }
            }
        }
    }
}

#[component]
fn UsageBody(ctx: PageContext) -> Element {
    rsx! { MainLayout { ctx: ctx.clone(), AuthGate { user: ctx.user.clone(), feature: Some("API usage".to_string()), return_url: Some(ctx.path.clone()), wallet_connected: ctx.wallet.address.is_some(),
        div { class: "container page-content space-y-6",
            PageHeader { title: "API usage".to_string(), description: Some("Request volume, reliability, and endpoint activity from real API-key logs.".to_string()), icon: Some("chart-line".to_string()) }
            match overview_load(&ctx, DEVELOPER_USAGE_DATA_PARAM, DEVELOPER_USAGE_STATE_PARAM) {
                Load::Ready(data) | Load::Empty(data) => rsx! { UsageReady { data } },
                Load::Forbidden => rsx! { ProblemState { kind: LOAD_FORBIDDEN, title: "API access required", detail: "Your current plan does not include usage reporting." } },
                Load::Unavailable => rsx! { ProblemState { kind: LOAD_UNAVAILABLE, title: "Usage unavailable", detail: "Usage analytics could not be reached." } },
                Load::Malformed => rsx! { ProblemState { kind: LOAD_MALFORMED, title: "Usage data rejected", detail: "The analytics response did not match the usage contract." } },
            }
        }
    } } }
}

#[component]
fn DocsBody(ctx: PageContext) -> Element {
    let state = ctx
        .params
        .get(DEVELOPER_OPENAPI_STATE_PARAM)
        .map(String::as_str);
    let operations = ctx
        .params
        .get(DEVELOPER_OPENAPI_DATA_PARAM)
        .and_then(|raw| serde_json::from_str(raw).ok())
        .and_then(decode_openapi);
    rsx! { MainLayout { ctx: ctx.clone(),
        div { class: "container page-content space-y-6",
            PageHeader { title: "API documentation".to_string(), description: Some("Generated from the backend operation registry.".to_string()), icon: Some("book-open".to_string()) }
            if state == Some(LOAD_READY) && operations.is_some() {
                div { class: "space-y-6", "data-developer-docs-state": "ready",
                    section { class: "rounded-2xl border border-border/20 bg-card p-6 shadow-xl",
                        label { class: "grid gap-2 text-sm font-medium text-foreground", "API key for Try It",
                            input { id: "developer-try-api-key", class: "input font-mono", r#type: "password", autocomplete: "off", spellcheck: "false", placeholder: "epsx_…", "data-developer-api-key-memory": "true" }
                        }
                        p { class: "mt-2 text-xs text-muted-foreground", "Kept only in this tab's memory and cleared on reload or close." }
                    }
                    for operation in operations.unwrap_or_default() {
                        article { class: "rounded-2xl border border-border/20 bg-card p-6 shadow-xl", id: format!("operation-{}", operation.operation_id),
                            div { class: "flex flex-wrap items-center gap-3",
                                span { class: "rounded-lg bg-purple-500/15 px-2 py-1 text-xs font-bold text-purple-700 dark:text-purple-300", "{operation.method}" }
                                code { class: "font-mono text-sm text-foreground", "{operation.path}" }
                            }
                            h2 { class: "mt-3 text-lg font-semibold text-foreground", "{operation.summary}" }
                            div { class: "mt-3 flex flex-wrap gap-2", for scope in operation.required_scopes.iter() { code { class: "rounded bg-background px-2 py-1 text-xs", "{scope}" } } }
                            if operation.api_key_callable {
                                div { class: "mt-5 grid gap-3",
                                    label { class: "grid gap-1 text-xs text-muted-foreground", "Query string (optional)", input { class: "input font-mono", "data-try-query": "true", placeholder: "country=US&limit=20" } }
                                    if operation.mutation { label { class: "grid gap-1 text-xs text-muted-foreground", "JSON body", textarea { class: "input min-h-28 font-mono", "data-try-body": "true", value: "{{}}" } } }
                                    button { class: "btn btn-primary justify-self-start", r#type: "button", "data-developer-try": "true", "data-operation-id": "{operation.operation_id}", "data-operation-mutation": if operation.mutation { "true" } else { "false" }, "Try It" }
                                    pre { class: "max-h-96 overflow-auto rounded-xl bg-slate-950 p-4 text-xs text-slate-200", "data-try-response": "true", hidden: true }
                                }
                            } else {
                                p { class: "mt-4 text-sm text-muted-foreground", "Browser-session operation; Try It is disabled for API keys." }
                            }
                        }
                    }
                }
            } else if state == Some(LOAD_MALFORMED) {
                ProblemState { kind: LOAD_MALFORMED, title: "API specification rejected", detail: "The OpenAPI document did not match the operation registry contract. Try It is disabled." }
            } else {
                ProblemState { kind: LOAD_UNAVAILABLE, title: "API specification unavailable", detail: "The backend OpenAPI document could not be loaded. Try It is disabled." }
            }
        }
    } }
}

pub fn render_overview(ctx: &PageContext) -> (PageMeta, Element) {
    (
        PageMeta::app("Developer"),
        rsx! { OverviewBody { ctx: ctx.clone() } },
    )
}

pub fn render_usage(ctx: &PageContext) -> (PageMeta, Element) {
    (
        PageMeta::app("API usage"),
        rsx! { UsageBody { ctx: ctx.clone() } },
    )
}

pub fn render_docs(ctx: &PageContext) -> (PageMeta, Element) {
    (
        PageMeta::app("API documentation"),
        rsx! { DocsBody { ctx: ctx.clone() } },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openapi_decoder_requires_safe_registry_extensions() {
        let valid = serde_json::json!({
            "openapi": "3.1.0",
            "paths": {"/api/analytics/rankings": {"get": {
                "operationId": "getRankings",
                "summary": "Rankings",
                "x-epsx-required-scopes": ["epsx:analytics:view"],
                "x-epsx-api-key-callable": true,
                "x-epsx-mutation": false,
                "x-epsx-idempotent": false
            }}}
        });
        assert_eq!(decode_openapi(valid).unwrap().len(), 1);
        let unsafe_spec = serde_json::json!({
            "openapi": "3.1.0",
            "paths": {"https://evil.test/": {"get": {}}}
        });
        assert!(decode_openapi(unsafe_spec).is_none());
    }

    #[test]
    fn overview_decoder_rejects_plaintext_and_admin_shaped_fields() {
        let value = serde_json::json!({
            "entitlement": {
                "plans": [], "assignable_scopes": ["admin:users:manage"],
                "rate_limits": {"per_minute": 0, "per_hour": 0, "per_day": 0, "burst": 0},
                "can_read": true, "can_write": true, "has_active_api_entitlement": true
            },
            "api_keys": [], "total_api_keys": 0,
            "usage": {"days": 7, "total_requests": 0, "successful_requests": 0,
                "error_requests": 0, "success_rate": 0.0, "error_rate": 0.0,
                "average_response_time_ms": 0.0, "daily": [], "top_endpoints": []}
        });
        assert!(decode_developer_overview(value).is_none());

        let key_with_secret = serde_json::json!({
            "id": Uuid::nil(),
            "key_prefix": "epsx_deadbee…",
            "name": "integration",
            "description": null,
            "status": "active",
            "scopes": ["epsx:analytics:view"],
            "total_requests": 0,
            "expires_at": null,
            "last_used_at": null,
            "created_at": "2026-08-21T00:00:00Z",
            "full_key": format!("epsx_{}", "a".repeat(64))
        });
        assert!(serde_json::from_value::<DeveloperApiKey>(key_with_secret).is_err());
    }
}

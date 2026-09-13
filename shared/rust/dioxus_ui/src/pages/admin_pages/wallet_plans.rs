//! Backend-authoritative read projections for wallet plan list/detail pages.
//!
//! Only strict redacted plan fields reach this shared Dioxus layer. Merchant
//! identity, creation timestamps, correlation IDs, and every plan mutation are
//! intentionally absent.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use super::super::{PageContext, PageMeta};
use super::wallet_hub::WalletManagementHub;
use crate::auth::AuthGate;
use crate::components::admin::data_state_banner::{AdminDataState, AdminDataStateBanner};
use crate::components::admin::page_layout::{PageGradient, PageHeader};
use crate::primitives::Icon;

const PLANS_PATH: &str = "/wallet-management/access/plans";
const MAX_PLANS: usize = 100;
const MAX_PLAN_NAME_CHARS: usize = 100;
const MAX_DESCRIPTION_CHARS: usize = 2_000;
const MAX_AMOUNT_CHARS: usize = 78;
const MAX_CURRENCY_CHARS: usize = 10;
const MAX_CHAIN_ID_CHARS: usize = 10;
const MAX_OFFSET: i64 = 10_000_000;

pub const ADMIN_PLANS_DATA_PARAM: &str = "data_admin_plans";
pub const ADMIN_PLANS_STATE_PARAM: &str = "data_admin_plans_state";
pub const ADMIN_PLAN_DETAIL_DATA_PARAM: &str = "data_admin_plan_detail";
pub const ADMIN_PLAN_DETAIL_STATE_PARAM: &str = "data_admin_plan_detail_state";

pub const ADMIN_PLANS_READY: &str = "ready";
pub const ADMIN_PLANS_EMPTY: &str = "empty";
pub const ADMIN_PLANS_FORBIDDEN: &str = "forbidden";
pub const ADMIN_PLANS_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_PLANS_MALFORMED: &str = "malformed";
pub const ADMIN_PLANS_UNAUTHENTICATED: &str = "unauthenticated";
pub const ADMIN_PLANS_UNAUTHORIZED: &str = "unauthorized";
pub const ADMIN_PLAN_DETAIL_UNAUTHENTICATED: &str = "unauthenticated";
pub const ADMIN_PLAN_DETAIL_UNAUTHORIZED: &str = "unauthorized";

/// Redacted fields from the service AdminPlan DTO. Merchant identity and
/// timestamps are deliberately not transported into PageContext.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminPlanProjection {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub amount: String,
    pub currency: String,
    pub chain_id: String,
    pub interval: i32,
    pub active: Option<bool>,
    pub version: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminPlanListProjection {
    pub items: Vec<AdminPlanProjection>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

pub fn decode_admin_plan_projection(value: serde_json::Value) -> Option<AdminPlanProjection> {
    let projection: AdminPlanProjection = serde_json::from_value(value).ok()?;
    projection.is_well_formed().then_some(projection)
}

pub fn decode_admin_plan_list_projection(
    value: serde_json::Value,
) -> Option<AdminPlanListProjection> {
    let projection: AdminPlanListProjection = serde_json::from_value(value).ok()?;
    if projection.items.len() > MAX_PLANS
        || !(1..=100).contains(&projection.limit)
        || !(0..=MAX_OFFSET).contains(&projection.offset)
        || projection.total < 0
        || projection.items.len() > usize::try_from(projection.limit).ok()?
        || projection
            .offset
            .checked_add(i64::try_from(projection.items.len()).ok()?)?
            > projection.total
        || projection.items.iter().any(|item| !item.is_well_formed())
    {
        return None;
    }
    Some(projection)
}

impl AdminPlanProjection {
    fn is_well_formed(&self) -> bool {
        valid_uuid(&self.id)
            && valid_text(&self.name, MAX_PLAN_NAME_CHARS)
            && self
                .description
                .as_deref()
                .is_none_or(|value| valid_optional_text(value, MAX_DESCRIPTION_CHARS))
            && valid_amount(&self.amount)
            && valid_currency(&self.currency)
            && valid_chain_id(&self.chain_id)
            && (1..=366).contains(&self.interval)
            && self.version >= 0
    }
}

fn valid_text(value: &str, max_chars: usize) -> bool {
    !value.trim().is_empty()
        && value.trim() == value
        && value.chars().count() <= max_chars
        && !value.chars().any(char::is_control)
}

fn valid_optional_text(value: &str, max_chars: usize) -> bool {
    value.chars().count() <= max_chars && !value.chars().any(char::is_control)
}

fn valid_amount(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_AMOUNT_CHARS
        && value.bytes().all(|byte| byte.is_ascii_digit())
}

fn valid_currency(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_CURRENCY_CHARS
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
}

fn valid_chain_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_CHAIN_ID_CHARS
        && value.bytes().all(|byte| byte.is_ascii_digit())
}

fn valid_uuid(value: &str) -> bool {
    super::wallet_access::valid_uuid(value)
}

fn canonical_plan_id(value: &str) -> Option<String> {
    valid_uuid(value).then(|| value.to_ascii_lowercase())
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PlansLoad {
    Ready(AdminPlanListProjection),
    Empty,
    Forbidden,
    Unavailable,
    Malformed,
    Unauthenticated,
    Unauthorized,
}

fn plans_load(ctx: &PageContext) -> PlansLoad {
    match ctx.params.get(ADMIN_PLANS_STATE_PARAM).map(String::as_str) {
        Some(ADMIN_PLANS_READY) | Some(ADMIN_PLANS_EMPTY) => {
            let Some(raw) = ctx.params.get(ADMIN_PLANS_DATA_PARAM) else {
                return PlansLoad::Malformed;
            };
            let Some(projection) = serde_json::from_str(raw)
                .ok()
                .and_then(decode_admin_plan_list_projection)
            else {
                return PlansLoad::Malformed;
            };
            match (
                ctx.params.get(ADMIN_PLANS_STATE_PARAM).map(String::as_str),
                projection.items.is_empty(),
                projection.total,
            ) {
                (Some(ADMIN_PLANS_READY), false, _) => PlansLoad::Ready(projection),
                (Some(ADMIN_PLANS_READY), true, total) if total > 0 => PlansLoad::Ready(projection),
                (Some(ADMIN_PLANS_EMPTY), true, 0) => PlansLoad::Empty,
                _ => PlansLoad::Malformed,
            }
        }
        Some(ADMIN_PLANS_FORBIDDEN) => PlansLoad::Forbidden,
        Some(ADMIN_PLANS_MALFORMED) => PlansLoad::Malformed,
        Some(ADMIN_PLANS_UNAUTHENTICATED) => PlansLoad::Unauthenticated,
        Some(ADMIN_PLANS_UNAUTHORIZED) => PlansLoad::Unauthorized,
        Some(ADMIN_PLANS_UNAVAILABLE) | None => PlansLoad::Unavailable,
        Some(_) => PlansLoad::Malformed,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PlanDetailLoad {
    Ready(AdminPlanProjection),
    Forbidden,
    Unavailable,
    Malformed,
    Unauthenticated,
    Unauthorized,
}

fn plan_detail_load(ctx: &PageContext) -> PlanDetailLoad {
    let Some(route_id) = ctx
        .params
        .get("planId")
        .and_then(|value| canonical_plan_id(value))
    else {
        return PlanDetailLoad::Malformed;
    };

    match ctx
        .params
        .get(ADMIN_PLAN_DETAIL_STATE_PARAM)
        .map(String::as_str)
    {
        Some(ADMIN_PLANS_READY) => {
            let Some(raw) = ctx.params.get(ADMIN_PLAN_DETAIL_DATA_PARAM) else {
                return PlanDetailLoad::Malformed;
            };
            let Some(projection) = serde_json::from_str(raw)
                .ok()
                .and_then(decode_admin_plan_projection)
            else {
                return PlanDetailLoad::Malformed;
            };
            if canonical_plan_id(&projection.id) == Some(route_id) {
                PlanDetailLoad::Ready(projection)
            } else {
                PlanDetailLoad::Malformed
            }
        }
        Some(ADMIN_PLANS_FORBIDDEN) => PlanDetailLoad::Forbidden,
        Some(ADMIN_PLANS_MALFORMED) => PlanDetailLoad::Malformed,
        Some(ADMIN_PLAN_DETAIL_UNAUTHENTICATED) => PlanDetailLoad::Unauthenticated,
        Some(ADMIN_PLAN_DETAIL_UNAUTHORIZED) => PlanDetailLoad::Unauthorized,
        Some(ADMIN_PLANS_UNAVAILABLE) | None => PlanDetailLoad::Unavailable,
        Some(_) => PlanDetailLoad::Malformed,
    }
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Wallet plans");
    (
        meta,
        rsx! {
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("the wallet plan workspace".to_string()),
                return_url: Some(PLANS_PATH.to_string()),
                RenderPlanList { ctx: ctx.clone() }
            }
        },
    )
}

pub fn render_plans(ctx: &PageContext) -> (PageMeta, Element) {
    render(ctx)
}

pub fn render_editor(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Wallet plan detail");
    (
        meta,
        rsx! {
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("the wallet plan workspace".to_string()),
                return_url: Some(PLANS_PATH.to_string()),
                RenderPlanDetail { ctx: ctx.clone() }
            }
        },
    )
}

#[component]
fn RenderPlanList(ctx: PageContext) -> Element {
    let mutation = match ctx.query_param("mutation").as_deref() {
        Some("success") | Some("conflict") | Some("forbidden") | Some("unavailable")
        | Some("malformed") => ctx.query_param("mutation"),
        _ => None,
    };
    match plans_load(&ctx) {
        PlansLoad::Ready(projection) => rsx! { PlanListReady { ctx, projection, mutation } },
        PlansLoad::Empty => rsx! {
            WalletManagementHub { ctx: ctx.clone(),
                PageHeader {
                    title: "Wallet plans".to_string(),
                    subtitle: Some("Review backend-authoritative plan definitions".to_string()),
                    icon: Some("layers".to_string()),
                    gradient: Some(PageGradient::Purple),
                    centered: Some(false),
                    extra_actions: None,
                    class_name: None,
                }
                section {
                    class: "rounded-2xl border border-border/30 bg-card p-10 text-center shadow-xl",
                    role: "status",
                    "data-admin-wallet-plans-state": ADMIN_PLANS_EMPTY,
                    h2 { class: "text-xl font-semibold text-foreground", "No wallet plans returned" }
                    p { class: "mx-auto mt-2 max-w-xl text-sm leading-6 text-muted-foreground",
                        "The backend returned an authoritative empty plan projection. Plan mutations are not offered."
                    }
                }
            }
        },
        PlansLoad::Forbidden => plan_problem_element(ctx, ADMIN_PLANS_FORBIDDEN, "Wallet plan access was denied", "The backend did not authorize this session to read plan definitions."),
        PlansLoad::Unavailable => plan_problem_element(ctx, ADMIN_PLANS_UNAVAILABLE, "Wallet plans are unavailable", "The subscription backend could not provide an authoritative plan response. No plans are being shown."),
        PlansLoad::Malformed => plan_problem_element(ctx, ADMIN_PLANS_MALFORMED, "Wallet plan data could not be verified", "The backend response did not match the strict redacted plan contract. No plans are being shown."),
        PlansLoad::Unauthenticated => plan_banner_element(ctx, AdminDataState::Unauthenticated),
        PlansLoad::Unauthorized => plan_banner_element(ctx, AdminDataState::Unauthorized),
    }
}

#[component]
fn PlanListReady(
    ctx: PageContext,
    projection: AdminPlanListProjection,
    mutation: Option<String>,
) -> Element {
    rsx! {
        WalletManagementHub { ctx,
            PageHeader {
                title: "Wallet plans".to_string(),
                subtitle: Some("Review backend-authoritative plan definitions".to_string()),
                icon: Some("layers".to_string()),
                gradient: Some(PageGradient::Purple),
                centered: Some(false),
                extra_actions: None,
                class_name: None,
            }
            section {
                class: "overflow-hidden rounded-2xl border border-border/30 bg-card shadow-xl",
                aria_labelledby: "admin-wallet-plans-title",
                "data-admin-wallet-plans-state": ADMIN_PLANS_READY,
                div { class: "h-1 bg-gradient-to-r from-[#7645d9] via-[#1fc7d4] to-[#ed4b9e]", aria_hidden: "true" }
                div { class: "p-5 sm:p-6",
                    h2 { id: "admin-wallet-plans-title", class: "text-lg font-semibold text-foreground", "Plan definitions" }
                    p { class: "mt-1 text-sm leading-6 text-muted-foreground",
                        "{projection.total} authoritative records in this bounded response. No plan operations are available."
                    }
                    if let Some(state) = mutation {
                        p { class: "mt-3 rounded-lg border border-amber-500/30 bg-amber-500/5 p-3 text-sm", role: if state == "forbidden" { "alert" } else { "status" },
                            "data-admin-wallet-plan-mutation-state": state,
                            "Plan mutation: {state}"
                        }
                    }
                }
                form { method: "post", action: PLANS_PATH, class: "grid gap-3 border-t border-border/30 p-5 sm:grid-cols-2 lg:grid-cols-4",
                    input { r#type: "hidden", name: "operation", value: "plan_create" }
                    input { r#type: "hidden", name: "idempotency_key", value: format!("admin.plan.create.{}", uuid::Uuid::new_v4()) }
                    input { class: "input input-bordered", name: "merchant_id", maxlength: 36, placeholder: "Merchant UUID", required: true }
                    input { class: "input input-bordered", name: "name", maxlength: 100, placeholder: "Plan name", required: true }
                    input { class: "input input-bordered", name: "amount", maxlength: 78, placeholder: "Amount minor units", required: true }
                    input { class: "input input-bordered", name: "currency", maxlength: 10, placeholder: "Currency", required: true }
                    input { class: "input input-bordered", name: "chain_id", maxlength: 10, placeholder: "Chain ID", required: true }
                    input { class: "input input-bordered", name: "interval", r#type: "number", min: 1, max: 366, placeholder: "Interval days", required: true }
                    input { class: "input input-bordered", name: "description", maxlength: 2000, placeholder: "Description" }
                    button { r#type: "submit", class: "btn btn-primary", "Create plan" }
                }
                if projection.items.is_empty() {
                    div { class: "border-t border-border/30 p-8 text-center", role: "status",
                        "No plans are present on this bounded page; additional continuation is unavailable."
                    }
                } else {
                    ul { class: "divide-y divide-border/30 border-t border-border/30", aria_label: "Wallet plans",
                        for plan in projection.items {
                            PlanListRow { plan }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PlanListRow(plan: AdminPlanProjection) -> Element {
    let href = plan_href(&plan.id);
    let state = plan_state_label(plan.active);

    rsx! {
        li { class: "grid gap-4 p-5 sm:grid-cols-[minmax(0,1fr)_auto_auto] sm:items-center",
            div {
                p { class: "font-semibold text-foreground", "{plan.name}" }
                if let Some(description) = plan.description {
                    p { class: "mt-1 max-w-3xl text-sm leading-6 text-muted-foreground", "{description}" }
                }
                p { class: "mt-1 text-xs text-muted-foreground", "Chain {plan.chain_id} · every {plan.interval} day(s)" }
            }
            div { class: "sm:text-right",
                p { class: "text-sm font-semibold text-foreground", "{plan.amount} {plan.currency}" }
                p { class: "mt-1 text-xs text-muted-foreground", "{state}" }
            }
            a { class: "btn btn-sm btn-outline", href, "Read detail" }
        }
    }
}

#[component]
fn RenderPlanDetail(ctx: PageContext) -> Element {
    let mutation = match ctx.query_param("mutation").as_deref() {
        Some("success") | Some("conflict") | Some("forbidden") | Some("unavailable")
        | Some("malformed") => ctx.query_param("mutation"),
        _ => None,
    };
    match plan_detail_load(&ctx) {
        PlanDetailLoad::Ready(plan) => rsx! {
            PlanDetailReady { ctx, plan, mutation }
        },
        PlanDetailLoad::Forbidden => plan_problem_element(ctx, ADMIN_PLANS_FORBIDDEN, "Wallet plan access was denied", "The backend did not authorize this session to read this plan."),
        PlanDetailLoad::Unavailable => plan_problem_element(ctx, ADMIN_PLANS_UNAVAILABLE, "Wallet plan detail is unavailable", "The subscription backend could not provide an authoritative plan response. No plan data is being shown."),
        PlanDetailLoad::Malformed => plan_problem_element(ctx, ADMIN_PLANS_MALFORMED, "Wallet plan detail could not be verified", "The route identifier or backend response did not match the strict plan contract. No plan data is being shown."),
        PlanDetailLoad::Unauthenticated => plan_banner_element(ctx, AdminDataState::Unauthenticated),
        PlanDetailLoad::Unauthorized => plan_banner_element(ctx, AdminDataState::Unauthorized),
    }
}

#[component]
fn PlanDetailReady(
    ctx: PageContext,
    plan: AdminPlanProjection,
    mutation: Option<String>,
) -> Element {
    let action = plan_href(&plan.id);
    let description = plan.description.clone().unwrap_or_default();
    let active = plan
        .active
        .map(|value| value.to_string())
        .unwrap_or_default();
    rsx! {
        WalletManagementHub { ctx,
            a {
                class: "inline-flex w-fit items-center gap-2 text-sm text-muted-foreground transition-colors hover:text-foreground",
                href: PLANS_PATH,
                Icon { name: "arrow-left".to_string(), size: Some(16) }
                "Back to Plans"
            }
            section {
                class: "overflow-hidden rounded-2xl border border-border/20 bg-card shadow-xl",
                "data-admin-wallet-plan-detail-state": ADMIN_PLANS_READY,
                div { class: "border-b border-border/20 px-4 py-4 sm:px-8",
                    div { class: "flex flex-wrap gap-3",
                        PlanEditorStat { value: "Unavailable".to_string(), label: "Members" }
                        PlanEditorStat { value: "Unavailable".to_string(), label: "Priority" }
                        PlanEditorStat { value: "Unavailable".to_string(), label: "Created" }
                        PlanEditorStat { value: format!("Version {}", plan.version), label: "Updated" }
                    }
                }
                if let Some(state) = mutation {
                    p { class: "mx-4 mt-4 rounded-lg border border-amber-500/30 bg-amber-500/5 p-3 text-sm sm:mx-8", role: if state == "forbidden" { "alert" } else { "status" },
                        "data-admin-wallet-plan-mutation-state": state,
                        "Plan mutation: {state}"
                    }
                }
                form { method: "post", action,
                    input { r#type: "hidden", name: "operation", value: "plan_update" }
                    input { r#type: "hidden", name: "plan_id", value: plan.id.clone() }
                    input { r#type: "hidden", name: "expected_version", value: plan.version.to_string() }
                    input { r#type: "hidden", name: "idempotency_key", value: format!("admin.plan.update.{}", uuid::Uuid::new_v4()) }
                    div { class: "flex items-center justify-between gap-3 border-b border-border/20 px-4 py-3 sm:px-8",
                        button { class: "btn btn-sm btn-outline cursor-not-allowed opacity-50", r#type: "button", disabled: true, title: "The subscription service does not expose a versioned delete contract", "Delete" }
                        div { class: "flex gap-2",
                            a { class: "btn btn-sm btn-outline", href: plan_href(&plan.id), "Discard" }
                            button { class: "btn btn-sm btn-primary", r#type: "submit", "Save Changes" }
                        }
                    }
                    div { class: "space-y-6 p-4 sm:p-8",
                        PlanEditorSectionHeader { title: "General" }
                        div { class: "grid grid-cols-1 gap-4 sm:grid-cols-2 sm:gap-6",
                            label { class: "space-y-2 text-sm font-medium text-foreground", "Plan Name"
                                input { class: "input input-bordered w-full bg-muted/30", name: "name", maxlength: 100, value: plan.name.clone(), required: true }
                            }
                            label { class: "space-y-2 text-sm font-medium text-foreground", "Description"
                                textarea { class: "textarea textarea-bordered min-h-20 w-full bg-muted/30", name: "description", maxlength: 2_000, rows: 3, value: description }
                            }
                            PlanUnavailableField { label: "Category", detail: "Not exposed by the subscription plan contract" }
                            PlanUnavailableField { label: "Display Group", detail: "Not exposed by the subscription plan contract" }
                        }

                        PlanEditorSectionHeader { title: "Pricing & Timing" }
                        div { class: "grid grid-cols-1 gap-4 sm:grid-cols-2 sm:gap-6",
                            label { class: "space-y-2 text-sm font-medium text-foreground", "Amount (minor units)"
                                input { class: "input input-bordered w-full bg-muted/30 font-mono", name: "amount", inputmode: "numeric", maxlength: 78, value: plan.amount.clone(), required: true }
                            }
                            label { class: "space-y-2 text-sm font-medium text-foreground", "Currency"
                                input { class: "input input-bordered w-full bg-muted/30 font-mono uppercase", name: "currency", maxlength: 10, value: plan.currency.clone(), required: true }
                            }
                            label { class: "space-y-2 text-sm font-medium text-foreground", "Chain ID"
                                input { class: "input input-bordered w-full bg-muted/30 font-mono", name: "chain_id", inputmode: "numeric", maxlength: 10, value: plan.chain_id.clone(), required: true }
                            }
                            label { class: "space-y-2 text-sm font-medium text-foreground", "Billing Interval (days)"
                                input { class: "input input-bordered w-full bg-muted/30", name: "interval", r#type: "number", min: 1, max: 366, value: plan.interval, required: true }
                            }
                        }

                        PlanEditorSectionHeader { title: "Status" }
                        div { class: "grid grid-cols-1 gap-4 sm:grid-cols-2 sm:gap-6",
                            label { class: "space-y-2 text-sm font-medium text-foreground", "Plan Status"
                                select { class: "select select-bordered w-full bg-muted/30", name: "active", value: active,
                                    option { value: "", "Backend default" }
                                    option { value: "true", "Active" }
                                    option { value: "false", "Inactive" }
                                }
                            }
                            PlanUnavailableField { label: "Public Visibility", detail: "Not exposed by the subscription plan contract" }
                        }

                        for section in ["Rate Limits", "Feature Toggles", "Pricing Page Features"] {
                            PlanEditorSectionHeader { title: section }
                            div { class: "grid grid-cols-1 gap-4 sm:grid-cols-2",
                                PlanUnavailableField { label: "Configuration", detail: "No typed backend field is available" }
                                PlanUnavailableField { label: "Configuration", detail: "No typed backend field is available" }
                            }
                        }

                        PlanEditorSectionHeader { title: "Permission Assignment" }
                        div { class: "grid min-h-64 grid-cols-1 gap-4 rounded-xl border border-border/20 p-4 sm:grid-cols-2",
                            div { class: "flex items-center justify-center rounded-xl border border-border/20 bg-background/30 p-6 text-center text-sm text-muted-foreground", "Available permissions are not part of this plan projection." }
                            div { class: "flex items-center justify-center rounded-xl border border-border/20 bg-background/30 p-6 text-center text-sm text-muted-foreground", "Assigned permissions are not part of this plan projection." }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PlanEditorStat(value: String, label: &'static str) -> Element {
    rsx! {
        div { class: "min-w-28 rounded-xl border border-border/20 bg-background/40 px-4 py-3",
            p { class: "font-semibold text-foreground", "{value}" }
            p { class: "text-[11px] text-muted-foreground", "{label}" }
        }
    }
}

#[component]
fn PlanEditorSectionHeader(title: &'static str) -> Element {
    rsx! {
        div { class: "flex items-center gap-3 pt-2",
            span { class: "whitespace-nowrap text-xs font-bold uppercase tracking-wider text-[#1fc7d4]", "{title}" }
            div { class: "h-px flex-1 bg-muted/30" }
        }
    }
}

#[component]
fn PlanUnavailableField(label: &'static str, detail: &'static str) -> Element {
    rsx! {
        label { class: "space-y-2 text-sm font-medium text-foreground", "{label}"
            input { class: "input input-bordered w-full cursor-not-allowed bg-muted/20 opacity-60", value: "Unavailable", disabled: true, title: detail }
            span { class: "block text-xs font-normal text-muted-foreground", "{detail}" }
        }
    }
}

fn plan_state_label(active: Option<bool>) -> &'static str {
    match active {
        Some(true) => "Active",
        Some(false) => "Inactive",
        None => "Status not reported",
    }
}

fn plan_href(id: &str) -> String {
    format!("{PLANS_PATH}/{id}")
}

fn plan_banner_element(ctx: PageContext, state: AdminDataState) -> Element {
    rsx! {
        WalletManagementHub { ctx,
            AdminDataStateBanner {
                state,
                subject: "Wallet plans".to_string(),
                return_path: PLANS_PATH.to_string(),
                retry_href: PLANS_PATH.to_string(),
            }
        }
    }
}

fn plan_problem_element(
    ctx: PageContext,
    state: &'static str,
    title: &'static str,
    detail: &'static str,
) -> Element {
    rsx! {
        WalletManagementHub { ctx,
            a {
                class: "inline-flex w-fit items-center gap-2 text-sm text-muted-foreground transition-colors hover:text-foreground",
                href: PLANS_PATH,
                Icon { name: "arrow-left".to_string(), size: Some(16) }
                "Back to Plans"
            }
            section {
                class: "rounded-2xl border border-amber-500/25 bg-amber-500/10 p-5 sm:p-6",
                role: if state == ADMIN_PLANS_FORBIDDEN { "alert" } else { "status" },
                aria_labelledby: "admin-wallet-plans-problem-title",
                "data-admin-wallet-plans-state": state,
                div { class: "flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between",
                    div {
                        h2 { id: "admin-wallet-plans-problem-title", class: "font-bold text-foreground", "{title}" }
                        p { class: "mt-1 max-w-3xl text-sm leading-6 text-muted-foreground", "{detail}" }
                    }
                    a { class: "btn btn-sm btn-outline shrink-0", href: PLANS_PATH, "Retry plan read" }
                }
            }
            PlanEditorScaffold {}
        }
    }
}

#[component]
fn PlanEditorScaffold() -> Element {
    rsx! {
        section { class: "overflow-hidden rounded-2xl border border-border/20 bg-card shadow-xl", aria_hidden: "true",
            div { class: "border-b border-border/20 px-4 py-4 sm:px-8",
                div { class: "flex flex-wrap gap-3",
                    for label in ["Members", "Priority", "Created", "Updated"] {
                        div { class: "min-w-28 rounded-xl border border-border/20 bg-background/40 px-4 py-3",
                            p { class: "font-semibold text-foreground", "Unavailable" }
                            p { class: "text-[11px] text-muted-foreground", "{label}" }
                        }
                    }
                }
            }
            div { class: "flex items-center justify-between gap-3 border-b border-border/20 px-4 py-3 sm:px-8",
                button { class: "btn btn-sm btn-outline cursor-not-allowed opacity-50", r#type: "button", disabled: true, "Delete" }
                div { class: "flex gap-2",
                    button { class: "btn btn-sm btn-outline cursor-not-allowed opacity-50", r#type: "button", disabled: true, "Discard" }
                    button { class: "btn btn-sm btn-primary cursor-not-allowed opacity-50", r#type: "button", disabled: true, "Save Changes" }
                }
            }
            div { class: "space-y-6 p-4 sm:p-8",
                for section in ["General", "Pricing & Timing", "Status", "Rate Limits", "Feature Toggles", "Pricing Page Features"] {
                    div { class: "space-y-4",
                        div { class: "flex items-center gap-3 pt-2",
                            span { class: "whitespace-nowrap text-xs font-bold uppercase tracking-wider text-[#1fc7d4]", "{section}" }
                            div { class: "h-px flex-1 bg-muted/30" }
                        }
                        div { class: "grid grid-cols-1 gap-4 sm:grid-cols-2",
                            div { class: "h-10 rounded-xl border border-border/20 bg-muted/20 opacity-50" }
                            div { class: "h-10 rounded-xl border border-border/20 bg-muted/20 opacity-50" }
                        }
                    }
                }
                div { class: "flex items-center gap-3 pt-2",
                    span { class: "whitespace-nowrap text-xs font-bold uppercase tracking-wider text-[#1fc7d4]", "Permission Assignment" }
                    div { class: "h-px flex-1 bg-muted/30" }
                }
                div { class: "grid min-h-64 grid-cols-1 gap-4 rounded-xl border border-border/20 p-4 sm:grid-cols-2",
                    div { class: "rounded-xl border border-border/20 bg-background/30" }
                    div { class: "rounded-xl border border-border/20 bg-background/30" }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::User;

    const PLAN_ID: &str = "00000000-0000-0000-0000-000000000001";

    fn session() -> User {
        User {
            id: "plan-session".to_string(),
            address: "0x1234".to_string(),
            chain_id: "56".to_string(),
            permissions: vec![],
            ..Default::default()
        }
    }

    fn list_context(state: &str, data: Option<serde_json::Value>, signed_in: bool) -> PageContext {
        let mut ctx = PageContext {
            user: signed_in.then(session),
            path: PLANS_PATH.to_string(),
            ..Default::default()
        };
        ctx.params
            .insert(ADMIN_PLANS_STATE_PARAM.to_string(), state.to_string());
        if let Some(data) = data {
            ctx.params
                .insert(ADMIN_PLANS_DATA_PARAM.to_string(), data.to_string());
        }
        ctx
    }

    fn detail_context(id: &str, state: &str, data: Option<serde_json::Value>) -> PageContext {
        let mut ctx = PageContext {
            user: Some(session()),
            path: format!("{PLANS_PATH}/{id}"),
            params: std::collections::HashMap::from([("planId".to_string(), id.to_string())]),
            ..Default::default()
        };
        ctx.params
            .insert(ADMIN_PLAN_DETAIL_STATE_PARAM.to_string(), state.to_string());
        if let Some(data) = data {
            ctx.params
                .insert(ADMIN_PLAN_DETAIL_DATA_PARAM.to_string(), data.to_string());
        }
        ctx
    }

    fn plan_json(id: &str) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "name": "Pro",
            "description": "Backend-defined plan",
            "amount": "2900",
            "currency": "USD",
            "chain_id": "56",
            "interval": 30,
            "active": true,
            "version": 2,
        })
    }

    fn list_json() -> serde_json::Value {
        serde_json::json!({
            "items": [plan_json(PLAN_ID)],
            "total": 1,
            "limit": 20,
            "offset": 0,
        })
    }

    #[test]
    fn strict_plan_projection_redacts_merchant_and_timestamp_fields() {
        assert!(decode_admin_plan_projection(plan_json(PLAN_ID)).is_some());
        assert!(decode_admin_plan_projection(serde_json::json!({
            "id": PLAN_ID,
            "name": "Pro",
            "description": null,
            "amount": "2900",
            "currency": "USD",
            "chain_id": "56",
            "interval": 30,
            "active": true,
            "version": 0,
            "merchant_id": "private-merchant",
            "created_at": "2026-01-01T00:00:00Z",
        }))
        .is_none());
        assert!(decode_admin_plan_projection(serde_json::json!({
            "id": "not-a-uuid",
            "name": "Pro",
            "description": null,
            "amount": "2900",
            "currency": "USD",
            "chain_id": "56",
            "interval": 30,
            "active": true,
            "version": 0,
        }))
        .is_none());
    }

    #[test]
    fn ready_list_and_detail_expose_bounded_plan_mutations() {
        let list = dioxus_ssr::render_element(
            render(&list_context(ADMIN_PLANS_READY, Some(list_json()), true)).1,
        );
        let detail = dioxus_ssr::render_element(
            render_editor(&detail_context(
                PLAN_ID,
                ADMIN_PLANS_READY,
                Some(plan_json(PLAN_ID)),
            ))
            .1,
        );
        assert!(list.contains("data-admin-wallet-plans-state=\"ready\""));
        assert!(list.contains("Wallet Management Hub"));
        assert!(list.contains("Read detail"));
        assert!(detail.contains("data-admin-wallet-plan-detail-state=\"ready\""));
        assert!(detail.contains("Wallet Management Hub"));
        assert!(detail.contains("Backend-defined plan"));
        assert!(list.contains("<form"));
        assert!(list.contains("Create plan"));
        assert!(list.contains("idempotency_key"));
        assert!(detail.contains("<form"));
        assert!(detail.contains("Save Changes"));
        assert!(detail.contains("expected_version"));
        assert!(detail.contains("value=\"2\""));
        assert!(!detail.contains("value=\"2/\""));
        assert!(!detail.contains("name=\"merchant_id\""));
    }

    #[test]
    fn detail_renders_only_a_closed_mutation_state() {
        let mut conflict = detail_context(PLAN_ID, ADMIN_PLANS_READY, Some(plan_json(PLAN_ID)));
        conflict.query = "mutation=conflict".to_string();
        let rendered = dioxus_ssr::render_element(render_editor(&conflict).1);
        assert!(rendered.contains("data-admin-wallet-plan-mutation-state=\"conflict\""));

        conflict.query = "mutation=unknown".to_string();
        let rendered = dioxus_ssr::render_element(render_editor(&conflict).1);
        assert!(!rendered.contains("data-admin-wallet-plan-mutation-state="));
    }

    #[test]
    fn invalid_or_mismatched_dynamic_plan_ids_are_malformed() {
        let invalid = detail_context("not-a-uuid", ADMIN_PLANS_READY, Some(plan_json(PLAN_ID)));
        let mismatch = detail_context(
            PLAN_ID,
            ADMIN_PLANS_READY,
            Some(plan_json("00000000-0000-0000-0000-000000000002")),
        );
        for ctx in [invalid, mismatch] {
            assert!(
                dioxus_ssr::render_element(render_editor(&ctx).1)
                    .contains("data-admin-wallets-state=\"malformed\"")
                    || dioxus_ssr::render_element(render_editor(&ctx).1)
                        .contains("data-admin-wallet-plans-state=\"malformed\"")
            );
        }
    }

    #[test]
    fn list_unauthenticated_and_unauthorized_render_shared_banner() {
        for state in [ADMIN_PLANS_UNAUTHENTICATED, ADMIN_PLANS_UNAUTHORIZED] {
            let rendered =
                dioxus_ssr::render_element(render(&list_context(state, Some(list_json()), true)).1);
            assert!(rendered.contains(&format!("data-admin-data-state=\"{state}\"")));
            assert!(rendered.contains("Wallet Management Hub"));
            assert!(!rendered.contains("Backend-defined plan"));
        }
    }

    #[test]
    fn detail_unauthenticated_and_unauthorized_render_shared_banner() {
        for state in [
            ADMIN_PLAN_DETAIL_UNAUTHENTICATED,
            ADMIN_PLAN_DETAIL_UNAUTHORIZED,
        ] {
            let rendered = dioxus_ssr::render_element(
                render_editor(&detail_context(PLAN_ID, state, Some(plan_json(PLAN_ID)))).1,
            );
            assert!(rendered.contains(&format!("data-admin-data-state=\"{state}\"")));
            assert!(!rendered.contains("Backend-defined plan"));
        }
    }

    #[test]
    fn forbidden_unavailable_malformed_and_signed_out_states_hide_data() {
        for state in [
            ADMIN_PLANS_FORBIDDEN,
            ADMIN_PLANS_UNAVAILABLE,
            ADMIN_PLANS_MALFORMED,
        ] {
            let rendered =
                dioxus_ssr::render_element(render(&list_context(state, Some(list_json()), true)).1);
            assert!(rendered.contains(&format!("data-admin-wallet-plans-state=\"{state}\"")));
            assert!(rendered.contains("Wallet Management Hub"));
            assert!(!rendered.contains("Backend-defined plan"));
        }
        let signed_out = dioxus_ssr::render_element(
            render(&list_context(ADMIN_PLANS_READY, Some(list_json()), false)).1,
        );
        assert!(signed_out.contains("Sign in required"));
        assert!(!signed_out.contains("Backend-defined plan"));
    }
}

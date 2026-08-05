//! Backend-authorized admin notification inventory, metrics, and lifecycle UI.
//!
//! The management page renders only a strict backend projection. Recipient
//! identity, message content, delivery errors, read state, action URLs, and all
//! mutations remain backend concerns; this leaf applies only the authenticated
//! session boundary.

use chrono::DateTime;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

use super::super::{PageContext, PageMeta};
use crate::auth::AuthGate;
use crate::components::admin::page_layout::{PageGradient, PageHeader, PageLayout, PageMaxWidth};
use crate::primitives::Icon;

const NOTIFICATIONS_PATH: &str = "/notifications/manage";
const NOTIFICATION_PAGE_LIMIT: i64 = 20;
const MAX_NOTIFICATION_OFFSET: i64 = 1_000_000;
const MAX_ID_CHARS: usize = 66;
const MAX_TITLE_CHARS: usize = 255;
const MAX_SUBJECT_CHARS: usize = 255;
const MAX_CHANNEL_CHARS: usize = 20;
const MAX_TYPE_CHARS: usize = 50;
const MIN_TIMESTAMP_CHARS: usize = 20;
const MAX_TIMESTAMP_CHARS: usize = 64;

pub const ADMIN_NOTIFICATIONS_DATA_PARAM: &str = "data_admin_notifications";
pub const ADMIN_NOTIFICATIONS_STATE_PARAM: &str = "data_admin_notifications_state";
pub const ADMIN_NOTIFICATIONS_PAGE_PARAM: &str = "admin_notifications_page";
pub const ADMIN_NOTIFICATIONS_MUTATION_PARAM: &str = "mutation";
pub const ADMIN_NOTIFICATIONS_STATUS_PARAM: &str = "status";
pub const ADMIN_NOTIFICATIONS_TYPE_PARAM: &str = "type";
pub const ADMIN_NOTIFICATIONS_PRIORITY_PARAM: &str = "priority";
pub const ADMIN_NOTIFICATION_METRICS_DATA_PARAM: &str = "data_admin_notification_metrics";
pub const ADMIN_NOTIFICATION_METRICS_STATE_PARAM: &str = "data_admin_notification_metrics_state";

pub const ADMIN_NOTIFICATIONS_READY: &str = "ready";
pub const ADMIN_NOTIFICATIONS_EMPTY: &str = "empty";
pub const ADMIN_NOTIFICATIONS_FORBIDDEN: &str = "forbidden";
pub const ADMIN_NOTIFICATIONS_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_NOTIFICATIONS_MALFORMED: &str = "malformed";
pub const ADMIN_NOTIFICATIONS_SEND_STATE_PARAM: &str = "data_admin_notifications_send_state";
pub const ADMIN_NOTIFICATIONS_SEND_ACCEPTED: &str = "accepted";
pub const ADMIN_NOTIFICATIONS_SEND_ERROR: &str = "error";

pub const ADMIN_NOTIFICATION_CREATE_DATA_PARAM: &str = "data_admin_notification_create";
pub const ADMIN_NOTIFICATION_CREATE_STATE_PARAM: &str = "data_admin_notification_create_state";
pub const ADMIN_NOTIFICATION_CREATE_FORM: &str = "form";
pub const ADMIN_NOTIFICATION_CREATE_PENDING: &str = "pending";
pub const ADMIN_NOTIFICATION_CREATE_SENT: &str = "sent";
pub const ADMIN_NOTIFICATION_CREATE_FAILED: &str = "failed";
pub const ADMIN_NOTIFICATION_CREATE_FORBIDDEN: &str = "forbidden";
pub const ADMIN_NOTIFICATION_CREATE_CONFLICT: &str = "conflict";
pub const ADMIN_NOTIFICATION_CREATE_INVALID: &str = "invalid";
pub const ADMIN_NOTIFICATION_CREATE_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_NOTIFICATION_CREATE_MALFORMED: &str = "malformed";

/// Safe mutation acknowledgement projected by the route-specific BFF. It
/// intentionally excludes recipients, bodies, template data, and delivery
/// error details from the page context.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminNotificationCreateResult {
    pub id: String,
    pub status: String,
    pub delivered: bool,
    pub request_id: String,
}

pub fn decode_admin_notification_create_result(
    value: serde_json::Value,
) -> Option<AdminNotificationCreateResult> {
    let result: AdminNotificationCreateResult = serde_json::from_value(value).ok()?;
    if !result.id.starts_with("idem_")
        || !valid_required_text(&result.id, MAX_ID_CHARS)
        || !matches!(
            result.status.as_str(),
            ADMIN_NOTIFICATION_CREATE_PENDING
                | ADMIN_NOTIFICATION_CREATE_SENT
                | ADMIN_NOTIFICATION_CREATE_FAILED
        )
        || result.delivered != (result.status == ADMIN_NOTIFICATION_CREATE_SENT)
        || !valid_required_text(&result.request_id, 128)
    {
        return None;
    }
    Some(result)
}

/// Deliberately excludes recipient and user identity, body/message/data/error,
/// read state, action URLs, and every field that could imply mutation access.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminNotificationSummary {
    pub id: String,
    pub title: Option<String>,
    pub subject: Option<String>,
    pub channel: String,
    pub status: String,
    pub notification_type: Option<String>,
    pub priority: Option<String>,
    pub sent_at: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminNotificationList {
    pub items: Vec<AdminNotificationSummary>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Redacted operational counters safe for the admin workspace. These are
/// bounded observations, not delivery receipts for any individual message.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminNotificationMetrics {
    pub queue_depth: i64,
    pub queue_age_seconds: Option<i64>,
    pub suppressed: i64,
    pub retry_wait: i64,
    pub terminal_failed: i64,
    pub dead_lettered: i64,
    pub provider_accepted: i64,
    pub attempting: i64,
    pub channel_outcomes: BTreeMap<String, i64>,
    pub provider_events: i64,
    pub delivery_attempts: i64,
    pub replay_cursors: i64,
    pub replay_cursor_age_seconds: Option<i64>,
    pub active_streams: i64,
    pub stream_connections_total: i64,
    pub stream_reconnects_total: i64,
    pub stream_replayed_events_total: i64,
    pub stream_lag_seconds: Option<i64>,
    pub stream_query_failures_total: i64,
}

pub fn decode_admin_notification_metrics(
    value: serde_json::Value,
) -> Option<AdminNotificationMetrics> {
    let metrics: AdminNotificationMetrics = serde_json::from_value(value).ok()?;
    let non_negative = |value: i64| (0..=10_000_000).contains(&value);
    if !non_negative(metrics.queue_depth)
        || !metrics.queue_age_seconds.is_none_or(non_negative)
        || !non_negative(metrics.suppressed)
        || !non_negative(metrics.retry_wait)
        || !non_negative(metrics.terminal_failed)
        || !non_negative(metrics.dead_lettered)
        || !non_negative(metrics.provider_accepted)
        || !non_negative(metrics.attempting)
        || !non_negative(metrics.provider_events)
        || !non_negative(metrics.delivery_attempts)
        || !non_negative(metrics.replay_cursors)
        || !metrics.replay_cursor_age_seconds.is_none_or(non_negative)
        || !non_negative(metrics.active_streams)
        || metrics.active_streams > 256
        || !non_negative(metrics.stream_connections_total)
        || !non_negative(metrics.stream_reconnects_total)
        || !non_negative(metrics.stream_replayed_events_total)
        || !metrics.stream_lag_seconds.is_none_or(non_negative)
        || !non_negative(metrics.stream_query_failures_total)
        || metrics.channel_outcomes.len() > 3
        || metrics.channel_outcomes.iter().any(|(channel, count)| {
            !matches!(channel.as_str(), "email" | "in_app" | "push") || !non_negative(*count)
        })
    {
        return None;
    }
    Some(metrics)
}

/// Decode the exact read projection and reject semantically impossible values
/// before any backend field reaches HTML.
pub fn decode_admin_notification_projection(
    value: serde_json::Value,
) -> Option<AdminNotificationList> {
    let projection: AdminNotificationList = serde_json::from_value(value).ok()?;
    let total = usize::try_from(projection.total).ok()?;
    let limit = usize::try_from(projection.limit).ok()?;
    let item_count = i64::try_from(projection.items.len()).ok()?;
    let page_end = projection.offset.checked_add(item_count)?;

    if projection.limit != NOTIFICATION_PAGE_LIMIT
        || projection.offset < 0
        || projection.offset > MAX_NOTIFICATION_OFFSET
        || projection.offset % NOTIFICATION_PAGE_LIMIT != 0
        || projection.items.len() > limit
        || total < projection.items.len()
        || (!projection.items.is_empty() && page_end > projection.total)
        || projection.items.iter().any(|item| !item.is_well_formed())
    {
        return None;
    }

    Some(projection)
}

impl AdminNotificationSummary {
    fn is_well_formed(&self) -> bool {
        valid_required_text(&self.id, MAX_ID_CHARS)
            && valid_optional_text(self.title.as_deref(), MAX_TITLE_CHARS)
            && valid_optional_text(self.subject.as_deref(), MAX_SUBJECT_CHARS)
            && valid_channel(&self.channel)
            && matches!(
                self.status.as_str(),
                "pending" | "sent" | "failed" | "read" | "suppressed" | "expired"
            )
            && valid_optional_text(self.notification_type.as_deref(), MAX_TYPE_CHARS)
            && self.priority.as_deref().is_none_or(|priority| {
                matches!(priority, "low" | "normal" | "high" | "critical" | "urgent")
            })
            && self.sent_at.as_deref().is_none_or(valid_timestamp)
            && valid_timestamp(&self.created_at)
    }
}

fn valid_required_text(value: &str, max_chars: usize) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= max_chars
        && !value.chars().any(char::is_control)
}

fn valid_optional_text(value: Option<&str>, max_chars: usize) -> bool {
    value.is_none_or(|value| {
        value.chars().count() <= max_chars && !value.chars().any(char::is_control)
    })
}

fn valid_channel(value: &str) -> bool {
    !value.is_empty()
        && value.chars().count() <= MAX_CHANNEL_CHARS
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

fn valid_timestamp(value: &str) -> bool {
    let length = value.len();
    (MIN_TIMESTAMP_CHARS..=MAX_TIMESTAMP_CHARS).contains(&length)
        && !value.chars().any(char::is_control)
        && DateTime::parse_from_rfc3339(value).is_ok()
}

fn notification_status_class(status: &str) -> &'static str {
    match status {
        "sent" => "border-green-500/30 bg-green-500/10 text-green-800 dark:text-green-300",
        "failed" => "border-red-500/30 bg-red-500/10 text-red-800 dark:text-red-300",
        "suppressed" => "border-amber-500/30 bg-amber-500/10 text-amber-800 dark:text-amber-300",
        _ => "border-amber-500/30 bg-amber-500/10 text-amber-900 dark:text-amber-300",
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NotificationPage {
    page: i64,
    status: Option<String>,
    notification_type: Option<String>,
    priority: Option<String>,
}

impl NotificationPage {
    fn from_ctx(ctx: &PageContext) -> Self {
        let page = ctx
            .params
            .get(ADMIN_NOTIFICATIONS_PAGE_PARAM)
            .and_then(|value| value.parse::<i64>().ok())
            .filter(|page| (1..=50_001).contains(page))
            .unwrap_or(1);
        Self {
            page,
            status: ctx.params.get(ADMIN_NOTIFICATIONS_STATUS_PARAM).cloned(),
            notification_type: ctx.params.get(ADMIN_NOTIFICATIONS_TYPE_PARAM).cloned(),
            priority: ctx.params.get(ADMIN_NOTIFICATIONS_PRIORITY_PARAM).cloned(),
        }
    }

    fn expected_offset(self) -> Option<i64> {
        self.page
            .checked_sub(1)?
            .checked_mul(NOTIFICATION_PAGE_LIMIT)
    }

    fn href(&self, page: i64) -> String {
        let mut href = format!("{NOTIFICATIONS_PATH}?page={}", page.clamp(1, 50_001));
        if let Some(status) = &self.status {
            href.push_str("&status=");
            href.push_str(status);
        }
        if let Some(notification_type) = &self.notification_type {
            href.push_str("&type=");
            href.push_str(notification_type);
        }
        if let Some(priority) = &self.priority {
            href.push_str("&priority=");
            href.push_str(priority);
        }
        href
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum NotificationLoad {
    Ready(AdminNotificationList),
    Empty,
    Forbidden,
    Unavailable,
    Malformed,
}

fn notification_load(ctx: &PageContext, page: NotificationPage) -> NotificationLoad {
    let state = ctx
        .params
        .get(ADMIN_NOTIFICATIONS_STATE_PARAM)
        .map(String::as_str);
    match state {
        Some(ADMIN_NOTIFICATIONS_READY) | Some(ADMIN_NOTIFICATIONS_EMPTY) => {
            let Some(raw) = ctx.params.get(ADMIN_NOTIFICATIONS_DATA_PARAM) else {
                return NotificationLoad::Malformed;
            };
            let Some(projection) = serde_json::from_str(raw)
                .ok()
                .and_then(decode_admin_notification_projection)
            else {
                return NotificationLoad::Malformed;
            };
            if Some(projection.offset) != page.expected_offset() {
                return NotificationLoad::Malformed;
            }

            match (state, projection.items.is_empty(), projection.total) {
                (Some(ADMIN_NOTIFICATIONS_READY), false, _) => NotificationLoad::Ready(projection),
                (Some(ADMIN_NOTIFICATIONS_READY), true, total) if total > 0 => {
                    NotificationLoad::Ready(projection)
                }
                (Some(ADMIN_NOTIFICATIONS_EMPTY), true, 0) => NotificationLoad::Empty,
                _ => NotificationLoad::Malformed,
            }
        }
        Some(ADMIN_NOTIFICATIONS_FORBIDDEN) => NotificationLoad::Forbidden,
        Some(ADMIN_NOTIFICATIONS_MALFORMED) => NotificationLoad::Malformed,
        Some(ADMIN_NOTIFICATIONS_UNAVAILABLE) | None => NotificationLoad::Unavailable,
        Some(_) => NotificationLoad::Malformed,
    }
}

fn notification_metrics(ctx: &PageContext) -> Option<AdminNotificationMetrics> {
    if ctx
        .params
        .get(ADMIN_NOTIFICATION_METRICS_STATE_PARAM)
        .map(String::as_str)
        != Some(ADMIN_NOTIFICATIONS_READY)
    {
        return None;
    }
    ctx.params
        .get(ADMIN_NOTIFICATION_METRICS_DATA_PARAM)
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok())
        .and_then(decode_admin_notification_metrics)
}

/// `/notifications/manage` — authenticated global inventory with
/// backend-authorized read/delete controls.
pub fn render_manage(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Notifications");
    (
        meta,
        rsx! {
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("the admin notification workspace".to_string()),
                return_url: Some(NOTIFICATIONS_PATH.to_string()),
                RenderNotificationList { ctx: ctx.clone() }
            }
        },
    )
}

/// `/notifications/create` — authenticated compose surface backed by the
/// notification service's idempotent admin send contract.
pub fn render_create(ctx: &PageContext) -> (PageMeta, Element) {
    let load = notification_create_load(ctx);
    let title = match &load {
        NotificationCreateLoad::Form => "Create notification",
        NotificationCreateLoad::Sent(_) => "Notification sent",
        NotificationCreateLoad::Pending(_) => "Notification delivery pending",
        NotificationCreateLoad::Failed(_) => "Notification delivery failed",
        NotificationCreateLoad::Forbidden => "Notification creation denied",
        NotificationCreateLoad::Conflict => "Notification request conflict",
        NotificationCreateLoad::Invalid => "Notification request is invalid",
        NotificationCreateLoad::Unavailable => "New notification unavailable",
        NotificationCreateLoad::Malformed => "Notification result could not be verified",
    };
    let meta = PageMeta::admin(title);
    (
        meta,
        rsx! {
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("creating notifications".to_string()),
                return_url: Some("/notifications/create".to_string()),
                PageLayout {
                    max_width: Some(PageMaxWidth::FourXl),
                    PageHeader {
                        title: "Command Center".to_string(),
                        subtitle: Some("Global broadcast protocol and network alert management".to_string()),
                        icon: Some("bell".to_string()),
                        gradient: Some(PageGradient::Info),
                        centered: Some(false),
                        extra_actions: None,
                        class_name: None,
                    }
                    NotificationCreateState { load }
                }
            }
        },
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum NotificationCreateLoad {
    Form,
    Sent(AdminNotificationCreateResult),
    Pending(AdminNotificationCreateResult),
    Failed(AdminNotificationCreateResult),
    Forbidden,
    Conflict,
    Invalid,
    Unavailable,
    Malformed,
}

fn notification_create_load(ctx: &PageContext) -> NotificationCreateLoad {
    let state = ctx
        .params
        .get(ADMIN_NOTIFICATION_CREATE_STATE_PARAM)
        .map(String::as_str);
    match state {
        Some(ADMIN_NOTIFICATION_CREATE_SENT)
        | Some(ADMIN_NOTIFICATION_CREATE_PENDING)
        | Some(ADMIN_NOTIFICATION_CREATE_FAILED) => {
            let Some(raw) = ctx.params.get(ADMIN_NOTIFICATION_CREATE_DATA_PARAM) else {
                return NotificationCreateLoad::Malformed;
            };
            let Some(result) = serde_json::from_str::<serde_json::Value>(raw)
                .ok()
                .and_then(decode_admin_notification_create_result)
            else {
                return NotificationCreateLoad::Malformed;
            };
            match (state, result.status.as_str()) {
                (Some(ADMIN_NOTIFICATION_CREATE_SENT), ADMIN_NOTIFICATION_CREATE_SENT) => {
                    NotificationCreateLoad::Sent(result)
                }
                (Some(ADMIN_NOTIFICATION_CREATE_PENDING), ADMIN_NOTIFICATION_CREATE_PENDING) => {
                    NotificationCreateLoad::Pending(result)
                }
                (Some(ADMIN_NOTIFICATION_CREATE_FAILED), ADMIN_NOTIFICATION_CREATE_FAILED) => {
                    NotificationCreateLoad::Failed(result)
                }
                _ => NotificationCreateLoad::Malformed,
            }
        }
        Some(ADMIN_NOTIFICATION_CREATE_FORBIDDEN) => NotificationCreateLoad::Forbidden,
        Some(ADMIN_NOTIFICATION_CREATE_CONFLICT) => NotificationCreateLoad::Conflict,
        Some(ADMIN_NOTIFICATION_CREATE_INVALID) => NotificationCreateLoad::Invalid,
        Some(ADMIN_NOTIFICATION_CREATE_UNAVAILABLE) => NotificationCreateLoad::Unavailable,
        Some(ADMIN_NOTIFICATION_CREATE_FORM) | None => NotificationCreateLoad::Form,
        Some(ADMIN_NOTIFICATION_CREATE_MALFORMED) => NotificationCreateLoad::Malformed,
        Some(_) => NotificationCreateLoad::Malformed,
    }
}

#[component]
fn RenderNotificationList(ctx: PageContext) -> Element {
    let page = NotificationPage::from_ctx(&ctx);
    let load = notification_load(&ctx, page.clone());

    rsx! {
        PageLayout {
            max_width: Some(PageMaxWidth::SevenXl),
            PageHeader {
                title: "Notifications".to_string(),
                subtitle: Some("Review backend-authoritative delivery summaries".to_string()),
                icon: Some("bell".to_string()),
                gradient: Some(PageGradient::Info),
                centered: Some(false),
                extra_actions: None,
                class_name: None,
            }
            NotificationFilters { page: page.clone() }
            if let Some(metrics) = notification_metrics(&ctx) {
                NotificationMetricsPanel { snapshot: metrics }
            }
            if let Some(state) = notification_mutation_state(&ctx) {
                NotificationMutationNotice { state }
            }
            if let Some(state) = ctx.params.get(ADMIN_NOTIFICATIONS_SEND_STATE_PARAM) {
                NotificationSendFeedback { state: state.clone() }
            }
            match load {
                NotificationLoad::Ready(projection) => rsx! {
                    NotificationReady { projection, page }
                },
                NotificationLoad::Empty => rsx! {
                    NotificationEmpty { page }
                },
                NotificationLoad::Forbidden => rsx! {
                    NotificationProblem {
                        state: ADMIN_NOTIFICATIONS_FORBIDDEN,
                        title: "Notification access was denied".to_string(),
                        detail: "The backend did not authorize this session to read the notification inventory.".to_string(),
                        retry_href: page.href(page.page),
                    }
                },
                NotificationLoad::Unavailable => rsx! {
                    NotificationProblem {
                        state: ADMIN_NOTIFICATIONS_UNAVAILABLE,
                        title: "Notification records are unavailable".to_string(),
                        detail: "The notification backend could not provide an authoritative response. No records are being shown.".to_string(),
                        retry_href: page.href(page.page),
                    }
                },
                NotificationLoad::Malformed => rsx! {
                    NotificationProblem {
                        state: ADMIN_NOTIFICATIONS_MALFORMED,
                        title: "Notification data could not be verified".to_string(),
                        detail: "The notification backend response did not match the inventory contract. No records are being shown.".to_string(),
                        retry_href: page.href(page.page),
                    }
                },
            }
        }
    }
}

#[component]
fn NotificationFilters(page: NotificationPage) -> Element {
    rsx! {
        form { class: "mb-6 flex flex-wrap items-end gap-3 rounded-2xl border border-border/30 bg-card p-4", method: "get", action: NOTIFICATIONS_PATH,
            div { class: "min-w-40",
                label { class: "text-xs font-semibold uppercase tracking-wide text-muted-foreground", r#for: "notification-status-filter", "Status" }
                select { id: "notification-status-filter", name: "status", class: "mt-1 w-full rounded-lg border border-border/40 bg-background px-3 py-2 text-sm",
                    option { value: "", "All statuses" }
                    option { value: "pending", "Pending" }
                    option { value: "sent", "Sent" }
                    option { value: "failed", "Failed" }
                    option { value: "read", "Read" }
                    option { value: "unread", "Unread" }
                }
            }
            div { class: "min-w-40",
                label { class: "text-xs font-semibold uppercase tracking-wide text-muted-foreground", r#for: "notification-type-filter", "Type" }
                input { id: "notification-type-filter", name: "type", maxlength: "50", pattern: "[a-z0-9_-]+", class: "mt-1 w-full rounded-lg border border-border/40 bg-background px-3 py-2 text-sm", placeholder: "Any type" }
            }
            div { class: "min-w-40",
                label { class: "text-xs font-semibold uppercase tracking-wide text-muted-foreground", r#for: "notification-priority-filter", "Priority" }
                select { id: "notification-priority-filter", name: "priority", class: "mt-1 w-full rounded-lg border border-border/40 bg-background px-3 py-2 text-sm",
                    option { value: "", "All priorities" }
                    option { value: "low", "Low" }
                    option { value: "normal", "Normal" }
                    option { value: "high", "High" }
                    option { value: "critical", "Critical" }
                    option { value: "urgent", "Urgent" }
                }
            }
            button { type: "submit", class: "btn btn-sm btn-outline", "Apply filters" }
            a { class: "btn btn-sm btn-ghost", href: page.href(1), "Clear" }
        }
    }
}

fn notification_mutation_state(ctx: &PageContext) -> Option<&'static str> {
    match ctx
        .params
        .get(ADMIN_NOTIFICATIONS_MUTATION_PARAM)
        .map(String::as_str)
    {
        Some("committed") => Some("committed"),
        Some("conflict") => Some("conflict"),
        Some("forbidden") => Some("forbidden"),
        Some("unavailable") => Some("unavailable"),
        Some("malformed") => Some("malformed"),
        _ => None,
    }
}

#[component]
fn NotificationMutationNotice(state: &'static str) -> Element {
    let (title, detail, class_name) = match state {
        "committed" => (
            "Notification updated",
            "The backend committed the notification operation.",
            "border-green-500/30 bg-green-500/10",
        ),
        "forbidden" => (
            "Notification operation denied",
            "The backend did not authorize this operation.",
            "border-red-500/30 bg-red-500/10",
        ),
        "unavailable" => (
            "Notification operation unavailable",
            "No authoritative mutation result is available.",
            "border-amber-500/30 bg-amber-500/10",
        ),
        "conflict" => (
            "Notification operation conflicted",
            "The backend reported a conflicting notification state.",
            "border-amber-500/30 bg-amber-500/10",
        ),
        _ => (
            "Notification operation could not be verified",
            "The mutation response did not match the strict contract.",
            "border-amber-500/30 bg-amber-500/10",
        ),
    };
    rsx! {
        section { class: "mb-6 rounded-2xl border p-4 {class_name}", role: "status", "data-admin-notifications-mutation": state,
            h2 { class: "font-semibold text-foreground", "{title}" }
            p { class: "mt-1 text-sm text-muted-foreground", "{detail}" }
        }
    }
}

#[component]
fn NotificationReady(projection: AdminNotificationList, page: NotificationPage) -> Element {
    let total_pages = (projection.total / projection.limit
        + i64::from(projection.total % projection.limit != 0))
    .max(1);
    let has_previous = page.page > 1;
    let has_next = page.page < total_pages;

    rsx! {
        section {
            class: "admin-notification-list overflow-hidden rounded-2xl border border-border/30 bg-card shadow-xl",
            aria_label: "Backend-authoritative notification inventory",
            "data-admin-notifications-state": ADMIN_NOTIFICATIONS_READY,
            div { class: "h-1 bg-gradient-to-r from-[#1fc7d4] via-[#7645d9] to-[#ffb237]" }
            div { class: "flex flex-wrap items-center justify-between gap-3 p-6",
                div {
                    h2 { class: "text-lg font-semibold text-foreground", "Delivery inventory" }
                    p { class: "text-sm text-muted-foreground", "{projection.total} authoritative records" }
                }
                p { class: "text-sm text-muted-foreground", "Page {page.page} of {total_pages}" }
            }
            if projection.items.is_empty() {
                div {
                    class: "border-t border-border/30 p-10 text-center",
                    role: "status",
                    "data-admin-notifications-page-state": "out-of-range",
                    h3 { class: "font-semibold text-foreground", "No notifications on this page" }
                    p { class: "mt-2 text-sm text-muted-foreground", "The inventory still contains records. Return to the first page or use Previous." }
                    a { class: "btn btn-sm btn-outline mt-5", href: page.href(1), "Return to first page" }
                }
            } else {
                ul { class: "grid gap-4 border-t border-border/30 p-4 lg:grid-cols-2 sm:p-6", aria_label: "Notification summaries",
                    for notification in projection.items {
                        NotificationCard { notification }
                    }
                }
            }
            nav { class: "flex items-center justify-between border-t border-border/30 p-4", aria_label: "Notification pagination",
                if has_previous {
                    a { class: "btn btn-sm btn-outline", href: page.href(page.page - 1), rel: "prev", "Previous" }
                } else {
                    span { class: "btn btn-sm btn-outline opacity-50", aria_disabled: "true", "Previous" }
                }
                if has_next {
                    a { class: "btn btn-sm btn-outline", href: page.href(page.page + 1), rel: "next", "Next" }
                } else {
                    span { class: "btn btn-sm btn-outline opacity-50", aria_disabled: "true", "Next" }
                }
            }
        }
    }
}

#[component]
fn NotificationCard(notification: AdminNotificationSummary) -> Element {
    let heading = notification
        .title
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            notification
                .subject
                .as_deref()
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or("Untitled notification");
    let subject = notification.subject.as_deref().filter(|subject| {
        !subject.trim().is_empty() && notification.title.as_deref() != Some(*subject)
    });
    let notification_type = notification
        .notification_type
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("Unspecified");
    let priority = notification.priority.as_deref().unwrap_or("Unspecified");
    let status_class = notification_status_class(&notification.status);

    rsx! {
        li { class: "rounded-2xl border border-border/30 bg-background/40 p-5",
            div { class: "flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between",
                div { class: "min-w-0",
                    h3 { class: "text-lg font-semibold text-foreground", "{heading}" }
                    if let Some(subject) = subject {
                        p { class: "mt-1 text-sm text-muted-foreground", "{subject}" }
                    }
                }
                span { class: "inline-flex w-fit rounded-full border px-2.5 py-1 text-xs font-semibold {status_class}", "{notification.status}" }
            }
            dl { class: "mt-5 grid grid-cols-2 gap-4 border-t border-border/20 pt-4 text-sm sm:grid-cols-3",
                NotificationFact { label: "Channel", value: notification.channel.clone() }
                NotificationFact { label: "Type", value: notification_type.to_string() }
                NotificationFact { label: "Priority", value: priority.to_string() }
                NotificationFact { label: "Sent", value: notification.sent_at.clone().unwrap_or_else(|| "Not sent".to_string()) }
                NotificationFact { label: "Created", value: notification.created_at.clone() }
            }
            div { class: "mt-5 flex flex-wrap gap-2 border-t border-border/20 pt-4",
                if notification.status != "read" {
                    form { method: "post", action: NOTIFICATIONS_PATH,
                        input { type: "hidden", name: "action", value: "read" }
                        input { type: "hidden", name: "id", value: notification.id.clone() }
                        button { type: "submit", class: "btn btn-sm btn-outline", "Mark read" }
                    }
                }
                form { method: "post", action: NOTIFICATIONS_PATH,
                    input { type: "hidden", name: "action", value: "delete" }
                    input { type: "hidden", name: "id", value: notification.id.clone() }
                    button { type: "submit", class: "btn btn-sm btn-outline text-red-700 dark:text-red-300", "Delete" }
                }
            }
        }
    }
}

#[component]
fn NotificationFact(label: &'static str, value: String) -> Element {
    rsx! {
        div {
            dt { class: "text-xs uppercase tracking-wide text-muted-foreground", "{label}" }
            dd { class: "mt-1 break-words text-foreground", "{value}" }
        }
    }
}

#[component]
fn NotificationEmpty(page: NotificationPage) -> Element {
    rsx! {
        section { class: "rounded-2xl border border-border/30 bg-card p-10 text-center", role: "status", "data-admin-notifications-state": ADMIN_NOTIFICATIONS_EMPTY,
            Icon { name: "bell".to_string(), size: Some(32) }
            h2 { class: "mt-4 text-lg font-semibold text-foreground", "No notifications found" }
            p { class: "mt-2 text-sm text-muted-foreground", "The backend returned an authoritative empty notification inventory." }
            a { class: "btn btn-sm btn-outline mt-5", href: page.href(1), "Refresh notifications" }
        }
    }
}

#[component]
fn NotificationProblem(
    state: &'static str,
    title: String,
    detail: String,
    retry_href: String,
) -> Element {
    rsx! {
        section { class: "rounded-2xl border border-amber-500/30 bg-amber-500/5 p-8", role: "alert", "data-admin-notifications-state": state,
            h2 { class: "text-lg font-semibold text-foreground", "{title}" }
            p { class: "mt-2 max-w-3xl text-sm leading-6 text-muted-foreground", "{detail}" }
            nav { class: "mt-5 flex flex-wrap gap-3", aria_label: "Notification recovery",
                a { class: "btn btn-sm btn-outline", href: retry_href, "Try again" }
                a { class: "btn btn-sm btn-ghost", href: "/", "Admin home" }
            }
        }
    }
}

#[component]
fn NotificationCreateState(load: NotificationCreateLoad) -> Element {
    match load {
        NotificationCreateLoad::Form => rsx! {
            NotificationCreateForm {}
        },
        NotificationCreateLoad::Sent(result) => rsx! {
            NotificationCreateOutcome {
                state: ADMIN_NOTIFICATION_CREATE_SENT,
                title: "Notification sent".to_string(),
                detail: "The backend accepted the request and reported successful delivery.".to_string(),
                result,
            }
        },
        NotificationCreateLoad::Pending(result) => rsx! {
            NotificationCreateOutcome {
                state: ADMIN_NOTIFICATION_CREATE_PENDING,
                title: "Notification delivery is pending".to_string(),
                detail: "The backend recorded this idempotent request, but delivery is not complete yet.".to_string(),
                result,
            }
        },
        NotificationCreateLoad::Failed(result) => rsx! {
            NotificationCreateOutcome {
                state: ADMIN_NOTIFICATION_CREATE_FAILED,
                title: "Notification delivery failed".to_string(),
                detail: "The backend recorded the request but did not report successful delivery. No retry is started by this page.".to_string(),
                result,
            }
        },
        NotificationCreateLoad::Forbidden => rsx! {
            NotificationCreateProblem {
                state: ADMIN_NOTIFICATION_CREATE_FORBIDDEN,
                title: "Notification creation was denied".to_string(),
                detail: "The backend did not authorize this session to send notifications.".to_string(),
            }
        },
        NotificationCreateLoad::Conflict => rsx! {
            NotificationCreateProblem {
                state: ADMIN_NOTIFICATION_CREATE_CONFLICT,
                title: "Notification request conflicted".to_string(),
                detail: "The idempotency key is already bound to a different request. No second delivery was attempted.".to_string(),
            }
        },
        NotificationCreateLoad::Invalid => rsx! {
            NotificationCreateProblem {
                state: ADMIN_NOTIFICATION_CREATE_INVALID,
                title: "Notification request was rejected".to_string(),
                detail: "The backend rejected the bounded request or its selected template. No delivery was attempted.".to_string(),
            }
        },
        NotificationCreateLoad::Unavailable => rsx! {
            NotificationCreateProblem {
                state: ADMIN_NOTIFICATION_CREATE_UNAVAILABLE,
                title: "Notification creation is unavailable".to_string(),
                detail: "No authoritative mutation result is available. Recipient selection and delivery actions remain hidden.".to_string(),
            }
        },
        NotificationCreateLoad::Malformed => rsx! {
            NotificationCreateProblem {
                state: ADMIN_NOTIFICATION_CREATE_MALFORMED,
                title: "Notification result could not be verified".to_string(),
                detail: "The backend response did not match the strict mutation acknowledgement contract. No delivery state is shown.".to_string(),
            }
        },
    }
}

#[component]
fn NotificationCreateForm() -> Element {
    let idempotency_key = format!("admin.notification.{}", Uuid::new_v4());
    rsx! {
        section {
            class: "rounded-2xl border border-border/30 bg-card p-8 shadow-xl",
            "data-admin-notifications-state": ADMIN_NOTIFICATION_CREATE_FORM,
            "data-admin-notifications-surface": "create",
            p { class: "text-sm leading-6 text-muted-foreground", "The notification service validates the recipient, content, permission, and idempotency key before recording delivery." }
            form { class: "mt-6 grid gap-5", method: "post", action: "/notifications/create",
                input { type: "hidden", name: "idempotency_key", value: idempotency_key }
                div {
                    label { class: "text-sm font-medium text-foreground", r#for: "notification-recipient", "Recipient wallet" }
                    input { id: "notification-recipient", name: "recipient_wallet_address", required: true, maxlength: "42", class: "mt-2 w-full rounded-xl border border-border/40 bg-background p-3 font-mono text-sm", placeholder: "0x…" }
                }
                div {
                    label { class: "text-sm font-medium text-foreground", r#for: "notification-title", "Title" }
                    input { id: "notification-title", name: "title", required: true, maxlength: "255", class: "mt-2 w-full rounded-xl border border-border/40 bg-background p-3 text-sm" }
                }
                div {
                    label { class: "text-sm font-medium text-foreground", r#for: "notification-message", "Message" }
                    textarea { id: "notification-message", name: "message", required: true, maxlength: "16384", rows: "6", class: "mt-2 w-full rounded-xl border border-border/40 bg-background p-3 text-sm" }
                }
                button { type: "submit", class: "btn btn-primary w-fit", "Send notification" }
            }
        }
    }
}

#[component]
fn NotificationCreateOutcome(
    state: &'static str,
    title: String,
    detail: String,
    result: AdminNotificationCreateResult,
) -> Element {
    let border_class = match state {
        ADMIN_NOTIFICATION_CREATE_SENT => "border-green-500/20",
        ADMIN_NOTIFICATION_CREATE_FAILED => "border-red-500/30",
        _ => "border-amber-500/30",
    };
    rsx! {
        section {
            class: "rounded-2xl border {border_class} bg-card p-8 shadow-xl",
            role: "status",
            "data-admin-notifications-state": state,
            "data-admin-notifications-surface": "create",
            h2 { class: "text-2xl font-semibold text-foreground", "{title}" }
            p { class: "mt-3 text-sm leading-6 text-muted-foreground", "{detail}" }
            dl { class: "mt-6 grid gap-4 border-t border-border/30 pt-6 text-sm sm:grid-cols-2",
                div {
                    dt { class: "text-xs uppercase tracking-wide text-muted-foreground", "Request status" }
                    dd { class: "mt-1 font-semibold text-foreground", "{result.status}" }
                }
                div {
                    dt { class: "text-xs uppercase tracking-wide text-muted-foreground", "Request reference" }
                    dd { class: "mt-1 break-all font-mono text-foreground", "{result.request_id}" }
                }
            }
            nav { class: "mt-8 flex flex-wrap gap-3 border-t border-border/30 pt-6", aria_label: "Notification route recovery",
                a { class: "btn btn-primary", href: NOTIFICATIONS_PATH, "Return to notifications" }
                a { class: "btn btn-outline", href: "/notifications/create", "Create another" }
            }
        }
    }
}

#[component]
fn NotificationCreateProblem(state: &'static str, title: String, detail: String) -> Element {
    rsx! {
        section {
            class: "rounded-2xl border border-amber-500/30 bg-card p-8 shadow-xl",
            role: "alert",
            "data-admin-notifications-state": state,
            "data-admin-notifications-surface": "create",
            h2 { class: "text-2xl font-semibold text-foreground", "{title}" }
            p { class: "mt-3 text-sm leading-6 text-muted-foreground", "{detail}" }
            nav { class: "mt-8 flex flex-wrap gap-3 border-t border-border/30 pt-6", aria_label: "Notification route recovery",
                a { class: "btn btn-primary", href: NOTIFICATIONS_PATH, "Return to notifications" }
                a { class: "btn btn-outline", href: "/", "Admin home" }
            }
        }
    }
}

#[component]
fn NotificationSendFeedback(state: String) -> Element {
    let (class_name, title, detail) = match state.as_str() {
        ADMIN_NOTIFICATIONS_SEND_ACCEPTED => (
            "border-green-500/30 bg-green-500/10 text-green-900 dark:text-green-200",
            "Notification queued",
            "The backend accepted the canonical-wallet notification request. Delivery remains asynchronous.",
        ),
        ADMIN_NOTIFICATIONS_SEND_ERROR => (
            "border-red-500/30 bg-red-500/10 text-red-900 dark:text-red-200",
            "Notification was not queued",
            "The backend rejected or could not complete the request. No delivery success is being claimed.",
        ),
        _ => return rsx! {},
    };
    rsx! {
        section { class: "mb-6 rounded-xl border p-4 {class_name}", role: "status", "data-admin-notifications-send-state": state,
            h2 { class: "font-semibold", "{title}" }
            p { class: "mt-1 text-sm", "{detail}" }
        }
    }
}

#[component]
fn NotificationMetricsPanel(snapshot: AdminNotificationMetrics) -> Element {
    rsx! {
        section {
            class: "mb-6 rounded-2xl border border-border/30 bg-card p-6 shadow-lg",
            role: "status",
            "data-admin-notification-metrics": ADMIN_NOTIFICATIONS_READY,
            h2 { class: "text-lg font-semibold text-foreground", "Operational queue snapshot" }
            p { class: "mt-1 text-xs text-muted-foreground", "Backend counters are observations only; they do not claim delivery of an individual notification." }
            div { class: "mt-5 grid gap-3 sm:grid-cols-2 lg:grid-cols-4",
                NotificationMetricValue { label: "Queued", value: snapshot.queue_depth }
                NotificationMetricValue { label: "Retry wait", value: snapshot.retry_wait }
                NotificationMetricValue { label: "Attempting", value: snapshot.attempting }
                NotificationMetricValue { label: "Provider accepted", value: snapshot.provider_accepted }
                NotificationMetricValue { label: "Terminal failed", value: snapshot.terminal_failed }
                NotificationMetricValue { label: "Dead lettered", value: snapshot.dead_lettered }
                NotificationMetricValue { label: "Suppressed", value: snapshot.suppressed }
                NotificationMetricValue { label: "Active streams", value: snapshot.active_streams }
            }
        }
    }
}

#[component]
fn NotificationMetricValue(label: &'static str, value: i64) -> Element {
    rsx! {
        div { class: "rounded-xl border border-border/30 bg-muted/20 p-3",
            p { class: "text-xs text-muted-foreground", "{label}" }
            p { class: "mt-1 text-xl font-semibold tabular-nums text-foreground", "{value}" }
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
            id: "notification-session".to_string(),
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

    fn notification() -> AdminNotificationSummary {
        AdminNotificationSummary {
            id: "notification-01".to_string(),
            title: Some("Production migration update".to_string()),
            subject: Some("Read-only inventory is live".to_string()),
            channel: "in_app".to_string(),
            status: "sent".to_string(),
            notification_type: Some("system".to_string()),
            priority: Some("high".to_string()),
            sent_at: Some("2026-07-22T10:00:00Z".to_string()),
            created_at: "2026-07-22T09:00:00Z".to_string(),
        }
    }

    fn projection(
        items: Vec<AdminNotificationSummary>,
        total: i64,
        offset: i64,
    ) -> AdminNotificationList {
        AdminNotificationList {
            items,
            total,
            limit: NOTIFICATION_PAGE_LIMIT,
            offset,
        }
    }

    fn metrics() -> AdminNotificationMetrics {
        AdminNotificationMetrics {
            queue_depth: 4,
            queue_age_seconds: Some(2),
            suppressed: 1,
            retry_wait: 1,
            terminal_failed: 0,
            dead_lettered: 0,
            provider_accepted: 3,
            attempting: 1,
            channel_outcomes: BTreeMap::from([(String::from("in_app"), 4)]),
            provider_events: 3,
            delivery_attempts: 4,
            replay_cursors: 2,
            replay_cursor_age_seconds: Some(1),
            active_streams: 1,
            stream_connections_total: 2,
            stream_reconnects_total: 1,
            stream_replayed_events_total: 1,
            stream_lag_seconds: Some(1),
            stream_query_failures_total: 0,
        }
    }

    fn ctx(state: &str, projection: Option<AdminNotificationList>, page: i64) -> PageContext {
        let mut params = HashMap::from([
            (
                ADMIN_NOTIFICATIONS_STATE_PARAM.to_string(),
                state.to_string(),
            ),
            (ADMIN_NOTIFICATIONS_PAGE_PARAM.to_string(), page.to_string()),
        ]);
        if let Some(projection) = projection {
            params.insert(
                ADMIN_NOTIFICATIONS_DATA_PARAM.to_string(),
                serde_json::to_string(&projection).unwrap(),
            );
        }
        PageContext {
            user: Some(session()),
            path: NOTIFICATIONS_PATH.to_string(),
            params,
            ..Default::default()
        }
    }

    fn html(ctx: &PageContext) -> String {
        dioxus_ssr::render_element(render_manage(ctx).1)
    }

    #[test]
    fn signed_out_routes_hide_projection_and_dynamic_params() {
        let mut manage = PageContext {
            path: NOTIFICATIONS_PATH.to_string(),
            ..Default::default()
        };
        manage.params.insert(
            ADMIN_NOTIFICATIONS_DATA_PARAM.to_string(),
            "PRIVATE_NOTIFICATION_PAYLOAD".to_string(),
        );
        manage.params.insert(
            ADMIN_NOTIFICATIONS_PAGE_PARAM.to_string(),
            "PRIVATE_PAGE".to_string(),
        );
        let create = PageContext {
            path: "/notifications/create".to_string(),
            ..Default::default()
        };

        for rendered in [
            html(&manage),
            dioxus_ssr::render_element(render_create(&create).1),
        ] {
            assert!(rendered.contains("Sign in required"));
            assert!(!rendered.contains("data-admin-notifications-state"));
            assert!(!rendered.contains("PRIVATE_NOTIFICATION_PAYLOAD"));
            assert!(!rendered.contains("PRIVATE_PAGE"));
            assert!(!rendered.contains("Notification creation is unavailable"));
        }
    }

    #[test]
    fn ready_projection_renders_escaped_inventory_with_backend_actions_without_private_fields() {
        let mut hostile = notification();
        hostile.id = "private-notification-id".to_string();
        hostile.title = Some("<script>alert(1)</script>".to_string());
        hostile.subject = Some("<b>subject</b>".to_string());
        let rendered = html(&ctx(
            ADMIN_NOTIFICATIONS_READY,
            Some(projection(vec![hostile], 1, 0)),
            1,
        ));

        assert!(rendered.contains("data-admin-notifications-state=\"ready\""));
        assert!(rendered.contains("&#60;script&#62;alert(1)&#60;/script&#62;"));
        assert!(rendered.contains("&#60;b&#62;subject&#60;/b&#62;"));
        assert!(!rendered.contains("<script>alert(1)</script>"));
        assert!(rendered.contains("1 authoritative records"));
        assert!(rendered.contains("in_app"));
        assert!(rendered.contains("2026-07-22T10:00:00Z"));
        assert!(rendered.contains("2026-07-22T09:00:00Z"));
        assert!(rendered.contains("private-notification-id"));
        assert!(rendered.contains("name=\"action\" value=\"delete\""));

        for private in [
            "recipient_wallet_address",
            "user_id",
            "body",
            "message",
            "data_payload",
            "delivery error",
            "read_at",
            "action_url",
        ] {
            assert!(
                !rendered.contains(private),
                "private field leaked: {private}"
            );
        }
    }

    #[test]
    fn empty_forbidden_unavailable_and_malformed_states_are_distinct() {
        let empty = html(&ctx(
            ADMIN_NOTIFICATIONS_EMPTY,
            Some(projection(vec![], 0, 0)),
            1,
        ));
        let forbidden = html(&ctx(ADMIN_NOTIFICATIONS_FORBIDDEN, None, 1));
        let unavailable = html(&ctx(ADMIN_NOTIFICATIONS_UNAVAILABLE, None, 1));
        let malformed = html(&ctx(ADMIN_NOTIFICATIONS_MALFORMED, None, 1));

        assert!(empty.contains("data-admin-notifications-state=\"empty\""));
        assert!(empty.contains("No notifications found"));
        assert!(forbidden.contains("data-admin-notifications-state=\"forbidden\""));
        assert!(forbidden.contains("Notification access was denied"));
        assert!(unavailable.contains("data-admin-notifications-state=\"unavailable\""));
        assert!(unavailable.contains("Notification records are unavailable"));
        assert!(malformed.contains("data-admin-notifications-state=\"malformed\""));
        assert!(malformed.contains("Notification data could not be verified"));
    }

    #[test]
    fn metrics_projection_is_strict_bounded_and_truthful() {
        let valid = serde_json::to_value(metrics()).unwrap();
        assert!(decode_admin_notification_metrics(valid.clone()).is_some());

        let mut unknown = valid.clone();
        unknown["recipient"] = serde_json::json!("private");
        assert!(decode_admin_notification_metrics(unknown).is_none());

        let mut negative = valid.clone();
        negative["queue_depth"] = serde_json::json!(-1);
        assert!(decode_admin_notification_metrics(negative).is_none());

        let mut too_many_streams = valid.clone();
        too_many_streams["active_streams"] = serde_json::json!(257);
        assert!(decode_admin_notification_metrics(too_many_streams).is_none());

        let mut unknown_channel = valid.clone();
        unknown_channel["channel_outcomes"] = serde_json::json!({"sms": 1});
        assert!(decode_admin_notification_metrics(unknown_channel).is_none());

        let mut page = ctx(ADMIN_NOTIFICATIONS_EMPTY, Some(projection(vec![], 0, 0)), 1);
        page.params.insert(
            ADMIN_NOTIFICATION_METRICS_STATE_PARAM.to_string(),
            ADMIN_NOTIFICATIONS_READY.to_string(),
        );
        page.params.insert(
            ADMIN_NOTIFICATION_METRICS_DATA_PARAM.to_string(),
            serde_json::to_string(&metrics()).unwrap(),
        );
        let rendered = html(&page);
        assert!(rendered.contains("Operational queue snapshot"));
        assert!(rendered.contains("Provider accepted"));
        assert!(rendered.contains("Backend counters are observations only"));
        assert!(!rendered.contains("recipient"));
    }

    #[test]
    fn decoder_rejects_unknown_fields_bounds_enums_dates_and_impossible_counts() {
        let valid = serde_json::to_value(projection(vec![notification()], 1, 0)).unwrap();
        assert!(decode_admin_notification_projection(valid.clone()).is_some());

        let mut cases = Vec::new();
        let mut unknown = valid.clone();
        unknown["unexpected"] = serde_json::json!(true);
        cases.push(unknown);
        let mut nested_unknown = valid.clone();
        nested_unknown["items"][0]["recipient"] = serde_json::json!("private");
        cases.push(nested_unknown);
        let mut id = valid.clone();
        id["items"][0]["id"] = serde_json::json!("x".repeat(MAX_ID_CHARS + 1));
        cases.push(id);
        let mut title = valid.clone();
        title["items"][0]["title"] = serde_json::json!("x".repeat(MAX_TITLE_CHARS + 1));
        cases.push(title);
        let mut control = valid.clone();
        control["items"][0]["subject"] = serde_json::json!("line one\nline two");
        cases.push(control);
        let mut channel = valid.clone();
        channel["items"][0]["channel"] = serde_json::json!("Push Email");
        cases.push(channel);
        let mut status = valid.clone();
        status["items"][0]["status"] = serde_json::json!("deleted");
        cases.push(status);
        let mut priority = valid.clone();
        priority["items"][0]["priority"] = serde_json::json!("highest");
        cases.push(priority);
        let mut timestamp = valid.clone();
        timestamp["items"][0]["created_at"] = serde_json::json!("yesterday");
        cases.push(timestamp);
        let mut limit = valid.clone();
        limit["limit"] = serde_json::json!(50);
        cases.push(limit);
        let mut offset = valid.clone();
        offset["offset"] = serde_json::json!(1);
        cases.push(offset);
        let mut total = valid;
        total["total"] = serde_json::json!(0);
        cases.push(total);
        cases.push(serde_json::to_value(projection(vec![notification(); 20], 21, 20)).unwrap());

        for malformed in cases {
            assert!(decode_admin_notification_projection(malformed).is_none());
        }
    }

    #[test]
    fn state_payload_and_requested_page_must_agree() {
        let empty_as_ready = html(&ctx(
            ADMIN_NOTIFICATIONS_READY,
            Some(projection(vec![], 0, 0)),
            1,
        ));
        let records_as_empty = html(&ctx(
            ADMIN_NOTIFICATIONS_EMPTY,
            Some(projection(vec![notification()], 1, 0)),
            1,
        ));
        let wrong_offset = html(&ctx(
            ADMIN_NOTIFICATIONS_READY,
            Some(projection(vec![notification()], 21, 20)),
            1,
        ));
        let impossible_nonempty_page = html(&ctx(
            ADMIN_NOTIFICATIONS_READY,
            Some(projection(vec![notification()], 20, 20)),
            2,
        ));

        for rendered in [
            empty_as_ready,
            records_as_empty,
            wrong_offset,
            impossible_nonempty_page,
        ] {
            assert!(rendered.contains("data-admin-notifications-state=\"malformed\""));
        }
    }

    #[test]
    fn nonzero_total_empty_out_of_range_page_is_ready_with_recovery() {
        let rendered = html(&ctx(
            ADMIN_NOTIFICATIONS_READY,
            Some(projection(vec![], 21, 40)),
            3,
        ));

        assert!(rendered.contains("data-admin-notifications-state=\"ready\""));
        assert!(rendered.contains("data-admin-notifications-page-state=\"out-of-range\""));
        assert!(rendered.contains("21 authoritative records"));
        assert!(rendered.contains("No notifications on this page"));
        assert!(rendered.contains("href=\"/notifications/manage?page=1\""));
        assert!(!rendered.contains("No notifications found"));
    }

    #[test]
    fn native_pagination_and_retry_preserve_the_page() {
        let page_two = html(&ctx(
            ADMIN_NOTIFICATIONS_READY,
            Some(projection(vec![notification()], 41, 20)),
            2,
        ));
        assert!(page_two.contains("aria-label=\"Notification pagination\""));
        assert!(page_two.contains("href=\"/notifications/manage?page=1\""));
        assert!(page_two.contains("href=\"/notifications/manage?page=3\""));
        assert!(page_two.contains("Mark read"));
        assert!(page_two.contains("Delete"));

        let unavailable = html(&ctx(ADMIN_NOTIFICATIONS_UNAVAILABLE, None, 2));
        assert!(unavailable.contains("href=\"/notifications/manage?page=2\""));
    }

    #[test]
    fn status_badges_use_light_and_dark_theme_contrast_classes() {
        assert_eq!(
            notification_status_class("sent"),
            "border-green-500/30 bg-green-500/10 text-green-800 dark:text-green-300"
        );
        assert_eq!(
            notification_status_class("failed"),
            "border-red-500/30 bg-red-500/10 text-red-800 dark:text-red-300"
        );
        assert_eq!(
            notification_status_class("suppressed"),
            "border-amber-500/30 bg-amber-500/10 text-amber-800 dark:text-amber-300"
        );
        assert_eq!(
            notification_status_class("pending"),
            "border-amber-500/30 bg-amber-500/10 text-amber-900 dark:text-amber-300"
        );
    }

    #[test]
    fn manage_and_create_expose_only_supported_surfaces_without_samples() {
        let manage = html(&ctx(
            ADMIN_NOTIFICATIONS_READY,
            Some(projection(vec![notification()], 1, 0)),
            1,
        ));
        let create = dioxus_ssr::render_element(
            render_create(&create_ctx(ADMIN_NOTIFICATION_CREATE_FORM, None)).1,
        );
        assert!(create.contains("data-admin-notifications-surface=\"create\""));
        assert!(create.contains("data-admin-notifications-state=\"form\""));
        assert!(create.contains("method=\"post\""));
        assert!(create.contains("name=\"idempotency_key\""));

        for rendered in [manage] {
            for forbidden in [
                "Search notifications",
                "Delivery analytics",
                "Total Sent",
                "Today's Pulse",
                "Welcome to the platform",
                "New feature: charts",
                "Maintenance window",
                "Send notification",
                "Purge",
                "Clear all",
            ] {
                assert!(
                    !rendered.contains(forbidden),
                    "unsupported notification surface leaked: {forbidden}"
                );
            }
        }
        for supported in [
            "Recipient wallet",
            "name=\"recipient_wallet_address\"",
            "name=\"title\"",
            "name=\"message\"",
            "Send notification",
        ] {
            assert!(create.contains(supported), "{supported}");
        }
        for unsupported in ["Global Broadcast", "Action URL", "Asset URL"] {
            assert!(!create.contains(unsupported), "{unsupported}");
        }
    }

    #[test]
    fn send_feedback_is_closed_and_never_claims_provider_delivery() {
        let mut accepted = ctx(
            ADMIN_NOTIFICATIONS_READY,
            Some(projection(vec![notification()], 1, 0)),
            1,
        );
        accepted.params.insert(
            ADMIN_NOTIFICATIONS_SEND_STATE_PARAM.to_string(),
            ADMIN_NOTIFICATIONS_SEND_ACCEPTED.to_string(),
        );
        let accepted_html = html(&accepted);
        assert!(accepted_html.contains("data-admin-notifications-send-state=\"accepted\""));
        assert!(accepted_html.contains("Delivery remains asynchronous"));
        assert!(!accepted_html.contains("delivered successfully"));

        accepted.params.insert(
            ADMIN_NOTIFICATIONS_SEND_STATE_PARAM.to_string(),
            ADMIN_NOTIFICATIONS_SEND_ERROR.to_string(),
        );
        let error_html = html(&accepted);
        assert!(error_html.contains("data-admin-notifications-send-state=\"error\""));
        assert!(error_html.contains("No delivery success is being claimed"));

        accepted.params.insert(
            ADMIN_NOTIFICATIONS_SEND_STATE_PARAM.to_string(),
            "forged".to_string(),
        );
        assert!(!html(&accepted).contains("data-admin-notifications-send-state"));
    }

    fn create_ctx(state: &str, result: Option<AdminNotificationCreateResult>) -> PageContext {
        let mut params = HashMap::from([(
            ADMIN_NOTIFICATION_CREATE_STATE_PARAM.to_string(),
            state.to_string(),
        )]);
        if let Some(result) = result {
            params.insert(
                ADMIN_NOTIFICATION_CREATE_DATA_PARAM.to_string(),
                serde_json::to_string(&result).unwrap(),
            );
        }
        PageContext {
            user: Some(session()),
            path: "/notifications/create".to_string(),
            params,
            ..Default::default()
        }
    }

    fn create_result(status: &str, delivered: bool) -> AdminNotificationCreateResult {
        AdminNotificationCreateResult {
            id: "idem_admin-send-01".to_string(),
            status: status.to_string(),
            delivered,
            request_id: "request-01".to_string(),
        }
    }

    #[test]
    fn create_acknowledgement_is_strict_and_state_matches_backend_status() {
        let sent = serde_json::to_value(create_result("sent", true)).unwrap();
        assert!(decode_admin_notification_create_result(sent.clone()).is_some());

        let mut unknown = sent.clone();
        unknown["error"] = serde_json::json!("smtp-secret");
        assert!(decode_admin_notification_create_result(unknown).is_none());

        let mut dishonest = sent;
        dishonest["delivered"] = serde_json::json!(false);
        assert!(decode_admin_notification_create_result(dishonest).is_none());

        let sent_html = dioxus_ssr::render_element(
            render_create(&create_ctx(
                ADMIN_NOTIFICATION_CREATE_SENT,
                Some(create_result("sent", true)),
            ))
            .1,
        );
        assert!(sent_html.contains("data-admin-notifications-state=\"sent\""));
        assert!(sent_html.contains("Notification sent"));
        assert!(sent_html.contains("request-01"));

        let failed_html = dioxus_ssr::render_element(
            render_create(&create_ctx(
                ADMIN_NOTIFICATION_CREATE_FAILED,
                Some(create_result("failed", false)),
            ))
            .1,
        );
        assert!(failed_html.contains("data-admin-notifications-state=\"failed\""));
        assert!(failed_html.contains("Notification delivery failed"));
    }

    #[test]
    fn create_conflict_invalid_unavailable_and_malformed_states_never_show_a_form() {
        for state in [
            ADMIN_NOTIFICATION_CREATE_CONFLICT,
            ADMIN_NOTIFICATION_CREATE_INVALID,
            ADMIN_NOTIFICATION_CREATE_UNAVAILABLE,
            ADMIN_NOTIFICATION_CREATE_MALFORMED,
        ] {
            let rendered = dioxus_ssr::render_element(render_create(&create_ctx(state, None)).1);
            assert!(rendered.contains(&format!("data-admin-notifications-state=\"{state}\"")));
            assert!(!rendered.contains("<form"));
            assert!(!rendered.contains("<input"));
            assert!(!rendered.contains("smtp-secret"));
        }
    }
}

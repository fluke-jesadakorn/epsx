//! Read-only admin notification inventory plus a fail-closed compose route.
//!
//! The management page renders only a strict backend projection. Recipient
//! identity, message content, delivery errors, read state, action URLs, and all
//! mutations remain backend concerns; this leaf applies only the authenticated
//! session boundary.

use chrono::DateTime;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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

pub const ADMIN_NOTIFICATIONS_READY: &str = "ready";
pub const ADMIN_NOTIFICATIONS_EMPTY: &str = "empty";
pub const ADMIN_NOTIFICATIONS_FORBIDDEN: &str = "forbidden";
pub const ADMIN_NOTIFICATIONS_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_NOTIFICATIONS_MALFORMED: &str = "malformed";
pub const ADMIN_NOTIFICATIONS_SEND_STATE_PARAM: &str = "data_admin_notifications_send_state";
pub const ADMIN_NOTIFICATIONS_SEND_ACCEPTED: &str = "accepted";
pub const ADMIN_NOTIFICATIONS_SEND_ERROR: &str = "error";
pub const ADMIN_NOTIFICATIONS_METRICS_PARAM: &str = "data_admin_notifications_metrics";
pub const ADMIN_NOTIFICATIONS_METRICS_STATE_PARAM: &str = "data_admin_notifications_metrics_state";
pub const ADMIN_NOTIFICATIONS_METRICS_READY: &str = "ready";
pub const ADMIN_NOTIFICATIONS_METRICS_FORBIDDEN: &str = "forbidden";
pub const ADMIN_NOTIFICATIONS_METRICS_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_NOTIFICATIONS_METRICS_MALFORMED: &str = "malformed";

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
                "pending" | "sent" | "failed" | "suppressed" | "expired"
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NotificationPage {
    page: i64,
}

impl NotificationPage {
    fn from_ctx(ctx: &PageContext) -> Self {
        let page = ctx
            .params
            .get(ADMIN_NOTIFICATIONS_PAGE_PARAM)
            .and_then(|value| value.parse::<i64>().ok())
            .filter(|page| (1..=50_001).contains(page))
            .unwrap_or(1);
        Self { page }
    }

    fn expected_offset(self) -> Option<i64> {
        self.page
            .checked_sub(1)?
            .checked_mul(NOTIFICATION_PAGE_LIMIT)
    }

    fn href(self, page: i64) -> String {
        format!("{NOTIFICATIONS_PATH}?page={}", page.clamp(1, 50_001))
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

/// `/notifications/manage` — authenticated, read-only global inventory.
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

/// `/notifications/create` — authenticated canonical-wallet compose form.
/// Broadcast, plan, scheduling, image, and arbitrary-data controls stay out of
/// this surface until their source-compatible backend contracts are complete.
pub fn render_create(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("New notification");
    (
        meta,
        rsx! {
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("creating notifications".to_string()),
                return_url: Some(NOTIFICATIONS_PATH.to_string()),
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
                    NotificationCreateForm {}
                }
            }
        },
    )
}

#[component]
fn RenderNotificationList(ctx: PageContext) -> Element {
    let page = NotificationPage::from_ctx(&ctx);
    let load = notification_load(&ctx, page);

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
            NotificationMetricsState { ctx: ctx.clone() }
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
                        detail: "The notification backend response did not match the read-only inventory contract. No records are being shown.".to_string(),
                        retry_href: page.href(page.page),
                    }
                },
            }
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
fn NotificationCreateForm() -> Element {
    rsx! {
        section {
            class: "relative overflow-hidden rounded-2xl border border-border/20 bg-card p-1 shadow-2xl",
            "data-admin-notifications-state": "create",
            "data-admin-notifications-surface": "create",
            div { class: "relative rounded-2xl bg-card/60 p-8 sm:p-12",
                div { class: "mx-auto max-w-4xl",
                    div { class: "mb-12",
                        h2 { class: "text-3xl font-black uppercase tracking-tight text-foreground", "Signal Generator" }
                        p { class: "mt-2 text-sm font-bold text-muted-foreground", "Construct and transmit high-priority system alerts" }
                    }
                    p { class: "mb-8 max-w-3xl text-sm leading-6 text-muted-foreground", "This Rust-owned form queues one notification for one canonical wallet. The backend validates authorization, ownership, content bounds, and delivery state before writing." }
                    form { class: "space-y-10", method: "post", action: "/notifications/create",
                        fieldset { class: "space-y-4", aria_label: "Transmission logic",
                            legend { class: "ml-2 text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground", "Transmission Logic" }
                            div { class: "grid grid-cols-1 gap-4 sm:grid-cols-2",
                                button { class: "flex items-center gap-6 rounded-xl border border-[#1fc7d4] bg-[#1fc7d4]/10 p-6 text-left shadow-[0_0_20px_rgba(31,199,212,0.1)]", r#type: "button", aria_pressed: "true",
                                    div { class: "flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-[#1fc7d4] text-white", Icon { name: "user".to_string(), size: Some(24) } }
                                    div {
                                        div { class: "text-sm font-black uppercase tracking-tight text-foreground", "Targeted Client" }
                                        div { class: "text-[10px] font-bold uppercase text-muted-foreground opacity-60", "Single Node Access" }
                                    }
                                }
                                button { class: "flex cursor-not-allowed items-center gap-6 rounded-xl border border-border/40 bg-muted/30 p-6 text-left opacity-50", r#type: "button", disabled: true, aria_disabled: "true",
                                    div { class: "flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-muted/50 text-muted-foreground/40", Icon { name: "users".to_string(), size: Some(24) } }
                                    div {
                                        div { class: "text-sm font-black uppercase tracking-tight text-foreground", "Global Broadcast" }
                                        div { class: "text-[10px] font-bold uppercase text-muted-foreground", "Unavailable until backend contract" }
                                    }
                                }
                            }
                        }
                        div { class: "grid grid-cols-1 gap-x-10 gap-y-8 lg:grid-cols-2",
                            label { class: "block space-y-3 lg:col-span-2",
                                span { class: "ml-2 text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground", "Destination Node" }
                                div { class: "relative",
                                    input { class: "input h-14 w-full rounded-2xl bg-muted/30 px-6 font-mono text-sm", r#type: "text", name: "recipient_wallet_address", required: true, maxlength: "42", autocomplete: "off", placeholder: "0x..." }
                                    span { class: "pointer-events-none absolute right-4 top-1/2 -translate-y-1/2 text-[10px] font-black uppercase text-muted-foreground/40", "Required" }
                                }
                            }
                            label { class: "block space-y-3",
                                span { class: "ml-2 text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground", "Classification" }
                                select { class: "input h-14 w-full rounded-2xl bg-muted/30 px-6 text-sm font-black uppercase tracking-widest", disabled: true, aria_disabled: "true",
                                    option { selected: true, "System Alert" }
                                    option { "Security Event" }
                                    option { "Permission Auth" }
                                    option { "Payment Transaction" }
                                    option { "General Message" }
                                }
                            }
                            label { class: "block space-y-3",
                                span { class: "ml-2 text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground", "Priority Vector" }
                                select { class: "input h-14 w-full rounded-2xl bg-muted/30 px-6 text-sm font-black uppercase tracking-widest", disabled: true, aria_disabled: "true",
                                    option { selected: true, "Normal Operation" }
                                    option { "Low Clearance" }
                                    option { "High Priority" }
                                    option { "Critical Override" }
                                }
                            }
                            label { class: "block space-y-3 lg:col-span-2",
                                span { class: "ml-2 text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground", "Subject Heading" }
                                input { class: "input h-14 w-full rounded-2xl bg-muted/30 px-6 text-sm font-bold", r#type: "text", name: "title", required: true, maxlength: "255", placeholder: "Payload designation..." }
                            }
                            label { class: "block space-y-3 lg:col-span-2",
                                span { class: "ml-2 text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground", "Message Payload" }
                                textarea { class: "input min-h-40 w-full resize-none rounded-2xl bg-muted/30 p-6 text-sm font-bold", name: "message", required: true, maxlength: "16384", placeholder: "Enter transmission data..." }
                            }
                            label { class: "block space-y-3 opacity-50",
                                span { class: "ml-2 text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground", "Action URL" }
                                input { class: "input h-12 w-full rounded-2xl bg-muted/30 px-5 text-xs", r#type: "url", disabled: true, aria_disabled: "true", placeholder: "Unavailable until action contract is verified" }
                            }
                            label { class: "block space-y-3 opacity-50",
                                span { class: "ml-2 text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground", "Asset URL" }
                                input { class: "input h-12 w-full rounded-2xl bg-muted/30 px-5 text-xs", r#type: "url", disabled: true, aria_disabled: "true", placeholder: "Image upload unavailable" }
                            }
                        }
                        p { class: "text-xs text-muted-foreground", "Broadcast, plan targeting, scheduling, images, and arbitrary payloads are intentionally unavailable until their contracts are verified." }
                        div { class: "flex items-center gap-6 border-t border-border/20 pt-10",
                            a { class: "btn btn-ghost flex-1 rounded-2xl py-7 text-[10px] font-black uppercase tracking-[0.2em] opacity-60", href: NOTIFICATIONS_PATH, "Abort" }
                            button { class: "btn btn-primary flex-[2] rounded-xl bg-gradient-to-r from-[#7645d9] to-[#5a33b8] py-7 font-black uppercase tracking-[0.2em]", r#type: "submit", "Queue Notification" }
                        }
                    }
                }
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
fn NotificationMetricsState(ctx: PageContext) -> Element {
    let Some(state) = ctx.params.get(ADMIN_NOTIFICATIONS_METRICS_STATE_PARAM) else {
        return rsx! {};
    };
    match state.as_str() {
        ADMIN_NOTIFICATIONS_METRICS_READY => {
            let metrics = ctx
                .params
                .get(ADMIN_NOTIFICATIONS_METRICS_PARAM)
                .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok())
                .and_then(decode_admin_notification_metrics);
            match metrics {
                Some(metrics) => rsx! { NotificationMetricsPanel { snapshot: metrics } },
                None => rsx! {
                    NotificationMetricsProblem { state: ADMIN_NOTIFICATIONS_METRICS_MALFORMED.to_string() }
                },
            }
        }
        ADMIN_NOTIFICATIONS_METRICS_FORBIDDEN
        | ADMIN_NOTIFICATIONS_METRICS_UNAVAILABLE
        | ADMIN_NOTIFICATIONS_METRICS_MALFORMED => {
            rsx! { NotificationMetricsProblem { state: state.clone() } }
        }
        _ => rsx! {
            NotificationMetricsProblem { state: ADMIN_NOTIFICATIONS_METRICS_MALFORMED.to_string() }
        },
    }
}

#[component]
fn NotificationMetricsPanel(snapshot: AdminNotificationMetrics) -> Element {
    rsx! {
        section {
            class: "mb-6 rounded-2xl border border-border/30 bg-card p-6 shadow-lg",
            role: "status",
            "data-admin-notifications-metrics-state": ADMIN_NOTIFICATIONS_METRICS_READY,
            h2 { class: "text-lg font-semibold text-foreground", "Operational queue snapshot" }
            p { class: "mt-1 text-xs text-muted-foreground", "Backend counters are observations only; they do not claim delivery of an individual notification." }
            div { class: "mt-5 grid gap-3 sm:grid-cols-2 lg:grid-cols-4",
                NotificationMetric { label: "Queued", value: snapshot.queue_depth }
                NotificationMetric { label: "Retry wait", value: snapshot.retry_wait }
                NotificationMetric { label: "Attempting", value: snapshot.attempting }
                NotificationMetric { label: "Provider accepted", value: snapshot.provider_accepted }
                NotificationMetric { label: "Terminal failed", value: snapshot.terminal_failed }
                NotificationMetric { label: "Dead lettered", value: snapshot.dead_lettered }
                NotificationMetric { label: "Suppressed", value: snapshot.suppressed }
                NotificationMetric { label: "Active streams", value: snapshot.active_streams }
            }
        }
    }
}

#[component]
fn NotificationMetric(label: &'static str, value: i64) -> Element {
    rsx! {
        div { class: "rounded-xl border border-border/30 bg-muted/20 p-3",
            p { class: "text-xs text-muted-foreground", "{label}" }
            p { class: "mt-1 text-xl font-semibold tabular-nums text-foreground", "{value}" }
        }
    }
}

#[component]
fn NotificationMetricsProblem(state: String) -> Element {
    let detail = match state.as_str() {
        ADMIN_NOTIFICATIONS_METRICS_FORBIDDEN => {
            "The backend did not authorize the operational snapshot. No counters are shown."
        }
        ADMIN_NOTIFICATIONS_METRICS_MALFORMED => {
            "The backend response did not match the bounded metrics contract. No counters are shown."
        }
        _ => "The operational snapshot is unavailable. No counters are shown.",
    };
    rsx! {
        section {
            class: "mb-6 rounded-xl border border-amber-500/30 bg-amber-500/5 p-4",
            role: "status",
            "data-admin-notifications-metrics-state": state,
            h2 { class: "font-semibold text-foreground", "Operational snapshot unavailable" }
            p { class: "mt-1 text-sm text-muted-foreground", "{detail}" }
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
    fn ready_projection_renders_escaped_read_only_inventory_without_private_fields() {
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
        assert!(!rendered.contains("private-notification-id"));

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
            ADMIN_NOTIFICATIONS_METRICS_STATE_PARAM.to_string(),
            ADMIN_NOTIFICATIONS_METRICS_READY.to_string(),
        );
        page.params.insert(
            ADMIN_NOTIFICATIONS_METRICS_PARAM.to_string(),
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
        assert!(!page_two.contains("<button"));

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
    fn manage_and_create_expose_only_the_supported_canonical_wallet_form() {
        let manage = html(&ctx(
            ADMIN_NOTIFICATIONS_READY,
            Some(projection(vec![notification()], 1, 0)),
            1,
        ));
        let create = dioxus_ssr::render_element(
            render_create(&ctx(ADMIN_NOTIFICATIONS_UNAVAILABLE, None, 1)).1,
        );
        assert!(create.contains("data-admin-notifications-surface=\"create\""));
        assert!(create.contains("method=\"post\""));
        assert!(create.contains("action=\"/notifications/create\""));
        for supported in [
            "name=\"recipient_wallet_address\"",
            "name=\"title\"",
            "name=\"message\"",
            "Queue Notification",
            "Signal Generator",
            "Transmission Logic",
            "Destination Node",
            "Subject Heading",
            "Message Payload",
        ] {
            assert!(
                create.contains(supported),
                "supported form field missing: {supported}"
            );
        }

        for rendered in [manage, create.clone()] {
            for forbidden in [
                "name=\"broadcast\"",
                "name=\"plan_id\"",
                "name=\"schedule\"",
                "name=\"image_url\"",
                "name=\"action_url\"",
                "name=\"data\"",
                "Search notifications",
                "Delivery analytics",
                "Total Sent",
                "Today's Pulse",
                "Welcome to the platform",
                "New feature: charts",
                "Maintenance window",
                "Delete",
                "Purge",
                "Mark read",
                "Mark unread",
                "Clear all",
            ] {
                assert!(
                    !rendered.contains(forbidden),
                    "unsupported notification surface leaked: {forbidden}"
                );
            }
        }
        assert!(create.contains("Global Broadcast"));
        assert!(create.contains("disabled"));
        assert!(create.contains("Action URL"));
        assert!(create.contains("Asset URL"));
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
}

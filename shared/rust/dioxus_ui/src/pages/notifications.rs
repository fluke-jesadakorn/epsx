//! Truthful, read-only owner notification center.
//!
//! The frontend BFF hydrates this page from the extracted notification
//! service's owner-scoped `GET /api/v1/notification/list` route. This page
//! deliberately does not expose notification mutations, action URLs, browser
//! permission simulation, push controls, or preference controls while their
//! backend lifecycle contracts remain blocked.

use chrono::{DateTime, SecondsFormat, Utc};
use dioxus::prelude::*;
use serde::Deserialize;

use crate::auth::AuthGate;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::primitives::*;

use super::{PageContext, PageMeta};

const NOTIFICATIONS_DATA_PARAM: &str = "data_notifications";
const NOTIFICATIONS_STATE_PARAM: &str = "data_notifications_state";

/// A wire field that must be present but may explicitly contain JSON `null`.
/// Serde otherwise gives missing fields and explicit `null` the same `None`
/// representation. The sentinel keeps those states distinct until the whole
/// service row is validated.
#[derive(Debug)]
enum RequiredNullable<T> {
    Missing,
    Present(Option<T>),
}

impl<T> Default for RequiredNullable<T> {
    fn default() -> Self {
        Self::Missing
    }
}

impl<'de, T> Deserialize<'de> for RequiredNullable<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Option::<T>::deserialize(deserializer).map(Self::Present)
    }
}

impl<T> RequiredNullable<T> {
    fn require(self) -> Result<Option<T>, ()> {
        match self {
            Self::Missing => Err(()),
            Self::Present(value) => Ok(value),
        }
    }
}

/// Exact read fields emitted by `services/notification/src/main.rs`.
///
/// Delivery, recipient, provider, and arbitrary data fields are intentionally
/// ignored by this read-only UI. `action_url` is parsed so schema drift is
/// visible in tests, but it is never copied into the render model because its
/// allowlist policy is not yet locked.
#[derive(Debug, Deserialize)]
struct ServiceNotification {
    id: String,
    #[serde(default)]
    subject: RequiredNullable<String>,
    body: String,
    created_at: DateTime<Utc>,
    #[serde(default)]
    read_at: RequiredNullable<DateTime<Utc>>,
    #[serde(default)]
    title: RequiredNullable<String>,
    #[serde(default)]
    notification_type: RequiredNullable<String>,
    #[serde(default)]
    priority: RequiredNullable<String>,
    #[serde(default, rename = "action_url")]
    _action_url: RequiredNullable<String>,
}

#[derive(Debug, Deserialize)]
struct ServiceNotificationList {
    items: Vec<ServiceNotification>,
    #[serde(rename = "total")]
    _total: i64,
}

/// Presentation-only shape. Ownership and access decisions remain in the
/// notification service and gateway; this type only maps already-authorized
/// rows to escaped Dioxus text nodes.
#[derive(Clone, Debug, PartialEq)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub body: String,
    pub kind: Option<String>,
    pub priority: Option<String>,
    pub read: bool,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<ServiceNotification> for Notification {
    type Error = ();

    fn try_from(value: ServiceNotification) -> Result<Self, Self::Error> {
        let subject = value.subject.require()?;
        let read_at = value.read_at.require()?;
        let title = value.title.require()?;
        let notification_type = value.notification_type.require()?;
        let priority = value.priority.require()?;
        let _action_url = value._action_url.require()?;
        let title = non_blank(title)
            .or_else(|| non_blank(subject))
            .unwrap_or_else(|| "Notification".to_string());
        Ok(Self {
            id: value.id,
            title,
            body: value.body,
            kind: non_blank(notification_type),
            priority: non_blank(priority),
            read: read_at.is_some(),
            created_at: value.created_at,
        })
    }
}

fn non_blank(value: Option<String>) -> Option<String> {
    value.filter(|candidate| !candidate.trim().is_empty())
}

/// Format an authoritative service timestamp for display without changing its
/// ordering or lifecycle meaning. `now` is injected so every row in one render
/// can share the same instant and the presentation boundaries stay testable.
fn notification_timestamp_label(created_at: &DateTime<Utc>, now: &DateTime<Utc>) -> String {
    let elapsed_seconds = now.signed_duration_since(*created_at).num_seconds();
    if elapsed_seconds < 60 {
        "Just now".to_string()
    } else if elapsed_seconds < 60 * 60 {
        format!("{}m ago", elapsed_seconds / 60)
    } else if elapsed_seconds < 24 * 60 * 60 {
        format!("{}h ago", elapsed_seconds / (60 * 60))
    } else if elapsed_seconds < 7 * 24 * 60 * 60 {
        format!("{}d ago", elapsed_seconds / (24 * 60 * 60))
    } else {
        created_at.format("%b %-d, %Y").to_string()
    }
}

fn notification_timestamp_datetime(created_at: &DateTime<Utc>) -> String {
    created_at.to_rfc3339_opts(SecondsFormat::AutoSi, true)
}

fn notification_timestamp_title(created_at: &DateTime<Utc>) -> String {
    created_at.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

#[derive(Clone, Debug, PartialEq)]
enum NotificationLoad {
    Loaded(Vec<Notification>),
    UpstreamError,
    Malformed,
}

fn notification_load(ctx: &PageContext) -> NotificationLoad {
    match ctx
        .params
        .get(NOTIFICATIONS_STATE_PARAM)
        .map(String::as_str)
    {
        Some("error") | None => NotificationLoad::UpstreamError,
        Some("ok") => {
            let Some(raw) = ctx.params.get(NOTIFICATIONS_DATA_PARAM) else {
                return NotificationLoad::Malformed;
            };
            match serde_json::from_str::<ServiceNotificationList>(raw) {
                Ok(payload) => match payload
                    .items
                    .into_iter()
                    .map(Notification::try_from)
                    .collect::<Result<Vec<_>, _>>()
                {
                    Ok(items) => NotificationLoad::Loaded(items),
                    Err(()) => NotificationLoad::Malformed,
                },
                Err(_) => NotificationLoad::Malformed,
            }
        }
        Some(_) => NotificationLoad::Malformed,
    }
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Notifications");
    (meta, rsx! { RenderNotifications { ctx: ctx.clone() } })
}

#[component]
fn RenderNotifications(ctx: PageContext) -> Element {
    let load = notification_load(&ctx);
    let description = match &load {
        NotificationLoad::Loaded(items) => format!("{} loaded", items.len()),
        NotificationLoad::UpstreamError | NotificationLoad::Malformed => {
            "Temporarily unavailable".to_string()
        }
    };

    rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("your notifications".to_string()),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content notifications-page",
                    PageHeader {
                        title: "Notifications".to_string(),
                        description: Some(description),
                        icon: Some("bell".to_string()),
                    }
                    match load {
                        NotificationLoad::Loaded(items) => rsx! {
                            NotificationListSection { items }
                        },
                        NotificationLoad::UpstreamError => rsx! {
                            NotificationUnavailable { malformed: false }
                        },
                        NotificationLoad::Malformed => rsx! {
                            NotificationUnavailable { malformed: true }
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn NotificationUnavailable(malformed: bool) -> Element {
    let (title, detail) = if malformed {
        (
            "Notifications could not be displayed safely",
            "The notification service returned an unexpected response. No notification data was shown.",
        )
    } else {
        (
            "Notifications are temporarily unavailable",
            "The notification service could not be reached. Your notification history was not replaced with sample data.",
        )
    };
    rsx! {
        section {
            class: "card card-glass notifications-unavailable",
            role: "alert",
            aria_labelledby: "notifications-unavailable-title",
            aria_describedby: "notifications-unavailable-detail",
            div { class: "card-body notifications-empty",
                Icon { name: "alert-circle".to_string(), size: Some(32) }
                h2 {
                    id: "notifications-unavailable-title",
                    class: "notifications-empty-title",
                    "{title}"
                }
                p {
                    id: "notifications-unavailable-detail",
                    class: "notifications-empty-hint",
                    "{detail}"
                }
                a { class: "btn btn-sm btn-outline", href: "/notifications", "Try again" }
            }
        }
    }
}

/// Render only the rows loaded into this page. Counts never claim to describe
/// rows beyond the current service response.
#[component]
fn NotificationListSection(items: Vec<Notification>) -> Element {
    if items.is_empty() {
        return rsx! {
            section {
                class: "notifications-list",
                aria_labelledby: "notifications-empty-title",
                div { class: "card card-glass notifications-list-card",
                    div { class: "card-body notifications-empty",
                        Icon { name: "bell-off".to_string(), size: Some(32) }
                        h2 {
                            id: "notifications-empty-title",
                            class: "notifications-empty-title",
                            "No notifications yet"
                        }
                        p { class: "notifications-empty-hint", "New notifications will appear here." }
                    }
                }
            }
        };
    }

    let unread_count = items
        .iter()
        .filter(|notification| !notification.read)
        .count();
    let unread_label = format!("{unread_count} unread in loaded list");
    let rendered_at = Utc::now();

    rsx! {
        section {
            class: "notifications-list",
            aria_labelledby: "notifications-list-title",
            aria_describedby: "notifications-list-summary",
            h2 {
                id: "notifications-list-title",
                class: "sr-only",
                "Loaded notifications"
            }
            div { class: "notifications-summary",
                p {
                    id: "notifications-list-summary",
                    class: "notifications-unread-count",
                    style: "margin: 0;",
                    "{unread_label}"
                }
            }

            div { class: "card card-glass notifications-list-card",
                ul {
                    class: "card-body p-0",
                    role: "list",
                    style: "list-style: none; margin: 0; padding: 0;",
                    for notification in items {
                        NotificationRow { notification, rendered_at }
                    }
                }
            }
        }
    }
}

#[component]
fn NotificationRow(notification: Notification, rendered_at: DateTime<Utc>) -> Element {
    let (icon_name, icon_class) = match notification.kind.as_deref() {
        Some("payment") => ("credit-card", "notification-icon-payment"),
        Some("subscription") => ("zap", "notification-icon-subscription"),
        Some("wallet") => ("wallet", "notification-icon-wallet"),
        Some("news") => ("newspaper", "notification-icon-news"),
        Some("chat") => ("message-circle", "notification-icon-chat"),
        Some("alert") => ("alert-triangle", "notification-icon-alert"),
        _ => ("info", "notification-icon-system"),
    };
    let row_class = if notification.read {
        "notification-row notification-row-read"
    } else {
        "notification-row notification-row-unread"
    };
    let unread_dot_class = if notification.read {
        "notification-unread-dot notification-unread-dot-empty"
    } else {
        "notification-unread-dot"
    };
    let timestamp_label = notification_timestamp_label(&notification.created_at, &rendered_at);
    let timestamp_datetime = notification_timestamp_datetime(&notification.created_at);
    let timestamp_title = notification_timestamp_title(&notification.created_at);
    let read_state_label = if notification.read {
        "Read: "
    } else {
        "Unread: "
    };

    rsx! {
        li {
            class: "{row_class}",
            "data-notification-id": "{notification.id}",
            div { class: "notification-icon {icon_class}",
                Icon { name: icon_name.to_string(), size: Some(16) }
            }
            div { class: "notification-body",
                div { class: "notification-headline",
                    h3 { class: "notification-title",
                        span { class: "sr-only", "{read_state_label}" }
                        "{notification.title}"
                    }
                    span { class: "{unread_dot_class}", aria_hidden: "true" }
                }
                p { class: "notification-text", "{notification.body}" }
                div { class: "notification-meta",
                    if let Some(kind) = &notification.kind {
                        span { class: "notification-kind",
                            span { class: "sr-only", "Type: " }
                            "{kind}"
                        }
                        span { class: "notification-meta-sep", aria_hidden: "true", "·" }
                    }
                    if let Some(priority) = &notification.priority {
                        span { class: "notification-priority",
                            span { class: "sr-only", "Priority: " }
                            "{priority}"
                        }
                        span { class: "notification-meta-sep", aria_hidden: "true", "·" }
                    }
                    time {
                        class: "notification-time",
                        datetime: "{timestamp_datetime}",
                        title: "{timestamp_title}",
                        span { class: "sr-only", "Received: " }
                        "{timestamp_label}"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::{AuthMethod, User};

    fn user_with(perms: &[&str]) -> User {
        User {
            id: "u1".to_string(),
            address: "0x1234abcd".to_string(),
            chain_id: "56".to_string(),
            roles: vec!["user".to_string()],
            email: None,
            tier: Some("Pro".to_string()),
            permissions: perms
                .iter()
                .map(|permission| permission.to_string())
                .collect(),
            last_login_at: None,
            auth_method: AuthMethod::Wallet,
            display_name: None,
        }
    }

    fn context(
        user: Option<User>,
        state: Option<&str>,
        payload: Option<serde_json::Value>,
    ) -> PageContext {
        let mut params = std::collections::HashMap::new();
        if let Some(state) = state {
            params.insert(NOTIFICATIONS_STATE_PARAM.to_string(), state.to_string());
        }
        if let Some(payload) = payload {
            params.insert(NOTIFICATIONS_DATA_PARAM.to_string(), payload.to_string());
        }
        PageContext {
            user,
            path: "/notifications".to_string(),
            params,
            ..Default::default()
        }
    }

    fn exact_target_payload() -> serde_json::Value {
        serde_json::json!({
            "items": [
                {
                    "id": "0x1",
                    "user_id": "0x1234abcd",
                    "channel": "in_app",
                    "recipient": "0x1234abcd",
                    "template_id": null,
                    "subject": "Subject fallback",
                    "body": "Unread body",
                    "data": null,
                    "status": "read",
                    "error": null,
                    "sent_at": null,
                    "created_at": "2026-07-22T01:00:00Z",
                    "read_at": null,
                    "title": null,
                    "notification_type": null,
                    "priority": null,
                    "action_url": "javascript:alert(1)"
                },
                {
                    "id": "0x2",
                    "user_id": "0x1234abcd",
                    "channel": "in_app",
                    "recipient": "0x1234abcd",
                    "template_id": null,
                    "subject": null,
                    "body": "Neutral title body",
                    "data": null,
                    "status": "pending",
                    "error": null,
                    "sent_at": null,
                    "created_at": "2026-07-22T02:00:00Z",
                    "read_at": "2026-07-22T03:00:00Z",
                    "title": null,
                    "notification_type": "system",
                    "priority": "high",
                    "action_url": "https://unapproved.example/"
                },
                {
                    "id": "0x3",
                    "user_id": "0x1234abcd",
                    "channel": "in_app",
                    "recipient": "0x1234abcd",
                    "template_id": null,
                    "subject": "Ignored subject",
                    "body": "<img src=x onerror=alert(1)>",
                    "data": null,
                    "status": "sent",
                    "error": null,
                    "sent_at": null,
                    "created_at": "2026-07-22T04:00:00Z",
                    "read_at": null,
                    "title": "<script>alert(\"x\")</script>",
                    "notification_type": "security",
                    "priority": "critical",
                    "action_url": null
                }
            ],
            "total": 999
        })
    }

    fn render_html(ctx: &PageContext) -> String {
        let (_meta, element) = render(ctx);
        dioxus_ssr::render_element(element)
    }

    fn exact_notifications() -> Vec<Notification> {
        match notification_load(&context(None, Some("ok"), Some(exact_target_payload()))) {
            NotificationLoad::Loaded(items) => items,
            state => panic!("exact notification fixture did not load: {state:?}"),
        }
    }

    fn timestamp(value: &str) -> DateTime<Utc> {
        value.parse().expect("test timestamp must be RFC3339")
    }

    #[test]
    fn notification_timestamp_labels_have_exact_human_boundaries() {
        let now = timestamp("2026-07-22T12:00:00Z");
        for (offset_seconds, expected) in [
            (-1, "Just now"),
            (0, "Just now"),
            (59, "Just now"),
            (60, "1m ago"),
            (3_599, "59m ago"),
            (3_600, "1h ago"),
            (86_399, "23h ago"),
            (86_400, "1d ago"),
            (604_799, "6d ago"),
            (604_800, "Jul 15, 2026"),
        ] {
            let created_at = now - chrono::Duration::seconds(offset_seconds);
            assert_eq!(
                notification_timestamp_label(&created_at, &now),
                expected,
                "unexpected label at {offset_seconds} elapsed seconds"
            );
        }
    }

    #[test]
    fn notification_row_renders_one_semantic_canonical_escaped_time() {
        let notification = Notification {
            id: "notification-1".to_string(),
            title: "<script>alert('title')</script>".to_string(),
            body: "<img src=x onerror=alert(1)>".to_string(),
            kind: Some("system".to_string()),
            priority: Some("normal".to_string()),
            read: false,
            created_at: timestamp("2026-07-22T10:00:00Z"),
        };
        let rendered_at = timestamp("2026-07-22T12:00:00Z");
        let html =
            dioxus_ssr::render_element(rsx! { NotificationRow { notification, rendered_at } });

        assert_eq!(html.matches("<time").count(), 1);
        assert!(html.contains("class=\"notification-time\""));
        assert!(html.contains("datetime=\"2026-07-22T10:00:00Z\""));
        assert!(html.contains("title=\"2026-07-22 10:00:00 UTC\""));
        assert!(html.contains(">Received: </span>2h ago</time>"));
        assert!(!html.contains("<script>alert('title')</script>"));
        assert!(!html.contains("<img src=x onerror=alert(1)>"));
        assert!(html.contains("&#60;script&#62;"));
        assert!(html.contains("&#60;img src=x onerror=alert(1)&#62;"));
    }

    #[test]
    fn loaded_notifications_are_a_named_native_noninteractive_list() {
        let html = dioxus_ssr::render_element(
            rsx! { NotificationListSection { items: exact_notifications() } },
        );

        assert!(html.contains("<section class=\"notifications-list\""));
        assert!(html.contains("aria-labelledby=\"notifications-list-title\""));
        assert!(html.contains("aria-describedby=\"notifications-list-summary\""));
        assert!(html.contains(
            "<h2 id=\"notifications-list-title\" class=\"sr-only\">Loaded notifications</h2>"
        ));
        assert!(html.contains("<p id=\"notifications-list-summary\""));
        assert!(html.contains(">2 unread in loaded list</p>"));
        assert_eq!(html.matches("<ul").count(), 1);
        assert!(html.contains("<ul class=\"card-body p-0\" role=\"list\""));
        assert_eq!(html.matches("<li").count(), 3);
        assert_eq!(html.matches("<h3 class=\"notification-title\"").count(), 3);
        assert!(!html.contains("<article"));

        assert_eq!(html.matches(">Unread: </span>").count(), 2);
        assert_eq!(html.matches(">Read: </span>").count(), 1);
        assert!(html.contains(">Unread: </span>Subject fallback</h3>"));
        assert!(html.contains(">Read: </span>Notification</h3>"));

        assert!(html.contains("class=\"notification-unread-dot\" aria-hidden=\"true\""));
        assert!(html.contains(
            "class=\"notification-unread-dot notification-unread-dot-empty\" aria-hidden=\"true\""
        ));
        assert_eq!(
            html.matches("class=\"notification-meta-sep\" aria-hidden=\"true\"")
                .count(),
            4
        );
        assert_eq!(html.matches(">Type: </span>").count(), 2);
        assert_eq!(html.matches(">Priority: </span>").count(), 2);
        assert_eq!(html.matches(">Received: </span>").count(), 3);

        let first = html.find("data-notification-id=\"0x1\"").unwrap();
        let second = html.find("data-notification-id=\"0x2\"").unwrap();
        let third = html.find("data-notification-id=\"0x3\"").unwrap();
        assert!(first < second && second < third);

        for forbidden in [
            "<a",
            "<button",
            "tabindex=",
            "onclick=",
            "role=\"button\"",
            "role=\"link\"",
        ] {
            assert!(
                !html.contains(forbidden),
                "loaded rows unexpectedly became interactive: {forbidden}"
            );
        }
    }

    #[test]
    fn empty_notifications_are_a_named_section_without_live_status() {
        let html = dioxus_ssr::render_element(
            rsx! { NotificationListSection { items: Vec::<Notification>::new() } },
        );

        assert!(html.contains("<section class=\"notifications-list\""));
        assert!(html.contains("aria-labelledby=\"notifications-empty-title\""));
        assert!(html.contains(
            "<h2 id=\"notifications-empty-title\" class=\"notifications-empty-title\">No notifications yet</h2>"
        ));
        assert!(html.contains("New notifications will appear here."));
        assert!(!html.contains("aria-live="));
        assert!(!html.contains("role=\"alert\""));
        assert!(!html.contains("role=\"status\""));
    }

    #[test]
    fn unavailable_notifications_are_named_described_alerts_with_native_retry() {
        for (malformed, title, detail) in [
            (
                false,
                "Notifications are temporarily unavailable",
                "The notification service could not be reached. Your notification history was not replaced with sample data.",
            ),
            (
                true,
                "Notifications could not be displayed safely",
                "The notification service returned an unexpected response. No notification data was shown.",
            ),
        ] {
            let html =
                dioxus_ssr::render_element(rsx! { NotificationUnavailable { malformed } });

            assert!(html.contains(
                "<section class=\"card card-glass notifications-unavailable\" role=\"alert\""
            ));
            assert!(html.contains("aria-labelledby=\"notifications-unavailable-title\""));
            assert!(html.contains("aria-describedby=\"notifications-unavailable-detail\""));
            assert!(html.contains(&format!(
                "<h2 id=\"notifications-unavailable-title\" class=\"notifications-empty-title\">{title}</h2>"
            )));
            assert!(html.contains(&format!(
                "<p id=\"notifications-unavailable-detail\" class=\"notifications-empty-hint\">{detail}</p>"
            )));
            assert!(html.contains(
                "<a class=\"btn btn-sm btn-outline\" href=\"/notifications\">Try again</a>"
            ));
        }
    }

    #[test]
    fn exact_target_payload_maps_nullable_fields_without_samples_or_action_urls() {
        let ctx = context(
            Some(user_with(&[])),
            Some("ok"),
            Some(exact_target_payload()),
        );
        let html = render_html(&ctx);

        assert!(html.contains("3 loaded"));
        assert!(html.contains("Subject fallback"));
        assert!(html.contains("Neutral title body"));
        assert!(html.contains(">Notification<"));
        assert!(html.contains("system"));
        assert!(html.contains("high"));
        assert!(!html.contains("javascript:"));
        assert!(!html.contains("unapproved.example"));
        assert!(!html.contains("Payment received"));
        assert!(!html.contains("New comment on your plan"));
    }

    #[test]
    fn read_state_comes_only_from_read_at_and_counts_loaded_rows_only() {
        let ctx = context(
            Some(user_with(&[])),
            Some("ok"),
            Some(exact_target_payload()),
        );
        let html = render_html(&ctx);

        assert_eq!(html.matches("notification-row-unread").count(), 2);
        assert_eq!(html.matches("notification-row-read").count(), 1);
        assert!(html.contains("2 unread in loaded list"));
        assert!(!html.contains("999 loaded"));
    }

    #[test]
    fn empty_payload_is_distinct_from_dependency_failure() {
        let ctx = context(
            Some(user_with(&[])),
            Some("ok"),
            Some(serde_json::json!({"items": [], "total": 0})),
        );
        let html = render_html(&ctx);

        assert!(html.contains("0 loaded"));
        assert!(html.contains("No notifications yet"));
        assert!(!html.contains("temporarily unavailable"));
    }

    #[test]
    fn missing_required_nullable_fields_are_malformed_even_when_nullable() {
        for field in [
            "subject",
            "read_at",
            "title",
            "notification_type",
            "priority",
            "action_url",
        ] {
            let mut payload = exact_target_payload();
            payload["items"][0]
                .as_object_mut()
                .expect("fixture notification must be an object")
                .remove(field);
            let ctx = context(Some(user_with(&[])), Some("ok"), Some(payload));
            let html = render_html(&ctx);

            assert!(
                html.contains("could not be displayed safely"),
                "missing {field} must fail closed"
            );
            assert!(!html.contains("Unread body"));
        }
    }

    #[test]
    fn invalid_created_at_or_non_null_read_at_is_malformed() {
        let mut invalid_created_at = exact_target_payload();
        invalid_created_at["items"][0]["created_at"] = serde_json::json!("not-a-timestamp");
        let created_at_html = render_html(&context(
            Some(user_with(&[])),
            Some("ok"),
            Some(invalid_created_at),
        ));
        assert!(created_at_html.contains("could not be displayed safely"));
        assert!(!created_at_html.contains("Unread body"));

        let mut invalid_read_at = exact_target_payload();
        invalid_read_at["items"][0]["read_at"] = serde_json::json!("not-a-timestamp");
        let read_at_html = render_html(&context(
            Some(user_with(&[])),
            Some("ok"),
            Some(invalid_read_at),
        ));
        assert!(read_at_html.contains("could not be displayed safely"));
        assert!(!read_at_html.contains("Unread body"));
    }

    #[test]
    fn malformed_and_upstream_states_are_truthful_and_sample_free() {
        let malformed = context(
            Some(user_with(&[])),
            Some("ok"),
            Some(serde_json::json!({"items": "not-an-array", "total": 0})),
        );
        let malformed_html = render_html(&malformed);
        assert!(malformed_html.contains("could not be displayed safely"));

        let upstream = context(Some(user_with(&[])), Some("error"), None);
        let upstream_html = render_html(&upstream);
        assert!(upstream_html.contains("temporarily unavailable"));
        assert!(upstream_html.contains("not replaced with sample data"));

        for html in [&malformed_html, &upstream_html] {
            assert!(!html.contains("Payment received"));
            assert!(!html.contains("Scheduled maintenance"));
        }
    }

    #[test]
    fn authenticated_owner_needs_no_frontend_permission_grant() {
        let ctx = context(
            Some(user_with(&[])),
            Some("ok"),
            Some(exact_target_payload()),
        );
        let html = render_html(&ctx);

        assert!(html.contains("Subject fallback"));
        assert!(!html.contains("Permission required"));
        assert!(!html.contains("notifications:read"));
    }

    #[test]
    fn signed_out_user_sees_auth_gate_and_no_owner_rows() {
        let ctx = context(None, Some("ok"), Some(exact_target_payload()));
        let html = render_html(&ctx);

        assert!(html.contains("Sign in required"));
        assert!(!html.contains("Subject fallback"));
        assert!(!html.contains("Unread body"));
    }

    #[test]
    fn notification_content_is_escaped_as_text() {
        let ctx = context(
            Some(user_with(&[])),
            Some("ok"),
            Some(exact_target_payload()),
        );
        let html = render_html(&ctx);

        assert!(!html.contains("<script>alert(\"x\")</script>"));
        assert!(!html.contains("<img src=x onerror=alert(1)>"));
        assert!(html.contains("&#60;script&#62;"));
        assert!(html.contains("&#60;img src=x onerror=alert(1)&#62;"));
    }

    #[test]
    fn lifecycle_delivery_and_unapproved_navigation_controls_are_absent() {
        let ctx = context(
            Some(user_with(&[])),
            Some("ok"),
            Some(exact_target_payload()),
        );
        let html = render_html(&ctx);

        for forbidden in [
            "Mark all read",
            "Clear all",
            "Mark read",
            "Delete",
            "Enable Browser Notifications",
            "Test Notification",
            "Notification Settings",
            "notification-action",
            "role=\"switch\"",
        ] {
            assert!(
                !html.contains(forbidden),
                "unexpected active control: {forbidden}"
            );
        }
    }

    #[test]
    fn hydration_less_page_has_no_inert_filter_buttons() {
        let ctx = context(
            Some(user_with(&[])),
            Some("ok"),
            Some(exact_target_payload()),
        );
        let html = render_html(&ctx);

        assert!(html.contains("2 unread in loaded list"));
        assert!(!html.contains("notifications-filterbar"));
        assert!(!html.contains("notifications-filters"));
        assert!(!html.contains("Filter loaded notifications"));
        assert!(!html.contains(">All<"));
        assert!(!html.contains(">Unread<"));
        assert!(!html.contains(">Read<"));
    }
}

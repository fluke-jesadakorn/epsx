//! Truthful owner notification center.
//!
//! The frontend BFF hydrates this page from the extracted notification
//! service's owner-scoped `GET /api/v1/notification/list` route. Mutation
//! controls are intentionally narrow: they call only the Rust BFF's
//! owner-scoped lifecycle endpoints and never infer delivery or provider
//! state from a successful request.

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
const NOTIFICATIONS_PAGE_PARAM: &str = "data_notifications_page";
const NOTIFICATIONS_STATUS_PARAM: &str = "data_notifications_status";
const NOTIFICATIONS_TYPE_PARAM: &str = "data_notifications_type";
const NOTIFICATIONS_PRIORITY_PARAM: &str = "data_notifications_priority";
const NOTIFICATIONS_START_DATE_PARAM: &str = "data_notifications_start_date";
const NOTIFICATIONS_END_DATE_PARAM: &str = "data_notifications_end_date";
const NOTIFICATIONS_INVALID_QUERY: &str = "invalid_query";
const NOTIFICATIONS_PAGE_SIZE: u64 = 20;
const NOTIFICATIONS_MAX_PAGE: u32 = 50_001;
const NOTIFICATIONS_WINDOW_ROWS: u64 =
    (NOTIFICATIONS_MAX_PAGE as u64 - 1) * NOTIFICATIONS_PAGE_SIZE + NOTIFICATIONS_PAGE_SIZE;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum NotificationStatusFilter {
    #[default]
    All,
    Read,
    Unread,
}

impl NotificationStatusFilter {
    fn from_param(value: Option<&str>) -> Option<Self> {
        match value {
            None | Some("all") => Some(Self::All),
            Some("read") => Some(Self::Read),
            Some("unread") => Some(Self::Unread),
            _ => None,
        }
    }

    fn query_value(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Read => "read",
            Self::Unread => "unread",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Read => "Read",
            Self::Unread => "Unread",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum NotificationTypeFilter {
    #[default]
    All,
    System,
    Security,
    Permission,
    WalletManagement,
    Wallet,
    Payment,
    General,
    Announcement,
    Advertisement,
    Chat,
}

impl NotificationTypeFilter {
    fn from_param(value: Option<&str>) -> Option<Self> {
        match value {
            None | Some("all") => Some(Self::All),
            Some("system") => Some(Self::System),
            Some("security") => Some(Self::Security),
            Some("permission") => Some(Self::Permission),
            Some("wallet_management") => Some(Self::WalletManagement),
            Some("wallet") => Some(Self::Wallet),
            Some("payment") => Some(Self::Payment),
            Some("general") => Some(Self::General),
            Some("announcement") => Some(Self::Announcement),
            Some("advertisement") => Some(Self::Advertisement),
            Some("chat") => Some(Self::Chat),
            _ => None,
        }
    }

    fn query_value(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::System => "system",
            Self::Security => "security",
            Self::Permission => "permission",
            Self::WalletManagement => "wallet_management",
            Self::Wallet => "wallet",
            Self::Payment => "payment",
            Self::General => "general",
            Self::Announcement => "announcement",
            Self::Advertisement => "advertisement",
            Self::Chat => "chat",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::All => "All types",
            Self::System => "System",
            Self::Security => "Security",
            Self::Permission => "Permission",
            Self::WalletManagement => "Wallet management",
            Self::Wallet => "Wallet",
            Self::Payment => "Payment",
            Self::General => "General",
            Self::Announcement => "Announcement",
            Self::Advertisement => "Advertisement",
            Self::Chat => "Chat",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum NotificationPriorityFilter {
    #[default]
    All,
    Low,
    Normal,
    High,
    Critical,
    Urgent,
}

impl NotificationPriorityFilter {
    fn from_param(value: Option<&str>) -> Option<Self> {
        match value {
            None | Some("all") => Some(Self::All),
            Some("low") => Some(Self::Low),
            Some("normal") => Some(Self::Normal),
            Some("high") => Some(Self::High),
            Some("critical") => Some(Self::Critical),
            Some("urgent") => Some(Self::Urgent),
            _ => None,
        }
    }

    fn query_value(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Low => "low",
            Self::Normal => "normal",
            Self::High => "high",
            Self::Critical => "critical",
            Self::Urgent => "urgent",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::All => "All priorities",
            Self::Low => "Low",
            Self::Normal => "Normal",
            Self::High => "High",
            Self::Critical => "Critical",
            Self::Urgent => "Urgent",
        }
    }
}

/// A wire field that must be present but may explicitly contain JSON `null`.
/// Serde otherwise gives missing fields and explicit `null` the same `None`
/// representation. The sentinel keeps those states distinct until the whole
/// service row is validated.
#[derive(Debug, Default)]
enum RequiredNullable<T> {
    #[default]
    Missing,
    Present(Option<T>),
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
    total: i64,
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
    if created_at > now {
        return created_at.format("%b %-d, %Y, %H:%M:%S UTC").to_string();
    }
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
struct NotificationPage {
    items: Vec<Notification>,
    total: u64,
    page: u32,
    total_pages: u64,
    status: NotificationStatusFilter,
    notification_type: NotificationTypeFilter,
    priority: NotificationPriorityFilter,
    start_date: Option<String>,
    end_date: Option<String>,
}

impl NotificationPage {
    fn loaded_summary(&self) -> String {
        let summary = format!(
            "Page {} of {} · {} loaded",
            self.page,
            self.total_pages.max(1),
            self.items.len()
        );
        if self.items.is_empty() {
            format!("{summary}. Showing 0 of {} notifications.", self.total)
        } else {
            let start = (u64::from(self.page) - 1) * NOTIFICATIONS_PAGE_SIZE + 1;
            let end = start + self.items.len() as u64 - 1;
            format!(
                "{summary}. Showing notifications {start}–{end} of {}.",
                self.total
            )
        }
    }

    fn accessible_pages(&self) -> u64 {
        self.total_pages.min(u64::from(NOTIFICATIONS_MAX_PAGE))
    }

    fn has_bounded_window(&self) -> bool {
        self.total_pages > u64::from(NOTIFICATIONS_MAX_PAGE)
    }

    fn is_authoritative_first_page_empty(&self) -> bool {
        self.page == 1 && self.total == 0 && self.items.is_empty()
    }

    fn is_out_of_range(&self) -> bool {
        self.items.is_empty() && !self.is_authoritative_first_page_empty()
    }
}

#[derive(Clone, Debug, PartialEq)]
enum NotificationLoad {
    Loaded(NotificationPage),
    UpstreamError(Option<u32>),
    Malformed(Option<u32>),
    InvalidQuery,
}

fn canonical_page_param(ctx: &PageContext) -> Option<u32> {
    let value = ctx.params.get(NOTIFICATIONS_PAGE_PARAM)?;
    if value.is_empty()
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    let page = value.parse::<u32>().ok()?;
    (1..=NOTIFICATIONS_MAX_PAGE).contains(&page).then_some(page)
}

fn total_pages(total: u64) -> u64 {
    if total == 0 {
        0
    } else {
        ((total - 1) / NOTIFICATIONS_PAGE_SIZE) + 1
    }
}

fn expected_page_items(total: u64, page: u32) -> Option<usize> {
    let offset = u64::from(page.checked_sub(1)?).checked_mul(NOTIFICATIONS_PAGE_SIZE)?;
    usize::try_from(total.saturating_sub(offset).min(NOTIFICATIONS_PAGE_SIZE)).ok()
}

fn notification_load(ctx: &PageContext) -> NotificationLoad {
    let page = canonical_page_param(ctx);
    let status = NotificationStatusFilter::from_param(
        ctx.params
            .get(NOTIFICATIONS_STATUS_PARAM)
            .map(String::as_str),
    );
    let notification_type = NotificationTypeFilter::from_param(
        ctx.params.get(NOTIFICATIONS_TYPE_PARAM).map(String::as_str),
    );
    let priority = NotificationPriorityFilter::from_param(
        ctx.params
            .get(NOTIFICATIONS_PRIORITY_PARAM)
            .map(String::as_str),
    );
    let start_date = match ctx.params.get(NOTIFICATIONS_START_DATE_PARAM) {
        None => Some(None),
        Some(value) if value == "all" => Some(None),
        Some(value) if value.len() <= 64 && DateTime::parse_from_rfc3339(value).is_ok() => {
            Some(Some(value.clone()))
        }
        _ => None,
    };
    let end_date = match ctx.params.get(NOTIFICATIONS_END_DATE_PARAM) {
        None => Some(None),
        Some(value) if value == "all" => Some(None),
        Some(value) if value.len() <= 64 && DateTime::parse_from_rfc3339(value).is_ok() => {
            Some(Some(value.clone()))
        }
        _ => None,
    };
    if status.is_none()
        || notification_type.is_none()
        || priority.is_none()
        || start_date.is_none()
        || end_date.is_none()
    {
        return NotificationLoad::Malformed(page);
    }
    let status = status.expect("status was checked above");
    let notification_type = notification_type.expect("type was checked above");
    let priority = priority.expect("priority was checked above");
    let start_date = start_date.expect("start date was checked above");
    let end_date = end_date.expect("end date was checked above");
    if start_date
        .as_deref()
        .zip(end_date.as_deref())
        .is_some_and(|(start, end)| {
            DateTime::parse_from_rfc3339(start).ok() > DateTime::parse_from_rfc3339(end).ok()
        })
    {
        return NotificationLoad::Malformed(page);
    }
    match ctx
        .params
        .get(NOTIFICATIONS_STATE_PARAM)
        .map(String::as_str)
    {
        Some(NOTIFICATIONS_INVALID_QUERY) => NotificationLoad::InvalidQuery,
        Some("error") | None => NotificationLoad::UpstreamError(page),
        Some("ok") => {
            let Some(page) = page else {
                return NotificationLoad::Malformed(None);
            };
            let Some(raw) = ctx.params.get(NOTIFICATIONS_DATA_PARAM) else {
                return NotificationLoad::Malformed(Some(page));
            };
            match serde_json::from_str::<ServiceNotificationList>(raw) {
                Ok(payload) => {
                    let Ok(total) = u64::try_from(payload.total) else {
                        return NotificationLoad::Malformed(Some(page));
                    };
                    let Ok(items) = payload
                        .items
                        .into_iter()
                        .map(Notification::try_from)
                        .collect::<Result<Vec<_>, _>>()
                    else {
                        return NotificationLoad::Malformed(Some(page));
                    };
                    if expected_page_items(total, page) != Some(items.len()) {
                        return NotificationLoad::Malformed(Some(page));
                    }
                    NotificationLoad::Loaded(NotificationPage {
                        items,
                        total,
                        page,
                        total_pages: total_pages(total),
                        status,
                        notification_type,
                        priority,
                        start_date,
                        end_date,
                    })
                }
                Err(_) => NotificationLoad::Malformed(Some(page)),
            }
        }
        Some(_) => NotificationLoad::Malformed(page),
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
        NotificationLoad::Loaded(page) => page.loaded_summary(),
        NotificationLoad::InvalidQuery => "Invalid notification page".to_string(),
        NotificationLoad::UpstreamError(_) | NotificationLoad::Malformed(_) => {
            "Temporarily unavailable".to_string()
        }
    };

    rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("your notifications".to_string()),
                return_url: Some(ctx.path.clone()),
                wallet_connected: ctx.wallet.address.is_some(),
                div { class: "container page-content notifications-page",
                    PageHeader {
                        title: "Notifications".to_string(),
                        description: Some(description),
                        // The source notification center uses a plain title
                        // row; the bell belongs to the navigation chrome,
                        // not a second icon above the content heading.
                        icon: None,
                    }
                    match load {
                        NotificationLoad::Loaded(page) => rsx! {
                            NotificationPageSection { page }
                        },
                        NotificationLoad::UpstreamError(page) => rsx! {
                            NotificationUnavailable { malformed: false, retry_page: page, start_date: None, end_date: None }
                        },
                        NotificationLoad::Malformed(page) => rsx! {
                            NotificationUnavailable { malformed: true, retry_page: page, start_date: None, end_date: None }
                        },
                        NotificationLoad::InvalidQuery => rsx! {
                            NotificationInvalidQuery {}
                        },
                    }
                }
            }
        }
    }
}

fn notification_page_href(
    page: u32,
    status: NotificationStatusFilter,
    notification_type: NotificationTypeFilter,
    priority: NotificationPriorityFilter,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> String {
    let mut params = Vec::new();
    if page > 1 {
        params.push(format!("page={page}"));
    }
    if status != NotificationStatusFilter::All {
        params.push(format!("status={}", status.query_value()));
    }
    if notification_type != NotificationTypeFilter::All {
        params.push(format!("type={}", notification_type.query_value()));
    }
    if priority != NotificationPriorityFilter::All {
        params.push(format!("priority={}", priority.query_value()));
    }
    if let Some(start_date) = start_date {
        params.push(format!("start_date={start_date}"));
    }
    if let Some(end_date) = end_date {
        params.push(format!("end_date={end_date}"));
    }
    if params.is_empty() {
        "/notifications".to_string()
    } else {
        format!("/notifications?{}", params.join("&"))
    }
}

#[component]
fn NotificationUnavailable(
    malformed: bool,
    retry_page: Option<u32>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Element {
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
    let retry_href = retry_page
        .map(|page| {
            notification_page_href(
                page,
                NotificationStatusFilter::All,
                NotificationTypeFilter::All,
                NotificationPriorityFilter::All,
                start_date.as_deref(),
                end_date.as_deref(),
            )
        })
        .unwrap_or_else(|| "/notifications".to_string());
    rsx! {
        Fragment {
            // Preserve the source notification-center geometry even when the
            // owner-scoped response is unavailable. These controls are
            // deliberately disabled: they communicate the available shape
            // without claiming that filtering or counts are live.
            section { class: "notifications-filter-preview card card-glass p-4 sm:p-5",
                div { class: "mb-3 flex items-center gap-2",
                    Icon { name: "filter".to_string(), size: Some(20), class_name: Some("text-orange-500".to_string()) }
                    h2 { class: "text-sm font-semibold text-slate-300", "Filters" }
                }
                div { class: "grid grid-cols-1 gap-4 md:grid-cols-3",
                    div {
                        label { class: "mb-2 block text-xs font-medium text-slate-400", "Status" }
                        div { class: "flex gap-2",
                            for (label, active) in [("All", true), ("Unread", false), ("Read", false)] {
                                button {
                                    r#type: "button",
                                    disabled: true,
                                    aria_disabled: "true",
                                    class: if active {
                                        "rounded-lg bg-orange-500/70 px-3 py-1.5 text-xs font-medium text-white/80"
                                    } else {
                                        "rounded-lg bg-slate-700/70 px-3 py-1.5 text-xs font-medium text-slate-300/70"
                                    },
                                    "{label}"
                                }
                            }
                        }
                    }
                    div {
                        label { class: "mb-2 block text-xs font-medium text-slate-400", "Type" }
                        div { class: "notifications-filter-option", aria_disabled: "true",
                            "All Types"
                            Icon { name: "chevron-down".to_string(), size: Some(16) }
                        }
                    }
                    div {
                        label { class: "mb-2 block text-xs font-medium text-slate-400", "Priority" }
                        div { class: "notifications-filter-option", aria_disabled: "true",
                            "All Priorities"
                            Icon { name: "chevron-down".to_string(), size: Some(16) }
                        }
                    }
                }
            }
            section {
                class: "card card-glass notifications-unavailable",
                role: "alert",
                aria_labelledby: "notifications-unavailable-title",
                aria_describedby: "notifications-unavailable-detail",
                div { class: "card-body notifications-empty min-h-[18rem] flex flex-col items-center justify-center",
                    Icon { name: "bell".to_string(), size: Some(56) }
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
                    div { class: "mt-4",
                        a { class: "btn btn-sm btn-outline", href: "{retry_href}", "Try again" }
                    }
                }
            }
        }
    }
}

#[component]
fn NotificationInvalidQuery() -> Element {
    rsx! {
        section {
            class: "card card-glass notifications-invalid-query",
            role: "alert",
            aria_labelledby: "notifications-invalid-query-title",
            aria_describedby: "notifications-invalid-query-detail",
            div { class: "card-body notifications-empty",
                Icon { name: "circle-alert".to_string(), size: Some(32) }
                h2 {
                    id: "notifications-invalid-query-title",
                    class: "notifications-empty-title",
                    "Notification page link is invalid"
                }
                p {
                    id: "notifications-invalid-query-detail",
                    class: "notifications-empty-hint",
                    "Use the notification page controls to open a bounded owner-history page."
                }
                a { class: "btn btn-sm btn-outline", href: "/notifications", "Open first page" }
            }
        }
    }
}

#[component]
fn NotificationPageSection(page: NotificationPage) -> Element {
    let summary = page.loaded_summary();
    let unread_count = page
        .items
        .iter()
        .filter(|notification| !notification.read)
        .count();
    let unread_label = format!("{unread_count} unread on this page");
    let rendered_at = Utc::now();
    let window_state = if page.has_bounded_window() {
        "bounded"
    } else {
        "complete"
    };
    let recovery_page = page
        .accessible_pages()
        .max(1)
        .min(u64::from(NOTIFICATIONS_MAX_PAGE)) as u32;
    let recovery_href = notification_page_href(
        recovery_page,
        page.status,
        page.notification_type,
        page.priority,
        page.start_date.as_deref(),
        page.end_date.as_deref(),
    );

    rsx! {
        section {
            class: "notifications-list",
            "data-notifications-window": window_state,
            aria_labelledby: if page.is_authoritative_first_page_empty() {
                "notifications-empty-title"
            } else if page.is_out_of_range() {
                "notifications-out-of-range-title"
            } else {
                "notifications-list-title"
            },
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
                    "aria-current": "page",
                    "{summary}"
                }
                p {
                    class: "notifications-live-status text-xs text-muted-foreground",
                    "data-notifications-live-status": "true",
                    role: "status",
                    aria_live: "polite",
                    "Live notification updates are connecting…"
                }
                if !page.items.is_empty() {
                    p { class: "notifications-unread-count", style: "margin: 0;", "{unread_label}" }
                }
                if page.has_bounded_window() {
                    p {
                        class: "notifications-window-note",
                        role: "note",
                        "The service reports {page.total} notifications across {page.total_pages} pages. Navigation is bounded to the first {NOTIFICATIONS_WINDOW_ROWS} records (page {NOTIFICATIONS_MAX_PAGE})."
                    }
                }
            }

            if !page.items.is_empty() {
                NotificationMutationToolbar { has_unread: unread_count > 0 }
            }
            NotificationStatusFilters {
                selected: page.status,
                notification_type: page.notification_type,
                priority: page.priority,
                start_date: page.start_date.clone(),
                end_date: page.end_date.clone(),
            }
            NotificationTypeFilters {
                selected: page.notification_type,
                status: page.status,
                priority: page.priority,
                start_date: page.start_date.clone(),
                end_date: page.end_date.clone(),
            }
            NotificationPriorityFilters {
                selected: page.priority,
                status: page.status,
                notification_type: page.notification_type,
                start_date: page.start_date.clone(),
                end_date: page.end_date.clone(),
            }

            if page.is_authoritative_first_page_empty() {
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
            } else if page.is_out_of_range() {
                div { class: "card card-glass notifications-list-card",
                    div { class: "card-body notifications-empty",
                        Icon { name: "list-restart".to_string(), size: Some(32) }
                        h2 {
                            id: "notifications-out-of-range-title",
                            class: "notifications-empty-title",
                            "This notification page is out of range"
                        }
                        p { class: "notifications-empty-hint",
                            if page.total == 0 {
                                "There are no notifications. Return to the first page."
                            } else {
                                "The requested page has no owner notifications. Return to the last available page."
                            }
                        }
                        a { class: "btn btn-sm btn-outline", href: "{recovery_href}",
                            if page.total == 0 { "Open first page" } else { "Open last available page" }
                        }
                    }
                }
            } else {
                div { class: "card card-glass notifications-list-card",
                ul {
                    class: "card-body p-0",
                    role: "list",
                    style: "list-style: none; margin: 0; padding: 0;",
                        for notification in page.items.clone() {
                        NotificationRow { notification, rendered_at }
                    }
                }
            }
                NotificationPagination {
                    page: page.page,
                    total_pages: page.total_pages,
                    loaded: page.items.len(),
                    total: page.total,
                    status: page.status,
                    notification_type: page.notification_type,
                    priority: page.priority,
                    start_date: page.start_date.clone(),
                    end_date: page.end_date.clone(),
                }
            }
        }
    }
}

#[component]
fn NotificationPagination(
    page: u32,
    total_pages: u64,
    loaded: usize,
    total: u64,
    status: NotificationStatusFilter,
    notification_type: NotificationTypeFilter,
    priority: NotificationPriorityFilter,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Element {
    if total_pages <= 1 || loaded == 0 {
        return rsx! {};
    }
    let display_pages = total_pages.max(1);
    let accessible_pages = total_pages.min(u64::from(NOTIFICATIONS_MAX_PAGE));
    let previous_href = page
        .checked_sub(1)
        .filter(|previous| *previous >= 1)
        .map(|previous| {
            notification_page_href(
                previous,
                status,
                notification_type,
                priority,
                start_date.as_deref(),
                end_date.as_deref(),
            )
        });
    let next_href = (u64::from(page) < accessible_pages).then(|| {
        notification_page_href(
            page + 1,
            status,
            notification_type,
            priority,
            start_date.as_deref(),
            end_date.as_deref(),
        )
    });
    let start = (u64::from(page) - 1) * NOTIFICATIONS_PAGE_SIZE + 1;
    let end = start + loaded as u64 - 1;
    let summary = format!(
        "Page {page} of {display_pages} · {loaded} loaded. Showing notifications {start}–{end} of {total}."
    );

    rsx! {
        nav {
            class: "notifications-pagination mt-6 flex items-center justify-center gap-3",
            aria_label: "Notification pages",
            if let Some(href) = previous_href {
                a { class: "btn btn-sm btn-outline", rel: "prev", href, "Previous" }
            } else {
                span { class: "btn btn-sm btn-outline", aria_disabled: "true", "Previous" }
            }
            span { class: "notifications-page-position text-sm text-muted-foreground", aria_current: "page",
                "{summary}"
            }
            if let Some(href) = next_href {
                a { class: "btn btn-sm btn-outline", rel: "next", href, "Next" }
            } else {
                span { class: "btn btn-sm btn-outline", aria_disabled: "true", "Next" }
            }
        }
    }
}

#[component]
fn NotificationStatusFilters(
    selected: NotificationStatusFilter,
    notification_type: NotificationTypeFilter,
    priority: NotificationPriorityFilter,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Element {
    rsx! {
        nav {
            class: "notifications-status-filters mt-4 flex flex-wrap items-center gap-2",
            aria_label: "Notification status filters",
            "data-notification-status-filters": "true",
            span { class: "text-xs text-muted-foreground", "Filter:" }
            for filter in [
                NotificationStatusFilter::All,
                NotificationStatusFilter::Unread,
                NotificationStatusFilter::Read,
            ] {
                a {
                    class: if filter == selected { "btn btn-sm btn-primary" } else { "btn btn-sm btn-outline" },
                    href: "{notification_page_href(1, filter, notification_type, priority, start_date.as_deref(), end_date.as_deref())}",
                    aria_current: if filter == selected { "page" } else { "false" },
                    "{filter.label()}"
                }
            }
        }
    }
}

#[component]
fn NotificationTypeFilters(
    selected: NotificationTypeFilter,
    status: NotificationStatusFilter,
    priority: NotificationPriorityFilter,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Element {
    rsx! {
        nav {
            class: "notifications-type-filters mt-2 flex flex-wrap items-center gap-2",
            aria_label: "Notification type filters",
            "data-notification-type-filters": "true",
            span { class: "text-xs text-muted-foreground", "Type:" }
            for filter in [
                NotificationTypeFilter::All,
                NotificationTypeFilter::System,
                NotificationTypeFilter::Security,
                NotificationTypeFilter::Permission,
                NotificationTypeFilter::WalletManagement,
                NotificationTypeFilter::Wallet,
                NotificationTypeFilter::Payment,
                NotificationTypeFilter::General,
                NotificationTypeFilter::Announcement,
                NotificationTypeFilter::Advertisement,
                NotificationTypeFilter::Chat,
            ] {
                a {
                    class: if filter == selected { "btn btn-sm btn-primary" } else { "btn btn-sm btn-outline" },
                    href: "{notification_page_href(1, status, filter, priority, start_date.as_deref(), end_date.as_deref())}",
                    aria_current: if filter == selected { "page" } else { "false" },
                    "{filter.label()}"
                }
            }
        }
    }
}

#[component]
fn NotificationPriorityFilters(
    selected: NotificationPriorityFilter,
    status: NotificationStatusFilter,
    notification_type: NotificationTypeFilter,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Element {
    rsx! {
        nav {
            class: "notifications-priority-filters mt-2 flex flex-wrap items-center gap-2",
            aria_label: "Notification priority filters",
            "data-notification-priority-filters": "true",
            span { class: "text-xs text-muted-foreground", "Priority:" }
            for filter in [
                NotificationPriorityFilter::All,
                NotificationPriorityFilter::Low,
                NotificationPriorityFilter::Normal,
                NotificationPriorityFilter::High,
                NotificationPriorityFilter::Critical,
                NotificationPriorityFilter::Urgent,
            ] {
                a {
                    class: if filter == selected { "btn btn-sm btn-primary" } else { "btn btn-sm btn-outline" },
                    href: "{notification_page_href(1, status, notification_type, filter, start_date.as_deref(), end_date.as_deref())}",
                    aria_current: if filter == selected { "page" } else { "false" },
                    "{filter.label()}"
                }
            }
        }
    }
}

#[component]
fn NotificationMutationToolbar(has_unread: bool) -> Element {
    rsx! {
        div {
            class: "notifications-mutation-toolbar mt-4 flex flex-wrap items-center gap-2",
            role: "group",
            aria_label: "Notification actions",
            "data-notification-mutation-toolbar": "true",
            if has_unread {
                button {
                    r#type: "button",
                    class: "btn btn-sm btn-outline",
                    "data-notification-mutation": "mark-all",
                    "Mark all as read"
                }
            }
            button {
                r#type: "button",
                class: "btn btn-sm btn-outline",
                "data-notification-mutation": "clear-all",
                "Remove all notifications"
            }
            span {
                class: "notifications-mutation-status text-xs text-muted-foreground",
                role: "status",
                aria_live: "polite",
                "data-notification-mutation-status": "true",
                "Changes are saved by the notification service."
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NotificationTypeDecoration {
    icon_name: &'static str,
    icon_class: &'static str,
}

/// Map service-owned type tags to presentation-only icon decoration.
///
/// The raw tag remains the visible metadata label. Unknown or absent tags
/// deliberately receive neutral decoration rather than being reclassified as
/// a known notification type.
fn notification_type_decoration(kind: Option<&str>) -> NotificationTypeDecoration {
    let (icon_name, icon_class) = match kind {
        Some("payment") => ("credit-card", "notification-icon-payment"),
        Some("wallet" | "walletmanagement" | "wallet_management" | "wallet-management") => {
            ("wallet", "notification-icon-wallet")
        }
        Some("chat") => ("message-circle", "notification-icon-chat"),
        Some("security") => ("shield", "notification-icon-system"),
        Some("permission") => ("key", "notification-icon-system"),
        Some("announcement" | "news") => ("newspaper", "notification-icon-news"),
        Some("portfolio-alert" | "alert") => ("alert-triangle", "notification-icon-alert"),
        Some("subscription") => ("zap", "notification-icon-subscription"),
        Some("system" | "general" | "advertisement") => ("info", "notification-icon-system"),
        Some(_) | None => ("info", "notification-icon-system"),
    };
    NotificationTypeDecoration {
        icon_name,
        icon_class,
    }
}

/// Map an exact service-owned priority value to a presentation-only chip.
///
/// The raw priority remains the visible label. This mapping is deliberately
/// case-sensitive and returns only static class names, so an arbitrary service
/// value can never become part of an HTML class or be relabelled as a known
/// priority.
fn notification_priority_class(priority: &str) -> &'static str {
    match priority {
        "critical" | "urgent" => "notification-priority-critical",
        "high" => "notification-priority-high",
        "normal" => "notification-priority-normal",
        "low" => "notification-priority-low",
        _ => "notification-priority-neutral",
    }
}

#[component]
fn NotificationRow(notification: Notification, rendered_at: DateTime<Utc>) -> Element {
    let decoration = notification_type_decoration(notification.kind.as_deref());
    let rendered_priority = notification
        .priority
        .as_deref()
        .filter(|priority| !priority.trim().is_empty());
    let priority_class = rendered_priority
        .map(notification_priority_class)
        .unwrap_or("notification-priority-neutral");
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
            div { class: "notification-icon {decoration.icon_class}",
                Icon { name: decoration.icon_name.to_string(), size: Some(16) }
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
                    if let Some(priority) = rendered_priority {
                        span {
                            class: "notification-priority {priority_class}",
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
                div {
                    class: "notification-actions mt-3 flex flex-wrap gap-2",
                    role: "group",
                    aria_label: "Notification actions",
                    if notification.read {
                        button {
                            r#type: "button",
                            class: "btn btn-sm btn-outline",
                            "data-notification-mutation": "unread",
                            "data-notification-id": "{notification.id}",
                            "Mark unread"
                        }
                    } else {
                        button {
                            r#type: "button",
                            class: "btn btn-sm btn-outline",
                            "data-notification-mutation": "read",
                            "data-notification-id": "{notification.id}",
                            "Mark read"
                        }
                    }
                    button {
                        r#type: "button",
                        class: "btn btn-sm btn-outline",
                        "data-notification-mutation": "acknowledge",
                        "data-notification-id": "{notification.id}",
                        "Acknowledge"
                    }
                    button {
                        r#type: "button",
                        class: "btn btn-sm btn-outline",
                        "data-notification-mutation": "dismiss",
                        "data-notification-id": "{notification.id}",
                        "Dismiss"
                    }
                    button {
                        r#type: "button",
                        class: "btn btn-sm btn-outline",
                        "data-notification-mutation": "delete",
                        "data-notification-id": "{notification.id}",
                        "Remove"
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
        params.insert(NOTIFICATIONS_PAGE_PARAM.to_string(), "1".to_string());
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
            "total": 3
        })
    }

    fn render_html(ctx: &PageContext) -> String {
        let (_meta, element) = render(ctx);
        dioxus_ssr::render_element(element)
    }

    fn exact_notifications() -> Vec<Notification> {
        match notification_load(&context(None, Some("ok"), Some(exact_target_payload()))) {
            NotificationLoad::Loaded(page) => page.items,
            state => panic!("exact notification fixture did not load: {state:?}"),
        }
    }

    fn exact_page() -> NotificationPage {
        match notification_load(&context(None, Some("ok"), Some(exact_target_payload()))) {
            NotificationLoad::Loaded(page) => page,
            state => panic!("exact notification fixture did not load: {state:?}"),
        }
    }

    fn timestamp(value: &str) -> DateTime<Utc> {
        value.parse().expect("test timestamp must be RFC3339")
    }

    fn notification_row_html(kind: Option<&str>, priority: Option<&str>) -> String {
        let notification = Notification {
            id: "notification-decoration-test".to_string(),
            title: "Decoration test".to_string(),
            body: "Body".to_string(),
            kind: kind.map(str::to_string),
            priority: priority.map(str::to_string),
            read: false,
            created_at: timestamp("2026-07-22T10:00:00Z"),
        };
        dioxus_ssr::render_element(rsx! {
            NotificationRow {
                notification,
                rendered_at: timestamp("2026-07-22T12:00:00Z"),
            }
        })
    }

    #[test]
    fn notification_type_decorations_cover_exact_tags_and_neutral_fallbacks() {
        for (kind, icon_name, icon_class) in [
            (Some("payment"), "credit-card", "notification-icon-payment"),
            (Some("wallet"), "wallet", "notification-icon-wallet"),
            (
                Some("walletmanagement"),
                "wallet",
                "notification-icon-wallet",
            ),
            (
                Some("wallet_management"),
                "wallet",
                "notification-icon-wallet",
            ),
            (
                Some("wallet-management"),
                "wallet",
                "notification-icon-wallet",
            ),
            (Some("chat"), "message-circle", "notification-icon-chat"),
            (Some("security"), "shield", "notification-icon-system"),
            (Some("permission"), "key", "notification-icon-system"),
            (Some("announcement"), "newspaper", "notification-icon-news"),
            (
                Some("portfolio-alert"),
                "alert-triangle",
                "notification-icon-alert",
            ),
            (
                Some("subscription"),
                "zap",
                "notification-icon-subscription",
            ),
            (Some("news"), "newspaper", "notification-icon-news"),
            (Some("alert"), "alert-triangle", "notification-icon-alert"),
            (Some("system"), "info", "notification-icon-system"),
            (Some("general"), "info", "notification-icon-system"),
            (Some("advertisement"), "info", "notification-icon-system"),
            (Some("unknown"), "info", "notification-icon-system"),
            (Some("Payment"), "info", "notification-icon-system"),
            (None, "info", "notification-icon-system"),
        ] {
            assert_eq!(
                notification_type_decoration(kind),
                NotificationTypeDecoration {
                    icon_name,
                    icon_class,
                },
                "unexpected decoration for {kind:?}",
            );
        }
    }

    #[test]
    fn notification_rows_render_known_icons_without_relabeling_raw_metadata() {
        for (kind, priority, icon_name, icon_class, priority_class) in [
            (
                "security",
                "critical",
                "shield",
                "notification-icon-system",
                "notification-priority-critical",
            ),
            (
                "wallet_management",
                "normal",
                "wallet",
                "notification-icon-wallet",
                "notification-priority-normal",
            ),
            (
                "portfolio-alert",
                "high",
                "alert-triangle",
                "notification-icon-alert",
                "notification-priority-high",
            ),
        ] {
            let html = notification_row_html(Some(kind), Some(priority));
            assert!(html.contains(&format!("class=\"notification-icon {icon_class}\"")));
            assert!(html.contains(&format!("class=\"lucide lucide-{icon_name} ")));
            assert!(html.contains(&format!(">Type: </span>{kind}</span>")));
            assert!(html.contains(&format!("class=\"notification-priority {priority_class}\"")));
            assert!(html.contains(&format!(">Priority: </span>{priority}</span>")));
        }
    }

    #[test]
    fn priority_presentation_classes_use_an_exact_static_raw_value_map() {
        for (priority, expected_class) in [
            ("critical", "notification-priority-critical"),
            ("urgent", "notification-priority-critical"),
            ("high", "notification-priority-high"),
            ("normal", "notification-priority-normal"),
            ("low", "notification-priority-low"),
            ("unknown", "notification-priority-neutral"),
            ("Critical", "notification-priority-neutral"),
            (" urgent", "notification-priority-neutral"),
            ("normal ", "notification-priority-neutral"),
        ] {
            assert_eq!(
                notification_priority_class(priority),
                expected_class,
                "unexpected presentation class for exact raw value {priority:?}",
            );
        }
    }

    #[test]
    fn unknown_type_and_priority_stay_escaped_visible_neutral_and_bounded() {
        let html = notification_row_html(
            Some("<script>unknown</script>"),
            Some("<img src=x onerror=alert(1)>"),
        );

        assert!(html.contains("class=\"notification-icon notification-icon-system\""));
        assert!(html.contains("class=\"lucide lucide-info "));
        assert!(html.contains("class=\"notification-priority notification-priority-neutral\""));
        assert!(html.contains(">Type: </span>&#60;script&#62;unknown&#60;/script&#62;</span>"));
        assert!(html.contains(">Priority: </span>&#60;img src=x onerror=alert(1)&#62;</span>"));
        assert!(!html.contains("<script>unknown</script>"));
        assert!(!html.contains("<img src=x onerror=alert(1)>"));
        assert!(!html.contains("notification-priority-&#60;"));
        for forbidden in ["<a", "tabindex=", "onclick=", "notification-action-url"] {
            assert!(
                !html.contains(forbidden),
                "decoration unexpectedly introduced interaction: {forbidden}"
            );
        }
    }

    #[test]
    fn null_and_blank_priorities_remain_absent_without_inventing_normal() {
        for priority in [None, Some(""), Some(" \t")] {
            let html = notification_row_html(Some("system"), priority);
            assert!(!html.contains("notification-priority"));
            assert!(!html.contains(">Priority: </span>"));
            assert!(!html.contains(">normal</span>"));
        }

        for priority in [
            serde_json::Value::Null,
            serde_json::json!(""),
            serde_json::json!(" \t"),
        ] {
            let mut payload = exact_target_payload();
            payload["items"][0]["priority"] = priority;
            let items = match notification_load(&context(None, Some("ok"), Some(payload))) {
                NotificationLoad::Loaded(page) => page.items,
                state => panic!("nullable priority fixture did not load: {state:?}"),
            };
            assert_eq!(items[0].priority, None);

            let html = dioxus_ssr::render_element(rsx! {
                NotificationRow {
                    notification: items[0].clone(),
                    rendered_at: timestamp("2026-07-22T12:00:00Z"),
                }
            });
            assert!(!html.contains("notification-priority"));
            assert!(!html.contains(">Priority: </span>"));
            assert!(!html.contains(">normal</span>"));
        }
    }

    #[test]
    fn every_priority_chip_renders_the_exact_static_class_and_stays_bounded() {
        for (priority, priority_class) in [
            ("critical", "notification-priority-critical"),
            ("urgent", "notification-priority-critical"),
            ("high", "notification-priority-high"),
            ("normal", "notification-priority-normal"),
            ("low", "notification-priority-low"),
            ("future-priority", "notification-priority-neutral"),
        ] {
            let html = notification_row_html(Some("system"), Some(priority));
            assert!(html.contains(&format!("class=\"notification-priority {priority_class}\"")));
            assert!(html.contains(&format!(">Priority: </span>{priority}</span>")));
            for forbidden in [
                "<a",
                "tabindex=",
                "onclick=",
                "role=\"link\"",
                "notification-action-url",
            ] {
                assert!(
                    !html.contains(forbidden),
                    "priority {priority:?} unexpectedly introduced interaction: {forbidden}"
                );
            }
        }
    }

    #[test]
    fn notification_timestamp_labels_have_exact_human_boundaries() {
        let now = timestamp("2026-07-22T12:00:00Z");
        for (created_at, expected) in [
            (
                now + chrono::Duration::nanoseconds(1),
                "Jul 22, 2026, 12:00:00 UTC",
            ),
            (
                now + chrono::Duration::seconds(1),
                "Jul 22, 2026, 12:00:01 UTC",
            ),
            (
                now + chrono::Duration::days(9),
                "Jul 31, 2026, 12:00:00 UTC",
            ),
        ] {
            assert_eq!(
                notification_timestamp_label(&created_at, &now),
                expected,
                "future timestamps must use a deterministic absolute UTC label"
            );
        }

        for (offset_seconds, expected) in [
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
    fn future_notification_row_uses_visible_absolute_utc_and_canonical_time_metadata() {
        let notification = Notification {
            id: "notification-future-time".to_string(),
            title: "Future timestamp".to_string(),
            body: "Timestamp truthfulness".to_string(),
            kind: None,
            priority: None,
            read: false,
            created_at: timestamp("2026-07-22T12:00:01Z"),
        };
        let html = dioxus_ssr::render_element(rsx! {
            NotificationRow {
                notification,
                rendered_at: timestamp("2026-07-22T12:00:00Z"),
            }
        });

        assert!(html.contains("datetime=\"2026-07-22T12:00:01Z\""));
        assert!(html.contains("title=\"2026-07-22 12:00:01 UTC\""));
        assert!(html.contains(">Received: </span>Jul 22, 2026, 12:00:01 UTC</time>"));
        assert!(!html.contains("Just now"));
    }

    #[test]
    fn hostile_unbroken_title_and_body_render_complete_in_their_semantic_elements() {
        let title_token = "T".repeat(1_024);
        let body_token = "B".repeat(2_048);
        let title = format!("title-head-<script>{title_token}</script>-title-tail");
        let body = format!("body-head-<img src=x onerror=alert(1)>{body_token}-body-tail");
        let notification = Notification {
            id: "notification-full-content-test".to_string(),
            title,
            body,
            kind: None,
            priority: None,
            read: false,
            created_at: timestamp("2026-07-22T10:00:00Z"),
        };
        let html = dioxus_ssr::render_element(rsx! {
            NotificationRow {
                notification,
                rendered_at: timestamp("2026-07-22T12:00:00Z"),
            }
        });
        let escaped_title =
            format!("title-head-&#60;script&#62;{title_token}&#60;/script&#62;-title-tail");
        let escaped_body =
            format!("body-head-&#60;img src=x onerror=alert(1)&#62;{body_token}-body-tail");

        let title_element = format!(
            "<h3 class=\"notification-title\"><span class=\"sr-only\">Unread: </span>{escaped_title}</h3>"
        );
        let body_element = format!("<p class=\"notification-text\">{escaped_body}</p>");
        let title_position = html.find(&title_element).expect("complete title element");
        let body_position = html.find(&body_element).expect("complete body element");
        let time_position = html.find("<time").expect("semantic received time");
        assert!(title_position < body_position && body_position < time_position);
        assert_eq!(html.matches("title-head-").count(), 1);
        assert_eq!(html.matches("-title-tail").count(), 1);
        assert_eq!(html.matches("body-head-").count(), 1);
        assert_eq!(html.matches("-body-tail").count(), 1);
        assert!(!html.contains("<script>"));
        assert!(!html.contains("<img"));
        for forbidden in [
            "<a",
            "<details",
            "<summary",
            "tabindex=",
            "onclick=",
            "role=\"button\"",
            "role=\"link\"",
            "notification-action-url",
        ] {
            assert!(
                !html.contains(forbidden),
                "full-content row unexpectedly became interactive: {forbidden}"
            );
        }
    }

    #[test]
    fn loaded_notifications_expose_bounded_native_owner_actions() {
        let html =
            dioxus_ssr::render_element(rsx! { NotificationPageSection { page: exact_page() } });

        assert!(html.contains("<section class=\"notifications-list\""));
        assert!(html.contains("data-notifications-window=\"complete\""));
        assert!(html.contains("aria-labelledby=\"notifications-list-title\""));
        assert!(html.contains("aria-describedby=\"notifications-list-summary\""));
        assert!(html.contains(
            "<h2 id=\"notifications-list-title\" class=\"sr-only\">Loaded notifications</h2>"
        ));
        assert!(html.contains("<p id=\"notifications-list-summary\""));
        assert!(html.contains(">Page 1 of 1 · 3 loaded. Showing notifications 1–3 of 3.</p>"));
        assert!(html.contains(">2 unread on this page</p>"));
        assert_eq!(html.matches("<ul").count(), 1);
        assert!(html.contains("<ul class=\"card-body p-0\" role=\"list\""));
        assert_eq!(html.matches("<li").count(), 3);
        assert_eq!(html.matches("<h3 class=\"notification-title\"").count(), 3);
        assert_eq!(html.matches("notification-row-unread").count(), 2);
        assert_eq!(html.matches("notification-row-read").count(), 1);
        assert!(!html.contains("<article"));
        assert!(html.contains("data-notification-mutation-toolbar=\"true\""));
        assert!(html.contains("data-notification-mutation=\"mark-all\""));
        assert!(html.contains("data-notification-mutation=\"clear-all\""));
        assert!(html.contains("data-notification-status-filters=\"true\""));
        assert!(html.contains("data-notification-type-filters=\"true\""));
        assert!(html.contains("data-notification-priority-filters=\"true\""));
        assert!(html.contains("href=\"/notifications?type=payment\""));
        assert!(html.contains("href=\"/notifications?type=wallet_management\""));
        assert!(html.contains("href=\"/notifications?type=announcement\""));
        assert!(html.contains("href=\"/notifications?type=advertisement\""));
        assert!(html.contains("href=\"/notifications?type=chat\""));
        assert!(html.contains("href=\"/notifications?priority=critical\""));
        assert_eq!(
            html.matches("data-notification-mutation=\"read\"").count(),
            2
        );
        assert_eq!(
            html.matches("data-notification-mutation=\"unread\"")
                .count(),
            1
        );
        assert_eq!(
            html.matches("data-notification-mutation=\"acknowledge\"")
                .count(),
            3
        );
        assert_eq!(
            html.matches("data-notification-mutation=\"dismiss\"")
                .count(),
            3
        );
        assert_eq!(
            html.matches("data-notification-mutation=\"delete\"")
                .count(),
            3
        );

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
        assert_eq!(html.matches("notification-priority-high").count(), 1);
        assert_eq!(html.matches("notification-priority-critical").count(), 1);
        assert_eq!(html.matches(">Received: </span>").count(), 3);

        assert!(html.contains("data-notification-status-filters=\"true\""));
        assert!(html.contains("href=\"/notifications\""));
        assert!(html.contains("href=\"/notifications?status=unread\""));
        assert!(html.contains("href=\"/notifications?status=read\""));
        assert!(html.contains(">All</a>"));
        assert!(html.contains(">Unread</a>"));
        assert!(html.contains(">Read</a>"));

        let first = html.find("data-notification-id=\"0x1\"").unwrap();
        let second = html.find("data-notification-id=\"0x2\"").unwrap();
        let third = html.find("data-notification-id=\"0x3\"").unwrap();
        assert!(first < second && second < third);
        assert!(!html.contains("aria-label=\"Notification pages\""));

        assert_eq!(html.matches("<a").count(), 20);
        assert!(!html.contains("href=\"javascript:"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("notification-action-url"));
    }

    #[test]
    fn empty_notifications_are_a_named_section_with_a_non_authoritative_live_status() {
        let html = dioxus_ssr::render_element(rsx! {
            NotificationPageSection {
                page: NotificationPage {
                    items: Vec::new(),
                    total: 0,
                    page: 1,
                    total_pages: 0,
                    status: NotificationStatusFilter::All,
                    notification_type: NotificationTypeFilter::All,
                    priority: NotificationPriorityFilter::All,
                    start_date: None,
                    end_date: None,
                }
            }
        });

        assert!(html.contains("<section class=\"notifications-list\""));
        assert!(html.contains("aria-labelledby=\"notifications-empty-title\""));
        assert!(html.contains("Page 1 of 1 · 0 loaded. Showing 0 of 0 notifications."));
        assert!(html.contains(
            "<h2 id=\"notifications-empty-title\" class=\"notifications-empty-title\">No notifications yet</h2>"
        ));
        assert!(html.contains("New notifications will appear here."));
        assert!(html.contains("data-notifications-live-status=\"true\""));
        assert!(html.contains("aria-live=\"polite\""));
        assert!(!html.contains("role=\"alert\""));
        assert!(html.contains("role=\"status\""));
        assert!(!html.contains("aria-label=\"Notification pages\""));
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
            let html = dioxus_ssr::render_element(rsx! {
                NotificationUnavailable { malformed, retry_page: Some(2) }
            });

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
                "<a class=\"btn btn-sm btn-outline\" href=\"/notifications?page=2\">Try again</a>"
            ));
        }
    }

    #[test]
    fn native_pagination_covers_first_middle_and_last_source_sized_pages() {
        let first = dioxus_ssr::render_element(rsx! {
            NotificationPagination {
                page: 1,
                total_pages: 3,
                loaded: 20,
                total: 53,
                status: NotificationStatusFilter::All,
                notification_type: NotificationTypeFilter::All,
                priority: NotificationPriorityFilter::All,
                start_date: None,
                end_date: None,
            }
        });
        assert!(first.contains("<nav class=\"notifications-pagination"));
        assert!(first.contains("aria-label=\"Notification pages\""));
        assert!(first.contains(
            "<span class=\"btn btn-sm btn-outline\" aria-disabled=\"true\">Previous</span>"
        ));
        assert!(first.contains(
            "<a class=\"btn btn-sm btn-outline\" rel=\"next\" href=\"/notifications?page=2\">Next</a>"
        ));
        assert!(first.contains("Page 1 of 3 · 20 loaded. Showing notifications 1–20 of 53."));
        assert!(!first.contains("tabindex"));

        let middle = dioxus_ssr::render_element(rsx! {
            NotificationPagination {
                page: 2,
                total_pages: 3,
                loaded: 20,
                total: 53,
                status: NotificationStatusFilter::All,
                notification_type: NotificationTypeFilter::All,
                priority: NotificationPriorityFilter::All,
                start_date: None,
                end_date: None,
            }
        });
        assert!(middle.contains(
            "<a class=\"btn btn-sm btn-outline\" rel=\"prev\" href=\"/notifications\">Previous</a>"
        ));
        assert!(middle.contains(
            "<a class=\"btn btn-sm btn-outline\" rel=\"next\" href=\"/notifications?page=3\">Next</a>"
        ));
        assert!(middle.contains("Page 2 of 3 · 20 loaded. Showing notifications 21–40 of 53."));

        let last = dioxus_ssr::render_element(rsx! {
            NotificationPagination {
                page: 3,
                total_pages: 3,
                loaded: 13,
                total: 53,
                status: NotificationStatusFilter::All,
                notification_type: NotificationTypeFilter::All,
                priority: NotificationPriorityFilter::All,
                start_date: None,
                end_date: None,
            }
        });
        assert!(last.contains(
            "<a class=\"btn btn-sm btn-outline\" rel=\"prev\" href=\"/notifications?page=2\">Previous</a>"
        ));
        assert!(last
            .contains("<span class=\"btn btn-sm btn-outline\" aria-disabled=\"true\">Next</span>"));
        assert!(last.contains("Page 3 of 3 · 13 loaded. Showing notifications 41–53 of 53."));
        assert!(!last.contains("href=\"/notifications?page=4\""));

        let unread = dioxus_ssr::render_element(rsx! {
            NotificationPagination {
                page: 2,
                total_pages: 3,
                loaded: 20,
                total: 53,
                status: NotificationStatusFilter::Unread,
                notification_type: NotificationTypeFilter::Payment,
                priority: NotificationPriorityFilter::Critical,
                start_date: Some("2026-01-01T00:00:00Z".to_string()),
                end_date: Some("2026-01-31T23:59:59Z".to_string()),
            }
        });
        assert!(unread.contains("href=\"/notifications?status=unread"));
        assert!(unread.contains("type=payment"));
        assert!(unread.contains("priority=critical"));
        assert!(unread.contains("start_date=2026-01-01T00:00:00Z"));
        assert!(unread.contains("end_date=2026-01-31T23:59:59Z"));
        assert!(unread.contains("href=\"/notifications?page=3"));
        assert!(unread.contains("status=unread"));
        assert!(unread.contains("type=payment"));
        assert!(unread.contains("priority=critical"));
    }

    #[test]
    fn out_of_range_and_invalid_pages_have_direct_canonical_recovery() {
        let mut out_of_range = context(
            Some(user_with(&[])),
            Some("ok"),
            Some(serde_json::json!({
                "items": [],
                "total": 25
            })),
        );
        out_of_range
            .params
            .insert(NOTIFICATIONS_PAGE_PARAM.to_string(), "3".to_string());
        let out_of_range_html = render_html(&out_of_range);
        assert!(out_of_range_html.contains("Page 3 of 2 · 0 loaded"));
        assert!(out_of_range_html.contains("This notification page is out of range"));
        assert!(out_of_range_html
            .contains("href=\"/notifications?page=2\">Open last available page</a>"));
        assert!(!out_of_range_html.contains("No notifications yet"));

        let mut empty_out_of_range = context(
            Some(user_with(&[])),
            Some("ok"),
            Some(serde_json::json!({
                "items": [],
                "total": 0
            })),
        );
        empty_out_of_range
            .params
            .insert(NOTIFICATIONS_PAGE_PARAM.to_string(), "2".to_string());
        let empty_out_of_range_html = render_html(&empty_out_of_range);
        assert!(empty_out_of_range_html.contains("href=\"/notifications\">Open first page</a>"));

        let mut invalid = context(
            Some(user_with(&[])),
            Some(NOTIFICATIONS_INVALID_QUERY),
            Some(exact_target_payload()),
        );
        invalid
            .params
            .insert(NOTIFICATIONS_PAGE_PARAM.to_string(), "50002".to_string());
        let invalid_html = render_html(&invalid);
        assert!(invalid_html.contains("Notification page link is invalid"));
        assert!(invalid_html.contains("href=\"/notifications\">Open first page</a>"));
        assert!(!invalid_html.contains("Subject fallback"));
        assert!(!invalid_html.contains("50002"));
    }

    #[test]
    fn pagination_rechecks_injected_page_total_cardinality_and_window_cap() {
        for injected_page in ["0", "02", "50002", "not-a-page"] {
            let mut ctx = context(
                Some(user_with(&[])),
                Some("ok"),
                Some(exact_target_payload()),
            );
            ctx.params.insert(
                NOTIFICATIONS_PAGE_PARAM.to_string(),
                injected_page.to_string(),
            );
            let html = render_html(&ctx);
            assert!(
                html.contains("could not be displayed safely"),
                "{injected_page}"
            );
            assert!(!html.contains("Subject fallback"), "{injected_page}");
        }

        let contradictory = context(
            Some(user_with(&[])),
            Some("ok"),
            Some(serde_json::json!({"items": [], "total": 1})),
        );
        assert!(render_html(&contradictory).contains("could not be displayed safely"));

        let notification = exact_notifications()
            .into_iter()
            .next()
            .expect("notification fixture");
        let bounded = dioxus_ssr::render_element(rsx! {
            NotificationPageSection {
                page: NotificationPage {
                    items: vec![notification; 20],
                    total: NOTIFICATIONS_WINDOW_ROWS + 1,
                    page: NOTIFICATIONS_MAX_PAGE,
                    total_pages: total_pages(NOTIFICATIONS_WINDOW_ROWS + 1),
                    status: NotificationStatusFilter::All,
                    notification_type: NotificationTypeFilter::All,
                    priority: NotificationPriorityFilter::All,
                    start_date: None,
                    end_date: None,
                }
            }
        });
        assert!(bounded.contains("data-notifications-window=\"bounded\""));
        assert!(bounded.contains("The service reports 1000021 notifications across 50002 pages."));
        assert!(
            bounded.contains("Navigation is bounded to the first 1000020 records (page 50001).")
        );
        assert!(!bounded.contains("href=\"/notifications?page=50002\""));
        assert!(bounded.contains("aria-disabled=\"true\">Next</span>"));
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
        assert!(html.contains("2 unread on this page"));
        assert!(html.contains("Showing notifications 1–3 of 3."));
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
    fn lifecycle_delivery_and_unapproved_navigation_controls_remain_absent() {
        let ctx = context(
            Some(user_with(&[])),
            Some("ok"),
            Some(exact_target_payload()),
        );
        let html = render_html(&ctx);

        for forbidden in [
            "Provider delivered",
            "Delivery confirmed",
            "Enable Browser Notifications",
            "Test Notification",
            "Notification Settings",
            "notification-action-url",
            "role=\"switch\"",
        ] {
            assert!(
                !html.contains(forbidden),
                "unexpected active control: {forbidden}"
            );
        }
    }

    #[test]
    fn hydration_less_page_exposes_native_status_filter_links() {
        let ctx = context(
            Some(user_with(&[])),
            Some("ok"),
            Some(exact_target_payload()),
        );
        let html = render_html(&ctx);

        assert!(html.contains("2 unread on this page"));
        assert!(html.contains("data-notification-status-filters=\"true\""));
        assert!(html.contains("aria-label=\"Notification status filters\""));
        assert!(html.contains("href=\"/notifications\""));
        assert!(html.contains("href=\"/notifications?status=unread\""));
        assert!(html.contains("href=\"/notifications?status=read\""));
        assert!(html.contains("href=\"/notifications?type=security\""));
        assert!(html.contains("href=\"/notifications?priority=high\""));
        assert!(html.contains(">All</a>"));
        assert!(html.contains(">Unread</a>"));
        assert!(html.contains(">Read</a>"));
        assert!(!html.contains("Filter loaded notifications"));
    }

    #[test]
    fn source_type_and_date_filters_are_bounded_and_preserved_through_pagination() {
        let mut ctx = context(
            Some(user_with(&[])),
            Some("ok"),
            Some(exact_target_payload()),
        );
        ctx.params
            .insert(NOTIFICATIONS_PAGE_PARAM.to_string(), "1".to_string());
        ctx.params.insert(
            NOTIFICATIONS_TYPE_PARAM.to_string(),
            "wallet_management".to_string(),
        );
        ctx.params.insert(
            NOTIFICATIONS_START_DATE_PARAM.to_string(),
            "2026-01-01T00:00:00Z".to_string(),
        );
        ctx.params.insert(
            NOTIFICATIONS_END_DATE_PARAM.to_string(),
            "2026-01-31T23:59:59Z".to_string(),
        );
        let html = render_html(&ctx);
        assert!(html.contains("type=wallet_management"));
        assert!(html.contains("start_date=2026-01-01T00"));
        assert!(html.contains("end_date=2026-01-31T23"));
    }
}

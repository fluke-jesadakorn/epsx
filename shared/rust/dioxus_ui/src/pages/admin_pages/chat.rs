//! Truthful authenticated admin chat projections for `/chat` and `/chat/{id}`.
//!
//! The route-specific admin BFF supplies strict, backend-owned list/detail
//! projections. This leaf renders only those authenticated reads. Conversation
//! mutations are bounded native forms; authorization, status
//! transitions, ownership, and persistence remain backend-owned.

use chrono::DateTime;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::auth::AuthGate;
use crate::primitives::Icon;

use super::super::{PageContext, PageMeta};

const CHAT_PATH: &str = "/chat";
const MAX_PAGE: u32 = 50_001;
const MAX_LIMIT: u32 = 50;
const MAX_TEXT_CHARS: usize = 16_384;

pub const ADMIN_CHAT_LIST_DATA_PARAM: &str = "data_admin_chat_list";
pub const ADMIN_CHAT_LIST_STATE_PARAM: &str = "data_admin_chat_list_state";
pub const ADMIN_CHAT_DETAIL_DATA_PARAM: &str = "data_admin_chat_detail";
pub const ADMIN_CHAT_DETAIL_STATE_PARAM: &str = "data_admin_chat_detail_state";

pub const ADMIN_CHAT_READY: &str = "ready";
pub const ADMIN_CHAT_EMPTY: &str = "empty";
pub const ADMIN_CHAT_FORBIDDEN: &str = "forbidden";
pub const ADMIN_CHAT_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_CHAT_MALFORMED: &str = "malformed";
pub const ADMIN_CHAT_MUTATION_PARAM: &str = "mutation";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminChatConversationSummary {
    pub id: String,
    pub topic_id: String,
    pub wallet_address: String,
    pub subject: String,
    pub status: String,
    pub assigned_agent: Option<String>,
    pub last_message_at: String,
    pub unread_user: i32,
    pub unread_agent: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminChatMessageSummary {
    pub id: String,
    pub conversation_id: String,
    pub sender_type: String,
    pub sender_address: Option<String>,
    pub content: String,
    pub is_read: bool,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminChatList {
    pub items: Vec<AdminChatConversationSummary>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
    pub has_next: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminChatStats {
    pub total_open: i64,
    pub total_in_progress: i64,
    pub total_resolved: i64,
    pub total_unassigned: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminChatTopicSummary {
    pub id: String,
    pub name: String,
    pub label: String,
    pub is_active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminChatInbox {
    pub conversations: AdminChatList,
    pub stats: AdminChatStats,
    pub topics: Vec<AdminChatTopicSummary>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminChatDetail {
    pub conversation: AdminChatConversationSummary,
    pub messages: Vec<AdminChatMessageSummary>,
}

pub fn decode_admin_chat_list(value: serde_json::Value) -> Option<AdminChatList> {
    let projection: AdminChatList = serde_json::from_value(value).ok()?;
    if projection.total < 0
        || !(1..=MAX_LIMIT).contains(&projection.limit)
        || !(1..=MAX_PAGE).contains(&projection.page)
        || projection.items.len() > projection.limit as usize
        || (projection.items.is_empty() && projection.has_next)
        || projection
            .items
            .iter()
            .any(|item| !valid_conversation(item))
    {
        return None;
    }
    let offset = i64::from(projection.page - 1).checked_mul(i64::from(projection.limit))?;
    if offset > 1_000_000
        || offset
            .checked_add(i64::try_from(projection.items.len()).ok()?)
            .is_none_or(|end| end > projection.total)
        || projection.has_next
            != offset
                .checked_add(i64::try_from(projection.items.len()).ok()?)
                .is_some_and(|end| end < projection.total)
    {
        return None;
    }
    Some(projection)
}

pub fn decode_admin_chat_inbox(value: serde_json::Value) -> Option<AdminChatInbox> {
    let projection: AdminChatInbox = serde_json::from_value(value).ok()?;
    decode_admin_chat_list(serde_json::to_value(&projection.conversations).ok()?)?;
    if projection.stats.total_open < 0
        || projection.stats.total_in_progress < 0
        || projection.stats.total_resolved < 0
        || projection.stats.total_unassigned < 0
        || projection.topics.len() > 200
        || projection.topics.iter().any(|topic| {
            !valid_uuid(&topic.id)
                || !bounded_text(&topic.name, 128)
                || !bounded_text(&topic.label, 128)
        })
    {
        return None;
    }
    let mut topic_ids = std::collections::HashSet::new();
    if projection
        .topics
        .iter()
        .any(|topic| !topic_ids.insert(topic.id.as_str()))
    {
        return None;
    }
    Some(projection)
}

pub fn decode_admin_chat_detail(value: serde_json::Value) -> Option<AdminChatDetail> {
    let projection: AdminChatDetail = serde_json::from_value(value).ok()?;
    if !valid_conversation(&projection.conversation)
        || projection.messages.len() > 500
        || projection
            .messages
            .iter()
            .any(|message| !valid_message(message))
        || projection
            .messages
            .iter()
            .any(|message| message.conversation_id != projection.conversation.id)
    {
        return None;
    }
    Some(projection)
}

fn valid_conversation(item: &AdminChatConversationSummary) -> bool {
    valid_uuid(&item.id)
        && valid_uuid(&item.topic_id)
        && bounded_text(&item.wallet_address, 128)
        && bounded_text(&item.subject, 255)
        && matches!(
            item.status.as_str(),
            "open" | "in_progress" | "resolved" | "closed"
        )
        && item.unread_user >= 0
        && item.unread_agent >= 0
        && item
            .assigned_agent
            .as_deref()
            .is_none_or(|agent| bounded_text(agent, 128))
        && valid_timestamp(&item.last_message_at)
        && valid_timestamp(&item.created_at)
        && valid_timestamp(&item.updated_at)
}

fn valid_message(item: &AdminChatMessageSummary) -> bool {
    valid_uuid(&item.id)
        && valid_uuid(&item.conversation_id)
        && bounded_text(&item.sender_type, 32)
        && item
            .sender_address
            .as_deref()
            .is_none_or(|sender| bounded_text(sender, 128))
        && bounded_text(&item.content, MAX_TEXT_CHARS)
        && valid_timestamp(&item.created_at)
}

fn valid_uuid(value: &str) -> bool {
    canonical_uuid(value).is_some()
}

fn bounded_text(value: &str, max_chars: usize) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= max_chars
        && !value.chars().any(char::is_control)
}

fn valid_timestamp(value: &str) -> bool {
    value.len() <= 64 && DateTime::parse_from_rfc3339(value).is_ok()
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ChatLoad {
    Ready(AdminChatInbox),
    Detail(AdminChatDetail),
    Empty(AdminChatInbox),
    Forbidden,
    Unavailable,
    Malformed,
}

fn list_load(ctx: &PageContext) -> ChatLoad {
    let state = ctx
        .params
        .get(ADMIN_CHAT_LIST_STATE_PARAM)
        .map(String::as_str);
    match state {
        Some(ADMIN_CHAT_READY) | Some(ADMIN_CHAT_EMPTY) => {
            let Some(raw) = ctx.params.get(ADMIN_CHAT_LIST_DATA_PARAM) else {
                return ChatLoad::Malformed;
            };
            let Some(inbox) = serde_json::from_str(raw)
                .ok()
                .and_then(decode_admin_chat_inbox)
            else {
                return ChatLoad::Malformed;
            };
            let list = &inbox.conversations;
            if matches!(state, Some(ADMIN_CHAT_EMPTY))
                && (list.total != 0 || !list.items.is_empty())
            {
                return ChatLoad::Malformed;
            }
            if matches!(state, Some(ADMIN_CHAT_READY)) && list.items.is_empty() {
                return ChatLoad::Malformed;
            }
            if list.items.is_empty() {
                ChatLoad::Empty(inbox)
            } else {
                ChatLoad::Ready(inbox)
            }
        }
        Some(ADMIN_CHAT_FORBIDDEN) => ChatLoad::Forbidden,
        Some(ADMIN_CHAT_MALFORMED) => ChatLoad::Malformed,
        Some(ADMIN_CHAT_UNAVAILABLE) | None => ChatLoad::Unavailable,
        Some(_) => ChatLoad::Malformed,
    }
}

fn detail_load(ctx: &PageContext) -> ChatLoad {
    match ctx
        .params
        .get(ADMIN_CHAT_DETAIL_STATE_PARAM)
        .map(String::as_str)
    {
        Some(ADMIN_CHAT_READY) => ctx
            .params
            .get(ADMIN_CHAT_DETAIL_DATA_PARAM)
            .and_then(|raw| serde_json::from_str(raw).ok())
            .and_then(decode_admin_chat_detail)
            .map(ChatLoad::Detail)
            .unwrap_or(ChatLoad::Malformed),
        Some(ADMIN_CHAT_FORBIDDEN) => ChatLoad::Forbidden,
        Some(ADMIN_CHAT_MALFORMED) => ChatLoad::Malformed,
        Some(ADMIN_CHAT_UNAVAILABLE) | None => ChatLoad::Unavailable,
        Some(_) => ChatLoad::Malformed,
    }
}

#[derive(Clone, Copy, PartialEq)]
enum ChatRoute {
    Inbox,
    Conversation,
}

impl ChatRoute {
    fn meta_title(self) -> &'static str {
        match self {
            Self::Inbox => "Support conversations",
            Self::Conversation => "Support conversation",
        }
    }

    fn surface(self) -> &'static str {
        match self {
            Self::Inbox => "inbox",
            Self::Conversation => "conversation",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Inbox => "Support conversations are unavailable",
            Self::Conversation => "This conversation cannot be verified",
        }
    }

    fn detail(self) -> &'static str {
        match self {
            Self::Inbox => {
                "No conversations, participants, messages, presence, unread counts, topics, statuses, assignments, or activity timestamps are shown because the backend did not provide an authoritative chat read response."
            }
            Self::Conversation => {
                "No participant, message, presence, assignment, status, ownership, or activity data is shown because the backend has not verified the requested conversation."
            }
        }
    }
}

/// `/chat` — authenticated chat inbox with backend-projected data or a truthful failure state.
pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    render_route(ctx, ChatRoute::Inbox, None)
}

/// `/chat/{id}` — authenticated conversation projection. The route value is
/// canonicalized before the backend lookup and never implies authorization.
pub fn render_conversation(ctx: &PageContext) -> (PageMeta, Element) {
    let route_reference = canonical_route_reference(ctx.params.get("id").map(String::as_str));
    render_route(ctx, ChatRoute::Conversation, route_reference)
}

fn render_route(
    ctx: &PageContext,
    route: ChatRoute,
    route_reference: Option<String>,
) -> (PageMeta, Element) {
    let meta = PageMeta::admin(route.meta_title());
    let retry_href = route_reference
        .as_deref()
        .map(conversation_href)
        .unwrap_or_else(|| CHAT_PATH.to_string());

    let load = match route {
        ChatRoute::Inbox => list_load(ctx),
        ChatRoute::Conversation => detail_load(ctx),
    };
    let mutation = match ctx.query_param(ADMIN_CHAT_MUTATION_PARAM).as_deref() {
        Some("success") | Some("conflict") | Some("forbidden") | Some("unavailable")
        | Some("malformed") => ctx.query_param(ADMIN_CHAT_MUTATION_PARAM),
        _ => None,
    };
    let selected_status = ctx
        .query_param("status")
        .filter(|value| {
            matches!(
                value.as_str(),
                "open" | "in_progress" | "resolved" | "closed"
            )
        })
        .unwrap_or_default();
    let selected_topic_id = ctx
        .query_param("topic_id")
        .and_then(|value| canonical_uuid(&value))
        .unwrap_or_default();

    (
        meta,
        rsx! {
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("the private support chat workspace".to_string()),
                // Keep route references out of the signed-out response. The
                // authenticated unavailable shell may offer a bounded retry,
                // but the login boundary returns only to the static inbox.
                return_url: Some(CHAT_PATH.to_string()),
                ChatSurface { route, retry_href, load, mutation, selected_status, selected_topic_id }
            }
        },
    )
}

/// Strip controls and cap the visible diagnostic value by Unicode scalar
/// count. Dioxus escapes the remaining display text at the HTML boundary.
fn canonical_route_reference(raw: Option<&str>) -> Option<String> {
    let raw = raw?.trim();
    canonical_uuid(raw)
}

fn canonical_uuid(value: &str) -> Option<String> {
    if value.len() != 36
        || !value.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                byte == b'-'
            } else {
                byte.is_ascii_hexdigit()
            }
        })
    {
        return None;
    }
    Some(value.to_ascii_lowercase())
}

/// Encode the already-bounded reference as one URL path segment. The display
/// value remains untrusted and unverified even when it is safe to navigate to.
fn conversation_href(reference: &str) -> String {
    let mut encoded = String::with_capacity(reference.len());
    for byte in reference.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(char::from(*byte));
            }
            _ => {
                const HEX: &[u8; 16] = b"0123456789ABCDEF";
                encoded.push('%');
                encoded.push(char::from(HEX[(byte >> 4) as usize]));
                encoded.push(char::from(HEX[(byte & 0x0f) as usize]));
            }
        }
    }
    format!("{CHAT_PATH}/{encoded}")
}

#[component]
fn ChatSurface(
    route: ChatRoute,
    retry_href: String,
    load: ChatLoad,
    mutation: Option<String>,
    selected_status: String,
    selected_topic_id: String,
) -> Element {
    match load {
        ChatLoad::Ready(inbox) => rsx! {
            ChatListReady { inbox, selected_status, selected_topic_id, state: ADMIN_CHAT_READY }
        },
        ChatLoad::Detail(detail) => rsx! { ChatDetailReady { detail, mutation } },
        ChatLoad::Empty(inbox) => rsx! {
            ChatListReady { inbox, selected_status, selected_topic_id, state: ADMIN_CHAT_EMPTY }
        },
        ChatLoad::Forbidden => rsx! {
            ChatUnavailable {
                route,
                state: ADMIN_CHAT_FORBIDDEN,
                title: "Chat access was denied".to_string(),
                detail: "The backend did not authorize this session to read support conversations.".to_string(),
                retry_href: retry_href.clone(),
            }
        },
        ChatLoad::Malformed => rsx! {
            ChatUnavailable {
                route,
                state: ADMIN_CHAT_MALFORMED,
                title: "Chat data could not be verified".to_string(),
                detail: "The backend response did not match the strict chat projection. No records are shown.".to_string(),
                retry_href: retry_href.clone(),
            }
        },
        ChatLoad::Unavailable => rsx! {
            ChatUnavailable {
                route,
                state: ADMIN_CHAT_UNAVAILABLE,
                title: route.title().to_string(),
                detail: route.detail().to_string(),
                retry_href,
            }
        },
    }
}

#[component]
fn ChatPageHeader() -> Element {
    rsx! {
        header { class: "mb-4 flex items-center gap-3 md:mb-6",
            div { class: "flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-violet-500 to-purple-600 text-white shadow-sm shadow-violet-500/20",
                Icon { name: "message-circle".to_string(), size: Some(20) }
            }
            div {
                h1 { class: "text-2xl font-bold tracking-tight text-foreground", "Chat Support" }
                p { class: "text-xs text-muted-foreground/60", "Manage support conversations" }
            }
        }
    }
}

#[component]
fn ChatStatsPanel(stats: Option<AdminChatStats>) -> Element {
    let cards = [
        (
            "message-circle",
            "Open",
            stats.as_ref().map(|value| value.total_open),
            "from-amber-500 to-orange-500",
            "bg-amber-500/15 text-amber-400",
        ),
        (
            "clock",
            "In Progress",
            stats.as_ref().map(|value| value.total_in_progress),
            "from-cyan-500 to-blue-500",
            "bg-cyan-500/15 text-cyan-400",
        ),
        (
            "check-circle",
            "Resolved",
            stats.as_ref().map(|value| value.total_resolved),
            "from-emerald-500 to-green-500",
            "bg-emerald-500/15 text-emerald-400",
        ),
        (
            "users",
            "Unassigned",
            stats.as_ref().map(|value| value.total_unassigned),
            "from-rose-500 to-red-500",
            "bg-rose-500/15 text-rose-400",
        ),
    ];
    rsx! {
        section { class: "mb-6 grid grid-cols-2 gap-3 md:grid-cols-4", aria_label: "Chat support summary",
            for (icon, label, value, accent, icon_class) in cards {
                article { class: "relative overflow-hidden rounded-xl border border-border/20 bg-card p-4",
                    div { class: "absolute inset-x-0 top-0 h-0.5 bg-gradient-to-r {accent} opacity-60" }
                    div { class: "mb-3 flex items-center justify-between",
                        span { class: "rounded-xl p-2 {icon_class}",
                            Icon { name: icon.to_string(), size: Some(18) }
                        }
                    }
                    if let Some(value) = value {
                        p { class: "text-3xl font-black tracking-tight text-foreground", "{value}" }
                    } else {
                        p { class: "text-xl font-black tracking-tight text-amber-400 md:text-2xl", "Unavailable" }
                    }
                    p { class: "mt-1 text-xs font-medium uppercase tracking-wider text-muted-foreground", "{label}" }
                }
            }
        }
    }
}

#[component]
fn ChatFilterBar(
    topics: Vec<AdminChatTopicSummary>,
    selected_status: String,
    selected_topic_id: String,
) -> Element {
    rsx! {
        form { method: "get", action: CHAT_PATH, class: "mb-3 flex flex-wrap items-center gap-2 rounded-xl border border-border/20 bg-card p-2.5",
            Icon { name: "sliders-horizontal".to_string(), size: Some(14) }
            select { class: "input h-9 min-w-40 flex-1 text-xs font-medium", name: "status", aria_label: "Conversation status",
                option { value: "", selected: selected_status.is_empty(), "All Status" }
                option { value: "open", selected: selected_status == "open", "Open" }
                option { value: "in_progress", selected: selected_status == "in_progress", "In Progress" }
                option { value: "resolved", selected: selected_status == "resolved", "Resolved" }
                option { value: "closed", selected: selected_status == "closed", "Closed" }
            }
            select { class: "input h-9 min-w-40 flex-1 text-xs font-medium", name: "topic_id", aria_label: "Conversation topic",
                option { value: "", selected: selected_topic_id.is_empty(), "All Topics" }
                for topic in topics.iter().filter(|topic| topic.is_active) {
                    option { value: "{topic.id}", selected: selected_topic_id == topic.id, "{topic.label}" }
                }
            }
            input { r#type: "hidden", name: "limit", value: "20" }
            button { class: "btn btn-sm btn-primary", r#type: "submit", "Apply" }
            a { class: "btn btn-sm btn-ghost", href: CHAT_PATH, "Reset" }
        }
    }
}

#[component]
fn ChatFilterBarUnavailable() -> Element {
    rsx! {
        div { class: "mb-3 flex flex-wrap items-center gap-2 rounded-xl border border-border/20 bg-card p-2.5",
            Icon { name: "sliders-horizontal".to_string(), size: Some(14) }
            select { class: "input h-9 min-w-40 flex-1 text-xs font-medium", disabled: true, aria_label: "Conversation status unavailable",
                option { "All Status" }
            }
            select { class: "input h-9 min-w-40 flex-1 text-xs font-medium", disabled: true, aria_label: "Conversation topic unavailable",
                option { "All Topics" }
            }
            button { class: "btn btn-sm btn-outline", r#type: "button", disabled: true, "Apply" }
        }
    }
}

#[component]
fn ChatListReady(
    inbox: AdminChatInbox,
    selected_status: String,
    selected_topic_id: String,
    state: &'static str,
) -> Element {
    let list = inbox.conversations;
    let total = list.total;
    let page = list.page;
    let limit = list.limit;
    let has_next = list.has_next;
    let topics = inbox.topics;
    rsx! {
        section { class: "p-4 md:p-8", "data-admin-chat-state": state, "data-admin-chat-surface": "inbox",
            ChatPageHeader {}
            ChatStatsPanel { stats: Some(inbox.stats) }
            div { class: "grid min-h-[34rem] gap-4 md:grid-cols-3",
                aside { class: "flex min-w-0 flex-col",
                    ChatFilterBar { topics: topics.clone(), selected_status: selected_status.clone(), selected_topic_id: selected_topic_id.clone() }
                    div { class: "flex-1 space-y-2 overflow-y-auto pr-1", aria_label: "Support conversations",
                        if list.items.is_empty() {
                            div { class: "flex flex-col items-center justify-center py-16 text-center",
                                div { class: "mb-3 flex h-12 w-12 items-center justify-center rounded-xl border border-border/40 bg-muted/30",
                                    Icon { name: "inbox".to_string(), size: Some(24) }
                                }
                                p { class: "mb-1 text-sm font-medium text-muted-foreground", "No conversations" }
                                p { class: "text-xs text-muted-foreground/50", "The backend returned an authoritative empty result" }
                            }
                        } else {
                            for conversation in list.items {
                                ChatConversationCard { conversation, topics: topics.clone() }
                            }
                        }
                    }
                    ChatPagination { page, limit, total, has_next, selected_status, selected_topic_id }
                }
                section { class: "hidden min-h-[34rem] overflow-hidden rounded-2xl border border-border/20 bg-card md:col-span-2 md:flex md:flex-col md:items-center md:justify-center md:text-center",
                    div { class: "mb-4 flex h-16 w-16 items-center justify-center rounded-xl border border-border/40 bg-muted/30",
                        Icon { name: "message-circle".to_string(), size: Some(32) }
                    }
                    p { class: "mb-1 text-sm font-medium text-muted-foreground", "Select a conversation" }
                    p { class: "text-xs text-muted-foreground/40", "Choose from the left panel to view details" }
                }
            }
        }
    }
}

#[component]
fn ChatConversationCard(
    conversation: AdminChatConversationSummary,
    topics: Vec<AdminChatTopicSummary>,
) -> Element {
    let topic = topics
        .iter()
        .find(|topic| topic.id == conversation.topic_id)
        .map(|topic| topic.label.clone());
    let wallet = truncate_wallet(&conversation.wallet_address);
    let unread = conversation.unread_agent;
    rsx! {
        a { class: "block w-full rounded-xl border border-border/20 bg-card p-3.5 text-left transition-colors hover:border-violet-500/25 hover:bg-violet-500/5", href: conversation_href(&conversation.id),
            div { class: "mb-2 flex items-start justify-between gap-2",
                p { class: "line-clamp-1 text-sm font-semibold text-foreground/90", "{conversation.subject}" }
                if unread > 0 {
                    span { class: "flex h-[22px] min-w-[22px] flex-shrink-0 items-center justify-center rounded-full bg-gradient-to-r from-violet-500 to-purple-500 px-1 text-[10px] font-bold text-white",
                        if unread > 9 { "9+" } else { "{unread}" }
                    }
                }
            }
            div { class: "mb-2 flex flex-wrap items-center gap-2 text-muted-foreground",
                span { class: "flex items-center gap-1 font-mono text-[11px]",
                    Icon { name: "wallet".to_string(), size: Some(12) }
                    "{wallet}"
                }
                if let Some(topic) = topic {
                    span { class: "text-[10px] text-muted-foreground/30", "|" }
                    span { class: "text-[11px] font-semibold text-violet-400", "{topic}" }
                }
            }
            div { class: "flex items-center justify-between",
                span { class: "rounded-full border border-border/30 px-2 py-1 text-[10px] font-medium capitalize text-muted-foreground", "{conversation.status}" }
                span { class: "flex items-center gap-1 text-[10px] text-muted-foreground/60",
                    Icon { name: "clock".to_string(), size: Some(10) }
                    "{conversation.last_message_at}"
                }
            }
        }
    }
}

#[component]
fn ChatPagination(
    page: u32,
    limit: u32,
    total: i64,
    has_next: bool,
    selected_status: String,
    selected_topic_id: String,
) -> Element {
    let previous = page.checked_sub(1).filter(|value| *value >= 1);
    rsx! {
        nav { class: "mt-3 flex items-center justify-between gap-2 text-xs text-muted-foreground", aria_label: "Conversation pages",
            span { "{total} total · Page {page}" }
            div { class: "flex gap-2",
                if let Some(previous) = previous {
                    a { class: "btn btn-xs btn-outline", href: chat_list_href(&selected_status, &selected_topic_id, previous, limit), "Previous" }
                }
                if has_next {
                    a { class: "btn btn-xs btn-outline", href: chat_list_href(&selected_status, &selected_topic_id, page.saturating_add(1), limit), "Next" }
                }
            }
        }
    }
}

fn truncate_wallet(value: &str) -> String {
    let characters = value.chars().collect::<Vec<_>>();
    if characters.len() <= 14 {
        return value.to_string();
    }
    format!(
        "{}...{}",
        characters.iter().take(6).collect::<String>(),
        characters.iter().rev().take(4).rev().collect::<String>()
    )
}

fn chat_list_href(status: &str, topic_id: &str, page: u32, limit: u32) -> String {
    let mut query = vec![format!("page={page}"), format!("limit={limit}")];
    if matches!(status, "open" | "in_progress" | "resolved" | "closed") {
        query.push(format!("status={status}"));
    }
    if canonical_uuid(topic_id).is_some() {
        query.push(format!("topic_id={topic_id}"));
    }
    format!("{CHAT_PATH}?{}", query.join("&"))
}

#[component]
fn ChatDetailReady(detail: AdminChatDetail, mutation: Option<String>) -> Element {
    let conversation_id = detail.conversation.id.clone();
    rsx! {
        section {
            class: "p-4 md:p-8",
            "data-admin-chat-state": ADMIN_CHAT_READY,
            "data-admin-chat-surface": "conversation",
            div { class: "overflow-hidden rounded-2xl border border-border/20 bg-card",
                header { class: "border-b border-border/20 p-4 md:p-5",
                    div { class: "flex flex-wrap items-center justify-between gap-3",
                        div { class: "flex min-w-0 items-center gap-3",
                            a { class: "btn btn-sm btn-ghost", href: CHAT_PATH, aria_label: "Back to conversation list",
                                Icon { name: "arrow-left".to_string(), size: Some(16) }
                            }
                            div { class: "min-w-0",
                                h1 { class: "truncate text-lg font-bold text-foreground", "{detail.conversation.subject}" }
                                p { class: "mt-1 text-xs text-muted-foreground", "{detail.conversation.wallet_address} · {detail.conversation.last_message_at}" }
                            }
                        }
                        span { class: "rounded-full border border-violet-500/25 bg-violet-500/10 px-3 py-1 text-xs font-medium capitalize text-violet-300", "{detail.conversation.status}" }
                    }
                }
                if let Some(state) = mutation {
                    section { class: "m-4 rounded-xl border border-amber-500/30 bg-amber-500/5 p-4", role: if state == "forbidden" { "alert" } else { "status" },
                        "data-admin-chat-mutation-state": state,
                        p { class: "text-sm text-foreground", "Chat mutation: {state}" }
                    }
                }
                div { class: "min-h-[22rem] bg-background/20 p-4 md:p-6",
                    if detail.messages.is_empty() {
                        p { class: "flex min-h-[18rem] items-center justify-center text-sm text-muted-foreground", role: "status", "No messages were returned for this conversation." }
                    } else {
                        ol { class: "space-y-4", aria_label: "Conversation messages",
                            for message in detail.messages {
                                li { class: "max-w-3xl rounded-2xl border border-border/20 bg-card p-4",
                                    p { class: "text-[10px] font-medium uppercase tracking-wide text-muted-foreground", "{message.sender_type} · {message.created_at}" }
                                    p { class: "mt-2 whitespace-pre-wrap text-sm leading-6 text-foreground", "{message.content}" }
                                }
                            }
                        }
                    }
                }
                div { class: "grid gap-5 border-t border-border/20 p-4 lg:grid-cols-[minmax(0,1fr)_auto]",
                    form { method: "post", action: format!("/chat/{conversation_id}"), class: "space-y-3",
                        input { r#type: "hidden", name: "operation", value: "reply" }
                        input { r#type: "hidden", name: "idempotency_key", value: format!("admin.chat.reply.{}", uuid::Uuid::new_v4()) }
                        label { class: "sr-only", r#for: "chat-reply", "Reply" }
                        textarea { id: "chat-reply", class: "textarea textarea-bordered min-h-24 w-full", name: "content", maxlength: MAX_TEXT_CHARS, required: true, placeholder: "Type your reply..." }
                        button { r#type: "submit", class: "btn btn-primary",
                            Icon { name: "send".to_string(), size: Some(16) }
                            " Send reply"
                        }
                    }
                    div { class: "space-y-2",
                        form { method: "post", action: format!("/chat/{conversation_id}"), class: "flex flex-wrap items-center gap-2",
                            input { r#type: "hidden", name: "operation", value: "status" }
                            input { r#type: "hidden", name: "idempotency_key", value: format!("admin.chat.status.{}", uuid::Uuid::new_v4()) }
                            select { class: "select select-bordered select-sm", name: "status", aria_label: "Conversation status",
                                option { value: "open", selected: detail.conversation.status == "open", "Open" }
                                option { value: "in_progress", selected: detail.conversation.status == "in_progress", "In progress" }
                                option { value: "resolved", selected: detail.conversation.status == "resolved", "Resolved" }
                                option { value: "closed", selected: detail.conversation.status == "closed", "Closed" }
                            }
                            button { r#type: "submit", class: "btn btn-sm btn-outline", "Update" }
                        }
                        form { method: "post", action: format!("/chat/{conversation_id}"), class: "flex flex-wrap items-center gap-2",
                            input { r#type: "hidden", name: "operation", value: "assign" }
                            input { r#type: "hidden", name: "idempotency_key", value: format!("admin.chat.assign.{}", uuid::Uuid::new_v4()) }
                            input { class: "input input-bordered input-sm", name: "agent_address", maxlength: 42, placeholder: "Agent wallet 0x...", aria_label: "Agent wallet" }
                            button { r#type: "submit", class: "btn btn-sm btn-outline", "Assign" }
                        }
                        form { method: "post", action: format!("/chat/{conversation_id}"),
                            input { r#type: "hidden", name: "operation", value: "read" }
                            input { r#type: "hidden", name: "idempotency_key", value: format!("admin.chat.read.{}", uuid::Uuid::new_v4()) }
                            button { r#type: "submit", class: "btn btn-sm btn-ghost", "Mark messages read" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ChatUnavailable(
    route: ChatRoute,
    state: &'static str,
    title: String,
    detail: String,
    retry_href: String,
) -> Element {
    if route == ChatRoute::Conversation {
        return rsx! {
            section { class: "p-4 md:p-8", role: "status", "data-admin-chat-state": state, "data-admin-chat-surface": route.surface(),
                div { class: "overflow-hidden rounded-2xl border border-border/20 bg-card",
                    header { class: "flex items-center gap-3 border-b border-border/20 p-4",
                        a { class: "btn btn-sm btn-ghost", href: CHAT_PATH, aria_label: "Back to conversation list",
                            Icon { name: "arrow-left".to_string(), size: Some(16) }
                        }
                        p { class: "font-semibold text-foreground", "Conversation" }
                    }
                    div { class: "flex min-h-[34rem] flex-col items-center justify-center p-8 text-center",
                        div { class: "mb-4 flex h-16 w-16 items-center justify-center rounded-xl border border-amber-500/20 bg-amber-500/10 text-amber-400",
                            Icon { name: "message-circle".to_string(), size: Some(30) }
                        }
                        h1 { class: "text-xl font-bold text-foreground", "{title}" }
                        p { class: "mt-3 max-w-2xl text-sm leading-6 text-muted-foreground", "{detail}" }
                        nav { class: "mt-6 flex flex-wrap justify-center gap-2", aria_label: "Conversation recovery",
                            a { class: "btn btn-sm btn-primary", href: retry_href, "Try again" }
                            a { class: "btn btn-sm btn-outline", href: CHAT_PATH, "Conversation list" }
                            a { class: "btn btn-sm btn-ghost", href: "/", "Admin home" }
                        }
                    }
                }
            }
        };
    }
    rsx! {
        section { class: "p-4 md:p-8",
            role: "status",
            "data-admin-chat-state": state,
            "data-admin-chat-surface": route.surface(),
            ChatPageHeader {}
            ChatStatsPanel { stats: None }
            div { class: "grid min-h-[34rem] gap-4 md:grid-cols-3",
                aside { class: "flex min-w-0 flex-col",
                    ChatFilterBarUnavailable {}
                    div { class: "flex flex-1 flex-col items-center justify-center rounded-xl border border-border/20 bg-card px-6 py-16 text-center",
                        Icon { name: "inbox".to_string(), size: Some(28) }
                        p { class: "mt-3 text-sm font-medium text-muted-foreground", "Conversations unavailable" }
                        p { class: "mt-1 text-xs text-muted-foreground/50", "No unverified records are shown" }
                    }
                }
                section { class: "hidden min-h-[26rem] flex-col items-center justify-center rounded-2xl border border-amber-500/20 bg-card p-8 text-center md:col-span-2 md:flex",
                    div { class: "mb-4 flex h-14 w-14 items-center justify-center rounded-xl bg-amber-500/10 text-amber-400",
                        Icon { name: "message-circle".to_string(), size: Some(26) }
                    }
                    h2 { class: "text-xl font-bold text-foreground", "{title}" }
                    p { class: "mt-3 max-w-xl text-sm leading-6 text-muted-foreground", "{detail}" }
                    nav { class: "mt-6 flex flex-wrap justify-center gap-2", aria_label: "Chat recovery",
                        a { class: "btn btn-sm btn-primary", href: retry_href,
                            Icon { name: "refresh-cw".to_string(), size: Some(14) }
                            " Retry"
                        }
                        a { class: "btn btn-sm btn-ghost", href: "/", "Admin home" }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::auth::User;

    fn authenticated_ctx(path: &str) -> PageContext {
        PageContext {
            user: Some(User {
                id: "admin-session".to_string(),
                address: "0x1234".to_string(),
                chain_id: "56".to_string(),
                roles: vec![],
                permissions: vec![],
                ..Default::default()
            }),
            path: path.to_string(),
            ..Default::default()
        }
    }

    fn conversation_ctx(id: &str, signed_in: bool) -> PageContext {
        let mut ctx = authenticated_ctx("/chat/case-42");
        ctx.user = signed_in.then(|| ctx.user.take().expect("test user"));
        ctx.params = HashMap::from([("id".to_string(), id.to_string())]);
        ctx
    }

    fn html(element: Element) -> String {
        dioxus_ssr::render_element(element)
    }

    #[test]
    fn signed_out_routes_keep_chat_state_and_reference_private() {
        let inbox = html(
            render(&PageContext {
                path: CHAT_PATH.to_string(),
                ..Default::default()
            })
            .1,
        );
        let conversation = html(render_conversation(&conversation_ctx("private-case", false)).1);

        for rendered in [inbox, conversation] {
            assert!(rendered.contains("Sign in required"));
            assert!(!rendered.contains("data-admin-chat-state"));
            assert!(!rendered.contains("Support conversations are unavailable"));
            assert!(!rendered.contains("private-case"));
        }
    }

    #[test]
    fn roles_empty_authenticated_session_reaches_both_unavailable_surfaces() {
        let inbox = html(render(&authenticated_ctx(CHAT_PATH)).1);
        let conversation = html(render_conversation(&conversation_ctx("case-42", true)).1);

        assert!(inbox.contains("data-admin-chat-state=\"unavailable\""));
        assert!(inbox.contains("data-admin-chat-surface=\"inbox\""));
        assert!(conversation.contains("data-admin-chat-state=\"unavailable\""));
        assert!(conversation.contains("data-admin-chat-surface=\"conversation\""));
        assert!(conversation.contains("This conversation cannot be verified"));
        assert!(!inbox.contains("Permission required"));
        assert!(!conversation.contains("Permission required"));
    }

    #[test]
    fn unavailable_surfaces_preserve_workspace_without_records_or_enabled_actions() {
        let inbox = html(render(&authenticated_ctx(CHAT_PATH)).1);
        let conversation = html(render_conversation(&conversation_ctx("case-42", true)).1);
        let combined = format!("{inbox}{conversation}");

        for forbidden in [
            "Plan upgrade question",
            "Payment issue",
            "API key question",
            "Subscription renewal",
            "0x1234…5678",
            "Conversation assigned to Alex",
            "How about the 1st of next month?",
            "Total open",
            "Resolved (7d)",
            "Search conversations",
            "Saved replies",
            "Assign to me",
            "Type your reply",
            "Mark resolved",
            "Send reply",
            "chat-conversation-list",
            "chat-message-list",
            "<textarea",
            "<form",
        ] {
            assert!(!combined.contains(forbidden), "leaked chat UI: {forbidden}");
        }
        assert!(inbox.contains("Chat Support"));
        assert!(inbox.contains("All Status"));
        assert!(inbox.contains("All Topics"));
        assert!(inbox.contains("disabled"));
    }

    #[test]
    fn legacy_and_hostile_non_id_params_are_ignored() {
        let mut ctx = authenticated_ctx(CHAT_PATH);
        ctx.query = "status=open&presence=online&unread=99".to_string();
        ctx.params = HashMap::from([
            (
                "messages".to_string(),
                "Conversation assigned to Alex".to_string(),
            ),
            ("reply".to_string(), "Send reply".to_string()),
            ("assignee".to_string(), "0xADMIN0000…0001".to_string()),
        ]);
        let rendered = html(render(&ctx).1);

        assert!(rendered.contains("data-admin-chat-state=\"unavailable\""));
        for forbidden in [
            "status=open",
            "presence=online",
            "unread=99",
            "Conversation assigned to Alex",
            "Send reply",
            "0xADMIN0000…0001",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "legacy value leaked: {forbidden}"
            );
        }
    }

    #[test]
    fn conversation_reference_requires_a_canonical_uuid_and_never_reflects_raw_input() {
        let hostile = format!("\u{0}\n\t\"><script>alert(1)</script>{}", "a".repeat(100));
        assert!(canonical_route_reference(Some(&hostile)).is_none());

        let rendered = html(render_conversation(&conversation_ctx(&hostile, true)).1);
        assert!(!rendered.contains("<script>"));
        assert!(rendered.contains("href=\"/chat\""));
        assert!(!rendered.contains("alert(1)"));
    }

    #[test]
    fn recovery_uses_native_safe_links_without_mutation_handlers() {
        let inbox = html(render(&authenticated_ctx(CHAT_PATH)).1);
        let conversation = html(render_conversation(&conversation_ctx("case 42", true)).1);

        assert!(inbox.contains("href=\"/chat\""));
        assert!(inbox.contains("href=\"/\""));
        assert!(conversation.contains("href=\"/chat\""));
        assert!(conversation.contains("Conversation list"));
        assert!(conversation.contains("href=\"/\""));
        assert!(!inbox.contains("onclick="));
        assert!(!conversation.contains("onclick="));
        assert!(!conversation.contains("javascript:"));
    }

    #[test]
    fn canonical_conversation_id_is_the_only_dynamic_retry_target() {
        let id = "550e8400-e29b-41d4-a716-446655440000";
        let rendered = html(render_conversation(&conversation_ctx(id, true)).1);
        assert!(rendered.contains("href=\"/chat/550e8400-e29b-41d4-a716-446655440000\""));
        assert!(!rendered.contains("case-42"));
    }
}

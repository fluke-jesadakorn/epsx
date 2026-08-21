//! Backend-backed support chat for `/chat`.
//!
//! The frontend renders only the owner-scoped projection supplied by the BFF.
//! Native forms keep create/send/resolve usable without client hydration while
//! the backend remains authoritative for ownership and state transitions.

use chrono::DateTime;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::auth::AuthGate;
use crate::chat::{Message, MessageBubble};
use crate::layout::main_layout::MainLayout;
use crate::primitives::Icon;

use super::{PageContext, PageMeta};

pub const CHAT_INBOX_DATA_PARAM: &str = "data_chat_inbox";
pub const CHAT_INBOX_STATE_PARAM: &str = "data_chat_inbox_state";
pub const CHAT_DETAIL_DATA_PARAM: &str = "data_chat_detail";
pub const CHAT_DETAIL_STATE_PARAM: &str = "data_chat_detail_state";

pub const CHAT_READY: &str = "ready";
pub const CHAT_EMPTY: &str = "empty";
pub const CHAT_FORBIDDEN: &str = "forbidden";
pub const CHAT_UNAVAILABLE: &str = "unavailable";
pub const CHAT_MALFORMED: &str = "malformed";

const MAX_TOPICS: usize = 100;
const MAX_CONVERSATIONS: usize = 200;
const MAX_MESSAGES: usize = 500;
const MAX_MESSAGE_CHARS: usize = 16_384;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChatTopic {
    pub id: String,
    pub name: String,
    pub label: String,
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChatConversation {
    pub id: String,
    pub topic_id: String,
    pub subject: String,
    pub status: String,
    pub assigned_agent: Option<String>,
    pub last_message_at: String,
    pub unread_user: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChatMessage {
    pub id: String,
    pub conversation_id: String,
    pub sender_type: String,
    pub content: String,
    pub is_read: bool,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChatInboxData {
    pub topics: Vec<ChatTopic>,
    pub conversations: Vec<ChatConversation>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChatDetailData {
    pub conversation: ChatConversation,
    pub messages: Vec<ChatMessage>,
}

pub fn decode_chat_inbox(value: serde_json::Value) -> Option<ChatInboxData> {
    let inbox = serde_json::from_value::<ChatInboxData>(value).ok()?;
    if inbox.topics.len() > MAX_TOPICS
        || inbox.conversations.len() > MAX_CONVERSATIONS
        || inbox.topics.iter().any(|topic| !valid_topic(topic))
        || inbox
            .conversations
            .iter()
            .any(|conversation| !valid_conversation(conversation))
    {
        return None;
    }
    let topic_ids = inbox
        .topics
        .iter()
        .map(|topic| topic.id.as_str())
        .collect::<std::collections::HashSet<_>>();
    inbox
        .conversations
        .iter()
        .all(|conversation| topic_ids.contains(conversation.topic_id.as_str()))
        .then_some(inbox)
}

pub fn decode_chat_detail(value: serde_json::Value) -> Option<ChatDetailData> {
    let detail = serde_json::from_value::<ChatDetailData>(value).ok()?;
    if !valid_conversation(&detail.conversation)
        || detail.messages.len() > MAX_MESSAGES
        || detail.messages.iter().any(|message| {
            !valid_message(message) || message.conversation_id != detail.conversation.id
        })
    {
        return None;
    }
    Some(detail)
}

fn valid_topic(topic: &ChatTopic) -> bool {
    valid_uuid(&topic.id)
        && bounded_text(&topic.name, 64)
        && bounded_text(&topic.label, 128)
        && topic
            .description
            .as_deref()
            .is_none_or(|value| bounded_text(value, 512))
        && topic
            .icon
            .as_deref()
            .is_none_or(|value| bounded_text(value, 64))
}

fn valid_conversation(conversation: &ChatConversation) -> bool {
    valid_uuid(&conversation.id)
        && valid_uuid(&conversation.topic_id)
        && bounded_text(&conversation.subject, 255)
        && matches!(
            conversation.status.as_str(),
            "open" | "in_progress" | "resolved" | "closed"
        )
        && conversation
            .assigned_agent
            .as_deref()
            .is_none_or(|value| bounded_text(value, 128))
        && conversation.unread_user >= 0
        && valid_timestamp(&conversation.last_message_at)
        && valid_timestamp(&conversation.created_at)
        && valid_timestamp(&conversation.updated_at)
}

fn valid_message(message: &ChatMessage) -> bool {
    valid_uuid(&message.id)
        && valid_uuid(&message.conversation_id)
        && matches!(
            message.sender_type.as_str(),
            "user" | "agent" | "ai" | "system"
        )
        && bounded_message(&message.content, MAX_MESSAGE_CHARS)
        && valid_timestamp(&message.created_at)
}

fn valid_uuid(value: &str) -> bool {
    uuid::Uuid::parse_str(value).is_ok_and(|id| id.to_string() == value)
}

fn bounded_text(value: &str, max_chars: usize) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= max_chars
        && !value.chars().any(char::is_control)
}

fn bounded_message(value: &str, max_chars: usize) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= max_chars
        && !value
            .chars()
            .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
}

fn valid_timestamp(value: &str) -> bool {
    value.len() <= 64 && DateTime::parse_from_rfc3339(value).is_ok()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ChatInboxLoad {
    Ready(ChatInboxData),
    Empty(ChatInboxData),
    Forbidden,
    Unavailable,
    Malformed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ChatDetailLoad {
    Ready(Box<ChatDetailData>),
    Forbidden,
    Unavailable,
    Malformed,
}

pub(crate) fn inbox_load(ctx: &PageContext) -> ChatInboxLoad {
    let state = ctx.params.get(CHAT_INBOX_STATE_PARAM).map(String::as_str);
    match state {
        Some(CHAT_READY) | Some(CHAT_EMPTY) => {
            let Some(inbox) = ctx
                .params
                .get(CHAT_INBOX_DATA_PARAM)
                .and_then(|raw| serde_json::from_str(raw).ok())
                .and_then(decode_chat_inbox)
            else {
                return ChatInboxLoad::Malformed;
            };
            let is_empty = inbox.conversations.is_empty();
            if (state == Some(CHAT_EMPTY)) != is_empty {
                return ChatInboxLoad::Malformed;
            }
            if is_empty {
                ChatInboxLoad::Empty(inbox)
            } else {
                ChatInboxLoad::Ready(inbox)
            }
        }
        Some(CHAT_FORBIDDEN) => ChatInboxLoad::Forbidden,
        Some(CHAT_MALFORMED) => ChatInboxLoad::Malformed,
        Some(CHAT_UNAVAILABLE) | None => ChatInboxLoad::Unavailable,
        Some(_) => ChatInboxLoad::Malformed,
    }
}

pub(crate) fn detail_load(ctx: &PageContext) -> ChatDetailLoad {
    match ctx.params.get(CHAT_DETAIL_STATE_PARAM).map(String::as_str) {
        Some(CHAT_READY) => ctx
            .params
            .get(CHAT_DETAIL_DATA_PARAM)
            .and_then(|raw| serde_json::from_str(raw).ok())
            .and_then(decode_chat_detail)
            .map(Box::new)
            .map(ChatDetailLoad::Ready)
            .unwrap_or(ChatDetailLoad::Malformed),
        Some(CHAT_FORBIDDEN) => ChatDetailLoad::Forbidden,
        Some(CHAT_MALFORMED) => ChatDetailLoad::Malformed,
        Some(CHAT_UNAVAILABLE) | None => ChatDetailLoad::Unavailable,
        Some(_) => ChatDetailLoad::Malformed,
    }
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let mut meta = PageMeta::app("Support Center");
    meta.body_class = Some("page-bg".to_string());
    (meta, rsx! { RenderChatInbox { ctx: ctx.clone() } })
}

#[component]
fn RenderChatInbox(ctx: PageContext) -> Element {
    let signed_in = ctx.user.is_some() || ctx.wallet.address.is_some();
    rsx! {
        MainLayout { ctx: ctx.clone(),
            if signed_in {
                AuthGate {
                    user: ctx.user.clone(),
                    feature: Some("private support chat".to_string()),
                    return_url: Some("/chat".to_string()),
                    wallet_connected: ctx.wallet.address.is_some(),
                    ChatInboxSurface { ctx: ctx.clone() }
                }
            } else {
                RenderPublicChat {}
            }
        }
    }
}

#[component]
fn ChatInboxSurface(ctx: PageContext) -> Element {
    match inbox_load(&ctx) {
        ChatInboxLoad::Ready(inbox) | ChatInboxLoad::Empty(inbox) => {
            rsx! { ChatInboxReady { ctx, inbox } }
        }
        ChatInboxLoad::Forbidden => rsx! { ChatProblem {
            title: "Support chat is not available for this account".to_string(),
            detail: "Your current account cannot access private support conversations.".to_string()
        } },
        ChatInboxLoad::Malformed => rsx! { ChatProblem {
            title: "Chat data could not be verified".to_string(),
            detail: "The support response did not match the expected contract. No conversation data is shown.".to_string()
        } },
        ChatInboxLoad::Unavailable => rsx! { ChatProblem {
            title: "Support chat is temporarily unavailable".to_string(),
            detail: "We could not reach the support service. Please try again shortly.".to_string()
        } },
    }
}

#[component]
fn ChatInboxReady(ctx: PageContext, inbox: ChatInboxData) -> Element {
    let show_new = inbox.conversations.is_empty()
        || url::form_urlencoded::parse(ctx.query.as_bytes())
            .any(|(key, value)| key == "new" && value == "1");
    let active = match detail_load(&ctx) {
        ChatDetailLoad::Ready(detail) => Some(*detail),
        ChatDetailLoad::Forbidden | ChatDetailLoad::Unavailable | ChatDetailLoad::Malformed => None,
    };
    let active_id = active
        .as_ref()
        .map(|detail| detail.conversation.id.as_str());
    let active_topic = active.as_ref().and_then(|detail| {
        inbox
            .topics
            .iter()
            .find(|topic| topic.id == detail.conversation.topic_id)
            .map(|topic| topic.label.clone())
    });
    let flash = mutation_flash(&ctx.query);

    rsx! {
        div {
            class: "fixed top-14 inset-x-0 bottom-0 overflow-hidden chat-page",
            "data-chat-state": if inbox.conversations.is_empty() { CHAT_EMPTY } else { CHAT_READY },
            div { class: if show_new { "chat-inbox-row chat-new-active" } else { "chat-inbox-row" }, style: "height:100%; min-height:0; border-radius:0; border-left:0; border-right:0;",
                aside { class: "chat-inbox",
                    div { class: "chat-inbox-header",
                        div { class: "chat-inbox-brand",
                            div { class: "chat-inbox-avatar",
                                Icon { name: "headset".to_string(), size: Some(20) }
                            }
                            div { class: "chat-inbox-titles",
                                h1 { class: "chat-inbox-title", "Support Center" }
                                p { class: "chat-inbox-subtitle", "Usually replies in minutes" }
                            }
                            span { class: "chat-inbox-count", "{inbox.conversations.len()}" }
                        }
                    }
                    div { class: "chat-inbox-list", aria_label: "Support conversations",
                        if inbox.conversations.is_empty() {
                            div { class: "chat-inbox-empty",
                                div { class: "chat-inbox-empty-icon",
                                    Icon { name: "inbox".to_string(), size: Some(20) }
                                }
                                p { class: "chat-inbox-empty-title", "No conversations yet" }
                                p { class: "chat-inbox-empty-hint", "Start a conversation to get help." }
                            }
                        } else {
                            for conversation in inbox.conversations.iter() {
                                ConversationCard {
                                    conversation: conversation.clone(),
                                    topic: inbox.topics.iter().find(|topic| topic.id == conversation.topic_id).cloned(),
                                    selected: active_id == Some(conversation.id.as_str())
                                }
                            }
                        }
                    }
                    div { class: "chat-inbox-newbar",
                        a { class: "chat-inbox-new", href: "/chat?new=1",
                            Icon { name: "plus".to_string(), size: Some(14) }
                            "New Conversation"
                        }
                    }
                }
                if show_new {
                    NewConversationPanel { topics: inbox.topics.clone(), flash }
                } else if let Some(detail) = active {
                    ConversationPanel {
                        detail,
                        topic_label: active_topic.unwrap_or_else(|| "Support".to_string()),
                        standalone: false,
                        flash
                    }
                } else {
                    section { class: "chat-panel chat-panel-empty", role: "status",
                        div { class: "chat-panel-empty-icon",
                            Icon { name: "message-circle".to_string(), size: Some(32) }
                        }
                        h2 { class: "chat-panel-empty-title", "Select a conversation" }
                        p { class: "chat-panel-empty-hint", "Choose a conversation from the inbox or start a new one." }
                    }
                }
            }
        }
    }
}

#[component]
fn ConversationCard(
    conversation: ChatConversation,
    topic: Option<ChatTopic>,
    selected: bool,
) -> Element {
    let class = if selected {
        "chat-inbox-card chat-inbox-card-selected"
    } else if conversation.unread_user > 0 {
        "chat-inbox-card chat-inbox-card-unread"
    } else {
        "chat-inbox-card"
    };
    rsx! {
        a { class, href: format!("/chat/{}", conversation.id),
            div { class: "chat-inbox-card-row",
                p { class: "chat-inbox-subject", "{conversation.subject}" }
                div { class: "chat-inbox-card-meta",
                    if conversation.unread_user > 0 {
                        span { class: "chat-inbox-unread", "{conversation.unread_user.min(99)}" }
                    }
                    span { class: "chat-inbox-time", "{short_date(&conversation.last_message_at)}" }
                }
            }
            div { class: "chat-inbox-card-foot",
                if let Some(topic) = topic {
                    span { class: "chat-inbox-topic", "{topic.label}" }
                }
                StatusBadge { status: conversation.status.clone() }
            }
        }
    }
}

#[component]
fn NewConversationPanel(topics: Vec<ChatTopic>, flash: Option<String>) -> Element {
    rsx! {
        section { class: "chat-panel chat-panel-new", "data-chat-surface": "new-conversation",
            a { class: "chat-panel-back chat-mobile-back", href: "/chat",
                Icon { name: "arrow-left".to_string(), size: Some(14) }
                "Back to conversations"
            }
            if let Some(message) = flash {
                ChatFlash { message }
            }
            div { class: "chat-topic-selector",
                h2 { class: "chat-topic-title", "How can we help?" }
                p { class: "chat-topic-subtitle", "Choose a topic and tell our support team what you need." }
                if topics.is_empty() {
                    p { class: "chat-panel-empty-hint", role: "status", "No support topics are available right now." }
                } else {
                    form { method: "post", action: "/chat", class: "chat-topic-composer",
                        fieldset { class: "chat-topic-grid",
                            legend { class: "chat-topic-form-label", "Topic" }
                            for (index, topic) in topics.iter().enumerate() {
                                label { class: "chat-topic-card",
                                    input {
                                        class: "sr-only",
                                        r#type: "radio",
                                        name: "topic_id",
                                        value: "{topic.id}",
                                        required: true,
                                        checked: index == 0,
                                    }
                                    div { class: "chat-topic-card-icon",
                                        Icon { name: topic.icon.clone().unwrap_or_else(|| "message-circle".to_string()), size: Some(16) }
                                    }
                                    div { class: "chat-topic-card-titles",
                                        p { class: "chat-topic-card-label", "{topic.label}" }
                                        if let Some(description) = &topic.description {
                                            p { class: "chat-topic-card-description", "{description}" }
                                        }
                                    }
                                }
                            }
                        }
                        label { class: "chat-topic-form-label",
                            "Subject"
                            input {
                                class: "chat-topic-form-input",
                                name: "subject",
                                maxlength: "255",
                                required: true,
                                autocomplete: "off",
                                placeholder: "Briefly describe what you need",
                            }
                        }
                        label { class: "chat-topic-form-label",
                            "Message"
                            textarea {
                                class: "chat-topic-form-textarea",
                                name: "message",
                                maxlength: "16384",
                                required: true,
                                placeholder: "Share the details with our support team...",
                            }
                        }
                        button { class: "chat-topic-start", r#type: "submit",
                            Icon { name: "send".to_string(), size: Some(16) }
                            "Start Conversation"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub(crate) fn ConversationPanel(
    detail: ChatDetailData,
    topic_label: String,
    standalone: bool,
    flash: Option<String>,
) -> Element {
    let root_class = if standalone {
        "chat-conv"
    } else {
        "chat-panel"
    };
    let can_resolve = matches!(detail.conversation.status.as_str(), "open" | "in_progress");
    let can_send = detail.conversation.status != "closed";
    let action = format!("/chat/{}", detail.conversation.id);
    rsx! {
        section { class: root_class, "data-chat-conversation-state": CHAT_READY,
            div { class: "chat-header",
                div { class: "chat-header-accent" }
                div { class: "chat-header-row",
                    if standalone {
                        a { class: "chat-conv-back", href: "/chat", aria_label: "Back to inbox",
                            Icon { name: "arrow-left".to_string(), size: Some(18) }
                        }
                    }
                    div { class: "chat-header-avatar",
                        Icon { name: "headset".to_string(), size: Some(18) }
                    }
                    div { class: "chat-header-titles",
                        h1 { class: "chat-header-subject", "{detail.conversation.subject}" }
                        div { class: "chat-conv-header-meta",
                            span { class: "chat-conv-header-topic", "{topic_label}" }
                            StatusBadge { status: detail.conversation.status.clone() }
                        }
                    }
                    if can_resolve {
                        form { method: "post", action: action.clone(),
                            input { r#type: "hidden", name: "operation", value: "resolve" }
                            button { class: "chat-header-resolve", r#type: "submit",
                                Icon { name: "check-circle".to_string(), size: Some(13) }
                                "Resolve"
                            }
                        }
                    }
                }
            }
            if let Some(message) = flash {
                ChatFlash { message }
            }
            div { class: "chat-messages", aria_live: "polite", aria_label: "Conversation messages",
                if detail.messages.is_empty() {
                    div { class: "chat-panel-empty", role: "status",
                        p { class: "chat-panel-empty-title", "No messages were returned." }
                    }
                } else {
                    for message in detail.messages.iter() {
                        MessageBubble {
                            message: bubble_message(message),
                            is_own_message: message.sender_type == "user"
                        }
                    }
                }
            }
            div { class: "chat-input",
                if can_send {
                    form { method: "post", action,
                        input { r#type: "hidden", name: "operation", value: "send" }
                        div { class: "chat-input-row",
                            textarea {
                                class: "chat-input-textarea",
                                name: "content",
                                rows: "1",
                                maxlength: "16384",
                                required: true,
                                placeholder: "Type a message...",
                                aria_label: "Message",
                            }
                            button { class: "chat-input-send", r#type: "submit", aria_label: "Send message",
                                Icon { name: "send".to_string(), size: Some(16) }
                            }
                        }
                        p { class: "chat-input-hint", "Write your message, then choose Send." }
                    }
                } else {
                    p { class: "chat-input-hint", role: "status", "This conversation is closed." }
                }
            }
        }
    }
}

#[component]
pub(crate) fn StatusBadge(status: String) -> Element {
    let class = match status.as_str() {
        "open" => "chat-status chat-status-open",
        "in_progress" => "chat-status chat-status-progress",
        "resolved" => "chat-status chat-status-resolved",
        _ => "chat-status chat-status-closed",
    };
    let label = match status.as_str() {
        "in_progress" => "In progress",
        "resolved" => "Resolved",
        "closed" => "Closed",
        _ => "Open",
    };
    rsx! {
        span { class,
            span { class: "chat-status-dot", aria_hidden: "true" }
            "{label}"
        }
    }
}

#[component]
fn ChatFlash(message: String) -> Element {
    rsx! {
        div {
            class: "mx-4 mt-3 rounded-xl border border-[#1fc7d4]/25 bg-[#1fc7d4]/8 px-4 py-2 text-xs text-foreground",
            role: "status",
            "data-chat-mutation-state": "complete",
            "{message}"
        }
    }
}

#[component]
fn ChatProblem(title: String, detail: String) -> Element {
    rsx! {
        div { class: "container page-content chat-page",
            section { class: "chat-panel chat-panel-empty", role: "alert", "data-chat-state": CHAT_UNAVAILABLE,
                div { class: "chat-panel-empty-icon",
                    Icon { name: "message-circle".to_string(), size: Some(32) }
                }
                h1 { class: "chat-panel-empty-title", "{title}" }
                p { class: "chat-panel-empty-hint", "{detail}" }
                a { class: "btn btn-outline mt-4", href: "/chat", "Try again" }
            }
        }
    }
}

pub(crate) fn mutation_flash(query: &str) -> Option<String> {
    let mut result = None;
    for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
        if key != "chat" || result.is_some() {
            continue;
        }
        result = match value.as_ref() {
            "created" => Some("Conversation started and your message was sent.".to_string()),
            "sent" => Some("Message sent.".to_string()),
            "resolved" => Some("Conversation marked as resolved.".to_string()),
            "error" => {
                Some("The support action could not be completed. Please try again.".to_string())
            }
            _ => None,
        };
    }
    result
}

fn bubble_message(message: &ChatMessage) -> Message {
    let sender_role = match message.sender_type.as_str() {
        "ai" => "AI Assistant",
        "agent" => "Support",
        _ => "",
    };
    Message {
        id: message.id.clone(),
        sender_name: sender_role.to_string(),
        sender_role: sender_role.to_string(),
        body: message.content.clone(),
        created_at: short_date(&message.created_at),
        is_read: message.is_read,
        is_own: message.sender_type == "user",
        is_system: message.sender_type == "system",
        sender_type: if message.sender_type == "ai" {
            "ai"
        } else {
            "support"
        }
        .to_string(),
        attachment: None,
    }
}

pub(crate) fn short_date(value: &str) -> String {
    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.format("%b %-d, %H:%M").to_string())
        .unwrap_or_else(|_| value.to_string())
}

/// Signed-out `/chat` mirrors the production support entry point.
#[component]
fn RenderPublicChat() -> Element {
    rsx! {
        div { class: "container mx-auto max-w-xl px-4 py-12 chat-public-page",
            style: "max-width: 36rem; width: 100%; margin-left: auto; margin-right: auto; padding: 3rem 1rem; box-sizing: border-box;",
            div { class: "mb-8 flex items-center gap-4",
                div { class: "flex h-12 w-12 items-center justify-center rounded-2xl bg-gradient-to-br from-[#7645d9] to-[#1fc7d4] shadow-lg shadow-[#7645d9]/25",
                    Icon { name: "headset".to_string(), size: Some(24), class_name: Some("text-white".to_string()) }
                }
                div {
                    h1 { class: "text-xl font-bold tracking-tight", "Support Center" }
                    p { class: "mt-0.5 text-xs text-muted-foreground", "Get help from our team · Usually replies in minutes" }
                }
            }
            div { class: "relative mb-6 overflow-hidden rounded-2xl border border-purple-500/30 bg-gradient-to-r from-purple-900/40 via-purple-800/30 to-pink-900/40 backdrop-blur-sm",
                div { class: "relative flex flex-col gap-4 p-5 sm:flex-row sm:items-center sm:justify-between",
                    div { class: "flex items-start gap-4",
                        div { class: "flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-gradient-to-br from-purple-500 to-pink-500 shadow-lg shadow-purple-500/30",
                            Icon { name: "lock".to_string(), size: Some(24), class_name: Some("text-white".to_string()) }
                        }
                        div {
                            p { class: "text-base font-bold text-white", "Sign in to access Support Chat" }
                            p { class: "mt-0.5 text-sm text-purple-300/80", "Connect your wallet to start a conversation with our team" }
                            div { class: "mt-2 flex flex-wrap gap-3",
                                span { class: "flex items-center gap-1 text-xs text-purple-300/70", "⌁ Top 100 stock rankings" }
                                span { class: "flex items-center gap-1 text-xs text-purple-300/70", "▥ Real-time EPS data" }
                                span { class: "flex items-center gap-1 text-xs text-purple-300/70", "ϟ AI-powered insights" }
                            }
                        }
                    }
                    a { class: "group inline-flex shrink-0 items-center gap-2 rounded-xl bg-gradient-to-r from-purple-500 to-pink-500 px-6 py-3 text-sm font-semibold text-white shadow-lg shadow-purple-500/30 transition-all duration-200 hover:-translate-y-0.5 hover:shadow-xl hover:shadow-purple-500/40 focus:outline-none",
                        href: "/auth?return_url=%2Fchat",
                        Icon { name: "log-in".to_string(), size: Some(16) }
                        "Sign In to Chat"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_inbox() -> ChatInboxData {
        ChatInboxData {
            topics: vec![ChatTopic {
                id: "17ea1b05-5ec1-4b6e-9e5a-13751ec2ed6d".into(),
                name: "general".into(),
                label: "General".into(),
                description: Some("General questions".into()),
                icon: Some("message-circle".into()),
            }],
            conversations: Vec::new(),
        }
    }

    #[test]
    fn chat_projection_is_strict_and_bounded() {
        let value = serde_json::to_value(sample_inbox()).unwrap();
        assert!(decode_chat_inbox(value).is_some());

        let mut unexpected = serde_json::to_value(sample_inbox()).unwrap();
        unexpected["client_authority"] = serde_json::json!(true);
        assert!(decode_chat_inbox(unexpected).is_none());
    }

    #[test]
    fn message_allows_newlines_but_rejects_other_controls() {
        assert!(bounded_message("hello\nworld", MAX_MESSAGE_CHARS));
        assert!(!bounded_message("hello\u{0000}world", MAX_MESSAGE_CHARS));
    }

    #[test]
    fn signed_out_surface_matches_production_entry_point() {
        let (_, element) = render(&PageContext {
            path: "/chat".into(),
            ..Default::default()
        });
        let html = dioxus_ssr::render_element(element);
        assert!(html.contains("Sign in to access Support Chat"));
        assert!(html.contains("href=\"/auth?return_url=%2Fchat\""));
        assert!(!html.contains("data_chat_inbox"));
    }
}

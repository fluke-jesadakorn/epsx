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
pub struct ChatAttachment {
    pub filename: String,
    pub url: String,
    pub file_type: String,
    pub size: u64,
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
    #[serde(default)]
    pub attachment: Option<ChatAttachment>,
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
        && message.attachment.as_ref().is_none_or(|att| {
            !att.filename.trim().is_empty()
                && att.filename.chars().count() <= 255
                && !att.filename.chars().any(char::is_control)
                && !att.url.trim().is_empty()
                && att.url.len() <= 2048
                && att.file_type.len() <= 128
                && att.size <= 10 * 1024 * 1024
        })
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
            class: "chat-page chat-page-full",
            style: "position:fixed;top:3.5rem;left:0;right:0;bottom:0;overflow:hidden; height:calc(100dvh - 3.5rem); display:flex; flex-direction:column;",
            "data-chat-state": if inbox.conversations.is_empty() { CHAT_EMPTY } else { CHAT_READY },
            div { class: if show_new { "chat-inbox-row chat-new-active chat-full" } else { "chat-inbox-row chat-full" }, style: "flex:1; min-height:0; height:100%; border-radius:0; border:0;",
                aside { class: "chat-inbox",
                    div { class: "chat-inbox-header",
                        div { class: "chat-inbox-brand",
                            div { class: "chat-inbox-avatar",
                                Icon { name: "headset".to_string(), size: Some(20) }
                                span { class: "chat-inbox-online-dot", aria_hidden: "true" }
                            }
                            div { class: "chat-inbox-titles",
                                h1 { class: "chat-inbox-title", "Support Center" }
                                p { class: "chat-inbox-subtitle", "Usually replies in minutes" }
                            }
                            if !inbox.conversations.is_empty() {
                                span { class: "chat-inbox-count", "{inbox.conversations.len()}" }
                            }
                        }
                    }
                    div { class: "chat-inbox-search",
                        Icon { name: "search".to_string(), size: Some(14) }
                        input {
                            class: "chat-inbox-search-input",
                            r#type: "search",
                            placeholder: "Search conversations...",
                            aria_label: "Search conversations",
                            autocomplete: "off",
                            "data-chat-search": "true"
                        }
                    }
                    div { class: "chat-inbox-filters",
                        select { class: "chat-inbox-filter", aria_label: "Filter by status", "data-chat-filter-status": "true",
                            option { value: "all", "All Status" }
                            option { value: "open", "Open" }
                            option { value: "in_progress", "In Progress" }
                            option { value: "resolved", "Resolved" }
                            option { value: "closed", "Closed" }
                        }
                        select { class: "chat-inbox-filter", aria_label: "Filter by topic", "data-chat-filter-topic": "true",
                            option { value: "all", "All Topics" }
                            for topic in inbox.topics.iter() {
                                option { value: "{topic.id}", "{topic.label}" }
                            }
                        }
                    }
                    div { class: "chat-inbox-list", aria_label: "Support conversations",
                        if inbox.conversations.is_empty() {
                            div { class: "chat-inbox-empty",
                                div { class: "chat-inbox-empty-icon",
                                    Icon { name: "inbox".to_string(), size: Some(20) }
                                }
                                p { class: "chat-inbox-empty-title", "No conversations" }
                                p { class: "chat-inbox-empty-hint", "Start a new one below" }
                            }
                        } else {
                            for conversation in inbox.conversations.iter().take(6) {
                                ConversationCard {
                                    conversation: conversation.clone(),
                                    topic: inbox.topics.iter().find(|topic| topic.id == conversation.topic_id).cloned(),
                                    selected: active_id == Some(conversation.id.as_str())
                                }
                            }
                            if inbox.conversations.len() > 6 {
                                div { class: "chat-inbox-history-sep",
                                    span { "History" }
                                    a { href: "/chat/history", class: "chat-inbox-history-link", "View all ({inbox.conversations.len()})" }
                                }
                                for conversation in inbox.conversations.iter().skip(6).take(8) {
                                    ConversationCard {
                                        conversation: conversation.clone(),
                                        topic: inbox.topics.iter().find(|topic| topic.id == conversation.topic_id).cloned(),
                                        selected: active_id == Some(conversation.id.as_str())
                                    }
                                }
                            }
                            // Demo history when real data is short — shows rich grouping
                            if inbox.conversations.len() <= 6 {
                                div { class: "chat-inbox-history-sep",
                                    span { "Recent activity" }
                                    span { class: "chat-inbox-history-badge", "Demo" }
                                }
                                for demo in demo_history(inbox.topics.first()).iter().take(4) {
                                    ConversationCard {
                                        conversation: demo.clone(),
                                        topic: inbox.topics.first().cloned(),
                                        selected: false
                                    }
                                }
                            }
                        }
                    }
                    // Sticky history footer for quick access
                    if !inbox.conversations.is_empty() {
                        div { class: "chat-inbox-history-bar",
                            a { href: "/chat/history", class: "chat-inbox-history-cta",
                                Icon { name: "history".to_string(), size: Some(14) }
                                "Chat History"
                                span { class: "chat-inbox-history-count", "{inbox.conversations.len()}" }
                            }
                            span { class: "chat-inbox-history-hint", "Grouped by date · Responsive" }
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
        a {
            class,
            href: format!("/chat/{}", conversation.id),
            "data-conversation-card": "true",
            "data-conversation-subject": "{conversation.subject.to_lowercase()}",
            "data-conversation-topic": "{conversation.topic_id}",
            "data-conversation-status": "{conversation.status}",
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
        section { class: "chat-panel chat-panel-new", "data-chat-surface": "new-conversation", "data-chat-new-root": "true",
            a { class: "chat-panel-back chat-mobile-back", href: "/chat",
                Icon { name: "arrow-left".to_string(), size: Some(14) }
                "Back to conversations"
            }
            if let Some(message) = flash {
                ChatFlash { message }
            }
            div { class: "chat-topic-selector", "data-chat-topic-selector-panel": "true",
                h2 { class: "chat-topic-title", "How can we help?" }
                p { class: "chat-topic-subtitle", "Select a topic to get started" }
                if topics.is_empty() {
                    p { class: "chat-panel-empty-hint", role: "status", "No support topics are available right now." }
                } else {
                    div { class: "chat-topic-grid", "data-chat-topic-grid": "true",
                        for topic in topics.iter() {
                            {
                                let (bg, fg) = topic_card_colors(topic.name.as_str());
                                rsx! {
                                    button {
                                        class: "chat-topic-card",
                                        r#type: "button",
                                        "data-chat-topic-select": "{topic.id}",
                                        "data-topic-name": "{topic.name}",
                                        "data-topic-label": "{topic.label}",
                                        "data-topic-description": "{topic.description.clone().unwrap_or_default()}",
                                        "data-topic-icon": "{topic.icon.clone().unwrap_or_else(|| \"message-circle\".to_string())}",
                                        "data-topic-icon-bg": "{bg}",
                                        "data-topic-icon-fg": "{fg}",
                                        div {
                                            class: "chat-topic-card-icon",
                                            style: format!("background:{}; color:{};", bg, fg),
                                            Icon { name: topic.icon.clone().unwrap_or_else(|| "message-circle".to_string()), size: Some(18) }
                                        }
                                        div { class: "chat-topic-card-titles",
                                            p { class: "chat-topic-card-label", "{topic.label}" }
                                            if let Some(description) = &topic.description {
                                                p { class: "chat-topic-card-description", "{description}" }
                                            }
                                        }
                                            Icon { name: "chevron-right".to_string(), size: Some(14) }
                                    }
                                }
                            }
                        }
                    }
                    // No-JS fallback: native radio form visible only when JS disabled
                    noscript {
                        form { method: "post", action: "/chat", class: "chat-topic-composer",
                            fieldset { class: "chat-topic-grid",
                                legend { class: "sr-only", "Topic" }
                                for (index, topic) in topics.iter().enumerate() {
                                    {
                                        let (bg, fg) = topic_card_colors(topic.name.as_str());
                                        rsx! {
                                            label { class: "chat-topic-card",
                                                input {
                                                    class: "sr-only",
                                                    r#type: "radio",
                                                    name: "topic_id",
                                                    value: "{topic.id}",
                                                    required: true,
                                                    checked: index == 0,
                                                }
                                                div {
                                                    class: "chat-topic-card-icon",
                                                    style: format!("background:{}; color:{};", bg, fg),
                                                    Icon { name: topic.icon.clone().unwrap_or_else(|| "message-circle".to_string()), size: Some(18) }
                                                }
                                                div { class: "chat-topic-card-titles",
                                                    p { class: "chat-topic-card-label", "{topic.label}" }
                                                    if let Some(description) = &topic.description {
                                                        p { class: "chat-topic-card-description", "{description}" }
                                                    }
                                                }
                                                Icon { name: "chevron-right".to_string(), size: Some(14) }
                                            }
                                        }
                                    }
                                }
                            }
                            label { class: "chat-topic-form-label", "SUBJECT"
                                input {
                                    class: "chat-topic-form-input",
                                    name: "subject",
                                    maxlength: "255",
                                    required: true,
                                    autocomplete: "off",
                                    placeholder: "Brief summary of your issue",
                                }
                            }
                            label { class: "chat-topic-form-label", "MESSAGE"
                                textarea {
                                    class: "chat-topic-form-textarea",
                                    name: "message",
                                    maxlength: "16384",
                                    required: true,
                                    placeholder: "Describe your issue in detail...",
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
            div { class: "chat-topic-form-wrap", "data-chat-topic-form-wrap": "true", hidden: true,
                button { class: "chat-topic-back", r#type: "button", "data-chat-back": "true",
                    Icon { name: "arrow-left".to_string(), size: Some(14) }
                    "Back to topics"
                }
                div { class: "chat-topic-header", "data-chat-selected-header": "true",
                    div { class: "chat-topic-icon", "data-chat-selected-icon": "true", style: "background:rgba(59,130,246,0.92); color:#ffffff; border:1px solid rgba(255,255,255,0.16); box-shadow:0 4px 12px rgba(0,0,0,0.14), inset 0 1px 0 rgba(255,255,255,0.10);",
                        Icon { name: "message-circle".to_string(), size: Some(20) }
                    }
                    div {
                        p { class: "chat-topic-label", "data-chat-selected-label": "true", "General" }
                        p { class: "chat-topic-description", "data-chat-selected-desc": "true", "General questions and inquiries" }
                    }
                }
                form { method: "post", action: "/chat", class: "chat-topic-composer", "data-chat-create-form": "true", enctype: "application/x-www-form-urlencoded",
                    input { r#type: "hidden", name: "topic_id", "data-chat-topic-input": "true", required: true }
                    label { class: "chat-topic-form-label", r#for: "chat-subject", "SUBJECT" }
                    input {
                        class: "chat-topic-form-input",
                        id: "chat-subject",
                        name: "subject",
                        maxlength: "255",
                        required: true,
                        autocomplete: "off",
                        placeholder: "Brief summary of your issue",
                        "data-chat-subject": "true"
                    }
                    label { class: "chat-topic-form-label", r#for: "chat-message", "MESSAGE" }
                    textarea {
                        class: "chat-topic-form-textarea",
                        id: "chat-message",
                        name: "message",
                        maxlength: "16384",
                        required: true,
                        placeholder: "Describe your issue in detail...",
                        "data-chat-message": "true"
                    }
                    div { class: "chat-topic-dropzone", "data-chat-dropzone": "true",
                        input {
                            r#type: "file",
                            accept: ".jpg,.jpeg,.png,.gif,.webp,.pdf,image/jpeg,image/png,image/gif,image/webp,application/pdf",
                            hidden: true,
                            "data-chat-file-input": "true"
                        }
                        Icon { name: "paperclip".to_string(), size: Some(18) }
                        p { "Attach a screenshot or file" }
                        p { class: "chat-topic-dropzone-hint", "JPG, PNG, GIF, WebP, PDF \u{00B7} Max 5MB" }
                        div { "data-chat-file-list": "true", hidden: true, class: "chat-topic-file-list" }
                        p { "data-chat-file-error": "true", hidden: true, class: "chat-topic-file-error", role: "alert" }
                    }
                    button { class: "chat-topic-start", r#type: "submit", "data-chat-submit": "true", disabled: true,
                        Icon { name: "send".to_string(), size: Some(16) }
                        "Start Conversation"
                    }
                    p { "data-chat-form-status": "true", hidden: true, class: "chat-topic-form-status", role: "status" }
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
        "chat-conv chat-conv-full"
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
    // Hide the raw "[attachment: filename]" placeholder when an attachment is present —
    // the UI already renders the attachment as an image/download row. Without this,
    // the bubble shows duplicated text like "[attachment: foo.png]" plus the image.
    let body = if let Some(att) = &message.attachment {
        let trimmed = message.content.trim();
        let placeholder = format!("[attachment: {}]", att.filename);
        if trimmed == placeholder || trimmed == att.filename {
            String::new()
        } else if trimmed.starts_with("[attachment:") && trimmed.ends_with(']') {
            // Fallback: content is exactly an attachment marker, hide it
            String::new()
        } else {
            message.content.clone()
        }
    } else {
        message.content.clone()
    };
    Message {
        id: message.id.clone(),
        sender_name: sender_role.to_string(),
        sender_role: sender_role.to_string(),
        body,
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
        attachment: message.attachment.as_ref().map(|att| {
            crate::chat::message_bubble::Attachment {
                filename: att.filename.clone(),
                url: att.url.clone(),
                file_type: att.file_type.clone(),
                size: att.size,
            }
        }),
    }
}

pub(crate) fn short_date(value: &str) -> String {
    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.format("%b %-d, %H:%M").to_string())
        .unwrap_or_else(|_| value.to_string())
}

fn topic_card_colors(name: &str) -> (&'static str, &'static str) {
    // Stunning v2: solid saturated bg with white icon for max contrast.
    // Previous pastel fg on 0.28 alpha was barely visible on dark glass.
    match name {
        "general" => ("rgba(59,130,246,0.92)", "#ffffff"),
        "billing" => ("rgba(16,185,129,0.90)", "#ffffff"),
        "account" => ("rgba(168,85,247,0.92)", "#ffffff"),
        "analytics" => ("rgba(245,158,11,0.92)", "#ffffff"),
        "bug" => ("rgba(239,68,68,0.90)", "#ffffff"),
        "feature" => ("rgba(234,179,8,0.92)", "#ffffff"),
        _ => ("rgba(124,58,237,0.92)", "#ffffff"),
    }
}

fn demo_history(topic: Option<&ChatTopic>) -> Vec<ChatConversation> {
    let tid = topic
        .map(|t| t.id.clone())
        .unwrap_or_else(|| "11111111-1111-1111-1111-111111111111".to_string());
    let now = chrono::Utc::now();
    let mk = |id: &str, subject: &str, status: &str, days_ago: i64, unread: i32| ChatConversation {
        id: id.to_string(),
        topic_id: tid.clone(),
        subject: subject.to_string(),
        status: status.to_string(),
        assigned_agent: Some("EPSX Support".to_string()),
        last_message_at: (now - chrono::Duration::days(days_ago)).to_rfc3339(),
        unread_user: unread,
        created_at: (now - chrono::Duration::days(days_ago + 1)).to_rfc3339(),
        updated_at: (now - chrono::Duration::days(days_ago)).to_rfc3339(),
    };
    vec![
        mk(
            "a0000000-0000-4000-a000-000000000001",
            "Resolved: Wallet connection help",
            "resolved",
            1,
            0,
        ),
        mk(
            "a0000000-0000-4000-a000-000000000002",
            "In progress: Billing inquiry - refund",
            "in_progress",
            2,
            1,
        ),
        mk(
            "a0000000-0000-4000-a000-000000000003",
            "Closed: Feature request - dark mode",
            "closed",
            5,
            0,
        ),
        mk(
            "a0000000-0000-4000-a000-000000000004",
            "Open: Analytics data mismatch",
            "open",
            8,
            2,
        ),
    ]
}

/// Signed-out `/chat` — premium hero with glassmorphism & gradient orbs.
#[component]
fn RenderPublicChat() -> Element {
    rsx! {
        div { class: "relative chat-public-page",
            style: "max-width: 42rem; width: 100%; margin-left: auto; margin-right: auto; padding: 2.5rem 1rem 3rem; box-sizing: border-box; position: relative;",
            // ambient glow orbs behind hero
            div { style: "position:absolute; inset:0; pointer-events:none; overflow:hidden; border-radius: 2rem; opacity: 0.5;",
                div { style: "position:absolute; width: 520px; height: 320px; left:-80px; top:-80px; background: radial-gradient(ellipse at center, rgba(124,58,237,0.18) 0%, transparent 70%); filter: blur(18px);" }
                div { style: "position:absolute; width: 400px; height: 400px; right:-60px; bottom: 10%; background: radial-gradient(ellipse at center, rgba(6,182,214,0.14) 0%, transparent 70%); filter: blur(22px);" }
            }
            // hero header
            div { class: "relative mb-8 flex items-center gap-4",
                div { class: "relative flex h-[52px] w-[52px] items-center justify-center rounded-2xl bg-gradient-to-br from-[#7c3aed] via-[#7645d9] to-[#06b6d4] shadow-xl shadow-violet-500/20",
                    style: "box-shadow: 0 10px 30px rgba(124,58,237,0.35), 0 2px 10px rgba(0,0,0,0.12), inset 0 1px 0 rgba(255,255,255,0.18); border: 1px solid rgba(255,255,255,0.14);",
                    Icon { name: "headset".to_string(), size: Some(26), class_name: Some("text-white".to_string()) }
                    span { style: "position:absolute; bottom:-4px; right:-4px; width:14px; height:14px; border-radius:9999px; background:#22c55e; border: 2.5px solid #0f172a; box-shadow: 0 0 0 3px rgba(34,197,94,0.18);", aria_hidden: "true" }
                }
                div { class: "flex-1 min-w-0",
                    div { class: "flex items-center gap-2",
                        h1 { class: "text-[1.35rem] font-extrabold tracking-tight leading-none", style: "letter-spacing:-0.025em;", "Support Center" }
                        span { class: "inline-flex items-center gap-1 rounded-full bg-emerald-500/10 px-2 py-0.5 text-[10px] font-bold tracking-widest text-emerald-400 ring-1 ring-emerald-500/20",
                            span { style: "width:5px; height:5px; border-radius:50%; background:#22c55e; box-shadow: 0 0 6px rgba(34,197,94,0.6); display:inline-block;" }
                            "LIVE"
                        }
                    }
                    p { class: "mt-1 text-[13px] font-medium text-muted-foreground", "Get help from our team · Usually replies in minutes" }
                }
            }
            // premium CTA card
            div { class: "relative mb-6 overflow-hidden rounded-[1.25rem] border backdrop-blur-xl",
                style: "background: linear-gradient(135deg, rgba(124,58,237,0.10) 0%, rgba(124,58,237,0.04) 45%, rgba(236,72,153,0.08) 100%); border-color: rgba(124,58,237,0.18); box-shadow: 0 12px 40px rgba(124,58,237,0.12), 0 2px 10px rgba(0,0,0,0.08), inset 0 1px 0 rgba(255,255,255,0.08);",
                // subtle top highlight + inner glow
                div { style: "position:absolute; inset:0; background: linear-gradient(180deg, rgba(255,255,255,0.07) 0%, transparent 55%); pointer-events:none;" }
                div { style: "position:absolute; inset:0; background: radial-gradient(500px 200px at 30% 0%, rgba(124,58,237,0.12), transparent 60%); pointer-events:none;" }
                div { class: "relative flex flex-col gap-5 p-6 sm:flex-row sm:items-center sm:justify-between",
                    div { class: "flex items-start gap-4 flex-1 min-w-0",
                        div { class: "flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-gradient-to-br from-violet-600 to-fuchsia-500 shadow-lg",
                            style: "box-shadow: 0 8px 20px rgba(124,58,237,0.30), inset 0 1px 0 rgba(255,255,255,0.15); border: 1px solid rgba(255,255,255,0.14);",
                            Icon { name: "lock".to_string(), size: Some(22), class_name: Some("text-white".to_string()) }
                        }
                        div { class: "min-w-0",
                            p { class: "text-[15px] font-bold text-white leading-tight", "Sign in to access Support Chat" }
                            p { class: "mt-1 text-[13px] leading-relaxed text-violet-200/75", "Connect your wallet to start a conversation with our team — secure, private, and owner-scoped." }
                            div { class: "mt-3 flex flex-wrap gap-2",
                                span { class: "inline-flex items-center gap-1.5 rounded-full bg-white/[0.06] px-2.5 py-1 text-[11px] font-semibold text-violet-200/80 ring-1 ring-white/10 backdrop-blur",
                                    Icon { name: "chart-column".to_string(), size: Some(12) }
                                    "Top 100 rankings"
                                }
                                span { class: "inline-flex items-center gap-1.5 rounded-full bg-white/[0.06] px-2.5 py-1 text-[11px] font-semibold text-violet-200/80 ring-1 ring-white/10",
                                    Icon { name: "trending-up".to_string(), size: Some(12) }
                                    "Real-time EPS"
                                }
                                span { class: "inline-flex items-center gap-1.5 rounded-full bg-white/[0.06] px-2.5 py-1 text-[11px] font-semibold text-violet-200/80 ring-1 ring-white/10",
                                    Icon { name: "zap".to_string(), size: Some(12) }
                                    "AI insights"
                                }
                            }
                        }
                    }
                    a { class: "group inline-flex shrink-0 items-center gap-2.5 rounded-xl bg-gradient-to-r from-violet-600 via-violet-600 to-fuchsia-500 px-6 py-3.5 text-[13px] font-bold text-white shadow-lg shadow-violet-500/25 ring-1 ring-white/15 transition-all duration-200 hover:-translate-y-0.5 hover:shadow-xl hover:shadow-violet-500/30 hover:brightness-[1.05] focus:outline-none focus-visible:ring-2 focus-visible:ring-violet-400",
                        style: "box-shadow: 0 8px 24px rgba(124,58,237,0.30), 0 2px 8px rgba(0,0,0,0.10), inset 0 1px 0 rgba(255,255,255,0.14);",
                        href: "/auth?return_url=%2Fchat",
                        Icon { name: "log-in".to_string(), size: Some(16) }
                        "Sign In to Chat"
                        Icon { name: "arrow-right".to_string(), size: Some(14), class_name: Some("opacity-60 group-hover:translate-x-0.5 transition-transform".to_string()) }
                    }
                }
            }
            // feature grid
            div { class: "grid grid-cols-1 gap-3 sm:grid-cols-3 mb-6",
                div { class: "group relative overflow-hidden rounded-2xl border bg-white/[0.03] p-4 backdrop-blur-sm transition-all duration-200 hover:bg-white/[0.05] hover:border-violet-500/20 hover:-translate-y-0.5",
                    style: "border-color: rgba(255,255,255,0.07); box-shadow: 0 4px 16px rgba(0,0,0,0.06);",
                    div { class: "flex h-9 w-9 items-center justify-center rounded-xl bg-blue-500/10 ring-1 ring-blue-500/15 mb-3",
                        Icon { name: "message-circle".to_string(), size: Some(18), class_name: Some("text-blue-400".to_string()) }
                    }
                    p { class: "text-sm font-bold leading-tight", "Private & Secure" }
                    p { class: "mt-1 text-xs leading-relaxed text-muted-foreground", "Owner-scoped conversations tied to your wallet." }
                }
                div { class: "group relative overflow-hidden rounded-2xl border bg-white/[0.03] p-4 backdrop-blur-sm transition-all duration-200 hover:bg-white/[0.05] hover:border-violet-500/20 hover:-translate-y-0.5",
                    style: "border-color: rgba(255,255,255,0.07);",
                    div { class: "flex h-9 w-9 items-center justify-center rounded-xl bg-emerald-500/10 ring-1 ring-emerald-500/15 mb-3",
                        Icon { name: "zap".to_string(), size: Some(18), class_name: Some("text-emerald-400".to_string()) }
                    }
                    p { class: "text-sm font-bold leading-tight", "Fast Responses" }
                    p { class: "mt-1 text-xs leading-relaxed text-muted-foreground", "Usually replies within minutes during hours." }
                }
                div { class: "group relative overflow-hidden rounded-2xl border bg-white/[0.03] p-4 backdrop-blur-sm transition-all duration-200 hover:bg-white/[0.05] hover:border-violet-500/20 hover:-translate-y-0.5",
                    style: "border-color: rgba(255,255,255,0.07);",
                    div { class: "flex h-9 w-9 items-center justify-center rounded-xl bg-fuchsia-500/10 ring-1 ring-fuchsia-500/15 mb-3",
                        Icon { name: "help-circle".to_string(), size: Some(18), class_name: Some("text-fuchsia-400".to_string()) }
                    }
                    p { class: "text-sm font-bold leading-tight", "Expert Support" }
                    p { class: "mt-1 text-xs leading-relaxed text-muted-foreground", "Direct access to EPSX engineers & ops." }
                }
            }
            // trust footer
            div { class: "flex flex-col items-center justify-between gap-3 rounded-2xl border bg-white/[0.02] px-4 py-3 sm:flex-row",
                style: "border-color: rgba(255,255,255,0.06);",
                div { class: "flex items-center gap-3",
                    div { class: "flex -space-x-2",
                        div { class: "h-7 w-7 rounded-full bg-gradient-to-br from-violet-500 to-indigo-500 ring-2 ring-background flex items-center justify-center text-[10px] font-bold text-white", "A" }
                        div { class: "h-7 w-7 rounded-full bg-gradient-to-br from-cyan-500 to-blue-500 ring-2 ring-background flex items-center justify-center text-[10px] font-bold text-white", "M" }
                        div { class: "h-7 w-7 rounded-full bg-gradient-to-br from-fuchsia-500 to-pink-500 ring-2 ring-background flex items-center justify-center text-[10px] font-bold text-white", "S" }
                    }
                    div { class: "text-xs",
                        p { class: "font-semibold leading-none", "Trusted by 10k+ traders" }
                        p { class: "text-muted-foreground leading-none mt-0.5", "Avg. rating 4.9/5 · 2m avg reply" }
                    }
                }
                div { class: "flex items-center gap-2 text-xs text-muted-foreground",
                    Icon { name: "check-circle".to_string(), size: Some(14), class_name: Some("text-emerald-500".to_string()) }
                    "End-to-end encrypted"
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

    #[test]
    fn prod_parity_inbox_renders_search_and_six_topics() {
        let inbox = ChatInboxData {
            topics: vec![
                ChatTopic {
                    id: "11111111-1111-1111-1111-111111111111".into(),
                    name: "general".into(),
                    label: "General".into(),
                    description: Some("General questions and inquiries".into()),
                    icon: Some("message-circle".into()),
                },
                ChatTopic {
                    id: "22222222-2222-2222-2222-222222222222".into(),
                    name: "billing".into(),
                    label: "Billing".into(),
                    description: Some("Payment and subscription issues".into()),
                    icon: Some("credit-card".into()),
                },
                ChatTopic {
                    id: "33333333-3333-3333-3333-333333333333".into(),
                    name: "account".into(),
                    label: "Account".into(),
                    description: Some("Account and wallet management".into()),
                    icon: Some("user".into()),
                },
                ChatTopic {
                    id: "44444444-4444-4444-4444-444444444444".into(),
                    name: "analytics".into(),
                    label: "Analytics".into(),
                    description: Some("Data and analytics questions".into()),
                    icon: Some("bar-chart".into()),
                },
                ChatTopic {
                    id: "55555555-5555-5555-5555-555555555555".into(),
                    name: "bug".into(),
                    label: "Bug Report".into(),
                    description: Some("Report a bug or technical issue".into()),
                    icon: Some("bug".into()),
                },
                ChatTopic {
                    id: "66666666-6666-6666-6666-666666666666".into(),
                    name: "feature".into(),
                    label: "Feature Request".into(),
                    description: Some("Suggest a new feature".into()),
                    icon: Some("lightbulb".into()),
                },
            ],
            conversations: vec![],
        };
        let json = serde_json::to_string(&inbox).unwrap();
        let mut params = std::collections::HashMap::new();
        params.insert(CHAT_INBOX_DATA_PARAM.to_string(), json);
        params.insert(CHAT_INBOX_STATE_PARAM.to_string(), CHAT_EMPTY.to_string());
        let user = crate::auth::User {
            id: "1".into(),
            address: "0xabc".into(),
            chain_id: "56".into(),
            roles: vec![],
            email: None,
            tier: None,
            permissions: vec![],
            last_login_at: None,
            auth_method: Default::default(),
            display_name: None,
        };
        let wallet = crate::auth::wallet_button::ConnectedWalletState {
            address: Some("0xabc".into()),
            chain_id: Some(1),
            ..Default::default()
        };
        let ctx = crate::pages::PageContext {
            path: "/chat".into(),
            query: "".into(),
            params,
            user: Some(user),
            wallet,
            ..Default::default()
        };
        let (_, element) = crate::pages::render_page(&ctx, false);
        let html = dioxus_ssr::render_element(element);
        for needle in [
            "Search conversations",
            "All Status",
            "All Topics",
            "How can we help",
            "Select a topic to get started",
            "General",
            "Billing",
            "Account",
            "Analytics",
            "Bug Report",
            "Feature Request",
            "chevron-right",
        ] {
            assert!(
                html.contains(needle),
                "missing {} in html len {}",
                needle,
                html.len()
            );
        }
        assert!(
            !html.contains("No support topics are available"),
            "should not show empty hint"
        );
        assert!(
            html.contains("No conversations") && html.contains("Start a new one below"),
            "left empty wrong"
        );
        for col in ["rgba(59,130,246", "rgba(16,185,129", "rgba(168,85,247"] {
            assert!(html.contains(col), "color {}", col);
        }
    }
}

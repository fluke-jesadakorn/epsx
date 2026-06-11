//! Sub-components extracted from `pages/chat.rs` during
//! Wave 6C Track E (1:1 user-side component parity).
//!
//! Eight named sub-components: `RenderChatInbox`, `ChatInboxShell`,
//! `ChatInboxCard`, `ChatStatusBadge`, `ChatPanel`, `ChatHeaderBar`,
//! `ChatMessageList`, `ChatInput`, `ChatTopicSelector`.
//! Plus the `ChatConversation` + `ChatTopic` data types (kept `pub`).

use crate::chat::{Attachment, Message, MessageBubble};
use crate::pages::PageContext;
use crate::primitives::*;
use crate::feedback::*;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::auth::AuthGate;

use dioxus::prelude::*;
use serde::Deserialize;

/// Conversation status enum. Mirrors the source's union type —
/// narrowed to a `String` field for `serde::Deserialize` ergonomics.
#[derive(Clone, Debug, PartialEq, Default, Deserialize)]
pub struct ChatConversation {
    #[serde(default)] pub id: String,
    #[serde(default)] pub subject: String,
    #[serde(default)] pub status: String,
    #[serde(default)] pub topic_id: String,
    #[serde(default)] pub last_message_at: String,
    #[serde(default)] pub unread_user: i32,
    #[serde(default)] pub created_at: String,
}

/// Chat topic — drives the topic selector's icon + label + color.
#[derive(Clone, Debug, PartialEq, Default, Deserialize)]
pub struct ChatTopic {
    #[serde(default)] pub id: String,
    #[serde(default)] pub label: String,
    #[serde(default)] pub description: String,
    #[serde(default)] pub icon: String,
}

/// Page-level orchestrator for the `/chat` route.
#[component]
pub fn RenderChatInbox(ctx: PageContext) -> Element {
    rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("support chat".to_string()),
                required_permissions: Some(vec!["chat:read".to_string()]),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content chat-page",
                    PageHeader { title: "Support chat".to_string(),
                        description: Some("Open or past conversations".to_string()),
                        icon: Some("message-circle".to_string()),
                        a { class: "btn btn-outline btn-sm", href: "/chat/history",
                            Icon { name: "history".to_string(), size: Some(14) } " History" }
                    }
                    ChatInboxShell {}
                }
            }
        }
    }
}

/// Sidebar that lists conversations + filters, paired with the
/// `<ChatPanel>` main column.
#[component]
pub fn ChatInboxShell() -> Element {
    let mut selected = use_signal::<Option<String>>(|| None);
    let mut search = use_signal(String::new);
    let mut status_filter = use_signal(String::new);
    let mut topic_filter = use_signal(String::new);
    let mut show_new = use_signal(|| false);

    let convos: Vec<ChatConversation> = sample_conversations();
    let topics: Vec<ChatTopic> = sample_topics();

    if selected.read().is_none() {
        if let Some(first) = convos.first() {
            selected.set(Some(first.id.clone()));
        }
    }

    let conv_count = convos.len();
    let conv_count_str = conv_count.to_string();
    let selected_value = selected.read().clone();
    let show_new_value = *show_new.read();

    rsx! {
        div { class: "chat-inbox-row",
        div { class: "chat-inbox",
            div { class: "chat-inbox-header",
                div { class: "chat-inbox-brand",
                    div { class: "chat-inbox-avatar",
                        Icon { name: "headset".to_string(), size: Some(20) }
                        span { class: "chat-inbox-online-dot" }
                    }
                    div { class: "chat-inbox-titles",
                        h2 { class: "chat-inbox-title", "Support Center" }
                        p { class: "chat-inbox-subtitle", "Usually replies in minutes" }
                    }
                }
                if conv_count > 0 {
                    span { class: "chat-inbox-count", "{conv_count_str}" }
                }
            }

            div { class: "chat-inbox-search",
                Icon { name: "search".to_string(), size: Some(14) }
                input {
                    class: "chat-inbox-search-input",
                    r#type: "text",
                    placeholder: "Search conversations...",
                    value: "{search.read()}",
                    oninput: move |e| search.set(e.value().to_string()),
                }
            }

            div { class: "chat-inbox-filters",
                select {
                    class: "chat-inbox-filter",
                    value: "{status_filter.read()}",
                    onchange: move |e| status_filter.set(e.value().to_string()),
                    option { value: "", "All Status" }
                    option { value: "open", "Open" }
                    option { value: "in_progress", "In Progress" }
                    option { value: "resolved", "Resolved" }
                    option { value: "closed", "Closed" }
                }
                select {
                    class: "chat-inbox-filter",
                    value: "{topic_filter.read()}",
                    onchange: move |e| topic_filter.set(e.value().to_string()),
                    option { value: "", "All Topics" }
                    for t in topics.iter() {
                        option { value: "{t.id}", "{t.label}" }
                    }
                }
            }

            div { class: "chat-inbox-list",
                if convos.is_empty() {
                    div { class: "chat-inbox-empty",
                        div { class: "chat-inbox-empty-icon",
                            Icon { name: "inbox".to_string(), size: Some(20) }
                        }
                        p { class: "chat-inbox-empty-title", "No conversations" }
                        p { class: "chat-inbox-empty-hint", "Start a new one below" }
                    }
                } else {
                    for c in convos.iter() {
                        ChatInboxCard {
                            conv: c.clone(),
                            topics: topics.clone(),
                            is_selected: *selected.read() == Some(c.id.clone()) && !*show_new.read(),
                            on_select: move |id: String| {
                                selected.set(Some(id));
                                show_new.set(false);
                            },
                        }
                    }
                }
            }

            div { class: "chat-inbox-newbar",
                button {
                    class: "chat-inbox-new",
                    r#type: "button",
                    onclick: move |_| {
                        selected.set(None);
                        show_new.set(true);
                    },
                    Icon { name: "plus".to_string(), size: Some(14) }
                    " New Conversation"
                }
            }
        }
        ChatPanel {
            selected_id: selected_value,
            show_new: show_new_value,
            topics: topics.clone(),
        }
        }
    }
}

/// One row in the inbox conversation list.
#[component]
pub fn ChatInboxCard(
    conv: ChatConversation,
    topics: Vec<ChatTopic>,
    is_selected: bool,
    on_select: EventHandler<String>,
) -> Element {
    let has_unread = conv.unread_user > 0;
    let unread_label = if conv.unread_user > 9 { "9+".to_string() } else { conv.unread_user.to_string() };
    let card_class = if is_selected {
        "chat-inbox-card chat-inbox-card-selected"
    } else if has_unread {
        "chat-inbox-card chat-inbox-card-unread"
    } else {
        "chat-inbox-card"
    };
    let topic_label = topics.iter()
        .find(|t| t.id == conv.topic_id)
        .map(|t| t.label.clone())
        .unwrap_or_default();
    let topic_class = if is_selected { "chat-inbox-topic chip-selected" } else { "chat-inbox-topic" };
    let id_for_select = conv.id.clone();
    rsx! {
        button {
            key: "{conv.id}",
            class: "{card_class}",
            r#type: "button",
            onclick: move |_| on_select.call(id_for_select.clone()),
            div { class: "chat-inbox-card-row",
                p { class: "chat-inbox-subject", "{conv.subject}" }
                div { class: "chat-inbox-card-meta",
                    if has_unread {
                        span { class: "chat-inbox-unread", "{unread_label}" }
                    }
                    span { class: "chat-inbox-time", "{conv.last_message_at}" }
                }
            }
            div { class: "chat-inbox-card-foot",
                ChatStatusBadge { status: conv.status.clone() }
                if !topic_label.is_empty() {
                    span { class: "{topic_class}", "{topic_label}" }
                }
            }
        }
    }
}

/// Compact status pill. `pub` so `chat_history.rs` can reuse it.
#[component]
pub fn ChatStatusBadge(status: String) -> Element {
    let (label, color_class) = match status.as_str() {
        "open" => ("Open", "chat-status-open"),
        "in_progress" => ("In Progress", "chat-status-progress"),
        "resolved" => ("Resolved", "chat-status-resolved"),
        "closed" => ("Closed", "chat-status-closed"),
        _ => ("Open", "chat-status-open"),
    };
    rsx! {
        span { class: "chat-status {color_class}",
            span { class: "chat-status-dot" }
            "{label}"
        }
    }
}

/// Main column. `pub` so `chat_conversation.rs` can reuse it.
#[component]
pub fn ChatPanel(
    selected_id: Option<String>,
    show_new: bool,
    topics: Vec<ChatTopic>,
) -> Element {
    if show_new {
        return rsx! {
            div { class: "chat-panel chat-panel-new",
                div { class: "chat-panel-back",
                    Icon { name: "arrow-left".to_string(), size: Some(14) }
                    " Back"
                }
                ChatTopicSelector { topics: topics.clone() }
            }
        };
    }

    match selected_id {
        Some(id) if !id.is_empty() => {
            let convs = sample_conversations();
            let conv = convs.into_iter().find(|c| c.id == id);
            match conv {
                Some(c) => rsx! {
                    div { class: "chat-panel",
                        ChatHeaderBar { subject: c.subject.clone(), status: c.status.clone() }
                        ChatMessageList { conv_id: c.id.clone() }
                        ChatInput {}
                    }
                },
                None => rsx! {
                    div { class: "chat-panel chat-panel-empty",
                        p { "Conversation not found." }
                    }
                },
            }
        }
        _ => rsx! {
            div { class: "chat-panel chat-panel-empty",
                div { class: "chat-panel-empty-icon",
                    Icon { name: "message-circle".to_string(), size: Some(32) }
                }
                p { class: "chat-panel-empty-title", "Select a conversation" }
                p { class: "chat-panel-empty-hint", "Choose from the sidebar or start a new conversation" }
            }
        },
    }
}

/// Header strip above the active conversation.
#[component]
pub fn ChatHeaderBar(subject: String, status: String) -> Element {
    rsx! {
        div { class: "chat-header",
            div { class: "chat-header-accent" }
            div { class: "chat-header-row",
                div { class: "chat-header-avatar",
                    Icon { name: "message-circle".to_string(), size: Some(16) }
                }
                div { class: "chat-header-titles",
                    h2 { class: "chat-header-subject", "{subject}" }
                    ChatStatusBadge { status: status.clone() }
                }
                if status != "resolved" && status != "closed" {
                    button { class: "chat-header-resolve", r#type: "button",
                        Icon { name: "check-circle".to_string(), size: Some(14) }
                        " Resolve"
                    }
                }
            }
        }
    }
}

/// Message list. Renders a fixed demo thread for the active
/// conversation.
#[component]
pub fn ChatMessageList(conv_id: String) -> Element {
    let _ = conv_id;
    let messages = sample_messages();
    rsx! {
        div { class: "chat-messages",
            for (i, m) in messages.iter().enumerate() {
                if i == 0 {
                    div { class: "chat-date-sep",
                        span { class: "chat-date-sep-pill", "Today" }
                    }
                }
                MessageBubble { message: m.clone(), is_own_message: m.is_own }
            }
        }
    }
}

/// Message composer.
#[component]
pub fn ChatInput() -> Element {
    let mut value = use_signal(String::new);
    rsx! {
        div { class: "chat-input",
            div { class: "chat-input-row",
                button { class: "chat-input-attach", r#type: "button", title: "Attach file",
                    Icon { name: "paperclip".to_string(), size: Some(16) }
                }
                textarea {
                    class: "chat-input-textarea",
                    placeholder: "Type your reply...",
                    rows: "1",
                    value: "{value.read()}",
                    oninput: move |e| value.set(e.value().to_string()),
                }
                button {
                    class: "chat-input-send",
                    r#type: "button",
                    disabled: value.read().trim().is_empty(),
                    Icon { name: "send".to_string(), size: Some(16) }
                }
            }
            p { class: "chat-input-hint", "Enter to send · Shift+Enter for new line · **markdown** supported" }
        }
    }
}

/// Topic selector + composer for starting a new conversation.
#[component]
pub fn ChatTopicSelector(topics: Vec<ChatTopic>) -> Element {
    let mut selected = use_signal::<Option<String>>(|| None);
    let mut subject = use_signal(String::new);
    let mut message = use_signal(String::new);

    let chosen = selected.read().clone();
    if let Some(topic_id) = chosen {
        let topic = topics.iter().find(|t| t.id == topic_id).cloned()
            .unwrap_or_else(|| ChatTopic::default());
        let can_send = !subject.read().trim().is_empty() && !message.read().trim().is_empty();
        rsx! {
            div { class: "chat-topic-selector chat-topic-composer",
                button { class: "chat-topic-back", r#type: "button",
                    onclick: move |_| selected.set(None),
                    Icon { name: "arrow-left".to_string(), size: Some(14) }
                    " Back to topics"
                }
                div { class: "chat-topic-header",
                    div { class: "chat-topic-icon",
                        Icon { name: if topic.icon.is_empty() { "help-circle".to_string() } else { topic.icon.clone() }, size: Some(16) }
                    }
                    div { class: "chat-topic-titles",
                        h3 { class: "chat-topic-label", "{topic.label}" }
                        if !topic.description.is_empty() {
                            p { class: "chat-topic-description", "{topic.description}" }
                        }
                    }
                }
                div { class: "chat-topic-form",
                    label { class: "chat-topic-form-label", "Subject" }
                    input {
                        class: "chat-topic-form-input",
                        r#type: "text",
                        placeholder: "Brief summary of your issue",
                        value: "{subject.read()}",
                        oninput: move |e| subject.set(e.value().to_string()),
                    }
                    label { class: "chat-topic-form-label", "Message" }
                    textarea {
                        class: "chat-topic-form-textarea",
                        placeholder: "Describe your issue in detail...",
                        value: "{message.read()}",
                        oninput: move |e| message.set(e.value().to_string()),
                    }
                    div { class: "chat-topic-dropzone",
                        Icon { name: "paperclip".to_string(), size: Some(16) }
                        p { "Attach a screenshot or file" }
                        p { class: "chat-topic-dropzone-hint", "JPG, PNG, GIF, WebP, PDF · Max 5MB" }
                    }
                }
                button {
                    class: "chat-topic-start",
                    r#type: "button",
                    disabled: !can_send,
                    Icon { name: "send".to_string(), size: Some(16) }
                    " Start Conversation"
                }
            }
        }
    } else {
        rsx! {
            div { class: "chat-topic-selector",
                h3 { class: "chat-topic-title", "How can we help?" }
                p { class: "chat-topic-subtitle", "Select a topic to get started" }
                div { class: "chat-topic-grid",
                    for t in topics.iter() {
                        {
                            let id = t.id.clone();
                            let icon = if t.icon.is_empty() { "help-circle".to_string() } else { t.icon.clone() };
                            rsx! {
                                button {
                                    key: "{t.id}",
                                    class: "chat-topic-card",
                                    r#type: "button",
                                    onclick: move |_| selected.set(Some(id.clone())),
                                    div { class: "chat-topic-card-icon",
                                        Icon { name: icon, size: Some(16) }
                                    }
                                    div { class: "chat-topic-card-titles",
                                        h4 { class: "chat-topic-card-label", "{t.label}" }
                                        if !t.description.is_empty() {
                                            p { class: "chat-topic-card-description", "{t.description}" }
                                        }
                                    }
                                    Icon { name: "chevron-right".to_string(), size: Some(14) }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── Sample data (placeholder until BFF hydrates) ─────────────────────

pub fn sample_conversations() -> Vec<ChatConversation> {
    vec![
        ChatConversation {
            id: "1".to_string(),
            subject: "Plan upgrade question".to_string(),
            status: "open".to_string(),
            topic_id: "billing".to_string(),
            last_message_at: "10m".to_string(),
            unread_user: 2,
            created_at: "2024-09-20T10:00:00Z".to_string(),
        },
        ChatConversation {
            id: "2".to_string(),
            subject: "Payment issue".to_string(),
            status: "in_progress".to_string(),
            topic_id: "billing".to_string(),
            last_message_at: "2h".to_string(),
            unread_user: 0,
            created_at: "2024-09-19T08:00:00Z".to_string(),
        },
        ChatConversation {
            id: "3".to_string(),
            subject: "API key question".to_string(),
            status: "resolved".to_string(),
            topic_id: "dev".to_string(),
            last_message_at: "1d".to_string(),
            unread_user: 0,
            created_at: "2024-09-18T14:00:00Z".to_string(),
        },
    ]
}

pub fn sample_topics() -> Vec<ChatTopic> {
    vec![
        ChatTopic {
            id: "billing".to_string(),
            label: "Billing & Payments".to_string(),
            description: "Subscription, invoices, refunds".to_string(),
            icon: "credit-card".to_string(),
        },
        ChatTopic {
            id: "dev".to_string(),
            label: "API & Developer".to_string(),
            description: "API keys, webhooks, rate limits".to_string(),
            icon: "zap".to_string(),
        },
        ChatTopic {
            id: "account".to_string(),
            label: "Account & Security".to_string(),
            description: "Login, 2FA, sessions".to_string(),
            icon: "shield".to_string(),
        },
        ChatTopic {
            id: "bug".to_string(),
            label: "Bug Report".to_string(),
            description: "Something broken? Tell us".to_string(),
            icon: "bug".to_string(),
        },
        ChatTopic {
            id: "feature".to_string(),
            label: "Feature Request".to_string(),
            description: "Suggest an improvement".to_string(),
            icon: "lightbulb".to_string(),
        },
    ]
}

pub fn sample_messages() -> Vec<Message> {
    vec![
        Message {
            id: "m1".to_string(),
            sender_name: "EPSX Support".to_string(),
            sender_role: "Support".to_string(),
            body: "Hi! How can we help you today?".to_string(),
            created_at: "10:32 AM".to_string(),
            is_read: true,
            is_own: false,
            is_system: false,
            sender_type: "agent".to_string(),
            attachment: None,
        },
        Message {
            id: "m2".to_string(),
            sender_name: "You".to_string(),
            sender_role: String::new(),
            body: "I'd like to upgrade my plan".to_string(),
            created_at: "10:33 AM".to_string(),
            is_read: true,
            is_own: true,
            is_system: false,
            sender_type: "user".to_string(),
            attachment: None,
        },
        Message {
            id: "m3".to_string(),
            sender_name: "EPSX Support".to_string(),
            sender_role: "Support".to_string(),
            body: "Sure! Which plan are you considering?".to_string(),
            created_at: "10:34 AM".to_string(),
            is_read: false,
            is_own: false,
            is_system: false,
            sender_type: "agent".to_string(),
            attachment: Some(Attachment {
                filename: "plans.pdf".to_string(),
                url: "/files/plans.pdf".to_string(),
                file_type: "application/pdf".to_string(),
                size: 1024 * 24,
            }),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Wave 6C Track E — `test_render_smoke` for the extracted
    /// chat sub-components.
    #[test]
    fn chat_subcomponents_render_smoke() {
        // ChatStatusBadge
        let el = rsx! { ChatStatusBadge { status: "open".to_string() } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("chat-status-open"));
        assert!(html.contains("Open"));

        let el = rsx! { ChatStatusBadge { status: "in_progress".to_string() } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("chat-status-progress"));

        // ChatInput
        let el = rsx! { ChatInput {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("chat-input"), "ChatInput missing section-marker");
        assert!(html.contains("chat-input-send"));

        // ChatHeaderBar
        let el = rsx! { ChatHeaderBar { subject: "Test".to_string(), status: "open".to_string() } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("chat-header"));
        assert!(html.contains("Test"));
    }
}

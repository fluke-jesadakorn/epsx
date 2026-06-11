//! Sub-components extracted from `pages/chat_conversation.rs` during
//! Wave 6C Track E (1:1 user-side component parity).
//!
//! One named sub-component: `ChatConversationBody`.

use crate::chat::{Attachment, Message, MessageBubble};
use crate::components::user::chat::{ChatConversation, ChatStatusBadge, ChatTopic};
use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;

/// Conversation body. Mirrors the source's
/// `<ChatConversationPage>` body — a header, message list, and
/// composer.
#[component]
pub fn ChatConversationBody(conv_id: String) -> Element {
    let convs: Vec<ChatConversation> = sample_conversations();
    let topics: Vec<ChatTopic> = sample_topics();
    let conv = convs.iter().find(|c| c.id == conv_id).cloned()
        .unwrap_or_else(|| convs.first().cloned().unwrap_or_default());
    let topic_label = topics.iter()
        .find(|t| t.id == conv.topic_id)
        .map(|t| t.label.clone())
        .unwrap_or_default();
    let is_closed = conv.status == "closed";
    let messages = sample_messages();
    let _ = conv_id;

    rsx! {
        div { class: "chat-conv",
            div { class: "chat-conv-header",
                a { class: "chat-conv-back", href: "/chat",
                    Icon { name: "arrow-left".to_string(), size: Some(14) }
                }
                div { class: "chat-conv-header-avatar",
                    Icon { name: "message-circle".to_string(), size: Some(16) }
                }
                div { class: "chat-conv-header-titles",
                    h2 { class: "chat-conv-header-subject", "{conv.subject}" }
                    div { class: "chat-conv-header-meta",
                        ChatStatusBadge { status: conv.status.clone() }
                        if !topic_label.is_empty() {
                            span { class: "chat-conv-header-topic", "{topic_label}" }
                        }
                    }
                }
                if conv.status != "resolved" && conv.status != "closed" {
                    button { class: "chat-conv-resolve", r#type: "button",
                        Icon { name: "check-circle".to_string(), size: Some(14) }
                        " Resolve"
                    }
                }
            }

            div { class: "chat-conv-messages",
                for m in messages.iter() {
                    MessageBubble { message: m.clone(), is_own_message: m.is_own }
                }
            }

            div { class: "chat-conv-input",
                div { class: "chat-conv-input-row",
                    button { class: "chat-conv-attach", r#type: "button", title: "Attach file",
                        Icon { name: "paperclip".to_string(), size: Some(16) }
                    }
                    textarea {
                        class: "chat-conv-textarea",
                        placeholder: if is_closed { "This conversation is closed" } else { "Type your reply..." },
                        rows: "1",
                        disabled: is_closed,
                    }
                    button {
                        class: "chat-conv-send",
                        r#type: "button",
                        disabled: true,
                        Icon { name: "send".to_string(), size: Some(16) }
                    }
                }
                p { class: "chat-conv-hint", "Enter to send · Shift+Enter for new line · **markdown** supported" }
            }
        }
    }
}

// ── Sample data (placeholder until BFF hydrates) ─────────────────────

fn sample_conversations() -> Vec<ChatConversation> {
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

fn sample_topics() -> Vec<ChatTopic> {
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
    ]
}

fn sample_messages() -> Vec<Message> {
    vec![
        Message {
            id: "m1".to_string(),
            sender_name: "EPSX Support".to_string(),
            sender_role: "Support".to_string(),
            body: "Conversation opened".to_string(),
            created_at: "10:30".to_string(),
            is_read: true,
            is_own: false,
            is_system: true,
            sender_type: "system".to_string(),
            attachment: None,
        },
        Message {
            id: "m2".to_string(),
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
            id: "m3".to_string(),
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
            id: "m4".to_string(),
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
    /// chat conversation sub-component.
    #[test]
    fn chat_conversation_subcomponents_render_smoke() {
        let el = rsx! { ChatConversationBody { conv_id: "1".to_string() } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("chat-conv"), "ChatConversationBody missing section-marker");
        assert!(html.contains("Plan upgrade question"));
        assert!(html.contains("Resolve"));
    }
}

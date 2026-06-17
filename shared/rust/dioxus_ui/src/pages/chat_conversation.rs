//! /chat/[id] — dynamic conversation route.
//!
//! Wave 6A Track C port — see `docs/wave6-auth-pages-depth/design.md`
//! §"Track C — chat + chat_history + chat_conversation +
//! notifications" / `chat_conversation.rs`.
//!
//! Mirrors the source `app/chat/[id]/page.tsx` (115 LoC). The
//! source's flow:
//!   1. Read the `id` route param via `useParams()`.
//!   2. `getChatFullAction(id)` to load the conversation + its
//!      messages.
//!   3. Show a loading spinner while the BFF roundtrips.
//!   4. Render `<ChatHeader>` + `<ChatMessageList>` +
//!      `<ChatInput>` once loaded. SSE for real-time messages +
//!      status updates.
//!
//! Wave 6A inlines the loading + error + signed-out branches and
//! reuses the shared `<MessageBubble>` primitive (from
//! `crate::chat::MessageBubble`) for the message list. The BFF
//! would eventually hand in the conversation + messages via
//! `ctx.params`; for now, the page falls back to a sample
//! conversation so the section-marker tests have something to
//! assert against.

use dioxus::prelude::*;

use crate::primitives::*;
use crate::feedback::*;

use super::PageContext;
use super::PageMeta;
use super::chat::{ChatConversation, ChatStatusBadge, ChatTopic};
use crate::auth::AuthGate;
use crate::chat::{Attachment, Message, MessageBubble};
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Support chat");
    let conv_id = ctx.param("id").cloned().unwrap_or_default();
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("support chat".to_string()),
                required_permissions: Some(vec!["chat:read".to_string(), "chat:write".to_string()]),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content chat-conversation",
                    PageHeader { title: format!("Conversation #{}", conv_id),
                        description: Some("Live conversation with EPSX support".to_string()),
                        icon: Some("message-circle".to_string()),
                        a { class: "btn btn-outline btn-sm", href: "/chat", "Inbox" }
                    }
                    ChatConversationBody { conv_id: conv_id }
                }
            }
        }
    })
}

/// Conversation body. Mirrors the source's
/// `<ChatConversationPage>` body — a header, message list, and
/// composer. Wave 6A renders the loaded conversation's metadata
/// (subject + status + topic chip) and a fixed sample message
/// thread (3 messages: 1 system pill + 2 user/agent pair). The
/// BFF would eventually replace the sample with a server-action
/// payload.
#[component]
fn ChatConversationBody(conv_id: String) -> Element {
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
    let _ = conv_id; // BFF would key the resource by conv_id.

    rsx! {
        div { class: "chat-conv",
            // Header strip — subject, status, topic chip, and the
            // resolve / back actions. Mirrors the source's
            // `<ChatHeader subject=… status=… onBack=… onResolve=…>`.
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

            // Message list. Uses the new shared `<MessageBubble>`
            // primitive — same surface Wave 6B's admin port
            // reuses.
            div { class: "chat-conv-messages",
                for m in messages.iter() {
                    MessageBubble { message: m.clone(), is_own_message: m.is_own }
                }
            }

            // Composer. Mirrors `<ChatInput onSend=… disabled=…>` —
            // disables when the conversation is closed.
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
    use crate::auth::user::{AuthMethod, User};

    fn conv_ctx(user_perms: &[&str]) -> PageContext {
        let user = User {
            id: "u1".to_string(),
            address: "0x1234…abcd".to_string(),
            chain_id: "56".to_string(),
            roles: vec!["user".to_string()],
            email: None,
            tier: Some("Pro".to_string()),
            permissions: user_perms.iter().map(|s| s.to_string()).collect(),
            last_login_at: None,
            auth_method: AuthMethod::Wallet,
            display_name: None,
        };
        let mut params = std::collections::HashMap::new();
        params.insert("id".to_string(), "1".to_string());
        PageContext {
            user: Some(user),
            path: "/chat/1".to_string(),
            params,
            ..Default::default()
        }
    }

    /// Wave 6A — `test_render_smoke`. Chat conversation page must
    /// render non-empty HTML.
    #[test]
    fn test_render_smoke() {
        let ctx = conv_ctx(&["chat:read", "chat:write"]);
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(!html.is_empty(), "Chat conversation page must render non-empty HTML.");
        assert!(html.len() > 200, "Chat conversation HTML is suspiciously short ({} bytes).", html.len());
    }
}

//! Sub-components extracted from `pages/chat_history.rs` during
//! Wave 6C Track E (1:1 user-side component parity).
//!
//! Two named sub-components: `ChatHistoryBody`, `ChatHistoryCard`.

use crate::components::user::chat::{ChatConversation, ChatStatusBadge, ChatTopic};
use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;

/// Filtered conversation list. Mirrors the source's
/// `<ChatHistoryPage>` body — a back-link, a filter bar, and a
/// list of cards.
#[component]
pub fn ChatHistoryBody() -> Element {
    let convos: Vec<ChatConversation> = sample_conversations();
    let topics: Vec<ChatTopic> = sample_topics();

    let mut status_filter = use_signal(|| "all".to_string());
    let mut topic_filter = use_signal(|| "all".to_string());

    let filtered: Vec<ChatConversation> = convos.iter()
        .filter(|c| {
            let status_ok = status_filter.read().as_str() == "all" || c.status == *status_filter.read();
            let topic_ok = topic_filter.read().as_str() == "all" || c.topic_id == *topic_filter.read();
            status_ok && topic_ok
        })
        .cloned()
        .collect();
    let count = convos.len();
    let count_str = count.to_string();
    let count_label = if count == 1 { "conversation" } else { "conversations" };

    rsx! {
        div { class: "chat-history-header",
            a { class: "chat-history-back", href: "/chat",
                Icon { name: "arrow-left".to_string(), size: Some(16) }
            }
            div { class: "chat-history-titles",
                h1 { class: "chat-history-title", "Chat History" }
                p { class: "chat-history-subtitle",
                    "{count_str} total {count_label}"
                }
            }
        }

        div { class: "chat-history-filters",
            Icon { name: "sliders".to_string(), size: Some(14) }
            select {
                class: "chat-history-filter",
                value: "{status_filter.read()}",
                onchange: move |e| status_filter.set(e.value().to_string()),
                option { value: "all", "All Statuses" }
                option { value: "open", "Open" }
                option { value: "in_progress", "In Progress" }
                option { value: "resolved", "Resolved" }
                option { value: "closed", "Closed" }
            }
            select {
                class: "chat-history-filter",
                value: "{topic_filter.read()}",
                onchange: move |e| topic_filter.set(e.value().to_string()),
                option { value: "all", "All Topics" }
                for t in topics.iter() {
                    option { value: "{t.id}", "{t.label}" }
                }
            }
        }

        if filtered.is_empty() {
            div { class: "chat-history-empty",
                div { class: "chat-history-empty-icon",
                    Icon { name: "inbox".to_string(), size: Some(28) }
                }
                p { class: "chat-history-empty-title", "No conversations found" }
                p { class: "chat-history-empty-hint", "Try adjusting your filters" }
            }
        } else {
            div { class: "chat-history-list",
                for (i, c) in filtered.iter().enumerate() {
                    ChatHistoryCard {
                        conv: c.clone(),
                        topics: topics.clone(),
                        is_last: i + 1 == filtered.len(),
                    }
                }
            }
        }
    }
}

/// Single conversation card in the history list.
#[component]
pub fn ChatHistoryCard(
    conv: ChatConversation,
    topics: Vec<ChatTopic>,
    is_last: bool,
) -> Element {
    let has_unread = conv.unread_user > 0;
    let unread_label = if conv.unread_user > 9 { "9+".to_string() } else { conv.unread_user.to_string() };
    let row_class = if has_unread {
        "chat-history-card chat-history-card-unread"
    } else {
        "chat-history-card"
    };
    let row_class = if is_last { format!("{} chat-history-card-last", row_class) } else { row_class.to_string() };
    let topic_label = topics.iter()
        .find(|t| t.id == conv.topic_id)
        .map(|t| t.label.clone())
        .unwrap_or_default();
    let href = format!("/chat/{}", conv.id);
    rsx! {
        a { class: "{row_class}", href: "{href}",
            div { class: "chat-history-card-main",
                h3 { class: "chat-history-card-subject", "{conv.subject}" }
                div { class: "chat-history-card-meta",
                    if !topic_label.is_empty() {
                        span { class: "chat-history-card-topic", "{topic_label}" }
                    }
                    ChatStatusBadge { status: conv.status.clone() }
                    span { class: "chat-history-card-time", "{conv.last_message_at}" }
                }
            }
            div { class: "chat-history-card-aside",
                if has_unread {
                    span { class: "chat-history-card-unread", "{unread_label}" }
                }
                Icon { name: "chevron-right".to_string(), size: Some(16) }
            }
        }
    }
}

fn sample_conversations() -> Vec<ChatConversation> {
    vec![
        ChatConversation {
            id: "1".to_string(),
            subject: "Plan upgrade question".to_string(),
            status: "open".to_string(),
            topic_id: "billing".to_string(),
            last_message_at: "10 minutes ago".to_string(),
            unread_user: 2,
            created_at: "2024-09-20T10:00:00Z".to_string(),
        },
        ChatConversation {
            id: "2".to_string(),
            subject: "Payment issue".to_string(),
            status: "in_progress".to_string(),
            topic_id: "billing".to_string(),
            last_message_at: "2 hours ago".to_string(),
            unread_user: 0,
            created_at: "2024-09-19T08:00:00Z".to_string(),
        },
        ChatConversation {
            id: "3".to_string(),
            subject: "API key question".to_string(),
            status: "resolved".to_string(),
            topic_id: "dev".to_string(),
            last_message_at: "1 day ago".to_string(),
            unread_user: 0,
            created_at: "2024-09-18T14:00:00Z".to_string(),
        },
        ChatConversation {
            id: "4".to_string(),
            subject: "Account security review".to_string(),
            status: "closed".to_string(),
            topic_id: "account".to_string(),
            last_message_at: "3 days ago".to_string(),
            unread_user: 0,
            created_at: "2024-09-16T11:00:00Z".to_string(),
        },
        ChatConversation {
            id: "5".to_string(),
            subject: "Bug: chart fails to render on Safari".to_string(),
            status: "open".to_string(),
            topic_id: "bug".to_string(),
            last_message_at: "5 days ago".to_string(),
            unread_user: 1,
            created_at: "2024-09-14T16:00:00Z".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Wave 6C Track E — `test_render_smoke` for the extracted
    /// chat history sub-components.
    #[test]
    fn chat_history_subcomponents_render_smoke() {
        // ChatHistoryBody
        let el = rsx! { ChatHistoryBody {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("chat-history"), "ChatHistoryBody missing section-marker");
        assert!(html.contains("Chat History"));
        assert!(html.contains("Plan upgrade question"));

        // ChatHistoryCard
        let conv = ChatConversation {
            id: "1".to_string(),
            subject: "Test subject".to_string(),
            status: "open".to_string(),
            topic_id: "billing".to_string(),
            last_message_at: "now".to_string(),
            unread_user: 3,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };
        let topics = sample_topics();
        let el = rsx! { ChatHistoryCard { conv, topics, is_last: false } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("chat-history-card"), "ChatHistoryCard missing section-marker");
        assert!(html.contains("Test subject"));
        assert!(html.contains("chat-history-card-unread"));
    }
}

//! Backend-backed support conversation history.

use dioxus::prelude::*;

use crate::auth::AuthGate;
use crate::layout::main_layout::MainLayout;
use crate::primitives::Icon;

use super::chat::{inbox_load, short_date, ChatInboxLoad, ChatTopic, StatusBadge};
use super::{PageContext, PageMeta};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let mut meta = PageMeta::app("Support chat history");
    meta.body_class = Some("page-bg".to_string());
    (
        meta,
        rsx! {
            MainLayout { ctx: ctx.clone(),
                AuthGate {
                    user: ctx.user.clone(),
                    feature: Some("private chat history".to_string()),
                    return_url: Some("/chat/history".to_string()),
                    wallet_connected: ctx.wallet.address.is_some(),
                    HistorySurface { ctx: ctx.clone() }
                }
            }
        },
    )
}

#[component]
fn HistorySurface(ctx: PageContext) -> Element {
    match inbox_load(&ctx) {
        ChatInboxLoad::Ready(inbox) | ChatInboxLoad::Empty(inbox) => {
            let (status, topic_id) = history_filters(&ctx.query);
            let filtered = inbox
                .conversations
                .iter()
                .filter(|conversation| {
                    status
                        .as_deref()
                        .is_none_or(|value| conversation.status == value)
                        && topic_id
                            .as_deref()
                            .is_none_or(|value| conversation.topic_id == value)
                })
                .cloned()
                .collect::<Vec<_>>();
            rsx! {
                div { class: "container page-content chat-history", "data-chat-history-state": if filtered.is_empty() { "empty" } else { "ready" },
                    div { class: "chat-history-header",
                        a { class: "chat-history-back", href: "/chat", aria_label: "Back to chat",
                            Icon { name: "arrow-left".to_string(), size: Some(16) }
                        }
                        div { class: "chat-history-titles",
                            h1 { class: "chat-history-title", "Chat History" }
                            p { class: "chat-history-subtitle", "{inbox.conversations.len()} total conversations" }
                        }
                    }
                    form { class: "chat-history-filters", method: "get", action: "/chat/history",
                        Icon { name: "sliders-horizontal".to_string(), size: Some(14) }
                        select { class: "chat-history-filter", name: "status", aria_label: "Status",
                            option { value: "all", selected: status.is_none(), "All Statuses" }
                            option { value: "open", selected: status.as_deref() == Some("open"), "Open" }
                            option { value: "in_progress", selected: status.as_deref() == Some("in_progress"), "In Progress" }
                            option { value: "resolved", selected: status.as_deref() == Some("resolved"), "Resolved" }
                            option { value: "closed", selected: status.as_deref() == Some("closed"), "Closed" }
                        }
                        select { class: "chat-history-filter", name: "topic", aria_label: "Topic",
                            option { value: "all", selected: topic_id.is_none(), "All Topics" }
                            for topic in inbox.topics.iter() {
                                option { value: "{topic.id}", selected: topic_id.as_deref() == Some(topic.id.as_str()), "{topic.label}" }
                            }
                        }
                        button { class: "btn btn-outline btn-sm", r#type: "submit", "Apply" }
                    }
                    if filtered.is_empty() {
                        div { class: "chat-history-empty", role: "status",
                            div { class: "chat-history-empty-icon",
                                Icon { name: "inbox".to_string(), size: Some(24) }
                            }
                            h2 { class: "chat-history-empty-title", "No conversations found" }
                            p { class: "chat-history-empty-hint", "Start a new conversation or adjust the filters." }
                        }
                    } else {
                        div { class: "chat-history-list",
                            for (index, conversation) in filtered.iter().enumerate() {
                                a {
                                    class: if index + 1 == filtered.len() { "chat-history-card chat-history-card-last" } else if conversation.unread_user > 0 { "chat-history-card chat-history-card-unread" } else { "chat-history-card" },
                                    href: format!("/chat/{}", conversation.id),
                                    div { class: "chat-history-card-main",
                                        h2 { class: "chat-history-card-subject", "{conversation.subject}" }
                                        div { class: "chat-history-card-meta",
                                            if let Some(topic) = topic_for(&inbox.topics, &conversation.topic_id) {
                                                span { class: "chat-history-card-topic", "{topic.label}" }
                                            }
                                            StatusBadge { status: conversation.status.clone() }
                                            span { class: "chat-history-card-time", "{short_date(&conversation.last_message_at)}" }
                                        }
                                    }
                                    div { class: "chat-history-card-aside",
                                        if conversation.unread_user > 0 {
                                            span { class: "chat-history-card-unread-badge", "{conversation.unread_user.min(99)}" }
                                        }
                                        Icon { name: "chevron-right".to_string(), size: Some(16) }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        ChatInboxLoad::Forbidden => {
            rsx! { HistoryProblem { title: "Chat history is not available for this account".to_string() } }
        }
        ChatInboxLoad::Malformed => {
            rsx! { HistoryProblem { title: "Chat history data could not be verified".to_string() } }
        }
        ChatInboxLoad::Unavailable => {
            rsx! { HistoryProblem { title: "Chat history is temporarily unavailable".to_string() } }
        }
    }
}

fn history_filters(query: &str) -> (Option<String>, Option<String>) {
    let mut status = None;
    let mut topic = None;
    for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
        match key.as_ref() {
            "status" if status.is_none() && value != "all" => {
                if matches!(
                    value.as_ref(),
                    "open" | "in_progress" | "resolved" | "closed"
                ) {
                    status = Some(value.into_owned());
                }
            }
            "topic" if topic.is_none() && value != "all" => {
                if uuid::Uuid::parse_str(&value).is_ok() {
                    topic = Some(value.into_owned());
                }
            }
            _ => {}
        }
    }
    (status, topic)
}

fn topic_for<'a>(topics: &'a [ChatTopic], id: &str) -> Option<&'a ChatTopic> {
    topics.iter().find(|topic| topic.id == id)
}

#[component]
fn HistoryProblem(title: String) -> Element {
    rsx! {
        div { class: "container page-content chat-history",
            div { class: "chat-history-empty", role: "alert", "data-chat-history-state": "unavailable",
                div { class: "chat-history-empty-icon",
                    Icon { name: "history".to_string(), size: Some(24) }
                }
                h1 { class: "chat-history-empty-title", "{title}" }
                a { class: "btn btn-outline mt-4", href: "/chat/history", "Try again" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_filters_accept_only_backend_fields() {
        assert_eq!(history_filters("status=open"), (Some("open".into()), None));
        assert_eq!(history_filters("status=admin&owner=other"), (None, None));
    }
}

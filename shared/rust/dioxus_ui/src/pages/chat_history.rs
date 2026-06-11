//! /chat/history — static list of past conversations with filters.
//!
//! Wave 6A Track C port — see `docs/wave6-auth-pages-depth/design.md`
//! §"Track C — chat + chat_history + chat_conversation +
//! notifications" / `chat_history.rs`.
//!
//! Mirrors the source `app/chat/history/page.tsx` (169 LoC). Section
//! list (in order):
//!   1. Header — back-link + page title + "N total conversations"
//!   2. Filter bar — status select + topic select inside a rounded
//!      panel
//!   3. Results list — conversation cards with subject, status
//!      badge, topic chip, time-ago label, and unread badge. An
//!      empty state renders when the filter returns nothing.
//!
//! Reuses `<ChatStatusBadge>` from `pages/chat.rs` (same Wave 6A
//! Track C scope). The `ChatConversation` + `ChatTopic` data shapes
//! live in `pages/chat.rs` and are re-imported here.

use dioxus::prelude::*;

use crate::primitives::*;
use crate::feedback::*;

use super::PageContext;
use super::PageMeta;
use super::chat::{ChatConversation, ChatStatusBadge, ChatTopic};
use crate::auth::AuthGate;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Support chat history");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("chat history".to_string()),
                required_permissions: Some(vec!["chat:read".to_string()]),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content chat-history",
                    PageHeader { title: "Chat history".to_string(),
                        description: Some("Past support conversations".to_string()),
                        icon: Some("history".to_string()) }
                    ChatHistoryBody {}
                }
            }
        }
    })
}

/// Filtered conversation list. Mirrors the source's
/// `<ChatHistoryPage>` body — a back-link, a filter bar, and a
/// list of cards (or an empty state). Server-renderable; the BFF
/// would hydrate the conversation list from
/// `listConversationsAction()` + `getTopicsAction()`.
#[component]
fn ChatHistoryBody() -> Element {
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
        // ── Section 1: Header (back-link + title + count) ──
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

        // ── Section 2: Filter bar ──
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

        // ── Section 3: Results ──
        if filtered.is_empty() {
            // Empty state (no conversations match the filter).
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

/// Single conversation card in the history list. Mirrors the
/// `filtered.map((convo, i) => …)` branch in
/// `app/chat/history/page.tsx` (lines 124-165). The card is
/// wrapped in a row that links to `/chat/<id>`.
#[component]
fn ChatHistoryCard(
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

/// Demo conversation list — the BFF would populate this from
/// `listConversationsAction()`. Mirrors the source's `convos`
/// shape (id, subject, status, topic_id, last_message_at,
/// unread_user, created_at). Five conversations give the filter
/// bar a meaningful set to filter against.
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

/// Demo topics — mirrored from `pages/chat.rs::sample_topics` so
/// the filter dropdowns stay consistent between the inbox and
/// history views.
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
    use crate::auth::user::{AuthMethod, User};

    /// Build a `PageContext` pointing at `/chat/history`. Only
    /// `user` + `path` matter for these tests.
    fn history_ctx(user_perms: &[&str]) -> PageContext {
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
        PageContext {
            user: Some(user),
            path: "/chat/history".to_string(),
            ..Default::default()
        }
    }

    /// Wave 6A — `test_render_smoke`. Chat history page must
    /// render non-empty HTML.
    #[test]
    fn test_render_smoke() {
        let ctx = history_ctx(&["chat:read"]);
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(!html.is_empty(), "Chat history page must render non-empty HTML.");
        assert!(html.len() > 200, "Chat history HTML is suspiciously short ({} bytes).", html.len());
    }
}

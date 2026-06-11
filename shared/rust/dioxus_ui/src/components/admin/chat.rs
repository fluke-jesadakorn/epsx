//! Sub-components for `/admin/chat` — Wave 6C Track D.
//!
//! 1:1 mirror of `apps-old/admin-frontend/components/chat/*.tsx`:
//!   1. `AdminChatInbox`              — left-hand conversation list
//!   2. `AdminChatConversationView`   — right-hand conversation view
//!   3. `ChatReplyInput`              — admin's reply input
//!   4. `ChatInboxSearch`             — search/filter bar
//!   5. `ChatUnreadBadge`             — unread-count pill
//!
//! The admin chat reuses the Wave 6A `<MessageBubble>` primitive from
//! `crate::chat::message_bubble`. Helper components
//! (`ConversationCard`, `CannedResponsesPopover`, `CannedResponseItem`,
//! `AssignAgentPopover`, `ChatStatsPanel`) stay private.

use crate::primitives::*;
use crate::chat::{Message, MessageBubble};

use dioxus::prelude::*;

// ============================================================================
// Section 5: ChatUnreadBadge
// ============================================================================

#[component]
pub fn ChatUnreadBadge(count: u32) -> Element {
    if count == 0 {
        return rsx! { Fragment {} };
    }
    rsx! {
        span { class: "chat-unread-badge",
            "{count}"
        }
    }
}

// ============================================================================
// Section 4: ChatInboxSearch
// ============================================================================

#[component]
pub fn ChatInboxSearch(
    status: String,
    topic: String,
    search: String,
    on_status_change: EventHandler<String>,
    on_topic_change: EventHandler<String>,
    on_search_change: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "chat-inbox-search p-3 border-b border-border/20 space-y-2",
            div { class: "flex items-center gap-2 px-3 py-2 bg-muted/30 rounded-lg",
                Icon { name: "search".to_string(), size: Some(14), class_name: Some("text-muted-foreground/50".to_string()) }
                input {
                    class: "flex-1 bg-transparent text-sm focus:outline-none",
                    placeholder: "Search conversations...",
                    value: "{search}",
                    oninput: move |e| on_search_change.call(e.value().to_string()),
                }
            }
            div { class: "grid grid-cols-2 gap-2",
                select {
                    class: "input",
                    value: "{status}",
                    onchange: move |e| on_status_change.call(e.value().to_string()),
                    option { value: "", "All statuses" }
                    option { value: "open", "Open" }
                    option { value: "in_progress", "In progress" }
                    option { value: "resolved", "Resolved" }
                    option { value: "closed", "Closed" }
                }
                input {
                    class: "input",
                    placeholder: "Topic...",
                    value: "{topic}",
                    oninput: move |e| on_topic_change.call(e.value().to_string()),
                }
            }
        }
    }
}

// ============================================================================
// Section 1: AdminChatInbox
// ============================================================================

#[component]
pub fn AdminChatInbox() -> Element {
    let mut status = use_signal(String::new);
    let mut topic = use_signal(String::new);
    let mut search = use_signal(String::new);
    let mut selected = use_signal(|| "1".to_string());

    let conversations = vec![
        ConversationRow { id: "1".into(), subject: "Plan upgrade question".into(), user: "0x1234\u{2026}5678".into(), status: "open".into(), last_reply: "2 min ago".into(), unread: 3, topic: "billing".into() },
        ConversationRow { id: "2".into(), subject: "Payment issue".into(), user: "0xabcd\u{2026}ef12".into(), status: "closed".into(), last_reply: "1 hour ago".into(), unread: 0, topic: "payments".into() },
        ConversationRow { id: "3".into(), subject: "API key question".into(), user: "0x9876\u{2026}5432".into(), status: "open".into(), last_reply: "5 min ago".into(), unread: 1, topic: "developers".into() },
        ConversationRow { id: "4".into(), subject: "Subscription renewal".into(), user: "0x5555\u{2026}aaaa".into(), status: "in_progress".into(), last_reply: "10 min ago".into(), unread: 0, topic: "subscriptions".into() },
    ];

    let selected_id = selected.read().clone();
    let cards: Vec<ConversationCardData> = conversations.iter().map(|c| ConversationCardData {
        id: c.id.clone(),
        subject: c.subject.clone(),
        user: c.user.clone(),
        status: c.status.clone(),
        last_reply: c.last_reply.clone(),
        unread: c.unread,
        selected: selected_id == c.id,
    }).collect();

    rsx! {
        div { class: "admin-chat-inbox h-full flex flex-col",
            ChatInboxSearch {
                status: status.read().clone(),
                topic: topic.read().clone(),
                search: search.read().clone(),
                on_status_change: move |v| status.set(v),
                on_topic_change: move |v| topic.set(v),
                on_search_change: move |v| search.set(v),
            }
            div { class: "flex-1 overflow-y-auto space-y-2 pr-1 scrollbar-thin",
                for card in cards {
                    ConversationCard {
                        id: card.id.clone(),
                        subject: card.subject.clone(),
                        user: card.user.clone(),
                        status: card.status.clone(),
                        last_reply: card.last_reply.clone(),
                        unread: card.unread,
                        selected: card.selected,
                        on_click: move |_| selected.set(card.id.clone()),
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ConversationCardData {
    id: String,
    subject: String,
    user: String,
    status: String,
    last_reply: String,
    unread: u32,
    selected: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct ConversationRow {
    id: String,
    subject: String,
    user: String,
    status: String,
    last_reply: String,
    unread: u32,
    topic: String,
}

#[component]
fn ConversationCard(
    id: String,
    subject: String,
    user: String,
    status: String,
    last_reply: String,
    unread: u32,
    selected: bool,
    on_click: EventHandler<MouseEvent>,
) -> Element {
    let mut cls = String::from("conversation-card p-3 rounded-lg cursor-pointer hover:bg-muted/30 transition-colors");
    if selected { cls.push_str(" bg-muted/50 border-l-2 border-primary"); }
    let status_cls = match status.as_str() {
        "open" => "badge-success",
        "in_progress" => "badge-info",
        "resolved" => "badge-primary",
        "closed" => "badge-outline",
        _ => "badge-outline",
    };
    rsx! {
        div { class: "{cls}", onclick: move |e| on_click.call(e),
            div { class: "flex items-start justify-between gap-2",
                div { class: "min-w-0 flex-1",
                    p { class: "text-sm font-semibold truncate", "{subject}" }
                    p { class: "text-xs text-muted-foreground truncate font-mono", "{user}" }
                }
                ChatUnreadBadge { count: unread }
            }
            div { class: "mt-1 flex items-center justify-between",
                span { class: "badge {status_cls} text-[10px]", "{status}" }
                span { class: "text-xs text-muted-foreground", "{last_reply}" }
            }
        }
    }
}

// ============================================================================
// Section 3: ChatReplyInput
// ============================================================================

#[component]
pub fn ChatReplyInput() -> Element {
    let mut msg = use_signal(String::new);
    let mut show_canned = use_signal(|| false);
    let mut show_assign = use_signal(|| false);
    rsx! {
        div { class: "chat-reply-input border-t border-border/20 p-4",
            div { class: "flex gap-2 mb-3 flex-wrap",
                button {
                    class: "px-3 py-1.5 text-[11px] font-bold bg-amber-500/10 text-amber-400 border border-amber-500/20 rounded-lg hover:bg-amber-500/20 transition-all flex items-center gap-1.5",
                    r#type: "button",
                    onclick: move |_| {
                        let cur = show_canned.read().clone();
                        show_canned.set(!cur);
                    },
                    Icon { name: "message-square".to_string(), size: Some(14) }
                    " Saved"
                }
                button {
                    class: "px-3 py-1.5 text-[11px] font-bold bg-blue-500/10 text-blue-400 border border-blue-500/20 rounded-lg hover:bg-blue-500/20 transition-all flex items-center gap-1.5",
                    r#type: "button",
                    onclick: move |_| {
                        let cur = show_assign.read().clone();
                        show_assign.set(!cur);
                    },
                    Icon { name: "user-plus".to_string(), size: Some(14) }
                    " Assign"
                }
                button {
                    class: "px-3 py-1.5 text-[11px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 rounded-lg hover:bg-emerald-500/20 transition-all flex items-center gap-1.5",
                    r#type: "button",
                    Icon { name: "check-circle".to_string(), size: Some(14) }
                    " Resolve"
                }
                button {
                    class: "px-3 py-1.5 text-[11px] font-bold bg-zinc-500/10 text-zinc-400 border border-zinc-500/20 rounded-lg hover:bg-zinc-500/20 transition-all flex items-center gap-1.5",
                    r#type: "button",
                    Icon { name: "x-circle".to_string(), size: Some(14) }
                    " Close"
                }
            }
            if *show_canned.read() {
                CannedResponsesPopover { on_select: move |text: String| { msg.set(text); show_canned.set(false); } }
            }
            if *show_assign.read() {
                AssignAgentPopover {}
            }
            div { class: "flex gap-2",
                textarea {
                    class: "input flex-1",
                    placeholder: "Type your reply... (supports **markdown**)",
                    rows: "2",
                    value: "{msg.read()}",
                    oninput: move |e| msg.set(e.value().to_string()),
                }
                button {
                    class: "px-4 rounded-xl flex items-center justify-center bg-gradient-to-r from-violet-500 to-purple-600 text-white hover:from-violet-400 hover:to-purple-500",
                    r#type: "button",
                    Icon { name: "send".to_string(), size: Some(16) }
                }
            }
        }
    }
}

#[component]
fn CannedResponsesPopover(on_select: EventHandler<String>) -> Element {
    rsx! {
        div { class: "canned-responses-popover mb-3 p-2 bg-card border border-border/20 rounded-lg",
            p { class: "text-xs font-bold text-muted-foreground uppercase tracking-wide px-1 mb-2", "Saved replies" }
            div { class: "space-y-1 max-h-48 overflow-y-auto",
                CannedResponseItem { label: "Welcome".to_string(), text: "Hello! Thanks for reaching out to EPSX support. How can I help you today?".to_string(), on_select: on_select }
                CannedResponseItem { label: "Looking into it".to_string(), text: "I'm looking into this for you right now. I'll get back to you shortly.".to_string(), on_select: on_select }
                CannedResponseItem { label: "Need more info".to_string(), text: "Could you please provide more details? Specifically, which wallet address and what transaction hash are you referring to?".to_string(), on_select: on_select }
                CannedResponseItem { label: "Resolved".to_string(), text: "I'm glad we could resolve this for you! Is there anything else I can help with?".to_string(), on_select: on_select }
                CannedResponseItem { label: "Docs link".to_string(), text: "You can find more information in our documentation at https://epsx.io/docs. Let me know if you have any questions!".to_string(), on_select: on_select }
            }
        }
    }
}

#[component]
fn CannedResponseItem(label: String, text: String, on_select: EventHandler<String>) -> Element {
    rsx! {
        button {
            class: "w-full px-3 py-2 text-left rounded-md hover:bg-muted/30 transition-colors",
            r#type: "button",
            onclick: move |_| on_select.call(text.clone()),
            p { class: "text-xs font-semibold", "{label}" }
            p { class: "text-[11px] text-muted-foreground/60 line-clamp-2", "{text}" }
        }
    }
}

#[component]
fn AssignAgentPopover() -> Element {
    rsx! {
        div { class: "assign-agent-popover mb-3 p-2 bg-card border border-border/20 rounded-lg",
            p { class: "text-xs font-bold text-muted-foreground uppercase tracking-wide px-1 mb-2", "Assign agent" }
            div { class: "space-y-1",
                div { class: "px-3 py-2 rounded-md hover:bg-blue-500/10 transition-colors cursor-pointer",
                    div { class: "flex items-center gap-2",
                        Icon { name: "user-plus".to_string(), size: Some(14), class_name: Some("text-blue-400".to_string()) }
                        div {
                            p { class: "text-xs font-semibold text-blue-400", "Assign to me" }
                            p { class: "text-[10px] text-muted-foreground/50 font-mono", "0xADMIN0000\u{2026}0001" }
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Section 2: AdminChatConversationView
// ============================================================================

#[component]
pub fn AdminChatConversationView(id: String) -> Element {
    let messages: Vec<Message> = vec![
        Message { id: "m1".into(), sender_name: "User".into(), sender_role: "User".into(), body: "Hi, I\u{2019}d like to upgrade my plan from Pro to Enterprise.".into(), created_at: "10:32 AM".into(), is_read: true, is_own: true, is_system: false, sender_type: "user".into(), attachment: None },
        Message { id: "m2".into(), sender_name: "Support".into(), sender_role: "Support".into(), body: "Sure! I can help with that. Do you have a preferred billing date?".into(), created_at: "10:33 AM".into(), is_read: true, is_own: false, is_system: false, sender_type: "agent".into(), attachment: None },
        Message { id: "m3".into(), sender_name: "User".into(), sender_role: "User".into(), body: "How about the 1st of next month?".into(), created_at: "10:35 AM".into(), is_read: false, is_own: true, is_system: false, sender_type: "user".into(), attachment: None },
        Message { id: "m4".into(), sender_name: "System".into(), sender_role: "".into(), body: "Conversation assigned to Alex".into(), created_at: "10:36 AM".into(), is_read: true, is_own: false, is_system: true, sender_type: "system".into(), attachment: None },
    ];

    rsx! {
        div { class: "admin-chat-conversation-view flex flex-col h-full",
            div { class: "border-b border-border/20 p-4",
                div { class: "flex items-start justify-between gap-4",
                    div { class: "min-w-0 flex-1",
                        h2 { class: "text-lg font-bold text-foreground truncate", "Plan upgrade question" }
                        div { class: "flex flex-wrap items-center gap-x-3 gap-y-1 mt-1",
                            code { class: "text-xs font-mono text-muted-foreground", "{id}" }
                            Badge { kind: BadgeKind::Success, "Open" }
                            Badge { kind: BadgeKind::Primary, "billing" }
                        }
                    }
                }
            }
            div { class: "flex-1 overflow-y-auto p-5 space-y-4 bg-background/20",
                for m in messages.iter() {
                    MessageBubble { message: m.clone(), is_own_message: m.is_own }
                }
            }
            ChatReplyInput {}
        }
    }
}

// ============================================================================
// ChatStatsPanel (used by the inbox page header)
// ============================================================================

#[component]
pub fn ChatStatsPanel() -> Element {
    rsx! {
        div { class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-6 chat-stats-panel",
            StatCard { label: "Total open".to_string(), value: "12".to_string(), icon: Some("inbox".to_string()) }
            StatCard { label: "In progress".to_string(), value: "5".to_string(), icon: Some("loader".to_string()) }
            StatCard { label: "Resolved (7d)".to_string(), value: "84".to_string(), icon: Some("check-circle".to_string()) }
            StatCard { label: "Unassigned".to_string(), value: "3".to_string(), icon: Some("user-plus".to_string()) }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn render_to_string(el: Element) -> String {
        dioxus_ssr::render_element(el)
    }

    #[test]
    fn test_render_smoke_chat_unread_badge() {
        let el = rsx! { ChatUnreadBadge { count: 3 } };
        let html = render_to_string(el);
        assert!(html.contains("chat-unread-badge"), "ChatUnreadBadge must render its class. Got: {}", html);
        assert!(html.contains("3"), "ChatUnreadBadge must render the count. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_chat_unread_badge_zero() {
        let el = rsx! { ChatUnreadBadge { count: 0 } };
        let html = render_to_string(el);
        assert!(!html.contains("chat-unread-badge"), "ChatUnreadBadge must render nothing when count=0. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_chat_inbox_search() {
        // EventHandler closures need a Dioxus runtime; use VirtualDom.
        let mut vdom = dioxus::prelude::VirtualDom::new(|| {
            rsx! {
                ChatInboxSearch {
                    status: String::new(),
                    topic: String::new(),
                    search: String::new(),
                    on_status_change: move |_| {},
                    on_topic_change: move |_| {},
                    on_search_change: move |_| {},
                }
            }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Search conversations..."), "ChatInboxSearch must render the search input. Got: {}", html);
        assert!(html.contains("All statuses"), "ChatInboxSearch must render the status select. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_admin_chat_inbox() {
        let el = rsx! { AdminChatInbox {} };
        let html = render_to_string(el);
        assert!(html.contains("Plan upgrade question"), "AdminChatInbox must render a sample conversation. Got: {}", html);
        assert!(html.contains("admin-chat-inbox"), "AdminChatInbox must render its class. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_chat_reply_input() {
        let el = rsx! { ChatReplyInput {} };
        let html = render_to_string(el);
        assert!(html.contains("chat-reply-input"), "ChatReplyInput must render its class. Got: {}", html);
        assert!(html.contains("Saved"), "ChatReplyInput must render the Saved button. Got: {}", html);
        assert!(html.contains("Resolve"), "ChatReplyInput must render the Resolve button. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_admin_chat_conversation_view() {
        let el = rsx! { AdminChatConversationView { id: "1".to_string() } };
        let html = render_to_string(el);
        assert!(html.contains("admin-chat-conversation-view"), "AdminChatConversationView must render its class. Got: {}", html);
        assert!(html.contains("Plan upgrade question"), "AdminChatConversationView must render the conversation subject. Got: {}", html);
    }
}

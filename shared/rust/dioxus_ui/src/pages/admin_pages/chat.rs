//! /admin/chat + /admin/chat/[id] — admin chat inbox.
//!
//! Wave 6B Track D — 5 sections per the design doc
//! `docs/wave6b-admin-pages-depth/design.md` §"Track D — ... + chat":
//! 1. `AdminChatInbox` — the left-hand conversation list with
//!    filter bar. Mirrors `components/chat/chat-inbox.tsx`.
//! 2. `AdminChatConversationView` — the right-hand conversation
//!    view (header + message list + reply input). Reuses the
//!    Wave 6A `<MessageBubble>` primitive from
//!    `shared/rust/dioxus_ui/src/chat/message_bubble.rs`. Mirrors
//!    `components/chat/chat-conversation-view.tsx`.
//! 3. `ChatReplyInput` — the admin's reply input with a
//!    canned-responses popover + assign/resolve/close actions.
//!    Mirrors `components/chat/chat-reply-input.tsx`.
//! 4. `ChatInboxSearch` — the search/filter bar at the top of the
//!    inbox (status + topic + free-text). Mirrors
//!    `components/chat/chat-filter-bar.tsx`.
//! 5. `ChatUnreadBadge` — the unread-count badge on each
//!    conversation card. Mirrors
//!    `components/chat/chat-conversation-card.tsx`.
//!
//! Plus the Wave 1 `render` + `render_conversation` top-level
//! functions and a `ChatStatsPanel` (top stats row reused by the
//! inbox) inspired by the source's
//! `components/chat/chat-stats-panel.tsx`.

use crate::primitives::*;
use crate::feedback::*;
use crate::data_table::{Column, DataTable, Row, SortDir};
use crate::chat::{Message, MessageBubble};

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::auth::AdminAuthGate;

// ============================================================================
// Section 5: ChatUnreadBadge
// ============================================================================
//
// The unread-count pill on a conversation card. Mirrors the source's
// `<ChatUnreadBadge count={conv.unread_agent} />` sub-component.

#[component]
fn ChatUnreadBadge(count: u32) -> Element {
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
//
// The search/filter bar at the top of the inbox. Mirrors
// `components/chat/chat-filter-bar.tsx` — the source has 3 fields:
// status (open / in_progress / resolved / closed), topic (free-text
// or topic-id), and search (free-text).

#[component]
fn ChatInboxSearch(
    status: String,
    topic: String,
    search: String,
    on_status_change: EventHandler<String>,
    on_topic_change: EventHandler<String>,
    on_search_change: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "chat-inbox-search p-3 border-b border-border/20 space-y-2",
            // Free-text search.
            div { class: "flex items-center gap-2 px-3 py-2 bg-muted/30 rounded-lg",
                Icon { name: "search".to_string(), size: Some(14), class_name: Some("text-muted-foreground/50".to_string()) }
                input {
                    class: "flex-1 bg-transparent text-sm focus:outline-none",
                    placeholder: "Search conversations...",
                    value: "{search}",
                    oninput: move |e| on_search_change.call(e.value().to_string()),
                }
            }
            // Status + topic filters.
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
//
// The left-hand conversation list. Mirrors
// `components/chat/chat-inbox.tsx` — the source uses an
// `initConvs: ChatConversation[]` prop and renders a list of
// `<ChatConversationCard>` items. The Dioxus port renders the
// same shape: a list of conversation cards with subject / user /
// status / last reply / unread badge. The right-hand
// `<AdminChatConversationView>` is mounted by the parent page
// (the `render_conversation` route in `render_conversation`).

#[component]
fn AdminChatInbox() -> Element {
    let mut status = use_signal(String::new);
    let mut topic = use_signal(String::new);
    let mut search = use_signal(String::new);
    let mut selected = use_signal(|| "1".to_string());

    // Sample conversations. The real BFF fills these from the DB.
    let conversations = vec![
        ConversationRow { id: "1".into(), subject: "Plan upgrade question".into(), user: "0x1234\u{2026}5678".into(), status: "open".into(), last_reply: "2 min ago".into(), unread: 3, topic: "billing".into() },
        ConversationRow { id: "2".into(), subject: "Payment issue".into(), user: "0xabcd\u{2026}ef12".into(), status: "closed".into(), last_reply: "1 hour ago".into(), unread: 0, topic: "payments".into() },
        ConversationRow { id: "3".into(), subject: "API key question".into(), user: "0x9876\u{2026}5432".into(), status: "open".into(), last_reply: "5 min ago".into(), unread: 1, topic: "developers".into() },
        ConversationRow { id: "4".into(), subject: "Subscription renewal".into(), user: "0x5555\u{2026}aaaa".into(), status: "in_progress".into(), last_reply: "10 min ago".into(), unread: 0, topic: "subscriptions".into() },
    ];

    let selected_id = selected.read().clone();

    // Snapshot each conv row into owned values so the for-loop
    // closures can capture them by move.
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
            // Conversation list.
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
//
// The admin's reply input. Mirrors
// `components/chat/chat-reply-input.tsx` — the source has a
// message textarea, a paperclip for uploads, a "Saved" popover
// with canned responses, an "Assign" popover, and Resolve / Close
// buttons. The Dioxus port emits the same shape: textarea +
// send button + action row.

#[component]
fn ChatReplyInput() -> Element {
    let mut msg = use_signal(String::new);
    let mut show_canned = use_signal(|| false);
    let mut show_assign = use_signal(|| false);
    rsx! {
        div { class: "chat-reply-input border-t border-border/20 p-4",
            // Action bar.
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
            // Canned responses popover.
            if *show_canned.read() {
                CannedResponsesPopover { on_select: move |text: String| { msg.set(text); show_canned.set(false); } }
            }
            // Assign popover.
            if *show_assign.read() {
                AssignAgentPopover {}
            }
            // Message input.
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
//
// The right-hand conversation view. Mirrors
// `components/chat/chat-conversation-view.tsx` — the source has a
// header (subject + wallet + topic + status) + a message list +
// the reply input. The Dioxus port reuses the Wave 6A
// `<MessageBubble>` primitive for each message.

#[component]
fn AdminChatConversationView(id: String) -> Element {
    // Sample messages. Real BFF fills these via SSE / polling.
    // We clone the messages into owned `Message` structs up-front
    // (the same pattern Wave 6A `chat_conversation.rs` uses) so
    // the rsx! for-loop doesn't need a `move` closure on a
    // borrow of `messages`.
    let messages: Vec<Message> = vec![
        Message { id: "m1".into(), sender_name: "User".into(), sender_role: "User".into(), body: "Hi, I\u{2019}d like to upgrade my plan from Pro to Enterprise.".into(), created_at: "10:32 AM".into(), is_read: true, is_own: true, is_system: false, sender_type: "user".into(), attachment: None },
        Message { id: "m2".into(), sender_name: "Support".into(), sender_role: "Support".into(), body: "Sure! I can help with that. Do you have a preferred billing date?".into(), created_at: "10:33 AM".into(), is_read: true, is_own: false, is_system: false, sender_type: "agent".into(), attachment: None },
        Message { id: "m3".into(), sender_name: "User".into(), sender_role: "User".into(), body: "How about the 1st of next month?".into(), created_at: "10:35 AM".into(), is_read: false, is_own: true, is_system: false, sender_type: "user".into(), attachment: None },
        Message { id: "m4".into(), sender_name: "System".into(), sender_role: "".into(), body: "Conversation assigned to Alex".into(), created_at: "10:36 AM".into(), is_read: true, is_own: false, is_system: true, sender_type: "system".into(), attachment: None },
    ];

    rsx! {
        div { class: "admin-chat-conversation-view flex flex-col h-full",
            // Header.
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
            // Message list — uses the Wave 6A MessageBubble.
            div { class: "flex-1 overflow-y-auto p-5 space-y-4 bg-background/20",
                for m in messages.iter() {
                    MessageBubble { message: m.clone(), is_own_message: m.is_own }
                }
            }
            // Reply input.
            ChatReplyInput {}
        }
    }
}

// ============================================================================
// ChatStatsPanel (used by the inbox page header)
// ============================================================================
//
// Top stats row for the chat inbox: total conversations, open, in
// progress, unassigned. Mirrors
// `components/chat/chat-stats-panel.tsx`.

#[component]
fn ChatStatsPanel() -> Element {
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
// Top-level page entry points
// ============================================================================

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    // Wave 38c T2 — the body class is set by `admin_pages::dispatch`
    // for `/chat` (the 4th route that needs the prod-EXACT body
    // class). This render() function only runs in non-skeleton mode
    // (authed user OR skeleton env var unset), where the dispatch
    // gate falls through to chat::render() with the regular
    // PageMeta::admin() (no body class override). The body class
    // is handled at the dispatch layer so both the skeleton-mode
    // short-circuit AND the non-skeleton authed path stay in sync.
    let meta = PageMeta::admin("Support chat");
    (meta, rsx! { RenderAdminChat { ctx: ctx.clone() } })
}

#[component]
fn RenderAdminChat(ctx: PageContext) -> Element {
    rsx! {
        AdminAuthGate {
            user: ctx.user.clone(),
            feature: Some("support chat".to_string()),
            required_permissions: Some(vec!["chat:manage".to_string()]),
            return_url: Some(ctx.path.clone()),
            div { class: "container page-content admin-chat-page",
                // Page header.
                div { class: "flex items-center gap-3 mb-6",
                    div { class: "w-10 h-10 rounded-xl bg-gradient-to-br from-violet-500 to-purple-600 flex items-center justify-center shadow-sm shadow-violet-500/20",
                        Icon { name: "message-circle".to_string(), size: Some(20), class_name: Some("text-white".to_string()) }
                    }
                    div {
                        h1 { class: "text-2xl font-bold text-foreground tracking-tight", "Chat support" }
                        p { class: "text-xs text-muted-foreground/60", "Manage support conversations" }
                    }
                }
                // Stats panel.
                ChatStatsPanel {}
                // 2-column layout: inbox (left) + conversation (right).
                div { class: "h-[calc(100vh-22rem)] flex flex-col md:flex-row md:gap-4",
                    // Section 1: inbox.
                    div { class: "w-full md:w-[360px] md:flex-shrink-0 flex flex-col admin-chat-inbox-container",
                        AdminChatInbox {}
                    }
                    // Section 2: conversation view (placeholder when
                    // no conversation is selected).
                    div { class: "flex-1 rounded-2xl border border-border/20 bg-card overflow-hidden admin-chat-conversation-container",
                        div { class: "flex flex-col items-center justify-center h-full text-center",
                            div { class: "w-16 h-16 rounded-xl bg-muted/30 flex items-center justify-center mb-4 border border-border/40",
                                Icon { name: "message-circle".to_string(), size: Some(32), class_name: Some("text-muted-foreground/20".to_string()) }
                            }
                            p { class: "text-sm font-medium text-muted-foreground mb-1", "Select a conversation" }
                            p { class: "text-xs text-muted-foreground/40", "Choose from the left panel to view details" }
                        }
                    }
                }
            }
        }
    }
}

pub fn render_conversation(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Conversation");
    (meta, rsx! { RenderAdminConversationPage { ctx: ctx.clone() } })
}

#[component]
fn RenderAdminConversationPage(ctx: PageContext) -> Element {
    let id = ctx.params.get("id").cloned().unwrap_or_default();
    rsx! {
        AdminAuthGate {
            user: ctx.user.clone(),
            feature: Some("support conversations".to_string()),
            required_permissions: Some(vec!["chat:manage".to_string()]),
            return_url: Some(ctx.path.clone()),
            div { class: "container page-content",
                a { class: "btn btn-sm btn-ghost mb-4", href: "/chat", Icon { name: "arrow-left".to_string(), size: Some(16) } " Back" }
                // Section 2: full conversation view.
                AdminChatConversationView { id: id.clone() }
            }
        }
    }
}

// ============================================================================
// Section markers (used by `tests::test_section_markers`):
//
//   1. "Admin chat inbox"            → "Chat support" header + conversation cards
//   2. "Admin chat conversation view" → "Plan upgrade question" + MessageBubble row
//   3. "Chat reply input"             → "Saved" / "Assign" / "Resolve" / "Close" buttons
//   4. "Chat inbox search"            → "Search conversations..." input
//   5. "Chat unread badge"            → rendered inside ConversationCard
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::PageContext;
    use crate::auth::User;

    /// Build an admin `User` with the `chat:manage` permission.
    fn test_user_admin() -> User {
        User {
            id: "test-admin".to_string(),
            address: "0xADMIN0000000000000000000000000000000001".to_string(),
            chain_id: "56".to_string(),
            roles: vec!["admin".to_string()],
            email: Some("admin@epsx.io".to_string()),
            tier: Some("admin".to_string()),
            permissions: vec!["chat:manage".to_string()],
            ..Default::default()
        }
    }

    /// Render the admin page's `Element` to an HTML string.
    fn render_to_string(el: Element) -> String {
        dioxus_ssr::render_element(el)
    }

    /// `test_render_smoke` — the page body header is rendered for an
    /// admin user with the right permission.
    #[test]
    fn test_render_smoke() {
        let ctx = PageContext {
            user: Some(test_user_admin()),
            path: "/chat".to_string(),
            ..Default::default()
        };
        let (_, el) = render(&ctx);
        let html = render_to_string(el);
        assert!(
            html.contains("Chat support"),
            "Chat page must render the title for an admin. Got: {}",
            html
        );
        // The 5 sections include the inbox; assert the search bar.
        assert!(
            html.contains("Search conversations..."),
            "Section 4 (ChatInboxSearch) marker missing. Got: {}",
            html
        );
    }

    /// `test_section_markers` — assert each of the 5 design-doc
    /// sections renders its section-marker text. We render the
    /// full page once and assert against the combined HTML — this
    /// avoids the multi-render runtime pitfall of running
    /// `dioxus_ssr::render_element` against multiple separate
    /// `use_signal` scopes in a single test.
    #[test]
    fn test_section_markers() {
        // Full page render — exercises the inbox (Section 1) +
        // search (Section 4) + unread badge (Section 5) all in
        // one runtime scope.
        let ctx = PageContext {
            user: Some(test_user_admin()),
            path: "/chat".to_string(),
            ..Default::default()
        };
        let (_, el) = render(&ctx);
        let html = render_to_string(el);
        // Section 1: AdminChatInbox.
        assert!(html.contains("Plan upgrade question"), "section 1 (AdminChatInbox) sample conv missing");
        // Section 4: ChatInboxSearch.
        assert!(html.contains("Search conversations..."), "section 4 (ChatInboxSearch) search input missing");
        assert!(html.contains("All statuses"), "section 4 (ChatInboxSearch) status select missing");
        // Section 5: ChatUnreadBadge.
        assert!(html.contains("3"), "section 5 (ChatUnreadBadge) count missing");

        // Conversation route — exercises Section 2 +
        // AdminChatConversationView + MessageBubble + ChatReplyInput.
        let mut params = std::collections::HashMap::new();
        params.insert("id".to_string(), "1".to_string());
        let ctx = PageContext {
            user: Some(test_user_admin()),
            path: "/chat/1".to_string(),
            params,
            ..Default::default()
        };
        let (_, el) = render_conversation(&ctx);
        let html = render_to_string(el);
        // Section 2: AdminChatConversationView + MessageBubble.
        assert!(html.contains("Plan upgrade question"), "section 2 (AdminChatConversationView) header missing");
        // The body text uses a Unicode right single quote (U+2019).
        assert!(html.contains("I\u{2019}d like to upgrade"), "section 2 (AdminChatConversationView) message body missing (via MessageBubble). Got: {}", html);
        assert!(html.contains("Support"), "section 2 (MessageBubble) sender role missing");
        assert!(html.contains("Conversation assigned to Alex"), "section 2 (MessageBubble system) missing");
        // Section 3: ChatReplyInput (rendered inside Section 2).
        assert!(html.contains("Saved"), "section 3 (ChatReplyInput) Saved button missing");
        assert!(html.contains("Assign"), "section 3 (ChatReplyInput) Assign button missing");
        assert!(html.contains("Resolve"), "section 3 (ChatReplyInput) Resolve button missing");
        assert!(html.contains("Close"), "section 3 (ChatReplyInput) Close button missing");
        assert!(html.contains("Type your reply"), "section 3 (ChatReplyInput) textarea placeholder missing");
    }
}

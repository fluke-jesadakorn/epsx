//! Truthful authenticated admin chat shells for `/chat` and `/chat/{id}`.
//!
//! The Rust admin BFF verifies the session audience, but it does not yet expose
//! typed chat reads or mutations. These routes therefore preserve a private,
//! production-shaped workspace while rendering an explicit unavailable state.
//! They do not infer authorization from frontend roles or permissions, and they
//! expose no sample conversations, messages, presence, counts, filters, canned
//! replies, assignments, status changes, or reply controls.

use chrono::DateTime;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::auth::AuthGate;
use crate::primitives::Icon;

use super::super::{PageContext, PageMeta};

const CHAT_PATH: &str = "/chat";
const MAX_PAGE: u32 = 50_001;
const MAX_LIMIT: u32 = 50;
const MAX_TEXT_CHARS: usize = 16_384;

pub const ADMIN_CHAT_LIST_DATA_PARAM: &str = "data_admin_chat_list";
pub const ADMIN_CHAT_LIST_STATE_PARAM: &str = "data_admin_chat_list_state";
pub const ADMIN_CHAT_DETAIL_DATA_PARAM: &str = "data_admin_chat_detail";
pub const ADMIN_CHAT_DETAIL_STATE_PARAM: &str = "data_admin_chat_detail_state";

pub const ADMIN_CHAT_READY: &str = "ready";
pub const ADMIN_CHAT_EMPTY: &str = "empty";
pub const ADMIN_CHAT_FORBIDDEN: &str = "forbidden";
pub const ADMIN_CHAT_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_CHAT_MALFORMED: &str = "malformed";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminChatConversationSummary {
    pub id: String,
    pub topic_id: String,
    pub wallet_address: String,
    pub subject: String,
    pub status: String,
    pub assigned_agent: Option<String>,
    pub last_message_at: String,
    pub unread_user: i32,
    pub unread_agent: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminChatMessageSummary {
    pub id: String,
    pub conversation_id: String,
    pub sender_type: String,
    pub sender_address: Option<String>,
    pub content: String,
    pub is_read: bool,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminChatList {
    pub items: Vec<AdminChatConversationSummary>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
    pub has_next: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminChatDetail {
    pub conversation: AdminChatConversationSummary,
    pub messages: Vec<AdminChatMessageSummary>,
}

pub fn decode_admin_chat_list(value: serde_json::Value) -> Option<AdminChatList> {
    let projection: AdminChatList = serde_json::from_value(value).ok()?;
    if projection.total < 0
        || !(1..=MAX_LIMIT).contains(&projection.limit)
        || !(1..=MAX_PAGE).contains(&projection.page)
        || projection.items.len() > projection.limit as usize
        || (projection.items.is_empty() && projection.has_next)
        || projection
            .items
            .iter()
            .any(|item| !valid_conversation(item))
    {
        return None;
    }
    let offset = i64::from(projection.page - 1).checked_mul(i64::from(projection.limit))?;
    if offset > 1_000_000
        || offset
            .checked_add(i64::try_from(projection.items.len()).ok()?)
            .is_none_or(|end| end > projection.total)
        || projection.has_next
            != offset
                .checked_add(i64::try_from(projection.items.len()).ok()?)
                .is_some_and(|end| end < projection.total)
    {
        return None;
    }
    Some(projection)
}

pub fn decode_admin_chat_detail(value: serde_json::Value) -> Option<AdminChatDetail> {
    let projection: AdminChatDetail = serde_json::from_value(value).ok()?;
    if !valid_conversation(&projection.conversation)
        || projection.messages.len() > 500
        || projection
            .messages
            .iter()
            .any(|message| !valid_message(message))
        || projection
            .messages
            .iter()
            .any(|message| message.conversation_id != projection.conversation.id)
    {
        return None;
    }
    Some(projection)
}

fn valid_conversation(item: &AdminChatConversationSummary) -> bool {
    valid_uuid(&item.id)
        && valid_uuid(&item.topic_id)
        && bounded_text(&item.wallet_address, 128)
        && bounded_text(&item.subject, 255)
        && matches!(
            item.status.as_str(),
            "open" | "in_progress" | "resolved" | "closed"
        )
        && item.unread_user >= 0
        && item.unread_agent >= 0
        && item
            .assigned_agent
            .as_deref()
            .is_none_or(|agent| bounded_text(agent, 128))
        && valid_timestamp(&item.last_message_at)
        && valid_timestamp(&item.created_at)
        && valid_timestamp(&item.updated_at)
}

fn valid_message(item: &AdminChatMessageSummary) -> bool {
    valid_uuid(&item.id)
        && valid_uuid(&item.conversation_id)
        && bounded_text(&item.sender_type, 32)
        && item
            .sender_address
            .as_deref()
            .is_none_or(|sender| bounded_text(sender, 128))
        && bounded_text(&item.content, MAX_TEXT_CHARS)
        && valid_timestamp(&item.created_at)
}

fn valid_uuid(value: &str) -> bool {
    uuid::Uuid::parse_str(value).is_ok()
}

fn bounded_text(value: &str, max_chars: usize) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= max_chars
        && !value.chars().any(char::is_control)
}

fn valid_timestamp(value: &str) -> bool {
    value.len() <= 64 && DateTime::parse_from_rfc3339(value).is_ok()
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ChatLoad {
    Ready(AdminChatList),
    Detail(AdminChatDetail),
    Empty,
    Forbidden,
    Unavailable,
    Malformed,
}

fn list_load(ctx: &PageContext) -> ChatLoad {
    let state = ctx
        .params
        .get(ADMIN_CHAT_LIST_STATE_PARAM)
        .map(String::as_str);
    match state {
        Some(ADMIN_CHAT_READY) | Some(ADMIN_CHAT_EMPTY) => {
            let Some(raw) = ctx.params.get(ADMIN_CHAT_LIST_DATA_PARAM) else {
                return ChatLoad::Malformed;
            };
            let Some(list) = serde_json::from_str(raw)
                .ok()
                .and_then(decode_admin_chat_list)
            else {
                return ChatLoad::Malformed;
            };
            if matches!(state, Some(ADMIN_CHAT_EMPTY))
                && (list.total != 0 || !list.items.is_empty())
            {
                return ChatLoad::Malformed;
            }
            if matches!(state, Some(ADMIN_CHAT_READY)) && list.items.is_empty() {
                return ChatLoad::Malformed;
            }
            if list.items.is_empty() {
                ChatLoad::Empty
            } else {
                ChatLoad::Ready(list)
            }
        }
        Some(ADMIN_CHAT_FORBIDDEN) => ChatLoad::Forbidden,
        Some(ADMIN_CHAT_MALFORMED) => ChatLoad::Malformed,
        Some(ADMIN_CHAT_UNAVAILABLE) | None => ChatLoad::Unavailable,
        Some(_) => ChatLoad::Malformed,
    }
}

fn detail_load(ctx: &PageContext) -> ChatLoad {
    match ctx
        .params
        .get(ADMIN_CHAT_DETAIL_STATE_PARAM)
        .map(String::as_str)
    {
        Some(ADMIN_CHAT_READY) => ctx
            .params
            .get(ADMIN_CHAT_DETAIL_DATA_PARAM)
            .and_then(|raw| serde_json::from_str(raw).ok())
            .and_then(decode_admin_chat_detail)
            .map(ChatLoad::Detail)
            .unwrap_or(ChatLoad::Malformed),
        Some(ADMIN_CHAT_FORBIDDEN) => ChatLoad::Forbidden,
        Some(ADMIN_CHAT_MALFORMED) => ChatLoad::Malformed,
        Some(ADMIN_CHAT_UNAVAILABLE) | None => ChatLoad::Unavailable,
        Some(_) => ChatLoad::Malformed,
    }
}

#[derive(Clone, Copy, PartialEq)]
enum ChatRoute {
    Inbox,
    Conversation,
}

impl ChatRoute {
    fn meta_title(self) -> &'static str {
        match self {
            Self::Inbox => "Support chat unavailable",
            Self::Conversation => "Conversation unavailable",
        }
    }

    fn surface(self) -> &'static str {
        match self {
            Self::Inbox => "inbox",
            Self::Conversation => "conversation",
        }
    }

    fn eyebrow(self) -> &'static str {
        match self {
            Self::Inbox => "Support workspace",
            Self::Conversation => "Conversation workspace",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Inbox => "Support conversations are unavailable",
            Self::Conversation => "This conversation cannot be verified",
        }
    }

    fn detail(self) -> &'static str {
        match self {
            Self::Inbox => {
                "No conversations, participants, messages, presence, unread counts, topics, statuses, assignments, or activity timestamps are shown because a backend-authoritative chat read contract is not connected."
            }
            Self::Conversation => {
                "No participant, message, presence, assignment, status, ownership, or activity data is shown because the backend has not verified the requested conversation."
            }
        }
    }
}

/// `/chat` — authenticated chat inbox shell with no compatibility data.
pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    render_route(ctx, ChatRoute::Inbox, None)
}

/// `/chat/{id}` — authenticated conversation shell. The route value is a
/// bounded, control-free, HTML-escaped diagnostic reference only. Its presence
/// never proves that a conversation exists, belongs to a user, or is readable.
pub fn render_conversation(ctx: &PageContext) -> (PageMeta, Element) {
    let route_reference = canonical_route_reference(ctx.params.get("id").map(String::as_str));
    render_route(ctx, ChatRoute::Conversation, route_reference)
}

fn render_route(
    ctx: &PageContext,
    route: ChatRoute,
    route_reference: Option<String>,
) -> (PageMeta, Element) {
    let meta = PageMeta::admin(route.meta_title());
    let retry_href = route_reference
        .as_deref()
        .map(conversation_href)
        .unwrap_or_else(|| CHAT_PATH.to_string());

    let load = match route {
        ChatRoute::Inbox => list_load(ctx),
        ChatRoute::Conversation => detail_load(ctx),
    };

    (
        meta,
        rsx! {
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("the private support chat workspace".to_string()),
                // Keep route references out of the signed-out response. The
                // authenticated unavailable shell may offer a bounded retry,
                // but the login boundary returns only to the static inbox.
                return_url: Some(CHAT_PATH.to_string()),
                ChatSurface { route, retry_href, load }
            }
        },
    )
}

/// Strip controls and cap the visible diagnostic value by Unicode scalar
/// count. Dioxus escapes the remaining display text at the HTML boundary.
fn canonical_route_reference(raw: Option<&str>) -> Option<String> {
    let raw = raw?.trim();
    uuid::Uuid::parse_str(raw).ok().map(|id| id.to_string())
}

/// Encode the already-bounded reference as one URL path segment. The display
/// value remains untrusted and unverified even when it is safe to navigate to.
fn conversation_href(reference: &str) -> String {
    let mut encoded = String::with_capacity(reference.len());
    for byte in reference.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(char::from(*byte));
            }
            _ => {
                const HEX: &[u8; 16] = b"0123456789ABCDEF";
                encoded.push('%');
                encoded.push(char::from(HEX[(byte >> 4) as usize]));
                encoded.push(char::from(HEX[(byte & 0x0f) as usize]));
            }
        }
    }
    format!("{CHAT_PATH}/{encoded}")
}

#[component]
fn ChatSurface(route: ChatRoute, retry_href: String, load: ChatLoad) -> Element {
    match load {
        ChatLoad::Ready(list) => rsx! { ChatListReady { list } },
        ChatLoad::Detail(detail) => rsx! { ChatDetailReady { detail } },
        ChatLoad::Empty => rsx! {
            ChatProblem {
                state: ADMIN_CHAT_EMPTY,
                title: "No support conversations were found".to_string(),
                detail: "The backend returned an authoritative empty conversation page.".to_string(),
                retry_href: retry_href.clone(),
            }
        },
        ChatLoad::Forbidden => rsx! {
            ChatProblem {
                state: ADMIN_CHAT_FORBIDDEN,
                title: "Chat access was denied".to_string(),
                detail: "The backend did not authorize this session to read support conversations.".to_string(),
                retry_href: retry_href.clone(),
            }
        },
        ChatLoad::Malformed => rsx! {
            ChatProblem {
                state: ADMIN_CHAT_MALFORMED,
                title: "Chat data could not be verified".to_string(),
                detail: "The backend response did not match the strict chat projection. No records are shown.".to_string(),
                retry_href: retry_href.clone(),
            }
        },
        ChatLoad::Unavailable => rsx! { ChatUnavailable { route, retry_href } },
    }
}

#[component]
fn ChatListReady(list: AdminChatList) -> Element {
    rsx! {
        section {
            class: "container page-content max-w-6xl py-10",
            "data-admin-chat-state": ADMIN_CHAT_READY,
            "data-admin-chat-surface": "inbox",
            h1 { class: "text-3xl font-black tracking-tight text-foreground", "Support conversations" }
            p { class: "mt-2 text-sm text-muted-foreground", "{list.total} backend-authoritative conversations" }
            ul { class: "mt-8 grid gap-4", aria_label: "Support conversations",
                for conversation in list.items {
                    li { class: "rounded-2xl border border-border/30 bg-card p-5 shadow-sm",
                        a { class: "block", href: conversation_href(&conversation.id),
                            div { class: "flex flex-wrap items-center justify-between gap-3",
                                h2 { class: "text-lg font-semibold text-foreground", "{conversation.subject}" }
                                span { class: "rounded-full border border-border/40 px-2 py-1 text-xs text-muted-foreground", "{conversation.status}" }
                            }
                            p { class: "mt-2 text-sm text-muted-foreground", "Last activity: {conversation.last_message_at}" }
                            if let Some(agent) = conversation.assigned_agent {
                                p { class: "mt-1 text-xs text-muted-foreground", "Assigned agent: {agent}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ChatDetailReady(detail: AdminChatDetail) -> Element {
    rsx! {
        section {
            class: "container page-content max-w-5xl py-10",
            "data-admin-chat-state": ADMIN_CHAT_READY,
            "data-admin-chat-surface": "conversation",
            a { class: "text-sm text-muted-foreground", href: CHAT_PATH, "← Conversation list" }
            h1 { class: "mt-4 text-3xl font-black tracking-tight text-foreground", "{detail.conversation.subject}" }
            p { class: "mt-2 text-sm text-muted-foreground", "Status: {detail.conversation.status} · Last activity: {detail.conversation.last_message_at}" }
            if detail.messages.is_empty() {
                p { class: "mt-8 rounded-2xl border border-border/30 bg-card p-6 text-sm text-muted-foreground", role: "status", "No messages were returned for this conversation." }
            } else {
                ol { class: "mt-8 space-y-4", aria_label: "Conversation messages",
                    for message in detail.messages {
                        li { class: "rounded-2xl border border-border/30 bg-card p-5",
                            p { class: "text-xs uppercase tracking-wide text-muted-foreground", "{message.sender_type} · {message.created_at}" }
                            p { class: "mt-3 whitespace-pre-wrap text-sm leading-6 text-foreground", "{message.content}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ChatProblem(state: &'static str, title: String, detail: String, retry_href: String) -> Element {
    rsx! {
        section {
            class: "container page-content max-w-5xl py-10",
            role: "status",
            "data-admin-chat-state": state,
            h1 { class: "text-3xl font-black tracking-tight text-foreground", "{title}" }
            p { class: "mt-4 max-w-3xl text-sm leading-6 text-muted-foreground", "{detail}" }
            nav { class: "mt-6 flex gap-3", aria_label: "Chat recovery",
                a { class: "btn btn-outline", href: retry_href, "Try again" }
                a { class: "btn btn-ghost", href: "/", "Admin home" }
            }
        }
    }
}

#[component]
fn ChatUnavailable(route: ChatRoute, retry_href: String) -> Element {
    let title_id = format!("admin-chat-{}-unavailable-title", route.surface());

    rsx! {
        div {
            class: "container page-content max-w-6xl py-10",
            "data-admin-chat-state": "unavailable",
            "data-admin-chat-surface": route.surface(),
            section {
                class: "relative overflow-hidden rounded-3xl border border-border/40 bg-card shadow-2xl",
                role: "status",
                aria_labelledby: title_id.clone(),
                div { class: "absolute inset-x-0 top-0 h-1 bg-gradient-to-r from-[#1fc7d4] via-[#7645d9] to-[#ed4b9e]" }
                div { class: "grid gap-8 p-8 md:grid-cols-[auto_1fr] md:p-12",
                    div {
                        class: "flex h-16 w-16 items-center justify-center rounded-2xl border border-violet-500/20 bg-violet-500/10 text-violet-400",
                        aria_hidden: "true",
                        Icon { name: "message-circle".to_string(), size: Some(30) }
                    }
                    div {
                        p { class: "text-xs font-black uppercase tracking-[0.22em] text-violet-400",
                            {route.eyebrow()}
                        }
                        h1 { id: title_id, class: "mt-3 text-3xl font-black tracking-tight text-foreground",
                            {route.title()}
                        }
                        div {
                            class: "mt-5 rounded-2xl border border-amber-500/20 bg-amber-500/10 p-5",
                            p { class: "text-sm font-semibold leading-6 text-foreground",
                                {route.detail()}
                            }
                        }
                        p { class: "mt-5 max-w-3xl text-sm leading-6 text-muted-foreground",
                            "The verified session keeps this workspace private, but only the Rust backend may authorize chat reads or management and return typed conversation data."
                        }
                        div { class: "mt-8 grid gap-4 sm:grid-cols-3",
                            BoundaryItem {
                                icon: "database",
                                title: "Conversation data",
                                detail: "Inbox and message records remain hidden without a typed backend response."
                            }
                            BoundaryItem {
                                icon: "shield",
                                title: "Authorization",
                                detail: "Frontend roles and permissions never grant read or management authority."
                            }
                            BoundaryItem {
                                icon: "send",
                                title: "Operations",
                                detail: "Replies, assignments, and status changes remain disabled without verified mutations."
                            }
                        }
                        nav { class: "mt-8 flex flex-wrap gap-3", aria_label: "Support chat recovery",
                            a { class: "btn btn-primary", href: retry_href,
                                Icon { name: "refresh-cw".to_string(), size: Some(16) }
                                " Retry chat availability"
                            }
                            if route == ChatRoute::Conversation {
                                a { class: "btn btn-outline", href: CHAT_PATH,
                                    Icon { name: "arrow-left".to_string(), size: Some(16) }
                                    " Conversation list"
                                }
                            }
                            a { class: "btn btn-ghost", href: "/",
                                Icon { name: "home".to_string(), size: Some(16) }
                                " Admin home"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn BoundaryItem(icon: &'static str, title: &'static str, detail: &'static str) -> Element {
    rsx! {
        div { class: "rounded-xl border border-border/20 bg-background/40 p-5",
            div { class: "flex items-center gap-2 font-semibold text-foreground",
                Icon { name: icon.to_string(), size: Some(18) }
                "{title}"
            }
            p { class: "mt-2 text-sm leading-6 text-muted-foreground", "{detail}" }
            span { class: "mt-3 inline-flex rounded-full border border-amber-500/30 bg-amber-500/10 px-2 py-1 text-xs font-medium text-amber-400",
                "Unavailable"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::auth::User;

    fn authenticated_ctx(path: &str) -> PageContext {
        PageContext {
            user: Some(User {
                id: "admin-session".to_string(),
                address: "0x1234".to_string(),
                chain_id: "56".to_string(),
                roles: vec![],
                permissions: vec![],
                ..Default::default()
            }),
            path: path.to_string(),
            ..Default::default()
        }
    }

    fn conversation_ctx(id: &str, signed_in: bool) -> PageContext {
        let mut ctx = authenticated_ctx("/chat/case-42");
        ctx.user = signed_in.then(|| ctx.user.take().expect("test user"));
        ctx.params = HashMap::from([("id".to_string(), id.to_string())]);
        ctx
    }

    fn html(element: Element) -> String {
        dioxus_ssr::render_element(element)
    }

    #[test]
    fn signed_out_routes_keep_chat_state_and_reference_private() {
        let inbox = html(
            render(&PageContext {
                path: CHAT_PATH.to_string(),
                ..Default::default()
            })
            .1,
        );
        let conversation = html(render_conversation(&conversation_ctx("private-case", false)).1);

        for rendered in [inbox, conversation] {
            assert!(rendered.contains("Sign in required"));
            assert!(!rendered.contains("data-admin-chat-state"));
            assert!(!rendered.contains("Support conversations are unavailable"));
            assert!(!rendered.contains("private-case"));
        }
    }

    #[test]
    fn roles_empty_authenticated_session_reaches_both_unavailable_surfaces() {
        let inbox = html(render(&authenticated_ctx(CHAT_PATH)).1);
        let conversation = html(render_conversation(&conversation_ctx("case-42", true)).1);

        assert!(inbox.contains("data-admin-chat-state=\"unavailable\""));
        assert!(inbox.contains("data-admin-chat-surface=\"inbox\""));
        assert!(conversation.contains("data-admin-chat-state=\"unavailable\""));
        assert!(conversation.contains("data-admin-chat-surface=\"conversation\""));
        assert!(conversation.contains("This conversation cannot be verified"));
        assert!(!inbox.contains("Permission required"));
        assert!(!conversation.contains("Permission required"));
    }

    #[test]
    fn unavailable_surfaces_emit_no_samples_counts_filters_or_actions() {
        let inbox = html(render(&authenticated_ctx(CHAT_PATH)).1);
        let conversation = html(render_conversation(&conversation_ctx("case-42", true)).1);
        let combined = format!("{inbox}{conversation}");

        for forbidden in [
            "Plan upgrade question",
            "Payment issue",
            "API key question",
            "Subscription renewal",
            "0x1234…5678",
            "Conversation assigned to Alex",
            "How about the 1st of next month?",
            "Total open",
            "Resolved (7d)",
            "Search conversations",
            "All statuses",
            "Saved replies",
            "Assign to me",
            "Type your reply",
            "Mark resolved",
            "Send reply",
            "chat-conversation-list",
            "chat-message-list",
            "<input",
            "<select",
            "<textarea",
            "<button",
        ] {
            assert!(!combined.contains(forbidden), "leaked chat UI: {forbidden}");
        }
    }

    #[test]
    fn legacy_and_hostile_non_id_params_are_ignored() {
        let mut ctx = authenticated_ctx(CHAT_PATH);
        ctx.query = "status=open&presence=online&unread=99".to_string();
        ctx.params = HashMap::from([
            (
                "messages".to_string(),
                "Conversation assigned to Alex".to_string(),
            ),
            ("reply".to_string(), "Send reply".to_string()),
            ("assignee".to_string(), "0xADMIN0000…0001".to_string()),
        ]);
        let rendered = html(render(&ctx).1);

        assert!(rendered.contains("data-admin-chat-state=\"unavailable\""));
        for forbidden in [
            "status=open",
            "presence=online",
            "unread=99",
            "Conversation assigned to Alex",
            "Send reply",
            "0xADMIN0000…0001",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "legacy value leaked: {forbidden}"
            );
        }
    }

    #[test]
    fn conversation_reference_requires_a_canonical_uuid_and_never_reflects_raw_input() {
        let hostile = format!("\u{0}\n\t\"><script>alert(1)</script>{}", "a".repeat(100));
        assert!(canonical_route_reference(Some(&hostile)).is_none());

        let rendered = html(render_conversation(&conversation_ctx(&hostile, true)).1);
        assert!(!rendered.contains("<script>"));
        assert!(rendered.contains("href=\"/chat\""));
        assert!(!rendered.contains("alert(1)"));
    }

    #[test]
    fn recovery_uses_native_safe_links_without_mutation_handlers() {
        let inbox = html(render(&authenticated_ctx(CHAT_PATH)).1);
        let conversation = html(render_conversation(&conversation_ctx("case 42", true)).1);

        assert!(inbox.contains("href=\"/chat\""));
        assert!(inbox.contains("href=\"/\""));
        assert!(conversation.contains("href=\"/chat\""));
        assert!(conversation.contains("Conversation list"));
        assert!(conversation.contains("href=\"/\""));
        assert!(!inbox.contains("onclick="));
        assert!(!conversation.contains("onclick="));
        assert!(!conversation.contains("javascript:"));
    }

    #[test]
    fn canonical_conversation_id_is_the_only_dynamic_retry_target() {
        let id = "550e8400-e29b-41d4-a716-446655440000";
        let rendered = html(render_conversation(&conversation_ctx(id, true)).1);
        assert!(rendered.contains("href=\"/chat/550e8400-e29b-41d4-a716-446655440000\""));
        assert!(!rendered.contains("case-42"));
    }
}

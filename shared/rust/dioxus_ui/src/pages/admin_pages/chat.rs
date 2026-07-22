//! Truthful authenticated admin chat shells for `/chat` and `/chat/{id}`.
//!
//! The Rust admin BFF verifies the session audience, but it does not yet expose
//! typed chat reads or mutations. These routes therefore preserve a private,
//! production-shaped workspace while rendering an explicit unavailable state.
//! They do not infer authorization from frontend roles or permissions, and they
//! expose no sample conversations, messages, presence, counts, filters, canned
//! replies, assignments, status changes, or reply controls.

use dioxus::prelude::*;

use crate::auth::AuthGate;
use crate::primitives::Icon;

use super::super::{PageContext, PageMeta};

const CHAT_PATH: &str = "/chat";
const MAX_ROUTE_REFERENCE_CHARS: usize = 64;

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
    let route_reference =
        bounded_route_reference(ctx.params.get("id").map(String::as_str).unwrap_or_default());
    render_route(ctx, ChatRoute::Conversation, Some(route_reference))
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

    // Query parameters and legacy hydration values are intentionally ignored.
    // Only a future backend-owned chat contract may create read or action state.
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
                ChatUnavailable { route, route_reference, retry_href }
            }
        },
    )
}

/// Strip controls and cap the visible diagnostic value by Unicode scalar
/// count. Dioxus escapes the remaining display text at the HTML boundary.
fn bounded_route_reference(raw: &str) -> String {
    let cleaned = raw
        .chars()
        .filter(|character| !character.is_control())
        .collect::<String>();
    let cleaned = cleaned.trim();

    if cleaned.is_empty() {
        return "not provided".to_string();
    }

    if cleaned.chars().count() <= MAX_ROUTE_REFERENCE_CHARS {
        return cleaned.to_string();
    }

    let mut bounded = cleaned
        .chars()
        .take(MAX_ROUTE_REFERENCE_CHARS.saturating_sub(1))
        .collect::<String>();
    bounded.push('…');
    bounded
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
fn ChatUnavailable(
    route: ChatRoute,
    route_reference: Option<String>,
    retry_href: String,
) -> Element {
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
                        if let Some(reference) = route_reference {
                            p { class: "mt-4 rounded-xl border border-border/30 bg-background/50 px-4 py-3 text-sm text-muted-foreground",
                                "Unverified route reference: "
                                code { "data-admin-chat-route-reference": "bounded", "{reference}" }
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
    fn conversation_reference_is_bounded_control_free_escaped_and_unverified() {
        let hostile = format!("\u{0}\n\t\"><script>alert(1)</script>{}", "a".repeat(100));
        let bounded = bounded_route_reference(&hostile);
        assert!(bounded.chars().count() <= MAX_ROUTE_REFERENCE_CHARS);
        assert!(!bounded.chars().any(char::is_control));

        let rendered = html(render_conversation(&conversation_ctx(&hostile, true)).1);
        assert!(rendered.contains("Unverified route reference"));
        assert!(rendered.contains("data-admin-chat-route-reference=\"bounded\""));
        assert!(!rendered.contains("<script>"));
        assert!(!rendered.contains("href=\"/chat/\"><script"));
        assert!(rendered.contains("%22%3E%3Cscript%3E"));
    }

    #[test]
    fn recovery_uses_native_safe_links_without_mutation_handlers() {
        let inbox = html(render(&authenticated_ctx(CHAT_PATH)).1);
        let conversation = html(render_conversation(&conversation_ctx("case 42", true)).1);

        assert!(inbox.contains("href=\"/chat\""));
        assert!(inbox.contains("href=\"/\""));
        assert!(conversation.contains("href=\"/chat/case%2042\""));
        assert!(conversation.contains("Conversation list"));
        assert!(conversation.contains("href=\"/\""));
        assert!(!inbox.contains("onclick="));
        assert!(!conversation.contains("onclick="));
        assert!(!conversation.contains("javascript:"));
    }
}

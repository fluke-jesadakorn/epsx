//! /chat — frontend chat surface (inbox shell + new conversation).
//!
//! Wave 6C Track E — the 9 chat sub-components were extracted to
//! `crate::components::user::chat`. The data types (`ChatConversation`,
//! `ChatTopic`) and the cross-page `pub` components (`ChatStatusBadge`,
//! `ChatPanel`) are re-exported here so `chat_history.rs` and
//! `chat_conversation.rs` keep their existing import paths.

use crate::components::user::chat::{ChatPanel, RenderChatInbox};
pub use crate::components::user::chat::{ChatConversation, ChatStatusBadge, ChatTopic};

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;

pub use crate::components::user::chat::sample_conversations as _chat_sample_conversations;
pub use crate::components::user::chat::sample_topics as _chat_sample_topics;
pub use crate::components::user::chat::sample_messages as _chat_sample_messages;

// ── Page entry ───────────────────────────────────────────────────────

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Support chat");
    (meta, rsx! { RenderChatInbox { ctx: ctx.clone() } })
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a `PageContext` pointing at `/chat`. Only `user` +
    /// `path` matter for these tests.
    fn chat_ctx(user_perms: &[&str]) -> PageContext {
        use crate::auth::user::{AuthMethod, User};
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
            path: "/chat".to_string(),
            ..Default::default()
        }
    }

    /// Wave 6A — `test_render_smoke`. Chat page must render
    /// non-empty HTML.
    #[test]
    fn test_render_smoke() {
        let ctx = chat_ctx(&["chat:read"]);
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(!html.is_empty(), "Chat page must render non-empty HTML.");
        assert!(html.len() > 200, "Chat page HTML is suspiciously short ({} bytes).", html.len());
    }

    /// Wave 6A — `test_section_markers`. The chat page must render
    /// every section the design doc claims.
    #[test]
    fn test_section_markers() {
        let ctx = chat_ctx(&["chat:read"]);
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        for marker in &[
            "chat-page",
            "chat-inbox",
            "chat-inbox-header",
            "chat-inbox-search",
            "chat-inbox-filters",
            "chat-inbox-list",
            "chat-inbox-newbar",
            "chat-status",
            "chat-inbox-new",
            "chat-panel",
            "chat-header",
            "chat-messages",
            "chat-message",
            "chat-bubble",
            "chat-input",
            "chat-input-row",
            "chat-input-send",
        ] {
            let needle = format!("class=\"{}\"", marker);
            let found = html.contains(&needle)
                || html.contains(&format!("\"{} ", marker))
                || html.contains(&format!(" {} ", marker))
                || html.contains(&format!(" {}", marker));
            assert!(
                found,
                "Chat page must contain section marker '{}'. Got: {}",
                needle, html
            );
        }
    }

    /// `<MessageBubble>` integration: the chat page must surface
    /// at least one rendered message bubble.
    #[test]
    fn test_message_bubble_renders() {
        let ctx = chat_ctx(&["chat:read"]);
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("chat-message") && html.contains("chat-bubble"),
            "Chat page must render at least one MessageBubble. Got: {}",
            html
        );
        assert!(
            html.contains("How can we help you today"),
            "Chat page must surface the sample message body. Got: {}",
            html
        );
    }
}

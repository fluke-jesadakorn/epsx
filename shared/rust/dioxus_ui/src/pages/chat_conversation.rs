//! /chat/[id] — dynamic conversation route.
//!
//! Wave 6C Track E — `ChatConversationBody` was extracted to
//! `crate::components::user::chat_conversation`. This page file
//! keeps the `render()` entry point and delegates.

use crate::components::user::chat_conversation::ChatConversationBody;
use crate::components::user::chat::{ChatConversation, ChatStatusBadge, ChatTopic};
use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
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

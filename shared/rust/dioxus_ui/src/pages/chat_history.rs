//! /chat/history — static list of past conversations with filters.
//!
//! Wave 6C Track E — `ChatHistoryBody` and `ChatHistoryCard` were
//! extracted to `crate::components::user::chat_history`. This page
//! file keeps the `render()` entry point and delegates.

use crate::components::user::chat_history::ChatHistoryBody;
use crate::components::user::chat::{ChatConversation, ChatStatusBadge, ChatTopic};
use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::{AuthMethod, User};

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

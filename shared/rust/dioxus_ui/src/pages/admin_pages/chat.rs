//! /admin/chat + /admin/chat/[id] — admin chat inbox.
//!
//! Wave 6C Track D — 5 sections per the design doc
//! `docs/wave6c-live-render-and-1to1/design.md` §"Track D":
//! 1. `AdminChatInbox`               — left-hand conversation list
//! 2. `AdminChatConversationView`    — right-hand conversation view
//! 3. `ChatReplyInput`               — admin's reply input
//! 4. `ChatInboxSearch`              — search/filter bar
//! 5. `ChatUnreadBadge`              — unread-count pill
//!
//! All 5 sub-components live in `components::admin::chat`. This
//! page just composes them inside the `AdminAuthGate` wrapper.

use crate::primitives::*;
use crate::components::admin::chat::{
    AdminChatInbox, AdminChatConversationView, ChatStatsPanel,
};

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::auth::AdminAuthGate;

// ============================================================================
// Top-level page entry points
// ============================================================================

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
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
                // Stats panel (helper, also from chat components).
                ChatStatsPanel {}
                // 2-column layout: inbox (left) + conversation (right).
                div { class: "h-[calc(100vh-22rem)] flex flex-col md:flex-row md:gap-4",
                    // Section 1.
                    div { class: "w-full md:w-[360px] md:flex-shrink-0 flex flex-col admin-chat-inbox-container",
                        AdminChatInbox {}
                    }
                    // Section 2 placeholder.
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
                // Section 2.
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
    /// sections renders its section-marker text.
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

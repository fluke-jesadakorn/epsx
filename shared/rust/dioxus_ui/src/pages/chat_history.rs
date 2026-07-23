//! `/chat/history` — private support-chat history.
//!
//! The development source loads owner-scoped conversations and topics, then
//! filters them in the client. Until Rust has that loader contract, this route
//! must not substitute sample rows, counts, statuses, timestamps, or filters.

use dioxus::prelude::*;

use crate::auth::AuthGate;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::primitives::*;

use super::{PageContext, PageMeta};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Support chat history unavailable");
    (
        meta,
        rsx! {
            MainLayout { ctx: ctx.clone(),
                AuthGate {
                    user: ctx.user.clone(),
                    feature: Some("private chat history".to_string()),
                    return_url: Some("/chat/history".to_string()),
                    div { class: "container page-content chat-history",
                        PageHeader {
                            title: "Chat history".to_string(),
                            description: Some("Past private support conversations".to_string()),
                            icon: Some("history".to_string()),
                            a { class: "btn btn-outline btn-sm", href: "/chat",
                                Icon { name: "arrow-left".to_string(), size: Some(14) }
                                " Inbox"
                            }
                        }
                        ChatHistoryUnavailable {}
                    }
                }
            }
        },
    )
}

#[component]
fn ChatHistoryUnavailable() -> Element {
    rsx! {
        section {
            class: "chat-history-empty card card-glass",
            role: "status",
            "data-section": "chat-history-unavailable",
            "data-chat-history-state": "unavailable",
            aria_labelledby: "chat-history-unavailable-title",
            div { class: "chat-history-empty-icon",
                Icon { name: "history".to_string(), size: Some(28) }
            }
            h2 { id: "chat-history-unavailable-title", class: "chat-history-empty-title",
                "Chat history unavailable"
            }
            p { class: "chat-history-empty-hint",
                "Owner-scoped conversation history cannot be loaded right now. No conversation count, result row, topic, status, unread state, or timestamp is being inferred."
            }
            div { class: "auth-gate-actions",
                a { class: "btn btn-outline", href: "/chat", "Return to inbox" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::{AuthMethod, User};

    fn signed_in_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "u1".to_string(),
                address: "0x1234…abcd".to_string(),
                chain_id: "56".to_string(),
                roles: vec!["user".to_string()],
                email: None,
                tier: None,
                permissions: vec![],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: None,
            }),
            path: "/chat/history".to_string(),
            ..Default::default()
        }
    }

    fn html(ctx: &PageContext) -> String {
        let (_, element) = render(ctx);
        dioxus_ssr::render_element(element)
    }

    #[test]
    fn signed_out_history_keeps_private_state_hidden() {
        let rendered = html(&PageContext {
            path: "/chat/history".to_string(),
            ..Default::default()
        });
        assert!(rendered.contains("Sign in required"));
        assert!(rendered.contains("return_url=%2Fchat%2Fhistory"));
        assert!(!rendered.contains("data-chat-history-state"));
    }

    #[test]
    fn authenticated_history_is_accessibly_unavailable_without_permission_gate() {
        let rendered = html(&signed_in_ctx());
        assert!(rendered.contains("data-chat-history-state=\"unavailable\""));
        assert!(rendered.contains("aria-labelledby=\"chat-history-unavailable-title\""));
        assert!(!rendered.contains("Permission required"));
    }

    #[test]
    fn history_suppresses_samples_counts_controls_and_self_recovery() {
        let rendered = html(&signed_in_ctx());
        for forbidden in [
            "Plan upgrade question",
            "Payment issue",
            "API key question",
            "total conversations",
            "10 minutes ago",
            "All Statuses",
            "All Topics",
            "<select",
            "<button",
            "href=\"/chat/history\"",
            ">Check again</a>",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "unsupported history content leaked: {forbidden}"
            );
        }
        assert!(rendered.contains("href=\"/chat\""));
        assert!(rendered.contains(">Return to inbox</a>"));
    }
}

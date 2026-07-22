//! `/chat` — private support-chat inbox.
//!
//! The pinned development implementation loads owner-scoped conversations and
//! topics before rendering its inbox. The Rust frontend does not yet have a
//! frozen owner-scoped loader or mutation contract, so this page deliberately
//! renders no conversation, topic, unread, presence, response-time, or message
//! claims. Authentication protects the private route; authorization remains a
//! backend concern once a real chat contract exists.

use dioxus::prelude::*;

use crate::auth::AuthGate;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::primitives::*;

use super::{PageContext, PageMeta};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Support chat unavailable");
    (meta, rsx! { RenderChatInbox { ctx: ctx.clone() } })
}

/// Preserve the recognizable inbox/panel shell without implying that an empty
/// owner response was received. No legacy query or hydration payload is read.
#[component]
fn RenderChatInbox(ctx: PageContext) -> Element {
    rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("private support chat".to_string()),
                return_url: Some("/chat".to_string()),
                div { class: "container page-content chat-page",
                    PageHeader {
                        title: "Support chat".to_string(),
                        description: Some("Private conversations with support".to_string()),
                        icon: Some("message-circle".to_string()),
                        a { class: "btn btn-outline btn-sm", href: "/chat/history",
                            Icon { name: "history".to_string(), size: Some(14) }
                            " History"
                        }
                    }
                    div { class: "chat-inbox-row",
                        aside { class: "chat-inbox",
                            div { class: "chat-inbox-header",
                                div { class: "chat-inbox-brand",
                                    div { class: "chat-inbox-avatar",
                                        Icon { name: "headset".to_string(), size: Some(20) }
                                    }
                                    div { class: "chat-inbox-titles",
                                        h2 { class: "chat-inbox-title", "Support Center" }
                                        p { class: "chat-inbox-subtitle", "Private support workspace" }
                                    }
                                }
                            }
                            div { class: "chat-inbox-list",
                                div { class: "chat-inbox-empty",
                                    div { class: "chat-inbox-empty-icon",
                                        Icon { name: "inbox".to_string(), size: Some(20) }
                                    }
                                    p { class: "chat-inbox-empty-title", "Inbox unavailable" }
                                    p { class: "chat-inbox-empty-hint",
                                        "No conversation count or empty-inbox claim is being inferred."
                                    }
                                }
                            }
                        }
                        section {
                            class: "chat-panel chat-panel-empty",
                            role: "status",
                            "data-section": "chat-unavailable",
                            "data-chat-state": "unavailable",
                            aria_labelledby: "chat-unavailable-title",
                            div { class: "chat-panel-empty-icon",
                                Icon { name: "message-circle".to_string(), size: Some(32) }
                            }
                            h2 { id: "chat-unavailable-title", class: "chat-panel-empty-title",
                                "Support conversations unavailable"
                            }
                            p { class: "chat-panel-empty-hint",
                                "Conversation data and chat actions are temporarily unavailable. No topics, messages, participants, timestamps, or statuses are shown."
                            }
                            div { class: "auth-gate-actions",
                                a { class: "btn btn-primary", href: "/chat", "Check again" }
                                a { class: "btn btn-outline", href: "/", "Back to home" }
                            }
                        }
                    }
                }
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
            path: "/chat".to_string(),
            ..Default::default()
        }
    }

    fn html(ctx: &PageContext) -> String {
        let (_, element) = render(ctx);
        dioxus_ssr::render_element(element)
    }

    #[test]
    fn signed_out_route_keeps_private_state_hidden() {
        let rendered = html(&PageContext {
            path: "/chat".to_string(),
            ..Default::default()
        });
        assert!(rendered.contains("Sign in required"));
        assert!(rendered.contains("href=\"/auth?return_url=%2Fchat\""));
        assert!(!rendered.contains("data-chat-state"));
    }

    #[test]
    fn authenticated_user_without_frontend_permission_sees_unavailable_state() {
        let rendered = html(&signed_in_ctx());
        assert!(rendered.contains("data-chat-state=\"unavailable\""));
        assert!(rendered.contains("aria-labelledby=\"chat-unavailable-title\""));
        assert!(!rendered.contains("Permission required"));
    }

    #[test]
    fn sample_and_presence_claims_are_suppressed() {
        let rendered = html(&signed_in_ctx());
        for forbidden in [
            "Plan upgrade question",
            "Payment issue",
            "API key question",
            "Usually replies in minutes",
            "Billing &amp; Payments",
            "EPSX Support",
            "Today",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "sample claim leaked: {forbidden}"
            );
        }
    }

    #[test]
    fn unavailable_inbox_has_no_fake_mutations_or_filters() {
        let rendered = html(&signed_in_ctx());
        for forbidden in [
            "<button",
            "<textarea",
            "<input",
            "<select",
            "New Conversation",
            "Start Conversation",
            "Resolve",
            "Type your reply",
            "Search conversations",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "fake chat control leaked: {forbidden}"
            );
        }
        assert!(rendered.contains("href=\"/chat\""));
        assert!(rendered.contains("href=\"/\""));
    }
}

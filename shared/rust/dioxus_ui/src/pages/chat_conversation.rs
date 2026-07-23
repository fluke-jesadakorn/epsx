//! `/chat/:id` — private support-chat conversation shell.
//!
//! The development route loads an owner-scoped conversation, messages, read
//! mutation, send/status mutations, and SSE updates. None of those contracts is
//! frozen for this Rust route. The only reflected value below is a bounded,
//! control-character-free route reference, explicitly labelled as unverified.

use dioxus::prelude::*;

use crate::auth::AuthGate;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::primitives::*;

use super::{PageContext, PageMeta};

const MAX_ROUTE_REFERENCE_CHARS: usize = 64;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Support conversation unavailable");
    let route_reference =
        bounded_route_reference(ctx.param("id").map(String::as_str).unwrap_or(""));

    (
        meta,
        rsx! {
            MainLayout { ctx: ctx.clone(),
                AuthGate {
                    user: ctx.user.clone(),
                    feature: Some("a private support conversation".to_string()),
                    return_url: Some("/chat".to_string()),
                    div { class: "container page-content chat-conversation",
                        PageHeader {
                            title: "Support conversation".to_string(),
                            description: Some("Private support workspace".to_string()),
                            icon: Some("message-circle".to_string()),
                            a { class: "btn btn-outline btn-sm", href: "/chat",
                                Icon { name: "arrow-left".to_string(), size: Some(14) }
                                " Inbox"
                            }
                        }
                        ChatConversationUnavailable { route_reference }
                    }
                }
            }
        },
    )
}

/// Bound the route label by Unicode scalar count and remove control characters.
/// Dioxus escapes the remaining text when it is inserted into HTML.
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

#[component]
fn ChatConversationUnavailable(route_reference: String) -> Element {
    rsx! {
        section {
            class: "chat-conv chat-panel chat-panel-empty",
            role: "status",
            "data-section": "chat-conversation-unavailable",
            "data-chat-conversation-state": "unavailable",
            aria_labelledby: "chat-conversation-unavailable-title",
            div { class: "chat-panel-empty-icon",
                Icon { name: "message-circle".to_string(), size: Some(32) }
            }
            h2 { id: "chat-conversation-unavailable-title", class: "chat-panel-empty-title",
                "Conversation unavailable"
            }
            p { class: "chat-panel-empty-hint",
                "This private conversation cannot be verified or loaded right now. No subject, participant, message, attachment, status, read state, or timestamp is shown."
            }
            p { class: "chat-conv-route-reference",
                "Unverified route reference: "
                code { "data-chat-route-reference": "bounded", "{route_reference}" }
            }
            div { class: "auth-gate-actions",
                a { class: "btn btn-primary", href: "/chat", "Return to inbox" }
                a { class: "btn btn-outline", href: "/", "Back to home" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::auth::user::{AuthMethod, User};

    fn ctx_with_id(id: &str, signed_in: bool) -> PageContext {
        let mut params = HashMap::new();
        params.insert("id".to_string(), id.to_string());
        // These hostile legacy payload names must have no effect on rendering.
        params.insert("data_chat".to_string(), "Plan upgrade question".to_string());
        params.insert("messages".to_string(), "EPSX Support".to_string());
        PageContext {
            user: signed_in.then(|| User {
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
            path: format!("/chat/{id}"),
            params,
            ..Default::default()
        }
    }

    fn html(ctx: &PageContext) -> String {
        let (_, element) = render(ctx);
        dioxus_ssr::render_element(element)
    }

    #[test]
    fn signed_out_conversation_hides_reference_and_private_state() {
        let rendered = html(&ctx_with_id("secret-owner-reference", false));
        assert!(rendered.contains("Sign in required"));
        assert!(!rendered.contains("secret-owner-reference"));
        assert!(!rendered.contains("data-chat-conversation-state"));
    }

    #[test]
    fn authenticated_conversation_is_accessibly_unavailable_without_permission_gate() {
        let rendered = html(&ctx_with_id("case-42", true));
        assert!(rendered.contains("data-chat-conversation-state=\"unavailable\""));
        assert!(rendered.contains("aria-labelledby=\"chat-conversation-unavailable-title\""));
        assert!(rendered.contains("Unverified route reference"));
        assert!(!rendered.contains("Permission required"));
    }

    #[test]
    fn route_reference_is_bounded_control_free_and_html_escaped() {
        let hostile = format!("<script>alert(1)</script>\n{}", "x".repeat(100));
        let label = bounded_route_reference(&hostile);
        assert!(label.chars().count() <= MAX_ROUTE_REFERENCE_CHARS);
        assert!(!label.chars().any(char::is_control));

        let rendered = html(&ctx_with_id(&hostile, true));
        assert!(!rendered.contains("<script>alert(1)</script>"));
        assert!(rendered.contains("&#60;script&#62;alert(1)&#60;/script&#62;"));
        assert!(rendered.contains("data-chat-route-reference=\"bounded\""));
        assert!(!rendered.contains(&"x".repeat(65)));
    }

    #[test]
    fn legacy_payloads_samples_and_fake_mutations_are_ignored() {
        let rendered = html(&ctx_with_id("case-42", true));
        for forbidden in [
            "Plan upgrade question",
            "EPSX Support",
            "Type your reply",
            "Resolve",
            "Attach file",
            "<button",
            "<textarea",
            "<form",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "unsupported conversation content leaked: {forbidden}"
            );
        }
        assert!(!rendered.contains("href=\"/chat/case-42\""));
        assert!(!rendered.contains(">Check again</a>"));
        assert!(rendered.contains("href=\"/chat\""));
        assert!(rendered.contains(">Return to inbox</a>"));
        assert!(rendered.contains("href=\"/\""));
        assert!(rendered.contains(">Back to home</a>"));
    }
}

//! Owner-scoped support conversation at `/chat/:id`.

use dioxus::prelude::*;

use crate::auth::AuthGate;
use crate::layout::main_layout::MainLayout;
use crate::primitives::Icon;

use super::chat::{detail_load, mutation_flash, ChatDetailLoad, ConversationPanel};
use super::{PageContext, PageMeta};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let mut meta = PageMeta::app("Support conversation");
    meta.body_class = Some("page-bg".to_string());
    (
        meta,
        rsx! {
            MainLayout { ctx: ctx.clone(),
                AuthGate {
                    user: ctx.user.clone(),
                    feature: Some("a private support conversation".to_string()),
                    return_url: Some(ctx.path.clone()),
                    wallet_connected: ctx.wallet.address.is_some(),
                    ConversationSurface { ctx: ctx.clone() }
                }
            }
        },
    )
}

#[component]
fn ConversationSurface(ctx: PageContext) -> Element {
    let flash = mutation_flash(&ctx.query);
    match detail_load(&ctx) {
        ChatDetailLoad::Ready(detail) => rsx! {
            div { class: "container page-content chat-conversation",
                ConversationPanel {
                    detail: *detail,
                    topic_label: "Support".to_string(),
                    standalone: true,
                    flash
                }
            }
        },
        ChatDetailLoad::Forbidden => rsx! { ConversationProblem {
            title: "Conversation not available".to_string(),
            detail: "This conversation does not belong to the signed-in account.".to_string()
        } },
        ChatDetailLoad::Malformed => rsx! { ConversationProblem {
            title: "Conversation data could not be verified".to_string(),
            detail: "The support response did not match the expected contract, so no messages are shown.".to_string()
        } },
        ChatDetailLoad::Unavailable => rsx! { ConversationProblem {
            title: "Conversation temporarily unavailable".to_string(),
            detail: "We could not load this support conversation. Please try again shortly.".to_string()
        } },
    }
}

#[component]
fn ConversationProblem(title: String, detail: String) -> Element {
    rsx! {
        div { class: "container page-content chat-conversation",
            section { class: "chat-conv chat-panel-empty", role: "alert", "data-chat-conversation-state": "unavailable",
                div { class: "chat-panel-empty-icon",
                    Icon { name: "message-circle".to_string(), size: Some(32) }
                }
                h1 { class: "chat-panel-empty-title", "{title}" }
                p { class: "chat-panel-empty-hint", "{detail}" }
                a { class: "btn btn-outline mt-4", href: "/chat", "Return to inbox" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_backend_projection_fails_closed() {
        let (_, element) = render(&PageContext {
            path: "/chat/550e8400-e29b-41d4-a716-446655440000".into(),
            ..Default::default()
        });
        let html = dioxus_ssr::render_element(element);
        assert!(html.contains("Sign in required"));
        assert!(!html.contains("Type a message"));
    }
}

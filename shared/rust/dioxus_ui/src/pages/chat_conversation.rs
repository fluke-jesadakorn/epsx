use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::auth::AuthGate;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Support chat");
    let conv_id = ctx.param("id").cloned().unwrap_or_default();
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("support chat".to_string()),
                div { class: "container page-content",
                    PageHeader { title: format!("Conversation #{}", conv_id), description: Some("Live conversation with EPSX support".to_string()), icon: Some("message-circle".to_string()),
                        a { class: "btn btn-outline", href: "/chat", "Inbox" }
                    }
                    div { class: "card card-glass",
                        div { class: "card-body chat-thread",
                            div { class: "chat-message chat-message-other",
                                div { class: "chat-bubble", "Hello! How can we help you today?" }
                                div { class: "chat-meta", "Support · 1m ago" }
                            }
                        }
                        div { class: "chat-input-wrap",
                            input { class: "input", placeholder: "Type a message…", name: "message" }
                            button { class: "btn btn-primary", "Send" }
                        }
                    }
                }
            }
        }
    })
}

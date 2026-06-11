use crate::primitives::*;
use crate::feedback::*;


use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::auth::AuthGate;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Support chat history");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("chat history".to_string()),
                required_permissions: Some(vec!["chat:read".to_string()]),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content",
                    PageHeader { title: "Conversation history".to_string(), description: Some("Past support conversations".to_string()), icon: Some("history".to_string()) }
                    div { class: "card card-glass",
                        div { class: "card-body",
                            EmptyState { title: "No past conversations".to_string(), description: Some("Once you finish a conversation it will show up here.".to_string()), icon: Some("history".to_string()) }
                        }
                    }
                }
            }
        }
    })
}

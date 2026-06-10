use crate::primitives::*;
use crate::feedback::*;


use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::{Navbar, Footer, PageHeader};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Support chat history");
    (meta, rsx! {
        Navbar { user: ctx.user.clone(), current_path: Some(ctx.path.clone()) }
        div { class: "container page-content",
            PageHeader { title: "Conversation history".to_string(), description: Some("Past support conversations".to_string()), icon: Some("history".to_string()) }
            div { class: "card card-glass",
                div { class: "card-body",
                    EmptyState { title: "No past conversations".to_string(), description: Some("Once you finish a conversation it will show up here.".to_string()), icon: Some("history".to_string()) }
                }
            }
        }
        Footer {}
    })
}

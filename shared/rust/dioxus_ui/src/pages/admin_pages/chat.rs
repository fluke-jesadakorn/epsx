//! /admin/chat + /admin/chat/[id] — admin chat inbox.

use crate::primitives::*;
use crate::feedback::*;
use crate::data_table::{Column, DataTable, Row, SortDir};

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::auth::AdminAuthGate;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Support chat");
    let columns = vec![
        Column { key: "subject".into(), label: "Subject".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("35%".into()), class_name: None },
        Column { key: "user".into(), label: "User".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("25%".into()), class_name: None },
        Column { key: "status".into(), label: "Status".into(), sortable: true, align: crate::primitives::data_table::Align::Center, width: Some("15%".into()), class_name: None },
        Column { key: "last_reply".into(), label: "Last reply".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("25%".into()), class_name: None },
    ];
    let rows = vec![
        Row { id: "1".into(), cells: vec!["Plan upgrade question".into(), "0x1234…5678".into(), "Open".into(), "2 min ago".into()] },
        Row { id: "2".into(), cells: vec!["Payment issue".into(), "0xabcd…ef12".into(), "Closed".into(), "1 hour ago".into()] },
        Row { id: "3".into(), cells: vec!["API key question".into(), "0x9876…5432".into(), "Open".into(), "5 min ago".into()] },
    ];
    (meta, rsx! {
        AdminAuthGate { user: ctx.user.clone(), feature: Some("support chat".to_string()), required_permissions: Some(vec!["chat:manage".to_string()]), return_url: Some(ctx.path.clone()),
            div { class: "container page-content",
                div { class: "flex items-center justify-between mb-6",
                    div {
                        h1 { class: "text-2xl font-bold", "Support chat" }
                        p { class: "text-muted-foreground", "Open conversations from users" }
                    }
                }
                DataTable { columns, rows, striped: true, page_size: 25, filter_placeholder: Some("Filter by subject, user, status...".to_string()), initial_sort: Some(("last_reply".to_string(), SortDir::Desc)) }
            }
        }
    })
}

pub fn render_conversation(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Conversation");
    (meta, rsx! { RenderAdminConversation { ctx: ctx.clone() } })
}

#[component]
fn RenderAdminConversation(ctx: PageContext) -> Element {
    let id = ctx.params.get("id").cloned().unwrap_or_default();
    let mut reply = use_signal(String::new);
    rsx! {
        AdminAuthGate { user: ctx.user.clone(), feature: Some("support conversations".to_string()), required_permissions: Some(vec!["chat:manage".to_string()]), return_url: Some(ctx.path.clone()),
            div { class: "container page-content",
                a { class: "btn btn-sm btn-ghost mb-4", href: "/chat", Icon { name: "arrow-left".to_string(), size: Some(16) } " Back" }
                div { class: "card card-glass",
                    div { class: "card-header",
                        div { class: "flex items-center justify-between",
                            div { h2 { class: "text-lg font-bold", "Plan upgrade question" } code { class: "text-sm text-muted-foreground", "{id}" } }
                            div { class: "flex gap-2", span { class: "badge badge-success", "Open" } button { class: "btn btn-sm btn-outline", r#type: "button", "Close" } }
                        }
                    }
                    div { class: "card-body",
                        div { class: "chat-messages space-y-4 mb-4",
                            div { class: "chat-message-self flex justify-end", div { class: "chat-bubble chat-bubble-self p-3 rounded-lg max-w-md bg-primary text-primary-foreground", p { "Hi, I'd like to upgrade my plan from Pro to Enterprise." } span { class: "text-xs opacity-70", "User · 10:32 AM" } } }
                            div { class: "chat-message-other", div { class: "chat-bubble chat-bubble-other p-3 rounded-lg max-w-md", p { "Sure! I can help with that. Do you have a preferred billing date?" } span { class: "text-xs text-muted-foreground", "Support · 10:33 AM" } } }
                            div { class: "chat-message-self flex justify-end", div { class: "chat-bubble chat-bubble-self p-3 rounded-lg max-w-md bg-primary text-primary-foreground", p { "How about the 1st of next month?" } span { class: "text-xs opacity-70", "User · 10:35 AM" } } }
                        }
                        div { class: "chat-input flex gap-2",
                            textarea { class: "input flex-1", placeholder: "Type your reply...", rows: "3", value: "{reply.read()}", oninput: move |e| reply.set(e.value().to_string()) }
                            button { class: "btn btn-primary", r#type: "button", Icon { name: "send".to_string(), size: Some(16) } " Send" }
                        }
                    }
                }
            }
        }
    }
}

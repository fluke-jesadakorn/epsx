//! /chat + /chat/[id] + /chat/history — frontend chat.

use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::auth::AuthGate;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Support chat");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("support chat".to_string()),
                div { class: "container page-content",
                    PageHeader { title: "Support chat".to_string(), description: Some("Open or past conversations".to_string()), icon: Some("message-circle".to_string()),
                        a { class: "btn btn-outline", href: "/chat/history", Icon { name: "history".to_string(), size: Some(16) } " History" }
                    }
                    div { class: "card card-glass",
                        div { class: "card-body",
                            EmptyState { title: "No conversations yet".to_string(), description: Some("Start a new conversation to get help.".to_string()), icon: Some("message-circle".to_string()),
                                action: Some("Start conversation".to_string()), action_href: Some("/chat/new".to_string()) }
                        }
                    }
                }
            }
        }
    })
}

pub fn render_conversation(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Conversation");
    (meta, rsx! { RenderConversation { ctx: ctx.clone() } })
}

#[component]
fn RenderConversation(ctx: PageContext) -> Element {
    let id = ctx.params.get("id").cloned().unwrap_or_default();
    let mut message = use_signal(String::new);
    rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("chat conversations".to_string()),
                div { class: "container page-content",
                    a { class: "btn btn-sm btn-ghost mb-4", href: "/chat", Icon { name: "arrow-left".to_string(), size: Some(16) } " Back" }
                    div { class: "card card-glass",
                        div { class: "card-header",
                            div { class: "flex items-center justify-between",
                                div { h2 { class: "text-lg font-bold", "Conversation" } code { class: "text-sm text-muted-foreground", "{id}" } }
                                span { class: "badge badge-success", "Open" }
                            }
                        }
                        div { class: "card-body",
                            div { class: "chat-messages space-y-4 mb-4",
                                div { class: "chat-message-other", div { class: "chat-bubble chat-bubble-other p-3 rounded-lg max-w-md", p { "Hi! How can we help you today?" } span { class: "text-xs text-muted-foreground", "10:32 AM" } } }
                                div { class: "chat-message-self flex justify-end", div { class: "chat-bubble chat-bubble-self p-3 rounded-lg max-w-md bg-primary text-primary-foreground", p { "I'd like to upgrade my plan" } span { class: "text-xs opacity-70", "10:33 AM" } } }
                            }
                            div { class: "chat-input flex gap-2",
                                textarea { class: "input flex-1", placeholder: "Type your message...", rows: "2", value: "{message.read()}", oninput: move |e| message.set(e.value().to_string()) }
                                button { class: "btn btn-primary", r#type: "button", Icon { name: "send".to_string(), size: Some(16) } }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn render_history(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Chat history");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("chat history".to_string()),
                div { class: "container page-content",
                    PageHeader { title: "Chat history".to_string(), description: Some("Past conversations with support".to_string()), icon: Some("history".to_string()) }
                    div { class: "card card-glass",
                        div { class: "card-body p-0",
                            div { class: "table-wrap",
                                table { class: "table",
                                    thead { tr { th { "Subject" } th { "Status" } th { "Last message" } th { "Date" } } }
                                    tbody {
                                        tr { td { a { href: "/chat/1", "Plan upgrade question" } } td { span { class: "badge badge-success", "Open" } } td { "I'd like to upgrade..." } td { "2024-09-20" } }
                                        tr { td { a { href: "/chat/2", "Payment issue" } } td { span { class: "badge badge-info", "Closed" } } td { "Thanks for the help!" } td { "2024-09-15" } }
                                        tr { td { a { href: "/chat/3", "API key question" } } td { span { class: "badge badge-success", "Open" } } td { "How do I create an API..." } td { "2024-09-10" } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}

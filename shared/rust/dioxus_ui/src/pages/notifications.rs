//! /notifications — notification center with bell widget behavior, list,
//! mark-read, delete, clear-all.

use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::auth::AuthGate;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Notifications");
    (meta, rsx! { RenderNotifications { ctx: ctx.clone() } })
}

#[component]
fn RenderNotifications(ctx: PageContext) -> Element {
    let data: Option<serde_json::Value> = ctx.params.get("data_notifications")
        .and_then(|s| serde_json::from_str(s).ok());
    let items: Vec<Notification> = data.as_ref()
        .and_then(|d| serde_json::from_value(d.get("items").cloned().unwrap_or(serde_json::json!([]))).ok())
        .unwrap_or_default();
    let mut filter = use_signal(|| "all".to_string());
    rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("your notifications".to_string()),
                required_permissions: Some(vec!["notifications:read".to_string()]),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content",
                    PageHeader { title: "Notifications".to_string(), description: Some(format!("{} notification(s)", items.len())), icon: Some("bell".to_string()) }
                    div { class: "flex gap-2 mb-4",
                        button { class: if *filter.read() == "all" { "btn btn-sm btn-primary" } else { "btn btn-sm btn-outline" }, onclick: move |_| filter.set("all".to_string()), "All" }
                        button { class: if *filter.read() == "unread" { "btn btn-sm btn-primary" } else { "btn btn-sm btn-outline" }, onclick: move |_| filter.set("unread".to_string()), "Unread" }
                        button { class: if *filter.read() == "read" { "btn btn-sm btn-primary" } else { "btn btn-sm btn-outline" }, onclick: move |_| filter.set("read".to_string()), "Read" }
                        div { class: "ml-auto flex gap-2",
                            button { class: "btn btn-sm btn-outline", r#type: "button", "Mark all read" }
                            button { class: "btn btn-sm btn-outline", r#type: "button", "Clear all" }
                        }
                    }
                    div { class: "card card-glass",
                        div { class: "card-body p-0",
                            if items.is_empty() {
                                div { class: "p-8", EmptyState { title: "You're all caught up".to_string(), description: Some("New notifications will appear here.".to_string()), icon: Some("bell-off".to_string()) } }
                            } else {
                                for n in items.iter() { NotificationRow { n: n.clone() } }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, PartialEq)]
pub struct Notification {
    #[serde(default)] pub id: String,
    #[serde(default)] pub title: String,
    #[serde(default)] pub body: String,
    #[serde(default)] pub kind: String,
    #[serde(default)] pub read: bool,
    #[serde(default)] pub created_at: String,
    #[serde(default)] pub action_url: Option<String>,
    #[serde(default)] pub action_label: Option<String>,
}

#[component]
fn NotificationRow(n: Notification) -> Element {
    let kind_icon = match n.kind.as_str() {
        "payment" => "credit-card",
        "subscription" => "zap",
        "wallet" => "wallet",
        "system" => "info",
        "alert" => "alert-triangle",
        _ => "bell",
    };
    rsx! {
        div {
            class: if n.read { "notification-row flex items-start gap-3 p-4 border-b border-border" } else { "notification-row flex items-start gap-3 p-4 border-b border-border bg-primary/5" },
            div { class: "w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center shrink-0",
                Icon { name: kind_icon.to_string(), size: Some(16) }
            }
            div { class: "flex-1",
                p { class: if n.read { "font-semibold" } else { "font-bold" }, "{n.title}" }
                p { class: "text-sm text-muted-foreground", "{n.body}" }
                div { class: "flex items-center gap-2 mt-1",
                    span { class: "text-xs text-muted-foreground", "{n.created_at}" }
                    if let (Some(lbl), Some(href)) = (&n.action_label, &n.action_url) {
                        span { "·" }
                        a { class: "text-xs text-primary underline", href: "{href}", "{lbl}" }
                    }
                }
            }
            div { class: "flex items-center gap-1",
                if !n.read {
                    button { class: "btn btn-sm btn-ghost", r#type: "button", title: "Mark read", Icon { name: "check".to_string(), size: Some(14) } }
                }
                button { class: "btn btn-sm btn-ghost", r#type: "button", title: "Delete", Icon { name: "trash".to_string(), size: Some(14) } }
            }
        }
    }
}

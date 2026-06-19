//! `ChatConversationList` — sidebar list of past conversations.
//!
//! Port of `apps-old/frontend/components/chat/chat-conversation-list.tsx`
//! (83 LoC). The TS source renders a vertical list of conversation
//! summaries (avatar + name + last message + timestamp). The
//! Dioxus port renders the same structure with a
//! `ConversationSummary` data prop.

use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct ConversationSummary {
    pub id: String,
    pub title: String,
    pub last_message: String,
    pub timestamp: String,
    pub unread: u32,
}

#[component]
pub fn ChatConversationList(
    #[props(default = Vec::new())] items: Vec<ConversationSummary>,
    /// Currently selected conversation id. When `Some`, the
    /// matching row gets a highlight class.
    #[props(default = None)] selected: Option<String>,
    /// Fired when the user clicks a row.
    #[props(default = None)] on_select: Option<EventHandler<String>>,
) -> Element {
    rsx! {
        ul { class: "chat-conversation-list space-y-1",
            for c in items.iter() {
                li {
                    class: if selected.as_deref() == Some(&c.id) {
                        "chat-conversation-list-item chat-conversation-list-item-selected flex items-center gap-3 p-3 rounded-lg cursor-pointer bg-orange-500/10"
                    } else {
                        "chat-conversation-list-item flex items-center gap-3 p-3 rounded-lg cursor-pointer hover:bg-slate-100 dark:hover:bg-slate-800"
                    },
                    onclick: {
                        let id = c.id.clone();
                        move |e| {
                            let _ = e;
                            if let Some(cb) = on_select.as_ref() {
                                cb.call(id.clone());
                            }
                        }
                    },
                    div { class: "chat-conversation-list-avatar w-10 h-10 rounded-full bg-orange-500/10 flex items-center justify-center",
                        Icon { name: "user".to_string(), size: Some(16), class_name: Some("text-orange-500".to_string()) }
                    }
                    div { class: "flex-1 min-w-0",
                        div { class: "chat-conversation-list-title-row flex items-center justify-between",
                            span { class: "chat-conversation-list-title text-sm font-medium text-foreground truncate", "{c.title}" }
                            span { class: "chat-conversation-list-time text-xs text-slate-500", "{c.timestamp}" }
                        }
                        p { class: "chat-conversation-list-last text-xs text-slate-500 truncate", "{c.last_message}" }
                    }
                    if c.unread > 0 {
                        span { class: "chat-conversation-list-unread inline-flex items-center justify-center w-5 h-5 rounded-full bg-orange-500 text-white text-xs font-bold",
                            "{c.unread}"
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

    #[test]
    fn conversation_summary_default() {
        let c = ConversationSummary::default();
        assert!(c.id.is_empty());
        assert_eq!(c.unread, 0);
    }

    #[test]
    fn chat_conversation_list_smoke() {
        
    }
}

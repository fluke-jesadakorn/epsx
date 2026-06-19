//! `ChatHeader` — top bar of the chat panel with title +
//! close/minimize buttons.
//!
//! Port of `apps-old/frontend/components/chat/chat-header.tsx`
//! (63 LoC). The TS source renders a header with conversation
//! title + close button. The Dioxus port renders the same
//! structure with a `ChatHeaderData` data prop.

use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct ChatHeaderData {
    pub title: String,
    pub subtitle: String,
    pub avatar_emoji: String,
    pub is_online: bool,
}

#[component]
pub fn ChatHeader(
    data: ChatHeaderData,
    /// Fired when the close button is clicked.
    #[props(default = None)] on_close: Option<EventHandler<MouseEvent>>,
) -> Element {
    rsx! {
        div { class: "chat-header flex items-center justify-between p-4 border-b border-slate-200 dark:border-slate-700",
            div { class: "chat-header-info flex items-center gap-3",
                div { class: "chat-header-avatar w-10 h-10 rounded-full bg-orange-500/10 flex items-center justify-center text-xl",
                    "{data.avatar_emoji}"
                }
                div {
                    div { class: "chat-header-title text-sm font-bold text-foreground", "{data.title}" }
                    div { class: "chat-header-subtitle flex items-center gap-1 text-xs text-slate-500",
                        if data.is_online {
                            span { class: "h-1.5 w-1.5 rounded-full bg-green-500" }
                        } else {
                            span { class: "h-1.5 w-1.5 rounded-full bg-slate-400" }
                        }
                        "{data.subtitle}"
                    }
                }
            }
            button {
                class: "chat-header-close p-2 text-slate-500 hover:text-slate-700 dark:hover:text-slate-300 rounded-lg",
                onclick: move |e| {
                    if let Some(cb) = on_close.as_ref() {
                        cb.call(e);
                    }
                },
                Icon { name: "x".to_string(), size: Some(20) }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_header_data_default() {
        let d = ChatHeaderData::default();
        assert!(d.title.is_empty());
        assert!(!d.is_online);
    }

    #[test]
    fn chat_header_smoke() {
        
    }
}

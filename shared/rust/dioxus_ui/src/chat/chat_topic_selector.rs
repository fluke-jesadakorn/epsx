//! `ChatTopicSelector` — horizontal scrollable list of chat
//! topic pills.
//!
//! Port of `apps-old/frontend/components/chat/chat-topic-selector.tsx`
//! (233 LoC). The TS source renders a horizontal list of topic
//! pills with the active one highlighted. The Dioxus port
//! renders the same structure with a `ChatTopic` data prop.

use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct ChatTopic {
    pub id: String,
    pub label: String,
    pub emoji: String,
}

#[component]
pub fn ChatTopicSelector(
    #[props(default = Vec::new())] topics: Vec<ChatTopic>,
    /// Currently selected topic id.
    #[props(default = None)] selected: Option<String>,
    /// Fired when the user clicks a topic pill.
    #[props(default = None)] on_select: Option<EventHandler<String>>,
) -> Element {
    rsx! {
        div { class: "chat-topic-selector flex items-center gap-2 overflow-x-auto py-2",
            for t in topics.iter() {
                button {
                    class: if selected.as_deref() == Some(&t.id) {
                        "chat-topic-selector-item chat-topic-selector-item-active inline-flex items-center gap-1 px-3 py-1.5 rounded-full text-sm font-medium bg-orange-500 text-white whitespace-nowrap"
                    } else {
                        "chat-topic-selector-item inline-flex items-center gap-1 px-3 py-1.5 rounded-full text-sm font-medium bg-slate-100 dark:bg-slate-800 text-slate-700 dark:text-slate-300 whitespace-nowrap hover:bg-slate-200 dark:hover:bg-slate-700"
                    },
                    onclick: {
                        let id = t.id.clone();
                        move |e| {
                            let _ = e;
                            if let Some(cb) = on_select.as_ref() {
                                cb.call(id.clone());
                            }
                        }
                    },
                    span { "{t.emoji}" }
                    "{t.label}"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_topic_default() {
        let t = ChatTopic::default();
        assert!(t.id.is_empty());
        assert!(t.emoji.is_empty());
    }

    #[test]
    fn chat_topic_selector_smoke() {
        
    }
}

//! `ChatMessageList` — vertical scrollable list of messages.
//!
//! Port of `apps-old/frontend/components/chat/chat-message-list.tsx`
//! (92 LoC). The TS source renders a scrollable list of message
//! bubbles with auto-scroll-to-bottom. The Dioxus port renders
//! the same structure with a `Vec<Message>` data prop.

use crate::chat::message_bubble::{Message, MessageBubble};

use dioxus::prelude::*;

#[component]
pub fn ChatMessageList(
    #[props(default = Vec::new())] messages: Vec<Message>,
) -> Element {
    rsx! {
        div { class: "chat-message-list p-4 space-y-3 overflow-y-auto max-h-[60vh]",
            for m in messages.iter() {
                MessageBubble { message: m.clone(), is_own_message: false }
            }
            if messages.is_empty() {
                div { class: "chat-message-list-empty text-center py-8 text-sm text-slate-500",
                    "No messages yet — say hi!"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_message_list_smoke() {
        
    }
}
